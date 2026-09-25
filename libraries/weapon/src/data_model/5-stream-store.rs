//! A trait for EventStreamStores, that allows type erasure

use std::{
    any::Any,
    collections::{BTreeMap, HashMap},
};

use crate::data_model::{EventStreamStore, MaybeSend, Timestamped, ValidToAddEvents};
use std::hash::Hash;

pub trait StreamStore<Device>: Any + MaybeSend {
    fn num_events_per_device(&self) -> HashMap<&Device, usize>;

    fn num_events(&self) -> usize {
        self.num_events_per_device().values().sum()
    }

    /// The device's events with `within_device_events_index >= from_index`, in index order —
    /// i.e. the ones a peer holding `from_index` of this device's events hasn't seen. Not a
    /// positional skip: events are stored in timestamp order, and a backdated event (e.g.
    /// `add_raw_event_at` with a past time) sorts before ones already persisted or uploaded.
    fn jsons(&self, device: &Device, from_index: usize) -> Vec<Timestamped<serde_json::Value>>;

    fn valid_to_add_event_jsons(
        &self,
        device: &Device,
        events: Vec<Timestamped<serde_json::Value>>,
    ) -> Option<ValidToAddEvents<Timestamped<serde_json::Value>>>;

    fn add_device_event_jsons(
        &mut self,
        device: Device,
        events: ValidToAddEvents<Timestamped<serde_json::Value>>,
    ) -> Result<usize, serde_json::Error>;

    fn timestamp_of_earliest_unsynced_event(
        &self,
        sync_state: &BTreeMap<Device, usize>,
    ) -> Option<chrono::DateTime<chrono::Utc>>;
}

/// Implementation for stores that hold versioned events (which are Serialize + DeserializeOwned).
impl<
    Device: Ord + Eq + Clone + Hash + MaybeSend + 'static,
    VersionedEvent: Ord + Clone + serde::Serialize + serde::de::DeserializeOwned + MaybeSend + 'static,
> StreamStore<Device> for EventStreamStore<Device, Timestamped<VersionedEvent>>
{
    fn num_events_per_device(&self) -> HashMap<&Device, usize> {
        self.events()
            .iter()
            .map(|(device, events)| (device, events.len()))
            .collect::<HashMap<&Device, usize>>()
    }

    fn jsons(&self, device: &Device, from_index: usize) -> Vec<Timestamped<serde_json::Value>> {
        let mut events: Vec<_> = self
            .events()
            .get(device)
            .into_iter()
            .flatten()
            .filter(|event| event.within_device_events_index >= from_index)
            .collect();
        events.sort_by_key(|event| event.within_device_events_index);
        events
            .into_iter()
            .map(|event| {
                event
                    .as_ref()
                    .map(|event| serde_json::to_value(event).unwrap())
            })
            .collect()
    }

    fn valid_to_add_event_jsons(
        &self,
        device: &Device,
        events: Vec<Timestamped<serde_json::Value>>,
    ) -> Option<ValidToAddEvents<Timestamped<serde_json::Value>>> {
        self.valid_to_add_events(device, events)
    }

    fn add_device_event_jsons(
        &mut self,
        device: Device,
        events: ValidToAddEvents<Timestamped<serde_json::Value>>,
    ) -> Result<usize, serde_json::Error> {
        let events = events.try_map(|event| {
            serde_json::from_value::<VersionedEvent>(event.clone()).inspect_err(|e| {
                log::error!("Error deserializing event JSON into event type: {e:?} in `{event}`");
            })
        })?;
        Ok(self.add_device_events(device, events))
    }

    fn timestamp_of_earliest_unsynced_event(
        &self,
        sync_state: &BTreeMap<Device, usize>,
    ) -> Option<chrono::DateTime<chrono::Utc>> {
        let mut earliest: Option<_> = None;
        for (device_id, events_set) in self.events() {
            let synced_count = sync_state.get(device_id).copied().unwrap_or(0);

            if events_set.len() > synced_count
                && let Some(ev) = events_set
                    .iter()
                    .find(|e| e.within_device_events_index == synced_count)
            {
                let candidate = ev.timestamp;
                earliest = match earliest {
                    None => Some(candidate),
                    Some(current) => Some(current.min(candidate)),
                };
            }
        }
        earliest
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(time: i64, index: usize, value: i32) -> Timestamped<i32> {
        Timestamped {
            timestamp: chrono::DateTime::from_timestamp(time, 0).unwrap(),
            timezone: chrono::FixedOffset::east_opt(0).unwrap(),
            within_device_events_index: index,
            event: value,
        }
    }

    #[test]
    fn jsons_selects_by_index_not_timestamp_position() {
        let mut stream = EventStreamStore::<&str, Timestamped<i32>>::default();
        stream.add_event_unchecked("a", event(20, 0, 0));
        // Written after index 0 was persisted, but backdated before it.
        stream.add_event_unchecked("a", event(10, 1, 1));
        let indices = |from| {
            stream
                .jsons(&"a", from)
                .iter()
                .map(|event| event.within_device_events_index)
                .collect::<Vec<_>>()
        };
        assert_eq!(indices(0), [0, 1]);
        assert_eq!(indices(1), [1]);
        assert_eq!(indices(2), [] as [usize; 0]);
    }
}
