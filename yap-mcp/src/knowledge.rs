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
pub struct ScoreOut {
    concept: String,
    score: f64,
}

fn scores_out(scores: &[(String, f64)]) -> Vec<ScoreOut> {
    scores
        .iter()
        .map(|(concept, score)| ScoreOut {
            concept: concept.clone(),
            score: *score,
        })
        .collect()
}

#[derive(Serialize, JsonSchema)]
pub struct ReviewOut {
    concept: String,
    at: String,
    score: f64,
    question: String,
    answer: String,
    notes: String,
    /// Any extra-credit question and how it went; doesn't affect scheduling.
    #[serde(skip_serializing_if = "String::is_empty")]
    extra_credit: String,
    /// Other concepts the same answer was scored on.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    also_scored: Vec<ScoreOut>,
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
    /// For each review: the linked concepts the answer may also have shown
    /// something about, whether understanding or a gap.
    linked_hints: Vec<String>,
    /// After reviews, a reminder to fold what they revealed back into the
    /// graph.
    after_review: Option<String>,
    synced: bool,
}

/// Passing scores below this count as partial, for the after-review note.
const SOLID_SCORE: f64 = 0.8;

/// What tends to help next, keyed on the batch's weakest review, plus a
/// reminder to fold what the reviews revealed back into the graph.
fn after_review(weakest: &ReviewOut, graph: &KnowledgeGraph, at: DateTime<Utc>) -> String {
    let pacing = if weakest.score < PASSING_SCORE {
        let comeback = match graph.concepts[&weakest.concept].due_at() {
            Some(due) if due > at + chrono::Duration::hours(1) => {
                let hours = (due - at).num_hours();
                let when = if hours < 36 {
                    format!("in about {hours} hours")
                } else {
                    format!("in about {} days", (hours + 12) / 24)
                };
                format!("The schedule brings '{}' back {when}", weakest.concept)
            }
            Some(_) => format!(
                "'{}' is due again right away, so the next session picks it up",
                weakest.concept
            ),
            None => format!(
                "'{}' stays on the frontier, so knowledge_next keeps offering it",
                weakest.concept
            ),
        };
        format!(
            "After a miss, a short explanation of what went wrong helps most while it's fresh. \
             Then a different question on the same concept a little later in the session lets \
             the user retrieve the corrected idea themselves, which is what makes it stick. If \
             that misses too, it's worth thinking about why: the explanation may not have \
             landed, or a prerequisite gap may be underneath (remediation lists the \
             candidates). Explaining it from another angle, probing a prerequisite, or moving \
             on and letting the schedule bring it back can each be the right call, depending on \
             what the answers showed. {comeback}, so if the user would rather study something \
             else for now, you can reassure them this topic won't be lost. Keeping it light \
             helps: a miss is information for the schedule, not a verdict."
        )
    } else if weakest.score < SOLID_SCORE {
        "A partial answer is usually best met by briefly filling in the missing piece and then \
         moving on: the shaky concept already comes back sooner on its own schedule, so \
         lingering adds little."
            .to_string()
    } else {
        "Momentum helps: after a solid pass, carrying on to the next concept (knowledge_next) \
         usually beats lingering, since new material is where most of the growth happens. If \
         the user has been on one subject for a while and concepts are waiting in other areas \
         (knowledge_next without a tag shows everything), offering a switch is often a good \
         idea: switching spaces out practice on each subject and interleaves them, and \
         research links both to better long-term retention, while the scheduler brings the \
         first subject back when it's due. Following their curiosity down the rabbit hole is \
         a fine choice too, so asking works well."
            .to_string()
    };
    format!(
        "{pacing} Separately, it's a good moment to fold what these reviews revealed back into \
         the graph, quietly: if a question only tested part of its concept, splitting lets the \
         score land on the part it measured; and if the answer leaned on another idea or showed \
         an insight, a link or a new concept captures it."
    )
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
    #[serde(skip_serializing_if = "String::is_empty")]
    extra_credit: String,
    /// Other concepts the same answer was scored on.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    also_scored: Vec<ScoreOut>,
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
    /// A reminder for the moment questions get written.
    note: &'static str,
    total_due: usize,
    /// The fewest tasks that cover every due review, in order.
    tasks: Vec<TaskOut>,
    /// Concepts ready to learn: not learned, all prerequisites learned.
    frontier: Vec<StudyConceptOut>,
    /// The next few concepts to come due.
    upcoming: Vec<ConceptSummaryOut>,
}

