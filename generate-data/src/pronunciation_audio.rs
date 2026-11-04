//! Generate and verify whole pronunciation cues. Only verified examples ship;
//! a guide with none is dropped and reported after bounded replacement rounds.
use anyhow::{Context, Result};
use futures::{StreamExt, TryStreamExt};
use language_utils::{
    Audio, Course, Language, PatternPosition, PronunciationClip, PronunciationData, Pronunciations,
    TimedSegment,
};
use phoneme_verify::{ClipVerification, TtsSynthesis};
use rustc_hash::FxHashMap;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::Write;
use std::path::Path;

/// Check both the example and the cue; a readable connector cannot conceal an
/// unsupported jamo, terminal small tsu, or unpronounceable letter name.
pub fn eligibility(language: Language, pattern: &str, example: &str) -> Result<()> {
    let spoken = language_utils::pronunciation_challenge_spoken_text(language, pattern, example);
    eligible_texts(language, example, &spoken)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VerificationPolicy {
    PhonemeAndTranscript,
    Transcript,
}

impl VerificationPolicy {
    fn for_language(language: Language) -> Self {
        match language.phoneme_label_source() {
            language_utils::PhonemeLabelSource::Unvalidated => Self::Transcript,
            _ => Self::PhonemeAndTranscript,
        }
    }
}

fn eligible_texts(language: Language, example: &str, spoken: &str) -> Result<()> {
    if VerificationPolicy::for_language(language) == VerificationPolicy::Transcript {
        anyhow::ensure!(
            whisper::whisper_language(language).is_some(),
            "no transcript verifier for {language:?}"
        );
        return Ok(());
    }
    // An empty example is used only to preflight the immutable cue before
    // asking for replacements. Real examples are checked independently.
    if !example.is_empty() {
        phoneme_verify::complete_target(example, language).context("example target")?;
    }
    phoneme_verify::complete_target(spoken, language).context("whole cue target")?;
    Ok(())
}

#[derive(serde::Serialize)]
struct CueAttempt {
    source: String,
    synthesis_error: Option<String>,
    phoneme: Option<ClipVerification>,
    #[serde(skip_serializing_if = "Option::is_none")]
    whisper: Option<crate::pronunciation_whisper::Verification>,
}

impl CueAttempt {
    fn passed(&self) -> bool {
        self.synthesis_error.is_none()
            && (self.phoneme.is_some() || self.whisper.is_some())
            && self.phoneme.as_ref().is_none_or(ClipVerification::passed)
            && self.whisper.as_ref().is_none_or(|w| w.verdict.passed())
    }

    fn reason(&self) -> &str {
        self.synthesis_error
            .as_deref()
            .or_else(|| {
                self.phoneme
                    .as_ref()
                    .and_then(|p| p.failure_reason.as_deref())
            })
            .unwrap_or_else(|| {
                self.whisper
                    .as_ref()
                    .filter(|w| !w.verdict.passed())
                    .map_or("passed", |w| w.verdict.reasoning.as_str())
            })
    }
}

#[derive(serde::Serialize)]
struct CueOutcome {
    text: String,
    passed: bool,
    target_identity: String,
    target_error: Option<String>,
    attempts: Vec<CueAttempt>,
    timing_error: Option<String>,
    segments: Vec<(String, u32, u32)>,
    #[serde(skip)]
    clip: Option<PronunciationClip>,
}

impl CueOutcome {
    fn reason(&self) -> String {
        self.target_error.clone().unwrap_or_else(|| {
            self.attempts
                .iter()
                .map(|a| format!("{}: {}", a.source, a.reason()))
                .collect::<Vec<_>>()
                .join("; ")
        })
    }
}

const REPLACEMENT_ROUNDS: usize = 2;

/// Encode the cached provider output, never the previously shipped MP3.
/// Return the decoded final container too: both gates must hear what we ship.
/// `None` for silent audio, which rejects the candidate like a synthesis defect.
fn encode_pronunciation_audio(
    source: &[u8],
    padding_ms: u32,
) -> Result<Option<(Vec<u8>, Vec<u8>, f64)>> {
    let dir = tempfile::tempdir()?;
    let input = dir.path().join("source");
    let pcm = dir.path().join("source.wav");
    let normalized = dir.path().join("normalized.wav");
    let encoded = dir.path().join("clip.m4a");
    std::fs::write(&input, source)?;
    let run = |program: &str, args: &[&str]| -> Result<()> {
        let output = std::process::Command::new(program)
            .args(args)
            .output()
            .with_context(|| format!("running {program}"))?;
        anyhow::ensure!(
            output.status.success(),
            "{program}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(())
    };
    run(
        "ffmpeg",
        &[
            "-v",
            "error",
            "-i",
            input.to_str().unwrap(),
            "-ac",
            "1",
            "-ar",
            "48000",
            pcm.to_str().unwrap(),
        ],
    )?;
    let (samples, rate) =
        audio_codec::decode_wav_to_f32(&std::fs::read(&pcm)?).map_err(anyhow::Error::msg)?;
    let threshold = 10_f32.powf(-45.0 / 20.0);
    let Some(first) = samples.iter().position(|s| s.abs() > threshold) else {
        return Ok(None);
    };
    let last = samples.iter().rposition(|s| s.abs() > threshold).unwrap() + 1;
    let padding = padding_ms as usize * rate as usize / 1000;
    let start = first.saturating_sub(padding);
    let end = (last + padding).min(samples.len());
    let filter = format!(
        "atrim=start_sample={start}:end_sample={end},asetpts=PTS-STARTPTS,loudnorm=I=-16:TP=-1.5:LRA=11"
    );
    run(
        "ffmpeg",
        &[
            "-v",
            "error",
            "-i",
            pcm.to_str().unwrap(),
            "-af",
            &filter,
            "-ar",
            "48000",
            "-ac",
            "1",
            normalized.to_str().unwrap(),
        ],
    )?;
    run(
        "fdkaac",
        &[
            "-S",
            "-p",
            "5",
            "-s",
            "2",
            "-b",
            "24000",
            "-G",
            "2",
            "--no-timestamp",
            "--moov-before-mdat",
            "-o",
            encoded.to_str().unwrap(),
            normalized.to_str().unwrap(),
        ],
    )?;
    let final_wav = dir.path().join("final.wav");
    run(
        "ffmpeg",
        &[
            "-v",
            "error",
            "-i",
            encoded.to_str().unwrap(),
            "-ac",
            "1",
            "-ar",
            "48000",
            "-fflags",
            "+bitexact",
            "-flags:a",
            "+bitexact",
            final_wav.to_str().unwrap(),
        ],
    )?;
    let decoded = std::fs::read(final_wav)?;
    // Measure decoder delay against the normalized encoder input rather than
    // assuming a codec delay. Correlate at 2 kHz over +/- 100 ms.
    let (reference, _) =
        audio_codec::decode_wav_to_f32(&std::fs::read(normalized)?).map_err(anyhow::Error::msg)?;
    let (final_samples, _) =
        audio_codec::decode_wav_to_f32(&decoded).map_err(anyhow::Error::msg)?;
    let delay = decoder_delay_ms(&reference, &final_samples);
    Ok(Some((
        std::fs::read(encoded)?,
        decoded,
        delay - start as f64 * 1000.0 / rate as f64,
    )))
}

fn decoder_delay_ms(reference: &[f32], decoded: &[f32]) -> f64 {
    let reference: Vec<_> = reference.iter().step_by(24).copied().collect();
    let decoded: Vec<_> = decoded.iter().step_by(24).copied().collect();
    let score = |lag: i32| {
        let mut dot = 0.0_f64;
        let mut a2 = 0.0_f64;
        let mut b2 = 0.0_f64;
        for (i, &a) in reference.iter().enumerate() {
            if let Some(&b) = usize::try_from(i as i32 + lag)
                .ok()
                .and_then(|j| decoded.get(j))
            {
                dot += f64::from(a) * f64::from(b);
                a2 += f64::from(a).powi(2);
                b2 += f64::from(b).powi(2);
            }
        }
        dot / (a2 * b2).sqrt().max(f64::EPSILON)
    };
    let lag = (-200..=200)
        .max_by(|&a, &b| score(a).total_cmp(&score(b)))
        .unwrap();
    f64::from(lag) / 2.0
}

fn replacement_count(desired: usize, verified: usize, round: usize) -> usize {
    if round >= REPLACEMENT_ROUNDS {
        0
    } else {
        desired.saturating_sub(verified)
    }
}

fn new_examples(
    examples: Vec<language_utils::WordPair>,
    tried: &mut BTreeSet<String>,
) -> Vec<language_utils::WordPair> {
    examples
        .into_iter()
        .filter(|example| tried.insert(example.target.clone()))
        .collect()
}

pub async fn generate_pronunciation_audio(
    pronunciation_data: &mut PronunciationData,
    word_to_pronunciation: &[(String, Pronunciations)],
    course: Course,
    http: &reqwest::Client,
    results_log: &Path,
) -> Result<FxHashMap<String, PronunciationClip>> {
    if pronunciation_data.guides.is_empty() {
        return Ok(FxHashMap::default());
    }
    let language = course.target_language;
    let policy = VerificationPolicy::for_language(language);
    if policy == VerificationPolicy::Transcript {
        eligible_texts(language, "", "")?;
    }
    // Snapshot before filtering. Generation produces one initial candidate set;
    // eligibility and audio retries share this one bound and tried-word set.
    let desired: Vec<_> = pronunciation_data
        .guides
        .iter()
        .map(|g| g.example_words.len().max(1))
        .collect();
    let mut tried: Vec<BTreeSet<String>> = pronunciation_data
        .guides
        .iter()
        .map(|g| g.example_words.iter().map(|e| e.target.clone()).collect())
        .collect();
    let mut total_examples: usize = pronunciation_data
        .guides
        .iter()
        .map(|g| g.example_words.len())
        .sum();
    let pronunciations: HashMap<_, _> = word_to_pronunciation
        .iter()
        .map(|(w, p)| (w.to_lowercase(), p.clone()))
        .collect();
    let mut text_rejections = BTreeMap::new();
    // Saved guides and newly generated ones obey the same pre-TTS gate.
    let inventory = &pronunciation_data.sounds;
    let checked: Vec<_> = futures::stream::iter(&mut pronunciation_data.guides)
        .map(|guide| {
            let pronunciations = &pronunciations;
            async move {
                let mut reasons = crate::pronunciation_patterns::reject_invalid_examples(
                    &mut guide.example_words,
                    &guide.pattern,
                    guide.position,
                    language,
                );
                reasons.extend(
                    crate::pronunciation_patterns::pedagogical_check(
                        course,
                        guide,
                        inventory,
                        pronunciations,
                    )
                    .await?,
                );
                anyhow::Ok(((guide.pattern.clone(), guide.position), reasons))
            }
        })
        .buffered(10)
        .try_collect()
        .await?;
    text_rejections.extend(checked);
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
                .map(|s| s.spoken.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            requests.insert(
                spoken,
                (segments, guide.pattern.clone(), example.target.clone()),
            );
        }
    }
    let store = crate::cache_remote::store();
    let ctx = match policy {
        VerificationPolicy::PhonemeAndTranscript => Some(phoneme_verify::VerifyContext::new(
            http,
            store.clone(),
            &pronunciations,
            language,
        )?),
        VerificationPolicy::Transcript => None,
    };
    let keys = phoneme_verify::TtsKeys::from_env();
    let (language_code, voice_name) = language.google_tts_voice();
    let voice = phoneme_verify::TtsVoice {
        language_code,
        voice_name,
    };
    let style = language_utils::pronunciation_challenge_tts_instructions(language);
    if let Some(parent) = results_log.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut log = std::io::BufWriter::new(
        std::fs::File::create(results_log)
            .with_context(|| format!("creating {}", results_log.display()))?,
    );
    let mut verified = FxHashMap::default();
    let mut failures = BTreeMap::new();
    for round in 0..=REPLACEMENT_ROUNDS {
        let mut generated = futures::stream::iter(requests)
            .map(|(spoken, (segments, pattern, example))| {
                let ctx = &ctx;
                let store = &store;
                let keys = &keys;
                let style = &style;
                async move {
                    let mut outcome = CueOutcome {
                        text: spoken.clone(),
                        passed: false,
                        target_identity: match policy {
                            VerificationPolicy::PhonemeAndTranscript => {
                                phoneme_verify::model_target_identity()
                            }
                            VerificationPolicy::Transcript => "whisper-transcript".into(),
                        },
                        target_error: None,
                        attempts: Vec::new(),
                        timing_error: None,
                        segments: Vec::new(),
                        clip: None,
                    };
                    match eligible_texts(language, &example, &spoken) {
                        Ok(()) => {}
                        Err(error) if phoneme_verify::target_infrastructure_error(&error) => {
                            return Err(error);
                        }
                        Err(error) => {
                            outcome.target_error = Some(format!("{error:#}"));
                            return Ok(outcome);
                        }
                    };
                    let candidates = [
                        TtsSynthesis::Gemini {
                            voice: google_speech::gemini::DEFAULT_VOICE.into(),
                            language: language_code,
                            style: style.clone(),
                            text: spoken.clone(),
                            attempt: 1,
                        },
                        TtsSynthesis::Gemini {
                            voice: google_speech::gemini::DEFAULT_VOICE.into(),
                            language: language_code,
                            style: style.clone(),
                            text: spoken.clone(),
                            attempt: 2,
                        },
                        TtsSynthesis::Google {
                            voice,
                            text: spoken.clone(),
                        },
                    ];
                    for candidate in candidates {
                        let (source, mut synthesis_error) =
                            phoneme_verify::synthesize(http, store, &candidate, &spoken, keys)
                                .await
                                .with_context(|| {
                                    format!("synthesizing {spoken:?} via {}", candidate.label())
                                })?;
                        let encoded = match synthesis_error {
                            // jpn-760 fails at 50/75 ms and passes at 100 ms (YAP-288).
                            // Subprocesses and correlation block, so keep them off the stream.
                            None => {
                                let source = source.clone();
                                tokio::task::spawn_blocking(move || {
                                    encode_pronunciation_audio(&source, 100)
                                })
                                .await??
                            }
                            Some(_) => None,
                        };
                        if encoded.is_none() && synthesis_error.is_none() {
                            synthesis_error = Some("silent audio".into());
                        }
                        let (bytes, decoded, timing_shift) = encoded.unwrap_or_default();
                        let phoneme = if synthesis_error.is_none() {
                            match ctx {
                                Some(ctx) => Some(
                                    phoneme_verify::verify_clip_bytes(
                                        ctx,
                                        "tts-pronunciation",
                                        &spoken,
                                        &candidate.label(),
                                        &decoded,
                                        phoneme_verify::expected_phoneme_variants(
                                            ctx, &spoken, None,
                                        ),
                                    )
                                    .await?,
                                ),
                                None => None,
                            }
                        } else {
                            None
                        };
                        let whisper = if synthesis_error.is_none()
                            && phoneme.as_ref().is_none_or(ClipVerification::passed)
                        {
                            crate::pronunciation_whisper::verify(
                                http, &decoded, language, &pattern, &example,
                            )
                            .await?
                        } else {
                            None
                        };
                        let verification = CueAttempt {
                            source: candidate.label(),
                            synthesis_error,
                            phoneme,
                            whisper,
                        };
                        if verification.passed() {
                            let spoken_segments: Vec<_> =
                                segments.iter().map(|s| s.spoken.as_str()).collect();
                            let timings = match ctx {
                                Some(ctx) => {
                                    phoneme_verify::segment_timings(ctx, &source, &spoken_segments)
                                        .await
                                }
                                None => Ok(Vec::new()),
                            };
                            let timed: Vec<_> = match timings {
                                Ok(timings) => segments
                                    .iter()
                                    .zip(timings)
                                    .map(|(segment, (start_ms, end_ms))| TimedSegment {
                                        text: segment.display.clone(),
                                        start_ms: (f64::from(start_ms) + timing_shift)
                                            .max(0.0)
                                            .round()
                                            as u32,
                                        end_ms: (f64::from(end_ms) + timing_shift).max(0.0).round()
                                            as u32,
                                    })
                                    .collect(),
                                Err(error) => {
                                    outcome.timing_error = Some(format!("{error:#}"));
                                    Vec::new()
                                }
                            };
                            outcome.segments = timed
                                .iter()
                                .map(|s| (s.text.clone(), s.start_ms, s.end_ms))
                                .collect();
                            outcome.clip = Some(PronunciationClip {
                                audio: Audio { bytes },
                                segments: timed,
                            });
                            outcome.passed = true;
                        }
                        outcome.attempts.push(verification);
                        if outcome.passed {
                            break;
                        }
                    }
                    anyhow::Ok(outcome)
                }
            })
            .buffered(16);
        while let Some(outcome) = generated.try_next().await? {
            serde_json::to_writer(&mut log, &outcome)?;
            writeln!(log)?;
            log.flush()?;
            let reason = outcome.reason();
            if let Some(clip) = outcome.clip {
                verified.insert(outcome.text, clip);
            } else {
                failures.insert(outcome.text, reason);
            }
        }
        log.flush()?;
        if round == REPLACEMENT_ROUNDS {
            break;
        }
        // Retain successes and refill every guide's missing examples, not only
        // completely failed guides, up to its original desired count.
        let replacements: Vec<_> =
            futures::stream::iter(pronunciation_data.guides.iter().enumerate())
                .map(|(index, guide)| {
                    let verified = &verified;
                    let failures = &failures;
                    let text_rejections = &text_rejections;
                    let tried = &tried[index];
                    let count = replacement_count(
                        desired[index],
                        guide
                            .example_words
                            .iter()
                            .filter(|e| {
                                verified.contains_key(
                                    &language_utils::pronunciation_challenge_spoken_text(
                                        language,
                                        &guide.pattern,
                                        &e.target,
                                    ),
                                )
                            })
                            .count(),
                        round,
                    );
                    async move {
                        if count == 0 {
                            return Ok((index, Vec::new()));
                        }
                        if let Err(error) = eligibility(language, &guide.pattern, "") {
                            if phoneme_verify::target_infrastructure_error(&error) {
                                return Err(error);
                            }
                            return Ok((index, Vec::new()));
                        }
                        let mut rejected = text_rejections
                            .get(&(guide.pattern.clone(), guide.position))
                            .cloned()
                            .unwrap_or_default();
                        rejected.extend(guide.example_words.iter().filter_map(|e| {
                            let text = language_utils::pronunciation_challenge_spoken_text(
                                language,
                                &guide.pattern,
                                &e.target,
                            );
                            failures
                                .get(&text)
                                .map(|reason| format!("{}: {reason}", e.target))
                        }));
                        let examples = crate::pronunciation_patterns::replacement_examples(
                            course,
                            guide,
                            &rejected,
                            tried,
                            count,
                            round + 1,
                        )
                        .await?;
                        anyhow::Ok((index, examples))
                    }
                })
                .buffered(10)
                .try_collect()
                .await?;
        requests = BTreeMap::new();
        for (index, examples) in replacements {
            let guide = &mut pronunciation_data.guides[index];
            // Mark every proposed target before validation: invalid-script words
            // removed below must never be tried again in another round.
            let mut examples = new_examples(examples, &mut tried[index]);
            total_examples += examples.len();
            let mut rejected = crate::pronunciation_patterns::reject_invalid_examples(
                &mut examples,
                &guide.pattern,
                guide.position,
                language,
            );
            if examples.is_empty() {
                text_rejections
                    .entry((guide.pattern.clone(), guide.position))
                    .or_default()
                    .extend(rejected);
                continue;
            }
            let new_targets: BTreeSet<_> = examples.iter().map(|e| e.target.clone()).collect();
            let mut candidate_guide = guide.clone();
            candidate_guide.example_words.retain(|e| {
                verified.contains_key(&language_utils::pronunciation_challenge_spoken_text(
                    language,
                    &guide.pattern,
                    &e.target,
                ))
            });
            candidate_guide.example_words.extend(examples);
            rejected.extend(
                crate::pronunciation_patterns::pedagogical_check(
                    course,
                    &mut candidate_guide,
                    &pronunciation_data.sounds,
                    &pronunciations,
                )
                .await?,
            );
            guide.description = candidate_guide.description;
            let (mut examples, retained): (Vec<_>, Vec<_>) = candidate_guide
                .example_words
                .into_iter()
                .partition(|e| new_targets.contains(&e.target));
            guide.example_words = retained;
            text_rejections
                .entry((guide.pattern.clone(), guide.position))
                .or_default()
                .extend(rejected);
            let verified_count = guide
                .example_words
                .iter()
                .filter(|e| {
                    verified.contains_key(&language_utils::pronunciation_challenge_spoken_text(
                        language,
                        &guide.pattern,
                        &e.target,
                    ))
                })
                .count();
            examples.truncate(replacement_count(desired[index], verified_count, round));
            for example in examples {
                let segments = language_utils::pronunciation_challenge_segments(
                    language,
                    &guide.pattern,
                    &example.target,
                );
                let spoken = segments
                    .iter()
                    .map(|s| s.spoken.as_str())
                    .collect::<Vec<_>>()
                    .join(" ");
                if !verified.contains_key(&spoken) && !failures.contains_key(&spoken) {
                    requests.insert(
                        spoken,
                        (segments, guide.pattern.clone(), example.target.clone()),
                    );
                }
                if !guide
                    .example_words
                    .iter()
                    .any(|e| e.target == example.target)
                {
                    guide.example_words.push(example);
                }
            }
        }
    }
    for ((pattern, position), reasons) in &text_rejections {
        if !reasons.is_empty() {
            serde_json::to_writer(
                &mut log,
                &serde_json::json!({"kind": "text_rejections", "pattern": pattern, "position": position, "reasons": reasons}),
            )?;
            writeln!(log)?;
        }
    }
    let before = pronunciation_data.guides.len();
    prune_unverified(
        pronunciation_data,
        &verified,
        &failures,
        &text_rejections,
        language,
    );
    let dropped_examples = total_examples
        - pronunciation_data
            .guides
            .iter()
            .map(|g| g.example_words.len())
            .sum::<usize>();
    println!(
        "{language:?}: {} verified pronunciation clips; dropped {dropped_examples} examples and {}/{} guides (reasons in {})",
        verified.len(),
        before - pronunciation_data.guides.len(),
        before,
        results_log.display()
    );
    Ok(verified)
}

fn prune_unverified(
    pronunciation_data: &mut PronunciationData,
    verified: &FxHashMap<String, PronunciationClip>,
    failures: &BTreeMap<String, String>,
    text_rejections: &BTreeMap<(String, PatternPosition), Vec<String>>,
    language: Language,
) {
    pronunciation_data.guides.retain_mut(|guide| {
        let mut reasons = text_rejections
            .get(&(guide.pattern.clone(), guide.position))
            .cloned()
            .unwrap_or_default();
        guide.example_words.retain(|example| {
            let text = language_utils::pronunciation_challenge_spoken_text(
                language,
                &guide.pattern,
                &example.target,
            );
            if verified.contains_key(&text) {
                true
            } else {
                reasons.push(format!(
                    "{}: {}",
                    example.target,
                    failures
                        .get(&text)
                        .map(String::as_str)
                        .unwrap_or("no audio target")
                ));
                false
            }
        });
        if guide.example_words.is_empty() {
            eprintln!(
                "{language:?}: dropped pronunciation guide {:?} ({:?}): {}",
                guide.pattern,
                guide.position,
                if reasons.is_empty() {
                    "no text-valid examples after bounded replacement rounds".into()
                } else {
                    reasons.join("; ")
                }
            );
            false
        } else {
            true
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measures_decoder_delay_in_both_directions() {
        let signal: Vec<f32> = (0..4800)
            .map(|i| ((i * 7919 % 1009) as f32 / 1009.0) - 0.5)
            .collect();
        let mut delayed = vec![0.0; 960];
        delayed.extend_from_slice(&signal);
        assert_eq!(decoder_delay_ms(&signal, &delayed), 20.0);
        assert_eq!(decoder_delay_ms(&delayed, &signal), -20.0);
    }

    fn example(target: &str) -> language_utils::WordPair {
        language_utils::WordPair {
            target: target.into(),
            native: "example".into(),
            position: language_utils::SoundPosition::Beginning,
            cultural_context: String::new(),
        }
    }

    #[test]
    fn replacements_refill_partial_successes_and_stop_at_original_count_or_bound() {
        assert_eq!(replacement_count(4, 1, 0), 3);
        assert_eq!(replacement_count(4, 3, 1), 1);
        assert_eq!(replacement_count(4, 4, 0), 0);
        assert_eq!(replacement_count(4, 5, 1), 0);
        assert_eq!(replacement_count(4, 1, REPLACEMENT_ROUNDS), 0);
    }

    #[test]
    fn tried_words_include_successes_invalid_text_and_duplicates() {
        let mut tried = BTreeSet::from(["Jaipur".into(), "जलेबी".into()]);
        let mut fresh = new_examples(
            vec![
                example("Jaipur"),
                example("जलेबी"),
                example("मंत्र"),
                example("mantra"),
                example("mantra"),
            ],
            &mut tried,
        );
        assert_eq!(
            fresh.iter().map(|e| e.target.as_str()).collect::<Vec<_>>(),
            ["मंत्र", "mantra"]
        );
        crate::pronunciation_patterns::reject_invalid_examples(
            &mut fresh,
            "म",
            PatternPosition::Beginning,
            Language::Hindi,
        );
        assert_eq!(fresh.len(), 1);
        let next = new_examples(
            vec![example("mantra"), example("मंत्र"), example("मेला")],
            &mut tried,
        );
        assert_eq!(
            next.iter().map(|e| e.target.as_str()).collect::<Vec<_>>(),
            ["मेला"]
        );
    }

    #[test]
    fn guides_without_any_verified_example_are_dropped() {
        let guide = serde_json::from_value(serde_json::json!({
            "pattern": "a", "position": "Anywhere", "description": "A sound",
            "familiarity": "LikelyAlreadyKnows", "difficulty": "Easy",
            "example_words": [{"target": "ami", "native": "friend", "position": "Beginning", "cultural_context": "A friend"}]
        })).unwrap();
        let mut data = PronunciationData {
            sounds: Vec::new(),
            guides: vec![guide],
            pattern_frequencies: Vec::new(),
        };
        let mut verified = FxHashMap::default();
        let spoken =
            language_utils::pronunciation_challenge_spoken_text(Language::French, "a", "ami");
        verified.insert(
            spoken,
            PronunciationClip {
                audio: Audio { bytes: vec![1] },
                segments: Vec::new(),
            },
        );
        prune_unverified(
            &mut data,
            &verified,
            &BTreeMap::new(),
            &BTreeMap::new(),
            Language::French,
        );
        assert_eq!(data.guides.len(), 1);
        prune_unverified(
            &mut data,
            &FxHashMap::default(),
            &BTreeMap::new(),
            &BTreeMap::new(),
            Language::French,
        );
        assert!(data.guides.is_empty());
    }

    #[test]
    fn european_portuguese_uses_transcripts_without_brazilian_g2p() {
        assert_eq!(
            VerificationPolicy::for_language(Language::PortugueseEuropean),
            VerificationPolicy::Transcript
        );
        assert!(Language::PortugueseEuropean.g2p_lang().is_none());
        assert!(eligibility(Language::PortugueseEuropean, "ão", "cão").is_ok());
        assert_eq!(
            VerificationPolicy::for_language(Language::French),
            VerificationPolicy::PhonemeAndTranscript
        );
        assert!(
            !CueAttempt {
                source: "test".into(),
                synthesis_error: None,
                phoneme: None,
                whisper: None
            }
            .passed()
        );
    }

    #[test]
    fn g2p_eligibility_rejects_bare_jamo_and_terminal_small_tsu() {
        let jamo = phoneme_verify::complete_target("ㅋㅋㅋ", Language::Korean).unwrap_err();
        assert!(format!("{jamo:#}").contains("korean_jamo"));
        let tsu = eligibility(Language::Japanese, "っ", "あっ").unwrap_err();
        assert!(format!("{tsu:#}").contains("japanese_terminal_sokuon"));
        assert!(eligibility(Language::Japanese, "っ", "はっぱ").is_ok());
    }
}
