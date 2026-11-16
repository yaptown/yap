use crate::UnigramSplitter;
use language_utils::sentence_codec::GramKey;
use language_utils::sentence_phonemes::{
    GramPhonemes, PhonemeSequenceSpur, PhonemeVariantListSpur,
};
use language_utils::{SentenceGram, SentenceGrams, SpurReadingGram};
use lasso::Key;
use rustc_hash::FxHashMap;

/// Each gram's phoneme sequences across the corpus, most common first.
pub(crate) struct Ranked<'a> {
    lists: FxHashMap<GramKey, Vec<&'a str>>,
    variants: FxHashMap<(GramKey, &'a str), u32>,
}

/// `phonemes` are each sentence's sequences, parallel to its grams.
pub(crate) fn rank<'a>(
    sentences: impl Iterator<Item = (&'a SentenceGrams<SpurReadingGram>, Option<&'a [String]>)>,
    grams: &mut lasso::Rodeo<SpurReadingGram, GramKey>,
) -> Ranked<'a> {
    let mut counts: FxHashMap<GramKey, FxHashMap<&str, u64>> = FxHashMap::default();
    for (sentence, phonemes) in sentences {
        let Some(phonemes) = phonemes else { continue };
        assert_eq!(
            phonemes.len(),
            sentence.grams.len(),
            "one sequence per gram"
        );
        for (gram, sequence) in sentence.grams.iter().zip(phonemes) {
            let (SentenceGram::Learnable(gram) | SentenceGram::Obvious(gram)) = gram;
            *counts
                .entry(grams.get_or_intern(*gram))
                .or_default()
                .entry(sequence)
                .or_default() += 1;
        }
    }
    let mut variants = FxHashMap::default();
    let lists = counts
        .into_iter()
        .map(|(key, counts)| {
            let mut list: Vec<_> = counts.into_iter().collect();
            list.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
            let list: Vec<_> = list.into_iter().map(|(sequence, _)| sequence).collect();
            for (variant, &sequence) in list.iter().enumerate() {
                variants.insert((key, sequence), u32::try_from(variant).unwrap());
            }
            (key, list)
        })
        .collect();
    Ranked { lists, variants }
}

impl Ranked<'_> {
    /// Each gram's variant index, parallel to the sentence's grams.
    pub(crate) fn variants(
        &self,
        sentence: &SentenceGrams<SpurReadingGram>,
        phonemes: &[String],
        grams: &lasso::Rodeo<SpurReadingGram, GramKey>,
    ) -> Vec<u32> {
        sentence
            .grams
            .iter()
            .zip(phonemes)
            .map(|(gram, sequence)| {
                let (SentenceGram::Learnable(gram) | SentenceGram::Obvious(gram)) = gram;
                self.variants[&(grams.get(gram).unwrap(), sequence.as_str())]
            })
            .collect()
    }

    pub(crate) fn into_table(self, gram_count: usize) -> GramPhonemes {
        let sequences = lasso::CompressedRodeoReader::<
            PhonemeSequenceSpur,
            lasso::RunTable,
            lasso::Text,
        >::build::<UnigramSplitter>(
            self.lists
                .values()
                .flatten()
                .map(|sequence| sequence.to_string())
                .collect(),
        );
        let mut variant_lists =
            lasso::Rodeo::<Vec<PhonemeSequenceSpur>, PhonemeVariantListSpur>::new();
        let mut by_gram = vec![None; gram_count];
        let mut lists: Vec<_> = self.lists.into_iter().collect();
        lists.sort_unstable_by_key(|(key, _)| *key);
        for (key, list) in lists {
            let list: Vec<_> = list
                .into_iter()
                .map(|sequence| sequences.get(&(), sequence).unwrap())
                .collect();
            by_gram[key.into_usize()] = Some(variant_lists.get_or_intern(list));
        }
        GramPhonemes {
            sequences,
            variant_lists: variant_lists.into_resolver(),
            by_gram,
        }
    }
}
