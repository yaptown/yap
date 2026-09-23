use language_utils::features::{Morphology, WordPrefix};
use language_utils::text_cleanup::remove_accents_lowercase;
use language_utils::{Atom, Gram, GramDefinition, Language, WordType};
use rustc_hash::{FxHashMap, FxHashSet};
use yap_frontend_reducers::{DefinitionView, definition_view};

use crate::{
    AudioRequest, CardData, CardIndicator, Deck, DeckEvent, LanguageEvent, LanguageEventContent,
};
use language_utils::{TtsProvider, TtsRequest};

/// Compute the grammatical prefix for a gram, given its definition
/// and target language.
///
/// For single-word grams (one `Atom::Tok`), uses the per-heteronym single-word
/// prefix — articles for nouns, subject pronouns for verbs, etc. — based on the
/// gram's heteronym and the first morphology entry from the dictionary definition.
/// Single-word prefixes only apply to Dictionary entries (Phrasebook entries lack
/// per-word morphology).
///
/// For multi-word grams (more than one `Atom::Tok`), uses the language-level
/// multiword prefix (e.g., Korean auxiliary connective endings on compound verbs).
/// Multi-word prefixes work for both Dictionary and Phrasebook entries since they
/// only depend on the gram's atom-level heteronym tags.
pub(crate) fn compute_word_prefix(
    gram: &Gram<String>,
    definition: &GramDefinition,
    target_language: Language,
) -> Option<WordPrefix> {
    let dict_def = match definition {
        GramDefinition::Dictionary(d) => Some(d),
        GramDefinition::Phrasebook(_) => None,
    };

    match gram.atoms() {
        // Single-word gram: per-heteronym single-word prefix, only meaningful for
        // Dictionary entries (which carry per-word morphology).
        [Atom::Tok(word)] => {
            let WordType::Heteronym(h) = &word.word_type else {
                return None;
            };
            let morphology = dict_def.and_then(|d| d.morphology.first());
            morphology.and_then(|m| m.get_prefix(&h.word, h.pos, target_language))
        }
        // Single-atom gram with no Tok: nothing to compute.
        [_] => None,
        // No grams: nothing to compute
        [] => None,
        // Multi-word gram: language-level multiword prefix.
        _ => Morphology::get_multiword_prefix(gram, target_language),
    }
}

/// Get dictionary words ordered by frequency (most common first).
/// Optionally filters by search query (accent-insensitive) and limits results.
#[bridgerton::bridge]
impl Deck {
    pub fn get_gram_dictionary_entries(
        &self,
        search_query: Option<String>,
        limit: usize,
    ) -> Vec<DictionaryWord> {
        self.get_gram_dictionary_page(search_query, 0, limit)
    }

