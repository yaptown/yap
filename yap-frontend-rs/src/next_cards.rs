use std::{
    cell::OnceCell,
    cmp::Reverse,
    collections::{BTreeSet, BinaryHeap},
};

use rustc_hash::FxHashMap;

use language_utils::{Atom, Frequency, SpurGram, SpurReadingGram, TaggedGram, Word, grm};
use lasso::Spur;
use ordered_float::NotNan;

use crate::{
    CARD_TYPES, CardData, CardIndicator, CardType, ChallengeRequirements, Deck, SmartAddRegime,
    deck_event::current::SentenceListSelection,
};

/// Partition the top `n` elements to the front (descending by first tuple element),
/// then sort only those. Much faster than a full sort when `n << len`.
fn partial_sort_desc<T: Ord>(values: &mut [(NotNan<f32>, T)], n: usize) {
    let compare =
        |a: &(NotNan<f32>, T), b: &(NotNan<f32>, T)| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1));

    if n == 0 || values.len() <= n {
        values.sort_by(compare);
    } else {
        values.select_nth_unstable_by(n - 1, compare);
        values[..n].sort_by(compare);
    }
}

/// Best reading score per sense, with a unique-sense top-N stopping bound.
struct RankedSenses {
    values: FxHashMap<TaggedGram<SpurGram>, NotNan<f32>>,
    top: BTreeSet<(NotNan<f32>, TaggedGram<SpurGram>)>,
    limit: usize,
}

impl RankedSenses {
    fn new(limit: usize) -> Self {
        Self {
            values: FxHashMap::default(),
            top: BTreeSet::new(),
            limit,
        }
    }

    fn saturated(&self, upper_bound: NotNan<f32>) -> bool {
        self.top.len() == self.limit
            && self
                .top
                .first()
                .is_some_and(|(score, _)| upper_bound < *score)
    }

    fn offer(&mut self, sense: TaggedGram<SpurGram>, score: NotNan<f32>) {
        if let Some(&previous) = self.values.get(&sense) {
            if previous >= score {
                return;
            }
            self.top.remove(&(previous, sense));
        }
        self.values.insert(sense, score);
        self.top.insert((score, sense));
        if self.top.len() > self.limit {
            self.top.pop_first();
        }
    }

    fn into_values(self) -> Vec<(NotNan<f32>, TaggedGram<SpurGram>)> {
        self.values
            .into_iter()
            .map(|(sense, score)| (score, sense))
            .collect()
    }
}

/// Returns the single word from a gram if it has exactly one Tok atom.
/// Returns None for multi-word grams or empty grams.
fn gram_single_word(gram: &grm<Spur>) -> Option<&Word<Spur>> {
    let mut words = gram.iter().filter_map(|atom| match atom {
        Atom::Tok(word) => Some(word),
        Atom::Control(_) => None,
    });
    let first = words.next()?;
    if words.next().is_some() {
        None // More than one word
    } else {
        Some(first)
    }
}

/// Share exactly the same source order between eager text and lazy listening.
fn gram_source<'a>(
    deck: &'a Deck,
    sentence_list: &Option<SentenceListSelection>,
) -> impl Iterator<Item = (&'a SpurReadingGram, Frequency)> {
    let context = &deck.context;
    // Determine which frequency list to use based on sentence_list
    let sentence_list_freq_list = sentence_list.as_ref().and_then(|g| {
        let source_id = match g {
            SentenceListSelection::Movie { id } => {
                language_utils::FrequencySourceId::Movie(id.clone())
            }
            SentenceListSelection::PimsleurLesson { level, lesson } => {
                language_utils::FrequencySourceId::PimsleurLesson(language_utils::PimsleurLesson {
                    level: *level,
                    lesson: *lesson,
                })
            }
        };
        context
            .language_pack
            .sentences
            .source_frequencies(&source_id)
    });

    match sentence_list_freq_list {
        Some(list) => itertools::Either::Left(list.resolved(&context.language_pack)),
        None => itertools::Either::Right(
            context
                .language_pack
                .gram_frequencies
                .resolved(&context.language_pack)
                .map(|(gram, frequency)| (gram, *frequency)),
        ),
    }
}

