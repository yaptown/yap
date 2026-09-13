//! Movie-clip manifests, fetching, and caching. Sentence cards show the film
//! clip the sentence was cut from (published to R2 by `subtitle-corpus
//! publish`) as their primary media, with TTS as the fallback, and challenge
//! selection prioritizes sentences that have a clip. All clip traffic goes
//! through the AI backend (`/clip/{lang}/sentences` + `/clip/{lang}/{id}/lo.mp4`);
//! the app never talks to the bucket directly.
//!
//! The source of truth on which sentences have clips is the per-language
//! *manifest* — the backend's list of `{clip_id, sentence, duration,
//! critical}` rows. It is cached in OPFS and mirrored into a synchronous
//! thread-local (the same shape as `audio.rs`'s `CACHED_CLIPS` mirror), so
//! challenge selection — a synchronous wasm call — can ask "does this
//! sentence have a clip?" without I/O. On boot the mirror seeds from OPFS
//! without blocking anything, and the frontend kicks off a background
//! refresh; until either lands, lookups just say "no clip" and the app
//! behaves as before.

use crate::persistent;
use crate::utils::hit_ai_server;
use bridgerton::Error;
use futures::FutureExt;
use futures::future::{LocalBoxFuture, Shared};
use language_utils::Language;
use opfs::{DirectoryHandle as _, FileHandle as _, WritableFileStream as _};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::rc::Rc;
use unicode_normalization::UnicodeNormalization;

/// A cached positive entry outside the prefetch keep set survives this long
/// after its last use — same reasoning as the audio cache's backstop.
const CLIP_MAX_UNUSED_AGE_SECS: i64 = 7 * 24 * 60 * 60;

/// Skip rewriting the last-used index when the entry was touched this
/// recently, so replaying the same clip doesn't rewrite the file every play.
const CLIP_TOUCH_GRANULARITY_SECS: i64 = 60 * 60;

/// Sidecar in the clips directory mapping clip id → unix seconds of last
/// use. OPFS exposes no modification times, so cleanup's age check needs its
/// own bookkeeping. (Same shape as the audio cache's index.)
const CLIP_INDEX_FILENAME: &str = "last_used.json";

/// One manifest row: a course sentence with a published clip, as served by
/// the backend (which NFC-normalizes the text).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClipRow {
    #[serde(rename = "id")]
    pub clip_id: String,
    pub sentence: String,
    pub duration_ms: u64,
    pub critical: ClipCritical,
    /// These are `None` until an older row's next publish.
    #[serde(default)]
    pub clear_before_ms: Option<i64>,
    #[serde(default)]
    pub clear_after_ms: Option<i64>,
    #[serde(default)]
    pub pad_before_ms: Option<i64>,
    #[serde(default)]
    pub pad_after_ms: Option<i64>,
    #[serde(default)]
    pub aspect_ratio: Option<f64>,
    /// Exact size of lo.mp4, used to revalidate the cached copy: the
    /// pipeline can republish corrected media under the same clip id.
    #[serde(default)]
    pub lo_bytes: u64,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct ClipCritical {
    pub start_ms: u64,
    pub end_ms: u64,
}

/// One subtitle cue overlaid on a clip, clip-relative milliseconds. `role`
/// is `"sentence"` for the target sentence's own cue and
/// `"context-before"` / `"context-after"` for the padded-in neighbor lines
/// (display only — they never passed verification). Crosses the wasm
/// boundary as a plain object so the frontend shares this type. Timestamps
/// are signed: the sidecar includes every cue *overlapping* the cut, so a
/// cue that starts before the video begins has a negative `at_ms`.
#[bridgerton::bridge(transparent)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ClipSubtitleCue {
    pub text: String,
    pub at_ms: i64,
    pub until_ms: i64,
    pub role: String,
}

