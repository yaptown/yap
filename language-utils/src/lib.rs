pub mod features;
pub mod indexmap;
pub mod language_pack;
pub mod profile;
pub mod text_cleanup;

use std::collections::BTreeMap;
use std::hash::Hash;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::features::Morphology;

#[derive(
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Hash,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Copy,
    tsify::Tsify,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[rkyv(
    compare(PartialEq, PartialOrd),
    derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash)
)]
pub enum PartOfSpeech {
    #[serde(rename = "ADJ")]
    Adj, // adjective
    #[serde(rename = "ADP")]
    Adp, // adposition
    #[serde(rename = "ADV")]
    Adv, // adverb
    #[serde(rename = "AUX")]
    Aux, // auxiliary
    #[serde(rename = "CCONJ")]
    Cconj, // coordinating conjunction
    #[serde(rename = "DET")]
    Det, // determiner
    #[serde(rename = "INTJ")]
    Intj, // interjection
    #[serde(rename = "NOUN")]
    Noun, // noun
    #[serde(rename = "NUM")]
    Num, // numeral
    #[serde(rename = "PART")]
    Part, // particle
    #[serde(rename = "PRON")]
    Pron, // pronoun
    #[serde(rename = "PROPN")]
    Propn, // proper noun
    #[serde(rename = "PUNCT")]
    Punct, // punctuation
    #[serde(rename = "SCONJ")]
    Sconj, // subordinating conjunction
    #[serde(rename = "SYM")]
    Sym, // symbol
    #[serde(rename = "VERB")]
    Verb, // verb
    #[serde(rename = "SPACE")]
    Space, // space
    #[serde(rename = "X")]
    X, // other
}

impl std::fmt::Display for PartOfSpeech {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let word = match self {
            PartOfSpeech::Adj => "adjective",
            PartOfSpeech::Adp => "adposition",
            PartOfSpeech::Adv => "adverb",
            PartOfSpeech::Aux => "auxiliary",
            PartOfSpeech::Cconj => "coordinating conjunction",
            PartOfSpeech::Det => "determiner",
            PartOfSpeech::Intj => "interjection",
            PartOfSpeech::Noun => "noun",
            PartOfSpeech::Num => "numeral",
            PartOfSpeech::Part => "particle",
            PartOfSpeech::Pron => "pronoun",
            PartOfSpeech::Propn => "proper noun",
            PartOfSpeech::Punct => "punctuation",
            PartOfSpeech::Sconj => "subordinating conjunction",
            PartOfSpeech::Sym => "symbol",
            PartOfSpeech::Verb => "verb",
            PartOfSpeech::Space => "space",
            PartOfSpeech::X => "other",
        };
        write!(f, "{word}")
    }
}

#[derive(
    Clone,
    Debug,
    serde::Deserialize,
    schemars::JsonSchema,
    serde::Serialize,
    tsify::Tsify,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct TargetToNativeWord {
    pub native: String,
    pub note: Option<String>,
    pub example_sentence_target_language: String,
    pub example_sentence_native_language: String,
}

#[derive(
    Clone,
    Debug,
    serde::Deserialize,
    schemars::JsonSchema,
    serde::Serialize,
    tsify::Tsify,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct PhrasebookEntryThoughts {
    pub thoughts: String,
    pub target_language_multi_word_term: String,
    pub meaning: String,
    pub additional_notes: String,
    pub target_language_example: String,
    pub native_language_example: String,
}

#[derive(
    Clone,
    Debug,
    serde::Deserialize,
    schemars::JsonSchema,
    serde::Serialize,
    tsify::Tsify,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct PhrasebookEntry {
    pub target_language_multi_word_term: String,
    pub meaning: String,
    pub additional_notes: String,
    pub target_language_example: String,
    pub native_language_example: String,
}

impl From<PhrasebookEntryThoughts> for PhrasebookEntry {
    fn from(entry: PhrasebookEntryThoughts) -> Self {
        Self {
            target_language_multi_word_term: entry.target_language_multi_word_term,
            meaning: entry.meaning,
            additional_notes: entry.additional_notes,
            target_language_example: entry.target_language_example,
            native_language_example: entry.native_language_example,
        }
    }
}

#[derive(
    Clone,
    Debug,
    serde::Deserialize,
    schemars::JsonSchema,
    serde::Serialize,
    tsify::Tsify,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct DictionaryEntryThoughts {
    pub thoughts: String,
    pub target_language_word: String,
    pub definitions: Vec<TargetToNativeWord>,
}

