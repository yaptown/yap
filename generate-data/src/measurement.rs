//! Write-isolated cache measurement. Never enters the pack-producing main path.
use anyhow::{Context, Result};
use language_utils::COURSES;
use serde_json::json;
use std::collections::BTreeSet;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

static ALLOW_MODAL: AtomicBool = AtomicBool::new(false);

#[derive(Debug, serde::Serialize)]
pub(crate) struct DependencyBarrier {
    pub component: &'static str,
    pub pending: usize,
    pub exhausted: usize,
    pub input: Option<std::path::PathBuf>,
}
impl std::fmt::Display for DependencyBarrier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "measurement dependency barrier: {} pending {} requests ({} exhausted inputs excluded)",
            self.pending, self.component, self.exhausted
        )?;
        if let Some(path) = &self.input {
            write!(f, " at {}", path.display())?;
        }
        Ok(())
    }
}
impl std::error::Error for DependencyBarrier {}

/// Only the explicitly opted-in Lexide and token-embedding prerequisites may
/// use Modal. All tysm providers remain guarded regardless of this setting.
pub(crate) fn modal_allowed() -> bool {
    tysm::measurement::enabled() && ALLOW_MODAL.load(Ordering::Relaxed)
}

pub(crate) fn is_cache_miss(error: &anyhow::Error) -> bool {
    error.chain().any(|error| {
        matches!(
            error.downcast_ref::<tysm::chat_completions::ChatError>(),
            Some(tysm::chat_completions::ChatError::CacheMiss)
        ) || matches!(
            error.downcast_ref::<tysm::chat_completions::IndividualChatError>(),
            Some(tysm::chat_completions::IndividualChatError::CacheMiss)
        )
    })
}

fn report(file: &mut std::fs::File, stage: &str, value: serde_json::Value) -> Result<()> {
    serde_json::to_writer(&mut *file, &json!({"stage":stage,"result":value}))?;
    writeln!(file)?;
    file.sync_data()?;
    Ok(())
}
fn result<T>(
    file: &mut std::fs::File,
    stage: &str,
    value: Result<T>,
    before: u64,
) -> Result<Option<T>> {
    let misses = tysm::measurement::misses() - before;
    match value {
        Ok(value) if misses == 0 => {
            report(
                file,
                stage,
                json!({"status":"cached", "tysm_cache_misses":0}),
            )?;
            Ok(Some(value))
        }
        Ok(_) => {
            report(
                file,
                stage,
                json!({"status":"dependency_barrier", "tysm_cache_misses":misses}),
            )?;
            Ok(None)
        }
        Err(error) => {
            report(
                file,
                stage,
                json!({"status":"dependency_barrier", "tysm_cache_misses":misses,"dependency":error.downcast_ref::<DependencyBarrier>(),"error":format!("{error:#}")}),
            )?;
            Ok(None)
        }
    }
}

fn copy_preparation_cache(
    destination: &Path,
    corpus: &Path,
    target: &Path,
    reuse: Option<&Path>,
) -> Result<()> {
    let mut inputs = std::collections::BTreeMap::new();
    for name in [
        "target_language_sentences_tokenization.jsonl",
        "restricted_sentences_tokenization.jsonl",
        "target_language_multiword_terms_tokenization.jsonl",
        "target_language_sentences_tokenization.jsonl.failures.jsonl",
        "restricted_sentences_tokenization.jsonl.failures.jsonl",
        "target_language_multiword_terms_tokenization.jsonl.failures.jsonl",
        "target_language_multiword_terms.txt",
        "wiktionary_multiword_terms.txt",
        "multiword_alt_forms.jsonl",
    ] {
        let source = reuse
            .map(|path| path.join(name))
            .filter(|path| path.is_file())
            .unwrap_or_else(|| corpus.join(name));
        if source.is_file() {
            std::fs::copy(&source, destination.join(name))?;
            inputs.insert(name, source.canonicalize()?);
        }
    }
    // Stable prompt samples belong to the target dialect, not its shared NLP corpus.
    for name in ["slot_match_samples.jsonl", "slot_hint_examples.jsonl"] {
        let source = target.join(name);
        if source.is_file() {
            std::fs::copy(&source, destination.join(name))?;
            inputs.insert(name, source.canonicalize()?);
        }
    }
    std::fs::write(
        destination.join("measurement-cache-inputs.json"),
        serde_json::to_vec_pretty(&inputs)?,
    )?;
    Ok(())
}

