pub mod features;
pub mod indexmap;
pub mod language_pack;
pub mod minimal_pairs;
pub mod profile;
pub mod text_cleanup;

use rustc_hash::FxHashMap;
use std::collections::BTreeMap;
use std::hash::Hash;

use crate::features::Morphology;

#[bridgerton::bridge(transparent)]
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
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
#[schemars(rename = "PartOfSpeech")]
pub enum PartOfSpeechTag {
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

impl std::fmt::Display for PartOfSpeechTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let word = match self {
            PartOfSpeechTag::Adj => "adjective",
            PartOfSpeechTag::Adp => "adposition",
            PartOfSpeechTag::Adv => "adverb",
            PartOfSpeechTag::Aux => "auxiliary",
            PartOfSpeechTag::Cconj => "coordinating conjunction",
            PartOfSpeechTag::Det => "determiner",
            PartOfSpeechTag::Intj => "interjection",
            PartOfSpeechTag::Noun => "noun",
            PartOfSpeechTag::Num => "numeral",
            PartOfSpeechTag::Part => "particle",
            PartOfSpeechTag::Pron => "pronoun",
            PartOfSpeechTag::Propn => "proper noun",
            PartOfSpeechTag::Punct => "punctuation",
            PartOfSpeechTag::Sconj => "subordinating conjunction",
            PartOfSpeechTag::Sym => "symbol",
            PartOfSpeechTag::Verb => "verb",
            PartOfSpeechTag::Space => "space",
            PartOfSpeechTag::X => "other",
        };
        write!(f, "{word}")
    }
}

/// Part-of-speech for heteronyms (excludes proper nouns, punctuation, spaces, and unknown)
#[bridgerton::bridge(transparent)]
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
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
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
    #[serde(rename = "SCONJ")]
    Sconj, // subordinating conjunction
    #[serde(rename = "SYM")]
    Sym, // symbol
    #[serde(rename = "VERB")]
    Verb, // verb
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
            PartOfSpeech::Sconj => "subordinating conjunction",
            PartOfSpeech::Sym => "symbol",
            PartOfSpeech::Verb => "verb",
        };
        write!(f, "{word}")
    }
}

#[bridgerton::bridge(transparent)]
#[derive(
    Clone,
    Debug,
    serde::Deserialize,
    schemars::JsonSchema,
    serde::Serialize,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct TargetToNativeWord {
    pub native: String,
    pub note: Option<String>,
    pub example_sentence_target_language: String,
    pub example_sentence_native_language: String,
    pub cognate: bool,
    pub false_cognate: bool,
}

#[bridgerton::bridge(transparent)]
#[derive(
    Clone,
    Debug,
    serde::Deserialize,
    schemars::JsonSchema,
    serde::Serialize,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct PhrasebookDefinitionEntry {
    pub target_language_multi_word_term: String,
    pub meaning: String,
    pub additional_notes: String,
    pub target_language_example: String,
    pub native_language_example: String,
    pub informal: bool,
    pub compositional: bool,
    pub cognate: bool,
    pub false_cognate: bool,
    pub can_be_translated_literally: bool,
}

#[derive(Clone, Debug, serde::Deserialize, schemars::JsonSchema)]
#[schemars(rename = "PhrasebookDefinitionEntry")]
pub struct PhrasebookDefinitionEntryV2 {
    pub target_language_multi_word_term: String,
    pub meanings: Vec<String>,
    pub additional_notes: String,
    pub target_language_example: String,
    pub native_language_example: String,
    pub informal: bool,
    pub compositional: bool,
    pub cognate: bool,
    pub false_cognate: bool,
    pub can_be_translated_literally: bool,
}

#[bridgerton::bridge(transparent)]
#[derive(
    Clone,
    Debug,
    serde::Deserialize,
    schemars::JsonSchema,
    serde::Serialize,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug), derive(Hash))]
pub struct ProperNounDefinition {
    pub is_person_name: bool,
    pub is_place_name: bool,
    pub is_organization_name: bool,
    pub is_other: bool,
    pub learner_native_language_translation: String,
    pub description: Option<String>,
}

#[derive(
    Clone,
    Debug,
    serde::Deserialize,
    schemars::JsonSchema,
    serde::Serialize,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct DictionaryDefinition {
    pub target_language_word: String,
    pub definitions: Vec<TargetToNativeWord>,
}

/// A gram definition - either a dictionary entry (single word) or phrasebook entry (multi-word)
#[bridgerton::bridge(transparent)]
#[derive(
    Clone,
    Debug,
    serde::Deserialize,
    serde::Serialize,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum GramDefinition {
    Dictionary(DictionaryEntry),
    Phrasebook(PhrasebookDefinitionEntry),
}

#[bridgerton::bridge(transparent)]
#[derive(
    Clone,
    Debug,
    serde::Deserialize,
    schemars::JsonSchema,
    serde::Serialize,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct DictionaryEntry {
    pub target_language_word: String,
    pub definitions: Vec<TargetToNativeWord>,
    pub morphology: Vec<Morphology>,
    /// Etymological segmentation of the word. Each entry pairs the surface
    /// substring with its canonical / lemma form — e.g. `destabilize` →
    /// `[(de, de), (stabl, stable), (ize, ize)]`.
    #[serde(default)]
    pub segments: Vec<MorphemeSegment<String>>,
}

impl From<(DictionaryDefinition, Vec<Morphology>)> for DictionaryEntry {
    fn from(entry: (DictionaryDefinition, Vec<Morphology>)) -> Self {
        let (entry, morphology) = entry;
        Self {
            target_language_word: entry.target_language_word,
            definitions: entry.definitions,
            morphology,
            segments: Vec::new(),
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
pub struct SentenceSource {
    /// Sentence came from an Anki deck
    pub from_anki: bool,
    /// Sentence came from Tatoeba
    pub from_tatoeba: bool,
    /// Sentence was manually added to extra/manual.txt
    pub from_manual: bool,
    /// Sentence came from a song in sentence-sources/songs/
    pub from_song: bool,
    /// Book slugs if this sentence came from translated/segmented books in
    /// sentence-sources/books/ (e.g. ["pale-lights"]; serde default so data
    /// written before this field existed still loads)
    #[serde(default)]
    pub book_ids: Vec<String>,
    /// Movie IDs if this sentence appears in movies (e.g., ["tt0211915", "tt0241527"])
    pub movie_ids: Vec<String>,
}

impl SentenceSource {
    /// Create a new source with all fields set to false
    pub fn none() -> Self {
        Self {
            from_anki: false,
            from_tatoeba: false,
            from_manual: false,
            from_song: false,
            book_ids: Vec::new(),
            movie_ids: Vec::new(),
        }
    }

    /// Returns true if the sentence came from a manual source (should never be filtered)
    pub fn is_manual(&self) -> bool {
        self.from_manual
    }

    /// Returns true if book prose is this sentence's *only* source.
    pub fn is_book_only(&self) -> bool {
        !self.book_ids.is_empty()
            && !self.from_anki
            && !self.from_tatoeba
            && !self.from_manual
            && !self.from_song
            && self.movie_ids.is_empty()
    }

    /// Merge two sources together (OR operation on all fields)
    pub fn merge(&mut self, other: &Self) {
        self.from_anki |= other.from_anki;
        self.from_tatoeba |= other.from_tatoeba;
        self.from_manual |= other.from_manual;
        self.from_song |= other.from_song;
        // Merge book IDs, avoiding duplicates
        for book_id in &other.book_ids {
            if !self.book_ids.contains(book_id) {
                self.book_ids.push(book_id.clone());
            }
        }
        // Merge movie IDs, avoiding duplicates
        for movie_id in &other.movie_ids {
            if !self.movie_ids.contains(movie_id) {
                self.movie_ids.push(movie_id.clone());
            }
        }
    }
}

/// A Pimsleur lesson identifier (level + lesson number)
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
)]
pub struct PimsleurLesson {
    pub level: u32,
    #[serde(alias = "unit")]
    pub lesson: u32,
}

/// Identifies a source for per-source gram frequency data
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
)]
pub enum FrequencySourceId {
    Movie(String),
    PimsleurLesson(PimsleurLesson),
}

/// Coerce a TMDB `original_language` into a real ISO 639-1 code.
///
/// TMDB reports `cn` for Cantonese — not an ISO 639-1 code at all (639-3 uses
/// `yue`). For our purposes those films are Chinese: the Hong Kong canon
/// (無間道, 重慶森林, 功夫, 花樣年華) is part of the Chinese courses' corpus and
/// its subtitles are the Chinese text learners actually study. Left as `cn`
/// they never equal [`Language::iso_639_1`], so 15 of the Chinese course's 45
/// native films silently vanished from native-language movie lists.
///
/// Normalizing on the way in keeps every consumer's plain `==` correct. The
/// alternative — teaching the Rust recommender and the TypeScript movie
/// filter the same alias list — is how these two drifted apart to begin with.
/// `cn` is the only non-ISO-639-1 code across the whole corpus.
pub fn normalize_original_language(code: &str) -> &str {
    match code {
        "cn" => "zh",
        other => other,
    }
}

/// `deserialize_with` adapter applying [`normalize_original_language`]. Shared
/// so the TMDB client normalizes on the way in too, rather than keeping a
/// second copy of the alias list.
pub fn deserialize_original_language<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    Ok(Option::<String>::deserialize(deserializer)?
        .map(|code| normalize_original_language(&code).to_owned()))
}

/// Basic movie metadata without poster bytes, for serialization to files
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct MovieMetadataBasic {
    /// Unique identifier (IMDb ID, e.g., "tt0211915")
    pub id: String,
    /// Movie title
    pub title: String,
    /// Release year
    pub year: Option<u16>,
    /// Original language of the movie (ISO 639-1 code, e.g., "en", "fr").
    /// Normalized on read, so consumers can compare it to
    /// [`Language::iso_639_1`] directly — see [`normalize_original_language`].
    #[serde(default, deserialize_with = "deserialize_original_language")]
    pub original_language: Option<String>,
    /// Rotten Tomatoes score (0-100)
    #[serde(default)]
    pub rotten_tomatoes_score: Option<u8>,
}

/// Full movie metadata including poster bytes, for runtime use
#[derive(
    Clone,
    Debug,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Ord,
    PartialOrd,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct MovieMetadata {
    /// Unique identifier (IMDb ID, e.g., "tt0211915")
    pub id: String,
    /// Movie title
    pub title: String,
    /// Release year
    pub year: Option<u16>,
    /// Original language of the movie (ISO 639-1 code, e.g., "en", "fr")
    pub original_language: Option<String>,
    /// Rotten Tomatoes score (0-100)
    pub rotten_tomatoes_score: Option<u8>,
    /// Poster image bytes (JPEG format)
    pub poster_bytes: Option<Vec<u8>>,
}

impl From<MovieMetadataBasic> for MovieMetadata {
    fn from(basic: MovieMetadataBasic) -> Self {
        MovieMetadata {
            id: basic.id,
            title: basic.title,
            year: basic.year,
            original_language: basic.original_language,
            rotten_tomatoes_score: basic.rotten_tomatoes_score,
            poster_bytes: None,
        }
    }
}

/// Metadata for a book used as a sentence source, keyed by the slug that
/// appears in [`SentenceSource::book_ids`]. Stored per language in
/// `sentence-sources/books/<series>/metadata.jsonl` (one line per book in the
/// series) and carried into the language pack for attribution display.
#[bridgerton::bridge(transparent)]
#[derive(
    Clone,
    Debug,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Ord,
    PartialOrd,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct BookMetadata {
    /// Book slug, e.g. "pale-lights"
    pub id: String,
    /// Series slug — the folder the book lives in under sentence-sources/books/
    /// (e.g. "pale-lights"; a standalone book is a one-book series)
    pub series: String,
    /// Book title (kept in the original language even for translated courses)
    pub title: String,
    /// Author name
    pub author: String,
    /// Original language of the book (ISO 639-1 code, e.g. "en")
    #[serde(default)]
    pub original_language: Option<String>,
    /// True when this language's sentences are a machine translation of the
    /// original. Attribution should then credit the author for the story but
    /// not the wording — they may not stand by the translation.
    #[serde(default)]
    pub machine_translated: bool,
    /// Model that produced the translation, when `machine_translated`
    /// (e.g. "gpt-5.6-luna")
    #[serde(default)]
    pub translator: Option<String>,
    /// Where the book lives, e.g. the web serial's site
    #[serde(default)]
    pub source_url: Option<String>,
}

#[derive(
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct SubtitleLine<S>
where
    S: rkyv::Archive + Hash + std::fmt::Debug + Eq + PartialEq + Ord + PartialOrd,
    <S as rkyv::Archive>::Archived: std::fmt::Debug,
{
    /// The sentence text (Spur reference to sentence)
    pub sentence: S,
    /// Start timestamp in milliseconds
    pub start_ms: u32,
    /// End timestamp in milliseconds
    pub end_ms: u32,
}

/// An encoded sentence with grams (atoms with learnability info) and display metadata
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct SentenceGrams<G> {
    /// The grams that make up this sentence, each marked as learnable or obvious
    pub grams: Vec<SentenceGram<G>>,
    /// Whether the first letter should be capitalized when displaying
    pub capitalize_first: bool,
    /// High-confidence multiword terms found in this sentence, with the word
    /// indices each match bound
    pub multiword_terms: Vec<MultiwordTermMatch<G>>,
    /// Low-confidence multiword terms found in this sentence, with the word
    /// indices each match bound
    pub low_confidence_multiword_terms: Vec<MultiwordTermMatch<G>>,
}

impl SentenceGrams<SpurGram> {
    pub fn to_literals(
        self,
        string_rodeo: &lasso::RodeoReader<String>,
        gram_rodeo: &lasso::RodeoReader<Gram<lasso::Spur>>,
        language: Language,
    ) -> Vec<SentenceGram<(SpurGram, Vec<Literal<String>>)>> {
        // First pass: collect all atoms with their gram index, preserving
        // Control tokens so whitespace corrections are honored.
        let mut all_atoms: Vec<(usize, Atom<String>)> = Vec::new();
        for (gram_idx, gram) in self.grams.iter().enumerate() {
            let gram_spur = match gram {
                SentenceGram::Learnable(g) | SentenceGram::Obvious(g) => g,
            };
            let gram_resolved = gram_rodeo.resolve(gram_spur).resolve(string_rodeo);
            for atom in gram_resolved.iter() {
                all_atoms.push((gram_idx, atom.clone()));
            }
        }

        // Collect just the words for capitalization
        if self.capitalize_first
            && let Some((_, Atom::Tok(first_word))) = all_atoms
                .iter_mut()
                .find(|(_, a)| matches!(a, Atom::Tok(_)))
        {
            first_word.text = capitalize_first_letter(&first_word.text);
        }

        // Build literals using control tokens for whitespace correction.
        // We need to look ahead across gram boundaries for correct whitespace,
        // so we iterate over the flat atom list, then group by gram index.
        let mut literals_with_gram_idx: Vec<(usize, Literal<String>)> = Vec::new();
        let mut i = 0;
        while i < all_atoms.len() {
            let (gram_idx, atom) = &all_atoms[i];
            match atom {
                Atom::Tok(word) => {
                    // Determine whitespace: check if next atom is a Control token
                    let whitespace = if i + 1 < all_atoms.len() {
                        match &all_atoms[i + 1].1 {
                            Atom::Control(ctrl) => {
                                i += 1; // consume the control token
                                ctrl.0
                            }
                            Atom::Tok(next_word) => {
                                predict_whitespace(word, Some(next_word), language)
                            }
                        }
                    } else {
                        predict_whitespace(word, None, language)
                    };

                    literals_with_gram_idx.push((
                        *gram_idx,
                        Literal {
                            word: word.clone(),
                            whitespace: whitespace.to_str().to_string(),
                        },
                    ));
                }
                Atom::Control(_) => {
                    // Standalone control token (not preceded by Tok) — skip
                }
            }
            i += 1;
        }

        // Group literals back into their grams
        self.grams
            .into_iter()
            .enumerate()
            .map(|(gram_idx, gram)| {
                gram.map(|gram_spur| {
                    let literals: Vec<Literal<String>> = literals_with_gram_idx
                        .iter()
                        .filter(|(idx, _)| *idx == gram_idx)
                        .map(|(_, lit)| lit.clone())
                        .collect();
                    (gram_spur, literals)
                })
            })
            .collect()
    }
}

impl SentenceGrams<SpurGram> {
    pub fn resolve(
        &self,
        rodeo: &lasso::RodeoReader<Gram<lasso::Spur>>,
    ) -> SentenceGrams<Gram<lasso::Spur>> {
        SentenceGrams {
            grams: self.grams.iter().map(|gram| gram.resolve(rodeo)).collect(),
            capitalize_first: self.capitalize_first,
            multiword_terms: self
                .multiword_terms
                .iter()
                .map(|m| m.map(|g| rodeo.resolve(g).to_gram()))
                .collect(),
            low_confidence_multiword_terms: self
                .low_confidence_multiword_terms
                .iter()
                .map(|m| m.map(|g| rodeo.resolve(g).to_gram()))
                .collect(),
        }
    }
}

impl SentenceGrams<Gram<lasso::Spur>> {
    pub fn resolve(&self, rodeo: &lasso::RodeoReader) -> SentenceGrams<Gram<String>> {
        SentenceGrams {
            grams: self.grams.iter().map(|gram| gram.resolve(rodeo)).collect(),
            capitalize_first: self.capitalize_first,
            multiword_terms: self
                .multiword_terms
                .iter()
                .map(|m| m.map(|gram| gram.resolve(rodeo)))
                .collect(),
            low_confidence_multiword_terms: self
                .low_confidence_multiword_terms
                .iter()
                .map(|m| m.map(|gram| gram.resolve(rodeo)))
                .collect(),
        }
    }
}

impl<S> SentenceGram<S> {
    pub fn map<T, F>(self, f: F) -> SentenceGram<T>
    where
        F: Fn(S) -> T,
    {
        match self {
            SentenceGram::Learnable(s) => SentenceGram::Learnable(f(s)),
            SentenceGram::Obvious(s) => SentenceGram::Obvious(f(s)),
        }
    }
}

impl SentenceGram<SpurGram> {
    pub fn resolve(
        &self,
        rodeo: &lasso::RodeoReader<Gram<lasso::Spur>>,
    ) -> SentenceGram<Gram<lasso::Spur>> {
        match self {
            SentenceGram::Learnable(g) => SentenceGram::Learnable(rodeo.resolve(g).to_gram()),
            SentenceGram::Obvious(g) => SentenceGram::Obvious(rodeo.resolve(g).to_gram()),
        }
    }
}

impl SentenceGram<Gram<lasso::Spur>> {
    pub fn resolve(&self, rodeo: &lasso::RodeoReader) -> SentenceGram<Gram<String>> {
        match self {
            SentenceGram::Learnable(g) => SentenceGram::Learnable(g.resolve(rodeo)),
            SentenceGram::Obvious(g) => SentenceGram::Obvious(g.resolve(rodeo)),
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
)]
pub struct MultiwordTerms<T> {
    pub high_confidence: Vec<T>,
    pub low_confidence: Vec<T>,
}

/// A multiword term found in a sentence, together with which words of the
/// sentence the match bound. Indices are into the sentence's word sequence
/// (the NLP tokenization, which is 1:1 with the `Atom::Tok`s / rendered
/// literals of the sentence) — so graders and UI can point at the exact words
/// that realized the term, e.g. "arriver à quelqu'un" matching "leur … arrivé".
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
)]
pub struct MultiwordTermMatch<G> {
    pub gram: G,
    /// Word indices the match bound, sorted. Empty when the match positions
    /// are unknown (e.g. produced by a matcher that doesn't report them).
    pub matched_word_indices: Vec<u16>,
}

