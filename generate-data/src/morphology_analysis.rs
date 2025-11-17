use language_utils::features::Morphology;
use language_utils::{DictionaryEntry, Heteronym, PartOfSpeech};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;

pub async fn create_morphology(
    language: Language,
    frequencies: &Vec<language_utils::FrequencyEntry<String>>,
) -> anyhow::Result<Vec<(Heteronym<String>, Vec<Morphology>)>> {
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

    let morphology = futures::stream::iter(target_language_heteronyms.iter())
        .map(|(heteronym, &freq)| {
            let morphology_response = todo!();
            (heteronym, morphology_response)
        })
        .buffer_unordered(50)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .filter_map(|(heteronym, dict_response)| match (dict_response.ok()) {
            Some(entry) => Some((heteronym.clone(), entry)),
            None => None,
        })
        .collect::<Vec<(Heteronym<String>, Morphology)>>();

    pb.finish_with_message(format!(
        "{:.2}",
        CHAT_CLIENT_O3.cost().unwrap_or(0.0) + CHAT_CLIENT_4O.cost().unwrap_or(0.0)
    ));

    Ok(dictionary)
}

mod llm_morphology {

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

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
    struct MoodResponse {
        #[serde(rename = "1. thoughts")]
        thoughts: String,
        #[serde(rename = "2. mood")]
        mood: Option<language_utils::features::Mood>,
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
        let mood_applies = Mood::applies_to(language, pos);

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
If it has a specific morphological politeness level, provide it. Otherwise, use `"2. politeness": null`. (Respond with JSON, using "1. thoughts" then "2. politeness".){}"#,
                if language.tv_politeness() {"\nPoliteness should only be non-null in the second person as this is a language with T-V distinction. Literary/archaic forms are not related to politeness."} else {""},
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

        let mood_future = async {
            if mood_applies {
                let result: Result<MoodResponse, _> = chat_client.chat_with_system_prompt(
                format!(
                    r#"Determine the mood of the provided {language} verb.
Think about whether this verb has a fixed mood. Mood expresses modality and subclassifies finite verb forms.

Common moods:
- Indicative: default mood, states facts (something happens/happened/will happen)
- Imperative: commands or requests ("Go!", "Please come")
- Conditional: actions under certain conditions ("would go", "would have gone")
- Subjunctive: uncertain/subjective actions in subordinate clauses

Less common moods (use only if applicable):
- Potential: possible but not certain action (can, might, be able to)
- Jussive: desire that action happens (used in Arabic, Sanskrit)
- Purposive: "in order to" (Amazonian/Australian languages)
- Quotative: expressing direct speech of another person
- Optative: exclamations/wishes ("May you...", "If only...")
- Desiderative: want/wish to do something
- Necessitative: must/should/have to
- Interrogative: special form for yes-no questions (Turkic languages)
- Irrealis: action not known to have happened (roof term for conditional/potential/desiderative)
- Admirative: surprise/irony/doubt (Albanian, Balkan languages)

If this verb has a fixed mood, provide it. If mood is not applicable or varies, use `"2. mood": null`. (Respond with JSON, using "1. thoughts" then "2. mood".)"#,
                ),
                format!("{language} word: {} (lemma: {}) (POS: {pos:?})", heteronym.word, heteronym.lemma)
            ).await;
                result.ok().and_then(|r| r.mood)
            } else {
                None
            }
        };

        // Execute all futures concurrently
        let (gender, number, politeness, tense, person, case, mood) = futures::join!(
            gender_future,
            number_future,
            politeness_future,
            tense_future,
            person_future,
            case_future,
            mood_future
        );

        Ok(Morphology {
            gender,
            number,
            politeness,
            tense,
            person,
            case,
            mood,
        })
    }
}

/// Groups dictionary entries by their lemma and part of speech
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LemmaGroup {
    pub lemma: String,
    pub pos: PartOfSpeech,
    pub forms: Vec<WordForm>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WordForm {
    pub word: String,
    pub morphology: Vec<Morphology>,
}

/// Analyzes morphological coverage by grouping words by lemma and POS
pub fn analyze_morphology(
    dictionary: &BTreeMap<Heteronym<String>, DictionaryEntry>,
) -> Vec<LemmaGroup> {
    // Group dictionary entries by (lemma, pos)
    let mut lemma_map: BTreeMap<(String, PartOfSpeech), Vec<WordForm>> = BTreeMap::new();

    for (heteronym, entry) in dictionary {
        let key = (heteronym.lemma.clone(), heteronym.pos);
        lemma_map.entry(key).or_default().push(WordForm {
            word: heteronym.word.clone(),
            morphology: entry.morphology.clone(),
        });
    }

    // Convert to LemmaGroup structure
    let mut groups: Vec<LemmaGroup> = lemma_map
        .into_iter()
        .map(|((lemma, pos), forms)| LemmaGroup { lemma, pos, forms })
        .collect();

    // Sort by number of forms (descending) for easier analysis
    groups.sort_by(|a, b| b.forms.len().cmp(&a.forms.len()));

    groups
}

/// Writes conjugation/declension groups to a JSONL file
pub fn write_conjugations_jsonl(
    groups: &[LemmaGroup],
    output_path: &std::path::Path,
) -> std::io::Result<()> {
    let mut file = File::create(output_path)?;

    for group in groups {
        let json = serde_json::to_string(group)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        writeln!(file, "{}", json)?;
    }

    Ok(())
}