const QUESTION_NOTE: &str = "Each question works best aimed at a single concept and no broader \
    than it, so its score means one thing: for Newton's first law, \"What is Newton's first \
    law?\" rather than also asking how it applies to an astronaut. If a question you're about to \
    ask naturally wants two parts, the concept is probably two, and splitting it before asking \
    lets each part be asked and scheduled on its own. A deeper or connecting question is \
    welcome as optional extra credit, labeled as such, with its outcome kept in the review's \
    extra_credit field rather than the score.";

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
                extra_credit: review.extra_credit.clone(),
                also_scored: scores_out(&review.also_scored),
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
        extra_credit: review.extra_credit.clone(),
        also_scored: scores_out(&review.also_scored),
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
        | KnowledgeEvent::RetireConcept { id } => vec![id],
        KnowledgeEvent::Review { scores, .. } => {
            scores.iter().map(|s| s.concept.as_str()).collect()
        }
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
        description = "Apply a batch of changes to the user's knowledge graph: add, edit, split or retire concepts, set links, mark concepts known, record reviews, and link documents. The batch's application is atomic, and later ops can refer to concepts added earlier in it.\n\nHow the graph works, Math Academy-style: concepts are linked in two ways. A prerequisite link means the basic concept is learned before the advanced one appears on the frontier. An encompassing link (0 to 1) is roughly the chance that practicing the advanced concept exercises the basic one. When a review passes, credit flows down encompassing links, and when one fails, a penalty flows up. That credit is discounted while the receiving concept's memory is still fresh, so practice right after learning adds little. Many prerequisites have encompassing 0, since needing one thing first doesn't mean practicing the other exercises it.\n\nScheduling works best when a single question can test each concept, and a description that needs an \"and\" is often two concepts in disguise. You should obviously consider this while making concepts, but it's also common for reviews to expose where a concept was not as atomic as you thought. For example, you may have a concept for \"newton's second law\", but then realize during reviews that the user remembered f=ma but forgot what the terms meant, and then of course they didn't connect it to the idea that forces determine the second derivative of motion. That is why you can split concepts and rewire the graph after-the-fact - a normal response to a partially failed review is to split out a separate concept for the part of the thing the user forgot. As another example, \"Loss functions\" is too broad a concept, better would be \"loss function are what? optimization target in machine learning\" (quiz: \"What do you think a loss function is in machine learning?\"), \"loss functions must have what? a gradient\" (quiz: \"Suppose your loss is a black box that only returns a number. You could still train by tweaking weights at random and keeping the tweaks that lower it. What does a differentiable loss give you that this doesn't?\"), \"loss function gradient should be what? informative\" (quiz: \"'1 if the prediction is correct and 0 otherwise' is differentiable almost everywhere and is minimized exactly when pred = true. Why is it still a poor loss function?\")\n\nLike Mendeleev leaving gaps in the periodic table for elements nobody had found yet, the graph can hold concepts the user never mentioned, when they're what makes the rest hang together. A few kinds of connection are worth seeking out: the unifying idea behind several things the user asked about, the dimension along which those things differ, and neighboring concepts that bridge or extend what's already there, so their understanding doesn't have quiet holes. For example, if the user asks about forward Euler and Heun's method for solving differential equations, the graph becomes much more useful with a node for numerical integration itself (approximating an ODE's solution by stepping forward in small time increments) and one for order of accuracy (forward Euler is first order, so halving the step size roughly halves the error, while Heun's method is second order, so halving it cuts the error to about a quarter). Both are prerequisites of the two methods, and they're often the ideas that make the named methods click. You might also notice a natural extension, like RK4 as the fourth-order member of the same family. Likewise, when a new concept naturally touches something already in the user's graph, linking the two ties the new idea into what they know, which helps both stick (you might add a new node connecting both numerical integration and gradient-descent, such as \"gradient step is a forward Euler step on the gradient flow\", to help the user connect the concepts to things they already know). That's worth doing where the connection is real and direct, and best skipped when it's a stretch: a forced link muddies the map. Concepts you add this way start unlearned, like any other. A brief mention makes sense when the user might already know one (so they can say so and it gets marked known), and otherwise the additions can just happen.\n\nKeeping the graph in good shape is your part of the collaboration. The user is here to learn and will rarely think to ask for a particular split or link, and if they do find themselves having to, it pulls them out of learning and into managing a tool, which makes for a worse experience. So the graph stays most useful when you shape it as you go rather than waiting to be asked. Splits, links and new nodes are bookkeeping in service of the user's learning, so making them quietly works well: the user's attention belongs on the material, and they can always look at the graph if they're curious. A few moments make natural triggers. When a question only reaches part of a concept (you want to ask about the cross-entropy gradient, but the concept also covers writing cross-entropy as log-sum-exp minus the correct logit), that part is really its own concept, and splitting before recording the review lets the score land on the part it actually measured while the untested part keeps an honest \"new\" status. When the user has an insight or a surprise (noticing that softmax is the gradient of log-sum-exp), that's worth adding as a concept right away, linked to what it connects; those moments are often the most memorable thing in a session, and capturing them lets later reviews build on them. And when an answer leans on another idea, in the graph or not yet, that suggests a link, and sometimes a missing node. The graph is cheap to change, so a change that turns out wrong is easy to fix later.\n\nTags help the user and later sessions find and scope what's in the graph, so it's worth adding them generously, both when creating concepts (add_concept's tags) and afterwards (the tag op, which can tag many concepts at once). They can describe the material (`machine-learning`, `linear-algebra`) or the context the user is learning it in (`for-work`, `reading-group`, `from-linear-algebra-done-right`). Context tags are especially useful: they tell a future session why the user cares, which helps it pick relevant examples and plan a focused session. knowledge_graph lists the tags already in use, and reusing those keeps tags reliable as filters.\n\nA review records one question, the user's answer and your notes, and it can reinforce more than one concept. Usually it just scores the concept the question was meant to test (0 to 1, with 0.5 and up passing), and the automatic propagation extends part of that credit to the concepts it builds on. But if the user clearly understood, or clearly failed to understand, some other concept along the way, it's fine to score that one explicitly in the same review: explaining the cross-entropy gradient correctly can show the user knows softmax, and a slip in it can show they've got softmax wrong. A concept scored this way moves on its own score instead of on credit propagated from the main one. Past questions come back from knowledge_next and knowledge_concept, along with any other concepts each one scored, which makes it easy to ask something fresh.\n\nThe main question works best when it's no broader than the concept it tests. There's an admirable pull toward a deep question that strikes at the heart of the matter, or several that let the user show real mastery, but a question wider than its concept measures something else. For `newtons-first-law`, \"What is Newton's first law?\" is the question; \"What is Newton's first law, and how does it apply to an astronaut floating in a spaceship?\" also tests applying it, which may be a concept of its own. The deeper question still has a home as extra credit: asked after the main one and labeled as extra credit for the user, so they know it's a bonus, it can paint the bigger picture or connect to other concepts. Adding extra credit is up to you and entirely optional, but if you do add it, its outcome goes in the review's extra_credit field (what you asked, what they said, what it showed), where later sessions can read it but it shouldn't affect the score; that way it doesn't affect the scheduler. (If the extra-credit answer shows the user knows another concept in the graph, or shows a gap in one, that concept can be scored too, on what the answer showed.)\n\nOne thing follows from keeping main questions simple: a simple question can be answered by recitation, so understanding gets tested by giving it concepts of its own: `newtons-first-law` stays a plain recall node, and something like `apply-newtons-first-law` holds the application (why a puck sliding on smooth ice keeps going), with its own question and schedule (Math Academy works this way, which is how its reviews test understanding without any single question doing too much). But it's worth repeating that there's nothing wrong with simple, flashcard-like nodes. A broad web of flashcards on its own has a way of turning into true understanding, or the foundation needed for true understanding if nothing else. The related subjects you want to talk about, that excite you and the user, the applications and examples, these can all form additional concepts to add to the graph as well. It's up to you and your read on what's right for the user! A concept is basically something that the user is guaranteed to revisit and practice - if they should revisit and practice explaining something in their own words, or applying a concept in a new way, that's fine too!\n\nA blended score is where scheduling quietly goes wrong. If the user nailed one part of a concept and missed another, a score like 0.6 tells the scheduler both parts are half-remembered, so the part they know comes back sooner than it needs to and the part they don't comes back later than it should. An answer that splits like that is usually a sign the concept is two concepts, and one batch can record it cleanly: a split_concept op first (with inherit_memory false for any part earlier reviews never really tested), then one review scoring each part on what the answer showed about that part. The same batch can carry it all, so nothing has to be recorded blended first and fixed later.\n\nDon't spoil a concept's prerequisites! (Unless you want to.) If you do so, you rob the user of the chance to prove their knowledge of the prerequisite of a concept as well as the concept itself. And it affects the scheduler, because credit from a pass flows down to what the concept builds on because answering is assumed to have exercised those foundations: recalling them was part of the work. A question that hands the user a foundation breaks that assumption. If a question about the cross-entropy gradient states \"the loss is log-sum-exp minus the correct logit\", the user never had to recall that formula, yet a pass would still credit the formula's concept. So questions work best giving only the setup (the logits, which class is correct) and leaving the user to bring the rest; the same goes for descriptions, where \"Given X, explain...\" with X a prerequisite invites handing X over. When supplying something really is needed for a fair question, listing it in the review's user_was_given keeps the credit honest: those concepts, and anything reached only through them, get nothing from that review.\n\nBy the way, it might be a good idea to keep the review notes factual. They aren't really supposed to be todos for your future self. They're just supposed to be a summary of what actually happened, and what the user actually said (quote them verbatim), rather than advice for how to teach them in the future, unless you're using them to encode a specific user request or something.\n\nMarking a concept known takes it off the schedule entirely, so it fits things the user has told you they already know, or subjects where they ace the pretest.",
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
            .flat_map(|op| match op {
                KnowledgeEvent::Review { scores, .. } => {
                    scores.iter().map(|s| s.concept.as_str()).collect()
                }
                _ => Vec::new(),
            })
            .collect();
        // Each score in a review op is the last review of its concept at that
        // point; walk back from the end so repeated ids map to their own
        // record.
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
        let linked_hints = reviews
            .iter()
            .filter_map(|review| {
                let linked: BTreeSet<&str> = graph
                    .links_down(&review.concept)
                    .chain(graph.links_up(&review.concept))
                    .map(|(id, _)| id.as_str())
                    .filter(|id| !reviewed_here.contains(id) && !graph.concepts[*id].known)
                    .collect();
                (!linked.is_empty()).then(|| {
                    format!(
                        "'{}' {}. An answer often shows something about linked concepts \
                         too, in either direction: it can clearly show the user understands \
                         one, or clearly show a gap in one. A review's scores can list each \
                         of those alongside the main concept, scored on what the answer \
                         showed about it, high or low. If this one showed something about a \
                         linked concept that isn't scored yet, a review with the same \
                         question and answer scoring just that concept records it. Linked \
                         concepts not scored here: {}.",
                        review.concept,
                        if review.score >= PASSING_SCORE {
                            "passed"
                        } else {
                            "missed"
                        },
                        linked.into_iter().collect::<Vec<_>>().join(", ")
                    )
                })
            })
            .collect();

        let after_review = reviews
            .iter()
            .min_by(|a, b| a.score.total_cmp(&b.score))
            .map(|weakest| after_review(weakest, &graph, at));
        let applied = params.ops.len();
        let synced = state.append_knowledge(params.ops, at).await;
        ok_typed(&ApplyOut {
            applied,
            concepts,
            reviews,
            remediation,
            split_hints,
            linked_hints,
            after_review,
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
        description = "Plan the next study session. Following Math Academy, the plan keeps reviews few so more of the user's time goes to learning: since practicing an advanced concept gives credit to what it encompasses, one well-chosen task can cover several due reviews, and the plan picks the fewest tasks that cover them all. A task is either a review of a learned concept or a lesson, which teaches a frontier concept for the first time and happens to cover due reviews along the way. Concepts whose last review failed always get a task of their own. Alongside the tasks come the frontier (concepts whose prerequisites are all learned, so they're ready to teach) and what comes due soon. Each task and frontier concept carries its description, its most recent questions with answers and notes, and its linked documents, which is usually enough to run the whole session without knowledge_concept. Passing a tag plans around just those concepts, which suits a session with one goal, like a context tag such as `for-work`.\n\nA lesson tends to go well when it opens with a question rather than an explanation. A pretest! A good pretest is a question that can only be answered by understanding the concept, posed while making it clear that you don't expect the user to know the answer yet and a guess is welcome. Trying to work something out before being taught (the pretesting effect, a cousin of the testing effect) makes the explanation that follows land better, even when the attempt is wrong, because the user has already found where their understanding runs out. That opening attempt is part of teaching rather than a test of recall, but you can still record it as a review of the concept, and potentially mark the concept as known if the user seems to already understand it. (For the review and scheduling system, there's nothing fundamentally different about a pretest question compared to any other question.)",
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
            note: QUESTION_NOTE,
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
