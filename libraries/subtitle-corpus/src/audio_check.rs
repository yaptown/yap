//! Does the extracted track really carry the film's own dialogue, in the
//! language the library says the film is in?
//!
//! Stream tags answer this only when the disc bothered: a director's
//! commentary comes first on plenty of rips with no `comment` disposition,
//! and Hong Kong discs label a Cantonese track "chi" exactly as they label
//! the Mandarin dub. Both slipped through in 2026-09-03 — The Mermaid was
//! transcribed from a commentary track, two films from Cantonese ones — and
//! the only symptom was clips that agreed with nothing. So before any
//! transcription is bought on a track, a model listens to a few windows of
//! it and says what it hears. A rejected track is evicted and recorded in
//! `audio-rejected.json`, and the extractor moves on to the next candidate
//! stream on the disc.

use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};
use google_speech::gemini::{GenerateContent, Output, Part};
use serde::{Deserialize, Serialize};

pub const MODEL: &str = "gemini-3.1-pro-preview";

/// Where in the runtime each sample starts, as a fraction: past the opening
/// credits, short of the closing ones, spread so a single foreign-language
/// scene cannot carry the verdict.
pub const SAMPLE_POINTS: &[f64] = &[0.2, 0.45, 0.7];
/// A second set, for when the first three windows land on fights, music
/// or silence: an action film can go minutes without a line. Silence is
/// never grounds for rejection — after this set the track is accepted
/// unheard.
pub const RETRY_POINTS: &[f64] = &[0.3, 0.55, 0.8];
const SAMPLE_SECS: f64 = 40.0;

/// What the model heard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verdict {
    /// The language the dialogue is mainly in, with the variety named.
    pub spoken_language: String,
    /// Whether that is the language the library expected.
    pub expected_language_spoken: bool,
    /// Whether the samples carried enough dialogue to judge the track by —
    /// a stray word in a fight scene is not evidence of anything.
    pub enough_dialogue: bool,
    /// People talking about the film over its soundtrack, not the film.
    pub commentary: bool,
    /// "high", "medium" or "low".
    pub confidence: String,
    pub notes: String,
}

impl Verdict {
    /// Only a track the model heard as the film's own dialogue, in the
    /// expected language, may feed the pipeline. A track that gave the
    /// listener too little to go on is not evidence against it.
    pub fn accepted(&self) -> bool {
        !self.commentary && (self.expected_language_spoken || !self.enough_dialogue)
    }
}

/// The language name a listener should judge the track against. Radarr names
/// the language, not the variety, and where a course teaches one variety the
/// other is not usable: "Chinese" covers Cantonese films the Mandarin course
/// cannot use, and "Portuguese" covers European films the Brazilian course
/// cannot use. Narrow to the variety the course teaches, so a track in the
/// other one comes back `expected_language_spoken: false` and is evicted.
///
/// Narrowing a language here invalidates the stored verdicts judged against
/// the old name — see `audio_checked`, which compares this string.
pub fn expected_language(original_language: &str) -> &str {
    match original_language {
        "Chinese" | "Mandarin" => "Mandarin Chinese",
        "Portuguese" => "Brazilian Portuguese",
        other => other,
    }
}

/// A window of the track at each of `points` (fractions of the runtime),
/// as mono 16 kHz opus for the wire.
///
/// Reads the extracted opus, so this costs a few seeks — never a pass over
/// the remux.
pub fn samples(audio: &Path, duration_ms: i64, points: &[f64]) -> Result<Vec<Vec<u8>>> {
    let runtime = duration_ms as f64 / 1000.0;
    if runtime < SAMPLE_SECS * 2.0 {
        bail!("track is only {runtime:.0}s long");
    }
    points
        .iter()
        .map(|point| {
            let start = (runtime * point).min(runtime - SAMPLE_SECS);
            let out = Command::new("ffmpeg")
                .args(["-v", "error", "-ss", &format!("{start:.3}")])
                .args(["-t", &format!("{SAMPLE_SECS:.3}"), "-i"])
                .arg(audio)
                .args([
                    "-ac", "1", "-ar", "16000", "-c:a", "libopus", "-b:a", "24k", "-f", "ogg", "-",
                ])
                .output()
                .context("ffmpeg failed to start")?;
            if !out.status.success() || out.stdout.is_empty() {
                bail!("ffmpeg could not cut the sample at {start:.0}s");
            }
            Ok(out.stdout)
        })
        .collect()
}

fn prompt(expected: &str) -> String {
    format!(
        "You are listening to three samples from one audio track of a feature film, taken at \
         roughly a quarter, half and three quarters of the way through. The library lists the \
         film's language as {expected}, and this track was picked as the film's own \
         original-language dialogue track. Judge whether it really is.\n\n\
         spoken_language: the language the dialogue is mainly in. Name the variety where it \
         matters — Mandarin against Cantonese, European against Brazilian Portuguese — since a \
         course in one variety cannot use the other.\n\
         expected_language_spoken: whether the dialogue is mainly in {expected}. A film may \
         switch languages for a scene, so answer for the track as a whole, not for any one \
         line.\n\
         enough_dialogue: whether the samples carried enough dialogue to judge the track \
         by — several full lines, not a stray word or two in a fight or a chase. When there \
         was not, say so here and leave expected_language_spoken false rather than guessing; \
         a track is never rejected for being quiet, only for what was clearly heard.\n\
         commentary: whether this is a commentary track — a director, cast or critics talking \
         about the film over its soundtrack, with the film's own dialogue faint or absent \
         underneath — rather than the film itself.\n\
         confidence: high, medium or low.\n\
         notes: a sentence or two on what you heard that decided it.\n\n\
         Music and silence are fine; a sample with no speech says nothing either way, so lean \
         on the ones that have some."
    )
}

/// Ask the model what it hears on the track.
pub async fn judge(
    client: &google_speech::gemini::GeminiClient,
    expected: &str,
    samples: &[Vec<u8>],
) -> Result<Verdict> {
    let mut parts = vec![Part::Text(prompt(expected))];
    for sample in samples {
        parts.push(Part::InlineData {
            mime_type: "audio/ogg".to_owned(),
            data: sample.clone(),
        });
    }
    let request = GenerateContent {
        model: MODEL.to_owned(),
        parts,
        output: Output::Json {
            schema: serde_json::json!({
                "type": "OBJECT",
                "properties": {
                    "spoken_language": { "type": "STRING" },
                    "expected_language_spoken": { "type": "BOOLEAN" },
                    "enough_dialogue": { "type": "BOOLEAN" },
                    "commentary": { "type": "BOOLEAN" },
                    "confidence": { "type": "STRING", "enum": ["high", "medium", "low"] },
                    "notes": { "type": "STRING" },
                },
                "required": ["spoken_language", "expected_language_spoken", "enough_dialogue", "commentary", "confidence", "notes"],
            }),
        },
    };
    let response = client.generate(&request).await?;
    let answer = response.text().context("response has no candidate text")?;
    serde_json::from_str(answer).with_context(|| format!("verdict is not the schema: {answer}"))
}
