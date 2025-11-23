use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use language_utils::{Course, DictionaryEntryThoughts, Heteronym, PhrasebookEntryThoughts};
use std::{collections::BTreeMap, sync::LazyLock};
use tysm::chat_completions::ChatClient;

static CHAT_CLIENT_4O: LazyLock<ChatClient> = LazyLock::new(|| {
    ChatClient::from_env("gpt-4o")
        .unwrap()
        .with_cache_directory("./.cache")
});

static CHAT_CLIENT_O3: LazyLock<ChatClient> = LazyLock::new(|| {
    ChatClient::from_env("o3")
        .unwrap()
        .with_cache_directory("./.cache")

    //.with_service_tier("flex")
});

pub async fn create_phrasebook(
    course: Course,
    frequencies: &Vec<language_utils::FrequencyEntry<String>>,
) -> anyhow::Result<Vec<(String, PhrasebookEntryThoughts)>> {
    let Course {
        native_language,
        target_language,
        ..
    } = course;

    let mut target_language_multi_word_terms: BTreeMap<String, u32> = BTreeMap::new();
    for entry in frequencies {
        if let Some(multiword_term) = entry.lexeme.multiword() {
            target_language_multi_word_terms
                .entry(multiword_term.clone())
                .or_insert(entry.count);
        }
    }

    let count = target_language_multi_word_terms.len();

    let pb = ProgressBar::new(count as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} phrasebook entries ({per_sec}, ${msg}, {eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let phrasebook = futures::stream::iter(target_language_multi_word_terms.iter()).map(|(multiword_term, &freq)| {
        let pb = pb.clone();
        let cost = CHAT_CLIENT_O3.cost().unwrap_or(0.0) + CHAT_CLIENT_4O.cost().unwrap_or(0.0);
        pb.set_message(format!("{cost:.2} ({multiword_term})"));
        async move {
        let chat_client = if freq > 500 { &*CHAT_CLIENT_O3 } else { &*CHAT_CLIENT_4O };
        let response: Result<PhrasebookEntryThoughts, _> = chat_client.chat_with_system_prompt(
            format!(r#"The input is a {target_language} multi-word term. Generate a phrasebook entry for it, to be used in an app for beginner {target_language} learners (whose native language is {native_language}). First, think about the word and its meaning, and what is likely to be relevant to a beginner learner. Your thoughts will not be shown to the user. Then, write the word, then provide the meaning in a concise way. (Skip any preamble like "the {target_language} term [term] is often used to indicate that...", or "a question phrase equivalent to..." and just get straight to the meaning.) Then, provide additional context for how the term is used in the "additional_notes" field. Finally, provide an example of the term's usage in a natural sentence.

Example:
Input: multiword term: `ce que`
Output: {{
    "thoughts":"'Ce que' is a common French phrase often used to introduce indirect questions or relative clauses.",
    "target_language_multi_word_term":"ce que",
    "meaning":"'what' or 'that which'.", // this field should be super concise
    "additional_notes": "Refers to something previously mentioned or understood from context.",
    "target_language_example":"Dis-moi ce que tu veux.",
    "native_language_example":"Tell me what you want."
}}

Of course, their native language is {native_language}, so you should write the meaning and additional notes in {native_language}.
            "#),
            format!("multiword term: `{multiword_term}`"),
        ).await.inspect_err(|e| {
            println!("error: {e:#?}");
        });

        pb.inc(1);

        (response, multiword_term)
    }})
    .buffer_unordered(50)
    .collect::<Vec<_>>()
    .await
    .into_iter()
    .filter_map(|(response, multiword_term)| {
        response.ok().map(|entry| (multiword_term.clone(), entry))
    })
    .collect::<Vec<_>>();

    pb.finish_with_message(format!(
        "{:.2}",
        CHAT_CLIENT_O3.cost().unwrap_or(0.0) + CHAT_CLIENT_4O.cost().unwrap_or(0.0)
    ));

    Ok(phrasebook)
}

pub async fn create_dictionary(
    course: Course,
    frequencies: &Vec<language_utils::FrequencyEntry<String>>,
) -> anyhow::Result<BTreeMap<Heteronym<String>, DictionaryEntryThoughts>> {
    let Course {
        native_language,
        target_language,
    } = course;
    // Process sentences to get unique words and track occurrences
    let mut target_language_heteronyms = BTreeMap::new();
    for entry in frequencies {
        if let Some(heteronym) = entry.lexeme.heteronym() {
            target_language_heteronyms
                .entry(heteronym.clone())
                .or_insert(entry.count);
        }
    }

    let count = target_language_heteronyms.len();

    let pb = ProgressBar::new(count as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} dictionary entries ({per_sec}, ${msg}, {eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let dictionary = futures::stream::iter(target_language_heteronyms.iter()).map(async |(heteronym, &freq)| {
        let cost = CHAT_CLIENT_O3.cost().unwrap_or(0.0) + CHAT_CLIENT_4O.cost().unwrap_or(0.0);
        pb.set_message(format!("{cost:.2} ({},{},{})", heteronym.word, heteronym.lemma, heteronym.pos));

        let chat_client = if freq > 500 { &*CHAT_CLIENT_O3 } else { &*CHAT_CLIENT_4O };

        let dict_response = {
            let response: Result<DictionaryEntryThoughts, _> = chat_client.chat_with_system_prompt(
            format!(r#"The input is a {target_language} word, along with its morphological information. Generate a dictionary entry for it, to be used in an app for beginner {target_language} learners (whose native language is {native_language}). (First, think about the word and its meaning, and what is likely to be relevant to a beginner learner.) First, write the word, then provide a list of one or more definitions. Each definition should be a JSON object with the following fields:

- "native" (string): The {native_language} translation(s) of the word. If a word has multiple very similar meanings (e.g. "this" and "that"), include them in the same string separated by commas. (If it's a verb, you don't have to include the infinitive form or information about conjugation - that will be displayed separately in the app.)
- "note" (string, optional): Use only for extra info about usage that is *not already implied* by the other fields. (For example, you can note that "tu" is informal, or that "on" often means "we" in speech.)
- "example_sentence_target_language" (string): A natural example sentence using the word in {target_language}. (Be sure that the word's usage in the example sentence has the same morphology as is provided.)
- "example_sentence_native_language" (string): A natural {native_language} translation of the example sentence.

You may return multiple definitions **only if the word has truly different meanings**. For example:
- ✅ `avocat` can mean "lawyer" or "avocado" — include both definitions.
- ✅ `fait` can mean "fact" (noun) or "done" (past participle of a verb) — include only the definition that makes sense given the morphological information provided.

However:
- ❌ Do NOT include rare or obscure meanings that are likely to confuse beginners.
- ❌ Do NOT include secondary meanings when one is overwhelmingly more common.

Each definition must correspond to exactly the word that is given. Do not define related forms or alternate spellings. If the word is ambiguous between forms (e.g. "avocat"), return all common meanings, but **do not speculate**.

Output the result as a JSON object containing an array of one or more definition objects. Of course, their native language is {native_language}, so you should write the notes in {native_language}."#),
                format!("word: `{word}`\nlemma: `{lemma}`,\npos: {pos}", word=heteronym.word, lemma=heteronym.lemma, pos=heteronym.pos),
            ).await.inspect_err(|e| {
                println!("error: {e:#?}");
            });
            response
        };

        pb.inc(1);

        (heteronym, dict_response)
    })
    .buffer_unordered(50)
    .collect::<Vec<_>>()
    .await
    .into_iter()
    .filter_map(|(heteronym, dict_response)| {
        dict_response.ok().map(|entry| (heteronym.clone(), entry))
    })
    .collect::<BTreeMap<Heteronym<String>, DictionaryEntryThoughts>>();

    pb.finish_with_message(format!(
        "{:.2}",
        CHAT_CLIENT_O3.cost().unwrap_or(0.0) + CHAT_CLIENT_4O.cost().unwrap_or(0.0)
    ));

    Ok(dictionary)
}
