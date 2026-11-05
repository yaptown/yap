#[cfg(test)]
mod sense_tests {
    #[test]
    fn visible_senses_use_frequency_order_and_survive_serialization() {
        let mut pack = pack_with_senses(&[(2, 20), (1, 10), (3, 5)]);
        let bare = pack.intern_gram(&gram("bank")).unwrap();
        let senses = pack.senses_of(bare).to_vec();
        assert_eq!(senses[0].sense.unwrap().get(), 2);
        assert_eq!(pack.visible_senses(bare, Language::English), senses);
        assert_eq!(pack.redundant_with(senses[0]), vec![senses[0]]);
        pack.sentences.redundant_senses.insert(
            pack.sentences.prefixed_text[&bare],
            vec![vec![senses[1], senses[0]]],
        );
        let standalone = TaggedGram {
            gram: bare,
            sense: NonZeroU32::new(3),
        };
        assert_eq!(
            pack.visible_senses(bare, Language::English),
            vec![senses[0], standalone]
        );
        assert_eq!(pack.redundant_with(senses[0]), vec![senses[1], senses[0]]);
        assert_eq!(pack.redundant_with(standalone), vec![standalone]);
        let untagged = TaggedGram {
            gram: bare,
            sense: None,
        };
        assert_eq!(pack.redundant_with(untagged), vec![untagged]);
        let LanguagePack {
            core, sentences, ..
        } = pack;
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&sentences).unwrap();
        let sentences =
            rkyv::from_bytes::<LanguagePackSentences, rkyv::rancor::Error>(&bytes).unwrap();
        let core = LanguagePack::from_parts(core, None);
        let bare = core.intern_gram(&gram("bank")).unwrap();
        assert!(core.sentences.redundant_senses.is_empty());
        assert_eq!(
            core.visible_senses(bare, Language::English),
            core.senses_of(bare)
        );
        let full = core.with_sentences(sentences);
        assert_eq!(
            full.visible_senses(bare, Language::English),
            vec![full.senses_of(bare)[0], standalone]
        );
        assert_eq!(full.redundant_with(full.senses_of(bare)[0]).len(), 2);
    }

    #[test]
    fn ease_order_is_deterministic_and_independent_of_frequency_order() {
        let entries = [(3, 4.0), (2, 1.0), (1, 4.0)];
        let a = ease_order(entries.into_iter());
        let b = ease_order(entries.into_iter().rev());
        assert_eq!(a.iter_from(0).collect::<Vec<_>>(), vec![1, 3, 2]);
        assert_eq!(
            a.iter_from(0).collect::<Vec<_>>(),
            b.iter_from(0).collect::<Vec<_>>()
        );
        assert_eq!(a.rank(&1), Some(0));
        assert_eq!(a.rank(&4), None);
        assert_eq!(a.ease_at(1), Some(4.0));
        assert_eq!(a.ease_at(3), None);
        assert_eq!(a.partition_point(|ease| ease >= 4.0), 2);
        assert_eq!(a.partition_point(|ease| ease >= 5.0), 0);
        assert_eq!(a.partition_point(|ease| ease >= 0.0), 3);
        assert_eq!(a.iter_from(3).count(), 0);
        let empty = ease_order::<u32>(std::iter::empty());
        assert_eq!(empty.partition_point(|_| true), 0);
        assert_eq!(empty.iter_from(0).count(), 0);
    }

    #[test]
    fn compressed_sentences_survive_pack_serialization() {
        let mut pack = pack_with_senses(&[(0, 30), (1, 20), (2, 10)]);
        pack.sentences.pronunciation_audio.insert(
            "cue".into(),
            PronunciationClip {
                audio: Audio {
                    bytes: b"....ftypM4A ".to_vec(),
                },
                segments: vec![],
            },
        );
        assert!(
            LanguagePack::from_parts(pack.core.clone(), None)
                .sentences
                .pronunciation_audio
                .is_empty()
        );
        // The builder also compares every decoded sentence with its input in tests.
        let expected: Vec<_> = pack.decoded_sentences().collect();
        let core = rkyv::to_bytes::<rkyv::rancor::Error>(&*pack.core).unwrap();
        let sentences = rkyv::to_bytes::<rkyv::rancor::Error>(&pack.sentences).unwrap();
        let restored = LanguagePack::from_parts(
            rkyv::from_bytes::<LanguagePackCore, rkyv::rancor::Error>(&core).unwrap(),
            Some(
                rkyv::from_bytes::<LanguagePackSentences, rkyv::rancor::Error>(&sentences).unwrap(),
            ),
        );
        assert_eq!(restored.decoded_sentences().collect::<Vec<_>>(), expected);
        assert_eq!(
            restored.sentences.pronunciation_audio,
            pack.sentences.pronunciation_audio
        );
    }

    #[test]
    fn stored_ease_orders_and_senses_agree_with_frequencies() {
        let pack = pack();
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&*pack.core).unwrap();
        let core = rkyv::from_bytes::<LanguagePackCore, rkyv::rancor::Error>(&bytes).unwrap();
        let loaded = LanguagePack::from_parts(core, None);
        for pack in [&pack, &loaded] {
            let bare = pack.intern_gram(&gram("bank")).unwrap();
            let frequencies = FrequencyList {
                entries: pack
                    .gram_frequencies
                    .resolved(pack)
                    .map(|(gram, frequency)| (*gram, *frequency))
                    .collect(),
                total_count: pack.gram_frequencies.total_count,
            };
            let (written, listening) = ease_orders(&frequencies);
            assert_eq!(pack.senses, sense_index(&pack.sense_frequencies));
            assert_eq!(
                pack.written_ease_order.iter_from(0).collect::<Vec<_>>(),
                written.iter_from(0).collect::<Vec<_>>()
            );
            for (rank, gram) in written.iter_from(0).enumerate() {
                assert_eq!(pack.written_ease_order.rank(&gram), Some(rank as u32));
            }
            assert_eq!(pack.listening_ease_order.entries, listening.entries);
            assert_eq!(pack.listening_ease_order.ranks, listening.ranks);
            assert_eq!(pack.gram_frequency_total(bare).unwrap().count, 30);
            assert_eq!(pack.listening_ease_order.ease_at(0), Some(20_f32.ln()));
            assert_eq!(pack.written_ease_order.ease_at(0), Some(20_f32.ln()));
        }
    }

    #[test]
    fn listening_frequency_sums_all_sense_counts() {
        let mut pack = pack();
        let bare = pack.intern_gram(&gram("bank")).unwrap();
        let entry = pack.senses_of(bare)[1];
        pack.sense_frequencies.get_mut(&entry).unwrap().direct_count = 7;
        let total = pack.gram_frequency_total(bare).unwrap();
        assert_eq!(total.count, 30);
        assert_eq!(total.direct_count, 27);
        assert_eq!(
            total.ease,
            pack.sense_frequencies
                .get(&pack.senses_of(bare)[0])
                .unwrap()
                .ease
        );
    }

    use super::super::*;
    use crate::frequencies::ease_order;
    use language_utils::{
        GramVocabEntry, Language, PronunciationData,
        ReadingGramFrequencyEntry as GramFrequencyEntry, Word,
    };
    use std::num::NonZeroU32;

    fn movie(id: &str, poster_bytes: Option<Vec<u8>>) -> MovieMetadata {
        MovieMetadata {
            id: id.into(),
            title: id.into(),
            year: None,
            original_language: None,
            rotten_tomatoes_score: None,
            variety: None,
            poster_bytes,
        }
    }

    #[test]
    fn packs_hide_movies_without_a_poster() {
        let movies: FxHashMap<_, _> = [
            ("visible".into(), movie("visible", Some(vec![1, 2, 3]))),
            ("hidden".into(), movie("hidden", None)),
        ]
        .into_iter()
        .collect();
        let loaded = build(
            ConsolidatedLanguageData {
                movies,
                ..Default::default()
            },
            Course {
                target_language: Language::English,
                native_language: Language::French,
            },
        );
        assert_eq!(
            loaded.sentences.movies.keys().collect::<Vec<_>>(),
            vec!["visible"]
        );
    }

    fn gram(word: &str) -> Gram<String> {
        Gram(vec![Atom::Tok(Word {
            text: word.into(),
            word_type: WordType::Heteronym(Heteronym {
                word: word.into(),
                lemma: word.into(),
                pos: PartOfSpeech::Noun,
            }),
        })])
    }

    #[test]
    fn sentence_upgrade_shares_core_and_keeps_placement_pack_usable() {
        let LanguagePack {
            core, sentences, ..
        } = pack();
        let core = LanguagePack::from_parts(core, None);
        let full = core.with_sentences(sentences);
        assert!(Arc::ptr_eq(&core.core, &full.core));
        assert_eq!(core.spur_space_fingerprint, full.spur_space_fingerprint);
        assert_eq!(core.sentence_keys().count(), 0);
        assert_ne!(full.sentence_keys().count(), 0);
        let gram = core.intern_gram(&gram("bank")).unwrap();
        assert_eq!(full.senses_of(gram), core.senses_of(gram));
        for (spur, text) in core.string_rodeo.iter() {
            assert_eq!(full.string_rodeo.resolve(&spur), text);
        }
    }

    #[test]
    fn sentence_half_only_references_core_keys() {
        let tagged = |text: &str| TaggedGram {
            gram: gram(text),
            sense: None,
        };
        let frequency = |text: &str| GramFrequencyEntry {
            gram: language_utils::ReadingTaggedGram {
                sense: tagged(text),
                reading: Some("reading-only".into()),
            },
            count: 10,
            direct_count: 10,
            disambiguation_key: 0,
        };
        let encoded = |word: &str, obvious: bool| {
            let entry = language_utils::ReadingTaggedGram {
                sense: tagged(word),
                reading: Some("reading-only".into()),
            };
            SentenceGrams {
                grams: vec![if obvious {
                    SentenceGram::Obvious(entry)
                } else {
                    SentenceGram::Learnable(entry)
                }],
                capitalize_first: false,
                multiword_terms: vec![],
                low_confidence_multiword_terms: vec![],
            }
        };
        let data = ConsolidatedLanguageData {
            target_language_sentences: vec!["bank".into(), "corpus-only".into()],
            translations: vec![
                (
                    "bank".into(),
                    vec!["bank".into(), "translation-only".into()],
                ),
                ("corpus-only".into(), vec!["translation-only".into()]),
            ],
            gram_vocabulary: ["bank", "corpus-only", "source-only", "unused"]
                .into_iter()
                .map(|word| GramVocabEntry {
                    atoms: gram(word),
                    frequency: 10,
                })
                .collect(),
            gram_frequencies: GramFrequencyList {
                entries: vec![frequency("bank"), frequency("source-only")],
                total_count: 20,
            },
            encoded_sentences: vec![
                ("bank".into(), encoded("bank", false)),
                ("corpus-only".into(), encoded("corpus-only", true)),
            ],
            sentence_sources: vec![(
                "corpus-only".into(),
                SentenceSource {
                    from_anki: true,
                    from_tatoeba: true,
                    from_manual: true,
                    from_song: true,
                    movie_ids: vec!["film-without-metadata".into(), "other-film".into()],
                    book_ids: vec!["training-book".into(), "reader-book".into()],
                },
            )],
            source_gram_frequencies: [(
                language_utils::FrequencySourceId::Movie("film".into()),
                GramFrequencyList {
                    entries: vec![GramFrequencyEntry {
                        count: 3,
                        ..frequency("source-only")
                    }],
                    total_count: 10,
                },
            )]
            .into_iter()
            .collect(),
            homophone_practice: [(
                HomophoneWordPair {
                    word1: "bank".into(),
                    word2: "banc".into(),
                },
                HomophonePractice {
                    sentence_pairs: vec![language_utils::HomophoneSentencePair {
                        sentence1: "bank".into(),
                        sentence2: "corpus-only".into(),
                    }],
                },
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        };
        let course = Course {
            target_language: Language::English,
            native_language: Language::French,
        };
        let repeated = build(data.clone(), course);
        let generated = build(data, course);
        assert_eq!(
            rkyv::to_bytes::<rkyv::rancor::Error>(&*generated.core)
                .unwrap()
                .as_slice(),
            rkyv::to_bytes::<rkyv::rancor::Error>(&*repeated.core)
                .unwrap()
                .as_slice(),
        );
        assert_eq!(
            rkyv::to_bytes::<rkyv::rancor::Error>(&generated.sentences)
                .unwrap()
                .as_slice(),
            rkyv::to_bytes::<rkyv::rancor::Error>(&repeated.sentences)
                .unwrap()
                .as_slice(),
        );
        let sentence = generated.sentence_key("corpus-only").unwrap();
        assert_eq!(generated.sentence_text(sentence), "corpus-only");
        let original_literals = generated
            .sentence_to_literals(&sentence, Language::English)
            .unwrap();
        let LanguagePack {
            core, sentences, ..
        } = generated;
        let fingerprint = core.spur_space_fingerprint;
        for shared in ["bank", "banc", "corpus-only", "source-only", "reading-only"] {
            assert!(
                core.string_rodeo.get(shared).is_some(),
                "missing shared string: {shared}"
            );
        }
        assert!(core.string_rodeo.get("translation-only").is_none());
        assert_eq!(sentences.encoded_sentences.len(), 2);
        assert_eq!(sentences.translation_rodeo.len(), 2);
        let core_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&*core).unwrap();
        let sentence_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&sentences).unwrap();
        let core = rkyv::from_bytes::<LanguagePackCore, rkyv::rancor::Error>(&core_bytes).unwrap();
        let sentences =
            rkyv::from_bytes::<LanguagePackSentences, rkyv::rancor::Error>(&sentence_bytes)
                .unwrap();
        let core = LanguagePack::from_parts(core, None);
        let full = core.with_sentences(sentences);
        assert_eq!(full.spur_space_fingerprint, fingerprint);
        assert!(core.sentences.encoded_sentences.is_empty());
        assert!(core.sentences.translation_rodeo.is_empty());
        let SentenceGram::Obvious(entry) = &full.sentence_grams(sentence).unwrap().grams[0] else {
            panic!("sentence-only gram must remain obvious after assembly")
        };
        assert_eq!(full.resolve_gram(&entry.sense.gram), gram("corpus-only"));
        assert_eq!(
            full.string_rodeo.resolve(&entry.reading.unwrap()),
            "reading-only"
        );
        assert_eq!(
            full.sentence_to_literals(&sentence, Language::English)
                .unwrap(),
            original_literals
        );
        let source_frequencies = &full.sentences.source_gram_frequencies
            [&language_utils::FrequencySourceId::Movie("film".into())];
        let (reading, frequency) = source_frequencies.resolved(&full).next().unwrap();
        let key = full.written_ease_order.grams.get(reading).unwrap();
        let global = full.gram_frequencies.entries.get(&key).unwrap();
        assert_eq!(frequency.count, 3);
        assert_eq!(global.count, 10);
        assert_eq!(
            (frequency.easy, frequency.compositional, frequency.ease),
            (global.easy, global.compositional, global.ease)
        );
        assert_eq!(source_frequencies.total_count, 10);
        assert!(
            std::panic::catch_unwind(|| sources::source_frequencies(
                [(key, 256)],
                256,
                &full.gram_frequencies
            ))
            .is_err()
        );
        let absent = GramKey::try_from_usize(full.written_ease_order.grams.len()).unwrap();
        assert!(
            std::panic::catch_unwind(|| sources::source_frequencies(
                [(absent, 1)],
                1,
                &full.gram_frequencies
            ))
            .is_err()
        );
        let source = full.source_of(sentence);
        assert!(source.from_anki && source.from_tatoeba && source.from_manual && source.from_song);
        assert_eq!(
            source.movie_ids().map(String::as_str).collect::<Vec<_>>(),
            ["film-without-metadata", "other-film"]
        );
        assert_eq!(
            source.book_ids().map(String::as_str).collect::<Vec<_>>(),
            ["training-book", "reader-book"]
        );
        let bank = full.sentence_key("bank").unwrap();
        let source = full.source_of(bank);
        assert!(
            !source.from_anki && !source.from_tatoeba && !source.from_manual && !source.from_song
        );
        assert_eq!(source.movie_ids().count(), 0);
        assert_eq!(source.book_ids().count(), 0);
        assert_eq!(full.translations_of(bank).len(), 2);
        assert_eq!(full.translations_of(sentence).len(), 1);
        for sentence in full.sentence_keys() {
            assert_eq!(
                full.sentence_key(&full.sentence_text(sentence)),
                Some(sentence)
            );
            for translation in full.translations_of(sentence) {
                assert!(
                    full.sentences
                        .translation_rodeo
                        .try_resolve(translation)
                        .is_some()
                );
            }
        }
        for (words, practice) in &full.sentences.homophone_practice {
            assert_eq!(words.resolve(&full.string_rodeo).word2, "banc");
            assert_eq!(
                full.sentence_text(practice.sentence_pairs[0].sentence2),
                "corpus-only"
            );
        }
        assert_eq!(
            full.decoded_sentences().collect::<Vec<_>>(),
            repeated.decoded_sentences().collect::<Vec<_>>()
        );
        assert_eq!(full.sentences.translations, repeated.sentences.translations);
        assert_eq!(
            full.sentences.sentence_sources,
            repeated.sentences.sentence_sources
        );
        assert!(Arc::ptr_eq(&core.core, &full.core));
    }

    #[test]
    #[should_panic(expected = "language pack halves are from different builds")]
    fn sentence_upgrade_rejects_mismatched_core() {
        let LanguagePack {
            core,
            mut sentences,
            ..
        } = pack();
        sentences.spur_space_fingerprint ^= 1;
        LanguagePack::from_parts(core, None).with_sentences(sentences);
    }

    #[test]
    fn ruby_segmentations_live_only_in_the_sentence_half() {
        let mut pack = pack();
        let expected = BTreeMap::from([(
            ("東 京".into(), "とうきょう".into()),
            vec![language_utils::readings::RubySegment {
                base: "東".into(),
                ruby: Some("とう".into()),
            }],
        )]);
        pack.sentences.ruby_segmentations = expected.clone();
        let LanguagePack {
            core, sentences, ..
        } = pack;
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&sentences).unwrap();
        let sentences =
            rkyv::from_bytes::<LanguagePackSentences, rkyv::rancor::Error>(&bytes).unwrap();
        let core_only = LanguagePack::from_parts(core, None);
        assert!(core_only.sentences.ruby_segmentations.is_empty());
        assert_eq!(
            core_only
                .with_sentences(sentences)
                .sentences
                .ruby_segmentations,
            expected
        );
    }

    fn pack() -> LanguagePack {
        pack_with_senses(&[(2, 20), (1, 10)])
    }

    /// The "bank" fixture with these (sense, count) frequency entries;
    /// sense 0 is the untagged entry.
    fn pack_with_senses(senses: &[(u32, u32)]) -> LanguagePack {
        pack_with_surface(senses, "bank")
    }
    fn pack_with_surface(senses: &[(u32, u32)], surface: &str) -> LanguagePack {
        let bare = gram("bank");
        let tagged = |sense| TaggedGram {
            gram: bare.clone(),
            sense: NonZeroU32::new(sense),
        };
        build(
            ConsolidatedLanguageData {
                redundant_senses: BTreeMap::new(),
                strokes: language_utils::StrokeTable::from_iter([(
                    "一".into(),
                    vec![language_utils::StrokeGlyph {
                        standard: language_utils::StrokeStandard::Japan,
                        strokes: vec![language_utils::Stroke {
                            points: vec![(0.1, 0.5), (0.9, 0.5)],
                        }],
                    }],
                )]),
                target_language_sentences: vec![surface.into()],
                translations: vec![(surface.into(), vec!["banque".into()])],
                nlp_sentences: vec![],
                phrasebook: BTreeMap::new(),
                proper_noun_definitions: BTreeMap::new(),
                source_gram_frequencies: FxHashMap::default(),
                word_to_pronunciation: vec![],
                pronunciation_to_words: vec![],
                minimal_pairs: vec![],
                pronunciation_data: PronunciationData {
                    sounds: vec![],
                    guides: vec![],
                    pattern_frequencies: vec![],
                },
                homophone_practice: BTreeMap::new(),
                movies: FxHashMap::default(),
                books: FxHashMap::default(),
                sentence_sources: vec![],
                gram_vocabulary: [bare.clone(), gram(surface)]
                    .into_iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .into_iter()
                    .map(|atoms| GramVocabEntry {
                        atoms,
                        frequency: 30,
                    })
                    .collect(),
                gram_frequencies: GramFrequencyList {
                    entries: senses
                        .iter()
                        .flat_map(|&(sense, count)| {
                            let direct_sense = senses
                                .iter()
                                .find(|&&(sense, _)| sense == 1)
                                .unwrap_or(&senses[0])
                                .0;
                            let mut readings = Vec::new();
                            if sense == direct_sense {
                                readings.push(Some("ばんく".to_owned()));
                            }
                            if sense == 0 {
                                readings.push(Some("ぎんこう".to_owned()));
                            }
                            if sense == 2 {
                                readings.push(Some("ていぼう".to_owned()));
                            }
                            if readings.is_empty() {
                                readings.push(None);
                            }
                            let extra = readings.len() as u32 - 1;
                            readings.into_iter().enumerate().map(move |(i, reading)| {
                                let count = if i == 0 { count - extra } else { 1 };
                                GramFrequencyEntry {
                                    count,
                                    direct_count: count,
                                    disambiguation_key: 0,
                                    gram: language_utils::ReadingTaggedGram {
                                        sense: TaggedGram {
                                            gram: gram(if sense == 1 { surface } else { "bank" }),
                                            sense: NonZeroU32::new(sense),
                                        },
                                        reading,
                                    },
                                }
                            })
                        })
                        .collect(),
                    total_count: senses.iter().map(|&(_, count)| u64::from(count)).sum(),
                },
                encoded_sentences: vec![(
                    surface.into(),
                    SentenceGrams {
                        grams: vec![SentenceGram::Learnable(language_utils::ReadingTaggedGram {
                            sense: TaggedGram {
                                gram: gram(surface),
                                sense: NonZeroU32::new(
                                    senses
                                        .iter()
                                        .find(|&&(sense, _)| sense == 1)
                                        .unwrap_or(&senses[0])
                                        .0,
                                ),
                            },
                            reading: Some("ばんく".into()),
                        })],
                        capitalize_first: false,
                        multiword_terms: vec![language_utils::MultiwordTermMatch {
                            gram: language_utils::ReadingTaggedGram {
                                sense: tagged(0),
                                reading: Some("ぎんこう".into()),
                            },
                            matched_word_indices: vec![0],
                        }]
                        .into_iter()
                        .filter(|term| {
                            senses
                                .iter()
                                .any(|&(sense, _)| NonZeroU32::new(sense) == term.gram.sense.sense)
                        })
                        .collect(),
                        low_confidence_multiword_terms: vec![language_utils::MultiwordTermMatch {
                            gram: language_utils::ReadingTaggedGram {
                                sense: tagged(2),
                                reading: Some("ていぼう".into()),
                            },
                            matched_word_indices: vec![0],
                        }]
                        .into_iter()
                        .filter(|term| {
                            senses
                                .iter()
                                .any(|&(sense, _)| NonZeroU32::new(sense) == term.gram.sense.sense)
                        })
                        .collect(),
                    },
                )],
                gram_dictionary: BTreeMap::new(),
                morphemes: BTreeMap::new(),
                human_audio: FxHashMap::default(),
                pronunciation_audio: FxHashMap::default(),
            },
            Course {
                target_language: Language::English,
                native_language: Language::French,
            },
        )
    }

    fn assert_resolution(pack: &LanguagePack) {
        let request = |word, sense| TaggedGram {
            gram: gram(word),
            sense: NonZeroU32::new(sense),
        };
        let exact = pack.resolve_entry(&request("bank", 1)).unwrap();
        assert_eq!(exact.sense, NonZeroU32::new(1));
        for sense in [0, 99] {
            assert_eq!(
                pack.resolve_entry(&request("bank", sense)).unwrap().sense,
                NonZeroU32::new(2)
            );
        }
        assert_eq!(
            pack.senses_of(exact.gram)
                .iter()
                .map(|g| g.sense.unwrap().get())
                .collect::<Vec<_>>(),
            [2, 1]
        );
        assert!(pack.resolve_entry(&request("foreign", 1)).is_none());
    }

    #[test]
    fn entry_resolution_fallbacks() {
        assert_resolution(&pack());
    }

    /// Regenerating a pack retires sense ids: a card written with an id that
    /// no longer exists must reach the gram's most frequent current sense.
    #[test]
    fn retired_sense_ids_fall_back_to_the_current_most_frequent_sense() {
        let card = |sense| TaggedGram {
            gram: gram("bank"),
            sense: NonZeroU32::new(sense),
        };
        // The gram became monosemous: only the untagged entry is left.
        let monosemous = pack_with_senses(&[(0, 30)]);
        for sense in [1, 2] {
            assert_eq!(monosemous.resolve_entry(&card(sense)).unwrap().sense, None);
        }
        // Every new sense id lies above the retired ones. (Packs list
        // frequencies most frequent first.)
        let renumbered = pack_with_senses(&[(5, 20), (4, 10)]);
        for sense in [0, 1, 2] {
            assert_eq!(
                renumbered.resolve_entry(&card(sense)).unwrap().sense,
                NonZeroU32::new(5)
            );
        }
        assert_eq!(
            renumbered.resolve_entry(&card(4)).unwrap().sense,
            NonZeroU32::new(4)
        );
    }

    #[test]
    fn strokes_live_in_sentences_and_round_trip() {
        let pack = pack();
        let strokes = pack.sentences.strokes.clone();
        assert!(!strokes.is_empty());
        let LanguagePack {
            core, sentences, ..
        } = pack;
        assert_eq!(sentences.strokes, strokes);
        let core_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&*core).unwrap();
        let sentence_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&sentences).unwrap();
        let core = rkyv::from_bytes::<LanguagePackCore, rkyv::rancor::Error>(&core_bytes).unwrap();
        let sentences =
            rkyv::from_bytes::<LanguagePackSentences, rkyv::rancor::Error>(&sentence_bytes)
                .unwrap();
        let pack = LanguagePack::from_parts(core, Some(sentences));
        assert_eq!(pack.sentences.strokes, strokes);
        let core = pack.core;
        assert!(
            LanguagePack::from_parts(core, None)
                .sentences
                .strokes
                .is_empty()
        );
    }

    #[test]
    fn phrase_readings_are_interned_and_the_index_survives_serialization() {
        let pack = pack_with_senses(&[(0, 30), (1, 20), (2, 10)]);
        assert_eq!(pack.sentences.encoded_sentences.len(), 1);
        let resolved = |pack: &LanguagePack| {
            pack.sentences
                .gram_readings
                .iter()
                .map(|(gram, readings)| {
                    (
                        TaggedGram {
                            gram: pack.resolve_gram(&gram.gram),
                            sense: gram.sense,
                        },
                        readings
                            .iter()
                            .map(|r| pack.string_rodeo.resolve(r).to_owned())
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<BTreeMap<_, _>>()
        };
        let expected = resolved(&pack);
        assert_eq!(expected.len(), 3);
        let LanguagePack {
            core, sentences, ..
        } = pack;
        for reading in ["ばんく", "ぎんこう", "ていぼう"] {
            assert!(core.string_rodeo.get(reading).is_some());
        }
        let core_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&*core).unwrap();
        let sentence_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&sentences).unwrap();
        let core = rkyv::from_bytes::<LanguagePackCore, rkyv::rancor::Error>(&core_bytes).unwrap();
        let sentences =
            rkyv::from_bytes::<LanguagePackSentences, rkyv::rancor::Error>(&sentence_bytes)
                .unwrap();
        let pack = LanguagePack::from_parts(core, Some(sentences));
        assert_eq!(resolved(&pack), expected);
        let sentence = pack
            .sentence_grams(pack.sentence_key("bank").unwrap())
            .unwrap();
        for (terms, reading) in [
            (&sentence.multiword_terms, "ぎんこう"),
            (&sentence.low_confidence_multiword_terms, "ていぼう"),
        ] {
            assert_eq!(terms.len(), 1);
            assert_eq!(
                pack.readings_for_sense(terms[0].gram.sense)
                    .collect::<Vec<_>>(),
                [reading]
            );
        }
    }

    #[test]
    fn loading_uses_the_stored_reading_index_without_scanning_sentences() {
        let LanguagePack {
            core,
            mut sentences,
            ..
        } = pack();
        let expected = sentences.gram_readings.clone();
        assert!(!expected.is_empty());
        sentences.encoded_sentences = lasso::CompressedRodeoReader::default();
        let pack = LanguagePack::from_parts(core, Some(sentences));
        assert_eq!(pack.sentences.gram_readings, expected);
    }

    #[test]
    fn phrase_readings_are_canonical_even_when_the_surface_is_inflected() {
        let pack = pack_with_surface(&[(0, 30), (1, 20), (2, 10)], "banked");
        assert_eq!(pack.sentences.gram_readings.len(), 3);
        let mut decoded: FxHashMap<_, _> = pack.decoded_sentences().collect();
        let sentence = decoded.values().next().unwrap();
        for term in sentence
            .multiword_terms
            .iter()
            .chain(&sentence.low_confidence_multiword_terms)
        {
            assert_eq!(
                pack.sentences.gram_readings[&term.gram.sense],
                [term.gram.reading.unwrap()]
            );
        }
        let sentence = decoded.values_mut().next().unwrap();
        let phrase = sentence.multiword_terms[0].gram;
        let token_reading = sentence.all_grams().next().unwrap().reading.unwrap();
        let token = language_utils::ReadingTaggedGram {
            sense: phrase.sense,
            reading: Some(token_reading),
        };
        sentence.grams.extend([
            SentenceGram::Learnable(token),
            SentenceGram::Learnable(token),
            SentenceGram::Learnable(phrase),
        ]);
        let readings = derive_readings(decoded.values(), &pack.string_rodeo);
        assert_eq!(
            readings[&phrase.sense],
            [phrase.reading.unwrap(), token_reading]
        );
        decoded.values_mut().next().unwrap().multiword_terms[0]
            .gram
            .reading = None;
        let readings = derive_readings(decoded.values(), &pack.string_rodeo);
        assert_eq!(
            readings[&phrase.sense],
            [token_reading, phrase.reading.unwrap()]
        );
    }
    #[test]
    fn token_reading_frequency_breaks_ties_lexically() {
        let pack = pack_with_senses(&[(0, 30), (1, 20), (2, 10)]);
        let mut decoded: FxHashMap<_, _> = pack.decoded_sentences().collect();
        let sentence = decoded.values_mut().next().unwrap();
        let SentenceGram::Learnable(entry) = sentence.grams[0] else {
            panic!()
        };
        for term in sentence
            .multiword_terms
            .iter()
            .chain(&sentence.low_confidence_multiword_terms)
        {
            sentence
                .grams
                .push(SentenceGram::Learnable(language_utils::ReadingTaggedGram {
                    sense: entry.sense,
                    reading: term.gram.reading,
                }));
        }
        sentence.grams.push(sentence.grams[1].clone());
        sentence.multiword_terms.clear();
        sentence.low_confidence_multiword_terms.clear();
        let readings = derive_readings(decoded.values(), &pack.string_rodeo);
        assert_eq!(
            readings[&entry.sense]
                .iter()
                .map(|r| pack.string_rodeo.resolve(r))
                .collect::<Vec<_>>(),
            ["ぎんこう", "ていぼう", "ばんく"]
        );
    }

    #[test]
    fn tagged_sentence_split_round_trip() {
        let LanguagePack {
            core, sentences, ..
        } = pack();
        let core_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&*core).unwrap();
        let sentence_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&sentences).unwrap();
        let core = rkyv::from_bytes::<LanguagePackCore, rkyv::rancor::Error>(&core_bytes).unwrap();
        let sentences =
            rkyv::from_bytes::<LanguagePackSentences, rkyv::rancor::Error>(&sentence_bytes)
                .unwrap();
        let pack = LanguagePack::from_parts(core, Some(sentences));
        assert_resolution(&pack);
        let sentence = pack.sentence_key("bank").unwrap();
        let SentenceGram::Learnable(entry) = pack.sentence_grams(sentence).unwrap().grams[0] else {
            panic!("expected a learnable gram")
        };
        assert_eq!(entry.sense.sense, NonZeroU32::new(1));
        assert_eq!(pack.resolve_gram(&entry.sense.gram), gram("bank"));
        assert_eq!(pack.string_rodeo.resolve(&entry.reading.unwrap()), "ばんく");
        assert_eq!(
            pack.readings_for_sense(entry.sense).collect::<Vec<_>>(),
            ["ばんく"]
        );
        let core = pack.core;
        assert_resolution(&LanguagePack::from_parts(core, None));
    }
}
