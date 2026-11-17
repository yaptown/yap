//! Knowledge-graph tools: a general (not language-specific) concept graph
//! with Math Academy-style implicit repetition, folded from the `knowledge`
//! event stream (see the `knowledge-graph` crate). Documents live in the
//! mutable `knowledge_documents` table; the graph only records which concepts
//! link to them. There is no app UI for any of this yet.

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use rmcp::{
    RoleServer, handler::server::wrapper::Parameters, model::CallToolResult,
    service::RequestContext, tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use knowledge_graph::{
    Concept, ConceptStatus, KnowledgeEvent, KnowledgeGraph, PASSING_SCORE, ReviewRecord, TaskKind,
};

use crate::server::{SyncStatus, YapMcp, YapState, error, ok_typed};
use crate::sync::{
    KnowledgeDocument, SupabaseAuth, delete_document, fetch_documents, save_document,
};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ApplyParams {
    /// Changes to apply in order, all or nothing. Later ops can refer to
    /// concepts added by earlier ones.
    ops: Vec<KnowledgeEvent>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GraphParams {
    /// Include retired concepts (default false).
    #[serde(default)]
    include_retired: bool,
    /// Only show concepts with this tag (and the links among them). The tag
    /// list in the output always covers the whole graph.
    #[serde(default)]
    tag: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct NextParams {
    /// Plan only around concepts with this tag, e.g. a context like
    /// `for-work` when the user wants a session on one goal.
    #[serde(default)]
    tag: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ConceptParams {
    /// The concept's id.
    id: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ReviewsParams {
    /// Maximum number of reviews to return, newest first (default 30).
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct DocumentParams {
    /// The document's id.
    id: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SaveDocumentParams {
    /// Permanent lowercase slug (a-z, 0-9, '-'). Saving an existing id
    /// replaces that document.
    id: String,
    title: String,
    /// Plain text.
    body: String,
}

#[derive(Serialize, JsonSchema)]
pub struct ConceptSummaryOut {
    id: String,
    title: String,
    status: ConceptStatus,
    /// Spaced repetitions completed (fractional; implicit credit counts).
    reps: Option<f64>,
    /// Current memory level, 0..=1; due at 0.5.
    memory: Option<f64>,
    /// RFC 3339 time the concept is (or was) due.
    due: Option<String>,
    /// The last review failed. Implicit credit skips the concept until a
    /// direct review passes, since a failure is the clearest sign it needs
    /// practice of its own.
    needs_explicit_review: bool,
    reviews: usize,
    tags: Vec<String>,
}

#[derive(Serialize, JsonSchema)]
pub struct LinkOut {
    advanced: String,
    basic: String,
    prerequisite: bool,
    encompassing: f64,
}

#[derive(Serialize, JsonSchema)]
pub struct DocumentSummaryOut {
    id: String,
    title: String,
    updated_at: String,
    /// Concepts linking to the document.
    concepts: Vec<String>,
}

#[derive(Serialize, JsonSchema)]
pub struct GraphOut {
    concepts: Vec<ConceptSummaryOut>,
    links: Vec<LinkOut>,
    documents: Vec<DocumentSummaryOut>,
    /// Every tag in use, with how many concepts carry it. Reusing these
    /// rather than coining near-synonyms keeps tags useful as filters.
    tags: Vec<TagCountOut>,
}

#[derive(Serialize, JsonSchema)]
pub struct TagCountOut {
    tag: String,
    concepts: usize,
}

#[derive(Serialize, JsonSchema)]
pub struct CreditOut {
    concept: String,
    /// Repetitions added (negative for a penalty).
    reps: f64,
}

#[derive(Serialize, JsonSchema)]
pub struct ReviewOut {
    concept: String,
    at: String,
    score: f64,
    question: String,
    answer: String,
    notes: String,
    /// Repetitions this review added to its own concept, after discounting
    /// for reviewing early.
    reps: f64,
    /// Implicit credit sent down to encompassed concepts (a pass) or penalty
    /// sent up to encompassing ones (a fail).
    propagated: Vec<CreditOut>,
}

#[derive(Serialize, JsonSchema)]
pub struct ApplyOut {
    applied: usize,
    /// The resulting state of every concept the ops touched.
    concepts: Vec<ConceptSummaryOut>,
    /// The reviews recorded, with the credit they spread.
    reviews: Vec<ReviewOut>,
    /// For each failed review, that concept's prerequisites: the likely
    /// culprits worth probing next.
    remediation: Vec<RemediationOut>,
    /// Reviews with a middling score, which often means the question tested
    /// more than one thing.
    split_hints: Vec<String>,
    /// For passing reviews: the linked concepts the answer may also have
    /// shown the user understands.
    credit_hints: Vec<String>,
    synced: bool,
}

/// Scores in this range get a split hint.
const MIDDLING_SCORE: std::ops::RangeInclusive<f64> = 0.3..=0.7;

#[derive(Serialize, JsonSchema)]
pub struct RemediationOut {
    failed: String,
    prerequisites: Vec<ConceptSummaryOut>,
}

#[derive(Serialize, JsonSchema)]
pub struct RelatedOut {
    concept: ConceptSummaryOut,
    prerequisite: bool,
    encompassing: f64,
}

#[derive(Serialize, JsonSchema)]
pub struct LinkedDocumentOut {
    id: String,
    /// How the document relates to the concept.
    note: String,
    /// Null when the document was deleted from the table.
    title: Option<String>,
    body: Option<String>,
}

#[derive(Serialize, JsonSchema)]
pub struct ConceptOut {
    #[serde(flatten)]
    summary: ConceptSummaryOut,
    description: String,
    created_at: String,
    split_from: Option<String>,
    split_into: Vec<String>,
    /// What this concept builds on.
    builds_on: Vec<RelatedOut>,
    /// What builds on this concept.
    built_on_by: Vec<RelatedOut>,
    documents: Vec<LinkedDocumentOut>,
    /// Every explicit review, newest first.
    review_history: Vec<ReviewOut>,
}

/// How many past questions knowledge_next includes per concept.
const RECENT_QUESTIONS: usize = 5;

/// A concept with what's needed to teach or quiz it, so a session needs no
/// knowledge_concept call per concept.
#[derive(Serialize, JsonSchema)]
pub struct StudyConceptOut {
    #[serde(flatten)]
    summary: ConceptSummaryOut,
    description: String,
    /// The most recent questions asked about it, newest first (all of them
    /// are in knowledge_concept). A fresh angle each time tests understanding
    /// rather than memory of a familiar question.
    recent_questions: Vec<PastQuestionOut>,
    /// Linked documents (read them with knowledge_document).
    documents: Vec<DocumentLinkOut>,
}

#[derive(Serialize, JsonSchema)]
pub struct PastQuestionOut {
    at: String,
    score: f64,
    question: String,
    answer: String,
    notes: String,
}

#[derive(Serialize, JsonSchema)]
pub struct DocumentLinkOut {
    id: String,
    /// How the document relates to the concept.
    note: String,
}

#[derive(Serialize, JsonSchema)]
pub struct TaskOut {
    concept: StudyConceptOut,
    /// "review" a learned concept, or a "lesson" learning a frontier concept
    /// whose practice also covers due reviews.
    kind: TaskKind,
    /// Due concepts this one task covers.
    covers: Vec<String>,
}

#[derive(Serialize, JsonSchema)]
pub struct NextOut {
    total_due: usize,
    /// The fewest tasks that cover every due review, in order.
    tasks: Vec<TaskOut>,
    /// Concepts ready to learn: not learned, all prerequisites learned.
    frontier: Vec<StudyConceptOut>,
    /// The next few concepts to come due.
    upcoming: Vec<ConceptSummaryOut>,
}

#[derive(Serialize, JsonSchema)]
pub struct ReviewsOut {
    reviews: Vec<ReviewOut>,
}

#[derive(Serialize, JsonSchema)]
pub struct DocumentOut {
    id: String,
    title: String,
    body: String,
    updated_at: String,
    concepts: Vec<ConceptSummaryOut>,
}

#[derive(Serialize, JsonSchema)]
pub struct SaveDocumentOut {
    id: String,
    /// Concepts linked to it (link with a set_document_link op).
    concepts: Vec<String>,
}

#[derive(Serialize, JsonSchema)]
pub struct DeleteDocumentOut {
    id: String,
    /// Concepts the document was unlinked from.
    unlinked: Vec<String>,
    synced: bool,
}

fn rfc3339(at: DateTime<Utc>) -> String {
    at.to_rfc3339()
}

fn summary(id: &str, concept: &Concept, now: DateTime<Utc>) -> ConceptSummaryOut {
    let memory = concept.memory.as_ref().filter(|_| !concept.known);
    ConceptSummaryOut {
        id: id.to_string(),
        title: concept.title.clone(),
        status: concept.status(),
        reps: memory.map(|m| m.reps),
        memory: memory.map(|m| m.memory_at(now)),
        due: concept.due_at().map(rfc3339),
        needs_explicit_review: concept.needs_explicit_review(),
        reviews: concept.reviews.len(),
        tags: concept.tags.iter().cloned().collect(),
    }
}

/// Whether a concept falls under an optional tag filter.
fn in_scope(tag: Option<&str>) -> impl Fn(&Concept) -> bool + '_ {
    move |concept| tag.is_none_or(|tag| concept.tags.contains(tag))
}

fn summary_of(graph: &KnowledgeGraph, id: &str, now: DateTime<Utc>) -> ConceptSummaryOut {
    summary(id, &graph.concepts[id], now)
}

fn study_of(graph: &KnowledgeGraph, id: &str, now: DateTime<Utc>) -> StudyConceptOut {
    let concept = &graph.concepts[id];
    StudyConceptOut {
        summary: summary(id, concept, now),
        description: concept.description.clone(),
        recent_questions: concept
            .reviews
            .iter()
            .rev()
            .take(RECENT_QUESTIONS)
            .map(|review| PastQuestionOut {
                at: rfc3339(review.at),
                score: review.score,
                question: review.question.clone(),
                answer: review.answer.clone(),
                notes: review.notes.clone(),
            })
            .collect(),
        documents: concept
            .documents
            .iter()
            .map(|(id, note)| DocumentLinkOut {
                id: id.clone(),
                note: note.clone(),
            })
            .collect(),
    }
}

fn review_out(concept: &str, review: &ReviewRecord) -> ReviewOut {
    ReviewOut {
        concept: concept.to_string(),
        at: rfc3339(review.at),
        score: review.score,
        question: review.question.clone(),
        answer: review.answer.clone(),
        notes: review.notes.clone(),
        reps: review.applied,
        propagated: review
            .propagated
            .iter()
            .map(|(concept, reps)| CreditOut {
                concept: concept.clone(),
                reps: *reps,
            })
            .collect(),
    }
}

/// The concepts an op refers to.
fn touched(op: &KnowledgeEvent) -> Vec<&str> {
    match op {
        KnowledgeEvent::AddConcept { id, .. }
        | KnowledgeEvent::EditConcept { id, .. }
        | KnowledgeEvent::SetKnown { id, .. }
        | KnowledgeEvent::Review { id, .. }
        | KnowledgeEvent::RetireConcept { id } => vec![id],
        KnowledgeEvent::SetLink {
            advanced, basic, ..
        } => vec![advanced, basic],
        KnowledgeEvent::SplitConcept { id, into } => std::iter::once(id.as_str())
            .chain(into.iter().map(|part| part.id.as_str()))
            .collect(),
        KnowledgeEvent::SetDocumentLink { concept, .. } => vec![concept],
        KnowledgeEvent::Tag { concepts, .. } | KnowledgeEvent::Untag { concepts, .. } => {
            concepts.iter().map(String::as_str).collect()
        }
    }
}

/// Lock this call's state and pull in fresh events, as every tool does.
macro_rules! synced_state {
    ($self:ident, $ctx:ident) => {{
        let slot = match $self.state_slot(&$ctx).await {
            Ok(slot) => slot,
            Err(e) => return error(e),
        };
        let mut state = slot.lock_owned().await;
        if let Err(e) = state.sync().await {
            log::warn!("sync failed, using possibly-stale events: {e:#}");
        }
        state
    }};
}

/// The user's `knowledge_documents` table, cloned out of the state so the
/// state isn't borrowed across awaits.
struct Documents {
    http: reqwest::Client,
    supabase: SupabaseAuth,
    user_id: String,
}

impl Documents {
    fn of(state: &YapState) -> Self {
        Documents {
            http: state.http.clone(),
            supabase: state.supabase.clone(),
            user_id: state.user_id.clone(),
        }
    }

    async fn fetch(
        &self,
        ids: Option<&[String]>,
        with_body: bool,
    ) -> Result<Vec<KnowledgeDocument>, CallToolResult> {
        fetch_documents(&self.http, &self.supabase, &self.user_id, ids, with_body)
            .await
            .map_err(|e| error(format!("failed to fetch documents: {e:#}")))
    }
}

#[tool_router(router = knowledge_router, vis = "pub(crate)")]
impl YapMcp {
    #[tool(
        title = "Change the knowledge graph",
        output_schema = rmcp::handler::server::common::schema_for_type::<ApplyOut>(),
        description = "Apply a batch of changes to the user's knowledge graph: add, edit, split or retire concepts, set links, mark concepts known, record reviews, and link documents. The batch is all or nothing, and later ops can refer to concepts added earlier in it.\n\nHow the graph works, Math Academy-style: concepts are linked in two ways. A prerequisite link means the basic concept is learned before the advanced one appears on the frontier. An encompassing link (0 to 1) is roughly the chance that practicing the advanced concept exercises the basic one. When a review passes, credit flows down encompassing links, and when one fails, a penalty flows up. That credit is discounted while the receiving concept's memory is still fresh, so practice right after learning adds little. Many prerequisites have encompassing 0, since needing one thing first doesn't mean practicing the other exercises it.\n\nScheduling works best when a single question can test each concept, and a description that needs an \"and\" is often two concepts in disguise. For example, \"Loss functions\" tends to invite a two-part question (what is it, and what makes a good one?), so it works better as two: `loss-function`, \"Say what a loss function is: a number scoring how far the model's output is from the target, which training minimizes\" (quiz: \"What does a loss function compute?\"), and `loss-needs-informative-gradient`, \"Explain why a loss must have an informative gradient: the optimizer only follows the gradient, so where it's flat or zero there's no direction to improve\" (quiz: \"Why is 0/1 accuracy a bad training loss?\"), with `loss-function` as its prerequisite. If reviewing the two together is still useful, splitting \"Loss functions\" into them keeps it as a composite.\n\nLike Mendeleev leaving gaps in the periodic table for elements nobody had found yet, the graph can hold concepts the user never mentioned, when they're what makes the rest hang together. A few kinds of connection are worth seeking out: the unifying idea behind several things the user asked about, the dimension along which those things differ, and neighboring concepts that bridge or extend what's already there, so their understanding doesn't have quiet holes. For example, if the user asks about forward Euler and Heun's method for solving differential equations, the graph becomes much more useful with a node for numerical integration itself (approximating an ODE's solution by stepping forward in small time increments) and one for order of accuracy (forward Euler is first order, so halving the step size roughly halves the error, while Heun's method is second order, so halving it cuts the error to about a quarter). Both are prerequisites of the two methods, and they're often the ideas that make the named methods click. You might also notice a natural extension, like RK4 as the fourth-order member of the same family. Likewise, when a new concept naturally touches something already in the user's graph, linking the two ties the new idea into what they know, which helps both stick (numerical integration might connect to a gradient-descent node, since a gradient step is a forward Euler step on the gradient flow). That's worth doing where the connection is real and direct, and best skipped when it's a stretch: a forced link sends review credit where it doesn't belong and muddies the map.Concepts you add this way start unlearned, like any other, and it's good to mention what you added and why, so the user sees the map grow and can say if something doesn't belong or that they already know it.\n\nTags help the user and later sessions find and scope what's in the graph, so it's worth adding them generously, both when creating concepts (add_concept's tags) and afterwards (the tag op, which can tag many concepts at once). They can describe the material (`machine-learning`, `linear-algebra`) or the context the user is learning it in (`for-work`, `reading-group`). Context tags are especially useful: they tell a future session why the user cares, which helps it pick relevant examples and plan a focused session. knowledge_graph lists the tags already in use, and reusing those keeps tags reliable as filters.\n\nA review records one question about one concept. The score (0 to 1, with 0.5 and up passing) is how well the user recalled that one thing, stored with the question, their answer and your notes. Past questions come back from knowledge_next and knowledge_concept, which makes it easy to ask something fresh. If a question wants to be two, or a score would really mean \"one of two parts\", that's usually a sign the concept is ready to split so each part gets its own schedule. Marking a concept known takes it off the schedule entirely, so it fits things the user has told you they already know.",
        annotations(
            title = "Change the knowledge graph",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn knowledge_apply(
        &self,
        ctx: RequestContext<RoleServer>,
        Parameters(params): Parameters<ApplyParams>,
    ) -> CallToolResult {
        if params.ops.is_empty() {
            return error("ops must not be empty");
        }
        let mut state = synced_state!(self, ctx);

        let linked: Vec<String> = params
            .ops
            .iter()
            .filter_map(|op| match op {
                KnowledgeEvent::SetDocumentLink {
                    document,
                    linked: true,
                    ..
                } => Some(document.clone()),
                _ => None,
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        if !linked.is_empty() {
            let found = match Documents::of(&state).fetch(Some(&linked), false).await {
                Ok(found) => found,
                Err(e) => return e,
            };
            let found: BTreeSet<&str> = found.iter().map(|d| d.id.as_str()).collect();
            let missing: Vec<&str> = linked
                .iter()
                .map(String::as_str)
                .filter(|id| !found.contains(id))
                .collect();
            if !missing.is_empty() {
                return error(format!(
                    "nothing was applied: there's no document {} yet. Links point at saved \
                     documents, so knowledge_save_document comes first",
                    missing.join(", ")
                ));
            }
        }

        // Validate the whole batch on a copy, at the timestamp the events
        // will carry, so what we store is exactly what we checked.
        let at = Utc::now();
        let mut graph = state.knowledge().clone();
        for (i, op) in params.ops.iter().enumerate() {
            if let Err(e) = graph.apply(op, at) {
                return error(format!("nothing was applied: op {i} is invalid: {e}"));
            }
        }

        let mut touched_ids: Vec<&str> = Vec::new();
        for id in params.ops.iter().flat_map(touched) {
            if !touched_ids.contains(&id) {
                touched_ids.push(id);
            }
        }
        let concepts = touched_ids
            .iter()
            .map(|id| summary_of(&graph, id, at))
            .collect();
        let reviewed: Vec<&str> = params
            .ops
            .iter()
            .filter_map(|op| match op {
                KnowledgeEvent::Review { id, .. } => Some(id.as_str()),
                _ => None,
            })
            .collect();
        // Each review op is the last review of its concept at that point;
        // walk back from the end so repeated ids map to their own record.
        let mut seen = std::collections::BTreeMap::<&str, usize>::new();
        let mut reviews: Vec<ReviewOut> = reviewed
            .iter()
            .rev()
            .map(|id| {
                let back = seen.entry(id).or_default();
                let history = &graph.concepts[*id].reviews;
                let review = &history[history.len() - 1 - *back];
                *back += 1;
                review_out(id, review)
            })
            .collect();
        reviews.reverse();
        let remediation = reviews
            .iter()
            .filter(|review| review.score < PASSING_SCORE)
            .map(|review| RemediationOut {
                failed: review.concept.clone(),
                prerequisites: graph
                    .prerequisites(&review.concept)
                    .map(|id| summary_of(&graph, id, at))
                    .collect(),
            })
            .collect();
        let split_hints = reviews
            .iter()
            .filter(|review| MIDDLING_SCORE.contains(&review.score))
            .map(|review| {
                format!(
                    "'{}' scored {}. A middling score sometimes means the question covered \
                     two things, one recalled and one not, and the scheduler can't tell that \
                     apart from shaky recall of a single thing. If that's what happened, \
                     splitting the concept lets each part keep its own schedule; \
                     inherit_memory false suits any part these reviews never really tested.",
                    review.concept, review.score
                )
            })
            .collect();
        let reviewed_here: BTreeSet<&str> = reviews.iter().map(|r| r.concept.as_str()).collect();
        let credit_hints = reviews
            .iter()
            .filter(|review| review.score >= PASSING_SCORE)
            .filter_map(|review| {
                let linked: BTreeSet<&str> = graph
                    .links_down(&review.concept)
                    .chain(graph.links_up(&review.concept))
                    .map(|(id, _)| id.as_str())
                    .filter(|id| !reviewed_here.contains(id) && !graph.concepts[*id].known)
                    .collect();
                (!linked.is_empty()).then(|| {
                    format!(
                        "'{}' passed. If the answer also clearly showed the user understands \
                         some linked concepts, they've earned credit there too: a review for \
                         each (same question and answer, scored on what the answer showed about \
                         that concept) records it. Credit for a concept reviewed recently is \
                         discounted automatically, so this won't over-count. Linked concepts: \
                         {}.",
                        review.concept,
                        linked.into_iter().collect::<Vec<_>>().join(", ")
                    )
                })
            })
            .collect();

        let applied = params.ops.len();
        let synced = state.append_knowledge(params.ops, at).await;
        ok_typed(&ApplyOut {
            applied,
            concepts,
            reviews,
            remediation,
            split_hints,
            credit_hints,
            synced: synced == SyncStatus::Synced,
        })
    }

    #[tool(
        title = "Show the knowledge graph",
        output_schema = rmcp::handler::server::common::schema_for_type::<GraphOut>(),
        description = "The user's whole knowledge graph: every concept with its learning state and tags, every link, every document (titles only), and every tag in use with its count. The tag list is handy before adding concepts, since reusing an existing tag keeps filtering by it reliable. Passing a tag narrows the view to concepts carrying it.",
        annotations(
            title = "Show the knowledge graph",
            read_only_hint = true,
            destructive_hint = false,
            open_world_hint = false
        )
    )]
    async fn knowledge_graph(
        &self,
        ctx: RequestContext<RoleServer>,
        Parameters(params): Parameters<GraphParams>,
    ) -> CallToolResult {
        let mut state = synced_state!(self, ctx);
        let docs = match Documents::of(&state).fetch(None, false).await {
            Ok(docs) => docs,
            Err(e) => return e,
        };
        let now = Utc::now();
        let graph = state.knowledge();
        let in_scope = in_scope(params.tag.as_deref());
        let shown: BTreeSet<&String> = graph
            .concepts
            .iter()
            .filter(|(_, c)| (params.include_retired || !c.retired) && in_scope(c))
            .map(|(id, _)| id)
            .collect();
        ok_typed(&GraphOut {
            concepts: shown
                .iter()
                .map(|id| summary(id, &graph.concepts[*id], now))
                .collect(),
            links: graph
                .links
                .iter()
                .filter(|((advanced, basic), _)| shown.contains(advanced) && shown.contains(basic))
                .map(|((advanced, basic), link)| LinkOut {
                    advanced: advanced.clone(),
                    basic: basic.clone(),
                    prerequisite: link.prerequisite,
                    encompassing: link.encompassing,
                })
                .collect(),
            documents: docs
                .into_iter()
                .filter_map(|doc| {
                    let concepts: Vec<String> = graph
                        .active_concepts()
                        .filter(|(_, c)| c.documents.contains_key(&doc.id))
                        .map(|(id, _)| id.clone())
                        .collect();
                    let relevant =
                        params.tag.is_none() || concepts.iter().any(|id| shown.contains(id));
                    relevant.then_some(DocumentSummaryOut {
                        concepts,
                        id: doc.id,
                        title: doc.title,
                        updated_at: doc.updated_at,
                    })
                })
                .collect(),
            tags: graph
                .tags()
                .into_iter()
                .map(|(tag, concepts)| TagCountOut {
                    tag: tag.clone(),
                    concepts,
                })
                .collect(),
        })
    }

    #[tool(
        title = "Inspect a concept",
        output_schema = rmcp::handler::server::common::schema_for_type::<ConceptOut>(),
        description = "Everything about one concept: description, learning state, what it builds on and what builds on it, its linked documents in full, and its whole review history (questions asked, answers, notes). The history shows what has been asked before, which helps when picking a new question.",
        annotations(
            title = "Inspect a concept",
            read_only_hint = true,
            destructive_hint = false,
            open_world_hint = false
        )
    )]
    async fn knowledge_concept(
        &self,
        ctx: RequestContext<RoleServer>,
        Parameters(params): Parameters<ConceptParams>,
    ) -> CallToolResult {
        let mut state = synced_state!(self, ctx);
        let doc_ids: Vec<String> = match state.knowledge().concept(&params.id) {
            Ok(concept) => concept.documents.keys().cloned().collect(),
            Err(e) => return error(e),
        };
        let docs = match Documents::of(&state).fetch(Some(&doc_ids), true).await {
            Ok(docs) => docs,
            Err(e) => return e,
        };
        let now = Utc::now();
        let graph = state.knowledge();
        let concept = &graph.concepts[&params.id];
        let related = |(id, link): (&String, knowledge_graph::Link)| RelatedOut {
            concept: summary_of(graph, id, now),
            prerequisite: link.prerequisite,
            encompassing: link.encompassing,
        };
        ok_typed(&ConceptOut {
            summary: summary(&params.id, concept, now),
            description: concept.description.clone(),
            created_at: rfc3339(concept.created_at),
            split_from: concept.split_from.clone(),
            split_into: concept.split_into.clone(),
            builds_on: graph.links_down(&params.id).map(related).collect(),
            built_on_by: graph.links_up(&params.id).map(related).collect(),
            documents: concept
                .documents
                .iter()
                .map(|(id, note)| {
                    let doc = docs.iter().find(|doc| &doc.id == id);
                    LinkedDocumentOut {
                        id: id.clone(),
                        note: note.clone(),
                        title: doc.map(|d| d.title.clone()),
                        body: doc.map(|d| d.body.clone()),
                    }
                })
                .collect(),
            review_history: concept
                .reviews
                .iter()
                .rev()
                .map(|review| review_out(&params.id, review))
                .collect(),
        })
    }

    #[tool(
        title = "What to study next",
        output_schema = rmcp::handler::server::common::schema_for_type::<NextOut>(),
        description = "Plan the next study session. Following Math Academy, the plan keeps reviews few so more of the user's time goes to learning: since practicing an advanced concept gives credit to what it encompasses, one well-chosen task can cover several due reviews, and the plan picks the fewest tasks that cover them all. A task is either a review of a learned concept or a lesson, which teaches a frontier concept for the first time and happens to cover due reviews along the way. Concepts whose last review failed always get a task of their own. Alongside the tasks come the frontier (concepts whose prerequisites are all learned, so they're ready to teach) and what comes due soon. Each task and frontier concept carries its description, its most recent questions with answers and notes, and its linked documents, which is usually enough to run the whole session without knowledge_concept. Passing a tag plans around just those concepts, which suits a session with one goal, like a context tag such as `for-work`.",
        annotations(
            title = "What to study next",
            read_only_hint = true,
            destructive_hint = false,
            open_world_hint = false
        )
    )]
    async fn knowledge_next(
        &self,
        ctx: RequestContext<RoleServer>,
        Parameters(params): Parameters<NextParams>,
    ) -> CallToolResult {
        let mut state = synced_state!(self, ctx);
        let now = Utc::now();
        let graph = state.knowledge();
        let in_scope = in_scope(params.tag.as_deref());
        let mut upcoming: Vec<(DateTime<Utc>, &String)> = graph
            .active_concepts()
            .filter(|(_, c)| in_scope(c))
            .filter_map(|(id, c)| c.due_at().filter(|due| *due > now).map(|due| (due, id)))
            .collect();
        upcoming.sort();
        ok_typed(&NextOut {
            total_due: graph
                .due(now)
                .into_iter()
                .filter(|id| in_scope(&graph.concepts[*id]))
                .count(),
            tasks: graph
                .compressed_reviews(now, &in_scope)
                .into_iter()
                .map(|task| TaskOut {
                    concept: study_of(graph, &task.concept, now),
                    kind: task.kind,
                    covers: task.covers,
                })
                .collect(),
            frontier: graph
                .frontier()
                .into_iter()
                .filter(|id| in_scope(&graph.concepts[*id]))
                .map(|id| study_of(graph, id, now))
                .collect(),
            upcoming: upcoming
                .into_iter()
                .take(5)
                .map(|(_, id)| summary_of(graph, id, now))
                .collect(),
        })
    }

    #[tool(
        title = "Recent knowledge reviews",
        output_schema = rmcp::handler::server::common::schema_for_type::<ReviewsOut>(),
        description = "The most recent explicit reviews across the whole knowledge graph, newest first, with the question asked, the user's answer, and the grading notes.",
        annotations(
            title = "Recent knowledge reviews",
            read_only_hint = true,
            destructive_hint = false,
            open_world_hint = false
        )
    )]
    async fn knowledge_reviews(
        &self,
        ctx: RequestContext<RoleServer>,
        Parameters(params): Parameters<ReviewsParams>,
    ) -> CallToolResult {
        let mut state = synced_state!(self, ctx);
        let graph = state.knowledge();
        let mut reviews: Vec<(&String, &ReviewRecord)> = graph
            .active_concepts()
            .flat_map(|(id, c)| c.reviews.iter().map(move |r| (id, r)))
            .collect();
        // Reviews from one batch share a timestamp: a stable oldest-first sort
        // keeps their order, so reversing it puts the latest first.
        reviews.sort_by_key(|(_, review)| review.at);
        reviews.reverse();
        ok_typed(&ReviewsOut {
            reviews: reviews
                .into_iter()
                .take(params.limit.unwrap_or(30))
                .map(|(id, review)| review_out(id, review))
                .collect(),
        })
    }

    #[tool(
        title = "Read a knowledge document",
        output_schema = rmcp::handler::server::common::schema_for_type::<DocumentOut>(),
        description = "A document's full text and the concepts linked to it.",
        annotations(
            title = "Read a knowledge document",
            read_only_hint = true,
            destructive_hint = false,
            open_world_hint = false
        )
    )]
    async fn knowledge_document(
        &self,
        ctx: RequestContext<RoleServer>,
        Parameters(params): Parameters<DocumentParams>,
    ) -> CallToolResult {
        let mut state = synced_state!(self, ctx);
        let doc = match Documents::of(&state)
            .fetch(Some(std::slice::from_ref(&params.id)), true)
            .await
        {
            Ok(mut docs) => match docs.pop() {
                Some(doc) => doc,
                None => return error(format!("no document '{}'", params.id)),
            },
            Err(e) => return e,
        };
        let now = Utc::now();
        let graph = state.knowledge();
        ok_typed(&DocumentOut {
            concepts: graph
                .active_concepts()
                .filter(|(_, c)| c.documents.contains_key(&doc.id))
                .map(|(id, c)| summary(id, c, now))
                .collect(),
            id: doc.id,
            title: doc.title,
            body: doc.body,
            updated_at: doc.updated_at,
        })
    }

    #[tool(
        title = "Save a knowledge document",
        output_schema = rmcp::handler::server::common::schema_for_type::<SaveDocumentOut>(),
        description = "Create or replace a plain-text document (notes, an article, a chapter) the user wants to learn from. Documents are source material for the knowledge graph: once linked to concepts (a set_document_link op in knowledge_apply), they show up in full when inspecting those concepts, so you can teach and quiz from what the user is actually reading. Unlike the graph, documents are ordinary editable records, and saving an existing id replaces it.",
        annotations(
            title = "Save a knowledge document",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn knowledge_save_document(
        &self,
        ctx: RequestContext<RoleServer>,
        Parameters(params): Parameters<SaveDocumentParams>,
    ) -> CallToolResult {
        let mut state = synced_state!(self, ctx);
        let docs = Documents::of(&state);
        if let Err(e) = save_document(
            &docs.http,
            &docs.supabase,
            &docs.user_id,
            &params.id,
            &params.title,
            &params.body,
        )
        .await
        {
            return error(format!("{e:#}"));
        }
        let graph = state.knowledge();
        ok_typed(&SaveDocumentOut {
            concepts: graph
                .active_concepts()
                .filter(|(_, c)| c.documents.contains_key(&params.id))
                .map(|(id, _)| id.clone())
                .collect(),
            id: params.id,
        })
    }

    #[tool(
        title = "Delete a knowledge document",
        output_schema = rmcp::handler::server::common::schema_for_type::<DeleteDocumentOut>(),
        description = "Delete a document and unlink it from every concept. Documents aren't kept in the event history, so a deletion can't be undone, which makes it worth checking with the user first.",
        annotations(
            title = "Delete a knowledge document",
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn knowledge_delete_document(
        &self,
        ctx: RequestContext<RoleServer>,
        Parameters(params): Parameters<DocumentParams>,
    ) -> CallToolResult {
        let mut state = synced_state!(self, ctx);
        let docs = Documents::of(&state);
        if let Err(e) = delete_document(&docs.http, &docs.supabase, &docs.user_id, &params.id).await
        {
            return error(format!("failed to delete document: {e:#}"));
        }
        let unlinked: Vec<String> = state
            .knowledge()
            .active_concepts()
            .filter(|(_, c)| c.documents.contains_key(&params.id))
            .map(|(id, _)| id.clone())
            .collect();
        let ops = unlinked
            .iter()
            .map(|concept| KnowledgeEvent::SetDocumentLink {
                document: params.id.clone(),
                concept: concept.clone(),
                linked: false,
                note: String::new(),
            })
            .collect();
        let synced = state.append_knowledge(ops, Utc::now()).await;
        ok_typed(&DeleteDocumentOut {
            id: params.id,
            unlinked,
            synced: synced == SyncStatus::Synced,
        })
    }
}
