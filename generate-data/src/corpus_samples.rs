//! Corpus sentences that ground the dictionary, phrasebook and sense prompts.

use anyhow::Context;
use language_utils::{Gram, SentenceGram, SentenceGrams, TaggedGram};
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
    sentences: FxHashSet<&'a str>,
}

impl<'a> CorpusIndex<'a> {
    pub fn new(encoded_sentences: &'a [(String, SentenceGrams<TaggedGram<Gram<String>>>)]) -> Self {
        let mut by_gram: FxHashMap<&'a Gram<String>, Vec<&'a str>> = FxHashMap::default();
        for (sentence, grams) in encoded_sentences {
            for gram in &grams.grams {
                let (SentenceGram::Learnable(gram) | SentenceGram::Obvious(gram)) = gram;
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
            sentences: encoded_sentences.iter().map(|(s, _)| s.as_str()).collect(),
        }
    }

    pub fn witnesses(&self, gram: &Gram<String>) -> &[&'a str] {
        self.by_gram.get(gram).map_or(&[], Vec::as_slice)
    }

    /// Whether the sentence is in the course's current corpus — not banned,
    /// not filtered out for another dialect.
    pub fn contains(&self, sentence: &str) -> bool {
        self.sentences.contains(sentence)
    }
}

/// Persistent, ordered prompt examples. Identity and selection strategy belong
/// to the caller; the cache populates each key once and never changes non-empty samples.
pub struct PinnedSamples<K, I = String> {
    path: PathBuf,
    samples: BTreeMap<K, SavedSamples<I>>,
}

struct SavedSamples<I> {
    samples: Vec<I>,
    // Historical metadata only; never used to decide whether to refresh.
    pool_size_at_pick: Option<usize>,
}

/// Legacy rows retain their exact two-field representation. Only a selection
/// made by this version has a known pool size and writes the third field.
#[derive(Serialize, Deserialize)]
struct SampleEntry<K, S>(
    K,
    S,
    #[serde(default, skip_serializing_if = "Option::is_none")] Option<usize>,
);

impl<K: Ord + Clone + Serialize + DeserializeOwned, I: Clone + Serialize + DeserializeOwned>
    PinnedSamples<K, I>
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

    pub fn needs_refresh(&self, key: &K) -> bool {
        self.get(key).is_empty()
    }

    /// Populate an empty key. Saved prompt context survives every corpus change.
    pub fn refresh_with(&mut self, key: &K, pool: &[I], select: impl FnOnce(&[I]) -> Vec<I>) {
        if tysm::measurement::enabled() {
            println!(
                "sample-cache-check {}",
                serde_json::json!({
                    "path": self.path, "key": key,
                    "reason": if self.needs_refresh(key) { "empty" } else { "unchanged" }
                })
            );
        }
        if self.needs_refresh(key) {
            self.samples.insert(
                key.clone(),
                SavedSamples {
                    samples: select(pool),
                    pool_size_at_pick: Some(pool.len()),
                },
            );
        }
    }
}

impl<K: Ord + Clone + Serialize + DeserializeOwned> PinnedSamples<K> {
    /// The original sentence sampler, including its first-population ordering.
    pub fn refresh(&mut self, key: &K, pool: &[&str]) {
        self.refresh_with(
            key,
            &pool.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>(),
            select_sentences,
        );
    }
}

/// Select from the caller's existing pool order, retaining duplicates.
pub fn select_first<I: Clone>(pool: &[I], target: usize) -> Vec<I> {
    pool.iter().take(target).cloned().collect()
}

pub fn select_evenly<I: Clone>(pool: &[I], target: usize) -> Vec<I> {
    let count = target.min(pool.len());
    (0..count)
        .map(|i| pool[i * pool.len() / count].clone())
        .collect()
}

fn select_sentences(pool: &[String]) -> Vec<String> {
    sample_to_target(pool.iter().collect(), 5, |s: &&String| s.as_str())
        .into_iter()
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn witnesses_deduplicate_repeated_grams_and_ignore_low_confidence_overlays() {
        use language_utils::{Atom, Heteronym, MultiwordTermMatch, PartOfSpeech, Word, WordType};
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
        assert_eq!(index.witnesses(&first.gram), ["twice", "other sense"]);
    }

    #[test]
    fn saved_samples_never_change_even_after_removal_or_reload() {
        let dir = tempfile::tempdir().unwrap();
        for original in ["[1,[2,2,1]]\n", "[1,[2,2,1],3]\n"] {
            let path = dir.path().join("samples.jsonl");
            std::fs::write(&path, original).unwrap();
            let mut cache = PinnedSamples::<u32, u32>::load(path.clone()).unwrap();
            for pool in [vec![1, 2, 3, 4], vec![1, 2], vec![9], vec![]] {
                assert!(!cache.needs_refresh(&1));
                cache.refresh_with(&1, &pool, |_| panic!("saved picks changed"));
                assert_eq!(cache.get(&1), [2, 2, 1]);
            }
            cache.save().unwrap();
            assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
            assert_eq!(
                PinnedSamples::<u32, u32>::load(path).unwrap().get(&1),
                [2, 2, 1]
            );
        }
    }

    #[test]
    fn empty_and_missing_keys_populate_in_order() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("samples.jsonl");
        let mut cache = PinnedSamples::<u32, u32>::load(path.clone()).unwrap();
        cache.refresh_with(&1, &[], |p| select_first(p, 3));
        assert!(cache.needs_refresh(&1));
        cache.refresh_with(&2, &[3, 1, 2], |p| select_first(p, 3));
        cache.refresh_with(&1, &[5, 4], |p| select_first(p, 3));
        cache.save().unwrap();
        assert_eq!(
            std::fs::read_to_string(path).unwrap(),
            "[1,[5,4],2]\n[2,[3,1,2],3]\n"
        );
    }

    #[test]
    fn first_sentence_selection_matches_original_sampler_bytes() {
        for size in [0, 1, 5, 6, 100] {
            let pool: Vec<_> = (0..size).map(|i| format!("sentence {i}")).collect();
            let expected =
                sample_to_target(pool.iter().map(String::as_str).collect(), 5, |s: &&str| *s);
            assert_eq!(
                serde_json::to_string(&select_sentences(&pool)).unwrap(),
                serde_json::to_string(&expected).unwrap()
            );
        }
    }

    #[test]
    fn evenly_spaced_first_population_matches_original_indices() {
        for size in [0, 1, 12, 13, 200, 201, 999] {
            let pool: Vec<_> = (0..size).collect();
            for target in [12, 200] {
                let count = target.min(size);
                let expected: Vec<_> = (0..count).map(|i| pool[i * size / count]).collect();
                assert_eq!(select_evenly(&pool, target), expected);
            }
        }
        assert_eq!(select_first(&[2, 2, 1, 3], 3), [2, 2, 1]);
    }
}