#[derive(
    Clone,
    Debug,
    serde::Deserialize,
    schemars::JsonSchema,
    serde::Serialize,
    tsify::Tsify,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct DictionaryEntry {
    pub target_language_word: String,
    pub definitions: Vec<TargetToNativeWord>,
    pub morphology: Morphology,
}

impl From<(DictionaryEntryThoughts, Morphology)> for DictionaryEntry {
    fn from(entry: (DictionaryEntryThoughts, Morphology)) -> Self {
        let (entry, morphology) = entry;
        Self {
            target_language_word: entry.target_language_word,
            definitions: entry.definitions,
            morphology,
        }
    }
}

/// Tracks the source(s) of a sentence. Since a sentence can appear in multiple sources,
/// we use boolean fields for each source type.
#[derive(
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct SentenceSource {
    /// Sentence came from an Anki deck
    pub from_anki: bool,
    /// Sentence came from Tatoeba
    pub from_tatoeba: bool,
    /// Sentence was manually added to extra/manual.txt
    pub from_manual: bool,
    /// Sentence came from a song in sentence-sources/songs/
    pub from_song: bool,
}

impl SentenceSource {
    /// Create a new source with all fields set to false
    pub fn none() -> Self {
        Self {
            from_anki: false,
            from_tatoeba: false,
            from_manual: false,
            from_song: false,
        }
    }

    /// Returns true if the sentence came from a manual source (should never be filtered)
    pub fn is_manual(&self) -> bool {
        self.from_manual
    }

    /// Merge two sources together (OR operation on all fields)
    pub fn merge(&mut self, other: &Self) {
        self.from_anki |= other.from_anki;
        self.from_tatoeba |= other.from_tatoeba;
        self.from_manual |= other.from_manual;
        self.from_song |= other.from_song;
    }
}

#[derive(
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct Sentence {
    pub french: Vec<String>,
    pub english: String,
}

#[derive(
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Hash,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct MultiwordTerms {
    pub high_confidence: Vec<String>,
    pub low_confidence: Vec<String>,
}

/// The raw output from the Spacy python script
#[derive(
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct NlpAnalyzedSentence {
    pub sentence: String,
    pub multiword_terms: MultiwordTerms,
    pub doc: Vec<DocToken>,
}

/// A more condensed version of NlpAnalyzedSentence
#[derive(
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Hash,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    tsify::Tsify,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(compare(PartialEq))]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct SentenceInfo {
    pub words: Vec<Literal<String>>,
    pub multiword_terms: MultiwordTerms,
}

impl SentenceInfo {
    /// The words and the high-confidence multiword terms
    pub fn lexemes(&self) -> impl Iterator<Item = Lexeme<String>> {
        self.words
            .iter()
            .filter_map(|token| {
                token
                    .heteronym
                    .as_ref()
                    .map(|heteronym| Lexeme::Heteronym(heteronym.clone()))
            })
            .chain(
                self.multiword_terms
                    .high_confidence
                    .iter()
                    .map(|term| Lexeme::Multiword(term.clone())),
            )
    }

    /// The words and the high-confidence and low-confidence multiword terms
    pub fn all_lexemes(&self) -> impl Iterator<Item = Lexeme<String>> {
        self.lexemes().chain(
            self.multiword_terms
                .low_confidence
                .iter()
                .map(|term| Lexeme::Multiword(term.clone())),
        )
    }
}

#[derive(
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Hash,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
pub struct DocToken {
    pub text: String,
    pub whitespace: String,
    pub pos: PartOfSpeech,
    pub lemma: String,
    pub morph: BTreeMap<String, String>,
}

#[derive(
    Copy,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Hash,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    tsify::Tsify,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
#[rkyv(
    compare(PartialEq, PartialOrd),
    derive(PartialEq, PartialOrd, Eq, Ord, Hash)
)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Heteronym<S>
where
    S: rkyv::Archive,
    <S as rkyv::Archive>::Archived: PartialEq + PartialOrd + Eq + Ord + Hash,
{
    pub word: S,
    pub lemma: S,
    pub pos: PartOfSpeech,
}

#[derive(
    Copy,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Hash,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    tsify::Tsify,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(compare(PartialEq), derive(PartialEq, PartialOrd, Eq, Ord, Hash))]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Literal<S>
where
    S: rkyv::Archive,
    <S as rkyv::Archive>::Archived: PartialEq + PartialOrd + Eq + Ord + Hash,
    <Option<Heteronym<S>> as rkyv::Archive>::Archived: PartialEq + PartialOrd + Eq + Ord + Hash,
{
    pub text: S,
    pub whitespace: S,
    pub heteronym: Option<Heteronym<S>>,
}

