use anyhow::{anyhow, Context, Result};
use clap::Parser;
use language_utils::MovieMetadata;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// Response from /discover/popular endpoint
#[derive(Debug, Deserialize)]
struct PopularMoviesResponse {
    data: Vec<PopularMovie>,
}

#[derive(Debug, Deserialize)]
struct PopularMovie {
    attributes: PopularMovieAttributes,
}

#[derive(Debug, Deserialize)]
struct PopularMovieAttributes {
    title: String,
    #[serde(rename = "imdb_id")]
    imdb_id: u64,
    year: Option<String>,
}

/// Response from /subtitles search endpoint
#[derive(Debug, Deserialize)]
struct SubtitleSearchResponse {
    data: Vec<SubtitleResult>,
}

#[derive(Debug, Deserialize)]
struct SubtitleResult {
    attributes: SubtitleAttributes,
}

#[derive(Debug, Deserialize)]
struct SubtitleAttributes {
    #[allow(dead_code)]
    #[serde(rename = "feature_details")]
    feature_details: FeatureDetails,
    files: Vec<SubtitleFile>,
    download_count: Option<u64>,
    #[serde(default)]
    from_trusted: Option<bool>,
    #[serde(default)]
    ai_translated: bool,
    #[serde(default)]
    machine_translated: bool,
    ratings: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct FeatureDetails {
    #[allow(dead_code)]
    #[serde(rename = "imdb_id")]
    imdb_id: u64,
    #[allow(dead_code)]
    title: String,
    #[allow(dead_code)]
    year: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct SubtitleFile {
    #[serde(rename = "file_id")]
    file_id: u64,
}

/// Download link response
#[derive(Debug, Deserialize)]
struct DownloadResponse {
    link: String,
    #[allow(dead_code)]
    #[serde(rename = "file_name")]
    file_name: String,
}

/// Subtitle line for JSON output
#[derive(Debug, Serialize)]
struct SubtitleLineJson {
    sentence: String,
    start_ms: u32,
    end_ms: u32,
}

struct OpenSubtitlesClient {
    api_key: String,
    client: reqwest::Client,
    access_token: Option<String>,
}

impl OpenSubtitlesClient {
    fn new(api_key: String) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("yap-language-learning v0.1")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            api_key,
            client,
            access_token: None,
        }
    }

    /// Login to get JWT access token
    async fn login(&mut self, username: &str, password: &str) -> Result<()> {
        let url = "https://api.opensubtitles.com/api/v1/login";

        let mut body = HashMap::new();
        body.insert("username", username);
        body.insert("password", password);

        let response = self
            .client
            .post(url)
            .header("Api-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        #[derive(Deserialize)]
        struct LoginResponse {
            token: String,
        }

        let login_response: LoginResponse = response.json().await?;
        self.access_token = Some(login_response.token);

        println!("✓ Successfully authenticated");
        Ok(())
    }

    /// Get popular movies from the discover/popular endpoint
    async fn get_popular_movies(&self, language: &str, limit: usize) -> Result<Vec<PopularMovie>> {
        let url = format!(
            "https://api.opensubtitles.com/api/v1/discover/popular?languages={language}&type=movie"
        );

        println!("Fetching popular movies: {url}");

        let response = self
            .client
            .get(&url)
            .header("Api-Key", &self.api_key)
            .send()
            .await?;

        let status = response.status();
        println!("Response status: {status}");

        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("API error ({}): {}", status, error_text));
        }

        let popular_response: PopularMoviesResponse = response.json().await?;

        println!("Found {} popular movies", popular_response.data.len());

        // Take only the first `limit` results
        Ok(popular_response.data.into_iter().take(limit).collect())
    }

    /// Search for subtitles for a specific movie by IMDB ID
    async fn search_subtitles_for_movie(
        &self,
        imdb_id: u64,
        language: &str,
    ) -> Result<Vec<SubtitleResult>> {
        let url = format!(
            "https://api.opensubtitles.com/api/v1/subtitles?imdb_id={imdb_id}&languages={language}"
        );

        let response = self
            .client
            .get(&url)
            .header("Api-Key", &self.api_key)
            .send()
            .await?
            .error_for_status()?;

        let search_response = response
            .text()
            .await
            .context("Failed to get subtitle search response")?;
        let search_response: SubtitleSearchResponse = serde_json::from_str(&search_response)
            .context(format!(
                "Failed to parse subtitle search response: {search_response}"
            ))
            .unwrap();

        // Return all results for filtering
        Ok(search_response.data)
    }

    /// Download a subtitle file
    async fn download_subtitle(&self, file_id: u64) -> Result<String> {
        let url = "https://api.opensubtitles.com/api/v1/download";

        let mut body = HashMap::new();
        body.insert("file_id", file_id);

        let mut request = self.client.post(url).header("Api-Key", &self.api_key);

        // Add Authorization header if we have a token
        if let Some(token) = &self.access_token {
            request = request.header("Authorization", format!("Bearer {token}"));
        }

        let response = request.json(&body).send().await?.error_for_status()?;

        let download_response: DownloadResponse = response.json().await?;

        // Download the actual SRT file from the link
        let srt_response = self.client.get(&download_response.link).send().await?;

        let srt_content = srt_response.text().await?;

        Ok(srt_content)
    }
}

