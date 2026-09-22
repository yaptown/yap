//! Translation review decisions. Browser request cancellation and draft storage
//! remain host adapters; this module never writes or modifies deck events.
use crate::{
    AudioRequest, DefinitionView, ProperNounGroup, Sound, definition_view, proper_noun_groups,
};
use language_utils::{
    Course, ProperNounDefinition, autograde,
    text_cleanup::{find_closest_match, normalize_for_grading},
};
use language_utils::{
    Gram, GramDefinition, Heteronym, Language, Literal,
    autograde::{AutoGradeTranslationResponse, Remembered},
};
use std::collections::BTreeSet;

// Preserve the field names already stored in pending translation drafts.
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualTranslationGrade {
    pub literal_grades: Vec<Option<Remembered>>,
    pub phrases_remembered: Vec<Gram<String>>,
    pub phrases_forgot: Vec<Gram<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encouragement: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub autograding_error: Option<String>,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum TranslationReviewResult {
    Perfect {
        encouragement: Option<String>,
        explanation: Option<String>,
    },
    Manual {
        grade: ManualTranslationGrade,
    },
}

#[bridgerton::bridge]
pub fn prepare_translation_review(
    literals: Vec<Literal<String>>,
    response: AutoGradeTranslationResponse,
) -> TranslationReviewResult {
    if crate::translation_is_perfect(literals, response.clone()) {
        TranslationReviewResult::Perfect {
            encouragement: response.encouragement,
            explanation: response.explanation,
        }
    } else {
        TranslationReviewResult::Manual {
            grade: ManualTranslationGrade {
                literal_grades: response.literal_grades,
                phrases_remembered: response.phrases_remembered,
                phrases_forgot: response.phrases_forgot,
                encouragement: response.encouragement,
                explanation: response.explanation,
                autograding_error: response.autograding_error,
            },
        }
    }
}

#[bridgerton::bridge]
pub fn failed_translation_review(literal_count: usize, error: String) -> ManualTranslationGrade {
    ManualTranslationGrade {
        literal_grades: vec![None; literal_count],
        phrases_remembered: vec![],
        phrases_forgot: vec![],
        encouragement: None,
        explanation: None,
        autograding_error: Some(error),
    }
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum TranslationGradeItem {
    Literal {
        #[serde(rename = "literalIndex")]
        literal_index: usize,
        display: String,
        status: Option<bool>,
    },
    Phrase {
        gram: Gram<String>,
        display: String,
        status: Option<bool>,
    },
}
impl TranslationGradeItem {
    fn has_grade(&self) -> bool {
        match self {
            Self::Literal { status, .. } | Self::Phrase { status, .. } => status.is_some(),
        }
    }
}

fn grade_items(
    literals: &[Literal<String>],
    phrases: &[Gram<String>],
    grade: &ManualTranslationGrade,
    language: Language,
) -> Vec<TranslationGradeItem> {
    let mut items = vec![];
    let failed = grade
        .autograding_error
        .as_ref()
        .is_some_and(|e| !e.is_empty());
    for (i, literal) in literals.iter().enumerate() {
        if literal.word.heteronym().is_none() {
            continue;
        }
        let status = grade
            .literal_grades
            .get(i)
            .and_then(|g| g.as_ref())
            .map(|g| *g == Remembered::Remembered);
        if status.is_none() && !failed {
            continue;
        }
        items.push(TranslationGradeItem::Literal {
            literal_index: i,
            display: literal.word.text.clone(),
            status,
        });
    }
    for gram in phrases {
        let status = if grade.phrases_remembered.contains(gram) {
            Some(true)
        } else if grade.phrases_forgot.contains(gram) {
            Some(false)
        } else {
            None
        };
        items.push(TranslationGradeItem::Phrase {
            gram: gram.clone(),
            display: gram.to_display_string(language),
            status,
        });
    }
    items
}

#[bridgerton::bridge]
pub fn apply_translation_grade(
    mut grade: ManualTranslationGrade,
    item: TranslationGradeItem,
    remembered: bool,
    literal_count: usize,
) -> ManualTranslationGrade {
    match item {
        TranslationGradeItem::Phrase { gram, .. } => {
            grade.phrases_remembered.retain(|g| *g != gram);
            grade.phrases_forgot.retain(|g| *g != gram);
            if remembered {
                grade.phrases_remembered.push(gram);
            } else {
                grade.phrases_forgot.push(gram);
            }
        }
        TranslationGradeItem::Literal { literal_index, .. } => {
            if literal_index < literal_count {
                if grade.literal_grades.len() <= literal_index {
                    grade.literal_grades.resize(literal_index + 1, None);
                }
                grade.literal_grades[literal_index] = Some(if remembered {
                    Remembered::Remembered
                } else {
                    Remembered::Forgot
                });
            }
        }
    }
    grade
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, serde::Serialize)]
pub struct ReviewDefinition {
    pub definition: DefinitionView,
    #[allow(clippy::type_complexity)]
    pub breakdown: Option<Vec<(String, Option<String>, Option<String>)>>,
}

#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize)]
pub struct TranslationReviewFeedback {
    pub grade_items: Vec<TranslationGradeItem>,
    pub can_continue: bool,
    pub definitions: Vec<ReviewDefinition>,
    pub tapped_gram_groups: Vec<usize>,
    pub heteronyms_tapped: Vec<Heteronym<String>>,
}

#[bridgerton::bridge]
pub fn get_translation_review_feedback(
    sentence: TranslateComprehensibleSentence,
    grade: Option<ManualTranslationGrade>,
    is_perfect: bool,
    tapped_words: Vec<usize>,
    language: Language,
) -> TranslationReviewFeedback {
    let items = grade
        .as_ref()
        .map(|g| {
            grade_items(
                &sentence.target_language_literals,
                &sentence.unique_target_language_phrases,
                g,
                language,
            )
        })
        .unwrap_or_default();
    let mut seen = BTreeSet::new();
    let mut tapped_gram_groups = vec![];
    let mut heteronyms_tapped = vec![];
    for index in tapped_words {
        if let Some(heteronym) = sentence
            .target_language_literals
            .get(index)
            .and_then(|l| l.word.heteronym())
        {
            heteronyms_tapped.push(heteronym.clone());
        }
        if let Some(&group) = sentence.literal_gram_indices.get(index)
            && seen.insert(group)
        {
            tapped_gram_groups.push(group);
        }
    }
    let mut groups = tapped_gram_groups.clone();
    let mut definitions = vec![];
    if let Some(grade) = &grade {
        for (i, value) in grade.literal_grades.iter().enumerate() {
            if value == &Some(Remembered::Forgot)
                && let Some(&group) = sentence.literal_gram_indices.get(i)
                && seen.insert(group)
            {
                groups.push(group);
            }
        }
    }
    for group in groups {
        if let Some(Some(definition)) = sentence.gram_definitions_for_lookup.get(group) {
            definitions.push(ReviewDefinition {
                definition: definition_view(definition.clone()),
                breakdown: sentence
                    .gram_breakdowns_for_lookup
                    .get(group)
                    .cloned()
                    .flatten(),
            });
        }
    }
    if let Some(grade) = &grade {
        for phrase in &grade.phrases_forgot {
            if let Some(i) = sentence
                .unique_target_language_phrases
                .iter()
                .position(|g| g == phrase)
                && let Some(Some(definition)) = sentence.phrase_definitions.get(i)
            {
                definitions.push(ReviewDefinition {
                    definition: definition_view(definition.clone()),
                    breakdown: sentence.phrase_breakdowns.get(i).cloned().flatten(),
                });
            }
        }
    }
    TranslationReviewFeedback {
        can_continue: is_perfect || items.iter().any(TranslationGradeItem::has_grade),
        grade_items: items,
        definitions,
        tapped_gram_groups,
        heteronyms_tapped,
    }
}

#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TranslateComprehensibleSentence {
    pub audio: AudioRequest,
    pub target_language: String,
    pub target_language_literals: Vec<Literal<String>>,
    /// For each literal, the index of the gram group it belongs to.
    pub literal_gram_indices: Vec<usize>,
    /// Definition for each gram group (indexed by group number). None if no definition is available.
    pub gram_definitions_for_lookup: Vec<Option<GramDefinition>>,
    /// Morpheme/word breakdown for each gram group (parallel to
    /// `gram_definitions_for_lookup`). None when the gram has no useful
    /// breakdown (e.g. unknown word, no morpheme data).
    #[allow(clippy::type_complexity)]
    pub gram_breakdowns_for_lookup: Vec<Option<Vec<(String, Option<String>, Option<String>)>>>,
    pub unique_target_language_phrases: Vec<Gram<String>>,
    /// Definition for each phrase in unique_target_language_phrases (indexed in parallel).
    pub phrase_definitions: Vec<Option<GramDefinition>>,
    /// Breakdown for each phrase (parallel to `unique_target_language_phrases`).
    #[allow(clippy::type_complexity)]
    pub phrase_breakdowns: Vec<Option<Vec<(String, Option<String>, Option<String>)>>>,
    pub native_translations: Vec<String>,
    pub movie_titles: Vec<(String, String)>,
    pub proper_noun_definitions: Vec<(String, ProperNounDefinition)>,
    /// The gram that motivated this challenge (the one being reviewed via spaced repetition).
    pub primary_expression: Gram<String>,
    /// True if the user recently got this sentence wrong in a translation challenge.
    pub second_chance: bool,
}

