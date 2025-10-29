use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use language_utils::{Course, Language};
use sentence_sampler::sample_to_target_with_stats;

/// Default target maximum number of sentences to import from Tatoeba
const DEFAULT_TARGET_SENTENCE_COUNT: usize = 200_000;

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
    target_count: Option<usize>,
) -> Vec<TatoebaPair> {
    let target_count = target_count.unwrap_or(DEFAULT_TARGET_SENTENCE_COUNT);

    // Use the master Tatoeba dump location
    let tatoeba_dir = Path::new("./generate-data/data/tatoeba");
    let sentences_file = tatoeba_dir.join("sentences.csv");
    let links_file = tatoeba_dir.join("links.csv");

    if !sentences_file.exists() {
        eprintln!("Tatoeba sentences file not found at: {}", sentences_file.display());
        return vec![];
    }

    if !links_file.exists() {
        eprintln!("Tatoeba links file not found at: {}", links_file.display());
        return vec![];
    }

    println!("Reading Tatoeba sentences from master dump...");

    // Get language codes
    let target_lang_code = course.target_language.iso_639_1();
    let native_lang_code = course.native_language.iso_639_1();

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
            sentences_by_id.insert(
                id,
                Sentence {
                    lang,
                    text,
                },
            );
        }
    }

    println!("Found {} target language sentences in Tatoeba", target_sentence_ids.len());
    println!("Loaded {} total sentences in both languages", sentences_by_id.len());

    // Second pass: read links and build translation pairs
    println!("Reading Tatoeba translation links...");

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

        links_map.entry(id1).or_insert_with(Vec::new).push(id2);
    }

    println!("Loaded {} translation links", links_map.len());

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
                    if !should_include_pair(&target_sentence.text, &native_sentence.text, course) {
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

    println!(
        "Loaded {} filtered sentence pairs from Tatoeba",
        pairs.len()
    );

    // Deduplicate based on target sentences
    let mut seen_targets = std::collections::HashSet::new();
    let unique_pairs: Vec<TatoebaPair> = pairs
        .into_iter()
        .filter(|pair| seen_targets.insert(pair.target.clone()))
        .collect();

    println!(
        "After deduplication: {} unique sentence pairs",
        unique_pairs.len()
    );

    // Apply random sampling if we have more sentences than the target
    let (sampled_pairs, stats) = sample_to_target_with_stats(unique_pairs, target_count, |pair| {
        (pair.target.clone(), pair.native.clone())
    });

    if stats.was_sampled {
        println!(
            "Applied random sampling: {} -> {} sentence pairs (target: {})",
            stats.original_count, stats.final_count, stats.target_count
        );
    }

    sampled_pairs
}

/// Check if a sentence pair should be included based on filtering criteria
fn should_include_pair(target_sentence: &str, native_sentence: &str, course: Course) -> bool {
    // 1. Skip sentences that are too short or too long
    if target_sentence.len() < 5 || target_sentence.len() > 80 {
        return false;
    }
    if native_sentence.len() < 5 || native_sentence.len() > 80 {
        return false;
    }

    // 2. Skip sentences ending with ellipsis
    if target_sentence.ends_with("...") || native_sentence.ends_with("...") {
        return false;
    }

    // 3. Skip sentences containing ellipsis anywhere
    if target_sentence.contains("...") || native_sentence.contains("...") {
        return false;
    }

    // 4. Check if sentences are "proper" according to language rules
    if !is_proper_sentence(target_sentence, course.target_language) {
        return false;
    }

    if !is_proper_sentence(native_sentence, course.native_language) {
        return false;
    }

    // 5. Skip sentences with multiple punctuation marks
    let target_punct_count = target_sentence.matches('.').count()
        + target_sentence.matches('!').count()
        + target_sentence.matches('?').count();
    let native_punct_count = native_sentence.matches('.').count()
        + native_sentence.matches('!').count()
        + native_sentence.matches('?').count();

    if target_punct_count > 1 || native_punct_count > 1 {
        return false;
    }

    // 6. Skip sentences with numbers
    if target_sentence.chars().any(|c| c.is_numeric())
        || native_sentence.chars().any(|c| c.is_numeric())
    {
        return false;
    }

    true
}

/// Check if a sentence is "proper" - language-specific validation
fn is_proper_sentence(text: &str, language: Language) -> bool {
    if text.is_empty() {
        return false;
    }

    // Reject sentences starting with dash/hyphen
    if text.starts_with('-') || text.starts_with('—') || text.starts_with('–') {
        return false;
    }

    let first_char = text.chars().next().unwrap();
    let last_char = text.chars().last().unwrap();

    // Language-specific checks
    match language {
        Language::English | Language::French | Language::Spanish | Language::German
        | Language::Portuguese | Language::Italian => {
            // Must start with uppercase letter
            if !first_char.is_uppercase() || !first_char.is_alphabetic() {
                return false;
            }

            // Must end with period, exclamation mark, or question mark
            if last_char != '.' && last_char != '!' && last_char != '?' {
                return false;
            }
        }
        Language::Russian => {
            // Russian sentences should not contain Latin letters
            if text
                .chars()
                .any(|c| c.is_ascii_lowercase() || c.is_ascii_uppercase())
            {
                return false;
            }

            // Must start with uppercase Cyrillic letter
            if !first_char.is_uppercase() {
                return false;
            }

            // Must end with period, exclamation mark, or question mark
            if last_char != '.' && last_char != '!' && last_char != '?' {
                return false;
            }
        }
        Language::Chinese => {
            // Chinese sentences should not contain Latin letters (except maybe proper nouns)
            // But we'll be strict and reject any with Latin letters
            if text
                .chars()
                .any(|c| c.is_ascii_lowercase() || c.is_ascii_uppercase())
            {
                return false;
            }

            // Must end with Chinese or Western punctuation
            if last_char != '。' && last_char != '！' && last_char != '？'
                && last_char != '.' && last_char != '!' && last_char != '?'
            {
                return false;
            }
        }
        Language::Japanese => {
            // Japanese sentences should not contain Latin letters (except maybe proper nouns)
            // But we'll be strict and reject any with Latin letters
            if text
                .chars()
                .any(|c| c.is_ascii_lowercase() || c.is_ascii_uppercase())
            {
                return false;
            }

            // Must end with Japanese or Western punctuation
            if last_char != '。' && last_char != '！' && last_char != '？'
                && last_char != '.' && last_char != '!' && last_char != '?'
            {
                return false;
            }
        }
        Language::Korean => {
            // Korean sentences should not contain Latin letters
            if text
                .chars()
                .any(|c| c.is_ascii_lowercase() || c.is_ascii_uppercase())
            {
                return false;
            }

            // Must end with appropriate Korean punctuation or period/exclamation/question
            if last_char != '.' && last_char != '!' && last_char != '?' {
                return false;
            }
        }
    }

    // Reject sentences with quotes (often dialogue or non-standard)
    if text.contains('"') || text.contains('\'') || text.contains('"') || text.contains('"') {
        return false;
    }

    // Reject sentences with special characters that indicate non-standard text
    if text.contains('~') || text.contains('*') || text.contains('_') {
        return false;
    }

    true
}
