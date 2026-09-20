use crate::{Challenge, Gram, TranscriptionState, TranslationState};

/// A captured challenge with its live reducer snapshot.
#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ChallengeFixture {
    pub challenge: Challenge<Gram<String>>,
    pub transcription: Option<TranscriptionState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub translation: Option<TranslationState>,
}

#[cfg(any(feature = "fixtures", test))]
#[bridgerton::bridge]
pub fn parse_challenge_fixture(json: String) -> Result<ChallengeFixture, bridgerton::Error> {
    serde_json::from_str(&json).map_err(|error| bridgerton::Error::new(error.to_string()))
}

#[cfg(any(feature = "fixtures", test))]
#[bridgerton::bridge]
pub fn challenge_fixture_json(fixture: ChallengeFixture) -> String {
    serde_json::to_string_pretty(&fixture).expect("challenge fixtures are JSON serializable")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captured_fixtures_round_trip() {
        let directory =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/challenges");
        let mut count = 0;
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path
                .extension()
                .is_some_and(|extension| extension == "json")
            {
                let json = std::fs::read_to_string(path).unwrap();
                let fixture = parse_challenge_fixture(json.clone()).unwrap();
                let encoded = challenge_fixture_json(fixture);
                assert_eq!(
                    serde_json::from_str::<serde_json::Value>(&json).unwrap(),
                    serde_json::from_str::<serde_json::Value>(&encoded).unwrap()
                );
                count += 1;
            }
        }
        assert!(count > 0, "capture at least one challenge fixture");
    }
}
