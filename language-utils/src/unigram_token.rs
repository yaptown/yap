use crate::{Atom, OtherWordType, WordType};
use omnigram::unigram::UnigramToken;

impl<S: Clone + Eq + std::hash::Hash + std::fmt::Debug> UnigramToken for Atom<S> {
    fn can_be_sequence_boundary(&self) -> bool {
        matches!(self, Atom::Tok(_))
    }

    fn is_content(&self) -> bool {
        matches!(self, Atom::Tok(word) if matches!(&word.word_type, WordType::Heteronym(_)))
    }

    fn is_excluded_from_sequences(&self) -> bool {
        matches!(
            self,
            Atom::Tok(word) if matches!(&word.word_type, WordType::Other(o) if o.other_tag == OtherWordType::Propn)
        )
    }
}
