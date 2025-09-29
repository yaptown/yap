use std::collections::{BTreeSet, HashMap};

use chrono::Utc;
use language_utils::Lexeme;
use lasso::Spur;

use crate::{CardIndicator, CardStatus, ChallengeType, Context, Deck, Regressions};

pub(crate) struct NextCardsIterator<'a> {
    pub(crate) cards: HashMap<CardIndicator<Spur>, CardStatus>,
    pub(crate) permitted_types: Vec<ChallengeType>,
    pub(crate) context: &'a Context,
    pub(crate) regressions: &'a Regressions,
}

impl<'a> NextCardsIterator<'a> {
    pub fn new(deck: &'a Deck, permitted_types: Vec<ChallengeType>) -> Self {
        Self {
            cards: deck.cards.clone(),
            permitted_types,
            context: &deck.context,
            regressions: &deck.regressions,
        }
    }

    fn next_text_card(&self) -> Option<(CardIndicator<Spur>, rs_fsrs::Card)> {
        // None of the first 20 cards can be multiword cards
        let added_over_20_cards = self
            .cards
            .iter()
            .filter(|(_, status)| matches!(status, CardStatus::Tracked(_)))
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

                status.unadded()?;

                let value =
                    self.context
                        .get_card_value_with_status(card, status, self.regressions)?;

                let fsrs_card = rs_fsrs::Card::new(Utc::now());

                Some((lexeme, fsrs_card, value))
            })
            .max_by_key(|(_, _, value)| *value)
            .map(|(card, fsrs_card, _)| {
                (CardIndicator::TargetLanguage { lexeme: *card }, fsrs_card)
            })
    }

    fn next_letter_pronunciation_card(&self) -> Option<(CardIndicator<Spur>, rs_fsrs::Card)> {
        // Find pronunciation patterns that haven't been added yet
        self.cards
            .iter()
            .filter_map(|(card, status)| {
                let CardIndicator::LetterPronunciation { pattern, position } = card else {
                    return None;
                };

                status.unadded()?;

                let value =
                    self.context
                        .get_card_value_with_status(card, status, self.regressions)?;

                let fsrs_card = rs_fsrs::Card::new(Utc::now());

                Some((pattern, position, fsrs_card, value))
            })
            .max_by_key(|(_, _, _, value)| *value)
            .map(|(pattern, position, fsrs_card, _)| {
                (
                    CardIndicator::LetterPronunciation {
                        pattern: *pattern,
                        position: *position,
                    },
                    fsrs_card,
                )
            })
    }

    fn next_listening_card(&self) -> Option<(CardIndicator<Spur>, rs_fsrs::Card)> {
        // Get all known words (already added text cards)
        let known_words: BTreeSet<Lexeme<Spur>> = self
            .cards
            .iter()
            .filter_map(|(card, status)| {
                if let CardIndicator::TargetLanguage { lexeme } = card {
                    if matches!(status, CardStatus::Tracked(_)) {
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

                status.unadded()?;

                let value =
                    self.context
                        .get_card_value_with_status(card, status, self.regressions)?;

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

                let fsrs_card = rs_fsrs::Card::new(Utc::now());

                Some((pronunciation, fsrs_card, value))
            })
            .max_by_key(|(_, _, value)| *value)
            .map(|(pronunciation, fsrs_card, _)| {
                (
                    CardIndicator::ListeningHomophonous {
                        pronunciation: *pronunciation,
                    },
                    fsrs_card,
                )
            })
    }
}

impl NextCardsIterator<'_> {
    fn next_card(&self) -> Option<(CardIndicator<Spur>, rs_fsrs::Card)> {
        if self.permitted_types.is_empty() {
            return None;
        }

        if self.permitted_types.len() == 1 {
            let card = match self.permitted_types[0] {
                ChallengeType::Text => self.next_text_card(),
                ChallengeType::Listening => self.next_listening_card(),
                ChallengeType::Speaking => self.next_letter_pronunciation_card(),
            }?;
            return Some(card);
        }

        let added_count = self
            .cards
            .iter()
            .filter(|(_, status)| matches!(status, CardStatus::Tracked(_)))
            .count();

        if added_count < 20 {
            let card = self.next_text_card()?;
            return Some(card);
        }

        // Count cards by type
        let mut type_counts = HashMap::new();
        for challenge_type in &self.permitted_types {
            type_counts.insert(*challenge_type, 0);
        }

        for (card, status) in &self.cards {
            if matches!(status, CardStatus::Tracked(_)) {
                let card_type = match card {
                    CardIndicator::TargetLanguage { .. } => ChallengeType::Text,
                    CardIndicator::ListeningHomophonous { .. }
                    | CardIndicator::ListeningLexeme { .. } => ChallengeType::Listening,
                    CardIndicator::LetterPronunciation { .. } => ChallengeType::Speaking,
                };
                if let Some(count) = type_counts.get_mut(&card_type) {
                    *count += 1;
                }
            }
        }

        // Determine desired ratios for card types
        // Text: 60%, Listening: 30%, LetterPronunciation: 10%
        let text_count = type_counts.get(&ChallengeType::Text).copied().unwrap_or(0);
        let listening_count = type_counts
            .get(&ChallengeType::Listening)
            .copied()
            .unwrap_or(0);
        let pronunciation_count = type_counts
            .get(&ChallengeType::Speaking)
            .copied()
            .unwrap_or(0);

        // Calculate which type is most underrepresented based on target ratios
        let total_tracked = text_count + listening_count + pronunciation_count;

        let mut candidates = vec![];

        // Check each permitted type and calculate its priority
        if self.permitted_types.contains(&ChallengeType::Text) {
            let target_ratio = 0.6;
            let current_ratio = if total_tracked > 0 {
                text_count as f64 / total_tracked as f64
            } else {
                0.0
            };
            let priority = target_ratio - current_ratio;
            candidates.push((ChallengeType::Text, priority));
        }

        if self.permitted_types.contains(&ChallengeType::Listening) {
            let target_ratio = 0.3;
            let current_ratio = if total_tracked > 0 {
                listening_count as f64 / total_tracked as f64
            } else {
                0.0
            };
            let priority = target_ratio - current_ratio;
            candidates.push((ChallengeType::Listening, priority));
        }

        if self.permitted_types.contains(&ChallengeType::Speaking) {
            let target_ratio = 0.1;
            let current_ratio = if total_tracked > 0 {
                pronunciation_count as f64 / total_tracked as f64
            } else {
                0.0
            };
            let priority = target_ratio - current_ratio;
            candidates.push((ChallengeType::Speaking, priority));
        }

        // Sort by priority (highest first)
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Try to get a card of each type in priority order
        for (challenge_type, _) in candidates {
            let card = match challenge_type {
                ChallengeType::Text => self.next_text_card(),
                ChallengeType::Listening => self.next_listening_card(),
                ChallengeType::Speaking => self.next_letter_pronunciation_card(),
            };
            if let Some(card) = card {
                return Some(card);
            }
        }
        None
    }
}

impl Iterator for NextCardsIterator<'_> {
    type Item = CardIndicator<Spur>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((card, fsrs_card)) = self.next_card() {
            self.cards.insert(
                card,
                CardStatus::Tracked(crate::CardData::Added { fsrs_card }),
            );
            Some(card)
        } else {
            None
        }
    }
}
