//! Loading and verification of human-recorded audio for a language.
//!
//! Reads per-contributor recordings declared under `<source_data>/audio/`,
//! runs each clip through the phonemic verifier (see [`crate::audio_verification`])
//! so badly-mismatched takes never reach the language pack, and encodes the
//! survivors to OGG/Opus for shipping.

use anyhow::Context;
use futures::{StreamExt, TryStreamExt};
use rustc_hash::FxHashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Load human-recorded audio for a language from `<source_data>/audio/<credit>/`.
///
/// Each contributor lives in their own subdirectory whose name becomes the credit.
/// The subdirectory must contain a `manifest.jsonl` mapping target-language text
/// to a relative .wav filename: `{"text": "tu", "file": "01a_tu.wav"}`.
///
/// Compensation (paid vs. volunteer) is declared in `<source_data>/audio/voice-actors.json`,
/// which must contain an entry for every contributor subdirectory. We require this
/// to be explicit so we never accidentally claim a volunteer contribution is paid.
///
/// Each .wav is encoded to OGG/Opus via ffmpeg. The browser already supports this
/// format (see `is_valid_audio_data` in yap-frontend-rs/src/audio.rs).
pub async fn load_human_audio(
    source_data_path: &Path,
    word_to_pronunciation: &[(String, language_utils::Pronunciations)],
    failures_log: &Path,
    all_results_log: &Path,
    target_language: language_utils::Language,
    http: &reqwest::Client,
) -> anyhow::Result<FxHashMap<language_utils::VoiceActor, FxHashMap<String, language_utils::Audio>>>
{
    let audio_root = source_data_path.join("audio");
    let mut out: FxHashMap<language_utils::VoiceActor, FxHashMap<String, language_utils::Audio>> =
        FxHashMap::default();

    // Wipe stale verification logs up front so each file reflects only the
    // most recent run — they're re-created at the end only if this run
    // produces matching results.
    for log_path in [failures_log, all_results_log] {
        if log_path.exists() {
            std::fs::remove_file(log_path)
                .with_context(|| format!("Failed to remove stale {}", log_path.display()))?;
        }
    }

    if !audio_root.exists() {
        return Ok(out);
    }

    #[derive(serde::Deserialize)]
    struct ManifestEntry {
        text: String,
        file: String,
        /// Optional speaker-realized IPA, whitespace-separated. When
        /// present, the verifier scores model output against this exact
        /// sequence instead of the language-wide wikipron+espeak ground
        /// truth — use to encode connected-speech realizations that
        /// differ from citation form (e.g. French `de` produced as
        /// /dø/ by speakers who merge unstressed schwa with ø).
        #[serde(default)]
        phonemic_transcription: Option<String>,
    }

    /// Quality tier for a voice actor. Drives whether their clips end up
    /// in the shipped language pack — only `High` ships; `Medium` and `Low`
    /// are kept in the repo for diagnostics / verifier benchmarking but
    /// never reach users. An entry that omits `quality` defaults to
    /// `Medium`, so clips ship only when an actor is *explicitly* marked
    /// `High` — an unannotated actor never reaches users by accident.
    #[derive(serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
    #[serde(rename_all = "lowercase")]
    enum VoiceActorQuality {
        High,
        #[default]
        Medium,
        Low,
    }

    #[derive(serde::Deserialize)]
    struct VoiceActorEntry {
        compensation: language_utils::Compensation,
        #[serde(default)]
        quality: VoiceActorQuality,
    }

    let mut contributors: Vec<PathBuf> = std::fs::read_dir(&audio_root)
        .with_context(|| format!("Failed to read audio dir {}", audio_root.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    contributors.sort();

    let voice_actors_path = audio_root.join("voice-actors.json");
    let voice_actors: FxHashMap<String, VoiceActorEntry> = if contributors.is_empty() {
        FxHashMap::default()
    } else {
        let text = std::fs::read_to_string(&voice_actors_path).with_context(|| {
            format!(
                "Missing {} — required to declare compensation for each contributor",
                voice_actors_path.display()
            )
        })?;
        serde_json::from_str(&text)
            .with_context(|| format!("Failed to parse {}", voice_actors_path.display()))?
    };

    let wp_lookup: std::collections::HashMap<String, language_utils::Pronunciations> =
        word_to_pronunciation
            .iter()
            .map(|(w, p)| (w.to_lowercase(), p.clone()))
            .collect();
    let verify_ctx = crate::audio_verification::VerifyContext::new(
        http,
        crate::cache_remote::store(),
        &wp_lookup,
        target_language,
    )?;

    let mut failures: Vec<crate::audio_verification::ClipVerification> = Vec::new();
    let mut all_results: Vec<crate::audio_verification::ClipVerification> = Vec::new();

    for contributor_dir in contributors {
        let name = contributor_dir
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("invalid contributor dir: {contributor_dir:?}"))?
            .to_string();
        let entry = voice_actors.get(&name).ok_or_else(|| {
            anyhow::anyhow!(
                "{} has no entry for contributor {:?} — add one with compensation: \"paid\" or \"volunteer\"",
                voice_actors_path.display(),
                name
            )
        })?;
        let compensation = entry.compensation;
        let quality = entry.quality;
        let actor = language_utils::VoiceActor { name, compensation };
        let manifest = contributor_dir.join("manifest.jsonl");
        if !manifest.exists() {
            log::warn!("audio dir {} has no manifest.jsonl; skipping", actor.name);
            continue;
        }
        let manifest_text = std::fs::read_to_string(&manifest)
            .with_context(|| format!("Failed to read {}", manifest.display()))?;

        let mut entries: Vec<ManifestEntry> = Vec::new();
        for (line_no, line) in manifest_text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            entries.push(serde_json::from_str(line).with_context(|| {
                format!("Failed to parse {}:{}", manifest.display(), line_no + 1)
            })?);
        }

        let clips = out.entry(actor.clone()).or_default();

        // Verify clips in parallel — Modal calls are I/O-bound and slow, and
        // running serially over a couple hundred clips is the dominant time
        // cost on first runs. Buffered(8) is a polite concurrency level.
        let v_ctx = &verify_ctx;
        let dir_ref = &contributor_dir;
        let actor_name_ref = actor.name.as_str();
        let verifications: Vec<_> = futures::stream::iter(&entries)
            .map(|entry| async move {
                let wav_path = dir_ref.join(&entry.file);
                let expected = crate::audio_verification::expected_phoneme_variants(
                    v_ctx,
                    &entry.text,
                    entry.phonemic_transcription.as_deref(),
                );
                let v = crate::audio_verification::verify_clip(
                    v_ctx,
                    actor_name_ref,
                    &entry.text,
                    &wav_path,
                    expected,
                )
                .await;
                match v {
                    Ok(v) => anyhow::Ok(Some((entry, wav_path, v))),
                    // A cache-only run never reaches the phoneme endpoint, so a
                    // miss there leaves the clip out of the pack rather than
                    // failing the course.
                    Err(e) if crate::cache_only() => {
                        eprintln!(
                            "cache-only: skipping unverified clip {}: {e:#}",
                            wav_path.display()
                        );
                        Ok(None)
                    }
                    Err(e) => Err(e),
                }
            })
            .buffered(32)
            .try_collect()
            .await?;

        let mut pass_count = 0usize;
        let mut fail_count = 0usize;
        let mut excluded_for_quality = 0usize;
        // Lower-quality actors still get fully verified (we want the
        // diagnostic data in audio_verification_all.jsonl), but their
        // passing clips don't reach the shipped language pack.
        let ship_clips = matches!(quality, VoiceActorQuality::High);
        for (entry, wav_path, verification) in verifications.into_iter().flatten() {
            all_results.push(verification.clone());
            if let Some(reason) = &verification.failure_reason {
                log::info!(
                    "audio verification rejected {:?} from {}: {reason}",
                    entry.text,
                    actor.name
                );
                fail_count += 1;
                failures.push(verification);
                continue;
            }
            pass_count += 1;
            if !ship_clips {
                excluded_for_quality += 1;
                continue;
            }
            let bytes = encode_wav_to_opus(&wav_path)
                .await
                .with_context(|| format!("Failed to encode {} to opus", wav_path.display()))?;
            // TODO: keep every take rather than last-take-wins. Several
            // manifest texts are recorded more than once (alternate takes we
            // commissioned), but the `(actor, text) -> Audio` map discards all
            // but the last. The plan: make this `(actor, text) -> Vec<Audio>`
            // and have the frontend `human_audio::lookup` registry rotate
            // through an actor's takes the same way it already rotates across
            // actors, so learners hear natural variation instead of one clip.
            if clips
                .insert(entry.text.clone(), language_utils::Audio { bytes })
                .is_some()
            {
                log::warn!(
                    "duplicate clip for {:?} from {}: keeping the later one",
                    entry.text,
                    actor.name
                );
            }
        }
        let quality_note = if !ship_clips {
            format!(", {excluded_for_quality} excluded from pack (quality={quality:?})")
        } else {
            String::new()
        };
        println!(
            "Loaded human audio from {} ({pass_count} passed, {fail_count} rejected{quality_note})",
            contributor_dir.display()
        );
        if clips.is_empty() {
            out.remove(&actor);
        }
    }

    // Run Google TTS on every unique text in the manifests and verify the
    // synthetic audio. Diagnostic only — we don't add TTS audio to the
    // dataset. Comprehensive coverage (every text, not just ones where
    // humans failed) lets us establish a verifier baseline: model failures
    // on clean studio audio are model bias, not recording quality.
    // Synthesis is a network call, so a cache-only run skips the diagnostic.
    if !crate::cache_only()
        && let (Ok(api_key), Some(voice)) = (
            std::env::var("GOOGLE_CLOUD_API_KEY"),
            crate::audio_verification::default_voice_for(target_language),
        )
    {
        let mut unique_texts: std::collections::BTreeSet<String> =
            std::collections::BTreeSet::new();
        for v in &all_results {
            unique_texts.insert(v.text.clone());
        }
        let texts: Vec<String> = unique_texts.into_iter().collect();

        if !texts.is_empty() {
            println!(
                "Running Google TTS verification on all {} unique text(s)...",
                texts.len()
            );
            let api_key_ref = api_key.as_str();
            let v_ctx = &verify_ctx;
            let tts_results: Vec<_> = futures::stream::iter(texts.iter())
                .map(|text| async move {
                    let v = crate::audio_verification::verify_with_google_tts(
                        v_ctx,
                        "google-tts",
                        text,
                        voice,
                        api_key_ref,
                    )
                    .await?;
                    anyhow::Ok(v)
                })
                .buffered(16)
                .try_collect()
                .await?;
            let tts_pass = tts_results.iter().filter(|v| v.passed()).count();
            println!(
                "  Google TTS: {tts_pass}/{} passed verification",
                tts_results.len()
            );
            // Append to all_results so they show up in the diagnostic log alongside
            // the human attempts. Don't push to `failures` (we don't drop synthetic
            // audio from the dataset — there's no dataset role for it).
            all_results.extend(tts_results);
        }
    } else if std::env::var("GOOGLE_CLOUD_API_KEY").is_err() {
        log::info!("Skipping Google TTS fallback (GOOGLE_CLOUD_API_KEY not set in env)");
    }

    if !all_results.is_empty() {
        if let Some(parent) = all_results_log.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!(
                    "Failed to create directory for {}",
                    all_results_log.display()
                )
            })?;
        }
        let mut f = File::create(all_results_log).with_context(|| {
            format!(
                "Failed to create audio verification log file {}",
                all_results_log.display()
            )
        })?;
        for v in &all_results {
            let json = serde_json::to_string(v)
                .context("Failed to serialize audio verification result")?;
            writeln!(f, "{json}").context("Failed to write audio verification result")?;
        }
        let pass_total = all_results.iter().filter(|v| v.passed()).count();
        println!(
            "Wrote {} audio verification result(s) to {} ({} passed, {} rejected)",
            all_results.len(),
            all_results_log.display(),
            pass_total,
            all_results.len() - pass_total,
        );
    }

    if !failures.is_empty() {
        if let Some(parent) = failures_log.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create directory for {}", failures_log.display())
            })?;
        }
        let mut f = File::create(failures_log).with_context(|| {
            format!(
                "Failed to create audio verification failures file {}",
                failures_log.display()
            )
        })?;
        for v in &failures {
            let json = serde_json::to_string(v)
                .context("Failed to serialize audio verification failure")?;
            writeln!(f, "{json}").context("Failed to write audio verification failure")?;
        }
        println!(
            "Wrote {} audio verification failure(s) to {} for manual review",
            failures.len(),
            failures_log.display()
        );
    }

    Ok(out)
}

