mod data;
mod definition_text;
mod frequencies;
pub mod minimal_pairs;
mod sentence_codec;
mod sources;
mod unigram_splitter;
use unigram_splitter::UnigramSplitter;
pub mod word_prefix;

pub use data::ConsolidatedLanguageData;
pub use frequencies::{GramFrequencyList, aggregate_sense_frequencies};
use frequencies::{ease_orders, sense_index};

use language_utils::indexmap::IndexMap;
use language_utils::language_pack::*;
use language_utils::sentence_codec::{GramKey, Symbol};
use language_utils::*;
use lasso::{Key, Spur};
use minimal_pairs::build_minimal_pairs_index;
use rustc_hash::FxHashMap;
use std::collections::BTreeMap;
use std::sync::Arc;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod word_prefix_tests;

/// Build a pack from generated corpus data. Runtime loaders use `LanguagePack::from_parts`.
pub fn build(language_data: ConsolidatedLanguageData, course: Course) -> LanguagePack {
    std::thread::scope(|scope| {
        let translation_texts = language_data
            .translations
            .iter()
            .flat_map(|(_, texts)| texts.iter().cloned())
            .collect();
        let translation_job = scope.spawn(move || {
            lasso::CompressedRodeoReader::<TranslationSpur, lasso::RunTable, lasso::Text>::build::<
                UnigramSplitter,
            >(translation_texts)
        });
        let target_language = course.target_language;
        let sense_frequencies =
            aggregate_sense_frequencies(&language_data.gram_frequencies.entries);
        let texts = word_prefix::prefixed_texts(
            &sense_frequencies,
            &language_data.gram_dictionary,
            &language_data.phrasebook,
            target_language,
        );
        let rodeo = {
            let mut rodeo = lasso::Rodeo::new();
            language_data.intern(&mut rodeo);
            for text in texts.values() {
                rodeo.get_or_intern(text);
            }
            rodeo.into_reader()
        };

        let gram_rodeo = {
            let mut grams: lasso::Rodeo<Gram<Spur>> = lasso::Rodeo::new();
            for entry in &language_data.gram_vocabulary {
                grams.get_or_intern(
                    entry
                        .get_interned(&rodeo)
                        .expect("vocabulary strings are interned")
                        .atoms,
                );
            }
            grams.into_reader()
        };
        let encoded: Vec<_> = language_data
            .encoded_sentences
            .iter()
            .filter_map(|(text, sentence)| {
                let sentence = sentence.clone().try_map(|gram| {
                    gram.get_interned(&rodeo)
                        .expect("encoded sentence strings must be interned")
                        .get_interned(&gram_rodeo)
                })?;
                Some((text, sentence))
            })
            .collect();

        let words_to_heteronyms: FxHashMap<Spur, Vec<Heteronym<Spur>>> = {
            let mut map: FxHashMap<Spur, Vec<(Heteronym<Spur>, u32)>> = FxHashMap::default();

            for entry in &sense_frequencies {
                if let Some(heteronym) = entry.gram.gram.heteronym() {
                    let word_spur = rodeo.get(&heteronym.word).unwrap();
                    let interned_het = Heteronym {
                        word: rodeo.get(&heteronym.word).unwrap(),
                        lemma: rodeo.get(&heteronym.lemma).unwrap(),
                        pos: heteronym.pos,
                    };
                    let vec = map.entry(word_spur).or_default();
                    if let Some(existing) = vec.iter_mut().find(|(h, _)| *h == interned_het) {
                        existing.1 = existing.1.max(entry.count);
                    } else {
                        vec.push((interned_het, entry.count));
                    }
                }
            }

            map.into_iter()
                .map(|(word, mut hets)| {
                    hets.sort_by_key(|b| std::cmp::Reverse(b.1));
                    (word, hets.into_iter().map(|(h, _)| h).collect::<Vec<_>>())
                })
                .collect()
        };

        let word_to_pronunciation = {
            language_data
                .word_to_pronunciation
                .iter()
                .map(|(word, pronunciations)| {
                    (
                        rodeo
                            .get(word)
                            .unwrap_or_else(|| panic!("word not in rodeo: {word:?}")),
                        // Language pack only stores the main pronunciation;
                        // verifier alternates aren't surfaced to the frontend yet.
                        rodeo.get(&pronunciations.main).unwrap_or_else(|| {
                            panic!("pronunciation not in rodeo: {:?}", pronunciations.main)
                        }),
                    )
                })
                .collect()
        };

        let pronunciation_to_words: FxHashMap<Spur, Vec<Spur>> = {
            language_data
                .pronunciation_to_words
                .iter()
                .map(|(pronunciation, words)| {
                    (
                        rodeo.get(pronunciation).unwrap(),
                        words.iter().map(|word| rodeo.get(word).unwrap()).collect(),
                    )
                })
                .collect()
        };

        let pronunciation_data = language_data.pronunciation_data.clone();

        let pattern_frequency_map = {
            pronunciation_data
                .pattern_frequencies
                .iter()
                .map(|((pattern, position), freq)| {
                    ((rodeo.get(pattern).unwrap(), *position), *freq)
                })
                .collect()
        };

        let movies = visible_movies(language_data.movies);
        let books = language_data.books;

        let human_audio = language_data.human_audio.clone();
        let pronunciation_audio = language_data.pronunciation_audio.clone();

        let proper_noun_definitions = {
            language_data
                .proper_noun_definitions
                .iter()
                .map(|(proper_noun, definition)| {
                    (rodeo.get(proper_noun).unwrap(), definition.clone())
                })
                .collect()
        };

        // Build as counts first; we'll compute the `easy` flag after gram_definitions is available.
        let gram_counts: IndexMap<SpurReadingGram, (u32, u32)> = {
            let mut map = IndexMap::new();
            for entry in &language_data.gram_frequencies.entries {
                let interned_gram = entry.gram.get_interned(&rodeo);
                if let Some(interned_gram) = interned_gram
                    && let Some(gram_spur) = interned_gram.get_interned(&gram_rodeo)
                {
                    map.insert(gram_spur, (entry.count, entry.direct_count));
                }
            }
            map
        };

        let gram_definitions: FxHashMap<TaggedGram<SpurGram>, GramDefinition> = {
            let mut map = FxHashMap::default();

            for (gram, definition) in &language_data.gram_dictionary {
                let interned_gram: Option<Gram<Spur>> = gram
                    .gram
                    .iter()
                    .map(|atom| atom.get_interned(&rodeo))
                    .collect();
                if let Some(interned_gram) = interned_gram
                    && let Some(gram_spur) = interned_gram.get_interned(&gram_rodeo)
                {
                    map.insert(
                        TaggedGram {
                            gram: gram_spur,
                            sense: gram.sense,
                        },
                        GramDefinition::Dictionary(definition.clone()),
                    );
                }
            }

            for (gram, entry) in &language_data.phrasebook {
                let interned_gram: Option<Gram<Spur>> = gram
                    .gram
                    .iter()
                    .map(|atom| atom.get_interned(&rodeo))
                    .collect();
                if let Some(interned_gram) = interned_gram
                    && let Some(gram_spur) = interned_gram.get_interned(&gram_rodeo)
                {
                    map.insert(
                        TaggedGram {
                            gram: gram_spur,
                            sense: gram.sense,
                        },
                        GramDefinition::Phrasebook(entry.clone()),
                    );
                }
            }

            map
        };

        let [mut target_texts, native_texts] =
            definition_text::text_sets(gram_definitions.values());
        target_texts.extend(native_texts);
        let definition_job = scope.spawn(move || {
        lasso::CompressedRodeoReader::<DefinitionTextSpur, lasso::RunTable, lasso::Text>::build::<UnigramSplitter>(target_texts)
    });

        let gram_frequencies_map: IndexMap<SpurReadingGram, Frequency> = {
            const COGNATE_BONUS: f32 = 2.0;

            // First pass: compute base ease for single-atom grams so we can reference them
            // when computing multi-atom gram ease.
            let mut single_atom_ease: FxHashMap<Atom<Spur>, f32> = FxHashMap::default();
            for (gram_spur, (count, _direct_count)) in gram_counts.iter() {
                let gram = gram_rodeo.resolve(&gram_spur.sense.gram);
                if gram.len() == 1
                    && let Some(atom) = gram.iter().next()
                {
                    let base_ease = (*count as f32).ln();
                    let has_cognate = !course.teaches_new_writing_system()
                        && has_cognate_definition(gram_definitions.get(&gram_spur.sense));
                    let ease = if has_cognate {
                        base_ease + COGNATE_BONUS
                    } else {
                        base_ease
                    };
                    single_atom_ease
                        .entry(*atom)
                        .and_modify(|old| *old = old.max(ease))
                        .or_insert(ease);
                }
            }

            let mut map = IndexMap::new();
            for (gram_spur, (count, direct_count)) in gram_counts.iter() {
                let gram = gram_rodeo.resolve(&gram_spur.sense.gram);
                let easy = !course.teaches_new_writing_system()
                    && is_gram_easy(gram, gram_definitions.get(&gram_spur.sense));

                let ln_count = (*count as f32).ln();

                let definition = gram_definitions.get(&gram_spur.sense);
                let (is_compositional, is_literally_translatable, is_cognate) = match definition {
                    Some(GramDefinition::Phrasebook(entry)) => (
                        entry.compositional,
                        entry.can_be_translated_literally,
                        entry.cognate && !entry.false_cognate,
                    ),
                    _ => (false, false, false),
                };
                let compositional = is_compositional || is_literally_translatable;

                let ease = if gram.len() <= 1 {
                    ln_count
                        + if !course.teaches_new_writing_system()
                            && has_cognate_definition(definition)
                        {
                            COGNATE_BONUS
                        } else {
                            0.0
                        }
                } else {
                    // Multi-atom gram: ease depends on compositionality and component word ease.
                    // A learnable component with no standalone gram never occurs outside this
                    // gram, so it can't be easier than the gram itself — fall back to ln_count
                    // instead of skipping it. Skipping would let the easy components set the
                    // ease (e.g. "est infecté" inheriting the ease of "est" when "infecté" has
                    // no entry of its own).
                    let min_component_ease = gram
                        .iter()
                        .filter_map(|atom| {
                            let ease = single_atom_ease.get(atom).copied();
                            match atom {
                                Atom::Tok(word)
                                    if matches!(word.word_type, WordType::Heteronym(_)) =>
                                {
                                    Some(ease.unwrap_or(ln_count))
                                }
                                _ => ease,
                            }
                        })
                        .reduce(f32::min);

                    let base_ease = match (
                        is_literally_translatable,
                        is_compositional,
                        min_component_ease,
                    ) {
                        // Can be translated word-for-word: as easy as the hardest component.
                        (true, _, Some(min_ease)) => min_ease.max(ln_count),
                        // Compositional but not literal: blend component ease with own frequency.
                        (false, true, Some(min_ease)) => {
                            (0.7 * min_ease + 0.3 * ln_count).max(ln_count)
                        }
                        // Non-compositional (idiom) or no component data: own frequency only.
                        _ => ln_count,
                    };

                    let cognate_bonus = if is_cognate && !course.teaches_new_writing_system() {
                        COGNATE_BONUS
                    } else {
                        0.0
                    };

                    base_ease + cognate_bonus
                };

                map.insert(
                    *gram_spur,
                    Frequency {
                        count: *count,
                        direct_count: *direct_count,
                        easy,
                        compositional,
                        ease,
                    },
                );
            }
            map
        };

        let gram_frequencies = FrequencyList {
            entries: gram_frequencies_map,
            total_count: language_data.gram_frequencies.total_count,
        };

        let sense_frequencies = FrequencyList::sense_entries(gram_frequencies.entries.iter());
        let senses = sense_index(&sense_frequencies);
        let (written_ease_order, listening_ease_order) = ease_orders(&gram_frequencies);
        for (_, sentence) in &encoded {
            let learnable = sentence.grams.iter().filter_map(SentenceGram::learnable);
            let phrases = sentence
                .multiword_terms
                .iter()
                .chain(&sentence.low_confidence_multiword_terms)
                .map(|term| &term.gram);
            for gram in learnable.chain(phrases) {
                assert!(
                    written_ease_order.rank(gram).is_some(),
                    "Encoded gram has no written-ease rank: {:?}",
                    gram.resolve(&gram_rodeo).resolve(&rodeo)
                );
            }
        }
        let heteronym_to_grams: FxHashMap<Heteronym<Spur>, Vec<SpurGram>> = {
            let mut map: FxHashMap<Heteronym<Spur>, Vec<(SpurGram, Frequency)>> =
                FxHashMap::default();
            for (gram_spur, freq) in sense_frequencies.iter() {
                let gram = gram_rodeo.resolve(&gram_spur.gram);
                if gram.len() == 1
                    && let Some(Atom::Tok(word)) = gram.iter().next()
                    && let WordType::Heteronym(heteronym) = &word.word_type
                {
                    map.entry(*heteronym)
                        .or_default()
                        .push((gram_spur.gram, *freq));
                }
            }
            map.into_iter()
                .map(|(k, mut v)| {
                    v.sort_by_key(|b| std::cmp::Reverse(b.1.count));
                    (k, {
                        let mut seen = std::collections::HashSet::new();
                        v.into_iter()
                            .map(|(spur, _)| spur)
                            .filter(|g| seen.insert(*g))
                            .collect()
                    })
                })
                .collect()
        };

        let pronunciation_max_freq_cache: FxHashMap<Spur, Frequency> = pronunciation_to_words
            .iter()
            .filter_map(|(pronunciation, words)| {
                let max_freq = words
                    .iter()
                    .flat_map(|word| {
                        words_to_heteronyms
                            .get(word)
                            .map(|heteronyms| heteronyms.iter())
                            .into_iter()
                            .flatten()
                    })
                    .filter_map(|heteronym| {
                        heteronym_to_grams
                            .get(heteronym)?
                            .iter()
                            .filter_map(|gram_spur| {
                                sense_frequencies
                                    .get(senses.get(gram_spur)?.first()?)
                                    .copied()
                            })
                            .max_by_key(|f| f.count)
                    })
                    .max_by_key(|f| f.count)?;
                Some((*pronunciation, max_freq))
            })
            .collect();

        // Rank minimal pairs by standalone usage; each heteronym
        // lists its single-atom grams in descending frequency order.
        let word_max_single_gram_freq: FxHashMap<Spur, u32> = words_to_heteronyms
            .iter()
            .filter_map(|(word, hets)| {
                let max = hets
                    .iter()
                    .filter_map(|het| {
                        let top_gram = heteronym_to_grams.get(het)?.first()?;
                        sense_frequencies
                            .get(senses.get(top_gram)?.first()?)
                            .map(|f| f.count)
                    })
                    .max()?;
                Some((*word, max))
            })
            .collect();

        let minimal_pairs = build_minimal_pairs_index(
            &language_data.minimal_pairs,
            &rodeo,
            &word_max_single_gram_freq,
        );

        let string_to_grams: FxHashMap<String, Vec<TaggedGram<SpurGram>>> = {
            let mut map: FxHashMap<String, Vec<TaggedGram<SpurGram>>> = FxHashMap::default();
            for (&gram_spur, _) in sense_frequencies.iter() {
                let resolved = gram_rodeo.resolve(&gram_spur.gram).resolve(&rodeo);
                let display = resolved.to_display_string(target_language);
                map.entry(display).or_default().push(gram_spur);
            }
            map
        };

        let morphemes: FxHashMap<MorphemeSegment<Spur>, MorphemeInfo<Spur>> = language_data
            .morphemes
            .iter()
            .map(|(segment, info)| {
                let key = MorphemeSegment {
                    surface: rodeo.get(&segment.surface).unwrap(),
                    canonical: rodeo.get(&segment.canonical).unwrap(),
                    tag: segment.tag.as_ref().map(|t| rodeo.get(t).unwrap()),
                };
                let interned = match info {
                    MorphemeInfo::Root { heteronym } => MorphemeInfo::Root {
                        heteronym: Heteronym {
                            word: rodeo.get(&heteronym.word).unwrap(),
                            lemma: rodeo.get(&heteronym.lemma).unwrap(),
                            pos: heteronym.pos,
                        },
                    },
                    MorphemeInfo::Bound { meaning } => MorphemeInfo::Bound {
                        meaning: meaning.as_ref().map(|m| rodeo.get(m).unwrap()),
                    },
                    MorphemeInfo::Derivation { meaning } => MorphemeInfo::Derivation {
                        meaning: meaning.as_ref().map(|m| rodeo.get(m).unwrap()),
                    },
                    MorphemeInfo::Inflection { meaning } => MorphemeInfo::Inflection {
                        meaning: meaning.as_ref().map(|m| rodeo.get(m).unwrap()),
                    },
                };
                (key, interned)
            })
            .collect();

        let prefixed_text: FxHashMap<_, _> = texts
            .iter()
            .filter_map(|(gram, text)| {
                Some((
                    gram.get_interned(&rodeo)?.get_interned(&gram_rodeo)?,
                    rodeo.get(text)?,
                ))
            })
            .collect();
        let mut grams_with_prefixed_text: FxHashMap<_, Vec<_>> = FxHashMap::default();
        for (&gram, _) in sense_frequencies.iter() {
            if let Some(&text) = prefixed_text.get(&gram.gram) {
                grams_with_prefixed_text.entry(text).or_default().push(gram);
            }
        }
        let redundant_senses = language_data
            .redundant_senses
            .iter()
            .filter_map(|(text, sets)| {
                let text = rodeo.get(text)?;
                let sets = sets
                    .iter()
                    .filter_map(|set| {
                        let members: Vec<_> = set
                            .iter()
                            .filter_map(|gram| gram.get_interned(&rodeo)?.get_interned(&gram_rodeo))
                            .collect();
                        (members.len() >= 2).then_some(members)
                    })
                    .collect();
                Some((text, sets))
            })
            .collect();

        let gram_readings = derive_readings(encoded.iter().map(|(_, sentence)| sentence), &rodeo);
        let mut reading_grams = lasso::Rodeo::<SpurReadingGram, GramKey>::new();
        let mut reading_keys: FxHashMap<_, Vec<_>> = FxHashMap::default();
        for reading in written_ease_order.iter_from(0) {
            let key = reading_grams.get_or_intern(reading);
            reading_keys.entry(reading.sense).or_default().push(key);
        }
        let learnable_count = reading_grams.len();
        let mut entries = IndexMap::new();
        for (reading, frequency) in gram_frequencies.entries.iter() {
            entries.insert(reading_grams.get_or_intern(*reading), *frequency);
        }
        let gram_frequencies = FrequencyList {
            entries,
            total_count: gram_frequencies.total_count,
        };
        let source_gram_frequencies = language_data
            .source_gram_frequencies
            .iter()
            .filter(|(_, list)| !list.entries.is_empty())
            .map(|(source, list)| {
                let entries = list.entries.iter().map(|entry| {
                    let reading = entry
                        .gram
                        .get_interned(&rodeo)
                        .unwrap()
                        .get_interned(&gram_rodeo)
                        .unwrap();
                    let key = reading_grams
                        .get(reading)
                        .expect("source gram must exist in global frequencies");
                    (key, entry.count)
                });
                (
                    source.clone(),
                    sources::source_frequencies(entries, list.total_count, &gram_frequencies),
                )
            })
            .collect();
        let sequences: std::collections::HashSet<_> = encoded
            .iter()
            .map(|(_, sentence)| {
                sentence_codec::encode(sentence, &mut reading_grams, learnable_count)
            })
            .collect();
        assert_eq!(
            sequences.len(),
            encoded.len(),
            "distinct sentences must encode distinctly"
        );
        let written_ease_order = WrittenEaseOrder {
            grams: reading_grams.into_reader(),
            ease: written_ease_order
                .entries
                .into_iter()
                .map(|(_, ease)| ease)
                .collect(),
        };
        let cx = language_utils::sentence_text::RenderContext {
            grams: &written_ease_order.grams,
            gram_rodeo: &gram_rodeo,
            strings: &rodeo,
            language: target_language,
        };
        let encoded_sentences =
            lasso::CompressedRodeoReader::<SentenceSpur, Vec<Symbol>, SentenceText>::build(
                &cx, sequences,
            );
        let sentence_key = |text: &str| {
            encoded_sentences
                .get(&cx, text)
                .expect("shipped sentence must be encoded and render exactly")
        };
        for text in &language_data.target_language_sentences {
            sentence_key(text);
        }
        let translation_rodeo = translation_job.join().unwrap();
        let mut translations = vec![Vec::new(); encoded_sentences.len()];
        for (text, texts) in &language_data.translations {
            translations[sentence_key(text).into_usize()] = texts
                .iter()
                .map(|text| translation_rodeo.get(&(), text).unwrap())
                .collect();
        }
        let translations = SentenceLists::new(translations);
        for (text, _) in &encoded {
            assert!(
                translations
                    .get(sentence_key(text))
                    .iter()
                    .any(|key| translation_rodeo.resolve(key).next().is_some()),
                "encoded sentence {text:?} must have a nonempty translation"
            );
        }
        let mut sources = vec![SentenceSource::none(); encoded_sentences.len()];
        for (text, source) in &language_data.sentence_sources {
            sources[sentence_key(text).into_usize()] = source.clone();
        }
        let sentence_sources = sources::sentence_sources(&sources);
        let homophone_practice = language_data
            .homophone_practice
            .iter()
            .map(|(words, practice)| {
                (
                    words.get_interned(&rodeo).unwrap(),
                    HomophonePractice {
                        sentence_pairs: practice
                            .sentence_pairs
                            .iter()
                            .map(|pair| language_utils::HomophoneSentencePair {
                                sentence1: encoded_sentences
                                    .get(&cx, &pair.sentence1)
                                    .expect("homophone sentence must be encoded"),
                                sentence2: encoded_sentences
                                    .get(&cx, &pair.sentence2)
                                    .expect("homophone sentence must be encoded"),
                            })
                            .collect(),
                    },
                )
            })
            .collect();
        let mut sentences_containing_gram_index: FxHashMap<
            TaggedGram<SpurGram>,
            Vec<SentenceSpur>,
        > = FxHashMap::default();
        for (text, sentence) in &encoded {
            let key = sentence_key(text);
            for gram in sentence.all_grams() {
                sentences_containing_gram_index
                    .entry(gram.sense)
                    .or_default()
                    .push(key);
            }
        }
        let definition_texts = definition_job.join().unwrap();
        let definitions =
            definition_text::intern_lists(gram_definitions.into_iter().map(|(key, definition)| {
                (
                    key,
                    definition.map_text(|text, _| definition_texts.get(&(), text).unwrap()),
                )
            }));
        let fingerprint = spur_space_fingerprint(&rodeo, &gram_rodeo, &written_ease_order);
        let pack = LanguagePack {
            sense_frequencies,
            core: Arc::new(LanguagePackCore {
                spur_space_fingerprint: fingerprint,
                string_rodeo: rodeo,
                gram_rodeo,
                senses,
                reading_keys,
                written_ease_order,
                listening_ease_order,
                words_to_heteronyms,
                word_to_pronunciation,
                pronunciation_to_words,
                minimal_pairs,
                pronunciation_data,
                pattern_frequency_map,
                pronunciation_max_freq_cache,
                proper_noun_definitions,
                gram_frequencies,
                definition_texts,
                gram_definitions: definitions.entries,
                definition_lists: definitions.definitions,
                definition_morphology: definitions.morphology,
                definition_segments: definitions.segments,
                heteronym_to_grams,
                string_to_grams,
                morphemes,
            }),
            sentences: LanguagePackSentences {
                spur_space_fingerprint: fingerprint,
                ruby_segmentations: BTreeMap::new(),
                gram_readings,
                prefixed_text,
                grams_with_prefixed_text,
                redundant_senses,
                target_language: course.target_language,
                translation_rodeo,
                translations,
                source_gram_frequencies,
                homophone_practice,
                movies,
                books,
                sentence_sources,
                encoded_sentences,
                sentences_containing_gram_index,
                human_audio,
                pronunciation_audio,
                strokes: language_data.strokes,
            },
        };
        assert_eq!(pack.sentences.encoded_sentences.len(), encoded.len());
        for (expected, _sentence) in &encoded {
            let key = pack
                .sentence_key(expected)
                .expect("sentence index must round-trip");
            let text = pack.sentence_text(key);
            assert_eq!(text, **expected, "decoded sentence text must round-trip");
            assert_eq!(
                language_utils::literals_to_text(
                    &pack.sentence_to_literals(&key, target_language).unwrap()
                ),
                text,
                "display and lookup rendering must agree"
            );
            #[cfg(test)]
            assert_eq!(
                pack.sentence_grams(key).as_ref(),
                Some(_sentence),
                "pack sentence round-trip"
            );
        }
        pack
    })
}
/// Check if a resolved gram is "easy" based on its definition.
/// Easy grams are single-word cognates with a single-word definition.
fn is_gram_easy(gram: &grm<Spur>, definition: Option<&GramDefinition>) -> bool {
    if gram.len() != 1 {
        return false;
    }
    let Some(Atom::Tok(word)) = gram.iter().next() else {
        return false;
    };
    let WordType::Heteronym(h) = &word.word_type else {
        return false;
    };
    if h.pos == PartOfSpeech::Intj {
        return false;
    }

    let Some(GramDefinition::Dictionary(entry)) = definition else {
        return false;
    };
    is_dictionary_entry_easy(entry)
}