impl Literal<String> {
    pub fn get_or_intern(&self, rodeo: &mut lasso::Rodeo) -> Literal<lasso::Spur> {
        Literal {
            text: rodeo.get_or_intern(&self.text),
            whitespace: rodeo.get_or_intern(&self.whitespace),
            heteronym: self.heteronym.as_ref().map(|h| h.get_or_intern(rodeo)),
        }
    }

    pub fn get_interned(&self, rodeo: &lasso::RodeoReader) -> Option<Literal<lasso::Spur>> {
        let heteronym = match self.heteronym.as_ref() {
            Some(h) => Some(h.get_interned(rodeo)?),
            None => None,
        };
        Some(Literal {
            text: rodeo.get(&self.text)?,
            whitespace: rodeo.get(&self.whitespace)?,
            heteronym,
        })
    }
}

impl Literal<lasso::Spur> {
    pub fn resolve(&self, rodeo: &lasso::RodeoReader) -> Literal<String> {
        Literal {
            text: rodeo.resolve(&self.text).to_string(),
            whitespace: rodeo.resolve(&self.whitespace).to_string(),
            heteronym: self.heteronym.as_ref().map(|h| h.resolve(rodeo)),
        }
    }
}

impl Heteronym<String> {
    pub fn get_or_intern(&self, rodeo: &mut lasso::Rodeo) -> Heteronym<lasso::Spur> {
        let word = rodeo.get_or_intern(&self.word);
        let lemma = rodeo.get_or_intern(&self.lemma);
        Heteronym {
            word,
            lemma,
            pos: self.pos,
        }
    }

    pub fn get_interned(&self, rodeo: &lasso::RodeoReader) -> Option<Heteronym<lasso::Spur>> {
        let word = rodeo.get(&self.word)?;
        let lemma = rodeo.get(&self.lemma)?;
        Some(Heteronym {
            word,
            lemma,
            pos: self.pos,
        })
    }
}

impl Heteronym<lasso::Spur> {
    pub fn resolve(&self, rodeo: &lasso::RodeoReader) -> Heteronym<String> {
        Heteronym {
            word: rodeo.resolve(&self.word).to_string(),
            lemma: rodeo.resolve(&self.lemma).to_string(),
            pos: self.pos,
        }
    }
}

#[derive(
    Copy,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Hash,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    tsify::Tsify,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
#[rkyv(compare(PartialEq), derive(PartialEq, PartialOrd, Eq, Ord, Hash))]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum Lexeme<S>
where
    S: rkyv::Archive,
    <S as rkyv::Archive>::Archived: PartialEq + PartialOrd + Eq + Ord + Hash,
    <Heteronym<S> as rkyv::Archive>::Archived: PartialEq + PartialOrd + Eq + Ord + Hash,
{
    Heteronym(Heteronym<S>),
    Multiword(S),
}

impl<S> Lexeme<S>
where
    S: rkyv::Archive,
    <S as rkyv::Archive>::Archived: PartialEq + PartialOrd + Eq + Ord + Hash,
    <Heteronym<S> as rkyv::Archive>::Archived: PartialEq + PartialOrd + Eq + Ord + Hash,
{
    pub fn heteronym(&self) -> Option<&Heteronym<S>> {
        match self {
            Lexeme::Heteronym(heteronym) => Some(heteronym),
            _ => None,
        }
    }

    pub fn multiword(&self) -> Option<&S> {
        match self {
            Lexeme::Multiword(multiword) => Some(multiword),
            _ => None,
        }
    }
}

impl Lexeme<String> {
    pub fn get_or_intern(&self, rodeo: &mut lasso::Rodeo) -> Lexeme<lasso::Spur> {
        match self {
            Lexeme::Heteronym(heteronym) => Lexeme::Heteronym(heteronym.get_or_intern(rodeo)),
            Lexeme::Multiword(multiword) => Lexeme::Multiword(rodeo.get_or_intern(multiword)),
        }
    }

    pub fn get_interned(&self, rodeo: &lasso::RodeoReader) -> Option<Lexeme<lasso::Spur>> {
        match self {
            Lexeme::Heteronym(heteronym) => Some(Lexeme::Heteronym(heteronym.get_interned(rodeo)?)),
            Lexeme::Multiword(multiword) => Some(Lexeme::Multiword(rodeo.get(multiword)?)),
        }
    }
}

