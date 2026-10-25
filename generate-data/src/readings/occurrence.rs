//! Occurrence labels with an explicit, guarded escape from an incomplete menu.
use super::*;

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(super) enum Label {
    Ranked {
        choices: Vec<Choice>,
    },
    Restated {
        proposed_reading: String,
        choices: Vec<Choice>,
    },
    Escape {
        missing_meaning: Option<String>,
        proposed_reading: String,
        reading: Option<String>,
    },
}
impl Label {
    pub fn choices(&self) -> Option<&[Choice]> {
        match self {
            Self::Ranked { choices } | Self::Restated { choices, .. } => Some(choices),
            Self::Escape { .. } => None,
        }
    }
    pub fn first(&self) -> Option<&Choice> {
        self.choices()?.first()
    }

    /// Pronunciation can be checked without establishing a listed meaning.
    pub fn reading(&self) -> Option<&str> {
        match self {
            Self::Escape { reading, .. } => reading.as_deref(),
            _ => self.first().map(|choice| choice.reading.as_str()),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub(super) struct Escape {
    pub occurrence: Occurrence,
    pub proposed_reading: String,
    pub reading: Option<String>,
    pub missing_meaning: Option<String>,
}
pub(super) type Escapes = BTreeMap<Gram<String>, Vec<Escape>>;

pub(super) fn resolve(answer: OccurrenceAnswer, hierarchy: &[SenseReadings]) -> Option<Label> {
    match answer.answer {
        OccurrenceDecision::Listed { choices } => Some(Label::Ranked {
            choices: resolve_choices(&choices, hierarchy)?,
        }),
        OccurrenceDecision::NoneOfThese {
            whole_gram_reading,
            missing_meaning,
        } => {
            if whole_gram_reading.trim().is_empty()
                || missing_meaning
                    .as_ref()
                    .is_some_and(|m| m.trim().is_empty())
            {
                return None;
            }
            Some(Label::Escape {
                missing_meaning,
                proposed_reading: whole_gram_reading,
                reading: None,
            })
        }
    }
}

fn guarded(
    answer: OccurrenceAnswer,
    hierarchy: &[SenseReadings],
    language: Language,
    text: &str,
) -> Option<Label> {
    let mut label = resolve(answer, hierarchy)?;
    if let Label::Escape {
        proposed_reading,
        reading,
        ..
    } = &mut label
    {
        *reading = canonical::menu_form(
            language,
            text,
            proposed_reading,
            hierarchy.iter().flat_map(|sense| &sense.readings),
        )
        .and_then(|r| canonical::checked_surface_reading(language, text, &r));
    }
    // Ranked and restated answers use the same final surface guard as escapes;
    // being on a menu is not permission to emit malformed phrase annotations.
    if let Label::Ranked { choices } | Label::Restated { choices, .. } = &mut label {
        for choice in choices {
            choice.reading = canonical::checked_surface_reading(language, text, &choice.reading)?;
        }
    }
    Some(label)
}

fn lemma_guard(
    mut label: Label,
    candidates: Option<&Candidates>,
    language: Language,
    text: &str,
) -> Option<Label> {
    let atoms = candidates.map_or(&[][..], |c| c.atoms.as_slice());
    let checked = |reading: &str| {
        learner_lemma_form(atoms, reading)
            .and_then(|r| canonical::checked_surface_reading(language, text, &r))
    };
    match &mut label {
        Label::Escape { reading, .. } => *reading = reading.as_deref().and_then(checked),
        Label::Ranked { choices } | Label::Restated { choices, .. } => {
            for choice in choices {
                choice.reading = checked(&choice.reading)?;
            }
        }
    }
    Some(label)
}

fn empty_boundary(character: Option<char>) -> bool {
    static EMPTY: std::sync::LazyLock<regex::Regex> =
        std::sync::LazyLock::new(|| regex::Regex::new(r"\A[\p{P}\s]\z").unwrap());
    character.is_none_or(|c| EMPTY.is_match(&c.to_string()))
}

/// Only external sound changes need neighbours outside this occurrence. Written
/// hiragana, katakana, Han, digits and other pronounced neighbours stay contextual.
fn occurrence_hierarchy(
    hierarchy: &[SenseReadings],
    occurrence: &Occurrence,
    language: Language,
) -> Vec<SenseReadings> {
    let before = empty_boundary(occurrence.text[..occurrence.bytes.0].chars().next_back());
    let after = empty_boundary(occurrence.text[occurrence.bytes.1..].chars().next());
    hierarchy
        .iter()
        .filter_map(|sense| {
            let mut local = sense.clone();
            if language == Language::Japanese && (before || after) {
                local.readings.retain(|reading| {
                    !sense.readings.iter().any(|plain| {
                        ((before
                            && reading.chars().next() != plain.chars().next()
                            && !reading.starts_with('っ'))
                            || (after
                                && reading.ends_with('っ')
                                && plain.ends_with(['く', 'き', 'ち', 'つ'])))
                            && canonical::sound_changed(reading, plain)
                    })
                });
            }
            (!local.readings.is_empty()).then_some(local)
        })
        .collect()
}

fn guard_attachment(label: &mut Label, hierarchy: &[SenseReadings], offered: &[SenseReadings]) {
    if let Label::Escape { reading, .. } = label
        && reading.as_ref().is_some_and(|r| {
            hierarchy.iter().any(|s| s.readings.contains(r))
                && !offered.iter().any(|s| s.readings.contains(r))
        })
    {
        *reading = None;
    }
}

fn restatement_menu(
    label: &Label,
    hierarchy: &[SenseReadings],
    language: Language,
    text: &str,
) -> Vec<SenseReadings> {
    let Label::Escape {
        reading: Some(reading),
        missing_meaning: None,
        ..
    } = label
    else {
        return vec![];
    };
    hierarchy
        .iter()
        .filter_map(|sense| {
            let readings: Vec<_> = sense
                .readings
                .iter()
                .filter(|r| {
                    canonical::checked_surface_reading(language, text, r).is_some_and(|form| {
                        form == *reading
                            || (language == Language::ChineseSimplified
                                && canonical::tone_variant(reading, &form))
                    })
                })
                .cloned()
                .collect();
            (!readings.is_empty()).then(|| SenseReadings {
                readings,
                ..sense.clone()
            })
        })
        .collect()
}

pub(super) fn select_requests<'a>(
    data: &CorpusReadings,
    groups: &'a Groups,
    selected: &'a BTreeSet<Gram<String>>,
    reconsider: &BTreeSet<Gram<String>>,
) -> Vec<(&'a Gram<String>, &'a Occurrence)> {
    selected
        .iter()
        .filter(|gram| !data.corrections_dropped.contains(*gram))
        .flat_map(|gram| groups[gram].iter().map(move |o| (gram, o)))
        .filter(|(gram, o)| {
            reconsider.contains(*gram)
                || data
                    .labels
                    .get(&(o.text.clone(), o.gram_index))
                    .is_none_or(|l| l.choices().is_none())
        })
        .collect()
}

fn prepare_rerun(
    data: &mut CorpusReadings,
    requests: &[(&Gram<String>, &Occurrence)],
) -> BTreeMap<(String, usize), Label> {
    let mut escapes = BTreeMap::new();
    for (_, occurrence) in requests {
        let key = (occurrence.text.clone(), occurrence.gram_index);
        if let Some(label @ Label::Escape { .. }) = data.labels.remove(&key) {
            escapes.insert(key, label);
        }
    }
    escapes
}

fn retry_label(
    label: Option<Label>,
    previous_escape: Option<Label>,
    occurrence: &Occurrence,
    unlabeled: &mut usize,
) -> Option<Label> {
    // An explicit escape survives a malformed retry. Without one, there is no
    // proposed pronunciation to verify, so count the missing ruby rather than
    // inventing a reading or treating a model-output failure as a fatal error.
    let label = label.or(previous_escape);
    if label.is_none() {
        *unlabeled += 1;
        log::warn!(
            "readings: occurrence rejected after retry; leaving reading unset: {}",
            occurrence.marked()
        );
    }
    label
}

pub(super) async fn annotate(
    data: &mut CorpusReadings,
    language: Language,
    groups: &Groups,
    selected: &BTreeSet<Gram<String>>,
    reconsider: &BTreeSet<Gram<String>>,
    verifier: &kanjidic::Verifier,
    round: &str,
) -> Result<Escapes> {
    tysm::measurement::stage(&format!("{}:readings-C:{round}", language.code()));
    for gram in selected {
        ensure!(
            data.corrections_dropped.contains(gram)
                || data
                    .candidates
                    .get(gram)
                    .is_none_or(|c| c.readings.is_empty())
                || data
                    .hierarchies
                    .get(gram)
                    .is_some_and(|h| h.iter().any(|s| !s.readings.is_empty())),
            "candidate-backed gram has no C menu: {:?}",
            gram.to_display_string(language)
        );
    }
    let requests = select_requests(data, groups, selected, reconsider);
    if requests.is_empty() {
        return Ok(Escapes::new());
    }
    let (mut boundary_occurrences, mut boundary_readings) = (0, 0);
    if language == Language::Japanese {
        for (gram, occurrence) in &requests {
            let hierarchy = data.hierarchies.get(*gram).map_or(&[][..], Vec::as_slice);
            let offered = occurrence_hierarchy(hierarchy, occurrence, language);
            let removed = hierarchy.iter().map(|s| s.readings.len()).sum::<usize>()
                - offered.iter().map(|s| s.readings.len()).sum::<usize>();
            boundary_occurrences += usize::from(removed > 0);
            boundary_readings += removed;
        }
    }
    println!(
        "readings[{}]: boundary filter occurrences={boundary_occurrences}, reading_slots={boundary_readings}, round={round}",
        language.code()
    );
    let reconsidered = requests
        .iter()
        .filter(|(_, o)| {
            data.labels
                .get(&(o.text.clone(), o.gram_index))
                .is_some_and(|l| l.choices().is_some())
        })
        .count();
    // A changed menu invalidates earlier choices for that gram. Unchanged menus
    // retain their successful labels and retry only unresolved occurrences.
    let mut retry_escapes = prepare_rerun(data, &requests);
    let client = crate::apply_cache_only(
        crate::base_chat_client("gpt-6-luna").with_reasoning_effort("medium"),
    );
    let system = &prompts::occurrence_system_prompt(language);
    let prompt = |(gram, o): &(&Gram<String>, &Occurrence)| {
        let hierarchy = occurrence_hierarchy(
            data.hierarchies.get(*gram).map_or(&[][..], Vec::as_slice),
            o,
            language,
        );
        let mut prompt: serde_json::Value =
            serde_json::from_str(&prompts::occurrence_prompt(o, &hierarchy, language)).unwrap();
        if let Some(candidates) = data.candidates.get(*gram) {
            prompt["dictionary_evidence"] = serde_json::json!(candidates.evidence);
            prompt["atoms"] = serde_json::json!(candidates.atoms);
            prompts::add_coverings(&mut prompt, candidates);
        }
        prompt.to_string()
    };
    println!(
        "readings[{}]: occurrence requests={}, reconsidered_ranked={reconsidered}, round={round}",
        language.code(),
        requests.len()
    );
    let mut accepted = Vec::new();
    let track_rounds = data.sample.is_some();
    let mut decided = BTreeSet::new();
    let mut retry = Vec::new();
    let mut rejected = BTreeMap::new();
    let parse = |request: &(&Gram<String>, &Occurrence), answer| {
        let hierarchy = data
            .hierarchies
            .get(request.0)
            .map_or(&[][..], Vec::as_slice);
        let offered = occurrence_hierarchy(hierarchy, request.1, language);
        guarded(
            answer,
            &offered,
            language,
            &request.0.to_display_string(language),
        )
        .and_then(|label| {
            lemma_guard(
                label,
                data.candidates.get(request.0),
                language,
                &request.0.to_display_string(language),
            )
        })
        .map(|mut label| {
            guard_attachment(&mut label, hierarchy, &offered);
            label
        })
    };
    for (request, answer) in
        batch::<_, OccurrenceAnswer>(&client, system, &requests, &prompt).await?
    {
        let previous = match &answer {
            Ok(answer) => {
                serde_json::json!({"rejected_answer":answer,"check":prompts::OCCURRENCE_CHECK})
            }
            Err(error) => serde_json::json!({"schema_error":error.to_string()}),
        };
        rejected.insert((request.1.text.clone(), request.1.gram_index), previous);
        match answer.ok().and_then(|a| parse(request, a)) {
            Some(label @ Label::Escape { reading: None, .. }) => {
                let key = (request.1.text.clone(), request.1.gram_index);
                if track_rounds && !retry_escapes.contains_key(&key) {
                    decided.insert(key.clone());
                }
                retry_escapes.entry(key).or_insert(label);
                retry.push(request);
            }
            Some(label) => {
                if track_rounds {
                    decided.insert((request.1.text.clone(), request.1.gram_index));
                }
                accepted.push((request, label));
            }
            None => retry.push(request),
        }
    }
    let retried = retry.len();
    let mut failures = 0;
    let mut unlabeled = 0;
    if !retry.is_empty() {
        for (request, answer) in batch::<_, OccurrenceAnswer>(&client, system, &retry, |r| {
            prompts::occurrence_retry(&format!(
                "{}\nRejected answer and check: {}",
                prompt(r),
                rejected[&(r.1.text.clone(), r.1.gram_index)]
            ))
        })
        .await?
        {
            let label = answer.ok().and_then(|a| parse(request, a));
            failures += usize::from(
                label
                    .as_ref()
                    .is_none_or(|l| matches!(l, Label::Escape { reading: None, .. })),
            );
            let key = (request.1.text.clone(), request.1.gram_index);
            if track_rounds && label.is_some() {
                decided.insert(key.clone());
            }
            if let Some(label) =
                retry_label(label, retry_escapes.remove(&key), request.1, &mut unlabeled)
            {
                accepted.push((*request, label));
            }
        }
    }
    let mut invalid_escapes = 0;
    let mut checks = Vec::new();
    for (i, ((gram, occurrence), label)) in accepted.iter_mut().enumerate() {
        if !matches!(label, Label::Escape { .. }) {
            continue;
        }
        let hierarchy = data.hierarchies.get(*gram).map_or(&[][..], Vec::as_slice);
        guard_attachment(
            label,
            hierarchy,
            &occurrence_hierarchy(hierarchy, occurrence, language),
        );
        let Label::Escape { reading, .. } = label else {
            unreachable!()
        };
        let text = gram.to_display_string(language);
        if let Some(reading) = reading {
            if language == Language::Japanese {
                let request = review::Request {
                    key: i,
                    gram: text.clone(),
                    sentence: occurrence.marked(),
                    reading: reading.clone(),
                    whole_word_readings: data
                        .dictionary
                        .get(&text)
                        .into_iter()
                        .flatten()
                        .map(|r| r.reading.clone())
                        .collect(),
                };
                if request.needs_check(verifier) {
                    checks.push(request);
                }
            }
        } else {
            invalid_escapes += 1;
        }
    }
    let mut verification = review::Stats::default();
    for (i, corrected) in review::verify_requests(&checks, verifier, &mut verification).await? {
        let ((gram, occurrence), label) = &mut accepted[i];
        let Label::Escape { reading, .. } = label else {
            unreachable!()
        };
        if track_rounds && *reading != corrected {
            decided.insert((occurrence.text.clone(), occurrence.gram_index));
        }
        *reading = corrected
            .and_then(|r| {
                learner_lemma_form(
                    data.candidates
                        .get(*gram)
                        .map_or(&[][..], |c| c.atoms.as_slice()),
                    &r,
                )
            })
            .and_then(|r| {
                canonical::checked_surface_reading(language, &gram.to_display_string(language), &r)
            });
        let hierarchy = data.hierarchies.get(*gram).map_or(&[][..], Vec::as_slice);
        guard_attachment(
            label,
            hierarchy,
            &occurrence_hierarchy(hierarchy, occurrence, language),
        );
    }
    // Pronunciation credit never implies semantic credit, even for a singleton.
    let focused: Vec<_> = accepted
        .iter()
        .enumerate()
        .filter_map(|(index, ((gram, occurrence), label))| {
            let offered = occurrence_hierarchy(
                data.hierarchies.get(*gram).map_or(&[][..], Vec::as_slice),
                occurrence,
                language,
            );
            let menu =
                restatement_menu(label, &offered, language, &gram.to_display_string(language));
            (!menu.is_empty()).then_some((index, menu))
        })
        .collect();
    for ((index, menu), answer) in
        batch::<_, OccurrenceAnswer>(&client, system, &focused, |(index, menu)| {
            let ((gram, o), label) = &accepted[*index];
            let Label::Escape {
                reading: Some(reading),
                ..
            } = label
            else {
                unreachable!()
            };
            prompts::occurrence_sense_followup(
                o,
                menu,
                &occurrence_hierarchy(&data.hierarchies[*gram], o, language),
                data.candidates.get(*gram),
                language,
                reading,
            )
        })
        .await?
    {
        let ((gram, _), label) = &mut accepted[*index];
        if let Ok(answer) = answer {
            let missing = match &answer.answer {
                OccurrenceDecision::NoneOfThese {
                    missing_meaning, ..
                } => missing_meaning.clone(),
                OccurrenceDecision::Listed { .. } => None,
            };
            if let Some(Label::Ranked { choices }) =
                guarded(answer, menu, language, &gram.to_display_string(language))
            {
                let Label::Escape {
                    proposed_reading, ..
                } = label
                else {
                    unreachable!()
                };
                *label = Label::Restated {
                    proposed_reading: proposed_reading.clone(),
                    choices,
                };
            } else if let Label::Escape {
                missing_meaning, ..
            } = label
            {
                *missing_meaning = missing.or_else(|| missing_meaning.clone());
            }
        }
    }
    let mut escapes = Escapes::new();
    let mut restated = 0;
    let mut file =
        std::io::BufWriter::new(std::fs::OpenOptions::new().create(true).append(true).open(
            format!(
                ".cache/readings/{}-occurrence-rounds.jsonl",
                language.code()
            ),
        )?);
    for (&(gram, o), label) in accepted {
        restated += usize::from(matches!(&label, Label::Restated { .. }));
        writeln!(
            file,
            "{}",
            serde_json::json!({"gram":gram,"occurrence":o,"round":round,"hierarchy":occurrence_hierarchy(data.hierarchies.get(gram).map_or(&[][..], Vec::as_slice), o, language),"label":label})
        )?;
        if let Label::Escape {
            proposed_reading,
            reading,
            missing_meaning,
        } = &label
        {
            escapes.entry(gram.clone()).or_default().push(Escape {
                occurrence: o.clone(),
                proposed_reading: proposed_reading.clone(),
                reading: reading.clone(),
                missing_meaning: missing_meaning.clone(),
            });
        }
        let key = (o.text.clone(), o.gram_index);
        if let Some(sample) = &mut data.sample
            && (decided.contains(&key) || matches!(&label, Label::Restated { .. }))
        {
            sample.rounds.insert(key.clone(), round.to_owned());
        }
        data.labels.insert(key, label);
    }
    file.flush()?;
    if let Some(sample) = &mut data.sample {
        sample.focused_sent += focused.len();
        sample.focused_resolved += restated;
        sample.reconsidered_ranked += reconsidered;
        sample.boundary_filtered_occurrences += boundary_occurrences;
        sample.boundary_filtered_readings += boundary_readings;
    }
    println!(
        "readings[{}]: focused sense follow-ups sent={}, resolved={restated}, round={round}",
        language.code(),
        focused.len()
    );
    println!(
        "readings[{}]: step C sent={}, retried={retried}, failed={failures}, unlabeled={unlabeled}, escape_grams={}, escaped_occurrences={}, invalid_escape_readings={invalid_escapes}, restated={restated}, round={round}",
        language.code(),
        requests.len(),
        escapes.len(),
        escapes.values().map(Vec::len).sum::<usize>()
    );
    println!(
        "readings[{}]: escape verification ({round}) {}",
        language.code(),
        serde_json::to_string(&verification)?
    );
    Ok(escapes)
}

/// Reading-stratified, bounded evidence. Frequent escape readings get priority;
/// within each reading the deterministic shuffle avoids favouring corpus order.
pub(super) fn forced_examples(escapes: &[Escape]) -> Vec<&Escape> {
    const LIMIT: usize = 20;
    let mut buckets: BTreeMap<_, Vec<&Escape>> = BTreeMap::new();
    for escape in escapes {
        buckets
            .entry((escape.reading.as_deref(), escape.missing_meaning.as_deref()))
            .or_default()
            .push(escape);
    }
    let mut buckets: Vec<_> = buckets.into_iter().collect();
    for (_, examples) in &mut buckets {
        examples.sort_by_key(|e| {
            (
                xxhash_rust::xxh3::xxh3_64(e.occurrence.text.as_bytes()),
                e.occurrence.gram_index,
            )
        });
    }
    buckets.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then(a.0.cmp(&b.0)));
    let mut selected = Vec::new();
    for i in 0..escapes.len() {
        let before = selected.len();
        for (_, examples) in &buckets {
            if let Some(escape) = examples.get(i) {
                selected.push(*escape);
            }
            if selected.len() == LIMIT {
                return selected;
            }
        }
        if selected.len() == before {
            break;
        }
    }
    selected
}

pub(super) fn force_into(request: &mut HierarchyRequest, escapes: &[Escape]) {
    let mut hints = Vec::new();
    for escape in escapes {
        let o = &escape.occurrence;
        let index = request
            .examples
            .iter()
            .position(|e| e.text == o.text && e.bytes == o.bytes)
            .unwrap_or_else(|| {
                request.examples.push(o.clone());
                request.examples.len() - 1
            });
        hints.push(serde_json::json!({"example_index":index,"suggested_whole_gram_reading":escape.reading,"missing_meaning":escape.missing_meaning}));
    }
    request.prompt["examples"] = prompts::numbered_examples(&request.examples);
    request.prompt["occurrence_escapes"] = serde_json::json!(hints);
    // This also makes B2 a distinct request when all escaped examples were
    // already sampled by B1: simply deduplicating them would reuse the old menu.
    request.prompt["review_note"] = serde_json::json!(prompts::REPAIR_NOTE);
}

pub(super) fn write_remaining(language: Language, escapes: &Escapes) -> Result<()> {
    let mut file = std::io::BufWriter::new(std::fs::File::create(format!(
        ".cache/readings/{}-remaining-escapes.jsonl",
        language.code()
    ))?);
    for (gram, occurrences) in escapes {
        for escape in occurrences {
            writeln!(
                file,
                "{}",
                serde_json::json!({"gram":gram,"text":gram.to_display_string(language),"sentence":escape.occurrence.text,"gram_index":escape.occurrence.gram_index,"escape_reading":escape.proposed_reading,"guarded_reading":escape.reading,"missing_meaning":escape.missing_meaning})
            )?;
        }
    }
    file.flush()?;
    println!(
        "readings[{}]: remaining escape_grams={}, occurrences={}; checked escape readings supply ruby without sense credit",
        language.code(),
        escapes.len(),
        escapes.values().map(Vec::len).sum::<usize>()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use language_utils::{Heteronym, PartOfSpeech, Word};
    fn gram(text: &str) -> Gram<String> {
        Gram(vec![Atom::Tok(Word {
            text: text.into(),
            word_type: WordType::Heteronym(Heteronym {
                word: text.into(),
                lemma: text.into(),
                pos: PartOfSpeech::Noun,
            }),
        })])
    }
    fn occurrence(text: &str) -> Occurrence {
        Occurrence {
            text: text.into(),
            gram_index: 0,
            bytes: (0, text.len()),
            spans: vec![(0, text.chars().count() as u32)],
        }
    }
    fn listed(choices: &[(u32, usize)]) -> OccurrenceAnswer {
        OccurrenceAnswer {
            answer: OccurrenceDecision::Listed {
                choices: choices
                    .iter()
                    .map(|&(sense, reading)| IndexedChoice { sense, reading })
                    .collect(),
            },
        }
    }
    fn escaped(reading: &str, meaning: Option<&str>) -> OccurrenceAnswer {
        OccurrenceAnswer {
            answer: OccurrenceDecision::NoneOfThese {
                whole_gram_reading: reading.into(),
                missing_meaning: meaning.map(str::to_owned),
            },
        }
    }

    #[test]
    fn rejected_retries_count_missing_ruby_without_aborting_or_inventing_a_reading() {
        let mut o = occurrence("你现在进来，谁也知道你是来查内鬼");
        o.bytes = ("你现在进来，".len(), "你现在进来，谁也".len());
        let menu = implicit_hierarchy(vec!["shéi yě".into()]);
        // Both cached attempts chose reading 1 from a menu with only index 0.
        let invalid = || {
            guarded(
                listed(&[(0, 1)]),
                &menu,
                Language::ChineseSimplified,
                "谁也",
            )
        };
        let initial = invalid();
        let retried = invalid();
        assert!(initial.is_none());
        assert!(retried.is_none());
        let mut unlabeled = 0;
        assert!(retry_label(retried, initial, &o, &mut unlabeled).is_none());
        assert_eq!(unlabeled, 1);
        let escape = guarded(
            escaped("shéi yě", Some("任何人都")),
            &menu,
            Language::ChineseSimplified,
            "谁也",
        )
        .unwrap();
        let retained = retry_label(None, Some(escape), &o, &mut unlabeled).unwrap();
        assert_eq!(retained.reading(), Some("shéi yě"));
        assert!(retained.first().is_none());
        let ranked = guarded(
            listed(&[(0, 0)]),
            &menu,
            Language::ChineseSimplified,
            "谁也",
        );
        assert_eq!(
            retry_label(ranked, Some(retained), &o, &mut unlabeled)
                .unwrap()
                .first()
                .unwrap()
                .reading,
            "shéi yě"
        );
        assert_eq!(unlabeled, 1);
    }

    #[tokio::test]
    async fn candidate_backed_empty_menu_fails_before_any_model_request() {
        let gram = gram("私");
        let dictionary = Dictionary::from([("私".into(), vec!["わたし".into()])]);
        let mut data = CorpusReadings::default();
        data.candidates.insert(
            gram.clone(),
            candidates(&gram, Language::Japanese, &dictionary),
        );
        let groups = Groups::from([(gram.clone(), vec![occurrence("私")])]);
        let error = annotate(
            &mut data,
            Language::Japanese,
            &groups,
            &BTreeSet::from([gram]),
            &BTreeSet::new(),
            &kanjidic::Verifier::default(),
            "initial",
        )
        .await
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("candidate-backed gram has no C menu")
        );
    }

    #[test]
    fn external_boundaries_filter_offers_and_cannot_be_reopened_by_followups() {
        let mut hierarchy = implicit_hierarchy(vec!["びとの".into(), "ひとの".into()]);
        hierarchy[0].synthetic_readings.insert("びとの".into());
        for (prefix, allowed) in [
            ("", false),
            ("。", false),
            (" ", false),
            ("\u{3000}", false),
            ("「", false),
            ("何でも", true),
            ("仕事", true),
            ("プレハブ", true),
            ("Ｘ", true),
            ("3", true),
        ] {
            let mut o = occurrence(&format!("{prefix}人の陰"));
            o.bytes = (prefix.len(), prefix.len() + "人の".len());
            let offered = occurrence_hierarchy(&hierarchy, &o, Language::Japanese);
            assert_eq!(
                offered[0].readings.contains(&"びとの".into()),
                allowed,
                "{prefix}"
            );
            if !allowed {
                assert_eq!(offered[0].readings, ["ひとの"]);
                assert_eq!(
                    resolve(listed(&[(0, 0)]), &offered)
                        .unwrap()
                        .first()
                        .unwrap()
                        .reading,
                    "ひとの"
                );
                let raw = guarded(
                    escaped("びとの", None),
                    &offered,
                    Language::Japanese,
                    "人の",
                )
                .unwrap();
                assert!(restatement_menu(&raw, &offered, Language::Japanese, "人の").is_empty());
                let mut checked = raw;
                guard_attachment(&mut checked, &hierarchy, &offered);
                assert!(matches!(checked, Label::Escape { reading: None, .. }));
                assert_eq!(checked.reading(), None);
                assert_eq!(checked.first(), None);
                let prompt: serde_json::Value = serde_json::from_str(&prompts::occurrence_prompt(
                    &o,
                    &offered,
                    Language::Japanese,
                ))
                .unwrap();
                assert_eq!(prompt["surface"], "人の");
                assert_eq!(prompt["hierarchy"][0]["readings"][0]["index"], 0);
            }
        }
        let mut hierarchy = implicit_hierarchy(vec!["ろく".into(), "ろっ".into()]);
        hierarchy[0].synthetic_readings.insert("ろっ".into());
        for (suffix, allowed) in [
            ("", false),
            ("。", false),
            (" ", false),
            ("本", true),
            ("こ", true),
        ] {
            let mut o = occurrence(&format!("六{suffix}"));
            o.bytes = (0, "六".len());
            assert_eq!(
                occurrence_hierarchy(&hierarchy, &o, Language::Japanese)[0]
                    .readings
                    .contains(&"ろっ".into()),
                allowed
            );
        }
        let mut internal =
            implicit_hierarchy(vec!["めんどうことを".into(), "めんどうごとを".into()]);
        internal[0]
            .synthetic_readings
            .insert("めんどうごとを".into());
        assert!(
            occurrence_hierarchy(&internal, &occurrence("面倒事を"), Language::Japanese)[0]
                .readings
                .contains(&"めんどうごとを".into())
        );
    }

    #[test]
    fn strict_answer_schema_has_an_object_root_and_exclusive_decisions() {
        let format = serde_json::to_value(tysm::chat_completions::JsonSchemaFormat::new::<
            OccurrenceAnswer,
        >())
        .unwrap();
        let schema = &format["schema"];
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["required"], serde_json::json!(["answer"]));
        assert_eq!(schema["additionalProperties"], false);
        assert!(!schema.to_string().contains("\"oneOf\""));
        let variants = schema["$defs"]["OccurrenceDecision"]["anyOf"]
            .as_array()
            .unwrap();
        assert_eq!(variants.len(), 2);
        for variant in variants {
            assert_eq!(variant["additionalProperties"], false);
            assert_eq!(
                variant["required"].as_array().unwrap().len(),
                variant["properties"].as_object().unwrap().len()
            );
        }
        for value in [
            serde_json::json!({"answer":{"decision":"listed","choices":[{"sense":0,"reading":0}],"missing_meaning":"explanatory gloss"}}),
            serde_json::json!({"answer":{"decision":"listed","choices":[{"sense":0,"reading":0}],"whole_gram_reading":"ねこ"}}),
            serde_json::json!({"answer":{"decision":"none_of_these","choices":[],"whole_gram_reading":"ねこ","missing_meaning":null}}),
        ] {
            assert!(serde_json::from_value::<OccurrenceAnswer>(value).is_err());
        }
        assert!(serde_json::from_value::<OccurrenceAnswer>(serde_json::json!({"answer":{"decision":"listed","choices":[{"sense":0,"reading":0}]}})).is_ok());
        assert!(serde_json::from_value::<OccurrenceAnswer>(serde_json::json!({"answer":{"decision":"none_of_these","whole_gram_reading":"ねこ","missing_meaning":null}})).is_ok());
    }

