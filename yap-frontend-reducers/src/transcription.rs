use crate::{ProperNounGroup, Sound, VerdictHeadline, VerdictTone, proper_noun_groups};
use language_utils::{
    Course, Language, ProperNounDefinition,
    text_cleanup::{normalize_for_grading, remove_accents_lowercase},
    transcription_challenge::{self, Grade, Part, PartGraded, PartSubmitted, WordGrade},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TranscribeComprehensibleSentence {
    pub target_language: String,
    pub audio: crate::AudioRequest,
    pub native_language: String,
    /// Readings per literal in flattened `parts`, without literal whitespace.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub literal_ruby: Vec<Vec<language_utils::readings::RubySegment>>,
    pub parts: Vec<Part>,
    /// Parallel to `parts`. For each part, one gram-group index per literal
    /// (length matches the literal count of the corresponding part). Lets the
    /// frontend map a graded literal back to its gram definition/breakdown.
    pub part_gram_indices: Vec<Vec<usize>>,
    /// Gram definitions indexed by gram group.
    pub gram_definitions_for_lookup: Vec<Option<language_utils::GramDefinition>>,
    /// Morpheme/word breakdowns indexed by gram group (parallel to
    /// `gram_definitions_for_lookup`).
    #[allow(clippy::type_complexity)]
    pub gram_breakdowns_for_lookup: Vec<Option<Vec<(String, Option<String>, Option<String>)>>>,
    pub movie_titles: Vec<(String, String)>,
    pub proper_noun_definitions: Vec<(String, ProperNounDefinition)>,
    /// True if the user recently got this sentence wrong in a transcription challenge.
    pub second_chance: bool,
}

#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct TranscriptionInput {
    pub index: usize,
    pub text: String,
}

#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize)]
pub struct TranscriptionSubmission {
    pub request: Vec<PartSubmitted>,
    pub all_blanks_filled: bool,
}

/// Chunk the displayed parts, retaining part indices for host-local inputs. A blank
/// occupies one token; its outer boundaries come from its first/last literals.
#[bridgerton::bridge]
pub fn transcription_sentence_chunks(
    parts: Vec<Part>,
    language: Language,
) -> Vec<language_utils::SentenceChunk> {
    let text: Vec<_> = parts
        .iter()
        .map(|part| match part {
            Part::Provided { part } => (part.word.text.clone(), part.whitespace),
            Part::AskedToTranscribe { parts } => (
                language_utils::literals_to_text(parts)
                    .trim_end()
                    .to_owned(),
                parts
                    .last()
                    .map_or(language_utils::Whitespace::None, |part| part.whitespace),
            ),
        })
        .collect();
    language_utils::sentence_chunks(
        &text
            .iter()
            .map(|(text, whitespace)| (text.as_str(), *whitespace))
            .collect::<Vec<_>>(),
        language,
    )
}

#[bridgerton::bridge]
pub fn transcription_mask_cue(cue_text: String, sentence: String, masked: String) -> String {
    transcription_challenge::mask_cue(&cue_text, &sentence, &masked)
}

fn has_text(text: &str) -> bool {
    !text.trim().is_empty()
}

#[bridgerton::bridge]
pub fn transcription_can_submit(texts: Vec<String>) -> bool {
    texts.iter().all(|text| has_text(text))
}

#[bridgerton::bridge]
pub fn prepare_transcription_submission(
    parts: Vec<Part>,
    inputs: Vec<TranscriptionInput>,
) -> TranscriptionSubmission {
    let inputs: BTreeMap<_, _> = inputs.into_iter().map(|i| (i.index, i.text)).collect();
    let mut all_blanks_filled = true;
    let request = parts
        .into_iter()
        .enumerate()
        .map(|(i, part)| match part {
            Part::Provided { part } => PartSubmitted::Provided { part },
            Part::AskedToTranscribe { parts } => {
                let submission = inputs.get(&i).map_or("", String::as_str).trim().to_string();
                all_blanks_filled &= has_text(&submission);
                PartSubmitted::AskedToTranscribe { parts, submission }
            }
        })
        .collect();
    TranscriptionSubmission {
        request,
        all_blanks_filled,
    }
}

#[bridgerton::bridge]
pub fn transcription_is_perfect(results: Vec<PartGraded>) -> bool {
    results.iter().all(|result| match result {
        PartGraded::Provided { .. } => true,
        PartGraded::AskedToTranscribe { parts, .. } => parts
            .iter()
            .all(|p| matches!(p.grade, WordGrade::Perfect { .. })),
    })
}

/// Operates on an owned snapshot: changing a grade must not mutate earlier
/// React state objects or a saved draft through a shared nested reference.
#[bridgerton::bridge]
pub fn apply_transcription_grade(
    mut results: Vec<PartGraded>,
    part_index: usize,
    word_index: usize,
    grade: WordGrade,
) -> Vec<PartGraded> {
    if let Some(PartGraded::AskedToTranscribe { parts, .. }) = results.get_mut(part_index)
        && let Some(part) = parts.get_mut(word_index)
    {
        part.grade = grade;
    }
    results
}

/// Offline/manual review uses the same fallback as a failed network grade.
#[bridgerton::bridge]
pub fn failed_transcription_review(
    submission: Vec<transcription_challenge::PartSubmitted>,
    course: Course,
) -> transcription_challenge::Grade {
    let mut grade = local_transcription_review(submission, course.target_language);
    grade.autograding_error = Some(yap_i18n::tr!(
        "transcription-the-llm-was-not-able-to-grade-this-transcription"
    ));
    grade
}

