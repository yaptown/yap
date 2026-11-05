use crate::{
    DictionaryEntry, GramDefinition, MorphemeSegment, PhrasebookDefinitionEntry, TargetToNativeWord,
};

/// The language of a definition field; segment labels belong with target text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefinitionTextKind {
    Target,
    Native,
}

impl<S, D, M, G> GramDefinition<S, D, M, G> {
    /// Change list storage without touching text or other metadata.
    pub fn map_lists<D2, M2, G2>(
        self,
        definitions: impl FnOnce(D) -> D2,
        morphology: impl FnOnce(M) -> M2,
        segments: impl FnOnce(G) -> G2,
    ) -> GramDefinition<S, D2, M2, G2> {
        match self {
            Self::Dictionary(entry) => GramDefinition::Dictionary(DictionaryEntry {
                target_language_word: entry.target_language_word,
                definitions: definitions(entry.definitions),
                morphology: morphology(entry.morphology),
                segments: segments(entry.segments),
            }),
            Self::Phrasebook(entry) => GramDefinition::Phrasebook(entry),
        }
    }
}

impl<S, D, M, G> GramDefinition<S, D, M, G>
where
    D: AsRef<[TargetToNativeWord<S>]>,
    M: AsRef<[crate::features::Morphology]>,
    G: AsRef<[MorphemeSegment<S>]>,
{
    /// Map every text field while preserving flags, morphology and field order.
    /// The same traversal collects, interns and resolves definition text.
    pub fn map_text<'a, T>(
        &'a self,
        mut map: impl FnMut(&'a S, DefinitionTextKind) -> T,
    ) -> GramDefinition<T> {
        use DefinitionTextKind::{Native, Target};
        match self {
            Self::Dictionary(entry) => GramDefinition::Dictionary(DictionaryEntry {
                target_language_word: map(&entry.target_language_word, Target),
                definitions: entry
                    .definitions
                    .as_ref()
                    .iter()
                    .map(|definition| TargetToNativeWord {
                        native: map(&definition.native, Native),
                        note: definition.note.as_ref().map(|note| map(note, Native)),
                        note_is_curated: definition.note_is_curated,
                        example_sentence_target_language: map(
                            &definition.example_sentence_target_language,
                            Target,
                        ),
                        example_sentence_native_language: map(
                            &definition.example_sentence_native_language,
                            Native,
                        ),
                        cognate: definition.cognate,
                        false_cognate: definition.false_cognate,
                    })
                    .collect(),
                morphology: entry.morphology.as_ref().to_vec(),
                segments: entry
                    .segments
                    .as_ref()
                    .iter()
                    .map(|segment| MorphemeSegment {
                        surface: map(&segment.surface, Target),
                        canonical: map(&segment.canonical, Target),
                        tag: segment.tag.as_ref().map(|tag| map(tag, Target)),
                    })
                    .collect(),
            }),
            Self::Phrasebook(entry) => GramDefinition::Phrasebook(PhrasebookDefinitionEntry {
                target_language_multi_word_term: map(
                    &entry.target_language_multi_word_term,
                    Target,
                ),
                meaning: map(&entry.meaning, Native),
                additional_notes: map(&entry.additional_notes, Native),
                note_is_curated: entry.note_is_curated,
                target_language_example: map(&entry.target_language_example, Target),
                native_language_example: map(&entry.native_language_example, Native),
                informal: entry.informal,
                compositional: entry.compositional,
                cognate: entry.cognate,
                false_cognate: entry.false_cognate,
                can_be_translated_literally: entry.can_be_translated_literally,
            }),
        }
    }
}