impl<G> MultiwordTermMatch<G> {
    pub fn map<H>(&self, f: impl FnOnce(&G) -> H) -> MultiwordTermMatch<H> {
        MultiwordTermMatch {
            gram: f(&self.gram),
            matched_word_indices: self.matched_word_indices.clone(),
        }
    }
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
pub struct NlpAnalyzedSentence {
    pub sentence: String,
    pub multiword_terms: MultiwordTerms<String>,
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
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct SentenceInfo {
    /// The sentence itself, in its canonical (encoded) representation. Only
    /// interpretable alongside the gram vocabulary that assigned the ids;
    /// decode with [`SentenceInfo::decode_words`].
    pub sentence: EncodedSentence,
    /// Multiword terms found in this sentence; `matched_word_indices` index
    /// into the decoded word sequence (one word per `Tok` atom, in order).
    pub multiword_terms: MultiwordTerms<MultiwordTermMatch<Gram<String>>>,
}

impl SentenceInfo {
    /// Decode the sentence's words (with whitespace and original
    /// capitalization) through the gram interners.
    pub fn decode_words(
        &self,
        interners: &GramInterners,
        language: Language,
    ) -> Vec<Literal<String>> {
        let atoms: Vec<Atom<String>> = self
            .sentence
            .tokens
            .iter()
            .flat_map(|key| interners.grams.resolve(key).iter())
            .map(|a| a.resolve(&interners.strings))
            .collect();
        let mut words = atoms_to_literals(&atoms, language);
        if self.sentence.capitalize_first
            && let Some(first_word) = words.first_mut()
        {
            first_word.word.text = capitalize_first_letter(&first_word.word.text);
        }
        words
    }

    /// The sentence's gram segmentation: each encoded gram with the range
    /// of decoded-word indices it covers (one word per `Tok` atom).
    pub fn gram_word_ranges(
        &self,
        interners: &GramInterners,
    ) -> Vec<(SpurGram, std::ops::Range<usize>)> {
        let mut ranges = Vec::with_capacity(self.sentence.tokens.len());
        let mut word_idx = 0usize;
        for &key in &self.sentence.tokens {
            let tok_count = interners
                .grams
                .resolve(&key)
                .iter()
                .filter(|a| matches!(a, Atom::Tok(_)))
                .count();
            ranges.push((key, word_idx..word_idx + tok_count));
            word_idx += tok_count;
        }
        ranges
    }
}

/// The interners behind a trained gram vocabulary: one rodeo for strings,
/// one for grams (whose atoms hold string Spurs). `SpurGram` keys are
/// assigned in vocabulary-id order, so `key.into_usize()` is the vocabulary
/// index.
pub struct GramInterners {
    pub strings: lasso::RodeoReader,
    pub grams: lasso::RodeoReader<Gram<lasso::Spur>>,
}

impl GramInterners {
    /// The gram's atoms in interned (Spur) form.
    pub fn atoms(&self, key: SpurGram) -> &[Atom<lasso::Spur>] {
        self.grams.resolve(&key).atoms()
    }

    /// Fully resolve a gram to its `String` form.
    pub fn resolve(&self, key: SpurGram) -> Gram<String> {
        Gram::from(
            self.atoms(key)
                .iter()
                .map(|a| a.resolve(&self.strings))
                .collect::<Vec<Atom<String>>>(),
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
pub struct DocToken {
    pub text: String,
    pub whitespace: String,
    pub pos: PartOfSpeechTag,
    pub lemma: String,
    pub morph: BTreeMap<String, String>,
}

#[bridgerton::bridge(transparent)]
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
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
pub struct Heteronym<S> {
    pub word: S,
    pub lemma: S,
    pub pos: PartOfSpeech,
}

/// One piece of a word's etymological decomposition.
///
/// - `surface`: the literal substring as it appears in the word.
/// - `canonical`: the dictionary / lemma form that substring refers to
///   (often differs — e.g. `stabl (stable)`).
/// - `tag`: optional UniMorph-style grammatical descriptor (e.g. `"pl"`,
///   `"1sg"`, `"prs.ind.1sg"`). Populated only when the `(surface, canonical)`
///   pair alone is ambiguous — e.g. French `-s` can be plural or 1sg/2sg verb
///   ending; English `-s` can be plural, 3sg present, or possessive. The tag
///   disambiguates so the same surface+canonical can map to distinct
///   `MorphemeInfo` entries.
#[bridgerton::bridge(transparent)]
#[derive(
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
    Hash,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct MorphemeSegment<S> {
    pub surface: S,
    pub canonical: S,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<S>,
}

/// Classification + learner-facing info for a single morpheme, as stored in
/// the language pack. Produced by the morpheme-analysis pass.
#[derive(
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[serde(tag = "type")]
pub enum MorphemeInfo<S> {
    /// A content morpheme that corresponds to a dictionary entry
    /// (e.g. English surface "stabl" whose dictionary form is "stable";
    /// Korean surface "먹" whose dictionary form is "먹다").
    Root { heteronym: Heteronym<S> },
    /// A bound content root — can't stand alone and isn't a standard affix
    /// (e.g. English "-cide", "-ology", Korean "-학" in 생물학).
    Bound { meaning: Option<S> },
    /// A derivational affix — changes meaning or part of speech
    /// (e.g. English "un-", "-ize"; Korean "-하다", "-화").
    Derivation { meaning: Option<S> },
    /// An inflectional affix — grammatical marker
    /// (e.g. English plural "-s", past "-ed"; Korean "-아", "-요").
    Inflection { meaning: Option<S> },
}

/// Type of non-heteronym word
#[bridgerton::bridge(transparent)]
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
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
pub enum OtherWordType {
    #[serde(rename = "PROPN")]
    Propn, // proper noun
    #[serde(rename = "PUNCT")]
    Punct, // punctuation
    #[serde(rename = "SPACE")]
    Space, // space
    #[serde(rename = "X")]
    X, // other/unknown
}

/// Information about a non-heteronym word
#[bridgerton::bridge(transparent)]
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
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
pub struct OtherWord {
    pub other_tag: OtherWordType,
}

/// The type of word in a literal - heteronym or other
#[bridgerton::bridge(transparent)]
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
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
#[serde(tag = "type")]
pub enum WordType<S> {
    /// A word with dictionary/grammatical information
    Heteronym(Heteronym<S>),
    /// Other (proper noun, punctuation, space, unknown)
    Other(OtherWord),
}

/// A word with its text and grammatical type
#[bridgerton::bridge(transparent)]
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
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
pub struct Word<S> {
    pub text: S,
    pub word_type: WordType<S>,
}

/// A literal token in a sentence (word + trailing whitespace)
#[bridgerton::bridge(transparent)]
#[derive(
    Copy,
    Clone,
    Debug,
    Hash,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Literal<S> {
    pub word: Word<S>,
    pub whitespace: S,
}

impl<S> Word<S> {
    /// Get the heteronym if this word represents a heteronym, otherwise None
    pub fn heteronym(&self) -> Option<&Heteronym<S>> {
        match &self.word_type {
            WordType::Heteronym(h) => Some(h),
            _ => None,
        }
    }
}

impl<S> Literal<S> {
    /// Get the heteronym if this literal represents a heteronym, otherwise None
    pub fn heteronym(&self) -> Option<&Heteronym<S>> {
        self.word.heteronym()
    }
}

impl Word<String> {
    pub fn get_or_intern(&self, rodeo: &mut lasso::Rodeo) -> Word<lasso::Spur> {
        Word {
            text: rodeo.get_or_intern(&self.text),
            word_type: match &self.word_type {
                WordType::Heteronym(h) => WordType::Heteronym(h.get_or_intern(rodeo)),
                WordType::Other(other) => WordType::Other(*other),
            },
        }
    }

    pub fn get_interned(&self, rodeo: &lasso::RodeoReader) -> Option<Word<lasso::Spur>> {
        let word_type = match &self.word_type {
            WordType::Heteronym(h) => WordType::Heteronym(h.get_interned(rodeo)?),
            WordType::Other(other) => WordType::Other(*other),
        };
        Some(Word {
            text: rodeo.get(&self.text)?,
            word_type,
        })
    }
}

impl Literal<String> {
    pub fn get_or_intern(&self, rodeo: &mut lasso::Rodeo) -> Literal<lasso::Spur> {
        Literal {
            word: self.word.get_or_intern(rodeo),
            whitespace: rodeo.get_or_intern(&self.whitespace),
        }
    }

    pub fn get_interned(&self, rodeo: &lasso::RodeoReader) -> Option<Literal<lasso::Spur>> {
        Some(Literal {
            word: self.word.get_interned(rodeo)?,
            whitespace: rodeo.get(&self.whitespace)?,
        })
    }
}

impl Word<lasso::Spur> {
    pub fn resolve(&self, rodeo: &lasso::RodeoReader) -> Word<String> {
        Word {
            text: rodeo.resolve(&self.text).to_string(),
            word_type: match &self.word_type {
                WordType::Heteronym(h) => WordType::Heteronym(h.resolve(rodeo)),
                WordType::Other(other) => WordType::Other(*other),
            },
        }
    }
}

impl Literal<lasso::Spur> {
    pub fn resolve(&self, rodeo: &lasso::RodeoReader) -> Literal<String> {
        Literal {
            word: self.word.resolve(rodeo),
            whitespace: rodeo.resolve(&self.whitespace).to_string(),
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
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
#[serde(tag = "type")]
pub enum Lexeme<S> {
    Heteronym { heteronym: Heteronym<S> },
    Multiword { phrase: S },
}

impl<S> Lexeme<S> {
    pub fn heteronym(&self) -> Option<&Heteronym<S>> {
        match self {
            Lexeme::Heteronym { heteronym } => Some(heteronym),
            _ => None,
        }
    }

    pub fn multiword(&self) -> Option<&S> {
        match self {
            Lexeme::Multiword { phrase } => Some(phrase),
            _ => None,
        }
    }
}

impl Lexeme<String> {
    pub fn get_or_intern(&self, rodeo: &mut lasso::Rodeo) -> Lexeme<lasso::Spur> {
        match self {
            Lexeme::Heteronym { heteronym } => Lexeme::Heteronym {
                heteronym: heteronym.get_or_intern(rodeo),
            },
            Lexeme::Multiword { phrase } => Lexeme::Multiword {
                phrase: rodeo.get_or_intern(phrase),
            },
        }
    }

    pub fn get_interned(&self, rodeo: &lasso::RodeoReader) -> Option<Lexeme<lasso::Spur>> {
        match self {
            Lexeme::Heteronym { heteronym } => Some(Lexeme::Heteronym {
                heteronym: heteronym.get_interned(rodeo)?,
            }),
            Lexeme::Multiword { phrase } => Some(Lexeme::Multiword {
                phrase: rodeo.get(phrase)?,
            }),
        }
    }

    pub fn get_disambiguation_key(&self) -> u32 {
        match self {
            Lexeme::Heteronym { heteronym: h } => {
                let combined = format!("{}\0{}\0{:?}", h.word, h.lemma, h.pos);
                xxhash_rust::xxh3::xxh3_64(combined.as_bytes()) as u32
            }
            Lexeme::Multiword { phrase } => xxhash_rust::xxh3::xxh3_64(phrase.as_bytes()) as u32,
        }
    }
}

impl Lexeme<lasso::Spur> {
    pub fn resolve(&self, rodeo: &lasso::RodeoReader) -> Lexeme<String> {
        match self {
            Lexeme::Heteronym { heteronym } => Lexeme::Heteronym {
                heteronym: heteronym.resolve(rodeo),
            },
            Lexeme::Multiword { phrase } => Lexeme::Multiword {
                phrase: rodeo.resolve(phrase).to_string(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct GramFrequencyEntry<S> {
    pub count: u32,

    /// Count of occurrences in actual sentences only (not including multi-word term occurrences).
    pub direct_count: u32,

    /// This key is different for each gram, which allows a consistent ordering with the same frequency.
    pub disambiguation_key: u32,

    pub gram: Gram<S>,
}

/// A frequency list with its total count (for percentage calculations).
/// Used in ConsolidatedLanguageData (pre-interning).
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GramFrequencyList {
    pub entries: Vec<GramFrequencyEntry<String>>,
    /// Total gram count from unfiltered data (for accurate percentage calculations)
    pub total_count: u64,
}

#[derive(
    Copy,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct Frequency {
    pub count: u32,
    /// Count of occurrences in actual sentences only (not including multi-word term occurrences).
    pub direct_count: u32,
    /// Whether this gram is considered "easy" (cognate with single-word definition).
    /// Easy grams are excluded from the isotonic regression and preferred during onboarding.
    pub easy: bool,
    /// Whether this is a compositional/literally-translatable multi-word gram.
    /// These are excluded from the isotonic regression (their difficulty depends on
    /// component words, violating the regression's independence assumption).
    pub compositional: bool,
    /// Estimated ease of this gram (higher = easier to already know).
    /// For single-atom grams: ln(count), with a cognate bonus.
    /// For multi-atom grams: depends on compositionality and component word ease.
    /// Used as the x-axis in the isotonic regression (instead of raw frequency_score).
    pub ease: f32,
}

impl Frequency {
    pub fn frequency_score(&self) -> f32 {
        (self.count as f32).ln()
    }

    pub fn direct_frequency_score(&self) -> f32 {
        (self.direct_count as f32).ln()
    }

    pub fn exclude_from_regression(&self) -> bool {
        self.easy || self.compositional
    }
}

pub mod autograde {
    use super::*;

    #[bridgerton::bridge(transparent)]
    #[derive(
        Clone,
        Debug,
        serde::Serialize,
        serde::Deserialize,
        schemars::JsonSchema,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
    )]
    pub enum Remembered {
        Remembered,
        Forgot,
    }

    /// Where a challenge sentence comes from, offered to the grader as
    /// disambiguation help: the film's title and the dialogue lines around
    /// the sentence (from the movie clip's subtitles, when one exists).
    /// Purely advisory — the grader must never grade the context itself.
    #[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
    pub struct GraderContext {
        pub movie_title: Option<String>,
        pub dialogue_before: Vec<String>,
        pub dialogue_after: Vec<String>,
    }

    impl GraderContext {
        pub fn is_empty(&self) -> bool {
            self.movie_title.is_none()
                && self.dialogue_before.is_empty()
                && self.dialogue_after.is_empty()
        }
    }

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    pub struct AutoGradeTranslationRequest {
        pub course: Course,
        pub challenge_sentence: String,
        pub user_sentence: String,
        pub literals: Vec<Literal<String>>,
        pub phrases: Vec<Gram<String>>,
        /// The gram that motivated this challenge — the LLM must always grade it.
        pub primary_expression: Gram<String>,
        #[serde(default)]
        pub context: GraderContext,
    }

    /// Response from autograde.
    #[bridgerton::bridge(transparent)]
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    pub struct AutoGradeTranslationResponse {
        pub encouragement: Option<String>,
        pub explanation: Option<String>,
        /// One entry per literal in order. None = ungradable (Other word type) or indeterminate.
        /// Covers single-word grams (heteronyms); multi-word grams use phrases_remembered/phrases_forgot.
        pub literal_grades: Vec<Option<Remembered>>,
        pub phrases_remembered: Vec<Gram<String>>,
        pub phrases_forgot: Vec<Gram<String>>,
        /// Set when heuristic grading was used instead of the LLM.
        pub autograding_error: Option<String>,
    }

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    pub struct AutoGradeTranscriptionRequest {
        pub course: Course,
        pub submission: Vec<transcription_challenge::PartSubmitted>,
        #[serde(default)]
        pub context: GraderContext,
    }

    /// Wrapper for passing gram grades across the WASM boundary.
    /// One entry per literal in order. None = ungradable (Other word type) or indeterminate.
    #[bridgerton::bridge(transparent)]
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    pub struct LiteralGrades(pub Vec<Option<Remembered>>);

    /// Wrapper for passing gram definitions across the WASM boundary.
    #[bridgerton::bridge(transparent)]
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    pub struct GramDefinitions(pub Vec<Option<GramDefinition>>);

    /// Wrapper for passing a challenge's `(imdb id, title)` pairs across the
    /// WASM boundary (bare tuples can't cross it as parameters).
    #[bridgerton::bridge(transparent)]
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    pub struct MovieTitles(pub Vec<(String, String)>);
}

pub mod transcription_challenge {
    use super::*;

    #[bridgerton::bridge(transparent, namespace)]
    #[derive(
        Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
    )]
    #[serde(tag = "type")]
    pub enum Part {
        AskedToTranscribe { parts: Vec<Literal<String>> },
        Provided { part: Literal<String> },
    }

    #[bridgerton::bridge(transparent)]
    #[derive(
        Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
    )]
    #[serde(tag = "type")]
    pub enum PartSubmitted {
        AskedToTranscribe {
            parts: Vec<Literal<String>>,
            submission: String,
        },
        Provided {
            part: Literal<String>,
        },
    }

    #[bridgerton::bridge(transparent, namespace)]
    #[derive(
        Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
    )]
    #[serde(tag = "type")]
    pub enum PartGraded {
        AskedToTranscribe {
            parts: Vec<PartGradedPart>,
            submission: String,
        },
        Provided {
            part: Literal<String>,
        },
    }

    #[bridgerton::bridge(transparent)]
    #[derive(
        Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
    )]
    pub struct PartGradedPart {
        pub heard: Literal<String>,
        pub grade: WordGrade,
    }

    #[bridgerton::bridge(transparent, namespace)]
    #[derive(
        Clone,
        Debug,
        serde::Serialize,
        serde::Deserialize,
        schemars::JsonSchema,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Hash,
    )]
    #[serde(tag = "type")]
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

    #[bridgerton::bridge(transparent)]
    #[derive(
        Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
    )]
    pub struct Grade {
        pub encouragement: Option<String>,
        pub explanation: Option<String>,
        pub results: Vec<PartGraded>,
        pub compare: Vec<String>,
        pub autograding_error: Option<String>,
    }
}

/// Represents whitespace between tokens.
/// We use an explicit enum rather than storing the actual string to normalize
/// the representation and make it more compact.
#[bridgerton::bridge(transparent)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
pub enum Whitespace {
    /// Regular space (U+0020)
    Space,
    /// Narrow non-breaking space (U+202F) - used in French before high punctuation
    NarrowNbsp,
    /// Regular non-breaking space (U+00A0)
    Nbsp,
    /// No whitespace
    None,
}

impl Whitespace {
    /// Convert whitespace enum to actual string
    pub fn to_str(&self) -> &'static str {
        match self {
            Whitespace::Space => " ",
            Whitespace::NarrowNbsp => "\u{202F}",
            Whitespace::Nbsp => "\u{00A0}",
            Whitespace::None => "",
        }
    }
}

