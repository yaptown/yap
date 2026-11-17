//! Typed tool outputs. Every tool's response is built from one of these
//! structs, and its declared MCP `outputSchema` derives from the same type
//! (schemars) — so the schema and the payload cannot drift. Doc comments
//! become schema descriptions the model can read.

use std::collections::BTreeMap;

use chrono::{DateTime, Duration, Utc};
use language_utils::{Gram, Language, TaggedGram};
use schemars::JsonSchema;
use serde::Serialize;
use yap_frontend_rs::{CardSummary, Deck, DefinitionView, REVIEW_LOCKUP_KEEP, VoiceActorInfo};

use crate::server::card_kind;

/// One card as listed by get_due_cards / unlock_cards.
#[derive(Serialize, JsonSchema)]
pub struct CardSummaryOut {
    pub language: Language,
    /// Opaque card object — pass back verbatim to log_review / present_card
    /// / present_translation.
    pub card: serde_json::Value,
    /// The card's display text (the word, or the pronunciation pattern).
    pub text: String,
    /// Extra display context, e.g. a lemma or part of speech.
    pub subtitle: Option<String>,
    /// "written" (recognize the written word), "listening" (normally an
    /// audio card), or "pronunciation" (a letter-sound pattern).
    pub kind: String,
    /// FSRS scheduling state: "New", "Learning", "Review", or "Relearning".
    pub fsrs_state: String,
    /// RFC 3339 timestamp when the card was/is due.
    pub due: Option<String>,
    /// True when the card is set aside in lockup.
    pub locked: bool,
}

/// A count of cards, split by kind and by FSRS state.
#[derive(Serialize, JsonSchema, Default)]
pub struct CardCountsOut {
    pub total: usize,
    /// Keyed by "written", "listening", "pronunciation".
    pub by_kind: BTreeMap<String, usize>,
    /// Keyed by "new", "learning", "review", "relearning".
    pub by_fsrs_state: BTreeMap<String, usize>,
}

impl CardCountsOut {
    fn of<'a>(cards: impl IntoIterator<Item = &'a CardSummary>) -> Self {
        let mut counts = Self::default();
        for card in cards {
            counts.total += 1;
            *counts
                .by_kind
                .entry(card_kind(&card.card_indicator).to_string())
                .or_default() += 1;
            *counts.by_fsrs_state.entry(card.state.clone()).or_default() += 1;
        }
        counts
    }
}

/// Lockup: when a big backlog is due at once, yap keeps a manageable handful
/// of cards active and sets the rest aside. A locked card stays locked until
/// it's released by unlock_cards or reviewed, even once it's due — so locked
/// cards split into ones that are due now (the backlog) and ones that aren't
/// due yet (no different from any future card).
#[derive(Serialize, JsonSchema)]
pub struct LockupOut {
    /// All cards set aside in lockup.
    pub total: usize,
    /// Locked cards that are due now — what unlock_cards releases from.
    pub due_now: CardCountsOut,
    /// Locked cards not due yet; they need no attention until they're due.
    pub not_yet_due: usize,
    /// RFC 3339 due date of the most overdue locked card.
    pub oldest_due: Option<String>,
    /// RFC 3339 time the next not-yet-due locked card becomes due.
    pub next_becomes_due: Option<String>,
    /// The most cards one unlock_cards call releases.
    pub release_batch_size: usize,
}

/// How many cards come due over the coming days. Cumulative and counting
/// locked cards too, so each figure is the full backlog the user would face
/// by then if they stopped reviewing now.
#[derive(Serialize, JsonSchema)]
pub struct DueForecastOut {
    pub due_now: usize,
    pub due_within_1_day: usize,
    pub due_within_7_days: usize,
    pub due_within_30_days: usize,
}

/// The deck's whole schedule, broken down.
#[derive(Serialize, JsonSchema)]
pub struct DeckBreakdownOut {
    /// Every scheduled card in the deck.
    pub all_cards: CardCountsOut,
    /// Due now and in the active queue (what get_due_cards lists).
    pub due_now_active: CardCountsOut,
    pub lockup: LockupOut,
    pub forecast: DueForecastOut,
    /// Cards with many lapses that keep being forgotten.
    pub leeches: usize,
}

