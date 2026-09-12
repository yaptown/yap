//! Pre-generate every pronunciation-challenge clip and embed only audio that
//! passes the phoneme model's verification gate.

use anyhow::{Context, Result};
use futures::{StreamExt, TryStreamExt};
use language_utils::{Audio, Language, PronunciationData, Pronunciations};
use rustc_hash::FxHashMap;
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::path::Path;

pub async fn generate_pronunciation_audio(
    pronunciation_data: &PronunciationData,
    word_to_pronunciation: &[(String, Pronunciations)],
    language: Language,
    http: &reqwest::Client,
    results_log: &Path,
) -> Result<FxHashMap<String, Audio>> {
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
            let spoken = language_utils::pronunciation_challenge_spoken_text(
                language,
                &guide.pattern,
                &example.target,
            );
            requests.insert(ssml, spoken);
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

    let generated: Vec<_> = futures::stream::iter(requests.iter())
        .map(|(ssml, spoken)| {
            let ctx = &ctx;
            let api_key = api_key.as_deref();
            async move {
                let (bytes, verification) =
                    crate::audio_verification::synthesize_verified_google_tts(
                        ctx,
                        "google-tts-pronunciation",
                        ssml,
                        spoken,
                        voice,
                        true,
                        api_key,
                    )
                    .await?;
                anyhow::Ok((ssml.clone(), bytes, verification))
            }
        })
        .buffered(4)
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
    for (_, _, verification) in &generated {
        serde_json::to_writer(&mut log, verification).context("serializing verification result")?;
        writeln!(log).context("writing pronunciation audio verification log")?;
    }

    let failures: Vec<_> = generated
        .iter()
        .filter(|(_, _, verification)| !verification.passed())
        .map(|(_, _, verification)| {
            format!(
                "{:?}: {}",
                verification.text,
                verification.failure_reason.as_deref().unwrap_or("rejected")
            )
        })
        .collect();
    if !failures.is_empty() {
        anyhow::bail!(
            "{} of {} pronunciation clips failed phoneme verification:\n{}",
            failures.len(),
            generated.len(),
            failures.join("\n")
        );
    }

    println!(
        "Precached and phoneme-verified {} pronunciation challenge clip(s)",
        generated.len()
    );
    Ok(generated
        .into_iter()
        .map(|(ssml, bytes, _)| (ssml, Audio { bytes }))
        .collect())
}
