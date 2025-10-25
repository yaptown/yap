mod classify;
mod utils;

use anyhow::{Context, anyhow};
use classify::{
    SentenceClassification, clean_sentence_with_llm, get_classifier, get_corrector,
    parse_dependencies_with_llm,
};
use futures::StreamExt;
use language_utils::{Language, NlpAnalyzedSentence};
use rand::prelude::IndexedRandom;
use sentence_sampler::sample_to_target;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write as _};
use std::path::PathBuf;
use std::sync::LazyLock;
use tysm::chat_completions::ChatClient;
use utils::{ValidationResult, validate_and_fix_whitespace};

static CHAT_CLIENT: LazyLock<ChatClient> = LazyLock::new(|| {
    ChatClient::from_env("gpt-5")
        .unwrap()
        .with_cache_directory("./.cache")
        .with_service_tier("flex")
});

static CHAT_CLIENT_MINI: LazyLock<ChatClient> = LazyLock::new(|| {
    ChatClient::from_env("gpt-5-mini")
        .unwrap()
        .with_cache_directory("./.cache")
        .with_service_tier("flex")
});

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    let command = &args[1];

    match command.as_str() {
        "print" => {
            if args.len() < 4 {
                eprintln!("Error: 'print' command requires a language code and count");
                eprintln!("Usage: clean-nlp-data print <language_code> <count>");
                eprintln!("Example: clean-nlp-data print fra 40");
                return Err(anyhow!("Missing arguments for 'print' command"));
            }

            let language_code = &args[2];
            let count: usize = args[3]
                .parse()
                .context("Failed to parse count as a number")?;

            let language = parse_language_code(language_code)?;

            println!("Loading NLP data for {language:?}...");
            let mut nlp_sentences = load_nlp_sentences(language)?;
            println!("Loaded {} sentences", nlp_sentences.len());

            // Apply corrections and filter out suspicious sentences
            let corrector = get_corrector(language);
            let classifier = get_classifier(language);

            let mut corrections_count = 0;
            let mut suspicious_count = 0;

            for sentence in &mut nlp_sentences {
                let correction_result = corrector.correct(sentence);
                if correction_result.corrected {
                    corrections_count += 1;
                }
            }

            let unknown_sentences: Vec<_> = nlp_sentences
                .into_iter()
                .filter(|sentence| {
                    let classification = classifier.classify(sentence);
                    match classification {
                        SentenceClassification::Unknown => true,
                        SentenceClassification::Suspicious { .. } => {
                            suspicious_count += 1;
                            false
                        }
                    }
                })
                .collect();

            println!("Applied {corrections_count} corrections");
            println!("Filtered out {suspicious_count} suspicious sentences");
            println!("\nShowing {count} random sentences:\n");

            print_random_sentences(&unknown_sentences, count);
        }
        "clean" => {
            clean_all_languages().await?;
        }
        _ => {
            eprintln!("Error: Unknown command '{command}'");
            print_usage();
            return Err(anyhow!("Unknown command"));
        }
    }

    Ok(())
}

fn print_usage() {
    eprintln!("Usage: clean-nlp-data <command> [args...]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  print <language_code> <count>  Print random sentences from the dataset");
    eprintln!("  clean                          Clean NLP data with LLM for all languages");
    eprintln!();
    eprintln!("Language codes (ISO 639-3):");
    eprintln!("  fra - French");
    eprintln!("  deu - German");
    eprintln!("  spa - Spanish");
    eprintln!("  eng - English");
    eprintln!("  kor - Korean");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  clean-nlp-data print fra 40    # Print 40 random French sentences");
    eprintln!("  clean-nlp-data print deu 20    # Print 20 random German sentences");
    eprintln!("  clean-nlp-data clean           # Clean NLP data with LLM");
}

fn parse_language_code(code: &str) -> anyhow::Result<Language> {
    match code.to_lowercase().as_str() {
        "fra" => Ok(Language::French),
        "deu" => Ok(Language::German),
        "spa" => Ok(Language::Spanish),
        "eng" => Ok(Language::English),
        "kor" => Ok(Language::Korean),
        _ => Err(anyhow!(
            "Unknown language code '{}'. Supported codes: fra, deu, spa, eng, kor",
            code
        )),
    }
}