/// Parse SRT content and extract cleaned sentences with timestamps
fn parse_srt(srt_content: &str) -> Result<Vec<SubtitleLineJson>> {
    use subparse::SubtitleFormat;

    let subtitle_file = subparse::parse_str(
        SubtitleFormat::SubRip,
        srt_content,
        25.0, // fps (not used for SRT but required parameter)
    )
    .map_err(|e| anyhow!("Failed to parse SRT: {:?}", e))?;

    let mut lines = Vec::new();

    for entry in subtitle_file
        .get_subtitle_entries()
        .map_err(|e| anyhow!("Failed to get subtitle entries: {:?}", e))?
    {
        // entry.line is Option<String>
        let text = match &entry.line {
            Some(line) => cleanup_subtitle_text(line),
            None => continue,
        };

        // Skip empty lines or very short lines
        if text.len() < 3 {
            continue;
        }

        // secs() returns i64, multiply by i64
        let start_ms = entry.timespan.start.secs() * 1000;
        let end_ms = entry.timespan.end.secs() * 1000;

        lines.push(SubtitleLineJson {
            sentence: text,
            start_ms: start_ms as u32,
            end_ms: end_ms as u32,
        });
    }

    Ok(lines)
}

/// Clean up subtitle text
fn cleanup_subtitle_text(text: &str) -> String {
    let mut result = text.to_string();

    // Remove HTML tags
    result = strip_html_tags(&result);

    // Remove hearing-impaired annotations
    result = result
        .replace("[MUSIC]", "")
        .replace("(MUSIC)", "")
        .replace("[music]", "")
        .replace("(music)", "")
        .replace("[DOOR SLAMS]", "")
        .replace("(DOOR SLAMS)", "")
        .replace("[PHONE RINGS]", "")
        .replace("(PHONE RINGS)", "");

    // Remove bracketed content (hearing impaired)
    let re_brackets = regex::Regex::new(r"\[.*?\]").unwrap();
    result = re_brackets.replace_all(&result, "").to_string();

    let re_parens = regex::Regex::new(r"\(.*?\)").unwrap();
    result = re_parens.replace_all(&result, "").to_string();

    // Remove speaker names like "JOHN:"
    let re_speaker = regex::Regex::new(r"^[A-Z][A-Z\s]+:\s*").unwrap();
    result = re_speaker.replace_all(&result, "").to_string();

    // Trim whitespace
    result = result.trim().to_string();

    // Remove multiple spaces
    let re_spaces = regex::Regex::new(r"\s+").unwrap();
    result = re_spaces.replace_all(&result, " ").to_string();

    result
}

/// Strip HTML tags from text
fn strip_html_tags(text: &str) -> String {
    let re = regex::Regex::new(r"<[^>]+>").unwrap();
    re.replace_all(text, "").to_string()
}

