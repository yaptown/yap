use futures::StreamExt;
use language_utils::{
    features::Morphology, Course, DictionaryEntryThoughts, Heteronym, Language, PhrasebookEntryThoughts
};
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

    let phrasebook = futures::stream::iter(target_language_multi_word_terms.iter()).enumerate().map(|(i, (multiword_term, &freq))| async move {
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

        if i % 200 == 0 {
            println!("{i} / {count} (${cost:.2})", cost=chat_client.cost().unwrap());
            println!("multiword_term: {multiword_term:?}");
            println!("{response:#?}");
        }
        (response, multiword_term)
    })
    .buffered(50)
    .collect::<Vec<_>>()
    .await
    .into_iter()
    .filter_map(|(response, multiword_term)| {
        response.ok().map(|entry| (multiword_term.clone(), entry))
    })
    .collect::<Vec<_>>();

    Ok(phrasebook)
}

pub async fn create_dictionary(
    course: Course,
    frequencies: &Vec<language_utils::FrequencyEntry<String>>,
) -> anyhow::Result<Vec<(Heteronym<String>, (DictionaryEntryThoughts, Morphology))>> {
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

    let dictionary = futures::stream::iter(target_language_heteronyms.iter()).enumerate().map(|(i, (heteronym, &freq))| async move {
        if heteronym.word == "t" && heteronym.lemma == "tu" {
            panic!("heteronym: {heteronym:?}");
        }
        let chat_client = if freq > 500 { &*CHAT_CLIENT_O3 } else { &*CHAT_CLIENT_4O };

        let dict_entry_future = async {
            let response: Result<DictionaryEntryThoughts, _> = chat_client.chat_with_system_prompt(
            format!(r#"The input is a {target_language} word, along with its morphological information. Generate a dictionary entry for it, to be used in an app for beginner {target_language} learners (whose native language is {native_language}). (First, think about the word and its meaning, and what is likely to be relevant to a beginner learner.) First, write the word, then provide a list of one or more definitions. Each definition should be a JSON object with the following fields:

- "native" (string): The {native_language} translation(s) of the word. If a word has multiple very similar meanings (e.g. “this” and “that”), include them in the same string separated by commas. (If it's a verb, you don't have to include the infinitive form or information about conjugation - that will be displayed separately in the app.)
- "note" (string, optional): Use only for extra info about usage that is *not already implied* by the other fields. (For example, you can note that "tu" is informal, or that "on" often means “we” in speech.)  
- "example_sentence_target_language" (string): A natural example sentence using the word in {target_language}. (Be sure that the word's usage in the example sentence has the same morphology as is provided.)
- "example_sentence_native_language" (string): A natural {native_language} translation of the example sentence.  

You may return multiple definitions **only if the word has truly different meanings**. For example:
- ✅ `avocat` can mean “lawyer” or “avocado” — include both definitions.
- ✅ `fait` can mean “fact” (noun) or “done” (past participle of a verb) — include only the definition that makes sense given the morphological information provided.

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

        let morphology_future = get_morphology(target_language, heteronym.clone(), chat_client);

        let (dict_response, morphology_result) = futures::join!(dict_entry_future, morphology_future);

        if i % 200 == 0 {
            println!("{i} / {count} (${cost:.2})", cost=chat_client.cost().unwrap());
            println!("Heteronym: {heteronym:?}");
            println!("{dict_response:#?}");
            println!("Morphology: {morphology_result:#?}");
        }
        (dict_response, morphology_result, heteronym)
    })
    .buffered(50)
    .collect::<Vec<_>>()
    .await
    .into_iter()
    .filter_map(|(dict_response, morphology_result, heteronym)| {
        match (dict_response.ok(), morphology_result.ok()) {
            (Some(entry), Some(morphology)) => Some((heteronym.clone(), (entry, morphology))),
            _ => None,
        }
    })
    .collect::<Vec<_>>();

    Ok(dictionary)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
struct GenderResponse {
    thoughts: String,
    gender: Option<language_utils::features::Gender>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
struct PoliteResponse {
    thoughts: String,
    politeness: Option<language_utils::features::Polite>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
struct TenseResponse {
    thoughts: String,
    tense: Option<language_utils::features::Tense>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
struct PersonResponse {
    thoughts: String,
    person: Option<language_utils::features::Person>,
}

pub async fn get_morphology(
    language: Language,
    heteronym: Heteronym<String>,
    chat_client: &ChatClient,
) -> anyhow::Result<Morphology> {
    use language_utils::features::{Gender, Person, Polite, Tense, FeatureSet};

    let pos = heteronym.pos;

    // Determine which features apply to this word
    let gender_applies = Gender::applies_to(language, pos);
    let politeness_applies = Polite::applies_to(language, pos);
    let tense_applies = Tense::applies_to(language, pos);
    let person_applies = Person::applies_to(language, pos);

    // Issue concurrent requests for all applicable features
    let gender_future = async {
        if gender_applies {
            let result: Result<GenderResponse, _> = chat_client.chat_with_system_prompt(
                format!(
                    "Determine the grammatical gender of the provided {language} word
Think about whether this word has a fixed grammatical gender. 
If it does, provide it. If the gender varies or is not applicable, return null.
Options are:
- Masculine
- Feminine
- Neuter

Additionally, some languages do not distinguish masculine/feminine most of the time but they do distinguish neuter vs. non-neuter (Swedish neutrum / utrum). The non-neuter is called common gender. This is only applicable in languages that do not distinguish masculine/feminine.
- Common

If the gender of the word is not uniquely determined, return null.
"                ),
                format!("word: {} (lemma: {}) (POS: {pos:?})", heteronym.word, heteronym.lemma)
            ).await;
            result.ok().and_then(|r| r.gender)
        } else {
            None
        }
    };

    let politeness_future = async {
        if politeness_applies {
            let result: Result<PoliteResponse, _> = chat_client.chat_with_system_prompt(
                format!(
                    "Determine the morphological politeness of the provided {language} word.
Think about whether this word is morphologically formal, informal, elevated, or humble.
If it has a specific morphological politeness level, provide it. Otherwise, return null.",
                ),
                format!("word: {}", heteronym.word)
            ).await;
            result.ok().and_then(|r| r.politeness)
        } else {
            None
        }
    };

    let tense_future = async {
        if tense_applies {
            let result: Result<TenseResponse, _> = chat_client.chat_with_system_prompt(
                format!(
                    "Determine the tense of the provided {language} word.
Think about whether this word has a fixed tense. Options are:
- Past
- Present
- Future
- Imperfect
- Pluperfect

If one of these options is applicable, provide it. If the tense varies or is not applicable, return null.",
                ),
                format!("word: {}", heteronym.word)
            ).await;
            result.ok().and_then(|r| r.tense)
        } else {
            None
        }
    };

    let person_future = async {
        if person_applies {
            let result: Result<PersonResponse, _> = chat_client.chat_with_system_prompt(
                format!(
                    "Determine the grammatical person of the provided {language} word.
Think about whether this word has a fixed person (e.g., first person pronoun, third person verb).
If it does, provide it. If the person varies or is not applicable, return null.

Options are:
- First
- Second
- Third
Additionally, some language have more than three persons. So Zeroth and Fourth are also allowed. Most languages only have the three standard persons.

If one of these options is applicable, provide it. If the person varies or is not applicable, return null.",
                ),
                format!("word: {}", heteronym.word)
            ).await;
            result.ok().and_then(|r| r.person)
        } else {
            None
        }
    };

    // Execute all futures concurrently
    let (gender, politeness, tense, person) = futures::join!(
        gender_future,
        politeness_future,
        tense_future,
        person_future
    );

    Ok(Morphology {
        gender,
        politeness,
        tense,
        person,
    })
}
