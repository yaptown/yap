use crate::indexmap::IndexMap;
use crate::{
    ConsolidatedLanguageData, DictionaryEntry, Frequency, Heteronym, HomophonePractice,
    HomophoneWordPair, Lexeme, Literal, PatternPosition, PhrasebookEntry, PronunciationData,
};
use lasso::Spur;
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct LanguagePack {
    pub rodeo: lasso::RodeoReader,
    pub translations: HashMap<Spur, Vec<Spur>>,
    pub words_to_heteronyms: HashMap<Spur, BTreeSet<Heteronym<Spur>>>,
    pub sentences_containing_lexeme_index: HashMap<Lexeme<Spur>, Vec<Spur>>,
    pub sentences_to_literals: HashMap<Spur, Vec<Literal<Spur>>>,
    pub sentences_to_lexemes: HashMap<Spur, Vec<Lexeme<Spur>>>,
    pub sentences_to_all_lexemes: HashMap<Spur, Vec<Lexeme<Spur>>>,
    pub word_frequencies: IndexMap<Lexeme<Spur>, Frequency>,
    pub total_word_count: u64,
    pub dictionary: BTreeMap<Heteronym<Spur>, DictionaryEntry>,
    pub phrasebook: BTreeMap<Spur, PhrasebookEntry>,
    pub word_to_pronunciation: HashMap<Spur, Spur>,
    pub pronunciation_to_words: HashMap<Spur, Vec<Spur>>,
    pub pronunciation_data: PronunciationData,
    pub pattern_frequency_map: HashMap<(Spur, PatternPosition), u32>,
    pub homophone_practice: HashMap<HomophoneWordPair<Spur>, HomophonePractice<Spur>>,
}

