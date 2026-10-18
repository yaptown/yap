//! Hand-reviewed overrides use the inventory's full gram key and exact readings.
//! JSONL actions are drop_reading(reading), drop_sense(readings), and
//! replace_reading(from, to). File order applies within each B round.
use super::*;
use anyhow::Context;

#[derive(Debug, Deserialize, Serialize)]
struct Correction {
    gram: Gram<String>,
    #[serde(flatten)]
    action: Action,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
enum Action {
    DropReading { reading: String },
    DropSense { readings: BTreeSet<String> },
    ReplaceReading { from: String, to: String },
}

impl Action {
    fn normalize(&mut self, language: Language, surface: &str) -> Result<()> {
        let canonical = |reading: &str| {
            canonical::canonicalize(language, surface, reading)
                .with_context(|| format!("invalid correction reading {reading:?} for {surface:?}"))
        };
        match self {
            Self::DropReading { reading } => *reading = canonical(reading)?,
            Self::DropSense { readings } => {
                *readings = readings
                    .iter()
                    .map(|r| canonical(r))
                    .collect::<Result<_>>()?;
            }
            Self::ReplaceReading { from, to } => {
                *from = canonical(from)?;
                // A hand-chosen pronunciation is authoritative: only the hard
                // guards apply, never the Japanese DP heuristic or an LLM.
                *to = canonical::checked_surface_reading(language, surface, to)
                    .with_context(|| format!("manual replacement {to:?} for {surface:?} fails the hard surface guards"))?;
            }
        }
        Ok(())
    }

    fn apply(
        &self,
        proposal: &mut Proposal<GroundedReading>,
        language: Language,
        surface: &str,
        candidates: &BTreeSet<String>,
    ) -> usize {
        let canonical = |r: &str| canonical::menu_form(language, surface, r, candidates).unwrap();
        let mut matched = 0;
        match self {
            Self::DropReading { reading } => {
                for sense in &mut proposal.senses {
                    sense.readings.retain(|r| {
                        let remove = r.reading == canonical(reading);
                        matched += usize::from(remove);
                        !remove
                    });
                }
            }
            Self::DropSense { readings } => {
                proposal.senses.retain(|sense| {
                    let current: BTreeSet<_> =
                        sense.readings.iter().map(|r| r.reading.clone()).collect();
                    let remove = current == readings.iter().map(|r| canonical(r)).collect();
                    matched += usize::from(remove);
                    !remove
                });
            }
            Self::ReplaceReading { from, to } => {
                for reading in proposal.senses.iter_mut().flat_map(|s| &mut s.readings) {
                    if reading.reading == canonical(from) {
                        reading.reading.clone_from(to);
                        matched += 1;
                    }
                }
            }
        }
        matched
    }
}

struct Rule {
    line: usize,
    correction: Correction,
    matched: usize,
}

pub(super) struct Corrections {
    language: Language,
    path: String,
    rules: Vec<Rule>,
}

impl Corrections {
    pub(super) fn load(language: Language) -> Result<Self> {
        let path = format!(
            "generate-data/data/{}/reading_corrections.jsonl",
            language.corpus_code()
        );
        let contents = std::fs::read_to_string(&path).with_context(|| format!("read {path}"))?;
        Self::parse(language, path, &contents)
    }

    pub(super) fn parse(language: Language, path: String, contents: &str) -> Result<Self> {
        let mut rules = Vec::new();
        for (index, line) in contents.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let location = format!("{path}:{}", index + 1);
            let mut correction: Correction =
                serde_json::from_str(line).with_context(|| location.clone())?;
            correction
                .action
                .normalize(language, &correction.gram.to_display_string(language))
                .with_context(|| location.clone())?;
            rules.push(Rule {
                line: index + 1,
                correction,
                matched: 0,
            });
        }
        Ok(Self {
            language,
            path,
            rules,
        })
    }

    pub(super) fn apply(
        &mut self,
        accepted: &mut [(&HierarchyRequest, Proposal<GroundedReading>)],
        round: &str,
    ) -> BTreeSet<Gram<String>> {
        let mut total = 0;
        let mut dropped = BTreeSet::new();
        for (request, proposal) in accepted {
            let mut changed = false;
            for rule in &mut self.rules {
                if rule.correction.gram != request.gram {
                    continue;
                }
                let matched = rule.correction.action.apply(
                    proposal,
                    self.language,
                    &request.gram.to_display_string(self.language),
                    &request.candidates.readings,
                );
                if matched > 0 {
                    rule.matched += matched;
                    total += matched;
                    changed = true;
                    if let Action::ReplaceReading { to, .. } = &rule.correction.action
                        && !request.candidates.readings.contains(to)
                    {
                        proposal.reopen = true;
                    }
                    println!(
                        "readings[{}]: correction {}:{} matched={matched}, round={round}: {}",
                        self.language.code(),
                        self.path,
                        rule.line,
                        serde_json::to_string(&rule.correction).unwrap()
                    );
                }
            }
            if changed {
                // Replacements can coalesce leaves; drops can remove the sole
                // exclusive reading of a new sense. Keep B's normal invariants.
                let mut checks = Rejections::default();
                if !validation::normalize_senses(proposal, self.language, &mut checks) {
                    proposal.senses.clear();
                    dropped.insert(request.gram.clone());
                }
            }
        }
        println!(
            "readings[{}]: manual correction matches={total}, round={round}",
            self.language.code()
        );
        dropped
    }

