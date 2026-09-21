use crate::Result;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};
use tiny_http::{Header, Method, Request, Response, Server};

#[derive(Default)]
pub struct Stats {
    pub downloads: Vec<(&'static str, u64)>,
    pub offline: bool,
    pub offline_downloads: usize,
}

pub struct PackServer {
    pub url: String,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<Result<Stats>>>,
}

struct Object {
    part: &'static str,
    path: PathBuf,
    size: u64,
}

impl PackServer {
    pub fn start(packs: &Path) -> Result<Self> {
        let metadata = fs::read_to_string(packs.join("fra_for_eng/language_data.hash"))?;
        let lines: Vec<_> = metadata.trim().lines().collect();
        if lines.len() != 2 {
            return Err("Expected core and sentences metadata".into());
        }
        let mut objects = HashMap::new();
        for (part, line) in ["core", "sentences"].into_iter().zip(lines) {
            let (hash, size) = line.split_once(';').ok_or("invalid pack metadata")?;
            let hash: u64 = hash.trim().parse()?;
            let size: u64 = size.trim().parse()?;
            let path = packs.join(format!("fra_for_eng/language_data_{part}.rkyv"));
            if size == 0 || fs::metadata(&path)?.len() != size {
                return Err(format!("Invalid fixture size for {part}").into());
            }
            objects.insert(
                format!("/fra_for_eng/language_data_{part}_{hash}.rkyv"),
                Object { part, path, size },
            );
        }
        let server = Server::http("127.0.0.1:0")?;
        let url = format!("http://{}", server.server_addr());
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = stop.clone();
        let thread = thread::spawn(move || {
            let mut stats = Stats::default();
            while !stopping.load(Ordering::Relaxed) {
                if let Some(request) = server.recv_timeout(Duration::from_millis(100))? {
                    handle(request, &objects, &mut stats)?;
                }
            }
            Ok(stats)
        });
        Ok(Self {
            url,
            stop,
            thread: Some(thread),
        })
    }

    pub fn finish(mut self) -> Result<Stats> {
        self.stop.store(true, Ordering::Relaxed);
        self.thread
            .take()
            .unwrap()
            .join()
            .map_err(|_| "pack server panicked")?
    }
}

fn header(name: &str, value: impl ToString) -> Header {
    Header::from_bytes(name, value.to_string()).expect("valid HTTP header")
}

fn handle(request: Request, objects: &HashMap<String, Object>, stats: &mut Stats) -> Result<()> {
    if request.method() == &Method::Post {
        let code = if request.url() == "/__offline" {
            stats.offline = true;
            204
        } else {
            404
        };
        request.respond(Response::empty(code))?;
        return Ok(());
    }
    if request.method() != &Method::Get {
        request.respond(Response::empty(501))?;
        return Ok(());
    }
    let Some(object) = objects.get(request.url()) else {
        request.respond(Response::empty(404))?;
        return Ok(());
    };
    if stats.offline {
        stats.offline_downloads += 1;
        request.respond(Response::empty(503))?;
        return Ok(());
    }
    let range = request
        .headers()
        .iter()
        .find(|h| h.field.equiv("Range"))
        .map(|h| h.value.as_str())
        .unwrap_or("");
    let Some((start, end)) = byte_range(range, object.size) else {
        request.respond(
            Response::empty(416)
                .with_header(header("Content-Range", format!("bytes */{}", object.size))),
        )?;
        return Ok(());
    };
    let mut file = File::open(&object.path)?;
    file.seek(SeekFrom::Start(start))?;
    let length = end - start + 1;
    let response = Response::new(
        206.into(),
        vec![
            header("Content-Type", "application/octet-stream"),
            header("Accept-Ranges", "bytes"),
            header(
                "Content-Range",
                format!("bytes {start}-{end}/{}", object.size),
            ),
        ],
        file.take(length),
        Some(length.try_into()?),
        None,
    )
    .with_chunked_threshold(usize::MAX);
    request.respond(response)?;
    stats.downloads.push((object.part, start));
    Ok(())
}

fn byte_range(value: &str, size: u64) -> Option<(u64, u64)> {
    let (start, end) = value.strip_prefix("bytes=")?.split_once('-')?;
    if start.is_empty()
        || end.is_empty()
        || !start.bytes().all(|b| b.is_ascii_digit())
        || !end.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    let start = start.parse::<u64>().ok()?;
    let end = end.parse::<u64>().ok()?;
    (start < size && end >= start).then(|| (start, end.min(size - 1)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_ranges_only() {
        assert_eq!(byte_range("bytes=2-99", 10), Some((2, 9)));
        assert_eq!(byte_range("bytes=0-0", 10), Some((0, 0)));
        for range in [
            "",
            "bytes=0-",
            "bytes=-2",
            "bytes=10-12",
            "bytes=4-2",
            "bytes=0-1,3-4",
            "bytes=+1-2",
            "bytes=1-2 ",
        ] {
            assert_eq!(byte_range(range, 10), None, "{range}");
        }
    }
}