pub(crate) struct NextCardsIterator<'a> {
    /// The original snapshot supplies scoring and comprehensibility, even when
    /// a pool is first needed after this iterator has selected some cards.
    deck: &'a Deck,
    sentence_list: Option<SentenceListSelection>,
    sorted_prefix_len: usize,
    /// This batch's additions; the deck stays borrowed and untouched.
    selected: BTreeSet<CardIndicator<SpurGram, Spur>>,
    pub(crate) allowed_cards: AllowedCards,
    // Cached counts to avoid repeated iteration
    added_count: usize,
    card_type_counts: FxHashMap<CardType, u32>,
    // Precomputed sorted lists (value desc)
    text_values: Vec<(NotNan<f32>, TaggedGram<SpurGram>)>,
    /// Single-word grams sorted by value descending (only if added_count < 20).
    /// Single-word is a hard onboarding constraint, but the order *within* the
    /// constraint set respects the regression so the placement test influences
    /// onboarding instead of being silently overridden by frequency order.
    single_word_grams: Option<Vec<(NotNan<f32>, TaggedGram<SpurGram>)>>,
    /// Easy single-word grams sorted by value descending (only if added_count < 5
    /// and !teaches_new_writing_system). Same reasoning as `single_word_grams`.
    easy_single_word_grams: Option<Vec<(NotNan<f32>, TaggedGram<SpurGram>)>>,
    listening_values: OnceCell<Vec<(NotNan<f32>, SpurGram)>>,
    pronunciation_values: Vec<(NotNan<f32>, CardIndicator<SpurGram, Spur>)>,
}

#[derive(Clone, Debug)]
pub(crate) enum AllowedCards {
    #[cfg_attr(not(test), expect(unused))]
    // All is not yet used, but could be used to express intent more clearly than an empty BannedRequirements set
    All,
    BannedRequirements(BTreeSet<ChallengeRequirements>),
    Type(CardType),
}

