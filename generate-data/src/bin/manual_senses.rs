//! Inspect packed grams and append handwritten, per-gram sense slots.
use anyhow::{Context, Result, bail, ensure};
use clap::{Parser, Subcommand, ValueEnum};
use generate_data::usage_discovery::{UsageAnchor, UsageEntry, UsageInventory};
use language_utils::{Atom, Gram, Language, WordType};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Parser)]
#[command(about = "List grams by lemma and append manual senses (run from the repo root)")]
struct Args {
    /// Target language, or a course slug such as fra_for_eng.
    #[arg(long)]
    course: String,
    #[arg(long, default_value = "eng")]
    native: String,
    #[arg(long, default_value = "out")]
    out: PathBuf,
    /// Override the inventory path, e.g. a scratch copy for trying an addition.
    #[arg(long)]
    inventory: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Include multiword grams containing any of these lemmas.
    List {
        #[arg(required = true)]
        lemmas: Vec<String>,
        #[arg(long)]
        json: bool,
    },
    /// Validate all proposal files, then append their senses atomically.
    Apply {
        #[arg(required = true)]
        proposals: Vec<PathBuf>,
    },
    /// Append the same usage to selected grams, never changing existing slots.
    Add {
        #[arg(required = true)]
        selectors: Vec<String>,
        #[arg(long, value_enum)]
        kind: Kind,
        /// Describe the sense in the target language.
        #[arg(long)]
        gloss: String,
        /// Repeat for each example; each must contain exactly one occurrence.
        #[arg(long)]
        example: Vec<String>,
    },
}

