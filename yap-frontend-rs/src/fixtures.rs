#[cfg(feature = "fixtures")]
use crate::Deck;
use crate::{
    AccomplishmentView, Challenge, Gram, IdleScreenView, TranscriptionState, TranslationState,
};

/// A captured challenge with its live reducer snapshot.
#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ChallengeFixture {
    pub challenge: Challenge<Gram<String>>,
    pub transcription: Option<TranscriptionState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub translation: Option<TranslationState>,
}

/// A captured screen; the nested view keeps screen and challenge tags distinct.
#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "view")]
pub enum Fixture {
    Challenge(Box<ChallengeFixture>),
    Idle(Box<IdleScreenView>),
    Accomplishment(Box<AccomplishmentView>),
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
        for entry in ["challenges", "screens"]
            .into_iter()
            .flat_map(|name| std::fs::read_dir(directory.join(name)).unwrap())
        {
            let path = entry.unwrap().path();
            if path
                .extension()
                .is_some_and(|extension| extension == "json")
            {
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
        assert!(count > 0, "capture at least one challenge fixture");
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
            .find_map(|gram| {
                let sentence = self.pick_translation_sentence(gram)?;
                self.translation_challenge_for_sentence(*gram, sentence)
            })
            .map(Challenge::TranslateComprehensibleSentence)
    }
}