/// Check if a gram has any cognate definition (more permissive than is_gram_easy).
fn has_cognate_definition(definition: Option<&GramDefinition>) -> bool {
    match definition {
        Some(GramDefinition::Dictionary(entry)) => entry
            .definitions
            .iter()
            .any(|d| d.cognate && !d.false_cognate),
        Some(GramDefinition::Phrasebook(entry)) => entry.cognate && !entry.false_cognate,
        None => false,
    }
}

fn is_dictionary_entry_easy(entry: &DictionaryEntry) -> bool {
    if entry.definitions.len() != 1 {
        return false;
    }
    let Some(definition) = entry.definitions.first() else {
        return false;
    };
    if definition.native.contains(' ') {
        return false;
    }
    if definition.native == entry.target_language_word {
        return false;
    }
    definition.cognate && !definition.false_cognate
}
/// Fingerprint of all three core key spaces and the learnable-key boundary.
/// A sentences half fits a core exactly when strings, grams, reading keys and
/// the learnable prefix agree. Identical content produces identical fingerprints.
fn spur_space_fingerprint(
    core_strings: &lasso::RodeoReader,
    core_grams: &lasso::RodeoReader<Gram<Spur>>,
    written: &WrittenEaseOrder,
) -> u64 {
    let mut hasher = xxhash_rust::xxh3::Xxh3::new();
    hasher.update(b"sentence-expression-index-v5-interned-definitions-xxh3");
    hasher.update(&(core_strings.len() as u64).to_le_bytes());
    hasher.update(&(core_grams.len() as u64).to_le_bytes());
    for (_, s) in core_strings.iter() {
        hasher.update(&(s.len() as u64).to_le_bytes());
        hasher.update(s.as_bytes());
    }
    for (_, g) in core_grams.iter() {
        // A gram is atoms over the (already-hashed) string spur space; its
        // rkyv encoding is a stable byte form of exactly that.
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&Gram(g.atoms().to_vec()))
            .expect("gram serialization for fingerprint cannot fail");
        hasher.update(&(bytes.len() as u64).to_le_bytes());
        hasher.update(&bytes);
    }
    let reading_grams = &written.grams;
    hasher.update(&(written.len() as u64).to_le_bytes());
    hasher.update(&(reading_grams.len() as u64).to_le_bytes());
    for (_, gram) in reading_grams.iter() {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(gram).unwrap();
        hasher.update(&bytes);
    }
    hasher.digest()
}

