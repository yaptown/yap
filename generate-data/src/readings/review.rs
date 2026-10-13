//! Focused checks shared by B proposals and C's escaped readings.
use super::*;

#[derive(Default, Debug, Serialize)]
pub(super) struct Stats {
    pub flagged: usize,
    pub verified: usize,
    pub corrected: usize,
    pub dropped: usize,
    pub correction_checks: usize,
    pub errors: usize,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct Verdict {
    /// Whether this is exactly the pronunciation of the marked surface in context.
    verified: bool,
    reason: String,
    corrected_whole_gram_reading: Option<String>,
}

#[derive(Clone, Serialize)]
pub(super) struct Request<K> {
    pub key: K,
    pub gram: String,
    pub sentence: String,
    pub reading: String,
    #[serde(skip)]
    pub whole_word_readings: BTreeSet<String>,
}
impl<K> Request<K> {
    fn prompt(&self) -> String {
        serde_json::json!({"gram":self.gram,"sentence":self.sentence,"whole_gram_reading":self.reading}).to_string()
    }
    /// A model cannot move the marked boundaries. If removing literal kana
    /// from an adjacent word gives a dictionary-supported reading, that kana
    /// belongs to the neighbour, not to an exotic pronunciation of this gram.
    fn absorbs_neighbour(&self, reading: &str, verifier: &kanjidic::Verifier) -> bool {
        if self.whole_word_readings.contains(reading)
            || kanjidic::matches(&self.gram, reading, verifier)
        {
            return false;
        }
        let Some((before, marked)) = self.sentence.split_once('«') else {
            return false;
        };
        let Some((_, after)) = marked.split_once('»') else {
            return false;
        };
        let before = hiragana(before);
        let after = hiragana(after);
        reading.char_indices().skip(1).any(|(i, _)| {
            let (prefix, suffix) = reading.split_at(i);
            (before.ends_with(prefix) && kanjidic::matches(&self.gram, suffix, verifier))
                || (after.starts_with(suffix) && kanjidic::matches(&self.gram, prefix, verifier))
        })
    }
    pub fn needs_check(&self, verifier: &kanjidic::Verifier) -> bool {
        !self.whole_word_readings.contains(&self.reading)
            && !kanjidic::matches(&self.gram, &self.reading, verifier)
    }
}

enum Decision {
    Verified,
    Corrected(String),
    CheckCorrection(String),
    Drop,
}
fn decision<K>(verdict: Verdict, request: &Request<K>, verifier: &kanjidic::Verifier) -> Decision {
    if verdict.verified {
        return if request.absorbs_neighbour(&request.reading, verifier) {
            Decision::Drop
        } else {
            Decision::Verified
        };
    }
    let Some(reading) = verdict
        .corrected_whole_gram_reading
        .and_then(|r| {
            canonical::menu_form(
                Language::Japanese,
                &request.gram,
                &r,
                &request.whole_word_readings,
            )
        })
        .and_then(|r| canonical::checked_surface_reading(Language::Japanese, &request.gram, &r))
        .filter(|r| !request.absorbs_neighbour(r, verifier))
    else {
        return Decision::Drop;
    };
    if request.whole_word_readings.contains(&reading)
        || kanjidic::matches(&request.gram, &reading, verifier)
    {
        Decision::Corrected(reading)
    } else {
        Decision::CheckCorrection(reading)
    }
}
fn record<K: Serialize>(
    file: &mut impl Write,
    request: &Request<K>,
    answer: &Result<Verdict, IndividualChatError>,
    correction: bool,
) -> Result<()> {
    let answer = match answer {
        Ok(a) => serde_json::to_value(a)?,
        Err(e) => serde_json::json!({"error":e.to_string()}),
    };
    writeln!(
        file,
        "{}",
        serde_json::json!({"request":request,"correction_check":correction,"answer":answer})
    )?;
    Ok(())
}

pub(super) async fn verify_requests<K: Clone + Ord + Serialize>(
    requests: &[Request<K>],
    verifier: &kanjidic::Verifier,
    stats: &mut Stats,
) -> Result<BTreeMap<K, Option<String>>> {
    if requests.is_empty() {
        return Ok(BTreeMap::new());
    }
    stats.flagged += requests.len();
    println!(
        "readings[jpn]: focused verification requests={}",
        requests.len()
    );
    let mut file = std::io::BufWriter::new(
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(".cache/readings/jpn-verifications.jsonl")?,
    );
    // These checks always use Luna-low, independently of B's chosen model.
    let client =
        crate::apply_cache_only(crate::base_chat_client("gpt-6-luna").with_reasoning_effort("low"));
    let system = prompts::verification_system_prompt();
    let mut replacements = BTreeMap::new();
    let mut corrections = Vec::new();
    for (request, answer) in batch::<_, Verdict>(&client, system, requests, Request::prompt).await?
    {
        record(&mut file, request, &answer, false)?;
        let outcome = match answer {
            Ok(answer) => decision(answer, request, verifier),
            Err(_) => {
                stats.errors += 1;
                Decision::Drop
            }
        };
        match outcome {
            Decision::Verified => {
                stats.verified += 1;
                replacements.insert(request.key.clone(), Some(request.reading.clone()));
            }
            Decision::Corrected(reading) => {
                stats.corrected += 1;
                replacements.insert(request.key.clone(), Some(reading));
            }
            Decision::CheckCorrection(reading) => corrections.push(Request {
                reading,
                ..request.clone()
            }),
            Decision::Drop => {
                stats.dropped += 1;
                replacements.insert(request.key.clone(), None);
            }
        }
    }
    file.flush()?;
    stats.correction_checks += corrections.len();
    if !corrections.is_empty() {
        for (request, answer) in
            batch::<_, Verdict>(&client, system, &corrections, Request::prompt).await?
        {
            record(&mut file, request, &answer, true)?;
            stats.errors += usize::from(answer.is_err());
            if answer.is_ok_and(|a| matches!(decision(a, request, verifier), Decision::Verified)) {
                stats.corrected += 1;
                replacements.insert(request.key.clone(), Some(request.reading.clone()));
            } else {
                stats.dropped += 1;
                replacements.insert(request.key.clone(), None);
            }
        }
    }
    file.flush()?;
    assert_eq!(replacements.len(), requests.len());
    assert_eq!(
        stats.flagged,
        stats.verified + stats.corrected + stats.dropped
    );
    println!(
        "readings[jpn]: focused verification {}",
        serde_json::to_string(stats)?
    );
    Ok(replacements)
}

fn corrected_reading(request: &HierarchyRequest, reading: &str) -> Option<String> {
    learner_lemma_form(&request.candidates.atoms, reading).and_then(|r| {
        canonical::checked_surface_reading(
            Language::Japanese,
            &request.gram.to_display_string(Language::Japanese),
            &r,
        )
    })
}

/// A rejected last reading rejoins B's normal retry queue. Corrections cannot
/// leave duplicate readings or turn a new slot into a meaning-only subdivision.
pub(super) async fn verify<'a>(
    language: Language,
    accepted: &mut Vec<(&'a HierarchyRequest, Proposal<GroundedReading>)>,
    verifier: &kanjidic::Verifier,
    checks: &mut Rejections,
    stats: &mut Stats,
) -> Result<Vec<&'a HierarchyRequest>> {
    if language != Language::Japanese {
        return Ok(Vec::new());
    }
    let mut requests = Vec::new();
    for (i, (request, proposal)) in accepted.iter().enumerate() {
        if !request.candidates.open && !proposal.reopen {
            continue;
        }
        let text = request.gram.to_display_string(language);
        for (j, sense) in proposal.senses.iter().enumerate() {
            for (k, reading) in sense.readings.iter().enumerate() {
                let check = Request {
                    key: (i, j, k),
                    gram: text.clone(),
                    // A dictionary reading has no example of its own; any
                    // sample shows the gram in an ordinary context.
                    sentence: request.examples[reading.example.unwrap_or(0)].marked(),
                    reading: reading.reading.clone(),
                    whole_word_readings: request.candidates.whole_word_readings.clone(),
                };
                if check.needs_check(verifier) {
                    requests.push(check);
                }
            }
        }
    }
    let mut replacements = verify_requests(&requests, verifier, stats).await?;
    let mut failed = Vec::new();
    for (i, (request, mut proposal)) in std::mem::take(accepted).into_iter().enumerate() {
        for (j, sense) in proposal.senses.iter_mut().enumerate() {
            sense.readings = std::mem::take(&mut sense.readings)
                .into_iter()
                .enumerate()
                .filter_map(|(k, mut reading)| {
                    if let Some(replacement) = replacements.remove(&(i, j, k)) {
                        reading.reading = replacement
                            .as_deref()
                            .and_then(|r| corrected_reading(request, r))?;
                    }
                    Some(reading)
                })
                .collect();
        }
        if validation::normalize_senses(&mut proposal, language, checks)
            && validation::covers_forced_examples(&proposal, request)
        {
            accepted.push((request, proposal));
        } else {
            failed.push(request);
        }
    }
    Ok(failed)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request(text: &str) -> Request<usize> {
        Request {
            key: 0,
            gram: text.into(),
            sentence: format!("«{text}»"),
            reading: String::new(),
            whole_word_readings: BTreeSet::new(),
        }
    }
    fn verdict(verified: bool, corrected: Option<&str>) -> Verdict {
        Verdict {
            verified,
            reason: "checked the marked span".into(),
            corrected_whole_gram_reading: corrected.map(str::to_owned),
        }
    }
    #[test]
    fn verifier_correction_cannot_reintroduce_noncanonical_iu_in_b() {
        let mut gram = super::super::tests::gram(&["夕方"]);
        gram.0.extend(
            super::super::tests::word("言った", "言う", language_utils::PartOfSpeech::Verb).0,
        );
        let dictionary = Dictionary::from([
            ("夕方".into(), vec!["ゆうがた".into()]),
            ("言う".into(), vec!["いう".into()]),
        ]);
        let request = HierarchyRequest {
            candidates: candidates(&gram, Language::Japanese, &dictionary),
            gram,
            prompt: serde_json::Value::Null,
            examples: vec![],
        };
        assert_eq!(
            corrected_reading(&request, "ゆうがたゆった").as_deref(),
            Some("ゆうがたいった")
        );
        assert!(corrected_reading(&request, "ゆうかたゆった").is_none());
        assert!(corrected_reading(&request, "ゆうがたゆう").is_none());
    }

