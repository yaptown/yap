//! Pure challenge state machines and computed views, with data-only host effects.
//! Keep dependencies lean and never depend on yap-frontend-rs:
//! this crate must also support a small standalone WASM for the MCP widget.
#[macro_use]
pub mod palette;
pub mod definition;
pub use definition::*;
pub mod design;
pub use design::*;
pub mod pending;
pub use pending::*;
pub mod proper_nouns;
pub use proper_nouns::*;
pub mod transcription;
pub use transcription::*;

#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum Sound {
    AiDoneGrading,
    Success,
}

pub mod translation;
pub use translation::*;

use language_utils::{TtsProvider, TtsRequest};
#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct AudioRequest {
    pub request: TtsRequest,
    pub provider: TtsProvider,
}
