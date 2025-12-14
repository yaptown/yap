use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::Context;
use indexmap::IndexSet;
use language_utils::{Course, SentenceSource};

/// Default target maximum number of sentences to import from Tatoeba
const DEFAULT_TARGET_SENTENCE_COUNT: usize = 200_000;

/// Get target language sentences with optional translations and source information.
///
/// This function collects sentences from all available sources (Anki, Tatoeba, manual, songs)
/// for a given course. It returns sentences with their native language translations when available
/// and tracks which sources each sentence came from.
/// It does not perform Google Translate translations and does not write to cache files.
///
/// # Arguments
///
/// * `course` - The course defining target and native languages
///
/// # Returns
///
/// A vector of tuples: (target_sentence, optional_native_translation, source_info)
pub fn get_target_sentences(
    course: Course,
) -> anyhow::Result<Vec<(String, Option<String>, SentenceSource)>> {
    let source_data_path = PathBuf::from(format!(
        "./generate-data/data/{}",
        course.target_language.iso_639_3()
    ));

    // Load banned sentences
    let banned_sentences = load_banned_sentences(&source_data_path)?;

    // Load manual sentences (should NEVER be filtered)
    let manual_sentences = load_manual_sentences(&source_data_path)?;

    // Get all data sources
    let all_cards = crate::read_anki::get_all_cards(&source_data_path);
    let target_sentence_count = match course.target_language.writing_system() {
        language_utils::WritingSystem::Latin => DEFAULT_TARGET_SENTENCE_COUNT,
        _ => DEFAULT_TARGET_SENTENCE_COUNT / 8, // these courses are low-quality anyway, so let's save money
    };
    let tatoeba_pairs =
        crate::tatoeba::get_tatoeba_pairs(&source_data_path, course, target_sentence_count);

    // Extract target sentences from Anki cards with their native translations
    let use_native_card_side = course.native_language == language_utils::Language::English;
    let anki_sentences = all_cards.iter().flat_map(|card| {
        card.target.iter().map(|target_language_sentence| {
            let native_sentence = if use_native_card_side {
                let trimmed_native = card.english.trim();
                if trimmed_native.is_empty() {
                    None
                } else {
                    Some(trimmed_native.to_string())
                }
            } else {
                None
            };
            let mut source = SentenceSource::none();
            source.from_anki = true;
            (target_language_sentence.clone(), native_sentence, source)
        })
    });

    // Extract target sentences from Tatoeba pairs with their translations
    let tatoeba_sentences = tatoeba_pairs.iter().map(|pair| {
        let native_sentence = if course.native_language == language_utils::Language::English {
            let trimmed_native = pair.native.trim();
            if trimmed_native.is_empty() {
                None
            } else {
                Some(trimmed_native.to_string())
            }
        } else {
            None
        };
        let mut source = SentenceSource::none();
        source.from_tatoeba = true;
        (pair.target.clone(), native_sentence, source)
    });

    // Add manual sentences with source tracking
    let manual_sentences_iter = manual_sentences.into_iter().map(|sentence| {
        let mut source = SentenceSource::none();
        source.from_manual = true;
        (sentence, None, source)
    });

    // Combine all sentences
    // Apply cleanup BEFORE checking banned sentences to ensure proper matching
    let all_sentences: Vec<(String, Option<String>, SentenceSource)> = anki_sentences
        .chain(tatoeba_sentences)
        .map(|(sentence, native, source)| {
            (
                language_utils::text_cleanup::cleanup_sentence(sentence, course.target_language),
                native,
                source,
            )
        })
        .filter(|(sentence, _, source)| {
            // Never filter manual sentences
            source.is_manual() || !banned_sentences.contains(&sentence.to_lowercase())
        })
        .chain(manual_sentences_iter)
        .collect();

    // Use IndexSet to deduplicate by target sentence while preserving order
    // When there are duplicates, prefer entries with translations and merge sources
    let mut seen_targets: IndexSet<String> = IndexSet::new();
    let mut result: Vec<(String, Option<String>, SentenceSource)> = Vec::new();
    let mut target_to_index: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for (target, native, source) in all_sentences {
        if let Some(&existing_index) = target_to_index.get(&target) {
            // If we already have this target sentence:
            // 1. Merge the sources
            result[existing_index].2.merge(&source);
            // 2. Update translation if the existing entry has no translation and this one does
            if result[existing_index].1.is_none() && native.is_some() {
                result[existing_index].1 = native;
            }
        } else if seen_targets.insert(target.clone()) {
            // New target sentence
            let index = result.len();
            result.push((target.clone(), native, source));
            target_to_index.insert(target, index);
        }
    }

    // Manual sentences also need cleanup (they weren't cleaned up earlier)
    let result = result
        .into_iter()
        .map(|(sentence, native, source)| {
            if source.is_manual() {
                (
                    language_utils::text_cleanup::cleanup_sentence(
                        sentence,
                        course.target_language,
                    ),
                    native,
                    source,
                )
            } else {
                // Already cleaned up
                (sentence, native, source)
            }
        })
        .collect();

    Ok(result)
}

/// Load banned sentences from both manual and AI-generated files
fn load_banned_sentences(source_data_path: &std::path::Path) -> anyhow::Result<HashSet<String>> {
    let mut banned_sentences = HashSet::new();

    // Load manually created banned sentences
    let banned_sentences_file = source_data_path.join("banned_sentences.txt");
    if banned_sentences_file.exists() {
        let content = std::fs::read_to_string(&banned_sentences_file)
            .context("Failed to read banned sentences file")?;
        for line in content.lines() {
            let line = line.trim().to_lowercase();
            if !line.is_empty() {
                banned_sentences.insert(line);
            }
        }
    }

    // Load AI-generated banned sentences
    let ai_banned_file = source_data_path.join("banned_sentences_ai.txt");
    if ai_banned_file.exists() {
        let content = std::fs::read_to_string(&ai_banned_file)
            .context("Failed to read AI banned sentences file")?;
        for line in content.lines() {
            // Parse JSON to extract just the sentence
            if let Ok(banned_entry) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(sentence) = banned_entry.get("sentence").and_then(|s| s.as_str()) {
                    banned_sentences.insert(sentence.to_lowercase());
                }
            }
        }
    }

    Ok(banned_sentences)
}

/// Load manual sentences from the extra/manual.txt file
/// These sentences should NEVER be filtered out
fn load_manual_sentences(source_data_path: &std::path::Path) -> anyhow::Result<Vec<String>> {
    let mut manual_sentences = Vec::new();

    let manual_file = source_data_path.join("sentence-sources/extra/manual.txt");
    if manual_file.exists() {
        let content = std::fs::read_to_string(&manual_file)
            .context("Failed to read manual sentences file")?;
        for line in content.lines() {
            let line = line.trim().to_string();
            if !line.is_empty() {
                manual_sentences.push(line);
            }
        }
        println!(
            "Loaded {} manual sentences from {}",
            manual_sentences.len(),
            manual_file.display()
        );
    }

    Ok(manual_sentences)
}
