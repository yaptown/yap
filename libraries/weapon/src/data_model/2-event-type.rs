//! # EventType
//! For more flexibility, we split events into "User events" and "Meta events".
//! User events are determined by application developer, and will typically be created by user actions.
//! Meta events are reserved for internal use. Currently, there are no meta events.
//! But they will be used for things like naming the device and storing other metadata.

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize)]
pub enum MetaEvent {}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, serde::Serialize)]
pub enum EventType<E> {
    User(E),
    Meta(MetaEvent),
    #[serde(untagged)]
    Unrecognized(RawJson),
}

impl<E> EventType<E> {
    pub fn map<G, F: Fn(E) -> G>(self, f: F) -> EventType<G> {
        match self {
            EventType::User(e) => EventType::User(f(e)),
            EventType::Meta(e) => EventType::Meta(e),
            EventType::Unrecognized(raw) => EventType::Unrecognized(raw),
        }
    }

    pub fn map_ref<G, F: Fn(&E) -> G>(&self, f: F) -> EventType<G> {
        match self {
            EventType::User(e) => EventType::User(f(e)),
            // MetaEvent is uninhabited, so this arm can never be reached.
            EventType::Meta(e) => match *e {},
            EventType::Unrecognized(raw) => EventType::Unrecognized(raw.clone()),
        }
    }
}

impl<E, Error> EventType<Result<E, Error>> {
    pub fn transpose(self) -> Result<EventType<E>, Error> {
        match self {
            EventType::User(e) => e.map(EventType::User),
            EventType::Meta(e) => Ok(EventType::Meta(e)),
            EventType::Unrecognized(raw) => Ok(EventType::Unrecognized(raw)),
        }
    }
}

impl<E: crate::Event> crate::Event for EventType<E> {
    type Versioned = EventType<E::Versioned>;
    type Context = E::Context;

    fn to_versioned(&self) -> Self::Versioned {
        self.map_ref(|e| e.to_versioned())
    }

    fn from_versioned(versioned: &Self::Versioned, context: &Self::Context) -> Option<Self> {
        match versioned {
            EventType::User(v) => E::from_versioned(v, context).map(EventType::User),
            // MetaEvent is uninhabited, so this arm can never be reached.
            EventType::Meta(e) => match *e {},
            EventType::Unrecognized(_) => None,
        }
    }
}

/// Validated JSON retained verbatim, including whitespace and number spelling.
/// Used throughout event transport so unknown events survive disk and sync unchanged.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RawJson(String);

impl RawJson {
    pub fn get(&self) -> &str {
        &self.0
    }

    pub fn from_serializable(value: &impl serde::Serialize) -> Result<Self, serde_json::Error> {
        serde_json::to_string(value).map(Self)
    }
}

impl serde::Serialize for RawJson {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serde_json::value::RawValue::from_string(self.0.clone())
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for RawJson {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = Box::<serde_json::value::RawValue>::deserialize(deserializer)?;
        Ok(Self(raw.get().to_owned()))
    }
}

impl<'de, E: serde::de::DeserializeOwned> serde::Deserialize<'de> for EventType<E> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        enum Known<E> {
            User(E),
            Meta(MetaEvent),
        }
        let raw = RawJson::deserialize(deserializer)?;
        Ok(match serde_json::from_str::<Known<E>>(raw.get()) {
            Ok(Known::User(event)) => Self::User(event),
            Ok(Known::Meta(event)) => Self::Meta(event),
            Err(error) => {
                log::warn!("Preserving unrecognized event: {error}");
                Self::Unrecognized(raw)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opaque_event_roundtrip_is_verbatim_and_upgrade_can_read_it() {
        let json = r#"{ "User": { "Future": [1e2, "a", {"z":0,"a":1}] } }"#;
        let old: EventType<u32> = serde_json::from_str(json).unwrap();
        assert!(matches!(old, EventType::Unrecognized(_)));
        let saved = serde_json::to_string(&old).unwrap();
        assert_eq!(saved, json);
        let upgraded: EventType<serde_json::Value> = serde_json::from_str(&saved).unwrap();
        assert!(matches!(upgraded, EventType::User(_)));
    }

    #[test]
    fn malformed_envelope_still_fails() {
        for json in [
            r#"{"event":{"User":{"Future":1}}}"#,
            r#"{"timestamp":"invalid","within_device_events_index":0,"event":{"Future":1}}"#,
        ] {
            assert!(
                serde_json::from_str::<crate::data_model::Timestamped<EventType<u32>>>(json)
                    .is_err()
            );
        }
    }
}
