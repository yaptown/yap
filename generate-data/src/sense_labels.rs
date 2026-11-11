//! Occurrence meaning labels shared by the reading and meaning pipelines.
use std::{collections::BTreeMap, io::Write, num::NonZeroU32, path::Path};

use anyhow::Result;
use language_utils::{Atom, Gram, GramInterners, Language, SentenceInfo};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::usage_discovery::UsageInventory;

/// First-choice occurrences a meaning needs to become a learnable sense.
pub(crate) const MIN_ATTESTATIONS: usize = 3;

#[derive(Default)]
pub struct SenseLabels(BTreeMap<(String, usize), Label>);

pub(crate) enum Label {
    Ranked(Vec<NonZeroU32>),
    FrequencyFallback,
}

impl FromIterator<((String, usize), NonZeroU32)> for SenseLabels {
    fn from_iter<T: IntoIterator<Item = ((String, usize), NonZeroU32)>>(iter: T) -> Self {
        Self(
            iter.into_iter()
                .map(|(key, sense)| (key, Label::Ranked(vec![sense])))
                .collect(),
        )
    }
}

impl SenseLabels {
    pub(crate) fn get(&self, text: &str, index: usize) -> Option<&Label> {
        self.0.get(&(text.to_owned(), index))
    }
}

fn system_prompt() -> String {
    format!(
        "Choose the meaning of the marked gram in its sentence. We insert « and » around the occurrence to show which one to judge; they are not part of the original text, so please read the sentence as if they were absent rather than as a quotation or a mention of the word. The usage menu contains persistent sense IDs. Return the compatible sense IDs ranked from most to least likely, with the best interpretation first; omit senses that do not fit. The absorber menu describes occurrences that are part of a fixed expression rather than a standalone usage: choosing an absorber means 'this occurrence is part of fixed expression <gloss>'. If neither menu fits, return none_of_these with a short missing_meaning description. Judge the marked occurrence in context, not the word's most common meaning. Return only the structured answer.\n\n{}",
        crate::readings::FRAGMENT_NOTE
    )
}

#[derive(Serialize)]
struct MenuEntry<'a> {
    id: usize,
    kind: &'a str,
    gloss: &'a str,
    examples: Vec<&'a str>,
}

#[derive(Serialize)]
struct Request<'a> {
    language: &'a str,
    gram: &'a Gram<String>,
    marked_sentence: String,
    usages: Vec<MenuEntry<'a>>,
    absorbers: Vec<MenuEntry<'a>>,
    #[serde(skip)]
    text: &'a str,
    #[serde(skip)]
    index: usize,
    #[serde(skip)]
    marked_span: (usize, usize),
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct Response {
    answer: Answer,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "decision", rename_all = "snake_case", deny_unknown_fields)]
enum Answer {
    Ranked { sense_ids: Vec<u32> },
    Absorber { id: usize },
    NoneOfThese { missing_meaning: String },
}

fn menu<'a>(
    entries: impl Iterator<Item = (usize, &'a crate::usage_discovery::UsageEntry)>,
) -> Vec<MenuEntry<'a>> {
    entries
        .map(|(i, entry)| MenuEntry {
            id: i + 1,
            kind: &entry.kind,
            gloss: &entry.gloss,
            examples: entry
                .anchors
                .iter()
                .take(4)
                .map(|a| a.sentence.as_str())
                .collect(),
        })
        .collect()
}

