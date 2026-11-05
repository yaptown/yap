use crate::{ConsolidatedLanguageData, GramFrequencyList, build};
use language_utils::features::{Morphology, Person};
use language_utils::*;

#[test]
fn cross_gram_pages_and_prefixed_text_survive_split() {
    use language_utils::{
        Course, GramVocabEntry, ReadingGramFrequencyEntry as GramFrequencyEntry, TaggedGram,
        language_pack::LanguagePack,
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
                    gram: language_utils::ReadingTaggedGram::without_reading(entry.clone()),
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
    let pack = build(
        data,
        Course {
            target_language: Language::French,
            native_language: Language::English,
        },
    );
    let a = pack.resolve_entry(&est).unwrap();
    let b = pack.resolve_entry(&phrase).unwrap();
    assert_eq!(
        pack.sentences.prefixed_text[&a.gram],
        pack.sentences.prefixed_text[&b.gram]
    );
    assert_eq!(
        pack.sentences.grams_with_prefixed_text[&pack.sentences.prefixed_text[&a.gram]],
        vec![a, b]
    );
    assert!(pack.is_visible(a, Language::French));
    assert!(pack.is_visible(b, Language::French));
    assert!(pack.is_visible(pack.resolve_entry(&noun).unwrap(), Language::French));
    assert!(!pack.is_visible(pack.resolve_entry(&particle).unwrap(), Language::French));
    let LanguagePack {
        core, sentences, ..
    } = pack;
    assert!(core.string_rodeo.get("il est").is_some());
    let core = LanguagePack::from_parts(core, None);
    assert!(core.sentences.prefixed_text.is_empty());
    let full = core.with_sentences(sentences);
    let a = full.resolve_entry(&est).unwrap();
    let b = full.resolve_entry(&phrase).unwrap();
    let text = full.sentences.prefixed_text[&a.gram];
    assert_eq!(full.string_rodeo.resolve(&text), "il est");
    assert_eq!(full.sentences.grams_with_prefixed_text[&text], vec![a, b]);
    assert_eq!(full.redundant_with(a), vec![a, b]);
}
