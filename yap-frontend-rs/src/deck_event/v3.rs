//! V3 deck event types.

use std::collections::BTreeSet;

use crate::deck_selection::DailyReviewTarget;
use crate::transcription_challenge;
use language_utils::{Gram, Language, Literal, PatternPosition};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[serde(tag = "type")]
pub enum CardIndicator<G, S> {
    WrittenGram {
        gram: G,
    },
    ListeningGram {
        gram: G,
    },
    LetterPronunciation {
        pattern: S,
        position: PatternPosition,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd)]
pub struct LiteralResult {
    pub remembered: Option<bool>,
    pub hinted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd)]
#[serde(tag = "type")]
pub enum SentenceReviewResult {
    Perfect {
        challenge: String,
        submission: String,
        /// None for Other word types, Some(hinted) for heteronyms
        literals: Vec<(Literal<String>, Option</* hinted */ bool>)>,
    },
    Graded {
        challenge: String,
        submission: String,
        /// None for Other word types, Some(result) for heteronyms
        literals: Vec<(Literal<String>, Option<LiteralResult>)>,
        phrases: Vec<(String, /* remembered */ Option<bool>)>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd)]
pub struct LanguageEvent {
    pub target_language: Language,
    pub native_language: Language,
    pub content: LanguageEventContent,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum Rating {
    Again,
    Remembered,
    Hard,
    Good,
    Easy,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd)]
pub struct PlacementTest {
    pub known_words: Vec<String>,
    pub unknown_words: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd)]
#[serde(tag = "type")]
pub enum SentenceListSelection {
    Movie {
        id: String,
    },
    PimsleurLesson {
        level: u32,
        #[serde(alias = "unit")]
        lesson: u32,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd)]
#[serde(tag = "type")]
pub enum LanguageEventContent {
    CompletePlacementTest {
        results: PlacementTest,
    },
    AddCards {
        cards: Vec<CardIndicator<Gram<String>, String>>,
        #[serde(default, alias = "goal")]
        sentence_list: Option<SentenceListSelection>,
    },
    ReviewCard {
        reviewed: CardIndicator<Gram<String>, String>,
        rating: Rating,
    },
    TranslationChallenge {
        review: SentenceReviewResult,
        #[serde(skip)]
        legacy: LegacyTranslationChallenge,
    },
    TranscriptionChallenge {
        challenge: Vec<transcription_challenge::PartGraded>,
    },
    #[serde(alias = "SetGoal")]
    SetSentenceList {
        #[serde(alias = "goal")]
        sentence_list: Option<SentenceListSelection>,
    },
    SetDailyReviewTarget {
        daily_review_target: DailyReviewTarget,
    },
    /// Lock up every added card EXCEPT the listed ones. Locked cards are
    /// hidden from the review queue (and only from the review queue) until
    /// released via `UnlockCards` or until any review touches them.
    #[serde(alias = "LockDueCardsExcept")]
    LockCardsExcept {
        keep: Vec<CardIndicator<Gram<String>, String>>,
    },
    /// Release the listed cards from lockup.
    UnlockCards {
        cards: Vec<CardIndicator<Gram<String>, String>>,
    },
}

#[derive(PartialEq, Eq, Ord, PartialOrd, Default, Clone, Debug)]
pub struct LegacyTranslationChallenge {
    pub lexemes_remembered: BTreeSet<language_utils::Lexeme<String>>,
    pub lexemes_forgotten: BTreeSet<language_utils::Lexeme<String>>,
    pub heteronyms_needed_hint: BTreeSet<language_utils::Heteronym<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd)]
#[serde(tag = "type")]
pub enum DeckEvent {
    Language(LanguageEvent),
}

impl CardIndicator<Gram<String>, String> {
    fn into_v4(
        self,
        context: &crate::Context,
    ) -> Option<super::current::CardIndicator<Gram<String>, String>> {
        use super::current::CardIndicator as New;
        Some(match self {
            Self::WrittenGram { gram } => {
                let entry = context
                    .language_pack
                    .resolve_entry(&language_utils::TaggedGram { gram, sense: None })?;
                New::WrittenGram {
                    gram: entry.map(|gram| context.language_pack.resolve_gram(&gram)),
                }
            }
            Self::ListeningGram { gram } => New::ListeningGram { gram },
            Self::LetterPronunciation { pattern, position } => {
                New::LetterPronunciation { pattern, position }
            }
        })
    }
}

