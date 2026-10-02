//! Tracked cards and their per-card projections. Mutation has a single boundary:
//! update the card, then replace its regression and knowledge entries.
use std::{
    cmp::Ordering,
    collections::{BTreeMap, hash_map::Entry},
    ops::Deref,
};

use isotonic::Point;
use language_utils::{SpurGram, TaggedGram};
use lasso::Spur;
use rustc_hash::FxHashMap;

use crate::comprehensible::CardKnowledge;
use crate::{CardData, CardIndicator, Context};

type Indicator = CardIndicator<SpurGram, Spur>;

#[derive(Clone, Copy, Debug)]
struct Ease(f32);
impl PartialEq for Ease {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for Ease {}
impl PartialOrd for Ease {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Ease {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Cards {
    cards: FxHashMap<Indicator, CardData>,
    written_points: BTreeMap<(Ease, Indicator), Point<f32>>,
    listening_points: BTreeMap<(Ease, Indicator), Point<f32>>,
    written_knowledge: FxHashMap<TaggedGram<SpurGram>, CardKnowledge>,
    listening_knowledge: FxHashMap<SpurGram, CardKnowledge>,
}

// Read-only map access; there is deliberately no DerefMut or mutable iterator.
impl Deref for Cards {
    type Target = FxHashMap<Indicator, CardData>;
    fn deref(&self) -> &Self::Target {
        &self.cards
    }
}

impl Cards {
    pub(crate) fn written_knowledge(&self) -> &FxHashMap<TaggedGram<SpurGram>, CardKnowledge> {
        &self.written_knowledge
    }
    pub(crate) fn listening_knowledge(&self) -> &FxHashMap<SpurGram, CardKnowledge> {
        &self.listening_knowledge
    }

