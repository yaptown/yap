//! Fetch original-language subtitles for owned films with no usable source.
//! Does not rewrite the plan: refresh re-inventories after successful downloads.

use std::path::Path;

use anyhow::{Context, Result};
use language_utils::Language;
use opensubtitles_downloader::{download::process_movie, OpenSubtitlesClient, Throttled};

use crate::library::{read_plan, Movie, Source};

fn target(movie: &Movie, out: &Path, imdb: Option<&str>) -> Option<Language> {
    (movie.source == Source::Missing && imdb.is_none_or(|id| movie.imdb_id == id))
        .then(|| movie.course(out))
        .flatten()
}

/// Returns how many subtitles were fetched. A film that fails is reported and
/// skipped, like every other per-film step; throttling ends the run early.
pub fn run(out: &Path, data_root: &Path, imdb: Option<&str>) -> Result<usize> {
    let plan = read_plan(out)?;
    let targets: Vec<_> = plan
        .iter()
        .filter_map(|movie| target(movie, out, imdb).map(|language| (movie, language)))
        .collect();
    println!("{} owned films need subtitles", targets.len());
    if targets.is_empty() {
        return Ok(0);
    }
    tokio::runtime::Runtime::new()?.block_on(async {
        let mut client = OpenSubtitlesClient::new(
            std::env::var("OPENSUBTITLES_API_KEY").context("OPENSUBTITLES_API_KEY not set")?,
        );
        if let (Ok(user), Ok(password)) = (
            std::env::var("OPENSUBTITLES_USERNAME"),
            std::env::var("OPENSUBTITLES_PASSWORD"),
        ) {
            client.login(&user, &password).await?;
        }
        let mut fetched = 0;
        for (movie, language) in targets {
            println!("{} {} ({})", movie.imdb_id, movie.title, language.code());
            let movies = data_root
                .join(language.corpus_code())
                .join("sentence-sources/movies");
            let failed = out
                .join(&movie.imdb_id)
                .join(format!("fetch-subtitles-{}.json", language.code()));
            let result = process_movie(&movie.imdb_id, &client, &movies, language, &failed)
                .await
                .and_then(|new| {
                    fetched += usize::from(new);
                    // export-yap skips downloads to avoid overwriting its own
                    // inputs, so their inventory metadata must be added here.
                    if movie_subtitles::raw_srt_path(&movies, &movie.imdb_id).exists() {
                        movie_metadata::append_movie_metadata(
                            &movies.join("metadata.jsonl"),
                            &language_utils::MovieMetadataBasic {
                                id: movie.imdb_id.clone(),
                                title: movie.title.clone(),
                                year: movie.year,
                                original_language: Some(language.iso_639_1().into()),
                                variety: Some(language.code().into()),
                                poster_path: None,
                                localizations: Default::default(),
                                rotten_tomatoes_score: None,
                            },
                            false,
                        )?;
                    }
                    Ok(())
                });
            if let Err(e) = result {
                println!("  ✗ {e:#}");
                // The daily quota is spent; the next refresh carries on.
                if e.downcast_ref::<Throttled>().is_some() {
                    break;
                }
            }
        }
        println!("{fetched} subtitles fetched");
        Ok(fetched)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn movie(language: &str, source: Source) -> Movie {
        Movie {
            imdb_id: "tt1234567".into(),
            title: "Film".into(),
            year: None,
            path: "film.mkv".into(),
            original_language: language.into(),
            source,
        }
    }

    #[test]
    fn only_missing_sources_in_supported_courses_are_targets() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(
            target(&movie("English", Source::Missing), root.path(), None),
            Some(Language::English)
        );
        assert!(target(&movie("Swedish", Source::Missing), root.path(), None).is_none());
        for source in [
            Source::NoOriginalAudio,
            Source::DiscText {
                index: 0,
                codec: "subrip".into(),
            },
            Source::DiscBitmap {
                index: 0,
                codec: "hdmv_pgs_subtitle".into(),
            },
            Source::Sidecar {
                path: "film.srt".into(),
            },
            Source::Downloaded {
                path: "film.srt".into(),
            },
        ] {
            assert!(target(&movie("Russian", source), root.path(), None).is_none());
        }
        let movie = movie("Korean", Source::Missing);
        assert!(target(&movie, root.path(), Some("tt7654321")).is_none());
        assert_eq!(
            target(&movie, root.path(), Some(&movie.imdb_id)),
            Some(Language::Korean)
        );
    }

    #[test]
    fn search_variants_share_corpus_storage() {
        let root = tempfile::tempdir().unwrap();
        let portuguese = target(&movie("Portuguese", Source::Missing), root.path(), None).unwrap();
        assert_eq!(portuguese.corpus_code(), "por");
        assert_eq!(portuguese.opensubtitles_languages(), "pt-br,pt-pt");
        assert_eq!(
            Language::PortugueseEuropean.opensubtitles_languages(),
            "pt-br,pt-pt"
        );
        assert_eq!(Language::PortugueseEuropean.corpus_code(), "por");
        let chinese = target(&movie("Chinese", Source::Missing), root.path(), None).unwrap();
        assert_eq!(chinese.corpus_code(), "zho-hans");
    }
}
