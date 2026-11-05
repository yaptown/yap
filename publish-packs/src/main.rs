//! Publish immutable packs by default; advance legacy pointers only with --pointers.
//! CI publishes immutable objects in parallel, then moves pointers inside the gated deploy job.
use clap::Parser;
use language_utils::language_pack::{
    LANGUAGE_DATA_CHUNK_SIZE, PACKS_ORIGIN, PackPart, course_from_directory_slug,
    language_data_chunk_count, language_data_chunk_len, pack_chunk_key, pack_chunk_url, pack_key,
    pack_url, parse_hash_metadata,
};
use rusty_s3::{Bucket, Credentials, S3Action, UrlStyle};
use std::{path::PathBuf, time::Duration};
use xxhash_rust::const_xxh3::xxh3_64;

const BUCKET: &str = "yap-packs";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser)]
#[command(about = "Validate and publish immutable language packs")]
struct Args {
    /// Read-only verification: HEAD immutable packs, or fresh GET pointers with --pointers.
    /// Neither check mode needs credentials.
    #[arg(long)]
    check: bool,
    /// Compress all chunks and report totals without network access or credentials.
    #[arg(long, conflicts_with_all = ["check", "pointers"])]
    dry_run: bool,
    /// Publish only metadata pointers, after HEAD-verifying every referenced pack.
    /// Requires language_data.hash files only, not local archives or LFS.
    #[arg(long)]
    pointers: bool,
    #[arg(long = "out-dir", default_value = "out")]
    out: PathBuf,
    /// Course directory slug, repeatable. Defaults to all discovered courses.
    #[arg(long = "course")]
    courses: Vec<String>,
}

struct Object {
    path: PathBuf,
    key: String,
    url: String,
    size: usize,
    /// Raw offset; None denotes the legacy whole-file object.
    offset: Option<usize>,
}

impl Object {
    fn compress(&self) -> std::io::Result<Option<Vec<u8>>> {
        let Some(offset) = self.offset else {
            return Ok(None);
        };
        let file = std::fs::File::open(&self.path)?;
        // Inputs remain stable for the duration of publication, as during validation.
        let bytes = unsafe { memmap2::Mmap::map(&file)? };
        let mut encoded = Vec::new();
        std::io::copy(
            &mut brotli::CompressorReader::new(&bytes[offset..offset + self.size], 4096, 11, 24),
            &mut encoded,
        )?;
        Ok(Some(encoded))
    }

    /// Ask for a chunk's stored brotli bytes. Cloudflare would otherwise decode
    /// it, and it compresses text pointers on the fly when a client accepts br.
    fn wire(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if self.offset.is_some() {
            request.header(reqwest::header::ACCEPT_ENCODING, "br")
        } else {
            request
        }
    }

    fn matches(&self, response: &reqwest::Response) -> bool {
        if response.status() != reqwest::StatusCode::OK {
            return false;
        }
        // A chunk's key fixes its part hash and index, so its raw length is
        // determined; R2's public domain doesn't return custom metadata anyway.
        if self.offset.is_some() {
            response
                .headers()
                .get("content-encoding")
                .is_some_and(|v| v == "br")
                && wire_size(response).is_some_and(|size| size > 0)
        } else {
            wire_size(response) == Some(self.size as u64)
        }
    }
}

// HEAD has an empty body, so Response::content_length() is not the object size.
fn wire_size(response: &reqwest::Response) -> Option<u64> {
    response
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)?
        .to_str()
        .ok()?
        .parse()
        .ok()
}

#[derive(Clone, Copy)]
enum ObjectKind {
    Pointer,
    Raw,
    Brotli,
}

struct PreparedCourse {
    parts: Vec<Object>,
    pointer: Pointer,
}

struct Pointer {
    key: String,
    url: String,
    bytes: Vec<u8>,
}

