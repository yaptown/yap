//! Sentence → clip mapping: for every sentence a film's subtitle contains,
//! the span of audio in which it is spoken, verified twice.
//!
//! A sentence earns a clip only when two independent witnesses agree with
//! the subtitle:
//!
//! 1. **The transcript.** The sentence's tokens are aligned to the full-film
//!    word-timed transcript ([`cues::align_sentence`]); the run of words that
//!    spells it must do so near-verbatim, sit clear of neighbouring speech and
//!    of any audio event, and its word stamps give the clip its bounds. Cue
//!    times are display times, authored for reading; word stamps come from
//!    something that heard the speech, and a cue holding two sentences has no
//!    per-sentence timing at all. The cut's edges and its clear margins come
//!    from the film's earshot speech profile, not the stamps: ElevenLabs
//!    stretches a word's end stamp up to the next onset and fabricates
//!    stamps for words it inferred from context, so a boundary built on a
//!    stamp cuts off real speech or reports a gap that is not there.
//! 2. **The phoneme model.** The clip is cut and the model's per-frame
//!    distribution fetched ([`phoneme_verify::frame_matrix`]); the espeak
//!    rendering of the sentence is scored under CTC against the model's own
//!    free reading, and its first and last phonemes must be found in the
//!    audio under forced alignment ([`phoneme_verify::FrameMatrix::force_align`])
//!    — the direct test that the *whole* sentence is inside the cut. Given
//!    (1) the words are right, so what the ratio rejects beyond that is
//!    audio a listener can't actually make out — music beds, mumbling,
//!    heavy compression — which a transcription model is too robust to
//!    notice.
//!
//! Sentences are the ones course ingestion would produce from the same
//! subtitle (`movie_subtitles::segment`), keyed the same way, so a language
//! pack can look its sentences' clips up by text — independent of *which*
//! subtitle file a sentence came from.
//!
//! One `clips.jsonl` per film beside its transcript: a provenance line, then
//! one [`Clip`] per sentence that reached the phoneme gate, passing or not,
//! with every score kept so the gate can be re-tuned from the file alone.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use futures::StreamExt;
use language_utils::Language;
use movie_subtitles::segment::SubtitleSegmenter;
use movie_subtitles::sentences::KeyedSentence;
use movie_subtitles::{cleanup_subtitle_text, SubtitleLine};
use phoneme_verify::{wav2vec2::RequestActivity, FrameMatrix, VerifyContext};
use serde::{Deserialize, Serialize};

use crate::cues::{
    agreement_tokens, agrees, align_sentence, load_transcript, parse_cues, slice_wav_padded,
    tokenization_for, AUDIO_PAD_MS, MATCH_SLOP_MS, MAX_CUE_MS, MIN_CUE_MS, MIN_TOKENS,
};
use crate::library::{course_dir, read_plan, Movie};
use crate::transcript::{Kind, Spoken};

/// Bump when the record format or the gating logic changes in a way that
/// makes existing `clips.jsonl` files not comparable.
const FORMAT_VERSION: u32 = 12;

/// How late earshot flags speech after it begins. Measured 2026-09-02 on
/// four films: with the profile allowed to trim inside the stamped words
/// at threshold 0.7, the start moved a median 60 ms later than the stamp
/// and the end 100 ms earlier, and the clips lost their first consonants
/// (edge and ratio rejects up, net yield down 7%). So the profile only
/// ever moves a boundary *outward*, and when it does — a squeezed or late
/// stamp — the onset is placed this much before the frame that first read
/// as speech.
const ONSET_LAG_MS: i64 = 80;

/// Where no neighbouring word bounds the search, how far from the stamp
/// the silence may be looked for.
const OPEN_SEARCH_MS: i64 = 2_000;

/// How many phonemes at each end of the target the edge check averages.
const EDGE_PHONEMES: usize = 3;

/// Target lead-in before the first word, clear margin permitting: a small
/// quiet lets the listener settle into the scene before the sentence
/// starts. The tail keeps the shorter [`AUDIO_PAD_MS`].
const LEAD_IN_MS: i64 = 300;

/// Freshness groups are compared independently; producers are observations on
/// rows, not invalidation inputs. A producer bug requires deliberate refresh:
/// `clips --refresh-g2p` refreshes targets, not unchanged-label model outputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    pub inputs: Inputs,
    pub cut: Cut,
    pub gate: GateCuts,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Inputs {
    pub format: u32,
    pub subtitle_digest: String,
    pub transcript_digest: String,
    pub segmentation: String,
    pub language: String,
    pub audio: AudioInput,
}

/// Only this projection of the existing extracted-audio stamp identifies the
/// recording. Never substitute a video path or the derived audio.opus pathname.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioInput {
    pub filename: String,
    pub stream_index: usize,
    pub duration_ms: i64,
}

