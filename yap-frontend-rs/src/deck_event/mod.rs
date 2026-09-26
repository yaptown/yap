//! Deck event types for tracking user actions and state changes.

pub mod current;
pub mod v1;
pub mod v2;
mod v3;

use crate::Context;
use serde::{Deserialize, Serialize};
use weapon::data_model::Event;

// Re-export current types for convenience
pub use current::*;

#[derive(Clone, Debug, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
#[serde(tag = "version")]
#[allow(private_interfaces)] // old types are not public outside of this module to prevent them from being used in our code
pub enum VersionedDeckEvent {
    V1(v1::DeckEvent),
    V2(v2::DeckEvent),
    V3(v3::DeckEvent),
    V4(current::DeckEvent),
}

impl Event for current::DeckEvent {
    type Versioned = VersionedDeckEvent;
    type Context = Context;

    fn to_versioned(&self) -> Self::Versioned {
        VersionedDeckEvent::from(self.clone())
    }

    fn from_versioned(versioned: &Self::Versioned, context: &Self::Context) -> Option<Self> {
        match versioned {
            VersionedDeckEvent::V1(event) => {
                event.clone().into_v2()?.into_v3(context)?.into_v4(context)
            }
            VersionedDeckEvent::V2(event) => event.clone().into_v3(context)?.into_v4(context),
            VersionedDeckEvent::V3(event) => event.clone().into_v4(context),
            VersionedDeckEvent::V4(event) => Some(event.clone()),
        }
    }
}

impl From<current::DeckEvent> for VersionedDeckEvent {
    fn from(event: current::DeckEvent) -> Self {
        VersionedDeckEvent::V4(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use weapon::data_model::EventType;

    #[test]
    fn exported_deck_roundtrips_and_future_content_is_preserved() {
        let json = r#"{"version":"V4","type":"Language","target_language":"French","native_language":"English","content":{"type":"AnkiDeckExported","deck_id":"70270b11-a563-4f91-baa5-f2f5cc28c331","options":{"card_types":"Both","word_cards":false},"taught_words":[{"gram":[],"sense":1}],"sentences":["Bonjour."]}}"#;
        let event: VersionedDeckEvent = serde_json::from_str(json).unwrap();
        assert_eq!(
            serde_json::from_str::<VersionedDeckEvent>(&serde_json::to_string(&event).unwrap())
                .unwrap(),
            event
        );
        assert!(matches!(
            &event,
            VersionedDeckEvent::V4(DeckEvent::Language(LanguageEvent {
                content: LanguageEventContent::AnkiDeckExported { .. },
                ..
            }))
        ));
        for future in [
            json.replace("AnkiDeckExported", "FutureLanguageEvent"),
            json.replace("V4", "V5"),
        ] {
            let wrapped = format!(r#"{{"User":{future}}}"#);
            let old: EventType<VersionedDeckEvent> = serde_json::from_str(&wrapped).unwrap();
            assert!(matches!(old, EventType::Unrecognized(_)));
            assert_eq!(serde_json::to_string(&old).unwrap(), wrapped);
        }
    }
}
