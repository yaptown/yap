//! This is a library for enabling cross-device local-first event syncing.
//! It was created for Yap.Town, so it doesn't include much that was not needed for that project.
//!
//! Syncing strategy:
//! 1. Each of the user's devices gets a unique ID.
//! 2. As users use your app, instead of the app modifying the state directly, they generate "events". Events are associated with the device generated them, as well as a timestamp and an index within the device's events.
//! 3. Starting from a default initial state, these events are "applied" in chronological order to get the current state.
//! 4. When syncing:
//!   1. The user's device asks the server how many events the server has, then sends any events that it has that the server doesn't.
//!   2. The user's device tells the server what events it has, then the server responds with the events that the user's device doesn't have.
//!
//! Sounds simple, but there are a few tricky parts that this library handles.

#[cfg(feature = "supabase")]
pub mod supabase;

#[cfg(feature = "opfs")]
pub mod opfs;

#[cfg(target_arch = "wasm32")]
#[cfg(feature = "indexeddb")]
pub mod indexeddb;

pub mod data_model;

use crate::data_model::{Event, Timestamped};

/// Core trait for partial event processing without derived state computation
pub trait PartialAppState: Sized {
    type Event: Event;

    /// The intermediate state type returned by process_event.
    /// For simple cases, this can just be Self.
    type Partial: Sized;

    /// Process an event partially, without computing derived state.
    /// This is called for each event when applying multiple events.
    fn process_event(partial: Self::Partial, event: &Timestamped<Self::Event>) -> Self::Partial;

    /// Finalize the state by computing any derived state (e.g., statistical models).
    /// This is called once after all events have been processed.
    fn finalize(partial: Self::Partial) -> Self;
}

/// Extension trait that provides apply_event for backward compatibility
pub trait AppState: PartialAppState {
    /// Apply a single event completely, including finalization.
    fn apply_event(self, event: &Timestamped<Self::Event>) -> Self;
}

/// Blanket implementation: anything that can convert Self -> Partial gets apply_event automatically
impl<T> AppState for T
where
    T: PartialAppState,
    T::Partial: From<T>,
{
    fn apply_event(self, event: &Timestamped<Self::Event>) -> Self {
        let partial = T::Partial::from(self);
        let partial = T::process_event(partial, event);
        T::finalize(partial)
    }
}
