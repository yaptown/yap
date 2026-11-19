//! Repetition compression: cover every due review with as few tasks as
//! possible, by preferring tasks whose implicit credit knocks out other due
//! reviews.

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};

use crate::{Concept, ConceptId, KnowledgeGraph};

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TaskKind {
    /// Review a concept already learned.
    Review,
    /// Learn a frontier concept for the first time; chosen because practicing
    /// it also covers due reviews.
    Lesson,
}

#[derive(Clone, Debug)]
pub struct Task {
    pub concept: ConceptId,
    pub kind: TaskKind,
    /// The due concepts this task covers, the concept itself included when it
    /// is due.
    pub covers: Vec<ConceptId>,
}

impl KnowledgeGraph {
    /// A small set of tasks that together cover every due review in scope
    /// (greedy set cover), using only in-scope concepts as tasks. Concepts
    /// whose last review failed can only be covered by reviewing them
    /// directly.
    pub fn compressed_reviews(
        &self,
        now: DateTime<Utc>,
        in_scope: impl Fn(&Concept) -> bool,
    ) -> Vec<Task> {
        let due: BTreeSet<&ConceptId> = self
            .due(now)
            .into_iter()
            .filter(|id| in_scope(&self.concepts[*id]))
            .collect();
        if due.is_empty() {
            return Vec::new();
        }
        let frontier: BTreeSet<&ConceptId> = self.frontier().into_iter().collect();
        let candidates: Vec<Task> = self
            .active_concepts()
            .filter(|(_, concept)| in_scope(concept))
            .filter_map(|(id, concept)| {
                let kind = if frontier.contains(id) {
                    TaskKind::Lesson
                } else if concept.due_at().is_some() {
                    TaskKind::Review
                } else {
                    return None;
                };
                let mut covers: Vec<ConceptId> = self
                    .encompassed_by(id)
                    .into_iter()
                    .filter(|(basic, weight)| {
                        due.contains(basic) && self.knocked_out_by(basic, *weight, now)
                    })
                    .map(|(basic, _)| basic)
                    .collect();
                if due.contains(id) {
                    covers.insert(0, id.clone());
                }
                (!covers.is_empty()).then(|| Task {
                    concept: id.clone(),
                    kind,
                    covers,
                })
            })
            .collect();

        let mut remaining = due;
        let mut tasks = Vec::new();
        while !remaining.is_empty() {
            // Most newly-covered reviews wins; ties go to reviewing a due
            // concept directly over a lesson.
            let Some(best) = candidates.iter().max_by_key(|task| {
                let fresh = task.covers.iter().filter(|c| remaining.contains(c)).count();
                (fresh, task.kind == TaskKind::Review)
            }) else {
                break;
            };
            let covers: Vec<ConceptId> = best
                .covers
                .iter()
                .filter(|c| remaining.contains(c))
                .cloned()
                .collect();
            if covers.is_empty() {
                break;
            }
            for c in &covers {
                remaining.remove(c);
            }
            tasks.push(Task {
                covers,
                ..best.clone()
            });
        }
        tasks
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use ordered_float::OrderedFloat;

    use crate::{ConceptScore, KnowledgeEvent, KnowledgeGraph, NewConcept};

    fn add(graph: &mut KnowledgeGraph, id: &str, at: chrono::DateTime<chrono::Utc>) {
        graph
            .apply(
                &KnowledgeEvent::AddConcept {
                    id: id.into(),
                    title: id.into(),
                    description: String::new(),
                    tags: Vec::new(),
                },
                at,
            )
            .unwrap();
    }

    fn review(graph: &mut KnowledgeGraph, id: &str, score: f64, at: chrono::DateTime<chrono::Utc>) {
        graph
            .apply(
                &KnowledgeEvent::Review {
                    scores: vec![ConceptScore {
                        concept: id.into(),
                        score: OrderedFloat(score),
                    }],
                    question: String::new(),
                    answer: String::new(),
                    notes: String::new(),
                    extra_credit: String::new(),
                    user_was_given: Vec::new(),
                },
                at,
            )
            .unwrap();
    }

    fn link(graph: &mut KnowledgeGraph, advanced: &str, basic: &str, encompassing: f64) {
        graph
            .apply(
                &KnowledgeEvent::SetLink {
                    advanced: advanced.into(),
                    basic: basic.into(),
                    prerequisite: true,
                    encompassing: OrderedFloat(encompassing),
                },
                chrono::DateTime::UNIX_EPOCH,
            )
            .unwrap();
    }

    /// The core FIRe loop: one review of the advanced concept covers the due
    /// review of what it encompasses, and the frontier only opens once
    /// prerequisites are learned.
    #[test]
    fn reviewing_an_advanced_concept_covers_what_it_encompasses() {
        let t0 = chrono::DateTime::UNIX_EPOCH;
        let mut graph = KnowledgeGraph::default();
        add(&mut graph, "add", t0);
        add(&mut graph, "multiply", t0);
        link(&mut graph, "multiply", "add", 1.0);
        assert_eq!(graph.frontier(), vec!["add"]);
        // A failed first attempt doesn't count as learning it.
        review(&mut graph, "add", 0.2, t0);
        assert_eq!(graph.frontier(), vec!["add"]);

        review(&mut graph, "add", 1.0, t0);
        assert_eq!(graph.frontier(), vec!["multiply"]);
        review(&mut graph, "multiply", 1.0, t0 + Duration::hours(1));

        let later = t0 + Duration::days(3);
        assert_eq!(graph.due(later).len(), 2);
        let tasks = graph.compressed_reviews(later, |_| true);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].concept, "multiply");
        assert_eq!(tasks[0].covers, vec!["multiply", "add"]);