    /// A bounded page in the same relevance/frequency order as dictionary search.
    /// Native callers must not transfer the whole dictionary through the bridge.
    pub fn get_gram_dictionary_page(
        &self,
        search_query: Option<String>,
        offset: usize,
        limit: usize,
    ) -> Vec<DictionaryWord> {
        let language_pack = &self.context.language_pack;
        let target_language = self.context.course.target_language;

        let query = search_query
            .filter(|q| !q.trim().is_empty())
            .map(|q| remove_accents_lowercase(&q));

        // Preserve the primary index even when only a later sense matches.
        let mut words = FxHashMap::default();
        for (frequency_index, (spur_gram, _)) in
            language_pack.gram_frequencies.entries.iter().enumerate()
        {
            let Some(gram_def) = language_pack.gram_definitions.get(spur_gram) else {
                continue;
            };
            let word = words
                .entry(spur_gram.gram)
                .or_insert((frequency_index, None));
            // All matching senses have the same display-based relevance.
            if word.1.is_some() {
                continue;
            }
            let relevance = if let Some(q) = &query {
                let display_text = language_pack
                    .resolve_gram(&spur_gram.gram)
                    .to_display_string(target_language);
                let normalized_display = remove_accents_lowercase(&display_text);
                let definition_contains = match gram_def {
                    GramDefinition::Dictionary(dict_def) => dict_def
                        .definitions
                        .iter()
                        .any(|d| remove_accents_lowercase(&d.native).contains(q.as_str())),
                    GramDefinition::Phrasebook(pb_def) => {
                        remove_accents_lowercase(&pb_def.meaning).contains(q.as_str())
                    }
                };
                if !normalized_display.contains(q.as_str()) && !definition_contains {
                    continue;
                }
                // Exact match, starts-with, then contains (including definition-only).
                if normalized_display == *q {
                    0
                } else if normalized_display.starts_with(q.as_str()) {
                    1
                } else {
                    2
                }
            } else {
                0
            };
            word.1 = Some(relevance);
        }
        let mut entries: Vec<(u8, usize)> = words
            .into_values()
            .filter_map(|(index, relevance)| relevance.map(|relevance| (relevance, index)))
            .collect();

        let end = offset.saturating_add(limit).min(entries.len());
        if end < entries.len() {
            entries.select_nth_unstable(end);
            entries.truncate(end);
        }
        entries.sort_unstable();
        entries
            .into_iter()
            .skip(offset)
            .filter_map(|(_, index)| self.gram_dictionary_entry(index))
            .collect()
    }

    /// Build the whole word containing any member sense's frequency index.
    /// Returns None for an invalid index or a word with no defined senses.
    pub fn gram_dictionary_entry(&self, frequency_index: usize) -> Option<DictionaryWord> {
        let language_pack = &self.context.language_pack;
        let target_language = self.context.course.target_language;
        let (spur_gram, _) = language_pack
            .gram_frequencies
            .entries
            .get_index(frequency_index)?;
        let mut senses: Vec<_> = language_pack
            .senses_of(spur_gram.gram)
            .iter()
            .filter_map(|sense_gram| {
                let definition =
                    definition_view(language_pack.gram_definitions.get(sense_gram)?.clone());
                let frequency_index = language_pack
                    .gram_frequencies
                    .entries
                    .get_index_of(sense_gram)?;
                let card = CardIndicator::WrittenGram { gram: *sense_gram };
                Some(DictionarySense {
                    frequency_index,
                    is_in_deck: matches!(self.cards.get(&card), Some(CardData::Added { .. })),
                    gloss: definition
                        .senses
                        .iter()
                        .take(2)
                        .map(|sense| sense.meaning.as_str())
                        .collect::<Vec<_>>()
                        .join("; "),
                    definition,
                })
            })
            .collect();
        senses.sort_unstable_by_key(|sense| sense.frequency_index);
        let primary = senses.first()?;
        let (primary_gram, _) = language_pack
            .gram_frequencies
            .entries
            .get_index(primary.frequency_index)?;
        let gram_def = language_pack.gram_definitions.get(primary_gram)?;
        let resolved_gram = language_pack.resolve_gram(&spur_gram.gram);
        Some(DictionaryWord {
            display_text: resolved_gram.to_display_string(target_language),
            frequency_index: primary.frequency_index,
            prefix: compute_word_prefix(&resolved_gram, gram_def, target_language),
            is_phrase: primary.definition.is_phrase,
            target_language,
            senses,
        })
    }

    /// Count distinct words with at least one defined sense.
    pub fn get_gram_dictionary_count(&self) -> usize {
        let language_pack = &self.context.language_pack;
        language_pack
            .gram_frequencies
            .entries
            .iter()
            .filter(|(spur_gram, _)| language_pack.gram_definitions.contains_key(spur_gram))
            .map(|(spur_gram, _)| spur_gram.gram)
            .collect::<FxHashSet<_>>()
            .len()
    }

