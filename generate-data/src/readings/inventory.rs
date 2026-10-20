//! Corpus attestation and the boundary between implicit and explicit senses.
use super::*;

type Counts = BTreeMap<Choice, usize>;
fn leaf(sense: u32, reading: &str) -> Choice {
    Choice {
        sense,
        reading: reading.to_owned(),
    }
}
fn count(counts: &Counts, sense: u32, reading: &str) -> usize {
    counts.get(&leaf(sense, reading)).copied().unwrap_or(0)
}
/// First choices a meaning needs, summed across every reading variant.
pub(super) const MIN_ATTESTATIONS: usize = 3;

/// This map changes only retained identities (temporary -> implicit/durable).
/// A missing ID means reject the sentence, never choose a different meaning.
fn prune(senses: &[SenseReadings], counts: &Counts) -> (Vec<SenseReadings>, BTreeMap<u32, u32>) {
    let mut kept: Vec<_> = senses
        .iter()
        .filter(|sense| {
            sense
                .readings
                .iter()
                .map(|r| count(counts, sense.index, r))
                .sum::<usize>()
                >= MIN_ATTESTATIONS
        })
        .cloned()
        .collect();
    let implicit = kept.len() == 1;
    let ids = kept
        .iter_mut()
        .map(|sense| {
            let old = sense.index;
            if implicit {
                sense.index = 0;
            }
            (old, sense.index)
        })
        .collect();
    (kept, ids)
}

fn reading_set(language: Language, text: &str, readings: &[String]) -> BTreeSet<String> {
    readings
        .iter()
        .map(|r| {
            canonical::canonicalize(language, text, r)
                .expect("stored reading identities must be canonicalizable")
        })
        .collect()
}

/// The ledger is consulted only after attestation. B and C use deterministic
/// temporary IDs, so yesterday's generated slots cannot alter their requests.
/// Match before retiring anchors; exact gloss/readings break overlap ties and
/// provide a fallback only when an old slot has no surviving evidence.
fn stable_ids(
    row: &UsageInventory,
    senses: &[SenseReadings],
    language: Language,
    evidence: &crate::sense_identity::Evidence,
) -> BTreeMap<u32, u32> {
    let text = row.gram.to_display_string(language);
    crate::sense_identity::assign(
        row.usages.len() as u32,
        row.usages
            .iter()
            .zip(1..)
            .filter(|(u, _)| u.is_reading_split())
            .map(|(u, id)| {
                (
                    id,
                    if u.n_assigned > 0 {
                        u.anchors.as_slice()
                    } else {
                        &[]
                    },
                )
            }),
        senses.iter().map(|s| s.index),
        evidence,
        |old, new| {
            let usage = &row.usages[old as usize - 1];
            let sense = senses.iter().find(|s| s.index == new).unwrap();
            usage.gloss == sense.gloss
                && usage.readings.as_ref().is_some_and(|old| {
                    reading_set(language, &text, old)
                        == reading_set(language, &text, &sense.readings)
                })
        },
    )
}

