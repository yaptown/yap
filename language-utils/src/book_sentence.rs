//! Wire types for the book reader's sentence tutor. Ranges on the wire are UTF-8 bytes.
use crate::Language;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct ExplainSentenceRequest {
    pub language: Language,
    pub native_language: Language,
    pub paragraph: String,
    pub offset: usize,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
pub struct SentenceExplanation {
    pub start: usize,
    pub end: usize,
    pub translation: String,
    pub word: String,
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
