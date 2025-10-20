mod classify;

use anyhow::{Context, anyhow};
use classify::{SentenceClassification, get_classifier, get_corrector};
use language_utils::{Language, NlpAnalyzedSentence};
use rand::prelude::IndexedRandom;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write as _};
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
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

            println!("Applied {} corrections", corrections_count);
            println!("Filtered out {} suspicious sentences", suspicious_count);
            println!("\nShowing {count} random sentences:\n");

            print_random_sentences(&unknown_sentences, count);
        }
        "clean" => {
            if args.len() < 3 {
                eprintln!("Error: 'clean' command requires a language code");
                eprintln!("Usage: clean-nlp-data clean <language_code>");
                eprintln!("Example: clean-nlp-data clean fra");
                return Err(anyhow!("Missing arguments for 'clean' command"));
            }

            let language_code = &args[2];
            let language = parse_language_code(language_code)?;

            clean_nlp_data(language)?;
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
    eprintln!("  clean <language_code>          Classify and correct NLP data");
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
    eprintln!("  clean-nlp-data clean fra       # Clean French NLP data");
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

fn clean_nlp_data(language: Language) -> anyhow::Result<()> {
    println!("Loading NLP data for {language:?}...");
    let mut sentences = load_nlp_sentences(language)?;
    println!("Loaded {} sentences", sentences.len());

    let corrector = get_corrector(language);
    let classifier = get_classifier(language);

    let mut unknown_sentences = Vec::new();
    let mut suspicious_sentences = Vec::new();
    let mut total_corrections = 0;

    println!("Processing sentences...");
    for sentence in &mut sentences {
        // First, apply word corrections
        let correction_result = corrector.correct(sentence);
        if correction_result.corrected {
            total_corrections += 1;
        }

        // Then classify the sentence
        let classification = classifier.classify(sentence);
        match classification {
            SentenceClassification::Unknown => {
                unknown_sentences.push(sentence.clone());
            }
            SentenceClassification::Suspicious { reason } => {
                println!("  Suspicious: {} - {}", sentence.sentence, reason);
                suspicious_sentences.push(sentence.clone());
            }
        }
    }

    println!("\nResults:");
    println!("  Total sentences: {}", sentences.len());
    println!("  Corrections made: {total_corrections}");
    println!("  Unknown sentences: {}", unknown_sentences.len());
    println!("  Suspicious sentences: {}", suspicious_sentences.len());

    // Write output files
    let output_dir = PathBuf::from(format!("./out/{}", language.iso_639_3()));

    write_sentences(
        &output_dir.join("unknown_sentences.jsonl"),
        &unknown_sentences,
    )?;

    write_sentences(
        &output_dir.join("suspicious_sentences.jsonl"),
        &suspicious_sentences,
    )?;

    println!("\nOutput files written to:");
    println!("  {}", output_dir.join("unknown_sentences.jsonl").display());
    println!(
        "  {}",
        output_dir.join("suspicious_sentences.jsonl").display()
    );

    Ok(())
}

fn write_sentences(path: &PathBuf, sentences: &[NlpAnalyzedSentence]) -> anyhow::Result<()> {
    let file = File::create(path).context(format!("Failed to create file: {path:?}"))?;
    let mut writer = BufWriter::new(file);

    for sentence in sentences {
        let json = serde_json::to_string(sentence).context("Failed to serialize sentence")?;
        writeln!(writer, "{json}").context("Failed to write sentence")?;
    }

    writer.flush().context("Failed to flush writer")?;
    Ok(())
}