    /// Create a DeckEvent for adding a gram/phrase by its frequency index
    pub fn add_gram_by_frequency_index(&self, frequency_index: usize) -> Option<DeckEvent> {
        let language_pack = &self.context.language_pack;
        let (spur_gram, _freq) = language_pack
            .gram_frequencies
            .entries
            .get_index(frequency_index)?;

        let card = CardIndicator::WrittenGram { gram: *spur_gram };
        let resolved_card = card.resolve(&language_pack.string_rodeo, &language_pack.gram_rodeo);

        Some(DeckEvent::Language(LanguageEvent {
            target_language: self.context.course.target_language,
            native_language: self.context.course.native_language,
            content: LanguageEventContent::AddCards {
                cards: vec![resolved_card],
                sentence_list: None,
            },
        }))
    }
}

#[derive(Debug, Clone)]
#[bridgerton::bridge(opaque)]
pub struct DictionaryWord {
    display_text: String,
    frequency_index: usize,
    prefix: Option<WordPrefix>,
    is_phrase: bool,
    target_language: Language,
    senses: Vec<DictionarySense>,
}

#[bridgerton::bridge(transparent)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DictionarySense {
    pub frequency_index: usize,
    pub is_in_deck: bool,
    pub gloss: String,
    pub definition: DefinitionView,
}

#[bridgerton::bridge]
impl DictionaryWord {
    #[bridge(getter)]
    pub fn display_text(&self) -> String {
        self.display_text.clone()
    }

    #[bridge(getter)]
    pub fn frequency_index(&self) -> usize {
        self.frequency_index
    }

    #[bridge(getter)]
    pub fn is_phrase(&self) -> bool {
        self.is_phrase
    }

    #[bridge(getter)]
    pub fn target_language(&self) -> Language {
        self.target_language
    }

    #[bridge(getter)]
    pub fn prefix(&self) -> Option<WordPrefix> {
        self.prefix.clone()
    }

    #[bridge(getter)]
    pub fn senses(&self) -> Vec<DictionarySense> {
        self.senses.clone()
    }

    /// A short word-level summary, taking the first meaning of up to two senses.
    #[bridge(getter)]
    pub fn gloss(&self) -> String {
        self.senses
            .iter()
            .filter_map(|sense| sense.definition.senses.first())
            .take(2)
            .map(|sense| sense.meaning.as_str())
            .collect::<Vec<_>>()
            .join("; ")
    }

