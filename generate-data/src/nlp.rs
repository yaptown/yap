use anyhow::{Context, Result};
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use language_utils::Language;
use lexide::Lexide;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
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
///
/// Returns a BTreeMap mapping each input sentence to its tokenization
pub async fn process_sentences(
    sentences: Vec<String>,
    output_file: &Path,
    language: Language,
) -> Result<BTreeMap<String, Vec<lexide::Token>>> {
    // Check if language is supported
    let lexide_language = to_lexide_language(language)
        .ok_or_else(|| anyhow::anyhow!("Language {} is not yet supported by lexide", language))?;

    // Initialize lexide
    println!("Initializing lexide NLP model...");
    let lexide = Lexide::from_server("https://anchpop--lexide-gemma-3-27b-vllm-serve.modal.run")
        .context("Failed to initialize lexide")?;

    // Load already processed sentences from output file (if it exists)
    let mut already_processed: BTreeMap<String, Vec<lexide::Token>> = BTreeMap::new();
    if output_file.exists() {
        println!("Loading already processed sentences...");
        let file = std::fs::File::open(output_file)?;
        let reader = BufReader::new(file);

        for line in reader.lines().map_while(Result::ok) {
            if let Ok(tokenized) = serde_json::from_str::<TokenizedSentence>(&line) {
                already_processed.insert(tokenized.sentence, tokenized.tokens);
            }
        }
        println!(
            "Found {} already processed sentences",
            already_processed.len()
        );
    }

    // Filter out already processed sentences
    let sentences_to_process: HashSet<String> = sentences
        .iter()
        .filter(|s| !already_processed.contains_key(*s))
        .cloned()
        .collect();

    println!(
        "Total sentences: {}",
        sentences_to_process.len() + already_processed.len()
    );
    println!("Already processed: {}", already_processed.len());
    println!("To process: {}", sentences_to_process.len());

    if sentences_to_process.is_empty() {
        println!("No new sentences to process!");
        // Return only the sentences that were requested
        let result: BTreeMap<String, Vec<lexide::Token>> = sentences
            .into_iter()
            .filter_map(|s| already_processed.get(&s).map(|tokens| (s, tokens.clone())))
            .collect();
        return Ok(result);
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

    // Process all sentences concurrently with buffering and collect results
    let mut newly_processed: BTreeMap<String, Vec<lexide::Token>> = BTreeMap::new();
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
        .buffer_unordered(1200);

    // Write results as they come in and collect them in memory
    while let Some(tokenized_opt) = results.next().await {
        if let Some(tokenized) = tokenized_opt {
            let json = serde_json::to_string(&tokenized)?;
            writeln!(writer, "{json}")?;
            newly_processed.insert(tokenized.sentence, tokenized.tokens);
        }
    }

    pb.finish_with_message("Tokenization complete");

    writer.flush()?;
    println!(
        "\nTokenization complete! Output written to {}",
        output_file.display()
    );

    // Build result map containing only the requested sentences
    // Merge newly processed sentences with already processed ones
    already_processed.extend(newly_processed);

    // Filter to only return the sentences that were requested
    let result: BTreeMap<String, Vec<lexide::Token>> = sentences
        .into_iter()
        .filter_map(|s| already_processed.get(&s).map(|tokens| (s, tokens.clone())))
        .collect();

    Ok(result)
}

/// Generate NLP analyzed sentences by matching multiword terms against sentences
/// using lemma matcher (high confidence) and dependency matcher (low confidence)
pub async fn generate_nlp_sentences(
    sentences_tokenizations: BTreeMap<String, Vec<lexide::Token>>,
    multiword_terms_tokenizations: BTreeMap<String, Vec<lexide::Token>>,
    output_file: &Path,
    _language: Language,
) -> Result<()> {
    use language_utils::{Heteronym, Literal, MultiwordTerms, PartOfSpeech, SentenceInfo};
    use lexide::matching::{DependencyMatcher, LemmaMatcher, TreeNode};

    println!(
        "Processing {} sentences with {} multiword terms...",
        sentences_tokenizations.len(),
        multiword_terms_tokenizations.len()
    );

    // Build matchers for all multiword terms
    println!("Building matchers for multiword terms...");

    // For lemma matcher (high confidence), create patterns from lemmas
    let lemma_patterns: Vec<(String, Vec<&str>)> = multiword_terms_tokenizations
        .iter()
        .map(|(term_str, tokens)| {
            (
                term_str.clone(),
                tokens
                    .iter()
                    .map(|token| token.lemma.lemma.as_str())
                    .collect(),
            )
        })
        .collect();
    let lemma_matcher = LemmaMatcher::new(&lemma_patterns);

    // For dependency matcher (low confidence), create tree patterns
    let tree_patterns: Vec<(String, TreeNode)> = multiword_terms_tokenizations
        .iter()
        .filter_map(|(term_str, tokens)| {
            let tokenization = lexide::Tokenization {
                tokens: tokens.clone(),
            };
            match TreeNode::try_from(tokenization.clone()) {
                Ok(tree) => Some((term_str.clone(), tree)),
                Err(e) => {
                    eprintln!("Warning: Failed to create TreeNode for '{term_str}': {e}");
                    None
                }
            }
        })
        .collect();

    println!(
        "Created {} tree patterns from {} multiword terms",
        tree_patterns.len(),
        multiword_terms_tokenizations.len()
    );

    let dependency_matcher = DependencyMatcher::new(&tree_patterns);

    println!("Processing sentences and matching multiword terms...");

    // Open output file
    let output_file_handle = std::fs::File::create(output_file)?;
    let mut writer = std::io::BufWriter::new(output_file_handle);

    // Helper function to convert lexide::PartOfSpeech to language_utils::PartOfSpeech
    let convert_pos = |pos: lexide::pos::PartOfSpeech| -> PartOfSpeech {
        // Both enums have identical variants with the same serde renames,
        // so we can convert by serializing and deserializing
        let json = serde_json::to_string(&pos).unwrap();
        serde_json::from_str(&json).unwrap()
    };

    // Helper function to convert lexide token to Literal
    let token_to_literal = |token: &lexide::Token| -> Literal<String> {
        // Create a heteronym from the token if it's not punctuation or space
        let heteronym = if matches!(
            token.pos,
            lexide::pos::PartOfSpeech::Punct
                | lexide::pos::PartOfSpeech::Space
                | lexide::pos::PartOfSpeech::X
                | lexide::pos::PartOfSpeech::Propn
        ) {
            None
        } else {
            Some(Heteronym {
                word: token.text.text.clone(),
                lemma: token.lemma.lemma.clone(),
                pos: convert_pos(token.pos),
            })
        };

        Literal {
            text: token.text.text.clone(),
            whitespace: token.whitespace.clone(),
            heteronym,
        }
    };

    for (sent_idx, (sentence_str, tokens)) in sentences_tokenizations.iter().enumerate() {
        let tokenization = lexide::Tokenization {
            tokens: tokens.clone(),
        };

        // Find high confidence matches using lemma matcher
        let lemma_matches = lemma_matcher.find_all(&tokenization);
        let high_confidence: Vec<String> = lemma_matches
            .iter()
            .map(|m| m.matched_label.clone())
            .collect();

        // Find low confidence matches using dependency matcher
        let low_confidence: Vec<String> = if let Ok(tree) = TreeNode::try_from(tokenization.clone())
        {
            let dep_matches = dependency_matcher.find_all(&tree);

            // Debug: log matches for first few sentences
            if sent_idx < 3 && !dep_matches.is_empty() {
                eprintln!("\n=== Sentence {sent_idx}: {sentence_str} ===");
                eprintln!("Sentence tree root lemma: {}", tree.token.lemma.lemma);
                eprintln!("Sentence tree has {} children", tree.children.len());
                eprintln!("Found {} dependency matches", dep_matches.len());
            }

            dep_matches
                .iter()
                .map(|m| m.matched_label.clone())
                // Filter out high confidence matches
                .filter(|term| !high_confidence.contains(term))
                .collect()
        } else {
            Vec::new()
        };

        // Convert lexide tokens to Literals
        let words: Vec<Literal<String>> = tokens.iter().map(token_to_literal).collect();

        let sentence_info = SentenceInfo {
            words,
            multiword_terms: MultiwordTerms {
                high_confidence,
                low_confidence,
            },
        };

        // Write (sentence, SentenceInfo) tuple to output file
        let json = serde_json::to_string(&(sentence_str, &sentence_info))?;
        writeln!(writer, "{json}")?;
    }

    writer.flush()?;
    println!(
        "NLP analyzed sentences written to: {}",
        output_file.display()
    );

    Ok(())
}
