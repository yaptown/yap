//! The shared TTS cache: verified clips in the public `yap-tts-cache` R2
//! bucket, served at [`language_utils::TTS_CACHE_ORIGIN`].
//!
//! Objects are named by [`language_utils::tts_cache_filename`], the same key
//! every client already computes for its own local cache. That is what makes
//! the bucket public-by-design: a client GETs the key directly and only asks
//! `/tts` on a miss, so a clip anyone has had verified never costs a Fly
//! request (or a cold start) again.
//!
//! Only clips that passed the backend's checks are written. A salvage clip —
//! the best-effort audio returned when every provider failed verification —
//! is handed to the caller but never stored, so it keeps being retried on
//! every fetch until some provider gets it right. Caching it would freeze a
//! clip we already doubt.
//!
//! Writing needs R2 credentials (`R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`,
//! plus `CLOUDFLARE_ACCOUNT_ID` for the endpoint). Without them the cache is
//! read-only: lookups still work because the bucket is public.

use language_utils::{audio_mime_type, tts_cache_url};
use rusty_s3::{Bucket, Credentials, S3Action, UrlStyle};
use std::sync::LazyLock;
use std::time::Duration;

const BUCKET: &str = "yap-tts-cache";

/// How long a presigned PUT stays valid. Only needs to outlive one upload.
const PRESIGN_TTL: Duration = Duration::from_secs(5 * 60);

struct Writer {
    bucket: Bucket,
    credentials: Credentials,
}

fn env_non_empty(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.is_empty())
}

static WRITER: LazyLock<Option<Writer>> = LazyLock::new(|| {
    let account = env_non_empty("CLOUDFLARE_ACCOUNT_ID")?;
    let key = env_non_empty("R2_ACCESS_KEY_ID")?;
    let secret = env_non_empty("R2_SECRET_ACCESS_KEY")?;
    let endpoint = format!("https://{account}.r2.cloudflarestorage.com")
        .parse()
        .ok()?;
    // R2's S3 endpoint is per-account, so the bucket goes in the path.
    let bucket = Bucket::new(endpoint, UrlStyle::Path, BUCKET, "auto").ok()?;
    Some(Writer {
        bucket,
        credentials: Credentials::new(key, secret),
    })
});

/// Whether this process can write to the bucket. Reads never need this.
pub fn write_enabled() -> bool {
    WRITER.is_some()
}

/// The cached clip for `cache_filename`, if the bucket has one. A miss, an
/// outage, or an empty body all read as `None` — the caller synthesizes as
/// if the cache didn't exist.
///
/// The URL carries a one-off query string. Cloudflare caches the bucket's
/// 404s at the edge for minutes, and this lookup runs right before the
/// write that would turn a miss into a hit: without the bust, every request
/// for the same clip inside that window would see the stale 404, synthesize
/// again, and overwrite the object with a different (if equally verified)
/// clip. R2 ignores the query when resolving the object; the edge keys its
/// cache on it, so a fresh value always reaches the bucket.
pub async fn lookup(http: &reqwest::Client, cache_filename: &str) -> Option<Vec<u8>> {
    let url = format!(
        "{}?fresh={}",
        tts_cache_url(cache_filename),
        uuid::Uuid::new_v4().simple()
    );
    let response = http.get(url).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    let bytes = response.bytes().await.ok()?;
    (!bytes.is_empty()).then(|| bytes.to_vec())
}

/// Find a cached clip's URL without downloading it before redirecting the learner.
/// Positive edge hits are immutable and cheap. Only a miss needs to bypass
/// the edge, since a cached 404 may hide a newly uploaded recording. Return
/// the URL that succeeded, including its cache buster when needed: redirecting
/// to the ordinary URL in that case would send the learner to the stale 404.
pub async fn existing_url(http: &reqwest::Client, cache_filename: &str) -> Option<String> {
    existing_url_at(http, &tts_cache_url(cache_filename)).await
}

async fn existing_url_at(http: &reqwest::Client, url: &str) -> Option<String> {
    for fresh in [false, true] {
        let mut request = http.head(url);
        if fresh {
            request = request.query(&[("fresh", uuid::Uuid::new_v4().simple().to_string())]);
        }
        if let Ok(response) = request.send().await
            && response.status().is_success()
        {
            return Some(response.url().to_string());
        }
    }
    None
}

/// Store a clip that passed every check, without holding up the response
/// that carries it. Failures are logged and otherwise ignored: the caller
/// already has its audio, and the next miss will simply try again.
pub fn store_in_background(http: reqwest::Client, cache_filename: String, audio: Vec<u8>) {
    if !write_enabled() {
        return;
    }
    tokio::spawn(async move {
        if let Err(reason) = store(&http, &cache_filename, audio).await {
            eprintln!("TTS cache: failed to store {cache_filename}: {reason}");
        }
    });
}

async fn store(http: &reqwest::Client, cache_filename: &str, audio: Vec<u8>) -> Result<(), String> {
    let writer = WRITER.as_ref().ok_or("no R2 credentials")?;
    let content_type = audio_mime_type(&audio);
    let url = writer
        .bucket
        .put_object(Some(&writer.credentials), cache_filename)
        .sign(PRESIGN_TTL);
    let response = http
        .put(url)
        .header(reqwest::header::CONTENT_TYPE, content_type)
        // Keys are content hashes of the full request (see
        // `tts_cache_filename`), so an object never changes under its name
        // and every layer between here and the learner may keep it forever.
        .header(
            reqwest::header::CACHE_CONTROL,
            "public, max-age=31536000, immutable",
        )
        .body(audio)
        .send()
        .await
        .map_err(|e| format!("upload request failed: {e}"))?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("R2 responded {}", response.status()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, http::StatusCode, routing::head};
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn existence_uses_edge_hits_but_rechecks_misses_at_origin() {
        for (edge, origin, expected, requests) in [
            (StatusCode::OK, StatusCode::NOT_FOUND, true, 1),
            (StatusCode::NOT_FOUND, StatusCode::OK, true, 2),
            (StatusCode::NOT_FOUND, StatusCode::NOT_FOUND, false, 2),
            (StatusCode::BAD_GATEWAY, StatusCode::OK, true, 2),
        ] {
            let queries = Arc::new(Mutex::new(Vec::new()));
            let observed = queries.clone();
            let app = Router::new().route(
                "/audio",
                head(move |uri: axum::http::Uri| {
                    let observed = observed.clone();
                    async move {
                        let query = uri.query().map(str::to_owned);
                        let status = if query.is_some() { origin } else { edge };
                        observed.lock().unwrap().push(query);
                        status
                    }
                }),
            );
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let url = format!("http://{}/audio", listener.local_addr().unwrap());
            let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
            let cached = existing_url_at(&reqwest::Client::new(), &url).await;
            assert_eq!(cached.is_some(), expected);
            server.abort();
            let queries = queries.lock().unwrap();
            assert_eq!(queries.len(), requests);
            if let Some(cached) = cached {
                let cached = reqwest::Url::parse(&cached).unwrap();
                assert_eq!(
                    cached.query(),
                    queries.last().unwrap().as_deref(),
                    "redirect to the URL that succeeded, not a stale 404"
                );
            }
            assert!(queries[0].is_none());
            if requests == 2 {
                assert!(queries[1].as_ref().unwrap().starts_with("fresh="));
            }
        }
    }
}