fn derive_readings<'a>(
    sentences: impl Iterator<Item = &'a SentenceGrams<SpurReadingGram>>,
    strings: &lasso::RodeoReader,
) -> FxHashMap<TaggedGram<SpurGram>, Vec<Spur>> {
    let mut counts: FxHashMap<_, FxHashMap<Spur, usize>> = FxHashMap::default();
    let mut phrases = FxHashMap::default();
    for sentence in sentences {
        for term in sentence
            .multiword_terms
            .iter()
            .chain(&sentence.low_confidence_multiword_terms)
        {
            let canonical = phrases.entry(term.gram.sense).or_insert(term.gram.reading);
            assert_eq!(
                *canonical, term.gram.reading,
                "phrase readings are canonical"
            );
        }
        for token in sentence.grams.iter().map(|g| match g {
            SentenceGram::Learnable(g) | SentenceGram::Obvious(g) => g,
        }) {
            if let Some(reading) = token.reading {
                *counts
                    .entry(token.sense)
                    .or_default()
                    .entry(reading)
                    .or_default() += 1;
            }
        }
    }
    let mut readings: FxHashMap<_, Vec<_>> = counts
        .into_iter()
        .map(|(sense, counts)| {
            let mut readings: Vec<_> = counts.into_iter().collect();
            readings.sort_by(|a, b| {
                b.1.cmp(&a.1)
                    .then_with(|| strings.resolve(&a.0).cmp(strings.resolve(&b.0)))
            });
            (
                sense,
                readings.into_iter().map(|(reading, _)| reading).collect(),
            )
        })
        .collect();
    for (sense, reading) in phrases {
        if let Some(reading) = reading {
            let observed = readings.entry(sense).or_default();
            observed.retain(|&value| value != reading);
            observed.insert(0, reading);
        }
    }
    readings
}
/// Only films with a poster (and so with metadata) are shown anywhere. Applied
/// when a pack is built. Sentence
/// provenance is untouched: hiding a film must not remove its sentences.
fn visible_movies(
    mut movies: FxHashMap<String, MovieMetadata>,
) -> FxHashMap<String, MovieMetadata> {
    movies.retain(|_, movie| movie.poster_bytes.is_some());
    movies
}