impl std::str::FromStr for Whitespace {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "" => Whitespace::None,
            " " => Whitespace::Space,
            "\u{202F}" => Whitespace::NarrowNbsp,
            "\u{00A0}" => Whitespace::Nbsp,
            // For multiple spaces or other whitespace, normalize to single space
            s if s.chars().all(|c| c.is_whitespace()) => {
                if s.contains('\u{202F}') {
                    Whitespace::NarrowNbsp
                } else if s.contains('\u{00A0}') {
                    Whitespace::Nbsp
                } else {
                    Whitespace::Space
                }
            }
            // Default to space for anything else
            _ => Whitespace::Space,
        })
    }
}

/// A control token that corrects wrong whitespace predictions.
/// When the predicted whitespace doesn't match the actual whitespace,
/// we emit a Control token to record the correct value.
#[bridgerton::bridge(transparent)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
pub struct ControlToken(pub Whitespace);

/// An atom is either a word token or a control token.
/// This is the basic unit after whitespace normalization.
#[bridgerton::bridge(transparent)]
#[derive(
    Copy,
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
pub enum Atom<S> {
    /// A regular word token
    Tok(Word<S>),
    /// A control token for whitespace correction
    Control(ControlToken),
}

impl Atom<String> {
    pub fn get_or_intern(&self, rodeo: &mut lasso::Rodeo) -> Atom<lasso::Spur> {
        match self {
            Atom::Tok(word) => Atom::Tok(word.get_or_intern(rodeo)),
            Atom::Control(ctrl) => Atom::Control(*ctrl),
        }
    }

    pub fn get_interned(&self, rodeo: &lasso::RodeoReader) -> Option<Atom<lasso::Spur>> {
        match self {
            Atom::Tok(word) => Some(Atom::Tok(word.get_interned(rodeo)?)),
            Atom::Control(ctrl) => Some(Atom::Control(*ctrl)),
        }
    }
}

impl Atom<lasso::Spur> {
    pub fn resolve(&self, rodeo: &lasso::RodeoReader) -> Atom<String> {
        match self {
            Atom::Tok(word) => Atom::Tok(word.resolve(rodeo)),
            Atom::Control(ctrl) => Atom::Control(*ctrl),
        }
    }
}

/// An encoded sentence with token IDs from the gram vocabulary
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Ord,
    PartialOrd,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(compare(PartialEq), derive(Debug, PartialEq, Eq))]
pub struct EncodedSentence {
    /// The sentence as a sequence of interned gram keys (assigned in
    /// vocabulary-id order, so `key.into_usize()` is the vocabulary index).
    pub tokens: Vec<SpurGram>,
    /// Whether the first letter should be capitalized when displaying
    pub capitalize_first: bool,
}

// Manual serde: keys serialize as their vocabulary indices (plain integers),
// keeping the artifact format independent of lasso's (bit-rotted) `serialize`
// feature and identical to the pre-SpurGram encoding.
impl serde::Serialize for EncodedSentence {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use lasso::Key;
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("EncodedSentence", 2)?;
        s.serialize_field(
            "tokens",
            &self
                .tokens
                .iter()
                .map(|k| k.into_usize() as u32)
                .collect::<Vec<u32>>(),
        )?;
        s.serialize_field("capitalize_first", &self.capitalize_first)?;
        s.end()
    }
}

impl<'de> serde::Deserialize<'de> for EncodedSentence {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use lasso::Key;
        #[derive(serde::Deserialize)]
        struct Raw {
            tokens: Vec<u32>,
            capitalize_first: bool,
        }
        let raw = Raw::deserialize(deserializer)?;
        Ok(EncodedSentence {
            tokens: raw
                .tokens
                .into_iter()
                .map(|id| {
                    SpurGram::try_from_usize(id as usize)
                        .ok_or_else(|| serde::de::Error::custom("gram key out of range"))
                })
                .collect::<Result<_, _>>()?,
            capitalize_first: raw.capitalize_first,
        })
    }
}

/// A gram is a sequence of atoms representing a learnable unit in a sentence.
/// This is a newtype wrapper around `Vec<Atom<S>>` that provides methods for
/// working with grams and implements `Internable` for use with lasso.
#[bridgerton::bridge(transparent)]
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
pub struct Gram<S>(pub Vec<Atom<S>>);
pub type SpurGram = lasso::Spur<Gram<lasso::Spur>>;

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
pub struct grm<S> {
    atoms: [Atom<S>],
}
impl<S> AsRef<grm<S>> for grm<S> {
    fn as_ref(&self) -> &grm<S> {
        self
    }
}

impl<S> grm<S> {
    pub fn from_slice(slice: &[Atom<S>]) -> &Self {
        // SAFETY: grm is repr(transparent) over [Atom<S>]
        unsafe { &*(slice as *const [Atom<S>] as *const grm<S>) }
    }

    pub fn atoms(&self) -> &[Atom<S>] {
        &self.atoms
    }

    pub fn len(&self) -> usize {
        self.atoms.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Atom<S>> {
        self.atoms.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.atoms.is_empty()
    }
}

impl<S: Clone> grm<S> {
    pub fn to_gram(&self) -> Gram<S> {
        Gram(self.atoms.to_vec())
    }
}

impl<S> std::ops::Deref for Gram<S> {
    type Target = grm<S>;

    fn deref(&self) -> &grm<S> {
        grm::from_slice(&self.0)
    }
}
impl<S> AsRef<grm<S>> for Gram<S> {
    fn as_ref(&self) -> &grm<S> {
        self
    }
}

impl<S> Gram<S> {
    /// Creates a new gram from a vector of atoms.
    pub fn new(atoms: Vec<Atom<S>>) -> Self {
        Gram(atoms)
    }

    /// Returns true if this gram contains at least one learnable atom (a heteronym).
    pub fn is_learnable(&self) -> bool {
        self.0.iter().any(|atom| match atom {
            Atom::Tok(word) => matches!(word.word_type, WordType::Heteronym(_)),
            Atom::Control(_) => false,
        })
    }

    /// Returns the number of atoms in this gram.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if this gram has no atoms.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator over the atoms in this gram.
    pub fn iter(&self) -> impl Iterator<Item = &Atom<S>> {
        self.0.iter()
    }

    /// Returns a reference to the first atom, if any.
    pub fn first(&self) -> Option<&Atom<S>> {
        self.0.first()
    }

    /// If this is a single-atom heteronym gram, return a reference to the heteronym.
    pub fn heteronym(&self) -> Option<&Heteronym<S>> {
        if self.0.len() != 1 {
            return None;
        }
        match self.0.first()? {
            Atom::Tok(word) => word.heteronym(),
            Atom::Control(_) => None,
        }
    }
}

impl grm<lasso::Spur> {
    pub fn resolve(&self, rodeo: &lasso::RodeoReader) -> Gram<String> {
        Gram(self.atoms.iter().map(|a| a.resolve(rodeo)).collect())
    }
}

impl Gram<String> {
    /// Returns a disambiguation key for this gram, used to maintain consistent ordering
    /// of grams with the same frequency.
    pub fn disambiguation_key(&self) -> u32 {
        use std::fmt::Write;
        let mut combined = String::new();
        for atom in &self.0 {
            match atom {
                Atom::Tok(word) => {
                    let _ = write!(combined, "{}\0", word.text);
                }
                Atom::Control(ctrl) => {
                    let _ = write!(combined, "{ctrl:?}\0");
                }
            }
        }
        xxhash_rust::xxh3::xxh3_64(combined.as_bytes()) as u32
    }

    /// Converts this gram to a display string using whitespace prediction.
    /// This properly reconstructs the text with correct spacing for the given language.
    pub fn to_display_string(&self, language: Language) -> String {
        let literals = atoms_to_literals(&self.0, language);
        literals_to_text(&literals)
    }

    /// Interns all strings in this gram using the given rodeo.
    pub fn get_or_intern(&self, rodeo: &mut lasso::Rodeo) -> Gram<lasso::Spur> {
        Gram(self.0.iter().map(|a| a.get_or_intern(rodeo)).collect())
    }

    /// Looks up all strings in this gram in the given rodeo reader.
    /// Returns None if any string is not found.
    pub fn get_interned(&self, rodeo: &lasso::RodeoReader) -> Option<Gram<lasso::Spur>> {
        Some(Gram(
            self.0
                .iter()
                .map(|a| a.get_interned(rodeo))
                .collect::<Option<Vec<_>>>()?,
        ))
    }
}

impl Gram<lasso::Spur> {
    pub fn get_interned(&self, rodeo: &lasso::RodeoReader<Gram<lasso::Spur>>) -> Option<SpurGram> {
        rodeo.get(self)
    }
}

impl Gram<lasso::Spur> {
    /// Resolves all interned strings in this gram using the given rodeo reader.
    pub fn resolve(&self, rodeo: &lasso::RodeoReader) -> Gram<String> {
        Gram(self.0.iter().map(|a| a.resolve(rodeo)).collect())
    }
}

impl<S> AsRef<[Atom<S>]> for Gram<S> {
    fn as_ref(&self) -> &[Atom<S>] {
        &self.0
    }
}

impl<S> From<Vec<Atom<S>>> for Gram<S> {
    fn from(atoms: Vec<Atom<S>>) -> Self {
        Gram(atoms)
    }
}

impl<S> FromIterator<Atom<S>> for Gram<S> {
    fn from_iter<I: IntoIterator<Item = Atom<S>>>(iter: I) -> Self {
        Gram(iter.into_iter().collect())
    }
}

impl<S> IntoIterator for Gram<S> {
    type Item = Atom<S>;
    type IntoIter = std::vec::IntoIter<Atom<S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, S> IntoIterator for &'a Gram<S> {
    type Item = &'a Atom<S>;
    type IntoIter = std::slice::Iter<'a, Atom<S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

// Implement Internable for Gram so it can be used with lasso::Rodeo<Gram<S>>
impl<S: Copy + Eq + std::hash::Hash + 'static> lasso::Internable for Gram<S> {
    type Ref = grm<S>;

    fn from_ref(r: &Self::Ref) -> Self {
        Gram(r.atoms().to_vec())
    }
}

impl<S: Copy + Eq + std::hash::Hash + 'static> lasso::InternableRef for grm<S> {
    const ALIGNMENT: usize = std::mem::align_of::<Atom<S>>();
    fn is_empty(&self) -> bool {
        self.atoms.is_empty()
    }
    fn as_bytes(&self) -> &[u8] {
        self.atoms.as_bytes()
    }
    fn len(&self) -> usize {
        self.atoms.len()
    }
    fn empty() -> &'static Self {
        grm::<S>::from_slice(<[Atom<S>] as lasso::InternableRef>::empty())
    }

    unsafe fn from_raw_parts<'a>(ptr: *const u8, count: usize) -> &'a Self {
        let slice: &[Atom<S>] =
            unsafe { <[Atom<S>] as lasso::InternableRef>::from_raw_parts(ptr, count) };
        unsafe { &*(slice as *const [Atom<S>] as *const grm<S>) }
    }
}

// By doing it like this instead of using a Boolean, we allow the Rust compiler to use the learnability in the niche left by the Gram type argument.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum SentenceGram<G> {
    Learnable(G),
    Obvious(G),
}

impl<S> From<Gram<S>> for SentenceGram<Gram<S>> {
    fn from(gram: Gram<S>) -> Self {
        if gram.is_learnable() {
            SentenceGram::Learnable(gram)
        } else {
            SentenceGram::Obvious(gram)
        }
    }
}

impl<G> SentenceGram<G> {
    /// Get a reference to the inner gram
    pub fn learnable(&self) -> Option<&G> {
        match self {
            SentenceGram::Learnable(g) => Some(g),
            SentenceGram::Obvious(_) => None,
        }
    }

    /// Transform the inner gram type with a fallible function
    pub fn try_map<H, F: FnOnce(G) -> Option<H>>(self, f: F) -> Option<SentenceGram<H>> {
        match self {
            SentenceGram::Learnable(g) => Some(SentenceGram::Learnable(f(g)?)),
            SentenceGram::Obvious(g) => Some(SentenceGram::Obvious(f(g)?)),
        }
    }
}

impl SentenceGram<Gram<String>> {
    pub fn get_or_intern(&self, rodeo: &mut lasso::Rodeo) -> SentenceGram<Gram<lasso::Spur>> {
        match self {
            SentenceGram::Learnable(gram) => SentenceGram::Learnable(gram.get_or_intern(rodeo)),
            SentenceGram::Obvious(gram) => SentenceGram::Obvious(gram.get_or_intern(rodeo)),
        }
    }

    pub fn get_interned(
        &self,
        rodeo: &lasso::RodeoReader,
    ) -> Option<SentenceGram<Gram<lasso::Spur>>> {
        match self {
            SentenceGram::Learnable(gram) => {
                Some(SentenceGram::Learnable(gram.get_interned(rodeo)?))
            }
            SentenceGram::Obvious(gram) => Some(SentenceGram::Obvious(gram.get_interned(rodeo)?)),
        }
    }
}

/// A vocabulary entry for a gram (supertoken)
#[derive(
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct GramVocabEntry<S> {
    /// The atoms that make up this gram
    pub atoms: Gram<S>,
    /// Frequency count in the corpus
    pub frequency: u32,
}

impl GramVocabEntry<String> {
    pub fn get_or_intern(&self, rodeo: &mut lasso::Rodeo) -> GramVocabEntry<lasso::Spur> {
        GramVocabEntry {
            atoms: self.atoms.get_or_intern(rodeo),
            frequency: self.frequency,
        }
    }

    pub fn get_interned(&self, rodeo: &lasso::RodeoReader) -> Option<GramVocabEntry<lasso::Spur>> {
        Some(GramVocabEntry {
            atoms: self.atoms.get_interned(rodeo)?,
            frequency: self.frequency,
        })
    }
}

impl GramVocabEntry<lasso::Spur> {
    pub fn resolve(&self, rodeo: &lasso::RodeoReader) -> GramVocabEntry<String> {
        GramVocabEntry {
            atoms: self.atoms.resolve(rodeo),
            frequency: self.frequency,
        }
    }
}

/// Whether a voice actor was paid for their recordings or contributed them as a volunteer.
#[bridgerton::bridge(transparent)]
#[derive(
    Debug,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[rkyv(derive(Hash, PartialEq, Eq))]
#[serde(rename_all = "snake_case")]
pub enum Compensation {
    Paid,
    Volunteer,
}

/// Identity of a person who contributed human-recorded audio clips.
///
/// Used as a map key so each actor's metadata lives once per actor rather
/// than being duplicated across every clip they recorded.
#[derive(Debug, Clone, Eq, PartialEq, Hash, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(derive(Hash, PartialEq, Eq))]
pub struct VoiceActor {
    pub name: String,
    pub compensation: Compensation,
}

/// A single human-recorded audio clip.
///
/// `bytes` is OGG-encapsulated Opus (the format already accepted by
/// `is_valid_audio_data` in yap-frontend-rs/src/audio.rs via the `OggS` magic).
#[derive(Debug, Clone, Eq, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct Audio {
    pub bytes: Vec<u8>,
}

/// One stretch of a pronunciation-challenge clip: what the learner sees for
/// it, and when the voice says it, so the app can highlight along. The
/// times come from the phoneme model's best alignment of the clip; CTC
/// emissions cluster at each phoneme's onset, so `end_ms` runs early — hold
/// a highlight until the next segment starts rather than until `end_ms`.
#[derive(Debug, Clone, Eq, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct TimedSegment {
    pub text: String,
    pub start_ms: u32,
    pub end_ms: u32,
}

/// Phonemizer-verified audio for one pronunciation-guide example, with the
/// timing of each segment of its transcript (see
/// [`pronunciation_challenge_segments`]). `segments` is empty when the
/// alignment couldn't be computed; the audio is still good.
#[derive(Debug, Clone, Eq, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct PronunciationClip {
    pub audio: Audio,
    pub segments: Vec<TimedSegment>,
}

