use futures::StreamExt;
use language_utils::{
    Course, Language, PatternPosition, PronunciationGuideThoughts, WordPair, WritingSystem,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;
use tysm::chat_completions::ChatClient;
use unicode_normalization::UnicodeNormalization;

static CHAT_CLIENT: LazyLock<ChatClient> =
    LazyLock::new(|| crate::migrating_chat_client("gpt-5.6-sol"));

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
struct SoundsListResponse {
    thoughts: String,
    sounds: Vec<String>,
}

/// Generate characteristic sounds/patterns for a language
/// Returns tuples of (clean_pattern, position)
pub async fn generate_language_sounds(
    language: Language,
) -> anyhow::Result<Vec<(String, PatternPosition)>> {
    let chat_client = &*CHAT_CLIENT;
    let response: SoundsListResponse = chat_client.chat_with_system_prompt(
        format!(r#"You are creating a comprehensive list of characteristic sounds and letter patterns for {language:?}.

Generate a list of the most important letter patterns and sounds that learners need to know. Include:
- All individual letters
- For languages that use accents (or similar), all accented letter forms
- Letter combinations (digraphs, trigraphs)
- Position-dependent pronunciations (use $ for end of word, ^ for beginning)
- Silent letters and their patterns

Use standard notation:
- $ means end of word (e.g., "ent$" for French -ent ending)
- ^ means beginning of word (e.g., "^kn" for English kn- beginning)
- Otherwise just the letters/pattern itself.
- If there are multiple common patterns, include them all separately. (e.g. "ch", "sh", "th", "ph".) Don't write "[c,s,t,p]h".
- Don't use any other syntax besides $, ^, and the literal letters as they would appear in the word.

Focus on patterns that are:
1. Common and frequently encountered
2. Different from how they might be pronounced in other languages
3. Important for correct pronunciation

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

    Ok(response.sounds.iter().map(|s| parse_sound(s)).collect())
}

/// Split a listed sound into its letters and position. The prompt asks for
/// `^` as a prefix and `$` as a suffix, but the model sometimes puts a
/// marker on the wrong end ("$ँ"); a marker anywhere counts, and never
/// survives into the pattern, where the cue would show and speak it.
fn parse_sound(sound: &str) -> (String, PatternPosition) {
    let position = if sound.contains('^') {
        PatternPosition::Beginning
    } else if sound.contains('$') {
        PatternPosition::End
    } else {
        PatternPosition::Anywhere
    };
    let pattern: String = sound.chars().filter(|c| !matches!(c, '^' | '$')).collect();
    (pattern, position)
}

/// What generating one sound's guide produced.
struct GuideOutcome {
    guide: Option<(String, PronunciationGuideThoughts)>,
    /// Examples thrown out for being romanized or not containing the pattern.
    dropped: usize,
    /// Whether a retry is what salvaged this guide.
    rescued: bool,
    /// Set when the API never answered, as opposed to answering unusably.
    api_failed: bool,
}

/// Generate pronunciation guides for each sound in a course
pub async fn generate_pronunciation_guides(
    course: Course,
    sounds: &[(String, PatternPosition)],
) -> anyhow::Result<Vec<(String, PronunciationGuideThoughts)>> {
    let chat_client = &*CHAT_CLIENT;
    let results = futures::stream::iter(sounds)
        .map(|(clean_pattern, position)| {
            let clean_pattern = clean_pattern.clone();
            let position = *position;
            async move {
                let position_note = match position {
                    PatternPosition::Beginning => "This pattern appears at the beginning of words.",
                    PatternPosition::End => "This pattern appears at the end of words.",
                    PatternPosition::Anywhere => "This pattern can appear anywhere in words.",
                };

                let system_prompt = format!(r#"You are creating a pronunciation guide for {native:?} speakers learning {target:?}.

Analyze the {target:?} sound/pattern: "{clean_pattern}"
{position_note}

IMPORTANT: Write the description and notes in {native:?} (the learner's native language), not in {target:?}. The words you choose for the example words should be in {target:?}, using the {target_alphabet:?} writing system.

Create a guide that includes:
1. A clear description IN {native:?} of the ways this pattern is pronounced, maybe analogizing it to words in {native:?} or explaining the difference from similar {native:?} sounds. Keep this part brief. For tricky sounds, you can include some pronunciation advice.
2. How familiar a {native:?} speaker would be with this sound
3. How difficult it is for a {native:?} speaker to pronounce
4. Example words that demonstrate this sound

For the example words, choose 1-4 {target:?} words that:
- Are VERY likely to be familiar to {native:?} speakers (brand names, food items, place names, cultural references, loan words)
- Clearly demonstrate the pattern, in all the ways it can be pronounced
- Contain the actual pattern (e.g. for the pattern "yn", "sphinx" would not be a good example as it does not contain "yn". You may want to spell out candidate words letter by letter while you're thinking, to help make sure they contain the pattern.)
- important: for words to contain the actual pattern, they must literally contain that pattern, with no added or removed accents or diacritics. For example, "chacón" does not contain the pattern "on", because "chacón" has an accent on the "o". This is where spelling the words out letter by letter is helpful.

HARD RULE: The `target` field MUST be written in {target_alphabet:?}; NEVER put a romanization or transliteration there. `native` is the {native:?} translation or gloss. Writing "Jaipur" instead of "जयपुर" for a Hindi target makes the example unusable.

For each word, specify:
- position: Where the sound appears ("Beginning", "Middle", "End", or "Multiple" if it appears more than once)
- cultural_context: Write IN {native:?} - concisely explain why they know this word. This should just be a short hint that the word means what they user probably thinks it might mean. Keep this part brief.

Return a JSON object with this structure:
{{
  "thoughts": "Brief analysis of this sound for {native:?} speakers",
  "pattern": "{clean_pattern}",
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

Good examples for Hindi "ज" and English speakers — `target` is Devanagari, while `native` is the English gloss:
[
  {{"target": "जलेबी", "native": "jalebi", "position": "Beginning", "cultural_context": "Popular spiral-shaped Indian sweet"}},
  {{"target": "ताजमहल", "native": "Taj Mahal", "position": "Middle", "cultural_context": "India's famous marble monument"}}
]
"#,
                        native = course.native_language,
                        target = course.target_language,
                        target_alphabet = course.target_language.writing_system(),
                        clean_pattern = clean_pattern,
                        position_note = position_note
                    );
                const ATTEMPTS: usize = 3;
                let writing_system = course.target_language.writing_system();
                let mut user_prompt = format!("Analyze sound: {clean_pattern}");
                let mut dropped = 0;
                for attempt in 1..=ATTEMPTS {
                    let response: Result<PronunciationGuideThoughts, _> = chat_client
                        .chat_with_system_prompt(system_prompt.clone(), user_prompt.clone())
                        .await;
                    let mut guide = match response {
                        Ok(guide) => guide,
                        Err(e) => {
                            eprintln!("ERROR: generating guide for '{clean_pattern}' failed on attempt {attempt}: {e:?}");
                            return GuideOutcome { guide: None, dropped, rescued: false, api_failed: true };
                        }
                    };
                    let rejected = reject_invalid_examples(
                        &mut guide.example_words,
                        &clean_pattern,
                        position,
                        course.target_language,
                    );
                    dropped += rejected.len();
                    for complaint in &rejected {
                        eprintln!("WARNING: Pattern '{clean_pattern}': dropped {complaint}");
                    }
                    if !guide.example_words.is_empty() {
                        // Use the requested pattern and position, not the model's echo.
                        guide.pattern = clean_pattern.clone();
                        guide.position = position;
                        return GuideOutcome {
                            guide: Some((clean_pattern, guide)),
                            dropped,
                            rescued: attempt > 1,
                            api_failed: false,
                        };
                    }
                    // Include the attempt as well as feedback: even identical bad replies
                    // must produce distinct prompts rather than replaying the cache.
                    let complaints = if rejected.is_empty() {
                        "No example words were supplied.".to_owned()
                    } else {
                        rejected.join("\n")
                    };
                    user_prompt.push_str(&format!(
                        "\nAttempt {attempt} had no usable examples:\n{complaints}\nCorrect the examples: every `target` must be written in {writing_system:?} and must literally contain \"{clean_pattern}\". {position_note} `native` must be the learner's-language translation or gloss.",
                    ));
                }
                eprintln!("WARNING: Pattern '{clean_pattern}' lost every example after {ATTEMPTS} attempts and is missing from the pack");
                GuideOutcome { guide: None, dropped, rescued: false, api_failed: false }
            }
        })
        .buffered(10)
        .collect::<Vec<_>>()
        .await;

    let dropped: usize = results.iter().map(|outcome| outcome.dropped).sum();
    let rescued = results.iter().filter(|outcome| outcome.rescued).count();
    let lost = results
        .iter()
        .filter(|outcome| outcome.guide.is_none() && !outcome.api_failed)
        .count();
    let api_failed = results.iter().filter(|outcome| outcome.api_failed).count();
    eprintln!(
        "Pronunciation guide validation: {dropped} examples dropped, {rescued} guides rescued by retry, {lost} guides lost to unusable examples"
    );

    // A guide the API never answered for is an outage, not a verdict on the
    // language. Shipping the pack anyway silently drops real sounds from the
    // course -- a flex-capacity blip once cost Hindi 150 of its 154 guides,
    // and the run still exited 0. Fail instead: responses are cached, so a
    // rerun resumes from whatever did succeed.
    anyhow::ensure!(
        api_failed == 0,
        "{api_failed} of {} pronunciation guides failed with API errors, so the pack would be \
         missing them. Rerun once the API is healthy; cached guides are reused.",
        results.len(),
    );

    Ok(results
        .into_iter()
        .filter_map(|outcome| outcome.guide)
        .collect())
}

/// Filter before guides leave generation, so invalid text never reaches TTS.
/// The same complaints are logged and sent back to the model on a retry.
fn reject_invalid_examples(
    examples: &mut Vec<WordPair>,
    pattern: &str,
    position: PatternPosition,
    language: Language,
) -> Vec<String> {
    let writing_system = language.writing_system();
    let mut rejected = Vec::new();
    examples.retain(|example| {
        let reason = if !writing_system.appears_in(&example.target) {
            Some(format!("is not {writing_system:?} target-language text (romanization/transliteration is not allowed)"))
        } else if !pattern_matches(&example.target, pattern, position, language) {
            Some(format!("does not contain the literal pattern \"{pattern}\" at position {position:?}"))
        } else {
            None
        };
        if let Some(reason) = reason {
            // Display, not Debug: Debug escapes combining marks, so a
            // Devanagari conjunct would reach the model as "\u{94d}" noise.
            rejected.push(format!("\"{}\": {reason}", example.target));
            false
        } else {
            true
        }
    });
    rejected
}

/// Match the spelling taught by a guide, using the same rules for examples and
/// corpus frequencies. Korean needs syllable decomposition and jamo that may
/// fill either slot; other scripts preserve accents while accepting
/// canonically equivalent text.
///
/// Normalizing is the expensive half, so it is split out: a caller comparing
/// many words against many patterns normalizes each side once and then calls
/// [`matches_normalized`], rather than redoing both for every pair.
pub fn pattern_matches(
    word: &str,
    pattern: &str,
    position: PatternPosition,
    language: Language,
) -> bool {
    matches_normalized(
        &normalize_word(word, language),
        &normalize_pattern(pattern, language),
        position,
    )
}

/// Compare a word against the spellings a pattern could take, matching when
/// any one of them fits. See [`normalize_pattern`] for why there can be several.
pub fn matches_normalized(word: &str, variants: &[String], position: PatternPosition) -> bool {
    variants.iter().any(|pattern| match position {
        PatternPosition::Beginning => word.starts_with(pattern),
        PatternPosition::End => word.ends_with(pattern),
        PatternPosition::Anywhere => word.contains(pattern),
    })
}

/// Reduce a word to the form patterns are compared against.
pub fn normalize_word(word: &str, language: Language) -> String {
    if language.writing_system() == WritingSystem::Hangul {
        // Decompose syllables into their constituent jamo.
        word.nfkd().collect::<String>()
    } else {
        word.to_lowercase().nfc().collect::<String>()
    }
}

/// The spellings a pattern could take, reduced to the form words compare against.
///
/// Most scripts give exactly one. Korean gives several, because a compatibility
/// jamo does not say which slot of a syllable it fills: ㄱ is the same character
/// beginning 감 as ending 각. The cross-syllable patterns are exactly the
/// ambiguous case -- "ㄱㅇ" means a final ㄱ meeting the next syllable's initial
/// ㅇ, as in 먹어, which no all-initial reading can ever match. Generating every
/// combination and taking any of them keeps those patterns matchable.
pub fn normalize_pattern(pattern: &str, language: Language) -> Vec<String> {
    if language.writing_system() != WritingSystem::Hangul {
        return vec![pattern.to_lowercase().nfc().collect()];
    }

    // Patterns are a handful of characters; the cap only stops a pathological
    // one from doubling its way into a memory problem.
    const MAX_VARIANTS: usize = 64;

    // Compatibility jamo are resolved before NFKD, which would decompose them
    // to their initial form and throw the ambiguity away.
    let mut variants = vec![String::new()];
    for ch in pattern.chars() {
        let choices: Vec<String> = match jungseong(ch) {
            Some(vowel) => vec![vowel.to_string()],
            None => match (choseong(ch), jongseong(ch)) {
                (None, None) => vec![ch.nfkd().collect()],
                (initial, final_) => initial
                    .into_iter()
                    .chain(final_)
                    .map(|jamo| jamo.to_string())
                    .collect(),
            },
        };
        variants = variants
            .iter()
            .flat_map(|prefix| {
                choices.iter().map(move |choice| {
                    let mut variant = prefix.clone();
                    variant.push_str(choice);
                    variant
                })
            })
            .collect();
        variants.truncate(MAX_VARIANTS);
    }
    variants
}

/// A compatibility jamo as a syllable-initial, when it can begin one.
fn choseong(ch: char) -> Option<char> {
    Some(match ch {
        'ㄱ' => 'ᄀ',
        'ㄲ' => 'ᄁ',
        'ㄴ' => 'ᄂ',
        'ㄷ' => 'ᄃ',
        'ㄸ' => 'ᄄ',
        'ㄹ' => 'ᄅ',
        'ㅁ' => 'ᄆ',
        'ㅂ' => 'ᄇ',
        'ㅃ' => 'ᄈ',
        'ㅅ' => 'ᄉ',
        'ㅆ' => 'ᄊ',
        'ㅇ' => 'ᄋ',
        'ㅈ' => 'ᄌ',
        'ㅉ' => 'ᄍ',
        'ㅊ' => 'ᄎ',
        'ㅋ' => 'ᄏ',
        'ㅌ' => 'ᄐ',
        'ㅍ' => 'ᄑ',
        'ㅎ' => 'ᄒ',
        _ => return None,
    })
}

/// A compatibility jamo as a syllable-final, when it can end one. The clusters
/// (ㄳ, ㄵ, ㅄ...) only ever appear here.
fn jongseong(ch: char) -> Option<char> {
    Some(match ch {
        'ㄱ' => 'ᆨ',
        'ㄲ' => 'ᆩ',
        'ㄳ' => 'ᆪ',
        'ㄴ' => 'ᆫ',
        'ㄵ' => 'ᆬ',
        'ㄶ' => 'ᆭ',
        'ㄷ' => 'ᆮ',
        'ㄹ' => 'ᆯ',
        'ㄺ' => 'ᆰ',
        'ㄻ' => 'ᆱ',
        'ㄼ' => 'ᆲ',
        'ㄽ' => 'ᆳ',
        'ㄾ' => 'ᆴ',
        'ㄿ' => 'ᆵ',
        'ㅀ' => 'ᆶ',
        'ㅁ' => 'ᆷ',
        'ㅂ' => 'ᆸ',
        'ㅄ' => 'ᆹ',
        'ㅅ' => 'ᆺ',
        'ㅆ' => 'ᆻ',
        'ㅇ' => 'ᆼ',
        'ㅈ' => 'ᆽ',
        'ㅊ' => 'ᆾ',
        'ㅋ' => 'ᆿ',
        'ㅌ' => 'ᇀ',
        'ㅍ' => 'ᇁ',
        'ㅎ' => 'ᇂ',
        _ => return None,
    })
}

/// A compatibility jamo as a medial vowel. Vowels fill one slot only, so they
/// are unambiguous.
fn jungseong(ch: char) -> Option<char> {
    Some(match ch {
        'ㅏ' => 'ᅡ',
        'ㅐ' => 'ᅢ',
        'ㅑ' => 'ᅣ',
        'ㅒ' => 'ᅤ',
        'ㅓ' => 'ᅥ',
        'ㅔ' => 'ᅦ',
        'ㅕ' => 'ᅧ',
        'ㅖ' => 'ᅨ',
        'ㅗ' => 'ᅩ',
        'ㅘ' => 'ᅪ',
        'ㅙ' => 'ᅫ',
        'ㅚ' => 'ᅬ',
        'ㅛ' => 'ᅭ',
        'ㅜ' => 'ᅮ',
        'ㅝ' => 'ᅯ',
        'ㅞ' => 'ᅰ',
        'ㅟ' => 'ᅱ',
        'ㅠ' => 'ᅲ',
        'ㅡ' => 'ᅳ',
        'ㅢ' => 'ᅴ',
        'ㅣ' => 'ᅵ',
        _ => return None,
    })
}

/// Calculate the frequency of each pronunciation pattern based on word frequency data
/// Returns a HashMap mapping each pattern to its total frequency across all words containing it
pub fn calculate_pattern_frequencies(
    sounds: &[(String, PatternPosition)],
    gram_frequencies: &[language_utils::GramFrequencyEntry<String>],
    language: Language,
) -> HashMap<(String, PatternPosition), u32> {
    let mut frequencies = HashMap::new();

    // Initialize all patterns with 0
    for (pattern, position) in sounds {
        frequencies.insert((pattern.clone(), *position), 0);
    }

    // Normalize each pattern once up front, and each word once below: the
    // corpus is large and the sound inventory is in the hundreds, so
    // normalizing inside the inner loop would redo both sides a word-count
    // times a pattern-count number of times.
    let normalized_sounds: Vec<(Vec<String>, PatternPosition, String)> = sounds
        .iter()
        .map(|(pattern, position)| {
            (
                normalize_pattern(pattern, language),
                *position,
                pattern.clone(),
            )
        })
        .collect();

    // Sum up frequencies for each pattern based on word occurrences
    for freq_entry in gram_frequencies {
        // Get the word text from the gram
        let word: String = if let Some(heteronym) = freq_entry.gram.gram.heteronym() {
            heteronym.word.clone()
        } else {
            freq_entry.gram.gram.to_display_string(language)
        };
        let word_normalized = normalize_word(&word, language);

        for (variants, position, pattern) in &normalized_sounds {
            if matches_normalized(&word_normalized, variants, *position) {
                // Add the word's frequency count to the pattern's total
                let current = frequencies.get_mut(&(pattern.clone(), *position)).unwrap();
                *current += freq_entry.count;
            }
        }
    }

    frequencies
}

#[cfg(test)]
mod sound_tests {
    use super::*;

    #[test]
    fn hindi_examples_must_contain_the_actual_conjunct() {
        for (word, pattern, expected) in [
            ("आगरा", "ग्र", false),
            ("ग्राम", "ग्र", true),
            ("बच्चा", "च्छ", false),
            ("हर्ष", "र्श", false),
            ("मंत्र", "त्र", true),
            ("Jaipur", "ज", false),
        ] {
            assert_eq!(
                pattern_matches(word, pattern, PatternPosition::Anywhere, Language::Hindi),
                expected,
                "{word}: {pattern}"
            );
        }
    }

    #[test]
    fn korean_compatibility_jamo_depend_on_position() {
        assert!(pattern_matches(
            "국",
            "ㄱ",
            PatternPosition::End,
            Language::Korean
        ));
        assert!(pattern_matches(
            "김치",
            "ㄱ",
            PatternPosition::Beginning,
            Language::Korean
        ));
        assert!(pattern_matches(
            "고기",
            "ㄱ",
            PatternPosition::Anywhere,
            Language::Korean
        ));
        assert!(!pattern_matches(
            "김치",
            "ㄱ",
            PatternPosition::End,
            Language::Korean
        ));
        // ㄱ is the batchim of 악, and "Anywhere" asks only that the letter
        // appear -- which the all-initial reading used to miss.
        assert!(pattern_matches(
            "악",
            "ㄱ",
            PatternPosition::Anywhere,
            Language::Korean
        ));
    }

    #[test]
    fn korean_cross_syllable_patterns_match_across_the_boundary() {
        // "ㄱㅇ" is a final ㄱ meeting the next syllable's initial ㅇ. Reading
        // both jamo as initials made these patterns unmatchable, which cost
        // Korean a run's worth of liaison guides.
        for (word, pattern) in [
            ("먹어", "ㄱㅇ"),
            ("좋다", "ㅎㄷ"),
            ("굳이", "ㄷ이"),
            ("한국어", "ㄱㅇ"),
        ] {
            assert!(
                pattern_matches(word, pattern, PatternPosition::Anywhere, Language::Korean),
                "{word} should contain {pattern}"
            );
        }
        // Still discriminating: 사과 has no final ㄱ before an initial ㅇ.
        assert!(!pattern_matches(
            "사과",
            "ㄱㅇ",
            PatternPosition::Anywhere,
            Language::Korean
        ));
    }

    #[test]
    fn korean_final_only_clusters_are_never_read_as_initials() {
        // ㅄ only ever ends a syllable, so it has exactly one realization.
        assert_eq!(normalize_pattern("ㅄ", Language::Korean), vec!["ᆹ"]);
        assert!(pattern_matches(
            "없다",
            "ㅄ",
            PatternPosition::Anywhere,
            Language::Korean
        ));
        // A vowel fills one slot, so it does not multiply the variants.
        assert_eq!(normalize_pattern("ㅏ", Language::Korean), vec!["ᅡ"]);
        // An ambiguous consonant yields both readings, initial first.
        assert_eq!(normalize_pattern("ㄱ", Language::Korean), vec!["ᄀ", "ᆨ"]);
    }

    #[test]
    fn accents_are_preserved_but_canonical_spellings_compare_equal() {
        assert!(!pattern_matches(
            "chacón",
            "on",
            PatternPosition::Anywhere,
            Language::French
        ));
        assert!(pattern_matches(
            "bon",
            "on",
            PatternPosition::Anywhere,
            Language::French
        ));
        assert!(pattern_matches(
            "cafe\u{301}",
            "é",
            PatternPosition::End,
            Language::French
        ));
        assert!(pattern_matches(
            "CAFÉ",
            "e\u{301}",
            PatternPosition::End,
            Language::French
        ));
        assert!(!pattern_matches(
            "cafe\u{301}",
            "e",
            PatternPosition::Anywhere,
            Language::French
        ));
        assert!(pattern_matches(
            "CHef",
            "ch",
            PatternPosition::Beginning,
            Language::French
        ));
        assert!(!pattern_matches(
            "chef",
            "ch",
            PatternPosition::End,
            Language::French
        ));
    }

    fn example(target: &str) -> WordPair {
        WordPair {
            target: target.into(),
            native: "gloss".into(),
            position: language_utils::SoundPosition::Beginning,
            cultural_context: "context".into(),
        }
    }

    #[test]
    fn validation_keeps_good_examples_and_explains_every_rejection() {
        let mut examples = vec![example("Jaipur"), example("ग्राम"), example("आगरा")];
        let rejected = reject_invalid_examples(
            &mut examples,
            "ग्र",
            PatternPosition::Anywhere,
            Language::Hindi,
        );
        assert_eq!(examples, vec![example("ग्राम")]);
        assert_eq!(rejected.len(), 2);
        assert!(rejected[0].contains("Jaipur") && rejected[0].contains("Devanagari"));
        assert!(rejected[1].contains("आगरा") && rejected[1].contains("ग्र"));
    }

    #[test]
    fn validation_can_remove_every_example_and_enforces_position() {
        let mut examples = vec![example("mantra"), example("मंत्र")];
        let rejected = reject_invalid_examples(
            &mut examples,
            "त्र",
            PatternPosition::Beginning,
            Language::Hindi,
        );
        assert!(examples.is_empty());
        assert_eq!(rejected.len(), 2);
        assert!(
            reject_invalid_examples(
                &mut examples,
                "त्र",
                PatternPosition::Anywhere,
                Language::Hindi
            )
            .is_empty()
        );
    }

    #[test]
    fn markers_anywhere_set_position_and_never_leak() {
        assert_eq!(
            parse_sound("^kn"),
            ("kn".into(), PatternPosition::Beginning)
        );
        assert_eq!(parse_sound("ent$"), ("ent".into(), PatternPosition::End));
        assert_eq!(parse_sound("$ँ"), ("ँ".into(), PatternPosition::End));
        assert_eq!(
            parse_sound("は^"),
            ("は".into(), PatternPosition::Beginning)
        );
        assert_eq!(parse_sound("ch"), ("ch".into(), PatternPosition::Anywhere));
    }
}
