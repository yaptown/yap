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
pub struct Symbol(NonZeroU32);

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

pub(crate) fn encode(
    sentence: &SentenceGrams<SpurReadingGram>,
    grams: &mut lasso::Rodeo<SpurReadingGram, GramKey>,
    learnable_count: usize,
) -> Vec<Symbol> {
    let mut out = Vec::new();
    if !sentence.capitalize_first {
        out.push(Symbol::new(3));
    }
    for gram in &sentence.grams {
        let (gram, learnable) = match gram {
            SentenceGram::Learnable(gram) => (gram, true),
            SentenceGram::Obvious(gram) => (gram, false),
        };
        let key = grams.get_or_intern(*gram);
        assert!(
            !learnable || key.into_usize() < learnable_count,
            "learnable gram has no ease rank"
        );
        out.push(Symbol::gram(key, learnable));
    }
    for (terms, marker) in [
        (&sentence.multiword_terms, 1),
        (&sentence.low_confidence_multiword_terms, 2),
    ] {
        if !terms.is_empty() {
            out.push(Symbol::new(marker));
        }
        for term in terms {
            let key = grams.get(term.gram).expect("phrase has no ease rank");
            assert!(
                key.into_usize() < learnable_count,
                "phrase has no ease rank"
            );
            out.push(Symbol::gram(key, true));
            let mut previous = 0;
            for &index in &term.matched_word_indices {
                let gap = u8::try_from(index.checked_sub(previous).expect("sorted phrase indices"))
                    .expect("phrase word gaps fit in a byte");
                out.push(Symbol::new(u32::from(gap) + 4));
                previous = index;
            }
        }
    }
    out
}

pub(crate) fn decode(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ReadingTaggedGram, SentenceSpur, SpurGram, TaggedGram};

    #[test]
    fn packed_symbols_roundtrip_through_compression_and_archive() {
        assert_eq!(std::mem::size_of::<Symbol>(), 4);
        assert_eq!(std::mem::size_of::<rkyv::Archived<Symbol>>(), 4);
        assert_eq!(std::mem::size_of::<Option<Symbol>>(), 4);
        let reading = |index| {
            ReadingTaggedGram::without_reading(TaggedGram {
                gram: SpurGram::try_from_usize(index).unwrap(),
                sense: None,
            })
        };
        let mut grams = lasso::Rodeo::<SpurReadingGram, GramKey>::new();
        let learnable = reading(0);
        let obvious = reading(1);
        assert_eq!(grams.get_or_intern(learnable).index(), 0);
        let sentence = SentenceGrams {
            grams: vec![
                SentenceGram::Learnable(learnable),
                SentenceGram::Obvious(obvious),
            ],
            capitalize_first: false,
            multiword_terms: vec![MultiwordTermMatch {
                gram: learnable,
                matched_word_indices: vec![0, 255, 510],
            }],
            low_confidence_multiword_terms: vec![MultiwordTermMatch {
                gram: learnable,
                matched_word_indices: vec![1, 3],
            }],
        };
        let symbols = encode(&sentence, &mut grams, 1);
        assert_eq!(
            symbols
                .iter()
                .filter(|symbol| symbol.is_learnable())
                .count(),
            3
        );
        assert!(
            symbols
                .iter()
                .filter(|symbol| symbol.is_learnable())
                .all(|symbol| symbol.key().index() == 0)
        );
        let mut sequences = lasso::Rodeo::<Vec<Symbol>, SentenceSpur>::new();
        let key = sequences.get_or_intern(symbols);
        let compressed = sequences.into_compressed_resolver();
        let grams = grams.into_reader();
        assert_eq!(decode(compressed.resolve(&key).copied(), &grams), sentence);
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&(compressed, grams)).unwrap();
        type Stored = (
            lasso::CompressedRodeoResolver<SentenceSpur, Symbol>,
            RodeoReader<SpurReadingGram, GramKey>,
        );
        let (compressed, grams): Stored =
            rkyv::from_bytes::<Stored, rkyv::rancor::Error>(&bytes).unwrap();
        assert_eq!(decode(compressed.resolve(&key).copied(), &grams), sentence);
    }
}
