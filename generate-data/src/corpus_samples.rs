//! Corpus sentences that ground the dictionary, phrasebook and sense prompts.

use anyhow::Context;
use language_utils::{Atom, Gram, SentenceGram, SentenceGrams, TaggedGram};
use rustc_hash::{FxHashMap, FxHashSet};
use sentence_sampler::sample_to_target;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
};

/// The course's sentences, indexed by the grams they witness. Built once per
/// course and shared by every stage that shows a prompt corpus sentences, so
/// they all agree on what the course's corpus is.
pub struct CorpusIndex<'a> {
    by_gram: FxHashMap<&'a Gram<String>, Vec<&'a str>>,
    by_sense: FxHashMap<&'a TaggedGram<Gram<String>>, Vec<&'a str>>,
    sentences: FxHashSet<&'a str>,
}

impl<'a> CorpusIndex<'a> {
    pub fn new(encoded_sentences: &'a [(String, SentenceGrams<TaggedGram<Gram<String>>>)]) -> Self {
        let mut by_gram: FxHashMap<&'a Gram<String>, Vec<&'a str>> = FxHashMap::default();
        let mut by_sense: FxHashMap<&TaggedGram<Gram<String>>, Vec<&str>> = FxHashMap::default();
        for (sentence, grams) in encoded_sentences {
            let mut sense_occurrences = FxHashSet::default();
            let mut word_index = 0;
            for gram in &grams.grams {
                let (SentenceGram::Learnable(gram) | SentenceGram::Obvious(gram)) = gram;
                let start = word_index;
                word_index += gram
                    .gram
                    .iter()
                    .filter(|a| matches!(a, Atom::Tok(_)))
                    .count();
                if gram.sense.is_some() {
                    sense_occurrences.insert((gram, (start..word_index).collect::<Vec<_>>()));
                }
                by_gram.entry(&gram.gram).or_default().push(sentence);
            }
            // A high-confidence match witnesses its gram as well as the encoded
            // stream does — and for a citation gram, whose matches are variant
            // occurrences rewritten to it (`pipeline::apply_citations`), matches
            // are the only witnesses: the citation form itself rarely occurs
            // literally, so without these its definition would be generated with
            // no example sentences at all.
            for term in &grams.multiword_terms {
                by_gram.entry(&term.gram.gram).or_default().push(sentence);
                if term.gram.sense.is_some() {
                    sense_occurrences.insert((
                        &term.gram,
                        term.matched_word_indices
                            .iter()
                            .map(|&i| usize::from(i))
                            .collect(),
                    ));
                }
            }
            // Keep repeated occurrences, but don't count an encoded occurrence
            // twice merely because an overlay also matches it.
            for (gram, _) in sense_occurrences {
                by_sense.entry(gram).or_default().push(sentence);
            }
        }
        // A sentence can witness the same gram twice (encoded + match, or two
        // match positions); pushes for one sentence are adjacent, so `dedup`
        // suffices to keep each pool duplicate-free.
        for sentences in by_gram.values_mut() {
            sentences.dedup();
        }
        Self {
            by_gram,
            by_sense,
            sentences: encoded_sentences.iter().map(|(s, _)| s.as_str()).collect(),
        }
    }

    pub fn witnesses(&self, gram: &Gram<String>) -> &[&'a str] {
        self.by_gram.get(gram).map_or(&[], Vec::as_slice)
    }

    /// Full current same-sense witnesses, not the inventory's bounded anchors.
    pub fn sense_witnesses(&self, gram: &TaggedGram<Gram<String>>) -> &[&'a str] {
        self.by_sense.get(gram).map_or(&[], Vec::as_slice)
    }

    /// Whether the sentence is in the course's current corpus — not banned,
    /// not filtered out for another dialect.
    pub fn contains(&self, sentence: &str) -> bool {
        self.sentences.contains(sentence)
    }
}

/// Persistent, ordered prompt examples. Identity and selection strategy belong
/// to the caller; the cache owns revalidation and the refill policy.
pub struct SampleCache<K, I = String> {
    path: PathBuf,
    samples: BTreeMap<K, SavedSamples<I>>,
}

struct SavedSamples<I> {
    samples: Vec<I>,
    // Historical metadata only; never used to decide whether to refresh.
    pool_size_at_pick: Option<usize>,
}

impl<I> Default for SavedSamples<I> {
    fn default() -> Self {
        Self {
            samples: Vec::new(),
            pool_size_at_pick: None,
        }
    }
}