/// The cache key and ffmpeg invocation share this recipe. Bitexact flags remove
/// encoder version tags and random Ogg serial numbers; the cache preserves the
/// encoded bytes even across encoder upgrades.
const OPUS_ARGS: &[&str] = &[
    "-af",
    "loudnorm=I=-16:TP=-1.5:LRA=11",
    "-c:a",
    "libopus",
    "-b:a",
    "32k",
    "-application",
    "voip",
    "-fflags",
    "+bitexact",
    "-flags:a",
    "+bitexact",
    "-f",
    "ogg",
    "pipe:1",
];

/// Cache the exact WAV bytes, not its path. Local encoding is also allowed in
/// cache-only mode: it needs no network and must not drop verified recordings.
async fn encode_wav_to_opus(wav_path: &Path) -> anyhow::Result<Vec<u8>> {
    let wav = std::fs::read(wav_path)?;
    cached_opus(&crate::cache_remote::store(), &wav, OPUS_ARGS, encode_opus).await
}

async fn cached_opus(
    store: &osmo::Store,
    wav: &[u8],
    recipe: &[&str],
    encode: impl FnOnce(&[u8], &[&str]) -> anyhow::Result<Vec<u8>>,
) -> anyhow::Result<Vec<u8>> {
    use xxhash_rust::xxh3::xxh3_64;

    // JSON preserves argument boundaries (unlike joining with spaces).
    let recipe_hash = xxh3_64(&serde_json::to_vec(recipe)?);
    let key = format!("human-audio/opus/{recipe_hash:016x}/{:016x}", xxh3_64(wav));
    if let Some(bytes) = store.read(&key).await {
        return Ok(bytes);
    }
    let bytes = encode(wav, recipe)?;
    store.write(&key, &bytes).await?;
    Ok(bytes)
}

