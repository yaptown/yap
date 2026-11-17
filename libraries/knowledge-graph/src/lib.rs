//! A mutable concept graph with Math Academy-style spaced repetition.
//!
//! The graph is the fold of an append-only event stream ([`KnowledgeEvent`]).
//! Concepts are linked two ways, following Math Academy's design:
//!
//! - **Prerequisite** (`basic` must be learned before `advanced`) gates what can
//!   be learned next — the [frontier](KnowledgeGraph::frontier).
//! - **Encompassing** (practicing `advanced` exercises `basic`, with a weight in
//!   0..=1) routes review credit. Passing a review of a concept sends
//!   discounted credit *down* to what it encompasses; failing one sends a
//!   penalty *up* to what encompasses it. This is Fractional Implicit
//!   Repetition (FIRe).
//!
//! Only explicit reviews are stored; implicit credit is derived by the fold,
//! using the links as they stood when each review happened. Editing a link
//! changes what future reviews spread, not what past ones earned.

mod graph;
mod memory;
mod plan;

pub use graph::{Concept, ConceptStatus, KnowledgeGraph, Link, ReviewRecord, check_slug};
pub use memory::{DUE_MEMORY, Memory, PASSING_SCORE};
pub use plan::{Task, TaskKind};

use ordered_float::OrderedFloat;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub type ConceptId = String;
pub type DocumentId = String;

/// One change to the knowledge graph. Serialized forms are stored forever, so
/// change them only by adding a new version.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum KnowledgeEvent {
    /// Create a concept. `id` is a permanent lowercase slug (a-z, 0-9, '-'),
    /// never reused, even after the concept is retired.
    AddConcept {
        id: ConceptId,
        title: String,
        /// The one thing the user can do once they know this, stated so a
        /// single question can test it. If it takes "and" to say, it is two
        /// concepts.
        #[serde(default)]
        description: String,
    },
    /// Change a concept's title and/or description.
    EditConcept {
        id: ConceptId,
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        description: Option<String>,
    },
    /// Set (or, with prerequisite false and encompassing 0, remove) the link
    /// between two concepts. `prerequisite`: `basic` must be learned before
    /// `advanced`. `encompassing` (0..=1): roughly the chance that practicing
    /// `advanced` exercises `basic`; review credit flows along it.
    SetLink {
        advanced: ConceptId,
        basic: ConceptId,
        prerequisite: bool,
        #[schemars(with = "f64")]
        encompassing: OrderedFloat<f64>,
    },
    /// Split a concept into new, finer parts. The concept stays as a
    /// composite that fully encompasses each part, so reviewing it still
    /// credits them all. Split when one question can't test the concept.
    SplitConcept {
        id: ConceptId,
        into: Vec<NewConcept>,
    },
    /// Mark a concept as already known: it satisfies prerequisites and never
    /// comes due. `known: false` reverses this.
    SetKnown { id: ConceptId, known: bool },
    /// An explicit review: one question about one concept. `score` is in
    /// 0..=1 (0.5 and above passes) and measures how well the user recalled
    /// that one thing. It is not the share of several things they got right;
    /// if the question covered several, split the concept and review each
    /// part.
    Review {
        id: ConceptId,
        #[schemars(with = "f64")]
        score: OrderedFloat<f64>,
        /// What was asked.
        question: String,
        /// What the user answered.
        answer: String,
        /// The grader's notes: what was right, wrong, or worth probing next.
        #[serde(default)]
        notes: String,
    },
    /// Remove a concept from the graph, along with its links. Its id stays
    /// reserved.
    RetireConcept { id: ConceptId },
    /// Attach (or with `linked: false`, detach) a document to a concept.
    /// Documents themselves live outside the event stream (they are mutable
    /// content); this only records the relationship. `note` says how the
    /// document relates, e.g. "section 3 derives this".
    SetDocumentLink {
        document: DocumentId,
        concept: ConceptId,
        linked: bool,
        #[serde(default)]
        note: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub struct NewConcept {
    pub id: ConceptId,
    pub title: String,
    /// As for add_concept: the one thing, testable by one question.
    #[serde(default)]
    pub description: String,
    /// Whether this part starts with the split concept's memory (its review
    /// history counts for the part). Set false for a part the past reviews
    /// never actually tested; it then starts unlearned. Defaults to true.
    #[serde(default = "inherit_by_default")]
    pub inherit_memory: bool,
}

fn inherit_by_default() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum VersionedKnowledgeEvent {
    V1(KnowledgeEvent),
}

impl weapon::data_model::Event for KnowledgeEvent {
    type Versioned = VersionedKnowledgeEvent;
    type Context = ();

    fn to_versioned(&self) -> Self::Versioned {
        VersionedKnowledgeEvent::V1(self.clone())
    }

    fn from_versioned(versioned: &Self::Versioned, _context: &()) -> Option<Self> {
        match versioned {
            VersionedKnowledgeEvent::V1(event) => Some(event.clone()),
        }
    }
}
