use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader};
use std::path::Path;

use language_utils::{Course, Language};
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Target maximum number of sentences to import from Tatoeba
const TARGET_SENTENCE_COUNT: usize = 200_000;

pub struct TatoebaPair {
    pub target: String,
    pub native: String,
}

/// Read Tatoeba TSV files and filter for proper sentences
pub fn get_tatoeba_pairs(data_path: &Path, course: Course) -> Vec<TatoebaPair> {
    let mut pairs = Vec::new();

    // Look for Tatoeba files in the data directory
    let tatoeba_dir = data_path.join("sentence_sources/tatoeba");
    if !tatoeba_dir.exists() {
        eprintln!("Tatoeba directory not found at: {}", tatoeba_dir.display());
        return pairs;
    }

    // Find the most recent TSV file (format: YYYY-MM-DD.tsv)
    let entries = match std::fs::read_dir(&tatoeba_dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Failed to read Tatoeba directory: {e}");
            return pairs;
        }
    };

    let mut tsv_files = Vec::new();
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        if path.extension() == Some(std::ffi::OsStr::new("tsv")) {
            tsv_files.push(path);
        }
    }

    if tsv_files.is_empty() {
        eprintln!("No TSV files found in Tatoeba directory");
        return pairs;
    }

    // Sort files by name to get the most recent one
    tsv_files.sort();
    let tsv_file = tsv_files.last().unwrap();

    println!("Reading Tatoeba file: {}", tsv_file.display());

    // Open and read the TSV file
    let file = match File::open(tsv_file) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open Tatoeba file: {e}");
            return pairs;
        }
    };

    let reader = BufReader::new(file);

    // Process each line - format: target_id\ttarget_sentence\tnative_id\tnative_sentence
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Error reading line: {e}");
                continue;
            }
        };

        // Skip BOM if present
        let line = if line.starts_with('\u{feff}') {
            &line[3..]
        } else {
            &line
        };

        // Parse TSV: columns are separated by tabs
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() != 4 {
            continue; // Skip malformed lines
        }

        let target_sentence = parts[1].trim();
        let native_sentence = parts[3].trim();

        // Apply filtering criteria

        // 1. Skip sentences that are too short or too long
        if target_sentence.len() < 5 || target_sentence.len() > 80 {
            continue;
        }
        if native_sentence.len() < 5 || native_sentence.len() > 80 {
            continue;
        }

        // 2. Skip sentences ending with ellipsis
        if target_sentence.ends_with("...") || native_sentence.ends_with("...") {
            continue;
        }

        // 3. Skip sentences containing ellipsis anywhere
        if target_sentence.contains("...") || native_sentence.contains("...") {
            continue;
        }

        // 4. Check if sentences are "proper" according to language rules
        if !is_proper_sentence(target_sentence, course.target_language) {
            continue;
        }

        // Only check native sentence if it's the language we're teaching from
        if course.native_language == Language::English
            && !is_proper_sentence(native_sentence, course.native_language)
        {
            continue;
        }

        // 5. Skip sentences with multiple punctuation marks
        let target_punct_count = target_sentence.matches('.').count()
            + target_sentence.matches('!').count()
            + target_sentence.matches('?').count();
        let native_punct_count = native_sentence.matches('.').count()
            + native_sentence.matches('!').count()
            + native_sentence.matches('?').count();

        if target_punct_count > 1 || native_punct_count > 1 {
            continue;
        }

        // 6. Skip sentences with numbers
        if target_sentence.chars().any(|c| c.is_numeric())
            || native_sentence.chars().any(|c| c.is_numeric())
        {
            continue;
        }

        pairs.push(TatoebaPair {
            target: target_sentence.to_string(),
            native: native_sentence.to_string(),
        });
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

    if unique_pairs.len() > TARGET_SENTENCE_COUNT {
        println!(
            "Applying random sampling to reduce from {} to approximately {} sentences",
            unique_pairs.len(),
            TARGET_SENTENCE_COUNT
        );

        // Calculate the probability of keeping each sentence
        let keep_probability = TARGET_SENTENCE_COUNT as f64 / unique_pairs.len() as f64;

        let sampled_pairs: Vec<TatoebaPair> = unique_pairs
            .into_iter()
            .filter(|pair| {
                // Create a deterministic seed based on the sentence content
                let mut hasher = DefaultHasher::new();
                pair.target.hash(&mut hasher);
                pair.native.hash(&mut hasher);
                let seed = hasher.finish();

                // Create RNG with this seed
                let mut rng = ChaCha8Rng::seed_from_u64(seed);

                // Keep this sentence with probability keep_probability
                rng.random::<f64>() < keep_probability
            })
            .collect();

        println!(
            "After random sampling: {} sentence pairs",
            sampled_pairs.len()
        );

        sampled_pairs
    } else {
        unique_pairs
    }
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
        Language::English | Language::French | Language::Spanish | Language::German => {
            // Must start with uppercase letter
            if !first_char.is_uppercase() || !first_char.is_alphabetic() {
                return false;
            }

            // Must end with period, exclamation mark, or question mark
            if last_char != '.' && last_char != '!' && last_char != '?' {
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
