use anyhow::{Context, Result};
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use language_utils::Language;
use lexide::Lexide;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// Convert language_utils::Language to lexide::Language
/// Returns None if the language is not supported by lexide
fn to_lexide_language(lang: Language) -> Option<lexide::Language> {
    match lang {
        Language::French => Some(lexide::Language::French),
        Language::English => Some(lexide::Language::English),
        Language::Spanish => Some(lexide::Language::Spanish),
        Language::Korean => Some(lexide::Language::Korean),
        Language::German => Some(lexide::Language::German),
        // Languages not yet supported by lexide
        Language::Chinese
        | Language::Japanese
        | Language::Russian
        | Language::Portuguese
        | Language::Italian => None,
    }
}

/// Tokenized sentence for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenizedSentence {
    sentence: String,
    tokens: Vec<lexide::Token>,
}

/// Tokenize a list of sentences and write results to an output file
/// This function implements incremental processing - it will only tokenize sentences
/// that are not already in the output file
pub async fn process_sentences(
    sentences: Vec<String>,
    output_file: &Path,
    language: Language,
) -> Result<()> {
    // Check if language is supported
    let lexide_language = to_lexide_language(language)
        .ok_or_else(|| anyhow::anyhow!("Language {} is not yet supported by lexide", language))?;

    // Initialize lexide
    println!("Initializing lexide NLP model...");
    let lexide = Lexide::from_server("https://anchpop--lexide-gemma-3-27b-vllm-serve.modal.run")
        .context("Failed to initialize lexide")?;

    // Load already processed sentences from output file (if it exists)
    let mut already_processed = HashSet::new();
    if output_file.exists() {
        println!("Loading already processed sentences...");
        let file = std::fs::File::open(output_file)?;
        let reader = BufReader::new(file);

        for line in reader.lines().map_while(Result::ok) {
            if let Ok(tokenized) = serde_json::from_str::<TokenizedSentence>(&line) {
                already_processed.insert(tokenized.sentence);
            }
        }
        println!(
            "Found {} already processed sentences",
            already_processed.len()
        );
    }

    // Filter out already processed sentences
    let sentences_to_process: HashSet<String> = sentences
        .into_iter()
        .filter(|s| !already_processed.contains(s))
        .collect();

    println!(
        "Total sentences: {}",
        sentences_to_process.len() + already_processed.len()
    );
    println!("Already processed: {}", already_processed.len());
    println!("To process: {}", sentences_to_process.len());

    if sentences_to_process.is_empty() {
        println!("No new sentences to process!");
        return Ok(());
    }

    // Open output file in append mode
    let output_file_handle = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(output_file)?;
    let mut writer = std::io::BufWriter::new(output_file_handle);

    // Process sentences concurrently
    println!("\nTokenizing sentences...");

    let pb = ProgressBar::new(sentences_to_process.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} sentences ({per_sec}, {eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    // Process all sentences concurrently with buffering
    let mut results = futures::stream::iter(sentences_to_process.into_iter())
        .map(|sentence| {
            let lexide = &lexide;
            let pb = pb.clone();
            async move {
                let result = match lexide.analyze(&sentence, lexide_language).await {
                    Ok(tokenization) => Some(TokenizedSentence {
                        sentence,
                        tokens: tokenization.tokens,
                    }),
                    Err(e) => {
                        eprintln!("Warning: Failed to analyze sentence '{sentence}': {e}");
                        None
                    }
                };

                pb.inc(1);
                result
            }
        })
        .buffered(100);

    // Write results as they come in
    while let Some(tokenized_opt) = results.next().await {
        if let Some(tokenized) = tokenized_opt {
            let json = serde_json::to_string(&tokenized)?;
            writeln!(writer, "{json}")?;
        }
    }

    pb.finish_with_message("Tokenization complete");

    writer.flush()?;
    println!(
        "\nTokenization complete! Output written to {}",
        output_file.display()
    );

    Ok(())
}
