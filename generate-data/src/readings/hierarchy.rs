//! The same grounded hierarchy inference serves the initial and escape-repair rounds.
use super::*;

pub(super) fn settings(language: Language) -> (&'static str, &'static str) {
    match language {
        Language::Japanese => ("gpt-6-luna", "medium"),
        Language::ChineseSimplified => ("gpt-6-luna", "medium"),
        _ => unreachable!("reading hierarchies are only generated for Japanese and Chinese"),
    }
}

pub(super) async fn infer<'a>(
    language: Language,
    requests: &'a [HierarchyRequest],
    verifier: &kanjidic::Verifier,
    population: usize,
    round: &str,
) -> Result<Vec<(&'a HierarchyRequest, Proposal<GroundedReading>)>> {
    tysm::measurement::stage(&format!("{}:readings-B:{round}", language.code()));
    let (model, effort) = settings(language);
    // Quality comparisons selected this exact configuration. Do not borrow
    // responses from an older model through a migration fallback.
    let client =
        crate::apply_cache_only(crate::base_chat_client(model).with_reasoning_effort(effort));
    let system = prompts::hierarchy_system_prompt(language);
    let mut accepted = Vec::new();
    let mut retry = Vec::new();
    let mut feedback = BTreeMap::new();
    let mut first_checks = Rejections::default();
    for (request, answer) in
        batch::<_, Proposal>(&client, &system, requests, |r| r.prompt.to_string()).await?
    {
        first_checks.schema_answers += usize::from(answer.is_err());
        first_checks.details.clear();
        let mut previous = match &answer {
            Ok(answer) => serde_json::json!({"rejected_answer": answer}),
            Err(error) => serde_json::json!({"schema_error": error.to_string()}),
        };
        if let Some(answer) = answer
            .ok()
            .and_then(|a| audited_proposal(a, request, language, verifier, &mut first_checks))
        {
            accepted.push((request, answer));
        } else {
            retry.push(request);
        }
        previous["checks"] = serde_json::json!(first_checks.details);
        feedback.insert(&request.gram, previous);
    }
    let mut verification = review::Stats::default();
    let verification_rejections = review::verify(
        language,
        &mut accepted,
        verifier,
        &mut first_checks,
        &mut verification,
    )
    .await?;
    for request in &verification_rejections {
        feedback.get_mut(&request.gram).unwrap()["checks"] = serde_json::json!([
            "Pronunciation verification removed the readings needed to cover this hierarchy or a forced example. Please check the complete marked surface and its supplied atoms."
        ]);
    }
    retry.extend(verification_rejections);
    println!(
        "readings[{}]: B first-pass checks ({round}, {model}/{effort}) {}",
        language.code(),
        serde_json::to_string(&first_checks)?
    );
    let retried = retry.len();
    let mut failures = 0;
    let mut retry_checks = Rejections::default();
    if !retry.is_empty() {
        let mut retry_accepted = Vec::new();
        for (request, answer) in batch::<_, Proposal>(&client, &system, &retry, |r| {
            prompts::hierarchy_retry(r, &feedback[&r.gram])
        })
        .await?
        {
            retry_checks.schema_answers += usize::from(answer.is_err());
            if let Some(answer) = answer
                .ok()
                .and_then(|a| audited_proposal(a, request, language, verifier, &mut retry_checks))
            {
                retry_accepted.push((*request, answer));
            } else {
                failures += 1;
                log::warn!(
                    "readings[{}]: hierarchy rejected after retry; no readings for {:?}",
                    language.code(),
                    request.gram.to_display_string(language)
                );
            }
        }
        let rejected = review::verify(
            language,
            &mut retry_accepted,
            verifier,
            &mut retry_checks,
            &mut verification,
        )
        .await?;
        failures += rejected.len();
        for request in rejected {
            log::warn!(
                "readings[{}]: hierarchy rejected after retry; no readings for {:?}",
                language.code(),
                request.gram.to_display_string(language)
            );
        }
        accepted.extend(retry_accepted);
    }
    println!(
        "readings[{}]: verification totals ({round}) {}",
        language.code(),
        serde_json::to_string(&verification)?
    );
    println!(
        "readings[{}]: B retry checks ({round}) {}",
        language.code(),
        serde_json::to_string(&retry_checks)?
    );
    println!(
        "readings[{}]: step B population={population}, sent={}, retried={retried}, failed={failures}, round={round}",
        language.code(),
        requests.len()
    );
    ensure!(
        !hierarchy_failure_limit_exceeded(failures, population),
        "reading hierarchy failures exceed 1%: {failures}/{population}"
    );
    // Dictionary normalization precedes authoritative manual corrections. Doing
    // this at installation would silently undo a corrected neutral tone.
    for (request, proposal) in &mut accepted {
        if language == Language::ChineseSimplified {
            fold_headword_tones(
                proposal,
                &request.candidates,
                &request.gram.to_display_string(language),
            );
        }
    }
    Ok(accepted)
}