/// The clip and its playback-relevant metadata.
#[derive(Clone)]
pub struct FetchedClip {
    pub bytes: Vec<u8>,
    /// The clip's id, `imdb_id - sentence hash - occurrence` — the first
    /// segment identifies the film the clip is from.
    pub clip_id: String,
    pub duration_ms: u64,
    pub critical: ClipCritical,
    /// Time-synced captions for the video. Empty when the subtitle fetch
    /// failed — captions are an enhancement, never a reason to hold a clip.
    pub subtitles: Vec<ClipSubtitleCue>,
}

type SharedClipFetch = Shared<LocalBoxFuture<'static, Result<Vec<u8>, String>>>;

thread_local! {
    /// Synchronous mirror of each language's clip manifest, keyed by exact
    /// sentence text (NFC). `None` for a language means no manifest has
    /// loaded yet — treated as "no clips", never as an error.
    static CLIP_MANIFESTS: RefCell<BTreeMap<Language, Rc<HashMap<String, ClipRow>>>> =
        const { RefCell::new(BTreeMap::new()) };

    /// Bumped whenever a manifest loads or refreshes. The frontend polls this
    /// to re-run challenge selection once clip knowledge arrives.
    static CLIP_MANIFEST_VERSION: Cell<u32> = const { Cell::new(0) };

    /// One in-flight download per clip cache key — the on-screen card and the
    /// background prefetcher routinely ask for the same clip at the same
    /// moment, and clips are megabytes, not kilobytes.
    static IN_FLIGHT_CLIP_FETCHES: RefCell<HashMap<String, SharedClipFetch>> =
        RefCell::new(HashMap::new());
}

pub(crate) fn publish_manifest(language: Language, rows: Vec<ClipRow>) {
    let by_sentence: HashMap<String, ClipRow> = rows
        .into_iter()
        .map(|row| (row.sentence.clone(), row))
        .collect();
    CLIP_MANIFESTS.with(|m| m.borrow_mut().insert(language, Rc::new(by_sentence)));
    CLIP_MANIFEST_VERSION.with(|v| v.set(v.get().wrapping_add(1)));
}

pub(crate) fn clip_manifest_version() -> u32 {
    CLIP_MANIFEST_VERSION.with(|v| v.get())
}

/// The manifest row for a sentence, if its language's manifest is loaded and
/// lists it. NFC-normalizes the lookup so callers can pass text from any
/// source.
pub(crate) fn clip_for_sentence(language: Language, text: &str) -> Option<ClipRow> {
    let text: String = text.nfc().collect();
    CLIP_MANIFESTS.with(|m| m.borrow().get(&language)?.get(&text).cloned())
}

/// Whether a sentence has a published clip, per the loaded manifest. `false`
/// both for "no clip" and "manifest not loaded yet" — selection should
/// simply not prioritize when it doesn't know. Unlike `clip_for_sentence`
/// this skips NFC normalization and the row clone: it sits in challenge
/// selection's hot loop, and pack text is already NFC (both it and the
/// manifest come from the same pipeline).
pub(crate) fn sentence_has_clip(language: Language, text: &str) -> bool {
    CLIP_MANIFESTS.with(|m| {
        m.borrow()
            .get(&language)
            .is_some_and(|rows| rows.contains_key(text))
    })
}

fn manifest_filename(language: Language) -> String {
    format!("manifest_{}.json", language.code())
}

/// Cache key for one clip's local files and in-flight downloads. Clip ids
/// are only unique *within* a language: the exporter derives them from film,
/// sentence text, and occurrence, so the same short line in two language
/// versions of a film shares an id. The language prefix keeps one course's
/// cached video from ever being served to another.
pub(crate) fn clip_cache_key(language: Language, clip_id: &str) -> String {
    format!("{}_{}", language.code(), clip_id)
}

#[derive(Clone)]
pub struct ClipCache {
    clips_dir: opfs::persistent::DirectoryHandle,
}