    #[test]
    fn missing_meaning_is_not_a_reading_restatement() {
        let menu = vec![SenseReadings {
            index: 0,
            kind: "meaning".into(),
            synthetic_readings: BTreeSet::new(),
            gloss: "不必".into(),
            readings: vec!["bù yòng".into()],
        }];
        let label = guarded(
            escaped("bù yòng", Some("不使用")),
            &menu,
            Language::ChineseSimplified,
            "不用",
        )
        .unwrap();
        assert!(matches!(
            label,
            Label::Escape {
                missing_meaning: Some(_),
                reading: Some(_),
                ..
            }
        ));
    }
    #[test]
    fn escape_contract_is_explicit_and_guarded() {
        let answer = escaped("はいった", None);
        assert!(
            matches!(guarded(answer,&[],Language::Japanese,"入った"),Some(Label::Escape {reading:Some(r),..}) if r=="はいった")
        );
        let answer = escaped("はいる", None);
        assert!(matches!(
            guarded(answer, &[], Language::Japanese, "入った"),
            Some(Label::Escape { reading: None, .. })
        ));
        assert!(resolve(listed(&[]), &[]).is_none());
        assert!(resolve(listed(&[(0, 0)]), &[]).is_none());
    }
    #[test]
    fn listed_decisions_use_only_explicit_meaning_and_reading_indices() {
        let menu = vec![SenseReadings {
            index: 1,
            kind: "meaning".into(),
            synthetic_readings: BTreeSet::new(),
            gloss: "milk".into(),
            readings: vec!["ちち".into()],
        }];
        assert!(resolve(listed(&[(0, 0)]), &menu).is_none());
        assert!(resolve(listed(&[(1, 1)]), &menu).is_none());
        let result = resolve(listed(&[(1, 0)]), &menu).unwrap();
        assert_eq!(result.first().unwrap().reading, "ちち");
    }