#[bridgerton::bridge]
pub fn find_closest_translation(
    user_translation: String,
    candidates: Vec<String>,
    language: Language,
) -> Option<String> {
    find_closest_match(&user_translation, &candidates, language)
}

/// Grade a translation locally when the submission exactly matches one of
/// the accepted translations (after normalization): every heteronym counts
/// as remembered and every phrase as remembered. Returns None when the
/// submission doesn't exactly match, i.e. when real grading is needed.
pub fn autograde_perfect_match(
    user_sentence: &str,
    native_translations: &[String],
    literals: &[Literal<String>],
    phrases: &[Gram<String>],
    native_language: Language,
) -> Option<autograde::AutoGradeTranslationResponse> {
    let normalized_user = normalize_for_grading(user_sentence, native_language);
    let is_perfect = native_translations
        .iter()
        .any(|translation| normalize_for_grading(translation, native_language) == normalized_user);
    if !is_perfect {
        return None;
    }

    // One entry per literal: Some(Remembered) for heteronyms, None for Other types
    let literal_grades = literals
        .iter()
        .map(|lit| {
            lit.word
                .heteronym()
                .is_some()
                .then_some(autograde::Remembered::Remembered)
        })
        .collect();

    Some(autograde::AutoGradeTranslationResponse {
        literal_grades,
        phrases_remembered: phrases.to_vec(),
        phrases_forgot: vec![],
        encouragement: Some("Perfect! You translated it correctly!".to_string()),
        explanation: None,
        autograding_error: None,
    })
}