impl From<crate::sync::AudioStamp> for AudioInput {
    fn from(stamp: crate::sync::AudioStamp) -> Self {
        Self {
            filename: stamp.filename,
            stream_index: stamp.stream.stream_index,
            duration_ms: stamp.duration_ms,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Cut {
    pub preferred_clear_ms: i64,
    pub min_clear_ms: i64,
    pub speech_threshold: f64,
}

/// A threshold belongs here only when the measurement it acts on is persisted.
/// min_verbatim uses the matching transcript-check report, not fresh segmentation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GateCuts {
    pub min_ratio: Option<f64>,
    pub min_edge_logp: f64,
    pub max_pad_speech: f64,
    pub max_lead_rms: f64,
    pub min_voiced: f64,
    pub min_verbatim: f64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct Producers {
    pub model: Option<phoneme_verify::ModelIdentity>,
    pub g2p: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum Work {
    Nothing,
    Regate,
    Redo(&'static str),
}

impl Provenance {
    fn work(&self, current: &Self) -> Work {
        if self.inputs.audio != current.inputs.audio {
            return Work::Redo("audio changed");
        }
        if self.inputs.transcript_digest != current.inputs.transcript_digest {
            return Work::Redo("transcript changed");
        }
        if self.inputs != current.inputs {
            return Work::Redo("inputs changed");
        }
        if self.cut != current.cut {
            return Work::Redo("cut changed");
        }
        if self.gate != current.gate {
            return Work::Regate;
        }
        Work::Nothing
    }
}

/// One transcript word inside a clip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipWord {
    pub text: String,
    pub at_ms: i64,
    pub until_ms: i64,
}

/// A sentence and the audio span it was spoken in, with both witnesses'
/// verdicts. `passed` is the mapping's answer; everything else is why.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    #[serde(flatten)]
    pub producers: Producers,
    /// False for pre-gate failures; re-gating must preserve their verdicts.
    pub measured: bool,
    /// Original padded WAV bytes; absent for failures before cutting. Export
    /// uses this with target_ipa to read the response cache without re-cutting.
    pub audio_hash: Option<u64>,
    pub sentence: String,
    pub imdb_id: String,
    /// Clip bounds from the transcript's word stamps (unpadded).
    pub start_ms: i64,
    pub end_ms: i64,
    /// Padding the scored cut used on each side: up to [`LEAD_IN_MS`] before
    /// and [`AUDIO_PAD_MS`] after, but never more than half the silence to
    /// the neighbouring speech, so the cut carries no one else's onset or
    /// tail. Cut the same way to hear exactly what was scored.
    pub pad_before_ms: i64,
    pub pad_after_ms: i64,
    /// How far the profile moved each bound from the boundary word's stamp
    /// (positive = widened past the stamp, negative = trimmed inside it);
    /// `start_ms`/`end_ms` already include it.
    pub repaired_before_ms: i64,
    pub repaired_after_ms: i64,
    pub words: Vec<ClipWord>,
    /// Diarized speaker when every word in the span agrees on one.
    pub speaker: Option<String>,
    /// Token edit distance between sentence and span, over sentence tokens.
    pub transcript_wer: f64,
    pub audio_event_overlap: bool,
    /// Silence between the span and the nearest other speech, either side.
    pub clear_before_ms: i64,
    pub clear_after_ms: i64,
    /// espeak's rendering of the sentence — the CTC target.
    pub target_ipa: Vec<String>,
    /// Target tokens the model has no row for (the score is of the target
    /// without them).
    pub oov: Vec<String>,
    /// Log-odds per phoneme of the target against the model's free reading
    /// (0 = the target is what the model would have said).
    pub ratio: Option<f64>,
    pub logp_target_per_phoneme: Option<f64>,
    /// Forced-alignment mean log-prob of the first/last [`EDGE_PHONEMES`]
    /// target phonemes — very low when the clip is missing the sentence's
    /// start or end, which the whole-sentence ratio forgives on a long
    /// sentence.
    pub edge_logp_start: Option<f64>,
    pub edge_logp_end: Option<f64>,
    /// Fraction of the lead-in/tail pad the phoneme model hears as speech
    /// ([`phoneme_verify::FrameMatrix::speech_fraction`]) — voices the
    /// transcript never wrote down still show up here.
    pub lead_speech: Option<f64>,
    pub tail_speech: Option<f64>,
    /// RMS of the lead-in over RMS of the spoken span: how loud the clip
    /// opens relative to its own dialogue (< 1 = quieter).
    pub lead_rms: Option<f64>,
    /// Voiced fraction of the spoken span ([`voiced_fraction`]) — near zero
    /// for whispered delivery.
    pub voiced: Option<f64>,
    /// The model's free reading, for display.
    pub heard_ipa: Vec<String>,
    pub passed: bool,
    pub reject: Option<String>,
}

/// Per-film summary printed as films complete.
#[derive(Debug, Default, Clone, Copy)]
pub struct FilmSummary {
    pub sentences: usize,
    pub aligned: usize,
    pub scored: usize,
    pub passed: usize,
    /// Median phoneme ratio of the scored clips, when there is a phoneme
    /// gate. A film whose median sits far below the cut is not speaking
    /// the language the targets were rendered in — a Cantonese track under
    /// Mandarin subtitles, a dub — however well its subtitle placed.
    pub median_ratio: Option<f64>,
}

/// Below this median ratio a film is flagged as not sounding like its
/// course language. Mandarin films run −1 to −2; the Cantonese ones that
/// slipped through in 2026-09-03 sat at −3.3 to −3.8.
pub const FOREIGN_AUDIO_RATIO: f64 = -3.0;

fn median(mut values: Vec<f64>) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(|a, b| a.total_cmp(b));
    Some(values[values.len() / 2])
}

/// Gate settings.
#[derive(Debug, Clone)]
pub struct Gate {
    /// Lowest CTC log-odds ratio a clip may have and still pass; `None`
    /// takes the per-language default from [`default_min_ratio`].
    pub min_ratio: Option<f64>,
    /// Pause length the boundary repair looks for before falling back to the
    /// transcript stamp. Blind-ASR adjudication (2026-08-30) put margins of
    /// 100–200 ms at 90–94% clean — as clean as the pass pool — while margins
    /// under 100 ms measured meaningfully worse, so the chosen preference is
    /// recorded in provenance even though a tighter margin is not rejected.
    pub preferred_clear_ms: i64,
    /// Acceptance floor for the measured quiet adjacent to each boundary.
    /// Zero keeps tight-margin clips; their padding naturally shrinks to zero.
    pub min_clear_ms: i64,
    /// Lowest forced-alignment mean log-prob the first/last
    /// [`EDGE_PHONEMES`] may have — the test that the sentence's start and
    /// end are actually inside the cut.
    pub min_edge_logp: f64,
    /// Most of the lead-in/tail pad the phoneme model may hear as speech —
    /// the voice-activity gate, run on the frame matrix itself rather than
    /// an energy VAD (earshot's levels proved film-mix-dependent: gaps sat
    /// at 0.3–0.4 against 0.7 during speech, with per-film baselines apart
    /// by 2×).
    pub max_pad_speech: f64,
    /// Loudest the lead-in may be relative to the spoken span (RMS ratio).
    /// 1.0 only rejects clips that open *louder* than their own dialogue.
    pub max_lead_rms: f64,
    /// Least voiced the spoken span may be — the whisper gate. The phoneme
    /// model, loudness, and ASR all comprehend whispers fine; only the
    /// missing glottal periodicity gives them away.
    pub min_voiced: f64,
    /// Least share of a film's eligible sentences its subtitle must place
    /// in the transcript ([`crate::verbatim`]) before the film is mapped at
    /// all. A rewrite or another-variety translation is not a source of
    /// sentences for this audio, however well synced.
    pub min_verbatim: Option<f64>,
    /// earshot score above which a 16 ms frame of the film's speech profile
    /// counts as speech. The clear margins are measured from the profile,
    /// not the transcript's word stamps: ElevenLabs stretches a word's end
    /// stamp up to the next onset, so stamped gaps cluster at 30–60 ms
    /// whatever the audio holds. On 120 stamp-rejected clips both earshot
    /// and the phoneme model heard a ≥100 ms pause in ~75% (2026-09-02);
    /// 0.7 matched the phoneme model on known-silent controls (92% vs 90%)
    /// where 0.5 called a quarter of them speech.
    pub speech_threshold: f64,
}

impl Default for Gate {
    fn default() -> Self {
        Self {
            min_verbatim: None,
            speech_threshold: 0.7,
            min_ratio: None,
            preferred_clear_ms: 100,
            min_clear_ms: 0,
            min_edge_logp: -4.0,
            // An ear test (2026-08-30) found rejects at these thresholds
            // are often fine clips — but a bad clip in the deck costs far
            // more than a missed good one, so the strict settings stand.
            max_pad_speech: 0.25,
            max_lead_rms: 1.0,
            min_voiced: 0.25,
        }
    }
}

/// The CTC cut per language, from `phoneme-corpus-eval` on transcript-
/// verified vs transcript-rejected cues (model edcbbbf43a7f, 2026-08-29),
/// choosing the loosest cut that still keeps ≤ ~5–10% of rejected cues —
/// here the transcript has already thrown out the wrong-words cases, so
/// what the cut trades is yield against audio the model itself finds hard
/// to make out.
///
/// | lang | AUC  | cut  | verified kept | rejected kept |
/// |------|------|------|---------------|---------------|
/// | spa  | 0.95 | −1.0 | 65%           | 3%            |
/// | fra  | 0.93 | −1.0 | 62%           | 3%            |
/// | ita  | 0.92 | −1.0 | 56%           | 5%            |
/// | eng  | 0.82 | −1.0 | 50%           | 14%           |
/// | rus  | 0.93 | −1.0 | 58%           | 5%            |
/// | deu  | 0.89 | −1.5 | 72%           | 9%            |
/// | por  | 0.84 | −1.5 | 52%           | 8%            |
///
/// Those cuts were then loosened by 1.0 after a blind-ASR adjudication of a
/// stratified 180-clip sample (Whisper large-v3 + Gemini 3.1 Pro,
/// 2026-08-30): among *transcript-verified* clips the −1.5…−1 band was as
/// clean as the band that passed (~80–90% exact), while real defects —
/// mostly boundary words missing from the cut, now caught structurally by
/// the squeeze repair and the edge check — concentrated below −2. The
/// ratio's remaining job is audio nobody can make out (music beds,
/// mumbling, heavy compression), which is where it separates.
///
/// Russian first measured AUC 0.70 with verified cues at a median of −4.5:
/// the espeak parser was emitting palatalization `ʲ` as its own token
/// where the model's labels bind it to the consonant (`tʲ`), so every soft
/// consonant was a mismatch. With the parser fixed it behaves like French.
/// The g2p-backed languages were audited the same way once their label
/// chains were ported (2026-09-03, same model, `phoneme_verify::model_target`
/// targets; Hindi had measured 0.77 against espeak, the wrong reference):
///
/// | lang     | AUC  | cut  | verified kept | rejected kept |
/// |----------|------|------|---------------|---------------|
/// | hin      | 0.86 | −2.0 | 71%           | 13%           |
/// | jpn      | 0.87 | −1.5 | 68%           | 11%           |
/// | zho-hans | 0.81 | −2.0 | 61%           | 15%           |
/// | tha      | 0.88 | −2.5 | 70%           | 8%            |
///
/// These cuts keep rejected cues at or under the 14% English tolerates and
/// have had no ear-test adjudication. Thai's verified cues sit a full point
/// lower than the others (median −2.05): the model hears colloquial r as l
/// and never emits the glottal stops its labels carry, so its cut is looser
/// by design. Korean has no cut at all: see [`audio_only`]. Thai's backend
/// runs through `uv`, which must be on PATH or the preflight above fails
/// the film.
pub fn default_min_ratio(code: &str) -> Option<f64> {
    Some(match code {
        "spa" | "fra" | "ita" | "eng" | "rus" | "hin" | "zho-hans" => -2.0,
        "deu" | "por" | "tha" => -2.5,
        "jpn" => -1.5,
        _ => return None,
    })
}

/// Languages mapped without a phoneme gate: the clip passes on the audio
/// gates alone (a clean pause each side from the earshot profile, no
/// voices in the pads, a lead-in no louder than the dialogue, not
/// whispered). Korean's phoneme labels come from the g2p crate now, but the
/// deployed pronunciation model was never trained on Korean, so scoring
/// against it would be scoring against a model that never heard the
/// language. Andre's call (2026-09-03): ship Korean clips on the audio
/// gates and turn the phoneme gate on once a model trained on the g2p-kor
/// labels exists.
pub fn audio_only(code: &str) -> bool {
    code == "kor"
}

/// Whether `clips` maps this language at all — with a phoneme gate or
/// [`audio_only`].
pub fn maps(code: &str) -> bool {
    default_min_ratio(code).is_some() || audio_only(code)
}

/// The 16-bit mono samples of a RIFF wav as `slice_wav_padded` emits it.
fn wav_samples(wav: &[u8]) -> Option<Vec<i16>> {
    let mut at = 12; // past "RIFF<len>WAVE"
    while at + 8 <= wav.len() {
        let len = u32::from_le_bytes(wav.get(at + 4..at + 8)?.try_into().ok()?) as usize;
        if &wav[at..at + 4] == b"data" {
            let data = wav.get(at + 8..at + 8 + len)?;
            return Some(
                data.chunks_exact(2)
                    .map(|b| i16::from_le_bytes([b[0], b[1]]))
                    .collect(),
            );
        }
        at += 8 + len + len % 2;
    }
    None
}

/// Fraction of speech-active frames with a periodic (voiced) signal:
/// a normalized autocorrelation peak in the 60–400 Hz pitch range. Whispered
/// speech has no glottal vibration, so it scores near zero however loud it
/// is — the signal loudness measures can't see. Calibrated on the ear test
/// (2026-08-30): whispered verdicts sat at a median 0.47 against 0.71 for
/// good clips, with only whispers and the softest speech below ~0.25.
fn voiced_fraction(samples: &[i16], sample_rate: usize) -> Option<f64> {
    let win = sample_rate * 30 / 1000;
    let hop = sample_rate * 10 / 1000;
    let (lag_lo, lag_hi) = (sample_rate / 400, sample_rate / 60);
    if samples.len() < win * 2 {
        return None;
    }
    let frames: Vec<&[i16]> = (0..)
        .map(|i| i * hop)
        .take_while(|&i| i + win <= samples.len())
        .map(|i| &samples[i..i + win])
        .collect();
    let energies: Vec<f64> = frames.iter().map(|f| rms(f)).collect();
    let mut sorted = energies.clone();
    sorted.sort_by(f64::total_cmp);
    // Only frames carrying speech count; the floor is relative to the
    // clip's own loud frames so quiet mixes are not all "inactive".
    let floor = sorted[sorted.len() * 8 / 10] * 0.25;
    let (mut voiced, mut active) = (0usize, 0usize);
    for (frame, energy) in frames.iter().zip(&energies) {
        if *energy < floor || *energy == 0.0 {
            continue;
        }
        active += 1;
        let mean = frame.iter().map(|&s| f64::from(s)).sum::<f64>() / frame.len() as f64;
        let x: Vec<f64> = frame.iter().map(|&s| f64::from(s) - mean).collect();
        let ac0: f64 = x.iter().map(|v| v * v).sum();
        if ac0 <= 0.0 {
            continue;
        }
        let peak = (lag_lo..lag_hi.min(x.len()))
            .map(|lag| {
                x[..x.len() - lag]
                    .iter()
                    .zip(&x[lag..])
                    .map(|(a, b)| a * b)
                    .sum::<f64>()
            })
            .fold(f64::NEG_INFINITY, f64::max);
        if peak / ac0 > 0.45 {
            voiced += 1;
        }
    }
    (active > 0).then(|| voiced as f64 / active as f64)
}

fn rms(samples: &[i16]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let sq: f64 = samples.iter().map(|&s| f64::from(s) * f64::from(s)).sum();
    (sq / samples.len() as f64).sqrt()
}

pub fn clips_path(dir: &Path) -> PathBuf {
    dir.join("clips.jsonl")
}

/// Completeness is independent of provenance: a matching input header does not
/// prove that every candidate received a verdict. Only complete files are written.
#[derive(Serialize, Deserialize)]
struct Header {
    #[serde(flatten)]
    provenance: Provenance,
    expected_candidates: usize,
    completion: Completion,
}

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Completion {
    Complete,
    RefreshG2p,
}

#[cfg(test)]
fn stored_provenance(path: &Path) -> Option<Provenance> {
    read_file(path).ok().map(|(provenance, _)| provenance)
}

pub fn read_file(path: &Path) -> Result<(Provenance, Vec<Clip>)> {
    let (header, clips) = read_manifest(path)?;
    anyhow::ensure!(
        header.completion == Completion::Complete,
        "G2P refresh unfinished"
    );
    Ok((header.provenance, clips))
}

fn read_manifest(path: &Path) -> Result<(Header, Vec<Clip>)> {
    let text = std::fs::read_to_string(path)?;
    let mut lines = text.lines();
    let header: Header = serde_json::from_str(lines.next().context("missing clip header")?)?;
    anyhow::ensure!(
        header.provenance.inputs.format == FORMAT_VERSION,
        "unsupported clip format"
    );
    let clips: Vec<Clip> = lines
        .map(serde_json::from_str)
        .collect::<std::result::Result<_, _>>()
        .context("malformed clip row")?;
    anyhow::ensure!(
        clips.len() == header.expected_candidates,
        "incomplete clip file"
    );
    anyhow::ensure!(clips.iter().all(Clip::resolved), "unresolved clip verdict");
    Ok((header, clips))
}

fn interrupted_refresh(dir: &Path) -> bool {
    read_manifest(&clips_path(dir))
        .is_ok_and(|(header, _)| header.completion == Completion::RefreshG2p)
}

/// Preserve old rows for inspection, but invalidate their completion claim
/// before touching targets. A crash then forces regeneration on any next run.
fn begin_refresh(dir: &Path, provenance: &Provenance) -> Result<()> {
    let (mut header, clips) = read_manifest(&clips_path(dir)).unwrap_or_else(|_| {
        (
            Header {
                provenance: provenance.clone(),
                expected_candidates: 0,
                completion: Completion::RefreshG2p,
            },
            Vec::new(),
        )
    });
    header.completion = Completion::RefreshG2p;
    write_manifest(dir, &header, &clips)
}

/// Writer/reader contract across mapper and export, in different runs: the
/// original cut's WAV hash and exact G2P labels. Producer versions never key it.
pub fn clip_key(audio_hash: u64, phonemes: &[String]) -> String {
    let inputs = serde_json::to_vec(&(audio_hash, phonemes)).expect("clip key is serializable");
    format!(
        "phoneme-response/clip/{:016x}",
        xxhash_rust::xxh3::xxh3_64(&inputs)
    )
}

/// Every clip, with strict format, row count and verdict validation.
pub fn read_clips(path: &Path) -> Result<Vec<Clip>> {
    read_file(path).map(|(_, clips)| clips)
}

impl Clip {
    fn resolved(&self) -> bool {
        self.passed != self.reject.is_some()
    }
}

/// The sentences of a subtitle track, keyed exactly as course ingestion keys
/// them ([`movie_subtitles::sentences::keyed_sentences`] — one shared
/// implementation, so a pack sentence and its clip agree byte-for-byte).
/// All sentences are returned, course-worthy or not; the flag rides along.
pub async fn subtitle_sentences(
    srt: &str,
    language: Language,
    segmenter: &SubtitleSegmenter,
) -> Result<Vec<KeyedSentence>> {
    movie_subtitles::sentences::keyed_sentences(&subtitle_lines(srt), language, segmenter).await
}

/// Segment every model-segmented film in `films` in one Batch API round
/// trip, so that the per-film [`subtitle_sentences`] calls that follow it
/// are cache hits. A step that works through films one at a time would
/// otherwise run a film-sized batch for each — and a batch can take a day
/// to come back. Films in a rule-segmented language cost nothing here, and
/// so does a queue the `segment` step has already warmed.
pub async fn warm_segmentation(out: &Path, films: &[Movie]) -> Result<()> {
    use movie_subtitles::llm_segment;
    let tracks = llm_tracks(out, films)?;
    if tracks.is_empty() {
        return Ok(());
    }
    let cues: usize = tracks.iter().map(|(_, _, lines)| lines.len()).sum();
    println!(
        "segmenting {} model-segmented films ({cues} cues) in one batch",
        tracks.len()
    );
    let borrowed: Vec<(&[SubtitleLine], Language)> = tracks
        .iter()
        .map(|(_, language, lines)| (lines.as_slice(), *language))
        .collect();
    let client = llm_segment::client()?;
    let (_, report) =
        llm_segment::split_tracks(&client, &borrowed, llm_segment::print_progress()).await?;
    println!(
        "  {} cues put to the model, {} fell back to per-cue",
        report.asked, report.fallbacks
    );
    Ok(())
}

/// The subtitle of every film in `films` whose language the model segments,
/// as the cue lines segmentation starts from: `(index into films, language,
/// lines)`. Films without a subtitle are skipped.
pub fn llm_tracks(
    out: &Path,
    films: &[Movie],
) -> Result<Vec<(usize, Language, Vec<SubtitleLine>)>> {
    films
        .iter()
        .enumerate()
        .filter_map(|(i, m)| {
            let language = course_dir(&m.original_language).and_then(Language::from_code)?;
            if !movie_subtitles::llm_segment::uses_llm(language) {
                return None;
            }
            let path = out.join(&m.imdb_id).join("subtitle.srt");
            if !path.exists() {
                return None;
            }
            Some(
                std::fs::read_to_string(&path)
                    .map(|srt| {
                        let lines =
                            movie_subtitles::sentences::prepared_lines(&subtitle_lines(&srt));
                        (i, language, lines)
                    })
                    .with_context(|| format!("read {}", path.display())),
            )
        })
        .collect()
}

/// A subtitle text as the cleaned cue lines segmentation starts from.
pub fn subtitle_lines(srt: &str) -> Vec<SubtitleLine> {
    parse_cues(srt)
        .into_iter()
        .filter_map(|cue| {
            let text = cleanup_subtitle_text(&cue.text);
            (!text.is_empty()).then_some(SubtitleLine {
                sentence: text,
                start_ms: cue.start_ms.max(0) as u32,
                end_ms: cue.end_ms.max(0) as u32,
            })
        })
        .collect()
}

/// A sentence's place in the transcript, or why it has none.
pub struct Placed {
    pub words: Vec<ClipWord>,
    pub speaker: Option<String>,
    pub wer: f64,
    pub audio_event_overlap: bool,
    /// Where the neighbouring words begin and end: the silence search
    /// never reaches into them.
    pub prev_word_start_ms: Option<i64>,
    pub next_word_end_ms: Option<i64>,
}

pub fn place(
    sentence: &str,
    passage_start: i64,
    passage_end: i64,
    transcript: &[Spoken],
    code: &str,
) -> std::result::Result<Placed, &'static str> {
    let tokenization = tokenization_for(code);
    let tokens = agreement_tokens(sentence, tokenization);
    if tokens.len() < MIN_TOKENS {
        return Err("too short");
    }
    // Digits poison both witnesses: the transcript spells numbers out and
    // espeak expands them its own way.
    if sentence.chars().any(|c| c.is_ascii_digit()) {
        return Err("contains a digit");
    }
    let (lo, hi) = (passage_start - MATCH_SLOP_MS, passage_end + MATCH_SLOP_MS);
    let window: Vec<(usize, &Spoken)> = transcript
        .iter()
        .enumerate()
        .filter(|(_, w)| w.kind == Kind::Word && w.at_ms < hi && w.until_ms > lo)
        .collect();
    let heard: Vec<String> = window
        .iter()
        .flat_map(|(_, w)| agreement_tokens(&w.text, tokenization))
        .collect();
    // Tokens and words are not 1:1 under `Chars` tokenization (or when a
    // transcript word holds punctuation-split pieces), so align on tokens and
    // map back to words through each word's token count.
    let word_of_token: Vec<usize> = window
        .iter()
        .enumerate()
        .flat_map(|(wi, (_, w))| {
            std::iter::repeat_n(wi, agreement_tokens(&w.text, tokenization).len())
        })
        .collect();
    let m = align_sentence(&tokens, &heard).ok_or("nothing heard")?;
    let wer = m.distance as f64 / tokens.len() as f64;
    if !agrees(m.distance, tokens.len()) {
        return Err("transcript disagrees");
    }
    let (first_word, last_word) = (word_of_token[m.first], word_of_token[m.last]);
    let (first_idx, last_idx) = (window[first_word].0, window[last_word].0);
    let span = &transcript[first_idx..=last_idx];
    let start_ms = span[0].at_ms;
    let end_ms = span[span.len() - 1].until_ms;
    if !(MIN_CUE_MS..=MAX_CUE_MS).contains(&(end_ms - start_ms)) {
        return Err("span length out of bounds");
    }
    let audio_event_overlap = transcript
        .iter()
        .any(|w| w.kind == Kind::AudioEvent && w.at_ms < end_ms && w.until_ms > start_ms);
    // The nearest *spoken* neighbour on each side. For the character-
    // tokenized languages the transcript emits punctuation as its own
    // zero-length word sharing the previous word's stamps ("。" ending
    // exactly where the sentence ends), which as a neighbour would leave
    // the silence search no window at all — every Japanese sentence then
    // read as "too close" while the profile held a second of quiet.
    let spoken =
        |w: &&Spoken| w.kind == Kind::Word && !agreement_tokens(&w.text, tokenization).is_empty();
    let prev = transcript[..first_idx]
        .iter()
        .rev()
        .filter(spoken)
        .find(|w| w.at_ms < start_ms);
    let next = transcript[last_idx + 1..]
        .iter()
        .filter(spoken)
        .find(|w| w.until_ms > end_ms);
    let mut speakers: Vec<&str> = span.iter().filter_map(|w| w.speaker.as_deref()).collect();
    speakers.dedup();
    Ok(Placed {
        words: span
            .iter()
            .filter(|w| w.kind == Kind::Word)
            .map(|w| ClipWord {
                text: w.text.clone(),
                at_ms: w.at_ms,
                until_ms: w.until_ms,
            })
            .collect(),
        speaker: match speakers.as_slice() {
            [only] => Some((*only).to_string()),
            _ => None,
        },
        wer,
        audio_event_overlap,
        prev_word_start_ms: prev.map(|w| w.at_ms),
        next_word_end_ms: next.map(|w| w.until_ms),
    })
}

/// The phoneme gate's verdict on a scored clip: the target must be in the
/// model's vocabulary, score above the language's cut, and have both its
/// edges inside the cut.
fn phoneme_reject(clip: &Clip, min_ratio: f64, gate: &Gate) -> Option<String> {
    match clip.ratio {
        _ if !clip.oov.is_empty() => Some(format!(
            "target phonemes outside the model vocabulary: {}",
            clip.oov.join(" ")
        )),
        None => Some("target could not be scored".into()),
        Some(r) if r < min_ratio => Some(format!("ratio {r:.2} below {min_ratio:.2}")),
        _ if clip.edge_logp_start.is_none_or(|e| e < gate.min_edge_logp) => {
            Some(match clip.edge_logp_start {
                Some(e) => format!("sentence start not in the cut (edge logp {e:.2})"),
                None => "target could not be aligned".into(),
            })
        }
        _ if clip.edge_logp_end.is_some_and(|e| e < gate.min_edge_logp) => Some(format!(
            "sentence end not in the cut (edge logp {:.2})",
            clip.edge_logp_end.unwrap()
        )),
        _ => None,
    }
}

/// The verdict of the gates that listen to the audio itself, phoneme model
/// or not: voices in the pads, a lead-in louder than the dialogue,
/// whispered delivery.
fn audio_reject(clip: &Clip, gate: &Gate) -> Option<String> {
    if let Some(v) = clip.lead_speech.filter(|v| *v > gate.max_pad_speech) {
        return Some(format!("voices in the lead-in (speech {v:.2})"));
    }
    if let Some(v) = clip.tail_speech.filter(|v| *v > gate.max_pad_speech) {
        return Some(format!("voices in the tail (speech {v:.2})"));
    }
    if let Some(v) = clip.lead_rms.filter(|v| *v > gate.max_lead_rms) {
        return Some(format!("lead-in louder than the dialogue (rms x{v:.2})"));
    }
    if let Some(v) = clip.voiced.filter(|v| *v < gate.min_voiced) {
        return Some(format!("whispered delivery (voiced {v:.2})"));
    }
    None
}

/// Fraction of the 16 ms frames in `[from_ms, to_ms)` the speech profile
/// scores at or above `threshold`; `None` for an empty stretch.
fn profile_speech_fraction(
    profile: &[f32],
    threshold: f32,
    from_ms: i64,
    to_ms: i64,
) -> Option<f64> {
    let frame_ms = (crate::vad::FRAME * 1000 / crate::vad::SAMPLE_RATE) as i64;
    let lo = (from_ms.max(0) / frame_ms) as usize;
    let hi = ((to_ms.max(0) + frame_ms - 1) / frame_ms) as usize;
    let hi = hi.min(profile.len());
    let lo = lo.min(hi);
    (hi > lo).then(|| {
        profile[lo..hi].iter().filter(|&&s| s >= threshold).count() as f64 / (hi - lo) as f64
    })
}

/// Stretches of `[from_ms, to_ms)` at least `min_ms` long in which every
/// 16 ms frame of the speech profile scores below `threshold`.
fn silences(
    profile: &[f32],
    threshold: f32,
    from_ms: i64,
    to_ms: i64,
    min_ms: i64,
) -> Vec<(i64, i64)> {
    let frame_ms = (crate::vad::FRAME * 1000 / crate::vad::SAMPLE_RATE) as i64;
    let lo = (from_ms.max(0) / frame_ms) as usize;
    let hi = ((to_ms.max(0) + frame_ms - 1) / frame_ms) as usize;
    let hi = hi.min(profile.len());
    let mut runs = Vec::new();
    let mut run_start: Option<usize> = None;
    // One frame past the end, never quiet, closes a run touching `hi`.
    let quiet_at = profile[lo.min(hi)..hi]
        .iter()
        .map(|&s| s < threshold)
        .chain(std::iter::once(false));
    for (i, quiet) in (lo..).zip(quiet_at) {
        match (quiet, run_start) {
            (true, None) => run_start = Some(i),
            (false, Some(s)) => {
                let (a, b) = (s as i64 * frame_ms, i as i64 * frame_ms);
                if b - a >= min_ms {
                    runs.push((a, b));
                }
                run_start = None;
            }
            _ => {}
        }
    }
    runs
}

/// Where the profile says the span really begins and ends, and how much
/// silence lies on each side.
struct Margins {
    start_ms: i64,
    end_ms: i64,
    clear_before_ms: i64,
    clear_after_ms: i64,
}

/// Measure the quiet run that touches a stamp, or zero when the profile says
/// speech reaches the stamp. This is only the fallback: preferred pauses may
/// sit farther out because they can repair a squeezed transcript boundary.
fn adjacent_quiet_before(profile: &[f32], threshold: f32, from_ms: i64, stamp_ms: i64) -> i64 {
    silences(profile, threshold, from_ms, stamp_ms, 0)
        .last()
        .filter(|(_, end)| *end >= stamp_ms)
        .map_or(0, |(start, _)| (stamp_ms - start).max(0))
}

fn adjacent_quiet_after(profile: &[f32], threshold: f32, stamp_ms: i64, to_ms: i64) -> i64 {
    silences(profile, threshold, stamp_ms, to_ms, 0)
        .first()
        .filter(|(start, _)| *start <= stamp_ms)
        .map_or(0, |(_, end)| (end - stamp_ms).max(0))
}

/// Find the best boundary on each side of a placed span in the film's speech
/// profile. The search runs between the neighbouring word's stamp and the
/// span's own stamp, never inside the span; the nearest pause at least
/// `preferred_clear_ms` long wins. A boundary only moves outward: when such a
/// pause ends before the stamped onset, speech began earlier than the stamp
/// says (a squeezed or late stamp) and the start moves back to it, less
/// [`ONSET_LAG_MS`]. A stamp stretched over silence is left alone — dead air
/// inside the span is harmless, a clipped consonant is not. When no preferred
/// pause exists on one side, that side keeps its stamp and records the shorter
/// quiet run actually touching it.
fn earshot_margins(
    profile: &[f32],
    threshold: f32,
    p: &Placed,
    preferred_clear_ms: i64,
) -> Margins {
    let first = &p.words[0];
    let last = &p.words[p.words.len() - 1];
    let head_start = p.prev_word_start_ms.unwrap_or(first.at_ms - OPEN_SEARCH_MS);
    let head = silences(
        profile,
        threshold,
        head_start,
        first.at_ms,
        preferred_clear_ms,
    );
    let (start_ms, clear_before_ms) = if let Some(&(gap_start, gap_end)) = head.last() {
        // The window ends at the stamp, so a pause reaching it means the stamp
        // is where speech begins; a pause ending short of it means speech
        // began earlier, at (roughly) the frame that first read as speech.
        let start_ms = if gap_end < first.at_ms {
            (gap_end - ONSET_LAG_MS).max(gap_start)
        } else {
            first.at_ms
        };
        (start_ms, start_ms - gap_start)
    } else {
        (
            first.at_ms,
            adjacent_quiet_before(profile, threshold, head_start, first.at_ms),
        )
    };
    let tail_end = p.next_word_end_ms.unwrap_or(last.until_ms + OPEN_SEARCH_MS);
    let tail = silences(
        profile,
        threshold,
        last.until_ms,
        tail_end,
        preferred_clear_ms,
    );
    let (end_ms, clear_after_ms) = if let Some(&(end_ms, quiet_end)) = tail.first() {
        (end_ms, quiet_end - end_ms)
    } else {
        (
            last.until_ms,
            adjacent_quiet_after(profile, threshold, last.until_ms, tail_end),
        )
    };
    Margins {
        start_ms,
        end_ms,
        clear_before_ms,
        clear_after_ms,
    }
}

/// Only descriptors survive discovery: corpus-wide WAVs and matrices would
/// dwarf the clip metadata. Misses are re-cut with bounded batch preparation.
struct PendingClip {
    index: usize,
    hash: u64,
}

struct PreparedFilm {
    dir: PathBuf,
    language: Language,
    provenance: Provenance,
    summary: FilmSummary,
    // Stable slots until all descriptors have resolved; failures leave holes.
    clips: Vec<Option<Clip>>,
    pending: Vec<PendingClip>,
}

enum FilmWork {
    Current(FilmSummary),
    Prepared(Box<PreparedFilm>),
}

fn current_provenance(
    dir: &Path,
    language: Language,
    code: &str,
    gate: &Gate,
) -> Result<Provenance> {
    let audio = crate::sync::read_audio_stamp(dir)
        .context("audio stamp missing (audio.json)")?
        .into();
    Ok(Provenance {
        inputs: Inputs {
            format: FORMAT_VERSION,
            subtitle_digest: crate::transcript::source_digest(&dir.join("subtitle.srt"))?,
            transcript_digest: crate::transcript::source_digest(&dir.join("transcript.jsonl"))?,
            segmentation: movie_subtitles::segment::provenance(language),
            language: code.into(),
            audio,
        },
        cut: Cut {
            preferred_clear_ms: gate.preferred_clear_ms,
            min_clear_ms: gate.min_clear_ms,
            speech_threshold: gate.speech_threshold,
        },
        gate: GateCuts {
            min_ratio: if audio_only(code) {
                None
            } else {
                Some(
                    gate.min_ratio
                        .or_else(|| default_min_ratio(code))
                        .context("no calibrated phoneme gate")?,
                )
            },
            min_edge_logp: gate.min_edge_logp,
            max_pad_speech: gate.max_pad_speech,
            max_lead_rms: gate.max_lead_rms,
            min_voiced: gate.min_voiced,
            min_verbatim: gate
                .min_verbatim
                .unwrap_or_else(|| crate::verbatim::min_fraction(code)),
        },
    })
}

/// No model/G2P discovery, audio cuts or segmentation in freshness inspection.
fn existing_work(
    dir: &Path,
    current: &Provenance,
) -> (Work, Option<(Vec<Clip>, crate::verbatim::Report)>) {
    if crate::sync::read_audio_stamp(dir).is_none() {
        return (Work::Redo("audio stamp missing"), None);
    }
    let Ok((stored, clips)) = read_file(&clips_path(dir)) else {
        return (Work::Redo("missing or incomplete clips"), None);
    };
    let work = stored.work(current);
    if matches!(work, Work::Redo(_)) {
        return (work, None);
    }
    let Some(report) = crate::verbatim::matching(
        dir,
        &current.inputs.subtitle_digest,
        &current.inputs.transcript_digest,
        current.gate.min_verbatim,
    ) else {
        return (Work::Redo("verbatim measurement missing or stale"), None);
    };
    (work, Some((clips, report)))
}

/// Prepare only genuinely stale films. Current files and threshold-only changes
/// return before the G2P canary, model discovery, segmentation or audio work.
async fn prepare_film(
    http: &reqwest::Client,
    store: &osmo::Store,
    movie: &Movie,
    dir: &Path,
    gate: &Gate,
    concurrency: usize,
    refresh_g2p: bool,
) -> Result<FilmWork> {
    let code = course_dir(&movie.original_language).context("unmapped language")?;
    let language = Language::from_code(code).context("unmapped course code")?;
    let subtitle = dir.join("subtitle.srt");
    let transcript_path = dir.join("transcript.jsonl");
    let audio = dir.join("audio.opus");
    for (what, p) in [
        ("subtitle", &subtitle),
        ("transcript", &transcript_path),
        ("audio", &audio),
    ] {
        if !p.exists() {
            bail!("no {what}");
        }
    }
    let provenance = current_provenance(dir, language, code, gate)?;
    let min_ratio = provenance.gate.min_ratio;
    let interrupted = interrupted_refresh(dir);
    let refresh_g2p = min_ratio.is_some() && (refresh_g2p || interrupted);
    let (mut work, stored) = if refresh_g2p {
        if interrupted {
            println!("{}: Resuming an interrupted refresh", movie.title);
        }
        begin_refresh(dir, &provenance)?;
        (Work::Redo("G2P refresh"), None)
    } else {
        existing_work(dir, &provenance)
    };
    if let Some((mut clips, report)) = stored {
        let verbatim = report.measure.verdict == crate::verbatim::Verdict::Verbatim;
        for clip in &mut clips {
            let previous = (clip.passed, clip.reject.clone());
            regate(clip, min_ratio, gate, verbatim);
            if previous != (clip.passed, clip.reject.clone()) {
                work = Work::Regate;
            }
        }
        let summary = FilmSummary {
            sentences: clips.len(),
            aligned: clips.len(),
            ..Default::default()
        };
        if work == Work::Nothing {
            return Ok(FilmWork::Current(summarize(&clips, summary)));
        }
        return Ok(FilmWork::Current(write_clips(
            dir, provenance, &clips, summary,
        )?));
    }
    // No current file may survive changed inputs that fail film admissibility.
    let check = crate::verbatim::check(dir, language, code, provenance.gate.min_verbatim).await?;
    if check.measure.verdict != crate::verbatim::Verdict::Verbatim {
        let _ = std::fs::remove_file(clips_path(dir));
        bail!(
            "subtitle not verbatim: {}",
            crate::verbatim::describe(&check.measure)
        );
    }

    if min_ratio.is_some() {
        if language.g2p_lang().is_none() {
            bail!("{code}: the g2p crate does not produce this language's model labels");
        }
        let canary = match language {
            Language::Hindi => "नमस्ते",
            Language::ChineseSimplified => "你好",
            Language::Japanese => "こんにちは",
            Language::Thai => "สวัสดี",
            _ => "bon",
        };
        match phoneme_verify::model_target(canary, language) {
            Some(Ok(p)) if !p.phonemes.is_empty() => {}
            other => bail!("G2P preflight: g2p produced {other:?} for a canary word"),
        }
    }

    // The margins come from the profile; without one there is nothing to
    // cut against, and falling back to stamps would quietly change what a
    // "clear" margin means from film to film.
    let profile = crate::vad::read_profile(&dir.join("speech-profile-16ms.f32"))
        .context("no 16 ms speech profile (run speech-profiles)")?;
    let threshold = gate.speech_threshold as f32;

    let empty = std::collections::HashMap::new();
    let ctx = match min_ratio {
        Some(_) => Some(VerifyContext::new(http, store.clone(), &empty, language)?),
        None => None,
    };
    let segmenter = SubtitleSegmenter::for_language(language)?;
    let transcript = load_transcript(&transcript_path)?;
    let sentences =
        subtitle_sentences(&std::fs::read_to_string(&subtitle)?, language, &segmenter).await?;

    let mut summary = FilmSummary {
        sentences: sentences.len(),
        ..Default::default()
    };
    let placed: Vec<(String, Placed)> = sentences
        .iter()
        .filter_map(|k| {
            place(
                &k.sentence,
                k.start_ms.into(),
                k.end_ms.into(),
                &transcript,
                code,
            )
            .ok()
            .map(|p| (k.sentence.clone(), p))
        })
        .collect();
    summary.aligned = placed.len();

    let prepared: Vec<Option<(Clip, Option<u64>)>> = futures::stream::iter(placed)
        .map(|(sentence, p)| {
            let audio = audio.clone();
            let imdb_id = movie.imdb_id.clone();
            let profile = &profile;
            async move {
                let stamped = (p.words[0].at_ms, p.words[p.words.len() - 1].until_ms);
                let margins = earshot_margins(profile, threshold, &p, gate.preferred_clear_ms);
                let (start_ms, end_ms, clear_before_ms, clear_after_ms) = (
                    margins.start_ms,
                    margins.end_ms,
                    margins.clear_before_ms,
                    margins.clear_after_ms,
                );
                let (repaired_before_ms, repaired_after_ms) =
                    (stamped.0 - start_ms, end_ms - stamped.1);
                let pad = |target: i64, clear: i64| target.min(clear / 2).max(0);
                let pad_before_ms = pad(LEAD_IN_MS, clear_before_ms);
                let pad_after_ms = pad(AUDIO_PAD_MS, clear_after_ms);
                let mut clip = Clip {
                    producers: Producers::default(),
                    measured: false,
                    audio_hash: None,
                    sentence: sentence.clone(),
                    imdb_id,
                    start_ms,
                    end_ms,
                    pad_before_ms,
                    pad_after_ms,
                    repaired_before_ms,
                    repaired_after_ms,
                    words: p.words,
                    speaker: p.speaker,
                    transcript_wer: p.wer,
                    audio_event_overlap: p.audio_event_overlap,
                    clear_before_ms,
                    clear_after_ms,
                    target_ipa: Vec::new(),
                    oov: Vec::new(),
                    ratio: None,
                    logp_target_per_phoneme: None,
                    edge_logp_start: None,
                    edge_logp_end: None,
                    lead_speech: None,
                    tail_speech: None,
                    lead_rms: None,
                    voiced: None,
                    heard_ipa: Vec::new(),
                    passed: false,
                    reject: None,
                };
                if clip.audio_event_overlap {
                    clip.reject = Some("audio event inside the span".into());
                    return Some((clip, None));
                }
                if clear_before_ms < gate.min_clear_ms || clear_after_ms < gate.min_clear_ms {
                    clip.reject = Some("neighbouring speech too close to cut clean".into());
                    return Some((clip, None));
                }
                let wav = match tokio::task::spawn_blocking(move || {
                    slice_wav_padded(&audio, start_ms, end_ms, pad_before_ms, pad_after_ms)
                })
                .await
                .expect("slice task panicked")
                {
                    Ok(w) => w,
                    Err(e) => {
                        clip.reject = Some(format!("cut: {e:#}"));
                        return Some((clip, None));
                    }
                };
                let hash = xxhash_rust::xxh3::xxh3_64(&wav);
                clip.audio_hash = Some(hash);
                let padded_ms = (end_ms - start_ms + pad_before_ms + pad_after_ms) as f64;
                // The mix's loudness and voicing, straight from the samples.
                if let Some(samples) = wav_samples(&wav) {
                    let n = |ms: f64| (ms * samples.len() as f64 / padded_ms) as usize;
                    let (lead_n, span_to) =
                        (n(pad_before_ms as f64), n(padded_ms - pad_after_ms as f64));
                    let span_samples =
                        &samples[lead_n.min(samples.len())..span_to.min(samples.len())];
                    let span = rms(span_samples);
                    if lead_n > 0 && span > 0.0 {
                        clip.lead_rms = Some(rms(&samples[..lead_n.min(samples.len())]) / span);
                    }
                    clip.voiced = voiced_fraction(span_samples, 16_000);
                }
                if min_ratio.is_some() {
                    // Raw CTC scores only the single default-voice target, without
                    // accepted-variant readings (including Spanish seseo). A seseo
                    // Spanish clip therefore scores worse than on the edit-distance path.
                    let target = match phoneme_verify::cached_model_target(
                        store,
                        language,
                        &sentence,
                        hash,
                        refresh_g2p,
                    )
                    .await
                    {
                        Ok(target) if !target.phonemized.phonemes.is_empty() => target,
                        Ok(_) => {
                            clip.reject = Some("g2p produced no phonemes".into());
                            return Some((clip, None));
                        }
                        Err(e) => {
                            clip.reject = Some(format!("g2p: {e:#}"));
                            return Some((clip, None));
                        }
                    };
                    clip.target_ipa = target.phonemized.phonemes;
                    clip.producers.g2p = Some(target.renderer);
                    return Some((clip, Some(())));
                } else {
                    // No model to listen to the pads: the earshot profile
                    // says whether anyone speaks in them.
                    clip.lead_speech = profile_speech_fraction(
                        profile,
                        threshold,
                        start_ms - pad_before_ms,
                        start_ms,
                    );
                    clip.tail_speech =
                        profile_speech_fraction(profile, threshold, end_ms, end_ms + pad_after_ms);
                }
                clip.measured = true;
                regate(&mut clip, min_ratio, gate, true);
                Some((clip, None))
            }
        })
        .buffered(concurrency.max(1))
        .then(|prepared| {
            let ctx = &ctx;
            async move {
                let (mut clip, needs_model) = prepared?;
                let Some(()) = needs_model else {
                    return Some((clip, None));
                };
                let hash = clip.audio_hash.expect("cut recorded its hash");
                let key = clip_key(hash, &clip.target_ipa);
                let ctx = ctx.as_ref().expect("gated languages have a verify context");
                // Corrupt/mismatched entries are misses, not permanent holes.
                match phoneme_verify::cached_frame_matrix(ctx, &key).await {
                    Some(Ok(frames)) => {
                        score_clip(&mut clip, &frames, min_ratio.unwrap(), gate);
                        Some((clip, None))
                    }
                    _ if phoneme_verify::cache_only() => {
                        eprintln!(
                            "  {}: frame-matrix cache miss; cache-only mode is enabled",
                            clip.sentence
                        );
                        None
                    }
                    _ => Some((clip, Some(hash))),
                }
            }
        })
        .collect()
        .await;

    let mut clips = Vec::with_capacity(prepared.len());
    let mut pending = Vec::new();
    for item in prepared {
        let index = clips.len();
        clips.push(item.map(|(clip, hash)| {
            if let Some(hash) = hash {
                pending.push(PendingClip { index, hash });
            }
            clip
        }));
    }
    Ok(FilmWork::Prepared(Box::new(PreparedFilm {
        dir: dir.to_owned(),
        language,
        provenance,
        summary,
        clips,
        pending,
    })))
}

fn regate(clip: &mut Clip, min_ratio: Option<f64>, gate: &Gate, verbatim: bool) {
    if !clip.measured {
        return;
    }
    clip.reject = if !verbatim {
        Some("film-level verbatim gate rejected this subtitle".into())
    } else {
        min_ratio
            .and_then(|ratio| phoneme_reject(clip, ratio, gate))
            .or_else(|| audio_reject(clip, gate))
    };
    clip.passed = clip.reject.is_none();
}

fn score_clip(clip: &mut Clip, frames: &FrameMatrix, min_ratio: f64, gate: &Gate) {
    clip.producers.model = phoneme_verify::frame_identity(frames);
    let padded_ms = (clip.end_ms - clip.start_ms + clip.pad_before_ms + clip.pad_after_ms) as f64;
    // Frames spread evenly over the sliced audio; the pads are its first and
    // last stretches. Keep this identical for cached and newly inferred audio.
    let frame_ms = padded_ms / frames.frames as f64;
    let lead_frames = (clip.pad_before_ms as f64 / frame_ms) as usize;
    let tail_frames = (clip.pad_after_ms as f64 / frame_ms) as usize;
    clip.lead_speech = frames.speech_fraction(0, lead_frames);
    clip.tail_speech =
        frames.speech_fraction(frames.frames.saturating_sub(tail_frames), frames.frames);
    let score = frames.score_target(&clip.target_ipa);
    clip.heard_ipa = frames
        .greedy_ids()
        .into_iter()
        .map(|id| frames.vocab[id].clone())
        .collect();
    clip.oov = score.oov;
    clip.ratio = score.ratio;
    clip.logp_target_per_phoneme = score.logp_target_per_phoneme;
    let ids: Vec<usize> = clip
        .target_ipa
        .iter()
        .filter_map(|t| frames.id(t))
        .collect();
    if let Some(spans) = frames.force_align(&ids) {
        let k = EDGE_PHONEMES.min(spans.len());
        let mean = |spans: &[phoneme_verify::AlignedPhoneme]| {
            spans.iter().map(|s| s.logp_mean).sum::<f64>() / spans.len() as f64
        };
        clip.edge_logp_start = Some(mean(&spans[..k]));
        clip.edge_logp_end = Some(mean(&spans[spans.len() - k..]));
    }
    clip.measured = true;
    regate(clip, Some(min_ratio), gate, true);
}

/// Failed inference leaves the film unfinished. Successful response cache writes
/// survive, but no current header may hide failed or still-pending candidates.
fn finish_film(work: FilmWork) -> Result<FilmSummary> {
    let film = match work {
        FilmWork::Current(summary) => return Ok(summary),
        FilmWork::Prepared(film) => film,
    };
    let PreparedFilm {
        dir,
        provenance,
        summary,
        clips,
        ..
    } = *film;
    anyhow::ensure!(
        clips
            .iter()
            .all(|clip| clip.as_ref().is_some_and(Clip::resolved)),
        "clip candidates unresolved; successful inference is cached, retry this film"
    );
    let clips: Vec<Clip> = clips.into_iter().flatten().collect();
    write_clips(&dir, provenance, &clips, summary)
}

fn summarize(clips: &[Clip], mut summary: FilmSummary) -> FilmSummary {
    summary.scored = clips.len();
    summary.passed = clips.iter().filter(|c| c.passed).count();
    summary.median_ratio = median(clips.iter().filter_map(|c| c.ratio).collect());
    summary
}

fn write_clips(
    dir: &Path,
    provenance: Provenance,
    clips: &[Clip],
    summary: FilmSummary,
) -> Result<FilmSummary> {
    write_manifest(
        dir,
        &Header {
            provenance,
            expected_candidates: clips.len(),
            completion: Completion::Complete,
        },
        clips,
    )?;
    Ok(summarize(clips, summary))
}

fn write_manifest(dir: &Path, header: &Header, clips: &[Clip]) -> Result<()> {
    let mut text = serde_json::to_string(header)?;
    text.push('\n');
    for clip in clips {
        text.push_str(&serde_json::to_string(clip)?);
        text.push('\n');
    }
    let tmp = dir.join("clips.jsonl.tmp");
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, clips_path(dir))?;
    Ok(())
}

