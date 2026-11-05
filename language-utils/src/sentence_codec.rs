//! Packed sentence symbols. Only this module knows their bit layout.
use std::num::NonZeroU32;

use lasso::{Key, RodeoReader};

use crate::{MultiwordTermMatch, SentenceGram, SentenceGrams, SpurReadingGram};

/// A reading-tagged gram's index; learnable keys are also their ease ranks.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Clone, Copy, Debug, PartialEq, Eq, Hash))]
pub struct GramKey(u32);

impl GramKey {
    pub fn index(self) -> u32 {
        self.0
    }
}

// Both forms preserve the index exactly.
unsafe impl Key for GramKey {
    fn into_usize(self) -> usize {
        self.0 as usize
    }
    fn try_from_usize(index: usize) -> Option<Self> {
        u32::try_from(index).ok().map(Self)
    }
}
unsafe impl Key for ArchivedGramKey {
    fn into_usize(self) -> usize {
        self.0.to_native() as usize
    }
    fn try_from_usize(index: usize) -> Option<Self> {
        u32::try_from(index).ok().map(|index| Self(index.into()))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SentencePlacement {
    Words,
    Phrases,
    LowConfidencePhrases,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolKind {
    Gram(GramKey),
    Placement(SentencePlacement),
    Lowercase,
    Gap(u8),
}

/// Bit 31 marks learnable grams; bit 30 marks other grams. The payload is
/// key + 1. Small reserved values encode separators, casing and phrase gaps.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[rkyv(derive(Clone, Copy))]
pub struct Symbol(NonZeroU32);

impl From<&Symbol> for Symbol {
    fn from(symbol: &Symbol) -> Self {
        *symbol
    }
}

impl From<&ArchivedSymbol> for Symbol {
    fn from(symbol: &ArchivedSymbol) -> Self {
        Self(symbol.0.to_native())
    }
}

impl Symbol {
    const LEARNABLE: u32 = 1 << 31;
    const OBVIOUS: u32 = 1 << 30;
    const PAYLOAD: u32 = Self::OBVIOUS - 1;

    fn new(value: u32) -> Self {
        Self(NonZeroU32::new(value).unwrap())
    }

    pub fn gram(key: GramKey, learnable: bool) -> Self {
        assert!(key.0 < Self::PAYLOAD);
        Self::new(
            (if learnable {
                Self::LEARNABLE
            } else {
                Self::OBVIOUS
            }) | (key.0 + 1),
        )
    }

    pub fn lowercase() -> Self {
        Self::new(3)
    }

    pub fn placement(placement: SentencePlacement) -> Self {
        Self::new(match placement {
            SentencePlacement::Words => panic!("words have no placement marker"),
            SentencePlacement::Phrases => 1,
            SentencePlacement::LowConfidencePhrases => 2,
        })
    }

    pub fn gap(gap: u8) -> Self {
        Self::new(u32::from(gap) + 4)
    }

    pub fn is_learnable(self) -> bool {
        self.0.get() & Self::LEARNABLE != 0
    }

    /// Call only for a gram symbol (the scan calls this after is_learnable).
    pub fn key(self) -> GramKey {
        GramKey((self.0.get() & Self::PAYLOAD) - 1)
    }

    pub fn kind(self) -> SymbolKind {
        match self.0.get() {
            1 => SymbolKind::Placement(SentencePlacement::Phrases),
            2 => SymbolKind::Placement(SentencePlacement::LowConfidencePhrases),
            3 => SymbolKind::Lowercase,
            value @ 4..=259 => SymbolKind::Gap((value - 4) as u8),
            _ => SymbolKind::Gram(self.key()),
        }
    }
}

pub fn decode(
    symbols: impl Iterator<Item = Symbol>,
    grams: &RodeoReader<SpurReadingGram, GramKey>,
) -> SentenceGrams<SpurReadingGram> {
    let mut sentence = SentenceGrams {
        grams: Vec::new(),
        capitalize_first: true,
        multiword_terms: Vec::new(),
        low_confidence_multiword_terms: Vec::new(),
    };
    let mut placement = SentencePlacement::Words;
    for symbol in symbols {
        match symbol.kind() {
            SymbolKind::Lowercase => sentence.capitalize_first = false,
            SymbolKind::Placement(next) => placement = next,
            SymbolKind::Gram(key) => {
                let gram = *grams.resolve(&key);
                match placement {
                    SentencePlacement::Words => sentence.grams.push(if symbol.is_learnable() {
                        SentenceGram::Learnable(gram)
                    } else {
                        SentenceGram::Obvious(gram)
                    }),
                    SentencePlacement::Phrases => {
                        sentence.multiword_terms.push(MultiwordTermMatch {
                            gram,
                            matched_word_indices: Vec::new(),
                        })
                    }
                    SentencePlacement::LowConfidencePhrases => sentence
                        .low_confidence_multiword_terms
                        .push(MultiwordTermMatch {
                            gram,
                            matched_word_indices: Vec::new(),
                        }),
                }
            }
            SymbolKind::Gap(gap) => {
                let terms = match placement {
                    SentencePlacement::Phrases => &mut sentence.multiword_terms,
                    SentencePlacement::LowConfidencePhrases => {
                        &mut sentence.low_confidence_multiword_terms
                    }
                    SentencePlacement::Words => panic!("gap outside a phrase"),
                };
                let indices = &mut terms.last_mut().unwrap().matched_word_indices;
                indices.push(
                    indices
                        .last()
                        .copied()
                        .unwrap_or(0)
                        .checked_add(u16::from(gap))
                        .unwrap(),
                );
            }
        }
    }
    sentence
}