        review(&mut graph, "multiply", 1.0, later);
        assert!(graph.due(later).is_empty());
    }

    #[test]
    fn failing_a_basic_concept_penalizes_what_encompasses_it() {
        let t0 = chrono::DateTime::UNIX_EPOCH;
        let mut graph = KnowledgeGraph::default();
        add(&mut graph, "add", t0);
        add(&mut graph, "multiply", t0);
        link(&mut graph, "multiply", "add", 1.0);
        review(&mut graph, "add", 1.0, t0);
        review(&mut graph, "multiply", 1.0, t0);

        review(&mut graph, "add", 0.0, t0 + Duration::hours(2));
        let now = t0 + Duration::hours(2);
        assert!(graph.concepts["multiply"].is_due(now));
        // A failed concept must be reviewed directly.
        let tasks = graph.compressed_reviews(now, |_| true);
        assert!(tasks.iter().any(|t| t.concept == "add"));
    }

    /// Scoring a foundation in the same review overrides the credit the
    /// advanced concept would have sent it, and stops the flow there.
    #[test]
    fn explicit_scores_block_implicit_credit() {
        let t0 = chrono::DateTime::UNIX_EPOCH;
        let mut graph = KnowledgeGraph::default();
        for id in ["add", "multiply", "exponent"] {
            add(&mut graph, id, t0);
            review(&mut graph, id, 1.0, t0);
        }
        link(&mut graph, "multiply", "add", 1.0);
        link(&mut graph, "exponent", "multiply", 1.0);
        let later = t0 + Duration::days(3);
        graph
            .apply(
                &KnowledgeEvent::Review {
                    scores: [("exponent", 1.0), ("multiply", 0.2)]
                        .map(|(concept, score)| ConceptScore {
                            concept: concept.into(),
                            score: OrderedFloat(score),
                        })
                        .to_vec(),
                    question: "2^3?".into(),
                    answer: "6".into(),
                    notes: String::new(),
                    extra_credit: String::new(),
                    user_was_given: Vec::new(),
                },
                later,
            )
            .unwrap();
        let exponent = graph.concepts["exponent"].reviews.last().unwrap();
        assert!(exponent.propagated.is_empty());
        assert_eq!(exponent.also_scored, vec![("multiply".to_string(), 0.2)]);
        assert!(graph.concepts["multiply"].needs_explicit_review());
        // 'add' got nothing: the only path to it ran through 'multiply'.
        assert_eq!(graph.concepts["add"].reviews.len(), 1);
        assert!(graph.concepts["add"].is_due(later));
    }

    /// A pass above and a fail below meet in the middle concept, which
    /// takes their sum whichever order the scores are listed in.
    #[test]
    fn score_order_does_not_matter() {
        let t0 = chrono::DateTime::UNIX_EPOCH;
        let later = t0 + Duration::days(3);
        let fold = |scores: [(&str, f64); 2]| {
            let mut graph = KnowledgeGraph::default();
            for id in ["basic", "middle", "advanced"] {
                add(&mut graph, id, t0);
                review(&mut graph, id, 1.0, t0);
            }
            link(&mut graph, "middle", "basic", 1.0);
            link(&mut graph, "advanced", "middle", 1.0);
            graph
                .apply(
                    &KnowledgeEvent::Review {
                        scores: scores
                            .map(|(concept, score)| ConceptScore {
                                concept: concept.into(),
                                score: OrderedFloat(score),
                            })
                            .to_vec(),
                        question: String::new(),
                        answer: String::new(),
                        notes: String::new(),
                        extra_credit: String::new(),
                        user_was_given: Vec::new(),
                    },
                    later,
                )
                .unwrap();
            graph.concepts["middle"].memory.clone()
        };
        assert_eq!(
            fold([("advanced", 1.0), ("basic", 0.0)]),
            fold([("basic", 0.0), ("advanced", 1.0)])
        );
    }

    #[test]
    fn v1_reviews_still_load() {
        use weapon::data_model::Event;
        let stored = r#"{"version":"V1","op":"review","id":"add","score":0.75,"question":"2+3?","answer":"5"}"#;
        let versioned: crate::VersionedKnowledgeEvent = serde_json::from_str(stored).unwrap();
        let Some(KnowledgeEvent::Review { scores, .. }) =
            KnowledgeEvent::from_versioned(&versioned, &())
        else {
            panic!("expected a review");
        };
        assert_eq!(scores[0].concept, "add");
        let resaved = serde_json::to_string(&versioned).unwrap();
        assert_eq!(
            serde_json::from_str::<crate::VersionedKnowledgeEvent>(&resaved).unwrap(),
            versioned
        );
        let other = r#"{"version":"V1","op":"set_known","id":"add","known":true}"#;
        let versioned: crate::VersionedKnowledgeEvent = serde_json::from_str(other).unwrap();
        assert!(matches!(
            KnowledgeEvent::from_versioned(&versioned, &()),
            Some(KnowledgeEvent::SetKnown { .. })
        ));
    }

    #[test]
    fn split_parts_inherit_memory_and_the_composite_covers_them() {
        let t0 = chrono::DateTime::UNIX_EPOCH;
        let mut graph = KnowledgeGraph::default();
        add(&mut graph, "fractions", t0);
        review(&mut graph, "fractions", 1.0, t0);
        graph
            .apply(
                &KnowledgeEvent::SplitConcept {
                    id: "fractions".into(),
                    into: ["adding-fractions", "multiplying-fractions"]
                        .map(|id| NewConcept {
                            id: id.into(),
                            title: id.into(),
                            description: String::new(),
                            // The past review only ever tested adding.
                            inherit_memory: id == "adding-fractions",
                        })
                        .to_vec(),
                },
                t0,
            )
            .unwrap();
        assert!(graph.concepts["adding-fractions"].is_learned());
        assert!(!graph.concepts["multiplying-fractions"].is_learned());
        let later = t0 + Duration::days(3);
        let tasks = graph.compressed_reviews(later, |_| true);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].concept, "fractions");
    }
}
