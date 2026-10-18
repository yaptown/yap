use indicatif::{ProgressBar, ProgressStyle};
use language_utils::{Course, Pronunciations};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::sync::LazyLock;
use tysm::chat_completions::ChatClient;

static CHAT_CLIENT: LazyLock<ChatClient> =
    LazyLock::new(|| crate::migrating_chat_client("gpt-6-luna"));

/// Pure inputs to pronunciation selection, shared by production and measurement.
/// Manual entries never go to the model, and preserve their file order.
pub struct PronunciationCandidates {
    pub automatic: HashMap<String, BTreeSet<String>>,
    pub manual: std::collections::BTreeMap<String, Pronunciations>,
}

pub fn load_candidates(
    source: &std::path::Path,
    frequent_words: &std::collections::HashSet<String>,
) -> anyhow::Result<PronunciationCandidates> {
    use anyhow::Context;
    use std::io::BufRead;
    let read = |name: &str| -> anyhow::Result<Vec<(String, String)>> {
        std::io::BufReader::new(std::fs::File::open(source.join(name))?)
            .lines()
            .filter_map(|line| match line {
                Ok(line) if line.trim().is_empty() => None,
                Ok(line) => Some((|| {
                    let (word, ipa) = line
                        .split_once('\t')
                        .context("invalid pronunciation TSV row")?;
                    Ok((word.trim().to_lowercase(), ipa.trim().to_owned()))
                })()),
                Err(error) => Some(Err(error.into())),
            })
            .filter(|row| match row {
                Ok((word, _)) => frequent_words.contains(word),
                Err(_) => true,
            })
            .collect()
    };
    let mut automatic = HashMap::<String, BTreeSet<String>>::new();
    for (word, ipa) in read("pronunciations.tsv")? {
        automatic.entry(word).or_default().insert(ipa);
    }
    let mut extra = std::collections::BTreeMap::<String, Vec<String>>::new();
    for (word, ipa) in read("extra_pronunciations.tsv")? {
        extra.entry(word).or_default().push(ipa);
    }
    let mut manual = std::collections::BTreeMap::new();
    for (word, values) in extra {
        let mut values = values.into_iter();
        let main = values.next().expect("nonempty manual pronunciations");
        let mut seen = std::collections::HashSet::new();
        let others = values
            .chain(automatic.remove(&word).into_iter().flatten())
            .filter(|ipa| ipa != &main && seen.insert(ipa.clone()))
            .collect();
        manual.insert(word, Pronunciations { main, others });
    }
    Ok(PronunciationCandidates { automatic, manual })
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
struct PronunciationResponse {
    #[serde(rename = "1. thoughts")]
    thoughts: String,
    #[serde(rename = "2. selected_pronunciation")]
    selected_pronunciation: String,
}

/// Select the canonical pronunciation for each word. Unambiguous words are
/// handled locally; ambiguous words are resolved together through the Batch API.
pub async fn select_common_pronunciations(
    course: Course,
    words_with_pronunciations: HashMap<String, BTreeSet<String>>,
) -> anyhow::Result<Vec<(String, Pronunciations)>> {
    let target_language = course.target_language;
    let count = words_with_pronunciations.len();
    let pb = ProgressBar::new(count as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} pronunciations ({per_sec}, ${msg}, {eta})")?
            .progress_chars("#>-"),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    // Keep this prompt byte-for-byte compatible with the historical live-call
    // prompt so existing gpt-5.2/high cache entries remain usable.
    let system_prompt = format!(
        r#"You are analyzing {target_language} word pronunciations to select the most common one for beginner learners.

Given a {target_language} word and its possible IPA pronunciations, select the pronunciation that:
1. Is most commonly used in standard metropolitan {target_language}
2. Would be most appropriate for beginners to learn
3. Represents the most frequent usage in everyday speech

If there are regional or contextual variations, prioritize the standard metropolitan pronunciation unless another variant is overwhelmingly more common. Return the selected pronunciation in the way it is given (retaining spaces as they're used to separate individual IPA characters, and without [] or / / surrounding it).
e
Output format:
{{
    "1. thoughts": "Brief analysis of the pronunciation options",
    "2. selected_pronunciation": "The chosen IPA pronunciation",
}}"#,
        target_language = target_language.prompt_name()
    );

    let mut selected = words_with_pronunciations
        .iter()
        .filter(|(_, pronunciations)| pronunciations.len() == 1)
        .map(|(word, pronunciations)| {
            (
                word.clone(),
                Pronunciations {
                    main: pronunciations.first().expect("one pronunciation").clone(),
                    others: Vec::new(),
                },
            )
        })
        .collect::<Vec<_>>();
    let ambiguous = words_with_pronunciations
        .into_iter()
        .filter(|(_, pronunciations)| pronunciations.len() > 1)
        .collect::<Vec<_>>();

    let responses = CHAT_CLIENT
        .batch_chat_with_system_prompt_fn::<_, _, PronunciationResponse>(
            system_prompt,
            &ambiguous,
            |(word, pronunciations)| {
                format!(
                    "Word: {}\nPronunciations: {}",
                    word,
                    pronunciations
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            },
            |batch| crate::report_batch_progress(&pb, 0, ambiguous.len(), batch),
        )
        .await?;
    for ((word, pronunciations), response) in responses {
        let main = response
            .ok()
            .map(|response| response.selected_pronunciation)
            .and_then(|candidate| {
                pronunciations
                    .iter()
                    .find(|pronunciation| {
                        pronunciation.replace(' ', "") == candidate.replace(' ', "")
                    })
                    .cloned()
            })
            .unwrap_or_else(|| {
                pronunciations
                    .first()
                    .expect("ambiguous pronunciations")
                    .clone()
            });
        let others = pronunciations
            .iter()
            .filter(|pronunciation| **pronunciation != main)
            .cloned()
            .collect();
        selected.push((word.clone(), Pronunciations { main, others }));
    }

    selected.sort_by(|(left, _), (right, _)| left.cmp(right));
    pb.set_position(count as u64);
    pb.finish_with_message(format!("{:.2}", CHAT_CLIENT.cost().unwrap_or(0.0)));
    Ok(selected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_inputs_need_no_dictionary_and_preserve_manual_precedence() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(
            directory.path().join("pronunciations.tsv"),
            " Mot\twiki\nmot\tpreferred\nmot\twiki\nambiguous\tb\nambiguous\ta\nexcluded\tx\n",
        )
        .unwrap();
        std::fs::write(
            directory.path().join("extra_pronunciations.tsv"),
            "Mot\tpreferred\nmot\tother\nmot\tpreferred\n",
        )
        .unwrap();
        let words = std::collections::HashSet::from(["mot".into(), "ambiguous".into()]);
        let candidates = load_candidates(directory.path(), &words).unwrap();
        assert_eq!(candidates.automatic.len(), 1);
        assert_eq!(
            candidates.automatic["ambiguous"],
            BTreeSet::from(["a".into(), "b".into()])
        );
        assert_eq!(candidates.manual["mot"].main, "preferred");
        assert_eq!(candidates.manual["mot"].others, ["other", "wiki"]);
        assert!(!directory.path().join("gram_dictionary.jsonl").exists());
    }
}
