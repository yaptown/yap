//! Structural checks on B's hierarchy, separate from linguistic judgments.
use super::canonical::{canonicalize, japanese_pattern, menu_form};
use super::{GroundedReading, HierarchyRequest, Proposal, ProposedSense, kanjidic};
use language_utils::Language;
use serde::Serialize;

/// Per-check counts are not exclusive: one reading can fail two checks.
#[derive(Default, Debug, Serialize)]
pub(super) struct Rejections {
    pub schema_answers: usize,
    pub structure_answers: usize,
    pub canonical_readings: usize,
    pub okurigana_readings: usize,
    pub kanjidic_readings: usize,
    pub index_disagreements: usize,
    /// Readings outside the dictionary that named no example needing them.
    pub ungrounded_new_readings: usize,
    pub empty_senses: usize,
    pub sound_change_merges: usize,
    /// The current proposal's concrete failures; aggregate counters remain numeric.
    #[serde(skip)]
    pub details: Vec<String>,
}

impl Rejections {
    fn require(&mut self, valid: bool, explanation: impl FnOnce() -> String) -> bool {
        if !valid {
            self.details.push(explanation());
        }
        valid
    }
}

pub(super) fn prepare_proposal(
    proposal: Proposal,
    request: &HierarchyRequest,
    language: Language,
    kanji: &kanjidic::Verifier,
    checks: &mut Rejections,
) -> Option<Proposal<GroundedReading>> {
    checks.details.clear();
    let mut structure_ok = checks.require(!proposal.senses.is_empty(), || {
        "The hierarchy has no senses.".into()
    });
    let mut readings_ok = true;
    let text = request.gram.to_display_string(language);
    let pattern = (language == Language::Japanese).then(|| japanese_pattern(&text));
    let candidates = request.candidates.ordered_readings();
    let open = request.candidates.open || proposal.reopen;
    let mut senses = Vec::new();
    for sense in proposal.senses {
        structure_ok &= checks.require(!sense.gloss.trim().is_empty(), || {
            "A sense has an empty gloss.".into()
        });
        for source in &sense.grounding.sources {
            structure_ok &= checks.require(
                request
                    .candidates
                    .dictionary_evidence()
                    .any(|e| &e.source == source),
                || format!("Source {source:?} is not among the displayed dictionary sources."),
            );
        }
        for &line in &sense.grounding.examples {
            structure_ok &= checks.require(line < request.examples.len(), || {
                format!(
                    "Example {line} is outside the {} displayed examples.",
                    request.examples.len()
                )
            });
        }
        if let Some(existing) = sense.grounding.existing_sense {
            let exists = request.prompt["existing_hierarchy"]
                .as_array()
                .is_some_and(|senses| {
                    senses
                        .iter()
                        .any(|s| s["index"].as_u64() == Some(existing.into()))
                });
            structure_ok &= checks.require(exists, || {
                format!("Existing sense {existing} is not in the supplied hierarchy.")
            });
        }
        let mut grounded = Vec::new();
        for answer in &sense.readings {
            // A listed candidate needs no corpus evidence: C attestation prunes
            // what learners never meet. A new reading needs the line that shows it.
            if answer.index.is_none() && open && answer.example_line.is_none() {
                checks.ungrounded_new_readings += 1;
                checks.details.push(format!(
                    "Unlisted reading {:?} needs an example_line showing its use.",
                    answer.reading
                ));
                continue;
            }
            structure_ok &= checks.require(
                answer
                    .example_line
                    .is_none_or(|line| line < request.examples.len()),
                || {
                    format!(
                        "Reading {:?} cites example {}, outside the {} displayed examples.",
                        answer.reading,
                        answer.example_line.unwrap(),
                        request.examples.len()
                    )
                },
            );
            let reading = canonicalize(language, &text, &answer.reading)
                .and_then(|r| super::learner_lemma_form(&request.candidates.atoms, &r));
            let Some(reading) =
                reading.and_then(|r| menu_form(language, &text, &r, &request.candidates.readings))
            else {
                checks.canonical_readings += 1;
                checks.details.push(format!("Reading {:?} cannot be normalized for marked gram {text:?} and its lemma atoms.", answer.reading));
                readings_ok = false;
                continue;
            };
            if pattern.as_ref().is_some_and(|p| !p.is_match(&reading)) {
                checks.okurigana_readings += 1;
                checks.details.push(format!(
                    "Reading {reading:?} does not preserve the written kana of {text:?}."
                ));
                readings_ok = false;
            }
            if language == Language::Japanese
                && open
                && !request.candidates.whole_word_readings.contains(&reading)
                && !kanjidic::matches(&text, &reading, kanji)
            {
                // This is a trigger for focused verification, not a veto: names
                // and colloquial readings need not compose from dictionary parts.
                checks.kanjidic_readings += 1;
            }
            // The index and text must name the same candidate; the pair catches
            // copying mistakes. A closed menu has no room for unlisted readings.
            if answer.index.is_some() || !open {
                let expected = answer
                    .index
                    .and_then(|index| candidates.get(index))
                    .and_then(|candidate| canonicalize(language, &text, candidate));
                if expected.as_deref() != Some(&reading) {
                    checks.index_disagreements += 1;
                    let matching = candidates.iter().position(|candidate| {
                        canonicalize(language, &text, candidate).as_deref() == Some(&reading)
                    });
                    let index = answer
                        .index
                        .map_or_else(|| "null".into(), |i| i.to_string());
                    let expected = expected.as_deref().unwrap_or("no displayed candidate");
                    let matching = matching.map_or_else(
                        || "no displayed index matches".into(),
                        |i| format!("the matching displayed index is {i}"),
                    );
                    checks.details.push(format!("Index {index} names {expected:?}, not {reading:?}; {matching}. Unlisted readings require reopen and an example_line."));
                    readings_ok = false;
                }
            }
            grounded.push(GroundedReading {
                reading,
                example: answer.example_line.filter(|_| answer.index.is_none()),
            });
        }
        senses.push(ProposedSense {
            grounding: sense.grounding,
            kind: sense.kind,
            gloss: sense.gloss,
            readings: grounded,
        });
    }
    if !structure_ok {
        checks.structure_answers += 1;
    }
    if !structure_ok || !readings_ok {
        return None;
    }
    // Distinct dictionary readings with exactly one identical dictionary
    // meaning (十分 じっぷん/じゅっぷん) are alternatives, not new senses.
    // Shared phonology alone says nothing about meaning (君/方).
    let meaning = |sense: &ProposedSense<GroundedReading>| -> Option<&String> {
        let mut meanings = sense.readings.iter().map(|r| {
            let meanings = request.candidates.senses.get(&r.reading)?;
            (meanings.len() == 1).then_some(&meanings[0])
        });
        let first = meanings.next()??;
        meanings.all(|m| m == Some(first)).then_some(first)
    };
    let mut i = 0;
    while language == Language::Japanese && i < senses.len() {
        if let Some(j) = (0..i).find(|&j| {
            meaning(&senses[i]).is_some()
                && senses[i].grounding.existing_sense == senses[j].grounding.existing_sense
                && meaning(&senses[i]) == meaning(&senses[j])
                && !senses[i]
                    .readings
                    .iter()
                    .any(|r| senses[j].readings.iter().any(|s| r.reading == s.reading))
        }) {
            let removed = senses.remove(i);
            senses[j]
                .grounding
                .sources
                .extend(removed.grounding.sources);
            senses[j]
                .grounding
                .examples
                .extend(removed.grounding.examples);
            senses[j].readings.extend(removed.readings);
        } else {
            i += 1;
        }
    }
    let mut proposal = Proposal {
        reopen: proposal.reopen,
        senses,
    };
    if !normalize_senses(&mut proposal, language, checks) {
        checks
            .details
            .push("No sense retained a usable reading after normalization.".into());
        return None;
    }
    if !covers_forced_examples(&proposal, request) {
        checks.structure_answers += 1;
        checks.details.push("A forced missing-meaning example is not cited by any surviving sense in grounding.examples.".into());
        return None;
    }
    Some(proposal)
}

