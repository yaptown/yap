//! Shared movie metadata providers and on-disk metadata/poster refresh.

use anyhow::{Context, Result, anyhow};
use language_utils::{Language, MovieMetadataBasic};
use serde::Deserialize;
use std::{collections::BTreeSet, fs, path::Path};

/// OMDB API response
#[derive(Debug, Deserialize)]
struct OmdbResponse {
    #[serde(rename = "Ratings", default)]
    ratings: Vec<OmdbRating>,
    /// OMDb reports failures (bad key, unknown id) as 200s with this set.
    #[serde(rename = "Error")]
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OmdbRating {
    #[serde(rename = "Source")]
    source: String,
    #[serde(rename = "Value")]
    value: String,
}

pub struct OmdbClient {
    api_key: String,
    client: reqwest::Client,
}

impl OmdbClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_rotten_tomatoes_score(&self, imdb_id: &str) -> Option<u8> {
        let url = format!(
            "https://www.omdbapi.com/?i={}&apikey={}",
            imdb_id, self.api_key
        );
        let omdb: OmdbResponse = match async {
            self.client
                .get(&url)
                .send()
                .await
                .map_err(reqwest::Error::without_url)?
                .json()
                .await
                .map_err(reqwest::Error::without_url)
        }
        .await
        {
            Ok(omdb) => omdb,
            Err(e) => {
                println!("  ⚠ OMDb request failed for {imdb_id}: {e}");
                return None;
            }
        };
        if let Some(error) = &omdb.error {
            // An invalid key fails every film the same way; without this the
            // run just quietly writes nulls for every score.
            println!("  ⚠ OMDb error for {imdb_id}: {error}");
            return None;
        }
        for rating in &omdb.ratings {
            if rating.source == "Rotten Tomatoes" {
                return rating.value.trim_end_matches('%').parse().ok();
            }
        }
        None
    }
}

/// TMDB API Movie Response
#[derive(Debug, Deserialize)]
pub struct TmdbMovie {
    pub title: String,
    pub release_date: Option<String>,
    pub poster_path: Option<String>,
    /// Normalized to real ISO 639-1 on the way in, so freshly written
    /// metadata never carries TMDB's `cn`.
    #[serde(
        default,
        deserialize_with = "language_utils::deserialize_original_language"
    )]
    pub original_language: Option<String>,
}

impl TmdbMovie {
    /// The `metadata.jsonl` row for this film.
    pub fn metadata(&self, imdb_id: &str, rotten_tomatoes_score: Option<u8>) -> MovieMetadataBasic {
        MovieMetadataBasic {
            id: imdb_id.to_owned(),
            title: self.title.clone(),
            year: self
                .release_date
                .as_deref()
                .and_then(|d| d.split('-').next()?.parse().ok()),
            original_language: self.original_language.clone(),
            rotten_tomatoes_score,
        }
    }
}

/// TMDB Find API Response
#[derive(Debug, Deserialize)]
struct TmdbFindResponse {
    movie_results: Vec<TmdbMovie>,
}

pub struct TmdbClient {
    api_key: String,
    client: reqwest::Client,
}

impl TmdbClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_movie(&self, imdb_id: &str, language: &str) -> Result<TmdbMovie> {
        // Use the find endpoint to search by IMDB ID
        let url = format!(
            "https://api.themoviedb.org/3/find/{}?api_key={}&external_source=imdb_id&language={}",
            imdb_id, self.api_key, language
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(reqwest::Error::without_url);
        // Also pace failed/unknown lookups.
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        let response = response?
            .error_for_status()
            .map_err(reqwest::Error::without_url)?;
        let response_text = response.text().await.map_err(reqwest::Error::without_url)?;
        let find_response: TmdbFindResponse = serde_json::from_str(&response_text)?;

        if find_response.movie_results.is_empty() {
            return Err(anyhow!("No movie found for IMDB ID {imdb_id}"));
        }

        Ok(find_response.movie_results.into_iter().next().unwrap())
    }
}

fn read_optional(path: &std::path::Path) -> Result<Option<Vec<u8>>> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e).with_context(|| format!("reading {}", path.display())),
    }
}

/// Append only missing IDs, retaining every existing byte (including unknown
/// fields and blank lines). Return whether a row was/would be appended.
pub fn append_movie_metadata(
    path: &std::path::Path,
    movie: &language_utils::MovieMetadataBasic,
    dry_run: bool,
) -> Result<bool> {
    use std::io::Write;

    #[derive(serde::Deserialize)]
    struct Id {
        id: String,
    }

    let existing = read_optional(path)?.unwrap_or_default();
    for line in existing.split(|&b| b == b'\n') {
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        let row: Id =
            serde_json::from_slice(line).with_context(|| format!("parsing {}", path.display()))?;
        if row.id == movie.id {
            return Ok(false);
        }
    }
    if !dry_run {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut row = serde_json::to_vec(movie)?;
        row.push(b'\n');
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        if !existing.is_empty() && !existing.ends_with(b"\n") {
            file.write_all(b"\n")?;
        }
        file.write_all(&row)?;
    }
    Ok(true)
}