/// Consolidated data structure containing all generated language data
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ConsolidatedLanguageData {
    /// All target language sentences from Anki cards
    pub target_language_sentences: Vec<String>,
    /// Mapping from target language sentences to all native translations
    pub translations: Vec<(String, Vec<String>)>,
    /// NLP-analyzed sentences with multiword terms and heteronyms
    pub nlp_sentences: Vec<(String, SentenceInfo)>,
    /// Unified phrasebook: maps gram to definition entry (both MWE phrases and learned multi-atom grams)
    pub phrasebook: BTreeMap<Gram<String>, PhrasebookDefinitionEntry>,
    /// Proper noun definitions for names, places, organizations
    pub proper_noun_definitions: BTreeMap<String, ProperNounDefinition>,
    /// Per-source gram frequencies (movies, Pimsleur lessons, etc.)
    pub source_gram_frequencies: FxHashMap<FrequencySourceId, GramFrequencyList>,
    /// Mapping from words to their IPA pronunciations
    /// (one canonical "main" plus optional documented alternates).
    pub word_to_pronunciation: Vec<(String, Pronunciations)>,
    /// Mapping from IPA pronunciations to lists of words
    pub pronunciation_to_words: Vec<(Pronunciation, Vec<String>)>,
    /// Minimal pairs grouped by their distinguishing phoneme pair. Built
    /// once in `generate-data`; the language pack translates this into its
    /// interned `MinimalPairs` and also derives the inverse
    /// `word → 1-off words` map from it.
    pub minimal_pairs: Vec<crate::minimal_pairs::MinimalPairGroup>,
    /// Pronunciation patterns and guides for the course
    pub pronunciation_data: PronunciationData,
    /// Homophone disambiguation practice sentences
    pub homophone_practice: BTreeMap<HomophoneWordPair<String>, HomophonePractice<String>>,
    /// Movie metadata indexed by movie ID
    pub movies: FxHashMap<String, MovieMetadata>,
    /// Book metadata indexed by book slug
    pub books: FxHashMap<String, BookMetadata>,
    /// Sentence source provenance tracking (including movie_ids/book_ids)
    pub sentence_sources: Vec<(String, SentenceSource)>,
    /// Gram vocabulary: maps gram ID to display info (index = gram ID)
    pub gram_vocabulary: Vec<GramVocabEntry<String>>,
    /// Gram frequencies for learnable grams
    pub gram_frequencies: GramFrequencyList,
    /// Encoded sentences: sentence text -> grams with learnability and capitalize_first
    pub encoded_sentences: Vec<(String, SentenceGrams<Gram<String>>)>,
    /// Gram dictionary: definitions for grams (keyed by Gram for correct surface-form matching)
    pub gram_dictionary: BTreeMap<Gram<String>, DictionaryEntry>,
    /// Morpheme classification + info, keyed by (surface, canonical) pair.
    /// The pair prevents ambiguity when the same surface maps to different
    /// underlying morphemes (e.g. `-er` as agent vs. comparative).
    pub morphemes: BTreeMap<MorphemeSegment<String>, MorphemeInfo<String>>,
    /// Human-recorded audio clips, indexed by voice actor and then by the
    /// target-language phrase they speak. The nested-map shape enforces that
    /// each (actor, phrase) pair has at most one clip.
    pub human_audio: FxHashMap<VoiceActor, FxHashMap<String, Audio>>,
    /// Phonemizer-verified TTS clips for pronunciation challenges, keyed by
    /// the cue's spoken text — the same string the frontend requests.
    pub pronunciation_audio: FxHashMap<String, PronunciationClip>,
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

        for gram in self.gram_dictionary.keys() {
            for atom in gram.iter() {
                if let Atom::Tok(word) = atom {
                    rodeo.get_or_intern(&word.text);
                    if let WordType::Heteronym(h) = &word.word_type {
                        rodeo.get_or_intern(&h.word);
                        rodeo.get_or_intern(&h.lemma);
                    }
                }
            }
        }
        for entry in self.phrasebook.values() {
            rodeo.get_or_intern(&entry.target_language_multi_word_term);
        }

        // Intern every string sentences can reference: sentence words are
        // encoded against the gram vocabulary, so interning the vocabulary's
        // atoms covers them (display-time capitalization is applied after
        // resolving, so capitalized variants need no Spurs).
        for entry in &self.gram_vocabulary {
            for atom in entry.atoms.iter() {
                if let Atom::Tok(word) = atom {
                    rodeo.get_or_intern(&word.text);
                    if let WordType::Heteronym(h) = &word.word_type {
                        rodeo.get_or_intern(&h.word);
                        rodeo.get_or_intern(&h.lemma);
                    }
                }
            }
        }

        // intern pronunciations (full IPA strings) and their individual phonemes
        // (space-separated tokens). Per-phoneme Spurs let the minimal-pairs index
        // store distinguishing phonemes compactly without re-interning sub-tokens.
        // Only the main pronunciation is interned — alternates exist in the
        // intermediate JSONL for the audio verifier but aren't part of the
        // packed data model, so embedding them would just bloat the archive.
        for (_word, pronunciations) in &self.word_to_pronunciation {
            rodeo.get_or_intern(&pronunciations.main);
            for phoneme in pronunciations.main.split_whitespace() {
                rodeo.get_or_intern(phoneme);
            }
        }

        // intern minimal-pair contents (words + distinguishing phonemes). These
        // strings are almost always already interned via gram_dictionary and
        // word_to_pronunciation above, but interning explicitly keeps the
        // field self-contained.
        for group in &self.minimal_pairs {
            for phoneme in &group.phonemes {
                rodeo.get_or_intern(phoneme);
            }
            for pair in &group.pairs {
                rodeo.get_or_intern(&pair.word_a);
                rodeo.get_or_intern(&pair.word_b);
            }
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

        // intern movie data
        for movie in self.movies.values() {
            rodeo.get_or_intern(&movie.id);
            rodeo.get_or_intern(&movie.title);
        }

        // intern sentence sources (sentences already interned, just need movie_ids)
        for (sentence, source) in &self.sentence_sources {
            rodeo.get_or_intern(sentence);
            for movie_id in &source.movie_ids {
                rodeo.get_or_intern(movie_id);
            }
        }

        // intern proper noun definitions keys
        for proper_noun in self.proper_noun_definitions.keys() {
            rodeo.get_or_intern(proper_noun);
        }

        // intern morpheme segment keys (surface + canonical + optional tag) and info payloads
        for (segment, info) in &self.morphemes {
            rodeo.get_or_intern(&segment.surface);
            rodeo.get_or_intern(&segment.canonical);
            if let Some(tag) = &segment.tag {
                rodeo.get_or_intern(tag);
            }
            match info {
                MorphemeInfo::Root { heteronym } => {
                    rodeo.get_or_intern(&heteronym.word);
                    rodeo.get_or_intern(&heteronym.lemma);
                }
                MorphemeInfo::Bound { meaning }
                | MorphemeInfo::Derivation { meaning }
                | MorphemeInfo::Inflection { meaning } => {
                    if let Some(m) = meaning {
                        rodeo.get_or_intern(m);
                    }
                }
            }
        }

        // intern gram vocabulary atom texts
        for entry in &self.gram_vocabulary {
            for atom in &entry.atoms {
                if let Atom::<String>::Tok(word) = atom {
                    rodeo.get_or_intern(&word.text);
                    // Also intern word and lemma if it's a heteronym
                    if let WordType::Heteronym(h) = &word.word_type {
                        rodeo.get_or_intern(&h.word);
                        rodeo.get_or_intern(&h.lemma);
                    }
                }
            }
        }

        // intern encoded sentence keys, atoms, and multiword term grams
        for (sentence, encoded) in &self.encoded_sentences {
            rodeo.get_or_intern(sentence);
            for gram in &encoded.grams {
                match gram {
                    SentenceGram::Learnable(atoms) | SentenceGram::Obvious(atoms) => {
                        for atom in atoms {
                            atom.get_or_intern(rodeo);
                        }
                    }
                }
            }
            for term in &encoded.multiword_terms {
                for atom in term.gram.iter() {
                    atom.get_or_intern(rodeo);
                }
            }
            for term in &encoded.low_confidence_multiword_terms {
                for atom in term.gram.iter() {
                    atom.get_or_intern(rodeo);
                }
            }
        }

        // intern source gram frequencies
        for freq_list in self.source_gram_frequencies.values() {
            for entry in &freq_list.entries {
                for atom in &entry.gram {
                    if let Atom::<String>::Tok(word) = atom {
                        rodeo.get_or_intern(&word.text);
                        if let WordType::Heteronym(h) = &word.word_type {
                            rodeo.get_or_intern(&h.word);
                            rodeo.get_or_intern(&h.lemma);
                        }
                    }
                }
            }
        }

        // intern master gram frequencies
        for entry in &self.gram_frequencies.entries {
            for atom in &entry.gram {
                if let Atom::<String>::Tok(word) = atom {
                    rodeo.get_or_intern(&word.text);
                    if let WordType::Heteronym(h) = &word.word_type {
                        rodeo.get_or_intern(&h.word);
                        rodeo.get_or_intern(&h.lemma);
                    }
                }
            }
        }
    }
}

#[bridgerton::bridge(transparent)]
#[derive(
    Clone,
    Copy,
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
pub enum TtsProvider {
    ElevenLabs,
    Google,
    OpenAI,
    Gemini,
}

pub type Pronunciation = String;

/// A word's pronunciations. `main` is the canonical pronunciation we show
/// to learners; `others` are additional documented variants from wikipron.
///
/// Each IPA string is space-separated phonemes (same convention as
/// `Pronunciation`).
#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Pronunciations {
    pub main: Pronunciation,
    #[serde(default)]
    pub others: Vec<Pronunciation>,
}

impl Pronunciations {
    /// Iterate the main pronunciation followed by each alternative.
    pub fn all(&self) -> impl Iterator<Item = &Pronunciation> {
        std::iter::once(&self.main).chain(self.others.iter())
    }
}

#[bridgerton::bridge(transparent)]
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
pub enum PronunciationDifficulty {
    Easy,
    Medium,
    Hard,
}

#[bridgerton::bridge(transparent)]
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
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
pub struct LanguageSoundPattern {
    pub pattern: String, // e.g. "ch", "ent$", "^h"
}

#[bridgerton::bridge(transparent)]
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
pub enum SoundPosition {
    Beginning,
    Middle,
    End,
    Multiple,
}

#[bridgerton::bridge(transparent)]
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
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
pub enum PatternPosition {
    Beginning,
    End,
    Anywhere,
}

#[bridgerton::bridge(transparent)]
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
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
pub struct PronunciationGuideThoughts {
    pub thoughts: String,
    pub pattern: String,
    pub position: PatternPosition,
    pub description: String,
    pub familiarity: PronunciationFamiliarity,
    pub difficulty: PronunciationDifficulty,
    pub example_words: Vec<WordPair>,
}

#[bridgerton::bridge(transparent)]
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
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    schemars::JsonSchema,
)]
pub struct PronunciationData {
    pub sounds: Vec<(String, PatternPosition)>, // List of characteristic sounds/patterns for the language
    pub guides: Vec<PronunciationGuide>,        // Detailed guides for each sound
    pub pattern_frequencies: Vec<((String, PatternPosition), u32)>, // Pattern frequencies sorted by frequency (descending)
}

#[bridgerton::bridge(transparent)]
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
    schemars::JsonSchema,
)]
pub enum Language {
    French,
    English,
    Spanish,
    Korean,
    German,
    /// Mandarin Chinese written in Simplified script (zh-CN).
    ///
    /// The Simplified/Traditional split is first-class: the two scripts have
    /// different corpora and dictionaries, and mixed-script data is exactly
    /// the bug the subtitle sanity checks exist to catch.
    #[serde(alias = "Chinese")]
    ChineseSimplified,
    /// Mandarin Chinese written in Traditional script (zh-TW).
    ChineseTraditional,
    Japanese,
    Russian,
    Portuguese,
    Italian,
    Hindi,
    Thai,
}

#[derive(
    Copy, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, schemars::JsonSchema,
)]
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
    /// Devanagari script (Hindi, etc.)
    Devanagari,
    /// Thai script (an abugida; no spaces between words)
    Thai,
}

impl WritingSystem {
    /// Whether this script separates words with spaces. Scriptio continua
    /// systems (Han, Japanese, Thai) attach adjacent words directly; the
    /// spaces that do appear in such text (e.g. subtitle pause marks, Thai
    /// phrase boundaries) are the exception, not the rule.
    pub fn separates_words_with_spaces(&self) -> bool {
        match self {
            WritingSystem::Latin
            | WritingSystem::Hangul
            | WritingSystem::Cyrillic
            | WritingSystem::Devanagari => true,
            WritingSystem::Han | WritingSystem::Japanese | WritingSystem::Thai => false,
        }
    }
}

/// Characters used ONLY in simplified or ONLY in traditional Chinese text.
/// Forms valid in both (里/后/云/台/只/干/面/…) are deliberately absent.
const SIMPLIFIED_ONLY: &str = "国会这说对时们来学见还没电车门问间东儿点开关认让话语读写听号妈谁么几个长张马鸟鱼龙风华为乐现买卖医难题双观欢击级红纪经给绝统继续绿网罗办变边币标产称迟处传单当党动断队发刚归龟汉护记举剧亲轻确热伤审圣书树术岁孙态万习县响择泽针诊争证织职执质钟种众专转庄状准务议译异样养药钥远运杂灾脏则贼赠纸骂";
const TRADITIONAL_ONLY: &str = "國會這說對時們來學見還沒電車門問間東兒點開關認讓話語讀寫聽號媽誰麼幾個長張馬鳥魚龍風華為樂現買賣醫難題雙觀歡擊級紅紀經給絕統繼續綠網羅辦變邊幣標產稱遲處傳單當黨動斷隊發剛歸龜漢護記舉劇親輕確熱傷審聖書樹術歲孫態萬習縣響擇澤針診爭證織職執質鐘種眾專轉莊狀準務議譯異樣養藥鑰遠運雜災臟則賊贈紙罵";

/// The one G2P source a language's phoneme labels may come from.
///
/// Mirrors lexide's `PHONEME_BACKENDS.md`; see
/// [`Language::phoneme_label_source`] for why mixing sources is a
/// correctness bug rather than a quality trade-off.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhonemeLabelSource {
    /// espeak-ng fork, with this voice code, via the g2p crate. Must match
    /// `g2p::label_source` for the language.
    Espeak(&'static str),
    /// The g2p crate's Hindi chain (lexide's `schwa-stress-hin`, ported).
    Hindi,
    /// The g2p crate's Mandarin chain (g2pM + pinyin-to-IPA, ported).
    Mandarin,
    /// The g2p crate's Japanese chain (OpenJTalk via jpreprocess).
    Japanese,
    /// The g2p crate's Thai chain (vachana-thai as an embedded pinned Python
    /// project; needs `uv` on the machine that scores).
    Thai,
    /// No validated label source. Scoring is not supported.
    Unvalidated,
}

impl PhonemeLabelSource {
    /// True when the g2p crate produces this language's model labels, i.e.
    /// `g2p::phonemize_lang` is the right way to get a scoring target.
    pub fn g2p_supported(&self) -> bool {
        !matches!(self, PhonemeLabelSource::Unvalidated)
    }
}

impl Language {
    /// True if `text` contains Han characters that belong exclusively to the
    /// *other* Chinese script (e.g. Traditional-only characters when `self` is
    /// `ChineseSimplified`). Always false for non-Chinese languages. Used to
    /// filter mixed-script sources like Wiktionary category listings, which
    /// interleave Simplified and Traditional page titles.
    pub fn contains_wrong_han_script(&self, text: &str) -> bool {
        match self {
            Language::ChineseSimplified => text.chars().any(|c| TRADITIONAL_ONLY.contains(c)),
            Language::ChineseTraditional => text.chars().any(|c| SIMPLIFIED_ONLY.contains(c)),
            _ => false,
        }
    }

    /// Where this language's phoneme labels legitimately come from.
    ///
    /// Not a preference — a correctness constraint, and the single source of
    /// truth for it on this side. The pronunciation model is trained on
    /// labels from exactly one G2P source per language (lexide's
    /// `PHONEME_BACKENDS.md`, mirrored here); scoring audio against targets
    /// from a *different* source compares two phoneme inventories that
    /// disagree about what a segment is, and nothing downstream can detect
    /// it. Shapes match, distances are computed, results are simply wrong.
    ///
    /// This was not hypothetical: Hindi was scored against espeak `hi` and
    /// measured as our worst language by a wide margin (AUC 0.75 against
    /// 0.97 for Spanish), because ~290 standalone `ʰ` tokens in the targets
    /// are not in the model's vocabulary at all — Hindi's labels come from
    /// `schwa-stress-hin`, which emits aspiration as one segment with its
    /// consonant. The model was never the problem.
    pub fn phoneme_label_source(&self) -> PhonemeLabelSource {
        match self {
            // espeak-ng's IPA is the training label source for these, using
            // the fork at the pinned branch. Voices must match lexide's
            // `LANG_TO_ESPEAK` exactly — including `pt-br`, not `pt`:
            // European Portuguese targets against Brazilian audio measured
            // 41% median phoneme distance where `pt-br` measured 31%.
            Language::French => PhonemeLabelSource::Espeak("fr-fr"),
            Language::English => PhonemeLabelSource::Espeak("en-us"),
            Language::Spanish => PhonemeLabelSource::Espeak("es"),
            Language::German => PhonemeLabelSource::Espeak("de"),
            Language::Italian => PhonemeLabelSource::Espeak("it"),
            Language::Portuguese => PhonemeLabelSource::Espeak("pt-br"),
            Language::Russian => PhonemeLabelSource::Espeak("ru"),
            // The g2p crate's port of lexide's schwa-stress-hin chain.
            Language::Hindi => PhonemeLabelSource::Hindi,
            // espeak has voices for all of these, and using them is a bug.
            // The provider strings are lexide's
            // `build_external_phoneme_sidecars.CONFIG` keys.
            // OpenJTalk via jpreprocess, inside the g2p crate.
            Language::Japanese => PhonemeLabelSource::Japanese,
            // The g2p crate's port of g2pM + pinyin-to-IPA.
            Language::ChineseSimplified => PhonemeLabelSource::Mandarin,
            // vachana-thai, run by the g2p crate as a pinned Python project.
            Language::Thai => PhonemeLabelSource::Thai,
            // Traditional-script Mandarin has no corpus of its own; the
            // model never saw it, so there is no label source to name.
            Language::ChineseTraditional => PhonemeLabelSource::Unvalidated,
            // The g2p crate labels Korean (g2pk2 + mecab-ko, 2026-09-03
            // audit: 95.6% of Wiktionary words; espeak `ko` 47%), but the
            // deployed model has never been trained on Korean — lexide has
            // no Korean audio yet. Unvalidated until a model trained on the
            // g2p-kor labels ships: refusing to score is honest, scoring
            // against a model that never heard the language is not.
            Language::Korean => PhonemeLabelSource::Unvalidated,
        }
    }