impl LanguagePack {
    /// Get all lexemes for words that share a pronunciation
    /// Returns an iterator over (word, lexeme) pairs
    pub fn pronunciation_to_lexemes(
        &self,
        pronunciation: &Spur,
    ) -> impl Iterator<Item = (Spur, Lexeme<Spur>)> + '_ {
        self.pronunciation_to_words
            .get(pronunciation)
            .into_iter()
            .flat_map(|words| words.iter())
            .flat_map(move |word| {
                self.words_to_heteronyms
                    .get(word)
                    .into_iter()
                    .flat_map(|heteronyms| heteronyms.iter())
                    .map(move |heteronym| (*word, Lexeme::Heteronym(*heteronym)))
            })
    }

    /// Get the maximum frequency for any word with this pronunciation
    pub fn pronunciation_max_frequency(&self, pronunciation: &Spur) -> Option<Frequency> {
        self.pronunciation_to_lexemes(pronunciation)
            .filter_map(|(_, lexeme)| self.word_frequencies.get(&lexeme).copied())
            .max()
    }

    pub fn new(language_data: ConsolidatedLanguageData) -> Self {
        let rodeo = {
            let mut rodeo = lasso::Rodeo::new();
            language_data.intern(&mut rodeo);
            rodeo.into_reader()
        };

        let sentences: Vec<Spur> = {
            language_data
                .target_language_sentences
                .iter()
                .map(|s| rodeo.get(s).unwrap())
                .collect()
        };

        let translations = {
            language_data
                .translations
                .iter()
                .map(|(target_language, native_languages)| {
                    (
                        rodeo.get(target_language).unwrap(),
                        native_languages
                            .iter()
                            .map(|n| rodeo.get(n).unwrap())
                            .collect(),
                    )
                })
                .collect()
        };

        let words_to_heteronyms = {
            let mut map: HashMap<Spur, BTreeSet<Heteronym<Spur>>> = HashMap::new();

            for freq in &language_data.frequencies {
                if let Lexeme::Heteronym(heteronym) = &freq.lexeme {
                    let word_spur = rodeo.get(&heteronym.word).unwrap();
                    map.entry(word_spur).or_default().insert({
                        Heteronym {
                            word: rodeo.get(&heteronym.word).unwrap(),
                            lemma: rodeo.get(&heteronym.lemma).unwrap(),
                            pos: heteronym.pos,
                        }
                    });
                }
            }

            map
        };

        let sentences_to_literals = {
            language_data
                .nlp_sentences
                .iter()
                .map(|(sentence, analysis)| {
                    (
                        rodeo.get(sentence).unwrap(),
                        analysis
                            .words
                            .iter()
                            .map(|word| {
                                word.get_interned(&rodeo).unwrap_or_else(|| {
                                    panic!("word not in rodeo: {word:?} in sentence: {sentence:?}")
                                })
                            })
                            .collect(),
                    )
                })
                .collect()
        };

        let sentences_to_lexemes: HashMap<Spur, Vec<Lexeme<Spur>>> = {
            language_data
                .nlp_sentences
                .iter()
                .map(|(sentence, analysis)| {
                    (
                        rodeo.get(sentence).unwrap(),
                        analysis
                            .lexemes()
                            .map(|l| l.get_interned(&rodeo).unwrap())
                            .collect(),
                    )
                })
                .collect()
        };

        let sentences_containing_lexeme_index = {
            let mut map = HashMap::new();
            for (i, sentence_spur) in sentences.iter().enumerate() {
                let _sentence = rodeo.resolve(sentence_spur);
                let Some(lexemes) = sentences_to_lexemes.get(sentence_spur) else {
                    continue;
                };
                for lexeme in lexemes.iter().cloned() {
                    map.entry(lexeme).or_insert(vec![]).push(sentences[i]);
                }
            }
            map
        };

        let sentences_to_all_lexemes = {
            language_data
                .nlp_sentences
                .iter()
                .map(|(sentence, analysis)| {
                    (
                        rodeo.get(sentence).unwrap(),
                        analysis
                            .all_lexemes()
                            .map(|l| l.get_interned(&rodeo).unwrap())
                            .collect(),
                    )
                })
                .collect()
        };

        let word_frequencies = {
            let mut map = IndexMap::new();
            for freq in &language_data.frequencies {
                map.insert(
                    freq.lexeme.get_interned(&rodeo).unwrap(),
                    Frequency { count: freq.count },
                );
            }
            map
        };

        let total_word_count = {
            language_data
                .frequencies
                .iter()
                .map(|freq| freq.count as u64)
                .sum()
        };

        let dictionary = {
            language_data
                .dictionary
                .iter()
                .map(|(heteronym, entry)| {
                    (
                        heteronym
                            .get_interned(&rodeo)
                            .unwrap_or_else(|| panic!("heteronym not in rodeo: {heteronym:?}")),
                        entry.clone(),
                    )
                })
                .collect()
        };

        let phrasebook = {
            language_data
                .phrasebook
                .iter()
                .map(|(multiword_term, entry)| (rodeo.get(multiword_term).unwrap(), entry.clone()))
                .collect()
        };

        let word_to_pronunciation = {
            language_data
                .word_to_pronunciation
                .iter()
                .map(|(word, pronunciation)| {
                    (
                        rodeo
                            .get(word)
                            .unwrap_or_else(|| panic!("word not in rodeo: {word:?}")),
                        rodeo.get(pronunciation).unwrap_or_else(|| {
                            panic!("pronunciation not in rodeo: {pronunciation:?}")
                        }),
                    )
                })
                .collect()
        };

        let pronunciation_to_words = {
            language_data
                .pronunciation_to_words
                .iter()
                .map(|(pronunciation, words)| {
                    (
                        rodeo.get(pronunciation).unwrap(),
                        words.iter().map(|word| rodeo.get(word).unwrap()).collect(),
                    )
                })
                .collect()
        };

        let pronunciation_data = language_data.pronunciation_data.clone();

        let pattern_frequency_map = {
            pronunciation_data
                .pattern_frequencies
                .iter()
                .map(|((pattern, position), freq)| {
                    ((rodeo.get(pattern).unwrap(), *position), *freq)
                })
                .collect()
        };

        let homophone_practice = language_data
            .homophone_practice
            .iter()
            .map(|(word_pair, practice)| {
                (
                    word_pair.get_interned(&rodeo).unwrap(),
                    practice.get_interned(&rodeo).unwrap(),
                )
            })
            .collect();

        Self {
            rodeo,
            translations,
            words_to_heteronyms,
            sentences_containing_lexeme_index,
            sentences_to_literals,
            sentences_to_lexemes,
            sentences_to_all_lexemes,
            word_frequencies,
            total_word_count,
            dictionary,
            phrasebook,
            word_to_pronunciation,
            pronunciation_to_words,
            pronunciation_data,
            pattern_frequency_map,
            homophone_practice,
        }
    }
}