fn local_transcription_review(submission: Vec<PartSubmitted>, target_language: Language) -> Grade {
    let results = submission
        .into_iter()
        .map(|part| match part {
            transcription_challenge::PartSubmitted::AskedToTranscribe { parts, submission } => {
                if !target_language.uses_word_spaces() {
                    let normalize = |text: &str| {
                        normalize_for_grading(text, target_language)
                            .chars()
                            .filter(|c| !c.is_whitespace())
                            .collect::<String>()
                    };
                    let expected: String =
                        parts.iter().map(|part| part.word.text.as_str()).collect();
                    let perfect = normalize(&expected) == normalize(&submission);
                    return PartGraded::AskedToTranscribe {
                        parts: parts
                            .into_iter()
                            .map(|part| {
                                let grade = if perfect {
                                    WordGrade::Perfect {
                                        wrote: Some(part.word.text.clone()),
                                    }
                                } else {
                                    // Without word boundaries, don't guess which word was missed.
                                    WordGrade::Missed {}
                                };
                                transcription_challenge::PartGradedPart { heard: part, grade }
                            })
                            .collect(),
                        submission,
                    };
                }
                let submitted_words = submission.split_whitespace().collect::<Vec<_>>();
                if submitted_words.len() != parts.len() {
                    return transcription_challenge::PartGraded::AskedToTranscribe {
                        parts: parts
                            .iter()
                            .map(|part| transcription_challenge::PartGradedPart {
                                heard: part.clone(),
                                grade: transcription_challenge::WordGrade::Missed {},
                            })
                            .collect(),
                        submission: submission.clone(),
                    };
                }

                transcription_challenge::PartGraded::AskedToTranscribe {
                    parts: parts
                        .iter()
                        .zip(submitted_words.iter())
                        .map(|(part, &submission)| {
                            let part_text = normalize_for_grading(&part.word.text, target_language)
                                .trim()
                                .to_string();
                            let submission = normalize_for_grading(submission, target_language)
                                .trim()
                                .to_string();
                            if part_text == submission {
                                transcription_challenge::PartGradedPart {
                                    heard: part.clone(),
                                    grade: transcription_challenge::WordGrade::Perfect {
                                        wrote: Some(submission.to_string()),
                                    },
                                }
                            } else if remove_accents_lowercase(&part_text)
                                == remove_accents_lowercase(&submission)
                            {
                                transcription_challenge::PartGradedPart {
                                    heard: part.clone(),
                                    grade: transcription_challenge::WordGrade::CorrectWithTypo {
                                        wrote: Some(submission.to_string()),
                                    },
                                }
                            // todo: check if word entered is in the set of homophones
                            // and if so, grade is as correct PhoneticallyIdenticalButContextuallyIncorrect
                            } else {
                                transcription_challenge::PartGradedPart {
                                    heard: part.clone(),
                                    grade: transcription_challenge::WordGrade::Incorrect {
                                        wrote: Some(submission.to_string()),
                                    },
                                }
                            }
                        })
                        .collect(),
                    submission: submission.clone(),
                }
            }
            transcription_challenge::PartSubmitted::Provided { part } => {
                transcription_challenge::PartGraded::Provided { part }
            }
        })
        .collect();

    transcription_challenge::Grade {
        encouragement: None,
        explanation: None,
        results,
        compare: Vec::new(),
        autograding_error: None,
    }
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TranscriptionState {
    pub phase: TranscriptionPhase,
}

#[bridgerton::bridge(transparent, namespace)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum TranscriptionPhase {
    Editing,
    Grading {
        submission: Vec<TranscriptionInput>,
        completed_at_ms: f64,
    },
    Graded {
        submission: Vec<TranscriptionInput>,
        completed_at_ms: f64,
        grade: Grade,
        translation_revealed: bool,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        completing: bool,
    },
}

#[bridgerton::bridge(transparent, namespace)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum TranscriptionEvent {
    Submit {
        inputs: Vec<TranscriptionInput>,
        now_ms: f64,
    },
    CancelGrading,
    Graded {
        grade: Grade,
    },
    WordGradeChanged {
        part_index: usize,
        word_index: usize,
        grade: WordGrade,
    },
    TranslationToggled,
    Continue,
}

