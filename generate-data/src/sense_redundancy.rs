//! Learner-facing redundancy groups across identical written fronts.

use crate::usage_discovery::UsageInventory;
use language_utils::{
    Course, DictionaryEntry, Gram, GramFrequencyEntry, PhrasebookDefinitionEntry, TaggedGram,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::LazyLock,
};
use tysm::chat_completions::ChatClient;

static CHAT_CLIENT: LazyLock<ChatClient> =
    LazyLock::new(|| crate::migrating_chat_client("gpt-5.6-terra"));
type Entry = TaggedGram<Gram<String>>;

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct SenseRedundancyResponse {
    /// Disjoint sets of member indices. Omit standalone members.
    redundancy_sets: Vec<Vec<usize>>,
}

fn validate_sets(sets: Vec<Vec<usize>>, members: &[Entry]) -> Vec<Vec<Entry>> {
    let mut seen = BTreeSet::new();
    sets.into_iter()
        .filter_map(|set| {
            let set: BTreeSet<_> = set
                .into_iter()
                .filter_map(|index| members.get(index).cloned())
                .filter(|member| !seen.contains(member))
                .collect();
            if set.len() < 2 {
                return None;
            }
            seen.extend(set.iter().cloned());
            Some(set.into_iter().collect())
        })
        .collect()
}