/// Legacy rows retain their exact two-field representation. Only a selection
/// made by this version has a known pool size and writes the third field.
#[derive(Serialize, Deserialize)]
struct SampleEntry<K, S>(
    K,
    S,
    #[serde(default, skip_serializing_if = "Option::is_none")] Option<usize>,
);

impl<
    K: Ord + Clone + Serialize + DeserializeOwned,
    I: Clone + PartialEq + Serialize + DeserializeOwned,
> SampleCache<K, I>
{
    pub fn load(path: PathBuf) -> anyhow::Result<Self> {
        let samples = if path.exists() {
            BufReader::new(File::open(&path)?)
                .lines()
                .map(|line| {
                    let SampleEntry(key, samples, pool_size_at_pick): SampleEntry<K, Vec<I>> =
                        serde_json::from_str(&line?)?;
                    Ok((
                        key,
                        SavedSamples {
                            samples,
                            pool_size_at_pick,
                        },
                    ))
                })
                .collect::<anyhow::Result<_>>()
                .with_context(|| format!("reading {}", path.display()))?
        } else {
            BTreeMap::new()
        };
        if tysm::measurement::enabled() {
            println!(
                "sample-cache-loaded {}",
                serde_json::json!({"path": path, "saved_entries": samples.len()})
            );
        }
        Ok(Self { path, samples })
    }

    pub fn save(&self) -> anyhow::Result<()> {
        if tysm::measurement::enabled() {
            return Ok(());
        }
        let mut file = BufWriter::new(File::create(&self.path)?);
        for (key, entry) in &self.samples {
            writeln!(
                file,
                "{}",
                serde_json::to_string(&SampleEntry(key, &entry.samples, entry.pool_size_at_pick))?
            )?;
        }
        file.flush()?;
        Ok(())
    }

    pub fn get(&self, key: &K) -> &[I] {
        self.samples
            .get(key)
            .map_or(&[], |entry| entry.samples.as_slice())
    }

    pub fn needs_refresh(&self, key: &K, pool: &[I]) -> bool {
        needs_refresh(self.get(key), pool)
    }

    /// The strategy receives survivors in their saved order and the full pool.
    /// It returns the complete selection, retaining that survivor prefix.
    pub fn refresh_with(&mut self, key: &K, pool: &[I], select: impl FnOnce(&[I], &[I]) -> Vec<I>) {
        if tysm::measurement::enabled()
            && let Some(entry) = self.samples.get(key)
        {
            println!(
                "sample-cache-check {}",
                serde_json::json!({
                    "path": self.path, "key": key,
                    "reason": if entry.samples.is_empty() { "empty" }
                        else if needs_refresh(&entry.samples, pool) { "stale" } else { "unchanged" }
                })
            );
        }
        let entry = self.samples.entry(key.clone()).or_default();
        let samples = &mut entry.samples;
        if needs_refresh(samples, pool) {
            entry.pool_size_at_pick = Some(pool.len());
            let mut available: Vec<_> = pool.iter().collect();
            samples.retain(|item| {
                if let Some(i) = available.iter().position(|candidate| *candidate == item) {
                    available.remove(i);
                    true
                } else {
                    false
                }
            });
            let selected = select(samples, pool);
            assert!(
                selected.starts_with(samples),
                "selection must preserve survivors"
            );
            *samples = selected;
        }
    }
}

fn needs_refresh<I: PartialEq>(samples: &[I], pool: &[I]) -> bool {
    samples.is_empty()
        || samples.iter().enumerate().any(|(i, item)| {
            samples[..=i].iter().filter(|s| *s == item).count()
                > pool.iter().filter(|s| *s == item).count()
        })
}

impl<K: Ord + Clone + Serialize + DeserializeOwned> SampleCache<K> {
    /// The original sentence sampler, including its first-population ordering.
    pub fn refresh(&mut self, key: &K, pool: &[&str]) {
        self.refresh_with(
            key,
            &pool.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>(),
            select_sentences,
        );
    }
}

/// Preserve a saved prefix, then fill from the caller's existing pool order.
pub fn select_first<I: Clone + PartialEq>(survivors: &[I], pool: &[I], target: usize) -> Vec<I> {
    survivors
        .iter()
        .cloned()
        .chain(
            remaining_items(pool, survivors)
                .into_iter()
                .take(target.saturating_sub(survivors.len())),
        )
        .collect()
}

