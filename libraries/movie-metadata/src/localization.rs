//! TMDB's ordinary title lookup silently falls back to English. Only an explicit
//! country/language translation is evidence of a regional release title.
use super::*;
use language_utils::MovieLocalization;
use serde_json::Value;
use std::collections::BTreeMap;
use std::io::Write;

pub(super) fn varieties(language: Language) -> Option<[Language; 2]> {
    match language {
        Language::PortugueseBrazilian | Language::PortugueseEuropean => {
            Some([Language::PortugueseBrazilian, Language::PortugueseEuropean])
        }
        Language::SpanishLatinAmerican | Language::SpanishPeninsular => {
            Some([Language::SpanishLatinAmerican, Language::SpanishPeninsular])
        }
        _ => None,
    }
}

fn translated_title<'a>(translations: &'a Value, locale: &str) -> Option<&'a str> {
    let (language, country) = locale.split_once('-').unwrap();
    translations["translations"]
        .as_array()?
        .iter()
        .find(|translation| {
            translation["iso_639_1"] == language && translation["iso_3166_1"] == country
        })?["data"]["title"]
        .as_str()
        .filter(|title| !title.trim().is_empty())
}

impl TmdbClient {
    async fn details(&self, id: u64, locale: &str, append: &str) -> Result<Value> {
        let response = self
            .client
            .get(format!("https://api.themoviedb.org/3/movie/{id}"))
            .query(&[
                ("api_key", self.api_key.as_str()),
                ("language", locale),
                ("append_to_response", append),
                (
                    "include_image_language",
                    &format!("{},pt,es,null", locale.split('-').next().unwrap()),
                ),
            ])
            .send()
            .await
            .map_err(reqwest::Error::without_url)?;
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        response
            .error_for_status()
            .map_err(reqwest::Error::without_url)?
            .json()
            .await
            .map_err(reqwest::Error::without_url)
            .map_err(Into::into)
    }

    async fn regional_metadata(
        &self,
        imdb: &str,
        varieties: [Language; 2],
    ) -> Result<MovieMetadataBasic> {
        let found = self.get_movie(imdb, "en-US").await?;
        let original_language = found.original_language.as_deref().unwrap_or("en");
        let details = self
            .details(found.id, original_language, "translations,images")
            .await?;
        let posters = details["images"]["posters"]
            .as_array()
            .context("TMDB posters missing")?;
        let original_poster = posters
            .iter()
            .find(|poster| poster["iso_639_1"] == original_language)
            .or_else(|| posters.iter().find(|poster| poster["iso_639_1"].is_null()))
            .and_then(|poster| poster["file_path"].as_str())
            .map(str::to_owned);
        let mut metadata = found.metadata(imdb, None);
        metadata.title = found.original_title.clone();
        metadata.poster_path = original_poster.clone();
        for language in varieties {
            let locale = language.tmdb_language_code();
            let localization =
                if let Some(title) = translated_title(&details["translations"], locale) {
                    let localized = self.details(found.id, locale, "").await?;
                    // Images carry a language, not a country. The locale endpoint selects
                    // the regional artwork; validate its language to reject English fallback.
                    let poster = localized["poster_path"]
                        .as_str()
                        .filter(|path| {
                            posters.iter().any(|poster| {
                                poster["file_path"] == *path
                                    && poster["iso_639_1"] == language.iso_639_1()
                            })
                        })
                        .map(str::to_owned)
                        .or_else(|| original_poster.clone());
                    MovieLocalization {
                        title: title.into(),
                        poster_path: poster,
                    }
                } else {
                    MovieLocalization {
                        title: found.original_title.clone(),
                        poster_path: original_poster.clone(),
                    }
                };
            metadata
                .localizations
                .insert(language.code().into(), localization);
        }
        Ok(metadata)
    }
}

pub(super) async fn refresh(
    movies_dir: &Path,
    varieties: [Language; 2],
    tmdb: &TmdbClient,
    omdb: &OmdbClient,
) -> Result<RefreshReport> {
    let path = movies_dir.join("metadata.jsonl");
    let mut existing = BTreeMap::new();
    for line in read_optional(&path)?
        .unwrap_or_default()
        .split(|b| *b == b'\n')
    {
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        let movie: MovieMetadataBasic = serde_json::from_slice(line)?;
        existing.insert(movie.id.clone(), movie);
    }
    let mut ids = subtitle_ids(movies_dir)?;
    ids.extend(existing.keys().cloned());
    let mut report = RefreshReport::default();
    for id in ids {
        let old = existing.get(&id);
        let complete = old.is_some_and(|m| {
            varieties
                .iter()
                .all(|v| m.localizations.contains_key(v.code()))
        });
        let movie = if complete {
            old.unwrap().clone()
        } else {
            match tmdb.regional_metadata(&id, varieties).await {
                Ok(mut movie) => {
                    if let Some(old) = old {
                        movie.variety = old.variety.clone();
                        movie.rotten_tomatoes_score = old.rotten_tomatoes_score;
                    } else {
                        movie.rotten_tomatoes_score = omdb.get_rotten_tomatoes_score(&id).await;
                    }
                    movie
                }
                Err(error) => {
                    eprintln!("Regional metadata unavailable for {id}: {error:#}");
                    report.no_metadata.push(id);
                    continue;
                }
            }
        };
        // Fallback and regional images have distinct filenames. Replacing the old
        // corpus poster once makes the fallback genuinely original-language artwork.
        // A region that kept the original artwork reuses the original file.
        let images = std::iter::once((format!("{id}.jpg"), movie.poster_path.as_deref())).chain(
            movie
                .localizations
                .iter()
                .filter(|(_, localization)| localization.poster_path != movie.poster_path)
                .map(|(code, localization)| {
                    (
                        format!("{id}.{code}.jpg"),
                        localization.poster_path.as_deref(),
                    )
                }),
        );
        for (filename, poster) in images {
            let destination = movies_dir.join("posters").join(&filename);
            if complete && destination.exists() {
                continue;
            }
            let Some(poster) = poster else {
                report.no_poster.push(filename);
                continue;
            };
            match fetch_image_bytes(
                &tmdb.client,
                &format!("https://image.tmdb.org/t/p/w500{poster}"),
            )
            .await
            {
                Ok(bytes) => {
                    fs::create_dir_all(destination.parent().unwrap())?;
                    let mut file = tempfile::NamedTempFile::new_in(destination.parent().unwrap())?;
                    file.write_all(&bytes)?;
                    file.persist(destination)?;
                    report.posters_fetched.push(filename);
                }
                Err(error) => {
                    eprintln!("Poster unavailable for {filename}: {error:#}");
                    report.no_poster.push(filename);
                }
            }
        }
        if append_movie_metadata(&path, &movie, false)? {
            report.appended_ids.push(id);
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn requires_exact_regional_title_instead_of_tmdb_fallback() {
        let translations = serde_json::json!({"translations": [
            {"iso_639_1":"pt", "iso_3166_1":"BR", "data":{"title":"Brasil"}},
            {"iso_639_1":"pt", "iso_3166_1":"PT", "data":{"title":""}},
        ]});
        assert_eq!(translated_title(&translations, "pt-BR"), Some("Brasil"));
        assert_eq!(translated_title(&translations, "pt-PT"), None);
        assert_eq!(translated_title(&translations, "es-MX"), None);
    }
}