impl ClipCache {
    pub async fn new() -> Result<Self, Error> {
        let root = persistent::app_specific_dir()
            .await
            .map_err(|e| Error::new(format!("Failed to get app directory: {e:?}")))?;
        let clips_dir = root
            .get_directory_handle_with_options(
                "clips",
                &opfs::GetDirectoryHandleOptions { create: true },
            )
            .await
            .map_err(|e| Error::new(format!("Failed to get clips directory: {e:?}")))?;
        Ok(Self { clips_dir })
    }

    async fn read_file(&self, filename: &str) -> Option<Vec<u8>> {
        let file = self
            .clips_dir
            .get_file_handle_with_options(filename, &opfs::GetFileHandleOptions { create: false })
            .await
            .ok()?;
        file.read().await.ok()
    }

    async fn write_file(&self, filename: &str, bytes: &[u8]) {
        if let Ok(mut file) = self
            .clips_dir
            .get_file_handle_with_options(filename, &opfs::GetFileHandleOptions { create: true })
            .await
            && let Ok(mut writable) = file
                .create_writable_with_options(&opfs::CreateWritableOptions {
                    keep_existing_data: false,
                })
                .await
        {
            let _ = writable.write_at_cursor_pos(bytes).await;
            let _ = writable.close().await;
        }
    }

    /// Load every manifest already in OPFS into the synchronous mirror.
    /// Called once at boot so the very first challenge selection can
    /// prioritize clip sentences from the previous session's knowledge,
    /// without waiting on the network.
    // Only called from the wasm-gated boot path; native (yap-mcp) never
    // seeds the mirror.
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub async fn seed_manifests(&self) {
        use futures::StreamExt;
        let Ok(mut entries) = self.clips_dir.entries().await else {
            return;
        };
        let mut manifest_files = Vec::new();
        while let Some(Ok((filename, _))) = entries.next().await {
            if let Some(code) = filename
                .strip_prefix("manifest_")
                .and_then(|rest| rest.strip_suffix(".json"))
                && let Some(language) = Language::from_code(code)
            {
                manifest_files.push((language, filename));
            }
        }
        for (language, filename) in manifest_files {
            if let Some(bytes) = self.read_file(&filename).await
                && let Ok(rows) = serde_json::from_slice::<Vec<ClipRow>>(&bytes)
            {
                publish_manifest(language, rows);
            }
        }
    }

    /// Fetch the latest manifest for a language from the backend, persist it,
    /// and publish it to the mirror. Run in the background on load — never on
    /// the path to showing a challenge.
    pub async fn refresh_manifest(
        &self,
        language: Language,
        access_token: Option<&String>,
    ) -> Result<(), Error> {
        let path = format!("/clip/{}/sentences", language.code());
        let response = hit_ai_server(fetch_happen::Method::GET, &path, None::<&()>, access_token)
            .await
            .map_err(|e| Error::new(format!("Clip manifest request error: {e:?}")))?;
        if !response.ok() {
            return Err(Error::new(format!(
                "Clip manifest HTTP error: {}",
                response.status()
            )));
        }
        let rows: Vec<ClipRow> = response
            .json()
            .await
            .map_err(|e| Error::new(format!("Clip manifest parse error: {e:?}")))?;

        if let Ok(bytes) = serde_json::to_vec(&rows) {
            self.write_file(&manifest_filename(language), &bytes).await;
        }
        publish_manifest(language, rows);
        Ok(())
    }

