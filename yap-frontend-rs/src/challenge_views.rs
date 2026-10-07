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
        (
            yap_i18n::tr!("challenge-didn-t-know"),
            yap_i18n::tr!("challenge-already-knew"),
        )
    } else {
        (
            yap_i18n::tr!("common-forgot"),
            yap_i18n::tr!("common-remembered"),
        )
    }
}

pub(crate) fn part_of_speech_label(pos: PartOfSpeech) -> String {
    match pos {
        PartOfSpeech::Adj => yap_i18n::tr!("challenge-adjective"),
        PartOfSpeech::Adp => yap_i18n::tr!("challenge-adposition"),
        PartOfSpeech::Adv => yap_i18n::tr!("challenge-adverb"),
        PartOfSpeech::Aux => yap_i18n::tr!("challenge-auxiliary"),
        PartOfSpeech::Cconj => yap_i18n::tr!("challenge-conjunction"),
        PartOfSpeech::Det => yap_i18n::tr!("challenge-determiner"),
        PartOfSpeech::Intj => yap_i18n::tr!("challenge-interjection"),
        PartOfSpeech::Noun => yap_i18n::tr!("challenge-noun"),
        PartOfSpeech::Num => yap_i18n::tr!("challenge-number"),
        PartOfSpeech::Part => yap_i18n::tr!("challenge-particle"),
        PartOfSpeech::Pron => yap_i18n::tr!("challenge-pronoun"),
        PartOfSpeech::Sconj => yap_i18n::tr!("challenge-subordinating-conjunction"),
        PartOfSpeech::Sym => yap_i18n::tr!("challenge-symbol"),
        PartOfSpeech::Verb => yap_i18n::tr!("challenge-verb"),
    }
}

pub(crate) fn part_of_speech_short_label(pos: PartOfSpeech) -> String {
    match pos {
        PartOfSpeech::Adj => yap_i18n::tr!("pos-short-adj"),
        PartOfSpeech::Adp => yap_i18n::tr!("pos-short-adp"),
        PartOfSpeech::Adv => yap_i18n::tr!("pos-short-adv"),
        PartOfSpeech::Aux => yap_i18n::tr!("pos-short-aux"),
        PartOfSpeech::Cconj => yap_i18n::tr!("pos-short-cconj"),
        PartOfSpeech::Det => yap_i18n::tr!("pos-short-det"),
        PartOfSpeech::Intj => yap_i18n::tr!("pos-short-intj"),
        PartOfSpeech::Noun => yap_i18n::tr!("pos-short-noun"),
        PartOfSpeech::Num => yap_i18n::tr!("pos-short-num"),
        PartOfSpeech::Part => yap_i18n::tr!("pos-short-part"),
        PartOfSpeech::Pron => yap_i18n::tr!("pos-short-pron"),
        PartOfSpeech::Sconj => yap_i18n::tr!("pos-short-sconj"),
        PartOfSpeech::Sym => yap_i18n::tr!("pos-short-sym"),
        PartOfSpeech::Verb => yap_i18n::tr!("pos-short-verb"),
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
    let all_new = matches!(&flashcard.content, CardContent::Gram { meanings, .. }
        if meanings.iter().all(|meaning| meaning.is_new));
    let (again_label, remembered_label) = if multiple && !all_new {
        (
            yap_i18n::tr!("challenge-forgot-all"),
            yap_i18n::tr!("challenge-remembered-all"),
        )
    } else {
        grade_labels(if multiple { all_new } else { is_new })
    };
    let (subtitle, tutorial, reveal_label, listening_header, cant_listen_label) =
        match flashcard.content {
            CardContent::Gram { gram, .. } => (
                None,
                TutorialPrompt {
                    before: yap_i18n::tr!("challenge-guess-meaning.before"),
                    target: Some(language_utils::literals_to_text(&gram).trim().to_owned()),
                    after: yap_i18n::tr!("challenge-guess-meaning.after"),
                },
                yap_i18n::tr!(
                    "challenge-reveal-language",
                    language = (get_language_metadata(native_language).common_name).to_string()
                ),
                None,
                None,
            ),
            CardContent::Listening { possible_grams } => {
                let language = get_language_metadata(target_language).common_name;
                (
                    Some(yap_i18n::tr!("challenge-guess-what-s-being-said")),
                    TutorialPrompt {
                        before: yap_i18n::tr!(
                            "challenge-guess-what-language-word-is-missing",
                            language = (language).to_string()
                        ),
                        target: None,
                        after: String::new(),
                    },
                    yap_i18n::tr!(
                        "challenge-show-language-word",
                        language = (language).to_string()
                    ),
                    (possible_grams.len() > 1)
                        .then(|| yap_i18n::tr!("challenge-it-could-have-been-any-of-these-words")),
                    Some(yap_i18n::tr!("transcription-can-t-listen-now")),
                )
            }
        };
    FlashcardView {
        subtitle,
        tutorial_prompt: disclosure.show_tutorial.then_some(tutorial),
        tutorial_hidden_hint: disclosure
            .show_tutorial
            .then(|| yap_i18n::tr!("challenge-then-tap-to-see-if-you-re-right")),
        tutorial_revealed_hint: disclosure
            .show_tutorial
            .then(|| yap_i18n::tr!("challenge-were-you-right")),
        reveal_label,
        require_answer_reveal: disclosure.require_answer_reveal,
        listening_header,
        known_label: yap_i18n::tr!("challenge-known"),
        again_label,
        remembered_label,
        cant_listen_label,
        new_label: (multiple && !all_new).then(|| yap_i18n::tr!("common-new")),
        again_rest_label: multiple.then(|| {
            if all_new {
                yap_i18n::tr!("challenge-didn-t-know")
            } else {
                yap_i18n::tr!("challenge-forgot-the-rest")
            }
        }),
        remembered_rest_label: multiple.then(|| {
            if all_new {
                yap_i18n::tr!("challenge-already-knew")
            } else {
                yap_i18n::tr!("challenge-remembered-the-rest")
            }
        }),
        continue_label: multiple.then(|| yap_i18n::tr!("common-continue")),
    }
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PronunciationExample {
    pub cue: PronunciationCue,
    /// Original-text runs aligned with cue.segments; hosts only render these.
    pub highlighted_segments: Vec<Vec<language_utils::pronunciation_pattern::HighlightedRun>>,
    pub cultural_context: Option<String>,
    pub chunks: Vec<language_utils::SentenceChunk>,
    pub segment_whitespace: Vec<String>,
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
            Some(yap_i18n::tr!("challenge-appears-at-the-beginning-of-words")),
        ),
        PatternPosition::End => (
            format!("___{pattern}{suffix}"),
            Some(yap_i18n::tr!("challenge-appears-at-the-end-of-words")),
        ),
        PatternPosition::Anywhere => (display_pattern.clone(), None),
    };
    let tutorial = should_show_challenge_tutorial(times_type_seen);
    let (again_label, remembered_label) = grade_labels(is_new);
    PronunciationView {
        tutorial_prompt: tutorial.then(|| TutorialPrompt {
            before: yap_i18n::tr!("challenge-practice-saying.before"),
            target: Some(display_pattern),
            after: yap_i18n::tr!("challenge-practice-saying.after"),
        }),
        tutorial_grade_prompt: tutorial
            .then(|| yap_i18n::tr!("challenge-how-was-your-pronunciation")),
        pattern: pattern.clone(),
        position: guide.position,
        positioned_pattern,
        position_note,
        examples: guide
            .example_words
            .into_iter()
            .zip(cues)
            .take(3)
            .map(|(example, cue)| {
                let (chunks, segment_whitespace) = pronunciation_chunks(&cue, target_language);
                PronunciationExample {
                    chunks,
                    segment_whitespace,
                    highlighted_segments: pronunciation_highlights(
                        &cue,
                        &pattern,
                        guide.position,
                        target_language,
                    ),
                    cue,
                    cultural_context: (!example.cultural_context.is_empty())
                        .then_some(example.cultural_context),
                }
            })
            .collect(),
        description: (!guide.description.is_empty()).then_some(guide.description),
        again_label,
        remembered_label,
        cant_speak_label: yap_i18n::tr!("challenge-can-t-speak-now"),
    }
}