/// Whether an autograde response should count the whole sentence as
/// perfectly translated: no phrase forgotten, every heteronym affirmatively
/// graded Remembered, and a real (non-heuristic) grading. An indeterminate
/// or missing grade for a heteronym is not perfect — promoting that would
/// credit a word the user never demonstrated. Shared by the app's
/// TranslationChallenge and yap-mcp's grade_translation so the promotion
/// rule lives in exactly one place.
#[bridgerton::bridge]
pub fn translation_is_perfect(
    literals: Vec<Literal<String>>,
    response: autograde::AutoGradeTranslationResponse,
) -> bool {
    response.autograding_error.is_none()
        && response.phrases_forgot.is_empty()
        && literals.iter().enumerate().all(|(i, literal)| {
            literal.word.heteronym().is_none()
                || response.literal_grades.get(i) == Some(&Some(autograde::Remembered::Remembered))
        })
}

fn extract_native_words(definition: &GramDefinition) -> Vec<String> {
    match definition {
        GramDefinition::Dictionary(entry) => {
            entry.definitions.iter().map(|d| d.native.clone()).collect()
        }
        GramDefinition::Phrasebook(entry) => {
            // The meaning field is the native translation for phrasebook entries
            vec![entry.meaning.clone()]
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn heuristic_grade_translation(
    user_sentence: &str,
    literals: &[Literal<String>],
    phrases: &[Gram<String>],
    gram_definitions: &[Option<GramDefinition>],
    literal_gram_indices: &[usize],
    phrase_definitions: &[Option<GramDefinition>],
    native_language: Language,
    error_msg: String,
) -> autograde::AutoGradeTranslationResponse {
    let normalized_user = normalize_for_grading(user_sentence, native_language);
    let user_words: Vec<&str> = normalized_user.split_whitespace().collect();

    // Grade each literal
    let literal_grades = literals
        .iter()
        .enumerate()
        .map(|(i, lit)| {
            // Only grade heteronyms
            lit.word.heteronym()?;

            let gram_idx = literal_gram_indices.get(i)?;
            let Some(definition) = gram_definitions.get(*gram_idx)? else {
                return None;
            };

            let native_words = extract_native_words(definition);
            if native_words.is_empty() {
                return None;
            }

            let found = native_words.iter().any(|native| {
                let normalized_native = normalize_for_grading(native, native_language);
                normalized_native
                    .split_whitespace()
                    .any(|word| user_words.contains(&word))
            });

            if found {
                Some(autograde::Remembered::Remembered)
            } else {
                Some(autograde::Remembered::Forgot)
            }
        })
        .collect();

    // Grade phrases
    let mut phrases_remembered = Vec::new();
    let mut phrases_forgot = Vec::new();

    for (i, phrase) in phrases.iter().enumerate() {
        let Some(Some(definition)) = phrase_definitions.get(i) else {
            // No definition available — can't grade, skip (won't appear in either list)
            continue;
        };

        let native_words = extract_native_words(definition);
        if native_words.is_empty() {
            continue;
        }

        let found = native_words.iter().any(|native| {
            let normalized_native = normalize_for_grading(native, native_language);
            normalized_native
                .split_whitespace()
                .any(|word| user_words.contains(&word))
        });

        if found {
            phrases_remembered.push(phrase.clone());
        } else {
            phrases_forgot.push(phrase.clone());
        }
    }

    autograde::AutoGradeTranslationResponse {
        encouragement: None,
        explanation: None,
        literal_grades,
        phrases_remembered,
        phrases_forgot,
        autograding_error: Some(error_msg),
    }
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TranslationState {
    pub sentence: TranslateComprehensibleSentence,
    pub course: Course,
    pub text: String,
    pub tapped: Vec<usize>,
    pub phase: TranslationPhase,
}

#[bridgerton::bridge(transparent, namespace)]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum TranslationPhase {
    Editing,
    Grading {
        completed_at_ms: f64,
        correct_translation: String,
    },
    Graded {
        completed_at_ms: f64,
        correct_translation: String,
        result: TranslationReviewResult,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        completing: bool,
    },
}

#[bridgerton::bridge(transparent, namespace)]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum TranslationEvent {
    TextChanged {
        text: String,
    },
    WordTapped {
        index: usize,
    },
    Submit {
        now_ms: f64,
    },
    CancelGrading,
    Graded {
        response: AutoGradeTranslationResponse,
    },
    GradingFailed {
        message: String,
    },
    ItemGraded {
        item_index: usize,
        grade: Remembered,
    },
    Continue,
}

#[bridgerton::bridge(transparent, namespace)]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum TranslationEffect {
    Autograde {
        submission: String,
    },
    PlaySound {
        sound: Sound,
    },
    Complete {
        outcome: TranslationReviewResult,
        heteronyms_tapped: Vec<Heteronym<String>>,
        submission: String,
        completed_at_ms: f64,
    },
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TranslationStep {
    pub state: TranslationState,
    pub effects: Vec<TranslationEffect>,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum TranslationWordTint {
    Neutral,
    Perfect,
    Tapped,
    Remembered,
    Forgot,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, serde::Serialize)]
pub struct TranslationWordView {
    pub text: String,
    pub whitespace: String,
    pub tint: TranslationWordTint,
    pub tappable: bool,
}

#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize)]
pub struct TranslationGradeItemView {
    pub label: String,
    pub grade: Option<Remembered>,
}

#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize)]
pub struct TranslationGradeSection {
    pub title: String,
    pub subtitle: String,
    pub open_by_default: bool,
    pub items: Vec<TranslationGradeItemView>,
}

#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize)]
pub struct TranslationVerdictView {
    pub submission: String,
    pub correct_translation: String,
    pub submission_label: String,
    pub correct_label: String,
    pub perfect: bool,
    pub encouragement: Option<String>,
    pub explanation: Option<String>,
    pub autograding_error: Option<String>,
}

#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize)]
pub struct TranslationView {
    pub badge: Option<String>,
    pub placeholder: String,
    pub words: Vec<TranslationWordView>,
    pub proper_nouns: Vec<ProperNounGroup>,
    pub verdict: Option<TranslationVerdictView>,
    pub correct_translation: Option<String>,
    pub is_grading: bool,
    pub can_submit: bool,
    pub submit_label: String,
    pub grade_section: Option<TranslationGradeSection>,
    pub can_continue: bool,
    pub continue_label: String,
    pub autograde_error: Option<String>,
    pub definitions: Vec<ReviewDefinition>,
}

#[bridgerton::bridge]
pub fn translation_start(
    sentence: TranslateComprehensibleSentence,
    course: Course,
) -> TranslationState {
    TranslationState {
        sentence,
        course,
        text: String::new(),
        tapped: vec![],
        phase: TranslationPhase::Editing,
    }
}

#[bridgerton::bridge]
pub fn translation_resume(state: TranslationState) -> TranslationStep {
    let effects = if matches!(state.phase, TranslationPhase::Grading { .. }) {
        vec![TranslationEffect::Autograde {
            submission: state.text.clone(),
        }]
    } else {
        translation_completion(&state).into_iter().collect()
    };
    TranslationStep { state, effects }
}

fn translation_completion(state: &TranslationState) -> Option<TranslationEffect> {
    let TranslationPhase::Graded {
        result,
        completed_at_ms,
        completing: true,
        ..
    } = &state.phase
    else {
        return None;
    };
    let feedback = get_translation_review_feedback(
        state.sentence.clone(),
        None,
        false,
        state.tapped.clone(),
        state.course.target_language,
    );
    Some(TranslationEffect::Complete {
        outcome: result.clone(),
        heteronyms_tapped: feedback.heteronyms_tapped,
        submission: state.text.clone(),
        completed_at_ms: *completed_at_ms,
    })
}

#[bridgerton::bridge]
pub fn translation_transition(
    mut state: TranslationState,
    event: TranslationEvent,
) -> TranslationStep {
    let can_continue =
        matches!(event, TranslationEvent::Continue) && translation_view(state.clone()).can_continue;
    let mut effects = vec![];
    match (&mut state.phase, event) {
        (
            TranslationPhase::Graded {
                completing: true, ..
            },
            _,
        ) => {}
        (TranslationPhase::Editing, TranslationEvent::TextChanged { text }) => state.text = text,
        (TranslationPhase::Editing, TranslationEvent::WordTapped { index }) => {
            if state
                .sentence
                .target_language_literals
                .get(index)
                .is_some_and(|l| l.word.heteronym().is_some())
                && !state.tapped.contains(&index)
            {
                state.tapped.push(index);
            }
        }
        (TranslationPhase::Editing, TranslationEvent::Submit { now_ms })
            if !state.text.trim().is_empty() =>
        {
            let correct_translation = find_closest_translation(
                state.text.clone(),
                state.sentence.native_translations.clone(),
                state.course.native_language,
            )
            .or_else(|| state.sentence.native_translations.first().cloned())
            .unwrap_or_default();
            state.phase = TranslationPhase::Grading {
                completed_at_ms: now_ms,
                correct_translation,
            };
            effects.push(TranslationEffect::Autograde {
                submission: state.text.clone(),
            });
        }
        (TranslationPhase::Grading { .. }, TranslationEvent::CancelGrading) => {
            state.phase = TranslationPhase::Editing
        }
        (
            TranslationPhase::Grading {
                completed_at_ms,
                correct_translation,
            },
            event @ (TranslationEvent::Graded { .. } | TranslationEvent::GradingFailed { .. }),
        ) => {
            let result = match event {
                TranslationEvent::Graded { response } => prepare_translation_review(
                    state.sentence.target_language_literals.clone(),
                    response,
                ),
                TranslationEvent::GradingFailed { message } => TranslationReviewResult::Manual {
                    grade: failed_translation_review(
                        state.sentence.target_language_literals.len(),
                        message,
                    ),
                },
                _ => unreachable!(),
            };
            effects.push(TranslationEffect::PlaySound {
                sound: Sound::AiDoneGrading,
            });
            if matches!(result, TranslationReviewResult::Perfect { .. }) {
                effects.push(TranslationEffect::PlaySound {
                    sound: Sound::Success,
                });
            }
            state.phase = TranslationPhase::Graded {
                completed_at_ms: *completed_at_ms,
                correct_translation: correct_translation.clone(),
                result,
                completing: false,
            };
        }
        (
            TranslationPhase::Graded {
                result: TranslationReviewResult::Manual { grade },
                ..
            },
            TranslationEvent::ItemGraded {
                item_index,
                grade: remembered,
            },
        ) => {
            let items = grade_items(
                &state.sentence.target_language_literals,
                &state.sentence.unique_target_language_phrases,
                grade,
                state.course.target_language,
            );
            if let Some(item) = items.get(item_index) {
                *grade = apply_translation_grade(
                    grade.clone(),
                    item.clone(),
                    remembered == Remembered::Remembered,
                    state.sentence.target_language_literals.len(),
                );
            }
        }
        (TranslationPhase::Graded { completing, .. }, TranslationEvent::Continue)
            if can_continue =>
        {
            *completing = true;
            effects.extend(translation_completion(&state));
        }
        _ => {}
    }
    TranslationStep { state, effects }
}

#[bridgerton::bridge]
pub fn translation_view(state: TranslationState) -> TranslationView {
    let editing = matches!(state.phase, TranslationPhase::Editing);
    let is_grading = matches!(state.phase, TranslationPhase::Grading { .. });
    let (grade, perfect, verdict, correct_translation) = match &state.phase {
        TranslationPhase::Editing => (None, false, None, None),
        TranslationPhase::Grading {
            correct_translation,
            ..
        } => (None, false, None, Some(correct_translation.clone())),
        TranslationPhase::Graded {
            result,
            correct_translation,
            ..
        } => {
            let (grade, perfect, encouragement, explanation, error) = match result {
                TranslationReviewResult::Perfect {
                    encouragement,
                    explanation,
                } => (None, true, encouragement.clone(), explanation.clone(), None),
                TranslationReviewResult::Manual { grade } => (
                    Some(grade),
                    false,
                    grade.encouragement.clone(),
                    grade.explanation.clone(),
                    grade.autograding_error.clone(),
                ),
            };
            (
                grade,
                perfect,
                Some(TranslationVerdictView {
                    submission: state.text.clone(),
                    correct_translation: correct_translation.clone(),
                    submission_label: "Your translation:".into(),
                    correct_label: "Correct translation:".into(),
                    perfect,
                    encouragement,
                    explanation,
                    autograding_error: error,
                }),
                Some(correct_translation.clone()),
            )
        }
    };
    let feedback = get_translation_review_feedback(
        state.sentence.clone(),
        grade.cloned(),
        perfect,
        state.tapped.clone(),
        state.course.target_language,
    );
    let words = state
        .sentence
        .target_language_literals
        .iter()
        .enumerate()
        .map(|(i, literal)| {
            let heteronym = literal.word.heteronym().is_some();
            let tapped = state
                .sentence
                .literal_gram_indices
                .get(i)
                .is_some_and(|g| feedback.tapped_gram_groups.contains(g))
                || (heteronym && state.tapped.contains(&i));
            let tint = if perfect {
                TranslationWordTint::Perfect
            } else if tapped {
                TranslationWordTint::Tapped
            } else if heteronym {
                match grade
                    .and_then(|g| g.literal_grades.get(i))
                    .and_then(Option::as_ref)
                {
                    Some(Remembered::Remembered) => TranslationWordTint::Remembered,
                    Some(Remembered::Forgot) => TranslationWordTint::Forgot,
                    None => TranslationWordTint::Neutral,
                }
            } else {
                TranslationWordTint::Neutral
            };
            TranslationWordView {
                text: literal.word.text.clone(),
                whitespace: literal.whitespace.clone(),
                tint,
                tappable: editing && heteronym,
            }
        })
        .collect();
    let autograde_error = grade.and_then(|g| g.autograding_error.clone());
    TranslationView {
        badge: state
            .sentence
            .second_chance
            .then(|| "Second Chance!".into()),
        placeholder: "Translation...".into(),
        words,
        proper_nouns: if editing {
            proper_noun_groups(&state.sentence.proper_noun_definitions)
        } else {
            vec![]
        },
        verdict,
        correct_translation,
        is_grading,
        can_submit: editing && !state.text.trim().is_empty(),
        submit_label: if is_grading {
            "AI is grading..."
        } else {
            "Check Answer"
        }
        .into(),
        grade_section: grade.map(|_| TranslationGradeSection {
            title: "Grade Words".into(),
            subtitle: "Mark as remembered (✓) or forgot (✗)".into(),
            open_by_default: autograde_error.is_some(),
            items: feedback
                .grade_items
                .into_iter()
                .map(|item| {
                    let (label, status) = match item {
                        TranslationGradeItem::Literal {
                            display, status, ..
                        }
                        | TranslationGradeItem::Phrase {
                            display, status, ..
                        } => (display, status),
                    };
                    TranslationGradeItemView {
                        label,
                        grade: status.map(|remembered| {
                            if remembered {
                                Remembered::Remembered
                            } else {
                                Remembered::Forgot
                            }
                        }),
                    }
                })
                .collect(),
        }),
        can_continue: feedback.can_continue
            && matches!(
                state.phase,
                TranslationPhase::Graded {
                    completing: false,
                    ..
                }
            ),
        continue_label: if perfect { "Nailed it!" } else { "Continue" }.into(),
        autograde_error,
        definitions: feedback.definitions,
    }
}

#[cfg(test)]
mod reducer_tests {
    use super::*;
    use serde_json::json;

    fn start() -> TranslationState {
        let sentence = serde_json::from_value(json!({
            "audio": {"request": {"text":"chat !", "language":"French", "is_ssml":false, "instructions":null, "speed":1.0, "verification_hints":[]}, "provider":"Google"},
            "target_language":"chat !",
            "target_language_literals": [
                {"word":{"text":"chat","word_type":{"type":"Heteronym","word":"chat","lemma":"chat","pos":"NOUN"}},"whitespace":" "},
                {"word":{"text":"!","word_type":{"type":"Other","other_tag":"PUNCT"}},"whitespace":""}
            ],
            "literal_gram_indices":[0,0], "gram_definitions_for_lookup":[], "gram_breakdowns_for_lookup":[],
            "unique_target_language_phrases":[], "phrase_definitions":[], "phrase_breakdowns":[],
            "native_translations":["cat", "a cat"], "movie_titles":[], "proper_noun_definitions":[],
            "primary_expression":[], "second_chance":true
        })).unwrap();
        translation_start(
            sentence,
            Course {
                target_language: Language::French,
                native_language: Language::English,
            },
        )
    }
    fn editing() -> TranslationState {
        translation_transition(
            start(),
            TranslationEvent::TextChanged {
                text: "a cat".into(),
            },
        )
        .state
    }
    fn submitted() -> TranslationState {
        translation_transition(editing(), TranslationEvent::Submit { now_ms: 1234.0 }).state
    }
    fn response() -> AutoGradeTranslationResponse {
        AutoGradeTranslationResponse {
            literal_grades: vec![Some(Remembered::Remembered), None],
            phrases_remembered: vec![],
            phrases_forgot: vec![],
            encouragement: Some("Nice!".into()),
            explanation: Some("A cat.".into()),
            autograding_error: None,
        }
    }
    fn snapshot(state: &TranslationState) -> serde_json::Value {
        serde_json::to_value(state).unwrap()
    }

    #[test]
    fn editing_taps_are_heteronyms_only_deduplicated_and_group_tinted() {
        let mut state = start();
        for index in [0, 0, 1, usize::MAX] {
            state = translation_transition(state, TranslationEvent::WordTapped { index }).state;
        }
        assert_eq!(state.tapped, [0]);
        let view = translation_view(state);
        assert_eq!(view.badge.as_deref(), Some("Second Chance!"));
        assert!(view.words[0].tappable);
        assert!(!view.words[1].tappable);
        assert!(
            view.words
                .iter()
                .all(|w| w.tint == TranslationWordTint::Tapped)
        );
        assert!(!view.can_submit);
        assert!(!view.can_continue);
    }
    #[test]
    fn submit_is_single_shot_and_cancel_preserves_input() {
        let empty = translation_transition(start(), TranslationEvent::Submit { now_ms: 1.0 });
        assert!(empty.effects.is_empty());
        let whitespace = translation_transition(
            start(),
            TranslationEvent::TextChanged { text: " \n".into() },
        )
        .state;
        assert!(
            translation_transition(whitespace, TranslationEvent::Submit { now_ms: 1.0 })
                .effects
                .is_empty()
        );
        let step = translation_transition(editing(), TranslationEvent::Submit { now_ms: 1234.0 });
        assert!(
            matches!(&step.effects[..], [TranslationEffect::Autograde { submission }] if submission == "a cat")
        );
        assert!(
            matches!(&step.state.phase, TranslationPhase::Grading { completed_at_ms: 1234.0, correct_translation } if correct_translation == "a cat")
        );
        for event in [
            TranslationEvent::Submit { now_ms: 9999.0 },
            TranslationEvent::TextChanged {
                text: "changed".into(),
            },
            TranslationEvent::WordTapped { index: 0 },
            TranslationEvent::Continue,
        ] {
            let ignored = translation_transition(step.state.clone(), event);
            assert_eq!(snapshot(&ignored.state), snapshot(&step.state));
            assert!(ignored.effects.is_empty());
        }
        let cancel = translation_transition(step.state, TranslationEvent::CancelGrading);
        assert_eq!(cancel.state.text, "a cat");
        assert!(matches!(cancel.state.phase, TranslationPhase::Editing));
        let stale = translation_transition(
            cancel.state.clone(),
            TranslationEvent::Graded {
                response: response(),
            },
        );
        assert_eq!(snapshot(&stale.state), snapshot(&cancel.state));
        assert!(stale.effects.is_empty());
    }
    #[test]
    fn server_error_preserves_heuristic_grades_and_feedback() {
        let mut response = response();
        response.autograding_error = Some("server unavailable".into());
        let step = translation_transition(submitted(), TranslationEvent::Graded { response });
        assert!(matches!(
            &step.effects[..],
            [TranslationEffect::PlaySound {
                sound: Sound::AiDoneGrading
            }]
        ));
        let view = translation_view(step.state.clone());
        let verdict = view.verdict.unwrap();
        assert!(!verdict.perfect);
        assert_eq!(verdict.encouragement.as_deref(), Some("Nice!"));
        assert_eq!(verdict.explanation.as_deref(), Some("A cat."));
        assert!(view.grade_section.unwrap().open_by_default);
        assert!(view.can_continue);
        let TranslationPhase::Graded {
            result: TranslationReviewResult::Manual { grade },
            ..
        } = step.state.phase
        else {
            panic!("expected manual")
        };
        assert_eq!(grade.literal_grades, [Some(Remembered::Remembered), None]);
    }
    #[test]
    fn network_failure_requires_manual_grade_then_completes_at_submit_time() {
        let state =
            translation_transition(editing(), TranslationEvent::WordTapped { index: 0 }).state;
        let state =
            translation_transition(state, TranslationEvent::Submit { now_ms: 1234.0 }).state;
        let step = translation_transition(
            state,
            TranslationEvent::GradingFailed {
                message: "You're offline".into(),
            },
        );
        assert!(matches!(
            &step.effects[..],
            [TranslationEffect::PlaySound {
                sound: Sound::AiDoneGrading
            }]
        ));
        assert!(!translation_view(step.state.clone()).can_continue);
        assert!(
            translation_transition(step.state.clone(), TranslationEvent::Continue)
                .effects
                .is_empty()
        );
        let state = translation_transition(
            step.state,
            TranslationEvent::ItemGraded {
                item_index: 0,
                grade: Remembered::Forgot,
            },
        )
        .state;
        let view = translation_view(state.clone());
        assert!(view.can_continue);
        assert_eq!(view.continue_label, "Continue");
        let ignored = translation_transition(
            state.clone(),
            TranslationEvent::ItemGraded {
                item_index: usize::MAX,
                grade: Remembered::Remembered,
            },
        );
        assert_eq!(snapshot(&ignored.state), snapshot(&state));
        assert!(translation_resume(state.clone()).effects.is_empty());
        assert!(snapshot(&state)["phase"].get("completing").is_none());
        let restored: TranslationState = serde_json::from_value(snapshot(&state)).unwrap();
        assert!(translation_view(restored).can_continue);
        let mut before_view = serde_json::to_value(translation_view(state.clone())).unwrap();
        let step = translation_transition(state, TranslationEvent::Continue);
        assert!(!translation_view(step.state.clone()).can_continue);
        before_view["can_continue"] = json!(false);
        assert_eq!(
            before_view,
            serde_json::to_value(translation_view(step.state.clone())).unwrap()
        );
        for event in [
            TranslationEvent::Continue,
            TranslationEvent::ItemGraded {
                item_index: 0,
                grade: Remembered::Remembered,
            },
            TranslationEvent::WordTapped { index: 0 },
            TranslationEvent::CancelGrading,
        ] {
            let ignored = translation_transition(step.state.clone(), event);
            assert_eq!(snapshot(&ignored.state), snapshot(&step.state));
            assert!(ignored.effects.is_empty());
        }
        let restored = serde_json::from_value(snapshot(&step.state)).unwrap();
        assert_eq!(
            serde_json::to_value(translation_resume(restored)).unwrap(),
            serde_json::to_value(&step).unwrap()
        );
        assert!(
            matches!(&step.effects[..], [TranslationEffect::Complete { outcome: TranslationReviewResult::Manual { .. }, completed_at_ms: 1234.0, submission, heteronyms_tapped }] if submission == "a cat" && heteronyms_tapped.len() == 1)
        );
    }
    #[test]
    fn perfect_plays_both_sounds_and_hides_manual_grading() {
        let step = translation_transition(
            submitted(),
            TranslationEvent::Graded {
                response: response(),
            },
        );
        assert!(matches!(
            &step.effects[..],
            [
                TranslationEffect::PlaySound {
                    sound: Sound::AiDoneGrading
                },
                TranslationEffect::PlaySound {
                    sound: Sound::Success
                }
            ]
        ));
        let view = translation_view(step.state.clone());
        assert!(view.can_continue);
        assert_eq!(view.continue_label, "Nailed it!");
        assert!(view.grade_section.is_none());
        assert!(
            view.words
                .iter()
                .all(|w| w.tint == TranslationWordTint::Perfect && !w.tappable)
        );
        assert!(translation_resume(step.state.clone()).effects.is_empty());
        let ignored = translation_transition(
            step.state.clone(),
            TranslationEvent::ItemGraded {
                item_index: 0,
                grade: Remembered::Forgot,
            },
        );
        assert_eq!(snapshot(&ignored.state), snapshot(&step.state));
        assert!(matches!(
            &translation_transition(step.state, TranslationEvent::Continue).effects[..],
            [TranslationEffect::Complete {
                outcome: TranslationReviewResult::Perfect { .. },
                ..
            }]
        ));
    }
    #[test]
    fn manual_view_drives_copy_grades_tints_and_deduplicated_definitions() {
        let mut state = editing();
        state.sentence.gram_definitions_for_lookup = vec![Some(GramDefinition::Dictionary(
            language_utils::DictionaryEntry {
                target_language_word: "chat".into(),
                definitions: vec![],
                morphology: vec![],
                segments: vec![],
            },
        ))];
        state.sentence.proper_noun_definitions = vec![(
            "Paris".into(),
            ProperNounDefinition {
                is_person_name: false,
                is_place_name: true,
                is_organization_name: false,
                is_other: false,
                learner_native_language_translation: "Paris".into(),
                description: None,
            },
        )];
        let view = translation_view(state.clone());
        assert_eq!(view.placeholder, "Translation...");
        assert_eq!(view.submit_label, "Check Answer");
        assert_eq!(
            view.proper_nouns,
            vec![ProperNounGroup {
                spans: vec![
                    crate::TextSpan {
                        text: "Paris".into(),
                        target_language: true
                    },
                    crate::TextSpan {
                        text: ": place".into(),
                        target_language: false
                    },
                ],
            }]
        );
        assert_eq!(view.words[0].tint, TranslationWordTint::Neutral);
        let state = translation_transition(state, TranslationEvent::WordTapped { index: 0 }).state;
        assert_eq!(translation_view(state.clone()).definitions.len(), 1);
        let state =
            translation_transition(state, TranslationEvent::Submit { now_ms: 1234.0 }).state;
        let view = translation_view(state.clone());
        assert!(view.is_grading);
        assert_eq!(view.submit_label, "AI is grading...");
        assert!(view.proper_nouns.is_empty());
        let mut response = response();
        response.literal_grades[0] = Some(Remembered::Forgot);
        let mut state = translation_transition(state, TranslationEvent::Graded { response }).state;
        let view = translation_view(state.clone());
        assert_eq!(
            view.definitions.len(),
            1,
            "tapped and forgotten share one definition"
        );
        assert_eq!(
            view.words[0].tint,
            TranslationWordTint::Tapped,
            "taps outrank manual grades"
        );
        let section = view.grade_section.unwrap();
        assert_eq!(section.title, "Grade Words");
        assert_eq!(section.subtitle, "Mark as remembered (✓) or forgot (✗)");
        assert!(!section.open_by_default);
        assert_eq!(section.items[0].label, "chat");
        assert_eq!(section.items[0].grade, Some(Remembered::Forgot));
        let verdict = view.verdict.unwrap();
        assert_eq!(verdict.submission_label, "Your translation:");
        assert_eq!(verdict.correct_label, "Correct translation:");
        state.tapped.clear();
        assert_eq!(
            translation_view(state.clone()).words[0].tint,
            TranslationWordTint::Forgot
        );
        let state = translation_transition(
            state,
            TranslationEvent::ItemGraded {
                item_index: 0,
                grade: Remembered::Remembered,
            },
        )
        .state;
        let view = translation_view(state);
        assert_eq!(view.words[0].tint, TranslationWordTint::Remembered);
        assert_eq!(view.words[1].tint, TranslationWordTint::Neutral);
        assert!(view.definitions.is_empty());
    }

    #[test]
    fn restore_reissues_request_without_changing_timestamp_or_answer() {
        let state = submitted();
        let restored = serde_json::from_value(snapshot(&state)).unwrap();
        let step = translation_resume(restored);
        assert_eq!(snapshot(&step.state), snapshot(&state));
        assert!(
            matches!(&step.effects[..], [TranslationEffect::Autograde { submission }] if submission == "a cat")
        );
        assert!(translation_resume(start()).effects.is_empty());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn literal() -> Literal<String> {
        serde_json::from_value(serde_json::json!({"word":{"text":"chat","word_type":{"type":"Heteronym","word":"chat","lemma":"chat","pos":"NOUN"}},"whitespace":""})).unwrap()
    }
    #[test]
    fn failure_requires_manual_input_and_real_ungradable_words_stay_hidden() {
        let mut grade = failed_translation_review(1, "offline".into());
        let items = grade_items(&[literal()], &[], &grade, Language::French);
        assert_eq!(items.len(), 1);
        assert!(!items.iter().any(TranslationGradeItem::has_grade));
        grade = apply_translation_grade(grade, items[0].clone(), false, 1);
        assert_eq!(grade.literal_grades, [Some(Remembered::Forgot)]);
        assert!(
            grade_items(&[literal()], &[], &grade, Language::French)
                .iter()
                .any(TranslationGradeItem::has_grade)
        );
        grade.literal_grades[0] = None;
        grade.autograding_error = None;
        assert!(grade_items(&[literal()], &[], &grade, Language::French).is_empty());
    }
    #[test]
    fn corrections_are_idempotent_and_support_missing_literal_grades() {
        let phrase = Gram::new(vec![language_utils::Atom::Tok(literal().word)]);
        let item = TranslationGradeItem::Phrase {
            gram: phrase.clone(),
            display: "chat".into(),
            status: None,
        };
        let mut grade = failed_translation_review(0, "offline".into());
        grade = apply_translation_grade(grade, item.clone(), true, 1);
        grade = apply_translation_grade(grade, item.clone(), true, 1);
        assert_eq!(grade.phrases_remembered.len(), 1);
        grade = apply_translation_grade(grade, item, false, 1);
        assert!(grade.phrases_remembered.is_empty());
        assert_eq!(grade.phrases_forgot, [phrase]);
        grade = apply_translation_grade(
            grade,
            TranslationGradeItem::Literal {
                literal_index: 0,
                display: "chat".into(),
                status: None,
            },
            true,
            1,
        );
        assert_eq!(grade.literal_grades, [Some(Remembered::Remembered)]);
        grade = apply_translation_grade(
            grade,
            TranslationGradeItem::Literal {
                literal_index: usize::MAX,
                display: "invalid".into(),
                status: None,
            },
            false,
            1,
        );
        assert_eq!(grade.literal_grades.len(), 1);
    }

    #[test]
    fn manual_grade_round_trips_existing_draft_shape() {
        let old = serde_json::json!({"literalGrades":[null,"Forgot"],"phrasesRemembered":[],"phrasesForgot":[],"autogradingError":"offline"});
        let grade: ManualTranslationGrade = serde_json::from_value(old.clone()).unwrap();
        assert_eq!(serde_json::to_value(grade).unwrap(), old);
    }
    #[test]
    fn heuristic_success_is_never_promoted_to_perfect() {
        let mut response = AutoGradeTranslationResponse {
            literal_grades: vec![Some(Remembered::Remembered)],
            phrases_remembered: vec![],
            phrases_forgot: vec![],
            encouragement: None,
            explanation: None,
            autograding_error: Some("offline".into()),
        };
        assert!(matches!(
            prepare_translation_review(vec![literal()], response.clone()),
            TranslationReviewResult::Manual { .. }
        ));
        response.autograding_error = None;
        assert!(matches!(
            prepare_translation_review(vec![literal()], response),
            TranslationReviewResult::Perfect { .. }
        ));
    }
}
