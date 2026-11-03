//! Unique atom boundaries for lemma normalization and synthetic reading variants.
use std::collections::BTreeMap;

/// Different outputs at the same input position stay ambiguous: appending an
/// identical suffix cannot make them equal again.
fn align_unique<T: Clone + Eq>(
    reading: &str,
    options: &[Vec<(String, String)>],
    initial: T,
    append: impl Fn(&T, &str) -> T,
) -> Option<T> {
    let mut states = BTreeMap::from([(0, Some(initial))]);
    for options in options {
        let mut next = BTreeMap::new();
        for (position, output) in states {
            for (input, replacement) in options {
                if reading[position..].starts_with(input) {
                    let value = output.as_ref().map(|out| append(out, replacement));
                    let existing = next
                        .entry(position + input.len())
                        .or_insert_with(|| value.clone());
                    if *existing != value {
                        *existing = None;
                    }
                }
            }
        }
        states = next;
    }
    states.remove(&reading.len()).flatten()
}

/// Lemma policy needs a unique final string, even if several atom alignments
/// agree on that string.
pub(super) fn align_atoms(reading: &str, options: &[Vec<(String, String)>]) -> Option<String> {
    align_unique(reading, options, String::new(), |out, part| {
        format!("{out}{part}")
    })
}

/// Synthetic changes need proven individual atom boundaries, not merely an
/// agreed whole reading. An ambiguous segmentation therefore stays unexpanded.
pub(super) fn aligned_atom_readings(reading: &str, options: &[Vec<String>]) -> Option<Vec<String>> {
    let options: Vec<_> = options
        .iter()
        .map(|readings| readings.iter().map(|r| (r.clone(), r.clone())).collect())
        .collect();
    align_unique(reading, &options, Vec::new(), |out, part| {
        let mut out = out.clone();
        out.push(part.into());
        out
    })
}