/// Apply EBU R128 loudness normalization so contributors land at a consistent
/// perceived volume. -16 LUFS is the Apple Podcasts spoken-word standard.
fn encode_opus(wav: &[u8], recipe: &[&str]) -> anyhow::Result<Vec<u8>> {
    // Encode exactly what was hashed, even if the original file changes. A temp
    // input also avoids deadlocking ffmpeg's stdin against its piped stdout.
    let mut input = tempfile::NamedTempFile::new()?;
    input.write_all(wav)?;
    let output = std::process::Command::new("ffmpeg")
        .args(["-y", "-loglevel", "error", "-i"])
        .arg(input.path())
        .args(recipe)
        .output()
        .context("failed to invoke ffmpeg (is it installed?)")?;
    if !output.status.success() {
        anyhow::bail!("ffmpeg failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    if !output.stdout.starts_with(b"OggS") {
        anyhow::bail!(
            "ffmpeg produced non-OGG output ({} bytes, magic={:?})",
            output.stdout.len(),
            &output.stdout.iter().take(4).copied().collect::<Vec<_>>()
        );
    }
    Ok(output.stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn same_wav_and_recipe_reuse_encoded_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let store = osmo::Store::open(dir.path());
        let bytes = cached_opus(&store, b"wav", OPUS_ARGS, |wav, recipe| {
            assert_eq!(wav, b"wav");
            assert_eq!(recipe, OPUS_ARGS);
            Ok(b"OggSencoded".to_vec())
        })
        .await
        .unwrap();
        let hit = cached_opus(&store, b"wav", OPUS_ARGS, |_, _| {
            panic!("cache hit must not invoke encoder")
        })
        .await
        .unwrap();
        assert_eq!(hit, bytes);
    }

    #[tokio::test]
    async fn changed_wav_or_recipe_encodes_again() {
        let dir = tempfile::tempdir().unwrap();
        let store = osmo::Store::open(dir.path());
        for (wav, recipe, expected) in [
            (b"wav".as_slice(), &["a b", "c"][..], b"first".as_slice()),
            (b"new wav", &["a b", "c"][..], b"changed wav"),
            (b"wav", &["a", "b c"][..], b"changed recipe"),
        ] {
            let bytes = cached_opus(&store, wav, recipe, |_, _| Ok(expected.to_vec()))
                .await
                .unwrap();
            assert_eq!(bytes, expected);
        }
    }

    #[tokio::test]
    async fn failed_encoding_is_not_cached() {
        let dir = tempfile::tempdir().unwrap();
        let store = osmo::Store::open(dir.path());
        let result = cached_opus(&store, b"wav", OPUS_ARGS, |_, _| {
            anyhow::bail!("encoder failed")
        })
        .await;
        assert_eq!(result.unwrap_err().to_string(), "encoder failed");
        let bytes = cached_opus(&store, b"wav", OPUS_ARGS, |_, _| Ok(b"retry".to_vec()))
            .await
            .unwrap();
        assert_eq!(bytes, b"retry");
    }

    /// Exercise the real encoder against repository recordings, without loading
    /// a course or contacting verification services.
    #[tokio::test]
    #[ignore = "requires ffmpeg and the French audio recordings"]
    async fn french_recordings_encode_reproducibly() {
        let dir = tempfile::tempdir().unwrap();
        let store = osmo::Store::open(dir.path());
        let version_tag = regex::Regex::new(r"(?:Lav[fc]|libopus)\s*\d").unwrap();
        for file in ["05a_bu.wav", "b185_je_sais_que.wav"] {
            let path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("data/fra/audio/Jean-Cavard")
                .join(file);
            let wav = std::fs::read(&path).unwrap();
            let encoded = cached_opus(&store, &wav, OPUS_ARGS, encode_opus)
                .await
                .unwrap();
            let hit = cached_opus(&store, &wav, OPUS_ARGS, |_, _| {
                panic!("{file}: cache hit must not invoke ffmpeg")
            })
            .await
            .unwrap();
            assert_eq!(hit, encoded);
            assert_eq!(encode_opus(&wav, OPUS_ARGS).unwrap(), encoded);
            assert!(encoded.starts_with(b"OggS"));
            let text = String::from_utf8_lossy(&encoded);
            // Bitexact keeps an unversioned "Lavc libopus" encoder label.
            assert!(!version_tag.is_match(&text), "{file}: encoder version tag");
            let mut ogg = tempfile::NamedTempFile::new().unwrap();
            ogg.write_all(&encoded).unwrap();
            let decoded = std::process::Command::new("ffmpeg")
                .args(["-v", "error", "-i"])
                .arg(ogg.path())
                .args(["-f", "f32le", "pipe:1"])
                .output()
                .unwrap();
            assert!(
                decoded.status.success(),
                "{}",
                String::from_utf8_lossy(&decoded.stderr)
            );
            assert!(!decoded.stdout.is_empty());
            println!(
                "{file}: {} Ogg bytes, {} decoded PCM bytes; cache hit and fresh encode identical, no version tags",
                encoded.len(),
                decoded.stdout.len()
            );
        }
    }
}