impl Lexeme<lasso::Spur> {
    pub fn resolve(&self, rodeo: &lasso::RodeoReader) -> Lexeme<String> {
        match self {
            Lexeme::Heteronym(heteronym) => Lexeme::Heteronym(heteronym.resolve(rodeo)),
            Lexeme::Multiword(multiword) => Lexeme::Multiword(rodeo.resolve(multiword).to_string()),
        }
    }
}

#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(compare(PartialEq), derive(PartialEq, PartialOrd, Eq, Ord, Hash))]
pub struct FrequencyEntry<S>
where
    S: rkyv::Archive + PartialEq + PartialOrd + Eq + Ord + Hash,
    <S as rkyv::Archive>::Archived: PartialEq + PartialOrd + Eq + Ord + Hash,
    <Lexeme<S> as rkyv::Archive>::Archived: PartialEq + PartialOrd + Eq + Ord + Hash,
{
    pub lexeme: Lexeme<S>,
    pub count: u32,
}

#[derive(
    Copy,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(compare(PartialEq), derive(PartialEq, PartialOrd, Eq, Ord, Hash))]
pub struct Frequency {
    pub count: u32,
}

impl Frequency {
    pub fn sqrt_frequency(&self) -> f64 {
        (self.count as f64).sqrt()
    }
}

impl FrequencyEntry<String> {
    pub fn get_or_intern(&self, rodeo: &mut lasso::Rodeo) -> FrequencyEntry<lasso::Spur> {
        FrequencyEntry {
            lexeme: self.lexeme.get_or_intern(rodeo),
            count: self.count,
        }
    }

    pub fn get_interned(&self, rodeo: &lasso::RodeoReader) -> Option<FrequencyEntry<lasso::Spur>> {
        Some(FrequencyEntry {
            lexeme: self.lexeme.get_interned(rodeo)?,
            count: self.count,
        })
    }
}

pub mod autograde {
    use super::*;

    #[derive(
        Clone,
        Debug,
        serde::Serialize,
        serde::Deserialize,
        schemars::JsonSchema,
        tsify::Tsify,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
    )]
    #[tsify(into_wasm_abi, from_wasm_abi)]
    pub enum Remembered {
        Remembered,
        Forgot,
    }

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, tsify::Tsify)]
    #[tsify(into_wasm_abi, from_wasm_abi)]
    pub struct AutoGradeTranslationRequest {
        pub language: Language,
        pub challenge_sentence: String,
        pub user_sentence: String,
        pub primary_expression: Lexeme<String>,
        pub lexemes: Vec<Lexeme<String>>,
    }
    #[derive(
        Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema, tsify::Tsify,
    )]
    #[tsify(into_wasm_abi, from_wasm_abi)]
    pub struct AutoGradeTranslationResponse {
        pub explanation: Option<String>,
        pub primary_expression_status: Remembered,
        pub expressions_remembered: Vec<Lexeme<String>>,
        pub expressions_forgot: Vec<Lexeme<String>>,
    }

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, tsify::Tsify)]
    #[tsify(into_wasm_abi, from_wasm_abi)]
    pub struct AutoGradeTranscriptionRequest {
        pub language: Language,
        pub submission: Vec<transcription_challenge::PartSubmitted>,
    }
}

pub mod transcription_challenge {
    use super::*;

    #[derive(
        Clone,
        Debug,
        serde::Serialize,
        serde::Deserialize,
        tsify::Tsify,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
    )]
    #[tsify(namespace, into_wasm_abi, from_wasm_abi)]
    pub enum Part {
        AskedToTranscribe { parts: Vec<Literal<String>> },
        Provided { part: Literal<String> },
    }

    #[derive(
        Clone,
        Debug,
        serde::Serialize,
        serde::Deserialize,
        tsify::Tsify,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
    )]
    #[tsify(namespace, into_wasm_abi, from_wasm_abi)]
    pub enum PartSubmitted {
        AskedToTranscribe {
            parts: Vec<Literal<String>>,
            submission: String,
        },
        Provided {
            part: Literal<String>,
        },
    }

    #[derive(
        Clone,
        Debug,
        serde::Serialize,
        serde::Deserialize,
        tsify::Tsify,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
    )]
    #[tsify(namespace, into_wasm_abi, from_wasm_abi)]
    pub enum PartGraded {
        AskedToTranscribe {
            parts: Vec<PartGradedPart>,
            submission: String,
        },
        Provided {
            part: Literal<String>,
        },
    }

    #[derive(
        Clone,
        Debug,
        serde::Serialize,
        serde::Deserialize,
        tsify::Tsify,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
    )]
    #[tsify(into_wasm_abi, from_wasm_abi)]
    pub struct PartGradedPart {
        pub heard: Literal<String>,
        pub grade: WordGrade,
    }

    #[derive(
        Clone,
        Debug,
        serde::Serialize,
        serde::Deserialize,
        schemars::JsonSchema,
        tsify::Tsify,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
    )]
    #[tsify(namespace, into_wasm_abi, from_wasm_abi)]
    pub enum WordGrade {
        Perfect {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            wrote: Option<String>,
        },
        CorrectWithTypo {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            wrote: Option<String>,
        },
        PhoneticallyIdenticalButContextuallyIncorrect {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            wrote: Option<String>,
        },
        PhoneticallySimilarButContextuallyIncorrect {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            wrote: Option<String>,
        },
        Incorrect {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            wrote: Option<String>,
        },
        Missed {},
    }

    #[derive(
        Clone,
        Debug,
        serde::Serialize,
        serde::Deserialize,
        tsify::Tsify,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
    )]
    #[tsify(into_wasm_abi, from_wasm_abi)]
    pub struct Grade {
        pub explanation: Option<String>,
        pub results: Vec<PartGraded>,
        pub compare: Vec<String>,
        pub autograding_error: Option<String>,
    }
}

