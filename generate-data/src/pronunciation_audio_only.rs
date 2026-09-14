//! Generate/verify only saved pronunciation cues. Audio is persisted by the
//! existing synthesis cache and verdicts/timings by the usual verification log.
//! This mode does not read, assemble, migrate, or update a language pack.

use anyhow::{Context, Result};
use language_utils::{
    COURSES, Course, PronunciationData, PronunciationGuideThoughts, Pronunciations,
};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader};
use std::path::Path;

type PronunciationMap = Vec<(String, Pronunciations)>;

fn read_jsonl<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
    BufReader::new(
        std::fs::File::open(path).with_context(|| format!("opening {}", path.display()))?,
    )
    .lines()
    .enumerate()
    .map(|(index, line)| {
        serde_json::from_str(&line?)
            .with_context(|| format!("parsing {} line {}", path.display(), index + 1))
    })
    .collect()
}

fn read_tsv(path: &Path) -> Result<BTreeMap<String, Vec<String>>> {
    let mut result: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (index, line) in BufReader::new(
        std::fs::File::open(path).with_context(|| format!("opening {}", path.display()))?,
    )
    .lines()
    .enumerate()
    {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let (word, ipa) = line.split_once('\t').with_context(|| {
            format!(
                "parsing {} line {}: expected word<TAB>IPA",
                path.display(),
                index + 1
            )
        })?;
        anyhow::ensure!(
            !word.trim().is_empty() && !ipa.trim().is_empty(),
            "empty word/IPA in {} line {}",
            path.display(),
            index + 1
        );
        result
            .entry(word.trim().to_lowercase())
            .or_default()
            .push(ipa.trim().to_owned());
    }
    Ok(result)
}

fn tsv_pronunciations(source: &Path) -> Result<PronunciationMap> {
    let wiki = read_tsv(&source.join("pronunciations.tsv"))?;
    let extra = read_tsv(&source.join("extra_pronunciations.tsv"))?;
    let words: BTreeSet<_> = wiki.keys().chain(extra.keys()).collect();
    Ok(words
        .into_iter()
        .map(|word| {
            let variants: BTreeSet<_> = wiki
                .get(word)
                .into_iter()
                .flatten()
                .chain(extra.get(word).into_iter().flatten())
                .cloned()
                .collect();
            // Match normal generation's hand-curated override; otherwise choose a
            // deterministic main rather than invoking paid canonical selection.
            // Every other pronunciation remains accepted by the verifier. The
            // ordering can differ from LLM-selected mains at its combination cap.
            let main = extra
                .get(word)
                .and_then(|p| p.first())
                .or_else(|| variants.first())
                .expect("each TSV word has a pronunciation")
                .clone();
            let others = extra
                .get(word)
                .into_iter()
                .flatten()
                .chain(variants.iter())
                .filter(|ipa| **ipa != main);
            let mut seen = BTreeSet::new();
            let others = others
                .filter(|ipa| seen.insert((*ipa).clone()))
                .cloned()
                .collect();
            (word.clone(), Pronunciations { main, others })
        })
        .collect())
}

fn load_inputs(
    out: &Path,
    sources: &Path,
    course: &Course,
) -> Result<(PronunciationData, PronunciationMap)> {
    let dir = out.join(format!(
        "{}_for_{}",
        course.target_language.code(),
        course.native_language.code()
    ));
    let guides = read_jsonl::<(String, PronunciationGuideThoughts)>(
        &dir.join("pronunciation_guides.jsonl"),
    )?
    .into_iter()
    .map(|(_, guide)| guide.into())
    .collect();
    let pronunciation_path = out
        .join(course.target_language.code())
        .join("word_to_pronunciation.jsonl");
    let pronunciations = match std::fs::metadata(&pronunciation_path) {
        Ok(_) => read_jsonl(&pronunciation_path)?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!(
                "{} missing: using local TSV pronunciation variants (curated main first, otherwise lexicographic; no LLM selection)",
                pronunciation_path.display()
            );
            tsv_pronunciations(&sources.join(course.target_language.code()))?
        }
        Err(e) => {
            return Err(e).with_context(|| format!("reading {}", pronunciation_path.display()));
        }
    };
    // generate_pronunciation_audio consumes only guides; sounds/frequencies
    // are full-pack concerns and deliberately are not loaded or generated.
    Ok((
        PronunciationData {
            sounds: vec![],
            guides,
            pattern_frequencies: vec![],
        },
        pronunciations,
    ))
}

