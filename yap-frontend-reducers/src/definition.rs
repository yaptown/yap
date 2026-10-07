//! Shared definition presentation, matching the web.

use language_utils::GramDefinition;
use language_utils::features::Morphology;

/// All displayable readings, with agreement kept together per alternative.
#[bridgerton::bridge]
pub fn morphology_label(morphology: Vec<Morphology>) -> String {
    language_utils::morphology_label::morphology_label_with(&morphology, |label| match label {
        "0th-person" => yap_i18n::tr!("morphology-0th-person"),
        "1st-person" => yap_i18n::tr!("morphology-1st-person"),
        "2nd-person" => yap_i18n::tr!("morphology-2nd-person"),
        "3rd-person" => yap_i18n::tr!("morphology-3rd-person"),
        "4th-person" => yap_i18n::tr!("morphology-4th-person"),
        "singular" => yap_i18n::tr!("morphology-singular"),
        "plural" => yap_i18n::tr!("morphology-plural"),
        "dual" => yap_i18n::tr!("morphology-dual"),
        "trial" => yap_i18n::tr!("morphology-trial"),
        "paucal" => yap_i18n::tr!("morphology-paucal"),
        "greater paucal" => yap_i18n::tr!("morphology-greater-paucal"),
        "greater plural" => yap_i18n::tr!("morphology-greater-plural"),
        "inverse number" => yap_i18n::tr!("morphology-inverse-number"),
        "count plural" => yap_i18n::tr!("morphology-count-plural"),
        "plural-only" => yap_i18n::tr!("morphology-plural-only"),
        "collective" => yap_i18n::tr!("morphology-collective"),
        "intimate" => yap_i18n::tr!("morphology-intimate"),
        "informal" => yap_i18n::tr!("morphology-informal"),
        "formal" => yap_i18n::tr!("morphology-formal"),
        "elevated" => yap_i18n::tr!("morphology-elevated"),
        "humble" => yap_i18n::tr!("morphology-humble"),
        "past tense" => yap_i18n::tr!("morphology-past-tense"),
        "present tense" => yap_i18n::tr!("morphology-present-tense"),
        "future tense" => yap_i18n::tr!("morphology-future-tense"),
        "imperfect tense" => yap_i18n::tr!("morphology-imperfect-tense"),
        "pluperfect tense" => yap_i18n::tr!("morphology-pluperfect-tense"),
        "masculine" => yap_i18n::tr!("morphology-masculine"),
        "feminine" => yap_i18n::tr!("morphology-feminine"),
        "neuter" => yap_i18n::tr!("morphology-neuter"),
        "common" => yap_i18n::tr!("morphology-common"),
        "nominative" => yap_i18n::tr!("morphology-nominative"),
        "accusative" => yap_i18n::tr!("morphology-accusative"),
        "absolutive" => yap_i18n::tr!("morphology-absolutive"),
        "ergative" => yap_i18n::tr!("morphology-ergative"),
        "dative" => yap_i18n::tr!("morphology-dative"),
        "genitive" => yap_i18n::tr!("morphology-genitive"),
        "vocative" => yap_i18n::tr!("morphology-vocative"),
        "instrumental" => yap_i18n::tr!("morphology-instrumental"),
        "partitive" => yap_i18n::tr!("morphology-partitive"),
        "distributive" => yap_i18n::tr!("morphology-distributive"),
        "essive" => yap_i18n::tr!("morphology-essive"),
        "translative" => yap_i18n::tr!("morphology-translative"),
        "comitative" => yap_i18n::tr!("morphology-comitative"),
        "abessive" => yap_i18n::tr!("morphology-abessive"),
        "causative" => yap_i18n::tr!("morphology-causative"),
        "benefactive" => yap_i18n::tr!("morphology-benefactive"),
        "considerative" => yap_i18n::tr!("morphology-considerative"),
        "comparative" => yap_i18n::tr!("morphology-comparative"),
        "equative" => yap_i18n::tr!("morphology-equative"),
        "locative" => yap_i18n::tr!("morphology-locative"),
        "lative" => yap_i18n::tr!("morphology-lative"),
        "terminative" => yap_i18n::tr!("morphology-terminative"),
        "inessive" => yap_i18n::tr!("morphology-inessive"),
        "illative" => yap_i18n::tr!("morphology-illative"),
        "elative" => yap_i18n::tr!("morphology-elative"),
        "additive" => yap_i18n::tr!("morphology-additive"),
        "adessive" => yap_i18n::tr!("morphology-adessive"),
        "allative" => yap_i18n::tr!("morphology-allative"),
        "ablative" => yap_i18n::tr!("morphology-ablative"),
        "superessive" => yap_i18n::tr!("morphology-superessive"),
        "superlative" => yap_i18n::tr!("morphology-superlative"),
        "delative" => yap_i18n::tr!("morphology-delative"),
        "subessive" => yap_i18n::tr!("morphology-subessive"),
        "sublative" => yap_i18n::tr!("morphology-sublative"),
        "subelative" => yap_i18n::tr!("morphology-subelative"),
        "perlative" => yap_i18n::tr!("morphology-perlative"),
        "temporal" => yap_i18n::tr!("morphology-temporal"),
        " " => yap_i18n::tr!("common-word-separator"),
        ", " => yap_i18n::tr!("common-list-comma"),
        " / " => yap_i18n::tr!("common-alternative-separator"),
        _ => panic!("unknown morphology display label: {label}"),
    })
}