pub(super) fn covers_forced_examples(
    proposal: &Proposal<GroundedReading>,
    request: &HierarchyRequest,
) -> bool {
    request.prompt["occurrence_escapes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|e| !e["missing_meaning"].is_null())
        .all(|example| {
            let line = example["example_index"].as_u64().unwrap() as usize;
            proposal
                .senses
                .iter()
                .any(|s| !s.readings.is_empty() && s.grounding.examples.contains(&line))
        })
}

/// Shared by B, corrections and verification, which can all change readings.
/// Senses that share a reading are separate meanings and stay separate.
/// Returns false when no reading is left, so the gram is retried or dropped.
pub(super) fn normalize_senses(
    proposal: &mut Proposal<GroundedReading>,
    language: Language,
    checks: &mut Rejections,
) -> bool {
    let senses = &mut proposal.senses;
    senses.retain(|sense| {
        checks.empty_senses += usize::from(sense.readings.is_empty());
        !sense.readings.is_empty()
    });
    if language == Language::Japanese {
        // Phonology can coalesce alternatives only within an identical meaning.
        // Dictionary-backed equivalences are handled above; never manufacture
        // semantic equivalence by concatenating two different glosses.
        while let Some((source, target)) = senses.iter().enumerate().find_map(|(i, sense)| {
            senses.iter().enumerate().find_map(|(j, other)| {
                (i != j
                    && sense.gloss == other.gloss
                    && sense.kind == other.kind
                    && sense.grounding.existing_sense == other.grounding.existing_sense
                    && sense.readings.iter().all(|r| {
                        other
                            .readings
                            .iter()
                            .any(|s| super::canonical::sound_changed(&r.reading, &s.reading))
                    }))
                .then_some((i, j))
            })
        }) {
            let removed = senses.remove(source);
            let target = &mut senses[target - usize::from(source < target)];
            target.grounding.sources.extend(removed.grounding.sources);
            target.grounding.examples.extend(removed.grounding.examples);
            target.readings.extend(removed.readings);
            checks.sound_change_merges += 1;
        }
    }
    for sense in senses.iter_mut() {
        let mut seen = std::collections::BTreeMap::new();
        for r in std::mem::take(&mut sense.readings) {
            let example: &mut Option<usize> = seen.entry(r.reading).or_default();
            *example = example.or(r.example);
        }
        sense.readings = seen
            .into_iter()
            .map(|(reading, example)| GroundedReading { reading, example })
            .collect();
    }
    !senses.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::readings::{Candidates, Occurrence, ProposedReading};
    use crate::usage_discovery::UsageKind;
    use language_utils::{Atom, Gram, Heteronym, PartOfSpeech, Word, WordType};

    fn request(candidates: &[&str]) -> HierarchyRequest {
        HierarchyRequest {
            gram: Gram(vec![Atom::Tok(Word {
                text: "看".into(),
                word_type: WordType::Heteronym(Heteronym {
                    word: "看".into(),
                    lemma: "看".into(),
                    pos: PartOfSpeech::Verb,
                }),
            })]),
            candidates: Candidates {
                senses: Default::default(),
                evidence: Default::default(),
                coverings: Default::default(),
                whole_word_readings: Default::default(),
                readings: candidates.iter().map(|r| (*r).to_owned()).collect(),
                open: candidates.is_empty(),
                atoms: vec![],
                prior_single: false,
            },
            prompt: serde_json::Value::Null,
            examples: vec![Occurrence {
                text: "看".into(),
                gram_index: 0,
                bytes: (0, 3),
                spans: vec![(0, 1)],
            }],
        }
    }
    fn reading(
        index: Option<usize>,
        reading: &str,
        example_line: Option<usize>,
    ) -> ProposedReading {
        ProposedReading {
            index,
            reading: reading.into(),
            example_line,
        }
    }
    fn sense(gloss: &str, readings: Vec<ProposedReading>) -> ProposedSense {
        ProposedSense {
            grounding: Default::default(),
            kind: UsageKind::Meaning,
            gloss: gloss.into(),
            readings,
        }
    }
    fn prepare(
        request: &HierarchyRequest,
        reopen: bool,
        senses: Vec<ProposedSense>,
    ) -> (Option<Proposal<GroundedReading>>, Rejections) {
        let mut checks = Rejections::default();
        let answer = prepare_proposal(
            Proposal { reopen, senses },
            request,
            Language::ChineseSimplified,
            &kanjidic::Verifier::default(),
            &mut checks,
        );
        (answer, checks)
    }
    #[test]
    fn priority_order_numbers_and_validation_share_one_menu() {
        let mut request = request(&["kàn", "kān"]);
        request.candidates.evidence.insert(
            "kān".into(),
            vec![crate::readings::DictionaryEvidence {
                source: "dictionary:common".into(),
                spelling: "看".into(),
                meaning: "look".into(),
                primary: "kān".into(),
                form: "primary".into(),
                notes: vec![],
                priority: vec!["ichi1".into()],
            }],
        );
        request.prompt = crate::readings::prompts::hierarchy_prompt(
            &request.gram,
            &request.candidates,
            Language::ChineseSimplified,
        );
        let offered = request.prompt["candidates"]["readings"].as_array().unwrap();
        assert_eq!(offered[0]["reading"], "kān");
        for (index, item) in offered.iter().enumerate() {
            assert_eq!(item["index"], index);
        }
        assert!(
            prepare(
                &request,
                false,
                vec![sense("look", vec![reading(Some(0), "kān", None)])]
            )
            .0
            .is_some()
        );
        let (proposal, checks) = prepare(
            &request,
            false,
            vec![sense("look", vec![reading(Some(1), "kān", None)])],
        );
        assert!(proposal.is_none());
        assert_eq!(checks.index_disagreements, 1);
        assert!(checks.details[0].contains("kàn"));
        assert!(checks.details[0].contains("the matching displayed index is 0"));
        assert!(!checks.details[0].contains("Some("));
        let (_, checks) = prepare(
            &request,
            false,
            vec![sense("look", vec![reading(Some(99), "kǒng", None)])],
        );
        assert!(
            checks
                .details
                .iter()
                .any(|s| s.contains("no displayed index matches"))
        );
        let (_, checks) = prepare(
            &request,
            false,
            vec![sense("look", vec![reading(Some(0), "kān", Some(9))])],
        );
        assert!(checks.details.iter().any(|s| s.contains("cites example 9")));
        assert!(checks.details.iter().all(|s| !s.contains("Some(")));
        let mut invalid = sense("look", vec![reading(Some(0), "kān", None)]);
        invalid.grounding.sources.push("dictionary:typo".into());
        let (_, checks) = prepare(&request, false, vec![invalid]);
        assert!(
            checks
                .details
                .iter()
                .any(|message| message.contains("dictionary:typo"))
        );
        let feedback =
            serde_json::json!({"rejected_answer":{"readings":[]},"checks":checks.details});
        let retry = crate::readings::prompts::hierarchy_retry(&request, &feedback);
        assert!(retry.contains("dictionary:typo"));
        assert!(retry.contains("rejected_answer"));
    }

    fn grounded(gloss: &str, readings: &[&str]) -> ProposedSense<GroundedReading> {
        ProposedSense {
            grounding: Default::default(),
            kind: UsageKind::Meaning,
            gloss: gloss.into(),
            readings: readings
                .iter()
                .map(|r| GroundedReading {
                    reading: (*r).into(),
                    example: None,
                })
                .collect(),
        }
    }

    #[test]
    fn repair_requires_explicit_existing_references_and_forced_meaning_coverage() {
        let mut request = request(&["kàn"]);
        request.prompt = serde_json::json!({"existing_hierarchy":[{"index":7}], "occurrence_escapes":[{"example_index":0,"missing_meaning":"visit"}]});
        let mut extension = sense("visit", vec![reading(Some(0), "kàn", None)]);
        extension.grounding.existing_sense = Some(7);
        assert!(
            prepare(&request, false, vec![extension.clone()])
                .0
                .is_none()
        );
        extension.grounding.examples.push(0);
        assert!(
            prepare(&request, false, vec![extension.clone()])
                .0
                .is_some()
        );
        let mut accepted = prepare(&request, false, vec![extension.clone()]).0.unwrap();
        accepted.senses[0].readings.clear();
        assert!(!covers_forced_examples(&accepted, &request));
        extension.grounding.existing_sense = Some(8);
        assert!(
            prepare(&request, false, vec![extension.clone()])
                .0
                .is_none()
        );
        extension.grounding.existing_sense = None;
        extension.grounding.sources.push("unlisted source".into());
        assert!(prepare(&request, false, vec![extension]).0.is_none());
    }

    #[test]
    fn phonological_similarity_does_not_merge_distinct_meanings() {
        for (a, b) in [("きみ", "ぎみ"), ("かた", "がた"), ("ほう", "がた")] {
            let mut proposal = Proposal {
                reopen: false,
                senses: vec![grounded("person", &[a]), grounded("suffix", &[b])],
            };
            assert!(normalize_senses(
                &mut proposal,
                Language::Japanese,
                &mut Rejections::default()
            ));
            assert_eq!(proposal.senses.len(), 2);
        }
    }
    #[test]
    fn dictionary_equivalence_merges_variants_not_same_reading_meanings() {
        let mut request = request(&["じっぷん", "じゅっぷん", "じゅうぶん"]);
        request.gram = super::super::tests::gram(&["十分"]);
        request.candidates.senses = [
            ("じっぷん", "ten minutes"),
            ("じゅっぷん", "ten minutes"),
            ("じゅうぶん", "enough"),
        ]
        .into_iter()
        .map(|(r, m)| (r.into(), vec![m.into()]))
        .collect();
        let proposal = Proposal {
            reopen: false,
            senses: vec![
                sense("ten minutes", vec![reading(Some(0), "じっぷん", None)]),
                sense("10 minutes", vec![reading(Some(2), "じゅっぷん", None)]),
                sense("adequately", vec![reading(Some(1), "じゅうぶん", None)]),
                sense("thoroughly", vec![reading(Some(1), "じゅうぶん", None)]),
            ],
        };
        let result = prepare_proposal(
            proposal,
            &request,
            Language::Japanese,
            &kanjidic::Verifier::default(),
            &mut Rejections::default(),
        )
        .unwrap();
        assert_eq!(result.senses.len(), 3);
        assert_eq!(result.senses[0].readings.len(), 2);
    }
    #[test]
    fn dictionary_readings_need_no_corpus_citation() {
        // Every listed reading and sense survives B; C attestation prunes later.
        // Candidates are sorted: index 0 is kàn, index 1 is kān.
        let request = request(&["kān", "kàn"]);
        let (answer, checks) = prepare(
            &request,
            false,
            vec![
                sense("to look at", vec![reading(Some(0), "kàn", None)]),
                sense("to look after", vec![reading(Some(1), "kān", None)]),
            ],
        );
        let answer = answer.unwrap();
        assert_eq!(answer.senses.len(), 2);
        assert!(
            answer
                .senses
                .iter()
                .flat_map(|s| &s.readings)
                .all(|r| r.example.is_none())
        );
        assert_eq!(checks.ungrounded_new_readings, 0);
    }

    #[test]
    fn meanings_that_share_a_reading_stay_separate_senses() {
        let request = request(&["kàn"]);
        let (answer, _) = prepare(
            &request,
            false,
            vec![
                sense("to see", vec![reading(Some(0), "kàn", None)]),
                sense("to visit (a doctor)", vec![reading(Some(0), "kàn", None)]),
            ],
        );
        let answer = answer.unwrap();
        assert_eq!(answer.senses.len(), 2);
        assert_eq!(answer.senses[1].gloss, "to visit (a doctor)");
    }

    #[test]
    fn readings_outside_the_dictionary_need_the_example_that_shows_them() {
        let request = request(&["kàn"]);
        let (answer, checks) = prepare(
            &request,
            true,
            vec![sense(
                "to see",
                vec![
                    reading(Some(0), "kàn", None),
                    reading(None, "kān", Some(0)),
                    reading(None, "kǎn", None),
                ],
            )],
        );
        let readings = &answer.unwrap().senses[0].readings;
        assert_eq!(
            readings
                .iter()
                .map(|r| (r.reading.as_str(), r.example))
                .collect::<Vec<_>>(),
            [("kàn", None), ("kān", Some(0))]
        );
        assert_eq!(checks.ungrounded_new_readings, 1);
        // An example must be one of the displayed lines.
        let (answer, checks) = prepare(
            &request,
            true,
            vec![sense("to see", vec![reading(None, "kān", Some(1))])],
        );
        assert!(answer.is_none());
        assert_eq!(checks.structure_answers, 1);
    }

    #[test]
    fn closed_menus_check_index_and_text_agreement() {
        let request = request(&["kān", "kàn"]);
        for (index, text) in [(Some(1), "kàn"), (None, "kàn")] {
            let (answer, checks) = prepare(
                &request,
                false,
                vec![sense("to see", vec![reading(index, text, None)])],
            );
            assert!(answer.is_none());
            assert_eq!(checks.index_disagreements, 1);
        }
        // A reopened menu still keeps listed candidates honest.
        let (answer, checks) = prepare(
            &request,
            true,
            vec![sense("to see", vec![reading(Some(1), "kàn", Some(0))])],
        );
        assert!(answer.is_none());
        assert_eq!(checks.index_disagreements, 1);
    }

    #[test]
    fn empty_answers_and_unglossed_senses_fail_validation() {
        let open = request(&[]);
        let (empty, checks) = prepare(&open, false, vec![]);
        assert!(empty.is_none());
        assert_eq!(checks.structure_answers, 1);
        assert!(
            prepare(
                &open,
                false,
                vec![sense("", vec![reading(None, "kàn", Some(0))])]
            )
            .0
            .is_none()
        );
        // Nothing grounded is left of an open gram's only reading.
        let (answer, checks) = prepare(
            &open,
            false,
            vec![sense("to see", vec![reading(None, "kàn", None)])],
        );
        assert!(answer.is_none());
        assert_eq!(checks.empty_senses, 1);
    }

    #[test]
    fn unindexed_b_readings_keep_tones_until_meaning_assignment() {
        let mut request = request(&["nà shi"]);
        request.gram = super::super::tests::gram(&["那", "是"]);
        let (answer, _) = prepare(
            &request,
            true,
            vec![
                sense("that is", vec![reading(None, "nà shì", Some(0))]),
                sense("of course", vec![reading(Some(0), "nà shi", None)]),
            ],
        );
        let answer = answer.unwrap();
        assert_eq!(answer.senses[0].readings[0].reading, "nà shì");
        assert_eq!(answer.senses[1].readings[0].reading, "nà shi");
    }

    #[test]
    fn sound_changed_readings_join_their_plain_sense() {
        for (a, b) in [("つくり", "づくり"), ("かた", "がた"), ("いち", "いっ")] {
            let mut proposal = Proposal {
                reopen: false,
                senses: vec![
                    grounded("same meaning", &[a]),
                    grounded("same meaning", &[b]),
                ],
            };
            let mut checks = Rejections::default();
            assert!(normalize_senses(
                &mut proposal,
                Language::Japanese,
                &mut checks
            ));
            assert_eq!(proposal.senses.len(), 1);
            assert_eq!(proposal.senses[0].readings.len(), 2);
            assert_eq!(proposal.senses[0].gloss, "same meaning");
            assert_eq!(checks.sound_change_merges, 1);
        }
    }

    #[test]
    fn a_sense_that_also_lists_the_plain_form_is_a_deliberate_meaning_split() {
        let mut proposal = Proposal {
            reopen: false,
            senses: vec![
                grounded("person", &["かた"]),
                grounded("way of doing", &["かた", "がた"]),
                grounded("direction", &["ほう"]),
            ],
        };
        assert!(normalize_senses(
            &mut proposal,
            Language::Japanese,
            &mut Rejections::default()
        ));
        assert_eq!(proposal.senses.len(), 3);
        // Chinese has no rendaku: the same shapes never merge.
        let mut proposal = Proposal {
            reopen: false,
            senses: vec![grounded("a", &["kàn"]), grounded("b", &["gàn"])],
        };
        assert!(normalize_senses(
            &mut proposal,
            Language::ChineseSimplified,
            &mut Rejections::default()
        ));
        assert_eq!(proposal.senses.len(), 2);
    }

    #[test]
    fn corrections_can_empty_a_sense_or_coalesce_its_readings() {
        let mut proposal = Proposal {
            reopen: false,
            senses: vec![grounded("a", &["kàn", "kàn"]), grounded("b", &[])],
        };
        let mut checks = Rejections::default();
        assert!(normalize_senses(
            &mut proposal,
            Language::ChineseSimplified,
            &mut checks
        ));
        assert_eq!(proposal.senses.len(), 1);
        assert_eq!(proposal.senses[0].readings.len(), 1);
        assert_eq!(checks.empty_senses, 1);
        let mut proposal = Proposal {
            reopen: false,
            senses: vec![grounded("a", &[])],
        };
        assert!(!normalize_senses(
            &mut proposal,
            Language::ChineseSimplified,
            &mut checks
        ));
    }
}