    /// Read the last-used index, treating a missing or corrupt file as empty.
    async fn read_index(&self) -> HashMap<String, i64> {
        match self.read_file(CLIP_INDEX_FILENAME).await {
            Some(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
            None => HashMap::new(),
        }
    }

    async fn write_index(&self, index: &HashMap<String, i64>) {
        if let Ok(bytes) = serde_json::to_vec(index) {
            self.write_file(CLIP_INDEX_FILENAME, &bytes).await;
        }
    }

    /// Record that a clip was just used, so age-based cleanup spares it.
    async fn touch_index(&self, cache_key: &str) {
        let now = chrono::Utc::now().timestamp();
        let mut index = self.read_index().await;
        if index
            .get(cache_key)
            .is_some_and(|&t| now - t < CLIP_TOUCH_GRANULARITY_SECS)
        {
            return;
        }
        index.insert(cache_key.to_string(), now);
        self.write_index(&index).await;
    }

    /// Drop the cached video (and its subtitles) for a sentence's clip, so
    /// the next `get` redownloads. Used when the video element fails to play
    /// cached bytes.
    pub async fn invalidate(&self, language: Language, text: &str) {
        if let Some(row) = clip_for_sentence(language, text) {
            self.remove_clip_files(&clip_cache_key(language, &row.clip_id))
                .await;
        }
    }

    async fn remove_clip_files(&self, cache_key: &str) {
        let mut dir = self.clips_dir.clone();
        let _ = dir.remove_entry(&format!("{cache_key}.mp4")).await;
        let _ = dir.remove_entry(&subtitles_filename(cache_key)).await;
    }

    /// The subtitle cues for a clip, from cache or the backend. Failures
    /// come back as an empty list and cache nothing — captions are an
    /// enhancement, never a reason to hold up (or retry-loop) the video.
    async fn subtitles(
        &self,
        language: Language,
        clip_id: &str,
        access_token: Option<&String>,
    ) -> Vec<ClipSubtitleCue> {
        let filename = subtitles_filename(&clip_cache_key(language, clip_id));
        if let Some(bytes) = self.read_file(&filename).await
            && let Ok(cues) = serde_json::from_slice(&bytes)
        {
            return cues;
        }
        match download_subtitles(language, clip_id, access_token).await {
            Ok(cues) => {
                if let Ok(bytes) = serde_json::to_vec(&cues) {
                    self.write_file(&filename, &bytes).await;
                }
                cues
            }
            Err(e) => {
                log::warn!("Failed to fetch subtitles for {clip_id}: {e}");
                Vec::new()
            }
        }
    }

    /// The clip for a sentence per the loaded manifest, from cache or the
    /// backend. `Ok(None)` means the sentence has no clip (or no manifest has
    /// loaded yet, which the app treats the same way).
    pub async fn fetch_and_cache(
        &self,
        language: Language,
        text: &str,
        access_token: Option<&String>,
    ) -> Result<Option<FetchedClip>, Error> {
        let Some(row) = clip_for_sentence(language, text) else {
            return Ok(None);
        };

        let cache_key = clip_cache_key(language, &row.clip_id);
        let video_filename = format!("{cache_key}.mp4");
        // Revalidate against the manifest's exact size: the pipeline can
        // republish corrected media under the same clip id, and a cached
        // copy of the old bytes would otherwise live for as long as it kept
        // being played. (lo_bytes 0 means a manifest from before the field
        // existed — accept any valid mp4 rather than redownloading forever.)
        let cached = self
            .read_file(&video_filename)
            .await
            .filter(|b| is_valid_mp4(b))
            .filter(|b| row.lo_bytes == 0 || b.len() as u64 == row.lo_bytes);
        let bytes = match cached {
            Some(bytes) => bytes,
            None => {
                // Stale or missing media: drop the cached subtitles too, so
                // captions can't pair old timings with the new video.
                let mut dir = self.clips_dir.clone();
                let _ = dir.remove_entry(&subtitles_filename(&cache_key)).await;
                self.download_coalesced(language, &row, access_token)
                    .await
                    .map_err(Error::new)?
            }
        };
        let subtitles = self.subtitles(language, &row.clip_id, access_token).await;
        self.touch_index(&cache_key).await;
        Ok(Some(FetchedClip {
            bytes,
            clip_id: row.clip_id,
            duration_ms: row.duration_ms,
            critical: row.critical,
            subtitles,
        }))
    }

    /// Download a clip through the per-id shared future (see
    /// `IN_FLIGHT_CLIP_FETCHES`), writing it to the cache exactly once.
    async fn download_coalesced(
        &self,
        language: Language,
        row: &ClipRow,
        access_token: Option<&String>,
    ) -> Result<Vec<u8>, String> {
        let cache_key = clip_cache_key(language, &row.clip_id);
        let fetch = IN_FLIGHT_CLIP_FETCHES.with(|map| {
            let mut map = map.borrow_mut();
            if let Some(fetch) = map.get(&cache_key) {
                return fetch.clone();
            }
            let cache = self.clone();
            let row = row.clone();
            let cache_key_in_future = cache_key.clone();
            let access_token = access_token.cloned();
            let fetch: SharedClipFetch = async move {
                let cache_key = cache_key_in_future;
                let result = download_clip(language, &row, access_token.as_ref()).await;
                if let Ok(bytes) = &result {
                    cache.write_file(&format!("{cache_key}.mp4"), bytes).await;
                }
                // Remove only after the cache write, so a caller arriving
                // between removal and return finds the file in OPFS.
                IN_FLIGHT_CLIP_FETCHES.with(|map| map.borrow_mut().remove(&cache_key));
                result
            }
            .boxed_local()
            .shared();
            map.insert(cache_key, fetch.clone());
            fetch
        });
        fetch.await
    }

    /// Evict cached videos that are neither in `keep_clip_ids` (the prefetch
    /// simulation's upcoming clips) nor recently used. Manifests are never
    /// evicted — they're small and the mirror depends on them at boot.
    pub async fn cleanup_except(&mut self, keep_cache_keys: BTreeSet<String>) -> Result<(), Error> {
        use futures::StreamExt;
        let now = chrono::Utc::now().timestamp();
        let mut index = self.read_index().await;
        let mut index_changed = false;

        let (files_to_delete, present_ids) = {
            let mut entries = self
                .clips_dir
                .entries()
                .await
                .map_err(|e| Error::new(format!("Failed to read clips directory: {e:?}")))?;
            let mut to_delete = Vec::new();
            let mut present = BTreeSet::new();
            while let Some(Ok((filename, _))) = entries.next().await {
                let Some(cache_key) = filename.strip_suffix(".mp4") else {
                    continue;
                };
                if keep_cache_keys.contains(cache_key) {
                    present.insert(cache_key.to_string());
                    continue;
                }
                match index.get(cache_key) {
                    Some(&last_used) if now - last_used > CLIP_MAX_UNUSED_AGE_SECS => {
                        to_delete.push(filename);
                    }
                    Some(_) => {
                        present.insert(cache_key.to_string());
                    }
                    None => {
                        // Unindexed (predates the index, or its touch
                        // failed): start its clock now rather than deleting
                        // something that may have been used a minute ago.
                        index.insert(cache_key.to_string(), now);
                        index_changed = true;
                        present.insert(cache_key.to_string());
                    }
                }
            }
            (to_delete, present)
        };

        for filename in files_to_delete {
            log::info!("Removing unused clip: {filename}");
            if let Err(e) = self.clips_dir.remove_entry(&filename).await {
                log::info!("Failed to remove clip {filename}: {e:?}");
                continue;
            }
            if let Some(cache_key) = filename.strip_suffix(".mp4") {
                let _ = self
                    .clips_dir
                    .clone()
                    .remove_entry(&subtitles_filename(cache_key))
                    .await;
                index.remove(cache_key);
                index_changed = true;
            }
        }

        // Drop index entries for clips that no longer exist.
        let stale: Vec<String> = index
            .keys()
            .filter(|id| !present_ids.contains(*id))
            .cloned()
            .collect();
        for id in stale {
            index.remove(&id);
            index_changed = true;
        }

        if index_changed {
            self.write_index(&index).await;
        }
        Ok(())
    }
}

/// Grader context for a sentence: a movie title from the challenge's
/// `(imdb_id, title)` candidates plus the surrounding dialogue lines from
/// the clip's subtitles, when a clip exists. A sentence can appear in
/// several films; when there's a clip, the title is the clip's own film so
/// it can't contradict the dialogue lines cut from it — otherwise the first
/// candidate. Best-effort: any failure just yields less context, never an
/// error.
pub(crate) async fn grader_context(
    language: Language,
    text: &str,
    movie_titles: Vec<(String, String)>,
) -> language_utils::autograde::GraderContext {
    let mut context = language_utils::autograde::GraderContext::default();
    let clip = clip_for_sentence(language, text);

    let clip_imdb = clip
        .as_ref()
        .and_then(|row| row.clip_id.split('-').next())
        .map(str::to_string);
    context.movie_title = match &clip_imdb {
        Some(imdb) => movie_titles
            .iter()
            .find(|(id, _)| id == imdb)
            .map(|(_, title)| title.clone()),
        None => movie_titles.first().map(|(_, title)| title.clone()),
    };

    // Cached cues only: the clip just played, so its subtitles are almost
    // certainly on disk — and grading must never wait on a network fetch
    // for optional context.
    if let Some(row) = clip
        && let Ok(cache) = ClipCache::new().await
        && let Some(bytes) = cache
            .read_file(&subtitles_filename(&clip_cache_key(language, &row.clip_id)))
            .await
        && let Ok(cues) = serde_json::from_slice::<Vec<ClipSubtitleCue>>(&bytes)
    {
        for cue in cues {
            match cue.role.as_str() {
                "context-before" => context.dialogue_before.push(cue.text),
                "context-after" => context.dialogue_after.push(cue.text),
                _ => {}
            }
        }
    }
    context
}

fn is_valid_mp4(bytes: &[u8]) -> bool {
    bytes.len() > 8 && &bytes[4..8] == b"ftyp"
}

fn subtitles_filename(cache_key: &str) -> String {
    format!("{cache_key}.subs.json")
}

/// Download a clip's subtitle cues from the backend.
async fn download_subtitles(
    language: Language,
    clip_id: &str,
    access_token: Option<&String>,
) -> Result<Vec<ClipSubtitleCue>, String> {
    let path = format!("/clip/{}/{}/subtitles", language.code(), clip_id);
    let response = hit_ai_server(fetch_happen::Method::GET, &path, None::<&()>, access_token)
        .await
        .map_err(|e| format!("Subtitle request error: {e:?}"))?;
    if !response.ok() {
        return Err(format!("Subtitle HTTP error: {}", response.status()));
    }
    response
        .json()
        .await
        .map_err(|e| format!("Subtitle parse error: {e:?}"))
}

/// Download a clip's lo.mp4 from the backend. The expected byte size rides
/// in the URL as a content revision: the video is served (and HTTP-cached)
/// as immutable, so when the pipeline republishes corrected media under the
/// same clip id, the changed size makes a fresh URL that no cache — the
/// browser's included — can satisfy with the old bytes.
async fn download_clip(
    language: Language,
    row: &ClipRow,
    access_token: Option<&String>,
) -> Result<Vec<u8>, String> {
    let path = format!(
        "/clip/{}/{}/lo.mp4?bytes={}",
        language.code(),
        row.clip_id,
        row.lo_bytes
    );
    let response = hit_ai_server(fetch_happen::Method::GET, &path, None::<&()>, access_token)
        .await
        .map_err(|e| format!("Clip download request error: {e:?}"))?;
    if !response.ok() {
        return Err(format!("Clip download HTTP error: {}", response.status()));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Clip download read error: {e:?}"))?;
    if !is_valid_mp4(&bytes) {
        return Err("Clip download returned invalid mp4".to_string());
    }
    Ok(bytes)
}