/// An example sentence in the target and native languages.
#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema, Clone, Debug, PartialEq)]
pub struct SenseExample {
    pub target: String,
    pub native: String,
}

#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema, Clone, Debug, PartialEq)]
pub struct DefinitionSense {
    pub meaning: String,
    pub note: Option<String>,
    pub example: Option<SenseExample>,
}

/// A definition as every surface shows it, with a target-language headword.
/// An empty morphology label means there is nothing to show.
#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema, Clone, Debug, PartialEq)]
pub struct DefinitionView {
    pub headword: String,
    pub is_phrase: bool,
    pub morphology_label: String,
    pub senses: Vec<DefinitionSense>,
}

#[bridgerton::bridge]
#[bridgerton::stable]
pub fn definition_view(definition: GramDefinition) -> DefinitionView {
    let example = |target: String, native: String| {
        (!target.is_empty()).then_some(SenseExample { target, native })
    };
    match definition {
        GramDefinition::Dictionary(entry) => DefinitionView {
            headword: entry.target_language_word,
            is_phrase: false,
            morphology_label: morphology_label(entry.morphology),
            senses: entry
                .definitions
                .into_iter()
                .map(|sense| DefinitionSense {
                    meaning: sense.native,
                    note: sense.note.filter(|note| !note.is_empty()),
                    example: example(
                        sense.example_sentence_target_language,
                        sense.example_sentence_native_language,
                    ),
                })
                .collect(),
        },
        GramDefinition::Phrasebook(entry) => DefinitionView {
            headword: entry.target_language_multi_word_term,
            is_phrase: true,
            morphology_label: String::new(),
            senses: vec![DefinitionSense {
                meaning: entry.meaning,
                // The canonical web presentation ignores phrasebook additional_notes.
                note: None,
                example: example(entry.target_language_example, entry.native_language_example),
            }],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use language_utils::features::{Aspect, Case, Gender, Mood, Number, Person, Polite, Tense};

    #[test]
    fn empty_morphology_has_no_label() {
        assert_eq!(morphology_label(vec![Morphology::default()]), "");
    }

    #[test]
    fn person_precedes_tense() {
        assert_eq!(
            morphology_label(vec![Morphology {
                person: Some(Person::First),
                tense: Some(Tense::Past),
                ..Default::default()
            }]),
            "1st-person, past tense"
        );
    }

    #[test]
    fn displays_only_web_fields_in_order() {
        assert_eq!(
            morphology_label(vec![Morphology {
                person: Some(Person::Third),
                gender: Some(Gender::Feminine),
                tense: Some(Tense::Pluperfect),
                politeness: Some(Polite::Humb),
                case: Some(Case::Superessive),
                number: Some(Number::Plural),
                mood: Some(Mood::Indicative),
                aspect: Some(Aspect::Perfect),
            }]),
            "3rd-person plural feminine humble, pluperfect tense, superessive"
        );
    }

    #[test]
    fn dictionary_projection_preserves_senses_and_all_morphology_readings() {
        let definition = GramDefinition::Dictionary(language_utils::DictionaryEntry {
            target_language_word: "parle".into(),
            definitions: vec![
                language_utils::TargetToNativeWord {
                    native: "speak".into(),
                    note: Some("a note".into()),
                    example_sentence_target_language: "Je parle.".into(),
                    example_sentence_native_language: "I speak.".into(),
                    cognate: false,
                    false_cognate: false,
                },
                language_utils::TargetToNativeWord {
                    native: "talk".into(),
                    note: Some(String::new()),
                    example_sentence_target_language: String::new(),
                    example_sentence_native_language: "Native alone is not an example".into(),
                    cognate: false,
                    false_cognate: false,
                },
            ],
            morphology: vec![
                Morphology {
                    person: Some(Person::First),
                    ..Default::default()
                },
                Morphology {
                    tense: Some(Tense::Past),
                    ..Default::default()
                },
            ],
            segments: vec![],
        });
        assert_eq!(
            definition_view(definition.clone()),
            DefinitionView {
                headword: "parle".into(),
                is_phrase: false,
                morphology_label: "1st-person / past tense".into(),
                senses: vec![
                    DefinitionSense {
                        meaning: "speak".into(),
                        note: Some("a note".into()),
                        example: Some(SenseExample {
                            target: "Je parle.".into(),
                            native: "I speak.".into()
                        }),
                    },
                    DefinitionSense {
                        meaning: "talk".into(),
                        note: None,
                        example: None
                    },
                ],
            }
        );
        let GramDefinition::Dictionary(mut entry) = definition else {
            unreachable!()
        };
        entry.morphology[0] = Morphology::default();
        assert_eq!(
            definition_view(GramDefinition::Dictionary(entry.clone())).morphology_label,
            "past tense"
        );
        entry.morphology.clear();
        entry.definitions[0].note = None;
        let view = definition_view(GramDefinition::Dictionary(entry));
        assert!(view.morphology_label.is_empty());
        assert_eq!(view.senses[0].note, None);
    }

    #[test]
    fn phrasebook_projection_has_one_sense_and_ignores_additional_notes() {
        let mut entry = language_utils::PhrasebookDefinitionEntry {
            target_language_multi_word_term: "tout à fait".into(),
            meaning: "absolutely".into(),
            additional_notes: "Not displayed by the canonical web presentation".into(),
            target_language_example: "Tout à fait !".into(),
            native_language_example: String::new(),
            informal: false,
            compositional: false,
            cognate: false,
            false_cognate: false,
            can_be_translated_literally: false,
        };
        assert_eq!(
            definition_view(GramDefinition::Phrasebook(entry.clone())),
            DefinitionView {
                headword: "tout à fait".into(),
                is_phrase: true,
                morphology_label: String::new(),
                senses: vec![DefinitionSense {
                    meaning: "absolutely".into(),
                    note: None,
                    example: Some(SenseExample {
                        target: "Tout à fait !".into(),
                        native: String::new()
                    }),
                }],
            }
        );
        entry.target_language_example.clear();
        entry.native_language_example = "Absolutely!".into();
        assert_eq!(
            definition_view(GramDefinition::Phrasebook(entry)).senses[0].example,
            None
        );
    }
}
