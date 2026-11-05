use language_utils::word_prefix::prefixed_text;
use language_utils::{Gram, GramDefinition, Language};

/// Choose one front per bare gram from its most frequent sense.
/// `frequencies` is in the pack's frequency order (including its tie order).
pub fn prefixed_texts(
    frequencies: &[language_utils::GramFrequencyEntry<String>],
    dictionary: &std::collections::BTreeMap<
        language_utils::TaggedGram<Gram<String>>,
        language_utils::DictionaryEntry,
    >,
    phrases: &std::collections::BTreeMap<
        language_utils::TaggedGram<Gram<String>>,
        language_utils::PhrasebookDefinitionEntry,
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
