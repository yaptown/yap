//! Corpus sentences that ground the dictionary, phrasebook and sense prompts.

use anyhow::Context;
use language_utils::{Gram, SentenceGram, SentenceGrams, TaggedGram};
use rustc_hash::{FxHashMap, FxHashSet};
use sentence_sampler::sample_to_target;
use serde::{Serialize, de::DeserializeOwned};
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

/// A persistent choice of up to five corpus sentences per key. The sentences
/// are part of each prompt and so of its cache key: a key keeps its sentences
/// until one of them leaves the corpus, so regenerating doesn't reshuffle them.
pub struct SampleCache<K> {
    path: PathBuf,
    samples: BTreeMap<K, Vec<String>>,
}

impl<K: Ord + Clone + Serialize + DeserializeOwned> SampleCache<K> {
    pub fn load(path: PathBuf) -> anyhow::Result<Self> {
        let samples = if path.exists() {
            BufReader::new(File::open(&path)?)
                .lines()
                .map(|line| Ok(serde_json::from_str(&line?)?))
                .collect::<anyhow::Result<_>>()
                .with_context(|| format!("reading {}", path.display()))?
        } else {
            BTreeMap::new()
        };
        Ok(Self { path, samples })
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let mut file = BufWriter::new(File::create(&self.path)?);
        for entry in &self.samples {
            writeln!(file, "{}", serde_json::to_string(&entry)?)?;
        }
        file.flush()?;
        Ok(())
    }

    /// Bring a key's sentences in line with its current pool.
    pub fn refresh(&mut self, key: &K, pool: &[&str]) {
        revalidate(self.samples.entry(key.clone()).or_default(), pool);
    }

    pub fn get(&self, key: &K) -> &[String] {
        self.samples.get(key).map_or(&[], Vec::as_slice)
    }
}

fn revalidate(samples: &mut Vec<String>, pool: &[&str]) {
    // A valid nonempty selection is part of the prompt cache key; leave it alone.
    if !samples.is_empty() && samples.iter().all(|s| pool.contains(&s.as_str())) {
        return;
    }
    samples.retain(|s| pool.contains(&s.as_str()));
    let candidates: Vec<_> = pool
        .iter()
        .copied()
        .filter(|s| !samples.iter().any(|old| old == s))
        .collect();
    samples.extend(
        sample_to_target(
            candidates,
            5usize.saturating_sub(samples.len()),
            |s: &&str| *s,
        )
        .into_iter()
        .map(str::to_owned),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samples_retain_order_and_replace_only_stale_members() {
        let mut samples = vec!["current b".into(), "current a".into()];
        let pool = ["current a", "current b", "new"];
        revalidate(&mut samples, &pool);
        assert_eq!(samples, ["current b", "current a"]);
        samples.push("Brazilian sentence removed by dialect filter".into());
        revalidate(&mut samples, &pool);
        assert_eq!(samples, ["current b", "current a", "new"]);
        revalidate(&mut samples, &[]);
        assert!(samples.is_empty());
    }
}