    fn unmatched(&self) -> impl Iterator<Item = &Rule> {
        self.rules.iter().filter(|rule| rule.matched == 0)
    }

    pub(super) fn report_unmatched(&self) {
        for rule in self.unmatched() {
            log::warn!(
                "readings[{}]: unmatched correction {}:{} did not match any B output: {}",
                self.language.code(),
                self.path,
                rule.line,
                serde_json::to_string(&rule.correction).unwrap()
            );
        }
        println!(
            "readings[{}]: manual corrections={}, unmatched={}",
            self.language.code(),
            self.rules.len(),
            self.unmatched().count()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use language_utils::{Heteronym, PartOfSpeech, Word};

    fn g(text: &str, pos: PartOfSpeech) -> Gram<String> {
        Gram(vec![Atom::Tok(Word {
            text: text.into(),
            word_type: WordType::Heteronym(Heteronym {
                word: text.into(),
                lemma: text.into(),
                pos,
            }),
        })])
    }
    fn req(gram: Gram<String>) -> HierarchyRequest {
        HierarchyRequest {
            gram,
            candidates: Candidates {
                senses: BTreeMap::new(),
                evidence: BTreeMap::new(),
                coverings: BTreeMap::new(),
                readings: BTreeSet::new(),
                whole_word_readings: BTreeSet::new(),
                open: true,
                atoms: vec![],
                prior_single: false,
            },
            prompt: serde_json::Value::Null,
            examples: vec![],
        }
    }
    fn proposal(readings: &[&[&str]]) -> Proposal<GroundedReading> {
        Proposal {
            reopen: false,
            senses: readings
                .iter()
                .enumerate()
                .map(|(i, rs)| ProposedSense {
                    grounding: Default::default(),
                    kind: UsageKind::Meaning,
                    gloss: format!("meaning {i}"),
                    readings: rs
                        .iter()
                        .enumerate()
                        .map(|(line, r)| GroundedReading {
                            reading: (*r).into(),
                            example: Some(line),
                        })
                        .collect(),
                })
                .collect(),
        }
    }
    fn parse(
        language: Language,
        gram: &Gram<String>,
        actions: &[serde_json::Value],
    ) -> Result<Corrections> {
        let contents = actions
            .iter()
            .map(|action| {
                let mut action = action.clone();
                action["gram"] = serde_json::to_value(gram).unwrap();
                action.to_string() + "\n"
            })
            .collect::<String>();
        Corrections::parse(language, "corrections.jsonl".into(), &contents)
    }

    #[test]
    fn explicit_tone_corrections_keep_manual_targets_authoritative() {
        let gram = g("得了", PartOfSpeech::Part);
        let mut request = req(gram.clone());
        request.candidates.readings.insert("dé liǎo".into());
        let mut corrections = parse(
            Language::ChineseSimplified,
            &gram,
            &[serde_json::json!({"action":"replace_reading","from":"dé liǎo","to":"de liǎo"})],
        )
        .unwrap();
        let mut accepted = vec![(&request, proposal(&[&["dé liǎo"]]))];
        corrections.apply(&mut accepted, "initial");
        assert_eq!(accepted[0].1.senses[0].readings[0].reading, "de liǎo");
        assert_eq!(corrections.unmatched().count(), 0);
    }

    #[test]
    fn checked_in_corrections_parse_and_unmatched_rows_only_warn() {
        for (language, contents, count) in [
            (
                Language::ChineseSimplified,
                include_str!("../../data/zho-hans/reading_corrections.jsonl"),
                6,
            ),
            (
                Language::Japanese,
                include_str!("../../data/jpn/reading_corrections.jsonl"),
                4,
            ),
        ] {
            let corrections = Corrections::parse(language, "fixture".into(), contents).unwrap();
            assert_eq!(corrections.unmatched().count(), count);
            corrections.report_unmatched();
        }
    }

    #[test]
    fn checked_in_rules_match_bad_b_rows_but_not_clean_or_different_grams() {
        for (language, contents, bad, good) in [
            (
                Language::Japanese,
                include_str!("../../data/jpn/reading_corrections.jsonl"),
                vec!["しゅつ", "のしゅつだ", "おおのおとこが", "ちりじり"],
                vec![
                    Some("で"),
                    Some("のでだ"),
                    Some("だいのおとこが"),
                    Some("ちりぢり"),
                ],
            ),
            (
                Language::ChineseSimplified,
                include_str!("../../data/zho-hans/reading_corrections.jsonl"),
                vec![
                    "dàng nián",
                    "è xīn",
                    "zài nǎ",
                    "dé liǎo",
                    "kàn jǐn diǎn",
                    "hóng mó fāng",
                ],
                vec![
                    Some("dāng nián"),
                    Some("ě xīn"),
                    None,
                    Some("de liǎo"),
                    Some("kān jǐn diǎn"),
                    Some("hóng mò fáng"),
                ],
            ),
        ] {
            for (i, (bad, good)) in bad.into_iter().zip(good).enumerate() {
                let mut corrections =
                    Corrections::parse(language, "fixture".into(), contents).unwrap();
                let gram = corrections.rules[i].correction.gram.clone();
                let request = req(gram.clone());
                // Same full gram, but the current B result is already clean.
                let clean = good.unwrap_or("zài nà");
                let mut accepted = vec![(&request, proposal(&[&[clean]]))];
                corrections.apply(&mut accepted, "initial");
                assert_eq!(corrections.rules[i].matched, 0);
                assert_eq!(accepted[0].1.senses[0].readings[0].reading, clean);
                // Same surface with another POS is not permission to broaden the rule.
                let mut different = gram;
                let Atom::Tok(word) = &mut different.0[0] else {
                    panic!()
                };
                match &mut word.word_type {
                    WordType::Heteronym(word) => {
                        word.pos = if word.pos == PartOfSpeech::Adj {
                            PartOfSpeech::Verb
                        } else {
                            PartOfSpeech::Adj
                        };
                    }
                    WordType::Other(_) => {
                        word.word_type = WordType::Heteronym(Heteronym {
                            word: word.text.clone(),
                            lemma: word.text.clone(),
                            pos: PartOfSpeech::Noun,
                        });
                    }
                }
                let other = req(different);
                let mut untouched = vec![(&other, proposal(&[&[bad]]))];
                corrections.apply(&mut untouched, "initial");
                assert_eq!(corrections.rules[i].matched, 0);
                assert_eq!(untouched[0].1.senses[0].readings[0].reading, bad);
                let mut accepted = vec![(&request, proposal(&[&[bad]]))];
                corrections.apply(&mut accepted, "escape-repair");
                assert_eq!(corrections.rules[i].matched, 1);
                if let Some(good) = good {
                    assert_eq!(accepted[0].1.senses[0].readings[0].reading, good);
                } else {
                    assert!(accepted[0].1.senses.is_empty());
                    // 在那's rule removes exactly the erroneous singleton sense,
                    // not a sense whose reading set also contains the valid form.
                    let mut mixed = vec![(&request, proposal(&[&[bad, clean]]))];
                    corrections.apply(&mut mixed, "escape-repair");
                    assert_eq!(mixed[0].1.senses[0].readings.len(), 2);
                    assert_eq!(corrections.rules[i].matched, 1);
                }
            }
        }
    }

    #[test]
    fn exact_gram_and_canonical_reading_sets_control_drops() {
        let gram = g("中", PartOfSpeech::Verb);
        let request = req(gram.clone());
        let mut corrections = parse(
            Language::ChineseSimplified,
            &gram,
            &[serde_json::json!({"action":"drop_sense","readings":["zhong4","zhong1"]})],
        )
        .unwrap();
        let mut accepted = vec![(&request, proposal(&[&["zhōng", "zhòng"], &["zhòng"]]))];
        corrections.apply(&mut accepted, "initial");
        assert_eq!(accepted[0].1.senses.len(), 1);
        assert_eq!(accepted[0].1.senses[0].gloss, "meaning 1");
        assert_eq!(accepted[0].1.senses[0].readings[0].reading, "zhòng");
        assert_eq!(corrections.unmatched().count(), 0);
        let other = req(g("中", PartOfSpeech::Noun));
        let mut untouched = vec![(&other, proposal(&[&["zhōng", "zhòng"]]))];
        corrections.apply(&mut untouched, "repair");
        assert_eq!(untouched[0].1.senses[0].readings.len(), 2);
    }

    #[tokio::test]
    async fn dropping_a_sole_sense_neither_falls_back_nor_reaches_c() {
        let gram = g("干", PartOfSpeech::Noun);
        let mut request = req(gram.clone());
        request.candidates.readings.insert("qiān".into());
        let mut corrections = parse(
            Language::ChineseSimplified,
            &gram,
            &[serde_json::json!({"action":"drop_sense","readings":["qiān"]})],
        )
        .unwrap();
        let mut accepted = vec![(&request, proposal(&[&["qiān"]]))];
        let dropped = corrections.apply(&mut accepted, "initial");
        assert_eq!(dropped, BTreeSet::from([gram.clone()]));
        assert!(accepted[0].1.senses.is_empty());
        let mut data = CorpusReadings::default();
        data.candidates
            .insert(gram.clone(), request.candidates.clone());
        data.corrections_dropped = dropped;
        data.install_candidate_fallbacks(&BTreeSet::new());
        assert!(data.hierarchies.is_empty());
        let groups = Groups::from([(
            gram.clone(),
            vec![Occurrence {
                text: "干".into(),
                gram_index: 0,
                bytes: (0, "干".len()),
                spans: vec![],
            }],
        )]);
        let selected = BTreeSet::from([gram]);
        assert!(
            occurrence::select_requests(&data, &groups, &selected, &BTreeSet::new()).is_empty()
        );
        assert!(
            occurrence::annotate(
                &mut data,
                Language::ChineseSimplified,
                &groups,
                &selected,
                &BTreeSet::new(),
                &kanjidic::Verifier::default(),
                "initial"
            )
            .await
            .unwrap()
            .is_empty()
        );
        assert!(data.labels.is_empty());
    }

    #[test]
    fn drops_reapply_to_repair_and_can_remove_the_entire_menu() {
        let gram = g("干", PartOfSpeech::Noun);
        let request = req(gram.clone());
        let mut corrections = parse(
            Language::ChineseSimplified,
            &gram,
            &[serde_json::json!({"action":"drop_reading","reading":"qian1"})],
        )
        .unwrap();
        for round in ["initial", "repair"] {
            let mut accepted = vec![(&request, proposal(&[&["qiān"]]))];
            corrections.apply(&mut accepted, round);
            assert!(accepted[0].1.senses.is_empty());
        }
        assert_eq!(corrections.rules[0].matched, 2);
    }

    #[test]
    fn replacements_are_guarded_canonicalized_and_keep_citations() {
        let gram = g("入った", PartOfSpeech::Verb);
        let request = req(gram.clone());
        let mut corrections = parse(
            Language::Japanese,
            &gram,
            &[serde_json::json!({"action":"replace_reading","from":"いった","to":"ハイッタ"})],
        )
        .unwrap();
        let mut accepted = vec![(&request, proposal(&[&["いった", "はいった"]]))];
        corrections.apply(&mut accepted, "initial");
        assert!(
            accepted[0].1.reopen,
            "a manual reading outside the dictionary must still get C"
        );
        let readings = &accepted[0].1.senses[0].readings;
        assert_eq!(readings.len(), 1);
        assert_eq!(readings[0].reading, "はいった");
        assert_eq!(readings[0].example, Some(0));
        let invalid = parse(
            Language::Japanese,
            &gram,
            &[serde_json::json!({"action":"replace_reading","from":"いった","to":"はいる"})],
        )
        .err()
        .unwrap();
        assert!(format!("{invalid:#}").contains("hard surface guards"));
        assert!(
            parse(
                Language::ChineseSimplified,
                &g("银行", PartOfSpeech::Noun),
                &[serde_json::json!({"action":"replace_reading","from":"yín háng","to":"yin2"})]
            )
            .is_err()
        );
    }

    #[test]
    fn unmatched_corrections_remain_visible_until_a_later_round_matches() {
        let gram = g("着", PartOfSpeech::Noun);
        let request = req(gram.clone());
        let mut corrections = parse(
            Language::ChineseSimplified,
            &gram,
            &[serde_json::json!({"action":"drop_reading","reading":"zhuó"})],
        )
        .unwrap();
        let mut initial = vec![(&request, proposal(&[&["zhe"]]))];
        corrections.apply(&mut initial, "initial");
        assert_eq!(
            corrections.unmatched().map(|r| r.line).collect::<Vec<_>>(),
            [1]
        );
        let mut repair = vec![(&request, proposal(&[&["zhe", "zhuó"]]))];
        corrections.apply(&mut repair, "repair");
        assert_eq!(corrections.unmatched().count(), 0);
        assert!(
            Corrections::parse(Language::Japanese, "empty".into(), "")
                .unwrap()
                .rules
                .is_empty()
        );
    }
}
