use crate::GramFrequencyList;
use language_utils::*;
use rustc_hash::FxHashMap;
use std::collections::BTreeMap;

/// Consolidated data structure containing all generated language data
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConsolidatedLanguageData {
    /// Learner-facing redundancy sets, keyed by the exact prefixed front.
    pub redundant_senses: BTreeMap<String, Vec<Vec<TaggedGram<Gram<String>>>>>,
    /// Downloaded stroke forms for the course's writable units.
    pub strokes: StrokeTable,
    /// All target language sentences from Anki cards
    pub target_language_sentences: Vec<String>,
    /// Mapping from target language sentences to all native translations
    pub translations: Vec<(String, Vec<String>)>,
    /// NLP-analyzed sentences with multiword terms and heteronyms
    pub nlp_sentences: Vec<(String, SentenceInfo)>,
    /// Unified phrasebook: maps gram to definition entry (both MWE phrases and learned multi-atom grams)
    pub phrasebook: BTreeMap<TaggedGram<Gram<String>>, PhrasebookDefinitionEntry>,
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
    pub encoded_sentences: Vec<(String, ReadingSentence<Gram<String>, String>)>,
    /// Gram dictionary: definitions for grams (keyed by Gram for correct surface-form matching)
    pub gram_dictionary: BTreeMap<TaggedGram<Gram<String>>, DictionaryEntry>,
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

        for gram in self.gram_dictionary.keys() {
            for atom in gram.gram.iter() {
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
        for word_pair in self.homophone_practice.keys() {
            rodeo.get_or_intern(&word_pair.word1);
            rodeo.get_or_intern(&word_pair.word2);
        }

        // intern movie data
        for movie in self.movies.values() {
            rodeo.get_or_intern(&movie.id);
            rodeo.get_or_intern(&movie.title);
        }

        // Intern source metadata, not sentence text keys.
        for (_, source) in &self.sentence_sources {
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

        // Intern encoded atoms and readings, not sentence text keys.
        for (_, encoded) in &self.encoded_sentences {
            for entry in encoded.all_grams() {
                if let Some(reading) = &entry.reading {
                    rodeo.get_or_intern(reading);
                }
                for atom in &entry.sense.gram {
                    atom.get_or_intern(rodeo);
                }
            }
        }

        // intern source gram frequencies
        for freq_list in self.source_gram_frequencies.values() {
            for entry in &freq_list.entries {
                if let Some(reading) = &entry.gram.reading {
                    rodeo.get_or_intern(reading);
                }
                for atom in &entry.gram.sense.gram {
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
            if let Some(reading) = &entry.gram.reading {
                rodeo.get_or_intern(reading);
            }
            for atom in &entry.gram.sense.gram {
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