    /// The language code to hand `g2p::phonemize_lang`, but **only** for
    /// languages whose deployed-model labels the g2p crate produces. `None`
    /// for the unvalidated languages (Korean, Traditional Mandarin), so a
    /// caller that reaches for a target there gets nothing rather than a
    /// plausible-looking wrong answer.
    pub fn g2p_lang(&self) -> Option<&'static str> {
        self.phoneme_label_source()
            .g2p_supported()
            .then(|| self.code())
    }

    /// Stable short code used for CLI arguments, data directories, and file
    /// names. ISO 639-3 where that's unambiguous; the Chinese variants append
    /// an ISO 15924 script subtag because 639-3 alone can't distinguish them.
    pub fn code(&self) -> &'static str {
        match self {
            Language::French => "fra",
            Language::English => "eng",
            Language::Spanish => "spa",
            Language::Korean => "kor",
            Language::German => "deu",
            Language::ChineseSimplified => "zho-hans",
            Language::ChineseTraditional => "zho-hant",
            Language::Japanese => "jpn",
            Language::Russian => "rus",
            Language::Portuguese => "por",
            Language::Italian => "ita",
            Language::Hindi => "hin",
            Language::Thai => "tha",
        }
    }

    /// The language tag Tatoeba's sentence dump uses. Tatoeba tags Mandarin
    /// as `cmn` whichever script a sentence is written in, so both Chinese
    /// courses read the same rows and filter by script afterwards; every
    /// other tag coincides with [`Language::code`].
    pub fn tatoeba_code(&self) -> &'static str {
        match self {
            Language::ChineseSimplified | Language::ChineseTraditional => "cmn",
            other => other.code(),
        }
    }

    /// Inverse of [`Language::code`]. Bare "zho" is deliberately not accepted:
    /// it doesn't say which script, and the whole point of the split is to
    /// make that ambiguity a loud error instead of a silent default.
    pub fn from_code(code: &str) -> Option<Self> {
        Some(match code {
            "fra" => Language::French,
            "eng" => Language::English,
            "spa" => Language::Spanish,
            "kor" => Language::Korean,
            "deu" => Language::German,
            "zho-hans" => Language::ChineseSimplified,
            "zho-hant" => Language::ChineseTraditional,
            "jpn" => Language::Japanese,
            "rus" => Language::Russian,
            "por" => Language::Portuguese,
            "ita" => Language::Italian,
            "hin" => Language::Hindi,
            "tha" => Language::Thai,
            _ => return None,
        })
    }

    pub fn iso_639_1(&self) -> &'static str {
        match self {
            Language::French => "fr",
            Language::English => "en",
            Language::Spanish => "es",
            Language::Korean => "ko",
            Language::German => "de",
            Language::ChineseSimplified | Language::ChineseTraditional => "zh",
            Language::Japanese => "ja",
            Language::Russian => "ru",
            Language::Portuguese => "pt",
            Language::Italian => "it",
            Language::Hindi => "hi",
            Language::Thai => "th",
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
            Language::ChineseSimplified | Language::ChineseTraditional => WritingSystem::Han,
            Language::Japanese => WritingSystem::Japanese,
            Language::Hindi => WritingSystem::Devanagari,
            Language::Thai => WritingSystem::Thai,
        }
    }

    /// Whether this language's writing is phonographic: written units stand
    /// for sounds, so a "this spelling sounds like that" pronunciation guide
    /// has something to teach. Han characters stand for morphemes, and the
    /// pinyin letters a sound inventory falls back to are not what a learner
    /// reads, so Chinese gets no pronunciation guides. Japanese counts: its
    /// inventory is the kana, and kana are a syllabary.
    pub fn is_phonographic(&self) -> bool {
        !matches!(self.writing_system(), WritingSystem::Han)
    }

    pub fn tv_politeness(&self) -> bool {
        matches!(
            self,
            Language::French
                | Language::Spanish
                | Language::German
                | Language::Portuguese
                | Language::Italian
                | Language::Hindi
                // Mandarin's politeness contrast is exactly 你 vs. 您 — a classic
                // second-person-only T-V distinction.
                | Language::ChineseSimplified
                | Language::ChineseTraditional
        )
    }

    /// The connector phrase used in pronunciation challenges (e.g., "comme dans" for French).
    pub fn pronunciation_connector(&self) -> &'static str {
        match self {
            Language::French => "comme dans",
            Language::Spanish => "como en",
            Language::Korean => "\u{cc98}\u{b7fc}",
            Language::English => "as in",
            Language::German => "wie in",
            Language::ChineseSimplified | Language::ChineseTraditional => "\u{5982}",
            Language::Japanese => "\u{306e}\u{3088}\u{3046}\u{306b}",
            Language::Russian => "\u{043a}\u{0430}\u{043a} \u{0432}",
            Language::Portuguese => "como em",
            Language::Italian => "come in",
            Language::Hindi => "जैसे",
            Language::Thai => "เหมือนใน",
        }
    }

    /// How a pronunciation cue says `letter`: the name a speaker of this
    /// language gives it where that name is more than the letter itself
    /// ("u Umlaut", "c cédille", "мягкий знак", "기역"). `None` means the
    /// bare letter is its own name — a voice reads a lone "b" as /be/, and
    /// espeak phonemizes it the same way, so spelling that out buys nothing
    /// and can hurt ("emme" phonemizes as /ɑ̃m/).
    ///
    /// This table is the single source for a cue's wording: the same spoken
    /// text goes to the TTS voice and to the phonemizer that verifies the
    /// clip, so the two can only ever disagree on delivery, never on which
    /// letter name was meant. Left to its own devices every voice invents a
    /// different reading of a bare "ã" or "ß"; given the name, they agree.
    /// Two kinds of letter therefore need an entry: any letter whose name is
    /// a phrase (accents, ligatures, modifier letters, Korean jamo), and any
    /// letter espeak would otherwise read as the homographic word (French "y"
    /// is the adverb /i/, English "a" the article /ə/). Spellings are chosen
    /// for what espeak makes of them: English "eh" is /eɪ/ where "ay" is
    /// /aɪ/. Languages whose sound inventory is syllabic (kana, Devanagari,
    /// Thai) read every character as itself.
    pub fn letter_name(&self, letter: char) -> Option<&'static str> {
        let letter = letter.to_lowercase().next().unwrap_or(letter);
        let name = match self {
            Language::French => match letter {
                'y' => "i grec",
                'à' => "a accent grave",
                'â' => "a accent circonflexe",
                'ä' => "a tréma",
                'ç' => "c cédille",
                'é' => "e accent aigu",
                'è' => "e accent grave",
                'ê' => "e accent circonflexe",
                'ë' => "e tréma",
                'î' => "i accent circonflexe",
                'ï' => "i tréma",
                'ô' => "o accent circonflexe",
                'ö' => "o tréma",
                'ù' => "u accent grave",
                'û' => "u accent circonflexe",
                'ü' => "u tréma",
                'ÿ' => "i grec tréma",
                'œ' => "e dans l'o",
                'æ' => "e dans l'a",
                _ => return None,
            },
            Language::Portuguese => match letter {
                'á' => "a acento agudo",
                'é' => "e acento agudo",
                'í' => "i acento agudo",
                'ó' => "o acento agudo",
                'ú' => "u acento agudo",
                'à' => "a acento grave",
                'â' => "a acento circunflexo",
                'ê' => "e acento circunflexo",
                'ô' => "o acento circunflexo",
                'ã' => "a til",
                'õ' => "o til",
                'ç' => "c cedilha",
                'ü' => "u trema",
                _ => return None,
            },
            Language::Spanish => match letter {
                'y' => "i griega",
                'ñ' => "eñe",
                'á' => "a con acento",
                'é' => "e con acento",
                'í' => "i con acento",
                'ó' => "o con acento",
                'ú' => "u con acento",
                'ü' => "u con diéresis",
                _ => return None,
            },
            Language::Italian => match letter {
                'à' => "a con accento grave",
                'è' => "e con accento grave",
                'é' => "e con accento acuto",
                'ì' => "i con accento grave",
                'ò' => "o con accento grave",
                'ó' => "o con accento acuto",
                'ù' => "u con accento grave",
                _ => return None,
            },
            Language::German => match letter {
                'ä' => "a Umlaut",
                'ö' => "o Umlaut",
                'ü' => "u Umlaut",
                'ß' => "Eszett",
                _ => return None,
            },
            Language::English => match letter {
                'a' => "eh",
                'à' => "a grave",
                'á' => "a acute",
                'â' => "a circumflex",
                'ä' => "a diaeresis",
                'å' => "a ring above",
                'æ' => "ash",
                'ç' => "c cedilla",
                'è' => "e grave",
                'é' => "e acute",
                'ê' => "e circumflex",
                'ë' => "e diaeresis",
                'í' => "i acute",
                'ï' => "i diaeresis",
                'ñ' => "n tilde",
                'ó' => "o acute",
                'ô' => "o circumflex",
                'ö' => "o diaeresis",
                'ú' => "u acute",
                'ü' => "u diaeresis",
                'œ' => "ethel",
                'ß' => "sharp s",
                _ => return None,
            },
            // Consonants are named in full: a Russian voice reading a bare
            // "щ" or "ч" picks between the sound and the name at random.
            // Vowels are their own names. The combining acute (U+0301) marks
            // a stressed vowel in the sound inventory and is read as such.
            Language::Russian => match letter {
                'б' => "бэ",
                'в' => "вэ",
                'г' => "гэ",
                'д' => "дэ",
                'ж' => "жэ",
                'з' => "зэ",
                'й' => "и краткое",
                'к' => "ка",
                'л' => "эль",
                'м' => "эм",
                'н' => "эн",
                'п' => "пэ",
                'р' => "эр",
                'с' => "эс",
                'т' => "тэ",
                'ф' => "эф",
                'х' => "ха",
                'ц' => "цэ",
                'ч' => "че",
                'ш' => "ша",
                'щ' => "ща",
                'ъ' => "твёрдый знак",
                'ь' => "мягкий знак",
                '\u{301}' => "с ударением",
                _ => return None,
            },
            // Jamo are named, not sounded: a lone "ㄱ" is 기역.
            Language::Korean => match letter {
                'ㄱ' => "기역",
                'ㄲ' => "쌍기역",
                'ㄳ' => "기역시옷",
                'ㄴ' => "니은",
                'ㄵ' => "니은지읒",
                'ㄶ' => "니은히읗",
                'ㄷ' => "디귿",
                'ㄸ' => "쌍디귿",
                'ㄹ' => "리을",
                'ㄺ' => "리을기역",
                'ㄻ' => "리을미음",
                'ㄼ' => "리을비읍",
                'ㄽ' => "리을시옷",
                'ㄾ' => "리을티읕",
                'ㄿ' => "리을피읖",
                'ㅀ' => "리을히읗",
                'ㅁ' => "미음",
                'ㅂ' => "비읍",
                'ㅃ' => "쌍비읍",
                'ㅄ' => "비읍시옷",
                'ㅅ' => "시옷",
                'ㅆ' => "쌍시옷",
                'ㅇ' => "이응",
                'ㅈ' => "지읒",
                'ㅉ' => "쌍지읒",
                'ㅊ' => "치읓",
                'ㅋ' => "키읔",
                'ㅌ' => "티읕",
                'ㅍ' => "피읖",
                'ㅎ' => "히읗",
                'ㅏ' => "아",
                'ㅐ' => "애",
                'ㅑ' => "야",
                'ㅒ' => "얘",
                'ㅓ' => "어",
                'ㅔ' => "에",
                'ㅕ' => "여",
                'ㅖ' => "예",
                'ㅗ' => "오",
                'ㅘ' => "와",
                'ㅙ' => "왜",
                'ㅚ' => "외",
                'ㅛ' => "요",
                'ㅜ' => "우",
                'ㅝ' => "워",
                'ㅞ' => "웨",
                'ㅟ' => "위",
                'ㅠ' => "유",
                'ㅡ' => "으",
                'ㅢ' => "의",
                'ㅣ' => "이",
                _ => return None,
            },
            // Kana read as themselves, except the small kana: a lone っ has
            // no sound to voice and the small vowels and y-kana merely
            // modify their neighbour, so each is named the way a teacher
            // does, "small tsu".
            Language::Japanese => match letter {
                'っ' => "小さいつ",
                'ッ' => "小さいツ",
                'ぁ' => "小さいあ",
                'ぃ' => "小さいい",
                'ぅ' => "小さいう",
                'ぇ' => "小さいえ",
                'ぉ' => "小さいお",
                'ゃ' => "小さいや",
                'ゅ' => "小さいゆ",
                'ょ' => "小さいよ",
                'ァ' => "小さいア",
                'ィ' => "小さいイ",
                'ゥ' => "小さいウ",
                'ェ' => "小さいエ",
                'ォ' => "小さいオ",
                'ャ' => "小さいヤ",
                'ュ' => "小さいユ",
                'ョ' => "小さいヨ",
                _ => return None,
            },
            Language::ChineseSimplified
            | Language::ChineseTraditional
            | Language::Hindi
            | Language::Thai => return None,
        };
        Some(name)
    }

    /// Google Cloud TTS locale and voice for this language.
    pub fn google_tts_voice(&self) -> (&'static str, &'static str) {
        match self {
            Language::French => ("fr-FR", "fr-FR-Chirp3-HD-Achernar"),
            Language::Spanish => ("es-US", "es-US-Chirp3-HD-Achernar"),
            Language::English => ("en-US", "en-US-Chirp3-HD-Achernar"),
            Language::Korean => ("ko-KR", "ko-KR-Chirp3-HD-Achernar"),
            Language::German => ("de-DE", "de-DE-Chirp3-HD-Achernar"),
            Language::Italian => ("it-IT", "it-IT-Chirp3-HD-Achernar"),
            Language::Portuguese => ("pt-BR", "pt-BR-Chirp3-HD-Achernar"),
            Language::Russian => ("ru-RU", "ru-RU-Chirp3-HD-Aoede"),
            Language::Japanese => ("ja-JP", "ja-JP-Chirp3-HD-Achernar"),
            Language::Hindi => ("hi-IN", "hi-IN-Chirp3-HD-Achernar"),
            Language::ChineseSimplified => ("cmn-CN", "cmn-CN-Chirp3-HD-Achernar"),
            Language::Thai => ("th-TH", "th-TH-Chirp3-HD-Achernar"),
            Language::ChineseTraditional => ("cmn-TW", "cmn-TW-Wavenet-A"),
        }
    }

    /// OpenSubtitles API language code (usually ISO 639-1, but pt-br for Portuguese)
    /// The OpenSubtitles `languages=` query for this language: every code a
    /// subtitle in it may be filed under, comma-separated as the API takes
    /// them. A query names its codes and silently returns nothing for the
    /// rest: there is no bare "zh" or "pt", and Spanish is split three ways
    /// — "es", "sp" (Spanish (EU)) and "ea" (Spanish (LA)) — with a film's
    /// only human-made subtitle sometimes under a variant alone (Abre los
    /// ojos: one file, under "sp").
    pub fn opensubtitles_languages(&self) -> &'static str {
        match self {
            Language::Portuguese => "pt-br",
            Language::ChineseSimplified => "zh-cn",
            Language::ChineseTraditional => "zh-tw",
            Language::Spanish => "es,sp,ea",
            other => other.iso_639_1(),
        }
    }

    /// TMDB API language code (language-REGION format)
    pub fn tmdb_language_code(&self) -> &'static str {
        match self {
            Language::French => "fr-FR",
            Language::English => "en-US",
            Language::Spanish => "es-ES",
            Language::German => "de-DE",
            Language::Korean => "ko-KR",
            Language::ChineseSimplified => "zh-CN",
            Language::ChineseTraditional => "zh-TW",
            Language::Japanese => "ja-JP",
            Language::Russian => "ru-RU",
            Language::Portuguese => "pt-BR",
            Language::Italian => "it-IT",
            Language::Hindi => "hi-IN",
            Language::Thai => "th-TH",
        }
    }

    /// Words that should appear in virtually any movie's subtitles for this language.
    /// Used as a sanity check to detect wrong-language subtitle files.
    /// Returns a list of words where ALL must appear at least once (as whole words) in the subtitle text.
    pub fn subtitle_sanity_words(&self) -> &'static [&'static str] {
        match self {
            Language::French => &["le", "de", "pas", "je"],
            Language::English => &["the", "to", "you", "is"],
            Language::Spanish => &["el", "de", "no", "que"],
            Language::German => &["ich", "das", "nicht", "du"],
            Language::Korean => &["이", "는", "을", "에"],
            // 的/了/是/不 are written identically in both scripts.
            Language::ChineseSimplified | Language::ChineseTraditional => &["的", "了", "是", "不"],
            Language::Japanese => &["の", "は", "を", "に"],
            Language::Russian => &["не", "что", "на", "это"],
            Language::Portuguese => &["que", "de", "não", "eu"],
            Language::Italian => &["che", "di", "non", "il"],
            Language::Hindi => &["है", "में", "के", "को"],
            // Thai has no spaces between words, so these are checked as
            // substrings (the non-Latin path in check_subtitle_sanity).
            Language::Thai => &["ไม่", "ที่", "ได้", "จะ"],
        }
    }

    /// Substrings that should NEVER appear in properly-encoded subtitles for this language.
    /// Used to detect systematic character corruption (e.g. t→r substitution from bad OCR/encoding).
    /// If ANY of these substrings appear frequently, the subtitle file is corrupt.
    /// Returns (forbidden_substring, what_it_should_be) pairs.
    pub fn subtitle_corruption_markers(&self) -> &'static [(&'static str, &'static str)] {
        match self {
            // t→r corruption: "est"→"esr", "être"→"êrre", "tout"→"rour", "cette"→"cerre"
            // OCR corruption: "Il"→"II" (two capital I's), "Il"→"ll", "l'"→"I'" (capital I for l)
            Language::French => &[
                ("c'esr", "c'est"),
                ("qu'esr", "qu'est"),
                ("êrre", "être"),
                ("rour", "tout"),
                ("cerre", "cette"),
                ("conrre", "contre"),
                ("enrre", "entre"),
                ("aurre", "autre"),
                ("perir", "petit"),
                ("norre", "notre"),
                ("vorre", "votre"),
                // OCR: two capital I's instead of "Il"
                ("II ", "Il "),
                // OCR: two lowercase l's instead of "Il"
                (" ll ", " Il "),
                // OCR: capital I instead of lowercase l before apostrophe
                // Detected separately via ocr_i_apostrophe check in check_subtitle_sanity
            ],
            // t→r corruption: "the"→"rhe", "that"→"rhar", "this"→"rhis", "it"→"ir"
            // OCR: lowercase l for uppercase I: "l'm" for "I'm", "lt" for "It"
            Language::English => &[
                (" rhe ", " the "),
                ("rhar", "that"),
                ("rhis", "this"),
                ("rhere", "there"),
                ("rhey", "they"),
                ("whar", "what"),
                ("abour", "about"),
                ("jusr", "just"),
                ("righr", "right"),
                ("don'r", "don't"),
                // OCR: lowercase l for uppercase I
                (" l'm", " I'm"),
                (" l'd", " I'd"),
                (" l'll", " I'll"),
                (" lt ", " It "),
                (" ln ", " In "),
                (" lf ", " If "),
            ],
            // t→r corruption: "está"→"esrá", "todo"→"rodo", "tiene"→"riene"
            Language::Spanish => &[
                ("esrá", "está"),
                ("esro", "esto"),
                ("riene", "tiene"),
                ("riempo", "tiempo"),
                ("rambién", "también"),
                ("conrra", "contra"),
                ("nuesrro", "nuestro"),
                // NOTE: "orro" and "parr" omitted — too many false positives from
                // legitimate Spanish words (socorro, horror, zorro, parrilla, parroquia, etc.)
            ],
            // t→r corruption: "nicht"→"nichr", "ist"→"isr", "mit"→"mir" (ambiguous)
            // OCR: lowercase l for uppercase I: "lch" for "Ich", "lhr" for "Ihr"
            Language::German => &[
                ("nichr", "nicht"),
                ("nichrs", "nichts"),
                ("jerzt", "jetzt"),
                ("harre", "hatte"),
                ("birer", "bitte"),
                ("lerzr", "letzt"),
                ("mussr", "musst"),
                ("kannsr", "kannst"),
                // OCR: lowercase l for uppercase I
                (" lch ", " Ich "),
                (" lhr", " Ihr"),
                (" lhm", " Ihm"),
                (" lhn", " Ihn"),
                (" lst ", " Ist "),
                (" ln ", " In "),
            ],
            // t→r corruption: "tutto"→"rurro", "questo"→"quesr-", "fatto"→"farro"
            // OCR: "Il"→"II" or "ll", "In"→"ln", zz→e'e'
            Language::Italian => &[
                ("rurro", "tutto"),
                ("rurra", "tutta"),
                ("quesr", "quest"),
                ("farro", "fatto"),
                ("derro", "detto"),
                ("sraro", "stato"),
                ("conrro", "contro"),
                ("nienre", "niente"),
                // OCR: two capital I's or two lowercase l's instead of "Il"
                ("II ", "Il "),
                (" ll ", " Il "),
                // OCR: "In" → "ln"
                (" ln ", " In "),
                // OCR: zz→e'e' (bizarre but systematic): "soluzione"→"solue'ione"
                ("e'e'", "zz"),
                // OCR: í (accented i) replacing normal i in common words
                ("Grazíe", "Grazie"),
                ("prímo", "primo"),
                ("díre", "dire"),
            ],
            // t→r corruption: "está"→"esrá", "tudo"→"rudo", "tem"→"rem" (ambiguous)
            // OCR: uppercase I for lowercase l: "paIavra" for "palavra"
            // OCR: "-Io" for "-lo" (clitic), "Ihe" for "lhe"
            Language::Portuguese => &[
                ("esrá", "está"),
                ("esre", "este"),
                ("rudo", "tudo"),
                ("conrra", "contra"),
                ("ourro", "outro"),
                ("denrro", "dentro"),
                ("enrão", "então"),
                // OCR: uppercase I substituted for lowercase l
                ("eIe", "ele"),
                ("paIavra", "palavra"),
                ("fIor", "flor"),
                ("Iugar", "lugar"),
                ("Iindo", "lindo"),
                ("reaImente", "realmente"),
                // OCR: I for l in clitics and common words
                ("-Io ", "-lo "),
                ("á-Io", "á-lo"),
                ("ê-Io", "ê-lo"),
                ("Ihe ", "lhe "),
                ("Ihes ", "lhes "),
                ("úItim", "últim"),
            ],
            // Encoding corruption: Windows-1251 decoded as Latin-1/ISO-8859-1 produces
            // garbled sequences like "Ð" prefixes. Also Latin homoglyphs mixed into Cyrillic:
            // Latin "a", "e", "o", "c", "p", "x" look identical to Cyrillic "а", "е", "о", "с", "р", "х"
            // but break text processing. We detect systematic Latin-for-Cyrillic substitution
            // by looking for Latin letters surrounded by Cyrillic context.
            Language::Russian => &[
                // Windows-1251 → Latin-1 mojibake: Cyrillic capital letters become Ð+something
                ("Ð\u{00B0}", "а"), // а
                ("Ð\u{00B5}", "е"), // е
                ("Ð¾", "о"),        // о (common mojibake pattern)
                ("Ñ\u{0082}", "т"), // т
                ("Ñ\u{0080}", "р"), // р
                // OCR: Ь (soft sign) misread as b
                ("6ы", "бы"),
                // Digit 3 for З (Ze)
                ("3а", "За"),
                ("3де", "Зде"),
            ],
            // No Latin script subtitles for these
            Language::Korean
            | Language::ChineseSimplified
            | Language::ChineseTraditional
            | Language::Japanese => &[],
            Language::Hindi => &[],
            Language::Thai => &[],
        }
    }

    /// Count Latin homoglyph characters that appear inside words that are otherwise Cyrillic.
    /// This detects subtitle corruption where visually identical Latin letters
    /// (a, e, o, c, p, x, y, A, B, C, E, H, K, M, O, P, T, X) replace their Cyrillic
    /// counterparts (а, е, о, с, р, х, у, А, В, С, Е, Н, К, М, О, Р, Т, Х).
    /// Non-homoglyph Latin chars (b, d, f, g, etc.) are ignored since they appear
    /// legitimately in foreign words with Russian case endings (e.g., "Mercedes'ом").
    fn count_latin_in_cyrillic_words(text: &str) -> usize {
        // Latin chars that are visual homoglyphs of Cyrillic chars
        const LATIN_HOMOGLYPHS: &[char] = &[
            'a', 'e', 'o', 'c', 'p', 'x', 'y', // lowercase
            'A', 'B', 'C', 'E', 'H', 'K', 'M', 'O', 'P', 'T', 'X', // uppercase
        ];
        let mut count = 0;
        for word in text.split(|c: char| c.is_whitespace() || c.is_ascii_punctuation()) {
            if word.is_empty() {
                continue;
            }
            let mut cyrillic_chars = 0usize;
            let mut homoglyph_chars = 0usize;
            for c in word.chars() {
                if ('\u{0400}'..='\u{04FF}').contains(&c) {
                    cyrillic_chars += 1;
                } else if LATIN_HOMOGLYPHS.contains(&c) {
                    homoglyph_chars += 1;
                }
            }
            // Only flag words that are mostly Cyrillic but have Latin homoglyphs mixed in
            if cyrillic_chars >= 2 && homoglyph_chars >= 1 {
                count += homoglyph_chars;
            }
        }
        count
    }

    /// Unified subtitle sanity check. Takes an iterator of subtitle sentence strings.
    /// Returns `Ok(())` if the subtitles pass, or `Err(reason)` describing why they failed.
    /// `skip_markers` allows whitelisting specific corruption markers for files with known
    /// false positives (e.g., a character named "Rourke" triggering the "rour" marker).
    pub fn check_subtitle_sanity<'a>(
        &self,
        sentences: impl Iterator<Item = &'a str>,
        skip_markers: &[&str],
    ) -> Result<(), String> {
        let lines: Vec<&str> = sentences.collect();
        let total_lines = lines.len();

        // 0. Minimum line count — reject fragments
        if total_lines < 75 {
            return Err(format!("too few lines ({total_lines}), likely a fragment"));
        }

        let all_text: String = lines.join(" ");
        let all_text_lower = all_text.to_lowercase();

        // 1. Check that all required sanity words appear
        let sanity_words = self.subtitle_sanity_words();
        for &word in sanity_words {
            let found = match self.writing_system() {
                WritingSystem::Latin | WritingSystem::Cyrillic => all_text_lower
                    .split(|c: char| !c.is_alphanumeric() && c != '\'')
                    .any(|w| w == word),
                _ => all_text_lower.contains(word),
            };
            if !found {
                return Err(format!("missing required word \"{word}\""));
            }
        }

        // 2. Check for systematic character corruption (e.g. t→r)
        let corruption_markers = self.subtitle_corruption_markers();
        for &(marker, expected) in corruption_markers {
            if skip_markers.contains(&marker) {
                continue;
            }
            // Some markers are case-sensitive (OCR patterns like "II"), check against original
            let count = if marker.chars().any(|c| c.is_uppercase()) {
                all_text.matches(marker).count()
            } else {
                all_text_lower.matches(marker).count()
            };
            if count >= 3 {
                return Err(format!(
                    "corruption: found \"{marker}\" {count} times (should be \"{expected}\")"
                ));
            }
        }

        // 3. OCR "I'" where "l'" should be (I followed by ' then lowercase) — French and Italian
        if matches!(self, Language::French | Language::Italian) {
            let mut i_apos_count = 0usize;
            let chars: Vec<char> = all_text.chars().collect();
            for i in 0..chars.len().saturating_sub(2) {
                if chars[i] == 'I'
                    && (chars[i + 1] == '\'' || chars[i + 1] == '\u{2019}')
                    && chars[i + 2].is_lowercase()
                {
                    i_apos_count += 1;
                }
            }
            if i_apos_count >= 5 {
                return Err(format!(
                    "OCR corruption: found \"I'\" followed by lowercase {i_apos_count} times (should be \"l'\")"
                ));
            }
        }

        // 4. SSA/ASS subtitle formatting codes that should have been stripped
        let ssa_count = all_text.matches("{\\an").count()
            + all_text.matches("{\\i1}").count()
            + all_text.matches("{\\i0}").count()
            + all_text.matches("{\\fs").count()
            + all_text.matches("{\\pos").count()
            + all_text.matches("{\\c&").count()
            + all_text.matches("{\\frz").count()
            + all_text.matches("\\h").count();
        if ssa_count >= 10 {
            return Err(format!(
                "SSA/ASS formatting codes found {ssa_count} times (should be stripped)"
            ));
        }

        // 5. BOM characters in text
        let bom_count = all_text.matches('\u{FEFF}').count();
        if bom_count >= 5 {
            return Err(format!("BOM characters (U+FEFF) found {bom_count} times"));
        }

        // 6. C1 control characters (U+0080-U+009F) indicate Windows-1252 mojibake
        let c1_count = all_text
            .chars()
            .filter(|&c| ('\u{0080}'..='\u{009F}').contains(&c))
            .count();
        if c1_count >= 5 {
            return Err(format!(
                "C1 control characters found {c1_count} times (Windows-1252 encoding corruption)"
            ));
        }

        // 7. Greek homoglyphs mixed into Latin text (subtitle copy-protection)
        if matches!(
            self.writing_system(),
            WritingSystem::Latin | WritingSystem::Cyrillic
        ) {
            let greek_count = all_text
                .chars()
                .filter(|&c| ('\u{0370}'..='\u{03FF}').contains(&c))
                .count();
            if greek_count >= 5 {
                return Err(format!(
                    "Greek homoglyph characters found {greek_count} times (copy-protection corruption)"
                ));
            }
        }

        // 8. Backtick or acute accent used as apostrophe
        let bad_apostrophe_count = all_text
            .chars()
            .filter(|&c| c == '`' || c == '\u{00B4}')
            .count();
        if bad_apostrophe_count >= 10 {
            return Err(format!(
                "Backtick/acute accent used as apostrophe {bad_apostrophe_count} times"
            ));
        }

        // 9. CJK characters in non-CJK subtitle files
        if !matches!(
            self,
            Language::ChineseSimplified
                | Language::ChineseTraditional
                | Language::Japanese
                | Language::Korean
                | Language::Hindi
        ) {
            let cjk_count = all_text
                .chars()
                .filter(|&c| {
                    ('\u{4E00}'..='\u{9FFF}').contains(&c)
                        || ('\u{3400}'..='\u{4DBF}').contains(&c)
                        || ('\u{F900}'..='\u{FAFF}').contains(&c)
                })
                .count();
            if cjk_count >= 20 {
                return Err(format!(
                    "CJK characters found {cjk_count} times in {self} subtitles (wrong language content)"
                ));
            }
        }

        // 10. Invisible directional Unicode markers (LRM U+200E, RLE U+202B, etc.)
        let invisible_dir_count = all_text
            .chars()
            .filter(|&c| {
                c == '\u{200E}'
                    || c == '\u{200F}'
                    || c == '\u{202A}'
                    || c == '\u{202B}'
                    || c == '\u{202C}'
                    || c == '\u{202D}'
                    || c == '\u{202E}'
                    || c == '\u{2066}'
                    || c == '\u{2067}'
                    || c == '\u{2068}'
                    || c == '\u{2069}'
                    || c == '\u{3164}'
            })
            .count();
        if invisible_dir_count >= 20 {
            return Err(format!(
                "Invisible directional/filler Unicode characters found {invisible_dir_count} times"
            ));
        }

        // 11. Spanish: missing inverted punctuation ¿ and ¡
        if matches!(self, Language::Spanish) && total_lines >= 100 {
            let questions = all_text.matches('?').count();
            let inv_questions = all_text.matches('¿').count();
            // If there are many questions but zero or near-zero inverted marks
            if questions >= 20 && inv_questions * 5 < questions {
                return Err(format!(
                    "Spanish missing ¿: {questions} questions but only {inv_questions} inverted marks"
                ));
            }
        }

        // 14. OCR: "fii" for "fi" (ligature mangling)
        if matches!(
            self,
            Language::English | Language::French | Language::Italian
        ) {
            let fii_count = all_text_lower.matches("fii").count();
            if fii_count >= 5 {
                return Err(format!(
                    "OCR ligature corruption: \"fii\" found {fii_count} times (should be \"fi\")"
                ));
            }
        }

        // 15. OCR: "0" for "O" at word boundaries (0lá, 0brigado, etc.)
        if matches!(self.writing_system(), WritingSystem::Latin) {
            let zero_for_o: usize = lines
                .iter()
                .map(|line| {
                    line.split_whitespace()
                        .filter(|w| {
                            w.starts_with('0')
                                && w.len() > 1
                                && w.chars().nth(1).is_some_and(|c| c.is_alphabetic())
                        })
                        .count()
                })
                .sum();
            if zero_for_o >= 5 {
                return Err(format!(
                    "OCR: digit 0 used for letter O found {zero_for_o} times"
                ));
            }
        }

        // 16. SRT timecodes leaked into text (e.g. "00:12:34,567 --> 00:12:36,789")
        let timecode_count = lines.iter().filter(|line| line.contains("-->")).count();
        if timecode_count >= 3 {
            return Err(format!(
                "SRT timecodes found in {timecode_count} lines (format conversion error)"
            ));
        }

        // 17. Russian: Latin homoglyphs mixed into Cyrillic text
        // Latin a/e/o/c/p/x/y/A/B/C/E/H/K/M/O/P/T/X look identical to
        // Cyrillic а/е/о/с/р/х/у/А/В/С/Е/Н/К/М/О/Р/Т/Х but break text processing.
        // Count Latin letters that appear inside otherwise-Cyrillic words.
        if matches!(self, Language::Russian) {
            let latin_in_cyrillic = Self::count_latin_in_cyrillic_words(&all_text);
            if latin_in_cyrillic >= 20 {
                return Err(format!(
                    "Latin homoglyphs mixed into Cyrillic text: {latin_in_cyrillic} Latin characters found inside Cyrillic words"
                ));
            }
        }

        // 18. Chinese script/variety check. OpenSubtitles files labeled zh-CN
        // are frequently Traditional (zh-TW/zh-HK) or even written Cantonese,
        // and vice versa — the sanity words 的/了/是/不 are identical in both
        // scripts, so this needs its own check. The wrong-script rejection is
        // symmetric: the Simplified course rejects Traditional-dominant files,
        // the Traditional course rejects Simplified-dominant ones.
        if matches!(
            self,
            Language::ChineseSimplified | Language::ChineseTraditional
        ) {
            // Written-Cantonese function words — Mandarin text has ~none.
            const CANTONESE_MARKERS: &str = "嘅咗唔佢哋冇咁嚟啲喺嗰乜嘢噉氹攞";

            let mut simplified = 0usize;
            let mut traditional = 0usize;
            let mut cantonese = 0usize;
            let mut han = 0usize;
            let mut kana = 0usize;
            let mut non_ws = 0usize;
            for c in all_text.chars() {
                if !c.is_whitespace() {
                    non_ws += 1;
                }
                if ('\u{3040}'..='\u{30FF}').contains(&c) {
                    kana += 1;
                }
                if ('\u{4E00}'..='\u{9FFF}').contains(&c) || ('\u{3400}'..='\u{4DBF}').contains(&c)
                {
                    han += 1;
                    if SIMPLIFIED_ONLY.contains(c) {
                        simplified += 1;
                    } else if TRADITIONAL_ONLY.contains(c) {
                        traditional += 1;
                    }
                    if CANTONESE_MARKERS.contains(c) {
                        cantonese += 1;
                    }
                }
            }
            // Japanese subs share han characters, so the sanity words alone
            // don't exclude them — kana does.
            if kana * 50 > non_ws {
                return Err(format!(
                    "Japanese contamination: {kana} kana characters in Chinese subtitles"
                ));
            }
            // Bilingual zh+en subs (every line carries an English translation)
            // still contain all the sanity words; require han-dominant text.
            if han * 2 < non_ws {
                return Err(format!(
                    "not predominantly Chinese: {han} han chars of {non_ws} total (bilingual or wrong language?)"
                ));
            }
            let marked = simplified + traditional;
            let (wrong_script, wrong_name, want_name) = match self {
                Language::ChineseSimplified => (traditional, "Traditional", "Simplified"),
                _ => (simplified, "Simplified", "Traditional"),
            };
            if marked >= 20 && wrong_script * 2 > marked {
                return Err(format!(
                    "{wrong_name} Chinese script ({wrong_script} of {marked} script-marked chars) — want {want_name}"
                ));
            }
            if cantonese >= 10 && cantonese * 1000 > han * 3 {
                return Err(format!(
                    "written Cantonese ({cantonese} Cantonese-only chars in {han} han chars)"
                ));
            }
        }

        // 16. Thai: script must dominate. The four sanity words above still all
        // appear in a bilingual Thai+English file, or an English file that
        // quotes a bit of Thai; a genuine Thai subtitle is overwhelmingly Thai
        // script. Mirrors the han-dominant check for Chinese above.
        if matches!(self, Language::Thai) {
            let mut thai = 0usize;
            let mut non_ws = 0usize;
            for c in all_text.chars() {
                if !c.is_whitespace() {
                    non_ws += 1;
                }
                if ('\u{0E00}'..='\u{0E7F}').contains(&c) {
                    thai += 1;
                }
            }
            if thai * 2 < non_ws {
                return Err(format!(
                    "not predominantly Thai: {thai} Thai chars of {non_ws} total (bilingual or wrong language?)"
                ));
            }
        }

        Ok(())
    }
}