/// An owned cut lets bounded preparation run independently of mutable result
/// slots. It is metadata only; the WAV exists only while filling a batch.
struct AudioCut {
    key: String,
    audio: PathBuf,
    language: Language,
    hash: u64,
    start: i64,
    end: i64,
    before: i64,
    after: i64,
}

enum FrameInput<P> {
    Cached(Box<FrameMatrix>),
    Request(P),
}

fn apply_frames(film: &mut PreparedFilm, index: usize, frames: Result<FrameMatrix>, gate: &Gate) {
    let clip = film.clips[index].as_mut().expect("pending clip has a slot");
    match frames {
        Ok(frames) => score_clip(
            clip,
            &frames,
            film.provenance
                .gate
                .min_ratio
                .expect("pending clips have a phoneme gate"),
            gate,
        ),
        Err(error) => {
            eprintln!("  {}: {error:#}", clip.sentence);
            film.clips[index] = None;
        }
    }
}

/// Keep a second full request queued remotely while the first runs, hiding
/// round-trip/response gaps even with one server container. This is a bounded
/// starting point, not a measured optimum; report HTTP idle time before tuning.
const INFERENCE_REQUESTS_IN_FLIGHT: usize = 2;

#[derive(Default)]
struct InferenceProgress {
    requests: usize,
    submitted: usize,
    completed: usize,
    failed: usize,
}