fn audited_proposal(
    proposal: Proposal,
    request: &HierarchyRequest,
    language: Language,
    verifier: &kanjidic::Verifier,
    checks: &mut Rejections,
) -> Option<Proposal<GroundedReading>> {
    let rejected = serde_json::to_value(&proposal).unwrap();
    let before = serde_json::to_value(&*checks).unwrap();
    let result = prepare_proposal(proposal, request, language, verifier, checks);
    if result.is_none() {
        let after = serde_json::to_value(&*checks).unwrap();
        let reasons: BTreeMap<_, _> = after
            .as_object()
            .unwrap()
            .iter()
            .filter_map(|(key, value)| {
                let count = value.as_u64().unwrap() - before[key].as_u64().unwrap();
                (count > 0).then_some((key, count))
            })
            .collect();
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(format!(
                ".cache/readings/{}-rejected-hierarchies.jsonl",
                language.code()
            ))
            .unwrap();
        writeln!(
            file,
            "{}",
            serde_json::json!({"gram":request.gram,"proposal":rejected,"reasons":reasons,"checks":checks.details,"example_count":request.examples.len()})
        )
        .unwrap();
    }
    result
}

type PrimaryTiling = Vec<(usize, usize, String)>;

fn grounded_tiling(
    parts: &coverings::Covering,
    sources: &[String],
    text: &str,
    reading: &str,
) -> Option<PrimaryTiling> {
    let syllables: Vec<_> = reading.split_whitespace().collect();
    parts
        .iter()
        .map(|part| {
            let han_count = |s: &str| {
                s.chars()
                    .filter(|&c| is_ruby_base(c, Language::ChineseSimplified))
                    .count()
            };
            let start = han_count(&text[..part.start]);
            let end = start + han_count(&text[part.start..part.end]);
            let spoken = syllables.get(start..end)?.join(" ");
            let primary: BTreeSet<_> = part
                .entry
                .evidence
                .iter()
                .filter(|e| sources.contains(&e.source))
                .filter_map(|e| {
                    canonical::checked_surface_reading(
                        Language::ChineseSimplified,
                        &part.text,
                        &e.primary,
                    )
                })
                .filter(|p| canonical::tone_variant(&spoken, p))
                .collect();
            (primary.len() == 1)
                .then(|| (part.start, part.end, primary.into_iter().next().unwrap()))
        })
        .collect()
}

/// B supplies semantic source references. A selected whole headword remains
/// authoritative; otherwise only an unambiguous, fully grounded tiling can fold.
fn fold_headword_tones(
    proposal: &mut Proposal<GroundedReading>,
    candidates: &Candidates,
    text: &str,
) {
    for sense in &mut proposal.senses {
        for reading in &mut sense.readings {
            let headwords: BTreeSet<_> = candidates
                .dictionary_evidence()
                .filter(|e| e.spelling == text && sense.grounding.sources.contains(&e.source))
                .filter_map(|e| {
                    canonical::checked_surface_reading(
                        Language::ChineseSimplified,
                        text,
                        &e.primary,
                    )
                })
                .filter(|p| canonical::tone_variant(&reading.reading, p))
                .collect();
            if !headwords.is_empty() {
                if headwords.len() == 1 {
                    reading.reading.clone_from(headwords.first().unwrap());
                }
                continue;
            }
            let tilings: BTreeSet<_> = candidates
                .coverings
                .values()
                .flatten()
                .filter_map(|parts| {
                    grounded_tiling(parts, &sense.grounding.sources, text, &reading.reading)
                })
                .collect();
            if tilings.len() == 1 {
                reading.reading = tilings
                    .first()
                    .unwrap()
                    .iter()
                    .map(|(_, _, p)| p.as_str())
                    .collect::<Vec<_>>()
                    .join(" ");
            }
        }
    }
    validation::normalize_senses(
        proposal,
        Language::ChineseSimplified,
        &mut Rejections::default(),
    );
}

pub(super) fn install(
    data: &mut CorpusReadings,
    accepted: Vec<(&HierarchyRequest, Proposal<GroundedReading>)>,
    language: Language,
    round: &str,
    groups: &Groups,
) -> Result<()> {
    let mut evidence =
        std::io::BufWriter::new(std::fs::OpenOptions::new().create(true).append(true).open(
            format!(".cache/readings/{}-hierarchies.jsonl", language.code()),
        )?);
    let mut proposed_senses = 0;
    for (request, proposal) in accepted {
        writeln!(
            evidence,
            "{}",
            serde_json::json!({"gram":request.gram,"examples":request.examples,"proposal":proposal,"round":round,"model":settings(language).0,"effort":settings(language).1})
        )?;
        proposed_senses +=
            install_proposal(data, request, proposal, round == "escape-repair", groups);
    }
    evidence.flush()?;
    println!(
        "readings[{}]: proposed explicit senses={proposed_senses}, round={round}; inventories wait for C attestation",
        language.code()
    );
    Ok(())
}

