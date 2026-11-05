use language_utils::features::Morphology;
use language_utils::language_pack::{
    DefinitionListSpur, DefinitionLists, DefinitionMorphologies, DefinitionSegments,
    DefinitionTextSpur, MorphologyListSpur, SegmentListSpur, StoredGramDefinition,
};
use language_utils::{DefinitionTextKind, GramDefinition};
use language_utils::{MorphemeSegment, SpurGram, TaggedGram, TargetToNativeWord};
use rustc_hash::FxHashMap;
use std::collections::HashSet;

/// Unique target/native values; one traversal owns the classification of fields.
pub(crate) fn text_sets<'a>(
    definitions: impl Iterator<Item = &'a GramDefinition>,
) -> [HashSet<String>; 2] {
    let mut target = HashSet::new();
    let mut native = HashSet::new();
    for definition in definitions {
        definition.map_text(|text, kind| {
            match kind {
                DefinitionTextKind::Target => &mut target,
                DefinitionTextKind::Native => &mut native,
            }
            .insert(text.as_str());
        });
    }
    [target, native].map(|set| set.into_iter().map(str::to_owned).collect())
}

pub(crate) struct InternedDefinitions {
    pub entries: FxHashMap<TaggedGram<SpurGram>, StoredGramDefinition>,
    pub definitions: DefinitionLists,
    pub morphology: DefinitionMorphologies,
    pub segments: DefinitionSegments,
}

pub(crate) fn intern_lists(
    entries: impl Iterator<Item = (TaggedGram<SpurGram>, GramDefinition<DefinitionTextSpur>)>,
) -> InternedDefinitions {
    let mut definitions =
        lasso::Rodeo::<Vec<TargetToNativeWord<DefinitionTextSpur>>, DefinitionListSpur>::new();
    let mut morphology = lasso::Rodeo::<Vec<Morphology>, MorphologyListSpur>::new();
    let mut segments =
        lasso::Rodeo::<Vec<MorphemeSegment<DefinitionTextSpur>>, SegmentListSpur>::new();
    let entries = entries
        .map(|(key, definition)| {
            (
                key,
                definition.map_lists(
                    |list| definitions.get_or_intern(list),
                    |list| morphology.get_or_intern(list),
                    |list| segments.get_or_intern(list),
                ),
            )
        })
        .collect();
    InternedDefinitions {
        entries,
        definitions: definitions.into_resolver(),
        morphology: morphology.into_resolver(),
        segments: segments.into_resolver(),
    }
}
