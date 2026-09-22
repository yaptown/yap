use crate::{ProperNounGroup, Sound, proper_noun_groups};
use language_utils::{
    Course, ProperNounDefinition,
    text_cleanup::{normalize_for_grading, remove_accents_lowercase},
    transcription_challenge::{self, Grade, Part, PartGraded, PartSubmitted, WordGrade},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize)]
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
                all_blanks_filled &= !submission.is_empty();
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
    let results = submission
        .into_iter()
        .map(|part| match part {
            transcription_challenge::PartSubmitted::AskedToTranscribe { parts, submission } => {
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
                            let part_text =
                                normalize_for_grading(&part.word.text, course.target_language)
                                    .trim()
                                    .to_string();
                            let submission =
                                normalize_for_grading(submission, course.target_language)
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
        autograding_error: Some("The LLM was not able to grade this transcription".to_string()),
    }
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TranscriptionState {
    pub parts: Vec<Part>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proper_noun_definitions: Vec<(String, ProperNounDefinition)>,
    pub inputs: BTreeMap<usize, String>,
    pub phase: TranscriptionPhase,
}

#[bridgerton::bridge(transparent, namespace)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum TranscriptionPhase {
    Editing,
    Grading {
        completed_at_ms: f64,
    },
    Graded {
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
    InputChanged {
        index: usize,
        text: String,
    },
    Submit {
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
    PhoneticallyIdentical,
    PhoneticallySimilar,
    Wrong,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BlankView {
    pub index: usize,
    pub text: String,
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
    pub perfect: bool,
    pub submission_text: String,
    pub correct_label: String,
    pub submission_label: String,
    pub continue_label: String,
    pub encouragement: Option<String>,
    pub explanation: Option<String>,
    pub autograding_error: Option<String>,
    pub compare: Vec<String>,
    pub translation_revealed: bool,
    pub word_grades: Vec<WordGradeView>,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TranscriptionView {
    pub proper_nouns: Vec<ProperNounGroup>,
    pub cant_listen_label: String,
    pub instructions: String,
    pub placeholder: String,
    pub blanks: Vec<BlankView>,
    pub can_submit: bool,
    pub can_continue: bool,
    pub submit_label: String,
    pub is_grading: bool,
    pub verdict: Option<VerdictView>,
    pub grade_options: Vec<GradeOptionView>,
}

impl TranscriptionState {
    fn submission(&self) -> TranscriptionSubmission {
        prepare_transcription_submission(
            self.parts.clone(),
            self.inputs
                .iter()
                .map(|(&index, text)| TranscriptionInput {
                    index,
                    text: text.clone(),
                })
                .collect(),
        )
    }
}

#[bridgerton::bridge]
pub fn transcription_start(
    parts: Vec<Part>,
    proper_noun_definitions: Vec<(String, ProperNounDefinition)>,
) -> TranscriptionState {
    TranscriptionState {
        parts,
        proper_noun_definitions,
        inputs: BTreeMap::new(),
        phase: TranscriptionPhase::Editing,
    }
}

#[bridgerton::bridge]
pub fn transcription_resume(state: TranscriptionState) -> TranscriptionStep {
    let effects = if matches!(state.phase, TranscriptionPhase::Grading { .. }) {
        vec![TranscriptionEffect::Autograde {
            submission: state.submission().request,
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
) -> TranscriptionStep {
    let can_continue = matches!(event, TranscriptionEvent::Continue)
        && transcription_view(state.clone()).can_continue;
    let mut effects = vec![];
    match (&mut state.phase, event) {
        (
            TranscriptionPhase::Graded {
                completing: true, ..
            },
            _,
        ) => {}
        (TranscriptionPhase::Editing, TranscriptionEvent::InputChanged { index, text }) => {
            if matches!(state.parts.get(index), Some(Part::AskedToTranscribe { .. })) {
                state.inputs.insert(index, text);
            }
        }
        (TranscriptionPhase::Editing, TranscriptionEvent::Submit { now_ms }) => {
            let submission = state.submission();
            if submission.all_blanks_filled {
                state.phase = TranscriptionPhase::Grading {
                    completed_at_ms: now_ms,
                };
                effects.push(TranscriptionEffect::Autograde {
                    submission: submission.request,
                });
            }
        }
        (TranscriptionPhase::Grading { .. }, TranscriptionEvent::CancelGrading) => {
            state.phase = TranscriptionPhase::Editing
        }
        (TranscriptionPhase::Grading { completed_at_ms }, TranscriptionEvent::Graded { grade }) => {
            effects.push(TranscriptionEffect::PlaySound {
                sound: Sound::AiDoneGrading,
            });
            if transcription_is_perfect(grade.results.clone()) {
                effects.push(TranscriptionEffect::PlaySound {
                    sound: Sound::Success,
                });
            }
            state.phase = TranscriptionPhase::Graded {
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
        BlankTint::Neutral
    } // Typos alone have no error tint on the web.
}

#[bridgerton::bridge]
pub fn transcription_view(state: TranscriptionState) -> TranscriptionView {
    let grade_options: Vec<_> = [
        ("Perfect", WordGrade::Perfect { wrote: None }),
        (
            "Correct with typo",
            WordGrade::CorrectWithTypo { wrote: None },
        ),
        (
            "Phonetically identical",
            WordGrade::PhoneticallyIdenticalButContextuallyIncorrect { wrote: None },
        ),
        (
            "Phonetically similar",
            WordGrade::PhoneticallySimilarButContextuallyIncorrect { wrote: None },
        ),
        ("Incorrect", WordGrade::Incorrect { wrote: None }),
        ("Missed", WordGrade::Missed {}),
    ]
    .into_iter()
    .map(|(label, grade)| GradeOptionView {
        label: label.into(),
        grade,
    })
    .collect();
    let editing = matches!(state.phase, TranscriptionPhase::Editing);
    let grade = match &state.phase {
        TranscriptionPhase::Graded { grade, .. } => Some(grade),
        _ => None,
    };
    let blanks: Vec<_> = state
        .parts
        .iter()
        .enumerate()
        .filter(|&(_index, part)| matches!(part, Part::AskedToTranscribe { .. }))
        .map(|(index, _part)| BlankView {
            index,
            text: state.inputs.get(&index).cloned().unwrap_or_default(),
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
            perfect,
            submission_text: blanks
                .iter()
                .map(|b| b.text.as_str())
                .collect::<Vec<_>>()
                .join(" "),
            correct_label: "Correct sentence:".into(),
            submission_label: "Your answer:".into(),
            continue_label: if perfect { "Nailed it!" } else { "Continue" }.into(),
            encouragement: grade.encouragement.clone(),
            explanation: grade.explanation.clone(),
            autograding_error: grade.autograding_error.clone(),
            compare: grade.compare.clone(),
            translation_revealed: *translation_revealed,
            word_grades,
        })
    } else {
        None
    };
    TranscriptionView {
        proper_nouns: if editing {
            proper_noun_groups(&state.proper_noun_definitions)
        } else {
            vec![]
        },
        cant_listen_label: "Can't listen now".into(),
        instructions: "Listen and fill in the blanks".into(),
        placeholder: "Write what you hear".into(),
        can_submit: editing && state.submission().all_blanks_filled,
        can_continue: matches!(
            state.phase,
            TranscriptionPhase::Graded {
                completing: false,
                ..
            }
        ),
        submit_label: "Check answer".into(),
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

    fn literal() -> Literal<String> {
        serde_json::from_value(serde_json::json!({"word":{"text":"chat","word_type":{"type":"Heteronym","word":"chat","lemma":"chat","pos":"NOUN"}},"whitespace":""})).unwrap()
    }
    fn start() -> TranscriptionState {
        transcription_start(
            vec![
                Part::Provided { part: literal() },
                Part::AskedToTranscribe {
                    parts: vec![literal()],
                },
            ],
            vec![],
        )
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
        let mut state = transcription_start(start().parts, definitions.clone());
        let view = transcription_view(state.clone());
        assert_eq!(view.proper_nouns, proper_noun_groups(&definitions));
        assert_eq!(view.cant_listen_label, "Can't listen now");
        state.phase = grading().phase;
        assert!(transcription_view(state.clone()).proper_nouns.is_empty());
        state.phase = graded().phase;
        assert!(transcription_view(state).proper_nouns.is_empty());
    }

    #[test]
    fn old_state_without_proper_nouns_still_deserializes() {
        let mut json = serde_json::to_value(start()).unwrap();
        json.as_object_mut()
            .unwrap()
            .remove("proper_noun_definitions");
        let state: TranscriptionState = serde_json::from_value(json.clone()).unwrap();
        assert!(state.proper_noun_definitions.is_empty());
        assert_eq!(serde_json::to_value(state).unwrap(), json);
    }

    fn filled() -> TranscriptionState {
        transcription_transition(
            start(),
            TranscriptionEvent::InputChanged {
                index: 1,
                text: " chat ".into(),
            },
        )
        .state
    }
    fn grading() -> TranscriptionState {
        transcription_transition(filled(), TranscriptionEvent::Submit { now_ms: 1234.0 }).state
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
        transcription_transition(
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
            transcription_transition(state.clone(), event),
            TranscriptionStep {
                state: state.clone(),
                effects: vec![]
            }
        );
    }

    #[test]
    fn editing_validates_blanks_and_submit_is_single_shot() {
        let state = start();
        assert!(!transcription_view(state.clone()).can_submit);
        unchanged(&state, TranscriptionEvent::Submit { now_ms: 1.0 });
        for index in [0, 99] {
            unchanged(
                &state,
                TranscriptionEvent::InputChanged {
                    index,
                    text: "ignored".into(),
                },
            );
        }
        let whitespace = transcription_transition(
            state,
            TranscriptionEvent::InputChanged {
                index: 1,
                text: " \t".into(),
            },
        )
        .state;
        unchanged(&whitespace, TranscriptionEvent::Submit { now_ms: 1.0 });
        let state = filled();
        assert!(transcription_view(state.clone()).can_submit);
        let step = transcription_transition(state, TranscriptionEvent::Submit { now_ms: 1234.0 });
        assert_eq!(step.state, grading());
        assert!(
            matches!(&step.effects[..], [TranscriptionEffect::Autograde { submission }] if matches!(&submission[1], PartSubmitted::AskedToTranscribe { submission, .. } if submission == "chat"))
        );
        unchanged(&step.state, TranscriptionEvent::Submit { now_ms: 5678.0 });
        let view = transcription_view(step.state);
        assert!(view.is_grading);
        assert!(!view.can_submit);
        assert!(view.blanks.iter().all(|b| !b.editable));
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
                TranscriptionEvent::InputChanged {
                    index: 1,
                    text: "ignored".into(),
                },
            );
            unchanged(&state, TranscriptionEvent::Submit { now_ms: 9.0 });
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
            let step = transcription_transition(
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
        let changed = transcription_transition(
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
        let toggled =
            transcription_transition(changed.state, TranscriptionEvent::TranslationToggled);
        assert!(toggled.effects.is_empty());
        assert!(
            transcription_view(toggled.state.clone())
                .verdict
                .unwrap()
                .translation_revealed
        );
        let complete =
            transcription_transition(toggled.state.clone(), TranscriptionEvent::Continue);
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
    fn persistence_resume_and_cancel_preserve_inputs_and_timestamp() {
        for state in [start(), filled(), graded()] {
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
            transcription_transition(filled(), TranscriptionEvent::Submit { now_ms: 1234.0 })
                .effects
        );
        let cancelled = transcription_transition(resumed.state, TranscriptionEvent::CancelGrading);
        assert_eq!(cancelled.state, filled());
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
        assert_eq!(view.submit_label, "Check answer");
        assert_eq!(view.blanks.len(), 1);
        assert_eq!(view.blanks[0].index, 1);
        let verdict = view.verdict.unwrap();
        assert_eq!(verdict.submission_text, " chat ");
        assert_eq!(verdict.continue_label, "Nailed it!");
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
                BlankTint::Neutral,
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
            let state = transcription_transition(
                grading(),
                TranscriptionEvent::Graded {
                    grade: grade(words),
                },
            )
            .state;
            assert_eq!(transcription_view(state).blanks[0].tint, tint);
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
