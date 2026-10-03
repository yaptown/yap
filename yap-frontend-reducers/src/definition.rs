//! Shared definition presentation, matching the web.

use language_utils::GramDefinition;
use language_utils::features::Morphology;

/// All displayable readings, with agreement kept together per alternative.
#[bridgerton::bridge]
pub fn morphology_label(morphology: Vec<Morphology>) -> String {
    language_utils::morphology_label::morphology_label(&morphology)
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