/// The direction a prompt-driven voice (Gemini TTS) is given ahead of a
/// pronunciation cue. Delivery is the only thing left to the prompt — the
/// words come from [`pronunciation_challenge_spoken_text`] — and the pace
/// matters: unprompted, the model leaves a second of silence between
/// spelled letters. Named in English because it addresses the model, not
/// the learner.
pub fn pronunciation_challenge_tts_instructions(language: Language) -> String {
    format!(
        "Read this short {language} pronunciation cue aloud for a flashcard, in a clear, warm, \
         natural {language} voice, at a normal conversational pace with no long pauses. Say the \
         letter names as a quick spelled-out sequence, then the connecting words, then the \
         example word as a normal word. Say nothing else."
    )
}

/// One thing the voice says in a pronunciation clip, paired with what the
/// learner sees for it. A letter of the pattern is shown as itself but
/// spoken by name where [`Language::letter_name`] has one ("ü" / "u
/// Umlaut"); connector and example words read the same both ways.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SpokenSegment {
    pub display: String,
    pub spoken: String,
}

/// What a pronunciation cue says, in order: each letter of the pattern by
/// name, the connector words, then the example's words. A combining mark
/// (the stress accent in Russian patterns) belongs to the letter before it:
/// shown attached, and its name spoken after the letter's.
pub fn pronunciation_challenge_segments(
    language: Language,
    pattern: &str,
    example: &str,
) -> Vec<SpokenSegment> {
    let mut letters: Vec<SpokenSegment> = Vec::new();
    for c in pattern.chars() {
        let spoken = match language.letter_name(c) {
            Some(name) => name.to_string(),
            None => c.to_string(),
        };
        let is_combining_mark = ('\u{300}'..='\u{36f}').contains(&c);
        match letters.last_mut() {
            Some(previous) if is_combining_mark => {
                previous.display.push(c);
                previous.spoken.push(' ');
                previous.spoken.push_str(&spoken);
            }
            _ => letters.push(SpokenSegment {
                display: c.to_string(),
                spoken,
            }),
        }
    }
    let words = |text: &str| {
        text.split_whitespace()
            .map(|word| SpokenSegment {
                display: word.to_string(),
                spoken: word.to_string(),
            })
            .collect::<Vec<_>>()
    };
    letters
        .into_iter()
        .chain(words(language.pronunciation_connector()))
        .chain(words(example))
        .collect()
}

/// The text a pronunciation cue is synthesized from and verified against:
/// the spoken side of [`pronunciation_challenge_segments`]. One string for
/// both, so the voice and the phonemizer are always given the same words.
pub fn pronunciation_challenge_spoken_text(
    language: Language,
    pattern: &str,
    example: &str,
) -> String {
    pronunciation_challenge_segments(language, pattern, example)
        .into_iter()
        .map(|segment| segment.spoken)
        .collect::<Vec<_>>()
        .join(" ")
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::French => write!(f, "French"),
            Language::English => write!(f, "English"),
            Language::Spanish => write!(f, "Spanish"),
            Language::Korean => write!(f, "Korean"),
            Language::German => write!(f, "German"),
            Language::ChineseSimplified => write!(f, "Chinese (Simplified)"),
            Language::ChineseTraditional => write!(f, "Chinese (Traditional)"),
            Language::Japanese => write!(f, "Japanese"),
            Language::Russian => write!(f, "Russian"),
            Language::Portuguese => write!(f, "Portuguese"),
            Language::Italian => write!(f, "Italian"),
            Language::Hindi => write!(f, "Hindi"),
            Language::Thai => write!(f, "Thai"),
        }
    }
}

#[bridgerton::bridge(transparent)]
#[derive(
    Copy, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "camelCase")]
pub struct Course {
    pub native_language: Language,
    pub target_language: Language,
}

impl Course {
    pub fn teaches_new_writing_system(&self) -> bool {
        self.native_language.writing_system() != self.target_language.writing_system()
    }

    /// URL slug of this course on the public dictionary site
    /// (yap.town/d/<slug>/), e.g. "french-to-english".
    pub fn dictionary_slug(&self) -> String {
        format!(
            "{}-to-{}",
            self.target_language.to_string().to_lowercase(),
            self.native_language.to_string().to_lowercase()
        )
    }
}