    #[bridge(getter)]
    pub fn audio_request(&self) -> AudioRequest {
        AudioRequest {
            request: TtsRequest {
                text: self.display_text.clone(),
                language: self.target_language,
                is_ssml: false,
                instructions: None,
                speed: 1.0,
                verification_hints: Vec::new(),
            },
            provider: TtsProvider::Google,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Context, DeckState};
    use language_utils::{
        ConsolidatedLanguageData, Course, DictionaryEntry, GramFrequencyEntry, GramFrequencyList,
        GramVocabEntry, Heteronym, PartOfSpeech, PronunciationData, TaggedGram, TargetToNativeWord,
        Word, language_pack::LanguagePack,
    };
    use std::{collections::BTreeMap, num::NonZeroU32, sync::Arc};

    // Entirely synthetic: these tests must never load the regenerating packs on disk.
    fn deck() -> Deck {
        let rows: &[(&str, u32, &[&str])] = &[
            ("bank", 3, &[]),
            ("bank", 2, &["finance", "credit", "lender"]),
            ("river", 0, &["water beside a bank"]),
            ("bank", 1, &["shore", "slope"]),
            ("banked", 0, &["deposited"]),
            ("embankment", 0, &["wall"]),
            ("missing", 0, &[]),
        ];
        let gram = |word: &str| {
            Gram(vec![Atom::Tok(Word {
                text: word.into(),
                word_type: WordType::Heteronym(Heteronym {
                    word: word.into(),
                    lemma: word.into(),
                    pos: PartOfSpeech::Noun,
                }),
            })])
        };
        let tagged = |word: &str, sense| TaggedGram {
            gram: gram(word),
            sense: NonZeroU32::new(sense),
        };
        let course = Course {
            target_language: Language::English,
            native_language: Language::French,
        };
        let pack = LanguagePack::new(
            ConsolidatedLanguageData {
                target_language_sentences: vec![],
                translations: vec![],
                nlp_sentences: vec![],
                phrasebook: BTreeMap::new(),
                proper_noun_definitions: BTreeMap::new(),
                source_gram_frequencies: FxHashMap::default(),
                word_to_pronunciation: vec![],
                pronunciation_to_words: vec![],
                minimal_pairs: vec![],
                pronunciation_data: PronunciationData {
                    sounds: vec![],
                    guides: vec![],
                    pattern_frequencies: vec![],
                },
                homophone_practice: BTreeMap::new(),
                movies: FxHashMap::default(),
                books: FxHashMap::default(),
                sentence_sources: vec![],
                gram_vocabulary: ["bank", "river", "banked", "embankment", "missing"]
                    .into_iter()
                    .map(|word| GramVocabEntry {
                        atoms: gram(word),
                        frequency: 10,
                    })
                    .collect(),
                gram_frequencies: GramFrequencyList {
                    entries: rows
                        .iter()
                        .map(|(word, sense, _)| GramFrequencyEntry {
                            count: 10,
                            direct_count: 10,
                            disambiguation_key: 0,
                            gram: tagged(word, *sense),
                        })
                        .collect(),
                    total_count: 70,
                },
                encoded_sentences: vec![],
                gram_dictionary: rows
                    .iter()
                    .filter(|(_, _, meanings)| !meanings.is_empty())
                    .map(|(word, sense, meanings)| {
                        (
                            tagged(word, *sense),
                            DictionaryEntry {
                                target_language_word: (*word).into(),
                                definitions: meanings
                                    .iter()
                                    .map(|meaning| TargetToNativeWord {
                                        native: (*meaning).into(),
                                        note: None,
                                        example_sentence_target_language: String::new(),
                                        example_sentence_native_language: String::new(),
                                        cognate: false,
                                        false_cognate: false,
                                    })
                                    .collect(),
                                morphology: vec![],
                                segments: vec![],
                            },
                        )
                    })
                    .collect(),
                morphemes: BTreeMap::new(),
                human_audio: FxHashMap::default(),
                pronunciation_audio: FxHashMap::default(),
            },
            course,
        );
        let context = Context {
            study_goal: None,
            language_pack: Arc::new(pack),
            course,
            timezone: chrono::FixedOffset::east_opt(0).unwrap(),
        };
        <Deck as weapon::AppState>::finalize(DeckState::new(), &context)
    }

    #[test]
    fn pages_and_lookup_group_defined_senses_by_word() {
        let deck = deck();
        assert_eq!(deck.get_gram_dictionary_count(), 4);
        let indices = |entries: Vec<DictionaryWord>| {
            entries
                .iter()
                .map(DictionaryWord::frequency_index)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            indices(deck.get_gram_dictionary_entries(None, 100)),
            [1, 2, 4, 5]
        );
        assert_eq!(indices(deck.get_gram_dictionary_page(None, 1, 2)), [2, 4]);
        assert!(
            deck.get_gram_dictionary_page(None, usize::MAX, 1)
                .is_empty()
        );
        assert!(deck.get_gram_dictionary_page(None, 0, 0).is_empty());
        for index in [0, 1, 3] {
            let word = deck.gram_dictionary_entry(index).unwrap();
            assert_eq!(word.frequency_index(), 1);
            assert_eq!(word.display_text(), "bank");
            assert_eq!(
                word.senses
                    .iter()
                    .map(|sense| sense.frequency_index)
                    .collect::<Vec<_>>(),
                [1, 3]
            );
            assert_eq!(word.senses[0].gloss, "finance; credit");
            assert_eq!(word.gloss(), "finance; shore");
        }
        assert!(deck.gram_dictionary_entry(6).is_none());
        assert!(deck.gram_dictionary_entry(usize::MAX).is_none());
    }

    #[test]
    fn search_matches_any_sense_but_keeps_the_primary_index() {
        let deck = deck();
        let words = deck.get_gram_dictionary_entries(Some("SHÔRE".into()), 10);
        assert_eq!(words.len(), 1);
        assert_eq!(words[0].frequency_index(), 1);
        assert_eq!(words[0].senses.len(), 2);
        let words = deck.get_gram_dictionary_entries(Some("bank".into()), 10);
        assert_eq!(
            words
                .iter()
                .map(DictionaryWord::frequency_index)
                .collect::<Vec<_>>(),
            [1, 4, 2, 5]
        );
        assert!(
            deck.get_gram_dictionary_entries(Some("absent".into()), 10)
                .is_empty()
        );
    }

    #[test]
    fn adding_a_sense_leaves_its_siblings_unadded() {
        let deck = deck();
        let event = deck.add_gram_by_frequency_index(3).unwrap();
        let context = deck.context.clone();
        let event = weapon::data_model::Timestamped {
            timestamp: chrono::Utc::now(),
            within_device_events_index: 0,
            timezone: Some(context.timezone),
            event,
        };
        let state =
            <Deck as weapon::AppState>::process_event(DeckState::from(deck), &context, &event);
        let deck = <Deck as weapon::AppState>::finalize(state, &context);
        let word = deck.gram_dictionary_entry(1).unwrap();
        assert!(!word.senses[0].is_in_deck);
        assert!(word.senses[1].is_in_deck);
    }
}

#[cfg(test)]
mod translation_sense_tests {
    use super::*;
    use crate::{CardData, Context, DeckEvent, DeckState};
    use language_utils::{
        ConsolidatedLanguageData, Course, DictionaryEntry, GramFrequencyEntry, GramFrequencyList,
        GramVocabEntry, Heteronym, MultiwordTermMatch, PartOfSpeech, SentenceGram, SentenceGrams,
        TaggedGram, TargetToNativeWord, Word,
        autograde::{LiteralGrades, Remembered},
        language_pack::LanguagePack,
    };
    use std::{num::NonZeroU32, sync::Arc};

