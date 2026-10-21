//! The sentence tutor's shared state, requests, and host-ready UTF-16 ranges.
use language_utils::{Language, book_sentence::*};
use serde::{Deserialize, Serialize};

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BookSentenceState {
    pub request: ExplainSentenceRequest,
    pub paragraph_index: usize,
    pub explanation: BookSentenceExplanation,
    pub messages: Vec<ChatTurn>,
    pub draft: String,
    pub reply_status: BookSentenceReplyStatus,
}

#[bridgerton::bridge(transparent, namespace)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum BookSentenceExplanation {
    Loading,
    Loaded { explanation: SentenceExplanation },
    Failed,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum BookSentenceReplyStatus {
    Idle,
    Waiting,
    Failed,
}

#[bridgerton::bridge(transparent, namespace)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum BookSentenceEvent {
    Explained { explanation: SentenceExplanation },
    ExplainFailed,
    DraftChanged { text: String },
    Ask,
    Replied { reply: SentenceChatReply },
    ReplyFailed,
    Retry,
}

#[bridgerton::bridge(transparent, namespace)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum BookSentenceEffect {
    Explain { request: ExplainSentenceRequest },
    Chat { request: SentenceChatRequest },
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BookSentenceStep {
    pub state: BookSentenceState,
    pub effects: Vec<BookSentenceEffect>,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BookSentenceHighlight {
    pub start: u32,
    pub end: u32,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BookSentenceMessage {
    pub from_learner: bool,
    pub text: String,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BookSentenceView {
    pub paragraph_index: usize,
    pub highlight: Option<BookSentenceHighlight>,
    pub sentence: Option<String>,
    pub translation: Option<String>,
    pub word: Option<String>,
    pub gloss: Option<String>,
    pub word_explanation: Option<String>,
    pub messages: Vec<BookSentenceMessage>,
    pub status: Option<String>,
    pub status_is_error: bool,
    pub retry_label: Option<String>,
    pub draft: String,
    pub placeholder: String,
    pub send_label: String,
    pub can_send: bool,
}

fn byte_offset(text: &str, utf16_offset: u32) -> usize {
    let mut position = 0;
    for (byte, ch) in text.char_indices() {
        position += ch.len_utf16() as u32;
        if position > utf16_offset {
            return byte;
        }
    }
    text.len()
}

fn utf16_offset(text: &str, byte_offset: usize) -> u32 {
    text[..byte_offset].encode_utf16().count() as u32
}

impl BookSentenceState {
    /// Hosts start through `yap_frontend_rs::book_sentence_start`, which reads the book context off the reader.
    pub fn start(
        language: Language,
        native_language: Language,
        book: BookContext,
        paragraph_index: usize,
        paragraph: String,
        utf16_offset: u32,
    ) -> BookSentenceStep {
        let request = ExplainSentenceRequest {
            language,
            native_language,
            book,
            offset: byte_offset(&paragraph, utf16_offset),
            paragraph,
        };
        BookSentenceStep {
            effects: vec![BookSentenceEffect::Explain {
                request: request.clone(),
            }],
            state: BookSentenceState {
                request,
                paragraph_index,
                explanation: BookSentenceExplanation::Loading,
                messages: vec![],
                draft: String::new(),
                reply_status: BookSentenceReplyStatus::Idle,
            },
        }
    }

    fn can_send(&self) -> bool {
        matches!(self.explanation, BookSentenceExplanation::Loaded { .. })
            && self.reply_status != BookSentenceReplyStatus::Waiting
            && !self.draft.trim().is_empty()
    }

    fn chat_effect(&self) -> BookSentenceEffect {
        let BookSentenceExplanation::Loaded { explanation } = &self.explanation else {
            unreachable!()
        };
        BookSentenceEffect::Chat {
            request: SentenceChatRequest {
                sentence: self.request.clone(),
                explanation: explanation.clone(),
                messages: self.messages.clone(),
            },
        }
    }
}

#[bridgerton::bridge]
pub fn book_sentence_transition(
    mut state: BookSentenceState,
    event: BookSentenceEvent,
) -> BookSentenceStep {
    let mut effects = vec![];
    match event {
        BookSentenceEvent::Explained { explanation }
            if matches!(state.explanation, BookSentenceExplanation::Loading) =>
        {
            state.explanation = BookSentenceExplanation::Loaded { explanation };
        }
        BookSentenceEvent::ExplainFailed
            if matches!(state.explanation, BookSentenceExplanation::Loading) =>
        {
            state.explanation = BookSentenceExplanation::Failed;
        }
        BookSentenceEvent::DraftChanged { text } => state.draft = text,
        BookSentenceEvent::Ask if state.can_send() => {
            state.messages.push(ChatTurn {
                role: ChatRole::Learner,
                text: state.draft.trim().into(),
            });
            state.draft.clear();
            state.reply_status = BookSentenceReplyStatus::Waiting;
            effects.push(state.chat_effect());
        }
        BookSentenceEvent::Replied { reply }
            if state.reply_status == BookSentenceReplyStatus::Waiting =>
        {
            state.messages.push(ChatTurn {
                role: ChatRole::Tutor,
                text: reply.text,
            });
            state.reply_status = BookSentenceReplyStatus::Idle;
        }
        BookSentenceEvent::ReplyFailed
            if state.reply_status == BookSentenceReplyStatus::Waiting =>
        {
            state.reply_status = BookSentenceReplyStatus::Failed;
        }
        BookSentenceEvent::Retry => {
            if matches!(state.explanation, BookSentenceExplanation::Failed) {
                state.explanation = BookSentenceExplanation::Loading;
                effects.push(BookSentenceEffect::Explain {
                    request: state.request.clone(),
                });
            } else if state.reply_status == BookSentenceReplyStatus::Failed {
                state.reply_status = BookSentenceReplyStatus::Waiting;
                effects.push(state.chat_effect());
            }
        }
        _ => {}
    }
    BookSentenceStep { state, effects }
}

#[bridgerton::bridge]
pub fn book_sentence_view(state: BookSentenceState) -> BookSentenceView {
    let explanation = match &state.explanation {
        BookSentenceExplanation::Loaded { explanation } => Some(explanation),
        _ => None,
    };
    let failed = matches!(state.explanation, BookSentenceExplanation::Failed)
        || state.reply_status == BookSentenceReplyStatus::Failed;
    BookSentenceView {
        paragraph_index: state.paragraph_index,
        highlight: explanation.map(|e| BookSentenceHighlight {
            start: utf16_offset(&state.request.paragraph, e.start),
            end: utf16_offset(&state.request.paragraph, e.end),
        }),
        sentence: explanation.map(|e| state.request.paragraph[e.start..e.end].to_owned()),
        translation: explanation.map(|e| e.translation.clone()),
        word: explanation.map(|e| e.word.clone()),
        gloss: explanation.map(|e| e.gloss.clone()),
        word_explanation: explanation.map(|e| e.word_explanation.clone()),
        messages: state
            .messages
            .iter()
            .map(|turn| BookSentenceMessage {
                from_learner: turn.role == ChatRole::Learner,
                text: turn.text.clone(),
            })
            .collect(),
        status: if failed {
            Some(yap_i18n::tr!("books-tutor-failed"))
        } else if matches!(state.explanation, BookSentenceExplanation::Loading) {
            Some(yap_i18n::tr!("books-tutor-translating"))
        } else if state.reply_status == BookSentenceReplyStatus::Waiting {
            Some(yap_i18n::tr!("books-tutor-thinking"))
        } else {
            None
        },
        status_is_error: failed,
        retry_label: failed.then(|| yap_i18n::tr!("books-tutor-retry")),
        can_send: state.can_send(),
        draft: state.draft,
        placeholder: yap_i18n::tr!("books-tutor-placeholder"),
        send_label: yap_i18n::tr!("books-tutor-send"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn book() -> BookContext {
        BookContext {
            title: "Arsène Lupin".into(),
            author: "Maurice Leblanc".into(),
            chapter: None,
            previous_paragraph: None,
            next_paragraph: None,
        }
    }

    fn start() -> BookSentenceStep {
        BookSentenceState::start(
            Language::French,
            Language::English,
            book(),
            3,
            "C'était l'été — «oui»".into(),
            10,
        )
    }

    fn loaded() -> BookSentenceState {
        let state = start().state;
        let end = state.request.paragraph.len();
        book_sentence_transition(
            state,
            BookSentenceEvent::Explained {
                explanation: SentenceExplanation {
                    start: 0,
                    end,
                    translation: "It was summer — ‘yes’".into(),
                    word: "été".into(),
                    gloss: "summer".into(),
                    word_explanation: "Summer.".into(),
                },
            },
        )
        .state
    }

    fn draft(state: BookSentenceState, text: &str) -> BookSentenceState {
        book_sentence_transition(state, BookSentenceEvent::DraftChanged { text: text.into() }).state
    }

    #[test]
    fn happy_path_and_conversation_history() {
        let initial = start();
        assert!(
            matches!(&initial.effects[..], [BookSentenceEffect::Explain { request }] if request == &initial.state.request)
        );
        let view = book_sentence_view(initial.state);
        assert_eq!(view.paragraph_index, 3);
        assert!(view.highlight.is_none());
        assert_eq!(view.status.as_deref(), Some("Translating…"));
        let state = loaded();
        let view = book_sentence_view(state.clone());
        assert_eq!(view.sentence.as_deref(), Some("C'était l'été — «oui»"));
        assert_eq!(view.gloss.as_deref(), Some("summer"));
        assert_eq!(
            view.highlight,
            Some(BookSentenceHighlight { start: 0, end: 21 })
        );
        let step = book_sentence_transition(draft(state, "  Why l'?  "), BookSentenceEvent::Ask);
        assert_eq!(step.state.messages[0].text, "Why l'?");
        assert!(step.state.draft.is_empty());
        assert!(
            matches!(&step.effects[..], [BookSentenceEffect::Chat { request }] if request.messages == step.state.messages)
        );
        let view = book_sentence_view(step.state.clone());
        assert_eq!(view.status.as_deref(), Some("Thinking…"));
        assert!(view.messages[0].from_learner);
        let state = book_sentence_transition(
            step.state,
            BookSentenceEvent::Replied {
                reply: SentenceChatReply {
                    text: "The article elides before a vowel.".into(),
                },
            },
        )
        .state;
        let view = book_sentence_view(state.clone());
        assert!(!view.messages[1].from_learner);
        assert!(view.status.is_none());
        let step =
            book_sentence_transition(draft(state, "Another example?"), BookSentenceEvent::Ask);
        assert!(
            matches!(&step.effects[..], [BookSentenceEffect::Chat { request }] if request.messages.len() == 3)
        );
    }

    #[test]
    fn ask_requires_loaded_nonblank_and_not_waiting() {
        for state in [draft(start().state, "Question"), draft(loaded(), " \n ")] {
            assert!(!book_sentence_view(state.clone()).can_send);
            assert_eq!(
                book_sentence_transition(state.clone(), BookSentenceEvent::Ask),
                BookSentenceStep {
                    state,
                    effects: vec![]
                }
            );
        }
        let waiting =
            book_sentence_transition(draft(loaded(), "Question"), BookSentenceEvent::Ask).state;
        let state = draft(waiting, "Next question");
        assert!(!book_sentence_view(state.clone()).can_send);
        assert_eq!(
            book_sentence_transition(state.clone(), BookSentenceEvent::Ask),
            BookSentenceStep {
                state,
                effects: vec![]
            }
        );
    }

    #[test]
    fn failures_retry_identical_requests_without_duplicate_turns() {
        let initial = start();
        let failed =
            book_sentence_transition(initial.state, BookSentenceEvent::ExplainFailed).state;
        let view = book_sentence_view(failed.clone());
        assert!(view.status_is_error);
        assert_eq!(view.retry_label.as_deref(), Some("Try again"));
        let retry = book_sentence_transition(failed, BookSentenceEvent::Retry);
        assert_eq!(retry.effects, initial.effects);
        assert!(!book_sentence_view(retry.state).status_is_error);
        let ask = book_sentence_transition(draft(loaded(), "Question"), BookSentenceEvent::Ask);
        let failed = book_sentence_transition(ask.state, BookSentenceEvent::ReplyFailed).state;
        assert_eq!(failed.messages.len(), 1);
        assert!(book_sentence_view(failed.clone()).status_is_error);
        let retry =
            book_sentence_transition(draft(failed, "Keep this draft"), BookSentenceEvent::Retry);
        assert_eq!(retry.effects, ask.effects);
        assert_eq!(retry.state.messages.len(), 1);
        assert_eq!(retry.state.draft, "Keep this draft");
        assert!(
            book_sentence_transition(retry.state, BookSentenceEvent::Retry)
                .effects
                .is_empty()
        );
    }

    #[test]
    fn unicode_offsets_round_trip_and_snap_surrogates() {
        for text in ["C'était l'été — «oui»", "😀你好。夏天！"] {
            for (byte, _) in text.char_indices() {
                assert_eq!(byte_offset(text, utf16_offset(text, byte)), byte);
            }
            assert_eq!(byte_offset(text, u32::MAX), text.len());
        }
        assert_eq!(byte_offset("😀你好", 1), 0);
        assert_eq!(byte_offset("😀你好", 2), 4);
        let state = BookSentenceState::start(
            Language::ChineseSimplified,
            Language::English,
            book(),
            2,
            "😀你好。夏天！".into(),
            6,
        )
        .state;
        assert_eq!(state.request.offset, 16);
        let state = book_sentence_transition(
            state,
            BookSentenceEvent::Explained {
                explanation: SentenceExplanation {
                    start: 13,
                    end: 22,
                    translation: "Summer!".into(),
                    word: "夏天".into(),
                    gloss: "summer".into(),
                    word_explanation: "Two characters, one word.".into(),
                },
            },
        )
        .state;
        let view = book_sentence_view(state);
        assert_eq!(view.sentence.as_deref(), Some("夏天！"));
        assert_eq!(
            view.highlight,
            Some(BookSentenceHighlight { start: 5, end: 8 })
        );
    }
}
