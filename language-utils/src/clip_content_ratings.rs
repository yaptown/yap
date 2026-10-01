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
