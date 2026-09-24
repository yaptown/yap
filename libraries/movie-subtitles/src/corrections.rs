//! Reviewed corrections to cleaned cues, applied in memory before segmentation.
//! Raw subtitles remain evidence; only a film with entries gets a new digest.

use std::{collections::BTreeMap, ops::Range, path::Path, sync::LazyLock};

use anyhow::{Context, Result};
use language_utils::Language;
use serde::{Deserialize, Serialize};

use crate::SubtitleLine;

pub const CORRECTIONS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/corrections");
// The build script generates only this include_str! table, sorted by code. A new
// language file is picked up on rebuild without another registration site.
const FILES: &[(&str, &str)] = include!(concat!(env!("OUT_DIR"), "/corrections.rs"));
static ENTRIES: LazyLock<BTreeMap<&'static str, Vec<Correction>>> = LazyLock::new(|| {
    FILES
        .iter()
        .map(|(code, text)| (*code, parse(text).expect("invalid embedded corrections")))
        .collect()
});

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Correction {
    Spelling {
        imdb: String,
        cue: String,
        from: String,
        to: String,
        reason: String,
    },
    AudioMismatch {
        imdb: String,
        sentence: String,
        reason: String,
    },
}

impl Correction {
    pub fn imdb(&self) -> &str {
        match self {
            Self::Spelling { imdb, .. } | Self::AudioMismatch { imdb, .. } => imdb,
        }
    }

    fn key(&self) -> (&str, &str, &str, &str) {
        match self {
            Self::Spelling {
                imdb, cue, from, ..
            } => (imdb, cue, "spelling", from),
            Self::AudioMismatch { imdb, sentence, .. } => (imdb, sentence, "audio_mismatch", ""),
        }
    }
}

pub fn parse(text: &str) -> Result<Vec<Correction>> {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| Ok(serde_json::from_str(line)?))
        .collect()
}

/// Upsert by semantic key, retaining older decisions that no longer get flagged.
pub fn merge(existing: Vec<Correction>, additions: Vec<Correction>) -> Vec<Correction> {
    let mut entries = BTreeMap::new();
    for entry in existing.into_iter().chain(additions) {
        let (imdb, text, kind, from) = entry.key();
        entries.insert(
            (
                imdb.to_owned(),
                text.to_owned(),
                kind.to_owned(),
                from.to_owned(),
            ),
            entry,
        );
    }
    entries.into_values().collect()
}

pub fn merge_file(dir: &Path, language: Language, additions: Vec<Correction>) -> Result<()> {
    let path = dir.join(format!("{}.jsonl", language.code()));
    let old = match std::fs::read_to_string(&path) {
        Ok(text) => parse(&text)?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(e) => return Err(e).with_context(|| format!("reading {}", path.display())),
    };
    let mut text = String::new();
    for entry in merge(old, additions) {
        text.push_str(&serde_json::to_string(&entry)?);
        text.push('\n');
    }
    std::fs::create_dir_all(dir)?;
    let temp = path.with_extension("jsonl.tmp");
    std::fs::write(&temp, text)?;
    std::fs::rename(temp, path)?;
    Ok(())
}

fn film_entries(language: Language, imdb: &str) -> impl Iterator<Item = &'static Correction> + '_ {
    ENTRIES
        .get(language.code())
        .into_iter()
        .flatten()
        .filter(move |entry| entry.imdb() == imdb)
}

