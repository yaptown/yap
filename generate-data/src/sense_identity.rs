//! Persistent sense identities follow occurrences, not generated wording.
use crate::usage_discovery::UsageAnchor;
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) type Evidence = BTreeMap<(String, Vec<(u32, u32)>), u32>;

/// Match active old slots against all fresh first-choice occurrences. Each old
/// slot can claim only a plurality of its surviving anchors (ties are allowed).
/// Greedy overlap wins first, exact metadata breaks ties, then numeric ids do.
/// With no surviving evidence, exact metadata may reclaim an otherwise unused
/// slot (including a retired one). Metadata never overrides conflicting evidence.
pub(crate) fn assign(
    high_water: u32,
    old: impl IntoIterator<Item = (u32, impl AsRef<[UsageAnchor]>)>,
    new: impl IntoIterator<Item = u32>,
    evidence: &Evidence,
    exact: impl Fn(u32, u32) -> bool,
) -> BTreeMap<u32, u32> {
    let new: BTreeSet<_> = new.into_iter().collect();
    let old: BTreeMap<_, _> = old.into_iter().collect();
    let mut evidenced = BTreeSet::new();
    let mut candidates = Vec::new();
    for (&id, anchors) in &old {
        let mut counts = BTreeMap::<u32, usize>::new();
        // Count identities, not duplicate records of the same occurrence.
        let identities: BTreeSet<_> = anchors
            .as_ref()
            .iter()
            .map(|a| (a.sentence.clone(), a.spans.clone()))
            .collect();
        for anchor in identities {
            if let Some(&sense) = evidence.get(&anchor) {
                *counts.entry(sense).or_default() += 1;
            }
        }
        if !counts.is_empty() {
            evidenced.insert(id);
        }
        let max = counts.values().copied().max().unwrap_or(0);
        for (sense, count) in counts {
            if count == max && new.contains(&sense) {
                candidates.push((Reverse(count), Reverse(exact(id, sense)), id, sense));
            }
        }
    }
    candidates.sort();
    let mut ids = BTreeMap::new();
    let mut used = BTreeSet::new();
    for (_, _, old, new) in candidates {
        if !used.contains(&old) && !ids.contains_key(&new) {
            used.insert(old);
            ids.insert(new, old);
        }
    }
    let mut next = high_water;
    for &new in &new {
        ids.entry(new).or_insert_with(|| {
            if let Some(&id) = old
                .keys()
                .find(|&&id| !used.contains(&id) && !evidenced.contains(&id) && exact(id, new))
            {
                used.insert(id);
                id
            } else {
                next += 1;
                next
            }
        });
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    fn anchors(sentences: &[&str]) -> Vec<UsageAnchor> {
        sentences
            .iter()
            .map(|s| UsageAnchor {
                sentence: (*s).into(),
                spans: vec![(0, 1)],
                gold: true,
            })
            .collect()
    }
    fn evidence(labels: &[(&str, u32)]) -> Evidence {
        labels
            .iter()
            .map(|&(s, id)| ((s.into(), vec![(0, 1)]), id))
            .collect()
    }

    #[test]
    fn greedy_overlap_beats_exact_metadata_then_ties_are_deterministic() {
        let old = [(1, anchors(&["a", "b"])), (2, anchors(&["c"]))];
        let labels = evidence(&[("a", 7), ("b", 7), ("c", 7)]);
        let result = assign(
            2,
            old.iter().map(|(id, a)| (*id, a)),
            [7],
            &labels,
            |id, _| id == 2,
        );
        assert_eq!(result[&7], 1);
        let old = [(1, anchors(&["a"])), (2, anchors(&["c"]))];
        assert_eq!(
            assign(
                2,
                old.iter().map(|(id, a)| (*id, a)),
                [7],
                &labels,
                |id, _| id == 2
            )[&7],
            2
        );
        assert_eq!(
            assign(2, old.into_iter().rev(), [7], &labels, |_, _| false)[&7],
            1
        );
    }

    #[test]
    fn losing_a_plurality_does_not_reuse_the_slot_for_a_minority() {
        let old = [
            (1, anchors(&["a", "b", "c"])),
            (2, anchors(&["d", "e", "f"])),
        ];
        let labels = evidence(&[("a", 7), ("b", 7), ("c", 8), ("d", 7), ("e", 7), ("f", 7)]);
        let result = assign(2, old, [7, 8], &labels, |old, new| old == 1 && new == 8);
        assert_eq!(result, BTreeMap::from([(7, 2), (8, 3)]));
    }

    #[test]
    fn identical_metadata_without_shared_occurrences_reuses_the_slot() {
        let result = assign(
            4,
            [(2, anchors(&["old"]))],
            [7],
            &evidence(&[("new", 7)]),
            |_, _| true,
        );
        assert_eq!(result[&7], 2);
        let unrelated = assign(
            4,
            [(2, anchors(&["old"]))],
            [7],
            &evidence(&[("new", 7)]),
            |_, _| false,
        );
        assert_eq!(unrelated[&7], 5);
    }

    #[test]
    fn pruned_plurality_does_not_promote_surviving_minority() {
        let result = assign(
            1,
            [(1, anchors(&["a", "b", "c"]))],
            [8],
            &evidence(&[("a", 7), ("b", 7), ("c", 8)]),
            |_, _| true,
        );
        assert_eq!(result[&8], 2);
    }
}
