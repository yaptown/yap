//! Pure challenge state machines and computed views, with data-only host effects.
//! Depend only on language-utils, bridgerton and serde, never yap-frontend-rs:
//! this crate must also support a small standalone WASM for the MCP widget.
#[macro_use]
pub mod palette;
pub mod design;
pub use design::*;
pub mod transcription;
pub use transcription::*;

#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum Sound {
    AiDoneGrading,
    Success,
}