fn local_courses(args: &Args) -> Result<Vec<PreparedCourse>> {
    let mut courses = args.courses.clone();
    if courses.is_empty() {
        for entry in std::fs::read_dir(&args.out)? {
            let entry = entry?;
            if entry.file_name().to_string_lossy().contains("_for_")
                && entry.path().join("language_data.hash").is_file()
            {
                courses.push(
                    entry
                        .file_name()
                        .into_string()
                        .map_err(|_| "non-UTF8 course")?,
                );
            }
        }
    }
    courses.sort();
    courses.dedup();
    if courses.is_empty() {
        return Err("no courses found".into());
    }
    let mut prepared = Vec::new();
    let mut errors = Vec::new();
    for slug in courses {
        let result = (|| -> Result<()> {
            let course = course_from_directory_slug(&slug).ok_or("invalid course slug")?;
            let directory = args.out.join(&slug);
            // Capture the exact bytes we parse now: never reread a possibly
            // regenerated pointer after verifying/uploading its pack objects.
            let pointer_bytes = std::fs::read(directory.join("language_data.hash"))?;
            let metadata = parse_hash_metadata(std::str::from_utf8(&pointer_bytes)?)?;
            let mut parts = Vec::new();
            for part in PackPart::ALL {
                let meta = metadata.part(part);
                let path = directory.join(part.filename());
                let validate = (|| -> Result<()> {
                    if args.pointers {
                        return Ok(());
                    }
                    let file = std::fs::File::open(&path)?;
                    if file.metadata()?.len() != meta.size as u64 {
                        return Err("size mismatch (is this an LFS pointer?)".into());
                    }
                    // The command requires stable input files for its duration; mmap
                    // avoids allocating entire sentence archives on the heap.
                    let bytes = unsafe { memmap2::Mmap::map(&file)? };
                    if xxh3_64(&bytes) != meta.hash {
                        return Err("hash mismatch".into());
                    }
                    Ok(())
                })();
                match validate {
                    Ok(()) => {
                        parts.push(Object {
                            path: path.clone(),
                            key: pack_key(course, part, meta.hash),
                            url: pack_url(PACKS_ORIGIN, course, part, meta.hash),
                            size: meta.size,
                            offset: None,
                        });
                        for index in 0..language_data_chunk_count(meta.size) {
                            parts.push(Object {
                                path: path.clone(),
                                key: pack_chunk_key(course, part, meta, index),
                                url: pack_chunk_url(PACKS_ORIGIN, course, part, meta, index),
                                size: language_data_chunk_len(meta.size, index),
                                offset: Some(index * LANGUAGE_DATA_CHUNK_SIZE),
                            });
                        }
                    }
                    Err(error) => errors.push(format!("{}: {error}", path.display())),
                }
            }
            let key = format!("{slug}/language_data.hash");
            prepared.push(PreparedCourse {
                parts,
                pointer: Pointer {
                    url: format!("{PACKS_ORIGIN}/{key}"),
                    key,
                    bytes: pointer_bytes,
                },
            });
            Ok(())
        })();
        if let Err(error) = result {
            errors.push(format!("{slug}: {error}"));
        }
    }
    if !errors.is_empty() {
        return Err(errors.join("\n").into());
    }
    Ok(prepared)
}

fn writer() -> Result<(Bucket, Credentials)> {
    let required = |key| {
        std::env::var(key)
            .ok()
            .filter(|v| !v.is_empty())
            .ok_or_else(|| format!("missing {key}"))
    };
    let endpoint = format!(
        "https://{}.r2.cloudflarestorage.com",
        required("CLOUDFLARE_ACCOUNT_ID")?
    )
    .parse()?;
    Ok((
        Bucket::new(endpoint, UrlStyle::Path, BUCKET, "auto")?,
        Credentials::new(
            required("R2_ACCESS_KEY_ID")?,
            required("R2_SECRET_ACCESS_KEY")?,
        ),
    ))
}

fn fresh_url(url: &str) -> Result<String> {
    let fresh = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    Ok(format!("{url}?fresh={fresh}"))
}

async fn verify_pointer(http: &reqwest::Client, pointer: &Pointer) -> Result<()> {
    let response = http.get(fresh_url(&pointer.url)?).send().await?;
    if response.status() != reqwest::StatusCode::OK {
        return Err(format!(
            "{}: pointer GET returned {}",
            pointer.key,
            response.status()
        )
        .into());
    }
    if response.bytes().await?.as_ref() != pointer.bytes {
        return Err(format!("{}: pointer bytes mismatch", pointer.key).into());
    }
    Ok(())
}

