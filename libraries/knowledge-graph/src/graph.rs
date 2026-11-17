use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use weapon::data_model::Timestamped;

use crate::memory::{DUE_MEMORY, Memory, PASSING_SCORE, review_credit};
use crate::{ConceptId, DocumentId, KnowledgeEvent, NewConcept};

/// Path weights below this don't carry implicit credit.
const MIN_FLOW_WEIGHT: f64 = 0.05;

#[derive(Clone, Debug, Default)]
pub struct KnowledgeGraph {
    /// Every concept ever added, retired ones included (their ids stay
    /// reserved).
    pub concepts: BTreeMap<ConceptId, Concept>,
    /// Links keyed by (advanced, basic), only between active concepts.
    pub links: BTreeMap<(ConceptId, ConceptId), Link>,
}

#[derive(Clone, Debug)]
pub struct Concept {
    pub title: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub known: bool,
    pub retired: bool,
    pub split_from: Option<ConceptId>,
    pub split_into: Vec<ConceptId>,
    /// None until the concept first passes a review (or inherits memory in a
    /// split).
    pub memory: Option<Memory>,
    pub reviews: Vec<ReviewRecord>,
    /// Linked documents, with a note on how each relates.
    pub documents: BTreeMap<DocumentId, String>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Link {
    pub prerequisite: bool,
    pub encompassing: f64,
}

#[derive(Clone, Debug)]
pub struct ReviewRecord {
    pub at: DateTime<Utc>,
    pub score: f64,
    pub question: String,
    pub answer: String,
    pub notes: String,
    /// Repetitions this review added to the concept itself, after the
    /// early-repetition discount.
    pub applied: f64,
    /// Implicit credit (or penalty) it sent to other concepts.
    pub propagated: Vec<(ConceptId, f64)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConceptStatus {
    /// Not learned yet: no review has passed.
    New,
    /// Learned and on a review schedule.
    Learning,
    /// Marked as already known: never reviewed.
    Known,
    Retired,
}

impl Concept {
    pub fn status(&self) -> ConceptStatus {
        if self.retired {
            ConceptStatus::Retired
        } else if self.known {
            ConceptStatus::Known
        } else if self.memory.is_some() {
            ConceptStatus::Learning
        } else {
            ConceptStatus::New
        }
    }

    /// Learned enough to count as a satisfied prerequisite.
    pub fn is_learned(&self) -> bool {
        self.known || self.memory.is_some()
    }

    /// The last explicit review failed: the concept takes no implicit credit
    /// until it is passed explicitly again.
    pub fn needs_explicit_review(&self) -> bool {
        self.reviews
            .last()
            .is_some_and(|review| review.score < PASSING_SCORE)
    }

    /// When the concept is due, if it is on a review schedule.
    pub fn due_at(&self) -> Option<DateTime<Utc>> {
        match self.status() {
            ConceptStatus::Learning => self.memory.as_ref().map(Memory::due_at),
            _ => None,
        }
    }

    pub fn is_due(&self, now: DateTime<Utc>) -> bool {
        self.due_at().is_some_and(|due| due <= now)
    }
}

/// Concept and document ids: lowercase a-z/0-9 words joined by single '-'.
/// Safe to put in a URL as is.
pub fn check_slug(id: &str) -> Result<(), String> {
    let valid = !id.is_empty()
        && id.split('-').all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        });
    if valid {
        Ok(())
    } else {
        Err(format!(
            "'{id}' is not a valid id: use lowercase a-z and 0-9 words joined by single '-'"
        ))
    }
}

fn check_unit(name: &str, value: f64) -> Result<(), String> {
    if (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(format!("{name} must be between 0 and 1, got {value}"))
    }
}

enum Direction {
    /// From advanced to the concepts it encompasses.
    Down,
    /// From basic to the concepts that encompass it.
    Up,
}

impl KnowledgeGraph {
    pub fn active_concepts(&self) -> impl Iterator<Item = (&ConceptId, &Concept)> {
        self.concepts.iter().filter(|(_, c)| !c.retired)
    }