    #[test]
    fn focused_semantics_cannot_change_to_an_unrelated_pronunciation() {
        let menu = vec![
            SenseReadings {
                index: 7,
                kind: "meaning".into(),
                synthetic_readings: BTreeSet::new(),
                gloss: "what".into(),
                readings: vec!["なに".into(), "なん".into()],
            },
            SenseReadings {
                index: 9,
                kind: "meaning".into(),
                synthetic_readings: BTreeSet::new(),
                gloss: "which".into(),
                readings: vec!["なん".into()],
            },
        ];
        let label = Label::Escape {
            missing_meaning: None,
            proposed_reading: "なん".into(),
            reading: Some("なん".into()),
        };
        let focused = restatement_menu(&label, &menu, Language::Japanese, "何");
        assert_eq!(focused.iter().map(|s| s.index).collect::<Vec<_>>(), [7, 9]);
        assert!(focused.iter().all(|s| s.readings == ["なん"]));
        assert!(label.first().is_none());
    }

    #[test]
    fn canonical_menu_restatements_need_explicit_semantic_followup() {
        for (language, surface, listed, proposed) in [
            (Language::Japanese, "母は", "ははは", "ははわ"),
            (Language::Japanese, "私は", "わたしは", "わたしわ"),
            (Language::Japanese, "学校へ", "がっこうへ", "がっこうえ"),
            (Language::Japanese, "香具師", "やし", "やし"),
            (Language::Japanese, "読まない", "よまない", "ヨマナイ"),
            (
                Language::Japanese,
                "食べてま〜す",
                "たべてまーす",
                "たべてま〜す",
            ),
            (
                Language::ChineseSimplified,
                "你一个人",
                "nǐ yī gè rén",
                "nǐ yí gè rén",
            ),
            (Language::ChineseSimplified, "不是", "bù shì", "bú shì"),
            (
                Language::ChineseSimplified,
                "告诉我",
                "gào su wǒ",
                "gào sù wǒ",
            ),
            (Language::ChineseSimplified, "朋友", "péng you", "péng yǒu"),
            (Language::ChineseSimplified, "啊", "a", "wa"),
            (
                Language::Japanese,
                "母は私に",
                "はははわたしに",
                "ははわわたしに",
            ),
        ] {
            let menu = vec![SenseReadings {
                index: 0,
                kind: "meaning".into(),
                synthetic_readings: BTreeSet::new(),
                gloss: String::new(),
                readings: vec![listed.into()],
            }];
            let result = guarded(escaped(proposed, None), &menu, language, surface).unwrap();
            assert!(matches!(&result, Label::Escape { .. }));
            assert!(result.first().is_none());
            let focused = restatement_menu(&result, &menu, language, surface);
            assert_eq!(focused.len(), 1);
            assert_eq!(focused[0].readings, [listed]);
        }
        let incomplete = guarded(
            escaped("ははわたしに", None),
            &implicit_hierarchy(vec!["はははわたしに".into()]),
            Language::Japanese,
            "母は私に",
        )
        .unwrap();
        assert!(
            matches!(incomplete, Label::Escape { .. }),
            "never insert a missing particle to restate the menu"
        );
        let menu = vec![SenseReadings {
            index: 0,
            kind: "meaning".into(),
            synthetic_readings: BTreeSet::new(),
            gloss: String::new(),
            readings: vec!["bā gè".into()],
        }];
        assert!(matches!(
            guarded(
                escaped("bā ge", None),
                &menu,
                Language::ChineseSimplified,
                "八个"
            ),
            Some(Label::Escape { .. })
        ));
    }
    #[test]
    fn a_missing_meaning_preserves_tones_in_both_directions() {
        for (offered, proposed) in [("nà shi", "nà shì"), ("nà shì", "nà shi")] {
            let menu = implicit_hierarchy(vec![offered.into()]);
            let label = guarded(
                escaped(proposed, Some("a different meaning")),
                &menu,
                Language::ChineseSimplified,
                "那是",
            )
            .unwrap();
            assert!(matches!(label, Label::Escape { reading: Some(r), .. } if r == proposed));
        }
    }
    #[test]
    fn tone_restatements_never_credit_a_meaning_without_selection() {
        let escape = |proposed: &str, readings: &[&str]| {
            let menu: Vec<_> = readings
                .iter()
                .enumerate()
                .map(|(i, r)| SenseReadings {
                    index: i as u32 + 1,
                    kind: "meaning".into(),
                    synthetic_readings: BTreeSet::new(),
                    gloss: format!("meaning {i}"),
                    readings: vec![(*r).into()],
                })
                .collect();
            guarded(
                escaped(proposed, None),
                &menu,
                Language::ChineseSimplified,
                "那是",
            )
            .unwrap()
        };
        assert!(
            matches!(escape("nà shi", &["nà shì", "nà shí"]), Label::Escape { reading: Some(r), .. } if r == "nà shi")
        );
        assert!(matches!(
            escape("nà shǐ", &["nà shì"]),
            Label::Escape { .. }
        ));
        assert!(matches!(
            escape("nà shi", &["nà shì"]),
            Label::Escape { .. }
        ));
        assert!(matches!(
            escape("nà shì", &["nà shi", "nà shi"]),
            Label::Escape { .. }
        ));
        assert!(matches!(
            escape("nà shi", &["nà shi", "nà shi"]),
            Label::Escape { .. }
        ));
        // Even an exact, uniquely matching pronunciation is not semantic credit.
        assert!(escape("nà shi", &["nà shi", "nà shì"]).first().is_none());
    }
    #[test]
    fn ranked_phrases_are_checked_and_a_is_canonical_on_every_path() {
        for (language, text, reading, expected) in [
            (Language::ChineseSimplified, "走啊", "zǒu á", Some("zǒu a")),
            (Language::ChineseSimplified, "走啊", "zǒu wa", Some("zǒu a")),
            (
                Language::ChineseSimplified,
                "东西",
                "dōng xī",
                Some("dōng xī"),
            ),
            (
                Language::ChineseSimplified,
                "东西",
                "dōng xi",
                Some("dōng xi"),
            ),
            (Language::ChineseSimplified, "的问题", "wèn tí", None),
            (
                Language::Japanese,
                "からの贈り物",
                "てんからのおくりもの",
                None,
            ),
            (Language::ChineseSimplified, "你的方法", "nǐ de fāng", None),
            (Language::Japanese, "には入って", "ににはいって", None),
            (Language::Japanese, "食べている", "たべる", None),
        ] {
            let answer = listed(&[(0, 0)]);
            let got = guarded(
                answer,
                &implicit_hierarchy(vec![reading.into()]),
                language,
                text,
            );
            assert_eq!(
                got.as_ref()
                    .and_then(Label::first)
                    .map(|c| c.reading.as_str()),
                expected,
                "{text}"
            );
        }
    }

