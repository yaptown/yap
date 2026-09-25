//! # Timestamped
//! Events created by a single device must be sequential. In other words, a device should never "forget" about an event.
//! To guarantee this, we store the `within_device_events_index` of each event. This is a monotonically increasing number that is unique within a device.
//! The nth event created by a device has a `within_device_events_index` of n.
//!
//! Events must also be able to be put in order across devices. This is enabled via the `timestamp` field.
//!
//! Alongside the UTC `timestamp`, we record the user's `timezone` (offset from UTC) at the moment
//! the event was created. This lets us reason about local time-of-day, streaks, and behavior
//! patterns without losing information to UTC normalization. Events serialized before this field
//! existed (a missing or null timezone) are treated as UTC.

/// Store a `chrono::FixedOffset` as offset-from-UTC seconds (`i32`).
/// Missing or null offsets from events predating the field deserialize as UTC.
mod timezone_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn utc() -> chrono::FixedOffset {
        chrono::FixedOffset::east_opt(0).unwrap()
    }

    pub fn serialize<S: Serializer>(
        tz: &chrono::FixedOffset,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        tz.local_minus_utc().serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<chrono::FixedOffset, D::Error> {
        let seconds = Option::<i32>::deserialize(deserializer)?.unwrap_or_default();
        chrono::FixedOffset::east_opt(seconds)
            .ok_or_else(|| serde::de::Error::custom("invalid timezone offset"))
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Timestamped<E> {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub within_device_events_index: usize,
    /// The user's timezone when the event was created, stored as offset-from-UTC seconds.
    /// Missing or null values from events predating this field are treated as UTC.
    #[serde(default = "timezone_serde::utc", with = "timezone_serde")]
    pub timezone: chrono::FixedOffset,
    pub event: E,
}

/// Ordering ignores `timezone`: events are ordered by `(timestamp, within_device_events_index,
/// event)`, matching the pre-timezone behavior. `FixedOffset` carries no ordering meaning here
/// and including it would only break ties between otherwise-identical events.
impl<E: Ord> Ord for Timestamped<E> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (
            &self.timestamp,
            &self.within_device_events_index,
            &self.event,
        )
            .cmp(&(
                &other.timestamp,
                &other.within_device_events_index,
                &other.event,
            ))
    }
}

impl<E: Ord> PartialOrd for Timestamped<E> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<E> Timestamped<E> {
    pub fn map<G, F: Fn(E) -> G>(self, f: F) -> Timestamped<G> {
        Timestamped {
            timestamp: self.timestamp,
            within_device_events_index: self.within_device_events_index,
            timezone: self.timezone,
            event: f(self.event),
        }
    }

    pub fn as_ref(&self) -> Timestamped<&E> {
        Timestamped {
            timestamp: self.timestamp,
            within_device_events_index: self.within_device_events_index,
            timezone: self.timezone,
            event: &self.event,
        }
    }

    pub fn map_ref<G, F: Fn(&E) -> G>(&self, f: F) -> Timestamped<G> {
        Timestamped {
            timestamp: self.timestamp,
            within_device_events_index: self.within_device_events_index,
            timezone: self.timezone,
            event: f(&self.event),
        }
    }
}

impl<E, Error> Timestamped<Result<E, Error>> {
    pub fn transpose(self) -> Result<Timestamped<E>, Error> {
        let Timestamped {
            event,
            timestamp,
            within_device_events_index,
            timezone,
        } = self;
        event.map(|event| Timestamped {
            event,
            timestamp,
            within_device_events_index,
            timezone,
        })
    }
}

impl<E: crate::Event> crate::Event for Timestamped<E> {
    type Versioned = Timestamped<E::Versioned>;
    type Context = E::Context;

    fn to_versioned(&self) -> Self::Versioned {
        self.map_ref(|e| e.to_versioned())
    }

    fn from_versioned(versioned: &Self::Versioned, context: &Self::Context) -> Option<Self> {
        E::from_versioned(&versioned.event, context).map(|event| Timestamped {
            timestamp: versioned.timestamp,
            within_device_events_index: versioned.within_device_events_index,
            timezone: versioned.timezone,
            event,
        })
    }
}

pub trait IndexedEvent {
    fn within_device_events_index(&self) -> usize;
}

impl<E> IndexedEvent for Timestamped<E> {
    fn within_device_events_index(&self) -> usize {
        self.within_device_events_index
    }
}

#[cfg(test)]
mod tests {
    use super::Timestamped;
    use serde_json::json;

    #[test]
    fn timezone_round_trips_and_legacy_offsets_become_utc() {
        for (timezone, seconds) in [(None, 0), (Some(json!(null)), 0), (Some(json!(7200)), 7200)] {
            let mut payload = json!({
                "timestamp": "2026-06-01T00:00:00Z",
                "within_device_events_index": 0,
                "event": "test",
            });
            if let Some(timezone) = timezone {
                payload["timezone"] = timezone;
            }
            let event: Timestamped<String> = serde_json::from_value(payload).unwrap();
            assert_eq!(
                event.timezone,
                chrono::FixedOffset::east_opt(seconds).unwrap()
            );

            let serialized = serde_json::to_value(&event).unwrap();
            assert_eq!(serialized["timezone"], json!(seconds));
            assert_eq!(
                serde_json::from_value::<Timestamped<String>>(serialized).unwrap(),
                event
            );
        }
    }

    #[test]
    fn out_of_range_timezone_is_rejected() {
        for seconds in [-86400, 86400] {
            let payload = json!({
                "timestamp": "2026-06-01T00:00:00Z",
                "within_device_events_index": 0,
                "timezone": seconds,
                "event": "test",
            });
            let error = serde_json::from_value::<Timestamped<String>>(payload).unwrap_err();
            assert!(error.to_string().contains("invalid timezone offset"));
        }
    }
}
