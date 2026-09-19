//! Local development pack hosting. Production clients use the public bucket.
use axum::{
    body::Body,
    extract::{Json, Path},
    http::{Request, StatusCode, header},
    response::Response,
};
use language_utils::{
    Course, PACKS_ORIGIN,
    language_pack::{
        PackPart, course_directory_slug, course_from_directory_slug, pack_key, pack_url,
        parse_hash_metadata,
    },
};
use std::path::Path as DiskPath;
use tower_http::services::ServeFile;

pub fn log_availability() {
    let root = std::env::var_os("YAP_OUT_DIR").unwrap_or_else(|| "out".into());
    if !DiskPath::new(&root).is_dir() {
        eprintln!(
            "Local packs: {} is absent; /packs returns 404 (production uses the public CDN)",
            DiskPath::new(&root).display()
        );
    }
}

pub async fn serve(
    Path((slug, filename)): Path<(String, String)>,
    request: Request<Body>,
) -> Result<Response, StatusCode> {
    let root = std::env::var_os("YAP_OUT_DIR").unwrap_or_else(|| "out".into());
    serve_from(DiskPath::new(&root), &slug, &filename, request).await
}

async fn serve_from(
    root: &DiskPath,
    slug: &str,
    filename: &str,
    request: Request<Body>,
) -> Result<Response, StatusCode> {
    let course = course_from_directory_slug(slug).ok_or(StatusCode::NOT_FOUND)?;
    let dir = root.join(slug);
    let metadata = tokio::fs::read_to_string(dir.join("language_data.hash"))
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let metadata = parse_hash_metadata(&metadata).map_err(|_| StatusCode::NOT_FOUND)?;
    let (part, meta) = [
        (PackPart::Core, metadata.core),
        (PackPart::Sentences, metadata.sentences),
    ]
    .into_iter()
    .find(|(part, meta)| pack_key(course, *part, meta.hash) == format!("{slug}/{filename}"))
    .ok_or(StatusCode::NOT_FOUND)?;
    let path = dir.join(part.filename());
    if tokio::fs::metadata(&path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?
        .len()
        != meta.size as u64
    {
        return Err(StatusCode::NOT_FOUND);
    }
    let response = ServeFile::new(path)
        .try_call(request)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut response = response.map(Body::new);
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/octet-stream"),
    );
    // Local files can be regenerated in place; do not advertise immutable caching.
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-cache"),
    );
    Ok(response)
}

#[derive(Debug, serde::Deserialize)]
pub struct LegacyRequest {
    course: Course,
    part: PackPart,
    chunk_index: Option<usize>,
    chunk_size: Option<usize>,
}

impl LegacyRequest {
    fn chunk(&self) -> Result<Option<(usize, usize)>, StatusCode> {
        match (self.chunk_index, self.chunk_size) {
            (None, None) => Ok(None),
            (Some(index), Some(size)) if size > 0 => Ok(Some((index, size))),
            _ => Err(StatusCode::BAD_REQUEST),
        }
    }

    fn range(&self, total: usize) -> Result<Option<(usize, usize)>, StatusCode> {
        let Some((index, size)) = self.chunk()? else {
            return Ok(None);
        };
        let start = index
            .checked_mul(size)
            .filter(|start| *start < total)
            .ok_or(StatusCode::RANGE_NOT_SATISFIABLE)?;
        let len = size.min(total - start);
        Ok(Some((start, start + len - 1)))
    }
}

