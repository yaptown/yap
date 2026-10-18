//! Temporary, write-isolated readings iteration and human-readable QA.
use super::*;
use serde_json::{Value, json};

type Labels = BTreeMap<(String, usize), Value>;

/// Use the full source/matching pipeline, but put all its out/ caches and
/// diagnostics in disposable scratch. Source loading is cache-only solely to
/// suppress metadata/poster refresh; B/C retain the caller's normal cache mode.
pub async fn run(sample: Sample, languages: &BTreeSet<String>) -> Result<()> {
    use anyhow::Context;
    let cache_only = crate::cache_only();
    let root = std::path::Path::new("target/readings-sample");
    std::fs::create_dir_all(root)?;
    for course in language_utils::COURSES
        .iter()
        .filter(|c| languages.contains(c.target_language.code()))
    {
        let scratch = tempfile::Builder::new()
            .prefix("preparation-")
            .tempdir_in(root)?;
        let source = std::path::PathBuf::from("out").join(course.target_language.corpus_code());
        for name in [
            "target_language_sentences_tokenization.jsonl",
            "restricted_sentences_tokenization.jsonl",
            "target_language_multiword_terms_tokenization.jsonl",
            "target_language_sentences_tokenization.jsonl.failures.jsonl",
            "restricted_sentences_tokenization.jsonl.failures.jsonl",
            "target_language_multiword_terms_tokenization.jsonl.failures.jsonl",
            "target_language_multiword_terms.txt",
            "wiktionary_multiword_terms.txt",
            "multiword_alt_forms.jsonl",
        ] {
            let from = source.join(name);
            if from.exists() {
                // Copy, never hardlink/symlink: preparation appends to some of these.
                std::fs::copy(&from, scratch.path().join(name))
                    .with_context(|| format!("copy sample input {}", from.display()))?;
            }
        }
        crate::set_cache_only(true);
        let sentences = crate::target_sentences::get_target_sentences(*course).await;
        crate::set_cache_only(cache_only);
        let sentences = sentences.context("sample source loading requires warm subtitle caches")?;
        let segmented =
            crate::pipeline::segment_corpus(course, &sentences, Some(scratch.path())).await?;
        let store = crate::cache_remote::store();
        crate::token_embeddings::ensure_token_embeddings(
            course.target_language,
            segmented
                .nlp_sentences
                .iter()
                .chain(segmented.restricted_nlp_sentences.iter()),
            &segmented.interners,
            &store,
        )
        .await?;
        CorpusReadings::generate(
            course.target_language,
            &segmented.nlp_sentences,
            &segmented.restricted_nlp_sentences,
            &segmented.interners,
            &store,
            Some(sample.clone()),
        )
        .await?;
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct Sample {
    percent: f64,
    pins: BTreeSet<String>,
    pub(super) initial: Labels,
    pub(super) rounds: BTreeMap<(String, usize), String>,
    pub(super) provenance: BTreeMap<Gram<String>, &'static str>,
    pub(super) b_initial: BTreeSet<Gram<String>>,
    pub(super) b_repair: BTreeSet<Gram<String>>,
    pub(super) b_fallback: BTreeSet<Gram<String>>,
    pub(super) focused_sent: usize,
    pub(super) focused_resolved: usize,
    pub(super) reconsidered_ranked: usize,
    pub(super) boundary_filtered_occurrences: usize,
    pub(super) boundary_filtered_readings: usize,
}

impl Sample {
    pub fn new(percent: f64) -> Result<Self> {
        let mut sample = Self::with_pins(percent, BTreeSet::new())?;
        // Sentences that are always in the sample, so audits can compare runs.
        sample.pins = include_str!("../../data/readings-sample-pinned.txt")
            .lines()
            .map(str::to_owned)
            .collect();
        Ok(sample)
    }

    pub(super) fn with_pins(percent: f64, pins: BTreeSet<String>) -> Result<Self> {
        ensure!(
            percent.is_finite() && percent > 0.0 && percent <= 100.0,
            "--readings-sample percent must be finite and in (0, 100]"
        );
        Ok(Self {
            percent,
            pins,
            initial: BTreeMap::new(),
            rounds: BTreeMap::new(),
            provenance: BTreeMap::new(),
            b_initial: BTreeSet::new(),
            b_repair: BTreeSet::new(),
            b_fallback: BTreeSet::new(),
            focused_sent: 0,
            focused_resolved: 0,
            reconsidered_ranked: 0,
            boundary_filtered_occurrences: 0,
            boundary_filtered_readings: 0,
        })
    }

    fn includes(&self, sentence: &str) -> bool {
        self.hash_includes(sentence) || self.pins.contains(sentence)
    }

    fn hash_includes(&self, sentence: &str) -> bool {
        // Fixed text hash, not corpus position: additions and reordering cannot
        // change membership, and larger percentages contain smaller samples.
        let hash = xxhash_rust::xxh3::xxh3_64(sentence.as_bytes());
        (hash as f64) / (u64::MAX as f64) * 100.0 < self.percent || self.percent == 100.0
    }

    pub(super) fn groups(&self, groups: &Groups) -> Groups {
        groups
            .iter()
            .filter_map(|(gram, occurrences)| {
                let occurrences: Vec<_> = occurrences
                    .iter()
                    .filter(|o| self.includes(&o.text))
                    .cloned()
                    .collect();
                (!occurrences.is_empty()).then(|| (gram.clone(), occurrences))
            })
            .collect()
    }

    pub(super) fn record_accepted(
        &mut self,
        accepted: &[(&HierarchyRequest, Proposal<GroundedReading>)],
        round: &'static str,
    ) {
        for (request, _) in accepted {
            self.provenance.insert(request.gram.clone(), round);
        }
    }

    pub(super) fn write(
        &self,
        data: &CorpusReadings,
        groups: &Groups,
        language: Language,
    ) -> Result<()> {
        let directory = std::path::PathBuf::from("target/readings-sample").join(language.code());
        std::fs::create_dir_all(&directory)?;
        let final_labels = labels(data, groups);
        let mut hierarchies =
            std::io::BufWriter::new(std::fs::File::create(directory.join("hierarchies.jsonl"))?);
        let mut occurrences =
            std::io::BufWriter::new(std::fs::File::create(directory.join("occurrences.jsonl"))?);
        for (gram, examples) in groups {
            writeln!(
                hierarchies,
                "{}",
                json!({
                    "gram": gram, "text": gram.to_display_string(language),
                    "provenance": self.provenance.get(gram).copied().unwrap_or("b_failed"),
                    "b_initial_attempted": self.b_initial.contains(gram),
                    "b_repair_attempted": self.b_repair.contains(gram),
                    "b_fallback_used": self.b_fallback.contains(gram),
                    "correction_dropped": data.corrections_dropped.contains(gram),
                    "hierarchy": data.hierarchies.get(gram),
                })
            )?;
            for o in examples {
                let key = (o.text.clone(), o.gram_index);
                writeln!(
                    occurrences,
                    "{}",
                    json!({
                        "sentence": o.text, "marked": o.marked(),
                        "gram": gram, "text": gram.to_display_string(language), "gram_index": o.gram_index,
                        "initial": self.initial[&key], "final": final_labels[&key],
                        "repair_attempted": self.b_repair.contains(gram),
                        "correction_dropped": data.corrections_dropped.contains(gram),
                    })
                )?;
            }
        }
        hierarchies.flush()?;
        occurrences.flush()?;
        let sentences: BTreeSet<_> = final_labels.keys().map(|(text, _)| text).collect();
        let hash_sentences = sentences
            .iter()
            .filter(|text| self.hash_includes(text))
            .count();
        let pinned_sentences = sentences
            .iter()
            .filter(|text| self.pins.contains(**text))
            .count();
        let pin_only_sentences = sentences
            .iter()
            .filter(|text| self.pins.contains(**text) && !self.hash_includes(text))
            .count();
        let summary = json!({
            "language": language.code(), "percent": self.percent,
            "sentences": sentences.len(),
            "hash_sentences": hash_sentences,
            "pinned_sentences": pinned_sentences,
            "pin_only_sentences": pin_only_sentences,
            "grams": groups.len(), "occurrences": final_labels.len(),
            "b_initial_requests": self.b_initial.len(), "b_repair_requests": self.b_repair.len(),
            "b_fallback_grams": self.b_fallback.len(),
            "focused_followups": {"sent":self.focused_sent,"resolved":self.focused_resolved},
            "reconsidered_ranked": self.reconsidered_ranked,
            "boundary_filter": {"occurrence_requests":self.boundary_filtered_occurrences,"reading_slots":self.boundary_filtered_readings},
            "corrections_dropped_grams": data.corrections_dropped.len(),
            "hierarchy_provenance": counts(groups.keys().map(|gram| self.provenance.get(gram).copied().unwrap_or("b_failed"))),
            "initial": label_counts(&self.initial), "final": label_counts(&final_labels),
        });
        std::fs::write(
            directory.join("summary.json"),
            serde_json::to_string_pretty(&summary)?,
        )?;
        println!("readings sample QA: {}", directory.display());
        Ok(())
    }
}

pub(super) fn labels(data: &CorpusReadings, groups: &Groups) -> Labels {
    groups
        .iter()
        .flat_map(|(gram, occurrences)| {
            occurrences.iter().map(move |o| {
                let key = (o.text.clone(), o.gram_index);
                let mut label = data.labels.get(&key).map_or_else(
                    || json!({"kind":"unset"}),
                    |label| {
                        let mut value = serde_json::to_value(label).unwrap();
                        if let Some(choices) = label.choices() {
                            value["choices"] = json!(choices.iter().map(|choice| json!({
                    "sense": choice.sense, "reading": choice.reading,
                    "gloss": data.hierarchies.get(gram).into_iter().flatten()
                        .find(|sense| sense.index == choice.sense).map(|sense| &sense.gloss),
                })).collect::<Vec<_>>());
                        }
                        value
                    },
                );
                label["reading"] = json!(data.reading(&o.text, o.gram_index));
                label["round"] = if data.labels.contains_key(&key) {
                    json!(data.sample.as_ref().and_then(|s| s.rounds.get(&key)))
                } else {
                    Value::Null
                };
                (key, label)
            })
        })
        .collect()
}

fn counts<'a>(values: impl Iterator<Item = &'a str>) -> BTreeMap<&'a str, usize> {
    let mut counts = BTreeMap::new();
    for kind in values {
        *counts.entry(kind).or_default() += 1;
    }
    counts
}
fn label_counts(labels: &Labels) -> Value {
    let mut kinds = counts(labels.values().map(|label| label["kind"].as_str().unwrap()));
    for kind in ["ranked", "restated", "escape", "unset"] {
        kinds.entry(kind).or_default();
    }
    json!({"types": kinds, "escape_rate": if labels.is_empty() { 0.0 } else { kinds["escape"] as f64 / labels.len() as f64 }})
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(percent: f64) -> Result<Sample> {
        Sample::with_pins(percent, BTreeSet::new())
    }

    #[test]
    fn pins_override_hash_membership_without_trimming() {
        let text = "猫がいる。";
        let pinned = Sample::with_pins(1.0, BTreeSet::from([text.into()])).unwrap();
        assert!(!sample(1.0).unwrap().includes(text));
        assert!(pinned.includes(text));
        assert_eq!(
            pinned.includes(" 猫がいる。"),
            pinned.hash_includes(" 猫がいる。")
        );
    }

    #[tokio::test]
    async fn sampled_b_keeps_full_request_bytes_and_c_keeps_whole_sentences() {
        let language = Language::Japanese;
        let gram = super::super::tests::gram(&["猫"]);
        let examples: Vec<_> = (0..100)
            .map(|i| Occurrence {
                text: format!("猫 {i}"),
                gram_index: 0,
                bytes: (0, 3),
                spans: vec![(0, 1)],
            })
            .collect();
        let full = BTreeMap::from([(gram.clone(), examples)]);
        let sample = sample(10.0).unwrap();
        let subset = sample.groups(&full);
        assert!(!subset[&gram].is_empty() && subset[&gram].len() < full[&gram].len());
        let cache = tempfile::tempdir().unwrap();
        let store = osmo::Store::open(cache.path());
        let candidate = candidates(&gram, language, &Dictionary::new());
        let full_request = HierarchyRequest::sampled(
            &gram,
            candidate.clone(),
            &full[&gram],
            language,
            &store,
            None,
        )
        .await;
        // Exactly the construction in annotate: sampled keys, full occurrences.
        for selected in subset.keys() {
            let request = HierarchyRequest::sampled(
                selected,
                candidate.clone(),
                &full[selected],
                language,
                &store,
                None,
            )
            .await;
            assert_eq!(full_request.prompt.to_string(), request.prompt.to_string());
        }
        let mut doubled = full.clone();
        doubled
            .get_mut(&gram)
            .unwrap()
            .extend(full[&gram].iter().cloned().map(|mut o| {
                o.gram_index = 1;
                o
            }));
        assert_eq!(
            sample.groups(&doubled)[&gram].len(),
            subset[&gram].len() * 2
        );
    }

    #[test]
    fn initial_snapshots_keep_original_glosses() {
        let gram = super::super::tests::gram(&["猫"]);
        let groups = BTreeMap::from([(
            gram.clone(),
            vec![Occurrence {
                text: "猫".into(),
                gram_index: 0,
                bytes: (0, 3),
                spans: vec![(0, 1)],
            }],
        )]);
        let key = ("猫".into(), 0);
        let mut data = CorpusReadings {
            sample: Some(sample(100.0).unwrap()),
            ..Default::default()
        };
        data.hierarchies.insert(
            gram.clone(),
            vec![SenseReadings {
                index: 1,
                kind: "meaning".into(),
                synthetic_readings: BTreeSet::new(),
                gloss: "initial meaning".into(),
                readings: vec!["ねこ".into()],
            }],
        );
        data.labels.insert(
            key.clone(),
            occurrence::Label::Ranked {
                choices: vec![Choice {
                    sense: 1,
                    reading: "ねこ".into(),
                }],
            },
        );
        data.sample
            .as_mut()
            .unwrap()
            .rounds
            .insert(key.clone(), "initial".into());
        let initial = labels(&data, &groups);
        data.hierarchies.get_mut(&gram).unwrap()[0].gloss = "repaired meaning".into();
        assert_eq!(initial[&key]["choices"][0]["gloss"], "initial meaning");
        assert_eq!(initial[&key]["round"], "initial");
        assert_eq!(
            labels(&data, &groups)[&key]["choices"][0]["gloss"],
            "repaired meaning"
        );
        data.labels.clear();
        assert_eq!(labels(&data, &groups)[&key]["kind"], "unset");
        assert!(labels(&data, &groups)[&key]["round"].is_null());
    }

    #[test]
    fn membership_is_stable_nested_and_sentence_wide() {
        assert_eq!(
            xxhash_rust::xxh3::xxh3_64("猫がいる。".as_bytes()),
            0x68401872e3ffb4f4
        );
        assert!(!sample(40.0).unwrap().includes("猫がいる。"));
        assert!(sample(41.0).unwrap().includes("猫がいる。"));
        assert!(!sample(50.0).unwrap().includes("sentence 0"));
        assert!(sample(50.0).unwrap().includes("sentence 1"));
        let small = sample(5.0).unwrap();
        let large = sample(25.0).unwrap();
        let all = sample(100.0).unwrap();
        let texts: Vec<_> = (0..1000).map(|i| format!("sentence {i}")).collect();
        let selected: BTreeSet<_> = texts.iter().filter(|s| small.includes(s)).collect();
        assert!(!selected.is_empty() && selected.len() < 100);
        assert_eq!(
            selected,
            texts.iter().rev().filter(|s| small.includes(s)).collect()
        );
        assert!(selected.iter().all(|s| large.includes(s)));
        assert!(texts.iter().all(|s| all.includes(s)));
        for percent in [0.0, -1.0, 100.1, f64::NAN, f64::INFINITY] {
            assert!(sample(percent).is_err());
        }
    }
}