    pub fn concept(&self, id: &str) -> Result<&Concept, String> {
        match self.concepts.get(id) {
            Some(concept) if !concept.retired => Ok(concept),
            Some(_) => Err(format!("concept '{id}' is retired")),
            None => Err(format!("no concept '{id}'")),
        }
    }

    fn concept_mut(&mut self, id: &str) -> Result<&mut Concept, String> {
        self.concept(id)?;
        Ok(self.concepts.get_mut(id).expect("checked above"))
    }

    /// Links where `id` is the advanced concept: what it builds on.
    pub fn links_down<'a>(&'a self, id: &'a str) -> impl Iterator<Item = (&'a ConceptId, Link)> {
        self.links
            .iter()
            .filter(move |((advanced, _), _)| advanced == id)
            .map(|((_, basic), link)| (basic, *link))
    }

    /// Links where `id` is the basic concept: what builds on it.
    pub fn links_up<'a>(&'a self, id: &'a str) -> impl Iterator<Item = (&'a ConceptId, Link)> {
        self.links
            .iter()
            .filter(move |((_, basic), _)| basic == id)
            .map(|((advanced, _), link)| (advanced, *link))
    }

    pub fn prerequisites<'a>(&'a self, id: &'a str) -> impl Iterator<Item = &'a ConceptId> {
        self.links_down(id)
            .filter(|(_, link)| link.prerequisite)
            .map(|(basic, _)| basic)
    }

    /// Whether `to` is reachable from `from` going down links that satisfy
    /// `follow`.
    fn reaches_down(&self, from: &str, to: &str, follow: impl Fn(&Link) -> bool) -> bool {
        let mut seen = BTreeSet::new();
        let mut stack = vec![from.to_string()];
        while let Some(id) = stack.pop() {
            if id == to {
                return true;
            }
            if seen.insert(id.clone()) {
                stack.extend(
                    self.links_down(&id)
                        .filter(|(_, link)| follow(link))
                        .map(|(basic, _)| basic.clone()),
                );
            }
        }
        false
    }

    /// The strongest encompassing path weight from `from` to every concept it
    /// reaches in `direction` (the product of the weights along the path).
    fn flow(&self, from: &str, direction: Direction) -> BTreeMap<ConceptId, f64> {
        let mut best: BTreeMap<ConceptId, f64> = BTreeMap::new();
        let mut stack = vec![(from.to_string(), 1.0)];
        while let Some((id, weight)) = stack.pop() {
            let next: Vec<(ConceptId, Link)> = match direction {
                Direction::Down => self.links_down(&id).map(|(c, l)| (c.clone(), l)).collect(),
                Direction::Up => self.links_up(&id).map(|(c, l)| (c.clone(), l)).collect(),
            };
            for (other, link) in next {
                let w = weight * link.encompassing;
                if w >= MIN_FLOW_WEIGHT && best.get(&other).is_none_or(|&b| w > b) {
                    best.insert(other.clone(), w);
                    stack.push((other, w));
                }
            }
        }
        best
    }

    /// The strongest encompassing path weight from `advanced` to each concept
    /// below it: how much of a repetition reviewing `advanced` sends there.
    pub fn encompassed_by(&self, advanced: &str) -> BTreeMap<ConceptId, f64> {
        self.flow(advanced, Direction::Down)
    }

    /// Concepts that are not learned but whose prerequisites all are: what
    /// can be learned next.
    pub fn frontier(&self) -> Vec<&ConceptId> {
        self.active_concepts()
            .filter(|(id, concept)| {
                !concept.is_learned()
                    && self
                        .prerequisites(id)
                        .all(|basic| self.concepts[basic].is_learned())
            })
            .map(|(id, _)| id)
            .collect()
    }

    fn add_concept(&mut self, new: &NewConcept, at: DateTime<Utc>) -> Result<(), String> {
        check_slug(&new.id)?;
        if self.concepts.contains_key(&new.id) {
            return Err(format!("concept id '{}' is already taken", new.id));
        }
        self.concepts.insert(
            new.id.clone(),
            Concept {
                title: new.title.clone(),
                description: new.description.clone(),
                created_at: at,
                known: false,
                retired: false,
                split_from: None,
                split_into: Vec::new(),
                memory: None,
                reviews: Vec::new(),
                documents: BTreeMap::new(),
            },
        );
        Ok(())
    }

    fn set_link(&mut self, advanced: &str, basic: &str, link: Link) -> Result<(), String> {
        self.concept(advanced)?;
        self.concept(basic)?;
        check_unit("encompassing", link.encompassing)?;
        if advanced == basic {
            return Err("a concept can't link to itself".to_string());
        }
        let key = (advanced.to_string(), basic.to_string());
        if !link.prerequisite && link.encompassing == 0.0 {
            self.links.remove(&key);
            return Ok(());
        }
        if link.prerequisite && self.reaches_down(basic, advanced, |l| l.prerequisite) {
            return Err(format!(
                "'{advanced}' is already (transitively) a prerequisite of '{basic}'"
            ));
        }
        if link.encompassing > 0.0 && self.reaches_down(basic, advanced, |l| l.encompassing > 0.0) {
            return Err(format!(
                "'{basic}' already (transitively) encompasses '{advanced}'"
            ));
        }
        self.links.insert(key, link);
        Ok(())
    }

    /// Apply credit to every learned concept that `from` reaches in
    /// `direction`, weighted by path weight. Positive credit skips concepts
    /// awaiting an explicit re-review.
    fn propagate(&mut self, from: &str, credit: f64, at: DateTime<Utc>) -> Vec<(ConceptId, f64)> {
        let direction = if credit > 0.0 {
            Direction::Down
        } else {
            Direction::Up
        };
        let mut propagated = Vec::new();
        for (id, weight) in self.flow(from, direction) {
            let concept = self
                .concepts
                .get_mut(&id)
                .expect("links are between concepts");
            if concept.known || (credit > 0.0 && concept.needs_explicit_review()) {
                continue;
            }
            let Some(memory) = concept.memory.as_mut() else {
                continue;
            };
            let applied = memory.credit(credit * weight, at);
            if applied.abs() >= 1e-3 {
                propagated.push((id, applied));
            }
        }
        propagated
    }

    /// Apply one event, or explain why it is invalid (leaving the graph
    /// unchanged).
    pub fn apply(&mut self, event: &KnowledgeEvent, at: DateTime<Utc>) -> Result<(), String> {
        match event {
            KnowledgeEvent::AddConcept {
                id,
                title,
                description,
            } => self.add_concept(
                &NewConcept {
                    id: id.clone(),
                    title: title.clone(),
                    description: description.clone(),
                },
                at,
            ),
            KnowledgeEvent::EditConcept {
                id,
                title,
                description,
            } => {
                let concept = self.concept_mut(id)?;
                if let Some(title) = title {
                    concept.title = title.clone();
                }
                if let Some(description) = description {
                    concept.description = description.clone();
                }
                Ok(())
            }
            KnowledgeEvent::SetLink {
                advanced,
                basic,
                prerequisite,
                encompassing,
            } => self.set_link(
                advanced,
                basic,
                Link {
                    prerequisite: *prerequisite,
                    encompassing: encompassing.0,
                },
            ),
            KnowledgeEvent::SplitConcept { id, into } => {
                let parent = self.concept(id)?.clone();
                if into.is_empty() {
                    return Err("a split needs at least one part".to_string());
                }
                let prerequisites: Vec<ConceptId> = self.prerequisites(id).cloned().collect();
                // Work on a copy so a bad part leaves the graph untouched.
                let mut next = self.clone();
                for part in into {
                    next.add_concept(part, at)?;
                    let concept = next.concepts.get_mut(&part.id).expect("just added");
                    concept.split_from = Some(id.clone());
                    concept.known = parent.known;
                    concept.memory = parent.memory.clone();
                    concept.documents = parent.documents.clone();
                    // The parent's foundations are each part's foundations;
                    // the parent becomes a composite built from its parts.
                    for basic in &prerequisites {
                        next.set_link(
                            &part.id,
                            basic,
                            Link {
                                prerequisite: true,
                                encompassing: 0.0,
                            },
                        )?;
                    }
                    next.set_link(
                        id,
                        &part.id,
                        Link {
                            prerequisite: true,
                            encompassing: 1.0,
                        },
                    )?;
                }
                next.concepts
                    .get_mut(id)
                    .expect("checked above")
                    .split_into
                    .extend(into.iter().map(|part| part.id.clone()));
                *self = next;
                Ok(())
            }
            KnowledgeEvent::SetKnown { id, known } => {
                self.concept_mut(id)?.known = *known;
                Ok(())
            }
            KnowledgeEvent::Review {
                id,
                score,
                question,
                answer,
                notes,
            } => {
                check_unit("score", score.0)?;
                let concept = self.concept_mut(id)?;
                if concept.known {
                    return Err(format!(
                        "'{id}' is marked known; unmark it before reviewing it"
                    ));
                }
                let credit = review_credit(score.0);
                // Only a pass starts a concept's schedule: a failed first
                // attempt leaves it unlearned (and still on the frontier).
                let applied = match (&mut concept.memory, credit > 0.0) {
                    (Some(memory), _) => memory.credit(credit, at),
                    (None, true) => concept.memory.insert(Memory::fresh(at)).credit(credit, at),
                    (None, false) => 0.0,
                };
                let propagated = self.propagate(id, credit, at);
                self.concepts
                    .get_mut(id)
                    .expect("checked above")
                    .reviews
                    .push(ReviewRecord {
                        at,
                        score: score.0,
                        question: question.clone(),
                        answer: answer.clone(),
                        notes: notes.clone(),
                        applied,
                        propagated,
                    });
                Ok(())
            }
            KnowledgeEvent::RetireConcept { id } => {
                self.concept_mut(id)?.retired = true;
                self.links
                    .retain(|(advanced, basic), _| advanced != id && basic != id);
                Ok(())
            }
            KnowledgeEvent::SetDocumentLink {
                document,
                concept,
                linked,
                note,
            } => {
                check_slug(document)?;
                let concept = self.concept_mut(concept)?;
                if *linked {
                    concept.documents.insert(document.clone(), note.clone());
                } else {
                    concept.documents.remove(document);
                }
                Ok(())
            }
        }
    }

    /// Concepts due for review at `now`.
    pub fn due(&self, now: DateTime<Utc>) -> Vec<&ConceptId> {
        self.active_concepts()
            .filter(|(_, concept)| concept.is_due(now))
            .map(|(id, _)| id)
            .collect()
    }

    /// Whether an implicit repetition of `weight` would lift `id` back above
    /// the due level at `now` (so reviewing something that encompasses it
    /// covers its review).
    pub(crate) fn knocked_out_by(&self, id: &str, weight: f64, now: DateTime<Utc>) -> bool {
        let concept = &self.concepts[id];
        !concept.needs_explicit_review()
            && concept
                .memory
                .as_ref()
                .is_some_and(|memory| memory.memory_at(now) + weight > DUE_MEMORY)
    }
}

impl weapon::AppState for KnowledgeGraph {
    type Event = KnowledgeEvent;
    type Partial = KnowledgeGraph;

    fn process_event(
        mut graph: Self::Partial,
        _context: &(),
        event: &Timestamped<KnowledgeEvent>,
    ) -> Self::Partial {
        // Writers validate against the folded graph before appending, so an
        // invalid event means two devices raced; skip it rather than fail the
        // whole fold.
        if let Err(e) = graph.apply(&event.event, event.timestamp) {
            log::warn!("skipping invalid knowledge event: {e}");
        }
        graph
    }

    fn finalize(graph: Self::Partial, _context: &()) -> Self {
        graph
    }
}
