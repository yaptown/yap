//! Reading state that syncs across devices.

use std::collections::BTreeMap;

use language_utils::Language;
use weapon::data_model::{Event, Timestamped};

use crate::books::BookPosition;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReadingEvent {
    SetBookPosition {
        language: Language,
        slug: String,
        position: BookPosition,
    },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(tag = "version")]
pub enum VersionedReadingEvent {
    V1(ReadingEvent),
}

impl Event for ReadingEvent {
    type Versioned = VersionedReadingEvent;
    type Context = ();

    fn to_versioned(&self) -> Self::Versioned {
        VersionedReadingEvent::V1(self.clone())
    }

    fn from_versioned(versioned: &Self::Versioned, _: &()) -> Option<Self> {
        let VersionedReadingEvent::V1(event) = versioned;
        Some(event.clone())
    }
}

#[derive(Default)]
pub struct ReadingPositions(pub BTreeMap<(Language, String), BookPosition>);

impl weapon::AppState for ReadingPositions {
    type Event = ReadingEvent;
    type Partial = Self;

    fn process_event(mut partial: Self, _: &(), event: &Timestamped<ReadingEvent>) -> Self {
        let ReadingEvent::SetBookPosition {
            language,
            slug,
            position,
        } = &event.event;
        partial
            .0
            .insert((*language, slug.clone()), position.clone());
        partial
    }

    fn finalize(partial: Self, _: &()) -> Self {
        partial
    }
}