/// Repair extends explicit references without redefining protected meanings.
fn install_proposal(
    data: &mut CorpusReadings,
    request: &HierarchyRequest,
    proposal: Proposal<GroundedReading>,
    repair: bool,
    groups: &Groups,
) -> usize {
    let mut added = 0;
    let mut senses = if repair {
        data.hierarchies.remove(&request.gram).unwrap_or_default()
    } else {
        vec![]
    };
    let implicit = senses.is_empty() && proposal.senses.len() == 1;
    let mut next = senses.iter().map(|s| s.index).max().unwrap_or(0).max(1) + 1;
    for (position, sense) in proposal.senses.into_iter().enumerate() {
        let mut readings: Vec<_> = sense.readings.into_iter().map(|r| r.reading).collect();
        readings.sort_by_key(|r| {
            prompts::evidence_priority(
                request
                    .candidates
                    .evidence
                    .get(r)
                    .map_or(&[], Vec::as_slice),
            )
        });
        if let Some(id) = sense.grounding.existing_sense {
            let existing = senses
                .iter_mut()
                .find(|s| s.index == id)
                .expect("validated repair reference");
            if existing.gloss.is_empty() {
                existing.gloss = sense.gloss;
            }
            for reading in readings {
                if !existing.readings.contains(&reading) {
                    existing.readings.push(reading);
                }
            }
        } else {
            let index = if implicit {
                0
            } else if repair {
                let id = next;
                next += 1;
                id
            } else {
                position as u32 + 1
            };
            added += usize::from(index != 0);
            senses.push(SenseReadings {
                index,
                kind: sense.kind.as_str().into(),
                synthetic_readings: BTreeSet::new(),
                gloss: sense.gloss,
                readings,
            });
        }
    }
    if senses.len() > 1
        && let Some(implicit) = senses.iter_mut().find(|s| s.index == 0)
    {
        implicit.index = 1;
        for occurrence in &groups[&request.gram] {
            if let Some(label) = data
                .labels
                .get_mut(&(occurrence.text.clone(), occurrence.gram_index))
            {
                let choices = match label {
                    occurrence::Label::Ranked { choices }
                    | occurrence::Label::Restated { choices, .. } => choices,
                    occurrence::Label::Escape { .. } => continue,
                };
                for choice in choices {
                    if choice.sense == 0 {
                        choice.sense = 1;
                    }
                }
            }
        }
    }
    if !senses.is_empty() {
        data.hierarchies.insert(request.gram.clone(), senses);
    }
    added
}