    #[test]
    fn c_tells_same_reading_senses_apart_by_bs_concise_gloss() {
        let menu = [("方向", "ほう"), ("比較の一方", "ほう"), ("人", "かた")]
            .into_iter()
            .zip(1..)
            .map(|((gloss, reading), index)| SenseReadings {
                index,
                kind: "meaning".into(),
                synthetic_readings: BTreeSet::new(),
                gloss: gloss.into(),
                readings: vec![reading.into()],
            })
            .collect::<Vec<_>>();
        let prompt: serde_json::Value = serde_json::from_str(&prompts::occurrence_prompt(
            &occurrence("この方がいい"),
            &menu,
            Language::Japanese,
        ))
        .unwrap();
        let shown: Vec<_> = prompt["hierarchy"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| (s["sense"].as_u64().unwrap(), s["gloss"].as_str().unwrap()))
            .collect();
        assert_eq!(shown, [(1, "方向"), (2, "比較の一方"), (3, "人")]);
    }
    #[test]
    fn remaining_escape_emits_only_its_checked_reading_without_sense_credit() {
        for (language, text, proposed, checked) in [
            (Language::Japanese, "入った", "はいった", Some("はいった")),
            (Language::Japanese, "入った", "はいる", None),
            // Verification may correct the original proposal.
            (Language::Japanese, "一日", "いちにち", Some("ついたち")),
            (
                Language::ChineseSimplified,
                "不用",
                "bù yòng",
                Some("bù yòng"),
            ),
        ] {
            let g = gram(text);
            let mut data = CorpusReadings::default();
            data.hierarchies
                .insert(g.clone(), implicit_hierarchy(vec![proposed.into()]));
            let mut label = guarded(
                escaped(proposed, Some("unlisted meaning")),
                &[],
                language,
                text,
            )
            .unwrap();
            if let Label::Escape { reading, .. } = &mut label {
                assert_eq!(reading.is_some(), checked.is_some());
                *reading = checked.map(str::to_owned);
            }
            assert!(label.first().is_none());
            data.labels.insert((text.into(), 0), label);
            assert_eq!(data.reading(text, 0), checked);
            assert_eq!(data.sense(text, 0), None);
            assert_eq!(data.reading(text, 1), None);
            let groups = Groups::from([(g.clone(), vec![occurrence(text)])]);
            let sample = sample_run::labels(&data, &groups);
            let exported = &sample[&(text.into(), 0)];
            assert_eq!(exported["reading"], serde_json::json!(checked));
            assert_eq!(exported["proposed_reading"], proposed);
            assert!(exported.get("choices").is_none());
            let sentence = SentenceGrams {
                grams: vec![language_utils::SentenceGram::Learnable(TaggedGram {
                    gram: g.clone(),
                    sense: data.sense(text, 0),
                })],
                capitalize_first: false,
                multiword_terms: vec![],
                low_confidence_multiword_terms: vec![],
            };
            let packed = data.pack(vec![(text.into(), sentence)], language);
            let entry = packed[0].1.all_grams().next().unwrap();
            assert_eq!(entry.reading.as_deref(), checked);
            assert_eq!(entry.sense.sense, None);
            assert_eq!(entry.sense.gram, g);
        }
    }
    #[test]
    fn changed_menus_reconsider_ranked_rows_while_unchanged_menus_keep_them() {
        let g = gram("一日");
        let escaped = occurrence("四月一日");
        let ranked = occurrence("一日働いた");
        let mut data = CorpusReadings::default();
        data.labels.insert(
            (escaped.text.clone(), 0),
            Label::Escape {
                proposed_reading: "ついたち".into(),
                missing_meaning: None,
                reading: Some("ついたち".into()),
            },
        );
        data.labels.insert(
            (ranked.text.clone(), 0),
            Label::Ranked {
                choices: vec![Choice {
                    sense: 7,
                    reading: "いちにち".into(),
                }],
            },
        );
        let groups = Groups::from([(g.clone(), vec![escaped.clone(), ranked.clone()])]);
        let selected = BTreeSet::from([g.clone()]);
        let unchanged = select_requests(&data, &groups, &selected, &BTreeSet::new());
        assert_eq!(unchanged.len(), 1);
        let retained = prepare_rerun(&mut data, &unchanged);
        assert_eq!(data.reading(&ranked.text, 0), Some("いちにち"));
        assert_eq!(data.sense(&ranked.text, 0).unwrap().get(), 7);
        data.labels.extend(retained);
        let changed = select_requests(&data, &groups, &selected, &selected);
        assert_eq!(changed.len(), 2);
        let retained = prepare_rerun(&mut data, &changed);
        assert_eq!(data.reading(&ranked.text, 0), None);
        assert_eq!(data.sense(&ranked.text, 0), None);
        assert_eq!(retained.len(), 1);
        assert!(matches!(
            retained[&(escaped.text.clone(), 0)],
            Label::Escape { .. }
        ));
        // With no trustworthy C2 replacement, this retained outcome is restored.
        data.labels.extend(retained);
        assert_eq!(data.reading(&escaped.text, 0), Some("ついたち"));
        assert_eq!(data.sense(&escaped.text, 0), None);
    }
    #[tokio::test]
    async fn previously_b_less_gram_gets_a_sampled_repair_request() {
        let g = gram("猫");
        let dictionary = BTreeMap::from([(
            "猫".into(),
            vec![DictionaryReading {
                reading: "ねこ".into(),
                senses: vec!["cat".into()],
                evidence: vec![],
            }],
        )]);
        let c = candidates(&g, Language::Japanese, &dictionary);
        assert!(!needs_hierarchy(&g, &c, 1));
        let cache = tempfile::tempdir().unwrap();
        let store = osmo::Store::open(cache.path());
        let examples = vec![occurrence("猫")];
        let mut request =
            HierarchyRequest::sampled(&g, c, &examples, Language::Japanese, &store, None).await;
        let initial_candidates = request.prompt["candidates"].clone();
        force_into(
            &mut request,
            &[Escape {
                occurrence: examples[0].clone(),
                proposed_reading: "ねこ".into(),
                reading: Some("ねこ".into()),
                missing_meaning: Some("別の意味".into()),
            }],
        );
        assert_eq!(request.examples.len(), 1);
        assert_eq!(request.prompt["candidates"], initial_candidates);
        assert_eq!(
            request.prompt["occurrence_escapes"][0]["missing_meaning"],
            "別の意味"
        );
    }