impl InferenceProgress {
    fn fill_rate(&self) -> f64 {
        self.submitted as f64 / (self.requests * phoneme_verify::MODAL_BATCH_SIZE).max(1) as f64
    }

    fn clips_per_minute(&self, elapsed: std::time::Duration) -> f64 {
        if elapsed.is_zero() {
            0.0
        } else {
            self.completed as f64 * 60.0 / elapsed.as_secs_f64()
        }
    }

    fn report(&self, activity: &RequestActivity) {
        let snapshot = activity.snapshot();
        let seconds = snapshot.elapsed.as_secs_f64();
        let idle = if seconds == 0.0 {
            0.0
        } else {
            snapshot.without_request.as_secs_f64() / seconds
        };
        println!(
            "phoneme inference phase: {} clips completed ({} failed), {:.1} clips/min, \
             HTTP request fill {:.1}%, no HTTP outstanding {:.1}% of {:.1}s \
             ({} attempts, {} retries, peak {} outstanding)",
            self.completed,
            self.failed,
            self.clips_per_minute(snapshot.elapsed),
            self.fill_rate() * 100.0,
            idle * 100.0,
            seconds,
            snapshot.attempts,
            snapshot.retries,
            snapshot.peak_requests,
        );
    }
}

/// The actual mapper scheduler, with I/O injected so its barrier, compaction,
/// and routing can be exercised without film audio or a live endpoint.
async fn map_staged<P>(
    preparation: impl futures::Stream<Item = (usize, Result<FilmWork>)>,
    prepare_request: impl AsyncFn(&AudioCut) -> Result<FrameInput<P>>,
    infer: impl AsyncFn(Vec<P>, &RequestActivity) -> Vec<Result<FrameMatrix>>,
    gate: &Gate,
) -> Vec<(usize, Result<FilmWork>)> {
    // Deliberate barrier: finish discovery over the entire selection first.
    // Only metadata and cache-miss descriptors survive, never WAVs/matrices.
    let mut films: Vec<_> = preparation.collect().await;
    // The measured inference phase includes initial filling and tail handling,
    // but excludes the deliberate all-films discovery barrier above.
    let activity = RequestActivity::default();
    let mut pending = Vec::new();
    for (film_index, (_, outcome)) in films.iter().enumerate() {
        if let Ok(FilmWork::Prepared(film)) = outcome {
            for descriptor in &film.pending {
                let clip = film.clips[descriptor.index]
                    .as_ref()
                    .expect("pending clip has a slot");
                pending.push((
                    film_index,
                    descriptor.index,
                    AudioCut {
                        key: clip_key(descriptor.hash, &clip.target_ipa),
                        audio: film.dir.join("audio.opus"),
                        language: film.language,
                        hash: descriptor.hash,
                        start: clip.start_ms,
                        end: clip.end_ms,
                        before: clip.pad_before_ms,
                        after: clip.pad_after_ms,
                    },
                ));
            }
        }
    }
    println!(
        "discovery complete: {} uncached clip candidates across {} selected films",
        pending.len(),
        films.len()
    );
    // 64 is the chosen HTTP cap, not measured GPU capacity. The endpoint
    // groups only within a request (up to 8 clips, length ratio <= 1.25;
    // group-norm checkpoints use single-clip forwards).
    // Nearby durations help fill those forwards and reduce padding within
    // each group, where every clip is padded to its longest neighbor.
    // Different padding groups may cause tiny
    // numerical differences; result slots, targets and gates remain unchanged.
    pending.sort_by_key(|(_, _, cut)| cut.end + cut.after - (cut.start - cut.before).max(0));
    let mut prepared = Box::pin(
        futures::stream::iter(pending)
            .map(|(film, clip, cut)| {
                let prepare_request = &prepare_request;
                async move { (film, clip, prepare_request(&cut).await) }
            })
            .buffered(8),
    );
    let mut in_flight = futures::stream::FuturesUnordered::new();
    let mut requests = Vec::new();
    let mut slots = Vec::new();
    let mut preparation_done = false;
    let mut progress = InferenceProgress::default();
    let mut completed_batches = 0;
    loop {
        if !requests.is_empty()
            && (requests.len() == phoneme_verify::MODAL_BATCH_SIZE || preparation_done)
            && in_flight.len() < INFERENCE_REQUESTS_IN_FLIGHT
        {
            progress.requests += 1;
            progress.submitted += requests.len();
            let requests = std::mem::take(&mut requests);
            let slots = std::mem::take(&mut slots);
            let (infer, activity) = (&infer, &activity);
            in_flight.push(async move { (slots, infer(requests, activity).await) });
        }
        if preparation_done && requests.is_empty() && in_flight.is_empty() {
            break;
        }
        // At most two inference jobs, one ready/partial next batch, and eight
        // local preparations. No future borrows film slots, so completions and
        // cache/error outcomes can be applied immediately in either order.
        tokio::select! {
            prepared = prepared.next(), if !preparation_done && requests.len() < phoneme_verify::MODAL_BATCH_SIZE => {
                let Some((film_index, clip_index, prepared)) = prepared else {
                    preparation_done = true;
                    continue;
                };
                let Ok(FilmWork::Prepared(film)) = &mut films[film_index].1 else { unreachable!() };
                match prepared {
                    Ok(FrameInput::Request(request)) => {
                        requests.push(request);
                        slots.push((film_index, clip_index));
                    }
                    Ok(FrameInput::Cached(frames)) => apply_frames(film, clip_index, Ok(*frames), gate),
                    Err(error) => apply_frames(film, clip_index, Err(error), gate),
                }
            }
            result = in_flight.next(), if !in_flight.is_empty() => {
                let (slots, results) = result.expect("an inference job was in flight");
                assert_eq!(results.len(), slots.len(), "one result per prepared request");
                progress.completed += results.len();
                progress.failed += results.iter().filter(|result| result.is_err()).count();
                completed_batches += 1;
                for ((film_index, clip_index), frames) in slots.into_iter().zip(results) {
                    let Ok(FilmWork::Prepared(film)) = &mut films[film_index].1 else { unreachable!() };
                    apply_frames(film, clip_index, frames, gate);
                }
                if completed_batches % 25 == 0 {
                    progress.report(&activity);
                }
            }
        }
    }
    // Logical request fill is not GPU-forward fill. HTTP activity is measured
    // around individual attempts, excluding backoff, local cache writes/scoring.
    // Neither metric claims that an outstanding request was executing on GPU.
    progress.report(&activity);
    films
}

