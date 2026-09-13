//! Reading a disc's bitmap subtitle track back into text.
//!
//! Billing is per token, not per request, and a cue image's tokens scale with
//! its pixels — so the only honest way to size the job is to measure a random
//! sample and extrapolate. That is what [`sample`] exists for; run it before
//! committing to the whole library.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use tysm::chat_completions::{ChatClient, ChatMessage, ChatMessageContent, ImageUrl, Role};

use crate::pgs;

// The model name is part of tysm's cache key, so re-asking a stronger model is
// automatically a fresh request rather than the same cached bad answer.
pub const FALLBACK_MODEL: &str = "gpt-5.6-terra";

const SYSTEM_PROMPT: &str = "\
You transcribe single lines of subtitle text from movie subtitle images.
Reproduce exactly what is written: same words, same punctuation, same accents,
same capitalisation, same line breaks (as \\n). Do not translate, correct,
paraphrase, or add commentary. Subtitles often begin with a dash for a new
speaker; keep it. If the image contains no readable text, return an empty
string.";

/// What the model gives back for one cue image.
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Transcription {
    /// The text exactly as written in the image, or "" if unreadable.
    pub text: String,
    /// True when the image is not subtitle text at all (a logo, a rating bar).
    pub not_text: bool,
}

pub fn client(model: &str) -> Result<ChatClient> {
    // Responses land in the shared cache store; tysm's cache key hashes the
    // whole request (model included), so re-running a movie after a prompt
    // tweak (or a crash) costs nothing for the cues already read.
    Ok(ChatClient::from_env(model)
        .context("OPENAI_API_KEY not set")?
        .with_cache_directory("./.cache"))
}

/// Whether decoded OCR text is safe to pass into subtitle output.
///
/// Strict JSON-schema output can mangle an accented letter's Unicode escape:
/// for example, `\\u00e9` may arrive as `\\u0000e9` or `\\u000e9`. JSON
/// decoding then leaves a C0 control character followed by stray hex digits,
/// so all controls except intentional subtitle whitespace must be rejected.
pub fn readable(text: &str) -> bool {
    text.chars()
        .all(|c| !c.is_control() || matches!(c, '\n' | '\r' | '\t'))
}

/// Retry a decoded-but-corrupt transcription with the stronger live model.
///
/// The boolean reports whether a retry was needed; an error after a retry is
/// deliberately treated like an unreadable batch item by callers.
pub async fn retry_unreadable(
    fallback_client: &ChatClient,
    png: &[u8],
    transcription: Transcription,
) -> (Result<Transcription>, bool) {
    if readable(&transcription.text) {
        return (Ok(transcription), false);
    }

    let result = transcribe(fallback_client, png)
        .await
        .context("fallback OCR request failed")
        .and_then(|fallback| {
            if readable(&fallback.text) {
                Ok(fallback)
            } else {
                bail!("fallback OCR response still contains control characters")
            }
        });
    (result, true)
}

/// What a film's batch of cue images boiled down to.
pub struct ReadLines {
    /// `(start_ms, end_ms, text)` for every cue that read as dialogue.
    pub lines: Vec<(u32, u32, String)>,
    /// Cues with no usable answer, after the fallback had its turn.
    pub unreadable: usize,
    /// Cues re-asked of the fallback model, and how many of those it read.
    pub retried: usize,
    pub rescued: usize,
}

/// Turn a batch's per-cue results into subtitle lines, sending every corrupt
/// answer through [`retry_unreadable`] first. A batch error and a fallback
/// that also fails count the same: an unreadable cue the caller may refuse
/// to write the film without.
pub async fn read_lines<E>(
    fallback_client: &ChatClient,
    images: &[CueImage],
    results: Vec<std::result::Result<Transcription, E>>,
) -> ReadLines {
    let mut read = ReadLines {
        lines: Vec::new(),
        unreadable: 0,
        retried: 0,
        rescued: 0,
    };
    for (img, result) in std::iter::zip(images, results) {
        let Ok(transcription) = result else {
            read.unreadable += 1;
            continue;
        };
        let (result, did_retry) = retry_unreadable(fallback_client, &img.png, transcription).await;
        read.retried += usize::from(did_retry);
        let Ok(transcription) = result else {
            read.unreadable += 1;
            continue;
        };
        read.rescued += usize::from(did_retry);
        let text = transcription.text.trim().to_string();
        if !transcription.not_text && !text.is_empty() {
            read.lines.push((img.start_ms, img.end_ms, text));
        }
    }
    read
}

/// The request for one cue image.
pub fn messages_for(png: &[u8]) -> Vec<ChatMessage> {
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(png);
    vec![
        ChatMessage::system(SYSTEM_PROMPT),
        ChatMessage::new(
            Role::User,
            vec![ChatMessageContent::ImageUrl {
                image: ImageUrl {
                    url: format!("data:image/png;base64,{b64}"),
                },
            }],
        ),
    ]
}

