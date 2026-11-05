//! Omnigram: a Unigram (SentencePiece-style) vocabulary trainer over any token type.
//!
//! generate-data trains supertokens on atoms with it, and the pack builder trains
//! the character runs that compress translation text.

pub mod unigram;

impl unigram::UnigramToken for char {
    fn can_be_sequence_boundary(&self) -> bool {
        true
    }
    fn is_content(&self) -> bool {
        true
    }
    fn is_excluded_from_sequences(&self) -> bool {
        false
    }
}
