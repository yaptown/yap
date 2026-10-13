//! Diverse examples from actual segmentation-gram occurrences, not component words.
use super::Occurrence;
use crate::{
    token_embeddings,
    usage_discovery::{dot, mean_normalized},
};
use futures::StreamExt;
use language_utils::Language;
use std::collections::{BTreeMap, BTreeSet};

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
fn farthest(vectors: &[(usize, Vec<f32>)], count: usize) -> Vec<usize> {
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
    let mut chosen = BTreeSet::new();
    let mut result = Vec::new();
    let mut nearest = vec![f32::NEG_INFINITY; vectors.len()];
    let mut next = seed;
    for _ in 0..count.min(vectors.len()) {
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
    if occurrences.len() <= EXAMPLES {
        return occurrences.to_vec();
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
    let mut chosen = farthest(&vectors, EXAMPLES);
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
    for (_, indices) in missing.into_iter().take(EXTRA_CONTEXTS) {
        chosen.push(indices[0]);
    }
    chosen.into_iter().map(|i| occurrences[i].clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
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
