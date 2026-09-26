//! Small reproducible live audit using the production judge.
//! Run from the workspace root: cargo run -p generate-data --example dialect_pilot
//! Fill the cache for every dialect course through the Batch API, ahead of pack builds:
//! cargo run -p generate-data --release --example dialect_pilot -- --warm
//! Full-loader count only (no dialect requests), optimized for local segmentation:
//! cargo run -p generate-data --release --example dialect_pilot -- --counts
//! Writes /tmp/dialect-pilot/{spa,por}.jsonl. Exports are a size proxy, not the
//! full loader union (which also contains restricted Pimsleur and book material).

use clap::Parser;
use generate_data::dialect::{self, Transport};
use language_utils::Language;
use rand::{SeedableRng, seq::SliceRandom};
use rand_chacha::ChaCha8Rng;
use std::{collections::BTreeMap, io::Write};

#[derive(Parser)]
#[command(
    about = "Audit dialect routing with 150 live requests per corpus plus Spanish regressions"
)]
struct Args {
    /// Count full-loader inputs in cache-only mode instead of running the pilot.
    #[arg(long)]
    counts: bool,
    /// Judge every dialect course's full input through the Batch API, filling the
    /// cache ahead of pack builds.
    #[arg(long)]
    warm: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    dotenvy::dotenv().ok();
    env_logger::init();
    if args.counts {
        generate_data::set_cache_only(true);
        for language in [
            Language::SpanishLatinAmerican,
            Language::PortugueseBrazilian,
        ] {
            let candidates =
                generate_data::target_sentences::load_routing_candidates(language_utils::Course {
                    target_language: language,
                    native_language: Language::English,
                })
                .await?;
            let distinct: std::collections::BTreeSet<_> = candidates
                .sentences
                .iter()
                .map(|(s, _, _)| s.as_str())
                .chain(
                    candidates
                        .restricted_sentences
                        .iter()
                        .map(|(s, _)| s.as_str()),
                )
                .collect();
            let bytes: usize = distinct
                .iter()
                .map(|text| dialect::request_content_bytes(language, text))
                .sum();
            println!(
                "{} full-loader: unrestricted={}, restricted={}, distinct union={} (one request each before cache); message+schema bytes={bytes}; rough input tokens={}..{} at 3–4 bytes/token; output/reasoning additional and uncertain",
                language.corpus_code(),
                candidates.sentences.len(),
                candidates.restricted_sentences.len(),
                distinct.len(),
                bytes / 4,
                bytes / 3
            );
        }
        return Ok(());
    }
    if args.warm {
        // Every course that routes by dialect, so no pack build later misses the cache.
        let mut corpora = BTreeMap::<&str, (Language, std::collections::BTreeSet<String>)>::new();
        for course in language_utils::COURSES {
            if course.target_language.sibling_dialects().is_empty() {
                continue;
            }
            let candidates =
                generate_data::target_sentences::load_routing_candidates(*course).await?;
            let (_, texts) = corpora
                .entry(course.target_language.corpus_code())
                .or_insert_with(|| (course.target_language, Default::default()));
            texts.extend(candidates.sentences.into_iter().map(|(s, _, _)| s));
            texts.extend(candidates.restricted_sentences.into_iter().map(|(s, _)| s));
        }
        let runs = corpora
            .iter()
            .map(|(corpus, (language, texts))| async move {
                println!("{corpus}: judging {} distinct sentences", texts.len());
                let labels = dialect::judge(
                    *language,
                    texts.iter().map(String::as_str),
                    Transport::Batch,
                )
                .await?;
                let mut counts = BTreeMap::<String, usize>::new();
                for judgement in labels.values() {
                    *counts
                        .entry(format!("{:?}", judgement.dialect))
                        .or_default() += 1;
                }
                println!("{corpus}: {counts:?}");
                anyhow::Ok(())
            });
        for result in futures::future::join_all(runs).await {
            result?;
        }
        return Ok(());
    }
    let out = std::path::Path::new("/tmp/dialect-pilot");
    std::fs::create_dir_all(out)?;
    for language in [
        Language::SpanishLatinAmerican,
        Language::PortugueseBrazilian,
    ] {
        let corpus = language.corpus_code();
        let path = format!("out/{corpus}/sentence_sources.jsonl");
        let mut sources = BTreeMap::<String, serde_json::Value>::new();
        let input = std::fs::read_to_string(path)?;
        for line in input.lines() {
            let (text, source): (String, serde_json::Value) = serde_json::from_str(line)?;
            sources.insert(text, source);
        }
        println!(
            "{corpus}: source-export proxy: {} rows, {} distinct sentences, {} sentence bytes (not full-loader counts)",
            input.lines().count(),
            sources.len(),
            sources.keys().map(|s| s.len()).sum::<usize>()
        );
        let mut sentences: Vec<_> = sources.keys().cloned().collect();
        sentences.shuffle(&mut ChaCha8Rng::seed_from_u64(113));
        sentences.truncate(150);
        if corpus == "spa" {
            for text in [
                "Estoy orgulloso de vosotras.",
                "Por eso podéis bailarlo.",
                "Llegáis tarde.",
                "Un caballero insiste en veros.",
            ] {
                if !sentences.iter().any(|s| s == text) {
                    sentences.push(text.to_owned());
                }
            }
        }
        let labels = dialect::judge(
            language,
            sentences.iter().map(String::as_str),
            Transport::Live,
        )
        .await?;
        let mut file = std::fs::File::create(out.join(format!("{corpus}.jsonl")))?;
        let mut examples = BTreeMap::<String, Vec<&str>>::new();
        for (index, text) in sentences.iter().enumerate() {
            let judgement = &labels[text];
            serde_json::to_writer(
                &mut file,
                &serde_json::json!({
                    "sentence": text, "source": sources.get(text),
                    "sample": if index < 150 { "random" } else { "ticket_regression" },
                    "judgement": judgement,
                }),
            )?;
            writeln!(file)?;
            examples
                .entry(format!("{:?}", judgement.dialect))
                .or_default()
                .push(text);
        }
        for (label, texts) in examples {
            println!("{corpus} {label}: {} / {}", texts.len(), sentences.len());
            for text in texts.iter().take(10) {
                println!("  {text} — {}", labels[*text].reason);
            }
        }
    }
    Ok(())
}
