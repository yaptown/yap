#[cfg(feature = "fixtures")]
use crate::{Challenge, Deck, Gram};
use crate::{
    DueWordsScreenView, GoalsScreenView, HomeScreenView, ReviewScreenView, StatsScreenView,
};
#[cfg(feature = "fixtures")]
use itertools::Itertools;
use serde::{Deserialize, Serialize};

/// A captured screen, shared by both hosts even without the fixture JSON codecs.
#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize)]
#[serde(tag = "screen", content = "view")]
// One fixture is rendered at a time; boxing would only complicate the bindings.
#[allow(clippy::large_enum_variant)]
pub enum Fixture {
    Review(ReviewScreenView),
    Home(HomeScreenView),
    Stats(StatsScreenView),
    Goals(GoalsScreenView),
    Due(DueWordsScreenView),
}

#[cfg(any(feature = "fixtures", test))]
#[bridgerton::bridge]
pub fn parse_fixture(json: String) -> Result<Fixture, bridgerton::Error> {
    serde_json::from_str(&json).map_err(|error| bridgerton::Error::new(error.to_string()))
}

#[cfg(any(feature = "fixtures", test))]
#[bridgerton::bridge]
pub fn fixture_json(fixture: Fixture) -> String {
    serde_json::to_string_pretty(&fixture).expect("screen fixtures are JSON serializable")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captured_fixtures_round_trip() {
        let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures");
        let mut count = 0;
        let mut names = std::collections::HashSet::new();
        for entry in std::fs::read_dir(&directory)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.is_dir())
            .flat_map(|path| std::fs::read_dir(path).unwrap())
        {
            let path = entry.unwrap().path();
            if path
                .extension()
                .is_some_and(|extension| extension == "json")
            {
                assert!(
                    names.insert(path.file_stem().unwrap().to_os_string()),
                    "duplicate fixture name: {}",
                    path.display()
                );
                count += 1;
                let json = std::fs::read_to_string(&path).unwrap();
                let fixture = parse_fixture(json.clone()).unwrap();
                let encoded = fixture_json(fixture);
                assert_eq!(
                    serde_json::from_str::<serde_json::Value>(&json).unwrap(),
                    serde_json::from_str::<serde_json::Value>(&encoded).unwrap()
                );
            }
        }
        assert!(count > 0, "capture at least one screen fixture");
    }
}

/// Capture helpers for the debug driver.
#[cfg(feature = "fixtures")]
#[bridgerton::bridge]
impl Deck {
    /// Any translation challenge this deck can pose right now, regardless of
    /// what's due. Lets the driver capture translation fixtures from a deck
    /// whose schedule wouldn't otherwise offer one.
    pub fn any_translation_challenge(&self) -> Option<Challenge<Gram<String>>> {
        self.get_comprehensible_written_grams(false)
            .iter()
            .sorted()
            .find_map(|gram| {
                let sentence = self.pick_translation_sentence(&gram)?;
                self.translation_challenge_for_sentence(gram, sentence)
            })
            .map(Challenge::TranslateComprehensibleSentence)
    }
}