async fn put(
    http: &reqwest::Client,
    bucket: &Bucket,
    credentials: &Credentials,
    key: &str,
    body: reqwest::Body,
    size: usize,
    kind: ObjectKind,
) -> Result<()> {
    let url = bucket
        .put_object(Some(credentials), key)
        .sign(Duration::from_secs(3600));
    let pointer = matches!(kind, ObjectKind::Pointer);
    let mut request = http.put(url);
    if let ObjectKind::Brotli = kind {
        request = request.header("Content-Encoding", "br");
    }
    request
        .header(
            reqwest::header::CONTENT_TYPE,
            if pointer {
                "text/plain"
            } else {
                "application/octet-stream"
            },
        )
        .header(reqwest::header::CONTENT_LENGTH, size)
        .header(
            reqwest::header::CACHE_CONTROL,
            if pointer {
                "public, max-age=60"
            } else {
                "public, max-age=31536000, immutable"
            },
        )
        .body(body)
        .send()
        .await
        // A presigned URL is a temporary write capability, not log data.
        .map_err(reqwest::Error::without_url)?
        .error_for_status()
        .map_err(reqwest::Error::without_url)?;
    Ok(())
}

async fn run() -> Result<()> {
    let args = Args::parse();
    let courses = local_courses(&args)?;
    let http = reqwest::Client::builder()
        // Chunk checks inspect the stored brotli bytes, so never decode them.
        .no_brotli()
        .timeout(Duration::from_secs(900))
        .build()?;
    // Pointer checks verify only the mutable metadata, using the captured local
    // bytes. Availability of immutable objects is a separate --check operation.
    if args.pointers && args.check {
        let mut errors = Vec::new();
        for course in &courses {
            match verify_pointer(&http, &course.pointer).await {
                Ok(()) => println!("verified {}", course.pointer.key),
                Err(error) => errors.push(error.to_string()),
            }
        }
        return if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("\n").into())
        };
    }
    let mut missing = Vec::new();
    let mut errors = Vec::new();
    let mut compressed_total = 0u64;
    let raw_total: usize = courses
        .iter()
        .flat_map(|c| &c.parts)
        .filter(|o| o.offset.is_some())
        .map(|o| o.size)
        .sum();
    for object in courses.iter().flat_map(|course| &course.parts) {
        if args.dry_run {
            if object.offset.is_some() {
                missing.push(object);
            }
            continue;
        }
        match object.wire(http.head(&object.url)).send().await {
            Ok(response) if object.matches(&response) => {
                if object.offset.is_some() {
                    compressed_total += wire_size(&response).unwrap();
                }
                println!(
                    "{} {}",
                    if args.check { "verified" } else { "skipped" },
                    object.key
                );
            }
            Ok(response)
                if response.status() == reqwest::StatusCode::NOT_FOUND
                    || response.status().is_success() =>
            {
                if args.check || args.pointers {
                    println!(
                        "missing or mismatched {} (HTTP {})",
                        object.key,
                        response.status()
                    );
                }
                missing.push(object);
            }
            Ok(response) => errors.push(format!(
                "{}: HEAD returned {}",
                object.key,
                response.status()
            )),
            Err(error) => errors.push(format!("{}: HEAD failed: {error}", object.key)),
        }
    }
    if (args.check || args.pointers) && !missing.is_empty() {
        errors.push(format!(
            "{} missing or mismatched pack objects",
            missing.len()
        ));
    }
    if !errors.is_empty() {
        return Err(errors.join("\n").into());
    }
    if args.check {
        return Ok(());
    }

    if args.pointers {
        // All referenced objects passed HEAD above. No pointer is written
        // before that global availability gate, and no archive is uploaded here.
        let (bucket, credentials) = writer()?;
        for course in courses {
            let pointer = course.pointer;
            put(
                &http,
                &bucket,
                &credentials,
                &pointer.key,
                reqwest::Body::from(pointer.bytes.clone()),
                pointer.bytes.len(),
                ObjectKind::Pointer,
            )
            .await?;
            verify_pointer(&http, &pointer).await?;
            println!("uploaded and verified {}", pointer.key);
        }
        return Ok(());
    }
    let writer = if args.dry_run || missing.is_empty() {
        None
    } else {
        Some(writer()?)
    };
    let mut compression_time = Duration::ZERO;
    // Quality 11 needs substantial scratch space: bound concurrent encoders.
    let parallelism = std::thread::available_parallelism()?.get().min(8);
    for batch in missing.chunks(parallelism) {
        let started = std::time::Instant::now();
        let encoded = std::thread::scope(|scope| {
            let tasks: Vec<_> = batch
                .iter()
                .map(|object| scope.spawn(|| object.compress()))
                .collect();
            tasks
                .into_iter()
                .map(|task| task.join().unwrap())
                .collect::<std::io::Result<Vec<_>>>()
        })?;
        compression_time += started.elapsed();
        for (object, encoded) in batch.iter().zip(encoded) {
            let (body, size, kind, chunk_hash) = if let Some(encoded) = encoded {
                let size = encoded.len();
                let hash = xxh3_64(&encoded);
                compressed_total += size as u64;
                println!(
                    "compressed {}: {} -> {} bytes",
                    object.key, object.size, size
                );
                (
                    reqwest::Body::from(encoded),
                    size,
                    ObjectKind::Brotli,
                    Some(hash),
                )
            } else {
                (
                    reqwest::Body::from(tokio::fs::File::open(&object.path).await?),
                    object.size,
                    ObjectKind::Raw,
                    None,
                )
            };
            let Some((bucket, credentials)) = &writer else {
                continue;
            };
            put(&http, bucket, credentials, &object.key, body, size, kind).await?;
            let mut request = object.wire(http.get(fresh_url(&object.url)?));
            if object.offset.is_none() {
                request = request.header(
                    reqwest::header::RANGE,
                    format!("bytes=0-{}", object.size.min(1024) - 1),
                );
            }
            let response = request.send().await?;
            if let Some(hash) = chunk_hash {
                if !object.matches(&response) {
                    return Err(format!("{}: invalid public chunk headers", object.key).into());
                }
                let bytes = response.bytes().await?;
                if xxh3_64(&bytes) != hash {
                    return Err(format!("{}: public chunk differs from upload", object.key).into());
                }
            } else if response.status() != reqwest::StatusCode::PARTIAL_CONTENT
                || response.bytes().await?.len() != object.size.min(1024)
            {
                return Err(format!("{}: invalid public range response", object.key).into());
            }
            println!("uploaded and verified {}", object.key);
        }
    }
    println!(
        "chunk totals: raw={raw_total} compressed={compressed_total} compression_wall_seconds={:.3}",
        compression_time.as_secs_f64()
    );
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(root: &std::path::Path, slug: &str) {
        let dir = root.join(slug);
        std::fs::create_dir(&dir).unwrap();
        for part in PackPart::ALL {
            std::fs::write(dir.join(part.filename()), b"test pack").unwrap();
        }
        let hash = xxh3_64(b"test pack");
        std::fs::write(
            dir.join("language_data.hash"),
            format!("{hash};9\n{hash};9\n"),
        )
        .unwrap();
    }
    #[tokio::test]
    async fn compressed_chunks_round_trip_and_head_uses_wire_headers() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let root = tempfile::tempdir().unwrap();
        fixture(root.path(), "fra_for_eng");
        let args =
            Args::try_parse_from(["publish-packs", "--out-dir", root.path().to_str().unwrap()])
                .unwrap();
        let courses = local_courses(&args).unwrap();
        let chunk = &courses[0].parts[1];
        let compressed = chunk.compress().unwrap().unwrap();
        let mut decoded = Vec::new();
        std::io::copy(
            &mut brotli::Decompressor::new(compressed.as_slice(), 4096),
            &mut decoded,
        )
        .unwrap();
        assert_eq!(decoded, b"test pack");
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            while !request.ends_with(b"\r\n\r\n") {
                request.push(socket.read_u8().await.unwrap());
            }
            assert!(request.starts_with(b"HEAD "));
            socket.write_all(format!("HTTP/1.1 200 OK\r\nContent-Encoding: br\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", compressed.len()).as_bytes()).await.unwrap();
        });
        let response = reqwest::Client::builder()
            .no_brotli()
            .build()
            .unwrap()
            .head(format!("http://{address}"))
            .send()
            .await
            .unwrap();
        assert_eq!(response.content_length(), Some(0));
        assert!(chunk.matches(&response));
        server.await.unwrap();
    }

    #[test]
    fn discovers_and_selects_courses() {
        let root = tempfile::tempdir().unwrap();
        fixture(root.path(), "fra_for_eng");
        fixture(root.path(), "spa_for_eng");
        fixture(root.path(), "unrelated");
        let mut args = Args {
            check: true,
            pointers: false,
            dry_run: false,
            out: root.path().to_owned(),
            courses: vec![],
        };
        assert_eq!(local_courses(&args).unwrap().len(), 2);
        args.courses = vec!["fra_for_eng".into(), "fra_for_eng".into()];
        assert_eq!(local_courses(&args).unwrap().len(), 1);
        args.courses = vec!["deu_for_eng".into()];
        assert!(local_courses(&args).is_err());
    }
    #[test]
    fn pointers_capture_exact_verified_metadata() {
        let root = tempfile::tempdir().unwrap();
        fixture(root.path(), "fra_for_eng");
        let path = root.path().join("fra_for_eng/language_data.hash");
        let original = std::fs::read(&path).unwrap();
        let args = Args {
            check: true,
            pointers: false,
            dry_run: false,
            out: root.path().to_owned(),
            courses: vec![],
        };
        let courses = local_courses(&args).unwrap();
        std::fs::write(path, b"changed after validation").unwrap();
        assert_eq!(courses[0].pointer.bytes, original);
        assert_eq!(courses[0].pointer.key, "fra_for_eng/language_data.hash");
        assert_eq!(courses[0].parts.len(), 4);
        let metadata =
            parse_hash_metadata(std::str::from_utf8(&courses[0].pointer.bytes).unwrap()).unwrap();
        assert!(
            courses[0].parts[0]
                .key
                .ends_with(&format!("_{}.rkyv", metadata.core.hash))
        );
    }

    #[test]
    fn incomplete_course_prevents_immutable_publication() {
        let root = tempfile::tempdir().unwrap();
        fixture(root.path(), "fra_for_eng");
        fixture(root.path(), "spa_for_eng");
        // One valid course must not hide another course's missing half.
        std::fs::remove_file(root.path().join("spa_for_eng/language_data_sentences.rkyv")).unwrap();
        let args = Args {
            check: false,
            pointers: false,
            dry_run: false,
            out: root.path().to_owned(),
            courses: vec![],
        };
        assert!(local_courses(&args).is_err());
    }

    #[test]
    fn pointer_mode_needs_only_valid_metadata() {
        let root = tempfile::tempdir().unwrap();
        let directory = root.path().join("fra_for_eng");
        std::fs::create_dir(&directory).unwrap();
        let metadata = b"123;1024\n456;2048\n";
        std::fs::write(directory.join("language_data.hash"), metadata).unwrap();
        let mut args = Args {
            check: false,
            pointers: true,
            dry_run: false,
            out: root.path().to_owned(),
            courses: vec![],
        };
        let courses = local_courses(&args).unwrap();
        assert_eq!(courses[0].pointer.bytes, metadata);
        assert_eq!(
            courses[0].parts[0].key,
            "fra_for_eng/language_data_core_123.rkyv"
        );
        assert_eq!(courses[0].parts[2].size, 2048);
        assert!(courses[0].parts.iter().all(|part| !part.path.exists()));
        args.pointers = false;
        assert!(local_courses(&args).is_err());
        args.pointers = true;
        std::fs::write(directory.join("language_data.hash"), b"invalid").unwrap();
        assert!(local_courses(&args).is_err());
    }

    #[test]
    fn pointer_publication_requires_explicit_flag() {
        assert!(!Args::try_parse_from(["publish-packs"]).unwrap().pointers);
        assert!(
            !Args::try_parse_from(["publish-packs", "--check"])
                .unwrap()
                .pointers
        );
        let args = Args::try_parse_from(["publish-packs", "--pointers", "--check"]).unwrap();
        assert!(args.pointers && args.check);
    }

    #[test]
    fn invalid_local_files_prevent_publishing() {
        let root = tempfile::tempdir().unwrap();
        let args = Args {
            check: true,
            pointers: false,
            dry_run: false,
            out: root.path().to_owned(),
            courses: vec![],
        };
        assert!(local_courses(&args).is_err());
        fixture(root.path(), "fra_for_eng");
        let path = root.path().join("fra_for_eng/language_data_core.rkyv");
        std::fs::write(&path, b"bad").unwrap();
        assert!(
            local_courses(&args)
                .err()
                .unwrap()
                .to_string()
                .contains("size mismatch")
        );
        std::fs::write(&path, b"wrongpack").unwrap();
        assert!(
            local_courses(&args)
                .err()
                .unwrap()
                .to_string()
                .contains("hash mismatch")
        );
    }
}
