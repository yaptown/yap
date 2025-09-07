use std::collections::{BTreeSet, HashMap};

use language_utils::Lexeme;
use lasso::Spur;

use crate::{CardIndicator, CardStatus, ChallengeType, Context, Deck};

pub(crate) struct NextCardsIterator<'a> {
    pub(crate) cards: HashMap<CardIndicator<Spur>, CardStatus>,
    pub(crate) permitted_types: Vec<ChallengeType>,
    pub(crate) context: &'a Context,
}

impl<'a> NextCardsIterator<'a> {
    pub fn new(deck: &'a Deck, permitted_types: Vec<ChallengeType>) -> Self {
        Self {
            cards: deck.cards.clone(),
            permitted_types,
            context: &deck.context,
        }
    }

    fn next_text_card(&self) -> Option<CardIndicator<Spur>> {
        // None of the first 20 cards can be multiword cards
        let added_over_20_cards = self
            .cards
            .iter()
            .filter(|(_, status)| matches!(status, CardStatus::Added(_)))
            .nth(20)
            .is_some();

        self.cards
            .iter()
            .filter_map(|(card, status)| {
                let CardIndicator::TargetLanguage { lexeme } = card else {
                    return None;
                };
                if !added_over_20_cards && lexeme.multiword().is_some() {
                    return None;
                }

                let value = status.value()?;

                Some((lexeme, value))
            })
            .max_by_key(|(_, value)| *value)
            .map(|(card, _)| CardIndicator::TargetLanguage { lexeme: *card })
    }

    fn next_listening_card(&self) -> Option<CardIndicator<Spur>> {
        // Get all known words (already added text cards)
        let known_words: BTreeSet<Lexeme<Spur>> = self
            .cards
            .iter()
            .filter_map(|(card, status)| {
                if let CardIndicator::TargetLanguage { lexeme } = card {
                    if matches!(status, CardStatus::Added(_)) {
                        Some(*lexeme)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        self.cards
            .iter()
            .filter_map(|(card, status)| {
                let CardIndicator::ListeningHomophonous { pronunciation } = card else {
                    return None;
                };

                let value = status.value()?;

                // Check if we know at least one word with this pronunciation
                let has_known_word = self
                    .context
                    .language_pack
                    .pronunciation_to_lexemes(pronunciation)
                    .any(|(_, lexeme)| known_words.contains(&lexeme));

                // Only include if we know at least one word with this pronunciation
                if !has_known_word {
                    return None;
                }

                Some((pronunciation, value))
            })
            .max_by_key(|(_, value)| *value)
            .map(|(pronunciation, _)| CardIndicator::ListeningHomophonous {
                pronunciation: *pronunciation,
            })
    }
}

impl NextCardsIterator<'_> {
    fn next_card(&self) -> Option<CardIndicator<Spur>> {
        if self.permitted_types.is_empty() {
            return None;
        }

        if self.permitted_types.len() == 1 {
            let card = match self.permitted_types[0] {
                ChallengeType::Text => self.next_text_card(),
                ChallengeType::Listening => self.next_listening_card(),
            }?;
            return Some(card);
        }

        let added_count = self
            .cards
            .iter()
            .filter(|(_, status)| matches!(status, CardStatus::Added(_)))
            .count();

        if added_count < 20 {
            let card = self.next_text_card()?;
            return Some(card);
        }

        let text_count = self
            .cards
            .iter()
            .filter(|(c, status)| {
                matches!(c, CardIndicator::TargetLanguage { .. })
                    && matches!(status, CardStatus::Added(_))
            })
            .count();
        let listening_count = self
            .cards
            .iter()
            .filter(|(c, status)| {
                matches!(c, CardIndicator::ListeningHomophonous { .. })
                    && matches!(status, CardStatus::Added(_))
            })
            .count();

        let desired = if listening_count < text_count / 2 {
            ChallengeType::Listening
        } else {
            ChallengeType::Text
        };

        let other = if desired == ChallengeType::Text {
            ChallengeType::Listening
        } else {
            ChallengeType::Text
        };

        for ty in [desired, other] {
            let card = match ty {
                ChallengeType::Text => self.next_text_card(),
                ChallengeType::Listening => self.next_listening_card(),
            };
            if let Some(card) = card {
                // Mark as added by creating a new CardData with default FSRS card
                return Some(card);
            }
        }
        None
    }
}

impl Iterator for NextCardsIterator<'_> {
    type Item = CardIndicator<Spur>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(card) = self.next_card() {
            self.cards.insert(
                card,
                CardStatus::Added(crate::CardData {
                    fsrs_card: rs_fsrs::Card::new(),
                }),
            );
            Some(card)
        } else {
            None
        }
    }
}
