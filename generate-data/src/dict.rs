use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use language_utils::{
    Course, DictionaryEntryThoughts, Heteronym, Language, PhrasebookEntryThoughts,
    features::Morphology,
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
        .with_service_tier("flex")
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

        // Update progress bar with cost from the appropriate client
        let cost = if freq > 500 {
            CHAT_CLIENT_O3.cost().unwrap_or(0.0)
        } else {
            CHAT_CLIENT_4O.cost().unwrap_or(0.0)
        };
        pb.set_message(format!("{:.2}", cost));
        pb.inc(1);

        (response, multiword_term)
    }})
    .buffered(50)
    .collect::<Vec<_>>()
    .await
    .into_iter()
    .filter_map(|(response, multiword_term)| {
        response.ok().map(|entry| (multiword_term.clone(), entry))
    })
    .collect::<Vec<_>>();

    pb.finish_with_message(format!("{:.2}", CHAT_CLIENT_O3.cost().unwrap_or(0.0) + CHAT_CLIENT_4O.cost().unwrap_or(0.0)));

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

    let pb = ProgressBar::new(count as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} dictionary entries ({per_sec}, ${msg}, {eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let dictionary = futures::stream::iter(target_language_heteronyms.iter()).map(|(heteronym, &freq)| {
        let pb = pb.clone();
        async move {
        if heteronym.word == "t" && heteronym.lemma == "tu" {
            panic!("heteronym: {heteronym:?}");
        }
        let chat_client = if freq > 500 { &*CHAT_CLIENT_O3 } else { &*CHAT_CLIENT_4O };

        let dict_entry_future = async {
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

        let morphology_future = get_morphology(target_language, heteronym.clone(), chat_client);

        let (dict_response, morphology_result) = futures::join!(dict_entry_future, morphology_future);

        // Update progress bar with cost from the appropriate client
        let cost = if freq > 500 {
            CHAT_CLIENT_O3.cost().unwrap_or(0.0)
        } else {
            CHAT_CLIENT_4O.cost().unwrap_or(0.0)
        };
        pb.set_message(format!("{:.2}", cost));
        pb.inc(1);

        (dict_response, morphology_result, heteronym)
    }})
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

    pb.finish_with_message(format!("{:.2}", CHAT_CLIENT_O3.cost().unwrap_or(0.0) + CHAT_CLIENT_4O.cost().unwrap_or(0.0)));

    Ok(dictionary)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
struct GenderResponse {
    #[serde(rename = "1. thoughts")]
    thoughts: String,
    #[serde(rename = "2. gender")]
    gender: Option<language_utils::features::Gender>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
struct PoliteResponse {
    #[serde(rename = "1. thoughts")]
    thoughts: String,
    #[serde(rename = "2. politeness")]
    politeness: Option<language_utils::features::Polite>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
struct TenseResponse {
    #[serde(rename = "1. thoughts")]
    thoughts: String,
    #[serde(rename = "2. tense")]
    tense: Option<language_utils::features::Tense>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
struct PersonResponse {
    #[serde(rename = "1. thoughts")]
    thoughts: String,
    #[serde(rename = "2. person")]
    person: Option<language_utils::features::Person>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
struct CaseResponse {
    #[serde(rename = "1. thoughts")]
    thoughts: String,
    #[serde(rename = "2. case")]
    case: Option<language_utils::features::Case>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
struct NumberResponse {
    #[serde(rename = "1. thoughts")]
    thoughts: String,
    #[serde(rename = "2. number")]
    number: Option<language_utils::features::Number>,
}

pub async fn get_morphology(
    language: Language,
    heteronym: Heteronym<String>,
    chat_client: &ChatClient,
) -> anyhow::Result<Morphology> {
    use language_utils::features::{Case, FeatureSet, Gender, Number, Person, Polite, Tense};

    let pos = heteronym.pos;

    // Determine which features apply to this word
    let gender_applies = Gender::applies_to(language, pos);
    let number_applies = Number::applies_to(language, pos);
    let politeness_applies = Polite::applies_to(language, pos);
    let tense_applies = Tense::applies_to(language, pos);
    let person_applies = Person::applies_to(language, pos);
    let case_applies = Case::applies_to(language, pos);

    // Issue concurrent requests for all applicable features
    let gender_future = async {
        if gender_applies {
            let result: Result<GenderResponse, _> = chat_client.chat_with_system_prompt(
                format!(
                    r#"Determine the grammatical gender of the provided {language} word
Think about whether this word has a fixed grammatical gender. 
If it does, provide it. If the gender varies or is not applicable, return null.
Options are:
- Masculine
- Feminine
- Neuter (only applicable in languages that do have a neuter gender.)

Additionally, some languages do not distinguish masculine/feminine most of the time but they do distinguish neuter vs. non-neuter (Swedish neutrum / utrum). The non-neuter is called common gender. This is only applicable in languages that do not distinguish masculine/feminine.
- Common

If the gender of the word is not uniquely determined, return null. Neuter is only applicable in languages that have a neuter gender. Like Common, it is not a placeholder for when the gender is not known. If the grammatical gender is ambiguous or not specified, use `"2. gender": null`. (Respond with JSON, using "1. thoughts" then "2. gender".)"# ),
                format!("{language} word: {} (lemma: {}) (POS: {pos:?})", heteronym.word, heteronym.lemma)
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
                    r#"Determine the morphological politeness of the provided {language} word.
Think about whether this word is morphologically formal, informal, elevated, or humble.
If it has a specific morphological politeness level, provide it. Otherwise, use `"2. politeness": null`. (Respond with JSON, using "1. thoughts" then "2. politeness".)"#,
                ),
                format!("{language} word: {} (lemma: {}) (POS: {pos:?})", heteronym.word, heteronym.lemma)
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
                    r#"Determine the tense of the provided {language} word.
Think about whether this word has a fixed tense. Options are:
- Past
- Present
- Future
- Imperfect
- Pluperfect

If one of these options is applicable, provide it. If the tense varies or is not applicable, use `"2. tense": null`. (Respond with JSON, using "1. thoughts" then "2. tense".)"#,
                ),
                format!("{language} word: {} (lemma: {}) (POS: {pos:?})", heteronym.word, heteronym.lemma)
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
                    r#"Determine the grammatical person of the provided {language} word.
Think about whether this word has a fixed person (e.g., first person pronoun, third person verb).
If it does, provide it. If the person varies or is not applicable, return null.

Options are:
- First
- Second
- Third
Additionally, some language have more than three persons. So Zeroth and Fourth are also allowed. Most languages only have the three standard persons.

If one of these options is applicable, provide it. If the person varies or is not applicable, use `"2. person": null`. (Respond with JSON, using "1. thoughts" then "2. person".)"#,
                ),
                format!("{language} word: {} (lemma: {}) (POS: {pos:?})", heteronym.word, heteronym.lemma)
            ).await;
            result.ok().and_then(|r| r.person)
        } else {
            None
        }
    };

    let case_future = async {
        if case_applies {
            let result: Result<CaseResponse, _> = chat_client.chat_with_system_prompt(
                format!(
                    r#"Determine the grammatical case of the provided {language} word.
Think about whether this word has a fixed case marking. Case helps specify the role of a noun phrase in the sentence.

Common cases include:
- Nominative: subject form (base form)
- Accusative: direct object form
- Dative: indirect object form
- Genitive: possessive form ("of" or "'s")
- Vocative: form used for direct address
- Instrumental: means or instrument ("with/by means of")
- Locative: location in space or time ("in/at/on")
- Ablative: movement from/away ("from")

Other cases (mainly in specific language families):
- Absolutive, Ergative (Basque and others)
- Partitive (Finnish: indefinite/unfinished actions)
- Comitative (together with), Abessive (without)
- Causative (cause/purpose), Benefactive (for)
- Essive (temporary state), Translative (change of state)
- Various locational cases (Adessive, Allative, Elative, Illative, Inessive, etc.)
- And more specialized cases as needed

If this word has a fixed grammatical case, provide it. If case is not applicable or varies, use `"2. case": null`. (Respond with JSON, using "1. thoughts" then "2. case".)"#,
                ),
                format!("{language} word: {} (lemma: {}) (POS: {pos:?})", heteronym.word, heteronym.lemma)
            ).await;
            result.ok().and_then(|r| r.case)
        } else {
            None
        }
    };

    let number_future = async {
        if number_applies {
            let result: Result<NumberResponse, _> = chat_client.chat_with_system_prompt(
                format!(
                    r#"Determine the grammatical number of the provided {language} word.
Think about whether this word has a fixed number marking.

Common number values:
- Singular: one person, animal or thing
- Plural: several persons, animals or things

(For verbs, it should reflect whether the verb is clearly conjugated for a particular number. For example, some verbs are only used for the plural "they", and some are only conjugated for the singular "he". For nouns, it should reflect whether the noun is clearly plural or singular.)

Less common number values (use only if applicable):
- Dual: exactly two items
- Trial: exactly three items
- Paucal: a few items
- GreaterPaucal: more than several but not many
- GreaterPlural: many/all possible items
- Inverse: non-default for that particular noun
- Count: special plural form used after numerals
- PluraleTantum: only appears in plural form but denotes one thing (like "scissors", "pants")
- Collective: grammatical singular describing sets of objects (like "mankind", "furniture")

If this word has a fixed grammatical number, provide it. If number is not applicable, is ambiguous, or varies, use `"2. number": null`. (Respond with JSON, using "1. thoughts" then "2. number".)"#,
                ),
                format!("{language} word: {} (lemma: {}) (POS: {pos:?})", heteronym.word, heteronym.lemma)
            ).await;
            result.ok().and_then(|r| r.number)
        } else {
            None
        }
    };

    // Execute all futures concurrently
    let (gender, number, politeness, tense, person, case) = futures::join!(
        gender_future,
        number_future,
        politeness_future,
        tense_future,
        person_future,
        case_future
    );

    Ok(Morphology {
        gender,
        number,
        politeness,
        tense,
        person,
        case,
    })
}
