//! Publish the chapters `build_book` wrote to the Supabase book tables, upserting so
//! reruns are idempotent. `cargo run -p generate-data --bin publish_book -- --check`
//! only prints what would be published.
use anyhow::{Context, ensure};
use clap::Parser;
use language_utils::book::{Book, BookChapter};
use serde_json::{Value, json};
use std::{collections::BTreeSet, path::PathBuf, time::Duration};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "books")]
    books: PathBuf,
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

/// Uploads the book's cover (`cover.webp` beside the build directory) to the packs
/// bucket under a content-hashed key, so a changed cover never meets a stale cached one.
async fn upload_cover(
    http: &reqwest::Client,
    path: &std::path::Path,
    slug: &str,
) -> anyhow::Result<String> {
    let bytes = std::fs::read(path)?;
    let key = format!(
        "books/{slug}/cover-{:016x}.webp",
        xxhash_rust::xxh3::xxh3_64(&bytes)
    );
    let var = |name| std::env::var(name).with_context(|| format!("{name} required"));
    let bucket = rusty_s3::Bucket::new(
        format!(
            "https://{}.r2.cloudflarestorage.com",
            var("CLOUDFLARE_ACCOUNT_ID")?
        )
        .parse()?,
        rusty_s3::UrlStyle::Path,
        "yap-packs",
        "auto",
    )?;
    let credentials =
        rusty_s3::Credentials::new(var("R2_ACCESS_KEY_ID")?, var("R2_SECRET_ACCESS_KEY")?);
    let url = rusty_s3::S3Action::sign(
        &bucket.put_object(Some(&credentials), &key),
        Duration::from_secs(600),
    );
    http.put(url)
        .header(reqwest::header::CONTENT_TYPE, "image/webp")
        .header(
            reqwest::header::CACHE_CONTROL,
            "public, max-age=31536000, immutable",
        )
        .body(bytes)
        .send()
        .await
        // A presigned URL is a temporary write capability, not log data.
        .map_err(reqwest::Error::without_url)?
        .error_for_status()
        .map_err(reqwest::Error::without_url)?;
    Ok(key)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let args = Args::parse();
    let catalog = if args.check {
        None
    } else {
        Some(Catalog {
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()?,
            url: std::env::var("SUPABASE_URL")
                .context("SUPABASE_URL required")?
                .trim_end_matches('/')
                .into(),
            key: std::env::var("SUPABASE_SERVICE_ROLE_KEY")
                .context("SUPABASE_SERVICE_ROLE_KEY required")?,
        })
    };
    for entry in generate_data::reader_books::discover(&args.books)? {
        let (book, chapters) = load(&entry.build())?;
        entry.validate(&book)?;
        ensure!(
            entry.translate
                || chapters
                    .iter()
                    .all(|c| c.language == book.original_language && !c.machine_translated),
            "public-domain books must only publish original editions"
        );
        let teasers: std::collections::BTreeMap<String, String> = match std::fs::read(
            entry.build().join("teasers.json"),
        ) {
            Ok(bytes) => serde_json::from_slice(&bytes)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                println!(
                    "{}: no teasers built; publishing chapter texts only, leaving existing teasers unchanged",
                    book.slug
                );
                Default::default()
            }
            Err(error) => return Err(error.into()),
        };
        ensure!(
            teasers.values().all(|t| !t.trim().is_empty()),
            "empty teaser"
        );
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
            continue;
        }
        let catalog = catalog.as_ref().unwrap();
        let cover_path = entry.directory.join("cover.webp");
        let cover = if cover_path.is_file() {
            Some(upload_cover(&catalog.http, &cover_path, &book.slug).await?)
        } else {
            None
        };
        // IDs are allocated only on insert, retained on conflict. NULLS NOT DISTINCT
        // lets PostgREST infer this unique constraint for official (ownerless) books.
        let rows = catalog
            .upsert(
                "books",
                "slug,owner_id",
                json!({
                    "slug":book.slug,"owner_id":null,"title":book.title,"author":book.author,
                    "original_language":book.original_language,"copyright":book.copyright,
                    "cover":cover
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
        let teasers: Vec<_> = teasers
            .into_iter()
            .map(
                |(language, teaser)| json!({"book_id":book_id,"language":language,"teaser":teaser}),
            )
            .collect();
        if !teasers.is_empty() {
            catalog
                .upsert("book_teasers", "book_id,language", json!(teasers))
                .await?;
        }
    }
    Ok(())
}
