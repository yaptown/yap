//! The two lists behind Stats' word counts: the cards a learner added, and the
//! words Yap counts as known without one. They answer different questions, so
//! each is its own screen rather than a filter over one list.
use language_utils::{Language, features::WordPrefix, readings::RubySegment};
use serde::{Deserialize, Serialize};
use yap_frontend_reducers::definition_view;

use crate::{
    AudioRequest, CardData, CardSummary, Deck, FrequencyKnowledgePoint,
    dictionary::word_audio_request,
};

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
    pub pagination: Option<PaginationView>,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssumedKnownScreenView {
    pub target_language: Language,
    pub summary: WordListSummaryView,
    pub note: String,
    pub frequency_knowledge_chart_title: String,
    pub frequency_knowledge_chart_data: Vec<FrequencyKnowledgePoint>,
    /// Most frequent first.
    pub words: Vec<AssumedKnownWord>,
    pub pagination: Option<PaginationView>,
}

/// One meaning Yap counts as known without a card.
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssumedKnownWord {
    /// Pass to `add_gram_by_frequency_index` to start learning this meaning.
    pub frequency_index: usize,
    /// Position in the pack's frequency list, which counts each sense and
    /// hidden gram, so it runs a little higher than a count of distinct words.
    pub frequency_rank_label: String,
    pub frequency_rank_accessibility_label: String,
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
        let rank = yap_i18n::format_count(frequency_index as u32 + 1);
        Some(AssumedKnownWord {
            frequency_index,
            frequency_rank_label: format!("#{rank}"),
            frequency_rank_accessibility_label: yap_i18n::tr!(
                "assumed-known-frequency-rank",
                rank = rank
            ),
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

const PAGE_SIZE: usize = 200;

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaginationOption {
    pub page: u32,
    pub label: String,
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaginationView {
    /// Zero-based, clamped to the available pages.
    pub page: u32,
    pub page_count: u32,
    pub previous: Option<u32>,
    pub next: Option<u32>,
    pub previous_label: String,
    pub next_label: String,
    pub status: String,
    pub options: Vec<PaginationOption>,
}

fn pagination(total: usize, page: u32) -> (std::ops::Range<usize>, Option<PaginationView>) {
    let page_count = total.div_ceil(PAGE_SIZE).max(1) as u32;
    let page = page.min(page_count - 1);
    let start = page as usize * PAGE_SIZE;
    let range = start..(start + PAGE_SIZE).min(total);
    let view = (page_count > 1).then(|| PaginationView {
        page,
        page_count,
        previous: page.checked_sub(1),
        next: (page + 1 < page_count).then_some(page + 1),
        previous_label: yap_i18n::tr!("pagination-previous"),
        next_label: yap_i18n::tr!("pagination-next"),
        status: yap_i18n::tr!(
            "pagination-status",
            page = yap_i18n::format_count(page + 1),
            pages = yap_i18n::format_count(page_count)
        ),
        options: (0..page_count)
            .map(|page| {
                let first = page * PAGE_SIZE as u32 + 1;
                let last = ((page + 1) * PAGE_SIZE as u32).min(total as u32);
                PaginationOption {
                    page,
                    label: if first == last {
                        yap_i18n::format_count(first)
                    } else {
                        format!(
                            "{}–{}",
                            yap_i18n::format_count(first),
                            yap_i18n::format_count(last)
                        )
                    },
                }
            })
            .collect(),
    });
    (range, view)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pagination_ranges_and_clamping() {
        assert_eq!(pagination(0, 0).0, 0..0);
        assert!(pagination(200, 8).1.is_none());
        let (range, view) = pagination(401, u32::MAX);
        assert_eq!(range, 400..401);
        let view = view.unwrap();
        assert_eq!((view.page, view.page_count), (2, 3));
        assert_eq!(
            view.options
                .iter()
                .map(|o| o.label.as_str())
                .collect::<Vec<_>>(),
            ["1–200", "201–400", "401"]
        );
        assert_eq!(pagination(401, 0).0, 0..200);
        assert_eq!(pagination(401, 1).0, 200..400);
    }
}

#[bridgerton::bridge]
impl Deck {
    /// A page of added cards, soonest due first.
    #[bridgerton::stable]
    pub fn your_cards_view(&self, page: u32) -> YourCardsScreenView {
        let mut cards: Vec<CardSummary> = self
            .cards
            .iter()
            .filter_map(|(indicator, card)| self.card_to_summary(indicator, card))
            .collect();
        let total = cards.len();
        cards.sort_by(|a, b| a.due_timestamp_ms.total_cmp(&b.due_timestamp_ms));
        let (range, pagination) = pagination(total, page);
        let cards = cards.drain(range).collect();
        YourCardsScreenView {
            target_language: self.get_target_language(),
            summary: self.your_cards_summary(total),
            pagination,
            cards,
        }
    }

    /// A page of assumed-known meanings, most frequent first.
    #[bridgerton::stable]
    pub fn assumed_known_view(&self, page: u32) -> AssumedKnownScreenView {
        let senses = self.assumed_known_senses();
        let (range, pagination) = pagination(senses.len(), page);
        let words: Vec<_> = senses[range]
            .iter()
            .filter_map(|&index| self.assumed_known_word(index))
            .collect();
        AssumedKnownScreenView {
            target_language: self.get_target_language(),
            summary: self.assumed_known_summary(senses.len()),
            note: yap_i18n::tr!("assumed-known-note"),
            frequency_knowledge_chart_title: yap_i18n::tr!("stats-word-knowledge-by-frequency"),
            frequency_knowledge_chart_data: self.get_frequency_knowledge_chart_data(),
            pagination,
            words,
        }
    }
}
