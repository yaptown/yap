//! Transcript matching and audio cutting shared by clip generation and export.

use anyhow::{bail, Context, Result};

pub use crate::sync::{parse_cues, Cue};
pub use crate::transcript::Spoken;
use std::path::Path;
use std::process::Command;

pub use movie_subtitles::sentences::repair_latin_homoglyphs;

/// How subtitle/transcript text is split into comparable agreement units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tokenization {
    /// Lowercased alphanumeric runs — for languages written with spaces.
    Words,
    /// Individual alphanumeric characters — for spaceless scripts
    /// (jpn/zho-hans/tha), where "word" tokens would be whole clauses and
    /// every comparison saturates. ElevenLabs transcript units for these
    /// languages are already near-character-sized, so both witnesses land
    /// in the same unit space. Korean is spaced but its spacing is
    /// orthographically unstable (particle/compound spacing varies between
    /// subtitles and ASR), so syllable-block chars are the comparable unit
    /// there too.
    Chars,
}

/// The tokenization a pronunciation-corpus lang code needs.
pub fn tokenization_for(code: &str) -> Tokenization {
    match code {
        "jpn" | "zho-hans" | "tha" | "kor" => Tokenization::Chars,
        _ => Tokenization::Words,
    }
}

/// Comparable agreement tokens under the given tokenization.
pub fn agreement_tokens(text: &str, tokenization: Tokenization) -> Vec<String> {
    match tokenization {
        Tokenization::Words => text
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| !t.is_empty())
            .map(str::to_owned)
            .collect(),
        Tokenization::Chars => text
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .map(String::from)
            .collect(),
    }
}

/// How far either side of a cue's span transcript words still count as "this
/// cue" — absorbs residual sync error and word-boundary rounding.
pub const MATCH_SLOP_MS: i64 = 500;
/// Padding added to each side of the audio slice sent to the model.
pub const AUDIO_PAD_MS: i64 = 150;
/// Cue length bounds — outside these, clips are degenerate or unmanageable.
pub const MIN_CUE_MS: i64 = 400;
pub const MAX_CUE_MS: i64 = 12_000;
/// WER at or below which the transcript confirms the cue.
pub const POS_WER: f64 = 0.12;

/// Whether an edit distance is close enough for the transcript to confirm a
/// sentence.
///
/// Applying [`POS_WER`] as a floating-point ratio creates an integer cliff:
/// every sentence shorter than nine tokens otherwise permits no edits. In the
/// 2026-09-13 corpus, 14,999 of 49,374 placement disagreements were exactly
/// one edit, all on sentences of at most eight tokens (for example, "Ah,
/// c'est du joli." against "Ah, c'est joli."). Four-token sentences are long
/// enough to grant that single-edit allowance; shorter ones remain exact.
pub fn agrees(distance: usize, tokens: usize) -> bool {
    let allowance = usize::from(tokens >= 4);
    distance <= allowance.max((POS_WER * tokens as f64).floor() as usize)
}

/// Fewest agreement tokens required for a clip.
pub const MIN_TOKENS: usize = 2;

/// Cut `[start-pad, end+pad]` out of the film's opus audio as WAV bytes.
/// WAV (not opus) because the prediction cache is keyed on these bytes and
/// WAV output is byte-reproducible; opus is not.
///
/// ffmpeg writes to a temp FILE, not stdout: piped WAV output leaves the
/// RIFF/data chunk sizes unfinalized (ffmpeg cannot seek a pipe), which
/// strict readers (Rust `hound`, e.g. lexide's vad_compute) reject as
/// "data chunk length is not a multiple of sample size". A seekable file
/// gets its header patched on close and is equally byte-reproducible.
pub fn slice_wav_padded(
    audio: &Path,
    start_ms: i64,
    end_ms: i64,
    pad_before_ms: i64,
    pad_after_ms: i64,
) -> Result<Vec<u8>> {
    let start = (start_ms - pad_before_ms).max(0);
    let dur = end_ms + pad_after_ms - start;
    let tmp = tempfile::Builder::new()
        .suffix(".wav")
        .tempfile()
        .context("creating temp wav")?;
    let status = Command::new("ffmpeg")
        .args(["-v", "error", "-y", "-ss"])
        .arg(format!("{:.3}", start as f64 / 1000.0))
        .args(["-t", &format!("{:.3}", dur as f64 / 1000.0)])
        .arg("-i")
        .arg(audio)
        .args(["-ac", "1", "-ar", "16000", "-c:a", "pcm_s16le", "-f", "wav"])
        .arg(tmp.path())
        .status()
        .context("ffmpeg failed to start")?;
    let bytes = std::fs::read(tmp.path()).context("reading temp wav")?;
    if !status.success() || bytes.is_empty() {
        bail!(
            "ffmpeg could not cut {start_ms}..{end_ms} from {}",
            audio.display()
        );
    }
    Ok(bytes)
}

