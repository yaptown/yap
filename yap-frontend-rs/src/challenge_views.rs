//! Shared self-graded challenge presentation, matching the web.

use language_utils::{Language, PartOfSpeech, PatternPosition, PronunciationGuide};
use serde::{Deserialize, Serialize};

use crate::{
    CardContent, FlashCard, PronunciationCue, get_flashcard_disclosure,
    learning_metadata::get_language_metadata, should_show_challenge_tutorial,
};

/// A prompt whose optional target span is rendered in the target language.
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TutorialPrompt {
    pub before: String,
    pub target: Option<String>,
    pub after: String,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FlashcardView {
    pub subtitle: Option<String>,
    pub tutorial_prompt: Option<TutorialPrompt>,
    pub tutorial_hidden_hint: Option<String>,
    pub tutorial_revealed_hint: Option<String>,
    pub reveal_label: String,
    pub require_answer_reveal: bool,
    pub listening_header: Option<String>,
    pub known_label: String,
    pub again_label: String,
    pub remembered_label: String,
    pub cant_listen_label: Option<String>,
    /// The rest are set only when a written card has several meanings, which
    /// the learner grades one by one: a badge on meanings they haven't
    /// reviewed yet, the grade buttons once some meanings are marked (they
    /// apply to the unmarked ones), and the single button once all are.
    pub new_label: Option<String>,
    pub again_rest_label: Option<String>,
    pub remembered_rest_label: Option<String>,
    pub continue_label: Option<String>,
}

fn grade_labels(is_new: bool) -> (String, String) {
    if is_new {
        ("Didn't know".into(), "Already knew".into())
    } else {
        ("Forgot".into(), "Remembered".into())
    }
}

pub(crate) fn part_of_speech_label(pos: PartOfSpeech) -> &'static str {
    match pos {
        PartOfSpeech::Adj => "Adjective",
        PartOfSpeech::Adp => "Adposition",
        PartOfSpeech::Adv => "Adverb",
        PartOfSpeech::Aux => "Auxiliary",
        PartOfSpeech::Cconj => "Conjunction",
        PartOfSpeech::Det => "Determiner",
        PartOfSpeech::Intj => "Interjection",
        PartOfSpeech::Noun => "Noun",
        PartOfSpeech::Num => "Number",
        PartOfSpeech::Part => "Particle",
        PartOfSpeech::Pron => "Pronoun",
        PartOfSpeech::Sconj => "Subordinating Conjunction",
        PartOfSpeech::Sym => "Symbol",
        PartOfSpeech::Verb => "Verb",
    }
}