    fn gram(text: &str, sense: u32) -> TaggedGram<Gram<String>> {
        TaggedGram {
            gram: Gram(
                text.split_whitespace()
                    .map(|word| {
                        Atom::Tok(Word {
                            text: word.into(),
                            word_type: WordType::Heteronym(Heteronym {
                                word: word.into(),
                                lemma: word.into(),
                                pos: PartOfSpeech::Noun,
                            }),
                        })
                    })
                    .collect(),
            ),
            sense: NonZeroU32::new(sense),
        }
    }

    fn deck() -> Deck {
        // Sense 1 is deliberately first in the inventory; sense 2 must never
        // silently become sense 1 when grading an occurrence or an inner word.
        let entries = [
            gram("bank", 1),
            gram("bank", 2),
            gram("river", 0),
            gram("river bank", 0),
        ];
        let encoded = |grams, multiword_terms| SentenceGrams {
            grams,
            capitalize_first: true,
            multiword_terms,
            low_confidence_multiword_terms: vec![],
        };
        let sentences: Vec<(String, SentenceGrams<TaggedGram<Gram<String>>>)> = vec![
            (
                "Bank".into(),
                encoded(vec![SentenceGram::Learnable(gram("bank", 2))], vec![]),
            ),
            (
                "Bank bank".into(),
                encoded(
                    vec![
                        SentenceGram::Learnable(gram("bank", 1)),
                        SentenceGram::Learnable(gram("bank", 2)),
                    ],
                    vec![],
                ),
            ),
            (
                "River bank".into(),
                encoded(vec![SentenceGram::Learnable(gram("river bank", 0))], vec![]),
            ),
            (
                "Bank river bank".into(),
                encoded(
                    vec![
                        SentenceGram::Learnable(gram("bank", 1)),
                        SentenceGram::Learnable(gram("river", 0)),
                        SentenceGram::Learnable(gram("bank", 2)),
                    ],
                    vec![MultiwordTermMatch {
                        gram: gram("river bank", 0),
                        matched_word_indices: vec![1, 2],
                    }],
                ),
            ),
        ];
        let course = Course {
            target_language: Language::English,
            native_language: Language::French,
        };
        let data = ConsolidatedLanguageData {
            target_language_sentences: sentences.iter().map(|(text, _)| text.clone()).collect(),
            translations: sentences
                .iter()
                .map(|(text, _)| (text.clone(), vec!["Une traduction".into()]))
                .collect(),
            encoded_sentences: sentences,
            gram_vocabulary: entries
                .iter()
                .map(|g| GramVocabEntry {
                    atoms: g.gram.clone(),
                    frequency: 10,
                })
                .collect(),
            gram_frequencies: GramFrequencyList {
                entries: entries
                    .iter()
                    .map(|g| GramFrequencyEntry {
                        gram: g.clone(),
                        count: if *g == gram("bank", 1) { 30 } else { 10 },
                        direct_count: if *g == gram("bank", 1) { 30 } else { 10 },
                        disambiguation_key: 0,
                    })
                    .collect(),
                total_count: 60,
            },
            gram_dictionary: entries
                .iter()
                .map(|g| {
                    (
                        g.clone(),
                        DictionaryEntry {
                            target_language_word: g.to_display_string(Language::English),
                            definitions: vec![TargetToNativeWord {
                                native: "meaning".into(),
                                note: None,
                                example_sentence_target_language: String::new(),
                                example_sentence_native_language: String::new(),
                                cognate: false,
                                false_cognate: false,
                            }],
                            morphology: vec![],
                            segments: vec![],
                        },
                    )
                })
                .collect(),
            ..Default::default()
        };
        let context = Context {
            study_goal: None,
            language_pack: Arc::new(LanguagePack::new(data, course)),
            course,
            timezone: chrono::FixedOffset::east_opt(0).unwrap(),
        };
        <Deck as weapon::AppState>::finalize(DeckState::new(), &context)
    }

