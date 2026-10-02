use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use language_utils::{
    Atom, Course, DictionaryDefinition, Gram, GramFrequencyEntry, Heteronym,
    PhrasebookDefinitionEntry, PhrasebookDefinitionEntryV2, SentenceGram, SentenceGrams,
    TargetToNativeWord, WordType,
};
use rustc_hash::FxHashMap;
use sentence_sampler::sample_to_target;
use std::{collections::BTreeMap, sync::LazyLock};
use tysm::chat_completions::{ChatClient, ChatMessage};

static CHAT_CLIENT_LUNA: LazyLock<ChatClient> =
    LazyLock::new(|| crate::migrating_chat_client("gpt-6-luna"));

static CHAT_CLIENT_TERRA: LazyLock<ChatClient> =
    LazyLock::new(|| crate::migrating_chat_client("gpt-5.6-terra"));

/// Frequent words go to Terra, the rest to Luna.
fn client(frequency: u32, threshold: u32) -> &'static ChatClient {
    if frequency > threshold {
        &CHAT_CLIENT_TERRA
    } else {
        &CHAT_CLIENT_LUNA
    }
}

/// How a course's dictionary-family prompts read.
/// Every course shares one dictionary prompt, naming its variety. The phrasebook and
/// sense prompts keep their historical wording byte for byte for most courses,
/// because the prompt is the tysm cache key; revised courses get the newer wording.
#[derive(Clone, Copy)]
struct DictionaryPolicy {
    name: &'static str,
    /// Guidance on the course's variety, appended to its prompts.
    variety: &'static str,
    /// Whether the phrasebook and sense prompts use the revised wording.
    revised: bool,
}

impl DictionaryPolicy {
    fn for_language(language: language_utils::Language) -> Self {
        use language_utils::Language;
        let historical = Self {
            name: language.prompt_name(),
            variety: "",
            revised: false,
        };
        match language {
            Language::Hindi => Self {
                revised: true,
                ..historical
            },
            Language::PortugueseEuropean => Self {
                name: "European Portuguese (Portugal)",
                variety: " Write examples as people in Portugal speak, e.g. tu and vocês for address and words like cão and autocarro where they fit; words shared with Brazil are welcome too.",
                revised: true,
            },
            Language::SpanishPeninsular => Self {
                name: "Spanish as spoken in Spain",
                variety: " Write examples as people in Spain speak, e.g. vosotros and words like coche and ordenador where they fit; words shared with Latin America are welcome too.",
                revised: true,
            },
            _ => historical,
        }
    }

    /// The phrasebook prompt's sentences on what the "meanings" field holds.
    fn meanings_rule(&self) -> &'static str {
        if self.revised {
            r#"Put grammatical notes and other context in the "additional_notes" field and keep "meanings" to the raw translation, except for a grammatical marker whose closest word would teach the wrong construction: give a short functional label for it instead, e.g. "(marks the subject of a past-tense transitive verb)"."#
        } else {
            r#"And don't include any parentheticals or other notes in the "meanings" field. Any grammatical notes, parentheticals, or other notes belong in the "additional_notes" field, not the "meanings". You can always provide additional context about things like context and gender and other notes about how the term is used in the "additional_notes" field. But what belongs in the "meanings" field is just the raw textual translation / meaning."#
        }
    }

    /// Appended to revised phrasebook and sense prompts; empty for historical ones.
    fn example_guidance(&self) -> String {
        if self.revised {
            format!(
                "\n\nExample sentences are a beginner's first model of the word, so make them grammatical, everyday and neutral in tone. Use the word naturally, in an inflected or contracted form where grammar calls for it, rather than forcing the exact spelling into an ungrammatical sentence.{}",
                self.variety
            )
        } else {
            String::new()
        }
    }
}

