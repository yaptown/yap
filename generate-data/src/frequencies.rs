use indicatif::{ProgressBar, ProgressStyle};
use language_utils::{FrequencyEntry, Heteronym, Language, Lexeme, SentenceInfo};
use std::cmp::Reverse;
use std::collections::{BTreeMap, HashSet};
use std::fs::File;
use std::io::Write;

/// Compute frequencies for the given sentences.
///
/// Low-confidence multiwords count for 30% of a high-confidence
/// multiword, except for French multiwords that start with "ne",
/// which count fully.
pub fn compute_frequencies(
    sentences: &BTreeMap<String, SentenceInfo>,
    language: Language,
    banned_words: &HashSet<Heteronym<String>>,
) -> BTreeMap<Lexeme<String>, u32> {
    let mut frequencies: BTreeMap<Lexeme<String>, f32> = BTreeMap::new();

    let pb = ProgressBar::new(sentences.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} sentences ({per_sec}, {eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    for (_sentence_str, sentence) in sentences {
        pb.inc(1);
        // Count individual words
        for word in &sentence.words {
            if let Some(heteronym) = &word.heteronym {
                if !banned_words.contains(heteronym) {
                    *frequencies
                        .entry(Lexeme::Heteronym(heteronym.clone()))
                        .or_insert(0.0) += 1.0;
                }
            }
        }

        // Count high-confidence multiword terms fully
        for term in &sentence.multiword_terms.high_confidence {
            *frequencies
                .entry(Lexeme::Multiword(term.clone()))
                .or_insert(0.0) += 1.0;
        }

        // Count low-confidence multiword terms with weighting
        for term in &sentence.multiword_terms.low_confidence {
            let weight = if language == Language::French && term.starts_with("ne ") {
                1.0
            } else {
                0.3
            };
            *frequencies
                .entry(Lexeme::Multiword(term.clone()))
                .or_insert(0.0) += weight;
        }
    }

    pb.finish();

    // Round fractional counts to integers for output
    frequencies
        .into_iter()
        .map(|(lexeme, count)| (lexeme, count.ceil() as u32))
        .collect()
}

pub fn write_frequencies_file(
    frequencies: BTreeMap<Lexeme<String>, u32>,
    output_path: &std::path::Path,
) -> anyhow::Result<()> {
    let mut frequencies: Vec<FrequencyEntry<String>> = frequencies
        .into_iter()
        .map(|(lexeme, count)| FrequencyEntry { lexeme, count })
        .collect();

    frequencies.sort_by_key(|entry| Reverse(entry.count));

    let mut file = File::create(output_path)?;

    for entry in frequencies {
        let json = serde_json::to_string(&entry)?;
        writeln!(file, "{json}")?;
    }

    Ok(())
}
