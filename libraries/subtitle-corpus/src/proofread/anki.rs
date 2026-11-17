//! The coherence pass over Anki-deck sentences. The decks (neri's frequency
//! lists, mined from subtitles) carry the same debris as our films, such as
//! "Qu'est-ce que s'est ?", but have no film to hang a correction on, so an
//! accepted flag bans the sentence in generate-data instead.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use anyhow::{bail, Result};
use language_utils::Language;
use serde::{Deserialize, Serialize};

use super::{
    audit_flags, chunks, judge, print_counts, prompts, sorted_paths, write_jsonl, AuditedFlag,
    CoherenceAnswer, Item, Options,
};

/// generate-data reads this beside `banned_sentences.txt`.
pub const BANS_FILE: &str = "banned_sentences_proofread.jsonl";
/// Chunks carry a film id; deck chunks all share this one.
const SOURCE: &str = "anki";

#[derive(Deserialize)]
struct Source {
    from_anki: bool,
}

#[derive(Serialize, Deserialize)]
struct Ban {
    sentence: String,
    reason: String,
}

/// Every Anki sentence of a course, in pack order, from the last generate-data run.
fn anki_sentences(course_out: &Path, language: Language) -> Result<Vec<String>> {
    let path = course_out
        .join(language.corpus_code())
        .join("sentence_sources.jsonl");
    std::fs::read_to_string(&path)?
        .lines()
        .filter_map(|line| {
            serde_json::from_str::<(String, Source)>(line)
                .map(|(sentence, source)| source.from_anki.then_some(sentence))
                .transpose()
        })
        .map(|row| Ok(row?))
        .collect()
}

pub(super) async fn run(options: &Options) -> Result<()> {
    let mut requests = Vec::new();
    for dir in sorted_paths(&options.data_root)? {
        let code = dir.file_name().unwrap().to_string_lossy().into_owned();
        let Some(language) = Language::from_code(&code) else {
            continue;
        };
        if !dir.join("sentence-sources/anki-decks").is_dir()
            || options.language.as_ref().is_some_and(|only| *only != code)
        {
            continue;
        }
        let items = anki_sentences(&options.course_out, language)?
            .into_iter()
            .map(|sentence| Item {
                index: 0,
                key: sentence.clone(),
                text: sentence,
                read_only: false,
            })
            .collect();
        // Pack order is alphabetical, so neighbouring sentences are no context.
        requests.extend(
            chunks(language, SOURCE, items, &mut BTreeSet::new())
                .into_iter()
                .step_by(options.every)
                .map(|mut chunk| {
                    chunk.before_read_only.clear();
                    chunk.after_read_only.clear();
                    chunk
                }),
        );
    }
    print_counts("anki coherence sentences", &requests);
    if options.dry_run {
        return Ok(());
    }
    let (answers, failed) =
        judge::<CoherenceAnswer>(&requests, prompts::coherence, options.live).await?;
    let mut flags: Vec<AuditedFlag> = answers
        .into_iter()
        .flat_map(|(chunk, answer)| audit_flags(chunk, answer))
        .collect();
    flags.sort_by(|a, b| (&a.language, &a.sentence).cmp(&(&b.language, &b.sentence)));
    let audit_dir = options.out.join("proofread");
    std::fs::create_dir_all(&audit_dir)?;
    write_jsonl(&audit_dir.join("anki-incoherent.jsonl"), &flags)?;
    for flag in &flags {
        println!("incoherent: {}", serde_json::to_string(flag)?);
    }
    if failed != 0 {
        bail!("{failed} coherence requests failed; audit saved, no bans written");
    }
    if options.no_merge {
        return Ok(());
    }
    let mut additions = BTreeMap::<String, Vec<Ban>>::new();
    for flag in flags.into_iter().filter(|flag| flag.accepted) {
        additions.entry(flag.language).or_default().push(Ban {
            sentence: flag.sentence.unwrap(),
            reason: flag.reason,
        });
    }
    for (code, new) in additions {
        let path = options.data_root.join(&code).join(BANS_FILE);
        let mut bans: Vec<Ban> = match std::fs::read_to_string(&path) {
            Ok(text) => text
                .lines()
                .map(serde_json::from_str)
                .collect::<Result<_, _>>()?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(error) => return Err(error.into()),
        };
        let known: BTreeSet<_> = bans.iter().map(|ban| ban.sentence.clone()).collect();
        let before = bans.len();
        bans.extend(new.into_iter().filter(|ban| !known.contains(&ban.sentence)));
        bans.sort_by(|a, b| a.sentence.cmp(&b.sentence));
        println!(
            "{code}: {} new bans in {}",
            bans.len() - before,
            path.display()
        );
        write_jsonl(&path, &bans)?;
    }
    Ok(())
}
