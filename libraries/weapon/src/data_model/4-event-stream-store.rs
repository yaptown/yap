//! # EventStreamStore
//! Weapon allows multiple "event streams" to be created. Each event stream combines events from all of a user's devices.
//! For example, in a Google Docs-like app, you could have one event stream for each document.
//! (This allows the memory consumption to be constant w.r.t. the number of documents, as only the events for the currently-active document would need to be loaded. Although the active document could still have a lot of events and use a lot of memory that way.)

use std::collections::{BTreeSet, HashMap};
use std::hash::Hash;
use std::ops::Bound::{Excluded, Unbounded};
use std::sync::Arc;

use crate::data_model::{EventType, Timestamped};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EventStreamStore<Device: Eq + Clone + Hash, Event: Ord + Clone> {
    events: HashMap<Device, BTreeSet<Event>>,
    // A new epoch invalidates cached prefixes after an earlier insertion. Never persisted.
    #[serde(skip)]
    fold_epoch: Arc<()>,
}

impl<Device: Eq + Clone + Hash, Event: Ord + Clone> Clone for EventStreamStore<Device, Event> {
    fn clone(&self) -> Self {
        Self {
            events: self.events.clone(),
            fold_epoch: Arc::new(()),
        }
    }
}

/// A reusable folded prefix. Reset to `Default` whenever the fold context changes.
/// Finalization always receives a clone, never the cached partial itself.
pub struct FoldCache<A: crate::AppState> {
    epoch: Option<Arc<()>>,
    last: Option<Timestamped<EventType<<A::Event as crate::Event>::Versioned>>>,
    partial: Option<A::Partial>,
}

impl<A: crate::AppState> Default for FoldCache<A> {
    fn default() -> Self {
        Self {
            epoch: None,
            last: None,
            partial: None,
        }
    }
}

impl<Device: Eq + Clone + Hash, Event: Ord + Clone> EventStreamStore<Device, Event> {
    pub fn events(&self) -> &HashMap<Device, BTreeSet<Event>> {
        &self.events
    }
}

impl<Device: Eq + Hash + Clone, Event: Ord + Clone> Default for EventStreamStore<Device, Event> {
    fn default() -> Self {
        Self {
            events: HashMap::new(),
            fold_epoch: Arc::new(()),
        }
    }
}

impl<Device: Eq + Hash + Clone, Event: Ord + Clone> EventStreamStore<Device, Timestamped<Event>> {
    pub fn len_device(&self, device: &Device) -> usize {
        self.events.get(device).map(|set| set.len()).unwrap_or(0)
    }

    pub(crate) fn valid_to_add_events<A>(
        &self,
        key: &Device,
        mut events: Vec<Timestamped<A>>,
    ) -> Option<ValidToAddEvents<Timestamped<A>>> {
        // Early return if events is empty
        if events.is_empty() {
            return None;
        }

        events.sort_by_key(|event| event.within_device_events_index);

        // Check that there are no gaps in the events
        // Note: this is safe because we know that events is not empty
        for i in 0..events.len() - 1 {
            if events[i].within_device_events_index + 1 != events[i + 1].within_device_events_index
            {
                log::warn!("Gap detected in events");
                return None;
            }
        }

        // Check that the lowest event has the index of the current length
        let expected_index = self.events().get(key).map(BTreeSet::len).unwrap_or(0);
        if events[0].within_device_events_index != expected_index {
            log::warn!(
                "Event out of order - expected index {}, got {}",
                expected_index,
                events[0].within_device_events_index
            );
            return None;
        }

        Some(ValidToAddEvents { events })
    }

    pub(crate) fn add_device_events(
        &mut self,
        key: Device,
        events: ValidToAddEvents<Timestamped<Event>>,
    ) -> usize {
        let mut events_added = 0;

        // double check the events are still valid
        let Some(events) = self.valid_to_add_events(&key, events.events) else {
            return events_added;
        };

        for event in events.events {
            events_added += 1;
            self.add_event_unchecked(key.clone(), event);
        }

        events_added
    }
}

