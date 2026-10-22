//! Film-level clip eligibility, deliberately separate from the subtitle plan.
use std::{collections::BTreeMap, path::Path};

use anyhow::{bail, Context, Result};
use serde::Deserialize;

use crate::library::{course_dir, read_plan, Movie};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    language: String,
    imdb_id: String,
    reason: String,
}

pub struct ClipExclusions(BTreeMap<(String, String), String>);

impl ClipExclusions {
    pub fn load() -> Result<Self> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config/clip-exclusions.jsonl");
        Self::parse(
            &std::fs::read_to_string(&path)
                .with_context(|| format!("reading {}", path.display()))?,
        )
    }

    pub(crate) fn parse(raw: &str) -> Result<Self> {
        let mut entries = BTreeMap::new();
        for (line, row) in raw.lines().enumerate() {
            if row.trim().is_empty() {
                continue;
            }
            let entry: Entry = serde_json::from_str(row)
                .with_context(|| format!("clip-exclusions.jsonl line {}", line + 1))?;
            if entry.language.trim().is_empty()
                || entry.imdb_id.trim().is_empty()
                || entry.reason.trim().is_empty()
            {
                bail!("clip-exclusions.jsonl line {}: empty field", line + 1);
            }
            let key = (entry.language, entry.imdb_id);
            if entries.insert(key.clone(), entry.reason).is_some() {
                bail!("duplicate clip exclusion: {} {}", key.0, key.1);
            }
        }
        Ok(Self(entries))
    }

    /// Validate against the full inventory, before any language/IMDb filtering.
    pub fn validate<'a>(&self, films: impl IntoIterator<Item = (&'a str, &'a str)>) -> Result<()> {
        let known: std::collections::HashSet<_> = films
            .into_iter()
            .filter_map(|(language, imdb)| course_dir(language).map(|code| (code, imdb)))
            .collect();
        for (code, imdb) in self.0.keys() {
            if !known.contains(&(code.as_str(), imdb.as_str())) {
                bail!("unknown clip exclusion: {code} {imdb} (not in full inventory/plan)");
            }
        }
        Ok(())
    }

    pub fn allows(&self, code: &str, imdb: &str) -> bool {
        !self.0.contains_key(&(code.to_owned(), imdb.to_owned()))
    }

    pub fn print(&self) {
        for ((code, imdb), reason) in &self.0 {
            println!("clip excluded: {code} {imdb} — {reason}");
        }
    }

    fn filter(&self, plan: Vec<Movie>) -> Result<Vec<Movie>> {
        self.validate(
            plan.iter()
                .map(|m| (m.original_language.as_str(), m.imdb_id.as_str())),
        )?;
        Ok(plan
            .into_iter()
            .filter(|m| {
                course_dir(&m.original_language).is_none_or(|code| self.allows(code, &m.imdb_id))
            })
            .collect())
    }
}

/// Only clip consumers use this; all upstream text stages keep `read_plan`.
pub fn read_clip_plan(out: &Path) -> Result<Vec<Movie>> {
    ClipExclusions::load()?.filter(read_plan(out)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::{plan_path, Source};

    const ROW: &str = r#"{"language":"fra","imdb_id":"tt1","reason":"English hardsubs"}"#;

    #[test]
    fn validates_config_and_matches_language() {
        assert!(ClipExclusions::parse("not json").is_err());
        assert!(ClipExclusions::parse(&format!("{ROW}\n{ROW}")).is_err());
        assert!(ClipExclusions::parse(&ROW.replace("English hardsubs", " ")).is_err());
        let exclusions = ClipExclusions::parse(ROW).unwrap();
        assert!(exclusions.validate([("Japanese", "tt1")]).is_err());
        assert!(exclusions.validate([("French", "tt1")]).is_ok());
        assert!(!exclusions.allows("fra", "tt1"));
        assert!(exclusions.allows("jpn", "tt1"));
    }

    #[test]
    fn clip_filter_leaves_raw_plan_unchanged() {
        let root = tempfile::tempdir().unwrap();
        let movies: Vec<_> = [("French", "tt1"), ("Japanese", "tt1")]
            .into_iter()
            .map(|(language, id)| Movie {
                imdb_id: id.into(),
                title: "fixture".into(),
                year: None,
                path: root.path().join("not-a-real-movie.mkv"),
                original_language: language.into(),
                source: Source::Missing,
            })
            .collect();
        let raw = serde_json::to_vec(&movies).unwrap();
        std::fs::write(plan_path(root.path()), &raw).unwrap();
        let filtered = ClipExclusions::parse(ROW)
            .unwrap()
            .filter(read_plan(root.path()).unwrap())
            .unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].original_language, "Japanese");
        assert_eq!(read_plan(root.path()).unwrap().len(), 2);
        assert_eq!(std::fs::read(plan_path(root.path())).unwrap(), raw);
    }
}