pub fn normalize_english_gloss(text: &mut String, native: language_utils::Language) {
    if native != language_utils::Language::English {
        return;
    }
    static PRONOUN: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"\bi\b(?:['’](?:m|ve|d|ll)\b)?").unwrap());
    *text = PRONOUN
        .replace_all(text, |caps: &regex::Captures<'_>| {
            format!("I{}", &caps[0][1..])
        })
        .into_owned();
}

fn normalize_definition(entry: &mut TargetToNativeWord, native: language_utils::Language) {
    normalize_english_gloss(&mut entry.native, native);
    normalize_english_gloss(&mut entry.example_sentence_native_language, native);
}

fn normalize_phrase(entry: &mut PhrasebookDefinitionEntry, native: language_utils::Language) {
    normalize_english_gloss(&mut entry.meaning, native);
    normalize_english_gloss(&mut entry.native_language_example, native);
}

async fn generate_dictionary_group(
    client: &ChatClient,
    entries: &[(Heteronym<String>, u32)],
    samples: &BTreeMap<Heteronym<String>, Vec<String>>,
    system_prompt: &str,
    policy: DictionaryPolicy,
    progress: &ProgressBar,
    progress_offset: u64,
) -> anyhow::Result<Vec<(Heteronym<String>, DictionaryDefinition)>> {
    let initial = client
        .batch_chat_with_system_prompt_fn::<_, _, DictionaryDefinition>(
            system_prompt,
            entries,
            |(heteronym, _)| {
                let sentences = samples[heteronym]
                    .iter()
                    .enumerate()
                    .map(|(i, sentence)| format!("{}. {sentence}", i + 1))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!(
                    "word: `{word}`\nlemma: `{lemma}`\npos: {pos}\nsentences from the corpus:\n{sentences}",
                    word = heteronym.word,
                    lemma = heteronym.lemma,
                    pos = heteronym.pos
                )
            },
            |batch| crate::report_batch_progress(progress, progress_offset, entries.len(), batch),
        )
        .await?;

    let mut accepted = Vec::new();
    let mut retries = Vec::new();
    for ((heteronym, _), response) in initial {
        let Ok(response) = response else { continue };
        let bad_examples = response
            .definitions
            .iter()
            .filter(|definition| {
                !definition
                    .example_sentence_target_language
                    .to_lowercase()
                    .contains(&heteronym.word.to_lowercase())
            })
            .map(|definition| definition.example_sentence_target_language.clone())
            .collect::<Vec<_>>();
        if bad_examples.is_empty() {
            accepted.push((heteronym.clone(), response));
        } else {
            retries.push((heteronym.clone(), response, bad_examples));
        }
    }

    let retried = client
        .batch_chat_with_messages_fn::<_, DictionaryDefinition>(&retries, |(heteronym, response, bad)| {
            let previous_json = serde_json::to_string(response).unwrap_or_default();
            vec![
                ChatMessage::system(system_prompt),
                ChatMessage::user(format!(
                    "You wrote this dictionary entry for the {target_language} word `{word}`:\n\n{previous_json}\n\nThese example sentences don't contain the exact form `{word}`: {bad}\n\nPlease write the entry again in the same format, with every example_sentence_target_language using the exact form `{word}`.",
                    target_language = policy.name,
                    word = heteronym.word,
                    bad = bad.join("; "),
                )),
            ]
        }, |_| {})
        .await?;
    accepted.extend(
        retried
            .into_iter()
            .filter_map(|((heteronym, _, _), response)| {
                response.ok().map(|response| (heteronym.clone(), response))
            }),
    );
    Ok(accepted)
}