impl<K: Eq + Hash + Clone, T: Ord + Clone> EventStreamStore<K, T> {
    /// Add an event without checking if it's out of order.
    #[allow(unused)]
    pub fn add_event_unchecked(&mut self, key: K, event: T) {
        // A new device can rehash the map and change the order of equal events.
        if !self.events.contains_key(&key)
            || self
                .events
                .values()
                .filter_map(|events| events.last())
                .max()
                .is_some_and(|last| &event <= last)
        {
            self.fold_epoch = Arc::new(());
        }
        self.events.entry(key).or_default().insert(event);
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.iter_after(None)
    }

    fn iter_after<'a>(&'a self, last: Option<&'a T>) -> impl Iterator<Item = &'a T> {
        // Seek each device directly to the first event beyond the folded prefix.
        let mut iters: Vec<_> = self
            .events
            .values()
            .map(|set| {
                set.range((last.map_or(Unbounded, Excluded), Unbounded))
                    .peekable()
            })
            .collect();

        // Use a custom iterator that performs a k-way merge
        std::iter::from_fn(move || {
            // Find the iterator with the smallest current element
            let mut min_idx = None;
            let mut min_val = None;

            for (idx, iter) in iters.iter_mut().enumerate() {
                if let Some(val) = iter.peek()
                    && (min_val.is_none() || val < min_val.unwrap())
                {
                    min_idx = Some(idx);
                    min_val = Some(val);
                }
            }

            // Advance the iterator that had the minimum value
            if let Some(idx) = min_idx {
                iters[idx].next()
            } else {
                None
            }
        })
    }

    pub fn num_events(&self) -> usize {
        self.events.values().map(BTreeSet::len).sum()
    }

    #[allow(unused)]
    pub fn map<U: Ord + Clone>(self, f: impl Fn(T) -> U + Clone) -> EventStreamStore<K, U> {
        EventStreamStore {
            fold_epoch: Arc::new(()),
            events: self
                .events
                .into_iter()
                .map(|(k, vs)| (k, vs.into_iter().map(f.clone()).collect::<BTreeSet<U>>()))
                .collect(),
        }
    }
}

/// Compute application state from a store that holds versioned events.
impl<Device: Eq + Hash + Clone, VersionedEvent: Ord + Clone>
    EventStreamStore<Device, Timestamped<EventType<VersionedEvent>>>
{
    /// Fold only the newly appended suffix, or refold after an out-of-order insertion.
    /// The caller must reset `cache` when context or initial state changes.
    pub fn state_cached<CurrentEvent, A>(
        &self,
        cache: &mut FoldCache<A>,
        initial_state: impl FnOnce() -> A::Partial,
        context: &CurrentEvent::Context,
    ) -> A
    where
        CurrentEvent: crate::Event<Versioned = VersionedEvent>,
        A: crate::AppState<Event = CurrentEvent>,
        A::Partial: Clone,
    {
        if !cache
            .epoch
            .as_ref()
            .is_some_and(|epoch| Arc::ptr_eq(epoch, &self.fold_epoch))
        {
            *cache = FoldCache::default();
        }
        let mut last = cache.last.clone();
        let events = self
            .iter_after(cache.last.as_ref())
            .inspect(|event| last = Some((*event).clone()));
        let partial = fold_events::<CurrentEvent, A>(
            events,
            cache.partial.take().unwrap_or_else(initial_state),
            context,
        );
        cache.last = last;
        cache.epoch = Some(self.fold_epoch.clone());
        let state = A::finalize(partial.clone(), context);
        cache.partial = Some(partial);
        state
    }

    /// Compute application state by applying stored events.
    /// The `CurrentEvent` type is what events are converted to for processing.
    /// The store holds `CurrentEvent::Versioned` (which should equal `VersionedEvent`).
    pub fn state<CurrentEvent, A>(
        &self,
        initial_state: A::Partial,
        context: &CurrentEvent::Context,
    ) -> A
    where
        CurrentEvent: crate::Event<Versioned = VersionedEvent>,
        A: crate::AppState<Event = CurrentEvent>,
    {
        apply_events_and_metaevents::<CurrentEvent, A>(self.iter(), initial_state, context)
    }
}