    fn replay(deck: Deck, event: DeckEvent) -> Deck {
        let context = deck.context.clone();
        let event = weapon::data_model::Timestamped {
            timestamp: chrono::Utc::now(),
            within_device_events_index: 0,
            timezone: Some(context.timezone),
            event,
        };
        let state =
            <Deck as weapon::AppState>::process_event(DeckState::from(deck), &context, &event);
        <Deck as weapon::AppState>::finalize(state, &context)
    }

    fn indicator(
        deck: &Deck,
        text: &str,
        sense: u32,
    ) -> CardIndicator<language_utils::SpurGram, lasso::Spur> {
        CardIndicator::WrittenGram {
            gram: deck
                .context
                .language_pack
                .resolve_entry(&gram(text, sense))
                .unwrap(),
        }
    }

    fn hold(mut deck: Deck, text: &str, sense: u32) -> Deck {
        let CardIndicator::WrittenGram { gram: entry } = indicator(&deck, text, sense) else {
            unreachable!()
        };
        let index = deck
            .context
            .language_pack
            .gram_frequencies
            .entries
            .get_index_of(&entry)
            .unwrap();
        let event = deck.add_gram_by_frequency_index(index).unwrap();
        deck = replay(deck, event);
        let event = deck
            .review_card(
                CardIndicator::WrittenGram {
                    gram: gram(text, sense),
                },
                crate::Rating::Remembered,
            )
            .unwrap();
        replay(deck, event)
    }

    fn card(deck: &Deck, text: &str, sense: u32) -> rs_fsrs::Card {
        match &deck.cards[&indicator(deck, text, sense)] {
            CardData::Added { fsrs_card } | CardData::Ghost { fsrs_card } => fsrs_card.clone(),
        }
    }