fn load_nlp_sentences(language: Language) -> anyhow::Result<Vec<NlpAnalyzedSentence>> {
    let nlp_file_path = PathBuf::from(format!(
        "./out/{}/target_language_sentences_nlp.jsonl",
        language.iso_639_3()
    ));

    let file = File::open(&nlp_file_path)
        .context(format!("Failed to open NLP file: {nlp_file_path:?}"))?;
    let reader = BufReader::new(file);

    let sentences: Vec<NlpAnalyzedSentence> = reader
        .lines()
        .enumerate()
        .map(|(idx, line)| {
            let line = line.context(format!("Failed to read line {idx}"))?;
            serde_json::from_str::<NlpAnalyzedSentence>(&line)
                .context(format!("Failed to deserialize line {idx}: {line}"))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(sentences)
}

fn print_random_sentences(sentences: &[NlpAnalyzedSentence], count: usize) {
    let mut rng = rand::rng();
    let sample_size = count.min(sentences.len());

    let sampled: Vec<_> = sentences.choose_multiple(&mut rng, sample_size).collect();

    for (i, sentence) in sampled.iter().enumerate() {
        if i > 0 {
            println!("\n======================================================================\n");
        }

        println!("Input: {}", sentence.sentence);
        println!("{}", "-".repeat(50));
        println!("Output:");

        for (idx, token) in sentence.doc.iter().enumerate() {
            println!("{}\t{}\t{:?}\t{}", idx, token.text, token.pos, token.lemma);
        }
    }
}

async fn clean_all_languages() -> anyhow::Result<()> {
    let languages = vec![
        Language::French,
        Language::German,
        Language::Spanish,
        Language::English,
        Language::Korean,
    ];

    for language in languages {
        println!("\n=== Cleaning {language:?} ===");
        clean_language_with_llm(language).await?;
    }

    Ok(())
}

async fn clean_language_with_llm(language: Language) -> anyhow::Result<()> {
    // We probably should get at least 10_000 samples per language to get good coverage.
    // Bare minimum to get a usable result is probably around 1_500.
    const SAMPLE_SIZE: usize = 2_000;

    println!("Loading NLP data for {language:?}...");
    let sentences = load_nlp_sentences(language)?;
    println!("Loaded {} sentences", sentences.len());

    let sampled_sentences = sample_to_target(sentences, SAMPLE_SIZE, |s: &NlpAnalyzedSentence| {
        s.sentence.clone()
    });

    println!("Sampled {} sentences", sampled_sentences.len());

    // Classify each sentence to get suspicious reasons
    let classifier = get_classifier(language);
    let corrector = get_corrector(language);
    let classified_sentences: Vec<_> = sampled_sentences
        .into_iter()
        .map(|mut sentence| {
            let classification = classifier.classify(&sentence);
            let suspicious_reason = match classification {
                SentenceClassification::Suspicious { reasons } => reasons,
                SentenceClassification::Unknown => vec![],
            };
            corrector.correct(&mut sentence);
            (sentence, suspicious_reason)
        })
        .collect();

    // Clean each sentence with LLM
    let cleaned_results = futures::stream::iter(classified_sentences.into_iter().enumerate())
        .map(|(i, (sentence, suspicious_reasons))| async move {
            if i % 25 == 0 {
                println!(
                    "  [{}/{}] (${cost:.2})",
                    i,
                    SAMPLE_SIZE,
                    cost = CHAT_CLIENT.cost().unwrap()
                );
            }

            let result =
                clean_sentence_with_llm(language, &sentence, suspicious_reasons, &CHAT_CLIENT)
                    .await;
            (sentence, result)
        })
        .buffer_unordered(50)
        .collect::<Vec<_>>()
        .await;

    // Write results to file
    let output_dir = PathBuf::from("./out");
    std::fs::create_dir_all(&output_dir).context("Failed to create output directory")?;

    let output_file = output_dir.join(format!("cleaned_{}.jsonl", language.iso_639_3()));
    let file = File::create(&output_file)
        .context(format!("Failed to create output file: {output_file:?}"))?;
    let mut writer = BufWriter::new(file);

    let mut skipped_count = 0;
    let mut auto_fixed_count = 0;

    // Validate and collect successfully cleaned sentences
    let mut validated_results = Vec::new();

    for (original_sentence, mut result) in cleaned_results {
        // Validate that the LLM response matches the original text
        if let Ok(ref mut corrected_tokens) = result {
            match validate_and_fix_whitespace(&original_sentence.sentence, corrected_tokens) {
                ValidationResult::Valid => {
                    // No issues, continue
                }
                ValidationResult::AutoFixed => {
                    auto_fixed_count += 1;
                    // Continue with the auto-fixed version
                }
                ValidationResult::Invalid {
                    original,
                    reconstructed,
                } => {
                    println!(
                        "WARNING: Skipping sentence due to text mismatch:\n  Original:      '{original}'\n  Reconstructed: '{reconstructed}'"
                    );
                    skipped_count += 1;
                    continue;
                }
            }
            validated_results.push((original_sentence, result.unwrap()));
        }
    }

    println!("\n=== Pass 2: Adding dependency information ===");

    // Second pass: Add dependency information
    let results_with_deps = futures::stream::iter(validated_results.into_iter().enumerate())
        .map(|(i, (original_sentence, corrected_tokens))| async move {
            if i % 25 == 0 {
                println!(
                    "  [{}/{}] Parsing dependencies (${cost:.2})",
                    i,
                    SAMPLE_SIZE,
                    cost = CHAT_CLIENT_MINI.cost().unwrap()
                );
            }

            let dep_result = parse_dependencies_with_llm(
                language,
                &original_sentence.sentence,
                &corrected_tokens,
                &CHAT_CLIENT_MINI,
            )
            .await;

            (original_sentence, corrected_tokens, dep_result)
        })
        .buffer_unordered(50)
        .collect::<Vec<_>>()
        .await;

    // Write results to file
    for (original_sentence, corrected_tokens, dep_result) in results_with_deps {
        // Combine token info with dependency info
        let tokens: Vec<_> = if let Ok(dep_response) = dep_result {
            corrected_tokens
                .into_iter()
                .enumerate()
                .map(|(i, token)| {
                    // Find matching dependency info
                    let dep_info = dep_response.dependencies.iter().find(|d| d.index == i + 1);

                    if let Some(dep) = dep_info {
                        serde_json::json!({
                            "text": token.text,
                            "whitespace": token.whitespace,
                            "pos": token.pos,
                            "lemma": token.lemma,
                            "dep": dep.dependency,
                            "head": dep.head,
                        })
                    } else {
                        // Dependency info missing, just include token info
                        serde_json::json!({
                            "text": token.text,
                            "whitespace": token.whitespace,
                            "pos": token.pos,
                            "lemma": token.lemma,
                        })
                    }
                })
                .collect()
        } else {
            // Dependency parsing failed, just include token info
            corrected_tokens
                .into_iter()
                .map(|token| {
                    serde_json::json!({
                        "text": token.text,
                        "whitespace": token.whitespace,
                        "pos": token.pos,
                        "lemma": token.lemma,
                    })
                })
                .collect()
        };

        let output = serde_json::json!({
            "sentence": original_sentence.sentence,
            "tokens": tokens,
        });
        writeln!(writer, "{}", serde_json::to_string(&output)?)
            .context("Failed to write to output file")?;
    }

    writer.flush().context("Failed to flush writer")?;

    println!("Results written to: {}", output_file.display());
    if auto_fixed_count > 0 {
        println!("Auto-fixed {auto_fixed_count} sentences with single-space mismatches");
    }
    if skipped_count > 0 {
        println!("Skipped {skipped_count} sentences due to text mismatches");
    }

    Ok(())
}
