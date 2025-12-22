use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use language_utils::Course;
use sentence_sampler::sample_to_target_with_stats;

pub struct TatoebaPair {
    pub target: String,
    pub native: String,
}

#[derive(Debug)]
struct Sentence {
    lang: String,
    text: String,
}

/// Read Tatoeba master dump and extract sentence pairs matching the course languages
///
/// # Arguments
///
/// * `course` - The language course to process
/// * `target_count` - Optional maximum number of sentences to return. If None, uses DEFAULT_TARGET_SENTENCE_COUNT.
///
pub fn get_tatoeba_pairs(
    _data_path: &Path,
    course: Course,
    target_count: usize,
) -> Vec<TatoebaPair> {
    // Use the master Tatoeba dump location
    let tatoeba_dir = Path::new("./generate-data/data/tatoeba");
    let sentences_file = tatoeba_dir.join("sentences.csv");
    let links_file = tatoeba_dir.join("links.csv");

    if !sentences_file.exists() {
        eprintln!(
            "Tatoeba sentences file not found at: {}",
            sentences_file.display()
        );
        return vec![];
    }

    if !links_file.exists() {
        eprintln!("Tatoeba links file not found at: {}", links_file.display());
        return vec![];
    }

    // Get language codes
    let target_lang_code = course.target_language.iso_639_3();
    let native_lang_code = course.native_language.iso_639_3();

    // First pass: read all sentences and build a map by ID and language
    let mut sentences_by_id: HashMap<u64, Sentence> = HashMap::new();
    let mut target_sentence_ids: Vec<u64> = Vec::new();

    let file = match File::open(&sentences_file) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open sentences file: {e}");
            return vec![];
        }
    };

    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        // Skip BOM if present
        let line = if let Some(line) = line.strip_prefix('\u{feff}') {
            line
        } else {
            &line
        };

        // Parse: id [tab] lang [tab] text
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 3 {
            continue;
        }

        let id = match parts[0].parse::<u64>() {
            Ok(id) => id,
            Err(_) => continue,
        };

        let lang = parts[1].trim().to_string();
        let text = parts[2].trim().to_string();

        // Only store sentences in our target or native language
        if lang == target_lang_code || lang == native_lang_code {
            if lang == target_lang_code {
                target_sentence_ids.push(id);
            }
            sentences_by_id.insert(id, Sentence { lang, text });
        }
    }

    let file = match File::open(&links_file) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open links file: {e}");
            return vec![];
        }
    };

    // Build a map of target_id -> vec of linked sentence IDs
    let mut links_map: HashMap<u64, Vec<u64>> = HashMap::new();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        // Skip BOM if present
        let line = if let Some(line) = line.strip_prefix('\u{feff}') {
            line
        } else {
            &line
        };

        // Parse: id1 [tab] id2
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }

        let id1 = match parts[0].parse::<u64>() {
            Ok(id) => id,
            Err(_) => continue,
        };

        let id2 = match parts[1].parse::<u64>() {
            Ok(id) => id,
            Err(_) => continue,
        };

        links_map.entry(id1).or_default().push(id2);
    }

    // Third pass: create pairs from target sentences that have native translations
    let mut pairs = Vec::new();

    for target_id in target_sentence_ids {
        let target_sentence = match sentences_by_id.get(&target_id) {
            Some(s) => s,
            None => continue,
        };

        // Find linked sentences
        let linked_ids = match links_map.get(&target_id) {
            Some(ids) => ids,
            None => continue,
        };

        // Look for a native language translation
        for linked_id in linked_ids {
            if let Some(native_sentence) = sentences_by_id.get(linked_id) {
                if native_sentence.lang == native_lang_code {
                    // Apply filtering criteria
                    if !crate::target_sentences::should_include_pair(
                        &target_sentence.text,
                        &native_sentence.text,
                        course,
                    ) {
                        continue;
                    }

                    pairs.push(TatoebaPair {
                        target: target_sentence.text.clone(),
                        native: native_sentence.text.clone(),
                    });
                    break; // Only take the first native translation
                }
            }
        }
    }

    // Deduplicate based on target sentences
    let mut seen_targets = std::collections::HashSet::new();
    let unique_pairs: Vec<TatoebaPair> = pairs
        .into_iter()
        .filter(|pair| seen_targets.insert(pair.target.clone()))
        .collect();

    // Apply random sampling if we have more sentences than the target
    let (sampled_pairs, _stats) = sample_to_target_with_stats(unique_pairs, target_count, |pair| {
        (pair.target.clone(), pair.native.clone())
    });

    sampled_pairs
}
