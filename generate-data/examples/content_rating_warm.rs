//! Temporary prewarm for the sentence-content pack regeneration; delete after regeneration.
//! Run from the data checkout, using the binary built in the feature worktree.
//! `--counts` loads cached routing candidates without submitting rating requests.

use clap::Parser;
use generate_data::content_rating::{self, Transport};
use language_utils::Language;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Parser)]
struct Args {
    #[arg(long)]
    counts: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    dotenvy::dotenv().ok();
    env_logger::init();
    // Loading must not refresh movie metadata or submit segmentation jobs in
    // the shared data checkout. Only the rating judge below may submit requests.
    generate_data::set_cache_only(true);
    let mut corpora = BTreeMap::<&str, (Language, BTreeSet<String>)>::new();
    for course in language_utils::COURSES {
        let candidates = generate_data::target_sentences::load_routing_candidates(*course).await?;
        let (_, texts) = corpora
            .entry(course.target_language.corpus_code())
            .or_insert_with(|| (course.target_language, BTreeSet::new()));
        texts.extend(candidates.sentences.into_iter().map(|(text, _, _)| text));
        texts.extend(
            candidates
                .restricted_sentences
                .into_iter()
                .map(|(text, _)| text),
        );
    }
    for (corpus, (language, texts)) in &corpora {
        let bytes: usize = texts
            .iter()
            .map(|text| content_rating::request_content_bytes(*language, text))
            .sum();
        println!(
            "{corpus}: distinct={}, message+schema bytes={bytes}",
            texts.len()
        );
    }
    if args.counts {
        return Ok(());
    }
    generate_data::set_cache_only(false);
    let runs = corpora
        .iter()
        .map(|(corpus, (language, texts))| async move {
            println!("{corpus}: submitting {} distinct sentences", texts.len());
            let ratings = content_rating::judge(
                *language,
                texts.iter().map(String::as_str),
                Transport::Batch,
            )
            .await?;
            let adult = ratings.values().filter(|&&adult| adult).count();
            println!(
                "{corpus}: adult={adult}/{} ({:.2}%)",
                ratings.len(),
                100.0 * adult as f64 / ratings.len().max(1) as f64
            );
            anyhow::Ok(())
        });
    for result in futures::future::join_all(runs).await {
        result?;
    }
    println!("Content rating warm complete.");
    Ok(())
}