/// `heteronym_sentences` is a persistent cache of heteronym -> corpus sentences,
/// revalidated against the current course corpus, so a word's prompt (and so its
/// cached entry) only changes when one of its sentences leaves the corpus.
pub async fn create_gram_dictionary(
    course: Course,
    gram_frequencies: &[GramFrequencyEntry<String>],
    encoded_sentences: &[(
        String,
        SentenceGrams<language_utils::TaggedGram<Gram<String>>>,
    )],
    heteronym_sentences: &mut BTreeMap<Heteronym<String>, Vec<String>>,
) -> anyhow::Result<BTreeMap<Heteronym<String>, DictionaryDefinition>> {
    let target_language_heteronyms = extract_single_atom_heteronyms(gram_frequencies);
    let Course {
        native_language,
        target_language,
    } = course;

    // Every single-atom gram of a heteronym (each sense, each capitalization)
    // witnesses it, so the pool covers all the ways the word is used.
    let gram_to_sentences = build_gram_to_sentences_index(encoded_sentences);
    let mut pools: BTreeMap<&Heteronym<String>, Vec<&str>> = BTreeMap::new();
    for entry in gram_frequencies {
        if let [Atom::Tok(word)] = entry.gram.gram.0.as_slice()
            && let WordType::Heteronym(heteronym) = &word.word_type
            && let Some(sentences) = gram_to_sentences.get(&entry.gram.gram)
        {
            pools.entry(heteronym).or_default().extend(sentences);
        }
    }
    for heteronym in target_language_heteronyms.keys() {
        let mut pool = pools.remove(heteronym).unwrap_or_default();
        pool.sort_unstable();
        pool.dedup();
        revalidate_samples(
            heteronym_sentences.entry(heteronym.clone()).or_default(),
            &pool,
        );
    }

    let count = target_language_heteronyms.len();

    let pb = ProgressBar::new(count as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} gram dictionary entries ({per_sec}, ${msg}, {eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let policy = DictionaryPolicy::for_language(target_language);
    let system_prompt = format!(
        r#"The input is a {target_language} word with its lemma, its part of speech, and up to five sentences from the course's corpus that use it (mostly film and TV subtitles). Write a dictionary entry for it, for an app that teaches beginner {target_language} learners whose native language is {native_language}. The schema first asks you to write the {target_language} word, then a list of one or more {native_language} definitions, each a JSON object with these fields:

- "native" (string): the closest {native_language} word or short phrase, as a bilingual dictionary gives it. Put very similar meanings in one string, separated by commas (e.g. "this, that"). For a verb, translate this form only; the app shows conjugation separately. Capitalize only where {native_language} spelling requires it (e.g. English "I", proper nouns, German nouns). For a grammatical particle or marker whose closest {native_language} word would teach the wrong construction (Hindi ने is not "by"), give a short functional label instead, e.g. "(marks the subject of a past-tense transitive verb)".
- "note" (string, optional): usage the learner needs that the other fields don't already convey, such as register (that tu is informal, or that a word is rude, archaic, or colloquial) or that a spelling is nonstandard. Leave it out otherwise.
- "example_sentence_target_language" (string): a short, natural sentence of your own that uses this exact form of the word in this meaning. It's the learner's first model of the word, so make it grammatical, everyday, and neutral in tone.
- "example_sentence_native_language" (string): a natural {native_language} translation of that sentence.
- "cognate" (bool): whether the word looks like a {native_language} word with this meaning. "avocat" is a cognate for "avocado", but not for "lawyer".
- "false_cognate" (bool): whether the word looks like a different {native_language} word and could easily be confused with it, like French "actuellement", which does not mean "actually".

Use the corpus sentences to decide which meanings to give and in what order. The learner will meet the word in sentences like these, so first give the meaning most of them use, then any other meaning a beginner will run into regularly. For example, Korean 씨 in film dialogue is almost always the polite suffix after a name ("Mr./Ms."), so that comes before "seed". The sentences are a small sample, so weigh them together with what you know of everyday usage, and leave out rare or obscure meanings, which only confuse beginners.

Give several definitions only when the word has truly different meanings, like French "avocat" ("lawyer" and "avocado"). When the part of speech settles which meaning applies, give only that one: French "fait" as a past participle is "done", not "fact". Define exactly the given form, not related forms or other spellings.

The word may be inflected. Translate the form itself, without adding a subject: Italian "è" is "is", not "he is", because the app shows the form in context. Likewise, French "est" given as an auxiliary is "has", and its example sentence should use it as an auxiliary.

Leave pronunciation, IPA, part of speech, gender, and conjugation out of the entry, unless a note truly needs them.

Write the notes in {native_language}.{variety}"#,
        native_language = native_language.prompt_name(),
        target_language = policy.name,
        variety = policy.variety,
    );
    let (terra, luna): (Vec<_>, Vec<_>) = target_language_heteronyms
        .into_iter()
        .partition(|(_, frequency)| *frequency > 500);
    let mut entries = generate_dictionary_group(
        &CHAT_CLIENT_TERRA,
        &terra,
        heteronym_sentences,
        &system_prompt,
        policy,
        &pb,
        0,
    )
    .await?;
    entries.extend(
        generate_dictionary_group(
            &CHAT_CLIENT_LUNA,
            &luna,
            heteronym_sentences,
            &system_prompt,
            policy,
            &pb,
            terra.len() as u64,
        )
        .await?,
    );
    pb.set_position(count as u64);
    pb.finish_with_message(format!(
        "{:.2}",
        CHAT_CLIENT_TERRA.cost().unwrap_or(0.0) + CHAT_CLIENT_LUNA.cost().unwrap_or(0.0)
    ));
    for (_, entry) in &mut entries {
        for definition in &mut entry.definitions {
            normalize_definition(definition, native_language);
        }
    }
    Ok(entries.into_iter().collect())
}

/// Deduplicate single-atom heteronyms, keeping their maximum frequency.
fn extract_single_atom_heteronyms(
    gram_frequencies: &[GramFrequencyEntry<String>],
) -> BTreeMap<Heteronym<String>, u32> {
    let mut heteronym_frequencies: BTreeMap<Heteronym<String>, u32> = BTreeMap::new();

    for entry in gram_frequencies {
        if entry.gram.sense.is_some() {
            continue;
        }
        let gram = &entry.gram.gram;

        if gram.len() != 1 {
            continue;
        }

        if let Some(Atom::Tok(word)) = gram.first()
            && let WordType::Heteronym(heteronym) = &word.word_type
        {
            heteronym_frequencies
                .entry(heteronym.clone())
                .and_modify(|freq| *freq = (*freq).max(entry.count))
                .or_insert(entry.count);
        }
    }

    heteronym_frequencies
}

fn build_gram_to_sentences_index<'a>(
    encoded_sentences: &'a [(
        String,
        SentenceGrams<language_utils::TaggedGram<Gram<String>>>,
    )],
) -> FxHashMap<&'a Gram<String>, Vec<&'a str>> {
    let mut index: FxHashMap<&'a Gram<String>, Vec<&'a str>> = FxHashMap::default();

    for (sentence_text, sentence_grams) in encoded_sentences {
        for gram in &sentence_grams.grams {
            let gram_ref = match gram {
                SentenceGram::Learnable(g) | SentenceGram::Obvious(g) => g,
            };
            index
                .entry(&gram_ref.gram)
                .or_default()
                .push(sentence_text.as_str());
        }
        // A high-confidence match witnesses its gram as well as the encoded
        // stream does — and for a citation gram, whose matches are variant
        // occurrences rewritten to it (`pipeline::apply_citations`), matches
        // are the only witnesses: the citation form itself rarely occurs
        // literally, so without these its definition would be generated with
        // no example sentences at all.
        for term in &sentence_grams.multiword_terms {
            index
                .entry(&term.gram.gram)
                .or_default()
                .push(sentence_text.as_str());
        }
    }
    // A sentence can witness the same gram twice (encoded + match, or two
    // match positions); pushes for one sentence are adjacent, so `dedup`
    // suffices to keep the example pool duplicate-free.
    for sentences in index.values_mut() {
        sentences.dedup();
    }

    index
}