/// Consolidated data structure containing all generated language data
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ConsolidatedLanguageData {
    /// All target language sentences from Anki cards
    pub target_language_sentences: Vec<String>,
    /// Mapping from target language sentences to all native translations
    pub translations: Vec<(String, Vec<String>)>,
    /// NLP-analyzed sentences with multiword terms and heteronyms
    pub nlp_sentences: Vec<(String, SentenceInfo)>,
    /// Dictionary entries for individual words
    pub dictionary: Vec<(Heteronym<String>, DictionaryEntry)>,
    /// Phrasebook entries for multiword terms
    pub phrasebook: Vec<(String, PhrasebookEntry)>,
    /// Frequency data for words and phrases
    pub frequencies: Vec<FrequencyEntry<String>>,
    /// Mapping from words to their IPA pronunciations
    pub word_to_pronunciation: Vec<(String, Pronunciation)>,
    /// Mapping from IPA pronunciations to lists of words
    pub pronunciation_to_words: Vec<(Pronunciation, Vec<String>)>,
    /// Pronunciation patterns and guides for the course
    pub pronunciation_data: PronunciationData,
    /// Homophone disambiguation practice sentences
    pub homophone_practice: BTreeMap<HomophoneWordPair<String>, HomophonePractice<String>>,
}

impl ConsolidatedLanguageData {
    pub fn intern(&self, rodeo: &mut lasso::Rodeo) {
        // Intern empty string and space, just to make sure it's in there
        let _ = rodeo.get_or_intern("");
        let _ = rodeo.get_or_intern(" ");

        // Intern sentences
        for sentence in &self.target_language_sentences {
            rodeo.get_or_intern(sentence);
        }

        // Intern translations
        for (french, englishes) in &self.translations {
            rodeo.get_or_intern(french);
            for english in englishes {
                rodeo.get_or_intern(english);
            }
        }

        // Intern words from frequency list
        for freq in &self.frequencies {
            match &freq.lexeme {
                Lexeme::Heteronym(heteronym) => {
                    rodeo.get_or_intern(&heteronym.word);
                    rodeo.get_or_intern(&heteronym.lemma);
                }
                Lexeme::Multiword(multiword) => {
                    rodeo.get_or_intern(multiword);
                }
            }
        }
        for (heteronym, _entry) in &self.dictionary {
            rodeo.get_or_intern(&heteronym.word);
            rodeo.get_or_intern(&heteronym.lemma);
        }
        for (multiword, _entry) in &self.phrasebook {
            rodeo.get_or_intern(multiword);
        }

        // Intern words used in sentences (includes proper nouns, plus capitalization might differ)
        for (_, sentence_info) in &self.nlp_sentences {
            for word in &sentence_info.words {
                rodeo.get_or_intern(&word.text);
                rodeo.get_or_intern(&word.whitespace);
            }
        }

        // intern pronunciations
        for (_word, pronunciation) in &self.word_to_pronunciation {
            rodeo.get_or_intern(pronunciation);
        }

        // intern pronunciation data
        for (sound, _) in &self.pronunciation_data.sounds {
            rodeo.get_or_intern(sound);
        }
        for guide in &self.pronunciation_data.guides {
            rodeo.get_or_intern(&guide.pattern);
            rodeo.get_or_intern(&guide.description);
            for word_pair in &guide.example_words {
                rodeo.get_or_intern(&word_pair.target);
                rodeo.get_or_intern(&word_pair.native);
                rodeo.get_or_intern(&word_pair.cultural_context);
            }
        }

        // intern homophone practice data
        for (word_pair, practice) in &self.homophone_practice {
            word_pair.get_or_intern(rodeo);
            practice.get_or_intern(rodeo);
        }
    }
}