pub fn film_digest(language: Language, imdb: &str) -> String {
    let entries: Vec<_> = film_entries(language, imdb).collect();
    if entries.is_empty() {
        return String::new();
    }
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in serde_json::to_vec(&entries).expect("serializable corrections") {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{hash:016x}")
}

pub fn audio_mismatch(language: Language, imdb: &str, sentence: &str) -> bool {
    film_entries(language, imdb).any(|entry| matches!(entry, Correction::AudioMismatch { sentence: rejected, .. } if rejected == sentence))
}

/// Whether a corpus language code compares text character by character
/// rather than word by word. The one list both the subtitle↔transcript matcher
/// and the overlay use, so a flagged run and its replacement agree on units.
pub fn uses_chars(code: &str) -> bool {
    matches!(code, "jpn" | "zho-hans" | "tha" | "kor")
}

/// Byte ranges of the same alphanumeric units the agreement matcher uses,
/// preserving the original surface for a case-sensitive overlay replacement.
pub fn token_ranges(text: &str, chars: bool) -> Vec<Range<usize>> {
    let mut ranges: Vec<Range<usize>> = Vec::new();
    for (index, ch) in text.char_indices() {
        if !ch.is_alphanumeric() {
            continue;
        }
        if !chars && ranges.last().is_some_and(|range| range.end == index) {
            ranges.last_mut().unwrap().end = index + ch.len_utf8();
        } else {
            ranges.push(index..index + ch.len_utf8());
        }
    }
    ranges
}

pub fn unique_occurrence(text: &str, from: &str, chars: bool) -> Option<Range<usize>> {
    if from.is_empty() {
        return None;
    }
    let ranges = token_ranges(text, false);
    let mut found = text
        .char_indices()
        .filter_map(|(start, _)| text[start..].starts_with(from).then_some((start, from)))
        .filter(|(start, value)| {
            chars
                || ranges
                    .iter()
                    .any(|range| range.start == *start && range.end == start + value.len())
        });
    let (start, value) = found.next()?;
    if found.next().is_some() {
        return None;
    }
    Some(start..start + value.len())
}

fn apply_entries(lines: &mut [SubtitleLine], chars: bool, entries: &[&Correction]) {
    for line in lines {
        // All keys refer to the original cleaned cue, even when two separately
        // judged sentences in it need corrections. Apply non-overlapping edits
        // right-to-left so neither the cue key nor byte offsets can drift.
        let original = &line.sentence;
        let mut edits = Vec::new();
        for entry in entries {
            if let Correction::Spelling { cue, from, to, .. } = entry {
                if cue == original {
                    if let Some(range) = unique_occurrence(original, from, chars) {
                        edits.push((range, to));
                    }
                }
            }
        }
        edits.sort_by_key(|(range, _)| range.start);
        if edits.windows(2).any(|pair| pair[0].0.end > pair[1].0.start) {
            continue;
        }
        for (range, to) in edits.into_iter().rev() {
            line.sentence.replace_range(range, to);
        }
    }
}

pub fn apply(lines: &mut [SubtitleLine], language: Language, imdb: &str) {
    apply_entries(
        lines,
        uses_chars(language.code()),
        &film_entries(language, imdb).collect::<Vec<_>>(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    fn spelling(cue: &str, from: &str, to: &str) -> Correction {
        Correction::Spelling {
            imdb: "tt1".into(),
            cue: cue.into(),
            from: from.into(),
            to: to.into(),
            reason: "grammar".into(),
        }
    }
    fn corrected(cue: &str, from: &str, to: &str) -> String {
        let mut lines = vec![SubtitleLine {
            sentence: cue.into(),
            start_ms: 0,
            end_ms: 1000,
        }];
        apply_entries(&mut lines, false, &[&spelling(cue, from, to)]);
        lines.remove(0).sentence
    }
    #[test]
    fn exact_surface_and_unique_whole_tokens() {
        assert_eq!(
            corrected("Déposé ici.", "Déposé", "Déposer"),
            "Déposer ici."
        );
        assert_eq!(
            corrected("déposée ici.", "déposé", "déposer"),
            "déposée ici."
        );
        assert_eq!(corrected("Déposé ici.", "déposé", "déposer"), "Déposé ici.");
        assert_eq!(
            corrected("déposé déposé", "déposé", "déposer"),
            "déposé déposé"
        );
        assert_eq!(unique_occurrence("私は東京へ", "東京", true), Some(6..12));
    }
    #[test]
    fn parse_merge_and_sort() {
        let old = spelling("b", "b", "c");
        let new = spelling("b", "b", "d");
        let first = spelling("a", "a", "c");
        let text = format!("{}\n", serde_json::to_string(&old).unwrap());
        let merged = merge(parse(&text).unwrap(), vec![new.clone(), first.clone()]);
        assert_eq!(merged, vec![first, new]);
        assert!(film_digest(Language::French, "not-a-film").is_empty());
    }
    #[test]
    fn multiple_edits_use_original_cue_key() {
        let mut lines = vec![SubtitleLine {
            sentence: "déposé. allé.".into(),
            start_ms: 0,
            end_ms: 1,
        }];
        apply_entries(
            &mut lines,
            false,
            &[
                &spelling("déposé. allé.", "déposé", "déposer"),
                &spelling("déposé. allé.", "allé", "aller"),
            ],
        );
        assert_eq!(lines[0].sentence, "déposer. aller.");
    }
}
