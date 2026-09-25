//! Temporary, write-once Anki packages. The bucket must expire objects after eight days.
use axum::{
    Json,
    body::Bytes,
    extract::{Path, Query},
    http::StatusCode,
};
use language_utils::ANKI_DECKS_ORIGIN;
use rusty_s3::{Bucket, Credentials, S3Action, UrlStyle};
use serde::Deserialize;
use std::{collections::BTreeMap, sync::LazyLock, time::Duration};
use uuid::Uuid;

pub const MAX_PACKAGE_BYTES: usize = 64 * 1024 * 1024;
type Failure = (StatusCode, &'static str);

#[derive(Deserialize)]
pub struct Params {
    d: String,
    code: String,
}

fn valid_code(code: &str) -> bool {
    !code.is_empty()
        && code.len() <= 12
        && code.bytes().all(|b| b.is_ascii_lowercase() || b == b'-')
}

// The browser's fflate writer uses level: 0 (stored entries). Accept just that
// format: no decompressor, ZIP64, encryption, or data descriptors are needed.
// In particular, the 64 MiB wire limit also bounds all uncompressed data.
fn u16_at(bytes: &[u8], offset: usize) -> Result<usize, &'static str> {
    let value = bytes.get(offset..offset + 2).ok_or("Truncated ZIP")?;
    Ok(u16::from_le_bytes(value.try_into().unwrap()) as usize)
}
fn u32_at(bytes: &[u8], offset: usize) -> Result<usize, &'static str> {
    let value = bytes.get(offset..offset + 4).ok_or("Truncated ZIP")?;
    Ok(u32::from_le_bytes(value.try_into().unwrap()) as usize)
}
fn digits(name: &str) -> bool {
    !name.is_empty() && name.bytes().all(|b| b.is_ascii_digit())
}

pub fn validate_package(bytes: &[u8]) -> Result<(), &'static str> {
    if bytes.len() > MAX_PACKAGE_BYTES {
        return Err("Package exceeds 64 MiB");
    }
    let end = bytes.len().checked_sub(22).ok_or("Truncated ZIP")?;
    // Our writer emits no ZIP comment; rejecting trailing data also prevents
    // ambiguous archives whose readers disagree about the directory.
    if bytes.get(end..end + 4) != Some(b"PK\x05\x06")
        || u16_at(bytes, end + 4)? != 0
        || u16_at(bytes, end + 6)? != 0
        || u16_at(bytes, end + 20)? != 0
        || u16_at(bytes, end + 8)? != u16_at(bytes, end + 10)?
    {
        return Err("Invalid ZIP directory");
    }
    let directory = u32_at(bytes, end + 16)?;
    if directory.checked_add(u32_at(bytes, end + 12)?) != Some(end) {
        return Err("Invalid ZIP directory size");
    }
    let mut cursor = directory;
    let mut local = 0;
    let mut entries = BTreeMap::new();
    for _ in 0..u16_at(bytes, end + 10)? {
        if bytes.get(cursor..cursor + 4) != Some(b"PK\x01\x02") {
            return Err("Invalid ZIP entry");
        }
        let flags = u16_at(bytes, cursor + 8)?;
        let size = u32_at(bytes, cursor + 20)?;
        if flags & !0x800 != 0
            || u16_at(bytes, cursor + 10)? != 0
            || size != u32_at(bytes, cursor + 24)?
            || u16_at(bytes, cursor + 34)? != 0
        {
            return Err("Only uncompressed, unencrypted ZIP entries are supported");
        }
        let name_len = u16_at(bytes, cursor + 28)?;
        let name_end = cursor + 46 + name_len;
        let name = std::str::from_utf8(
            bytes
                .get(cursor + 46..name_end)
                .ok_or("Truncated ZIP name")?,
        )
        .map_err(|_| "Invalid ZIP name")?;
        if name != "collection.anki2" && name != "media" && !digits(name) {
            return Err("Unexpected ZIP entry");
        }
        if u32_at(bytes, cursor + 42)? != local
            || bytes.get(local..local + 4) != Some(b"PK\x03\x04")
            || u16_at(bytes, local + 6)? != flags
            || u16_at(bytes, local + 8)? != 0
            || u32_at(bytes, local + 14)? != u32_at(bytes, cursor + 16)?
            || u32_at(bytes, local + 18)? != size
            || u32_at(bytes, local + 22)? != size
            || u16_at(bytes, local + 26)? != name_len
            || bytes.get(local + 30..local + 30 + name_len) != Some(name.as_bytes())
        {
            return Err("ZIP local entry does not match directory");
        }
        let start = local + 30 + name_len + u16_at(bytes, local + 28)?;
        local = start.checked_add(size).ok_or("Invalid ZIP size")?;
        if local > directory {
            return Err("ZIP entry overlaps directory");
        }
        let data = bytes.get(start..local).ok_or("Truncated ZIP entry")?;
        if entries.insert(name, data).is_some() {
            return Err("Duplicate ZIP entry");
        }
        cursor = name_end + u16_at(bytes, cursor + 30)? + u16_at(bytes, cursor + 32)?;
        if cursor > end {
            return Err("Invalid ZIP directory size");
        }
    }
    if cursor != end || local != directory {
        return Err("Invalid ZIP directory size");
    }
    if !entries
        .get("collection.anki2")
        .is_some_and(|db| db.starts_with(b"SQLite format 3\0"))
    {
        return Err("Missing or invalid SQLite collection");
    }
    let media: serde_json::Value =
        serde_json::from_slice(entries.get("media").ok_or("Missing media manifest")?)
            .map_err(|_| "Invalid media JSON")?;
    let media = media
        .as_object()
        .ok_or("Media manifest must be an object")?;
    for (key, value) in media {
        if !digits(key)
            || !value
                .as_str()
                .is_some_and(|name| !name.is_empty() && !name.contains(['/', '\\']))
        {
            return Err("Invalid media filename");
        }
    }
    Ok(())
}