/// Transcribe one cue image with a live request.
///
/// Used by the sampler, where an answer is wanted in seconds. The full run goes
/// through the Batch API instead — half the price, and a film's ~1,200 cues are
/// naturally one batch.
pub async fn transcribe(client: &ChatClient, png: &[u8]) -> Result<Transcription> {
    Ok(client.chat_with_messages(messages_for(png)).await?)
}

/// Pull a movie's bitmap subtitle track out to a `.sup`.
pub fn extract_sup(video: &Path, index: u32, dest: &Path) -> Result<()> {
    if dest.exists() {
        return Ok(());
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = dest.with_extension("sup.tmp");
    let status = Command::new("ffmpeg")
        .args(["-v", "error", "-y", "-i"])
        .arg(video)
        // `-f sup` explicitly: the temp file's `.sup.tmp` name defeats
        // ffmpeg's extension-based format inference.
        .args(["-map", &format!("0:{index}"), "-c", "copy", "-f", "sup"])
        .arg(&tmp)
        .status()
        .context("ffmpeg failed to start")?;
    if !status.success() {
        let _ = std::fs::remove_file(&tmp);
        bail!("ffmpeg exited with {status}");
    }
    std::fs::rename(&tmp, dest)?;
    Ok(())
}

/// A cue rendered to PNG, ready to send.
pub struct CueImage {
    pub start_ms: u32,
    pub end_ms: u32,
    pub png: Vec<u8>,
    pub width: u16,
    pub height: u16,
}

/// Decode a `.sup` and render the cues that look like dialogue.
pub fn cue_images(sup: &Path) -> Result<Vec<CueImage>> {
    let data = std::fs::read(sup).with_context(|| format!("Failed to read {}", sup.display()))?;
    render_cues(pgs::cues(&data), pgs::Cue::looks_like_text)
}

/// Decode a `dvd_subtitle` (VobSub) track straight out of the MKV.
///
/// The PGS text filter demands ≥4 antialiased colours, but a DVD subpicture
/// has only four palette entries *total* — usually text, outline, and a
/// transparent background — so the only graphics filter that survives is
/// "something is actually inked". The OCR model's `not_text` verdict catches
/// the rare logo the cheap filter lets through.
pub fn vobsub_cue_images(video: &Path, index: u32) -> Result<Vec<CueImage>> {
    render_cues(crate::vobsub::cues(video, index)?, |cue| {
        cue.height >= 8 && cue.ink_and_colours().0 > 0.001
    })
}

fn render_cues(
    cues: Vec<pgs::Cue>,
    looks_like_text: impl Fn(&pgs::Cue) -> bool,
) -> Result<Vec<CueImage>> {
    let mut out = Vec::new();
    for cue in cues {
        if !looks_like_text(&cue) {
            continue;
        }
        let img = cue.to_rgb([0, 0, 0]);
        let mut png = std::io::Cursor::new(Vec::new());
        img.write_to(&mut png, image::ImageFormat::Png)?;
        out.push(CueImage {
            start_ms: cue.start_ms,
            end_ms: cue.end_ms,
            png: png.into_inner(),
            width: cue.width,
            height: cue.height,
        });
    }
    Ok(out)
}

/// Deterministically pick `n` items spread across `items`.
///
/// A fixed stride rather than a shuffle: the sample must be reproducible so a
/// cost estimate can be re-checked, and cues early in a film (credits, titles)
/// are unrepresentative of the rest.
pub fn spread<T>(items: &[T], n: usize) -> Vec<&T> {
    if items.is_empty() || n == 0 {
        return vec![];
    }
    if n >= items.len() {
        return items.iter().collect();
    }
    (0..n)
        .map(|i| &items[i * items.len() / n + items.len() / (2 * n)])
        .collect()
}

/// Render transcribed cues as an SRT, keeping the disc's own timings.
///
/// Those timings are the whole reason this path is preferred: they were
/// authored against this exact file, so the result needs no synchronisation.
pub fn to_srt(lines: &[(u32, u32, String)]) -> String {
    fn stamp(ms: u32) -> String {
        let (h, m, s, milli) = (ms / 3_600_000, ms / 60_000 % 60, ms / 1000 % 60, ms % 1000);
        format!("{h:02}:{m:02}:{s:02},{milli:03}")
    }
    let mut out = String::new();
    for (i, (start, end, text)) in lines.iter().enumerate() {
        // The OCR model sometimes returns a two-line cue with a literal
        // backslash-n; in SRT a line break is a real newline.
        let text = text.trim().replace("\\n", "\n");
        out.push_str(&format!(
            "{}\n{} --> {}\n{}\n\n",
            i + 1,
            stamp(*start),
            stamp(*end),
            text
        ));
    }
    out
}

/// Path for a movie's extracted `.sup` under the corpus root.
pub fn sup_path(out: &Path, imdb_id: &str) -> PathBuf {
    out.join(imdb_id).join("subtitle.sup")
}

#[cfg(test)]
mod tests {
    use super::readable;

    #[test]
    fn readable_rejects_controls_but_allows_subtitle_whitespace() {
        assert!(readable("répétition\nsérieuse\r\n\t"));
        assert!(!readable("r\0e9pétition"));
        assert!(!readable("sérieuse\u{000e}9"));
    }
}
