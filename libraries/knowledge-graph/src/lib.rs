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
/// A lowercase slug, like concept ids, so the same idea always spells the
/// same tag.
pub type TagName = String;

/// One change to the knowledge graph. Serialized forms are stored forever, so
/// change them only by adding a new version.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum KnowledgeEvent {
    /// Create a concept. `id` is a permanent lowercase slug (a-z, 0-9, '-').
    /// Reviews and links refer to it forever, so it's never reused, even
    /// after the concept is retired; the title can change freely.
    AddConcept {
        id: ConceptId,
        title: String,
        /// The one thing the user can do once they know this, phrased so a
        /// single question could test it. A description that needs an "and"
        /// is often two concepts.
        #[serde(default)]
        description: String,
        /// Tags for finding and scoping the concept later (see the tag op).
        #[serde(default)]
        tags: Vec<TagName>,
    },
    /// Change a concept's title and/or description.
    EditConcept {
        id: ConceptId,
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        description: Option<String>,
    },
    /// Set the link between two concepts (prerequisite false with
    /// encompassing 0 removes it). `prerequisite` means `basic` is learned
    /// before `advanced` can appear on the frontier. `encompassing` (0 to 1)
    /// is roughly the chance that practicing `advanced` exercises `basic`:
    /// passing reviews send that share of credit down, and failures send a
    /// penalty up. The two are independent, since needing something first
    /// doesn't mean practicing the advanced concept exercises it.
    SetLink {
        advanced: ConceptId,
        basic: ConceptId,
        prerequisite: bool,
        #[schemars(with = "f64")]
        encompassing: OrderedFloat<f64>,
    },
    /// Split a concept into new, finer parts, which helps once one question
    /// can't test the concept. The concept stays as a composite that fully
    /// encompasses each part (and the parts become its prerequisites), so
    /// reviewing it still credits them all. Parts also take on the
    /// concept's own prerequisites, tags and linked documents.
    SplitConcept {
        id: ConceptId,
        into: Vec<NewConcept>,
    },
    /// Mark a concept as already known: it satisfies prerequisites and never
    /// comes due, which suits things the user has told you they know.
    /// `known: false` puts it back on the schedule.
    SetKnown { id: ConceptId, known: bool },
    /// An explicit review: one question about one concept. `score` (0 to 1,
    /// with 0.5 and up passing) is how well the user recalled that one thing;
    /// a pass moves the concept forward roughly in proportion, and a fail
    /// sets it back. When a question ends up covering several things, a
    /// partial score blurs them together, and splitting the concept usually
    /// gives the scheduler a cleaner signal. A concept's first pass is what
    /// starts its schedule; a failed first attempt is recorded but leaves it
    /// unlearned.
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
        /// An optional extra-credit question asked alongside the main one, to
        /// paint the bigger picture or connect to other concepts: what was
        /// asked, what the user said, and what it showed. It's kept for
        /// later sessions to read and never affects scheduling.
        #[serde(default)]
        extra_credit: String,
        /// Concepts the question handed the user (a formula stated, a
        /// definition given) rather than making them recall it. Credit from
        /// a pass normally flows to what this concept builds on, on the
        /// assumption that answering exercised it; concepts listed here
        /// (and anything reached only through them) are skipped, since the
        /// answer showed nothing about them.
        #[serde(default)]
        user_was_given: Vec<ConceptId>,
    },
    /// Remove a concept from the graph, along with its links. Its id stays
    /// reserved.
    RetireConcept { id: ConceptId },
    /// Add tags to concepts. Tags are how the user (and later sessions) find
    /// and scope their studies, so they're worth adding generously. They can
    /// describe the material (`machine-learning`, `linear-algebra`) or the
    /// context the user is learning it in (`for-work`, `reading-group`), and
    /// context tags in particular help pick relevant examples and plan a
    /// session around one goal. Reusing tags already in the graph (listed by
    /// knowledge_graph) keeps them useful as filters.
    Tag {
        concepts: Vec<ConceptId>,
        tags: Vec<TagName>,
    },
    /// Remove tags from concepts.
    Untag {
        concepts: Vec<ConceptId>,
        tags: Vec<TagName>,
    },
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
    /// Whether this part starts with the split concept's memory, so its past
    /// reviews count for the part. False suits a part those reviews never
    /// really tested, which then starts unlearned. Defaults to true.
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