/// Fetch a poster, rejecting HTTP errors rather than saving them as JPEGs.
pub async fn fetch_image_bytes(client: &reqwest::Client, url: &str) -> Result<Vec<u8>> {
    Ok(client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?
        .to_vec())
}

#[derive(Debug, Default)]
pub struct RefreshReport {
    pub appended_ids: Vec<String>,
    pub posters_fetched: Vec<String>,
    pub no_metadata: Vec<String>,
    pub no_poster: Vec<String>,
}

impl std::fmt::Display for RefreshReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} metadata rows appended, {} posters fetched, {} missing metadata, {} missing posters",
            self.appended_ids.len(),
            self.posters_fetched.len(),
            self.no_metadata.len(),
            self.no_poster.len()
        )
    }
}

/// Every IMDb id with a subtitle on disk, whichever route it arrived by.
fn subtitle_ids(movies_dir: &Path) -> Result<BTreeSet<String>> {
    let mut ids = BTreeSet::new();
    for (directory, extension) in [("subtitles-raw", "srt"), ("subtitles", "jsonl")] {
        let entries = match fs::read_dir(movies_dir.join(directory)) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e.into()),
        };
        for entry in entries {
            let path = entry?.path();
            if path.extension().is_some_and(|e| e == extension)
                && let Some(id) = path.file_stem().and_then(|s| s.to_str())
                && id.starts_with("tt")
            {
                ids.insert(id.to_owned());
            }
        }
    }
    Ok(ids)
}