    pub(crate) fn modify(
        &mut self,
        context: &Context,
        indicator: Indicator,
        update: impl FnOnce(Entry<'_, Indicator, CardData>),
    ) {
        update(self.cards.entry(indicator));
        let card = self.cards.get(&indicator);
        let knowledge = card.map(CardKnowledge::from);
        let pack = &context.language_pack;
        match indicator {
            CardIndicator::WrittenGram { gram }
                if pack.written_ease_order.rank(&gram).is_some() =>
            {
                if let Some(knowledge) = knowledge {
                    self.written_knowledge.insert(gram, knowledge);
                } else {
                    self.written_knowledge.remove(&gram);
                }
            }
            CardIndicator::ListeningGram { gram }
                if pack.listening_ease_order.rank(&gram).is_some() =>
            {
                if let Some(knowledge) = knowledge {
                    self.listening_knowledge.insert(gram, knowledge);
                } else {
                    self.listening_knowledge.remove(&gram);
                }
            }
            _ => {}
        }
        let points = match indicator {
            CardIndicator::WrittenGram { .. } => &mut self.written_points,
            CardIndicator::ListeningGram { .. } => &mut self.listening_points,
            CardIndicator::LetterPronunciation { .. } => return,
        };
        if let Some(frequency) = context.get_card_frequency(&indicator) {
            let key = (Ease(frequency.ease), indicator);
            if let Some(card) = card
                && !frequency.exclude_from_regression()
                && !card.is_new()
            {
                points.insert(
                    key,
                    Point::new(frequency.ease, card.pre_existing_knowledge()),
                );
            } else {
                points.remove(&key);
            }
        }
    }

    /// Tests seed decks directly; production state only grows through events.
    #[cfg(test)]
    pub(crate) fn insert(&mut self, context: &Context, indicator: Indicator, card: CardData) {
        self.modify(context, indicator, |entry| {
            entry.insert_entry(card);
        });
    }

    pub(crate) fn written_points(&self) -> impl ExactSizeIterator<Item = &Point<f32>> {
        self.written_points.values()
    }

    pub(crate) fn listening_points(&self) -> impl ExactSizeIterator<Item = &Point<f32>> {
        self.listening_points.values()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Deck, DeckEvent, DeckState, Rating};
    use chrono::{Duration, TimeZone, Utc};
    use weapon::{AppState, data_model::Timestamped};

    // Deliberately derive everything from the raw cards, independently of modify.
    fn assert_scratch(deck: &Deck) {
        let pack = &deck.context.language_pack;
        let mut written = Vec::new();
        let mut listening = Vec::new();
        let mut written_knowledge = FxHashMap::default();
        let mut listening_knowledge = FxHashMap::default();
        for (&indicator, card) in deck.cards.iter() {
            let (fsrs, added) = match card {
                CardData::Added { fsrs_card } => (fsrs_card, true),
                CardData::Ghost { fsrs_card } => (fsrs_card, false),
            };
            let knowledge = if fsrs.state == rs_fsrs::State::Review {
                CardKnowledge::Now
            } else if added {
                CardKnowledge::Planned
            } else {
                CardKnowledge::Excluded
            };
            match indicator {
                CardIndicator::WrittenGram { gram }
                    if pack.written_ease_order.rank(&gram).is_some() =>
                {
                    written_knowledge.insert(gram, knowledge);
                }
                CardIndicator::ListeningGram { gram }
                    if pack.listening_ease_order.rank(&gram).is_some() =>
                {
                    listening_knowledge.insert(gram, knowledge);
                }
                _ => {}
            }
            if fsrs.state == rs_fsrs::State::New {
                continue;
            }
            let Some(frequency) = deck.context.get_card_frequency(&indicator) else {
                continue;
            };
            if frequency.exclude_from_regression() {
                continue;
            }
            let point = (
                indicator,
                Point::new(frequency.ease, card.pre_existing_knowledge()),
            );
            match indicator {
                CardIndicator::WrittenGram { .. } => written.push(point),
                CardIndicator::ListeningGram { .. } => listening.push(point),
                _ => {}
            }
        }
        assert_eq!(deck.cards.written_knowledge(), &written_knowledge);
        assert_eq!(deck.cards.listening_knowledge(), &listening_knowledge);
        for (mut expected, actual, regression) in [
            (
                written,
                deck.cards.written_points().copied().collect::<Vec<_>>(),
                deck.regressions.target_language_regression.as_ref(),
            ),
            (
                listening,
                deck.cards.listening_points().copied().collect(),
                deck.regressions.listening_regression.as_ref(),
            ),
        ] {
            expected.sort_by(|a, b| a.1.x().total_cmp(b.1.x()).then(a.0.cmp(&b.0)));
            let mut expected: Vec<_> = expected.into_iter().map(|(_, point)| point).collect();
            assert_eq!(actual, expected);
            let should_fit = expected.len() >= 2 || deck.placement_test_results.is_some();
            assert_eq!(regression.is_some(), should_fit);
            let mut bias = vec![
                Point::new_with_weight(1_f32.ln(), 0.0, 5.0),
                Point::new_with_weight(25_f32.ln(), 0.0, 5.0),
                Point::new_with_weight(64_f32.ln(), 0.0, 5.0),
            ];
            if let Some(placement) = &deck.placement_test_results {
                bias.extend(deck.context.get_placement_test_points(placement));
            } else {
                bias.extend(
                    [
                        (400_f32, 3.0),
                        (800.0, 3.0),
                        (1000.0, 3.0),
                        (1500.0, 3.0),
                        (2000.0, 2.0),
                        (2500.0, 2.0),
                        (3000.0, 2.0),
                        (3500.0, 2.0),
                        (4000.0, 2.0),
                    ]
                    .map(|(frequency, weight)| Point::new_with_weight(frequency.ln(), 0.0, weight)),
                );
            }
            if let Some(regression) = regression {
                expected.extend(bias);
                expected.sort_by(|a, b| a.x().total_cmp(b.x()));
                let window = pack
                    .gram_frequencies
                    .entries
                    .get_index(0)
                    .map_or(1.0, |(_, f)| f.ease * 0.2);
                let scratch = isotonic::SmoothRegression::new_sorted(
                    &expected,
                    isotonic::Direction::Ascending,
                    window,
                );
                for point in expected {
                    assert_eq!(
                        regression.interpolate(*point.x()),
                        scratch.interpolate(*point.x())
                    );
                }
            }
        }
        for planned in [false, true] {
            let written = deck.get_comprehensible_written_grams(planned);
            let bits = written.rank_bitset();
            for (&indicator, card) in deck.cards.iter() {
                let expected = deck.context.is_comprehensible(
                    &indicator,
                    Some(card),
                    &deck.regressions,
                    planned,
                );
                match indicator {
                    CardIndicator::WrittenGram { gram } => {
                        assert_eq!(written.contains(&gram), expected);
                        if let Some(rank) = pack.written_ease_order.rank(&gram) {
                            assert_eq!(bits.contains(rank), expected);
                        }
                    }
                    CardIndicator::ListeningGram { gram } => {
                        for sense in pack.senses_of(gram) {
                            assert_eq!(
                                deck.get_comprehensible_listening_grams(planned)
                                    .contains(sense),
                                expected
                            );
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn apply(deck: Deck, event: DeckEvent, step: &mut u32) -> Deck {
        let context = deck.context.clone();
        *step += 1;
        let state = Deck::process_event(
            deck.into(),
            &context,
            &Timestamped {
                event,
                timestamp: Utc.timestamp_opt(1_700_000_000, 0).unwrap()
                    + Duration::days(i64::from(*step)),
                timezone: context.timezone,
                within_device_events_index: *step as _,
            },
        );
        let deck = Deck::finalize(state, &context);
        assert_scratch(&deck);
        deck
    }

    fn review(deck: Deck, card: Indicator, rating: Rating, step: &mut u32) -> Deck {
        let resolved = card.resolve(
            &deck.context.language_pack.string_rodeo,
            &deck.context.language_pack.gram_rodeo,
        );
        let event = DeckEvent::Language(crate::LanguageEvent {
            target_language: deck.context.course.target_language,
            native_language: deck.context.course.native_language,
            content: crate::LanguageEventContent::ReviewCard {
                reviewed: resolved,
                rating,
            },
        });
        apply(deck, event, step)
    }

    #[test]
    fn incremental_projections_match_scratch_after_each_event() {
        let mut deck = Deck::default();
        let mut step = 0;
        assert_scratch(&deck);
        let grams: Vec<_> = deck
            .context
            .language_pack
            .gram_frequencies
            .entries
            .iter()
            .filter(|(_, f)| !f.exclude_from_regression())
            .map(|(gram, _)| *gram)
            .take(4)
            .collect();
        let cards: Vec<_> = grams
            .iter()
            .flat_map(|&gram| {
                [
                    CardIndicator::WrittenGram { gram },
                    CardIndicator::ListeningGram { gram: gram.gram },
                ]
            })
            .collect();
        let event = deck.cards_to_event(&cards[..4], &None).unwrap();
        deck = apply(deck, event, &mut step);
        for &card in &cards[..4] {
            deck = review(deck, card, Rating::Remembered, &mut step);
            assert_eq!(deck.cards[&card].pre_existing_knowledge(), 1.0);
            deck = review(deck, card, Rating::Again, &mut step);
            assert_eq!(deck.cards[&card].pre_existing_knowledge(), 0.0);
        }
        // Unadded reviews create ghosts; promotion must preserve their evidence.
        for &card in &cards[4..] {
            deck = review(deck, card, Rating::Remembered, &mut step);
            assert!(matches!(deck.cards[&card], CardData::Ghost { .. }));
        }
        deck = review(deck, cards[4], Rating::Again, &mut step);
        let event = deck.cards_to_event(&cards[4..], &None).unwrap();
        deck = apply(deck, event, &mut step);
        assert!(matches!(deck.cards[&cards[4]], CardData::Added { .. }));
        for known in [vec!["le".to_owned(), "être".to_owned()], vec![]] {
            let event = deck.complete_placement_test(known, vec!["chat".to_owned()]);
            deck = apply(deck, event, &mut step);
        }
        // Actual Review/Again cycles reach the leech reset, without seeding FSRS.
        let card = cards[0];
        for _ in 0..40 {
            deck = review(deck, card, Rating::Easy, &mut step);
            deck = review(deck, card, Rating::Again, &mut step);
            if deck.leeches.contains_key(&card) {
                break;
            }
        }
        assert!(deck.leeches.contains_key(&card), "exercise leech reset");
        let CardData::Added { fsrs_card } = &deck.cards[&card] else {
            panic!()
        };
        assert_eq!(fsrs_card.state, rs_fsrs::State::New);
        assert_eq!(
            deck.cards.written_knowledge()[&grams[0]],
            CardKnowledge::Planned
        );
        // Re-finalizing the moved state reuses the same projections.
        let context = deck.context.clone();
        assert_scratch(&Deck::finalize(DeckState::from(deck), &context));
    }
}