#[bridgerton::bridge]
#[bridgerton::stable]
pub fn flashcard_view(
    flashcard: FlashCard,
    is_new: bool,
    total_card_count: usize,
    times_type_seen: u32,
    target_language: Language,
    native_language: Language,
) -> FlashcardView {
    let disclosure = get_flashcard_disclosure(total_card_count, times_type_seen);
    let multiple =
        matches!(&flashcard.content, CardContent::Gram { meanings, .. } if meanings.len() > 1);
    let (again_label, remembered_label) = if multiple {
        ("Forgot all".into(), "Remembered all".into())
    } else {
        grade_labels(is_new)
    };
    let (subtitle, tutorial, reveal_label, listening_header, cant_listen_label) = match flashcard
        .content
    {
        CardContent::Gram { gram, .. } => (
            None,
            TutorialPrompt {
                before: "Guess what \"".into(),
                target: Some(language_utils::literals_to_text(&gram).trim().to_owned()),
                after: "\" means…".into(),
            },
            format!(
                "Show {}",
                get_language_metadata(native_language).common_name
            ),
            None,
            None,
        ),
        CardContent::Listening { possible_grams } => {
            let language = get_language_metadata(target_language).common_name;
            (
                Some("Guess what's being said!".into()),
                TutorialPrompt {
                    before: format!("Guess what {language} word is missing"),
                    target: None,
                    after: String::new(),
                },
                format!("Show {language} word"),
                (possible_grams.len() > 1).then(|| "It could have been any of these words:".into()),
                Some("Can't listen now".into()),
            )
        }
    };
    FlashcardView {
        subtitle,
        tutorial_prompt: disclosure.show_tutorial.then_some(tutorial),
        tutorial_hidden_hint: disclosure
            .show_tutorial
            .then(|| "Then, tap to see if you're right!".into()),
        tutorial_revealed_hint: disclosure.show_tutorial.then(|| "Were you right?".into()),
        reveal_label,
        require_answer_reveal: disclosure.require_answer_reveal,
        listening_header,
        known_label: "(known)".into(),
        again_label,
        remembered_label,
        cant_listen_label,
        new_label: multiple.then(|| "New".into()),
        again_rest_label: multiple.then(|| "Forgot the rest".into()),
        remembered_rest_label: multiple.then(|| "Remembered the rest".into()),
        continue_label: multiple.then(|| "Continue".into()),
    }
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PronunciationExample {
    pub cue: PronunciationCue,
    /// Original-text runs aligned with cue.segments; hosts only render these.
    pub highlighted_segments: Vec<Vec<language_utils::pronunciation_pattern::HighlightedRun>>,
    pub cultural_context: Option<String>,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PronunciationView {
    pub pattern: String,
    pub position: PatternPosition,
    pub positioned_pattern: String,
    pub position_note: Option<String>,
    pub tutorial_prompt: Option<TutorialPrompt>,
    pub tutorial_grade_prompt: Option<String>,
    pub examples: Vec<PronunciationExample>,
    pub description: Option<String>,
    pub again_label: String,
    pub remembered_label: String,
    pub cant_speak_label: String,
}

#[bridgerton::bridge]
#[bridgerton::stable]
pub fn pronunciation_view(
    pattern: String,
    guide: PronunciationGuide,
    cues: Vec<PronunciationCue>,
    is_new: bool,
    times_type_seen: u32,
    target_language: Language,
) -> PronunciationView {
    let display_pattern =
        language_utils::pronunciation_pattern::display_pattern(&pattern, target_language);
    let suffix = display_pattern.strip_prefix(&pattern).unwrap();
    let (positioned_pattern, position_note) = match guide.position {
        PatternPosition::Beginning => (
            format!("{pattern}___{suffix}"),
            Some("Appears at the beginning of words".into()),
        ),
        PatternPosition::End => (
            format!("___{pattern}{suffix}"),
            Some("Appears at the end of words".into()),
        ),
        PatternPosition::Anywhere => (display_pattern.clone(), None),
    };
    let tutorial = should_show_challenge_tutorial(times_type_seen);
    let (again_label, remembered_label) = grade_labels(is_new);
    PronunciationView {
        tutorial_prompt: tutorial.then(|| TutorialPrompt {
            before: "Let's practice saying \"".into(),
            target: Some(display_pattern),
            after: "\"".into(),
        }),
        tutorial_grade_prompt: tutorial.then(|| "How was your pronunciation?".into()),
        pattern: pattern.clone(),
        position: guide.position,
        positioned_pattern,
        position_note,
        examples: guide
            .example_words
            .into_iter()
            .zip(cues)
            .take(3)
            .map(|(example, cue)| PronunciationExample {
                highlighted_segments: pronunciation_highlights(
                    &cue,
                    &pattern,
                    guide.position,
                    target_language,
                ),
                cue,
                cultural_context: (!example.cultural_context.is_empty())
                    .then_some(example.cultural_context),
            })
            .collect(),
        description: (!guide.description.is_empty()).then_some(guide.description),
        again_label,
        remembered_label,
        cant_speak_label: "Can't speak now".into(),
    }
}

fn pronunciation_highlights(
    cue: &PronunciationCue,
    pattern: &str,
    position: PatternPosition,
    language: Language,
) -> Vec<Vec<language_utils::pronunciation_pattern::HighlightedRun>> {
    use language_utils::{
        CueSegmentRole,
        pronunciation_pattern::{HighlightedRun, highlighted_runs},
    };
    let first = cue
        .segments
        .iter()
        .position(|s| s.role == CueSegmentRole::Example);
    let last = cue
        .segments
        .iter()
        .rposition(|s| s.role == CueSegmentRole::Example);
    cue.segments
        .iter()
        .enumerate()
        .map(|(index, segment)| {
            if segment.role == CueSegmentRole::Example
                && match position {
                    PatternPosition::Anywhere => true,
                    PatternPosition::Beginning => Some(index) == first,
                    PatternPosition::End => Some(index) == last,
                }
            {
                highlighted_runs(&segment.text, pattern, position, language)
            } else {
                vec![HighlightedRun {
                    text: segment.text.clone(),
                    highlighted: false,
                }]
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use language_utils::{Heteronym, Literal, OtherWord, OtherWordType, Word, WordType};
    use serde_json::json;

    fn gram(phrase: bool) -> CardContent {
        CardContent::Gram {
            gram: vec![Literal {
                word: Word {
                    text: "bonjour".into(),
                    word_type: WordType::Heteronym(Heteronym {
                        word: "bonjour".into(),
                        lemma: "bonjour".into(),
                        pos: PartOfSpeech::Intj,
                    }),
                },
                whitespace: language_utils::Whitespace::Space,
            }],
            meanings: vec![crate::FlashcardMeaning {
                cards: vec![],
                definition: crate::DefinitionView {
                    headword: "bonjour".into(),
                    is_phrase: phrase,
                    morphology_label: String::new(),
                    senses: vec![],
                },
                label: Some(if phrase { "Multiword" } else { "Interjection" }.into()),
                is_new: true,
            }],
            prefix: None,
            breakdown: None,
        }
    }

    fn flash(content: CardContent, new: bool, count: usize, seen: u32) -> FlashcardView {
        flashcard_view(
            FlashCard {
                content,
                audio: None,
            },
            new,
            count,
            seen,
            Language::French,
            Language::English,
        )
    }

    #[test]
    fn gram_subtitle_tutorial_and_reveal_match_web() {
        let view = flash(gram(false), true, 1, 0);
        assert_eq!(view.subtitle, None);
        assert_eq!(
            view.tutorial_prompt,
            Some(TutorialPrompt {
                before: "Guess what \"".into(),
                target: Some("bonjour".into()),
                after: "\" means…".into(),
            })
        );
        assert_eq!(
            view.tutorial_hidden_hint.as_deref(),
            Some("Then, tap to see if you're right!")
        );
        assert_eq!(
            view.tutorial_revealed_hint.as_deref(),
            Some("Were you right?")
        );
        assert_eq!(view.reveal_label, "Show English");
        assert!(view.require_answer_reveal);
        assert_eq!(view.cant_listen_label, None);
        assert_eq!(flash(gram(true), true, 1, 0).subtitle.as_deref(), None);
        let CardContent::Gram {
            mut gram, meanings, ..
        } = gram(false)
        else {
            unreachable!()
        };
        gram[0].word.word_type = WordType::Other(OtherWord {
            other_tag: OtherWordType::Propn,
        });
        let no_pos = CardContent::Gram {
            gram: gram.clone(),
            meanings: meanings.clone(),
            prefix: None,
            breakdown: None,
        };
        assert_eq!(flash(no_pos, false, 50, 10).subtitle, None);
        gram.push(Literal {
            word: Word {
                text: "que".into(),
                word_type: WordType::Heteronym(Heteronym {
                    word: "que".into(),
                    lemma: "que".into(),
                    pos: PartOfSpeech::Sconj,
                }),
            },
            whitespace: language_utils::Whitespace::Space,
        });
        let view = flash(
            CardContent::Gram {
                gram,
                meanings,
                prefix: None,
                breakdown: None,
            },
            true,
            1,
            0,
        );
        assert_eq!(view.subtitle.as_deref(), None);
        assert_eq!(
            view.tutorial_prompt.unwrap().target.as_deref(),
            Some("bonjour que")
        );
    }

    #[test]
    fn multiple_meanings_have_bulk_labels() {
        let mut content = gram(false);
        let CardContent::Gram { meanings, .. } = &mut content else {
            unreachable!()
        };
        meanings.push(meanings[0].clone());
        for is_new in [false, true] {
            let view = flash(content.clone(), is_new, 1, 0);
            assert_eq!(view.again_label, "Forgot all");
            assert_eq!(view.remembered_label, "Remembered all");
            assert_eq!(view.again_rest_label.as_deref(), Some("Forgot the rest"));
            assert_eq!(
                view.remembered_rest_label.as_deref(),
                Some("Remembered the rest")
            );
            assert_eq!(view.new_label.as_deref(), Some("New"));
            assert_eq!(view.continue_label.as_deref(), Some("Continue"));
        }
        let view = flash(gram(false), true, 1, 0);
        assert!(view.new_label.is_none());
        assert!(view.again_rest_label.is_none());
        assert!(view.remembered_rest_label.is_none());
        assert!(view.continue_label.is_none());
    }

    #[test]
    fn listening_candidates_and_language_labels() {
        for count in 0..=3 {
            let view = flash(
                CardContent::Listening {
                    possible_grams: vec![(false, vec![], vec![]); count],
                },
                true,
                1,
                1,
            );
            assert_eq!(view.subtitle.as_deref(), Some("Guess what's being said!"));
            assert_eq!(view.reveal_label, "Show French word");
            assert_eq!(
                view.tutorial_prompt,
                Some(TutorialPrompt {
                    before: "Guess what French word is missing".into(),
                    target: None,
                    after: String::new(),
                })
            );
            assert_eq!(
                view.listening_header.as_deref(),
                (count > 1).then_some("It could have been any of these words:")
            );
            assert_eq!(view.known_label, "(known)");
            assert_eq!(view.cant_listen_label.as_deref(), Some("Can't listen now"));
        }
    }

    #[test]
    fn flashcard_labels_tutorial_threshold_and_grading_policy() {
        for new in [false, true] {
            for seen in [0, 1, 2, 9, 10, 20] {
                for count in [0, 49, 50, 100] {
                    let view = flash(gram(false), new, count, seen);
                    assert_eq!(
                        (view.again_label.as_str(), view.remembered_label.as_str()),
                        if new {
                            ("Didn't know", "Already knew")
                        } else {
                            ("Forgot", "Remembered")
                        }
                    );
                    assert_eq!(view.tutorial_prompt.is_some(), seen < 2);
                    assert_eq!(view.tutorial_hidden_hint.is_some(), seen < 2);
                    assert_eq!(view.tutorial_revealed_hint.is_some(), seen < 2);
                    assert_eq!(view.require_answer_reveal, count < 50 || seen < 10);
                }
            }
        }
    }

    fn guide() -> PronunciationGuide {
        serde_json::from_value(json!({
            "pattern": "r", "position": "Beginning", "description": "A **sound**",
            "familiarity": "ProbablyDoesNotKnow", "difficulty": "Easy", "example_words": []
        }))
        .unwrap()
    }

    fn cue() -> PronunciationCue {
        serde_json::from_value(json!({
            "audio": { "request": { "text": "r comme rue", "language": "French" }, "provider": "ElevenLabs" },
            "segments": [], "native_connector": "as in"
        })).unwrap()
    }

    #[test]
    fn pronunciation_positions_tutorials_and_labels() {
        for (position, positioned, note) in [
            (
                PatternPosition::Beginning,
                "r___",
                Some("Appears at the beginning of words"),
            ),
            (
                PatternPosition::End,
                "___r",
                Some("Appears at the end of words"),
            ),
            (PatternPosition::Anywhere, "r", None),
        ] {
            for new in [false, true] {
                for seen in [0, 1, 2, 10] {
                    let mut guide = guide();
                    guide.position = position;
                    let view =
                        pronunciation_view("r".into(), guide, vec![], new, seen, Language::French);
                    assert_eq!(view.pattern, "r");
                    assert_eq!(view.position, position);
                    assert_eq!(view.positioned_pattern, positioned);
                    assert_eq!(view.position_note.as_deref(), note);
                    assert_eq!(view.description.as_deref(), Some("A **sound**"));
                    assert_eq!(
                        view.tutorial_prompt,
                        (seen < 2).then(|| TutorialPrompt {
                            before: "Let's practice saying \"".into(),
                            target: Some("r".into()),
                            after: "\"".into(),
                        })
                    );
                    assert_eq!(
                        view.tutorial_grade_prompt.as_deref(),
                        (seen < 2).then_some("How was your pronunciation?")
                    );
                    assert_eq!(
                        (view.again_label.as_str(), view.remembered_label.as_str()),
                        if new {
                            ("Didn't know", "Already knew")
                        } else {
                            ("Forgot", "Remembered")
                        }
                    );
                    assert_eq!(view.cant_speak_label, "Can't speak now");
                }
            }
        }
    }

    #[test]
    fn korean_display_explains_jamo_without_changing_identity_or_audio() {
        let mut guide = guide();
        guide.pattern = "ㅡ".into();
        guide.position = PatternPosition::Anywhere;
        guide.example_words.push(language_utils::WordPair {
            target: "극".into(),
            native: "theatre".into(),
            position: language_utils::SoundPosition::Middle,
            cultural_context: String::new(),
        });
        let mut cue = cue();
        cue.segments = vec![crate::CueSegment {
            text: "극".into(),
            role: language_utils::CueSegmentRole::Example,
            start_ms: None,
        }];
        let audio = cue.audio.clone();
        let view = pronunciation_view("ㅡ".into(), guide, vec![cue], true, 0, Language::Korean);
        assert_eq!(view.pattern, "ㅡ");
        assert_eq!(view.positioned_pattern, "ㅡ (eu)");
        assert_eq!(
            view.tutorial_prompt.unwrap().target.as_deref(),
            Some("ㅡ (eu)")
        );
        assert_eq!(view.examples[0].cue.audio, audio);
        assert_eq!(
            view.examples[0].highlighted_segments[0],
            vec![language_utils::pronunciation_pattern::HighlightedRun {
                text: "극".into(),
                highlighted: true
            }]
        );
    }

    #[test]
    fn pronunciation_examples_zip_cues_and_limit_to_three() {
        for examples in 0..=5 {
            for cues in 0..=5 {
                let mut guide = guide();
                guide.description.clear();
                guide.example_words = (0..examples)
                    .map(|index| language_utils::WordPair {
                        target: "rue".into(),
                        native: "street".into(),
                        position: language_utils::SoundPosition::Beginning,
                        cultural_context: if index == 0 {
                            String::new()
                        } else {
                            format!("context {index}")
                        },
                    })
                    .collect();
                let cues: Vec<_> = (0..cues)
                    .map(|index| PronunciationCue {
                        native_connector: format!("cue {index}"),
                        ..cue()
                    })
                    .collect();
                let view =
                    pronunciation_view("r".into(), guide, cues.clone(), false, 2, Language::French);
                assert_eq!(view.examples.len(), examples.min(cues.len()).min(3));
                assert_eq!(view.description, None);
                for (index, example) in view.examples.iter().enumerate() {
                    assert_eq!(example.cue, cues[index]);
                    assert_eq!(
                        example.cultural_context,
                        (index != 0).then(|| format!("context {index}"))
                    );
                }
            }
        }
    }
}