#[derive(
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Hash,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    tsify::Tsify,
)]
#[rkyv(compare(PartialEq), derive(Debug))]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum TtsProvider {
    ElevenLabs,
    Google,
}

pub type Pronunciation = String;

#[derive(
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Hash,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    tsify::Tsify,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[rkyv(compare(PartialEq))]
pub enum PronunciationDifficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Hash,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    tsify::Tsify,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[rkyv(compare(PartialEq))]
pub enum PronunciationFamiliarity {
    LikelyAlreadyKnows,
    MaybeAlreadyKnows,
    ProbablyDoesNotKnow,
}

#[derive(
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Hash,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    tsify::Tsify,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[rkyv(compare(PartialEq))]
pub struct LanguageSoundPattern {
    pub pattern: String, // e.g. "ch", "ent$", "^h"
}

#[derive(
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Hash,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    tsify::Tsify,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[rkyv(compare(PartialEq))]
pub enum SoundPosition {
    Beginning,
    Middle,
    End,
    Multiple,
}

#[derive(
    Copy,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Hash,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    tsify::Tsify,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[rkyv(compare(PartialEq), derive(PartialEq, PartialOrd, Eq, Ord, Hash))]
pub enum PatternPosition {
    Beginning,
    End,
    Anywhere,
}

#[derive(
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Hash,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    tsify::Tsify,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[rkyv(compare(PartialEq))]
pub struct WordPair {
    pub target: String,
    pub native: String,
    pub position: SoundPosition,  // Where the sound appears in the word
    pub cultural_context: String, // Cultural reference or familiarity note (in native language)
}

#[derive(
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Hash,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    tsify::Tsify,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[rkyv(compare(PartialEq))]
pub struct PronunciationGuideThoughts {
    pub thoughts: String,
    pub pattern: String,
    pub position: PatternPosition,
    pub description: String,
    pub familiarity: PronunciationFamiliarity,
    pub difficulty: PronunciationDifficulty,
    pub example_words: Vec<WordPair>,
}

#[derive(
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Hash,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    tsify::Tsify,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[rkyv(compare(PartialEq))]
pub struct PronunciationGuide {
    pub pattern: String,
    pub position: PatternPosition,
    pub description: String,
    pub familiarity: PronunciationFamiliarity,
    pub difficulty: PronunciationDifficulty,
    pub example_words: Vec<WordPair>,
}

impl From<PronunciationGuideThoughts> for PronunciationGuide {
    fn from(thoughts: PronunciationGuideThoughts) -> Self {
        Self {
            pattern: thoughts.pattern,
            position: thoughts.position,
            description: thoughts.description,
            familiarity: thoughts.familiarity,
            difficulty: thoughts.difficulty,
            example_words: thoughts.example_words,
        }
    }
}

#[derive(
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    tsify::Tsify,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[rkyv(compare(PartialEq))]
pub struct PronunciationData {
    pub sounds: Vec<(String, PatternPosition)>, // List of characteristic sounds/patterns for the language
    pub guides: Vec<PronunciationGuide>,        // Detailed guides for each sound
    pub pattern_frequencies: Vec<((String, PatternPosition), u32)>, // Pattern frequencies sorted by frequency (descending)
}

#[derive(
    Copy,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    Ord,
    PartialOrd,
    tsify::Tsify,
    schemars::JsonSchema,
)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum Language {
    French,
    English,
    Spanish,
    Korean,
    German,
    Chinese,
    Japanese,
    Russian,
    Portuguese,
    Italian,
}

#[derive(
    Copy,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    tsify::Tsify,
    schemars::JsonSchema,
)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum WritingSystem {
    /// Latin alphabet (Romance languages, Germanic languages, etc.)
    Latin,
    /// Korean Hangul script
    Hangul,
    /// Cyrillic alphabet (Russian, etc.)
    Cyrillic,
    /// Chinese Han characters (simplified and traditional)
    Han,
    /// Japanese writing system (combines Kanji, Hiragana, and Katakana)
    Japanese,
}

