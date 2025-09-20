use futures::StreamExt;
use language_utils::{Course, Language, PronunciationGuideThoughts};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use tysm::chat_completions::ChatClient;

static CHAT_CLIENT: LazyLock<ChatClient> = LazyLock::new(|| {
    ChatClient::from_env("o3")
        .unwrap()
        .with_cache_directory("./.cache")
});

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
struct SoundsListResponse {
    thoughts: String,
    sounds: Vec<String>,
}

/// Generate characteristic sounds/patterns for a language
pub async fn generate_language_sounds(language: Language) -> anyhow::Result<Vec<String>> {
    println!("Generating characteristic sounds for {language:?}...");

    let response: SoundsListResponse = CHAT_CLIENT.chat_with_system_prompt(
        format!(r#"You are creating a comprehensive list of characteristic sounds and letter patterns for {language:?}.

Generate a list of the most important letter patterns and sounds that learners need to know. Include:
- Individual letters with special pronunciations
- Letter combinations (digraphs, trigraphs)
- Position-dependent pronunciations (use $ for end of word, ^ for beginning)
- Silent letters and their patterns

For {language:?}, generate 30-50 characteristic patterns. Use standard notation:
- $ means end of word (e.g., "ent$" for French -ent ending)
- ^ means beginning of word (e.g., "^kn" for English kn- beginning)
- Otherwise just the letters/pattern itself.
- If there are multiple common patterns, include them all separately. (e.g. "ch", "sh", "th", "ph".) Don't write "[c,s,t,p]h".

Focus on patterns that are:
1. Common and frequently encountered
2. Different from how they might be pronounced in other languages
3. Important for correct pronunciation
4. If the target language is very different from the native language, you might just want to include every letter and relevant pattern!

Return a JSON object with:
{{
  "thoughts": "Your analysis of {language:?} pronunciation patterns",
  "sounds": ["pattern1", "pattern2", "pattern3", ...]
}}

Examples of patterns:
- French: "é", "eau", "ent$", "^h", "ch", "oi", "eu"
- Spanish: "ñ", "ll", "rr", "g", "j", "v", "z"
- Korean: "ㄱ", "ㄴ", "ㄷ", "ㄹ", "ㅂ", "ㅅ", "ㅇ", "ㅎ", "ㅏ", "ㅓ", "ㅗ", "ㅜ"
- English: "gh$", "^kn", "tion$", "ch", "sh", "th", "ph"
"#),
        format!("Generate sounds for {language:?}"),
    ).await?;

    println!(
        "Generated {} sounds for {language:?}",
        response.sounds.len()
    );
    Ok(response.sounds)
}

/// Generate pronunciation guides for each sound in a course
pub async fn generate_pronunciation_guides(
    course: Course,
    sounds: &[String],
) -> anyhow::Result<Vec<(String, PronunciationGuideThoughts)>> {
    println!(
        "Generating pronunciation guides for {:?} speakers learning {:?}...",
        course.native_language, course.target_language
    );

    let guides = futures::stream::iter(sounds)
        .map(|sound| {
            let sound = sound.clone();
            async move {
                let response: Result<PronunciationGuideThoughts, _> = CHAT_CLIENT.chat_with_system_prompt(
                    format!(r#"You are creating a pronunciation guide for {native:?} speakers learning {target:?}.

Analyze the {target:?} sound/pattern: "{sound}"

IMPORTANT: Write the description and notes in {native:?} (the learner's native language), not in {target:?}.

Create a guide that includes:
1. A clear description IN {native:?} of how this sound is pronounced
2. How familiar a {native:?} speaker would be with this sound
3. How difficult it is for a {native:?} speaker to pronounce
4. Example words that demonstrate this sound

For the example words, choose 2-3 words that:
- Are VERY likely to be familiar to {native:?} speakers (brand names, food items, place names, cultural references, loan words)
- Clearly demonstrate the sound pattern
- Help the learner recognize sounds they might already know

For each word, specify:
- position: Where the sound appears ("Beginning", "Middle", "End", or "Multiple" if it appears more than once)
- cultural_context: Write IN {native:?} - concisely explain why they know this word.

Return a JSON object with this structure:
{{
  "thoughts": "Brief analysis of this sound for {native:?} speakers",
  "pattern": "{sound}",
  "description": "Clear description IN {native:?} of how to pronounce this",
  "familiarity": "LikelyAlreadyKnows" | "MaybeAlreadyKnows" | "ProbablyDoesNotKnow",
  "difficulty": "Easy" | "Medium" | "Hard",
  "example_words": [
    {{
      "target": "target_word", 
      "native": "native_translation",
      "position": "Beginning" | "Middle" | "End" | "Multiple",
      "cultural_context": "why they know this IN {native:?}"
    }}
  ]
}}

Good examples for Spanish "ñ" and English speakers:
[
  {{"target": "jalapeño", "native": "jalapeño", "position": "End", "cultural_context": "Popular Mexican pepper used in many dishes"}},
  {{"target": "piña colada", "native": "piña colada", "position": "Middle", "cultural_context": "Famous tropical cocktail"}},
  {{"target": "El Niño", "native": "El Niño", "position": "End", "cultural_context": "Weather pattern you hear about in the news"}}
]

Good examples for French "ch" and English speakers:
[
  {{"target": "champagne", "native": "champagne", "position": "Beginning", "cultural_context": "Sparkling wine used for celebrations"}},
  {{"target": "chef", "native": "chef", "position": "Beginning", "cultural_context": "Same word in English - head cook"}},
  {{"target": "cliché", "native": "cliché", "position": "Middle", "cultural_context": "Same word in English - overused phrase"}}
]
"#,
                        native = course.native_language,
                        target = course.target_language,
                        sound = sound
                    ),
                    format!("Analyze sound: {sound}"),
                ).await;

                match response {
                    Ok(guide) => Some((sound.clone(), guide)),
                    Err(e) => {
                        eprintln!("Error generating guide for '{sound}': {e:?}");
                        None
                    }
                }
            }
        })
        .buffered(10)
        .filter_map(|result| async { result })
        .collect::<Vec<_>>()
        .await;

    println!("Generated {} guides", guides.len());
    Ok(guides)
}