pub async fn run(out: &Path, languages: &BTreeSet<String>) -> Result<()> {
    let started = std::time::Instant::now();
    let before = google_tts::request_counts();
    let result = run_inner(out, languages).await;
    let after = google_tts::request_counts();
    // Always emitted, including probe failures and cache misses. Requests are
    // counted at HTTP send sites, including synthesis retries/fallbacks.
    println!(
        "Pronunciation audio synthesis HTTP requests: Gemini={}, Chirp3={} (wall time {:.1}s)",
        after.gemini - before.gemini,
        after.chirp3 - before.chirp3,
        started.elapsed().as_secs_f64()
    );
    result
}

async fn run_inner(out: &Path, languages: &BTreeSet<String>) -> Result<()> {
    // Preflight every selected course before any paid work.
    let courses: Vec<_> = COURSES
        .iter()
        .filter(|course| languages.is_empty() || languages.contains(course.target_language.code()))
        .map(|course| {
            Ok((
                course,
                load_inputs(out, Path::new("generate-data/data"), course)?,
            ))
        })
        .collect::<Result<_>>()?;
    let http = reqwest::Client::new();
    for (course, (mut data, pronunciations)) in courses {
        println!(
            "Pronunciation audio only: {} -> {} (pack not updated)",
            course.native_language, course.target_language
        );
        let clips = crate::pronunciation_audio::generate_pronunciation_audio(
            &mut data,
            &pronunciations,
            course.target_language,
            &http,
            &out.join(course.target_language.code())
                .join("pronunciation_audio_verification.jsonl"),
        )
        .await?;
        println!(
            "Verified {} pronunciation clips; synthesis audio cached and verification log written; pack not updated",
            clips.len()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn saved_guide() -> PronunciationGuideThoughts {
        PronunciationGuideThoughts {
            thoughts: "generation rationale".into(),
            pattern: "a".into(),
            position: language_utils::PatternPosition::Anywhere,
            description: "a sound".into(),
            familiarity: language_utils::PronunciationFamiliarity::ProbablyDoesNotKnow,
            difficulty: language_utils::PronunciationDifficulty::Easy,
            example_words: vec![],
        }
    }

    #[test]
    fn isolated_inputs_need_no_pack_corpus_sounds_or_frequencies() {
        let root = tempfile::tempdir().unwrap();
        let course = &COURSES[0];
        let dir = root.path().join(format!(
            "{}_for_{}",
            course.target_language.code(),
            course.native_language.code()
        ));
        std::fs::create_dir(&dir).unwrap();
        let target = root.path().join(course.target_language.code());
        std::fs::create_dir(&target).unwrap();
        let guide = saved_guide();
        let path = dir.join("pronunciation_guides.jsonl");
        let saved = format!("{}\n", serde_json::to_string(&("a", &guide)).unwrap());
        std::fs::write(&path, &saved).unwrap();
        std::fs::write(target.join("word_to_pronunciation.jsonl"), "").unwrap();
        let (data, pronunciations) =
            load_inputs(root.path(), &root.path().join("no-sources"), course).unwrap();
        assert_eq!(data.guides, vec![guide.into()]);
        assert!(
            data.sounds.is_empty()
                && data.pattern_frequencies.is_empty()
                && pronunciations.is_empty()
        );
        assert_eq!(std::fs::read_to_string(&path).unwrap(), saved);
        std::fs::write(&path, "not json\n").unwrap();
        assert!(
            format!(
                "{:#}",
                load_inputs(root.path(), root.path(), course).unwrap_err()
            )
            .contains("line 1")
        );
    }

    #[test]
    fn guide_loader_rejects_control_characters_in_cue_text() {
        let root = tempfile::tempdir().unwrap();
        let course = Course {
            native_language: language_utils::Language::English,
            target_language: language_utils::Language::German,
        };
        let dir = root.path().join("deu_for_eng");
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("pronunciation_guides.jsonl");
        let mut guide = saved_guide();
        guide.pattern = "k".into();
        guide.example_words = vec![language_utils::WordPair {
            target: "Köln".into(),
            native: "Cologne (the German city on the Rhine)".into(),
            position: language_utils::SoundPosition::Beginning,
            cultural_context: "The German name for Cologne.".into(),
        }];
        let valid = serde_json::to_value(&guide).unwrap();
        for (field, bad_text, codepoint) in [
            ("target", format!("K{}f6ln", char::from(0)), "U+0000"),
            ("pattern", format!("k{}", char::from(0)), "U+0000"),
            ("target", format!("K{}öln", char::from(9)), "U+0009"),
            ("pattern", format!("k{}", char::from(10)), "U+000A"),
            ("target", format!("K{}öln", char::from(127)), "U+007F"),
            ("pattern", format!("k{}", char::from(133)), "U+0085"),
        ] {
            let mut malformed = valid.clone();
            if field == "pattern" {
                malformed["pattern"] = bad_text.into();
            } else {
                malformed["example_words"][0]["target"] = bad_text.into();
            }
            std::fs::write(&path, format!("{}\n", serde_json::json!(["k", malformed]))).unwrap();
            let error = format!(
                "{:#}",
                load_inputs(root.path(), root.path(), &course).unwrap_err()
            );
            assert!(error.contains(&path.display().to_string()), "{error}");
            assert!(error.contains("line 1"), "{error}");
            assert!(
                error.contains("pronunciation cue text contains control character"),
                "{error}"
            );
            assert!(error.contains(codepoint), "{error}");
            // The pack-facing guide type has the same validation as the
            // saved/generated guide-with-thoughts type used by the loader.
            assert!(
                serde_json::from_value::<language_utils::PronunciationGuide>(malformed)
                    .unwrap_err()
                    .to_string()
                    .contains(codepoint)
            );
        }

        // Valid cues are not normalized or rewritten at the JSON boundary.
        for (pattern, target) in [("k", "Köln"), ("á", "à la carte"), ("क्ष", "क्षमा")]
        {
            guide.pattern = pattern.into();
            guide.example_words[0].target = target.into();
            let json = serde_json::to_string(&guide).unwrap();
            let loaded: PronunciationGuideThoughts = serde_json::from_str(&json).unwrap();
            assert_eq!(loaded, guide);
            let loaded: language_utils::PronunciationGuide = serde_json::from_str(&json).unwrap();
            assert_eq!(loaded, guide.clone().into());
        }
        assert_eq!(
            language_utils::pronunciation_challenge_spoken_text(
                course.target_language,
                "k",
                "Köln"
            ),
            "k wie in Köln"
        );
    }

    #[test]
    fn tsv_fallback_preserves_all_variants_and_curated_priority() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join("pronunciations.tsv"),
            "Word\tz\nword\ta\nword\tz\nother\ty\nother\tb\n",
        )
        .unwrap();
        std::fs::write(
            root.path().join("extra_pronunciations.tsv"),
            "WORD\tmanual\nword\tsecond\nextra-only\tx\n",
        )
        .unwrap();
        let restored: BTreeMap<_, _> = tsv_pronunciations(root.path())
            .unwrap()
            .into_iter()
            .collect();
        assert_eq!(restored["word"].main, "manual");
        assert_eq!(restored["word"].others, ["second", "a", "z"]);
        assert_eq!(restored["other"].main, "b");
        assert_eq!(restored["other"].others, ["y"]);
        assert_eq!(restored["extra-only"].main, "x");
        std::fs::write(root.path().join("extra_pronunciations.tsv"), "malformed").unwrap();
        assert!(tsv_pronunciations(root.path()).is_err());
    }

    #[test]
    fn missing_saved_guides_fail_before_any_synthesis() {
        let root = tempfile::tempdir().unwrap();
        assert!(load_inputs(root.path(), root.path(), &COURSES[0]).is_err());
    }
}