impl DeckBreakdownOut {
    pub fn new(deck: &Deck, now: DateTime<Utc>) -> Self {
        let cards = deck.card_summaries_with_lockup(now.timestamp_millis() as f64);
        let now_ms = now.timestamp_millis() as f64;
        let due_by = |horizon: Duration| {
            let cutoff = (now + horizon).timestamp_millis() as f64;
            cards
                .iter()
                .filter(|(card, _)| card.due_timestamp_ms <= cutoff)
                .count()
        };
        let locked_due: Vec<&CardSummary> = cards
            .iter()
            .filter(|(card, locked)| *locked && card.due_timestamp_ms <= now_ms)
            .map(|(card, _)| card)
            .collect();
        let locked_future = || {
            cards
                .iter()
                .filter(|(card, locked)| *locked && card.due_timestamp_ms > now_ms)
        };
        // `cards` is sorted most overdue first.
        let rfc3339 = |card: &CardSummary| {
            DateTime::from_timestamp_millis(card.due_timestamp_ms as i64).map(|d| d.to_rfc3339())
        };

        Self {
            all_cards: CardCountsOut::of(cards.iter().map(|(card, _)| card)),
            due_now_active: CardCountsOut::of(
                cards
                    .iter()
                    .filter(|(card, locked)| !*locked && card.due_timestamp_ms <= now_ms)
                    .map(|(card, _)| card),
            ),
            lockup: LockupOut {
                total: locked_due.len() + locked_future().count(),
                oldest_due: locked_due.first().and_then(|card| rfc3339(card)),
                due_now: CardCountsOut::of(locked_due),
                not_yet_due: locked_future().count(),
                next_becomes_due: locked_future().next().and_then(|(card, _)| rfc3339(card)),
                release_batch_size: REVIEW_LOCKUP_KEEP,
            },
            forecast: DueForecastOut {
                due_now: due_by(Duration::zero()),
                due_within_1_day: due_by(Duration::days(1)),
                due_within_7_days: due_by(Duration::days(7)),
                due_within_30_days: due_by(Duration::days(30)),
            },
            leeches: deck.get_leeches().len(),
        }
    }
}

#[derive(Serialize, JsonSchema)]
pub struct GetDueCardsOut {
    /// Cards due right now in the active queue (may exceed the number shown).
    pub total_due: usize,
    /// How many cards are included in `cards`.
    pub showing: usize,
    pub cards: Vec<CardSummaryOut>,
    /// The active due cards, split by kind and FSRS state.
    pub due_breakdown: CardCountsOut,
    pub lockup: LockupOut,
    pub note: String,
}

#[derive(Serialize, JsonSchema)]
pub struct UnlockCardsOut {
    /// Cards released back into the due queue.
    pub unlocked: Vec<CardSummaryOut>,
    /// Cards still set aside in lockup.
    pub lockup: LockupOut,
    pub note: String,
    /// Whether the unlock event reached the server (present only when an
    /// unlock actually happened).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synced: Option<bool>,
}

#[derive(Serialize, JsonSchema)]
pub struct AddCardsOut {
    /// Display text of each newly added card.
    pub added: Vec<String>,
    /// Requested words that were already in the deck (not re-added).
    pub already_in_deck: Vec<String>,
    pub total_cards_in_deck: usize,
    /// Whether the change reached the server (false = queued locally).
    pub synced: bool,
}

/// One word from search_dictionary, with independently addable senses.
#[derive(Serialize, JsonSchema)]
pub struct DictionaryMatchOut {
    pub language: Language,
    pub display_text: String,
    pub is_phrase: bool,
    pub senses: Vec<DictionarySenseMatchOut>,
}

#[derive(Serialize, JsonSchema)]
pub struct DictionarySenseMatchOut {
    /// The exact sense-tagged gram — pass verbatim with the word's language
    /// to add_cards, get_sentences, or log_review.
    pub gram: TaggedGram<Gram<String>>,
    pub gloss: String,
    /// 1 is the most common sense in the course.
    pub frequency_rank: usize,
    pub in_deck: bool,
    pub definition: DefinitionView,
}

#[derive(Serialize, JsonSchema)]
pub struct SearchDictionaryOut {
    pub query: String,
    pub results: Vec<DictionaryMatchOut>,
    pub note: String,
}

/// An example sentence with translations and provenance.
#[derive(Serialize, JsonSchema)]
pub struct SentenceOut {
    pub text: String,
    pub translations: Vec<String>,
    /// Where the sentence comes from (Anki deck, Tatoeba, a movie, …).
    pub sources: Vec<String>,
}

#[derive(Serialize, JsonSchema)]
pub struct GetSentencesOut {
    /// Display text of the requested gram.
    pub word: String,
    /// Sentences composed only of words the user already knows.
    pub comprehensible_sentences: Vec<SentenceOut>,
    pub total_comprehensible: usize,
    /// Sampled from everything containing the word; may include unknown
    /// words and may lack translations.
    pub other_sentences: Vec<SentenceOut>,
    pub total_containing_word: usize,
    pub note: String,
}

