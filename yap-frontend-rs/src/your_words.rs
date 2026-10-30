//! The two lists behind Stats' word counts: the cards a learner added, and the
//! words Yap counts as known without one. They answer different questions, so
//! each is its own screen rather than a filter over one list.
use language_utils::{Language, features::WordPrefix, readings::RubySegment};
use serde::{Deserialize, Serialize};
use yap_frontend_reducers::definition_view;

use crate::{AudioRequest, CardData, CardSummary, Deck, dictionary::word_audio_request};

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WordListSummaryView {
    pub title: String,
    pub count: u64,
    pub label: String,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct YourCardsScreenView {
    pub target_language: Language,
    pub summary: WordListSummaryView,
    /// Soonest due first.
    pub cards: Vec<CardSummary>,
    /// The button label while more cards exist than were asked for.
    pub show_more: Option<String>,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssumedKnownScreenView {
    pub target_language: Language,
    pub summary: WordListSummaryView,
    pub note: String,
    /// Most frequent first.
    pub words: Vec<AssumedKnownWord>,
    /// The button label while more words exist than were asked for.
    pub show_more: Option<String>,
}

/// One meaning Yap counts as known without a card.
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssumedKnownWord {
    /// Pass to `add_gram_by_frequency_index` to start learning this meaning.
    pub frequency_index: usize,
    pub display_text: String,
    pub ruby: Vec<RubySegment>,
    pub prefix: Option<WordPrefix>,
    pub gloss: String,
    pub add_label: String,
    pub audio_request: AudioRequest,
}

impl Deck {
    /// The most frequent meaning of each word the regression counts as known
    /// (directly, or through a ghost card from sentence reviews) that the
    /// learner hasn't added, as frequency indices, most frequent first. One
    /// row per spelling: sibling meanings, and grams that display alike (the
    /// two j'ai), often share a gloss and would read as duplicates.
    pub(crate) fn assumed_known_senses(&self) -> Vec<usize> {
        let pack = &self.context.language_pack;
        let language = self.context.course.target_language;
        let mut senses: Vec<usize> = self
            .get_comprehensible_written_grams(false)
            .iter()
            .filter(|sense| {
                pack.gram_definitions.contains_key(sense)
                    && pack.is_visible(*sense, language)
                    && !self.sense_in_deck(*sense)
            })
            .filter_map(|sense| pack.gram_frequencies.entries.get_index_of(&sense))
            .collect();
        senses.sort_unstable();
        let mut spellings = rustc_hash::FxHashSet::default();
        senses.retain(|&index| {
            let (sense, _) = pack.gram_frequencies.entries.get_index(index).unwrap();
            spellings.insert(pack.resolve_gram(&sense.gram).to_display_string(language))
        });
        senses
    }

    fn assumed_known_word(&self, frequency_index: usize) -> Option<AssumedKnownWord> {
        let pack = &self.context.language_pack;
        let language = self.context.course.target_language;
        let (sense, _) = pack.gram_frequencies.entries.get_index(frequency_index)?;
        let definition = definition_view(pack.gram_definitions.get(sense)?.clone());
        let display_text = pack.resolve_gram(&sense.gram).to_display_string(language);
        Some(AssumedKnownWord {
            frequency_index,
            ruby: crate::ruby::word_ruby(
                &display_text,
                pack.readings_for_sense(*sense).next(),
                language,
                &pack.ruby_segmentations,
            ),
            prefix: pack.word_prefix(sense.gram, language),
            gloss: definition
                .senses
                .iter()
                .take(2)
                .map(|sense| sense.meaning.as_str())
                .collect::<Vec<_>>()
                .join("; "),
            add_label: yap_i18n::tr!("dictionary-screen-add-to-deck"),
            audio_request: word_audio_request(&display_text, language),
            display_text,
        })
    }

    pub(crate) fn added_card_count(&self) -> usize {
        self.cards
            .values()
            .filter(|card| matches!(card, CardData::Added { .. }))
            .count()
    }

    pub(crate) fn your_cards_summary(&self, count: usize) -> WordListSummaryView {
        WordListSummaryView {
            title: yap_i18n::tr!("your-cards-title"),
            count: count as u64,
            label: yap_i18n::tr!(
                "your-cards-summary",
                count = count,
                number = yap_i18n::format_count(count as u32)
            ),
        }
    }

    pub(crate) fn assumed_known_summary(&self, count: usize) -> WordListSummaryView {
        WordListSummaryView {
            title: yap_i18n::tr!("assumed-known-title"),
            count: count as u64,
            label: yap_i18n::tr!(
                "assumed-known-summary",
                count = count,
                number = yap_i18n::format_count(count as u32)
            ),
        }
    }
}

fn show_more(shown: usize, total: usize) -> Option<String> {
    (shown < total).then(|| yap_i18n::tr!("common-show-more"))
}

#[bridgerton::bridge]
impl Deck {
    /// The first `limit` added cards; hosts raise the limit to show more.
    #[bridgerton::stable]
    pub fn your_cards_view(&self, limit: u32) -> YourCardsScreenView {
        let mut cards: Vec<CardSummary> = self
            .cards
            .iter()
            .filter_map(|(indicator, card)| self.card_to_summary(indicator, card))
            .collect();
        let total = cards.len();
        cards.sort_by(|a, b| a.due_timestamp_ms.total_cmp(&b.due_timestamp_ms));
        cards.truncate(limit as usize);
        YourCardsScreenView {
            target_language: self.get_target_language(),
            summary: self.your_cards_summary(total),
            show_more: show_more(cards.len(), total),
            cards,
        }
    }

    /// The first `limit` assumed-known meanings; hosts raise the limit to show more.
    #[bridgerton::stable]
    pub fn assumed_known_view(&self, limit: u32) -> AssumedKnownScreenView {
        let senses = self.assumed_known_senses();
        let words: Vec<_> = senses
            .iter()
            .take(limit as usize)
            .filter_map(|&index| self.assumed_known_word(index))
            .collect();
        AssumedKnownScreenView {
            target_language: self.get_target_language(),
            summary: self.assumed_known_summary(senses.len()),
            note: yap_i18n::tr!("assumed-known-note"),
            show_more: show_more(words.len(), senses.len()),
            words,
        }
    }
}
