//! The words a learner meets first get a note written for flashcards when the
//! gloss alone wouldn't teach them: Japanese は glossed "(topic marker)" tells
//! a beginner nothing. A cheap model picks out those meanings; a stronger one
//! rewrites their note for someone brand new to the language. Runs after every
//! stage that puts notes into a prompt, so those prompts and their cache keys
//! stay as they were.

use language_utils::{
    Course, DictionaryEntry, Gram, GramFrequencyEntry, PhrasebookDefinitionEntry, TaggedGram,
};
use std::{
    cmp::Reverse,
    collections::BTreeMap,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
    sync::LazyLock,
};
use tysm::chat_completions::ChatClient;

/// How many of the most frequent grams are checked.
const EARLY_GRAMS: usize = 400;

static FLASH: LazyLock<ChatClient> =
    LazyLock::new(|| gemini("gemini-3.8-flash").with_reasoning_effort("low"));
static ARGON: LazyLock<ChatClient> = LazyLock::new(|| gemini("gemini-4-argon"));

fn gemini(model: &str) -> ChatClient {
    let key = std::env::var("GEMINI_API_KEY").unwrap_or_default();
    crate::apply_cache_only(
        ChatClient::new(key, model)
            .with_url("https://generativelanguage.googleapis.com/v1beta/openai/")
            .with_no_batch()
            .with_max_concurrent_requests(32)
            .with_cache_directory("./.cache"),
    )
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct Verdict {
    reasoning: String,
    needs_explanation: bool,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct Explanation {
    explanation: String,
}

/// One meaning on one card: a dictionary sense or a phrase.
#[derive(Clone, Copy)]
enum Slot<'a> {
    Sense(&'a TaggedGram<Gram<String>>, usize),
    Phrase(&'a TaggedGram<Gram<String>>),
}

pub async fn curate(
    course: Course,
    frequencies: &[GramFrequencyEntry<String>],
    dictionary: &mut BTreeMap<TaggedGram<Gram<String>>, DictionaryEntry>,
    phrasebook: &mut BTreeMap<TaggedGram<Gram<String>>, PhrasebookDefinitionEntry>,
    review_file: &Path,
) -> anyhow::Result<()> {
    let target = course.target_language.prompt_name();
    let native = course.native_language.prompt_name();
    let mut early: Vec<_> = frequencies.iter().collect();
    early.sort_by_key(|entry| Reverse(*entry));
    early.truncate(EARLY_GRAMS);

    // Entries with a hand-curated note are already done.
    let mut slots = Vec::new();
    for entry in early {
        let gram = &entry.gram;
        if let Some(definition) = dictionary.get(gram) {
            if definition
                .definitions
                .iter()
                .all(|sense| !sense.note_is_curated)
            {
                for index in 0..definition.definitions.len() {
                    slots.push((
                        Slot::Sense(gram, index),
                        describe_sense(gram, definition, index),
                    ));
                }
            }
        } else if let Some(phrase) = phrasebook.get(gram)
            && !phrase.note_is_curated
        {
            slots.push((Slot::Phrase(gram), describe_phrase(phrase)));
        }
    }

    let verdicts = FLASH
        .batch_chat_with_system_prompt_fn::<_, _, Verdict>(
            &check_prompt(target, native),
            &slots,
            |(_, prompt)| prompt.clone(),
            |_| {},
        )
        .await?;
    let flagged: Vec<_> = verdicts
        .into_iter()
        .filter_map(|((slot, prompt), verdict)| match verdict {
            Ok(verdict) => verdict.needs_explanation.then(|| {
                (
                    *slot,
                    format!("{prompt}\nwhy it needs explaining: {}", verdict.reasoning),
                )
            }),
            Err(e) => {
                eprintln!("curated note check failed: {e}");
                None
            }
        })
        .collect();
    let explanations = ARGON
        .batch_chat_with_system_prompt_fn::<_, _, Explanation>(
            &explain_prompt(target, native),
            &flagged,
            |(_, prompt)| prompt.clone(),
            |_| {},
        )
        .await?;

    // The packs hold the notes; this file is for reading them.
    let mut review = BufWriter::new(File::create(review_file)?);
    let mut curated = 0;
    for ((slot, _), explanation) in explanations {
        let explanation = match explanation {
            // Cards show plain text, and argon sometimes *emphasizes*.
            Ok(explanation) => explanation.explanation.trim().replace('*', ""),
            Err(e) => {
                eprintln!("curated note explanation failed: {e}");
                continue;
            }
        };
        curated += 1;
        let (gram, gloss, note) = match *slot {
            Slot::Sense(gram, index) => {
                let sense = &mut dictionary.get_mut(gram).unwrap().definitions[index];
                sense.note_is_curated = true;
                (gram, &sense.native, &*sense.note.insert(explanation))
            }
            Slot::Phrase(gram) => {
                let phrase = phrasebook.get_mut(gram).unwrap();
                phrase.note_is_curated = true;
                phrase.additional_notes = explanation;
                (gram, &phrase.meaning, &phrase.additional_notes)
            }
        };
        serde_json::to_writer(
            &mut review,
            &serde_json::json!({
                "gram": gram.gram.to_display_string(course.target_language),
                "gloss": gloss,
                "note": note,
            }),
        )?;
        writeln!(review)?;
    }
    review.flush()?;
    println!("curated notes: {curated} of {} early meanings", slots.len());
    Ok(())
}

fn describe_sense(
    gram: &TaggedGram<Gram<String>>,
    definition: &DictionaryEntry,
    index: usize,
) -> String {
    let sense = &definition.definitions[index];
    let heteronym = gram
        .gram
        .heteronym()
        .expect("dictionary grams are single words");
    let others: Vec<_> = definition
        .definitions
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != index)
        .map(|(_, other)| other.native.as_str())
        .collect();
    let mut prompt = format!(
        "word: {}\npart of speech: {}\ngloss: {}\nnote: {}\nexample: {}\nexample translation: {}",
        heteronym.word,
        heteronym.pos,
        sense.native,
        sense.note.as_deref().unwrap_or(""),
        sense.example_sentence_target_language,
        sense.example_sentence_native_language,
    );
    if !others.is_empty() {
        prompt += &format!(
            "\nother meanings, on their own cards: {}",
            others.join("; ")
        );
    }
    prompt
}

fn describe_phrase(phrase: &PhrasebookDefinitionEntry) -> String {
    format!(
        "phrase: {}\ngloss: {}\nnote: {}\nexample: {}\nexample translation: {}",
        phrase.target_language_multi_word_term,
        phrase.meaning,
        phrase.additional_notes,
        phrase.target_language_example,
        phrase.native_language_example,
    )
}

fn check_prompt(target: &str, native: &str) -> String {
    format!(
        r#"We make flashcards for {native} speakers learning {target}. Each card shows a {target} word or phrase with a short {native} gloss. You'll be given one meaning of one word or phrase, with an example sentence that uses it in that meaning.

Decide whether this meaning is especially tricky for {native} speakers, such that the gloss alone would leave a beginner unsure what it means or how it's used, and some additional explanation of its meaning or usage is warranted. This refers specifically to how it's used in the example sentence, not how it's used in general: its other meanings are treated separately.

Say yes only for the genuinely tricky minority: markers and particles with no {native} counterpart, words used in a way their closest translation would get wrong, politeness and register markers, constructions {native} doesn't have. Say no when the gloss says it all, even for a common or irregular word.

For English speakers, these would warrant an explanation:
- Japanese は marking what the sentence is about
- Korean 에 marking where someone is going
- French "en" meaning "some (of it)", as in "j'en ai"
- Spanish "se" in "aquí se habla español"
- German "doch" contradicting a negative question
- Mandarin 了 marking that something has happened or changed
- Hindi ने marking the subject of a past-tense transitive verb

and these wouldn't: French "chat" (cat), Spanish "comer" (to eat), German "Haus" (house), Japanese 本 (book), Korean 학교 (school).

Give your reasoning first."#
    )
}

fn explain_prompt(target: &str, native: &str) -> String {
    format!(
        r#"We make flashcards for {native} speakers learning {target}. Each card shows a {target} word or phrase with a short {native} gloss. You'll be given one meaning that's tricky for {native} speakers. Write the explanation shown under the gloss, so that a learner who is brand new to {target} understands what it means and how it's used the first time they meet it.

Write two to four plain sentences in {native}, suitable for beginners. Explain from first principles, assuming no knowledge of grammar: say what {target} does here and how that differs from {native}, then give one or two short, real, everyday {target} examples with their meaning. Write the pronunciation in parentheses after examples in a non-Latin script. Avoid grammatical terms like "topic" or "particle", or explain them as you use them.

Explain only this meaning; the word's other meanings have their own cards. The example sentence you're given is for your context only and isn't shown to the learner, so don't refer to it; write out any example you want them to see.

Three explanations for English speakers, to show the kind of thing we want:

Korean 에, "to, at (a place)": Korean shows how a word fits into a sentence by attaching a small ending to it, where English would put a separate word like "to" in front. 에 (e) goes right after a place to mean "to" or "at" that place. 학교 (hakgyo) means "school", so 학교에 가요 (hakgyo-e gayo) means "I'm going to school."

Japanese は, "(topic marker)": Japanese sentences often start by naming what they're about, then say something about it. は goes right after that thing; used this way it's pronounced "wa", even though the same character is read "ha" elsewhere. 私は学生です (watashi wa gakusei desu) is literally "as for me, student", which simply means "I'm a student." English usually leaves it untranslated.

French en, "some (of it)": when French talks about an amount of something already mentioned, it can't just leave it out the way English can; it puts en before the verb. « Tu as du pain ? — Oui, j'en ai. » means "Do you have bread? — Yes, I have some." Without en, the answer would sound unfinished."#
    )
}