/// Apply versioned events by converting them to current form during processing.
/// Events that return None from from_versioned are skipped (e.g., no-op migrations).
/// Context is passed to from_versioned for conversions that need external data.
pub(crate) fn apply_events_and_metaevents<'a, E, A>(
    events: impl Iterator<Item = &'a Timestamped<EventType<E::Versioned>>>,
    initial_state: A::Partial,
    context: &E::Context,
) -> A
where
    E: crate::data_model::Event + 'a,
    E::Versioned: 'a,
    A: crate::AppState<Event = E>,
{
    A::finalize(fold_events::<E, A>(events, initial_state, context), context)
}

fn fold_events<'a, E, A>(
    events: impl Iterator<Item = &'a Timestamped<EventType<E::Versioned>>>,
    initial_state: A::Partial,
    context: &E::Context,
) -> A::Partial
where
    E: crate::data_model::Event + 'a,
    E::Versioned: 'a,
    A: crate::AppState<Event = E>,
{
    let mut state = initial_state;

    // Process events one at a time, passing context to from_versioned
    for event in events {
        match event {
            Timestamped {
                event: EventType::User(versioned_event),
                timestamp,
                within_device_events_index,
                timezone,
            } => {
                // Convert from versioned, passing context
                if let Some(current_event) = E::from_versioned(versioned_event, context) {
                    let timestamped = Timestamped {
                        event: current_event,
                        timestamp: *timestamp,
                        within_device_events_index: *within_device_events_index,
                        timezone: *timezone,
                    };
                    state = A::process_event(state, context, &timestamped);
                }
            }
            Timestamped {
                event: EventType::Unrecognized(_),
                ..
            } => {}
            Timestamped {
                event: EventType::Meta(event),
                ..
            } => match *event {},
        }
    }

    state
}

pub struct ValidToAddEvents<Event> {
    events: Vec<Event>,
}

