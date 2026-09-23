//! Core translation-autograding logic, shared between the AI backend handler
//! and offline evaluation tooling.
//!
//! This is the exact prompt-construction + LLM-call + response-mapping pipeline
//! that `yap-ai-backend` serves at `/autograde-translation`, lifted out of the
//! HTTP handler so it can be reused (e.g. for model evals) without spinning up
//! the server. The handler is now a thin wrapper: it selects a `ChatClient`
//! (based on auth / difficulty) and calls [`grade_translation`].

use language_utils::autograde::{
    AutoGradeTranslationRequest, AutoGradeTranslationResponse, GraderContext, Remembered,
};
use serde::Deserialize;
use tysm::chat_completions::ChatClient;

/// Render a grader-context block for a user prompt: the film and the
/// dialogue around the challenge sentence, when known. Empty context renders
/// as nothing, so prompts without context are byte-identical to before the
/// field existed. Shared by translation grading here and the backend's
/// transcription handler.
pub fn grader_context_display(context: &GraderContext) -> String {
    if context.is_empty() {
        return String::new();
    }
    let mut block = String::from("Context (for disambiguation only — never grade these lines):\n");
    if let Some(title) = &context.movie_title {
        block.push_str(&format!("From the film: {title}\n"));
    }
    for line in &context.dialogue_before {
        block.push_str(&format!("Dialogue before: {line}\n"));
    }
    for line in &context.dialogue_after {
        block.push_str(&format!("Dialogue after: {line}\n"));
    }
    block.push('\n');
    block
}

/// Shared assistant persona prepended to grading/feedback prompts.
pub const PERSONALITY: &str = r#"You are a helpful assistant that helps users learn languages. You are friendly and encouraging, and you always try to help the user learn from their mistakes. When correcting the user's mistakes, first congratulate them on the parts they did well on, and then explain the mistakes they made and how they can improve. But the main thing to do is to explain the mistakes in a helpful (but concise) way, and encourage the user. You speak conversationally, as if you were speaking to the user directly. You don't use bullet points or headings, but you do break concepts into individual lines as necessary."#;

#[derive(Debug, thiserror::Error)]
pub enum GradeError {
    #[error("llm error: {0}")]
    Llm(String),
}

/// Convert zero-based literal positions to the grader's one-based indices.
/// Never fall back to matching text: a repeated form can have another sense.
fn primary_expression_instruction(
    primary_expression: &language_utils::TaggedGram<language_utils::Gram<String>>,
    primary_literal_indices: &[usize],
    index_to_position: &[usize],
    primary_is_phrase: bool,
    target_language: language_utils::Language,
) -> String {
    let display = primary_expression.to_display_string(target_language);
    let indices = index_to_position
        .iter()
        .enumerate()
        .filter(|(_, position)| primary_literal_indices.contains(position))
        .map(|(index, _)| (index + 1).to_string())
        .collect::<Vec<_>>()
        .join(", ");
    if primary_is_phrase {
        let occurrence = if indices.is_empty() {
            String::new()
        } else {
            format!(" at literal grading indices {indices}")
        };
        format!(
            "The phrase \"{display}\"{occurrence} motivated this challenge, so please always include it in either phrases_remembered or phrases_forgot."
        )
    } else if indices.is_empty() {
        // Older requests or unmatched expressions have no reliable occurrence.
        // Do not force a grade on an arbitrary word with the same spelling.
        String::new()
    } else {
        format!(
            "The expression \"{display}\" at literal grading indices {indices} motivated this challenge, so please always grade these occurrences as Remembered or Forgot (not null) in literal_grades. Other occurrences of the same text are not the primary expression."
        )
    }
}