/// Temporary bridge for clients loaded before the R2 cutover (2026-09-19).
/// Delete this handler, request type, POST route and smoke probe once those
/// clients have refreshed. Pack bytes are streamed from R2, never embedded.
pub async fn legacy(Json(request): Json<LegacyRequest>) -> Result<Response, StatusCode> {
    use std::{sync::LazyLock, time::Duration};
    static HTTP: LazyLock<reqwest::Client> = LazyLock::new(|| {
        reqwest::Client::builder()
            .no_gzip()
            .no_brotli()
            .no_deflate()
            .no_zstd()
            .connect_timeout(Duration::from_secs(30))
            .read_timeout(Duration::from_secs(60))
            .build()
            .expect("pack proxy HTTP client")
    });
    request.chunk()?;
    if !language_utils::COURSES.contains(&request.course) {
        return Err(StatusCode::NOT_FOUND);
    }
    let pointer = format!(
        "{PACKS_ORIGIN}/{}/language_data.hash",
        course_directory_slug(request.course)
    );
    let metadata = HTTP
        .get(pointer)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    if metadata.status() != reqwest::StatusCode::OK {
        return Err(StatusCode::BAD_GATEWAY);
    }
    let metadata = metadata.text().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
    let meta = parse_hash_metadata(&metadata)
        .map_err(|_| StatusCode::BAD_GATEWAY)?
        .part(request.part);
    let range = request.range(meta.size)?;
    let mut upstream = HTTP.get(pack_url(
        PACKS_ORIGIN,
        request.course,
        request.part,
        meta.hash,
    ));
    if let Some((start, end)) = range {
        upstream = upstream.header(header::RANGE, format!("bytes={start}-{end}"));
    }
    let upstream = upstream.send().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
    let expected_status = if range.is_some() {
        StatusCode::PARTIAL_CONTENT
    } else {
        StatusCode::OK
    };
    if upstream.status() != expected_status {
        return Err(StatusCode::BAD_GATEWAY);
    }
    let expected_len = range.map_or(meta.size, |(start, end)| end - start + 1);
    if upstream.content_length() != Some(expected_len as u64) {
        return Err(StatusCode::BAD_GATEWAY);
    }
    if let Some((start, end)) = range {
        let expected = format!("bytes {start}-{end}/{}", meta.size);
        if upstream
            .headers()
            .get(header::CONTENT_RANGE)
            .and_then(|v| v.to_str().ok())
            != Some(expected.as_str())
        {
            return Err(StatusCode::BAD_GATEWAY);
        }
    }

    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream");
    if let Some(length) = upstream.headers().get(header::CONTENT_LENGTH) {
        response = response.header(header::CONTENT_LENGTH, length);
    }
    response
        .body(Body::from_stream(upstream.bytes_stream()))
        .map_err(|_| StatusCode::BAD_GATEWAY)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn old_request(index: Option<usize>, size: Option<usize>) -> LegacyRequest {
        LegacyRequest {
            course: course_from_directory_slug("fra_for_eng").unwrap(),
            part: PackPart::Core,
            chunk_index: index,
            chunk_size: size,
        }
    }

    #[test]
    fn legacy_range_validation() {
        assert_eq!(old_request(None, None).range(10), Ok(None));
        assert_eq!(old_request(Some(0), Some(4)).range(10), Ok(Some((0, 3))));
        assert_eq!(old_request(Some(2), Some(4)).range(10), Ok(Some((8, 9))));
        assert_eq!(
            old_request(Some(0), Some(usize::MAX)).range(10),
            Ok(Some((0, 9)))
        );
        for request in [
            old_request(Some(0), None),
            old_request(None, Some(4)),
            old_request(Some(0), Some(0)),
        ] {
            assert_eq!(request.range(10), Err(StatusCode::BAD_REQUEST));
        }
        for request in [
            old_request(Some(3), Some(4)),
            old_request(Some(usize::MAX), Some(2)),
        ] {
            assert_eq!(request.range(10), Err(StatusCode::RANGE_NOT_SATISFIABLE));
        }
    }

    #[tokio::test]
    async fn unsupported_legacy_course_is_rejected_without_network() {
        let mut request = old_request(Some(0), Some(1024));
        request.course = course_from_directory_slug("fra_for_jpn").unwrap();
        assert_eq!(
            legacy(Json(request)).await.unwrap_err(),
            StatusCode::NOT_FOUND
        );
    }

    #[test]
    fn legacy_json_matches_old_clients() {
        let request: LegacyRequest = serde_json::from_str(r#"{"course":{"nativeLanguage":"English","targetLanguage":"French"},"part":"core","chunk_index":0,"chunk_size":1024}"#).unwrap();
        assert_eq!(request.part, PackPart::Core);
        assert_eq!(request.range(2000), Ok(Some((0, 1023))));
        let request: LegacyRequest = serde_json::from_str(r#"{"course":{"nativeLanguage":"English","targetLanguage":"French"},"part":"sentences"}"#).unwrap();
        assert_eq!(request.part, PackPart::Sentences);
        assert_eq!(request.range(2000), Ok(None));
    }

    #[tokio::test]
    async fn disk_ranges_and_invalid_paths() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("fra_for_eng");
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("language_data.hash"), "123;8\n456;8\n").unwrap();
        for part in PackPart::ALL {
            std::fs::write(dir.join(part.filename()), b"abcdefgh").unwrap();
        }
        for (part, hash) in [(PackPart::Core, 123), (PackPart::Sentences, 456)] {
            let name = format!("language_data_{}_{hash}.rkyv", part.slug());
            let request = Request::builder()
                .header(header::RANGE, "bytes=1-3")
                .body(Body::empty())
                .unwrap();
            let response = serve_from(root.path(), "fra_for_eng", &name, request)
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
            assert_eq!(response.headers()[header::CONTENT_RANGE], "bytes 1-3/8");
            assert_eq!(
                axum::body::to_bytes(response.into_body(), 10)
                    .await
                    .unwrap(),
                "bcd"
            );
            let request = Request::builder()
                .header(header::RANGE, "bytes=99-100")
                .body(Body::empty())
                .unwrap();
            assert_eq!(
                serve_from(root.path(), "fra_for_eng", &name, request)
                    .await
                    .unwrap()
                    .status(),
                StatusCode::RANGE_NOT_SATISFIABLE
            );
        }
        for (slug, name) in [
            ("../fra_for_eng", "language_data_core_123.rkyv"),
            ("fra_for_eng", "language_data_core_999.rkyv"),
            ("fra_for_eng", "../language_data.hash"),
        ] {
            assert_eq!(
                serve_from(root.path(), slug, name, Request::new(Body::empty()))
                    .await
                    .unwrap_err(),
                StatusCode::NOT_FOUND
            );
        }
    }
}