impl<Event> ValidToAddEvents<Timestamped<Event>> {
    pub(crate) fn try_map<A, Error, F: Fn(Event) -> Result<A, Error>>(
        self,
        f: F,
    ) -> Result<ValidToAddEvents<Timestamped<A>>, Error> {
        Ok(ValidToAddEvents {
            events: self
                .events
                .into_iter()
                .map(move |timestamped| timestamped.map(&f))
                .map(Timestamped::transpose)
                .collect::<Result<Vec<_>, Error>>()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AppState, Event};
    use std::cell::Cell;

    #[derive(
        Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
    )]
    struct Number(i32);
    struct Context {
        conversions: Cell<usize>,
        multiplier: i32,
    }
    impl Event for Number {
        type Versioned = Self;
        type Context = Context;
        fn to_versioned(&self) -> Self {
            self.clone()
        }
        fn from_versioned(value: &Self, context: &Context) -> Option<Self> {
            context.conversions.set(context.conversions.get() + 1);
            (value.0 != 0).then(|| Number(value.0 * context.multiplier))
        }
    }
    #[derive(Debug, PartialEq)]
    struct State(Vec<i32>);
    impl AppState for State {
        type Event = Number;
        type Partial = Vec<i32>;
        fn process_event(
            mut state: Vec<i32>,
            _: &Context,
            event: &Timestamped<Number>,
        ) -> Vec<i32> {
            state.extend([event.event.0, event.timezone.local_minus_utc()]);
            state
        }
        fn finalize(mut state: Vec<i32>, _: &Context) -> Self {
            // Finalize must not mutate the prefix used by the next incremental fold.
            state.push(-1);
            Self(state)
        }
    }
    fn event(time: i64, index: usize, value: i32) -> Timestamped<EventType<Number>> {
        Timestamped {
            timestamp: chrono::DateTime::from_timestamp(time, 0).unwrap(),
            timezone: chrono::FixedOffset::east_opt(0).unwrap(),
            within_device_events_index: index,
            event: EventType::User(Number(value)),
        }
    }
    #[test]
    fn unknown_event_keeps_indices_and_is_skipped_by_full_and_cached_folds() {
        use crate::data_model::{RawJson, StreamStore};
        let unknown = r#"{ "User" : {"Future": [1e2, "a"]} }"#;
        let raw_event = |index, json: &str| Timestamped {
            event: serde_json::from_str::<RawJson>(json).unwrap(),
            timestamp: chrono::DateTime::from_timestamp(10 + index as i64, 0).unwrap(),
            timezone: chrono::FixedOffset::east_opt(0).unwrap(),
            within_device_events_index: index,
        };
        let mut stream = EventStreamStore::<&str, Timestamped<EventType<Number>>>::default();
        let batch = vec![
            raw_event(0, r#"{"User":1}"#),
            raw_event(1, unknown),
            raw_event(2, r#"{"User":2}"#),
        ];
        let valid = stream.valid_to_add_event_jsons(&"a", batch).unwrap();
        assert_eq!(stream.add_device_event_jsons("a", valid).unwrap(), 3);
        let context = Context {
            conversions: Cell::new(0),
            multiplier: 1,
        };
        let mut cache = FoldCache::<State>::default();
        assert_eq!(
            stream.state_cached(&mut cache, Vec::new, &context),
            State(vec![1, 0, 2, 0, -1])
        );
        let valid = stream
            .valid_to_add_event_jsons(&"a", vec![raw_event(3, r#"{"User":3}"#)])
            .unwrap();
        assert_eq!(stream.add_device_event_jsons("a", valid).unwrap(), 1);
        assert_eq!(
            stream.state_cached(&mut cache, Vec::new, &context),
            State(vec![1, 0, 2, 0, 3, 0, -1])
        );
        assert_eq!(
            stream.state::<Number, State>(vec![], &context),
            State(vec![1, 0, 2, 0, 3, 0, -1])
        );
        assert_eq!(stream.jsons(&"a", 1)[0].event.get(), unknown);
        let serialized = serde_json::to_string(&stream).unwrap();
        let restored: EventStreamStore<String, Timestamped<EventType<Number>>> =
            serde_json::from_str(&serialized).unwrap();
        assert_eq!(restored.jsons(&"a".to_owned(), 1)[0].event.get(), unknown);
    }

    #[test]
    fn incremental_fold_matches_full_for_append_skip_older_equal_and_replacement() {
        let context = Context {
            conversions: Cell::new(0),
            multiplier: 1,
        };
        let mut stream = EventStreamStore::default();
        let mut cache = FoldCache::<State>::default();
        let check = |stream: &EventStreamStore<&str, _>, cache: &mut FoldCache<State>, count| {
            context.conversions.set(0);
            let cached = stream.state_cached(cache, Vec::new, &context);
            assert_eq!(context.conversions.get(), count);
            assert_eq!(cached, stream.state::<Number, State>(vec![], &context));
        };
        check(&stream, &mut cache, 0);
        stream.add_event_unchecked("a", event(10, 0, 1));
        check(&stream, &mut cache, 1);
        check(&stream, &mut cache, 0);
        stream.add_event_unchecked("a", event(20, 1, 2));
        check(&stream, &mut cache, 1);
        stream.add_event_unchecked("a", event(30, 2, 0)); // skipped migration still advances marker
        check(&stream, &mut cache, 1);
        check(&stream, &mut cache, 0);
        stream.add_event_unchecked("b", event(15, 0, 3)); // older remote event
        check(&stream, &mut cache, 4);
        let mut equal = event(20, 1, 2);
        equal.timezone = chrono::FixedOffset::east_opt(3600).unwrap();
        stream.add_event_unchecked("b", equal); // equal ordering key on another device
        check(&stream, &mut cache, 5);
        stream.add_event_unchecked("a", event(40, 3, 4));
        check(&stream, &mut cache, 1);
        stream.add_event_unchecked("c", event(50, 0, 5)); // new device changes tie iteration
        check(&stream, &mut cache, 7);
        let cloned = stream.clone();
        check(&cloned, &mut cache, 7);
        let restored: EventStreamStore<String, Timestamped<EventType<Number>>> =
            serde_json::from_str(&serde_json::to_string(&stream).unwrap()).unwrap();
        assert_eq!(
            restored.state_cached(&mut cache, Vec::new, &context),
            restored.state::<Number, State>(vec![], &context)
        );
        let new_context = Context {
            conversions: Cell::new(0),
            multiplier: 2,
        };
        cache = FoldCache::default();
        assert_eq!(
            stream.state_cached(&mut cache, Vec::new, &new_context),
            stream.state::<Number, State>(vec![], &new_context)
        );
        let replacement = EventStreamStore::<&str, _>::default();
        check(&replacement, &mut cache, 0);
    }
}
