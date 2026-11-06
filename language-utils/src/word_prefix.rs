use crate::features::{Morphology, WordPrefix};
use crate::{Atom, Gram, GramDefinition, Language, WordType};

/// Grammatical context displayed before a written gram.
pub fn compute_word_prefix(
    gram: &Gram<String>,
    morphology: Option<&Morphology>,
    language: Language,
) -> Option<WordPrefix> {
    match gram.atoms() {
        [Atom::Tok(word)] => {
            let WordType::Heteronym(h) = &word.word_type else {
                return None;
            };
            morphology?.get_prefix(&h.word, h.pos, language)
        }
        [] | [_] => None,
        _ => Morphology::get_multiword_prefix(gram, language),
    }
}

/// Exact written front; callers supply the most frequent sense's definition.
pub fn prefixed_text(
    gram: &Gram<String>,
    definition: &GramDefinition,
    language: Language,
) -> String {
    let display = gram.to_display_string(language);
    let morphology = match definition {
        GramDefinition::Dictionary(entry) => entry.morphology.first(),
        GramDefinition::Phrasebook(_) => None,
    };
    match compute_word_prefix(gram, morphology, language) {
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