/// Sampling selects whole grams before building requests: sampled and full
/// requests (including their menus and cache keys) are byte-identical.
pub async fn generate(
    language: Language,
    app: &BTreeMap<String, SentenceInfo>,
    restricted: &BTreeMap<String, SentenceInfo>,
    interners: &GramInterners,
    rows: &[UsageInventory],
    sample_percent: Option<f64>,
    output: &Path,
) -> Result<SenseLabels> {
    let inventories: BTreeMap<_, _> = rows
        .iter()
        .filter(|row| {
            row.usages.iter().filter(|u| u.is_live()).count() >= 2
                && sample_percent.is_none_or(|percent| {
                    let key = serde_json::to_vec(&row.gram).unwrap();
                    (xxhash_rust::xxh3::xxh3_64(&key) as f64) / (u64::MAX as f64) * 100.0 < percent
                })
        })
        .filter_map(|row| {
            let gram = row
                .gram
                .get_interned(&interners.strings)?
                .get_interned(&interners.grams)?;
            Some((gram, row))
        })
        .collect();
    let mut requests = Vec::new();
    for (text, info) in app.iter().chain(restricted) {
        let words = info.decode_words(interners, language);
        let mut starts = Vec::new();
        let mut byte = 0;
        for literal in &words {
            starts.push((byte, byte + literal.word.text.len()));
            byte += literal.word.text.len() + literal.whitespace.as_str().len();
        }
        let mut word = 0;
        for (index, token) in info.sentence.tokens.iter().enumerate() {
            let count = interners
                .atoms(token.gram)
                .iter()
                .filter(|a| matches!(a, Atom::Tok(_)))
                .count();
            let start = word;
            word += count;
            let Some(row) = inventories.get(&token.gram) else {
                continue;
            };
            if count == 0 {
                continue;
            }
            let (left, right) = (starts[start].0, starts[word - 1].1);
            requests.push(Request {
                language: language.code(),
                gram: &row.gram,
                marked_sentence: format!(
                    "{}«{}»{}",
                    &text[..left],
                    &text[left..right],
                    &text[right..]
                ),
                usages: menu(
                    row.usages
                        .iter()
                        .enumerate()
                        .filter(|(_, entry)| entry.is_live()),
                ),
                absorbers: menu(row.absorbers.iter().enumerate()),
                text,
                index,
                marked_span: (text[..left].chars().count(), text[..right].chars().count()),
            });
        }
    }
    let mut client = crate::apply_cache_only(
        crate::base_chat_client("gpt-6-luna").with_reasoning_effort("medium"),
    );
    // Samples are for fast iteration; the cache is shared, so the full run
    // still batches only what the sample didn't already answer.
    if sample_percent.is_some() {
        client = client.with_no_batch();
    }
    let answers =
        crate::readings::batch::<_, Response>(&client, &system_prompt(), &requests, |r| {
            serde_json::to_string(r).unwrap()
        })
        .await?;
    std::fs::create_dir_all(output)?;
    let mut missing = std::fs::File::create(output.join("missing_senses.jsonl"))?;
    let mut raw = std::fs::File::create(output.join("sense_labels.jsonl"))?;
    let mut labels = SenseLabels::default();
    for (request, answer) in answers {
        writeln!(
            raw,
            "{}",
            serde_json::json!({"request": request, "response": answer.as_ref().ok(), "error": answer.as_ref().err().map(ToString::to_string)})
        )?;
        let label = match answer.map(|response| response.answer) {
            Ok(Answer::Ranked { sense_ids })
                if !sense_ids.is_empty()
                    && sense_ids
                        .iter()
                        .all(|id| request.usages.iter().any(|u| u.id == *id as usize)) =>
            {
                Label::Ranked(
                    sense_ids
                        .into_iter()
                        .map(|id| NonZeroU32::new(id).unwrap())
                        .collect(),
                )
            }
            Ok(Answer::NoneOfThese { missing_meaning }) => {
                writeln!(
                    missing,
                    "{}",
                    serde_json::json!({
                        "key": request.gram.to_display_string(language), "gram": request.gram,
                        "sentence": request.text, "gram_index": request.index,
                        "marked_span": request.marked_span, "marked_sentence": request.marked_sentence,
                        "missing_meaning": missing_meaning,
                    })
                )?;
                Label::FrequencyFallback
            }
            Ok(Answer::Absorber { id }) if request.absorbers.iter().any(|u| u.id == id) => {
                Label::FrequencyFallback
            }
            other => {
                log::warn!(
                    "sense-labels[{}]: invalid answer for {:?}: {:?}",
                    language.code(),
                    request.marked_sentence,
                    other
                );
                Label::FrequencyFallback
            }
        };
        labels
            .0
            .insert((request.text.to_owned(), request.index), label);
    }
    Ok(labels)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::usage_discovery::{UsageAnchor, UsageEntry};

    #[tokio::test]
    #[ignore = "live French smoke; requires model credentials"]
    async fn encore_live_smoke() -> Result<()> {
        dotenvy::from_path(concat!(env!("CARGO_MANIFEST_DIR"), "/../.env")).ok();
        let _ = rustls::crypto::ring::default_provider().install_default();
        let contents = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/fra/usage_inventories.jsonl"
        ))?;
        let mut row = contents
            .lines()
            .map(serde_json::from_str::<UsageInventory>)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .find(|r| r.key == "encore")
            .unwrap();
        row.usages.push(UsageEntry {
            source: Some("manual".into()),
            readings: None,
            kind: "meaning".into(),
            gloss: "de nouveau, une autre fois ; répétition d'une action".into(),
            n_gold: 0,
            n_assigned: 0,
            loo_correct: 0,
            anchors: vec![UsageAnchor {
                sentence: "Dis-le encore.".into(),
                spans: vec![(7, 13)],
                gold: true,
            }],
        });
        let cases = [
            ("Dis-le encore.", "Dis-le «encore»."),
            ("Il dort encore.", "Il dort «encore»."),
            (
                "Il n'est pas encore arrivé.",
                "Il n'est pas «encore» arrivé.",
            ),
        ];
        let requests: Vec<_> = cases
            .iter()
            .map(|(text, marked)| Request {
                language: "fra",
                gram: &row.gram,
                marked_sentence: (*marked).into(),
                usages: menu(
                    row.usages
                        .iter()
                        .enumerate()
                        .filter(|(_, entry)| entry.is_live()),
                ),
                absorbers: menu(row.absorbers.iter().enumerate()),
                text,
                index: 0,
                marked_span: {
                    let start = text[..text.find("encore").unwrap()].chars().count();
                    (start, start + 6)
                },
            })
            .collect();
        let client = crate::base_chat_client("gpt-6-luna")
            .with_cache_directory(concat!(env!("CARGO_MANIFEST_DIR"), "/../.cache"))
            .with_reasoning_effort("medium")
            .with_no_batch();
        let answers =
            crate::readings::batch::<_, Response>(&client, &system_prompt(), &requests, |r| {
                serde_json::to_string(r).unwrap()
            })
            .await?;
        for (request, answer) in &answers {
            println!(
                "{}",
                serde_json::json!({"sentence": request.text, "response": answer.as_ref().ok(), "error": answer.as_ref().err().map(ToString::to_string)})
            );
        }
        assert!(
            matches!(&answers[0].1, Ok(Response { answer: Answer::Ranked { sense_ids } }) if sense_ids.first() == Some(&(row.usages.len() as u32)))
        );
        assert!(answers.iter().all(|(_, answer)| answer.is_ok()));
        Ok(())
    }
}
