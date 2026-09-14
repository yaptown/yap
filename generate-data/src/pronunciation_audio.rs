//! Pre-generate every pronunciation-challenge clip, keep only the audio that
//! passes the phoneme model's verification gate, and prune the guides down to
//! the examples that have it — so a language pack never advertises an example
//! it can't play. A guide whose every example is rejected is dropped, which
//! keeps its pattern from ever becoming a card. Each kept clip also records
//! when each segment of its transcript is said, for highlighting along.

use anyhow::{Context, Result};
use futures::{StreamExt, TryStreamExt};
use language_utils::{
    Audio, Language, PronunciationClip, PronunciationData, PronunciationGuide, Pronunciations,
    TimedSegment,
};
use phoneme_verify::TtsSynthesis;
use rustc_hash::FxHashMap;
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::path::Path;

/// The Gemini voice that reads the cues. Achernar was the clearest of the
/// prebuilt voices across every course language in listening tests.
const GEMINI_VOICE: &str = google_tts::gemini::DEFAULT_VOICE;

pub async fn generate_pronunciation_audio(
    pronunciation_data: &mut PronunciationData,
    word_to_pronunciation: &[(String, Pronunciations)],
    language: Language,
    http: &reqwest::Client,
    results_log: &Path,
) -> Result<FxHashMap<String, PronunciationClip>> {
    if language.g2p_lang().is_none() {
        // Nothing to verify against: leave the guides alone and let the app
        // fall back to runtime TTS, as it did before clips were precached.
        println!(
            "Skipping pronunciation audio for {language:?}: no phonemizer ground truth, \
             guides keep runtime TTS"
        );
        return Ok(FxHashMap::default());
    }

    // BTreeMap both deduplicates repeated guide examples and makes generation
    // and diagnostics deterministic. Keyed by the spoken text: the one string
    // every voice is given and every clip is verified against.
    let mut requests = BTreeMap::new();
    for guide in &pronunciation_data.guides {
        for example in &guide.example_words {
            let segments = language_utils::pronunciation_challenge_segments(
                language,
                &guide.pattern,
                &example.target,
            );
            let spoken = segments
                .iter()
                .map(|segment| segment.spoken.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            requests.insert(spoken, segments);
        }
    }
    if requests.is_empty() {
        return Ok(FxHashMap::default());
    }

    let pronunciations: HashMap<String, Pronunciations> = word_to_pronunciation
        .iter()
        .map(|(word, pronunciations)| (word.to_lowercase(), pronunciations.clone()))
        .collect();
    let ctx = crate::audio_verification::VerifyContext::new(
        http,
        crate::cache_remote::store(),
        &pronunciations,
        language,
    )?;
    let keys = crate::audio_verification::TtsKeys::from_env();
    let instructions = language_utils::pronunciation_challenge_tts_instructions(language);
    let (language_code, voice_name) = language.google_tts_voice();
    let google_voice = crate::audio_verification::TtsVoice {
        language_code,
        voice_name,
    };

    // Infrastructure errors (TTS call, phoneme endpoint, cache-only miss)
    // still abort: they mean the run can't tell good audio from bad. Only a
    // verdict of "rejected" is a per-clip outcome.
    let generated: Vec<_> = futures::stream::iter(requests.iter())
        .map(|(spoken, segments)| {
            let ctx = &ctx;
            let keys = &keys;
            let instructions = &instructions;
            async move {
                // Gemini reads cues far more naturally than Cloud TTS, but
                // it is stochastic: a draw that garbles a letter name is
                // usually followed by one that doesn't. Two draws, then the
                // literal Chirp 3 voice, and the first clip to pass the
                // phoneme gate is the one the pack ships.
                let candidates = [
                    TtsSynthesis::Gemini {
                        voice: GEMINI_VOICE.to_string(),
                        instructions: instructions.clone(),
                        text: spoken.clone(),
                        attempt: 1,
                    },
                    TtsSynthesis::Gemini {
                        voice: GEMINI_VOICE.to_string(),
                        instructions: instructions.clone(),
                        text: spoken.clone(),
                        attempt: 2,
                    },
                    TtsSynthesis::Google {
                        voice: google_voice,
                        text: spoken.clone(),
                    },
                ];
                let mut attempts = Vec::new();
                let mut winner = None;
                for candidate in candidates {
                    let (bytes, verification) = crate::audio_verification::synthesize_verified(
                        ctx,
                        "tts-pronunciation",
                        &candidate,
                        spoken,
                        keys,
                    )
                    .await?;
                    let passed = verification.passed();
                    attempts.push(verification);
                    if passed {
                        winner = Some(bytes);
                        break;
                    }
                }
                let spoken_words = segments
                    .iter()
                    .map(|segment| segment.spoken.as_str())
                    .collect::<Vec<_>>();
                let mut timed = Vec::new();
                if let Some(bytes) = &winner {
                    match crate::audio_verification::segment_timings(ctx, bytes, &spoken_words)
                        .await
                    {
                        Ok(timings) => {
                            timed = segments
                                .iter()
                                .zip(timings)
                                .map(|(segment, (start_ms, end_ms))| TimedSegment {
                                    text: segment.display.clone(),
                                    start_ms,
                                    end_ms,
                                })
                                .collect();
                        }
                        Err(e) => println!(
                            "WARNING: no segment timings for {spoken:?}, clip kept without them: {e:#}"
                        ),
                    }
                }
                let clip = winner.map(|bytes| PronunciationClip {
                    audio: Audio { bytes },
                    segments: timed,
                });
                anyhow::Ok((spoken.clone(), clip, attempts))
            }
        })
        .buffered(32)
        .try_collect()
        .await?;

    if let Some(parent) = results_log.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    let mut log = std::io::BufWriter::new(
        std::fs::File::create(results_log)
            .with_context(|| format!("creating {}", results_log.display()))?,
    );
    for (_, clip, attempts) in &generated {
        for verification in attempts {
            // Every attempt's verdict, plus where each segment of the kept
            // clip landed, so a timing can be checked against the clip
            // without unpacking the language pack.
            let mut line =
                serde_json::to_value(verification).context("serializing verification")?;
            if verification.passed()
                && let Some(clip) = clip
            {
                let segments: Vec<_> = clip
                    .segments
                    .iter()
                    .map(|s| serde_json::json!([s.text, s.start_ms, s.end_ms]))
                    .collect();
                line["segments"] = serde_json::Value::Array(segments);
            }
            serde_json::to_writer(&mut log, &line).context("serializing verification result")?;
            writeln!(log).context("writing pronunciation audio verification log")?;
        }
    }

    let failures: Vec<String> = generated
        .iter()
        .filter(|(_, clip, _)| clip.is_none())
        .map(|(spoken, _, attempts)| {
            let reasons = attempts
                .iter()
                .map(|v| {
                    format!(
                        "{}: {}",
                        v.wav_path,
                        v.failure_reason.as_deref().unwrap_or("rejected")
                    )
                })
                .collect::<Vec<_>>()
                .join("; ");
            format!("  {spoken:?}: {reasons}")
        })
        .collect();
    if !failures.is_empty() {
        println!(
            "WARNING: {} of {} pronunciation clips failed phoneme verification with every voice \
             and are left out of the language pack (predicted vs expected phonemes in {}):\n{}",
            failures.len(),
            generated.len(),
            results_log.display(),
            failures.join("\n")
        );
    }
    let by_voice = generated
        .iter()
        .filter_map(|(_, clip, attempts)| clip.as_ref().map(|_| attempts.last()))
        .flatten()
        .fold(BTreeMap::new(), |mut counts, v| {
            *counts.entry(v.wav_path.as_str()).or_insert(0usize) += 1;
            counts
        });
    println!(
        "Pronunciation clips by winning voice: {}",
        by_voice
            .iter()
            .map(|(voice, n)| format!("{voice} {n}"))
            .collect::<Vec<_>>()
            .join(", ")
    );

    let verified: FxHashMap<String, PronunciationClip> = generated
        .into_iter()
        .filter_map(|(spoken, clip, _)| clip.map(|clip| (spoken, clip)))
        .collect();
    let untimed = verified
        .values()
        .filter(|clip| clip.segments.is_empty())
        .count();

    let mut dropped_examples = 0;
    let mut dropped_guides = Vec::new();
    pronunciation_data.guides.retain_mut(|guide| {
        let PronunciationGuide {
            pattern,
            position,
            example_words,
            ..
        } = guide;
        let before = example_words.len();
        example_words.retain(|example| {
            verified.contains_key(&language_utils::pronunciation_challenge_spoken_text(
                language,
                pattern,
                &example.target,
            ))
        });
        dropped_examples += before - example_words.len();
        if example_words.is_empty() {
            dropped_guides.push(format!("{pattern:?} ({position:?})"));
        }
        !example_words.is_empty()
    });
    if dropped_examples > 0 {
        println!(
            "Dropped {dropped_examples} unverified pronunciation example(s); {} guide(s) lost every \
             example and were removed: {}",
            dropped_guides.len(),
            dropped_guides.join(", ")
        );
    }

    println!(
        "Precached and phoneme-verified {} pronunciation challenge clip(s), {untimed} without \
         segment timings",
        verified.len()
    );
    Ok(verified)
}