#[derive(Clone, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
enum Kind {
    Meaning,
    Construction,
    Formula,
}
impl Kind {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Meaning => "meaning",
            Self::Construction => "construction",
            Self::Formula => "formula",
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Sense {
    kind: Kind,
    gloss: String,
    #[serde(default)]
    examples: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Proposal {
    gram: String,
    senses: Vec<Sense>,
}

struct Entry {
    gram: Gram<String>,
    count: u64,
    direct_count: u64,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let course = language_utils::language_pack::course_from_directory_slug(&args.course)
        .or_else(|| {
            Some(language_utils::Course {
                target_language: Language::from_code(&args.course)?,
                native_language: Language::from_code(&args.native)?,
            })
        })
        .context("unknown course/language code")?;
    let language = course.target_language;
    let dir = args
        .out
        .join(language_utils::language_pack::course_directory_slug(course));
    let pack = language_utils::language_pack::load_split_dir(&dir)
        .with_context(|| format!("load built pack {}", dir.display()))?;
    let mut grams = BTreeMap::<Gram<String>, (u64, u64)>::new();
    for (tagged, frequency) in pack.gram_frequencies.resolved(&pack) {
        let counts = grams
            .entry(pack.resolve_gram(&tagged.sense.gram))
            .or_default();
        counts.0 += u64::from(frequency.count);
        counts.1 += u64::from(frequency.direct_count);
    }
    let mut entries = BTreeMap::new();
    for (gram, (count, direct_count)) in grams {
        let id = format!(
            "{:016x}",
            xxhash_rust::xxh3::xxh3_64(&serde_json::to_vec(&gram)?)
        );
        ensure!(
            entries
                .insert(
                    id,
                    Entry {
                        gram,
                        count,
                        direct_count
                    }
                )
                .is_none(),
            "gram identity hash collision"
        );
    }
    let path = args.inventory.unwrap_or_else(|| {
        PathBuf::from(format!(
            "generate-data/data/{}/usage_inventories.jsonl",
            language.corpus_code()
        ))
    });
    let contents = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error.into()),
    };
    let rows: Vec<UsageInventory> = contents
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()?;
    match args.command {
        Command::List { lemmas, json } => {
            let mut found: Vec<_> = entries.iter().filter(|(_, entry)| entry.gram.iter().any(|atom| {
                matches!(atom, Atom::Tok(word) if word.heteronym().is_some_and(|h| lemmas.contains(&h.lemma)))
            })).collect();
            found.sort_by_key(|(id, entry)| (std::cmp::Reverse(entry.count), *id));
            for (id, entry) in found {
                let length = (6..=id.len())
                    .find(|&n| {
                        entries
                            .keys()
                            .filter(|other| other.starts_with(&id[..n]))
                            .count()
                            == 1
                    })
                    .unwrap();
                if json {
                    let usages: Vec<_> = rows
                        .iter()
                        .find(|row| row.gram == entry.gram)
                        .into_iter()
                        .flat_map(|row| row.usages.iter().enumerate())
                        .map(|(i, usage)| {
                            serde_json::json!({
                                "id": i + 1, "kind": usage.kind, "gloss": usage.gloss,
                                "live": usage.is_live(), "manual": usage.is_manual(),
                                "source": usage.source.as_deref().unwrap_or("discovery"),
                                "n_assigned": usage.n_assigned,
                            })
                        })
                        .collect();
                    let lemmas: Vec<_> = entry
                        .gram
                        .iter()
                        .filter_map(|atom| match atom {
                            Atom::Tok(word) => word
                                .heteronym()
                                .map(|h| serde_json::json!({"lemma": h.lemma, "pos": h.pos})),
                            _ => None,
                        })
                        .collect();
                    println!(
                        "{}",
                        serde_json::json!({
                            "selector": &id[..length], "display": entry.gram.to_display_string(language),
                            "frequency": entry.count, "direct_count": entry.direct_count,
                            "lemmas": lemmas, "usages": usages,
                        })
                    );
                    continue;
                }
                println!(
                    "{}\t{}\tfrequency={}\tdirect={}\t{}",
                    &id[..length],
                    entry.gram.to_display_string(language),
                    entry.count,
                    entry.direct_count,
                    entry
                        .gram
                        .iter()
                        .filter_map(|a| match a {
                            Atom::Tok(w) => w.heteronym().map(|h| format!("{}@{}", h.lemma, h.pos)),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join(" ")
                );
                if let Some(row) = rows.iter().find(|row| row.gram == entry.gram) {
                    for (i, usage) in row.usages.iter().enumerate() {
                        println!(
                            "  {}\t{}\t{}\tsource={}\tn_assigned={}",
                            i + 1,
                            usage.kind,
                            usage.gloss,
                            usage.source.as_deref().unwrap_or("discovery"),
                            usage.n_assigned
                        );
                    }
                } else {
                    println!("  no inventory usages");
                }
            }
        }
        Command::Add {
            selectors,
            kind,
            gloss,
            example,
        } => {
            ensure!(
                selectors.len() == 1 || example.is_empty(),
                "examples are only allowed with one selector"
            );
            let sense = Sense {
                kind,
                gloss,
                examples: example,
            };
            let proposals = selectors
                .into_iter()
                .enumerate()
                .map(|(i, gram)| {
                    (
                        format!("add:{}", i + 1),
                        Ok(Proposal {
                            gram,
                            senses: vec![sense.clone()],
                        }),
                    )
                })
                .collect();
            let updated = apply_proposals(&contents, &entries, language, proposals)?;
            write_inventory(&path, &updated)?;
        }
        Command::Apply { proposals } => {
            let mut lines = Vec::new();
            for file in proposals {
                match std::fs::read_to_string(&file) {
                    Ok(text) => {
                        for (i, line) in text.lines().enumerate() {
                            // Recover the selector even when another field fails to deserialize.
                            let proposal =
                                serde_json::from_str::<Proposal>(line).map_err(|error| {
                                    let selector = serde_json::from_str::<serde_json::Value>(line)
                                        .ok()
                                        .and_then(|value| value.get("gram").cloned());
                                    anyhow::anyhow!(
                                        "selector {}: {error}",
                                        selector.unwrap_or(serde_json::Value::Null)
                                    )
                                });
                            lines.push((format!("{}:{}", file.display(), i + 1), proposal));
                        }
                    }
                    Err(error) => lines.push((
                        format!("{}:0", file.display()),
                        Err(anyhow::anyhow!("selector <unknown>: {error}")),
                    )),
                }
            }
            let updated = apply_proposals(&contents, &entries, language, lines)?;
            write_inventory(&path, &updated)?;
        }
    }
    Ok(())
}

/// Match the exact surface token sequence, not lemma-expanded inflections.
fn example_spans(
    gram: &Gram<String>,
    language: Language,
    sentence: &str,
) -> Result<Vec<(u32, u32)>> {
    let words = language_utils::atoms_to_literals(&gram.0, language);
    ensure!(!words.is_empty(), "gram has no words");
    let mut pattern = String::from("(?i)");
    if language.uses_word_spaces() && words[0].word.text.starts_with(char::is_alphanumeric) {
        pattern.push_str(r"\b");
    }
    let mut captures = Vec::new();
    for (i, literal) in words.iter().enumerate() {
        if matches!(literal.word.word_type, WordType::Heteronym(_)) {
            captures.push(captures.len() + 1);
            pattern.push('(');
            pattern.push_str(&regex::escape(&literal.word.text));
            pattern.push(')');
        } else {
            pattern.push_str(&regex::escape(&literal.word.text));
        }
        if i + 1 < words.len() && !literal.whitespace.as_str().is_empty() {
            pattern.push_str(r"\s+");
        }
    }
    if language.uses_word_spaces()
        && words
            .last()
            .unwrap()
            .word
            .text
            .ends_with(char::is_alphanumeric)
    {
        pattern.push_str(r"\b");
    }
    let re = regex::Regex::new(&pattern)?;
    let matches: Vec<_> = re.captures_iter(sentence).collect();
    ensure!(
        matches.len() == 1,
        "expected one occurrence of {:?} in {:?}, found {}",
        gram.to_display_string(language),
        sentence,
        matches.len()
    );
    ensure!(!captures.is_empty(), "gram has no learnable words");
    captures
        .into_iter()
        .map(|i| {
            let capture = matches[0].get(i).unwrap();
            Ok((
                u32::try_from(sentence[..capture.start()].chars().count())?,
                u32::try_from(sentence[..capture.end()].chars().count())?,
            ))
        })
        .collect()
}

/// Build the complete replacement in memory; errors never reach the writer.
fn apply_proposals(
    contents: &str,
    entries: &BTreeMap<String, Entry>,
    language: Language,
    proposals: Vec<(String, Result<Proposal>)>,
) -> Result<String> {
    let rows: Vec<UsageInventory> = contents
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()?;
    let mut additions = BTreeMap::new();
    let mut seen = BTreeSet::new();
    let mut errors = Vec::new();
    for (location, proposal) in proposals {
        let proposal = match proposal {
            Ok(proposal) => proposal,
            Err(error) => {
                errors.push(format!("{location}: {error}"));
                continue;
            }
        };
        let mut report = |reason: String| {
            errors.push(format!(
                "{location}: selector {:?}: {reason}",
                proposal.gram
            ))
        };
        let matches: Vec<_> = entries
            .iter()
            .filter(|(id, _)| id.starts_with(&proposal.gram))
            .collect();
        if proposal.gram.len() < 6 || matches.len() != 1 {
            report(format!(
                "use a unique selector printed by list (at least 6 characters); matched {} grams",
                matches.len()
            ));
            continue;
        }
        let (id, entry) = matches[0];
        if !seen.insert(id) {
            report("duplicate selector for this gram".into());
        }
        let existing: Vec<_> = rows.iter().filter(|row| row.gram == entry.gram).collect();
        if existing.len() > 1 {
            report("duplicate inventory rows for this gram".into());
        }
        let usages: Vec<_> = existing.iter().flat_map(|row| &row.usages).collect();
        if usages.iter().filter(|usage| usage.is_live()).count() + proposal.senses.len() < 2 {
            report("row must end up with at least two live senses".into());
        }
        if proposal.senses.is_empty() {
            report("senses must not be empty".into());
        }
        let mut glosses: BTreeSet<_> = usages
            .iter()
            .map(|usage| usage.gloss.trim().to_lowercase())
            .collect();
        let mut new_usages = Vec::new();
        for sense in proposal.senses {
            if sense.gloss.trim().is_empty() {
                report("gloss must not be empty".into());
            }
            if !glosses.insert(sense.gloss.trim().to_lowercase()) {
                report(format!("duplicate gloss {:?}", sense.gloss));
            }
            let mut anchors = Vec::new();
            for sentence in sense.examples {
                match example_spans(&entry.gram, language, &sentence) {
                    Ok(spans) => anchors.push(UsageAnchor {
                        sentence,
                        spans,
                        gold: true,
                    }),
                    Err(error) => report(error.to_string()),
                }
            }
            new_usages.push(UsageEntry {
                source: Some("manual".into()),
                readings: None,
                kind: sense.kind.as_str().into(),
                gloss: sense.gloss,
                n_gold: 0,
                n_assigned: 0,
                loo_correct: 0,
                anchors,
            });
        }
        additions.insert(entry.gram.clone(), new_usages);
    }
    if !errors.is_empty() {
        bail!(
            "proposal validation failed; nothing written:\n{}",
            errors.join("\n")
        );
    }
    let mut result = String::new();
    for (line, mut row) in contents.split_inclusive('\n').zip(rows) {
        if let Some(usages) = additions.remove(&row.gram) {
            row.usages.extend(usages);
            result.push_str(&serde_json::to_string(&row)?);
            if line.ends_with('\n') {
                result.push('\n');
            }
        } else {
            result.push_str(line);
        }
    }
    for (gram, usages) in additions {
        if !result.is_empty() && !result.ends_with('\n') {
            result.push('\n');
        }
        let row = UsageInventory {
            key: gram.to_display_string(language),
            gram,
            n: 0,
            n_expression: 0,
            usages,
            absorbers: vec![],
            silhouette: 0.0,
            source: "manual".into(),
        };
        result.push_str(&serde_json::to_string(&row)?);
        result.push('\n');
    }
    Ok(result)
}

fn write_inventory(path: &Path, updated: &str) -> Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent)?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    file.write_all(updated.as_bytes())?;
    file.persist(path)?;
    println!("appended manual senses to {}", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn apply_validates_every_proposal_before_writing() -> Result<()> {
        let gram: Gram<String> = serde_json::from_str(
            r#"[{"Tok":{"text":"encore","word_type":{"type":"Heteronym","word":"encore","lemma":"encore","pos":"ADV"}}}]"#,
        )?;
        let entries = BTreeMap::from([(
            "abcdef123456".into(),
            Entry {
                gram,
                count: 1,
                direct_count: 1,
            },
        )]);
        let propose = |glosses: &[&str]| Proposal {
            gram: "abcdef".into(),
            senses: glosses
                .iter()
                .map(|gloss| Sense {
                    kind: Kind::Meaning,
                    gloss: (*gloss).into(),
                    examples: vec![],
                })
                .collect(),
        };
        let apply = |contents: &str, proposals| {
            apply_proposals(contents, &entries, Language::French, proposals)
        };
        let error = apply("", vec![("one.jsonl:1".into(), Ok(propose(&["again"])))]).unwrap_err();
        assert!(error.to_string().contains("at least two live senses"));
        let contents = apply(
            "",
            vec![("one.jsonl:1".into(), Ok(propose(&["again", "still"])))],
        )?;
        let file = tempfile::NamedTempFile::new()?;
        std::fs::write(file.path(), &contents)?;
        let result = apply(
            &contents,
            vec![
                ("one.jsonl:1".into(), Ok(propose(&["more"]))),
                ("two.jsonl:3".into(), Ok(propose(&[" AGAIN "]))),
                ("two.jsonl:4".into(), Err(anyhow::anyhow!("invalid JSON"))),
            ],
        )
        .and_then(|updated| write_inventory(file.path(), &updated));
        let error = result.unwrap_err().to_string();
        assert!(
            error.contains("two.jsonl:3")
                && error.contains("duplicate selector")
                && error.contains("duplicate gloss")
        );
        assert!(error.contains("two.jsonl:4: invalid JSON"));
        assert_eq!(std::fs::read_to_string(file.path())?, contents);
        let updated = apply(
            &contents,
            vec![("one.jsonl:1".into(), Ok(propose(&["more"])))],
        )?;
        let row: UsageInventory = serde_json::from_str(&updated)?;
        assert_eq!(row.usages.len(), 3);
        assert!(row.usages.iter().all(UsageEntry::is_live));
        Ok(())
    }

    #[test]
    fn manual_examples_use_character_offsets_and_reject_missing_or_ambiguous_matches() {
        let gram: Gram<String> = serde_json::from_str(r#"[{"Tok":{"text":"encore","word_type":{"type":"Heteronym","word":"encore","lemma":"encore","pos":"ADV"}}}]"#).unwrap();
        assert_eq!(
            example_spans(&gram, Language::French, "Écoute encore.").unwrap(),
            vec![(7, 13)]
        );
        assert!(example_spans(&gram, Language::French, "encore encore").is_err());
        assert!(example_spans(&gram, Language::French, "encorement").is_err());
    }
}
