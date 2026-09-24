//! Deck event types for tracking user actions and state changes.

pub mod current;
pub mod v1;
pub mod v2;

use crate::Context;
use serde::{Deserialize, Serialize};
use weapon::data_model::Event;

// Re-export current types for convenience
pub use current::*;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
#[allow(private_interfaces)] // old types are not public outside of this module to prevent them from being used in our code
pub enum VersionedDeckEvent {
    V1(v1::DeckEvent),
    V2(v2::DeckEvent),
    V3(current::DeckEvent),
    /// An event tagged with a `version` this build doesn't know about yet — e.g. one written by
    /// a newer client and synced down before this device updated. Carries the untouched JSON so
    /// it round-trips instead of failing to deserialize, which would otherwise block every other
    /// event in the same sync batch (per-device event indices must stay contiguous). Treated as
    /// a no-op by `from_versioned` until a build ships that understands the new version.
    Unknown(String),
}

/// Deserializes by version tag, falling back to `Unknown` for anything but `V1`/`V2`/`V3` so a
/// future event version never fails deserialization outright.
impl<'de> Deserialize<'de> for VersionedDeckEvent {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let value = serde_json::Value::deserialize(deserializer)?;
        Ok(
            match value.get("version").and_then(serde_json::Value::as_str) {
                Some("V1") => {
                    VersionedDeckEvent::V1(serde_json::from_value(value).map_err(D::Error::custom)?)
                }
                Some("V2") => {
                    VersionedDeckEvent::V2(serde_json::from_value(value).map_err(D::Error::custom)?)
                }
                Some("V3") => {
                    VersionedDeckEvent::V3(serde_json::from_value(value).map_err(D::Error::custom)?)
                }
                _ => VersionedDeckEvent::Unknown(value.to_string()),
            },
        )
    }
}

/// Serializes `V1`/`V2`/`V3` the same way the old `#[serde(tag = "version")]` derive did
/// (flattening a `"version"` field into the event's own JSON object), and serializes `Unknown`
/// back to its original JSON so it round-trips losslessly.
impl Serialize for VersionedDeckEvent {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::Error;
        let (version, value) = match self {
            VersionedDeckEvent::V1(event) => ("V1", serde_json::to_value(event)),
            VersionedDeckEvent::V2(event) => ("V2", serde_json::to_value(event)),
            VersionedDeckEvent::V3(event) => ("V3", serde_json::to_value(event)),
            VersionedDeckEvent::Unknown(raw) => {
                return serde_json::from_str::<serde_json::Value>(raw)
                    .map_err(S::Error::custom)?
                    .serialize(serializer);
            }
        };
        let mut value = value.map_err(S::Error::custom)?;
        match &mut value {
            serde_json::Value::Object(map) => {
                map.insert(
                    "version".to_string(),
                    serde_json::Value::String(version.to_string()),
                );
            }
            _ => {
                return Err(S::Error::custom(
                    "expected deck event to serialize to a JSON object",
                ));
            }
        }
        value.serialize(serializer)
    }
}

impl Event for current::DeckEvent {
    type Versioned = VersionedDeckEvent;
    type Context = Context;

    fn to_versioned(&self) -> Self::Versioned {
        VersionedDeckEvent::from(self.clone())
    }

    fn from_versioned(versioned: &Self::Versioned, context: &Self::Context) -> Option<Self> {
        match versioned {
            VersionedDeckEvent::V1(event) => event.clone().into_v2()?.into_v3(context),
            VersionedDeckEvent::V2(event) => event.clone().into_v3(context),
            VersionedDeckEvent::V3(event) => Some(event.clone()),
            // A version this build doesn't understand yet; skip it rather than fail.
            VersionedDeckEvent::Unknown(_) => None,
        }
    }
}

impl From<current::DeckEvent> for VersionedDeckEvent {
    fn from(event: current::DeckEvent) -> Self {
        VersionedDeckEvent::V3(event)
    }
}