struct Writer {
    bucket: Bucket,
    credentials: Credentials,
}
static WRITER: LazyLock<Option<Writer>> = LazyLock::new(|| {
    let env = |name| std::env::var(name).ok().filter(|s| !s.is_empty());
    let account = env("CLOUDFLARE_ACCOUNT_ID")?;
    Some(Writer {
        bucket: Bucket::new(
            format!("https://{account}.r2.cloudflarestorage.com")
                .parse()
                .ok()?,
            UrlStyle::Path,
            "yap-anki-decks",
            "auto",
        )
        .ok()?,
        credentials: Credentials::new(env("R2_ACCESS_KEY_ID")?, env("R2_SECRET_ACCESS_KEY")?),
    })
});

fn internal(error: impl std::fmt::Display) -> Failure {
    eprintln!("anki package: {error}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Could not store deck package",
    )
}

/// Only a confirmed 404 permits an upload; storage outages must not look like misses.
fn require_missing_package(
    status: Result<StatusCode, impl std::fmt::Display>,
) -> Result<(), Failure> {
    match status {
        Ok(StatusCode::NOT_FOUND) => Ok(()),
        Ok(status) if status.is_success() => {
            Err((StatusCode::CONFLICT, "Deck package already uploaded"))
        }
        other => {
            match other {
                Ok(status) => eprintln!("anki package: R2 HEAD returned {status}"),
                Err(error) => eprintln!("anki package: R2 HEAD failed: {error}"),
            }
            Err((StatusCode::BAD_GATEWAY, "Could not check package storage"))
        }
    }
}