fn check_recut_hash(wav: &[u8], expected: u64) -> Result<()> {
    anyhow::ensure!(
        xxhash_rust::xxh3::xxh3_64(wav) == expected,
        "re-cut WAV changed since discovery; refusing mismatched cache identity"
    );
    Ok(())
}

async fn prepare_pending<'a>(
    http: &'a reqwest::Client,
    store: &osmo::Store,
    empty: &'a std::collections::HashMap<String, language_utils::Pronunciations>,
    cut: &AudioCut,
) -> Result<FrameInput<(VerifyContext<'a>, phoneme_verify::PreparedFrameRequest)>> {
    let key = cut.key.clone();
    let ctx = VerifyContext::new(http, store.clone(), empty, cut.language)?
        .with_cache_key(move |_| key.clone());
    // Another batch/process may have filled this key since discovery. A
    // duplicate WAV still has its own slot, target and language-specific gate.
    if let Some(Ok(frames)) = phoneme_verify::cached_frame_matrix(&ctx, &cut.key).await {
        return Ok(FrameInput::Cached(Box::new(frames)));
    }
    anyhow::ensure!(
        !phoneme_verify::cache_only(),
        "frame-matrix cache miss; cache-only mode is enabled"
    );
    let (start, end, before, after) = (cut.start, cut.end, cut.before, cut.after);
    let audio = cut.audio.clone();
    let wav =
        tokio::task::spawn_blocking(move || slice_wav_padded(&audio, start, end, before, after))
            .await
            .context("re-cut task failed")??;
    check_recut_hash(&wav, cut.hash)?;
    let request = phoneme_verify::prepare_frame_request(wav).await?;
    Ok(FrameInput::Request((ctx, request)))
}