    #[test]
    fn verified_colloquial_reading_does_not_need_a_dictionary_path() {
        assert!(matches!(
            decision(
                verdict(true, None),
                &request("凄え"),
                &kanjidic::Verifier::default()
            ),
            Decision::Verified
        ));
    }
    #[test]
    fn a_verified_answer_cannot_absorb_a_written_neighbour() {
        let verifier = kanjidic::Verifier::new(
            BTreeMap::new(),
            &BTreeMap::from([("説明".into(), vec!["せつめい".into()])]),
            BTreeMap::new(),
        );
        let mut r = request("説明しましょう");
        r.sentence = "では、ここでご«説明しましょう»".into();
        r.reading = "ごせつめいしましょう".into();
        assert!(matches!(
            decision(verdict(true, None), &r, &verifier),
            Decision::Drop
        ));
        assert!(matches!(
            decision(verdict(false, Some("ごせつめいしましょう")), &r, &verifier),
            Decision::Drop
        ));
        assert!(matches!(
            decision(verdict(false, Some("せつめいしましょう")), &r, &verifier),
            Decision::Corrected(_)
        ));
    }
    #[test]
    fn short_jukujikun_and_written_particle_corrections_survive() {
        let mut r = request("私は");
        r.whole_word_readings.insert("わたしは".into());
        assert!(
            matches!(decision(verdict(false, Some("わたしわ")), &r, &kanjidic::Verifier::default()), Decision::Corrected(r) if r == "わたしは")
        );
        assert!(
            matches!(decision(verdict(false, Some("やし")), &request("香具師"), &kanjidic::Verifier::default()), Decision::CheckCorrection(r) if r == "やし")
        );
    }
    #[test]
    fn correction_must_pass_the_existing_surface_guards() {
        for (base, correction) in [("入った", Some("はいる")), ("何", None)] {
            assert!(matches!(
                decision(
                    verdict(false, correction),
                    &request(base),
                    &kanjidic::Verifier::default()
                ),
                Decision::Drop
            ));
        }
    }
    #[test]
    fn dictionary_correction_is_kept_and_other_corrections_need_verification() {
        let verifier = kanjidic::Verifier::new(
            BTreeMap::from([('一', vec!["いち".into()]), ('日', vec!["にち".into()])]),
            &BTreeMap::new(),
            BTreeMap::new(),
        );
        assert!(
            matches!(decision(verdict(false,Some("いちにち")),&request("一日"),&verifier),Decision::Corrected(r) if r=="いちにち")
        );
        assert!(
            matches!(decision(verdict(false,Some("すげえ")),&request("凄え"),&verifier),Decision::CheckCorrection(r) if r=="すげえ")
        );
    }
}
