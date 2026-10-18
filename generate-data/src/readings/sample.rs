//! Diverse examples from actual segmentation-gram occurrences, not component words.
use super::Occurrence;
use crate::corpus_samples::SampleCache;
use crate::{
    token_embeddings,
    usage_discovery::{dot, mean_normalized},
};
use futures::StreamExt;
use language_utils::{Gram, Language};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct Identity {
    sentence: String,
    bytes: (usize, usize),
}
impl From<&Occurrence> for Identity {
    fn from(o: &Occurrence) -> Self {
        Self {
            sentence: o.text.clone(),
            bytes: o.bytes,
        }
    }
}
pub(super) type Cache = SampleCache<Gram<String>, Identity>;

pub(super) async fn persistent_select(
    cache: Option<&Cache>,
    gram: &Gram<String>,
    occurrences: &[Occurrence],
    language: Language,
    store: &osmo::Store,
) -> Vec<Occurrence> {
    let Some(cache) = cache else {
        return select(occurrences, language, store).await;
    };
    let pool: Vec<_> = occurrences.iter().map(Identity::from).collect();
    let survivors: Vec<_> = cache
        .get(gram)
        .iter()
        .filter_map(|id| pool.iter().position(|live| live == id))
        .collect();
    if !cache.needs_refresh(gram, &pool) {
        return survivors
            .into_iter()
            .map(|i| occurrences[i].clone())
            .collect();
    }
    extend(occurrences, language, store, &survivors).await
}

pub(super) fn remember(
    cache: &mut Cache,
    gram: &Gram<String>,
    pool: &[Occurrence],
    selected: &[Occurrence],
) {
    cache.refresh_with(
        gram,
        &pool.iter().map(Identity::from).collect::<Vec<_>>(),
        |_, _| selected.iter().map(Identity::from).collect(),
    );
}

const EXAMPLES: usize = 20;
const EXTRA_CONTEXTS: usize = 4;

fn random_key(o: &Occurrence) -> u64 {
    xxhash_rust::xxh3::xxh3_64(format!("{}\0{}:{}", o.text, o.bytes.0, o.bytes.1).as_bytes())
}
fn context(o: &Occurrence) -> (Option<char>, Option<char>) {
    (
        o.text[..o.bytes.0]
            .chars()
            .rev()
            .find(|c| c.is_alphanumeric()),
        o.text[o.bytes.1..].chars().find(|c| c.is_alphanumeric()),
    )
}

/// Seed at the most central point, then maximize distance from the nearest
/// selected point. Cached occurrence vectors are normalized means of token vectors.
#[cfg(test)]
fn farthest(vectors: &[(usize, Vec<f32>)], count: usize) -> Vec<usize> {
    farthest_from(vectors, count, &[])
}

fn farthest_from(vectors: &[(usize, Vec<f32>)], count: usize, survivors: &[usize]) -> Vec<usize> {
    if vectors.is_empty() {
        return vec![];
    }
    let center = mean_normalized(&vectors.iter().map(|(_, v)| v).collect::<Vec<_>>());
    let seed = (0..vectors.len())
        .max_by(|&a, &b| {
            dot(&vectors[a].1, &center)
                .total_cmp(&dot(&vectors[b].1, &center))
                .then_with(|| b.cmp(&a))
        })
        .unwrap();
    let mut chosen: BTreeSet<_> = vectors
        .iter()
        .enumerate()
        .filter_map(|(i, (id, _))| survivors.contains(id).then_some(i))
        .collect();
    let mut result = Vec::new();
    let mut nearest = vec![f32::NEG_INFINITY; vectors.len()];
    for &selected in &chosen {
        for (i, (_, v)) in vectors.iter().enumerate() {
            nearest[i] = nearest[i].max(dot(v, &vectors[selected].1));
        }
    }
    let mut next = if chosen.is_empty() {
        seed
    } else {
        let Some(next) = (0..vectors.len())
            .filter(|i| !chosen.contains(i))
            .min_by(|&a, &b| nearest[a].total_cmp(&nearest[b]).then_with(|| a.cmp(&b)))
        else {
            return result;
        };
        next
    };
    for _ in 0..count.min(vectors.len().saturating_sub(chosen.len())) {
        chosen.insert(next);
        result.push(vectors[next].0);
        for (i, (_, v)) in vectors.iter().enumerate() {
            nearest[i] = nearest[i].max(dot(v, &vectors[next].1));
        }
        let Some(candidate) = (0..vectors.len())
            .filter(|i| !chosen.contains(i))
            .min_by(|&a, &b| nearest[a].total_cmp(&nearest[b]).then_with(|| a.cmp(&b)))
        else {
            break;
        };
        next = candidate;
    }
    result
}

