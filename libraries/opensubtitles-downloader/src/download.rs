//! One acquisition path for the standalone downloader and owned-film corpus.

use std::{collections::BTreeMap, fs, path::Path};

use anyhow::{Context, Result};
use language_utils::Language;
use movie_subtitles::SubtitleLine;
use serde::{Deserialize, Serialize};

use crate::{quality, rank_by_quality, OpenSubtitlesClient, SubtitleAttributes, Throttled};

/// Negative results only: searches are always retried to discover new uploads,
/// but a file rejected on its contents never consumes another download credit.
#[derive(Default, Serialize, Deserialize)]
struct Failed {
    rejected: BTreeMap<u64, String>,
}

impl Failed {
    fn load(path: &Path) -> Result<Self> {
        match fs::read(path) {
            Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e.into()),
        }
    }

    fn save(&self, path: &Path) -> Result<()> {
        fs::create_dir_all(path.parent().unwrap())?;
        let temp = path.with_extension("json.tmp");
        fs::write(&temp, serde_json::to_vec_pretty(self)?)?;
        fs::rename(temp, path)?;
        Ok(())
    }

    fn reject(&mut self, file_id: u64, reason: String, path: &Path) -> Result<()> {
        println!("  Rejected file {file_id}: {reason}");
        self.rejected.insert(file_id, reason);
        // Persist before the next request: a later outage must not lose this verdict.
        self.save(path)
    }
}

fn candidate_rejection(attrs: &SubtitleAttributes) -> Option<&'static str> {
    if attrs.ai_translated || attrs.machine_translated {
        Some("machine/AI translated")
    } else if attrs.files.len() != 1 {
        // Multi-disc uploads reset their clocks. Even the first file alone is
        // only a fragment, not a complete subtitle for the owned film.
        Some("not a single-file subtitle")
    } else {
        None
    }
}

fn parse_and_check(srt: &str, language: Language) -> Result<Vec<SubtitleLine>, String> {
    let lines = movie_subtitles::parse_srt(srt).map_err(|e| format!("SRT parse: {e}"))?;
    language.check_subtitle_sanity(lines.iter().map(|l| l.sentence.as_str()), &[])?;
    Ok(lines)
}

async fn download_candidates(
    client: &OpenSubtitlesClient,
    imdb: &str,
    movies: &Path,
    language: Language,
    failed_path: &Path,
    mut candidates: Vec<crate::SubtitleResult>,
) -> Result<bool> {
    let mut failed = Failed::load(failed_path)?;
    rank_by_quality(&mut candidates);
    println!("  Found {} subtitle options", candidates.len());

    let mut last_download_error = None;
    for candidate in candidates {
        let attrs = candidate.attributes;
        if let Some(reason) = candidate_rejection(&attrs) {
            for file in &attrs.files {
                if !failed.rejected.contains_key(&file.file_id) {
                    failed.reject(file.file_id, reason.into(), failed_path)?;
                }
            }
            continue;
        }
        let file_id = attrs.files[0].file_id;
        if let Some(reason) = failed.rejected.get(&file_id) {
            println!("  Skipping rejected file {file_id}: {reason}");
            continue;
        }
        println!("  Downloading file {file_id}");
        // A stale download link must not hide usable lower-ranked candidates.
        // Network failures are not content verdicts and never blacklist a file.
        let srt = match client.download_subtitle(file_id).await {
            Ok(srt) => srt,
            Err(e) if e.downcast_ref::<Throttled>().is_some() => return Err(e),
            Err(e) => {
                println!("  Download failed for {file_id}: {e:#}; trying next candidate");
                last_download_error = Some(e);
                continue;
            }
        };
        let lines = match parse_and_check(&srt, language) {
            Ok(lines) => lines,
            Err(reason) => {
                failed.reject(file_id, reason, failed_path)?;
                continue;
            }
        };
        println!("  Checking quality of {} dialogue lines", lines.len());
        let verdict = quality::check(&lines, language)
            .await
            .context("subtitle quality check")?;
        if !verdict.clean {
            failed.reject(file_id, verdict.reason, failed_path)?;
            continue;
        }

        // Preserve the provider's exact SRT first; JSONL is just a lossy cache.
        movie_subtitles::write_raw_srt(movies, imdb, &srt)?;
        movie_subtitles::write_derived_jsonl(
            &movie_subtitles::derived_jsonl_path(movies, imdb),
            &lines,
        )?;
        println!("  Saved {imdb}: {} dialogue lines", lines.len());
        return Ok(true);
    }
    if let Some(e) = last_download_error {
        return Err(e);
    }
    println!("  No usable subtitles for {imdb}");
    Ok(false)
}