#[derive(Default)]
struct ModelCounts {
    models: std::collections::BTreeMap<String, usize>,
    no_model: usize,
    unreadable_files: usize,
}

fn count_models(paths: impl IntoIterator<Item = PathBuf>) -> ModelCounts {
    let mut counts = ModelCounts::default();
    for path in paths {
        match read_clips(&path) {
            Ok(clips) => {
                for clip in clips {
                    match clip.producers.model {
                        Some(model) => {
                            *counts
                                .models
                                .entry(format!(
                                    "{}@{} decoder={}",
                                    model.model_id,
                                    model.model_revision,
                                    model.decoder_version.as_deref().unwrap_or("unknown")
                                ))
                                .or_default() += 1
                        }
                        None => counts.no_model += 1,
                    }
                }
            }
            Err(_) => counts.unreadable_files += 1,
        }
    }
    counts
}

/// Inspect actual recorded producers locally; old formats are counted, not read
/// through a compatibility layer or silently reported as zero successful rows.
pub fn clip_models(out: &Path) -> Result<()> {
    let paths = std::fs::read_dir(out)?
        .map(|entry| entry.map(|entry| clips_path(&entry.path())))
        .collect::<std::io::Result<Vec<_>>>()?;
    let counts = count_models(paths.into_iter().filter(|path| path.exists()));
    for (model, count) in counts.models {
        println!("{count}\t{model}");
    }
    println!("{}\tno-model rows", counts.no_model);
    println!(
        "{}\tunreadable/old-format/incomplete files",
        counts.unreadable_files
    );
    Ok(())
}