pub(super) async fn select(
    occurrences: &[Occurrence],
    language: Language,
    store: &osmo::Store,
) -> Vec<Occurrence> {
    extend(occurrences, language, store, &[]).await
}

async fn extend(
    occurrences: &[Occurrence],
    language: Language,
    store: &osmo::Store,
    survivors: &[usize],
) -> Vec<Occurrence> {
    if occurrences.len() <= EXAMPLES {
        let mut chosen = survivors.to_vec();
        chosen.extend((0..occurrences.len()).filter(|i| !survivors.contains(i)));
        return chosen.into_iter().map(|i| occurrences[i].clone()).collect();
    }
    let vectors: Vec<_> =
        futures::stream::iter(occurrences.iter().enumerate().map(|(index, o)| async move {
            let vectors =
                token_embeddings::read_word_vectors(store, language, &o.text, &o.spans).await?;
            Some((index, mean_normalized(&vectors.iter().collect::<Vec<_>>())))
        }))
        .buffered(8)
        .filter_map(|v| async { v })
        .collect()
        .await;
    let mut chosen = survivors.to_vec();
    chosen.extend(farthest_from(
        &vectors,
        EXAMPLES.saturating_sub(chosen.len()),
        survivors,
    ));
    let mut present: BTreeSet<_> = chosen.iter().copied().collect();
    let mut random: Vec<_> = (0..occurrences.len()).collect();
    random.sort_by_key(|&i| (random_key(&occurrences[i]), i));
    for &i in &random {
        if chosen.len() >= EXAMPLES {
            break;
        }
        if present.insert(i) {
            chosen.push(i);
        }
    }
    // A bounded guard against a dominant context hiding rare suffix/date uses.
    // Include uncovered neighbour patterns, rarest first; never grow a prompt
    // without bound merely because a frequent gram has thousands of contexts.
    let mut patterns: BTreeMap<_, Vec<usize>> = BTreeMap::new();
    for &i in &random {
        patterns
            .entry(context(&occurrences[i]))
            .or_default()
            .push(i);
    }
    let covered: BTreeSet<_> = chosen.iter().map(|&i| context(&occurrences[i])).collect();
    let mut missing: Vec<_> = patterns
        .into_iter()
        .filter(|(key, _)| !covered.contains(key))
        .collect();
    missing.sort_by(|a, b| a.1.len().cmp(&b.1.len()).then_with(|| a.0.cmp(&b.0)));
    for (_, indices) in missing.into_iter().take(
        (EXAMPLES + EXTRA_CONTEXTS)
            .saturating_sub(chosen.len())
            .min(EXTRA_CONTEXTS),
    ) {
        chosen.push(indices[0]);
    }
    chosen.into_iter().map(|i| occurrences[i].clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn occurrences(count: usize) -> Vec<Occurrence> {
        (0..count)
            .map(|i| Occurrence {
                text: format!("猫 {i}"),
                gram_index: 0,
                bytes: (0, 3),
                spans: vec![(0, 1)],
            })
            .collect()
    }

    // Independent reference for the old selector: central seed, nearest-point
    // diversity, hash fill, and four rare uncovered contexts, in that order.
    fn original_selection(pool: &[Occurrence], vectors: &[(usize, Vec<f32>)]) -> Vec<Occurrence> {
        if pool.len() <= 20 {
            return pool.to_vec();
        }
        let mut chosen = Vec::new();
        if !vectors.is_empty() {
            let center = mean_normalized(&vectors.iter().map(|(_, v)| v).collect::<Vec<_>>());
            let seed = (0..vectors.len())
                .max_by(|&a, &b| {
                    dot(&vectors[a].1, &center)
                        .total_cmp(&dot(&vectors[b].1, &center))
                        .then_with(|| b.cmp(&a))
                })
                .unwrap();
            chosen.push(vectors[seed].0);
            while chosen.len() < 20.min(vectors.len()) {
                let next = vectors
                    .iter()
                    .filter(|(id, _)| !chosen.contains(id))
                    .min_by(|(a, av), (b, bv)| {
                        let similarity = |v: &[f32]| {
                            chosen
                                .iter()
                                .map(|id| dot(v, &vectors.iter().find(|(i, _)| i == id).unwrap().1))
                                .fold(f32::NEG_INFINITY, f32::max)
                        };
                        similarity(av)
                            .total_cmp(&similarity(bv))
                            .then_with(|| a.cmp(b))
                    })
                    .unwrap()
                    .0;
                chosen.push(next);
            }
        }
        let mut random: Vec<_> = (0..pool.len()).collect();
        random.sort_by_key(|&i| (random_key(&pool[i]), i));
        for &i in &random {
            if chosen.len() >= 20 {
                break;
            }
            if !chosen.contains(&i) {
                chosen.push(i);
            }
        }
        let mut contexts = BTreeMap::<_, Vec<_>>::new();
        for &i in &random {
            contexts.entry(context(&pool[i])).or_default().push(i);
        }
        let mut missing: Vec<_> = contexts
            .into_iter()
            .filter(|(pattern, _)| !chosen.iter().any(|&i| context(&pool[i]) == *pattern))
            .collect();
        missing.sort_by(|a, b| a.1.len().cmp(&b.1.len()).then_with(|| a.0.cmp(&b.0)));
        chosen.extend(missing.into_iter().take(4).map(|(_, indices)| indices[0]));
        chosen.into_iter().map(|i| pool[i].clone()).collect()
    }

    #[tokio::test]
    async fn first_population_matches_original_prompt_bytes_with_and_without_vectors() {
        let gram = super::super::tests::gram(&["猫"]);
        for count in [0, 1, 20, 21, 50] {
            for embedded in [false, true] {
                let dir = tempfile::tempdir().unwrap();
                let store = osmo::Store::open(dir.path().join("vectors"));
                let mut cache = Cache::load(dir.path().join("examples.jsonl")).unwrap();
                let pool = occurrences(count);
                let mut vectors = Vec::new();
                if embedded {
                    for (i, o) in pool.iter().enumerate().filter(|(i, _)| i % 3 != 0) {
                        let vector = vec![if i % 2 == 0 { 1.0 } else { -1.0 }, 0.0];
                        let mut record: Vec<_> = [2u32, 1, 0, 1]
                            .into_iter()
                            .flat_map(u32::to_le_bytes)
                            .collect();
                        record.extend(
                            vector
                                .iter()
                                .flat_map(|&v| half::f16::from_f32(v).to_le_bytes()),
                        );
                        store
                            .write(
                                &token_embeddings::cache_key(Language::Japanese, &o.text),
                                &record,
                            )
                            .await
                            .unwrap();
                        vectors.push((i, vector));
                    }
                }
                let expected = original_selection(&pool, &vectors);
                let selected =
                    persistent_select(Some(&cache), &gram, &pool, Language::Japanese, &store).await;
                assert_eq!(
                    super::super::prompts::numbered_examples(&selected),
                    super::super::prompts::numbered_examples(&expected),
                    "count={count} embedded={embedded}"
                );
                remember(&mut cache, &gram, &pool, &selected);
                cache.save().unwrap();
                let cache = Cache::load(dir.path().join("examples.jsonl")).unwrap();
                assert_eq!(
                    cache.get(&gram),
                    selected.iter().map(Identity::from).collect::<Vec<_>>()
                );
            }
        }
    }

    #[tokio::test]
    async fn persistence_growth_staleness_and_live_occurrence_mapping() {
        let dir = tempfile::tempdir().unwrap();
        let store = osmo::Store::open(dir.path().join("vectors"));
        let mut cache = Cache::load(dir.path().join("examples.jsonl")).unwrap();
        let gram = super::super::tests::gram(&["猫"]);
        let mut pool = occurrences(2);
        let initial =
            persistent_select(Some(&cache), &gram, &pool, Language::Japanese, &store).await;
        remember(&mut cache, &gram, &pool, &initial);
        pool.reverse();
        pool[1].gram_index = 17;
        pool[1].spans = vec![(3, 4)];
        let stable =
            persistent_select(Some(&cache), &gram, &pool, Language::Japanese, &store).await;
        assert_eq!(stable[0].gram_index, 17);
        assert_eq!(stable[0].spans, [(3, 4)]);
        pool.push(occurrences(3).pop().unwrap());
        assert_eq!(
            persistent_select(Some(&cache), &gram, &pool, Language::Japanese, &store)
                .await
                .len(),
            2
        );
        pool.remove(0);
        let repaired =
            persistent_select(Some(&cache), &gram, &pool, Language::Japanese, &store).await;
        assert_eq!(
            repaired.iter().map(|o| &o.text).collect::<Vec<_>>(),
            [&pool[0].text, &pool[1].text]
        );
        // Sample mode ignores persistent choices.
        let bypass = persistent_select(None, &gram, &pool, Language::Japanese, &store).await;
        assert_eq!(bypass.len(), pool.len());
    }

    #[test]
    fn diversity_refill_starts_from_survivors_not_the_center() {
        let vectors = vec![(0, vec![1., 0.]), (1, vec![1., 0.]), (2, vec![-1., 0.])];
        assert_eq!(farthest_from(&vectors, 1, &[2]), [0]);
        assert_eq!(farthest_from(&vectors, 1, &[0]), [2]);
    }

    #[test]
    fn central_seed_then_minority_cluster() {
        let vectors = vec![
            (0, vec![1., 0.]),
            (1, vec![1., 0.]),
            (2, vec![1., 0.]),
            (3, vec![-1., 0.]),
        ];
        assert_eq!(farthest(&vectors, 2), [0, 3]);
        assert_eq!(farthest(&vectors, 20).len(), 4);
        assert!(farthest(&[], 20).is_empty());
    }
    #[tokio::test]
    async fn missing_vectors_get_reproducible_random_fill_and_rare_contexts() {
        let dir = tempfile::tempdir().unwrap();
        let store = osmo::Store::open(dir.path());
        let mut occurrences: Vec<_> = (0..40)
            .map(|i| Occurrence {
                text: format!("{i}月一日です"),
                gram_index: 0,
                bytes: (i.to_string().len() + 3, i.to_string().len() + 9),
                spans: vec![],
            })
            .collect();
        occurrences[39].text = "39年一日です".into();
        let a = select(&occurrences, Language::Japanese, &store).await;
        let b = select(&occurrences, Language::Japanese, &store).await;
        assert_eq!(
            a.iter().map(|o| &o.text).collect::<Vec<_>>(),
            b.iter().map(|o| &o.text).collect::<Vec<_>>()
        );
        assert!((EXAMPLES..=EXAMPLES + EXTRA_CONTEXTS).contains(&a.len()));
        assert!(a.iter().any(|o| o.text == "39年一日です"));
    }
}