pub async fn run(
    directory: &Path,
    languages: &BTreeSet<String>,
    allow_modal: bool,
    cache_from: Option<&Path>,
) -> Result<()> {
    anyhow::ensure!(
        !languages.is_empty(),
        "measurement requires explicit language codes"
    );
    if let Some(source) = cache_from {
        anyhow::ensure!(
            languages.len() == 1,
            "--measure-cache-from requires one explicit target language"
        );
        anyhow::ensure!(
            source
                .join("target_language_sentences_tokenization.jsonl")
                .is_file(),
            "--measure-cache-from must name a real prior segmentation preparation directory: {}",
            source.display()
        );
    }
    std::fs::create_dir(directory)
        .context("measurement output directory must not already exist")?;
    tysm::measurement::start(&directory.join("requests.jsonl"))?;
    crate::set_cache_only(true);
    ALLOW_MODAL.store(allow_modal, Ordering::Relaxed);
    let mut stages = std::fs::File::create(directory.join("stages.jsonl"))?;
    report(
        &mut stages,
        "inference-policy",
        json!({"tysm_inference_allowed":false,"modal_tokenization_and_embeddings_allowed":allow_modal}),
    )?;
    let mut preparations = std::collections::BTreeMap::<&str, std::path::PathBuf>::new();
    for course in COURSES
        .iter()
        .filter(|course| languages.contains(course.target_language.code()))
    {
        let lang = course.target_language;
        // This branch is independent of corpus/translation/readings failures.
        if lang.is_phonographic() {
            let stage = format!("{}:sounds", lang.code());
            tysm::measurement::stage(&stage);
            let before = tysm::measurement::misses();
            let sounds = crate::pronunciation_patterns::generate_language_sounds(lang).await;
            if let Some(sounds) = result(&mut stages, &stage, sounds, before)? {
                let stage = format!("{}:guides:{}", lang.code(), course.native_language.code());
                tysm::measurement::stage(&stage);
                let before = tysm::measurement::misses();
                let guides =
                    crate::pronunciation_patterns::generate_pronunciation_guides(*course, &sounds)
                        .await;
                result(&mut stages, &stage, guides, before)?;
            } else {
                report(
                    &mut stages,
                    &format!("{}:guides:{}", lang.code(), course.native_language.code()),
                    json!({"blocked_by":"sounds"}),
                )?;
            }
        }
        let stage = format!("{}:source-loading", lang.code());
        tysm::measurement::stage(&stage);
        let before = tysm::measurement::misses();
        let sources = crate::target_sentences::get_target_sentences(*course).await;
        let Some(mut sources) = result(&mut stages, &stage, sources, before)? else {
            report(
                &mut stages,
                lang.code(),
                json!({"blocked_by":"source-loading", "downstream":["translation","segmentation","readings-B","readings-C","sense-assignment","dictionary","morphology","etymology","morphemes","sense-definitions","sense-redundancy","pronunciations","homophone-practice"]}),
            )?;
            continue;
        };
        // Translation has its own text-only cache, independent of segmentation.
        let stage = format!(
            "{}:translation:{}",
            lang.code(),
            course.native_language.code()
        );
        tysm::measurement::stage(&stage);
        let translator = crate::translate::Translator::new(
            lang,
            course.native_language,
            crate::cache_remote::store(),
            crate::translate::TranslationBackend::OpenAi {
                model: "gpt-6-luna".into(),
            },
        )
        .await?;
        let texts = sources
            .app_sentences
            .iter()
            .map(|(text, _, _)| text.clone())
            .collect::<Vec<_>>();
        report(&mut stages, &stage, translator.measure(&texts).await?)?;
        let scratch = if let Some(path) = preparations.get(lang.code()) {
            path.clone()
        } else {
            let path = tempfile::Builder::new()
                .prefix("segmentation-")
                .tempdir_in(directory)?
                .keep();
            copy_preparation_cache(
                &path,
                &Path::new("out").join(lang.corpus_code()),
                &Path::new("out").join(lang.code()),
                cache_from,
            )?;
            preparations.insert(lang.code(), path.clone());
            path
        };
        let stage = format!("{}:segmentation", lang.code());
        tysm::measurement::stage(&stage);
        let before = tysm::measurement::misses();
        let segmented = crate::pipeline::segment_corpus(course, &sources, Some(&scratch)).await;
        let segmented = match segmented {
            Ok(segmented) if segmented.measurement.readings_ready() => {
                report(
                    &mut stages,
                    &stage,
                    json!({
                        "status":if segmented.measurement.overlays_ready() {"cached"} else {"partial_overlays"},
                        "dependencies":segmented.measurement,
                        "readings_inputs_complete":true,
                    }),
                )?;
                Some(segmented)
            }
            other => result(&mut stages, &stage, other, before)?,
        };
        let Some(mut segmented) = segmented else {
            report(
                &mut stages,
                lang.code(),
                json!({"blocked_by":"segmentation", "downstream":["readings-B","readings-C","sense-assignment","dictionary","morphology","etymology","morphemes","sense-definitions","sense-redundancy","pronunciations","homophone-practice"]}),
            )?;
            continue;
        };
        let stage = format!("{}:token-embeddings", lang.code());
        tysm::measurement::stage(&stage);
        let before = tysm::measurement::misses();
        let embeddings = crate::token_embeddings::ensure_token_embeddings(
            lang,
            segmented
                .nlp_sentences
                .iter()
                .chain(segmented.restricted_nlp_sentences.iter()),
            &segmented.interners,
            &crate::cache_remote::store(),
        )
        .await;
        if result(&mut stages, &stage, embeddings, before)?.is_none() {
            report(
                &mut stages,
                lang.code(),
                json!({"blocked_by":"token-embeddings", "downstream":["readings-B","readings-C","sense-assignment","dictionary","phrasebook","morphology","etymology","morphemes","sense-definitions","sense-redundancy"]}),
            )?;
            continue;
        }
        let stage = format!("{}:readings", lang.code());
        tysm::measurement::stage(&stage);
        let before = tysm::measurement::misses();
        let readings = crate::readings::CorpusReadings::generate(
            lang,
            &segmented.nlp_sentences,
            &segmented.restricted_nlp_sentences,
            &segmented.interners,
            &crate::cache_remote::store(),
            None,
        )
        .await;
        let Some(readings) = result(&mut stages, &stage, readings, before)? else {
            report(
                &mut stages,
                lang.code(),
                json!({"blocked_by":"readings", "downstream":["readings-C-if-B-missing","sense-assignment","dictionary","phrasebook","morphology","etymology","morphemes","sense-definitions","sense-redundancy","pronunciations","homophone-practice"]}),
            )?;
            continue;
        };
        readings.retain_corpus(
            &mut sources,
            &mut segmented.nlp_sentences,
            &mut segmented.restricted_nlp_sentences,
        );
        later_stages(*course, segmented, &readings, &mut stages).await?;
    }
    Ok(())
}