fn materialize(
    data: &mut CorpusReadings,
    rows: &mut Vec<UsageInventory>,
    groups: &Groups,
    language: Language,
    raw_labels: &BTreeMap<(String, usize), occurrence::Label>,
) {
    data.rejected_sentences.clear();
    // Keep old anchors until each gram has been matched against fresh C labels.
    let mut materialized = BTreeSet::new();
    let indices: BTreeMap<_, _> = rows
        .iter()
        .enumerate()
        .map(|(i, r)| (r.gram.clone(), i))
        .collect();
    let mut monosemous = 0;
    let mut new_before = 0;
    let mut new_after = 0;
    let mut appended_slots = 0;
    let mut pruned_senses = 0;
    for (gram, occurrences) in groups {
        let Some(before) = data.hierarchies.get(gram) else {
            continue;
        };
        new_before += before.iter().filter(|s| s.index != 0).count();
        let mut counts = Counts::new();
        for o in occurrences {
            if let Some(first) = raw_labels
                .get(&(o.text.clone(), o.gram_index))
                .and_then(|v| v.first())
            {
                *counts.entry(first.clone()).or_default() += 1;
            }
        }
        let (mut after, mut remap) = prune(before, &counts);
        pruned_senses += before.len() - after.len();
        let monosemous_gram = after.len() <= 1;
        let row_index = (!monosemous_gram).then(|| {
            indices.get(gram).copied().unwrap_or_else(|| {
                rows.push(UsageInventory {
                    key: gram.to_display_string(language),
                    gram: gram.clone(),
                    n: occurrences.len(),
                    n_expression: 0,
                    usages: vec![],
                    absorbers: vec![],
                    silhouette: 0.0,
                    source: format!(
                        "reading-hierarchy:{}:{}",
                        hierarchy::settings(language).0,
                        hierarchy::settings(language).1
                    ),
                });
                rows.len() - 1
            })
        });
        if let Some(i) = row_index {
            let evidence = occurrences
                .iter()
                .filter_map(|o| {
                    let choice = raw_labels.get(&(o.text.clone(), o.gram_index))?.first()?;
                    Some(((o.text.clone(), o.spans.clone()), choice.sense))
                })
                .collect();
            let ids = stable_ids(&rows[i], &after, language, &evidence);
            rows[i].retire_usages();
            materialized.insert(i);
            for sense in &mut after {
                sense.index = ids[&sense.index];
            }
            for sense in remap.values_mut() {
                *sense = ids[sense];
            }
            after.sort_by_key(|s| s.index);
        }
        let mut support = Counts::new();
        for (choice, n) in &counts {
            if let Some(&sense) = remap.get(&choice.sense) {
                *support.entry(leaf(sense, &choice.reading)).or_default() += n;
            }
        }
        for o in occurrences {
            let key = (o.text.clone(), o.gram_index);
            let Some(raw) = raw_labels.get(&key).and_then(occurrence::Label::choices) else {
                // Unresolved escapes name no selected sense; pruning does not
                // invent one or reject their sentence on that basis.
                continue;
            };
            if !remap.contains_key(&raw[0].sense) {
                data.rejected_sentences.insert(o.text.clone());
                data.labels.remove(&key);
                continue;
            }
            let choices = raw
                .iter()
                .filter_map(|c| remap.get(&c.sense).map(|&sense| leaf(sense, &c.reading)))
                .collect();
            data.labels
                .insert(key, occurrence::Label::Ranked { choices });
        }
        if let Some(index) = row_index {
            for sense in &after {
                let anchors: Vec<_> = occurrences
                    .iter()
                    .filter(|o| {
                        raw_labels
                            .get(&(o.text.clone(), o.gram_index))
                            .and_then(|v| v.first())
                            .and_then(|c| remap.get(&c.sense))
                            .is_some_and(|&id| id == sense.index)
                    })
                    .take(16)
                    .map(|o| UsageAnchor {
                        sentence: o.text.clone(),
                        spans: o.spans.clone(),
                        gold: true,
                    })
                    .collect();
                assert!(
                    anchors.len() >= MIN_ATTESTATIONS,
                    "surviving senses need direct C attestation"
                );
                let entry = UsageEntry {
                    source: Some(format!(
                        "reading-hierarchy:{}:{}",
                        hierarchy::settings(language).0,
                        hierarchy::settings(language).1
                    )),
                    kind: sense.kind.clone(),
                    gloss: sense.gloss.clone(),
                    readings: Some(sense.readings.clone()),
                    n_gold: anchors.len(),
                    n_assigned: support
                        .iter()
                        .filter(|(c, _)| c.sense == sense.index)
                        .map(|(_, n)| n)
                        .sum(),
                    loo_correct: 0,
                    anchors,
                };
                new_after += 1;
                if sense.index as usize <= rows[index].usages.len() {
                    rows[index].usages[sense.index as usize - 1] = entry;
                } else {
                    assert_eq!(sense.index as usize, rows[index].usages.len() + 1);
                    rows[index].usages.push(entry);
                    appended_slots += 1;
                }
            }
        } else {
            monosemous += 1;
        }
        if after.is_empty() {
            data.hierarchies.remove(gram);
        } else {
            data.hierarchies.insert(gram.clone(), after);
        }
    }
    // Unsplit grams use the implicit card, not an invented explicit identity.
    // Missing grams and split -> implicit transitions retain the ledger only.
    for (i, row) in rows.iter_mut().enumerate() {
        if !materialized.contains(&i) {
            row.retire_usages();
        }
    }
    assert!(rows.iter().all(|r| r.usages.len() >= 2));
    println!(
        "readings[{}]: attestation new_senses_before={new_before}, new_senses_after={new_after}, appended_slots={appended_slots}, pruned_senses={pruned_senses}, dropped_sentences={}, monosemous_grams={monosemous}",
        language.code(),
        data.rejected_sentences.len()
    );
}

