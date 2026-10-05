//! The content labels the clip publish puts on each index row
//! (`libraries/subtitle-corpus/src/export/enrichment.rs` writes them).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentRatingLevel {
    None,
    Mild,
    Intense,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentRatings {
    pub profanity: ContentRatingLevel,
    pub horror: ContentRatingLevel,
    pub alcohol_drugs: ContentRatingLevel,
    pub sexual_nudity: ContentRatingLevel,
    pub violence_weapons: ContentRatingLevel,
}

impl ContentRatings {
    /// Explicit content in any category; mild content is allowed.
    pub fn is_explicit(&self) -> bool {
        [
            self.profanity,
            self.horror,
            self.alcohol_drugs,
            self.sexual_nudity,
            self.violence_weapons,
        ]
        .into_iter()
        .any(|level| level == ContentRatingLevel::Intense)
    }

    /// Nothing in any category, so Apple's age-rating answer can be "None".
    pub fn is_all_audiences(&self) -> bool {
        [
            self.profanity,
            self.horror,
            self.alcohol_drugs,
            self.sexual_nudity,
            self.violence_weapons,
        ]
        .into_iter()
        .all(|level| level == ContentRatingLevel::None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_intense_content_is_explicit_in_every_category() {
        for level in [
            ContentRatingLevel::None,
            ContentRatingLevel::Mild,
            ContentRatingLevel::Intense,
        ] {
            for category in 0..5 {
                let mut ratings = ContentRatings {
                    profanity: ContentRatingLevel::None,
                    horror: ContentRatingLevel::None,
                    alcohol_drugs: ContentRatingLevel::None,
                    sexual_nudity: ContentRatingLevel::None,
                    violence_weapons: ContentRatingLevel::None,
                };
                *[
                    &mut ratings.profanity,
                    &mut ratings.horror,
                    &mut ratings.alcohol_drugs,
                    &mut ratings.sexual_nudity,
                    &mut ratings.violence_weapons,
                ][category] = level;
                assert_eq!(ratings.is_explicit(), level == ContentRatingLevel::Intense);
                assert_eq!(
                    ratings.is_all_audiences(),
                    level == ContentRatingLevel::None
                );
            }
        }
    }
}
