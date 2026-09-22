use crate::{ReviewDefinition, TranscribeComprehensibleSentence};
use language_utils::transcription_challenge::{PartGraded, WordGrade};
use std::collections::BTreeSet;
use yap_frontend_reducers::definition_view;

fn wrong_gram_groups(results: &[PartGraded], indices: &[Vec<usize>]) -> Vec<usize> {
    let mut seen = BTreeSet::new();
    let mut groups = vec![];
    for (part_index, result) in results.iter().enumerate() {
        if let PartGraded::AskedToTranscribe { parts, .. } = result {
            for (word_index, part) in parts.iter().enumerate() {
                if matches!(
                    part.grade,
                    WordGrade::Perfect { .. } | WordGrade::CorrectWithTypo { .. }
                ) {
                    continue;
                }
                if let Some(&group) = indices.get(part_index).and_then(|p| p.get(word_index))
                    && seen.insert(group)
                {
                    groups.push(group);
                }
            }
        }
    }
    groups
}

#[bridgerton::bridge]
pub fn get_transcription_review_definitions(
    challenge: TranscribeComprehensibleSentence,
    results: Vec<PartGraded>,
) -> Vec<ReviewDefinition> {
    wrong_gram_groups(&results, &challenge.part_gram_indices)
        .into_iter()
        .filter_map(|group| {
            challenge
                .gram_definitions_for_lookup
                .get(group)?
                .as_ref()
                .map(|definition| ReviewDefinition {
                    definition: definition_view(definition.clone()),
                    breakdown: challenge
                        .gram_breakdowns_for_lookup
                        .get(group)
                        .cloned()
                        .flatten(),
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use language_utils::{Literal, transcription_challenge::PartGradedPart};
    use yap_frontend_reducers::transcription_is_perfect;
    fn literal() -> Literal<String> {
        serde_json::from_value(serde_json::json!({"word":{"text":"chat","word_type":{"type":"Heteronym","word":"chat","lemma":"chat","pos":"NOUN"}},"whitespace":""})).unwrap()
    }
    fn result(grades: Vec<WordGrade>) -> Vec<PartGraded> {
        vec![PartGraded::AskedToTranscribe {
            parts: grades
                .into_iter()
                .map(|grade| PartGradedPart {
                    heard: literal(),
                    grade,
                })
                .collect(),
            submission: "answer".into(),
        }]
    }
    #[test]
    fn typo_is_not_perfect_but_does_not_request_a_definition() {
        let grades = result(vec![
            WordGrade::CorrectWithTypo {
                wrote: Some("caht".into()),
            },
            WordGrade::Incorrect { wrote: None },
            WordGrade::Missed {},
        ]);
        assert!(!transcription_is_perfect(grades.clone()));
        assert_eq!(wrong_gram_groups(&grades, &[vec![0, 2, 2]]), vec![2]);
        assert_eq!(wrong_gram_groups(&grades, &[vec![0, 2, 1]]), vec![2, 1]);
    }
}
