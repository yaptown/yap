//! Stable repair witnesses within the current universe of meanings/readings.
use super::{
    Choice, Occurrence,
    occurrence::{Escape, forced_examples},
    sample::Identity,
};
use crate::corpus_samples::PinnedSamples;
use language_utils::{Gram, Language};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

type Bucket = (Option<String>, Option<String>);
type ForcedKey = (Gram<String>, BTreeSet<Bucket>);
type AttestedKey = (Gram<String>, BTreeSet<Choice>);

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Forced {
    occurrence: Identity,
    reading: Option<String>,
    missing_meaning: Option<String>,
}
impl From<&Escape> for Forced {
    fn from(escape: &Escape) -> Self {
        Self {
            occurrence: Identity::from(&escape.occurrence),
            reading: escape.reading.clone(),
            missing_meaning: escape.missing_meaning.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct Attested {
    sentence: String,
    choices: Vec<Choice>,
}
impl Attested {
    pub fn new(occurrence: &Occurrence, choices: &[Choice]) -> Self {
        Self {
            sentence: occurrence.marked(),
            choices: choices.to_vec(),
        }
    }
}

pub(super) struct Samples {
    forced: PinnedSamples<ForcedKey, Forced>,
    attested: PinnedSamples<AttestedKey, Attested>,
}
impl Samples {
    pub fn load(language: Language) -> anyhow::Result<Self> {
        Self::load_from(&std::path::PathBuf::from("out").join(language.corpus_code()))
    }

    fn load_from(directory: &std::path::Path) -> anyhow::Result<Self> {
        Ok(Self {
            forced: PinnedSamples::load(directory.join("reading_repair_escapes.jsonl"))?,
            attested: PinnedSamples::load(directory.join("reading_repair_attested.jsonl"))?,
        })
    }

    pub fn save(&self) -> anyhow::Result<()> {
        self.forced.save()?;
        self.attested.save()
    }
}

pub(super) fn forced(
    cache: Option<&mut Samples>,
    gram: &Gram<String>,
    escapes: &[Escape],
) -> Vec<Escape> {
    let Some(cache) = cache else {
        return forced_examples(escapes).into_iter().cloned().collect();
    };
    let pool: Vec<_> = escapes.iter().map(Forced::from).collect();
    // A new reading/meaning needs a fresh balanced selection, not a frozen
    // full set of witnesses for the old problem.
    let universe = pool
        .iter()
        .map(|e| (e.reading.clone(), e.missing_meaning.clone()))
        .collect();
    let key = (gram.clone(), universe);
    cache.forced.refresh_with(&key, &pool, |_| {
        forced_examples(escapes)
            .into_iter()
            .map(Forced::from)
            .collect()
    });
    let occurrences: Vec<_> = escapes.iter().map(|e| e.occurrence.clone()).collect();
    cache
        .forced
        .get(&key)
        .iter()
        .map(|id| Escape {
            occurrence: id.occurrence.occurrence(&occurrences),
            // Proposed readings are diagnostic only; repair prompts use the guarded reading.
            proposed_reading: id.reading.clone().unwrap_or_default(),
            reading: id.reading.clone(),
            missing_meaning: id.missing_meaning.clone(),
        })
        .collect()
}

pub(super) fn attested(
    cache: Option<&mut Samples>,
    gram: &Gram<String>,
    pool: &[Attested],
) -> Vec<Attested> {
    let Some(cache) = cache else {
        return cover(pool);
    };
    let universe: BTreeSet<_> = pool.iter().flat_map(|e| &e.choices).cloned().collect();
    let key = (gram.clone(), universe);
    cache.attested.refresh_with(&key, pool, cover);
    cache.attested.get(&key).to_vec()
}

fn cover(pool: &[Attested]) -> Vec<Attested> {
    let mut covered = BTreeSet::new();
    let mut selected = Vec::new();
    for example in pool {
        let previous = covered.len();
        covered.extend(example.choices.iter().cloned());
        if covered.len() > previous {
            selected.push(example.clone());
        }
    }
    selected
}

#[cfg(test)]
mod tests {
    use super::*;

    fn occurrence(text: String) -> Occurrence {
        Occurrence {
            text,
            gram_index: 0,
            bytes: (0, 3),
            spans: vec![(0, 1)],
        }
    }
    fn escape(i: usize, reading: &str) -> Escape {
        Escape {
            occurrence: occurrence(format!("猫 {i}")),
            proposed_reading: reading.into(),
            reading: Some(reading.into()),
            missing_meaning: None,
        }
    }
    fn choice(sense: u32) -> Choice {
        Choice {
            sense,
            reading: "ねこ".into(),
        }
    }
    fn example(text: &str, senses: &[u32]) -> Attested {
        Attested::new(
            &occurrence(format!("猫 {text}")),
            &senses.iter().copied().map(choice).collect::<Vec<_>>(),
        )
    }

    #[test]
    fn forced_initial_bytes_growth_new_meanings_and_live_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let mut cache = Samples::load_from(dir.path()).unwrap();
        let gram = super::super::tests::gram(&["猫"]);
        let mut pool: Vec<_> = (0..40)
            .map(|i| escape(i, if i % 4 == 0 { "びょう" } else { "ねこ" }))
            .collect();
        let expected: Vec<_> = forced_examples(&pool)
            .into_iter()
            .map(|e| {
                (
                    e.occurrence.marked(),
                    e.reading.clone(),
                    e.missing_meaning.clone(),
                )
            })
            .collect();
        let render = |es: &[Escape]| {
            es.iter()
                .map(|e| {
                    (
                        e.occurrence.marked(),
                        e.reading.clone(),
                        e.missing_meaning.clone(),
                    )
                })
                .collect::<Vec<_>>()
        };
        let selected = forced(Some(&mut cache), &gram, &pool);
        assert_eq!(render(&selected), expected);
        cache.save().unwrap();
        let mut cache = Samples::load_from(dir.path()).unwrap();
        pool.extend((40..200).map(|i| escape(i, "ねこ")));
        pool.reverse();
        for e in &mut pool {
            e.occurrence.gram_index = 17;
            e.proposed_reading = "ignored metadata".into();
        }
        let stable = forced(Some(&mut cache), &gram, &pool);
        assert_eq!(render(&stable), expected);
        assert!(stable.iter().all(|e| e.occurrence.gram_index == 17));
        // Removed witnesses remain verbatim prompt context.
        let removed = selected[0].occurrence.text.clone();
        pool.retain(|e| e.occurrence.text != removed);
        let repaired = forced(Some(&mut cache), &gram, &pool);
        assert_eq!(render(&repaired), expected);
        pool.push(escape(999, "new reading"));
        assert!(
            forced(Some(&mut cache), &gram, &pool)
                .iter()
                .any(|e| e.reading.as_deref() == Some("new reading"))
        );
        assert_eq!(
            render(&forced(None, &gram, &pool)),
            render(
                &forced_examples(&pool)
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>()
            )
        );
    }

    #[test]
    fn attested_original_greedy_bytes_growth_and_new_choices() {
        let dir = tempfile::tempdir().unwrap();
        let mut cache = Samples::load_from(dir.path()).unwrap();
        let gram = super::super::tests::gram(&["猫"]);
        let mut pool = vec![
            example("b", &[1]),
            example("c", &[1, 2]),
            example("d", &[2]),
        ];
        let expected = serde_json::json!([
            {"sentence":"«猫» b", "choices":[{"sense":1,"reading":"ねこ"}]},
            {"sentence":"«猫» c", "choices":[{"sense":1,"reading":"ねこ"},{"sense":2,"reading":"ねこ"}]}
        ]);
        assert_eq!(
            serde_json::to_value(attested(Some(&mut cache), &gram, &pool)).unwrap(),
            expected
        );
        pool.insert(0, example("a", &[1, 2]));
        assert_eq!(
            serde_json::to_value(attested(Some(&mut cache), &gram, &pool)).unwrap(),
            expected
        );
        assert_eq!(attested(None, &gram, &pool), [pool[0].clone()]);
        pool.push(example("new", &[3]));
        let changed = attested(Some(&mut cache), &gram, &pool);
        assert!(changed.iter().any(|e| e.choices.contains(&choice(3))));
        let first = changed[0].clone();
        pool.retain(|e| e != &first);
        let repaired = attested(Some(&mut cache), &gram, &pool);
        assert_eq!(repaired, changed);
        assert_eq!(
            repaired
                .iter()
                .flat_map(|e| &e.choices)
                .cloned()
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([choice(1), choice(2), choice(3)])
        );
    }
}
