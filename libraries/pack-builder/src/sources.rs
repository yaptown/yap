use language_utils::language_pack::{
    FrequencyList, SentenceLists, SentenceSources, SourceFrequencyList, SourceIds,
};
use language_utils::{SentenceSource, sentence_codec::GramKey};
use rustc_hash::FxHashMap;

fn source_ids<'a>(rows: impl Iterator<Item = &'a [String]>) -> SourceIds {
    let mut ids = Vec::new();
    let mut indices = FxHashMap::default();
    let sentences = SentenceLists::new(rows.map(|row| {
        row.iter()
            .map(|id| {
                *indices.entry(id).or_insert_with(|| {
                    let index = u16::try_from(ids.len()).unwrap();
                    ids.push(id.clone());
                    index
                })
            })
            .collect::<Vec<_>>()
    }));
    SourceIds { ids, sentences }
}
pub fn sentence_sources(rows: &[SentenceSource]) -> SentenceSources {
    SentenceSources {
        flags: rows
            .iter()
            .map(|source| {
                u8::from(source.from_anki)
                    | (u8::from(source.from_tatoeba) << 1)
                    | (u8::from(source.from_manual) << 2)
                    | (u8::from(source.from_song) << 3)
            })
            .collect(),
        movies: source_ids(rows.iter().map(|source| source.movie_ids.as_slice())),
        books: source_ids(rows.iter().map(|source| source.book_ids.as_slice())),
    }
}
pub fn source_frequencies(
    entries: impl IntoIterator<Item = (GramKey, u32)>,
    total_count: u64,
    global: &FrequencyList,
) -> SourceFrequencyList {
    let (keys, counts) = entries
        .into_iter()
        .map(|(key, count)| {
            assert!(
                global.entries.contains_key(&key),
                "source gram must exist in global frequencies"
            );
            (
                key,
                u8::try_from(count).expect("source count must fit in u8"),
            )
        })
        .unzip();
    SourceFrequencyList {
        keys,
        counts,
        total_count,
    }
}