impl<'a> NextCardsIterator<'a> {
    pub fn new(
        deck: &'a Deck,
        allowed_cards: AllowedCards,
        sentence_list: &Option<SentenceListSelection>,
        limit: usize,
    ) -> Self {
        // Buffer beyond the requested limit to account for cards split across types
        // (text vs listening vs pronunciation) and the iterator's balancing logic.
        let sorted_prefix_len = (limit * 3).max(100);

        let cards = &*deck.cards;
        let context = &deck.context;
        let regressions = &deck.regressions;

        // Initialize counts by iterating once over tracked cards. Ghosts are
        // excluded — onboarding thresholds measure deliberate adds, not words
        // the user happened to encounter through challenge sentences.
        let mut added_count = 0;
        let mut card_type_counts: FxHashMap<CardType, u32> =
            CARD_TYPES.iter().map(|card_type| (*card_type, 0)).collect();

        for (card, card_data) in cards.iter() {
            if !matches!(card_data, CardData::Added { .. }) {
                continue;
            }
            added_count += 1;
            let card_type = card.card_type();
            card_type_counts
                .entry(card_type)
                .and_modify(|count| *count += 1);
        }

        let permits = |kind: CardType| match &allowed_cards {
            AllowedCards::All => true,
            AllowedCards::BannedRequirements(banned) => !banned.contains(&kind.challenge_type()),
            AllowedCards::Type(allowed) => *allowed == kind,
        };
        let need_text = permits(CardType::TargetLanguage);

        // Score each reading before collapsing to the sense-keyed card. A known
        // easy reading must not hide an unknown harder reading of the same sense.
        // Deduplicate before top-N selection so readings cannot crowd out cards.
        let need_single_word = need_text && added_count < 20;
        let need_easy_single_word =
            need_text && added_count < 5 && !context.course.teaches_new_writing_system();
        let mut text = RankedSenses::new(sorted_prefix_len);
        let mut single = RankedSenses::new(sorted_prefix_len);
        let mut easy = RankedSenses::new(sorted_prefix_len);
        for (reading, frequency) in
            gram_source(deck, sentence_list).take(if need_text { usize::MAX } else { 0 })
        {
            // Lists descend by count. A reading's value cannot exceed ln(count),
            // so stop only after every active pool has enough UNIQUE senses above
            // every remaining reading's upper bound.
            let upper_bound = NotNan::new(frequency.frequency_score()).unwrap_or_default();
            let text_done = text.saturated(upper_bound);
            let single_done = !need_single_word || single.saturated(upper_bound);
            let easy_done = !need_easy_single_word || easy.saturated(upper_bound);
            if text_done && single_done && easy_done {
                break;
            }
            let card = CardIndicator::WrittenGram {
                gram: reading.sense,
            };
            if matches!(cards.get(&card), Some(CardData::Added { .. })) {
                continue;
            }
            let Some(value) =
                context.card_value_with_frequency(&card, cards.get(&card), regressions, frequency)
            else {
                continue;
            };
            if !text_done {
                text.offer(reading.sense, value);
            }
            if (!single_done || !easy_done)
                && gram_single_word(
                    context
                        .language_pack
                        .gram_rodeo
                        .resolve(&reading.sense.gram),
                )
                .is_some()
            {
                if !single_done {
                    single.offer(reading.sense, value);
                }
                if !easy_done && frequency.easy {
                    easy.offer(reading.sense, value);
                }
            }
        }
        let mut text_values = text.into_values();
        let mut single_word_values = single.into_values();
        let mut easy_single_word_values = easy.into_values();

        partial_sort_desc(&mut text_values, sorted_prefix_len);
        let single_word_grams = if need_single_word {
            partial_sort_desc(&mut single_word_values, sorted_prefix_len);
            Some(single_word_values)
        } else {
            None
        };
        let easy_single_word_grams = if need_easy_single_word {
            partial_sort_desc(&mut easy_single_word_values, sorted_prefix_len);
            Some(easy_single_word_values)
        } else {
            None
        };

        // Precompute pronunciation card values
        let mut pronunciation_values: Vec<(NotNan<f32>, CardIndicator<SpurGram, Spur>)> = context
            .language_pack
            .pronunciation_data
            .guides
            .iter()
            .take(if permits(CardType::LetterPronunciation) {
                usize::MAX
            } else {
                0
            })
            .filter_map(|guide| {
                let pattern = context.language_pack.string_rodeo.get(&guide.pattern)?;
                let card = CardIndicator::LetterPronunciation {
                    pattern,
                    position: guide.position,
                };
                if cards.contains_key(&card) {
                    return None;
                }
                let value = context.get_card_value_with_status(&card, None, regressions)?;
                Some((value, card))
            })
            .collect();
        pronunciation_values.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));

        Self {
            deck,
            sentence_list: sentence_list.clone(),
            sorted_prefix_len,
            selected: BTreeSet::new(),
            allowed_cards,
            added_count,
            card_type_counts,
            text_values,
            single_word_grams,
            easy_single_word_grams,
            listening_values: OnceCell::new(),
            pronunciation_values,
        }
    }

    /// Returns true if a card is Already Added (not Ghost, not absent).
    fn is_added(&self, card: &CardIndicator<SpurGram, Spur>) -> bool {
        self.selected.contains(card)
            || matches!(self.deck.cards.get(card), Some(CardData::Added { .. }))
    }

    /// Which pool the next text-card pick will come from. The pools'
    /// existence already encodes the added_count thresholds (the constructor
    /// only builds easy when added_count<5 and single-word when <20), so
    /// regime is just "first non-empty preferred pool, else General."
    pub(crate) fn smart_add_regime(&self) -> SmartAddRegime {
        if let Some(easy) = &self.easy_single_word_grams
            && self.first_unadded_gram(easy).is_some()
        {
            SmartAddRegime::Easy
        } else if let Some(single) = &self.single_word_grams
            && self.first_unadded_gram(single).is_some()
        {
            SmartAddRegime::SingleWord
        } else {
            SmartAddRegime::General
        }
    }

    fn next_text_card(&self) -> Option<CardIndicator<SpurGram, Spur>> {
        let preferred_gram = match self.smart_add_regime() {
            SmartAddRegime::Easy => self
                .easy_single_word_grams
                .as_ref()
                .and_then(|g| self.first_unadded_gram(g)),
            SmartAddRegime::SingleWord => self
                .single_word_grams
                .as_ref()
                .and_then(|g| self.first_unadded_gram(g)),
            SmartAddRegime::General => None,
        };

        let gram = preferred_gram.or_else(|| {
            self.text_values.iter().find_map(|(_, gram)| {
                let card = CardIndicator::WrittenGram { gram: *gram };
                if self.is_added(&card) {
                    None
                } else {
                    Some(*gram)
                }
            })
        });

        gram.map(|gram| CardIndicator::WrittenGram { gram })
    }

    fn first_unadded_gram(
        &self,
        grams: &[(NotNan<f32>, TaggedGram<SpurGram>)],
    ) -> Option<TaggedGram<SpurGram>> {
        grams.iter().find_map(|&(_value, gram)| {
            let card = CardIndicator::WrittenGram { gram };
            if self.is_added(&card) {
                None
            } else {
                Some(gram)
            }
        })
    }

    fn next_letter_pronunciation_card(&self) -> Option<CardIndicator<SpurGram, Spur>> {
        self.pronunciation_values.iter().find_map(|(_, card)| {
            if self.is_added(card) {
                None
            } else {
                Some(*card)
            }
        })
    }

    fn build_listening_values(&self) -> Vec<(NotNan<f32>, SpurGram)> {
        let deck = self.deck;
        let cards = &*deck.cards;
        let context = &deck.context;
        let regressions = &deck.regressions;
        let comprehensible_written = deck.get_comprehensible_written_grams(true);

        // Keep only the sorted prefix, with the same score-desc/gram-asc order
        // as text. Filter and deduplicate in source order BEFORE scoring: the
        // first comprehensible reading of each gram remains authoritative.
        // Unlike text, scores use GLOBAL gram totals, so source frequencies
        // do not bound them; scan the source rather than stopping early.
        let mut listening_seen = std::collections::HashSet::new();
        let mut listening_top = BinaryHeap::new();
        for value in gram_source(deck, &self.sentence_list).filter_map(|(gram, _)| {
            if !comprehensible_written.contains(gram) || !listening_seen.insert(gram.sense.gram) {
                return None;
            }
            let card = CardIndicator::ListeningGram {
                gram: gram.sense.gram,
            };
            if matches!(cards.get(&card), Some(CardData::Added { .. })) {
                return None;
            }
            let value = context.card_value_with_frequency(
                &card,
                cards.get(&card),
                regressions,
                context
                    .language_pack
                    .gram_frequency_total(gram.sense.gram)?,
            )?;
            Some((value, gram.sense.gram))
        }) {
            let candidate = (Reverse(value.0), value.1);
            if listening_top.len() < self.sorted_prefix_len {
                listening_top.push(candidate);
            } else if let Some(mut worst) = listening_top.peek_mut()
                && candidate < *worst
            {
                *worst = candidate;
            }
        }
        listening_top
            .into_sorted_vec()
            .into_iter()
            .map(|(Reverse(score), gram)| (score, gram))
            .collect()
    }

    fn next_listening_card(&self) -> Option<CardIndicator<SpurGram, Spur>> {
        // listening_values is pre-filtered to only comprehensible written grams
        self.listening_values
            .get_or_init(|| self.build_listening_values())
            .iter()
            .find_map(|(_, gram)| {
                let card = CardIndicator::ListeningGram { gram: *gram };
                if self.is_added(&card) {
                    return None;
                }
                Some(card)
            })
    }
}