/// Map every transcribed film (or the ones selected), skipping films whose
/// `clips.jsonl` is already current.
pub async fn clips_all(
    out: PathBuf,
    films_in_flight: usize,
    limit: usize,
    imdb: Option<String>,
    langs: Option<Vec<String>>,
    gate: Gate,
    refresh_g2p: bool,
) -> Result<()> {
    let plan = read_plan(&out)?;
    let mut queue: Vec<Movie> = plan
        .into_iter()
        .filter(|m| imdb.as_deref().is_none_or(|id| m.imdb_id == id))
        .filter(|m| {
            langs.as_ref().is_none_or(|l| {
                course_dir(&m.original_language).is_some_and(|c| l.iter().any(|x| x == c))
            })
        })
        .filter(|m| {
            let dir = out.join(&m.imdb_id);
            dir.join("subtitle.srt").exists()
                && dir.join("transcript.jsonl").exists()
                && dir.join("audio.opus").exists()
        })
        .collect();
    if limit > 0 {
        queue.truncate(limit);
    }
    let total = queue.len();
    println!("{total} transcribed films to map");
    if total == 0 {
        return Ok(());
    }
    let redo: Vec<Movie> = queue
        .iter()
        .filter(|movie| {
            let Some(code) = course_dir(&movie.original_language) else {
                return false;
            };
            let Some(language) = Language::from_code(code) else {
                return false;
            };
            let dir = out.join(&movie.imdb_id);
            current_provenance(&dir, language, code, &gate).is_ok_and(|p| {
                (refresh_g2p && p.gate.min_ratio.is_some())
                    || interrupted_refresh(&dir)
                    || matches!(existing_work(&dir, &p).0, Work::Redo(_))
            })
        })
        .cloned()
        .collect();
    warm_segmentation(&out, &redo).await?;

    let store = osmo::Store::open("./.cache");
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;
    let empty = std::collections::HashMap::new();
    let mut discovered = 0;
    let preparation = futures::stream::iter(queue.iter().enumerate())
        .map(|(index, movie)| {
            let (http, store, out, gate) = (&http, &store, &out, &gate);
            async move {
                let dir = out.join(&movie.imdb_id);
                (
                    index,
                    prepare_film(http, store, movie, &dir, gate, 8, refresh_g2p).await,
                )
            }
        })
        .buffer_unordered(films_in_flight.max(1))
        .inspect(|(index, outcome)| {
            discovered += 1;
            if let Err(error) = outcome {
                eprintln!("discovery {}: {error:#}", queue[*index].title);
            }
            if discovered % 25 == 0 || discovered == total {
                println!("discovery: {discovered}/{total} films prepared");
            }
        });
    let films = map_staged(
        preparation,
        async |cut| prepare_pending(&http, &store, &empty, cut).await,
        async |requests, activity| {
            let (contexts, requests): (Vec<_>, Vec<_>) = requests.into_iter().unzip();
            phoneme_verify::infer_frame_batch(
                contexts.iter().zip(requests).collect(),
                Some(activity),
            )
            .await
        },
        &gate,
    )
    .await;

    let mut done = Vec::new();
    for (n, (index, outcome)) in films.into_iter().enumerate() {
        let n = n + 1;
        let movie = &queue[index];
        let title = crate::library::truncate(&movie.title, 34);
        // Preparation and finalization failures belong to this film only.
        match outcome.and_then(finish_film) {
            Ok(s) => {
                println!(
                    "[{n}/{total}] {title} ✓ {} sentences → {} placed → {} scored → {} pass",
                    s.sentences, s.aligned, s.scored, s.passed
                );
                if let Some(m) = s.median_ratio.filter(|m| *m < FOREIGN_AUDIO_RATIO) {
                    println!(
                        "    ⚠ median phoneme ratio {m:.2}: the audio does not sound like \
                         {} — another language or variety on this track?",
                        movie.original_language
                    );
                }
                done.push(s);
            }
            Err(e) => println!("[{n}/{total}] {title} ✗ {e:#}"),
        }
    }
    println!(
        "\n{} films mapped: {} sentences, {} placed by the transcript, {} passed both gates",
        done.len(),
        done.iter().map(|s| s.sentences).sum::<usize>(),
        done.iter().map(|s| s.aligned).sum::<usize>(),
        done.iter().map(|s| s.passed).sum::<usize>()
    );
    Ok(())
}