/// Fill missing metadata and posters without rewriting rows or retrying existing
/// Rotten Tomatoes scores. Provider failures are reported per film so a missing
/// poster cannot block a pack build; local I/O and malformed metadata are errors.
pub async fn refresh(
    movies_dir: &Path,
    language: Language,
    tmdb: &TmdbClient,
    omdb: &OmdbClient,
) -> Result<RefreshReport> {
    let metadata_path = movies_dir.join("metadata.jsonl");
    let existing = read_optional(&metadata_path)?.unwrap_or_default();
    let mut metadata = BTreeSet::new();
    for line in existing.split(|&b| b == b'\n') {
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        let movie: MovieMetadataBasic = serde_json::from_slice(line)
            .with_context(|| format!("parsing {}", metadata_path.display()))?;
        metadata.insert(movie.id);
    }
    let mut ids = subtitle_ids(movies_dir)?;
    ids.extend(metadata.iter().cloned());
    let mut report = RefreshReport::default();
    for id in ids {
        let missing_metadata = !metadata.contains(&id);
        let poster_file = movies_dir.join("posters").join(format!("{id}.jpg"));
        if !missing_metadata && poster_file.exists() {
            continue;
        }
        // One lookup serves both the metadata and poster, including new rows.
        match tmdb.get_movie(&id, language.tmdb_language_code()).await {
            Ok(movie) => {
                if missing_metadata {
                    let row = movie.metadata(&id, omdb.get_rotten_tomatoes_score(&id).await);
                    if append_movie_metadata(&metadata_path, &row, false)? {
                        report.appended_ids.push(id.clone());
                    }
                }
                if !poster_file.exists()
                    && let Some(poster_path) = movie.poster_path
                {
                    let url = format!("https://image.tmdb.org/t/p/w500{poster_path}");
                    match fetch_image_bytes(&tmdb.client, &url).await {
                        Ok(bytes) => {
                            fs::create_dir_all(movies_dir.join("posters"))?;
                            fs::write(&poster_file, bytes)?;
                            report.posters_fetched.push(id.clone());
                        }
                        Err(e) => eprintln!("⚠ WARNING: poster request failed for {id}: {e}"),
                    }
                }
            }
            Err(e) => {
                eprintln!("⚠ WARNING: TMDB metadata unavailable for {id}: {e}");
                if missing_metadata {
                    report.no_metadata.push(id.clone());
                }
            }
        }
        if !poster_file.exists() {
            report.no_poster.push(id);
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use language_utils::MovieMetadataBasic;

    fn movie() -> MovieMetadataBasic {
        MovieMetadataBasic {
            id: "tt1234567".into(),
            title: "A title\nwith a newline".into(),
            year: Some(2001),
            original_language: Some("fr".into()),
            rotten_tomatoes_score: None,
        }
    }

    #[test]
    fn existing_metadata_is_byte_preserved() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("metadata.jsonl");
        let original =
            b"\n {\"id\":\"tt1234567\", \"title\":\"Hand curated\", \"extra\":42}\r\n \t\n";
        std::fs::write(&path, original).unwrap();
        assert!(!append_movie_metadata(&path, &movie(), false).unwrap());
        assert_eq!(std::fs::read(&path).unwrap(), original);
    }

    #[test]
    fn new_metadata_is_one_line_and_preserves_existing_bytes() {
        for original in [
            b"".as_slice(),
            b"\n \t\n{\"id\":\"other\",\"extra\":true}\n",
            b"\n{\"id\":\"other\"}",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("metadata.jsonl");
            std::fs::write(&path, original).unwrap();
            assert!(append_movie_metadata(&path, &movie(), false).unwrap());
            let bytes = std::fs::read(&path).unwrap();
            assert!(bytes.starts_with(original));
            let suffix = &bytes[original.len()..];
            let suffix = if !original.is_empty() && !original.ends_with(b"\n") {
                assert_eq!(suffix[0], b'\n');
                &suffix[1..]
            } else {
                suffix
            };
            assert_eq!(suffix.iter().filter(|&&b| b == b'\n').count(), 1);
            assert!(suffix.ends_with(b"\n"));
            assert_eq!(
                serde_json::from_slice::<MovieMetadataBasic>(suffix).unwrap(),
                movie()
            );
            assert!(!append_movie_metadata(&path, &movie(), false).unwrap());
            assert_eq!(std::fs::read(&path).unwrap(), bytes);
        }
    }

    #[test]
    fn metadata_dry_run_does_not_create_or_modify_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing/metadata.jsonl");
        assert!(append_movie_metadata(&path, &movie(), true).unwrap());
        assert!(!path.parent().unwrap().exists());
        let path = dir.path().join("metadata.jsonl");
        let original = b"{\"id\":\"other\"}";
        std::fs::write(&path, original).unwrap();
        assert!(append_movie_metadata(&path, &movie(), true).unwrap());
        assert_eq!(std::fs::read(&path).unwrap(), original);
    }

    #[test]
    fn invalid_or_unreadable_metadata_is_not_treated_as_missing() {
        let dir = tempfile::tempdir().unwrap();
        assert!(append_movie_metadata(dir.path(), &movie(), false).is_err());
        let path = dir.path().join("metadata.jsonl");
        std::fs::write(&path, b"not json\n").unwrap();
        assert!(append_movie_metadata(&path, &movie(), false).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"not json\n");
    }

    #[test]
    fn finds_only_imdb_subtitles_and_deduplicates_raw_and_derived() {
        let dir = tempfile::tempdir().unwrap();
        for file in [
            "subtitles-raw/tt123.srt",
            "subtitles/tt123.jsonl",
            "subtitles/tt456.jsonl",
            "subtitles-raw/tt789.srt",
            "subtitles-raw/tt999.jsonl",
            "subtitles/tt888.srt",
            "subtitles-raw/not-imdb.srt",
            "subtitles/tt123.jsonl.bak",
        ] {
            let path = dir.path().join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "").unwrap();
        }
        assert_eq!(
            subtitle_ids(dir.path()).unwrap(),
            ["tt123", "tt456", "tt789"]
                .into_iter()
                .map(str::to_owned)
                .collect()
        );
    }

    #[tokio::test]
    async fn complete_metadata_and_posters_need_no_network_or_score_retry() {
        let dir = tempfile::tempdir().unwrap();
        let metadata_path = dir.path().join("metadata.jsonl");
        append_movie_metadata(&metadata_path, &movie(), false).unwrap();
        let original = fs::read(&metadata_path).unwrap();
        fs::create_dir(dir.path().join("posters")).unwrap();
        let poster = dir.path().join("posters/tt1234567.jpg");
        fs::write(&poster, b"existing poster").unwrap();
        let report = refresh(
            dir.path(),
            Language::French,
            &TmdbClient::new(String::new()),
            &OmdbClient::new(String::new()),
        )
        .await
        .unwrap();
        assert!(report.appended_ids.is_empty());
        assert!(report.posters_fetched.is_empty());
        assert!(report.no_metadata.is_empty());
        assert!(report.no_poster.is_empty());
        assert_eq!(fs::read(metadata_path).unwrap(), original);
        assert_eq!(fs::read(poster).unwrap(), b"existing poster");
    }

    #[tokio::test]
    async fn empty_directory_needs_no_network_and_bad_metadata_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let tmdb = TmdbClient::new(String::new());
        let omdb = OmdbClient::new(String::new());
        let report = refresh(dir.path(), Language::French, &tmdb, &omdb)
            .await
            .unwrap();
        assert!(report.appended_ids.is_empty());
        assert!(report.no_metadata.is_empty());
        assert!(!dir.path().join("metadata.jsonl").exists());
        fs::write(dir.path().join("metadata.jsonl"), "bad json").unwrap();
        assert!(
            refresh(dir.path(), Language::French, &tmdb, &omdb)
                .await
                .is_err()
        );
    }
}