/// URL slug of a dictionary entry on the public dictionary site, derived from
/// its display text. Colliding slugs get a `-2`/`-3`... suffix at site
/// generation time, which this function alone cannot know about.
pub fn dictionary_entry_slug(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_lowercase().next().unwrap_or(c)
            } else if c == ' ' || c == '\'' || c == '-' || c == '\u{2019}' {
                '-'
            } else {
                '_'
            }
        })
        .collect::<String>()
        .replace("--", "-")
        .trim_matches('-')
        .trim_matches('_')
        .to_string()
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
    Course {
        native_language: Language::English,
        target_language: Language::Italian,
    },
    Course {
        native_language: Language::English,
        target_language: Language::Portuguese,
    },
    Course {
        native_language: Language::French,
        target_language: Language::Portuguese,
    },
    Course {
        native_language: Language::English,
        target_language: Language::Russian,
    },
    Course {
        native_language: Language::English,
        target_language: Language::Hindi,
    },
    Course {
        native_language: Language::English,
        target_language: Language::Thai,
    },
    Course {
        native_language: Language::English,
        target_language: Language::ChineseSimplified,
    },
    Course {
        native_language: Language::English,
        target_language: Language::Japanese,
    },
];

pub const LANGUAGES: &[Language] = &[
    Language::French,
    Language::Spanish,
    Language::English,
    Language::Korean,
    Language::German,
    Language::ChineseSimplified,
    Language::ChineseTraditional,
    Language::Japanese,
    Language::Russian,
    Language::Portuguese,
    Language::Italian,
    Language::Hindi,
    Language::Thai,
];

/// A sentence example for the landing page showcase.
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ShowcaseExampleSentence {
    pub target: String,
    pub native: String,
}

/// A phrase entry for the landing page showcase.
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShowcasePhrase {
    pub display_text: String,
    pub definition: String,
    pub examples: Vec<ShowcaseExampleSentence>,
}

/// Landing page showcase data for a single course.
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseShowcase {
    pub target_language: Language,
    pub native_language: Language,
    pub sentence_count: usize,
    pub phrases: Vec<ShowcasePhrase>,
}

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
    S: rkyv::Archive + Hash + std::fmt::Debug + Eq + PartialEq + Ord + PartialOrd,
    <S as rkyv::Archive>::Archived: PartialEq + PartialOrd + Eq + Ord + Hash + std::fmt::Debug,
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
pub struct HomophoneSentencePair<S>
where
    S: rkyv::Archive + Hash + std::fmt::Debug + Eq + PartialEq + Ord + PartialOrd,
    <S as rkyv::Archive>::Archived: PartialEq + PartialOrd + Eq + Ord + Hash + std::fmt::Debug,
{
    /// Sentence using the first word (lexicographically)
    pub sentence1: S,
    /// Sentence using the second word (lexicographically)
    pub sentence2: S,
}

impl HomophoneSentencePair<String> {
    pub fn get_interned(
        &self,
        rodeo: &lasso::RodeoReader,
    ) -> Option<HomophoneSentencePair<lasso::Spur>> {
        Some(HomophoneSentencePair {
            sentence1: rodeo.get(&self.sentence1)?,
            sentence2: rodeo.get(&self.sentence2)?,
        })
    }

    fn get_or_intern(&self, rodeo: &mut lasso::Rodeo) -> HomophoneSentencePair<lasso::Spur> {
        HomophoneSentencePair {
            sentence1: rodeo.get_or_intern(&self.sentence1),
            sentence2: rodeo.get_or_intern(&self.sentence2),
        }
    }
}