async fn later_stages(
    course: language_utils::Course,
    mut segmented: crate::pipeline::SegmentedCorpus,
    readings: &crate::readings::CorpusReadings,
    stages: &mut std::fs::File,
) -> Result<()> {
    use language_utils::{Atom, Gram, SentenceGram};
    use std::collections::{BTreeMap, HashSet};
    let lang = course.target_language;
    let overlays_complete = segmented.measurement.overlays_ready();
    macro_rules! measure {
        ($name:expr, $future:expr) => {{
            let stage = format!(
                "{}:{}:{}",
                lang.code(),
                $name,
                course.native_language.code()
            );
            tysm::measurement::stage(&stage);
            let before = tysm::measurement::misses();
            let value = $future.await;
            result(stages, &stage, value, before)?
        }};
    }
    let inventories = measure!(
        "sense-assignment",
        crate::usage_discovery::assign_senses(
            lang,
            &mut segmented.nlp_sentences,
            &mut segmented.restricted_nlp_sentences,
            &segmented.interners,
            &crate::cache_remote::store(),
            readings,
        )
    );
    let Some(inventories) = inventories else {
        report(
            stages,
            &format!(
                "{}:downstream:{}",
                lang.code(),
                course.native_language.code()
            ),
            json!({"blocked_by":"sense-assignment"}),
        )?;
        return Ok(());
    };
    let initial = crate::pipeline::initial_gram_frequencies(&segmented.gram_vocabulary);
    let allowed: HashSet<_> = initial
        .iter()
        .filter(|entry| entry.count > 3)
        .map(|entry| entry.gram.gram.clone())
        .collect();
    let encoded: Vec<_> = segmented
        .nlp_sentences
        .iter()
        .map(|(text, info)| (text.clone(), info.sentence.clone()))
        .collect();
    let sentences: Vec<_> = crate::pipeline::sentence_grams(
        &encoded,
        &segmented.nlp_sentences,
        &segmented.gram_vocabulary,
    )
    .into_iter()
    .filter(|(_, sentence)| {
        sentence.grams.iter().all(|gram| match gram {
            SentenceGram::Learnable(gram) => allowed.contains(&gram.gram),
            SentenceGram::Obvious(_) => true,
        })
    })
    .collect();
    let frequencies =
        crate::frequencies::compute_gram_frequencies(&sentences, &segmented.gram_vocabulary);
    let filtered: Vec<_> = frequencies
        .iter()
        .filter(|entry| entry.count > 3 && (overlays_complete || entry.gram.gram.len() == 1))
        .cloned()
        .collect();
    if !overlays_complete {
        report(
            stages,
            &format!(
                "{}:overlay-dependent:{}",
                lang.code(),
                course.native_language.code()
            ),
            json!({"blocked_by":"slot-overlays","pending_slot_requests":segmented.measurement.overlay_misses,
                "downstream":["phrasebook","phrase-sense-definitions","sense-redundancy"],"unigram_inputs_complete":true}),
        )?;
    }
    let corpus = crate::corpus_samples::CorpusIndex::new(&sentences);
    let native = Path::new("out").join(format!(
        "{}_for_{}",
        lang.code(),
        course.native_language.code()
    ));
    let data = Path::new("generate-data/data").join(lang.corpus_code());
    let banned = crate::pipeline::load_banned_words(&course)?;
    let frequent_words: HashSet<_> = frequencies
        .iter()
        .filter_map(|entry| entry.gram.gram.heteronym())
        .filter(|h| !banned.contains(*h))
        .map(|h| h.word.clone())
        .collect();
    // These prompts depend on the retained encoded words, not on definitions,
    // morphology, phrasebook answers, or slot-only multi-atom overlays.
    let pronunciations = measure!("pronunciations", async {
        let candidates = crate::pronunciations::load_candidates(&data, &frequent_words)?;
        let mut selected: BTreeMap<_, _> =
            crate::pronunciations::select_common_pronunciations(course, candidates.automatic)
                .await?
                .into_iter()
                .collect();
        selected.extend(candidates.manual);
        Ok::<_, anyhow::Error>(selected)
    });
    let existing_homophones = Path::new("out")
        .join(lang.code())
        .join("homophones_top_1000.jsonl");
    let homophones_from_cache = if existing_homophones.is_file() {
        // Production reuses this exact file without consulting new frequencies.
        let scratch = tempfile::tempdir()?;
        std::fs::copy(
            &existing_homophones,
            scratch.path().join("homophones_top_1000.jsonl"),
        )?;
        let homophones = crate::disambiguation_practice::generate_homophones(
            course,
            scratch.path(),
            &frequencies,
            1000,
        )?;
        measure!(
            "homophone-practice",
            crate::disambiguation_practice::generate_homophone_practice(
                course,
                &homophones,
                scratch.path()
            )
        );
        true
    } else {
        false
    };
    measure!(
        "proper-noun-definitions",
        crate::proper_noun_definitions::generate_proper_noun_definitions(
            course,
            &segmented.nlp_sentences,
            &segmented.interners
        )
    );
    let mut words =
        crate::corpus_samples::SampleCache::load(native.join("heteronym_sentences.jsonl"))?;
    let dictionary = measure!(
        "dictionary",
        crate::dict::create_gram_dictionary(course, &filtered, &corpus, &mut words)
    );
    let mut phrases = crate::corpus_samples::SampleCache::<Gram<String>>::load(
        native.join("gram_sentences.jsonl"),
    )?;
    let phrasebook = if overlays_complete {
        measure!(
            "phrasebook",
            crate::dict::create_gram_phrasebook(course, &filtered, &corpus, &mut phrases)
        )
    } else {
        None
    };
    let senses = measure!(
        if overlays_complete {
            "sense-definitions"
        } else {
            "word-sense-definitions"
        },
        crate::dict::create_sense_definitions(course, &filtered, &inventories, &corpus)
    );
    let morphology = measure!(
        "morphology",
        crate::morphology_analysis::create_morphology(lang, &filtered)
    );
    let golden = data.join("golden_morphemes.jsonl");
    let etymology = if !golden.exists() {
        Some(BTreeMap::new())
    } else if let Some(morphology) = &morphology {
        let mut per_word: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (heteronym, forms) in morphology {
            let forms = language_utils::features::Morphology::leipzig_many(forms);
            let summary = if forms.is_empty() {
                format!("{:?} {}", heteronym.pos, heteronym.lemma)
            } else {
                format!("{:?} {} [{}]", heteronym.pos, heteronym.lemma, forms)
            };
            per_word
                .entry(heteronym.word.clone())
                .or_default()
                .push(summary);
        }
        let descriptions = per_word
            .into_iter()
            .map(|(word, values)| (word, values.join("; ")))
            .collect();
        let learnable: Vec<_> = filtered
            .iter()
            .filter_map(|entry| match entry.gram.gram.0.as_slice() {
                [Atom::Tok(word)] if word.heteronym().is_some() => Some(word.text.clone()),
                _ => None,
            })
            .collect();
        measure!(
            "etymology",
            crate::etymology::build_etymology_segmentations_with_llm(
                course,
                &golden,
                &learnable,
                &descriptions
            )
        )
    } else {
        report(
            stages,
            &format!(
                "{}:etymology:{}",
                lang.code(),
                course.native_language.code()
            ),
            json!({"blocked_by":"morphology"}),
        )?;
        None
    };
    if let Some(etymology) = &etymology {
        if !etymology.is_empty() {
            let by_word = crate::morpheme_info::invert_segmentations(etymology);
            let info = filtered
                .iter()
                .filter_map(|entry| match entry.gram.gram.0.as_slice() {
                    [Atom::Tok(word)] => word
                        .heteronym()
                        .map(|h| (word.text.clone(), (h.lemma.clone(), h.pos))),
                    _ => None,
                })
                .collect();
            measure!("morphemes", async {
                Ok(crate::morpheme_info::analyze_morphemes(course, &by_word, &info).await)
            });
        }
    } else {
        report(
            stages,
            &format!(
                "{}:morphemes:{}",
                lang.code(),
                course.native_language.code()
            ),
            json!({"blocked_by":"etymology"}),
        )?;
    }
    // Redundancy and pronunciation selection consume the fully assembled,
    // definition-filtered vocabulary. Never substitute partial dictionary results.
    let blockers: Vec<_> = [
        ("dictionary", dictionary.is_none()),
        ("phrasebook", phrasebook.is_none()),
        ("sense-definitions", senses.is_none()),
        ("morphology", morphology.is_none()),
        ("etymology", etymology.is_none()),
        ("slot-overlays", !overlays_complete),
    ]
    .into_iter()
    .filter_map(|(name, missing)| missing.then_some(name))
    .collect();
    let (Some(dictionary), Some(mut phrasebook), Some(senses), Some(morphology), Some(etymology)) =
        (dictionary, phrasebook, senses, morphology, etymology)
    else {
        report(
            stages,
            lang.code(),
            json!({"blocked_by":blockers, "downstream":if homophones_from_cache {vec!["sense-redundancy"]} else {vec!["sense-redundancy","homophone-practice"]},"pronunciation_selection_already_observed":true}),
        )?;
        return Ok(());
    };
    {
        let custom = crate::custom_definitions::load(&data, course)?;
        let dictionary: BTreeMap<_, _> = dictionary
            .into_iter()
            .filter_map(|(heteronym, mut definition)| {
                if let Some(value) = custom.get(&heteronym) {
                    definition = value.clone();
                }
                for value in &mut definition.definitions {
                    crate::dict::normalize_english_gloss(&mut value.native, course.native_language);
                    crate::dict::normalize_english_gloss(
                        &mut value.example_sentence_native_language,
                        course.native_language,
                    );
                }
                let entry = language_utils::DictionaryEntry {
                    target_language_word: definition.target_language_word,
                    definitions: definition.definitions,
                    morphology: morphology.get(&heteronym)?.clone(),
                    segments: etymology.get(&heteronym.word).cloned().unwrap_or_default(),
                };
                Some((heteronym, entry))
            })
            .collect();
        crate::custom_phrasebook::apply(&data, course, &mut phrasebook)?;
        let mut phrasebook: BTreeMap<_, _> = phrasebook
            .into_iter()
            .map(|(gram, value)| (language_utils::TaggedGram { gram, sense: None }, value))
            .collect();
        let (sense_definitions, sense_phrases) = senses;
        let sense_dictionary: BTreeMap<_, _> = sense_definitions
            .into_iter()
            .filter_map(|(gram, definition)| {
                let heteronym = gram.gram.heteronym()?;
                let entry = language_utils::DictionaryEntry {
                    target_language_word: heteronym.word.clone(),
                    definitions: vec![definition],
                    morphology: morphology.get(heteronym)?.clone(),
                    segments: etymology.get(&heteronym.word).cloned().unwrap_or_default(),
                };
                Some((gram, entry))
            })
            .collect();
        phrasebook.extend(sense_phrases);
        let frequencies: Vec<_> = frequencies
            .into_iter()
            .filter(|entry| {
                if entry.gram.gram.len() == 1 {
                    entry.gram.gram.heteronym().is_some_and(|h| {
                        if entry.gram.sense.is_some() {
                            sense_dictionary.contains_key(&entry.gram)
                        } else {
                            dictionary.contains_key(h)
                        }
                    })
                } else {
                    phrasebook.contains_key(&entry.gram)
                }
            })
            .collect();
        let mut keyed: BTreeMap<_, _> = frequencies
            .iter()
            .filter_map(|entry| {
                let h = entry.gram.gram.heteronym()?;
                Some((entry.gram.clone(), dictionary.get(h)?.clone()))
            })
            .collect();
        keyed.extend(sense_dictionary);
        measure!(
            "sense-redundancy",
            crate::sense_redundancy::find_redundant_senses(
                course,
                &keyed,
                &phrasebook,
                &inventories,
                &frequencies
            )
        );
        if let Some(selected) = pronunciations.filter(|_| !homophones_from_cache) {
            let scratch = tempfile::tempdir()?;
            let mut pronunciation_file =
                std::fs::File::create(scratch.path().join("word_to_pronunciation.jsonl"))?;
            for row in selected {
                writeln!(pronunciation_file, "{}", serde_json::to_string(&row)?)?;
            }
            let homophones = crate::disambiguation_practice::generate_homophones(
                course,
                scratch.path(),
                &frequencies,
                1000,
            )?;
            measure!(
                "homophone-practice",
                crate::disambiguation_practice::generate_homophone_practice(
                    course,
                    &homophones,
                    scratch.path()
                )
            );
        } else if !homophones_from_cache {
            report(
                stages,
                &format!(
                    "{}:homophone-practice:{}",
                    lang.code(),
                    course.native_language.code()
                ),
                json!({"blocked_by":"pronunciations"}),
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_reuse_copies_real_inputs_without_changing_sources_or_dialect_samples() {
        let root = tempfile::tempdir().unwrap();
        let corpus = root.path().join("corpus");
        let target = root.path().join("target");
        let reuse = root.path().join("reuse");
        let destination = root.path().join("new");
        for directory in [&corpus, &target, &reuse, &destination] {
            std::fs::create_dir(directory).unwrap();
        }
        let name = "target_language_sentences_tokenization.jsonl";
        std::fs::write(corpus.join(name), "original bytes\n").unwrap();
        std::fs::write(reuse.join(name), "retained bytes\n").unwrap();
        std::fs::write(target.join("slot_match_samples.jsonl"), "target samples\n").unwrap();
        std::fs::write(
            reuse.join("slot_match_samples.jsonl"),
            "other dialect samples\n",
        )
        .unwrap();
        copy_preparation_cache(&destination, &corpus, &target, Some(&reuse)).unwrap();
        assert_eq!(
            std::fs::read(destination.join(name)).unwrap(),
            b"retained bytes\n"
        );
        assert_eq!(
            std::fs::read(corpus.join(name)).unwrap(),
            b"original bytes\n"
        );
        assert_eq!(
            std::fs::read(reuse.join(name)).unwrap(),
            b"retained bytes\n"
        );
        assert_eq!(
            std::fs::read(destination.join("slot_match_samples.jsonl")).unwrap(),
            b"target samples\n"
        );
    }

    #[test]
    fn only_typed_cache_misses_can_leave_overlays_pending() {
        let missing =
            anyhow::Error::new(tysm::chat_completions::ChatError::CacheMiss).context("slot hints");
        assert!(is_cache_miss(&missing));
        assert!(!is_cache_miss(&anyhow::anyhow!(
            "CacheMiss-looking IO error"
        )));
    }

    #[test]
    fn dependency_counts_survive_error_context() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("stages.jsonl");
        let mut file = std::fs::File::create(&path).unwrap();
        let error = anyhow::Error::new(DependencyBarrier {
            component: "tokenization",
            pending: 2,
            exhausted: 3,
            input: None,
        })
        .context("corpus preparation");
        assert!(
            result::<()>(&mut file, "test", Err(error), tysm::measurement::misses())
                .unwrap()
                .is_none()
        );
        let row: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(row["result"]["dependency"]["pending"], 2);
        assert_eq!(row["result"]["dependency"]["exhausted"], 3);
        assert_eq!(row["result"]["tysm_cache_misses"], 0);
    }
}
