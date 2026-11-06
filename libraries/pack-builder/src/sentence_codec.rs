use language_utils::sentence_codec::{GramKey, SentencePlacement, Symbol};
use language_utils::{SentenceGram, SentenceGrams, SpurReadingGram};
use lasso::Key;

pub(crate) fn encode(
    sentence: &SentenceGrams<SpurReadingGram>,
    grams: &mut lasso::Rodeo<SpurReadingGram, GramKey>,
    learnable_count: usize,
) -> Vec<Symbol> {
    let mut out = Vec::new();
    if !sentence.capitalize_first {
        out.push(Symbol::lowercase());
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
        (&sentence.multiword_terms, SentencePlacement::Phrases),
        (
            &sentence.low_confidence_multiword_terms,
            SentencePlacement::LowConfidencePhrases,
        ),
    ] {
        if !terms.is_empty() {
            out.push(Symbol::placement(marker));
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
                out.push(Symbol::gap(gap));
                previous = index;
            }
        }
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    use language_utils::sentence_codec::decode;
    use language_utils::{
        MultiwordTermMatch, ReadingTaggedGram, SentenceSpur, SpurGram, TaggedGram,
    };
    use lasso::RodeoReader;

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
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&(compressed, grams)).unwrap();
        type Stored = (
            lasso::CompressedRodeoResolver<SentenceSpur, Vec<Symbol>>,
            RodeoReader<SpurReadingGram, GramKey>,
        );
        let archived = rkyv::access::<rkyv::Archived<Stored>, rkyv::rancor::Error>(&bytes).unwrap();
        assert_eq!(
            decode(archived.0.resolve(&key).copied(), &archived.1),
            sentence
        );
    }
}