impl NextCardsIterator<'_> {
    fn next_card(&self) -> Option<CardIndicator<SpurGram, Spur>> {
        if self.added_count < 20 {
            let can_only_add_text_cards = match &self.allowed_cards {
                AllowedCards::All | AllowedCards::Type(CardType::TargetLanguage) => true,
                AllowedCards::BannedRequirements(r) => r.is_empty(),
                _ => false,
            };
            if can_only_add_text_cards {
                let card = self.next_text_card()?;
                return Some(card);
            }
        }

        let total_cards: u32 = self.card_type_counts.values().cloned().sum();
        let next_card_types = {
            let mut card_type_ratios = self
                .card_type_counts
                .iter()
                .filter(|(card_type, _)| match &self.allowed_cards {
                    AllowedCards::All => true,
                    AllowedCards::BannedRequirements(banned_requirements) => {
                        !banned_requirements.contains(&card_type.challenge_type())
                    }
                    AllowedCards::Type(allowed_card_type) => **card_type == *allowed_card_type,
                })
                .map(|(card_type, count)| {
                    (*card_type, {
                        if total_cards == 0 {
                            0.0
                        } else {
                            let target_ratio = match card_type {
                                CardType::TargetLanguage => 0.65,
                                CardType::Listening => 0.33,
                                CardType::LetterPronunciation => 0.02,
                            };
                            (*count as f64 / total_cards as f64) / target_ratio
                        }
                    })
                })
                .collect::<Vec<(CardType, f64)>>();
            card_type_ratios.sort_by_key(|(_, ratio)| NotNan::new(*ratio).unwrap());
            card_type_ratios
                .into_iter()
                .map(|(card_type, _)| card_type)
                .collect::<Vec<_>>()
        };

        // Try to get a card of each type in priority order
        for card_type in next_card_types {
            let card = match card_type {
                CardType::TargetLanguage => self.next_text_card(),
                CardType::Listening => self.next_listening_card(),
                CardType::LetterPronunciation => self.next_letter_pronunciation_card(),
            };
            if let Some(card) = card {
                return Some(card);
            }
        }
        None
    }
}