fn describe_definitions(definitions: &[language_utils::TargetToNativeWord]) -> Option<String> {
    (!definitions.is_empty()).then(|| {
        definitions
            .iter()
            .map(|definition| {
                format!(
                    "native gloss: {}\nnote: {}\ntarget example: {}\nnative example: {}",
                    definition.native,
                    definition.note.as_deref().unwrap_or(""),
                    definition.example_sentence_target_language,
                    definition.example_sentence_native_language
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    })
}

pub async fn find_redundant_senses(
    course: Course,
    dictionary: &BTreeMap<Entry, DictionaryEntry>,
    phrases: &BTreeMap<Entry, PhrasebookDefinitionEntry>,
    inventories: &BTreeMap<Gram<String>, UsageInventory>,
    frequencies: &[GramFrequencyEntry<String>],
) -> anyhow::Result<BTreeMap<String, Vec<Vec<Entry>>>> {
    let texts = language_utils::word_prefix::prefixed_texts(
        frequencies,
        dictionary,
        phrases,
        course.target_language,
    );
    let system = format!(
        "You are helping design flashcards for someone learning {} whose native language is {}. A flashcard shows one piece of {} text on its front. On the back, it lists every meaning of that text the learner is studying, one row per meaning, and the learner grades each row. Each request gives you one front text and the dictionary entries that produce it, called members. A member may be a single word shown with a short prefix for context (French «est» is shown as «il est»), or a whole phrase.\n\n\
        The member list is noisy. The entries were discovered automatically, so one usage is often split in two on surface features alone: where the word sits in a sentence, whether the sentence is a question, or a part-of-speech or lemma tag that differs while the meaning does not. When that happens the learner sees two rows that read the same to them, and has to grade the same thing twice.\n\n\
        We would like you to group the members whose rows would be redundant. The test is: if the learner already knows one member's row, would another member's row teach them anything new? If not, they belong in the same set. Read each member as it appears on the card, front text included: a verb glossed 'is' on a card whose front reads «il est» means 'he is' there, so it teaches nothing beyond a phrase glossed 'he is'. A member can list several definitions; compare its whole meaning, not one overlapping gloss. A row does teach something new when its meaning differs, or when it is a pattern the learner has to learn on its own, such as an imperative, a tag question, an auxiliary use, a standalone answer, or a term of address.\n\n\
        A worked example, from French with English glosses, on the front «il est»: the phrase 'he is', the verb 'is', and the verb 'is (in a question)' are one set, since a question made by intonation uses the very same verb. The auxiliary 'has' (as in «il est parti», 'he has left') stays separate, because using être to form the past tense is something new to learn. Another, from Japanese: の tagged as an adposition and as a particle, both glossed 'of; 's', are one set.\n\n\
        Report disjoint sets of two or more member numbers; any member you leave out stands alone. We pick which member of a set to show ourselves, so there is no need to name one.",
        course.target_language.prompt_name(),
        course.native_language.prompt_name(),
        course.target_language.prompt_name()
    );
    let mut groups: BTreeMap<String, Vec<(Entry, String)>> = BTreeMap::new();
    for entry in frequencies {
        let gram = &entry.gram;
        let Some(text) = texts.get(&gram.gram) else {
            continue;
        };
        let definition = if let Some(d) = dictionary.get(gram) {
            describe_definitions(&d.definitions)
        } else if let Some(d) = phrases.get(gram) {
            Some(format!(
                "native gloss: {}\nnote: {}\ntarget example: {}\nnative example: {}",
                d.meaning, d.additional_notes, d.target_language_example, d.native_language_example
            ))
        } else {
            continue;
        };
        // Empty lists are legal dictionary data, but supply no meaning to
        // compare. These entries remain in the pack and stand alone.
        let Some(definition) = definition else {
            continue;
        };
        let kind = gram
            .sense
            .and_then(|id| {
                inventories
                    .get(&gram.gram)?
                    .usages
                    .get(id.get() as usize - 1)
            })
            .map(|usage| usage.kind.to_string())
            .unwrap_or_default();
        let pos = gram
            .gram
            .heteronym()
            .map(|h| format!("{:?}", h.pos))
            .unwrap_or_else(|| "Multiword".into());
        let written = gram.gram.to_display_string(course.target_language);
        let shown = if written == *text {
            format!("written: {written}")
        } else {
            format!("written: {written} (shown on the card as «{text}»)")
        };
        let kind = if kind.is_empty() {
            String::new()
        } else {
            format!("usage: {kind}\n")
        };
        let description = format!("{shown}\npart of speech: {pos}\n{kind}{definition}");
        groups
            .entry(text.clone())
            .or_default()
            .push((gram.clone(), description));
    }
    let groups: Vec<_> = groups
        .into_iter()
        .filter(|(_, members)| members.len() >= 2)
        .collect();
    println!("sense redundancy: {} eligible text requests", groups.len());
    let prompts: Vec<_> = groups
        .into_iter()
        .map(|(text, members)| {
            let prompt = format!(
                "Front: «{text}»\n\n{}",
                members
                    .iter()
                    .enumerate()
                    .map(|(i, (_, description))| format!("Member {i}\n{description}"))
                    .collect::<Vec<_>>()
                    .join("\n\n")
            );
            (text, members, prompt)
        })
        .collect();
    let results: BTreeMap<_, _> = CHAT_CLIENT
        .batch_chat_with_system_prompt_fn::<_, _, SenseRedundancyResponse>(
            &system,
            &prompts,
            |(_, _, prompt)| prompt.clone(),
            |_| {},
        )
        .await?
        .into_iter()
        .filter_map(|((text, members, _), response)| {
            let response = response
                .inspect_err(|error| eprintln!("sense redundancy failed for '{text}': {error}"))
                .ok()?;
            let members: Vec<_> = members.iter().map(|(gram, _)| gram.clone()).collect();
            let sets = validate_sets(response.redundancy_sets, &members);
            (!sets.is_empty()).then(|| (text.clone(), sets))
        })
        .collect();
    println!(
        "sense redundancy: {} texts with sets; client cost ${:.2}",
        results.len(),
        CHAT_CLIENT.cost().unwrap_or(0.0)
    );
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn describes_every_definition_and_handles_empty_entries() {
        assert_eq!(describe_definitions(&[]), None);
        let definition = |native: &str| language_utils::TargetToNativeWord {
            native: native.into(),
            note: Some(format!("note for {native}")),
            example_sentence_target_language: format!("example for {native}"),
            example_sentence_native_language: format!("translation for {native}"),
            cognate: false,
            false_cognate: false,
        };
        let description =
            describe_definitions(&[definition("to approach"), definition("to address a topic")])
                .unwrap();
        assert!(description.contains("native gloss: to approach"));
        assert!(description.contains("native gloss: to address a topic"));
        assert!(description.contains("note for to address a topic"));
        assert!(description.contains("translation for to address a topic"));
    }

    #[test]
    fn validates_cross_gram_disjoint_sets() {
        let member = |word: &str| TaggedGram {
            gram: Gram(
                word.split_whitespace()
                    .map(|word| {
                        language_utils::Atom::Tok(language_utils::Word {
                            text: word.into(),
                            word_type: language_utils::WordType::Heteronym(
                                language_utils::Heteronym {
                                    word: word.into(),
                                    lemma: word.into(),
                                    pos: language_utils::PartOfSpeech::Verb,
                                },
                            ),
                        })
                    })
                    .collect(),
            ),
            sense: std::num::NonZeroU32::new(1),
        };
        let a = member("est");
        let b = member("il est");
        let members = vec![a.clone(), b.clone(), a.clone()];
        assert_eq!(
            validate_sets(vec![vec![0, 0, 99, 3], vec![0, 1, 2], vec![1, 2]], &members),
            vec![vec![a, b]]
        );
    }
}