#[cfg(test)]
mod batching_tests;

#[cfg(test)]
mod margin_tests {
    use super::*;

    fn placed() -> Placed {
        Placed {
            words: vec![
                ClipWord {
                    text: "first".into(),
                    at_ms: 320,
                    until_ms: 480,
                },
                ClipWord {
                    text: "last".into(),
                    at_ms: 480,
                    until_ms: 640,
                },
            ],
            speaker: None,
            wer: 0.0,
            audio_event_overlap: false,
            prev_word_start_ms: Some(0),
            next_word_end_ms: Some(1_024),
        }
    }

    #[test]
    fn preferred_pause_wins_over_shorter_adjacent_pause() {
        let mut profile = vec![1.0; 64];
        profile[4..14].fill(0.0); // 160 ms farther out.
        profile[16..20].fill(0.0); // 64 ms adjacent to the onset stamp.
        profile[40..50].fill(0.0); // A preferred tail pause.

        let margins = earshot_margins(&profile, 0.7, &placed(), 100);

        // This is the pre-fallback boundary calculation byte for byte: the
        // 160 ms pause ends at 224, then the 80 ms onset lag lands at 144.
        assert_eq!(margins.start_ms, 144);
        assert_eq!(margins.clear_before_ms, 80);
        assert_eq!(margins.end_ms, 640);
        assert_eq!(margins.clear_after_ms, 160);
    }

    #[test]
    fn short_adjacent_pause_falls_back_to_stamp_and_is_measured() {
        let mut profile = vec![1.0; 64];
        profile[16..20].fill(0.0); // 64 ms before the onset stamp.
        profile[40..44].fill(0.0); // 64 ms after the end stamp.

        let margins = earshot_margins(&profile, 0.7, &placed(), 100);

        assert_eq!(margins.start_ms, 320);
        assert_eq!(margins.end_ms, 640);
        assert_eq!(margins.clear_before_ms, 64);
        assert_eq!(margins.clear_after_ms, 64);
    }
}