fn revalidate_samples(samples: &mut Vec<String>, pool: &[&str]) {
    // A valid nonempty selection is part of the prompt cache key; leave it alone.
    if !samples.is_empty() && samples.iter().all(|s| pool.contains(&s.as_str())) {
        return;
    }
    samples.retain(|s| pool.contains(&s.as_str()));
    let candidates: Vec<_> = pool
        .iter()
        .copied()
        .filter(|s| !samples.iter().any(|old| old == s))
        .collect();
    samples.extend(
        sample_to_target(
            candidates,
            5usize.saturating_sub(samples.len()),
            |s: &&str| *s,
        )
        .into_iter()
        .map(str::to_owned),
    );
}

/// `gram_sentences` is a persistent cache of gram -> example sentences, revalidated
/// against the current course corpus before it is used in prompts.
pub async fn create_gram_phrasebook(
    course: Course,
    gram_frequencies: &[GramFrequencyEntry<String>],
    encoded_sentences: &[(
        String,
        SentenceGrams<language_utils::TaggedGram<Gram<String>>>,
    )],
    gram_sentences: &mut BTreeMap<Gram<String>, Vec<String>>,
) -> anyhow::Result<Vec<(Gram<String>, PhrasebookDefinitionEntry)>> {
    let Course {
        native_language,
        target_language,
        ..
    } = course;

    let gram_to_sentences = build_gram_to_sentences_index(encoded_sentences);

    let mut multi_atom_grams: BTreeMap<Gram<String>, u32> = BTreeMap::new();
    for entry in gram_frequencies {
        if entry.gram.sense.is_some() {
            continue;
        }
        let gram = &entry.gram.gram;
        if gram.len() > 1 {
            multi_atom_grams.entry(gram.clone()).or_insert(entry.count);
        }
    }

    for gram in multi_atom_grams.keys() {
        let pool = gram_to_sentences.get(gram).cloned().unwrap_or_default();
        let samples = gram_sentences.entry(gram.clone()).or_default();
        revalidate_samples(samples, &pool);
    }

    let count = multi_atom_grams.len();

    let pb = ProgressBar::new(count as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} gram phrasebook entries ({per_sec}, ${msg}, {eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let mut phrasebook = futures::stream::iter(multi_atom_grams.iter())
        .map(|(gram, &freq)| {
            let pb = pb.clone();
            let gram_text = gram.to_display_string(target_language);
            let cost = CHAT_CLIENT_TERRA.cost().unwrap_or(0.0)
                + CHAT_CLIENT_LUNA.cost().unwrap_or(0.0);
            pb.set_message(format!("{cost:.2} ({gram_text})"));

            let example_sentences = gram_sentences
                .get(gram)
                .cloned()
                .unwrap_or_default();

            async move {
                let examples_text = if example_sentences.is_empty() {
                    String::from("(No example sentences available)")
                } else {
                    example_sentences
                        .iter()
                        .enumerate()
                        .map(|(i, s)| format!("{}. {}", i + 1, s))
                        .collect::<Vec<_>>()
                        .join("\n")
                };

                let policy = DictionaryPolicy::for_language(target_language);
                let chat_client = client(freq, 250);

                // kind of ugly, but the old system prompt is bad, but it's too expensive to regenerate all of them so i'll just do it for the most important words
                let system_prompt= format!(r#"The input is a {target_language} multi-word term along with example sentences showing its usage. Generate a phrasebook entry for it, to be used in an app for beginner {target_language} learners (whose native language is {native_language}).

Think about the word and its meaning based on how it's used in the example sentences, and what is likely to be relevant to a beginner learner. Your thoughts will not be shown to the user. Then, write the word, then provide the meaning as the closest {native_language} equivalent word or short phrase — just like a dictionary translation (e.g. "just did", "what" or "that which", "as soon as"). Skip any preamble like "the {target_language} term [term] is often used to indicate that...", or "a question phrase equivalent to..." and just give the {native_language} equivalent. {meanings_rule}

Next, provide your own example of the term's usage in a natural sentence.

If the term is informal/slang, you can also set the "informal" field to true. For example, the english phrase "kick the bucket" is informal. If the term makes sense from the component words, you can also set the "compositional" field to true. (E.g. "Être sur son 31" makes no sense from its individual words, nor does "Altes Haus" in german. But "C'est" does make sense from its individual words) More examples: "pass away" is non-compositional (it's a phrasal verb whose meaning isn't obvious from "pass" + "away") but not informal. And "it is what it is" is informal but compositional.

"Cognate" and "false_cognate" are boolean fields that indicate whether the {target_language} word is a cognate or false cognate in {native_language}.

For our purposes, a phrase is a cognate to a definition if it looks similar to the {native_language} phrase. So "carte de crédit" is a cognate for "credit card", and "en général" is a cognate for "in general".
And a phrase is a false cognate to a definition if it looks similar to a different {native_language} phrase and might be easily confused, a classic example being the spanish phrase "en absoluto" which looks like "absolutely" but means "not at all". (Another example is French "passer un examen" which looks like "pass an exam" but actually means "take an exam" with no implication of success).

Lastly, "can_be_translated_literally" should be set to true if the phrase can be translated literally into the {native_language} phrase. For example, "carte de crédit" can be translated literally into "credit card", and "en général" can be translated literally into "in general".

More french/english examples:
"ce que" → not a cognate, but can be translated literally ("that which")
"avoir lieu" → not a cognate, cannot be translated literally ("to have place" ≠ "to take place")

Example:
Input: multiword term: `ce que`
Example sentences:
1. Dis-moi ce que tu veux.
2. Je ne sais pas ce que tu veux dire.

(Here, both sentences use `ce que` to mean `what` in English, so you can just say ["what"] for the `meanings` field.)

Output: {{
    "target_language_multi_word_term":"ce que",
    "meanings": ["what"], // this field should be super concise - ideally include 1 string, maybe 2 strings of the most common meanings, no extra notes or context. If there are multiple meanings, include them in order of frequency. You might be able to get a hint from the example sentences. 
    "additional_notes": "Refers to something previously mentioned or understood from context.",
    "target_language_example":"C'est ce que je pensais.",
    "native_language_example":"That's what I thought.",
    "informal": false,
    "compositional": true,
    "cognate": false,
    "false_cognate": false,
    "can_be_translated_literally": true
}}

Don't capitalize the first letter of the meaning unless it makes sense (e.g. english proper nouns, german nouns, etc). Do not add pronunciation, IPA, part of speech, gender, or conjugation info unless it's in the "additional_notes" field and truly necessary.

Of course, their native language is {native_language}, so you should write the meaning and additional notes in {native_language}.{example_guidance}"#,
        native_language = native_language.prompt_name(),
        target_language = policy.name,
        meanings_rule = policy.meanings_rule(),
        example_guidance = policy.example_guidance());

                let response: Result<PhrasebookDefinitionEntry, _> = if freq > 100  {
                    let response: Result<PhrasebookDefinitionEntryV2, _> = chat_client
                        .chat_with_system_prompt(
                            system_prompt,
                            format!(
                                "multiword term: `{gram_text}`\n\nExample sentences:\n{examples_text}"
                            ),
                        )
                        .await
                        .inspect_err(|e| {
                            println!("error: {e:#?}");
                        });
                    response.map(|entry| PhrasebookDefinitionEntry {
                        target_language_multi_word_term: entry.target_language_multi_word_term,
                        meaning: entry.meanings.first().cloned().unwrap_or_default(),
                        additional_notes: entry.additional_notes,
                        target_language_example: entry.target_language_example,
                        native_language_example: entry.native_language_example,
                        informal: entry.informal,
                        compositional: entry.compositional,
                        cognate: entry.cognate,
                        false_cognate: entry.false_cognate,
                        can_be_translated_literally: entry.can_be_translated_literally,
                    })
                } else {
                    chat_client
                        .chat_with_system_prompt(
                            system_prompt,
                            format!(
                                "multiword term: `{gram_text}`\n\nExample sentences:\n{examples_text}"
                            ),
                        )
                        .await
                        .inspect_err(|e| {
                            println!("error: {e:#?}");
                        })
                };

                let response = if let Ok(ref resp) = response {
                    if !policy.revised && !resp.target_language_example.to_lowercase().contains(&gram_text.to_lowercase()) {
                        let previous_json = serde_json::to_string(resp).unwrap_or_default();
                        chat_client.chat_with_messages(vec![
                            ChatMessage::system(format!("You are a {target_language} phrasebook entry generator for {native_language} speakers.", native_language = native_language.prompt_name(), target_language = target_language.prompt_name())),
                            ChatMessage::user(format!(
                                "I asked you to generate a phrasebook entry for the {target_language} multi-word term `{gram_text}`, and you gave me this response:\n\n{previous_json}\n\nHowever, your target_language_example `{example}` does not contain the exact term `{gram_text}`. Please regenerate the entire response with the same format, making sure target_language_example contains the exact term `{gram_text}`. Here are some example sentences for inspiration that use the term:\n{examples_text}", target_language = target_language.prompt_name(),
                                example = resp.target_language_example,
                            )),
                        ]).await.inspect_err(|e| {
                            println!("retry error: {e:#?}");
                        })
                    } else {
                        response
                    }
                } else {
                    response
                };

                pb.inc(1);

                (response, gram)
            }
        })
        .buffer_unordered(15)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .filter_map(|(response, gram)| response.ok().map(|entry| (gram.clone(), entry)))
        .collect::<Vec<_>>();

    phrasebook.sort_by(|(a, _), (b, _)| a.cmp(b));

    pb.finish_with_message(format!(
        "{:.2}",
        CHAT_CLIENT_TERRA.cost().unwrap_or(0.0) + CHAT_CLIENT_LUNA.cost().unwrap_or(0.0)
    ));

    for (_, entry) in &mut phrasebook {
        normalize_phrase(entry, native_language);
    }
    Ok(phrasebook)
}

/// Generate only the inventoried sense; the existing untagged prompts and
/// cache keys remain unchanged.
pub async fn create_sense_definitions(
    course: Course,
    frequencies: &[GramFrequencyEntry<String>],
    inventories: &BTreeMap<Gram<String>, crate::usage_discovery::UsageInventory>,
) -> anyhow::Result<(
    BTreeMap<language_utils::TaggedGram<Gram<String>>, TargetToNativeWord>,
    BTreeMap<language_utils::TaggedGram<Gram<String>>, PhrasebookDefinitionEntry>,
)> {
    let policy = DictionaryPolicy::for_language(course.target_language);
    let system = format!(
        "Generate a dictionary or phrasebook entry for a beginner learning {} whose native language is {}. The input identifies one pedagogical sense (a meaning, construction, or conversational formula) and gives examples of that sense. Describe this sense only, not other meanings of the same spelling. Respect the supplied surface form, lemma, and part of speech. Write one concise native-language equivalent and note, and one example sentence that illustrates this sense and uses the supplied surface form. Follow the response schema. Morphology, etymology, and pronunciation are supplied separately.{}",
        policy.name,
        course.native_language.prompt_name(),
        policy.example_guidance(),
    );
    let entries = futures::stream::iter(
        frequencies
            .iter()
            .filter(|entry| entry.gram.sense.is_some()),
    )
    .map(|entry| {
        let system = &system;
        async move {
            let gram = &entry.gram;
            let usage = &inventories[&gram.gram].usages[gram.sense.unwrap().get() as usize - 1];
            let mut anchors: Vec<_> = usage.anchors.iter().collect();
            anchors.sort_by_key(|anchor| !anchor.gold);
            let examples = anchors
                .iter()
                .take(5)
                .map(|anchor| anchor.sentence.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            let identity = if let Some(h) = gram.gram.heteronym() {
                format!("word: {}\nlemma: {}\npos: {}", h.word, h.lemma, h.pos)
            } else {
                format!(
                    "phrase: {}",
                    gram.gram.to_display_string(course.target_language)
                )
            };
            let prompt = format!(
                "{identity}\nkind: {}\ngloss: {}\nAnchor sentences:\n{examples}",
                usage.kind, usage.gloss
            );
            let threshold = if gram.gram.len() == 1 { 500 } else { 250 };
            let client = client(entry.count, threshold);
            // A failed sense (or a cache miss in cache-only mode) is skipped
            // like every other definition stage, not fatal to the run.
            let result = if gram.gram.len() == 1 {
                client
                    .chat_with_system_prompt::<TargetToNativeWord>(system, prompt)
                    .await
                    .map(|definition| (gram.clone(), Some(definition), None))
            } else {
                client
                    .chat_with_system_prompt::<PhrasebookDefinitionEntry>(system, prompt)
                    .await
                    .map(|definition| (gram.clone(), None, Some(definition)))
            };
            result
                .inspect_err(|e| {
                    eprintln!(
                        "sense definition failed for '{}' sense {}: {e}",
                        gram.gram.to_display_string(course.target_language),
                        gram.sense.unwrap()
                    )
                })
                .ok()
        }
    })
    .buffer_unordered(32)
    .collect::<Vec<Option<_>>>()
    .await;
    let mut dictionary = BTreeMap::new();
    let mut phrasebook = BTreeMap::new();
    for (gram, definition, phrase) in entries.into_iter().flatten() {
        if let Some(mut definition) = definition {
            normalize_definition(&mut definition, course.native_language);
            dictionary.insert(gram.clone(), definition);
        }
        if let Some(mut phrase) = phrase {
            normalize_phrase(&mut phrase, course.native_language);
            phrasebook.insert(gram, phrase);
        }
    }
    Ok((dictionary, phrasebook))
}

#[cfg(test)]
mod tests {
    use super::*;
    use language_utils::Language;

    #[test]
    fn prompt_revision_is_scoped_and_legacy_requests_are_unchanged() {
        for course in language_utils::COURSES {
            let language = course.target_language;
            let policy = DictionaryPolicy::for_language(language);
            let revised = matches!(
                language,
                Language::Hindi | Language::PortugueseEuropean | Language::SpanishPeninsular
            );
            assert_eq!(policy.revised, revised);
            if !revised {
                assert_eq!(policy.name, language.prompt_name());
                assert_eq!(policy.example_guidance(), "");
            }
        }
    }

    #[test]
    fn samples_retain_order_and_replace_only_stale_members() {
        let mut samples = vec!["current b".into(), "current a".into()];
        let pool = ["current a", "current b", "new"];
        revalidate_samples(&mut samples, &pool);
        assert_eq!(samples, ["current b", "current a"]);
        samples.push("Brazilian sentence removed by dialect filter".into());
        revalidate_samples(&mut samples, &pool);
        assert_eq!(samples, ["current b", "current a", "new"]);
        revalidate_samples(&mut samples, &[]);
        assert!(samples.is_empty());
    }

    #[test]
    fn english_pronoun_capitalization_is_mechanical_and_language_scoped() {
        let mut text =
            "i was, i'm, i've, i'd, i'll; i’m i’ve i’d i’ll; iris, taxi, I am".to_owned();
        normalize_english_gloss(&mut text, Language::English);
        assert_eq!(
            text,
            "I was, I'm, I've, I'd, I'll; I’m I’ve I’d I’ll; iris, taxi, I am"
        );
        let mut italian = "i ragazzi".to_owned();
        normalize_english_gloss(&mut italian, Language::Italian);
        assert_eq!(italian, "i ragazzi");
    }
}