#[bridgerton::bridge(transparent, namespace)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum TranscriptionEffect {
    Autograde {
        submission: Vec<PartSubmitted>,
    },
    PlaySound {
        sound: Sound,
    },
    Complete {
        results: Vec<PartGraded>,
        completed_at_ms: f64,
    },
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TranscriptionStep {
    pub state: TranscriptionState,
    pub effects: Vec<TranscriptionEffect>,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum BlankTint {
    Neutral,
    Perfect,
    /// Right word, misspelled.
    Typo,
    PhoneticallyIdentical,
    PhoneticallySimilar,
    Wrong,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BlankView {
    pub index: usize,
    pub text: Option<String>,
    pub tint: BlankTint,
    pub editable: bool,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GradeOptionView {
    pub label: String,
    pub grade: WordGrade,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WordGradeView {
    pub part_index: usize,
    pub word_index: usize,
    pub heard: String,
    pub selected: usize,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct VerdictView {
    pub headline: VerdictHeadline,
    pub perfect: bool,
    pub submission_text: String,
    pub submission_label: String,
    pub continue_label: String,
    pub encouragement: Option<String>,
    pub explanation: Option<String>,
    pub autograding_error: Option<String>,
    pub compare: Vec<String>,
    pub compare_label: String,
    pub translation_label: String,
    pub translation_revealed: bool,
    pub word_grades_title: String,
    /// Collapsed unless the autograder failed and the words need grading by hand.
    pub word_grades_open_by_default: bool,
    pub word_grades: Vec<WordGradeView>,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TranscriptionView {
    pub sentence_ruby: Vec<language_utils::readings::RubySegment>,
    pub masked_sentence: Option<String>,
    pub proper_nouns: Vec<ProperNounGroup>,
    pub cant_listen_label: String,
    /// Shown from the moment grading starts, beside the sentence.
    pub correct_label: String,
    pub instructions: String,
    pub placeholder: String,
    pub blanks: Vec<BlankView>,
    pub chunks: Vec<language_utils::SentenceChunk>,
    pub can_continue: bool,
    pub submit_label: String,
    pub is_grading: bool,
    pub verdict: Option<VerdictView>,
    pub grade_options: Vec<GradeOptionView>,
}

#[bridgerton::bridge]
pub fn transcription_start() -> TranscriptionState {
    TranscriptionState {
        phase: TranscriptionPhase::Editing,
    }
}

#[bridgerton::bridge]
pub fn transcription_resume(
    state: TranscriptionState,
    challenge: TranscribeComprehensibleSentence,
) -> TranscriptionStep {
    let effects = if let TranscriptionPhase::Grading { submission, .. } = &state.phase {
        vec![TranscriptionEffect::Autograde {
            submission: prepare_transcription_submission(challenge.parts, submission.clone())
                .request,
        }]
    } else {
        transcription_completion(&state).into_iter().collect()
    };
    TranscriptionStep { state, effects }
}

fn transcription_completion(state: &TranscriptionState) -> Option<TranscriptionEffect> {
    let TranscriptionPhase::Graded {
        grade,
        completed_at_ms,
        completing: true,
        ..
    } = &state.phase
    else {
        return None;
    };
    Some(TranscriptionEffect::Complete {
        results: grade.results.clone(),
        completed_at_ms: *completed_at_ms,
    })
}

#[bridgerton::bridge]
pub fn transcription_transition(
    mut state: TranscriptionState,
    event: TranscriptionEvent,
    challenge: TranscribeComprehensibleSentence,
    target_language: Language,
) -> TranscriptionStep {
    let can_continue = matches!(event, TranscriptionEvent::Continue)
        && transcription_view(state.clone(), challenge.clone(), target_language).can_continue;
    let mut effects = vec![];
    match (&mut state.phase, event) {
        (
            TranscriptionPhase::Graded {
                completing: true, ..
            },
            _,
        ) => {}
        (TranscriptionPhase::Editing, TranscriptionEvent::Submit { inputs, now_ms }) => {
            let submission =
                prepare_transcription_submission(challenge.parts.clone(), inputs.clone());
            if submission.all_blanks_filled {
                state.phase = TranscriptionPhase::Grading {
                    submission: inputs,
                    completed_at_ms: now_ms,
                };
                let grade = local_transcription_review(submission.request.clone(), target_language);
                if transcription_is_perfect(grade.results.clone()) {
                    return transcription_transition(
                        state,
                        TranscriptionEvent::Graded { grade },
                        challenge,
                        target_language,
                    );
                }
                effects.push(TranscriptionEffect::Autograde {
                    submission: submission.request,
                });
            }
        }
        (TranscriptionPhase::Grading { .. }, TranscriptionEvent::CancelGrading) => {
            state.phase = TranscriptionPhase::Editing
        }
        (
            TranscriptionPhase::Grading {
                submission,
                completed_at_ms,
            },
            TranscriptionEvent::Graded { grade },
        ) => {
            effects.push(TranscriptionEffect::PlaySound {
                sound: Sound::AiDoneGrading,
            });
            if transcription_is_perfect(grade.results.clone()) {
                effects.push(TranscriptionEffect::PlaySound {
                    sound: Sound::Success,
                });
            }
            state.phase = TranscriptionPhase::Graded {
                submission: submission.clone(),
                completed_at_ms: *completed_at_ms,
                grade,
                translation_revealed: false,
                completing: false,
            };
        }
        (
            TranscriptionPhase::Graded { grade, .. },
            TranscriptionEvent::WordGradeChanged {
                part_index,
                word_index,
                grade: word_grade,
            },
        ) => {
            grade.results = apply_transcription_grade(
                std::mem::take(&mut grade.results),
                part_index,
                word_index,
                word_grade,
            );
        }
        (
            TranscriptionPhase::Graded {
                translation_revealed,
                ..
            },
            TranscriptionEvent::TranslationToggled,
        ) => *translation_revealed = !*translation_revealed,
        (TranscriptionPhase::Graded { completing, .. }, TranscriptionEvent::Continue)
            if can_continue =>
        {
            *completing = true;
            effects.extend(transcription_completion(&state));
        }
        _ => {}
    }
    TranscriptionStep { state, effects }
}

fn blank_tint(result: Option<&PartGraded>) -> BlankTint {
    let Some(PartGraded::AskedToTranscribe { parts, .. }) = result else {
        return BlankTint::Neutral;
    };
    if parts
        .iter()
        .all(|p| matches!(p.grade, WordGrade::Perfect { .. }))
    {
        BlankTint::Perfect
    } else if parts.iter().any(|p| {
        matches!(
            p.grade,
            WordGrade::PhoneticallyIdenticalButContextuallyIncorrect { .. }
        )
    }) {
        BlankTint::PhoneticallyIdentical
    } else if parts.iter().any(|p| {
        matches!(
            p.grade,
            WordGrade::PhoneticallySimilarButContextuallyIncorrect { .. }
        )
    }) {
        BlankTint::PhoneticallySimilar
    } else if parts.iter().any(|p| {
        matches!(
            p.grade,
            WordGrade::Incorrect { .. } | WordGrade::Missed { .. }
        )
    }) {
        BlankTint::Wrong
    } else {
        BlankTint::Typo
    }
}

#[bridgerton::bridge]
#[bridgerton::stable]
pub fn transcription_view(
    state: TranscriptionState,
    challenge: TranscribeComprehensibleSentence,
    language: Language,
) -> TranscriptionView {
    let grade_options: Vec<_> = [
        (
            yap_i18n::tr!("transcription-perfect"),
            WordGrade::Perfect { wrote: None },
        ),
        (
            yap_i18n::tr!("transcription-correct-with-typo"),
            WordGrade::CorrectWithTypo { wrote: None },
        ),
        (
            yap_i18n::tr!("transcription-phonetically-identical"),
            WordGrade::PhoneticallyIdenticalButContextuallyIncorrect { wrote: None },
        ),
        (
            yap_i18n::tr!("transcription-phonetically-similar"),
            WordGrade::PhoneticallySimilarButContextuallyIncorrect { wrote: None },
        ),
        (
            yap_i18n::tr!("transcription-incorrect"),
            WordGrade::Incorrect { wrote: None },
        ),
        (yap_i18n::tr!("transcription-missed"), WordGrade::Missed {}),
    ]
    .into_iter()
    .map(|(label, grade)| GradeOptionView { label, grade })
    .collect();
    let editing = matches!(state.phase, TranscriptionPhase::Editing);
    let grade = match &state.phase {
        TranscriptionPhase::Graded { grade, .. } => Some(grade),
        _ => None,
    };
    let submission = match &state.phase {
        TranscriptionPhase::Editing => None,
        TranscriptionPhase::Grading { submission, .. }
        | TranscriptionPhase::Graded { submission, .. } => Some(submission),
    };
    let blanks: Vec<_> = challenge
        .parts
        .iter()
        .enumerate()
        .filter(|&(_index, part)| matches!(part, Part::AskedToTranscribe { .. }))
        .map(|(index, _part)| BlankView {
            index,
            text: submission
                .and_then(|inputs| inputs.iter().rev().find(|input| input.index == index))
                .map(|input| input.text.clone()),
            tint: blank_tint(grade.and_then(|g| g.results.get(index))),
            editable: editing,
        })
        .collect();
    let verdict = if let TranscriptionPhase::Graded {
        grade,
        translation_revealed,
        ..
    } = &state.phase
    {
        let perfect = transcription_is_perfect(grade.results.clone());
        let only_typos = grade.results.iter().all(|result| match result {
            PartGraded::Provided { .. } => true,
            PartGraded::AskedToTranscribe { parts, .. } => parts.iter().all(|p| {
                matches!(
                    p.grade,
                    WordGrade::Perfect { .. } | WordGrade::CorrectWithTypo { .. }
                )
            }),
        });
        let mut word_grades = vec![];
        for (part_index, part) in grade.results.iter().enumerate() {
            if let PartGraded::AskedToTranscribe { parts, .. } = part {
                for (word_index, word) in parts.iter().enumerate() {
                    let selected = grade_options
                        .iter()
                        .position(|o| {
                            std::mem::discriminant(&o.grade) == std::mem::discriminant(&word.grade)
                        })
                        .expect("all word grades have an option");
                    word_grades.push(WordGradeView {
                        part_index,
                        word_index,
                        heard: word.heard.word.text.clone(),
                        selected,
                    });
                }
            }
        }
        Some(VerdictView {
            headline: if perfect {
                VerdictTone::Perfect
            } else if only_typos {
                VerdictTone::Almost
            } else {
                VerdictTone::Wrong
            }
            .into(),
            perfect,
            submission_text: blanks
                .iter()
                .filter_map(|b| b.text.as_deref())
                .collect::<Vec<_>>()
                .join(" "),
            submission_label: yap_i18n::tr!("transcription-your-answer"),
            continue_label: yap_i18n::tr!("common-continue"),
            encouragement: grade.encouragement.clone(),
            explanation: grade.explanation.clone(),
            autograding_error: grade.autograding_error.clone(),
            compare: grade.compare.clone(),
            compare_label: yap_i18n::tr!("transcription-listen"),
            translation_label: yap_i18n::tr!("transcription-english-translation-click-to-reveal"),
            translation_revealed: *translation_revealed,
            word_grades_title: yap_i18n::tr!("transcription-grade-words-manually"),
            word_grades_open_by_default: grade.autograding_error.is_some(),
            word_grades,
        })
    } else {
        None
    };
    let mut sentence_ruby = Vec::new();
    if !editing
        && challenge
            .literal_ruby
            .iter()
            .flatten()
            .any(|segment| segment.ruby.is_some())
    {
        let literals = challenge.parts.iter().flat_map(|part| match part {
            Part::AskedToTranscribe { parts } => parts.as_slice(),
            Part::Provided { part } => std::slice::from_ref(part),
        });
        for (i, literal) in literals.enumerate() {
            match challenge
                .literal_ruby
                .get(i)
                .filter(|ruby| !ruby.is_empty())
            {
                Some(ruby) => sentence_ruby.extend(ruby.clone()),
                None => sentence_ruby.push(language_utils::readings::RubySegment {
                    base: literal.word.text.clone(),
                    ruby: None,
                }),
            }
            if !literal.whitespace.as_str().is_empty() {
                sentence_ruby.push(language_utils::readings::RubySegment {
                    base: literal.whitespace.to_string(),
                    ruby: None,
                });
            }
        }
    }
    TranscriptionView {
        sentence_ruby,
        masked_sentence: editing
            .then(|| language_utils::transcription_challenge::masked_sentence(&challenge.parts)),
        proper_nouns: if editing {
            proper_noun_groups(&challenge.proper_noun_definitions)
        } else {
            vec![]
        },
        cant_listen_label: yap_i18n::tr!("transcription-can-t-listen-now"),
        correct_label: yap_i18n::tr!("transcription-correct-sentence"),
        instructions: yap_i18n::tr!("transcription-listen-and-fill-in-the-blanks"),
        placeholder: yap_i18n::tr!("transcription-write-what-you-hear"),
        chunks: transcription_sentence_chunks(challenge.parts, language),
        can_continue: matches!(
            state.phase,
            TranscriptionPhase::Graded {
                completing: false,
                ..
            }
        ),
        submit_label: if matches!(state.phase, TranscriptionPhase::Grading { .. }) {
            yap_i18n::tr!("common-grading")
        } else {
            yap_i18n::tr!("common-check-answer")
        },
        is_grading: matches!(state.phase, TranscriptionPhase::Grading { .. }),
        blanks,
        verdict,
        grade_options,
    }
}

#[cfg(test)]
mod reducer_tests {
    use super::*;
    use language_utils::{Literal, transcription_challenge::PartGradedPart};

    fn transition(state: TranscriptionState, event: TranscriptionEvent) -> TranscriptionStep {
        transcription_transition(state, event, challenge(), Language::French)
    }
    fn transcription_view(state: TranscriptionState) -> TranscriptionView {
        super::transcription_view(state, challenge(), Language::French)
    }
    fn transcription_resume(state: TranscriptionState) -> TranscriptionStep {
        super::transcription_resume(state, challenge())
    }
    fn submission(
        state: &TranscriptionState,
        challenge: &TranscribeComprehensibleSentence,
    ) -> TranscriptionSubmission {
        let (TranscriptionPhase::Grading { submission, .. }
        | TranscriptionPhase::Graded { submission, .. }) = &state.phase
        else {
            panic!("not submitted")
        };
        prepare_transcription_submission(challenge.parts.clone(), submission.clone())
    }

    fn literal() -> Literal<String> {
        serde_json::from_value(serde_json::json!({"word":{"text":"chat","word_type":{"type":"Heteronym","word":"chat","lemma":"chat","pos":"NOUN"}},"whitespace":""})).unwrap()
    }
    fn start() -> TranscriptionState {
        transcription_start()
    }
    fn challenge() -> TranscribeComprehensibleSentence {
        TranscribeComprehensibleSentence {
            target_language: "chatchat".into(),
            native_language: "cat cat".into(),
            audio: serde_json::from_value(serde_json::json!({"request": {"text":"chat", "language":"French", "is_ssml":false, "instructions":null, "speed":1.0, "verification_hints":[]}, "provider":"Google"})).unwrap(),
            literal_ruby: vec![],
            parts: vec![
                Part::Provided { part: literal() },
                Part::AskedToTranscribe { parts: vec![literal()] },
            ],
            part_gram_indices: vec![],
            gram_definitions_for_lookup: vec![],
            gram_breakdowns_for_lookup: vec![],
            movie_titles: vec![],
            proper_noun_definitions: vec![],
            second_chance: false,
        }
    }
    #[test]
    fn ruby_is_hidden_until_grading_and_spells_the_correct_sentence() {
        let mut word = literal();
        word.word.text = "猫".into();
        word.whitespace = language_utils::Whitespace::Space;
        let mut state = start();
        let mut challenge = challenge();
        challenge.literal_ruby = vec![
            vec![language_utils::readings::RubySegment {
                base: "猫".into(),
                ruby: Some("ねこ".into()),
            }],
            vec![],
        ];
        challenge.parts = vec![Part::AskedToTranscribe {
            parts: vec![word, literal()],
        }];
        let transcription_view =
            |state| super::transcription_view(state, challenge.clone(), Language::French);
        assert!(transcription_view(state.clone()).sentence_ruby.is_empty());
        state.phase = TranscriptionPhase::Grading {
            submission: vec![TranscriptionInput {
                index: 0,
                text: "cat cat".into(),
            }],
            completed_at_ms: 1.0,
        };
        let view = transcription_view(state);
        assert_eq!(
            view.sentence_ruby
                .iter()
                .map(|s| s.base.as_str())
                .collect::<String>(),
            "猫 chat"
        );
        assert_eq!(view.sentence_ruby[0].ruby.as_deref(), Some("ねこ"));
    }

    #[test]
    fn chunks_keep_blank_indices_and_attach_elisions_and_punctuation() {
        let mut prefix = literal();
        prefix.word.text = "qu'".into();
        let mut answer = literal();
        answer.word.text = "il".into();
        answer.whitespace = language_utils::Whitespace::NarrowNbsp;
        let mut punctuation = literal();
        punctuation.word.text = "?".into();
        let parts = vec![
            Part::Provided { part: prefix },
            Part::AskedToTranscribe {
                parts: vec![answer],
            },
            Part::Provided { part: punctuation },
        ];
        let chunks = transcription_sentence_chunks(parts, Language::French);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].indices, vec![0, 1, 2]);
        assert_eq!(chunks[0].trailing_whitespace, "");
    }

    #[test]
    fn captions_are_masked_only_while_editing() {
        assert_eq!(
            transcription_view(start()).masked_sentence.as_deref(),
            Some("chat____")
        );
        assert_eq!(transcription_view(grading()).masked_sentence, None);
        assert_eq!(transcription_view(graded()).masked_sentence, None);
    }

    #[test]
    fn proper_nouns_and_skip_copy_are_shared_and_hints_are_editing_only() {
        let definitions = vec![(
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
        let mut state = start();
        let mut challenge = challenge();
        challenge.proper_noun_definitions = definitions.clone();
        let transcription_view =
            |state| super::transcription_view(state, challenge.clone(), Language::French);
        let view = transcription_view(state.clone());
        assert_eq!(view.proper_nouns, proper_noun_groups(&definitions));
        assert_eq!(view.cant_listen_label, "Can't listen now");
        state.phase = grading().phase;
        assert!(transcription_view(state.clone()).proper_nouns.is_empty());
        state.phase = graded().phase;
        assert!(transcription_view(state).proper_nouns.is_empty());
    }

    #[test]
    fn editing_state_contains_only_phase() {
        let json = serde_json::to_value(start()).unwrap();
        assert_eq!(json, serde_json::json!({"phase":{"type":"Editing"}}));
        let state: TranscriptionState = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(serde_json::to_value(state).unwrap(), json);
    }

    fn inputs() -> Vec<TranscriptionInput> {
        vec![TranscriptionInput {
            index: 1,
            text: " chien ".into(),
        }]
    }
    fn grading() -> TranscriptionState {
        transition(
            start(),
            TranscriptionEvent::Submit {
                inputs: inputs(),
                now_ms: 1234.0,
            },
        )
        .state
    }
    fn answer(
        language: Language,
        words: &[&str],
        submitted: &str,
        second: &str,
    ) -> (TranscriptionStep, TranscribeComprehensibleSentence) {
        let parts = words
            .iter()
            .map(|text| {
                let mut part = literal();
                part.word.text = (*text).into();
                part
            })
            .collect();
        let mut challenge = challenge();
        challenge.parts = vec![
            Part::AskedToTranscribe { parts },
            Part::Provided { part: literal() },
            Part::AskedToTranscribe {
                parts: vec![literal()],
            },
        ];
        let step = transcription_transition(
            start(),
            TranscriptionEvent::Submit {
                inputs: vec![
                    TranscriptionInput {
                        index: 0,
                        text: submitted.into(),
                    },
                    TranscriptionInput {
                        index: 2,
                        text: second.into(),
                    },
                ],
                now_ms: 1234.0,
            },
            challenge.clone(),
            language,
        );
        (step, challenge)
    }

    #[test]
    fn exact_answers_grade_locally_and_complete_like_ai_answers() {
        for (language, words, submitted) in [
            (Language::Thai, vec!["สวัสดี", "ครับ"], "สวัสดีครับ"),
            (Language::Thai, vec!["สวัสดี", "ครับ"], "สวัสดี \tครับ"),
            (Language::Thai, vec!["คุณ", "ได้", "ไป"], "คุณได้ไป"),
            (Language::Thai, vec!["นาย", "ครับ"], "นายครับ"),
            (Language::French, vec!["un", "chat"], "Un chat!"),
        ] {
            let (step, challenge) = answer(language, &words, submitted, "chat");
            assert_eq!(
                step.effects,
                vec![
                    TranscriptionEffect::PlaySound {
                        sound: Sound::AiDoneGrading
                    },
                    TranscriptionEffect::PlaySound {
                        sound: Sound::Success
                    },
                ]
            );
            let TranscriptionPhase::Graded {
                grade,
                completed_at_ms,
                translation_revealed,
                completing,
                ..
            } = &step.state.phase
            else {
                panic!("expected local grade")
            };
            assert_eq!(*completed_at_ms, 1234.0);
            assert!(!translation_revealed && !completing);
            assert_eq!(grade.autograding_error, None);
            assert_eq!(grade.encouragement, None);
            assert_eq!(grade.explanation, None);
            assert!(grade.compare.is_empty());
            assert!(matches!(&grade.results[1], PartGraded::Provided { .. }));
            let PartGraded::AskedToTranscribe { parts, submission } = &grade.results[0] else {
                panic!()
            };
            assert_eq!(submission, submitted);
            assert_eq!(parts.len(), words.len());
            for (part, expected) in parts.iter().zip(&words) {
                assert_eq!(part.heard.word.text, *expected);
                assert_eq!(
                    part.grade,
                    WordGrade::Perfect {
                        wrote: Some((*expected).into())
                    }
                );
            }
            let complete = transcription_transition(
                step.state.clone(),
                TranscriptionEvent::Continue,
                challenge.clone(),
                language,
            );
            assert_eq!(
                complete.effects,
                vec![TranscriptionEffect::Complete {
                    results: grade.results.clone(),
                    completed_at_ms: 1234.0
                }]
            );
            assert!(
                transcription_transition(
                    complete.state,
                    TranscriptionEvent::Continue,
                    challenge,
                    language
                )
                .effects
                .is_empty()
            );
        }
    }

    #[test]
    fn nonperfect_answers_still_request_ai_for_the_whole_submission() {
        for (language, words, submitted, second) in [
            (Language::Thai, vec!["สวัสดี", "ครับ"], "สวัสดีค่ะ", "chat"),
            (Language::Thai, vec!["สวัสดี", "ครับ"], "สวัสดีครับ", "chien"),
            (Language::French, vec!["été"], "ete", "chat"),
            (Language::French, vec!["un", "chat"], "unchat", "chat"),
            (Language::French, vec!["chat"], "chat chat", "chat"),
            (Language::French, vec!["chat"], "chat", "chien"),
        ] {
            let (step, challenge) = answer(language, &words, submitted, second);
            assert!(matches!(
                step.state.phase,
                TranscriptionPhase::Grading {
                    completed_at_ms: 1234.0,
                    ..
                }
            ));
            assert_eq!(
                step.effects,
                vec![TranscriptionEffect::Autograde {
                    submission: submission(&step.state, &challenge).request
                }]
            );
        }
    }

    #[test]
    fn fallback_shares_local_grades_but_reports_failure() {
        for (language, words, submitted, expected_grade) in [
            (
                Language::Thai,
                vec!["สวัสดี", "ครับ"],
                "สวัสดีครับ",
                WordGrade::Perfect {
                    wrote: Some("สวัสดี".into()),
                },
            ),
            (
                Language::Thai,
                vec!["สวัสดี", "ครับ"],
                "สวัสดีค่ะ",
                WordGrade::Missed {},
            ),
            (
                Language::French,
                vec!["été"],
                "ete",
                WordGrade::CorrectWithTypo {
                    wrote: Some("ete".into()),
                },
            ),
            (
                Language::French,
                vec!["un", "chat"],
                "unchat",
                WordGrade::Missed {},
            ),
        ] {
            let (step, challenge) = answer(language, &words, submitted, "chat");
            let grade = failed_transcription_review(
                submission(&step.state, &challenge).request,
                Course {
                    target_language: language,
                    native_language: Language::English,
                },
            );
            assert_eq!(
                grade.autograding_error.as_deref(),
                Some("The LLM was not able to grade this transcription")
            );
            let PartGraded::AskedToTranscribe { parts, .. } = &grade.results[0] else {
                panic!()
            };
            assert_eq!(parts[0].grade, expected_grade);
        }
    }

    fn grade(grades: Vec<WordGrade>) -> Grade {
        Grade {
            results: vec![
                PartGraded::Provided { part: literal() },
                PartGraded::AskedToTranscribe {
                    parts: grades
                        .into_iter()
                        .map(|grade| PartGradedPart {
                            heard: literal(),
                            grade,
                        })
                        .collect(),
                    submission: "chat".into(),
                },
            ],
            encouragement: Some("Good!".into()),
            explanation: None,
            compare: vec!["chat".into()],
            autograding_error: None,
        }
    }
    fn graded() -> TranscriptionState {
        transition(
            grading(),
            TranscriptionEvent::Graded {
                grade: grade(vec![WordGrade::Perfect {
                    wrote: Some("chat".into()),
                }]),
            },
        )
        .state
    }
    fn unchanged(state: &TranscriptionState, event: TranscriptionEvent) {
        assert_eq!(
            transition(state.clone(), event),
            TranscriptionStep {
                state: state.clone(),
                effects: vec![]
            }
        );
    }

    #[test]
    fn editing_validates_blanks_and_submit_is_single_shot() {
        let state = start();
        assert!(
            transcription_view(state.clone())
                .blanks
                .iter()
                .all(|b| b.text.is_none())
        );
        for inputs in [
            vec![],
            vec![TranscriptionInput {
                index: 1,
                text: " \t".into(),
            }],
            vec![TranscriptionInput {
                index: 0,
                text: "ignored".into(),
            }],
        ] {
            unchanged(
                &state,
                TranscriptionEvent::Submit {
                    inputs,
                    now_ms: 1.0,
                },
            );
        }
        assert!(!transcription_can_submit(vec![" \t".into()]));
        assert!(transcription_can_submit(vec![" chien ".into()]));
        let step = transition(
            state,
            TranscriptionEvent::Submit {
                inputs: inputs(),
                now_ms: 1234.0,
            },
        );
        assert_eq!(step.state, grading());
        assert!(
            matches!(&step.effects[..], [TranscriptionEffect::Autograde { submission }] if matches!(&submission[1], PartSubmitted::AskedToTranscribe { submission, .. } if submission == "chien"))
        );
        unchanged(
            &step.state,
            TranscriptionEvent::Submit {
                inputs: inputs(),
                now_ms: 5678.0,
            },
        );
        let view = transcription_view(step.state);
        assert!(view.is_grading);
        assert!(view.blanks.iter().all(|b| b.text.is_some()));
        assert!(view.blanks.iter().all(|b| !b.editable));
    }

    #[test]
    fn submit_requires_every_blank_including_missing_inputs() {
        let mut challenge = challenge();
        challenge.parts.push(Part::AskedToTranscribe {
            parts: vec![literal()],
        });
        for inputs in [
            inputs(),
            vec![
                TranscriptionInput {
                    index: 1,
                    text: "chien".into(),
                },
                TranscriptionInput {
                    index: 2,
                    text: " \n".into(),
                },
            ],
        ] {
            assert!(
                !prepare_transcription_submission(challenge.parts.clone(), inputs.clone())
                    .all_blanks_filled
            );
            let step = transcription_transition(
                start(),
                TranscriptionEvent::Submit {
                    inputs,
                    now_ms: 1.0,
                },
                challenge.clone(),
                Language::French,
            );
            assert_eq!(step.state, start());
            assert!(step.effects.is_empty());
        }
    }

    #[test]
    fn phase_inappropriate_events_have_no_effects() {
        for state in [start(), grading()] {
            unchanged(&state, TranscriptionEvent::Continue);
            unchanged(&state, TranscriptionEvent::TranslationToggled);
            unchanged(
                &state,
                TranscriptionEvent::WordGradeChanged {
                    part_index: 1,
                    word_index: 0,
                    grade: WordGrade::Missed {},
                },
            );
        }
        for state in [start(), graded()] {
            unchanged(
                &state,
                TranscriptionEvent::Graded {
                    grade: grade(vec![]),
                },
            );
            unchanged(&state, TranscriptionEvent::CancelGrading);
        }
        for state in [grading(), graded()] {
            unchanged(
                &state,
                TranscriptionEvent::Submit {
                    inputs: inputs(),
                    now_ms: 9.0,
                },
            );
        }
    }

    #[test]
    fn grading_sounds_overrides_translation_and_completion() {
        for (word, sounds) in [
            (
                WordGrade::Perfect { wrote: None },
                vec![Sound::AiDoneGrading, Sound::Success],
            ),
            (WordGrade::Missed {}, vec![Sound::AiDoneGrading]),
        ] {
            let step = transition(
                grading(),
                TranscriptionEvent::Graded {
                    grade: grade(vec![word]),
                },
            );
            assert_eq!(
                step.effects,
                sounds
                    .into_iter()
                    .map(|sound| TranscriptionEffect::PlaySound { sound })
                    .collect::<Vec<_>>()
            );
        }
        let original = graded();
        let changed = transition(
            original.clone(),
            TranscriptionEvent::WordGradeChanged {
                part_index: 1,
                word_index: 0,
                grade: WordGrade::Missed {},
            },
        );
        assert!(changed.effects.is_empty());
        assert!(transcription_view(original).verdict.unwrap().perfect);
        let view = transcription_view(changed.state.clone());
        assert!(!view.verdict.as_ref().unwrap().perfect);
        assert_eq!(view.verdict.as_ref().unwrap().continue_label, "Continue");
        assert_eq!(view.verdict.unwrap().word_grades[0].selected, 5);
        unchanged(
            &changed.state,
            TranscriptionEvent::WordGradeChanged {
                part_index: 99,
                word_index: 0,
                grade: WordGrade::Perfect { wrote: None },
            },
        );
        let toggled = transition(changed.state, TranscriptionEvent::TranslationToggled);
        assert!(toggled.effects.is_empty());
        assert!(
            transcription_view(toggled.state.clone())
                .verdict
                .unwrap()
                .translation_revealed
        );
        let complete = transition(toggled.state.clone(), TranscriptionEvent::Continue);
        let before_view = transcription_view(toggled.state.clone());
        let mut expected = toggled.state;
        if let TranscriptionPhase::Graded { completing, .. } = &mut expected.phase {
            *completing = true;
        }
        assert_eq!(complete.state, expected);
        let mut completing_view = transcription_view(complete.state.clone());
        assert!(!completing_view.can_continue);
        completing_view.can_continue = true;
        assert_eq!(completing_view, before_view);
        for event in [
            TranscriptionEvent::Continue,
            TranscriptionEvent::TranslationToggled,
            TranscriptionEvent::WordGradeChanged {
                part_index: 1,
                word_index: 0,
                grade: WordGrade::Perfect { wrote: None },
            },
        ] {
            unchanged(&complete.state, event);
        }
        let restored =
            serde_json::from_str(&serde_json::to_string(&complete.state).unwrap()).unwrap();
        assert_eq!(transcription_resume(restored), complete);
        assert!(
            matches!(&complete.effects[..], [TranscriptionEffect::Complete { completed_at_ms: 1234.0, results }] if matches!(&results[1], PartGraded::AskedToTranscribe { parts, .. } if matches!(parts[0].grade, WordGrade::Missed {})))
        );
    }

    #[test]
    fn persistence_resume_retains_submission_and_cancel_discards_it() {
        for state in [start(), graded()] {
            assert_eq!(
                transcription_resume(state.clone()),
                TranscriptionStep {
                    state,
                    effects: vec![]
                }
            );
        }
        let state: TranscriptionState =
            serde_json::from_str(&serde_json::to_string(&grading()).unwrap()).unwrap();
        let resumed = transcription_resume(state);
        assert_eq!(resumed.state, grading());
        assert_eq!(
            resumed.effects,
            transition(
                start(),
                TranscriptionEvent::Submit {
                    inputs: inputs(),
                    now_ms: 1234.0
                }
            )
            .effects
        );
        let cancelled = transition(resumed.state, TranscriptionEvent::CancelGrading);
        assert_eq!(cancelled.state, start());
        assert!(cancelled.effects.is_empty());
        unchanged(
            &cancelled.state,
            TranscriptionEvent::Graded {
                grade: grade(vec![]),
            },
        );
    }

    #[test]
    fn view_contract_and_tint_priority_match_web_including_typos() {
        let view = transcription_view(graded());
        assert_eq!(view.instructions, "Listen and fill in the blanks");
        assert_eq!(view.placeholder, "Write what you hear");
        assert_eq!(view.submit_label, "Check Answer");
        assert_eq!(view.blanks.len(), 1);
        assert_eq!(view.blanks[0].index, 1);
        let verdict = view.verdict.unwrap();
        assert_eq!(verdict.submission_text, " chien ");
        assert_eq!(verdict.headline.text, "Nailed it!");
        assert_eq!(verdict.continue_label, "Continue");
        assert_eq!(verdict.word_grades[0].selected, 0); // wrote does not affect selection
        assert_eq!(
            view.grade_options
                .iter()
                .map(|o| o.label.as_str())
                .collect::<Vec<_>>(),
            [
                "Perfect",
                "Correct with typo",
                "Phonetically identical",
                "Phonetically similar",
                "Incorrect",
                "Missed"
            ]
        );
        let cases = [
            (vec![WordGrade::Perfect { wrote: None }], BlankTint::Perfect),
            (
                vec![
                    WordGrade::Perfect { wrote: None },
                    WordGrade::CorrectWithTypo { wrote: None },
                ],
                BlankTint::Typo,
            ),
            (
                vec![
                    WordGrade::Missed {},
                    WordGrade::PhoneticallySimilarButContextuallyIncorrect { wrote: None },
                    WordGrade::PhoneticallyIdenticalButContextuallyIncorrect { wrote: None },
                ],
                BlankTint::PhoneticallyIdentical,
            ),
            (
                vec![
                    WordGrade::Incorrect { wrote: None },
                    WordGrade::PhoneticallySimilarButContextuallyIncorrect { wrote: None },
                ],
                BlankTint::PhoneticallySimilar,
            ),
            (
                vec![
                    WordGrade::CorrectWithTypo { wrote: None },
                    WordGrade::Missed {},
                ],
                BlankTint::Wrong,
            ),
        ];
        for (words, tint) in cases {
            let state = transition(
                grading(),
                TranscriptionEvent::Graded {
                    grade: grade(words),
                },
            )
            .state;
            assert_eq!(transcription_view(state).blanks[0].tint, tint);
        }
    }

    #[test]
    fn headline_forgives_only_typos() {
        let cases = [
            (
                vec![WordGrade::Perfect { wrote: None }],
                VerdictTone::Perfect,
            ),
            (
                vec![
                    WordGrade::Perfect { wrote: None },
                    WordGrade::CorrectWithTypo { wrote: None },
                ],
                VerdictTone::Almost,
            ),
            (
                vec![
                    WordGrade::CorrectWithTypo { wrote: None },
                    WordGrade::PhoneticallyIdenticalButContextuallyIncorrect { wrote: None },
                ],
                VerdictTone::Wrong,
            ),
        ];
        for (words, tone) in cases {
            let state = transition(
                grading(),
                TranscriptionEvent::Graded {
                    grade: grade(words),
                },
            )
            .state;
            assert_eq!(
                transcription_view(state).verdict.unwrap().headline.tone,
                tone
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use language_utils::{Literal, transcription_challenge::PartGradedPart};
    fn literal() -> Literal<String> {
        serde_json::from_value(serde_json::json!({"word":{"text":"chat","word_type":{"type":"Heteronym","word":"chat","lemma":"chat","pos":"NOUN"}},"whitespace":""})).unwrap()
    }
    fn result(grades: Vec<WordGrade>) -> Vec<PartGraded> {
        vec![PartGraded::AskedToTranscribe {
            parts: grades
                .into_iter()
                .map(|grade| PartGradedPart {
                    heard: literal(),
                    grade,
                })
                .collect(),
            submission: "answer".into(),
        }]
    }
    #[test]
    fn offline_fallback_preserves_parts_and_allows_manual_correction() {
        let course = language_utils::Course {
            target_language: language_utils::Language::French,
            native_language: language_utils::Language::English,
        };
        let submission = prepare_transcription_submission(
            vec![
                Part::Provided { part: literal() },
                Part::AskedToTranscribe {
                    parts: vec![literal()],
                },
            ],
            vec![TranscriptionInput {
                index: 1,
                text: "chien".into(),
            }],
        );
        let grade = failed_transcription_review(submission.request, course);
        assert!(grade.autograding_error.is_some());
        assert!(matches!(&grade.results[0], PartGraded::Provided { .. }));
        assert!(!transcription_is_perfect(grade.results.clone()));
        assert!(transcription_is_perfect(apply_transcription_grade(
            grade.results,
            1,
            0,
            WordGrade::Perfect { wrote: None },
        )));
    }

    #[test]
    fn only_requested_parts_need_answers_and_whitespace_matches_web() {
        let parts = vec![
            Part::Provided { part: literal() },
            Part::AskedToTranscribe {
                parts: vec![literal()],
            },
        ];
        assert!(!prepare_transcription_submission(parts.clone(), vec![]).all_blanks_filled);
        let input = prepare_transcription_submission(
            parts,
            vec![TranscriptionInput {
                index: 1,
                text: " chat \u{a0}".into(),
            }],
        );
        assert!(input.all_blanks_filled);
        assert!(
            matches!(&input.request[1], PartSubmitted::AskedToTranscribe { submission, .. } if submission == "chat")
        );
    }
    #[test]
    fn corrections_leave_prior_snapshot_unchanged_and_ignore_invalid_indices() {
        let before = result(vec![WordGrade::Incorrect {
            wrote: Some("chien".into()),
        }]);
        let after =
            apply_transcription_grade(before.clone(), 0, 0, WordGrade::Perfect { wrote: None });
        assert!(!transcription_is_perfect(before.clone()));
        assert!(transcription_is_perfect(after));
        assert_eq!(
            apply_transcription_grade(before.clone(), 2, 0, WordGrade::Missed {}),
            before
        );
    }
}
