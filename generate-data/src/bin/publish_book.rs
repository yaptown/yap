//! Publish the chapters `build_book` wrote to the Supabase book tables, upserting so
//! reruns are idempotent. `cargo run -p generate-data --bin publish_book -- --check`
//! only prints what would be published.
use anyhow::{Context, ensure};
use clap::Parser;
use language_utils::book::{Book, BookChapter};
use serde_json::{Value, json};
use std::{collections::BTreeSet, path::PathBuf};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "books/mistress-of-the-waves/build")]
    build: PathBuf,
    /// Validate and print the exact publication plan without network calls or credentials.
    #[arg(long)]
    check: bool,
}

/// Loads the original's structure plus every chapter text the builder wrote. The
/// builder only writes fully translated chapters, so an edition may be partial.
fn load(directory: &std::path::Path) -> anyhow::Result<(Book, Vec<BookChapter>)> {
    let book: Book = serde_json::from_slice(&std::fs::read(directory.join("book.json"))?)?;
    let mut chapters = vec![];
    for language in std::fs::read_dir(directory)? {
        let language = language?.path();
        if !language.is_dir() {
            continue;
        }
        for file in std::fs::read_dir(language)? {
            chapters.push(serde_json::from_slice(&std::fs::read(file?.path())?)?);
        }
    }
    Ok((book, chapters))
}

struct Catalog {
    http: reqwest::Client,
    url: String,
    key: String,
}

impl Catalog {
    async fn upsert(
        &self,
        table: &str,
        conflict: &str,
        value: Value,
    ) -> anyhow::Result<Vec<Value>> {
        let response = self
            .http
            .post(format!("{}/rest/v1/{table}", self.url))
            .query(&[("on_conflict", conflict)])
            .header("apikey", &self.key)
            .bearer_auth(&self.key)
            .header(
                "Prefer",
                "resolution=merge-duplicates,return=representation",
            )
            .json(&value)
            .send()
            .await?;
        let status = response.status();
        let body = response.text().await?;
        ensure!(
            status.is_success(),
            "{table} upsert failed ({status}): {body}"
        );
        Ok(serde_json::from_str(&body)?)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let args = Args::parse();
    let (book, chapters) = load(&args.build)?;
    let languages: BTreeSet<_> = chapters.iter().map(|c| c.language.as_str()).collect();
    println!(
        "{}: {} complete chapter texts in {} languages; {} structural chapters",
        book.slug,
        chapters.len(),
        languages.len(),
        book.chapters.len()
    );
    for language in &languages {
        println!(
            "{language}: {} complete chapters",
            chapters.iter().filter(|c| c.language == *language).count()
        );
    }
    if args.check {
        println!("Read-only check: no database writes.");
        return Ok(());
    }
    let catalog = Catalog {
        http: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()?,
        url: std::env::var("SUPABASE_URL")
            .context("SUPABASE_URL required")?
            .trim_end_matches('/')
            .into(),
        key: std::env::var("SUPABASE_SERVICE_ROLE_KEY")
            .context("SUPABASE_SERVICE_ROLE_KEY required")?,
    };
    // IDs are allocated only on insert, retained on conflict. NULLS NOT DISTINCT
    // lets PostgREST infer this unique constraint for official (ownerless) books.
    let rows = catalog
        .upsert(
            "books",
            "slug,owner_id",
            json!({
                "slug":book.slug,"owner_id":null,"title":book.title,"author":book.author,
                "original_language":book.original_language,"copyright":book.copyright
            }),
        )
        .await?;
    let book_id = rows
        .first()
        .and_then(|r| r["id"].as_str())
        .context("book upsert returned no id")?;
    let mut part_start = None;
    let mut previous_part = None;
    // Each part's title per language, from whichever of its chapters that language has.
    let mut parts = std::collections::BTreeMap::new();
    for original in &book.chapters {
        if original.part != previous_part {
            part_start = original.part.as_ref().map(|_| original.position);
            previous_part = original.part.clone();
        }
        let texts: Vec<_> = chapters
            .iter()
            .filter(|c| c.position == original.position)
            .collect();

        let rows = catalog
            .upsert(
                "book_chapters",
                "book_id,position",
                json!({
                    "book_id":book_id,"position":original.position,"part_start":part_start
                }),
            )
            .await?;
        let chapter_id = rows
            .first()
            .and_then(|r| r["id"].as_str())
            .context("chapter upsert returned no id")?;
        for text in &texts {
            if let (Some(start), Some(title)) = (part_start, &text.part) {
                parts.insert((start, &text.language), title);
            }
        }
        let values: Vec<_> = texts
            .iter()
            .map(|c| {
                json!({
                    "chapter_id":chapter_id,"language":c.language,"title":c.title,
                    "paragraphs":c.paragraphs,"machine_translated":c.machine_translated
                })
            })
            .collect();
        if !values.is_empty() {
            catalog
                .upsert("chapter_texts", "chapter_id,language", json!(values))
                .await?;
        }
        println!(
            "Published chapter {} in {} languages",
            original.position,
            texts.len()
        );
    }
    let parts: Vec<_> = parts
        .into_iter()
        .map(|((start, language), title)| {
            json!({"book_id":book_id,"start":start,"language":language,"title":title})
        })
        .collect();
    if !parts.is_empty() {
        catalog
            .upsert("book_parts", "book_id,start,language", json!(parts))
            .await?;
    }
    println!("Published {} part titles", parts.len());
    Ok(())
}