    fn grade(deck: Deck, sentence: &str, hints: Vec<usize>) -> Deck {
        let literals = deck
            .context
            .language_pack
            .sentence_to_literals(
                &deck
                    .context
                    .language_pack
                    .string_rodeo
                    .get(sentence)
                    .unwrap(),
                Language::English,
            )
            .unwrap();
        let event = deck
            .translate_sentence_wrong(
                sentence.into(),
                "translation".into(),
                LiteralGrades(vec![Some(Remembered::Remembered); literals.len()]),
                hints,
                vec![],
                vec![],
            )
            .unwrap();
        replay(deck, event)
    }

    #[test]
    fn exact_sentence_sense_updates_only_its_card() {
        let deck = hold(hold(deck(), "bank", 1), "bank", 2);
        assert_eq!(
            deck.context.language_pack.resolve_entry(&gram("bank", 0)),
            deck.context.language_pack.resolve_entry(&gram("bank", 1))
        );
        let before = [card(&deck, "bank", 1), card(&deck, "bank", 2)];
        let deck = grade(deck, "Bank", vec![]);
        assert_eq!(card(&deck, "bank", 1).reps, before[0].reps);
        assert_eq!(card(&deck, "bank", 2).reps, before[1].reps + 1);
    }

    #[test]
    fn inner_word_uses_unique_held_sense_but_skips_ambiguity() {
        for held_senses in [vec![], vec![2], vec![1, 2]] {
            let mut deck = deck();
            for &sense in &held_senses {
                deck = hold(deck, "bank", sense);
            }
            deck = hold(deck, "river bank", 0);
            let before_unit = card(&deck, "river bank", 0).reps;
            let before: Vec<_> = held_senses
                .iter()
                .map(|&sense| card(&deck, "bank", sense).reps)
                .collect();
            let deck = grade(deck, "River bank", vec![]);
            assert_eq!(card(&deck, "river bank", 0).reps, before_unit + 1);
            // The single-sense inventory is safe even without an Added card.
            assert_eq!(card(&deck, "river", 0).reps, 1);
            for (&sense, reps) in held_senses.iter().zip(before) {
                assert_eq!(
                    card(&deck, "bank", sense).reps,
                    reps + i32::from(held_senses.len() == 1)
                );
            }
            if held_senses.len() < 2 {
                assert!(!deck.cards.contains_key(&indicator(&deck, "bank", 1)));
            }
            if held_senses.is_empty() {
                assert!(!deck.cards.contains_key(&indicator(&deck, "bank", 2)));
            }
        }
    }

    #[test]
    fn repeated_word_hint_only_fails_the_tapped_sense() {
        for perfect in [false, true] {
            let deck = hold(hold(deck(), "bank", 1), "bank", 2);
            let before = [card(&deck, "bank", 1), card(&deck, "bank", 2)];
            let deck = if perfect {
                let event = deck
                    .translate_sentence_perfect(vec![0], "Bank bank".into())
                    .unwrap();
                replay(deck, event)
            } else {
                grade(deck, "Bank bank", vec![0])
            };
            assert_eq!(card(&deck, "bank", 1).lapses, before[0].lapses + 1);
            assert_eq!(card(&deck, "bank", 2).lapses, before[1].lapses);
            assert_eq!(card(&deck, "bank", 2).reps, before[1].reps + 1);
        }
    }

    #[test]
    fn primary_positions_use_tagged_occurrences_and_phrase_matches() {
        let deck = deck();
        for (sentence, text, sense, indices) in [
            ("Bank bank", "bank", 2, vec![1]),
            ("River bank", "river bank", 0, vec![0, 1]),
            ("Bank river bank", "river bank", 0, vec![1, 2]),
        ] {
            let pack = &deck.context.language_pack;
            let challenge = deck
                .translation_challenge_for_sentence(
                    pack.resolve_entry(&gram(text, sense)).unwrap(),
                    pack.string_rodeo.get(sentence).unwrap(),
                )
                .unwrap();
            assert_eq!(challenge.primary_literal_indices, indices);
        }
    }
}