/// Grade a user's translation attempt, identifying which words/phrases they
/// remembered vs. forgot. Pure logic: pass in whichever [`ChatClient`] you want
/// to use (reasoning effort, model, endpoint are all configured on the client).
pub async fn grade_translation(
    client: &ChatClient,
    request: &AutoGradeTranslationRequest,
) -> Result<AutoGradeTranslationResponse, GradeError> {
    let AutoGradeTranslationRequest {
        challenge_sentence,
        user_sentence,
        literals,
        phrases,
        course,
        primary_expression,
        primary_literal_indices,
        context,
    } = request;

    let target_language = course.target_language;
    let native_language = course.native_language;

    // Dedup phrases
    let phrases: Vec<language_utils::TaggedGram<language_utils::Gram<String>>> = phrases
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

    // Build display string → gram map for converting LLM output back to grams
    // The model only sees display text, not sense identities. Its judgement
    // therefore applies to every sense sharing that text in this challenge.
    let phrase_display_strings: Vec<String> = phrases
        .iter()
        .map(|g| g.to_display_string(target_language))
        .collect();
    let mut display_to_gram: std::collections::HashMap<
        &str,
        Vec<&language_utils::TaggedGram<language_utils::Gram<String>>>,
    > = std::collections::HashMap::new();
    for (display, gram) in phrase_display_strings.iter().zip(phrases.iter()) {
        display_to_gram.entry(display).or_default().push(gram);
    }

    // Check whether the primary expression is a phrase or a literal-level gram
    let primary_is_phrase = phrases.contains(primary_expression);

    // Count gradable literals early for threshold checks
    let gradable_count = literals
        .iter()
        .filter(|l| l.word.heteronym().is_some())
        .count();

    // Early return if nothing to grade. `literal_grades` keeps its contract of
    // one entry per literal (all None — nothing was gradable).
    if gradable_count == 0 && phrases.is_empty() {
        return Ok(AutoGradeTranslationResponse {
            encouragement: Some("Good effort!".to_string()),
            explanation: None,
            literal_grades: vec![None; literals.len()],
            phrases_remembered: vec![],
            phrases_forgot: vec![],
            autograding_error: None,
        });
    }

    let target_language_name = target_language.prompt_name();
    let native_language_name = native_language.prompt_name();

    // Build the literals list with indices for gradable words, _ for ungradable
    // Track which literal positions have gradable words (for mapping indices back)
    let mut literals_display = String::new();
    let mut gradable_index = 1u32;
    let mut index_to_position: Vec<usize> = Vec::new(); // Maps 1-based index to literal position
    for (position, literal) in literals.iter().enumerate() {
        let is_gradable = literal.word.heteronym().is_some();
        if is_gradable {
            index_to_position.push(position);
            literals_display.push_str(&format!(
                "{}. \"{}\" (lemma: {}, pos: {})\n",
                gradable_index,
                literal.word.text,
                literal
                    .word
                    .heteronym()
                    .map(|h| h.lemma.as_str())
                    .unwrap_or(&literal.word.text),
                literal
                    .word
                    .heteronym()
                    .map(|h| format!("{:?}", h.pos))
                    .unwrap_or_else(|| "OTHER".to_string())
            ));
            gradable_index += 1;
        } else {
            literals_display.push_str(&format!(
                "_. \"{}\" (does not need to be graded)\n",
                literal.word.text
            ));
        }
    }

    // Build phrases list
    let phrases_display = if phrases.is_empty() {
        "(none)".to_string()
    } else {
        phrase_display_strings
            .iter()
            .map(|p| format!("- \"{p}\""))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let primary_expression_system_instruction = primary_expression_instruction(
        primary_expression,
        primary_literal_indices,
        &index_to_position,
        primary_is_phrase,
        target_language,
    );

    let system_prompt = format!(
        r#"{PERSONALITY}The user is learning {target_language_name}. They were challenged to translate a {target_language_name} sentence to {native_language_name}. Your goal is to identify which {target_language_name} words or phrases they remembered, and which ones they forgot. If they translated the sentence correctly, that means they remembered everything! But if they translated the sentence incorrectly, we need to figure out what words and phrases they seemed to have remembered correctly, and which ones they seem to have remembered incorrectly. This will be used as part of a spaced-repetition system, which will help users study the words they need to.

{primary_expression_system_instruction}

You will be given:
1. Literals: Individual words in order, each with an index number. Words marked with "_" do not need grading (proper nouns, punctuation, etc.).
2. Phrases: Multi-word expressions that should be graded as units.

For each indexed literal, decide if the user remembered it ("Remembered"), forgot it ("Forgot"), or if it's indeterminate (null). Grade each literal individually based on the user's translation.

For phrases, list which ones were remembered and which were forgotten. If one was netiher remembered nor forgotten (e.g. it was not in the sentence), just don't mention it at all. There might be a lot of phrases in the provided list that are not actually in the sentence - that's just to give you a large block of marble to carve from, but our phrase detection is very liberal and expansive so it often picks up false positives that you should basically ignore.

The grades feed a spaced-repetition scheduler, so each "Forgot" should point at a word the user actually needs to study. Keep these distinctions in mind:

"Forgot" means there is positive evidence the user did not know this word: they rendered it with the wrong meaning, or the specific sense, tense, mood, or modality it carries is missing from their translation even though the overall sentence reads plausibly. A defensible English sentence does not by itself show that every word in it was understood. If "doit" comes out as "should", or "il y a longtemps" loses the "ago", the verb or phrase that carries that meaning was forgotten.

null means there is no evidence either way. When the user misparses a sentence, several words often vanish from their translation as a side effect, especially small function words like "que", "il", "ce", "de", or "en". Those words were not tested; the misparse was caused by something else. Mark the word or phrase that caused the misparse as "Forgot", and mark the words that merely dropped out as null. Grading them "Forgot" would schedule extra reviews of words the user probably knows.

"Remembered" means the user's translation reflects the word's meaning in this context, even if rendered non-literally. Do not punish learners for non-literal translations when the meaning is preserved. An obvious typo in the user's own language (a letter swapped or dropped in a word that is otherwise clearly the right one) is not evidence of forgetting.

Many sentences will be "partial sentences," such as "Ne pas." meaning "Do not." These are still valid test sentences.

Respond with JSON in this format:
{{
  "encouragement": "Always provide: short positive message (1-2 sentences) highlighting what they got right",
  "explanation": "Only if errors: brief explanation of mistakes and how to improve",
  "literal_grades": [{{"index": 1, "result": "Remembered"}}, {{"index": 2, "result": "Forgot"}}, {{"index": 3, "result": null}}],
  "phrases_remembered": ["phrase1"],
  "phrases_forgot": ["phrase2"]
}}

Example:
Input:
Challenge sentence: Ça se passe bien.
User response: It passes itself well.

Literals:
1. "Ça" (lemma: ce, pos: Pron)
2. "se" (lemma: se, pos: Pron)
3. "passe" (lemma: passer, pos: Verb)
4. "bien" (lemma: bien, pos: Adv)
_. "." (does not need to be graded)

Phrases:
- "se passer"

Output:
{{
  "encouragement": "Good effort tackling this sentence!",
  "explanation": "The French expression '<word>se passer</word>' means 'to happen.' You translated it literally as 'pass itself.' A correct translation is: 'It's going well.'",
  "literal_grades": [{{"index": 1, "result": "Remembered"}}, {{"index": 2, "result": "Remembered"}}, {{"index": 3, "result": "Remembered"}}, {{"index": 4, "result": "Remembered"}}],
  "phrases_remembered": [],
  "phrases_forgot": ["se passer"]
}}

Note: Even though "se passer" was forgotten, the individual words "se" and "passe" were understood (the user knew they mean "itself" and "pass"), so they are marked as remembered.

Second example, a misparse with one culprit:
Input:
Challenge sentence: Où as-tu mis les clés ?
User response: Who has the keys?

Literals:
1. "Où" (lemma: où, pos: Adv)
2. "as" (lemma: avoir, pos: Aux)
3. "tu" (lemma: tu, pos: Pron)
4. "mis" (lemma: mettre, pos: Verb)
5. "les" (lemma: le, pos: Det)
6. "clés" (lemma: clé, pos: Noun)
_. "?" (does not need to be graded)

Output:
{{
  "encouragement": "You got <word>les clés</word> right away!",
  "explanation": "<word>Où</word> means 'where', not 'who', so the sentence asks 'Where did you put the keys?'",
  "literal_grades": [{{"index": 1, "result": "Forgot"}}, {{"index": 2, "result": null}}, {{"index": 3, "result": null}}, {{"index": 4, "result": null}}, {{"index": 5, "result": "Remembered"}}, {{"index": 6, "result": "Remembered"}}],
  "phrases_remembered": [],
  "phrases_forgot": []
}}

Note: Misreading "Où" as "who" derailed the sentence, so "as", "tu", and "mis" never got a fair test: the translation does not show whether they were understood in place, so they are null rather than "Forgot". "les clés" was clearly understood, so those are "Remembered".

The encouragement should always be provided, focus on what they got right, and be written as if speaking directly to the user. The explanation should only be provided if there are errors. Markdown formatting is allowed (no bullet points or numbered lists). Keep both short and concise. Respond in {native_language_name}!

When you mention a {target_language_name} word or phrase inside the encouragement or explanation, wrap it in a <word>...</word> tag (e.g. <word>word</word>). This lets the UI style and pronounce it correctly. Do not wrap {native_language_name} text.

The input may include a Context block naming the film the sentence comes from and the dialogue lines around it. Use it only to disambiguate meaning, tone, or register — never grade the context lines themselves, and never require the user's translation to reflect information that only appears in the context.
"#,
    );

    let user_prompt = format!(
        r#"{context_display}Challenge sentence: {challenge_sentence}
User response: {user_sentence}

Literals:
{literals_display}
Phrases:
{phrases_display}"#,
        context_display = grader_context_display(context),
    );

    // LLM response format uses indexed grades for easier model tracking
    #[derive(Deserialize, schemars::JsonSchema)]
    struct LiteralGrade {
        index: u32,
        result: Option<Remembered>,
    }

    #[derive(Deserialize, schemars::JsonSchema)]
    struct LlmResponse {
        encouragement: Option<String>,
        explanation: Option<String>,
        literal_grades: Vec<LiteralGrade>,
        phrases_remembered: Vec<String>,
        phrases_forgot: Vec<String>,
    }

    let llm_response: LlmResponse = client
        .chat_with_system_prompt(system_prompt, &user_prompt)
        .await
        .map_err(|e| GradeError::Llm(format!("{e:?}")))?;

    // Map indexed grades back to positional array (one entry per literal)
    // Ungradable literals (Other word types) remain None
    let mut positional_grades: Vec<Option<Remembered>> = vec![None; literals.len()];
    for grade in llm_response.literal_grades {
        if grade.index >= 1 && (grade.index as usize) <= index_to_position.len() {
            let position = index_to_position[(grade.index - 1) as usize];
            positional_grades[position] = grade.result;
        }
    }

    // Sanitize phrase outputs:
    // 1. Map LLM display strings back to Gram<String> using the display_to_gram map (filters unknown phrases)
    // 2. Resolve contradictions: if same phrase in both, keep in forgot (forgot takes precedence)
    let mut phrases_forgot: Vec<language_utils::TaggedGram<language_utils::Gram<String>>> =
        llm_response
            .phrases_forgot
            .into_iter()
            .flat_map(|p| {
                display_to_gram
                    .get(p.as_str())
                    .into_iter()
                    .flatten()
                    .map(|g| (*g).clone())
            })
            .collect();
    phrases_forgot.sort();
    phrases_forgot.dedup();

    let forgot_set: std::collections::BTreeSet<
        &language_utils::TaggedGram<language_utils::Gram<String>>,
    > = phrases_forgot.iter().collect();
    let mut phrases_remembered: Vec<language_utils::TaggedGram<language_utils::Gram<String>>> =
        llm_response
            .phrases_remembered
            .into_iter()
            .flat_map(|p| {
                display_to_gram
                    .get(p.as_str())
                    .into_iter()
                    .flatten()
                    .map(|g| (*g).clone())
            })
            .filter(|p| !forgot_set.contains(p))
            .collect();
    phrases_remembered.sort();
    phrases_remembered.dedup();

    Ok(AutoGradeTranslationResponse {
        encouragement: llm_response.encouragement,
        explanation: llm_response.explanation,
        literal_grades: positional_grades,
        phrases_remembered,
        phrases_forgot,
        autograding_error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_instruction_names_the_repeated_occurrence_not_its_spelling() {
        use language_utils::{Atom, Gram, Heteronym, Language, PartOfSpeech, TaggedGram, Word};
        let primary = TaggedGram {
            gram: Gram(vec![Atom::Tok(Word {
                text: "bank".into(),
                word_type: language_utils::WordType::Heteronym(Heteronym {
                    word: "bank".into(),
                    lemma: "bank".into(),
                    pos: PartOfSpeech::Noun,
                }),
            })]),
            sense: std::num::NonZeroU32::new(2),
        };
        // Literals are [bank, comma, bank]; only positions 0 and 2 are
        // gradable, so the second bank is grading index 2, not 3.
        let instruction =
            primary_expression_instruction(&primary, &[2], &[0, 2], false, Language::English);
        assert!(instruction.contains("\"bank\" at literal grading indices 2"));
        assert!(!instruction.contains("indices 1"));
        assert!(!instruction.contains("indices 3"));
        assert!(
            instruction
                .contains("Other occurrences of the same text are not the primary expression")
        );
        let phrase =
            primary_expression_instruction(&primary, &[0, 2], &[0, 2], true, Language::English);
        assert!(phrase.contains("indices 1, 2"));
        assert!(phrase.contains("phrases_remembered or phrases_forgot"));
        assert!(
            primary_expression_instruction(&primary, &[], &[0, 2], false, Language::English)
                .is_empty()
        );
    }

    #[test]
    fn grader_context_renders_only_when_present() {
        assert_eq!(grader_context_display(&GraderContext::default()), "");

        let context = GraderContext {
            movie_title: Some("Delicatessen".to_string()),
            dialogue_before: vec!["-Pourquoi vous faites ça ?".to_string()],
            dialogue_after: vec!["Vraiment ?".to_string(), "Dites quelque chose.".to_string()],
        };
        let display = grader_context_display(&context);
        assert!(display.starts_with("Context (for disambiguation only"));
        assert!(display.contains("From the film: Delicatessen"));
        assert!(display.contains("Dialogue before: -Pourquoi vous faites ça ?"));
        assert!(display.contains("Dialogue after: Dites quelque chose."));
        assert!(display.ends_with("\n\n"));
    }
}