impl HomophoneSentencePair<lasso::Spur> {
    pub fn resolve(&self, rodeo: &lasso::RodeoReader) -> HomophoneSentencePair<String> {
        HomophoneSentencePair {
            sentence1: rodeo.resolve(&self.sentence1).to_string(),
            sentence2: rodeo.resolve(&self.sentence2).to_string(),
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
pub struct HomophonePractice<S>
where
    S: rkyv::Archive + Hash + std::fmt::Debug + Eq + PartialEq + Ord + PartialOrd,
    <S as rkyv::Archive>::Archived: PartialEq + PartialOrd + Eq + Ord + Hash + std::fmt::Debug,
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
#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, schemars::JsonSchema)]
pub struct TtsRequest {
    pub text: String,
    pub language: Language,
    #[serde(default)]
    pub is_ssml: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(default = "default_speed")]
    pub speed: f64,
    /// Proper nouns occurring in `text`, handed to the backend's ASR gate as
    /// decoding context so it doesn't mistake a name it has never seen for a
    /// mispronunciation. Purely a verification aid — it never reaches the TTS
    /// provider and never changes the synthesized audio.
    ///
    /// The caller supplies these because only the client has the language
    /// pack's NLP tags; the backend has no way to find proper nouns itself.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub verification_hints: Vec<String>,
}

fn default_speed() -> f64 {
    1.0
}

/// Bumped whenever a backend change alters the audio produced for a request
/// that is itself unchanged — a different voice, model, or post-processing
/// step. Nothing in a `TtsRequest` describes *how* the backend renders it, so
/// without this a clip cached before such a change is indistinguishable from
/// one made after, and every cache (OPFS, the shared bucket) keeps serving
/// the old one forever.
///
/// A bump costs every user one round of cache misses. That is the whole point:
/// the alternative is a learner who already cached a defective clip never
/// hearing the fix.
///
/// - 1: Chirp3-HD started eating the text next to a `<break>`, so pronunciation
///   cards had cached audio that omitted the very letter they exist to teach.
const TTS_SYNTHESIS_REVISION: u32 = 1;

/// Cache filename for a TTS request. The key must include *every* input that
/// changes the synthesized audio — otherwise a request differing only in, say,
/// `speed` would be served a stale clip rendered at a different speed. One
/// function is shared by the browser's OPFS caches, the MCP server's temp
/// cache, and the backend's shared bucket so none of them can drift apart —
/// which is also what lets a client fetch the shared bucket directly, by a
/// key it computed itself, before ever asking the backend.
///
/// That includes inputs the request doesn't carry: `TTS_SYNTHESIS_REVISION`
/// stands in for the backend's own rendering decisions.
///
/// `verification_hints` is keyed too, which is easy to talk yourself out of —
/// it never reaches a TTS provider, so it can't change a single sample of any
/// one attempt. It does decide which attempt comes back: hints feed the ASR
/// gate, the gate decides whether a clip is accepted or the providers race,
/// and so the same text with and without hints can legitimately return
/// different audio. Keying them means a pack update that tags a new proper
/// noun re-verifies the sentences containing it rather than trusting a clip
/// that was accepted without ever knowing the name.
pub fn tts_cache_filename(request: &TtsRequest, provider: &TtsProvider) -> String {
    // Distinguish instructions None ('n') from Some("") ('s'): the /tts
    // handlers treat them differently (None = default prompt, Some("") = empty
    // prefix), so they must key to different clips.
    let (itag, instructions) = match request.instructions.as_deref() {
        Some(s) => ('s', s),
        None => ('n', ""),
    };
    // Length-prefix the free-form fields (text, instructions) so two distinct
    // requests can't collide via a colon embedded in the text — e.g. text
    // "a:b" vs text "a" + instructions "b" would otherwise hash identically.
    // The remaining fields have bounded, colon-free Debug/Display/numeric
    // forms, so they're safe to join directly.
    // Hints are joined with a separator that can't appear inside one (they're
    // single words from the pack) and length-prefixed like the other
    // free-form fields, so ["a", "b"] can't collide with ["a b"].
    let hints = request.verification_hints.join("\u{1f}");
    let cache_text = format!(
        "r{TTS_SYNTHESIS_REVISION}|{provider:?}|{language}|{speed}|{is_ssml}\
         |{tlen}:{text}|{itag}{ilen}:{instructions}|{hlen}:{hints}",
        language = request.language,
        speed = request.speed,
        is_ssml = request.is_ssml,
        tlen = request.text.len(),
        text = request.text,
        ilen = instructions.len(),
        hlen = hints.len(),
    );
    let cache_key = xxhash_rust::const_xxh3::xxh3_64(cache_text.as_bytes());
    format!("{cache_key}.mp3")
}

/// Public origin of the shared TTS cache: the `yap-tts-cache` R2 bucket on a
/// custom domain. Objects are named by [`tts_cache_filename`], so any client
/// can look a clip up without the backend — a hit skips Fly entirely (and
/// its cold start). Only clips that passed the backend's checks are ever
/// written, so a hit is always a verified clip.
pub const TTS_CACHE_ORIGIN: &str = "https://ttscache.yap.town";

/// Where the shared cache would serve the clip for `cache_filename`.
pub fn tts_cache_url(cache_filename: &str) -> String {
    format!("{TTS_CACHE_ORIGIN}/{cache_filename}")
}

/// Container format sniffed from magic bytes, as a mime type for playback
/// (and for the shared cache's `Content-Type`). The cache filename always
/// ends in `.mp3` regardless of what the winning provider actually returned,
/// so the bytes are the only trustworthy source.
pub fn audio_mime_type(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(b"RIFF") {
        "audio/wav"
    } else if bytes.starts_with(b"OggS") {
        "audio/ogg"
    } else {
        "audio/mpeg"
    }
}

#[cfg(test)]
mod tts_cache_key_tests {
    use super::*;

    fn request(text: &str) -> TtsRequest {
        TtsRequest {
            text: text.to_string(),
            language: Language::French,
            is_ssml: false,
            instructions: None,
            speed: 1.0,
            verification_hints: vec![],
        }
    }

    #[test]
    fn key_is_stable_across_processes() {
        // The browser computes this key and fetches the bucket by it; the
        // backend computes it independently when it writes. If this value
        // ever changes without a revision bump, every cached clip goes dark.
        assert_eq!(
            tts_cache_filename(&request("Bonjour tout le monde."), &TtsProvider::ElevenLabs),
            "5553454482266024564.mp3"
        );
    }

    #[test]
    fn every_input_changes_the_key() {
        let base = request("Bonjour");
        let base_key = tts_cache_filename(&base, &TtsProvider::ElevenLabs);
        let variants = [
            TtsRequest {
                text: "Bonjour!".into(),
                ..base.clone()
            },
            TtsRequest {
                language: Language::Spanish,
                ..base.clone()
            },
            TtsRequest {
                is_ssml: true,
                ..base.clone()
            },
            TtsRequest {
                instructions: Some(String::new()),
                ..base.clone()
            },
            TtsRequest {
                speed: 0.8,
                ..base.clone()
            },
            TtsRequest {
                verification_hints: vec!["Bonjour".into()],
                ..base.clone()
            },
        ];
        for variant in &variants {
            assert_ne!(
                tts_cache_filename(variant, &TtsProvider::ElevenLabs),
                base_key,
                "{variant:?} should not share a key with the base request"
            );
        }
        assert_ne!(tts_cache_filename(&base, &TtsProvider::Google), base_key);
    }

    #[test]
    fn mime_type_follows_the_bytes_not_the_name() {
        assert_eq!(audio_mime_type(b"RIFF...."), "audio/wav");
        assert_eq!(audio_mime_type(b"OggS...."), "audio/ogg");
        assert_eq!(audio_mime_type(b"ID3....."), "audio/mpeg");
    }
}

// ============================================================================
// Whitespace prediction for reconstructing text from atoms
// ============================================================================

/// French punctuation that requires narrow non-breaking space before it
const FRENCH_HIGH_PUNCT: &[char] = &['?', '!', ';', '»'];

/// Common Korean particles that attach directly to the preceding word
const KOREAN_PARTICLES: &[&str] = &[
    "이", "가", "을", "를", "은", "는", "에", "에서", "으로", "로", "와", "과", "하고", "의", "도",
    "만", "까지", "부터", "처럼", "같이", "보다", "마다", "이나", "나",
];

/// Punctuation that typically has no space after it
/// Includes Spanish inverted punctuation (¿ ¡) which attach to the following word
const NO_SPACE_AFTER: &[char] = &[
    '(', '[', '{', '«', '\'', '\u{2018}', '"', '\u{201C}', '-', '¿', '¡',
];

/// Punctuation that typically has no space before it
/// Note: apostrophe/quote chars are handled specially (opening vs closing)
/// Note: French » (closing guillemet) has space BEFORE it, so not in this list
/// Note: । and ॥ are the Devanagari danda/double danda (Hindi sentence enders)
const NO_SPACE_BEFORE: &[char] = &[
    ')', ']', '}', ',', '.', '?', '!', ';', '\u{2019}', '"', '\u{201D}', '-', '…', '।', '॥',
];

/// Characters that end words and attach directly (apostrophes, hyphens in compounds)
const ATTACHING_SUFFIXES: &[char] = &['\'', '\u{2019}', '-'];

/// English contraction clitics. lexide tokenizes "I'm", "don't" and "Tom's" as
/// "I" + "'m", "do" + "n't" and "Tom" + "'s": the clitic is its own token, with
/// its own lemma (be, not, be), but is written onto the word before it.
const ENGLISH_CLITICS: &[&str] = &["'s", "'m", "'re", "'ve", "'ll", "'d", "n't"];

/// Whether `text` is a clitic that attaches to the preceding word with no
/// space ("'s", "n't"). Straight and curly apostrophes both count; a lone
/// quote mark or a word that merely begins with an apostrophe ("'cause") is
/// not a clitic.
pub fn attaches_to_preceding_word(text: &str) -> bool {
    let normalized = text.to_lowercase().replace('\u{2019}', "'");
    ENGLISH_CLITICS.contains(&normalized.as_str())
}

/// Capitalize the first letter of a string, leaving the rest unchanged.
pub fn capitalize_first_letter(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

/// Returns true if the first letter of this word is always capitalized regardless of position.
///
/// This is used to avoid lowercasing words that carry meaning through their capitalization:
/// - Proper nouns in any language (e.g. "Paris", "Marie")
/// - All nouns in German (German capitalizes every noun)
/// - The pronoun "I" in English
/// - Fully-uppercase tokens with two or more uppercase letters in any
///   language ("OK", "DVD", "ADN"): their capitalization is a property of the
///   token, and lowercasing only the first letter would mint a freak gram
///   ("oK") distinct from the mid-sentence form. Two letters, not one, so
///   that elided clitics ("L'", "J'", "C'" — one uppercase letter plus an
///   apostrophe) still normalize to their lowercase grams.
pub fn first_letter_always_capitalized<S: AsRef<str>>(word: &Word<S>, language: Language) -> bool {
    // English "I" is always capitalized
    if language == Language::English && word.text.as_ref() == "I" {
        return true;
    }
    let text = word.text.as_ref();
    if text.chars().filter(|c| c.is_uppercase()).count() >= 2
        && !text.chars().any(|c| c.is_lowercase())
    {
        return true;
    }
    match &word.word_type {
        WordType::Other(other) => matches!(other.other_tag, OtherWordType::Propn),
        WordType::Heteronym(h) => language == Language::German && h.pos == PartOfSpeech::Noun,
    }
}

/// Lowercase the first letter of a word to match encoded gram form,
/// but only if the word's capitalization isn't intrinsic (e.g. proper nouns,
/// German nouns). Returns whether a change was made.
pub fn normalize_word_capitalization_for_gram_matching(
    word: &mut Word<String>,
    language: Language,
) -> bool {
    if !first_letter_always_capitalized(word, language) {
        let (lowercased, changed) = lowercase_first_letter(&word.text);
        if changed {
            word.text = lowercased;
            return true;
        }
    }
    false
}

/// Lowercase the first letter of a string, leaving the rest unchanged.
/// Returns the modified string and whether a change was made.
pub fn lowercase_first_letter(s: &str) -> (String, bool) {
    let mut chars = s.chars();
    match chars.next() {
        None => (String::new(), false),
        Some(first) => {
            if first.is_uppercase() {
                let lowercased: String = first.to_lowercase().chain(chars).collect();
                let changed = lowercased != s;
                (lowercased, changed)
            } else {
                (s.to_string(), false)
            }
        }
    }
}

/// Predicts the whitespace between two tokens based on deterministic rules.
///
/// This is the core function that enables whitespace normalization. By predicting
/// whitespace from token properties, we can omit explicit whitespace storage
/// and only emit Control tokens when the prediction is wrong.
///
/// Rules:
/// - After apostrophes/elisions: no space (l'amour, j'ai)
/// - Before French high punctuation (?, !, ;, ») in French: narrow nbsp
/// - After opening brackets/quotes: no space
/// - Before closing brackets/quotes/punctuation: no space
/// - After hyphen in compounds: no space
/// - Default: space
pub fn predict_whitespace(
    left: &Word<String>,
    right: Option<&Word<String>>,
    language: Language,
) -> Whitespace {
    let left_text = &left.text;

    // If there's no right token, no whitespace needed
    let right = match right {
        Some(r) => r,
        None => return Whitespace::None,
    };

    let right_text = &right.text;

    if attaches_to_preceding_word(right_text) {
        return Whitespace::None;
    }

    let left_last = left_text.chars().last();
    let right_first = right_text.chars().next();

    if let Some(c) = left_last
        && ATTACHING_SUFFIXES.contains(&c)
    {
        return Whitespace::None;
    }

    if let Some(c) = right_first {
        // No space before closing punct, commas, periods
        if NO_SPACE_BEFORE.contains(&c) {
            // Special case: French high punctuation needs narrow nbsp (only for French)
            if language == Language::French && FRENCH_HIGH_PUNCT.contains(&c) && c != '»' {
                return Whitespace::NarrowNbsp;
            }
            return Whitespace::None;
        }
    }

    if let Some(c) = left_last
        && NO_SPACE_AFTER.contains(&c)
    {
        return Whitespace::None;
    }

    // Exception: after colon before quotes, there's a space (e.g., "dit : 'hello'")
    // Exception: Spanish inverted punctuation (¿ ¡) has space before it after comma
    let left_is_punct =
        matches!(&left.word_type, WordType::Other(o) if o.other_tag == OtherWordType::Punct);
    let right_is_punct =
        matches!(&right.word_type, WordType::Other(o) if o.other_tag == OtherWordType::Punct);

    if left_is_punct && right_is_punct {
        // After colon, there's typically a space (before quotes, etc.)
        if left_last == Some(':') {
            return Whitespace::Space;
        }
        // Spanish: space before inverted punctuation (¿ ¡) after comma/period
        if right_first == Some('¿') || right_first == Some('¡') {
            return Whitespace::Space;
        }
        return Whitespace::None;
    }

    // Korean: particles attach directly to the preceding word, but only when
    // the POS actually indicates a particle/adposition (not a verb or other POS
    // that happens to share the same text as a particle).
    if language == Language::Korean && KOREAN_PARTICLES.contains(&right_text.as_str()) {
        let is_particle_pos = match &right.word_type {
            WordType::Heteronym(h) => matches!(h.pos, PartOfSpeech::Part | PartOfSpeech::Adp),
            WordType::Other(_) => true, // Non-heteronym tokens that match particle text are likely particles
        };
        if is_particle_pos {
            return Whitespace::None;
        }
    }

    // Scriptio continua (Japanese, Chinese, Thai): words attach directly.
    // Real spaces in source text (subtitle pauses, Thai phrase boundaries)
    // are the exception and get encoded as control tokens.
    if !language.writing_system().separates_words_with_spaces() {
        return Whitespace::None;
    }

    // Default: regular space
    Whitespace::Space
}

/// Convert a sequence of Literals into a sequence of Atoms.
///
/// This is the forward conversion that removes explicit whitespace and
/// replaces it with Control tokens where the prediction is wrong.
pub fn literals_to_atoms(
    literals: &[Literal<String>],
    language: Language,
) -> (Vec<Atom<String>>, bool) {
    if literals.is_empty() {
        return (Vec::new(), false);
    }

    let mut atoms = Vec::new();
    let mut capitalize_first = false;

    for (i, literal) in literals.iter().enumerate() {
        let mut word = literal.word.clone();

        if i == 0 && normalize_word_capitalization_for_gram_matching(&mut word, language) {
            capitalize_first = true;
        }

        // Emit the word token
        atoms.push(Atom::Tok(word.clone()));

        let next_word = literals.get(i + 1).map(|l| &l.word);
        let predicted = predict_whitespace(&word, next_word, language);
        let actual: Whitespace = literal.whitespace.parse().unwrap();

        // If prediction is wrong, emit a control token
        if predicted != actual {
            atoms.push(Atom::Control(ControlToken(actual)));
        }
    }

    (atoms, capitalize_first)
}

/// Convert a sequence of Atoms back into Literals.
///
/// This is the reverse conversion that reconstructs the original
/// whitespace from predictions and control tokens.
pub fn atoms_to_literals(atoms: &[Atom<String>], language: Language) -> Vec<Literal<String>> {
    let mut literals = Vec::new();
    let mut i = 0;

    while i < atoms.len() {
        let atom = &atoms[i];

        match atom {
            Atom::<String>::Tok(word) => {
                let word = word.clone();

                // Look ahead to determine whitespace
                let whitespace = if i + 1 < atoms.len() {
                    match &atoms[i + 1] {
                        // If next is a control token, use its whitespace
                        Atom::<String>::Control(ctrl) => {
                            i += 1; // consume the control token
                            ctrl.0
                        }
                        // Otherwise predict based on next word
                        Atom::<String>::Tok(next_word) => {
                            predict_whitespace(&word, Some(next_word), language)
                        }
                    }
                } else {
                    // Last token - predict with no lookahead
                    predict_whitespace(&word, None, language)
                };

                literals.push(Literal {
                    word,
                    whitespace: whitespace.to_str().to_string(),
                });
            }
            Atom::<String>::Control(_) => {
                // Standalone control tokens shouldn't happen in well-formed input,
                // but if they do, skip them
            }
        }

        i += 1;
    }

    literals
}

/// Reconstruct the original sentence text from literals
pub fn literals_to_text(literals: &[Literal<String>]) -> String {
    literals
        .iter()
        .map(|lit| format!("{}{}", lit.word.text, lit.whitespace))
        .collect()
}

#[cfg(test)]
mod original_language_tests {
    use super::*;

    /// The whole point: a normalized `original_language` compares equal to
    /// `Language::iso_639_1`, so plain `==` is correct at every call site.
    #[test]
    fn tmdb_cn_matches_the_chinese_courses() {
        let movie: MovieMetadataBasic = serde_json::from_str(
            r#"{"id":"tt0338564","title":"無間道","year":2002,"original_language":"cn"}"#,
        )
        .unwrap();

        assert_eq!(movie.original_language.as_deref(), Some("zh"));
        for chinese in [Language::ChineseSimplified, Language::ChineseTraditional] {
            assert_eq!(
                movie.original_language.as_deref(),
                Some(chinese.iso_639_1())
            );
        }
    }

    #[test]
    fn real_iso_codes_pass_through_untouched() {
        for code in ["en", "fr", "ja", "zh", "th", "ka", "tl"] {
            assert_eq!(normalize_original_language(code), code);
        }
    }

    #[test]
    fn a_missing_original_language_stays_none() {
        let movie: MovieMetadataBasic =
            serde_json::from_str(r#"{"id":"tt0000001","title":"?","year":null}"#).unwrap();
        assert_eq!(movie.original_language, None);
    }
}

#[cfg(test)]
mod capitalization_tests {
    use super::*;

    fn word(text: &str, pos: PartOfSpeech) -> Word<String> {
        Word {
            text: text.to_string(),
            word_type: WordType::Heteronym(Heteronym {
                word: text.to_lowercase(),
                lemma: text.to_lowercase(),
                pos,
            }),
        }
    }

    #[test]
    fn english_clitics_attach_to_the_preceding_word() {
        let left = word("I", PartOfSpeech::Pron);
        for clitic in ["'m", "'s", "n't", "\u{2019}ll", "'S"] {
            assert_eq!(
                predict_whitespace(
                    &left,
                    Some(&word(clitic, PartOfSpeech::Aux)),
                    Language::English
                ),
                Whitespace::None,
                "{clitic} should attach to the word before it"
            );
        }
        // A word that merely starts with an apostrophe keeps its space.
        assert_eq!(
            predict_whitespace(
                &left,
                Some(&word("'cause", PartOfSpeech::Sconj)),
                Language::English
            ),
            Whitespace::Space
        );
    }

    #[test]
    fn all_caps_tokens_keep_their_capitalization() {
        for caps in ["OK", "DVD", "ADN", "L.A."] {
            assert!(
                first_letter_always_capitalized(&word(caps, PartOfSpeech::Intj), Language::French),
                "{caps} should keep its capitalization"
            );
        }
    }

    #[test]
    fn elided_clitics_still_normalize() {
        // One uppercase letter plus an apostrophe is an elision, not an
        // acronym: sentence-initial "L'homme" must still normalize "L'" to
        // the "l'" gram (a regression here splits the article vocabulary).
        for clitic in ["L'", "J'", "C'", "D'", "N'", "M'", "S'", "T'"] {
            assert!(
                !first_letter_always_capitalized(
                    &word(clitic, PartOfSpeech::Det),
                    Language::French
                ),
                "{clitic} should still be decapitalized"
            );
        }
        assert!(!first_letter_always_capitalized(
            &word("Le", PartOfSpeech::Det),
            Language::French
        ));
    }
}

#[cfg(test)]
mod predict_whitespace_tests {
    use super::*;

    fn word(text: &str, pos: PartOfSpeech) -> Word<String> {
        Word {
            text: text.to_string(),
            word_type: WordType::Heteronym(Heteronym {
                word: text.to_string(),
                lemma: text.to_string(),
                pos,
            }),
        }
    }

    fn predict(left: &Word<String>, right: &Word<String>, language: Language) -> Whitespace {
        predict_whitespace(left, Some(right), language)
    }

    #[test]
    fn japanese_words_attach() {
        let kare = word("彼", PartOfSpeech::Pron);
        let wa = word("は", PartOfSpeech::Adp);
        assert_eq!(predict(&kare, &wa, Language::Japanese), Whitespace::None);
    }

    #[test]
    fn chinese_words_attach() {
        let wo = word("我", PartOfSpeech::Pron);
        let hao = word("好", PartOfSpeech::Adj);
        assert_eq!(
            predict(&wo, &hao, Language::ChineseSimplified),
            Whitespace::None
        );
        assert_eq!(
            predict(&wo, &hao, Language::ChineseTraditional),
            Whitespace::None
        );
    }

    #[test]
    fn thai_words_attach() {
        let sawasdee = word("สวัสดี", PartOfSpeech::Intj);
        let khrap = word("ครับ", PartOfSpeech::Part);
        assert_eq!(predict(&sawasdee, &khrap, Language::Thai), Whitespace::None);
    }

    #[test]
    fn hindi_words_are_spaced_but_danda_attaches() {
        let hai = word("है", PartOfSpeech::Aux);
        let acha = word("अच्छा", PartOfSpeech::Adj);
        let danda = Word {
            text: "।".to_string(),
            word_type: WordType::Other(OtherWord {
                other_tag: OtherWordType::Punct,
            }),
        };
        assert_eq!(predict(&acha, &hai, Language::Hindi), Whitespace::Space);
        assert_eq!(predict(&hai, &danda, Language::Hindi), Whitespace::None);
    }

    #[test]
    fn spaced_languages_unchanged() {
        let le = word("le", PartOfSpeech::Det);
        let chat = word("chat", PartOfSpeech::Noun);
        assert_eq!(predict(&le, &chat, Language::French), Whitespace::Space);
    }
}

#[cfg(test)]
mod subtitle_script_tests {
    use super::*;

    // check_subtitle_sanity rejects anything under 75 lines before the script
    // check runs, so repeat the sample enough times to clear that bar. The
    // sanity words 的/了/是/不 must also all appear; the filler providing them
    // matches the script under test so it doesn't skew the script counts.
    fn lines(sample: &[&str], filler: &str) -> Vec<String> {
        let mut out = Vec::new();
        while out.len() < 80 {
            out.extend(sample.iter().map(|s| s.to_string()));
            out.push(filler.to_string());
        }
        out
    }

    fn check_as(language: Language, sample: &[&str]) -> Result<(), String> {
        let filler = if language == Language::ChineseTraditional {
            "他說的是不了"
        } else {
            "他说的是不了"
        };
        let lines = lines(sample, filler);
        language.check_subtitle_sanity(lines.iter().map(|s| s.as_str()), &[])
    }

    fn check(sample: &[&str]) -> Result<(), String> {
        check_as(Language::ChineseSimplified, sample)
    }

    #[test]
    fn simplified_mandarin_passes() {
        check(&[
            "我们这时候还没开门",
            "你说的对，他们来学校了",
            "这个问题很难，谁能回答",
        ])
        .unwrap();
    }

    #[test]
    fn traditional_script_rejected() {
        let err = check(&[
            "我們這時候還沒開門",
            "你說的對，他們來學校了",
            "這個問題很難，誰能回答",
        ])
        .unwrap_err();
        assert!(err.contains("Traditional"), "{err}");
    }

    #[test]
    fn traditional_passes_for_traditional_course() {
        check_as(
            Language::ChineseTraditional,
            &[
                "我們這時候還沒開門",
                "你說的對，他們來學校了",
                "這個問題很難，誰能回答",
            ],
        )
        .unwrap();
    }

    #[test]
    fn simplified_rejected_for_traditional_course() {
        let err = check_as(
            Language::ChineseTraditional,
            &[
                "我们这时候还没开门",
                "你说的对，他们来学校了",
                "这个问题很难，谁能回答",
            ],
        )
        .unwrap_err();
        assert!(err.contains("Simplified"), "{err}");
    }

    #[test]
    fn bilingual_zh_en_rejected() {
        let err = check(&[
            "深渊怪物能助你 And the Abyss monsters can help destroy your enemy.",
            "修复仙酿！ Elixir Reparo! This line is mostly English text overall.",
            "我们走吧 Let us go now, everyone, the ceremony is about to begin.",
        ])
        .unwrap_err();
        assert!(err.contains("predominantly"), "{err}");
    }

    #[test]
    fn japanese_kana_rejected() {
        let err = check(&[
            "骗人的吧",
            "そうですね、これは日本語の字幕ですから、だめですよ",
            "什么意思",
        ])
        .unwrap_err();
        assert!(err.contains("Japanese"), "{err}");
    }

    #[test]
    fn written_cantonese_rejected() {
        let err = check(&[
            "佢哋唔知道你喺邊度",
            "我冇嘢講，你咁樣做係唔啱嘅",
            "佢咗嗰度攞啲嘢",
        ])
        .unwrap_err();
        // Cantonese subs are usually Traditional too; either rejection is correct.
        assert!(
            err.contains("Cantonese") || err.contains("Traditional"),
            "{err}"
        );
    }

    // Thai has no spaces between words, so the helper above (which relies on a
    // Chinese filler to supply the sanity words) doesn't fit; build Thai lines
    // directly. The filler carries all four Thai sanity words: ไม่ ที่ ได้ จะ.
    fn thai_lines(sample: &[&str]) -> Vec<String> {
        let filler = "เขาบอกว่าไม่ได้ที่จะไปที่นั่น";
        let mut out = Vec::new();
        while out.len() < 80 {
            out.extend(sample.iter().map(|s| s.to_string()));
            out.push(filler.to_string());
        }
        out
    }

    #[test]
    fn thai_passes() {
        let lines = thai_lines(&[
            "เรายังไม่ได้เปิดร้านตอนนี้",
            "สิ่งที่คุณพูดถูกต้องแล้ว",
            "คำถามนี้ยากมากใครจะตอบได้",
        ]);
        Language::Thai
            .check_subtitle_sanity(lines.iter().map(|s| s.as_str()), &[])
            .unwrap();
    }

    #[test]
    fn bilingual_thai_en_rejected() {
        // Every line is mostly English with a little Thai, so the sanity words
        // still appear but Thai script no longer dominates.
        let lines = thai_lines(&[
            "ไม่ And the monsters here can help you destroy every one of your enemies.",
            "ที่ Elixir! This particular line is overwhelmingly English text overall.",
            "จะ Let us all go now, everyone, because the ceremony is about to begin.",
        ]);
        let err = Language::Thai
            .check_subtitle_sanity(lines.iter().map(|s| s.as_str()), &[])
            .unwrap_err();
        assert!(err.contains("predominantly Thai"), "{err}");
    }

    #[test]
    fn english_rejected_as_thai() {
        // A plain English subtitle mislabeled as Thai: no Thai sanity words.
        let sample = [
            "I don't know where you are right now, my friend.",
            "What you said earlier was actually completely correct.",
            "This question is very hard, who among us could answer it.",
        ];
        let mut lines = Vec::new();
        while lines.len() < 80 {
            lines.extend(sample.iter().map(|s| s.to_string()));
        }
        let err = Language::Thai
            .check_subtitle_sanity(lines.iter().map(|s| s.as_str()), &[])
            .unwrap_err();
        // Fails on the first missing Thai sanity word.
        assert!(err.contains("missing required word"), "{err}");
    }
}

#[cfg(test)]
mod autograde_request_compat_tests {
    use super::autograde::*;
    use super::*;

    /// A request serialized by a client that predates `GraderContext` (no
    /// `context` field) must still deserialize — the field defaults to empty.
    #[test]
    fn autograde_requests_accept_missing_context() {
        let course = Course {
            target_language: Language::French,
            native_language: Language::English,
        };
        let translation = AutoGradeTranslationRequest {
            course,
            challenge_sentence: "Vous y tenez ?".to_string(),
            user_sentence: "Do you like it?".to_string(),
            literals: vec![],
            phrases: vec![],
            primary_expression: Gram(vec![]),
            context: Default::default(),
        };
        let mut old_json = serde_json::to_value(&translation).unwrap();
        old_json.as_object_mut().unwrap().remove("context");
        let parsed: AutoGradeTranslationRequest = serde_json::from_value(old_json).unwrap();
        assert!(parsed.context.is_empty());

        let transcription = AutoGradeTranscriptionRequest {
            course,
            submission: vec![],
            context: GraderContext {
                movie_title: Some("Delicatessen".to_string()),
                dialogue_before: vec![],
                dialogue_after: vec![],
            },
        };
        let mut old_json = serde_json::to_value(&transcription).unwrap();
        old_json.as_object_mut().unwrap().remove("context");
        let parsed: AutoGradeTranscriptionRequest = serde_json::from_value(old_json).unwrap();
        assert!(parsed.context.is_empty());
        // And a round trip with context keeps it.
        let round: AutoGradeTranscriptionRequest =
            serde_json::from_value(serde_json::to_value(&transcription).unwrap()).unwrap();
        assert_eq!(round.context.movie_title.as_deref(), Some("Delicatessen"));
    }
}

#[cfg(test)]
mod pronunciation_challenge_audio_tests {
    use super::*;

    #[test]
    fn bare_letters_are_their_own_names() {
        assert_eq!(
            pronunciation_challenge_spoken_text(Language::French, "ch", "chat"),
            "c h comme dans chat"
        );
    }

    #[test]
    fn accented_letters_speak_their_technical_names() {
        assert_eq!(
            pronunciation_challenge_spoken_text(Language::German, "ü", "über"),
            "u Umlaut wie in über"
        );
        assert_eq!(
            pronunciation_challenge_spoken_text(Language::German, "sch", "Schule"),
            "s c h wie in Schule"
        );
        assert_eq!(
            pronunciation_challenge_spoken_text(Language::Spanish, "ñ", "niño"),
            "eñe como en niño"
        );
        assert_eq!(
            pronunciation_challenge_spoken_text(Language::Russian, "щ", "борщ"),
            "ща как в борщ"
        );
        assert_eq!(
            pronunciation_challenge_spoken_text(Language::Korean, "ㅋ", "코"),
            "키읔 처럼 코"
        );
        assert_eq!(
            pronunciation_challenge_spoken_text(Language::Japanese, "きぇ", "きぇーっ"),
            "き 小さいえ のように きぇーっ"
        );
    }

    #[test]
    fn stress_marks_stay_attached_to_their_vowel() {
        let segments = pronunciation_challenge_segments(Language::Russian, "а\u{301}", "мама");
        let pairs: Vec<(&str, &str)> = segments
            .iter()
            .map(|s| (s.display.as_str(), s.spoken.as_str()))
            .collect();
        assert_eq!(
            pairs,
            [
                ("а\u{301}", "а с ударением"),
                ("как", "как"),
                ("в", "в"),
                ("мама", "мама"),
            ]
        );
    }

    #[test]
    fn han_is_the_only_non_phonographic_script() {
        assert!(!Language::ChineseSimplified.is_phonographic());
        assert!(!Language::ChineseTraditional.is_phonographic());
        assert!(Language::Japanese.is_phonographic());
        assert!(Language::Thai.is_phonographic());
        assert!(Language::French.is_phonographic());
    }

    #[test]
    fn tts_instructions_name_the_language() {
        let instructions = pronunciation_challenge_tts_instructions(Language::Portuguese);
        assert!(instructions.contains("Portuguese pronunciation cue"));
        assert!(instructions.ends_with("Say nothing else."));
    }

    #[test]
    fn spoken_text_names_letters_that_are_also_words() {
        assert_eq!(
            pronunciation_challenge_spoken_text(Language::French, "y", "pays"),
            "i grec comme dans pays"
        );
        assert_eq!(
            pronunciation_challenge_spoken_text(Language::French, "à", "voilà"),
            "a accent grave comme dans voilà"
        );
        assert_eq!(
            pronunciation_challenge_spoken_text(Language::French, "oy", "Troyes"),
            "o i grec comme dans Troyes"
        );
        assert_eq!(
            pronunciation_challenge_spoken_text(Language::English, "ea", "bread"),
            "e eh as in bread"
        );
    }

    #[test]
    fn portuguese_accented_letters_speak_their_names() {
        assert_eq!(
            pronunciation_challenge_spoken_text(Language::Portuguese, "á", "chá"),
            "a acento agudo como em chá"
        );
        assert_eq!(
            pronunciation_challenge_spoken_text(Language::Portuguese, "ãe", "pães"),
            "a til e como em pães"
        );
    }

    #[test]
    fn segments_show_letters_but_speak_their_names() {
        let segments = pronunciation_challenge_segments(Language::French, "oy", "à la carte");
        let pairs: Vec<(&str, &str)> = segments
            .iter()
            .map(|s| (s.display.as_str(), s.spoken.as_str()))
            .collect();
        assert_eq!(
            pairs,
            [
                ("o", "o"),
                ("y", "i grec"),
                ("comme", "comme"),
                ("dans", "dans"),
                ("à", "à"),
                ("la", "la"),
                ("carte", "carte"),
            ]
        );
    }
}