impl DeckEvent {
    pub(super) fn into_v4(self, context: &crate::Context) -> Option<super::current::DeckEvent> {
        use super::current as new;
        let Self::Language(event) = self;
        let list = |list: Option<SentenceListSelection>| {
            list.map(|list| match list {
                SentenceListSelection::Movie { id } => new::SentenceListSelection::Movie { id },
                SentenceListSelection::PimsleurLesson { level, lesson } => {
                    new::SentenceListSelection::PimsleurLesson { level, lesson }
                }
            })
        };
        let content = match event.content {
            LanguageEventContent::CompletePlacementTest { results } => {
                new::LanguageEventContent::CompletePlacementTest {
                    results: new::PlacementTest {
                        known_words: results.known_words,
                        unknown_words: results.unknown_words,
                    },
                }
            }
            LanguageEventContent::AddCards {
                cards,
                sentence_list,
            } => new::LanguageEventContent::AddCards {
                cards: cards
                    .into_iter()
                    .filter_map(|card| card.into_v4(context))
                    .collect(),
                sentence_list: list(sentence_list),
            },
            LanguageEventContent::ReviewCard { reviewed, rating } => {
                new::LanguageEventContent::ReviewCard {
                    reviewed: reviewed.into_v4(context)?,
                    rating: match rating {
                        Rating::Again => new::Rating::Again,
                        Rating::Remembered => new::Rating::Remembered,
                        Rating::Hard => new::Rating::Hard,
                        Rating::Good => new::Rating::Good,
                        Rating::Easy => new::Rating::Easy,
                    },
                }
            }
            LanguageEventContent::TranslationChallenge { review, legacy } => {
                new::LanguageEventContent::TranslationChallenge {
                    review: match review {
                        SentenceReviewResult::Perfect {
                            challenge,
                            submission,
                            literals,
                        } => new::SentenceReviewResult::Perfect {
                            challenge,
                            submission,
                            literals,
                        },
                        SentenceReviewResult::Graded {
                            challenge,
                            submission,
                            literals,
                            phrases,
                        } => {
                            let pack = &context.language_pack;
                            let cleaned = language_utils::text_cleanup::cleanup_sentence(
                                challenge.clone(),
                                context.course.target_language,
                            );
                            let encoded = pack
                                .string_rodeo
                                .get(&cleaned)
                                .and_then(|sentence| pack.encoded_sentences.get(&sentence));
                            let phrases = phrases
                                .into_iter()
                                .filter_map(|(text, remembered)| {
                                    let encoded = encoded?;
                                    let entry = encoded
                                        .multiword_terms
                                        .iter()
                                        .chain(&encoded.low_confidence_multiword_terms)
                                        .map(|term| &term.gram)
                                        .find(|entry| {
                                            pack.resolve_gram(&entry.gram)
                                                .to_display_string(context.course.target_language)
                                                == text
                                        })?;
                                    Some((entry.map(|gram| pack.resolve_gram(&gram)), remembered))
                                })
                                .collect();
                            new::SentenceReviewResult::Graded {
                                challenge,
                                submission,
                                literals: literals
                                    .into_iter()
                                    .map(|(literal, result)| {
                                        (
                                            literal,
                                            result.map(|r| new::LiteralResult {
                                                remembered: r.remembered,
                                                hinted: r.hinted,
                                            }),
                                        )
                                    })
                                    .collect(),
                                phrases,
                            }
                        }
                    },
                    legacy: new::LegacyTranslationChallenge {
                        lexemes_remembered: legacy.lexemes_remembered,
                        lexemes_forgotten: legacy.lexemes_forgotten,
                        heteronyms_needed_hint: legacy.heteronyms_needed_hint,
                    },
                }
            }
            LanguageEventContent::TranscriptionChallenge { challenge } => {
                new::LanguageEventContent::TranscriptionChallenge { challenge }
            }
            LanguageEventContent::SetSentenceList { sentence_list } => {
                new::LanguageEventContent::SetSentenceList {
                    sentence_list: list(sentence_list),
                }
            }
            LanguageEventContent::SetDailyReviewTarget {
                daily_review_target,
            } => new::LanguageEventContent::SetDailyReviewTarget {
                daily_review_target,
            },
            LanguageEventContent::LockCardsExcept { keep } => {
                new::LanguageEventContent::LockCardsExcept {
                    keep: keep
                        .into_iter()
                        .filter_map(|card| card.into_v4(context))
                        .collect(),
                }
            }
            LanguageEventContent::UnlockCards { cards } => new::LanguageEventContent::UnlockCards {
                cards: cards
                    .into_iter()
                    .filter_map(|card| card.into_v4(context))
                    .collect(),
            },
        };
        Some(new::DeckEvent::Language(new::LanguageEvent {
            target_language: event.target_language,
            native_language: event.native_language,
            content,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use language_utils::language_pack::LanguagePack;
    use language_utils::{
        Atom, ConsolidatedLanguageData, Course, GramFrequencyEntry, GramFrequencyList,
        GramVocabEntry, Heteronym, PartOfSpeech, SentenceGram, SentenceGrams, TaggedGram, Word,
        WordType,
    };
    use std::{num::NonZeroU32, sync::Arc};

    fn context() -> (crate::Context, Gram<String>) {
        let gram = Gram(vec![Atom::Tok(Word {
            text: "bank".into(),
            word_type: WordType::Heteronym(Heteronym {
                word: "bank".into(),
                lemma: "bank".into(),
                pos: PartOfSpeech::Noun,
            }),
        })]);
        let tagged = |sense| TaggedGram {
            gram: gram.clone(),
            sense: NonZeroU32::new(sense),
        };
        let course = Course {
            target_language: Language::English,
            native_language: Language::French,
        };
        let data = ConsolidatedLanguageData {
            gram_vocabulary: vec![GramVocabEntry {
                atoms: gram.clone(),
                frequency: 30,
            }],
            gram_frequencies: GramFrequencyList {
                entries: vec![
                    GramFrequencyEntry {
                        count: 20,
                        direct_count: 20,
                        disambiguation_key: 0,
                        gram: tagged(2),
                    },
                    GramFrequencyEntry {
                        count: 10,
                        direct_count: 10,
                        disambiguation_key: 0,
                        gram: tagged(1),
                    },
                ],
                total_count: 30,
            },
            encoded_sentences: vec![(
                "bank".into(),
                SentenceGrams {
                    grams: vec![SentenceGram::Learnable(tagged(2))],
                    capitalize_first: false,
                    multiword_terms: vec![language_utils::MultiwordTermMatch {
                        gram: tagged(1),
                        matched_word_indices: vec![0],
                    }],
                    low_confidence_multiword_terms: vec![],
                },
            )],
            ..Default::default()
        };
        (
            crate::Context {
                study_goal: None,
                language_pack: Arc::new(LanguagePack::new(data, course)),
                course,
                timezone: chrono::FixedOffset::east_opt(0).unwrap(),
            },
            gram,
        )
    }

    #[test]
    fn written_card_migrates_to_most_frequent_sense() {
        let (context, gram) = context();
        let card = CardIndicator::WrittenGram { gram: gram.clone() }
            .into_v4(&context)
            .unwrap();
        assert_eq!(
            card,
            super::super::current::CardIndicator::WrittenGram {
                gram: TaggedGram {
                    gram,
                    sense: NonZeroU32::new(2)
                }
            }
        );
    }

    #[test]
    fn phrase_migration_cleans_challenge_and_matches_only_overlays() {
        let (context, gram) = context();
        let old = DeckEvent::Language(LanguageEvent {
            target_language: context.course.target_language,
            native_language: context.course.native_language,
            content: LanguageEventContent::TranslationChallenge {
                review: SentenceReviewResult::Graded {
                    challenge: " \tbank\n ".into(),
                    submission: "banque".into(),
                    literals: vec![],
                    phrases: vec![("bank".into(), Some(true)), ("missing".into(), Some(false))],
                },
                legacy: Default::default(),
            },
        });
        let super::super::current::DeckEvent::Language(event) = old.into_v4(&context).unwrap();
        let super::super::current::LanguageEventContent::TranslationChallenge {
            review: super::super::current::SentenceReviewResult::Graded { phrases, .. },
            ..
        } = event.content
        else {
            panic!("expected a graded translation")
        };
        assert_eq!(
            phrases,
            vec![(
                TaggedGram {
                    gram,
                    sense: NonZeroU32::new(1)
                },
                Some(true)
            )]
        );
    }
}
