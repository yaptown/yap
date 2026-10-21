//! Wire types for the book reader's sentence tutor. Ranges on the wire are UTF-8 bytes.
use crate::Language;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct ExplainSentenceRequest {
    pub language: Language,
    pub native_language: Language,
    pub book: BookContext,
    pub paragraph: String,
    pub offset: usize,
}

/// Where the paragraph sits in its book, so the model reads it in context.
#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct BookContext {
    pub title: String,
    pub author: String,
    pub chapter: Option<String>,
    pub previous_paragraph: Option<String>,
    pub next_paragraph: Option<String>,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct SentenceExplanation {
    pub start: usize,
    pub end: usize,
    pub translation: String,
    pub word: String,
    /// A few words in the native language: what the word means here.
    pub gloss: String,
    pub word_explanation: String,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct SentenceChatRequest {
    pub sentence: ExplainSentenceRequest,
    pub explanation: SentenceExplanation,
    pub messages: Vec<ChatTurn>,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct SentenceChatReply {
    pub text: String,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct ChatTurn {
    pub role: ChatRole,
    pub text: String,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub enum ChatRole {
    Learner,
    Tutor,
}