pub fn remaining_items<I: Clone + PartialEq>(pool: &[I], survivors: &[I]) -> Vec<I> {
    let mut remaining = pool.to_vec();
    for survivor in survivors {
        if let Some(i) = remaining.iter().position(|item| item == survivor) {
            remaining.remove(i);
        }
    }
    remaining
}

pub fn select_evenly<I: Clone + PartialEq>(survivors: &[I], pool: &[I], target: usize) -> Vec<I> {
    let remaining = remaining_items(pool, survivors);
    let count = target.saturating_sub(survivors.len()).min(remaining.len());
    survivors
        .iter()
        .cloned()
        .chain((0..count).map(|i| remaining[i * remaining.len() / count].clone()))
        .collect()
}

fn select_sentences(survivors: &[String], pool: &[String]) -> Vec<String> {
    let mut samples = survivors.to_vec();
    let candidates = pool.iter().filter(|s| !survivors.contains(s)).collect();
    samples.extend(
        sample_to_target(
            candidates,
            5usize.saturating_sub(samples.len()),
            |s: &&String| s.as_str(),
        )
        .into_iter()
        .cloned(),
    );
    samples
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sense_witnesses_keep_occurrence_multiplicity_without_double_counting_overlays() {
        use language_utils::{Heteronym, MultiwordTermMatch, PartOfSpeech, Word, WordType};
        let gram = Gram(vec![Atom::Tok(Word {
            text: "word".into(),
            word_type: WordType::Heteronym(Heteronym {
                word: "word".into(),
                lemma: "word".into(),
                pos: PartOfSpeech::Noun,
            }),
        })]);
        let first = TaggedGram {
            gram: gram.clone(),
            sense: std::num::NonZeroU32::new(1),
        };
        let second = TaggedGram {
            gram,
            sense: std::num::NonZeroU32::new(2),
        };
        let corpus = vec![
            (
                "twice".into(),
                SentenceGrams {
                    grams: vec![
                        SentenceGram::Learnable(first.clone()),
                        SentenceGram::Learnable(first.clone()),
                    ],
                    capitalize_first: false,
                    multiword_terms: vec![MultiwordTermMatch {
                        gram: first.clone(),
                        matched_word_indices: vec![0],
                    }],
                    low_confidence_multiword_terms: vec![MultiwordTermMatch {
                        gram: first.clone(),
                        matched_word_indices: vec![1],
                    }],
                },
            ),
            (
                "other sense".into(),
                SentenceGrams {
                    grams: vec![SentenceGram::Learnable(second.clone())],
                    capitalize_first: false,
                    multiword_terms: vec![],
                    low_confidence_multiword_terms: vec![],
                },
            ),
            (
                "low confidence only".into(),
                SentenceGrams {
                    grams: vec![],
                    capitalize_first: false,
                    multiword_terms: vec![],
                    low_confidence_multiword_terms: vec![MultiwordTermMatch {
                        gram: first.clone(),
                        matched_word_indices: vec![0],
                    }],
                },
            ),
        ];
        let index = CorpusIndex::new(&corpus);
        assert_eq!(index.sense_witnesses(&first), ["twice", "twice"]);
        assert_eq!(index.sense_witnesses(&second), ["other sense"]);
        assert_eq!(index.witnesses(&first.gram), ["twice", "other sense"]);
    }

    #[test]
    fn legacy_selection_below_target_ignores_arbitrary_pool_growth() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("samples.jsonl");
        let original = "[1,[2,1]]\n";
        std::fs::write(&path, original).unwrap();
        let mut cache = SampleCache::<u32, u32>::load(path.clone()).unwrap();
        for size in [3, 20, 1000] {
            let pool: Vec<_> = (1..=size).collect();
            assert!(!cache.needs_refresh(&1, &pool));
            cache.refresh_with(&1, &pool, |_, _| panic!("growth changed saved picks"));
        }
        assert_eq!(cache.get(&1), [2, 1]);
        assert_eq!(cache.samples[&1].pool_size_at_pick, None);
        cache.save().unwrap();
        assert_eq!(std::fs::read_to_string(path).unwrap(), original);
    }

    #[test]
    fn pool_size_is_recorded_only_on_selection_and_survives_reload() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("samples.jsonl");
        let mut cache = SampleCache::<u32, u32>::load(path.clone()).unwrap();
        cache.refresh_with(&1, &[2, 1, 3], |s, p| select_first(s, p, 2));
        assert_eq!(cache.samples[&1].pool_size_at_pick, Some(3));
        cache.save().unwrap();
        let original = std::fs::read_to_string(&path).unwrap();
        let mut cache = SampleCache::<u32, u32>::load(path.clone()).unwrap();
        cache.refresh_with(&1, &(1..100).collect::<Vec<_>>(), |_, _| {
            panic!("growth refreshed")
        });
        assert_eq!(cache.samples[&1].pool_size_at_pick, Some(3));
        cache.save().unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
        cache.refresh_with(&1, &[1, 4], |s, p| select_first(s, p, 2));
        assert_eq!(cache.get(&1), [1, 4]);
        assert_eq!(cache.samples[&1].pool_size_at_pick, Some(2));
    }

    #[test]
    fn persistence_is_sorted_and_preserves_item_order() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("samples.jsonl");
        let mut cache = SampleCache::<u32, u32>::load(path.clone()).unwrap();
        cache.refresh_with(&2, &[3, 1, 2], |old, pool| select_first(old, pool, 3));
        cache.refresh_with(&1, &[5, 4], |old, pool| select_first(old, pool, 3));
        cache.save().unwrap();
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "[1,[5,4],2]\n[2,[3,1,2],3]\n"
        );
        let reloaded = SampleCache::<u32, u32>::load(path).unwrap();
        assert_eq!(reloaded.get(&2), [3, 1, 2]);
    }

    #[test]
    fn first_sentence_selection_matches_original_sampler_bytes() {
        for size in [0, 1, 5, 6, 100] {
            let pool: Vec<_> = (0..size).map(|i| format!("sentence {i}")).collect();
            let expected =
                sample_to_target(pool.iter().map(String::as_str).collect(), 5, |s: &&str| *s);
            assert_eq!(
                serde_json::to_string(&select_sentences(&[], &pool)).unwrap(),
                serde_json::to_string(&expected).unwrap()
            );
        }
    }

    #[test]
    fn duplicate_multiplicity_is_revalidated_without_reordering_survivors() {
        let dir = tempfile::tempdir().unwrap();
        let mut cache = SampleCache::<u32>::load(dir.path().join("samples.jsonl")).unwrap();
        cache.refresh_with(&0, &["b".into(), "b".into(), "a".into()], |s, p| {
            select_first(s, p, 3)
        });
        assert_eq!(cache.get(&0), ["b", "b", "a"]);
        cache.refresh_with(&0, &["a".into(), "b".into(), "c".into()], |s, p| {
            select_first(s, p, 3)
        });
        assert_eq!(cache.get(&0), ["b", "a", "c"]);
        assert_eq!(select_first(&["b"], &["b", "b", "a"], 3), ["b", "b", "a"]);
    }

    #[test]
    fn evenly_spaced_first_population_matches_original_indices() {
        for size in [0, 1, 12, 13, 200, 201, 999] {
            let pool: Vec<_> = (0..size).collect();
            for target in [12, 200] {
                let count = target.min(size);
                let expected: Vec<_> = (0..count).map(|i| pool[i * size / count]).collect();
                assert_eq!(select_evenly(&[], &pool, target), expected);
            }
        }
    }

    #[test]
    fn first_prefix_selections_preserve_duplicates_and_order() {
        let pool = vec![
            ("b".to_string(), 0),
            ("b".to_string(), 1),
            ("a".to_string(), 0),
            ("c".to_string(), 0),
            ("d".to_string(), 0),
            ("e".to_string(), 0),
        ];
        for target in [3, 5] {
            let expected: Vec<_> = pool.iter().take(target).cloned().collect();
            assert_eq!(select_first(&[], &pool, target), expected);
        }
        assert_eq!(select_first(&pool[..2], &pool, 3), pool[..3]);
    }

    #[test]
    fn samples_retain_order_and_replace_only_stale_members() {
        let dir = tempfile::tempdir().unwrap();
        let mut cache = SampleCache::load(dir.path().join("samples.jsonl")).unwrap();
        cache.samples.insert(
            0,
            SavedSamples {
                samples: vec!["current b".into(), "current a".into()],
                pool_size_at_pick: None,
            },
        );
        let pool = ["current a", "current b", "new"];
        cache.refresh(&0, &pool);
        assert_eq!(cache.get(&0), ["current b", "current a"]);
        cache
            .samples
            .get_mut(&0)
            .unwrap()
            .samples
            .push("Brazilian sentence removed by dialect filter".into());
        cache.refresh(&0, &pool);
        assert_eq!(cache.get(&0), ["current b", "current a", "new"]);
        cache.refresh(&0, &[]);
        assert!(cache.get(&0).is_empty());
    }
}