/// Ensure a film has a raw subtitle, returning whether one was fetched.
/// The caller owns metadata enrichment and chooses where failed attempts live.
/// A derived JSONL without its raw source is not a completed download.
pub async fn process_movie(
    imdb: &str,
    client: &OpenSubtitlesClient,
    movies: &Path,
    language: Language,
    failed_path: &Path,
) -> Result<bool> {
    if already_downloaded(movies, imdb)? {
        return Ok(false);
    }
    let candidates = client
        .search_subtitles_for_movie(
            imdb.strip_prefix("tt").unwrap().parse()?,
            language.opensubtitles_languages(),
        )
        .await?;
    download_candidates(client, imdb, movies, language, failed_path, candidates).await
}

fn already_downloaded(movies: &Path, imdb: &str) -> Result<bool> {
    let raw = movie_subtitles::raw_srt_path(movies, imdb);
    let derived = movie_subtitles::derived_jsonl_path(movies, imdb);
    if raw.exists() {
        if !derived.exists() {
            let lines = movie_subtitles::parse_srt(&fs::read_to_string(raw)?)?;
            movie_subtitles::write_derived_jsonl(&derived, &lines)?;
        }
        println!("  Subtitle already downloaded");
        return Ok(true);
    }
    Ok(false)
}

/// Group uploads by episode IMDb, never by the parent show's ID.
pub fn group_episodes(
    candidates: Vec<crate::SubtitleResult>,
) -> BTreeMap<String, Vec<crate::SubtitleResult>> {
    let mut episodes = BTreeMap::new();
    for candidate in candidates {
        if let Some(id) = candidate
            .attributes
            .feature_details
            .imdb_id
            .filter(|id| *id != 0)
        {
            episodes
                .entry(format!("tt{id:07}"))
                .or_insert_with(Vec::new)
                .push(candidate);
        }
    }
    episodes
}

/// Acquire each episode independently, sharing the movie quality and retry path.
pub async fn process_show(
    parent_imdb: &str,
    client: &OpenSubtitlesClient,
    movies: &Path,
    language: Language,
) -> Result<usize> {
    let candidates = client
        .search_subtitles_for_show(
            parent_imdb.strip_prefix("tt").unwrap().parse()?,
            language.opensubtitles_languages(),
        )
        .await?;
    let mut downloaded = 0;
    for (imdb, candidates) in group_episodes(candidates) {
        let details = &candidates[0].attributes.feature_details;
        println!(
            "  {} S{:02}E{:02} ({imdb})",
            details.parent_title.as_deref().unwrap_or(parent_imdb),
            details.season_number.unwrap_or(0),
            details.episode_number.unwrap_or(0)
        );
        if already_downloaded(movies, &imdb)? {
            continue;
        }
        let failed_path = movies.join("fetch-failures").join(format!("{imdb}.json"));
        match download_candidates(client, &imdb, movies, language, &failed_path, candidates).await {
            Ok(true) => downloaded += 1,
            Ok(false) => {}
            Err(e) if e.downcast_ref::<Throttled>().is_some() => return Err(e),
            Err(e) => println!("  Error for {imdb}: {e:#}"),
        }
    }
    Ok(downloaded)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attrs(files: &[u64], ai: bool, machine: bool) -> SubtitleAttributes {
        serde_json::from_value(serde_json::json!({
            "feature_details": {},
            "files": files.iter().map(|id| serde_json::json!({"file_id":id})).collect::<Vec<_>>(),
            "ai_translated": ai,
            "machine_translated": machine
        }))
        .unwrap()
    }

    #[test]
    fn groups_episode_uploads_by_padded_imdb_not_episode_number() {
        let candidates = [Some(708760), Some(708760), Some(12345678), None, Some(0)]
            .into_iter()
            .map(|id| {
                serde_json::from_value(serde_json::json!({
                    "attributes": {"feature_details": {"imdb_id": id}, "files": []}
                }))
                .unwrap()
            })
            .collect();
        let episodes = group_episodes(candidates);
        assert_eq!(episodes.len(), 2);
        assert_eq!(episodes["tt0708760"].len(), 2);
        assert_eq!(episodes["tt12345678"].len(), 1);
    }

    #[test]
    fn only_human_single_file_candidates() {
        assert!(candidate_rejection(&attrs(&[1], false, false)).is_none());
        for candidate in [
            attrs(&[], false, false),
            attrs(&[1, 2], false, false),
            attrs(&[1], true, false),
            attrs(&[1], false, true),
        ] {
            assert!(candidate_rejection(&candidate).is_some());
        }
    }

    #[test]
    fn hindi_requires_hindi_not_english_or_other_scripts() {
        for text in [
            "I am here and you are not going to leave.",
            "ഇത് ഒരു മലയാളം സംഭാഷണമാണ്",
            "මේ සිංහල භාෂාවෙන් ලියන ලද දෙබසකි",
            "Main tumhare saath hoon aur tum kahan ja rahe ho?",
        ] {
            assert!(Language::Hindi
                .check_subtitle_sanity(std::iter::repeat_n(text, 80), &[])
                .is_err());
        }
        assert!(Language::Hindi
            .check_subtitle_sanity(
                std::iter::repeat_n("वह घर में है और उसके दोस्त को पता है।", 80),
                &[]
            )
            .is_ok());
    }
}