#[derive(Serialize, JsonSchema)]
pub struct TierOut {
    pub name: String,
    /// 1-indexed tier number.
    pub tier: u32,
    /// 1-indexed level within this tier.
    pub level: u32,
    pub total_levels: u32,
    /// Percentage of this tier's words the user knows.
    pub percent_known: f64,
}

#[derive(Serialize, JsonSchema)]
pub struct DayOut {
    /// ISO date (YYYY-MM-DD).
    pub date: Option<String>,
    pub reviews: u32,
    pub time_spent_seconds: u32,
    pub new_cards: u32,
    /// Cards that graduated from Learning to Review that day.
    pub learned_cards: u32,
}

#[derive(Serialize, JsonSchema)]
pub struct GetStatsOut {
    /// e.g. "French for English speakers".
    pub course: String,
    pub daily_streak: u32,
    pub xp: f64,
    pub total_reviews: u64,
    /// RFC 3339 timestamp of the user's first review.
    pub started: Option<String>,
    pub cards: DeckBreakdownOut,
    pub tier: TierOut,
    /// The last 7 days of activity, most recent first.
    pub recent_days: Vec<DayOut>,
}

#[derive(Serialize, JsonSchema)]
pub struct LogReviewOut {
    /// Display text of the reviewed card.
    pub reviewed: String,
    /// The rating that was recorded.
    pub rating: String,
    /// Whether the card was actually due when reviewed.
    pub was_due: bool,
    /// FSRS state after the review.
    pub fsrs_state: Option<String>,
    /// RFC 3339 timestamp when the card is next due.
    pub next_due: Option<String>,
    pub remaining_due: usize,
    pub total_reviews: u64,
    /// Whether the review reached the server (false = queued locally).
    pub synced: bool,
}

/// Per-word grade in a translation, aligned with the sentence's words.
#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum LiteralGradeOut {
    Remembered,
    Forgot,
}

#[derive(Serialize, JsonSchema)]
pub struct GradeTranslationOut {
    /// True when the translation was graded fully correct.
    pub perfect: bool,
    /// The reference translation closest to the user's submission.
    pub correct_translation: Option<String>,
    pub encouragement: Option<String>,
    pub explanation: Option<String>,
    /// Set when AI grading failed and a heuristic fallback was used.
    pub autograding_error: Option<String>,
    /// One entry per word in the sentence; null = not graded.
    pub literal_grades: Vec<Option<LiteralGradeOut>>,
    pub phrases_remembered: Vec<String>,
    pub phrases_forgot: Vec<String>,
    pub remaining_due: usize,
    /// Whether the review reached the server (false = queued locally).
    pub synced: bool,
}

#[derive(Serialize, JsonSchema)]
pub struct GetAudioOut {
    /// The audio clip, base64-encoded. Intended for the review widget, not
    /// for reading in conversation.
    pub audio_base64: String,
    /// e.g. "audio/mpeg" or "audio/ogg".
    pub mime_type: String,
    /// Set when the clip is a human voice-actor recording.
    pub voice_actor: Option<VoiceActorInfo>,
    /// "human_recording" or "tts".
    pub source: String,
}

/// One search hit, per the standard search/fetch contract.
#[derive(Serialize, JsonSchema)]
pub struct SearchResultOut {
    /// Pass to fetch for the full entry.
    pub id: String,
    pub title: String,
    /// Citable public dictionary URL.
    pub url: String,
}

#[derive(Serialize, JsonSchema)]
pub struct SearchOut {
    pub results: Vec<SearchResultOut>,
}

#[derive(Serialize, JsonSchema)]
pub struct FetchMetadataOut {
    pub language: Language,
    pub senses: Vec<FetchSenseOut>,
    /// The most frequent defined sense's rank.
    pub frequency_rank: usize,
    /// True if any sense is in the user's deck.
    pub in_deck: bool,
    pub is_phrase: bool,
}

#[derive(Serialize, JsonSchema)]
pub struct FetchSenseOut {
    /// The exact sense-tagged gram — pass verbatim with metadata.language
    /// to add_cards, get_sentences, or log_review.
    pub gram: TaggedGram<Gram<String>>,
    pub gloss: String,
    pub frequency_rank: usize,
    pub in_deck: bool,
}

#[derive(Serialize, JsonSchema)]
pub struct FetchOut {
    pub id: String,
    pub title: String,
    /// The full entry rendered as readable text.
    pub text: String,
    /// Citable public dictionary URL.
    pub url: String,
    pub metadata: FetchMetadataOut,
}

/// Result of presenting an interactive card widget.
#[derive(Serialize, JsonSchema)]
pub struct PresentOut {
    /// The widget's render payload. Consumed by the review widget iframe,
    /// not meant to be interpreted in conversation; the widget reports the
    /// review outcome back when the user grades the card.
    pub challenge: serde_json::Value,
}
