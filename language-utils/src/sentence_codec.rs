//! Lossless sentence grammar symbols. The symbol interner exists only while building.
use crate::{
    MultiwordTermMatch, ReadingTaggedGram, SentenceGram, SentenceGrams, SentenceSpur, SpurGram,
};
use lasso::{CompressedRodeoResolver, Key, Rodeo, Spur};

pub type Sentence = SentenceGrams<ReadingTaggedGram<SpurGram, Spur>>;
pub type SentenceResolver = CompressedRodeoResolver<SentenceSpur, Spur<SentenceSymbol>>;

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub enum SentenceList {
    Learnable,
    Obvious,
    Multiword,
    LowConfidenceMultiword,
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
pub enum SentenceSymbol {
    Gram {
        entry: ReadingTaggedGram<SpurGram, Spur>,
        list: SentenceList,
    },
    /// Difference from the previous matched index, reset to zero at each phrase.
    MatchedWord(u16),
    Lowercase,
}

pub(crate) fn encode(sentence: &Sentence) -> Vec<SentenceSymbol> {
    let mut out = Vec::new();
    if !sentence.capitalize_first {
        out.push(SentenceSymbol::Lowercase);
    }
    for gram in &sentence.grams {
        let (entry, list) = match gram {
            SentenceGram::Learnable(g) => (*g, SentenceList::Learnable),
            SentenceGram::Obvious(g) => (*g, SentenceList::Obvious),
        };
        out.push(SentenceSymbol::Gram { entry, list });
    }
    for (terms, list) in [
        (&sentence.multiword_terms, SentenceList::Multiword),
        (
            &sentence.low_confidence_multiword_terms,
            SentenceList::LowConfidenceMultiword,
        ),
    ] {
        for term in terms {
            out.push(SentenceSymbol::Gram {
                entry: term.gram,
                list,
            });
            let mut previous = 0;
            for &index in &term.matched_word_indices {
                out.push(SentenceSymbol::MatchedWord(
                    index
                        .checked_sub(previous)
                        .expect("sorted matched word indices"),
                ));
                previous = index;
            }
        }
    }
    out
}

pub(crate) fn decode(symbols: impl Iterator<Item = SentenceSymbol>) -> Sentence {
    let mut sentence = Sentence {
        grams: Vec::new(),
        capitalize_first: true,
        multiword_terms: Vec::new(),
        low_confidence_multiword_terms: Vec::new(),
    };
    let mut list = SentenceList::Learnable;
    for symbol in symbols {
        match symbol {
            SentenceSymbol::Lowercase => sentence.capitalize_first = false,
            SentenceSymbol::Gram { entry, list: next } => {
                list = next;
                match list {
                    SentenceList::Learnable => sentence.grams.push(SentenceGram::Learnable(entry)),
                    SentenceList::Obvious => sentence.grams.push(SentenceGram::Obvious(entry)),
                    SentenceList::Multiword => sentence.multiword_terms.push(MultiwordTermMatch {
                        gram: entry,
                        matched_word_indices: Vec::new(),
                    }),
                    SentenceList::LowConfidenceMultiword => sentence
                        .low_confidence_multiword_terms
                        .push(MultiwordTermMatch {
                            gram: entry,
                            matched_word_indices: Vec::new(),
                        }),
                }
            }
            SentenceSymbol::MatchedWord(delta) => {
                let terms = match list {
                    SentenceList::Multiword => &mut sentence.multiword_terms,
                    SentenceList::LowConfidenceMultiword => {
                        &mut sentence.low_confidence_multiword_terms
                    }
                    _ => panic!("matched index outside a phrase"),
                };
                let indices = &mut terms.last_mut().unwrap().matched_word_indices;
                indices.push(
                    indices
                        .last()
                        .copied()
                        .unwrap_or(0)
                        .checked_add(delta)
                        .unwrap(),
                );
            }
        }
    }
    sentence
}

pub(crate) fn compress(
    sentences: &[(SentenceSpur, Sentence)],
) -> (SentenceResolver, Vec<SentenceSymbol>) {
    let mut symbols = Vec::new();
    let mut symbol_keys = rustc_hash::FxHashMap::default();
    let mut sequences = Rodeo::<Vec<Spur<SentenceSymbol>>, SentenceSpur>::new();
    for (text, sentence) in sentences {
        let encoded = encode(sentence);
        let keys: Vec<_> = encoded
            .into_iter()
            .map(|symbol| {
                *symbol_keys.entry(symbol).or_insert_with(|| {
                    let key = Spur::<SentenceSymbol>::try_from_usize(symbols.len()).unwrap();
                    symbols.push(symbol);
                    key
                })
            })
            .collect();
        assert_eq!(
            sequences.get_or_intern(keys),
            *text,
            "sentence sequence deduplication/order must match text spurs"
        );
    }
    let resolver = sequences.into_compressed_resolver();
    for (text, sentence) in sentences {
        assert_eq!(
            decode(resolver.resolve(text).map(|key| symbols[key.into_usize()])),
            *sentence,
            "compressed sentence roundtrip"
        );
    }
    (resolver, symbols)
}

pub(crate) fn empty() -> SentenceResolver {
    Rodeo::<Vec<Spur<SentenceSymbol>>, SentenceSpur>::new().into_compressed_resolver()
}