pub fn load_transcript(path: &Path) -> Result<Vec<Spoken>> {
    let text = std::fs::read_to_string(path)?;
    Ok(text
        .lines()
        // The provenance first line isn't a Spoken and fails to parse — skipped.
        .filter_map(|l| serde_json::from_str::<Spoken>(l).ok())
        .collect())
}

/// Where a sentence sits in the transcript: the run of transcript words that
/// best matches it, and how well.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentenceMatch {
    /// Indices into the window the sentence was aligned against, inclusive.
    pub first: usize,
    pub last: usize,
    /// Token edit distance between the sentence and that run.
    pub distance: usize,
}

/// Align a sentence's tokens to a window of transcript words and return the
/// contiguous run of words that spells it best.
///
/// Uses free-start / free-end edit alignment and recovers which words
/// matched — and from them, when the sentence was actually
/// spoken. Sentence timing from the transcript's word stamps is what makes a
/// clip trustworthy: a cue's own times are display times, authored for
/// reading, and a cue holding two sentences has no per-sentence timing at
/// all. `None` when either side is empty.
pub fn align_sentence(sentence: &[String], heard: &[String]) -> Option<SentenceMatch> {
    let (m, n) = (sentence.len(), heard.len());
    if m == 0 || n == 0 {
        return None;
    }
    // cost[i][j]: best cost matching sentence[..i] to a run of heard ending
    // at j; start[i][j]: where that run began. Row 0 is free.
    let mut cost = vec![vec![0usize; n + 1]; m + 1];
    let mut start = vec![vec![0usize; n + 1]; m + 1];
    for (j, slot) in start[0].iter_mut().enumerate() {
        *slot = j;
    }
    for i in 1..=m {
        cost[i][0] = i;
        start[i][0] = 0;
        for j in 1..=n {
            let sub = cost[i - 1][j - 1] + usize::from(sentence[i - 1] != heard[j - 1]);
            let del = cost[i - 1][j] + 1; // sentence token unheard
            let ins = cost[i][j - 1] + 1; // extra heard token inside the run
            let (c, s) = if sub <= del && sub <= ins {
                (sub, start[i - 1][j - 1])
            } else if del <= ins {
                (del, start[i - 1][j])
            } else {
                (ins, start[i][j - 1])
            };
            cost[i][j] = c;
            start[i][j] = s;
        }
    }
    // Free end: the best-scoring end position. Ties go to the run whose
    // length is closest to the sentence's (a cheap partial match that stops
    // early costs the same as full coverage with two substitutions), then to
    // the earliest, so a sentence repeated later in the window doesn't steal
    // the match.
    let (last, distance) = (1..=n)
        .map(|j| (j, cost[m][j]))
        .min_by_key(|&(j, c)| (c, (j - start[m][j]).abs_diff(m), j))?;
    let first = start[m][last];
    if first >= last {
        return None;
    }
    Some(SentenceMatch {
        first,
        last: last - 1,
        distance,
    })
}

#[cfg(test)]
mod align_tests {
    use super::*;

    fn toks(s: &str) -> Vec<String> {
        agreement_tokens(s, Tokenization::Words)
    }

    #[test]
    fn finds_the_run_inside_a_longer_window() {
        let heard = toks("oui bien sûr je vous supplie de ne pas chercher à nous retrouver merci");
        let m = align_sentence(
            &toks("Je vous supplie de ne pas chercher à nous retrouver."),
            &heard,
        )
        .unwrap();
        assert_eq!((m.first, m.last, m.distance), (3, 12, 0));
    }

    #[test]
    fn tolerates_a_misheard_word_and_reports_it() {
        let heard = toks("plus haut jusqu'aux genoux oh");
        let m = align_sentence(&toks("Plus haut, jusqu'au genou !"), &heard).unwrap();
        assert_eq!((m.first, m.last), (0, 4));
        assert_eq!(m.distance, 2);
    }

    #[test]
    fn prefers_the_first_of_two_identical_runs() {
        let heard = toks("bois bois bois attends bois bois");
        let m = align_sentence(&toks("Bois ! Bois !"), &heard).unwrap();
        assert_eq!((m.first, m.last, m.distance), (0, 1, 0));
    }

    #[test]
    fn agreement_allows_one_short_sentence_edit_without_loosening_tiny_sentences() {
        assert!(!agrees(1, 3));
        assert!(agrees(1, 4));
        assert!(!agrees(2, 8));
        assert!(agrees(1, 9));
        assert!(agrees(2, 17));
    }
}