    #[test]
    fn removed_repair_witnesses_deduplicate_by_saved_byte_bounds() {
        let g = gram("猫");
        let c = candidates(&g, Language::Japanese, &BTreeMap::new());
        let examples: Vec<_> = [(0, 3), (3, 6)]
            .into_iter()
            .map(|bytes| Occurrence {
                text: "猫猫".into(),
                bytes,
                gram_index: usize::MAX,
                spans: vec![],
            })
            .collect();
        let mut request = HierarchyRequest {
            prompt: prompts::hierarchy_prompt(&g, &c, Language::Japanese),
            gram: g,
            candidates: c,
            examples: examples.clone(),
        };
        let escapes: Vec<_> = examples
            .into_iter()
            .map(|occurrence| Escape {
                occurrence,
                proposed_reading: "ねこ".into(),
                reading: Some("ねこ".into()),
                missing_meaning: None,
            })
            .collect();
        force_into(&mut request, &escapes);
        assert_eq!(request.examples.len(), 2);
        assert_eq!(
            request.prompt["examples"],
            serde_json::json!([
                {"index":0,"sentence":"«猫»猫"},
                {"index":1,"sentence":"猫«猫»"}
            ])
        );
        assert_eq!(request.prompt["occurrence_escapes"][0]["example_index"], 0);
        assert_eq!(request.prompt["occurrence_escapes"][1]["example_index"], 1);
    }

