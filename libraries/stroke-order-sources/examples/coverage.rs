//! Reports how much of a corpus a pack can draw: the share of writable units
//! (by occurrence and by distinct unit) with at least one glyph, and the most
//! frequent units without one.
//!
//! cargo run -p stroke-order-sources --example coverage -- <cache-dir> <lang code> <sentences.jsonl>
//!
//! The JSONL has one sentence per line, either a bare string (the
//! `target_language_sentences.jsonl` corpora under `out/`) or an object with
//! a "sentence" field. Whitespace units are ignored.
use anyhow::{Context, Result, ensure};
use language_utils::Language;
use std::{collections::HashMap, io::BufRead, path::PathBuf};

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    ensure!(
        args.len() == 3,
        "usage: coverage <cache-dir> <lang> <sentences.jsonl>"
    );
    let cache = PathBuf::from(&args[0]);
    std::fs::create_dir_all(&cache)?;
    let language = Language::from_code(&args[1]).context("unknown language code")?;
    let mut sentences = Vec::new();
    for line in std::io::BufReader::new(std::fs::File::open(&args[2])?).lines() {
        let record: serde_json::Value = serde_json::from_str(&line?)?;
        if let Some(sentence) = record.as_str().or_else(|| record["sentence"].as_str()) {
            sentences.push(sentence.to_owned());
        }
    }
    let table =
        stroke_order_sources::table(language, sentences.iter().map(String::as_str), |url| {
            let path = cache.join(url.rsplit('/').next().unwrap());
            async move {
                if path.exists() {
                    return Ok(std::fs::read(path)?);
                }
                let bytes = reqwest::get(url).await?.error_for_status()?.bytes().await?;
                std::fs::write(&path, &bytes)?;
                Ok(bytes.to_vec())
            }
        })
        .await?;

    let pack = stroke_order::StrokePack::new(language.writing_system(), table);
    let mut counts: HashMap<String, usize> = HashMap::new();
    for sentence in &sentences {
        for unit in pack.segment(sentence) {
            if !unit.trim().is_empty() {
                *counts.entry(unit.to_owned()).or_default() += 1;
            }
        }
    }
    let mut missing: Vec<(&String, &usize)> = counts
        .iter()
        .filter(|(unit, _)| pack.glyphs(unit).is_empty())
        .collect();
    missing.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    let total: usize = counts.values().sum();
    let missing_total: usize = missing.iter().map(|(_, n)| **n).sum();
    println!(
        "{}: {:.2}% of {total} occurrences drawable; {} of {} distinct units missing",
        args[1],
        100.0 * (total - missing_total) as f64 / total as f64,
        missing.len(),
        counts.len()
    );
    // What a per-course table of every drawable unit would hold.
    let (mut forms, mut points) = (0usize, 0usize);
    for unit in counts.keys() {
        for glyph in pack.glyphs(unit) {
            forms += 1;
            points += glyph.strokes.iter().map(|s| s.points.len()).sum::<usize>();
        }
    }
    println!(
        "table: {} units, {forms} forms, {points} points (~{:.1} MB as f32 pairs)",
        counts.len() - missing.len(),
        points as f64 * 8.0 / 1e6
    );
    for (unit, n) in missing.iter().take(400) {
        println!("  {n:>7}  {unit:?}");
    }
    Ok(())
}