/// Download movie subtitles from OpenSubtitles
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Language codes (ISO 639-3: fra, eng, spa, deu, kor, zho, jpn, rus, por, ita)
    #[arg(short, long, num_args = 1..)]
    language: Vec<String>,

    /// Number of movies to download per language
    #[arg(short, long, default_value_t = 5)]
    count: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if it exists
    dotenv::dotenv().ok();

    // Get API key from environment
    let api_key = std::env::var("OPENSUBTITLES_API_KEY")
        .context("OPENSUBTITLES_API_KEY environment variable not set")?;

    // Get optional login credentials
    let username = std::env::var("OPENSUBTITLES_USERNAME").ok();
    let password = std::env::var("OPENSUBTITLES_PASSWORD").ok();

    // Parse command line arguments
    let args = Args::parse();
    let languages = args.language;
    let count = args.count;

    // Create client once for all languages
    let mut client = OpenSubtitlesClient::new(api_key);

    // Login if credentials are provided
    if let (Some(user), Some(pass)) = (username, password) {
        println!("Logging in to OpenSubtitles...");
        client.login(&user, &pass).await?;
    } else {
        println!("No login credentials provided - using unauthenticated mode (limited downloads)");
        println!("Set OPENSUBTITLES_USERNAME and OPENSUBTITLES_PASSWORD in .env to authenticate");
    }

    // Process each language
    for language_iso639_3 in languages {
        // Map ISO 639-3 to ISO 639-1 for OpenSubtitles API
        let language_iso639_1 = match language_iso639_3.as_str() {
            "fra" => "fr",
            "eng" => "en",
            "spa" => "es",
            "deu" => "de",
            "kor" => "ko",
            "zho" => "zh",
            "jpn" => "ja",
            "rus" => "ru",
            "por" => "pt",
            "ita" => "it",
            _ => {
                eprintln!("Unsupported language code: {language_iso639_3}");
                eprintln!("Supported: fra, eng, spa, deu, kor, zho, jpn, rus, por, ita");
                continue; // Skip this language instead of exiting
            }
        };

        println!(
            "\n========================================\nDownloading {count} subtitles for language: {language_iso639_3}\n========================================"
        );

        // Create output directory using ISO 639-3 to match generate-data pipeline
        let output_dir = PathBuf::from(format!(
            "./generate-data/data/{language_iso639_3}/sentence-sources/movies"
        ));
        fs::create_dir_all(&output_dir)?;
        fs::create_dir_all(output_dir.join("subtitles"))?;

        // Count already downloaded movies
        let subtitles_dir = output_dir.join("subtitles");
        let existing_count = if subtitles_dir.exists() {
            fs::read_dir(&subtitles_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "jsonl"))
                .count()
        } else {
            0
        };

        if existing_count > 0 {
            println!("Found {existing_count} already downloaded movies");
        }

        // Get popular movies using ISO 639-1 for OpenSubtitles API
        // Request more than needed to account for already-downloaded movies and low-quality subtitles
        let fetch_count = count * 3 + existing_count;
        println!("Searching for popular movies...");
        let popular_movies = client
            .get_popular_movies(language_iso639_1, fetch_count)
            .await?;

        println!("Found {} popular movies", popular_movies.len());

        let mut movies = Vec::new();
        let mut downloaded_count = 0;

        for popular_movie in popular_movies.iter() {
            // Stop if we've downloaded enough new movies
            if downloaded_count >= count {
                break;
            }
            let attrs = &popular_movie.attributes;
            let imdb_id = attrs.imdb_id;
            let imdb_id_str = format!("tt{imdb_id:07}");
            let title = &attrs.title;
            let year = attrs.year.as_ref().and_then(|y| y.parse::<u16>().ok());

            println!(
                "\n[Downloaded: {}/{}] {} ({})",
                downloaded_count,
                count,
                title,
                year.unwrap_or(0)
            );

            // Check if subtitle file already exists
            let subtitle_path = output_dir.join(format!("subtitles/{imdb_id_str}.jsonl"));
            let (_subtitle_lines, total_word_count, is_new) = if subtitle_path.exists() {
                println!("  ✓ Already downloaded, skipping...");

                // Read existing file to get word count
                let content = fs::read_to_string(&subtitle_path)?;
                let mut line_count = 0;
                let mut word_count = 0;
                for line in content.lines() {
                    if line.trim().is_empty() {
                        continue;
                    }
                    if let Ok(subtitle) = serde_json::from_str::<serde_json::Value>(line) {
                        if let Some(sentence) = subtitle.get("sentence").and_then(|s| s.as_str()) {
                            line_count += 1;
                            word_count += sentence.split_whitespace().count();
                        }
                    }
                }
                println!("  {line_count} dialogue lines, {word_count} words");
                (Vec::new(), word_count, false) // Not a new download
            } else {
                // Search for subtitles for this specific movie
                println!("  Searching for subtitles...");
                let Ok(mut subtitle_results) = client
                    .search_subtitles_for_movie(imdb_id, language_iso639_1)
                    .await
                else {
                    println!("  ✗ No subtitles found for this movie");
                    continue;
                };

                if subtitle_results.is_empty() {
                    println!("  ✗ No subtitles found for this movie");
                    continue;
                }

                println!("  Found {} subtitle options", subtitle_results.len());

                // Filter and sort to find the best subtitle
                // 1. Remove AI/machine translated
                subtitle_results
                    .retain(|s| !s.attributes.ai_translated && !s.attributes.machine_translated);

                if subtitle_results.is_empty() {
                    println!("  ✗ No human-translated subtitles available");
                    continue;
                }

                // 2. Sort by quality (prefer: trusted, higher downloads, higher ratings)
                subtitle_results.sort_by(|a, b| {
                    // Trusted first
                    match (a.attributes.from_trusted, b.attributes.from_trusted) {
                        (Some(true), _) => return std::cmp::Ordering::Less,
                        (_, Some(true)) => return std::cmp::Ordering::Greater,
                        _ => {}
                    }
                    // Then by download count
                    match (a.attributes.download_count, b.attributes.download_count) {
                        (Some(a_count), Some(b_count)) => {
                            if a_count != b_count {
                                return b_count.cmp(&a_count);
                            }
                        }
                        (Some(_), None) => return std::cmp::Ordering::Less,
                        (None, Some(_)) => return std::cmp::Ordering::Greater,
                        _ => {}
                    }
                    // Then by ratings
                    match (a.attributes.ratings, b.attributes.ratings) {
                        (Some(a_rating), Some(b_rating)) => b_rating
                            .partial_cmp(&a_rating)
                            .unwrap_or(std::cmp::Ordering::Equal),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        _ => std::cmp::Ordering::Equal,
                    }
                });

                // Try each subtitle in order until one succeeds
                let mut download_result = None;
                for subtitle_result in subtitle_results {
                    println!(
                        "  Trying subtitle: {} downloads, trusted: {}, rating: {:.1}",
                        subtitle_result.attributes.download_count.unwrap_or(0),
                        subtitle_result.attributes.from_trusted.unwrap_or(false),
                        subtitle_result.attributes.ratings.unwrap_or(0.0)
                    );

                    // Get file ID
                    let Some(file_id) = subtitle_result.attributes.files.first().map(|f| f.file_id)
                    else {
                        println!("  ✗ No files found for this subtitle, trying next...");
                        continue;
                    };

                    // Download subtitle
                    println!("  Downloading subtitle (file_id: {file_id})...");
                    let srt_content = match client.download_subtitle(file_id).await {
                        Ok(content) => content,
                        Err(e) => {
                            println!("  ✗ Download failed: {e}, trying next...");
                            continue;
                        }
                    };

                    // Parse SRT
                    println!("  Parsing SRT...");
                    let subtitle_lines = match parse_srt(&srt_content) {
                        Ok(lines) => lines,
                        Err(e) => {
                            println!("  ✗ Parse failed: {e}, trying next...");
                            continue;
                        }
                    };

                    println!("  Extracted {} dialogue lines", subtitle_lines.len());

                    // Count total words
                    let total_word_count: usize = subtitle_lines
                        .iter()
                        .map(|line| line.sentence.split_whitespace().count())
                        .sum();

                    // Save subtitle lines to JSON
                    let subtitle_file = match fs::File::create(&subtitle_path) {
                        Ok(file) => file,
                        Err(e) => {
                            println!("  ✗ Failed to create file: {e}, trying next...");
                            continue;
                        }
                    };
                    for line in &subtitle_lines {
                        if let Err(e) = serde_json::to_writer(&subtitle_file, &line) {
                            println!("  ✗ Failed to write subtitle: {e}");
                            break;
                        }
                        if let Err(e) = writeln!(&subtitle_file) {
                            println!("  ✗ Failed to write newline: {e}");
                            break;
                        }
                    }

                    println!("  ✓ Saved to {}", subtitle_path.display());
                    download_result = Some((subtitle_lines, total_word_count, true));
                    break;
                }

                match download_result {
                    Some(data) => data,
                    None => {
                        println!("  ✗ Failed to download any subtitle for this movie");
                        continue;
                    }
                }
            };

            // Count this as a new download if it wasn't skipped
            if is_new {
                downloaded_count += 1;
            }

            // Create movie metadata
            let movie = MovieMetadata {
                id: imdb_id_str.clone(),
                title: title.to_string(),
                year,
                genres: vec![],   // TODO: Could fetch from TMDB API
                poster_url: None, // TODO: Could fetch from TMDB API
                total_dialogue_word_count: total_word_count as u64,
            };

            movies.push(movie);

            // Rate limiting: wait 1 second between requests
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        // Warn if we couldn't download enough movies
        if downloaded_count < count {
            println!(
                "\n⚠ Warning: Only found {downloaded_count} movies with subtitles (requested {count})"
            );
        }

        // Save metadata
        let metadata_path = output_dir.join("metadata.jsonl");
        let metadata_file = fs::File::create(&metadata_path)?;
        for movie in &movies {
            serde_json::to_writer(&metadata_file, &movie)?;
            writeln!(&metadata_file)?;
        }

        println!("\nMetadata saved to {}", metadata_path.display());
        println!(
            "Done! Downloaded {} new movies for {} (total: {} movies)",
            downloaded_count,
            language_iso639_3,
            movies.len()
        );
    }

    println!("\n========================================");
    println!("All languages processed successfully!");
    println!("========================================");

    Ok(())
}