pub(super) fn finish(
    data: &mut CorpusReadings,
    rows: &mut Vec<UsageInventory>,
    groups: &Groups,
    language: Language,
) -> Result<()> {
    let raw_labels = data.labels.clone();
    let raw_hierarchies = data.hierarchies.clone();
    materialize(data, rows, groups, language, &raw_labels);
    if tysm::measurement::enabled() {
        return Ok(());
    }
    // Retain full evidence across reruns too: temporary IDs are interpreted
    // against the exact hierarchy C saw, not the subsequently compacted file.
    let mut file =
        std::io::BufWriter::new(std::fs::OpenOptions::new().create(true).append(true).open(
            format!(".cache/readings/{}-occurrences.jsonl", language.code()),
        )?);
    for (gram, occurrences) in groups {
        for o in occurrences {
            let key = (o.text.clone(), o.gram_index);
            let Some(choices) = raw_labels.get(&key) else {
                continue;
            };
            writeln!(
                file,
                "{}",
                serde_json::json!({
                    "gram":gram,"sentence":o.text,"gram_index":o.gram_index,
                    "hierarchy":raw_hierarchies.get(gram),"raw_label":choices,"label":data.labels.get(&key)
                })
            )?;
        }
    }
    file.flush()?;
    assert!(rows.iter().all(|r| r.usages.len() >= 2));
    write_inventories(
        &format!(
            "generate-data/data/{}/usage_inventories.jsonl",
            language.corpus_code()
        ),
        rows,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn s(index: u32, readings: &[&str]) -> SenseReadings {
        SenseReadings {
            index,
            kind: "meaning".into(),
            synthetic_readings: BTreeSet::new(),
            gloss: format!("meaning {index}"),
            readings: readings.iter().map(|r| (*r).into()).collect(),
        }
    }
    #[test]
    fn sense_attestation_sums_variants_and_never_maps_a_pruned_meaning() {
        let counts = BTreeMap::from([
            (leaf(1, "じっぷん"), 2),
            (leaf(1, "じゅっぷん"), 2),
            (leaf(2, "じゅうぶん"), 10),
            (leaf(3, "じゅうぶん"), 2),
        ]);
        let (after, ids) = prune(
            &[
                s(1, &["じっぷん", "じゅっぷん"]),
                s(2, &["じゅうぶん"]),
                s(3, &["じゅうぶん"]),
            ],
            &counts,
        );
        assert_eq!(after.len(), 2);
        assert_eq!(ids, BTreeMap::from([(1, 1), (2, 2)]));
        assert_eq!(after[0].readings, ["じっぷん", "じゅっぷん"]);
    }
    #[test]
    fn all_rare_senses_including_implicit_zero_are_pruned() {
        for senses in [vec![s(0, &["あ"])], vec![s(1, &["あ"]), s(2, &["い"])]] {
            let counts = senses
                .iter()
                .map(|s| (leaf(s.index, &s.readings[0]), 2))
                .collect();
            let (after, ids) = prune(&senses, &counts);
            assert!(after.is_empty());
            assert!(ids.is_empty());
        }
    }
    fn fixture(senses: Vec<SenseReadings>, choices: Vec<Vec<Choice>>) -> (CorpusReadings, Groups) {
        let gram = Gram(Vec::new());
        let occurrences: Vec<_> = choices
            .iter()
            .enumerate()
            .map(|(i, _)| Occurrence {
                text: format!("sentence {i}"),
                gram_index: 0,
                bytes: (0, 1),
                spans: vec![(0, 1)],
            })
            .collect();
        let labels = occurrences
            .iter()
            .zip(choices)
            .map(|(o, c)| {
                (
                    (o.text.clone(), 0),
                    occurrence::Label::Ranked { choices: c },
                )
            })
            .collect();
        (
            CorpusReadings {
                hierarchies: BTreeMap::from([(gram.clone(), senses)]),
                labels,
                ..Default::default()
            },
            BTreeMap::from([(gram, occurrences)]),
        )
    }
    #[test]
    fn packed_sentences_keep_c_readings_or_are_dropped_never_reassigned() {
        let choices = vec![
            vec![leaf(1, "じっぷん")],
            vec![leaf(1, "じっぷん")],
            vec![leaf(1, "じゅっぷん")],
            vec![leaf(1, "じゅっぷん")],
            vec![leaf(2, "じゅうぶん")],
            vec![leaf(2, "じゅうぶん")],
            vec![leaf(2, "じゅうぶん")],
            vec![leaf(3, "じゅうぶん"), leaf(2, "じゅうぶん")],
            vec![leaf(4, "かた")],
        ];
        let (mut data, groups) = fixture(
            vec![
                s(1, &["じっぷん", "じゅっぷん"]),
                s(2, &["じゅうぶん"]),
                s(3, &["じゅうぶん"]),
                s(4, &["かた"]),
            ],
            choices,
        );
        let raw = data.labels.clone();
        materialize(&mut data, &mut vec![], &groups, Language::Japanese, &raw);
        assert_eq!(
            data.rejected_sentences,
            BTreeSet::from(["sentence 7".into(), "sentence 8".into()])
        );
        let inputs: Vec<_> = (0..10)
            .map(|i| {
                let text = format!("sentence {i}");
                let sentence = SentenceGrams {
                    grams: vec![language_utils::SentenceGram::Learnable(TaggedGram {
                        gram: Gram(vec![]),
                        sense: data.sense(&text, 0),
                    })],
                    capitalize_first: false,
                    multiword_terms: vec![],
                    low_confidence_multiword_terms: vec![],
                };
                (text, sentence)
            })
            .collect();
        let packed = data.pack(inputs, Language::Japanese);
        assert_eq!(packed.len(), 8);
        for (text, sentence) in packed {
            assert!(data.includes_sentence(&text));
            let language_utils::SentenceGram::Learnable(entry) = &sentence.grams[0] else {
                panic!()
            };
            assert_eq!(
                entry.reading.as_deref(),
                raw.get(&(text, 0))
                    .and_then(|l| l.first())
                    .map(|c| c.reading.as_str())
            );
        }
    }
    #[test]
    fn secondary_rankings_do_not_attest_or_reject_the_first_choice() {
        let (mut data, groups) = fixture(
            vec![s(1, &["あ"]), s(2, &["い"])],
            vec![vec![leaf(1, "あ"), leaf(2, "い")]; 4],
        );
        let raw = data.labels.clone();
        materialize(&mut data, &mut vec![], &groups, Language::Japanese, &raw);
        assert!(data.rejected_sentences.is_empty());
        assert!(
            data.labels
                .values()
                .all(|l| l.choices() == Some(&[leaf(0, "あ")][..]))
        );
    }
    #[test]
    fn monosemous_materialization_does_not_mint_written_card_identities() {
        let mut choices = vec![vec![leaf(1, "a")]; 3];
        choices.push(vec![leaf(2, "a")]);
        let (mut data, groups) = fixture(vec![s(1, &["a"]), s(2, &["a"])], choices);
        let raw = data.labels.clone();
        let mut rows = Vec::new();
        materialize(&mut data, &mut rows, &groups, Language::Japanese, &raw);
        assert!(rows.is_empty());
        assert!(
            groups
                .values()
                .flatten()
                .all(|o| data.sense(&o.text, o.gram_index).is_none())
        );
        assert!(
            data.labels
                .values()
                .all(|c| c.first() == Some(&leaf(0, "a")))
        );
    }
    #[test]
    fn unresolved_meaning_escapes_stay_unresolved_without_sentence_rejection() {
        let mut choices = vec![vec![leaf(1, "ほう")]; 3];
        choices.extend(vec![vec![leaf(2, "ほう")]; 4]);
        choices.push(vec![]);
        let (mut data, groups) = fixture(
            vec![sense(1, "ほう", "方向"), sense(2, "ほう", "比較")],
            choices,
        );
        data.labels.insert(
            ("sentence 7".into(), 0),
            occurrence::Label::Escape {
                proposed_reading: "ほう".into(),
                reading: Some("ほう".into()),
                missing_meaning: Some("別の意味".into()),
            },
        );
        let mut rows = vec![row(vec![
            usage(None, None, "old a"),
            usage(None, None, "old b"),
        ])];
        let raw = data.labels.clone();
        materialize(&mut data, &mut rows, &groups, Language::Japanese, &raw);
        assert!(matches!(
            data.labels[&("sentence 7".into(), 0)],
            occurrence::Label::Escape { .. }
        ));
        assert!(data.includes_sentence("sentence 7"));
        assert_eq!(rows[0].usages[3].n_assigned, 4);
        assert_eq!(rows[0].usages[3].anchors.len(), 4);
    }

    #[test]
    fn escaped_tokens_keep_checked_readings_but_no_sense_through_materialization() {
        let (mut data, groups) = fixture(vec![s(0, &["a"])], vec![vec![leaf(0, "a")]; 4]);
        data.labels.insert(
            ("sentence 3".into(), 0),
            occurrence::Label::Escape {
                proposed_reading: "b".into(),
                missing_meaning: None,
                reading: Some("b".into()),
            },
        );
        let raw = data.labels.clone();
        let mut rows = Vec::new();
        materialize(&mut data, &mut rows, &groups, Language::Japanese, &raw);
        assert_eq!(
            data.labels
                .values()
                .filter(|label| label.first().is_some())
                .count(),
            3
        );
        assert_eq!(data.reading("sentence 3", 0), Some("b"));
        assert_eq!(data.sense("sentence 3", 0), None);
        assert!(data.includes_sentence("sentence 3"));
    }
    #[test]
    fn prior_reading_output_does_not_change_an_implicit_menu_or_identity() {
        let mut choices = vec![vec![leaf(1, "ほう")]; 3];
        choices.extend(vec![vec![leaf(2, "かた")]; 3]);
        let (mut previous, groups) = fixture(vec![s(1, &["ほう"]), s(2, &["かた"])], choices);
        let mut rows = Vec::new();
        let labels = previous.labels.clone();
        materialize(
            &mut previous,
            &mut rows,
            &groups,
            Language::Japanese,
            &labels,
        );
        assert_eq!(rows[0].usages.len(), 2);
        let (mut current, groups) = fixture(vec![s(0, &["ほう"])], vec![vec![leaf(0, "ほう")]; 3]);
        let labels = current.labels.clone();
        materialize(
            &mut current,
            &mut rows,
            &groups,
            Language::Japanese,
            &labels,
        );
        assert_eq!(rows[0].usages.len(), 2);
        assert!(
            rows[0]
                .usages
                .iter()
                .all(|u| u.n_assigned == 0 && u.anchors.is_empty())
        );
        assert_eq!(current.sense("sentence 0", 0), None);
        assert_eq!(current.hierarchies.values().next().unwrap()[0].index, 0);
        // Returning to an identical explicit split can reclaim the old slots.
        let split = run(
            &mut rows,
            vec![s(1, &["ほう"]), s(2, &["かた"])],
            &[1, 1, 1, 2, 2, 2],
        );
        assert_eq!(split.sense("sentence 0", 0).unwrap().get(), 1);
        assert_eq!(split.sense("sentence 3", 0).unwrap().get(), 2);
        assert_eq!(rows[0].usages.len(), 2);
    }
    fn usage(reading: Option<&str>, source: Option<&str>, gloss: &str) -> UsageEntry {
        UsageEntry {
            source: source.map(str::to_owned),
            readings: reading.map(|r| vec![r.into()]),
            kind: "meaning".into(),
            gloss: gloss.into(),
            n_gold: 5,
            n_assigned: 5,
            loo_correct: 5,
            anchors: vec![UsageAnchor {
                sentence: "old evidence".into(),
                spans: vec![(0, 1)],
                gold: true,
            }],
        }
    }
    fn row(usages: Vec<UsageEntry>) -> UsageInventory {
        UsageInventory {
            key: String::new(),
            gram: Gram(Vec::new()),
            n: 0,
            n_expression: 0,
            absorbers: vec![],
            silhouette: 0.0,
            source: "usage-discovery".into(),
            usages,
        }
    }
    fn sense(index: u32, reading: &str, gloss: &str) -> SenseReadings {
        SenseReadings {
            gloss: gloss.into(),
            ..s(index, &[reading])
        }
    }
    fn run(
        rows: &mut Vec<UsageInventory>,
        senses: Vec<SenseReadings>,
        labels: &[u32],
    ) -> CorpusReadings {
        let choices = labels
            .iter()
            .map(|&id| {
                vec![leaf(
                    id,
                    &senses.iter().find(|s| s.index == id).unwrap().readings[0],
                )]
            })
            .collect();
        let (mut data, groups) = fixture(senses, choices);
        let raw = data.labels.clone();
        materialize(&mut data, rows, &groups, Language::Japanese, &raw);
        data
    }

    #[test]
    fn exact_metadata_reclaims_retired_reading_slots() {
        let mut retired = usage(Some("ホウ"), Some("reading-hierarchy:old"), "方向");
        retired.anchors.clear();
        retired.n_assigned = 0;
        let mut rows = vec![row(vec![
            retired,
            usage(Some("かた"), Some("reading-hierarchy:old"), "人"),
        ])];
        let data = run(
            &mut rows,
            vec![sense(1, "ほう", "方向"), sense(2, "かた", "人")],
            &[1, 1, 1, 2, 2, 2],
        );
        assert_eq!(rows[0].usages.len(), 2);
        assert_eq!(data.sense("sentence 0", 0).unwrap().get(), 1);
        assert_eq!(rows[0].usages[0].readings.as_ref().unwrap(), &["ほう"]);
    }

    #[test]
    fn rename_and_reading_rewrite_follow_occurrences_and_rerun_stably() {
        let mut rows = vec![];
        run(
            &mut rows,
            vec![s(1, &["あ"]), s(2, &["い"])],
            &[1, 1, 1, 2, 2, 2],
        );
        let renamed = vec![
            sense(1, "う", "new wording"),
            sense(2, "え", "other wording"),
        ];
        run(&mut rows, renamed.clone(), &[1, 1, 1, 2, 2, 2]);
        assert_eq!(rows[0].usages.len(), 2);
        assert_eq!(rows[0].usages[0].gloss, "new wording");
        assert_eq!(rows[0].usages[0].readings.as_ref().unwrap(), &["う"]);
        let bytes = serde_json::to_string(&rows).unwrap();
        run(&mut rows, renamed, &[1, 1, 1, 2, 2, 2]);
        assert_eq!(serde_json::to_string(&rows).unwrap(), bytes);
    }

    #[test]
    fn split_plurality_beats_matching_gloss_and_unrelated_sense_appends() {
        let mut rows = vec![];
        run(
            &mut rows,
            vec![s(1, &["あ"]), s(2, &["い"])],
            &[1, 1, 1, 1, 1, 1, 1, 2, 2, 2],
        );
        // Four of slot 1's seven anchors choose new 3, only three choose new 1
        // despite new 1 retaining the old gloss and reading.
        let data = run(
            &mut rows,
            vec![s(1, &["あ"]), s(2, &["い"]), s(3, &["う"]), s(4, &["え"])],
            &[1, 1, 1, 3, 3, 3, 3, 2, 2, 2, 4, 4, 4],
        );
        assert_eq!(data.sense("sentence 3", 0).unwrap().get(), 1);
        assert_eq!(data.sense("sentence 0", 0).unwrap().get(), 3);
        assert_eq!(data.sense("sentence 10", 0).unwrap().get(), 4);
        assert_eq!(rows[0].usages[0].gloss, "meaning 3");
    }

    #[test]
    fn merge_winners_keep_ids_and_losers_retire_stably() {
        let mut rows = vec![];
        let original = vec![s(1, &["あ"]), s(2, &["い"]), s(3, &["う"]), s(4, &["え"])];
        run(
            &mut rows,
            original,
            &[1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4],
        );
        run(
            &mut rows,
            vec![s(1, &["あ"]), s(3, &["う"]), s(4, &["え"])],
            &[1, 1, 1, 1, 1, 1, 1, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4],
        );
        assert_eq!(rows[0].usages[0].n_assigned, 0);
        assert!(rows[0].usages[0].anchors.is_empty());
        // Slot 3's eight old anchors beat slot 2's seven after the first merge.
        let merged = vec![s(1, &["あ"]), s(4, &["え"])];
        let labels = &[1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 4, 4, 4];
        run(&mut rows, merged.clone(), labels);
        assert_eq!(rows[0].usages[0].n_assigned, 0);
        assert!(rows[0].usages[0].anchors.is_empty());
        assert_eq!(rows[0].usages[1].n_assigned, 0);
        assert_eq!(rows[0].usages[2].n_assigned, 15);
        assert_eq!(rows[0].usages.len(), 4);
        let bytes = serde_json::to_string(&rows).unwrap();
        run(&mut rows, merged, labels);
        assert_eq!(serde_json::to_string(&rows).unwrap(), bytes);
    }

    #[test]
    fn matching_uses_all_new_occurrences_before_capping_stored_anchors() {
        let senses = vec![s(1, &["あ"]), s(2, &["い"])];
        let labels: Vec<_> = std::iter::repeat_n(1, 40)
            .chain(std::iter::repeat_n(2, 40))
            .collect();
        let mut rows = vec![];
        run(&mut rows, senses.clone(), &labels);
        assert!(rows[0].usages.iter().all(|u| u.anchors.len() == 16));
        let choices = labels
            .iter()
            .map(|&id| vec![leaf(id, if id == 1 { "あ" } else { "い" })])
            .collect();
        let (mut data, mut groups) = fixture(senses, choices);
        // The new first 16 anchors per sense are disjoint from the old 16.
        groups
            .values_mut()
            .for_each(|occurrences| occurrences.reverse());
        let raw = data.labels.clone();
        materialize(&mut data, &mut rows, &groups, Language::Japanese, &raw);
        assert_eq!(rows[0].usages.len(), 2);
        assert!(
            rows[0]
                .usages
                .iter()
                .all(|u| u.anchors.len() == 16 && u.n_assigned == 40)
        );
    }

    #[test]
    fn new_evidence_is_not_capped_and_occurrence_spans_matter() {
        let mut old = usage(Some("あ"), Some("reading-hierarchy:old"), "old");
        old.anchors[0].sentence = "late sentence".into();
        old.anchors[0].spans = vec![(4, 5)];
        let mut row = row(vec![old, usage(None, None, "semantic")]);
        let mut evidence: crate::sense_identity::Evidence = (0..100)
            .map(|i| ((format!("sentence {i}"), vec![(0, 1)]), 1))
            .collect();
        evidence.insert(("late sentence".into(), vec![(0, 1)]), 1);
        evidence.insert(("late sentence".into(), vec![(4, 5)]), 2);
        let senses = vec![s(1, &["あ"]), s(2, &["い"])];
        let matched = stable_ids(&row, &senses, Language::Japanese, &evidence);
        assert_eq!(matched, BTreeMap::from([(1, 3), (2, 1)]));
        row.retire_usages();
        let matched = stable_ids(&row, &senses, Language::Japanese, &evidence);
        assert_eq!(matched, BTreeMap::from([(1, 3), (2, 4)]));
    }
    #[test]
    fn every_old_slot_retires_and_new_senses_land_past_the_high_water_mark() {
        // The committed inventory's meaning-discovery usages own ids 1 and 2.
        let mut rows = vec![row(vec![
            usage(None, None, "方向"),
            usage(None, None, "人"),
        ])];
        let mut choices = vec![vec![leaf(1, "ほう")]; 3];
        choices.extend(vec![vec![leaf(2, "かた")]; 3]);
        let (mut data, groups) = fixture(
            vec![sense(1, "ほう", "方向"), sense(2, "かた", "人")],
            choices,
        );
        let raw = data.labels.clone();
        materialize(&mut data, &mut rows, &groups, Language::Japanese, &raw);
        let usages = &rows[0].usages;
        assert_eq!(usages.len(), 4);
        // Old ids keep their place but carry no evidence, so nothing uses them.
        assert!(
            usages[..2]
                .iter()
                .all(|u| u.anchors.is_empty() && u.n_assigned == 0)
        );
        assert!(
            usages[2..]
                .iter()
                .all(|u| u.is_reading_split() && u.anchors.len() == 3)
        );
        assert_eq!(data.sense("sentence 0", 0).map(|s| s.get()), Some(3));
        assert_eq!(data.sense("sentence 3", 0).map(|s| s.get()), Some(4));
        // A rerun with the same senses keeps the same ids and appends nothing.
        let (mut again, groups) = fixture(
            vec![sense(1, "ほう", "方向"), sense(2, "かた", "人")],
            (0..6)
                .map(|i| {
                    vec![leaf(
                        if i < 3 { 1 } else { 2 },
                        if i < 3 { "ほう" } else { "かた" },
                    )]
                })
                .collect(),
        );
        let raw = again.labels.clone();
        materialize(&mut again, &mut rows, &groups, Language::Japanese, &raw);
        assert_eq!(rows[0].usages.len(), 4);
        assert_eq!(again.sense("sentence 0", 0).map(|s| s.get()), Some(3));
    }
    #[tokio::test]
    async fn rewrite_is_byte_identical_and_hits_tysm_cache() {
        use std::io::{BufRead, Read};
        let cache = tempfile::tempdir().unwrap();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/v1/", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(10)))
                .unwrap();
            let mut reader = std::io::BufReader::new(stream.try_clone().unwrap());
            let mut length = 0;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line == "\r\n" {
                    break;
                }
                if let Some(value) = line.to_lowercase().strip_prefix("content-length:") {
                    length = value.trim().parse().unwrap();
                }
            }
            reader.read_exact(&mut vec![0; length]).unwrap();
            let body = serde_json::json!({"id":"local", "object":"chat.completion", "created":0, "model":"gpt-6-luna", "choices":[{"index":0,"message":{"role":"assistant","content":"{\"reopen\":false,\"senses\":[]}"},"finish_reason":"stop"}],"usage":{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2}}).to_string();
            write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).unwrap();
        });
        let choices = vec![vec![leaf(1, "ほう")]; 3]
            .into_iter()
            .chain(vec![vec![leaf(2, "かた")]; 3])
            .chain([vec![leaf(1, "こ")]])
            .collect::<Vec<_>>();
        let make = || {
            fixture(
                vec![s(1, &["ほう", "こ"]), s(2, &["かた"])],
                choices.clone(),
            )
        };
        let (mut first, groups) = make();
        let gram = groups.keys().next().unwrap();
        let c = candidates(gram, Language::Japanese, &Dictionary::new());
        let prompt = prompts::hierarchy_prompt(gram, &c, Language::Japanese).to_string();
        let system = prompts::hierarchy_system_prompt(Language::Japanese);
        let client = ChatClient::new("local-test-key", "gpt-6-luna")
            .with_reasoning_effort("medium")
            .with_url(&url)
            .with_cache_directory(cache.path());
        let _: Proposal = client
            .chat_with_system_prompt(&system, &prompt)
            .await
            .unwrap();
        server.join().unwrap();
        let mut rows = Vec::new();
        let raw = first.labels.clone();
        materialize(&mut first, &mut rows, &groups, Language::Japanese, &raw);
        let path = cache.path().join("usages.jsonl");
        write_inventories(path.to_str().unwrap(), &rows).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        // B never reads the inventory, so the rewrite cannot change its request.
        let rewritten_prompt = prompts::hierarchy_prompt(gram, &c, Language::Japanese).to_string();
        assert_eq!(prompt, rewritten_prompt);
        let (mut second, _) = make();
        let raw = second.labels.clone();
        materialize(&mut second, &mut rows, &groups, Language::Japanese, &raw);
        write_inventories(path.to_str().unwrap(), &rows).unwrap();
        assert_eq!(bytes, std::fs::read(&path).unwrap());
        assert_eq!(
            rows[0].usages.len(),
            2,
            "pruned raw alternatives must not mint new slots on rewrite"
        );
        let cached = ChatClient::new("local-test-key", "gpt-6-luna")
            .with_reasoning_effort("medium")
            .with_url(&url)
            .with_cache_directory(cache.path())
            .with_cached_only();
        let requests = [rewritten_prompt];
        let answers = cached
            .batch_chat_with_system_prompt_fn::<_, _, Proposal>(
                &system,
                &requests,
                |p| p.clone(),
                |_| {},
            )
            .await
            .unwrap();
        assert!(answers.iter().all(|(_, answer)| answer.is_ok()));
        assert_eq!(cached.usage().total_tokens, 0);
        assert_eq!(cached.batch_usage().total_tokens, 0);
    }
}