pub async fn upload(
    Path(deck_id): Path<Uuid>,
    Query(params): Query<Params>,
    bytes: Bytes,
) -> Result<Json<serde_json::Value>, Failure> {
    if crate::deck_token::verify(&params.d) != Some(deck_id) {
        return Err((StatusCode::FORBIDDEN, "Invalid deck token"));
    }
    if !valid_code(&params.code) {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "Invalid language code"));
    }
    validate_package(&bytes).map_err(|reason| (StatusCode::UNPROCESSABLE_ENTITY, reason))?;
    let client = crate::service_role_client().map_err(internal)?;
    let response = client
        .from("anki_decks")
        .select("revoked_at,created_at")
        .eq("id", deck_id.to_string())
        .execute()
        .await
        .map_err(internal)?;
    if !response.status().is_success() {
        return Err(internal(response.status()));
    }
    #[derive(Deserialize)]
    struct Deck {
        revoked_at: Option<String>,
        created_at: String,
    }
    let decks: Vec<Deck> = response.json().await.map_err(internal)?;
    let deck = decks
        .first()
        .ok_or((StatusCode::NOT_FOUND, "Deck not found"))?;
    if deck.revoked_at.is_some() {
        return Err((StatusCode::FORBIDDEN, "Deck revoked"));
    }
    // The token never expires and ships inside the package, so the upload
    // window does: the browser uploads seconds after minting, and once the
    // bucket has expired the object nobody can refill its URL.
    let minted = chrono::DateTime::parse_from_rfc3339(&deck.created_at).map_err(internal)?;
    if chrono::Utc::now() - minted.with_timezone(&chrono::Utc) > chrono::Duration::hours(1) {
        return Err((StatusCode::FORBIDDEN, "Deck upload window closed"));
    }
    let writer = WRITER.as_ref().ok_or((
        StatusCode::NOT_IMPLEMENTED,
        "Package storage is not configured",
    ))?;
    let key = format!("{deck_id}.apkg");
    let http = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(internal)?;
    let head = writer.bucket.head_object(Some(&writer.credentials), &key);
    require_missing_package(
        http.head(head.sign(Duration::from_secs(300)))
            .send()
            .await
            .map(|response| response.status()),
    )?;
    let disposition = format!("attachment; filename=\"yap-{}.apkg\"", params.code);
    let headers = [
        ("content-type", "application/octet-stream"),
        ("content-disposition", disposition.as_str()),
        ("if-none-match", "*"),
        // Do not let a CDN/browser cache extend the bucket's retention period.
        ("cache-control", "no-store"),
    ];
    let mut put = writer.bucket.put_object(Some(&writer.credentials), &key);
    for (name, value) in headers {
        put.headers_mut().insert(name, value);
    }
    let mut request = http.put(put.sign(Duration::from_secs(300)));
    for (name, value) in headers {
        request = request.header(name, value);
    }
    let response = request.body(bytes).send().await.map_err(internal)?;
    // R2's conditional write is the arbiter even if two requests HEAD the key
    // before either upload completes. Never overwrite a successfully stored deck.
    if matches!(response.status().as_u16(), 409 | 412) {
        return Err((StatusCode::CONFLICT, "Deck package already uploaded"));
    }
    if !response.status().is_success() {
        return Err(internal(response.status()));
    }
    Ok(Json(
        serde_json::json!({"url": format!("{ANKI_DECKS_ORIGIN}/{key}")}),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    // The same stored-entry format emitted by fflate's level: 0 writer.
    fn zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut directory = Vec::new();
        for (name, data) in entries {
            let offset = bytes.len() as u32;
            let mut crc = !0u32;
            for byte in *data {
                crc ^= u32::from(*byte);
                for _ in 0..8 {
                    crc = (crc >> 1) ^ (0xedb88320 & 0u32.wrapping_sub(crc & 1));
                }
            }
            let crc = !crc;
            bytes.extend(b"PK\x03\x04");
            for value in [20u16, 0, 0, 0, 0] {
                bytes.extend(value.to_le_bytes());
            }
            for value in [crc, data.len() as u32, data.len() as u32] {
                bytes.extend(value.to_le_bytes());
            }
            bytes.extend((name.len() as u16).to_le_bytes());
            bytes.extend(0u16.to_le_bytes());
            bytes.extend(name.as_bytes());
            bytes.extend(*data);
            directory.extend(b"PK\x01\x02");
            for value in [20u16, 20, 0, 0, 0, 0] {
                directory.extend(value.to_le_bytes());
            }
            for value in [crc, data.len() as u32, data.len() as u32] {
                directory.extend(value.to_le_bytes());
            }
            for value in [name.len() as u16, 0, 0, 0, 0] {
                directory.extend(value.to_le_bytes());
            }
            directory.extend(0u32.to_le_bytes());
            directory.extend(offset.to_le_bytes());
            directory.extend(name.as_bytes());
        }
        let offset = bytes.len() as u32;
        bytes.extend(&directory);
        bytes.extend(b"PK\x05\x06");
        for value in [0u16, 0, entries.len() as u16, entries.len() as u16] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend((directory.len() as u32).to_le_bytes());
        bytes.extend(offset.to_le_bytes());
        bytes.extend(0u16.to_le_bytes());
        bytes
    }
    fn package(media: &[u8]) -> Vec<u8> {
        zip(&[
            ("collection.anki2", b"SQLite format 3\0"),
            ("media", media),
            ("0", b"audio"),
        ])
    }
    #[test]
    fn accepts_package() {
        assert_eq!(validate_package(&package(br#"{"0":"voice.mp3"}"#)), Ok(()));
    }
    #[test]
    fn rejects_wrong_entry_and_duplicates() {
        assert_eq!(
            validate_package(&zip(&[("../file", b"")])),
            Err("Unexpected ZIP entry")
        );
        assert_eq!(
            validate_package(&zip(&[("media", b"{}"), ("media", b"{}")])),
            Err("Duplicate ZIP entry")
        );
    }
    #[test]
    fn rejects_bad_collection_and_missing_manifest() {
        assert_eq!(
            validate_package(&zip(&[
                ("collection.anki2", b"not sqlite"),
                ("media", b"{}")
            ])),
            Err("Missing or invalid SQLite collection")
        );
        assert_eq!(
            validate_package(&zip(&[("collection.anki2", b"SQLite format 3\0")])),
            Err("Missing media manifest")
        );
    }
    #[test]
    fn validates_media() {
        assert_eq!(
            validate_package(&package(b"[]")),
            Err("Media manifest must be an object")
        );
        for media in [
            br#"{"x":"a"}"#.as_slice(),
            br#"{"0":""}"#,
            br#"{"0":"a/b"}"#,
            br#"{"0":"a\\b"}"#,
            br#"{"0":5}"#,
            br#"{"":"a"}"#,
        ] {
            assert_eq!(
                validate_package(&package(media)),
                Err("Invalid media filename")
            );
        }
    }
    #[test]
    fn rejects_oversize_truncated_and_compressed() {
        assert_eq!(
            validate_package(&vec![0; MAX_PACKAGE_BYTES + 1]),
            Err("Package exceeds 64 MiB")
        );
        let good = package(b"{}");
        for end in 0..good.len() {
            assert!(validate_package(&good[..end]).is_err());
        }
        let mut compressed = good.clone();
        let directory = u32_at(&good, good.len() - 6).unwrap();
        compressed[directory + 10] = 8;
        assert!(validate_package(&compressed).is_err());
        let mut mismatched = good;
        mismatched[30] = b'x';
        assert!(validate_package(&mismatched).is_err());
    }
    #[tokio::test]
    async fn route_rejects_forged_tokens_and_limits_only_this_body() {
        use axum::{body::Body, http::Request};
        use tower::ServiceExt;
        let uri = format!("/anki/deck/{}/package?d=forged&code=fra", Uuid::nil());
        // Above axum's ordinary 2 MiB limit, but under this route's limit:
        // authentication, not body extraction, rejects the request.
        for (size, expected) in [
            (2 * 1024 * 1024 + 1, StatusCode::FORBIDDEN),
            (MAX_PACKAGE_BYTES + 1, StatusCode::PAYLOAD_TOO_LARGE),
        ] {
            let response = crate::app()
                .oneshot(Request::put(&uri).body(Body::from(vec![0; size])).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), expected);
        }
    }

    #[test]
    fn rejects_inconsistent_sizes_and_trailing_data() {
        let good = package(b"{}");
        let directory = u32_at(&good, good.len() - 6).unwrap();
        for offset in [18, 22, directory + 20, directory + 24, directory + 42] {
            let mut bad = good.clone();
            bad[offset..offset + 4].copy_from_slice(&u32::MAX.to_le_bytes());
            assert!(validate_package(&bad).is_err());
        }
        let mut bad = good;
        bad.push(0);
        assert!(validate_package(&bad).is_err());
    }

    #[test]
    fn only_a_confirmed_storage_miss_allows_upload() {
        assert_eq!(
            require_missing_package(Ok::<_, &str>(StatusCode::NOT_FOUND)),
            Ok(())
        );
        for status in [StatusCode::OK, StatusCode::NO_CONTENT] {
            assert_eq!(
                require_missing_package(Ok::<_, &str>(status))
                    .unwrap_err()
                    .0,
                StatusCode::CONFLICT
            );
        }
        for status in [
            StatusCode::FORBIDDEN,
            StatusCode::INTERNAL_SERVER_ERROR,
            StatusCode::MOVED_PERMANENTLY,
        ] {
            assert_eq!(
                require_missing_package(Ok::<_, &str>(status))
                    .unwrap_err()
                    .0,
                StatusCode::BAD_GATEWAY
            );
        }
        assert_eq!(
            require_missing_package(Err("connection failed"))
                .unwrap_err()
                .0,
            StatusCode::BAD_GATEWAY
        );
    }

    #[test]
    fn language_codes_are_safe_in_headers() {
        for code in ["fra", "spa-es", "zho-hant"] {
            assert!(valid_code(code));
        }
        for code in ["", "French", "fra\r\n", "../fra", "abcdefghijklm"] {
            assert!(!valid_code(code));
        }
    }
}