impl Language {
    pub fn iso_639_3(&self) -> &str {
        match self {
            Language::French => "fra",
            Language::English => "eng",
            Language::Spanish => "spa",
            Language::Korean => "kor",
            Language::German => "deu",
            Language::Chinese => "zho",
            Language::Japanese => "jpn",
            Language::Russian => "rus",
            Language::Portuguese => "por",
            Language::Italian => "ita",
        }
    }

    pub fn iso_639_1(&self) -> &'static str {
        match self {
            Language::French => "fr",
            Language::English => "en",
            Language::Spanish => "es",
            Language::Korean => "ko",
            Language::German => "de",
            Language::Chinese => "zh",
            Language::Japanese => "ja",
            Language::Russian => "ru",
            Language::Portuguese => "pt",
            Language::Italian => "it",
        }
    }

    pub fn writing_system(&self) -> WritingSystem {
        match self {
            Language::French
            | Language::English
            | Language::Spanish
            | Language::German
            | Language::Portuguese
            | Language::Italian => WritingSystem::Latin,
            Language::Korean => WritingSystem::Hangul,
            Language::Russian => WritingSystem::Cyrillic,
            Language::Chinese => WritingSystem::Han,
            Language::Japanese => WritingSystem::Japanese,
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::French => write!(f, "French"),
            Language::English => write!(f, "English"),
            Language::Spanish => write!(f, "Spanish"),
            Language::Korean => write!(f, "Korean"),
            Language::German => write!(f, "German"),
            Language::Chinese => write!(f, "Chinese"),
            Language::Japanese => write!(f, "Japanese"),
            Language::Russian => write!(f, "Russian"),
            Language::Portuguese => write!(f, "Portuguese"),
            Language::Italian => write!(f, "Italian"),
        }
    }
}

#[derive(
    Copy,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    tsify::Tsify,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Course {
    pub native_language: Language,
    pub target_language: Language,
}

impl Course {
    pub fn teaches_new_writing_system(&self) -> bool {
        self.native_language.writing_system() != self.target_language.writing_system()
    }
}

pub const COURSES: &[Course] = &[
    Course {
        native_language: Language::English,
        target_language: Language::French,
    },
    Course {
        native_language: Language::French,
        target_language: Language::English,
    },
    Course {
        native_language: Language::English,
        target_language: Language::Spanish,
    },
    Course {
        native_language: Language::English,
        target_language: Language::Korean,
    },
    Course {
        native_language: Language::English,
        target_language: Language::German,
    },
];

pub const LANGUAGES: &[Language] = &[
    Language::French,
    Language::Spanish,
    Language::English,
    Language::Korean,
    Language::German,
    Language::Chinese,
    Language::Japanese,
    Language::Russian,
    Language::Portuguese,
    Language::Italian,
];

/// A pair of homophone words, lexicographically sorted to ensure consistency
#[derive(
    Copy,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Hash,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    schemars::JsonSchema,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Hash), derive(PartialEq), derive(Eq))]
pub struct HomophoneWordPair<S>
where
    S: rkyv::Archive,
    <S as rkyv::Archive>::Archived: PartialEq + PartialOrd + Eq + Ord + Hash,
{
    pub word1: S,
    pub word2: S,
}

impl HomophoneWordPair<String> {
    /// Create a new word pair, ensuring lexicographic ordering.
    /// Returns None if the words are the same.
    pub fn new(word_a: String, word_b: String) -> Option<Self> {
        if word_a == word_b {
            return None;
        }

        let (word1, word2) = if word_a < word_b {
            (word_a, word_b)
        } else {
            (word_b, word_a)
        };

        Some(Self { word1, word2 })
    }

    pub fn get_interned(
        &self,
        rodeo: &lasso::RodeoReader,
    ) -> Option<HomophoneWordPair<lasso::Spur>> {
        Some(HomophoneWordPair {
            word1: rodeo.get(&self.word1)?,
            word2: rodeo.get(&self.word2)?,
        })
    }

    fn get_or_intern(&self, rodeo: &mut lasso::Rodeo) -> HomophoneWordPair<lasso::Spur> {
        HomophoneWordPair {
            word1: rodeo.get_or_intern(&self.word1),
            word2: rodeo.get_or_intern(&self.word2),
        }
    }
}

impl HomophoneWordPair<lasso::Spur> {
    pub fn resolve(&self, rodeo: &lasso::RodeoReader) -> HomophoneWordPair<String> {
        HomophoneWordPair {
            word1: rodeo.resolve(&self.word1).to_string(),
            word2: rodeo.resolve(&self.word2).to_string(),
        }
    }
}

