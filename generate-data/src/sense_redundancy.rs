//! Learner-facing redundancy groups; sense identities and review scheduling stay intact.

use futures::StreamExt;
use language_utils::{Course, DictionaryEntry, Gram, PhrasebookDefinitionEntry, TaggedGram};
use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroU32,
    sync::LazyLock,
};
use tysm::chat_completions::ChatClient;

use crate::usage_discovery::UsageInventory;

static CHAT_CLIENT: LazyLock<ChatClient> =
    LazyLock::new(|| crate::migrating_chat_client("gpt-5.6-terra"));

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct SenseRedundancyResponse {
    /// Disjoint groups of redundant sense IDs. Omit standalone senses and groups with fewer than two members.
    redundancy_sets: Vec<Vec<u32>>,
}

fn validate_sets(sets: Vec<Vec<u32>>, known: &BTreeSet<u32>) -> Vec<Vec<NonZeroU32>> {
    let mut seen = BTreeSet::new();
    sets.into_iter()
        .filter_map(|set| {
            let members: BTreeSet<_> = set
                .into_iter()
                .filter(|id| known.contains(id) && !seen.contains(id))
                .filter_map(NonZeroU32::new)
                .collect();
            if members.len() < 2 {
                return None;
            }
            seen.extend(members.iter().map(|id| id.get()));
            Some(members.into_iter().collect())
        })
        .collect()
}

pub async fn find_redundant_senses(
    course: Course,
    dictionary: &BTreeMap<TaggedGram<Gram<String>>, DictionaryEntry>,
    phrases: &BTreeMap<TaggedGram<Gram<String>>, PhrasebookDefinitionEntry>,
    inventories: &BTreeMap<Gram<String>, UsageInventory>,
) -> BTreeMap<Gram<String>, Vec<Vec<NonZeroU32>>> {
    let system = format!(
        "You are reviewing a dictionary for someone learning {} whose native language is {}. A word in this dictionary can have several pedagogical senses, each with its own definition in the learner's language. The senses were discovered by clustering the word's occurrences in a sentence corpus, and that process sometimes splits one usage into several on surface features alone: where the word sits in the sentence, whether the sentence is a question or a statement, an exclamation versus a full clause. When that happens the learner sees a page with several senses that read the same to them, which is confusing and wastes their attention.\n\n\
        We would like you to group the senses of one word into sets that are redundant from the learner's point of view, so the dictionary can show one sense per set. The golden rule is that two senses belong in a set when their native-language glosses are the same or nearly the same and they play a very similar grammatical role, so that learning one teaches the other. Senses with genuinely different meanings stay separate, and so does the same meaning in a pattern a learner would have to meet separately: an imperative use, a tag question, a form standing alone as an answer, a term of address. Only sets of two or more senses need reporting; any sense you leave out stands alone. We pick which member to show ourselves, so there is no need to name a preferred one.\n\n\
        A worked example, from French with English glosses: the senses of «donc» glossed 'therefore; so' in mid-clause, 'so; therefore' at the start of a statement, and 'so' opening a question are one set, since they are the same connective in different positions. The senses glossed 'do (for emphasis)' in invitations and 'on earth' in impatient questions stay separate from that set and from each other.",
        course.target_language.prompt_name(),
        course.native_language.prompt_name(),
    );
    let mut groups: BTreeMap<Gram<String>, BTreeMap<u32, String>> = BTreeMap::new();
    let definitions = dictionary
        .iter()
        .map(|(gram, entry)| {
            let d = &entry.definitions[0];
            (
                gram,
                d.native.as_str(),
                d.note.as_deref().unwrap_or(""),
                d.example_sentence_target_language.as_str(),
                d.example_sentence_native_language.as_str(),
            )
        })
        .chain(phrases.iter().map(|(gram, d)| {
            (
                gram,
                d.meaning.as_str(),
                d.additional_notes.as_str(),
                d.target_language_example.as_str(),
                d.native_language_example.as_str(),
            )
        }));
    for (gram, gloss, note, target_example, native_example) in definitions {
        let Some(id) = gram.sense else { continue };
        let usage = &inventories[&gram.gram].usages[id.get() as usize - 1];
        groups.entry(gram.gram.clone()).or_default().insert(id.get(), format!(
            "Sense {}\nkind: {}\ninventory gloss: {}\nnative gloss: {gloss}\nnote: {note}\ntarget example: {target_example}\nnative example: {native_example}",
            id, usage.kind, usage.gloss,
        ));
    }
    let results = futures::stream::iter(groups.into_iter().filter(|(_, senses)| senses.len() >= 2))
        .map(|(gram, senses)| {
            let system = &system;
            async move {
                let prompt = format!(
                    "Gram: {}\n\n{}",
                    gram.to_display_string(course.target_language),
                    senses.values().cloned().collect::<Vec<_>>().join("\n\n")
                );
                let response = CHAT_CLIENT
                    .chat_with_system_prompt::<SenseRedundancyResponse>(system, prompt)
                    .await
                    .inspect_err(|error| {
                        eprintln!(
                            "sense redundancy failed for '{}': {error}",
                            gram.to_display_string(course.target_language)
                        )
                    })
                    .ok()?;
                let sets =
                    validate_sets(response.redundancy_sets, &senses.keys().copied().collect());
                (!sets.is_empty()).then_some((gram, sets))
            }
        })
        .buffer_unordered(32)
        .collect::<Vec<_>>()
        .await;
    let results: BTreeMap<_, _> = results.into_iter().flatten().collect();
    println!(
        "sense redundancy: {} grams with sets; client cost ${:.2}",
        results.len(),
        CHAT_CLIENT.cost().unwrap_or(0.0)
    );
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_and_sorts_disjoint_sets() {
        let known = BTreeSet::from([1, 2, 3, 4, 5, 6]);
        let sets = validate_sets(
            vec![
                vec![3, 1, 3, 0, 99],
                vec![3, 2, 4],
                vec![5],
                vec![5, 6],
                vec![],
            ],
            &known,
        );
        let ids: Vec<Vec<_>> = sets
            .into_iter()
            .map(|s| s.into_iter().map(NonZeroU32::get).collect())
            .collect();
        assert_eq!(ids, vec![vec![1, 3], vec![2, 4], vec![5, 6]]);
    }

    #[test]
    fn drops_empty_unknown_and_duplicate_only_groups() {
        assert!(
            validate_sets(vec![vec![], vec![0, 99], vec![1, 1]], &BTreeSet::from([1])).is_empty()
        );
    }
}
