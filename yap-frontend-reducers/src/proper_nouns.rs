//! Finished proper-noun hint rows, shared by sentence challenge hosts.
use language_utils::ProperNounDefinition;
use serde::{Deserialize, Serialize};

/// One run of text; target-language spans are proper nouns, the rest is muted connective copy.
#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TextSpan {
    pub text: String,
    pub target_language: bool,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProperNounGroup {
    pub spans: Vec<TextSpan>,
}

fn group(words: &[&str], label: String) -> ProperNounGroup {
    let mut spans = Vec::new();
    for (index, word) in words.iter().enumerate() {
        if index > 0 {
            spans.push(TextSpan {
                text: if index == words.len() - 1 {
                    " and "
                } else {
                    ", "
                }
                .into(),
                target_language: false,
            });
        }
        spans.push(TextSpan {
            text: (*word).into(),
            target_language: true,
        });
    }
    spans.push(TextSpan {
        text: format!(": {label}"),
        target_language: false,
    });
    ProperNounGroup { spans }
}

pub fn proper_noun_groups(definitions: &[(String, ProperNounDefinition)]) -> Vec<ProperNounGroup> {
    let mut people = Vec::new();
    let mut places = Vec::new();
    let mut organizations = Vec::new();
    let mut translated_descriptions = Vec::new();
    let mut translations = Vec::new();
    let mut descriptions = Vec::new();

    for (noun, definition) in definitions {
        let translation = &definition.learner_native_language_translation;
        // Match the web's truthiness check: an empty description is absent.
        let description = definition.description.as_deref().filter(|s| !s.is_empty());
        if translation == noun {
            if let Some(description) = description {
                descriptions.push(group(&[noun], description.into()));
            } else if definition.is_person_name {
                people.push(noun.as_str());
            } else if definition.is_place_name {
                places.push(noun.as_str());
            } else if definition.is_organization_name {
                organizations.push(noun.as_str());
            }
        } else if let Some(description) = description {
            translated_descriptions.push(group(&[noun], format!("{translation} ({description})")));
        } else {
            translations.push(group(&[noun], translation.clone()));
        }
    }

    let mut groups = Vec::new();
    for (words, singular, plural) in [
        (people, "person", "people"),
        (places, "place", "places"),
        (organizations, "organization", "organizations"),
    ] {
        if !words.is_empty() {
            let label = if words.len() == 1 { singular } else { plural };
            groups.push(group(&words, label.into()));
        }
    }
    groups.extend(translated_descriptions);
    groups.extend(translations);
    groups.extend(descriptions);
    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    fn definition(
        translation: &str,
        description: Option<&str>,
        flags: [bool; 3],
    ) -> ProperNounDefinition {
        ProperNounDefinition {
            learner_native_language_translation: translation.into(),
            description: description.map(String::from),
            is_person_name: flags[0],
            is_place_name: flags[1],
            is_organization_name: flags[2],
            is_other: false,
        }
    }

    fn text(group: &ProperNounGroup) -> String {
        group.spans.iter().map(|span| span.text.as_str()).collect()
    }

    #[test]
    fn mixed_buckets_preserve_web_order_and_span_styles() {
        let definitions = vec![
            (
                "Description".into(),
                definition("Description", Some("a title"), [true; 3]),
            ),
            (
                "ローマ".into(),
                definition("Rome", None, [false, true, false]),
            ),
            ("Org".into(), definition("Org", None, [false, false, true])),
            (
                "パリ".into(),
                definition("Paris", Some("a city"), [false, true, false]),
            ),
            (
                "Place".into(),
                definition("Place", Some(""), [false, true, true]),
            ),
            ("A".into(), definition("A", None, [true; 3])),
            ("B".into(), definition("B", Some(""), [true, false, false])),
            (
                "東京".into(),
                definition("Tokyo", Some(""), [false, true, false]),
            ),
            ("Other".into(), definition("Other", None, [false; 3])),
        ];
        let groups = proper_noun_groups(&definitions);
        assert_eq!(
            groups.iter().map(text).collect::<Vec<_>>(),
            [
                "A and B: people",
                "Place: place",
                "Org: organization",
                "パリ: Paris (a city)",
                "ローマ: Rome",
                "東京: Tokyo",
                "Description: a title",
            ]
        );
        assert_eq!(
            groups[0].spans,
            vec![
                TextSpan {
                    text: "A".into(),
                    target_language: true
                },
                TextSpan {
                    text: " and ".into(),
                    target_language: false
                },
                TextSpan {
                    text: "B".into(),
                    target_language: true
                },
                TextSpan {
                    text: ": people".into(),
                    target_language: false
                },
            ]
        );
        for group in &groups[1..] {
            assert_eq!(group.spans.len(), 2);
            assert!(group.spans[0].target_language);
            assert!(!group.spans[1].target_language);
        }
    }

    #[test]
    fn joins_one_two_and_three_names_without_an_oxford_comma() {
        for (flags, singular, plural) in [
            ([true, false, false], "person", "people"),
            ([false, true, false], "place", "places"),
            ([false, false, true], "organization", "organizations"),
        ] {
            let definitions: Vec<_> = ["A", "B", "C"]
                .into_iter()
                .map(|word| (word.into(), definition(word, None, flags)))
                .collect();
            for (count, expected) in [
                (1, format!("A: {singular}")),
                (2, format!("A and B: {plural}")),
                (3, format!("A, B and C: {plural}")),
            ] {
                assert_eq!(
                    text(&proper_noun_groups(&definitions[..count])[0]),
                    expected
                );
            }
        }
    }

    #[test]
    fn drops_unclassified_untranslated_names_and_empty_input() {
        let mut other = definition("Other", Some(""), [false; 3]);
        other.is_other = true;
        assert!(proper_noun_groups(&[("Other".into(), other)]).is_empty());
        assert!(proper_noun_groups(&[]).is_empty());
    }
}