/// A single sentence with a word that should be underlined (marked by asterisks in LLM output)
#[derive(
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(compare(PartialEq))]
pub struct HomophoneSentence<S>
where
    S: rkyv::Archive,
    <S as rkyv::Archive>::Archived: PartialEq + PartialOrd + Eq + Ord + Hash,
{
    /// The part of the sentence before the underlined word
    pub before: S,
    /// The underlined word itself
    pub word: S,
    /// The part of the sentence after the underlined word
    pub after: S,
}

impl HomophoneSentence<String> {
    pub fn get_interned(
        &self,
        rodeo: &lasso::RodeoReader,
    ) -> Option<HomophoneSentence<lasso::Spur>> {
        Some(HomophoneSentence {
            before: rodeo.get(&self.before)?,
            word: rodeo.get(&self.word)?,
            after: rodeo.get(&self.after)?,
        })
    }

    fn get_or_intern(&self, rodeo: &mut lasso::Rodeo) -> HomophoneSentence<lasso::Spur> {
        HomophoneSentence {
            before: rodeo.get_or_intern(&self.before),
            word: rodeo.get_or_intern(&self.word),
            after: rodeo.get_or_intern(&self.after),
        }
    }
}

impl HomophoneSentence<lasso::Spur> {
    pub fn resolve(&self, rodeo: &lasso::RodeoReader) -> HomophoneSentence<String> {
        HomophoneSentence {
            before: rodeo.resolve(&self.before).to_string(),
            word: rodeo.resolve(&self.word).to_string(),
            after: rodeo.resolve(&self.after).to_string(),
        }
    }
}
/// A pair of practice sentences for disambiguating two homophones
#[derive(
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(compare(PartialEq))]
pub struct HomophoneSentencePair<S>
where
    S: rkyv::Archive,
    <S as rkyv::Archive>::Archived: PartialEq + PartialOrd + Eq + Ord + Hash,
{
    /// Sentence using the first word (lexicographically)
    pub sentence1: HomophoneSentence<S>,
    /// Sentence using the second word (lexicographically)
    pub sentence2: HomophoneSentence<S>,
}

impl HomophoneSentencePair<String> {
    pub fn get_interned(
        &self,
        rodeo: &lasso::RodeoReader,
    ) -> Option<HomophoneSentencePair<lasso::Spur>> {
        Some(HomophoneSentencePair {
            sentence1: self.sentence1.get_interned(rodeo)?,
            sentence2: self.sentence2.get_interned(rodeo)?,
        })
    }

    fn get_or_intern(&self, rodeo: &mut lasso::Rodeo) -> HomophoneSentencePair<lasso::Spur> {
        HomophoneSentencePair {
            sentence1: self.sentence1.get_or_intern(rodeo),
            sentence2: self.sentence2.get_or_intern(rodeo),
        }
    }
}

impl HomophoneSentencePair<lasso::Spur> {
    pub fn resolve(&self, rodeo: &lasso::RodeoReader) -> HomophoneSentencePair<String> {
        HomophoneSentencePair {
            sentence1: self.sentence1.resolve(rodeo),
            sentence2: self.sentence2.resolve(rodeo),
        }
    }
}
/// Complete disambiguation practice data for a pair of homophones
#[derive(
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(compare(PartialEq))]
pub struct HomophonePractice<S>
where
    S: rkyv::Archive,
    <S as rkyv::Archive>::Archived: PartialEq + PartialOrd + Eq + Ord + Hash,
{
    pub sentence_pairs: Vec<HomophoneSentencePair<S>>,
}

impl HomophonePractice<String> {
    pub fn get_interned(
        &self,
        rodeo: &lasso::RodeoReader,
    ) -> Option<HomophonePractice<lasso::Spur>> {
        Some(HomophonePractice {
            sentence_pairs: self
                .sentence_pairs
                .iter()
                .map(|s| s.get_interned(rodeo).unwrap())
                .collect(),
        })
    }

    fn get_or_intern(&self, rodeo: &mut lasso::Rodeo) -> HomophonePractice<lasso::Spur> {
        HomophonePractice {
            sentence_pairs: self
                .sentence_pairs
                .iter()
                .map(|s| s.get_or_intern(rodeo))
                .collect(),
        }
    }
}

impl HomophonePractice<lasso::Spur> {
    pub fn resolve(&self, rodeo: &lasso::RodeoReader) -> HomophonePractice<String> {
        HomophonePractice {
            sentence_pairs: self
                .sentence_pairs
                .iter()
                .map(|s| s.resolve(rodeo))
                .collect(),
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, tsify::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct TtsRequest {
    pub text: String,
    pub language: Language,
}
