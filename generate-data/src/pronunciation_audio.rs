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
use rustc_hash::FxHashMap;
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::path::Path;

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
    // and diagnostics deterministic.
    let mut requests = BTreeMap::new();
    for guide in &pronunciation_data.guides {
        for example in &guide.example_words {
            let ssml = language_utils::pronunciation_challenge_ssml(
                language,
                &guide.pattern,
                &example.target,
            );
            let segments = language_utils::pronunciation_challenge_segments(
                language,
                &guide.pattern,
                &example.target,
            );
            requests.insert(ssml, segments);
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
    let (language_code, voice_name) = language.google_tts_voice(true);
    let voice = crate::audio_verification::TtsVoice {
        language_code,
        voice_name,
    };
    let api_key = std::env::var("GOOGLE_CLOUD_API_KEY").ok();

    // Infrastructure errors (TTS call, phoneme endpoint, cache-only miss)
    // still abort: they mean the run can't tell good audio from bad. Only a
    // verdict of "rejected" is a per-clip outcome.
    let generated: Vec<_> = futures::stream::iter(requests.iter())
        .map(|(ssml, segments)| {
            let ctx = &ctx;
            let api_key = api_key.as_deref();
            async move {
                let spoken = segments
                    .iter()
                    .map(|segment| segment.spoken.as_str())
                    .collect::<Vec<_>>();
                let (bytes, verification) =
                    crate::audio_verification::synthesize_verified_google_tts(
                        ctx,
                        "google-tts-pronunciation",
                        ssml,
                        &spoken.join(" "),
                        voice,
                        true,
                        api_key,
                    )
                    .await?;
                let mut timed = Vec::new();
                if verification.passed() {
                    match crate::audio_verification::segment_timings(ctx, &bytes, &spoken).await {
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
                            "WARNING: no segment timings for {:?}, clip kept without them: {e:#}",
                            verification.text
                        ),
                    }
                }
                let clip = PronunciationClip {
                    audio: Audio { bytes },
                    segments: timed,
                };
                anyhow::Ok((ssml.clone(), clip, verification))
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
    for (_, clip, verification) in &generated {
        // The verdict plus where each segment landed, so a timing can be
        // checked against the clip without unpacking the language pack.
        let mut line = serde_json::to_value(verification).context("serializing verification")?;
        let segments: Vec<_> = clip
            .segments
            .iter()
            .map(|s| serde_json::json!([s.text, s.start_ms, s.end_ms]))
            .collect();
        line["segments"] = serde_json::Value::Array(segments);
        serde_json::to_writer(&mut log, &line).context("serializing verification result")?;
        writeln!(log).context("writing pronunciation audio verification log")?;
    }

    let failures: Vec<String> = generated
        .iter()
        .filter(|(_, _, verification)| !verification.passed())
        .map(|(_, _, verification)| {
            format!(
                "  {:?}: {}",
                verification.text,
                verification.failure_reason.as_deref().unwrap_or("rejected")
            )
        })
        .collect();
    if !failures.is_empty() {
        println!(
            "WARNING: {} of {} pronunciation clips failed phoneme verification and are left out \
             of the language pack (predicted vs expected phonemes in {}):\n{}",
            failures.len(),
            generated.len(),
            results_log.display(),
            failures.join("\n")
        );
    }

    let verified: FxHashMap<String, PronunciationClip> = generated
        .into_iter()
        .filter(|(_, _, verification)| verification.passed())
        .map(|(ssml, clip, _)| (ssml, clip))
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
            verified.contains_key(&language_utils::pronunciation_challenge_ssml(
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