    #[test]
    fn forced_evidence_is_bounded_stratified_and_changes_an_already_seen_request() {
        let mut escapes: Vec<_> = (0..50)
            .map(|i| Escape {
                occurrence: occurrence(&format!("方{i}")),
                proposed_reading: "ほう".into(),
                missing_meaning: None,
                reading: Some("ほう".into()),
            })
            .collect();
        escapes.push(Escape {
            occurrence: occurrence("方rare"),
            proposed_reading: "かた".into(),
            missing_meaning: None,
            reading: Some("かた".into()),
        });
        let selected = forced_examples(&escapes);
        assert_eq!(selected.len(), 20);
        assert!(
            selected
                .iter()
                .any(|e| e.reading.as_deref() == Some("かた"))
        );
        let g = gram("方");
        let c = candidates(&g, Language::Japanese, &BTreeMap::new());
        let mut request = HierarchyRequest {
            prompt: prompts::hierarchy_prompt(&g, &c, Language::Japanese),
            gram: g,
            candidates: c,
            examples: vec![escapes[0].occurrence.clone()],
        };
        let before = request.prompt.clone();
        force_into(&mut request, &escapes[..1]);
        assert_eq!(request.examples.len(), 1);
        assert_ne!(request.prompt, before);
        assert_eq!(request.prompt["occurrence_escapes"][0]["example_index"], 0);
    }
}