/// Expand only after B, preserving its meaning assignment. Multi-atom readings
/// expand only after an exact, unique alignment proves their atom boundaries.
pub(super) fn expand_variants(data: &mut CorpusReadings, verifier: &kanjidic::Verifier) {
    for (gram, senses) in &mut data.hierarchies {
        let words: Vec<_> = gram
            .iter()
            .filter_map(|a| match a {
                Atom::Tok(w) => Some(w),
                _ => None,
            })
            .collect();
        if words.is_empty() {
            continue;
        }
        let text = gram.to_display_string(Language::Japanese);
        let options: Vec<Vec<String>> = data
            .candidates
            .get(gram)
            .map(|c| {
                c.atoms
                    .iter()
                    .map(|a| {
                        if a.pos == "Punct" {
                            vec![String::new()]
                        } else {
                            a.readings.iter().map(|r| r.reading.clone()).collect()
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();
        let original = senses.clone();
        for sense in senses {
            let mut generated = BTreeSet::new();
            for reading in &sense.readings {
                let aligned = if words.len() == 1 {
                    Some(vec![reading.clone()])
                } else if options.len() == words.len() {
                    phrases::aligned_atom_readings(reading, &options)
                } else {
                    None
                };
                let Some(aligned) = aligned else { continue };
                let mut product = BTreeSet::from([String::new()]);
                for (i, (word, part)) in words.iter().zip(aligned).enumerate() {
                    // Internal neighbours are known. Only the outer boundaries
                    // retain the conditional, occurrence-dependent offerings.
                    let onset = i == 0
                        || words[i - 1]
                            .text
                            .chars()
                            .last()
                            .is_some_and(|c| is_ruby_base(c, Language::Japanese));
                    let ending = i + 1 == words.len()
                        || words[i + 1]
                            .text
                            .chars()
                            .next()
                            .is_some_and(|c| is_ruby_base(c, Language::Japanese));
                    let variants = kanjidic::atom_variants(&word.text, &part, onset, ending);
                    product = product
                        .iter()
                        .flat_map(|prefix| {
                            variants
                                .iter()
                                .map(move |suffix| format!("{prefix}{suffix}"))
                        })
                        .collect();
                }
                generated.extend(product);
            }
            for variant in generated {
                if !sense.readings.contains(&variant)
                    && !original.iter().any(|other| {
                        other.index != sense.index && other.readings.contains(&variant)
                    })
                    && canonical::checked_surface_reading(Language::Japanese, &text, &variant)
                        .is_some()
                    && kanjidic::matches(&text, &variant, verifier)
                {
                    sense.synthetic_readings.insert(variant.clone());
                    sense.readings.push(variant);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sense(gloss: &str, readings: &[&str]) -> ProposedSense<GroundedReading> {
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
    fn entry(text: &str, reading: &str, source: &str, meaning: &str) -> DictionaryReading {
        DictionaryReading {
            reading: reading.into(),
            senses: vec![meaning.into()],
            evidence: vec![DictionaryEvidence {
                source: source.into(),
                spelling: text.into(),
                meaning: meaning.into(),
                primary: reading.into(),
                form: "primary".into(),
                notes: vec![],
                priority: vec![],
            }],
        }
    }

    #[test]
    fn mandarin_compositions_keep_structural_and_refusal_candidates() {
        let dictionary = Dictionary::from([
            (
                "得了".into(),
                vec![entry(
                    "得了",
                    "dé liǎo",
                    "rhetorical",
                    "possible in rhetorical questions",
                )],
            ),
            (
                "得".into(),
                vec![entry("得", "de", "structural", "structural particle")],
            ),
            (
                "了".into(),
                vec![
                    entry("了", "liǎo", "completion", "to finish"),
                    entry("了", "le", "final", "change of state"),
                ],
            ),
            ("不".into(), vec![entry("不", "bù", "negative", "not")]),
            (
                "不了".into(),
                vec![
                    entry("不了", "bù le", "refusal", "no thanks"),
                    entry("不了", "bù liǎo", "inability", "unable to"),
                ],
            ),
            ("我".into(), vec![entry("我", "wǒ", "self", "I; me")]),
        ]);
        for (parts, expected, source) in [
            (vec!["得了"], "de liǎo", "structural"),
            (vec!["不", "了", "，", "我"], "bù le wǒ", "refusal"),
            (vec!["「不了！」"], "bù le", "refusal"),
        ] {
            let mut gram = super::super::tests::gram(&parts);
            for atom in &mut gram.0 {
                if let Atom::Tok(word) = atom
                    && word.text == "，"
                {
                    word.word_type = WordType::Other(language_utils::OtherWord {
                        other_tag: language_utils::OtherWordType::Punct,
                    });
                }
            }
            let c = candidates(&gram, Language::ChineseSimplified, &dictionary);
            assert!(c.readings.contains(expected), "{parts:?}: {:?}", c.readings);
            if parts == ["「不了！」"] {
                let headword = c.coverings[expected]
                    .iter()
                    .find(|parts| parts.len() == 1)
                    .unwrap();
                assert_eq!(headword[0].text, "不了");
                assert_eq!(headword[0].start, "「".len());
            }
            let prompt = prompts::hierarchy_prompt(&gram, &c, Language::ChineseSimplified);
            assert!(prompt.to_string().contains(expected));
            assert!(c.dictionary_evidence().any(|e| e.source == source));
            let index = c
                .ordered_readings()
                .iter()
                .position(|r| r.as_str() == expected)
                .unwrap();
            let request = HierarchyRequest {
                gram,
                candidates: c,
                prompt,
                examples: vec![],
            };
            let mut proposal = prepare_proposal(
                Proposal {
                    reopen: false,
                    senses: vec![ProposedSense {
                        grounding: SenseGrounding {
                            sources: vec![source.into()],
                            ..Default::default()
                        },
                        kind: UsageKind::Construction,
                        gloss: "construction".into(),
                        readings: vec![ProposedReading {
                            index: Some(index),
                            reading: expected.into(),
                            example_line: None,
                        }],
                    }],
                },
                &request,
                Language::ChineseSimplified,
                &kanjidic::Verifier::default(),
                &mut Rejections::default(),
            )
            .unwrap();
            fold_headword_tones(
                &mut proposal,
                &request.candidates,
                &request.gram.to_display_string(Language::ChineseSimplified),
            );
            assert_eq!(proposal.senses[0].readings[0].reading, expected);
        }
    }

    #[test]
    fn manual_structural_tones_survive_installation_and_c_readout() {
        let language = Language::ChineseSimplified;
        let contents = include_str!("../../data/zho-hans/reading_corrections.jsonl");
        let rule: serde_json::Value =
            serde_json::from_str(contents.lines().nth(3).unwrap()).unwrap();
        let gram: Gram<String> = serde_json::from_value(rule["gram"].clone()).unwrap();
        let dictionary = Dictionary::from([(
            "得了".into(),
            vec![entry(
                "得了",
                "dé liǎo",
                "cedict:possible",
                "(emphatically, in rhetorical questions) possible",
            )],
        )]);
        let request = HierarchyRequest {
            candidates: candidates(&gram, language, &dictionary),
            gram: gram.clone(),
            prompt: serde_json::Value::Null,
            examples: vec![],
        };
        for repair in [false, true] {
            let mut proposal = Proposal {
                reopen: true,
                senses: vec![sense("能够完成", &["de liǎo"])],
            };
            proposal.senses[0]
                .grounding
                .sources
                .push("cedict:possible".into());
            fold_headword_tones(&mut proposal, &request.candidates, "得了");
            assert_eq!(proposal.senses[0].readings[0].reading, "dé liǎo");
            let mut accepted = vec![(&request, proposal)];
            let mut corrections =
                corrections::Corrections::parse(language, "fixture".into(), contents).unwrap();
            corrections.apply(
                &mut accepted,
                if repair { "escape-repair" } else { "initial" },
            );
            let mut data = CorpusReadings::default();
            install_proposal(
                &mut data,
                &request,
                accepted.pop().unwrap().1,
                repair,
                &Groups::new(),
            );
            let menu = &data.hierarchies[&gram];
            assert_eq!(menu[0].readings, ["de liǎo"]);
            assert_eq!(
                canonical::menu_form(language, "得了", "de liǎo", &menu[0].readings).as_deref(),
                Some("de liǎo")
            );
            let label = occurrence::resolve(
                OccurrenceAnswer {
                    answer: OccurrenceDecision::Listed {
                        choices: vec![IndexedChoice {
                            sense: 0,
                            reading: 0,
                        }],
                    },
                },
                menu,
            )
            .unwrap();
            assert_eq!(label.reading(), Some("de liǎo"));
        }
    }

    #[test]
    fn primary_tones_follow_explicit_meaning_sources_in_both_directions() {
        for (text, variant, primary) in [
            ("能耐", "néng nai", "néng nài"),
            ("谈谈", "tán tan", "tán tán"),
            ("了不起", "liǎo bù qǐ", "liǎo bu qǐ"),
            ("街上", "jiē shàng", "jiē shang"),
            ("看一看", "kàn yi kàn", "kàn yī kàn"),
        ] {
            let gram = super::super::tests::gram(&[text]);
            let mut candidates = candidates(&gram, Language::ChineseSimplified, &Dictionary::new());
            candidates.evidence.insert(
                variant.into(),
                vec![DictionaryEvidence {
                    source: "meaning:1".into(),
                    spelling: text.into(),
                    meaning: "selected meaning".into(),
                    primary: primary.into(),
                    form: "also".into(),
                    notes: vec![],
                    priority: vec![],
                }],
            );
            let mut proposal = Proposal {
                reopen: true,
                senses: vec![
                    sense("selected", &[variant]),
                    sense("unrelated", &[variant]),
                ],
            };
            proposal.senses[0]
                .grounding
                .sources
                .push("meaning:1".into());
            fold_headword_tones(&mut proposal, &candidates, text);
            assert_eq!(proposal.senses[0].readings[0].reading, primary);
            assert_eq!(proposal.senses[1].readings[0].reading, variant);
            // Citing a compositional fallback alongside the selected lexical
            // meaning must not suppress its exact primary (能耐, 了不起).
            let mut offset = 0;
            let parts = text
                .chars()
                .zip(variant.split_whitespace())
                .enumerate()
                .map(|(i, (character, reading))| {
                    let source = format!("part:{i}");
                    proposal.senses[0].grounding.sources.push(source.clone());
                    let start = offset;
                    offset += character.len_utf8();
                    coverings::Part {
                        start,
                        end: offset,
                        text: character.to_string(),
                        entry: entry(&character.to_string(), reading, &source, "part"),
                    }
                })
                .collect();
            candidates.coverings.insert(variant.into(), vec![parts]);
            proposal.senses[0].readings[0].reading = variant.into();
            fold_headword_tones(&mut proposal, &candidates, text);
            assert_eq!(proposal.senses[0].readings[0].reading, primary);
        }
        assert!(!canonical::tone_variant("rèn", "rén"));
        for (text, a, b) in [
            ("东西", "dōng xi", "dōng xī"),
            ("回回", "huí hui", "huí huí"),
            ("说法", "shuō fa", "shuō fǎ"),
        ] {
            let gram = super::super::tests::gram(&[text]);
            let mut candidates = candidates(&gram, Language::ChineseSimplified, &Dictionary::new());
            for (source, primary) in [("a", a), ("b", b)] {
                candidates
                    .evidence
                    .entry(primary.into())
                    .or_default()
                    .push(DictionaryEvidence {
                        source: source.into(),
                        spelling: text.into(),
                        meaning: source.into(),
                        primary: primary.into(),
                        form: "primary".into(),
                        notes: vec![],
                        priority: vec![],
                    });
            }
            let mut proposal = Proposal {
                reopen: false,
                senses: vec![sense("a", &[b]), sense("b", &[a])],
            };
            proposal.senses[0].grounding.sources.push("a".into());
            proposal.senses[1].grounding.sources.push("b".into());
            fold_headword_tones(&mut proposal, &candidates, text);
            assert_eq!(proposal.senses[0].readings[0].reading, a);
            assert_eq!(proposal.senses[1].readings[0].reading, b);
        }
    }
    #[test]
    fn internal_headword_sources_are_citable_and_only_unique_grounded_tilings_fold() {
        let dictionary = Dictionary::from([
            (
                "一个人".into(),
                vec![entry(
                    "一个人",
                    "yī gè rén",
                    "cedict:410:0",
                    "by oneself; alone",
                )],
            ),
            ("一个".into(), vec![entry("一个", "yī ge", "one", "one; a")]),
            ("一".into(), vec![entry("一", "yī", "one-character", "one")]),
            (
                "个".into(),
                vec![entry("个", "gè", "classifier", "classifier")],
            ),
            ("人".into(), vec![entry("人", "rén", "person", "person")]),
            ("待".into(), vec![entry("待", "dāi", "stay", "to stay")]),
            (
                "着".into(),
                vec![entry("着", "zhe", "aspect", "ongoing aspect")],
            ),
        ]);
        let gram = super::super::tests::gram(&["一", "个", "人", "待", "着"]);
        let candidates = candidates(&gram, Language::ChineseSimplified, &dictionary);
        assert!(candidates.evidence.is_empty());
        assert!(
            candidates
                .dictionary_evidence()
                .any(|e| e.source == "cedict:410:0")
        );
        let prompt = prompts::hierarchy_prompt(&gram, &candidates, Language::ChineseSimplified);
        assert!(prompt.to_string().contains("cedict:410:0"));
        assert!(!prompt.to_string().contains("cover:"));
        let table = prompt["candidates"]["covering_parts"].as_array().unwrap();
        let mut occurrences = 0;
        for (reading, paths) in &candidates.coverings {
            let shown = prompt["candidates"]["readings"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["reading"] == *reading)
                .unwrap();
            let shown_paths = shown["coverings"].as_array().unwrap();
            assert_eq!(shown_paths.len(), paths.len());
            for (parts, indices) in paths.iter().zip(shown_paths) {
                let indices = indices.as_array().unwrap();
                assert_eq!(indices.len(), parts.len());
                occurrences += parts.len();
                for (part, index) in parts.iter().zip(indices) {
                    let mut shown = table[index.as_u64().unwrap() as usize].clone();
                    shown.as_object_mut().unwrap().remove("part_index");
                    assert_eq!(shown, serde_json::to_value(part).unwrap());
                }
            }
        }
        assert!(
            table.len() < occurrences,
            "shared parts are serialized once, without dropping paths"
        );
        let reading = "yī gè rén dāi zhe";
        let index = candidates
            .readings
            .iter()
            .position(|r| r == reading)
            .unwrap();
        let request = HierarchyRequest {
            gram,
            candidates,
            prompt,
            examples: vec![],
        };
        let proposal = Proposal {
            reopen: false,
            senses: vec![ProposedSense {
                grounding: SenseGrounding {
                    sources: vec!["cedict:410:0".into()],
                    ..Default::default()
                },
                kind: UsageKind::Meaning,
                gloss: "独自待着".into(),
                readings: vec![ProposedReading {
                    index: Some(index),
                    reading: reading.into(),
                    example_line: None,
                }],
            }],
        };
        let mut proposal = prepare_proposal(
            proposal,
            &request,
            Language::ChineseSimplified,
            &kanjidic::Verifier::default(),
            &mut Rejections::default(),
        )
        .unwrap();
        proposal.senses[0].readings[0].reading = "yī ge rén dāi zhe".into();
        fold_headword_tones(&mut proposal, &request.candidates, "一个人待着");
        assert_eq!(
            proposal.senses[0].readings[0].reading, "yī ge rén dāi zhe",
            "uncited suffix prevents a complete grounded tiling"
        );
        proposal.senses[0]
            .grounding
            .sources
            .extend(["stay".into(), "aspect".into()]);
        fold_headword_tones(&mut proposal, &request.candidates, "一个人待着");
        assert_eq!(proposal.senses[0].readings[0].reading, reading);
        proposal.senses[0]
            .grounding
            .sources
            .extend(["one".into(), "person".into()]);
        proposal.senses[0].readings[0].reading = "yī ge ren dāi zhe".into();
        fold_headword_tones(&mut proposal, &request.candidates, "一个人待着");
        assert_eq!(
            proposal.senses[0].readings[0].reading, "yī ge ren dāi zhe",
            "different grounded tilings cannot choose a meaning for B"
        );
    }

    #[test]
    fn variants_stay_in_their_sense_and_respect_written_edges() {
        let verifier = kanjidic::Verifier::new(
            BTreeMap::from([
                ('本', vec!["ほん".into()]),
                ('六', vec!["ろく".into()]),
                ('方', vec!["かた".into()]),
            ]),
            &BTreeMap::new(),
            BTreeMap::new(),
        );
        let mut data = CorpusReadings::default();
        for (texts, reading) in [
            (vec!["本"], "ほん"),
            (vec!["六"], "ろく"),
            (vec!["ほん"], "ほん"),
            (vec!["六つ"], "ろくつ"),
            (vec!["本", "方"], "ほんかた"),
        ] {
            data.hierarchies.insert(
                super::super::tests::gram(&texts),
                implicit_hierarchy(vec![reading.into()]),
            );
        }
        expand_variants(&mut data, &verifier);
        let readings = |texts: &[&str]| {
            data.hierarchies[&super::super::tests::gram(texts)][0]
                .readings
                .clone()
        };
        assert_eq!(readings(&["本"]), ["ほん", "ぼん", "ぽん"]);
        assert!(readings(&["六"]).contains(&"ろっ".into()));
        assert_eq!(readings(&["ほん"]), ["ほん"]);
        assert_eq!(readings(&["六つ"]), ["ろくつ"]);
        assert_eq!(readings(&["本", "方"]), ["ほんかた"]);
        let gram = super::super::tests::gram(&["本"]);
        data.hierarchies.insert(
            gram.clone(),
            vec![
                SenseReadings {
                    index: 1,
                    kind: "meaning".into(),
                    synthetic_readings: BTreeSet::new(),
                    gloss: "a".into(),
                    readings: vec!["ほん".into()],
                },
                SenseReadings {
                    index: 2,
                    kind: "meaning".into(),
                    synthetic_readings: BTreeSet::new(),
                    gloss: "b".into(),
                    readings: vec!["ぼん".into()],
                },
            ],
        );
        expand_variants(&mut data, &verifier);
        assert!(!data.hierarchies[&gram][0].readings.contains(&"ぼん".into()));
    }

    #[test]
    fn variants_follow_proven_atoms_and_internal_suffix_boundaries() {
        let dictionary = Dictionary::from([
            ("事".into(), vec!["こと".into()]),
            ("面倒".into(), vec!["めんどう".into()]),
            ("続き".into(), vec!["つづき".into()]),
            ("三".into(), vec!["さん".into()]),
            ("六".into(), vec!["ろく".into()]),
            ("本".into(), vec!["ほん".into()]),
            ("一".into(), vec!["いち".into()]),
            ("匹".into(), vec!["ひき".into()]),
        ]);
        let verifier = kanjidic::Verifier::new(
            BTreeMap::from([
                ('事', vec!["こと".into()]),
                ('節', vec!["せつ".into()]),
                ('続', vec!["つづ.く".into()]),
                ('三', vec!["さん".into()]),
                ('六', vec!["ろく".into()]),
                ('本', vec!["ほん".into()]),
                ('一', vec!["いち".into()]),
                ('匹', vec!["ひき".into()]),
            ]),
            &dictionary,
            BTreeMap::new(),
        );
        let mut data = CorpusReadings::default();
        for (words, reading) in [
            (vec!["事", "を"], "ことを"),
            (vec!["この", "事", "を"], "このことを"),
            (vec!["面倒", "事", "を"], "めんどうことを"),
            (vec!["六", "を"], "ろくを"),
            (vec!["節ちゃん"], "せつちゃん"),
            (vec!["続き"], "つづき"),
            (vec!["三", "本"], "さんほん"),
            (vec!["六", "本"], "ろくほん"),
            (vec!["一", "匹"], "いちひき"),
            (vec!["未知", "事"], "みちこと"),
        ] {
            let gram = super::super::tests::gram(&words);
            data.candidates.insert(
                gram.clone(),
                candidates(&gram, Language::Japanese, &dictionary),
            );
            data.hierarchies
                .insert(gram, implicit_hierarchy(vec![reading.into()]));
        }
        expand_variants(&mut data, &verifier);
        for (words, expected) in [
            (vec!["事", "を"], "ごとを"),
            (vec!["面倒", "事", "を"], "めんどうごとを"),
            (vec!["節ちゃん"], "せっちゃん"),
            (vec!["三", "本"], "さんぼん"),
            (vec!["六", "本"], "ろっぽん"),
            (vec!["一", "匹"], "いっぴき"),
        ] {
            let sense = &data.hierarchies[&super::super::tests::gram(&words)][0];
            assert!(sense.readings.iter().any(|r| r == expected), "{expected}");
            assert!(sense.synthetic_readings.contains(expected));
            assert!(sense.attachment_only(expected));
        }
        assert!(!canonical::sound_changed(
            "めんどうごとを",
            "めんどうことを"
        ));
        assert!(
            !data.hierarchies[&super::super::tests::gram(&["この", "事", "を"])][0]
                .readings
                .contains(&"このごとを".into())
        );
        assert!(
            !data.hierarchies[&super::super::tests::gram(&["六", "を"])][0]
                .readings
                .contains(&"ろっを".into())
        );
        assert!(
            !data.hierarchies[&super::super::tests::gram(&["続き"])][0]
                .readings
                .contains(&"づづき".into())
        );
        assert_eq!(
            data.hierarchies[&super::super::tests::gram(&["未知", "事"])][0].readings,
            ["みちこと"]
        );
        assert!(
            phrases::aligned_atom_readings(
                "あああ",
                &[
                    vec!["あ".into(), "ああ".into()],
                    vec!["あ".into(), "ああ".into()]
                ]
            )
            .is_none()
        );
    }

    #[test]
    fn additive_repair_fills_a_missing_gloss_without_rewriting_a_meaning() {
        let gram = super::super::tests::gram(&["等下"]);
        let mut data = CorpusReadings::default();
        data.hierarchies
            .insert(gram.clone(), implicit_hierarchy(vec!["děng xià".into()]));
        let request = HierarchyRequest {
            gram: gram.clone(),
            candidates: candidates(&gram, Language::ChineseSimplified, &Dictionary::new()),
            prompt: serde_json::Value::Null,
            examples: vec![],
        };
        for (gloss, expected) in [
            ("稍等；等一会儿", "稍等；等一会儿"),
            ("a different definition", "稍等；等一会儿"),
        ] {
            let mut extension = sense(gloss, &["děng xià"]);
            extension.grounding.existing_sense = Some(0);
            install_proposal(
                &mut data,
                &request,
                Proposal {
                    reopen: false,
                    senses: vec![extension],
                },
                true,
                &Groups::new(),
            );
            assert_eq!(data.hierarchies[&gram][0].gloss, expected);
            assert_eq!(data.hierarchies[&gram][0].index, 0);
            assert_eq!(data.hierarchies[&gram][0].readings, ["děng xià"]);
        }
    }

    #[test]
    fn additive_install_protects_labels_and_promotes_implicit_references() {
        let gram = super::super::tests::gram(&["不用"]);
        let occurrence = Occurrence {
            text: "不用".into(),
            gram_index: 0,
            bytes: (0, 6),
            spans: vec![],
        };
        let groups = BTreeMap::from([(gram.clone(), vec![occurrence.clone()])]);
        let mut data = CorpusReadings::default();
        let mut existing = implicit_hierarchy(vec!["bù yòng".into()]);
        existing[0].gloss = "no need".into();
        data.hierarchies.insert(gram.clone(), existing);
        data.labels.insert(
            (occurrence.text.clone(), 0),
            occurrence::Label::Ranked {
                choices: vec![Choice {
                    sense: 0,
                    reading: "bù yòng".into(),
                }],
            },
        );
        let request = HierarchyRequest {
            gram: gram.clone(),
            candidates: candidates(&gram, Language::ChineseSimplified, &Dictionary::new()),
            examples: vec![occurrence.clone()],
            prompt: serde_json::Value::Null,
        };
        let mut extension = sense("a rephrasing must not replace the old gloss", &["bù yòng"]);
        extension.grounding.existing_sense = Some(0);
        let proposal = Proposal {
            reopen: false,
            senses: vec![extension, sense("not use", &["bù yòng"])],
        };
        install_proposal(&mut data, &request, proposal, true, &groups);
        assert_eq!(data.hierarchies[&gram][0].gloss, "no need");
        assert_eq!(data.hierarchies[&gram][0].index, 1);
        assert_eq!(data.hierarchies[&gram][1].index, 2);
        assert_eq!(
            data.labels[&(occurrence.text.clone(), 0)]
                .first()
                .unwrap()
                .sense,
            1
        );
        let preserved = serde_json::to_value(&data.hierarchies[&gram]).unwrap();
        install_proposal(
            &mut data,
            &request,
            Proposal {
                reopen: false,
                senses: vec![],
            },
            true,
            &groups,
        );
        assert_eq!(
            serde_json::to_value(&data.hierarchies[&gram]).unwrap(),
            preserved
        );
    }

    #[test]
    fn sound_change_menus_explain_attachment_and_keep_counter_forms() {
        for (surface, plain, changed) in [
            ("人", "ひと", "ぴと"),
            ("時", "とき", "どき"),
            ("本", "ほん", "ぽん"),
            ("本", "ほん", "ぼん"),
            ("匹", "ひき", "ぴき"),
            ("六", "ろく", "ろっ"),
            ("一", "いち", "いっ"),
        ] {
            let menu = implicit_hierarchy(vec![plain.into(), changed.into()]);
            let occurrence = Occurrence {
                text: surface.into(),
                gram_index: 0,
                bytes: (0, surface.len()),
                spans: vec![],
            };
            let shown: serde_json::Value = serde_json::from_str(&prompts::occurrence_prompt(
                &occurrence,
                &menu,
                Language::Japanese,
            ))
            .unwrap();
            assert!(
                shown["hierarchy"][0]["readings"][0]
                    .get("attachment")
                    .is_none()
            );
            assert!(
                shown["hierarchy"][0]["readings"][1]["attachment"]
                    .as_str()
                    .unwrap()
                    .contains("same compound")
            );
        }
        let policy = prompts::occurrence_system_prompt(Language::Japanese);
        for example in ["橋桁", "finite verb", "Numeral-counter"] {
            assert!(policy.contains(example));
        }
    }

    #[test]
    fn hierarchy_models_are_chosen_per_language() {
        assert_eq!(settings(Language::Japanese), ("gpt-6-luna", "medium"));
        assert_eq!(
            settings(Language::ChineseSimplified),
            ("gpt-6-luna", "medium")
        );
    }
}