impl Iterator for NextCardsIterator<'_> {
    type Item = CardIndicator<SpurGram, Spur>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(card) = self.next_card() {
            let card_type = card.card_type();

            self.selected.insert(card);

            self.added_count += 1;
            self.card_type_counts
                .entry(card_type)
                .and_modify(|count| *count += 1);

            Some(card)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn unique_sense_bound_matches_full_scan_after_reading_improvements() {
        use super::{RankedSenses, partial_sort_desc};
        use language_utils::{SpurGram, TaggedGram};
        use lasso::Key;
        use ordered_float::NotNan;
        let candidates = [
            (0, 100., 1.),
            (0, 99., 90.),
            (0, 98., 2.),
            (0, 97., 89.),
            (1, 96., 80.),
            (0, 95., 94.),
            (2, 94., 70.),
            (3, 93., 90.),
            (1, 92., 91.),
            (4, 89., 85.),
            (5, 88., 88.),
            (6, 80., 80.),
        ];
        let mut bounded = RankedSenses::new(3);
        let mut full = RankedSenses::new(3);
        let mut stopped = false;
        for (index, (id, upper, score)) in candidates.into_iter().enumerate() {
            let sense = TaggedGram {
                gram: SpurGram::try_from_usize(id).unwrap(),
                sense: None,
            };
            let score = NotNan::new(score).unwrap();
            full.offer(sense, score);
            if !stopped {
                stopped = bounded.saturated(NotNan::new(upper).unwrap());
                if !stopped {
                    bounded.offer(sense, score);
                }
            }
            if index == 3 {
                assert!(
                    !bounded.saturated(NotNan::new(0.).unwrap()),
                    "four readings are still only one sense"
                );
            }
        }
        assert!(stopped, "exercise the early-stop path");
        let mut bounded = bounded.into_values();
        let mut full = full.into_values();
        assert!(bounded.len() < full.len());
        partial_sort_desc(&mut bounded, 3);
        partial_sort_desc(&mut full, 3);
        assert_eq!(&bounded[..3], &full[..3]);
    }
    use std::collections::BTreeSet;

    use super::{AllowedCards, NextCardsIterator, gram_single_word};
    use crate::{CardIndicator, Deck};

    #[test]
    fn test_first_20_cards_stay_single_word_when_available() {
        let deck = Deck::default();
        if deck
            .context
            .language_pack
            .gram_frequencies
            .entries
            .is_empty()
        {
            return;
        }

        let mut iter = NextCardsIterator::new(
            &deck,
            AllowedCards::BannedRequirements(BTreeSet::new()),
            &None,
            20,
        );
        let Some(single_word_grams) = &iter.single_word_grams else {
            return;
        };
        if single_word_grams.len() < 20 {
            return;
        }

        assert!(iter.listening_values.get().is_none());
        let cards: Vec<_> = iter.by_ref().take(20).collect();
        assert!(iter.listening_values.get().is_none());
        assert_eq!(cards.len(), 20);
        for card in cards {
            let CardIndicator::WrittenGram { gram } = card else {
                panic!("expected written card during early onboarding");
            };
            assert!(
                gram_single_word(deck.context.language_pack.gram_rodeo.resolve(&gram.gram))
                    .is_some()
            );
        }
    }
    #[test]
    fn listening_initializes_on_first_need_from_the_original_snapshot() {
        let deck = Deck::default();
        let mut iter = NextCardsIterator::new(&deck, AllowedCards::All, &None, 30);
        let eager = iter.build_listening_values();
        assert!(iter.listening_values.get().is_none());
        for _ in 0..20 {
            assert!(matches!(
                iter.next(),
                Some(CardIndicator::WrittenGram { .. })
            ));
        }
        assert!(iter.listening_values.get().is_none());
        iter.next_listening_card();
        assert_eq!(iter.listening_values.get().unwrap(), &eager);
    }
}
