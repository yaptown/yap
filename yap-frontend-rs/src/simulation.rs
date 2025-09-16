use crate::Rating;
use crate::{Challenge, Deck, TranscribeComprehensibleSentence, TranslateComprehensibleSentence};
use chrono::{DateTime, Duration, Utc};
use language_utils::transcription_challenge;
use weapon::AppState;
use weapon::data_model::Timestamped;

/// Iterator that simulates daily usage of a deck, yielding all challenges for each day
pub struct DailySimulationIterator {
    deck: Deck,
    current_time: DateTime<Utc>,
    event_index: usize,
}

impl DailySimulationIterator {
    pub fn new(deck: Deck) -> Self {
        Self {
            deck,
            current_time: Utc::now(),
            event_index: 0,
        }
    }
}

impl DailySimulationIterator {
    pub(crate) fn next(mut self) -> (Self, Vec<Challenge<String>>) {
        let mut day_challenges = Vec::new();

        // Process all due reviews for the day
        loop {
            let review_info = self
                .deck
                .get_review_info(vec![], self.current_time.timestamp_millis() as f64);
            if let Some(challenge) = review_info.get_next_challenge(&self.deck) {
                day_challenges.push(challenge.clone());

                // Answer the challenge, marking new flashcards as forgotten once
                let event = match challenge {
                    Challenge::FlashCardReview {
                        indicator, is_new, ..
                    } => {
                        let rating = if is_new {
                            Rating::Again
                        } else {
                            Rating::Remembered
                        };
                        self.deck.review_card(indicator, rating)
                    }
                    Challenge::TranslateComprehensibleSentence(
                        TranslateComprehensibleSentence {
                            target_language, ..
                        },
                    ) => self.deck.translate_sentence_perfect(target_language),
                    Challenge::TranscribeComprehensibleSentence(
                        TranscribeComprehensibleSentence { parts, .. },
                    ) => {
                        let graded = parts
                            .into_iter()
                            .map(|part| match part {
                                transcription_challenge::Part::AskedToTranscribe { parts } => {
                                    let submission = parts
                                        .iter()
                                        .map(|p| p.text.clone())
                                        .collect::<Vec<_>>()
                                        .join(" ");
                                    transcription_challenge::PartGraded::AskedToTranscribe {
                                        submission,
                                        parts: parts
                                            .into_iter()
                                            .map(|p| transcription_challenge::PartGradedPart {
                                                heard: p,
                                                grade:
                                                    transcription_challenge::WordGrade::Perfect {},
                                            })
                                            .collect(),
                                    }
                                }
                                transcription_challenge::Part::Provided { part } => {
                                    transcription_challenge::PartGraded::Provided { part }
                                }
                            })
                            .collect();
                        self.deck.transcribe_sentence(graded)
                    }
                };

                if let Some(event) = event {
                    let ts = Timestamped {
                        timestamp: self.current_time,
                        within_device_events_index: self.event_index,
                        event,
                    };
                    self.deck = self.deck.apply_event(&ts);
                    self.event_index += 1;
                }
            } else {
                break;
            }
        }

        // Add 10 new cards at the end of the day
        if let Some(event) = self.deck.add_next_unknown_cards(None, 10) {
            let ts = Timestamped {
                timestamp: self.current_time,
                within_device_events_index: self.event_index,
                event,
            };
            self.deck = self.deck.apply_event(&ts);
            self.event_index += 1;
        }

        // Advance to next day
        self.current_time += Duration::days(1);

        (self, day_challenges)
    }
}

impl Deck {
    /// Create an iterator that simulates daily usage.
    /// The iterator yields all challenges for each day as a Vec, answering them perfectly,
    /// and adds 10 new cards at the end of each day.
    /// Use .take(n) to limit to n days.
    pub fn simulate_usage(&self) -> DailySimulationIterator {
        DailySimulationIterator::new(self.clone())
    }
}
