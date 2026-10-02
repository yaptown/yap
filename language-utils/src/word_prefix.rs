use crate::features::{Morphology, WordPrefix};
use crate::{Atom, Gram, GramDefinition, Language, WordType};

/// Grammatical context displayed before a written gram.
pub fn compute_word_prefix(
    gram: &Gram<String>,
    definition: &GramDefinition,
    language: Language,
) -> Option<WordPrefix> {
    match gram.atoms() {
        [Atom::Tok(word)] => {
            let WordType::Heteronym(h) = &word.word_type else {
                return None;
            };
            let GramDefinition::Dictionary(d) = definition else {
                return None;
            };
            d.morphology.first()?.get_prefix(&h.word, h.pos, language)
        }
        [] | [_] => None,
        _ => Morphology::get_multiword_prefix(gram, language),
    }
}

/// Choose one front per bare gram from its most frequent sense.
/// `frequencies` is in the pack's frequency order (including its tie order).
pub fn prefixed_texts(
    frequencies: &[crate::GramFrequencyEntry<String>],
    dictionary: &std::collections::BTreeMap<
        crate::TaggedGram<Gram<String>>,
        crate::DictionaryEntry,
    >,
    phrases: &std::collections::BTreeMap<
        crate::TaggedGram<Gram<String>>,
        crate::PhrasebookDefinitionEntry,
    >,
    language: Language,
) -> std::collections::BTreeMap<Gram<String>, String> {
    let mut texts = std::collections::BTreeMap::new();
    for entry in frequencies {
        if texts.contains_key(&entry.gram.gram) {
            continue;
        }
        let definition = dictionary
            .get(&entry.gram)
            .cloned()
            .map(GramDefinition::Dictionary)
            .or_else(|| {
                phrases
                    .get(&entry.gram)
                    .cloned()
                    .map(GramDefinition::Phrasebook)
            });
        let text = definition
            .map(|definition| prefixed_text(&entry.gram.gram, &definition, language))
            .unwrap_or_else(|| entry.gram.gram.to_display_string(language));
        texts.insert(entry.gram.gram.clone(), text);
    }
    texts
}

/// Exact written front; callers supply the most frequent sense's definition.
pub fn prefixed_text(
    gram: &Gram<String>,
    definition: &GramDefinition,
    language: Language,
) -> String {
    let display = gram.to_display_string(language);
    match compute_word_prefix(gram, definition, language) {
        Some(prefix) => format!("{}{}{display}", prefix.prefix, prefix.separator),
        None => display,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::{Number, Person};
    use crate::{DictionaryEntry, Heteronym, PartOfSpeech, Word};

    #[test]
    fn cross_gram_pages_and_prefix_extension_survive_split() {
        use crate::{
            ConsolidatedLanguageData, Course, GramFrequencyEntry, GramFrequencyList,
            GramVocabEntry, TaggedGram, language_pack::LanguagePack,
        };
        let member = |text: &str, pos| TaggedGram {
            gram: Gram(
                text.split(' ')
                    .map(|text| {
                        Atom::Tok(Word {
                            text: text.into(),
                            word_type: WordType::Heteronym(Heteronym {
                                word: text.into(),
                                lemma: text.into(),
                                pos,
                            }),
                        })
                    })
                    .collect(),
            ),
            sense: None,
        };
        let est = member("est", PartOfSpeech::Verb);
        let phrase = member("il est", PartOfSpeech::Verb);
        let noun = member("の", PartOfSpeech::Adp);
        let particle = member("の", PartOfSpeech::Part);
        let entries = [est.clone(), phrase.clone(), noun.clone(), particle.clone()];
        let dictionary = entries
            .iter()
            .map(|entry| {
                (
                    entry.clone(),
                    DictionaryEntry {
                        target_language_word: entry.gram.to_display_string(Language::French),
                        definitions: vec![],
                        segments: vec![],
                        morphology: if *entry == est {
                            vec![Morphology {
                                person: Some(Person::Third),
                                ..Default::default()
                            }]
                        } else {
                            vec![]
                        },
                    },
                )
            })
            .collect();
        let data = ConsolidatedLanguageData {
            gram_vocabulary: entries
                .iter()
                .map(|entry| GramVocabEntry {
                    atoms: entry.gram.clone(),
                    frequency: 10,
                })
                .collect(),
            gram_frequencies: GramFrequencyList {
                entries: entries
                    .iter()
                    .enumerate()
                    .map(|(index, entry)| GramFrequencyEntry {
                        gram: entry.clone(),
                        count: 40 - index as u32,
                        direct_count: 40 - index as u32,
                        disambiguation_key: index as u32,
                    })
                    .collect(),
                total_count: 154,
            },
            gram_dictionary: dictionary,
            redundant_senses: [
                ("il est".into(), vec![vec![est.clone(), phrase.clone()]]),
                ("の".into(), vec![vec![noun.clone(), particle.clone()]]),
            ]
            .into_iter()
            .collect(),
            ..Default::default()
        };
        let pack = LanguagePack::new(
            data,
            Course {
                target_language: Language::French,
                native_language: Language::English,
            },
        );
        let a = pack.resolve_entry(&est).unwrap();
        let b = pack.resolve_entry(&phrase).unwrap();
        assert_eq!(pack.prefixed_text[&a.gram], pack.prefixed_text[&b.gram]);
        assert_eq!(
            pack.grams_with_prefixed_text[&pack.prefixed_text[&a.gram]],
            vec![a, b]
        );
        assert!(pack.is_visible(a, Language::French));
        assert!(pack.is_visible(b, Language::French));
        assert!(pack.is_visible(pack.resolve_entry(&noun).unwrap(), Language::French));
        assert!(!pack.is_visible(pack.resolve_entry(&particle).unwrap(), Language::French));
        let (core, sentences) = pack.split();
        assert!(sentences.string_extension.contains(&"il est".into()));
        let core = LanguagePack::from_parts(core, None);
        assert!(core.prefixed_text.is_empty());
        let full = core.with_sentences(sentences);
        let a = full.resolve_entry(&est).unwrap();
        let b = full.resolve_entry(&phrase).unwrap();
        let text = full.prefixed_text[&a.gram];
        assert_eq!(full.string_rodeo.resolve(&text), "il est");
        assert_eq!(full.grams_with_prefixed_text[&text], vec![a, b]);
        assert_eq!(full.redundant_with(a), vec![a, b]);
    }

    #[test]
    fn french_third_person_est_has_same_front_as_il_est() {
        let gram = |words: &[(&str, PartOfSpeech)]| {
            Gram(
                words
                    .iter()
                    .map(|(text, pos)| {
                        Atom::Tok(Word {
                            text: (*text).into(),
                            word_type: WordType::Heteronym(Heteronym {
                                word: (*text).into(),
                                lemma: (*text).into(),
                                pos: *pos,
                            }),
                        })
                    })
                    .collect(),
            )
        };
        let est = gram(&[("est", PartOfSpeech::Verb)]);
        let phrase = gram(&[("il", PartOfSpeech::Pron), ("est", PartOfSpeech::Verb)]);
        let definition = GramDefinition::Dictionary(DictionaryEntry {
            target_language_word: "est".into(),
            definitions: vec![],
            segments: vec![],
            morphology: vec![Morphology {
                person: Some(Person::Third),
                number: Some(Number::Singular),
                ..Default::default()
            }],
        });
        assert_eq!(prefixed_text(&est, &definition, Language::French), "il est");
        assert_eq!(
            prefixed_text(&est, &definition, Language::French),
            prefixed_text(&phrase, &definition, Language::French)
        );
    }
}
