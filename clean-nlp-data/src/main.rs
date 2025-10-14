use anyhow::{Context, anyhow};
use language_utils::{Language, NlpAnalyzedSentence};
use rand::prelude::IndexedRandom;
use std::fs::File;
use std::io::{BufRead, BufReader};
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
            let nlp_sentences = load_nlp_sentences(language)?;
            println!("Loaded {} sentences", nlp_sentences.len());
            println!("\nShowing {count} random sentences:\n");

            print_random_sentences(&nlp_sentences, count);
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