fn pronunciation_chunks(
    cue: &PronunciationCue,
    language: Language,
) -> (Vec<language_utils::SentenceChunk>, Vec<String>) {
    use language_utils::{CueSegmentRole, Whitespace};
    let whitespace: Vec<_> = cue
        .segments
        .iter()
        .enumerate()
        .map(|(index, segment)| match cue.segments.get(index + 1) {
            None => Whitespace::None,
            Some(next)
                if segment.role == CueSegmentRole::Pattern
                    && next.role == CueSegmentRole::Pattern =>
            {
                Whitespace::None
            }
            _ => Whitespace::Space,
        })
        .collect();
    // The connector is one swapping target/native view, not separate words.
    let tokens: Vec<_> = cue
        .segments
        .iter()
        .enumerate()
        .map(|(index, segment)| {
            let gap = if segment.role == CueSegmentRole::Connector
                && cue
                    .segments
                    .get(index + 1)
                    .is_some_and(|next| next.role == CueSegmentRole::Connector)
            {
                Whitespace::Nbsp
            } else {
                whitespace[index]
            };
            (segment.text.as_str(), gap)
        })
        .collect();
    (
        language_utils::sentence_chunks(&tokens, language),
        whitespace.into_iter().map(|gap| gap.to_string()).collect(),
    )
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
    fn all_new_meanings_use_compact_new_card_labels_without_badges() {
        let mut content = gram(false);
        let CardContent::Gram { meanings, .. } = &mut content else {
            unreachable!()
        };
        meanings.push(meanings[0].clone());
        for is_new in [false, true] {
            let view = flash(content.clone(), is_new, 1, 0);
            assert_eq!(view.again_label, "Didn't know");
            assert_eq!(view.remembered_label, "Already knew");
            assert_eq!(view.again_rest_label.as_deref(), Some("Didn't know"));
            assert_eq!(view.remembered_rest_label.as_deref(), Some("Already knew"));
            assert_eq!(view.new_label, None);
            assert_eq!(view.continue_label.as_deref(), Some("Continue"));
        }
    }

    #[test]
    fn multiple_meanings_have_bulk_labels() {
        let mut content = gram(false);
        let CardContent::Gram { meanings, .. } = &mut content else {
            unreachable!()
        };
        meanings.push(meanings[0].clone());
        meanings[0].is_new = false;
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
    fn pronunciation_chunks_keep_pattern_letters_and_connector_together() {
        use language_utils::CueSegmentRole::{Connector, Example, Pattern};
        let mut cue = cue();
        cue.segments = [
            ("c", Pattern),
            ("h", Pattern),
            ("as", Connector),
            ("in", Connector),
            ("chat", Example),
        ]
        .into_iter()
        .map(|(text, role)| crate::CueSegment {
            text: text.into(),
            role,
            start_ms: None,
        })
        .collect();
        let (chunks, whitespace) = pronunciation_chunks(&cue, Language::French);
        assert_eq!(
            chunks
                .iter()
                .map(|chunk| chunk.indices.clone())
                .collect::<Vec<_>>(),
            vec![vec![0, 1], vec![2, 3], vec![4]]
        );
        assert_eq!(whitespace, vec!["", " ", " ", " ", ""]);
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
