use language_utils::indexmap::IndexMap;
use language_utils::language_pack::{EaseOrder, FrequencyList};
use language_utils::{
    Frequency, Gram, GramFrequencyEntry, ReadingGramFrequencyEntry, SpurGram, SpurReadingGram,
    TaggedGram,
};
use rustc_hash::FxHashMap;
use std::{collections::BTreeMap, hash::Hash};

/// Derived sense view for dictionary and morphology consumers.
pub fn aggregate_sense_frequencies(
    entries: &[ReadingGramFrequencyEntry<String>],
) -> Vec<GramFrequencyEntry<String>> {
    let mut senses: BTreeMap<TaggedGram<Gram<String>>, GramFrequencyEntry<String>> =
        BTreeMap::new();
    for entry in entries {
        let total = senses
            .entry(entry.gram.sense.clone())
            .or_insert(GramFrequencyEntry {
                gram: entry.gram.sense.clone(),
                count: 0,
                direct_count: 0,
                disambiguation_key: entry.disambiguation_key,
            });
        total.count += entry.count;
        total.direct_count += entry.direct_count;
    }
    let mut entries: Vec<_> = senses.into_values().collect();
    entries.sort_by_key(|entry| std::cmp::Reverse(entry.clone()));
    entries
}

pub(crate) fn ease_order<K: Copy + Eq + Hash + Ord>(
    entries: impl Iterator<Item = (K, f32)>,
) -> EaseOrder<K> {
    let mut entries: Vec<_> = entries.collect();
    entries.sort_unstable_by(|(a, x), (b, y)| y.total_cmp(x).then_with(|| a.cmp(b)));
    let ranks = entries
        .iter()
        .enumerate()
        .map(|(i, (key, _))| (*key, i as u32))
        .collect();
    EaseOrder { entries, ranks }
}

pub(crate) fn ease_orders(
    frequencies: &FrequencyList<SpurReadingGram>,
) -> (EaseOrder<SpurReadingGram>, EaseOrder<SpurGram>) {
    let written = ease_order(
        frequencies
            .entries
            .iter()
            .map(|(key, frequency)| (*key, frequency.ease)),
    );
    let mut listening: FxHashMap<SpurGram, f32> = FxHashMap::default();
    for (key, frequency) in frequencies.entries.iter() {
        listening
            .entry(key.sense.gram)
            .and_modify(|ease| *ease = ease.max(frequency.ease))
            .or_insert(frequency.ease);
    }
    (written, ease_order(listening.into_iter()))
}

pub(crate) fn sense_index(
    frequencies: &IndexMap<TaggedGram<SpurGram>, Frequency>,
) -> FxHashMap<SpurGram, Vec<TaggedGram<SpurGram>>> {
    let mut index: FxHashMap<SpurGram, Vec<TaggedGram<SpurGram>>> = FxHashMap::default();
    for (entry, _) in frequencies.iter() {
        index.entry(entry.gram).or_default().push(*entry);
    }
    index
}

/// A frequency list with its total count (for percentage calculations).
/// Used in ConsolidatedLanguageData (pre-interning).
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub struct GramFrequencyList {
    pub entries: Vec<ReadingGramFrequencyEntry<String>>,
    /// Total gram count from unfiltered data (for accurate percentage calculations)
    pub total_count: u64,
}
