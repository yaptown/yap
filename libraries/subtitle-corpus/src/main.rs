//! Tools for building a machine-readable, correctly-synced subtitle for every
//! movie in the library — the substrate everything else (clip cutting, audio
//! datasets) needs first.
//!
//! Sources are preferred in the order that costs least and is most trustworthy:
//! a text track already on the disc, then the disc's bitmap track read by OCR,
//! then a downloaded subtitle synchronised to the file. The first two are
//! authored against this exact file, so their timings need no correction at
//! all — which is why two thirds of the library is free.

use subtitle_corpus::{audio_check, library, ocr, pgs, sync, transcript, vad, vobsub};

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use language_utils::Language;
use library::{
    disc_track, film_filename, film_stamp, output_is_fresh, plan_path, read_plan, subtitle_stamp,
    truncate, FilmStamp, Movie, Source,
};
use serde::{Deserialize, Serialize};
use subtitle_corpus::export::export_yap;
use whisper::CloudflareWhisper;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command_,
}

#[derive(Subcommand, Debug)]
enum Command_ {
    /// Proofread orthography and flag incoherent course sentences.
    Proofread(subtitle_corpus::proofread::Options),
    /// Detect single-word disagreements and review spelling versus audio mismatches.
    WordCheck {
        #[arg(long, default_value = "/data/andrep/subtitle-corpus")]
        out: PathBuf,
        /// Course language code (for example fra or zho-hans).
        #[arg(long)]
        language: Option<String>,
        #[arg(long)]
        imdb: Option<String>,
        /// Stop after this many films (0 = all).
        #[arg(long, default_value_t = 0)]
        limit: usize,
        /// Write candidates without calling the judge or changing corrections.
        #[arg(long)]
        dry_run: bool,
        /// Judge hand-labeled JSONL through the production path, without applying.
        #[arg(long)]
        eval: Option<PathBuf>,
    },
    /// Probe every movie and decide where its subtitle will come from.
    Inventory {
        /// JSON from `arr radarr raw GET /movie`.
        #[arg(long)]
        library: PathBuf,
        /// Root of the yap language data (for already-downloaded subtitles).
        #[arg(long, default_value = "./generate-data/data")]
        data_root: PathBuf,
        #[arg(long, default_value = "/data/andrep/subtitle-corpus")]
        out: PathBuf,
        #[arg(long, default_value_t = 8)]
        jobs: usize,
    },
    /// Bring the corpus up to date with whatever arrived since last time.
    ///
    /// Runs inventory, subtitle/audio extraction, synchronization, transcription,
    /// verification and clip mapping, skipping finished work,
    /// so a run where nothing changed costs nearly nothing. OCR is included:
    /// its spend per new film is trivial and its batches are cached, so an
    /// interrupted film simply completes on the next refresh.
    Refresh {
        /// JSON from `arr radarr raw GET /movie`.
        #[arg(long)]
        library: PathBuf,
        #[arg(long, default_value = "./generate-data/data")]
        data_root: PathBuf,
        #[arg(long, default_value = "/data/andrep/subtitle-corpus")]
        out: PathBuf,
        /// Limit full-film transcription and its audio extraction to these
        /// IMDb ids.
        ///
        /// May be repeated. Every other refresh stage still considers the whole
        /// inventory. With no values, refresh processes every film as before.
        #[arg(long)]
        transcribe_imdb: Vec<String>,
    },
    /// Pull out the subtitles that are already text on the disc.
    Extract {
        #[arg(long, default_value = "/data/andrep/subtitle-corpus")]
        out: PathBuf,
        #[arg(long, default_value_t = 6)]
        jobs: usize,
        /// Stop after this many movies (0 = all).
        #[arg(long, default_value_t = 0)]
        limit: usize,
    },
    /// Pull each film's original-language audio out as a seekable opus file.
    ///
    /// Written into the corpus (never beside the videos) as `audio.opus`, with
    /// `audio.json` recording the source file *and* the exact stream it came
    /// from — a changed video or a reshuffled remux evicts the track rather
    /// than posing as it. The syncers prefer this artifact when it is current:
    /// whisper windows and VAD profiles then read a few hundred MB of opus
    /// instead of demuxing a lossless track out of a 30GB remux.
    ExtractAudio {
        #[arg(long, default_value = "/data/andrep/subtitle-corpus")]
        out: PathBuf,
        #[arg(long, default_value_t = 6)]
        jobs: usize,
        /// Stop after this many movies (0 = all).
        #[arg(long, default_value_t = 0)]
        limit: usize,
        /// Extract audio for this film alone, by IMDb id.
        #[arg(long)]
        imdb: Option<String>,
    },
    /// Listen to each extracted track and confirm it is the film's own
    /// dialogue in the language the library expects.
    ///
    /// Three windows of `audio.opus` go to Gemini, which names the language
    /// spoken (variety included: Mandarin against Cantonese) and says whether
    /// people are talking *about* the film — a commentary. A track that fails
    /// is evicted along with everything derived from it, recorded in
    /// `audio-rejected.json` so the extractor never picks it again, and the
    /// next candidate stream on the disc is extracted and judged in turn.
    /// The verdict is kept in `audio-check.json`, so a track is heard once.
    AudioCheck {
        #[arg(long, default_value = "/data/andrep/subtitle-corpus")]
        out: PathBuf,
        /// Films in flight at once.
        #[arg(long, default_value_t = 4)]
        jobs: usize,
        /// Stop after this many movies (0 = all).
        #[arg(long, default_value_t = 0)]
        limit: usize,
        /// Check this film alone, by IMDb id.
        #[arg(long)]
        imdb: Option<String>,
    },
    /// Read every bitmap subtitle track in the library back into text.
    Ocr {
        #[arg(long, default_value = "/data/andrep/subtitle-corpus")]
        out: PathBuf,
        #[arg(long, default_value = "gpt-5.6-luna")]
        model: String,
        /// Films whose batches run at once (0 = every film in the queue). A
        /// batch can take a day to come back, so films waiting on one
        /// another's batches would turn a backlog into weeks.
        #[arg(long, default_value_t = 0)]
        films_in_flight: usize,
        /// Stop after this many movies (0 = all).
        #[arg(long, default_value_t = 0)]
        limit: usize,
    },
    /// Align downloaded subtitles to the films on disk, using Whisper.
    Sync {
        #[arg(long, default_value = "/data/andrep/subtitle-corpus")]
        out: PathBuf,
        /// Root of the yap language data, where downloaded raw SRTs live.
        #[arg(long, default_value = "./generate-data/data")]
        data_root: PathBuf,
        /// Films aligned at once.
        #[arg(long, default_value_t = 4)]
        films_in_flight: usize,
        /// Stop after this many films (0 = all).
        #[arg(long, default_value_t = 0)]
        limit: usize,
    },
    /// Transcribe every film in full, so its subtitle can be checked against
    /// what was actually said.
    ///
    /// Reads the extracted `audio.opus`, cuts it at the quietest seams the
    /// film's speech profile offers, and writes `transcript.jsonl` beside the
    /// subtitle. Responses are cached in the shared store by chunk bytes, so
    /// re-running is free for everything already transcribed.
    Transcribe {
        #[arg(long, default_value = "/data/andrep/subtitle-corpus")]
        out: PathBuf,
        /// Films transcribed at once.
        #[arg(long, default_value_t = 4)]
        films_in_flight: usize,
        /// Stop after this many films (0 = all).
        #[arg(long, default_value_t = 0)]
        limit: usize,
        /// Transcribe this film alone, by IMDb id.
        #[arg(long)]
        imdb: Option<String>,
    },
    /// Build earshot's 16 ms speech profile for every film with extracted
    /// audio (`speech-profile-16ms.f32` beside it).
    ///
    /// The profile used to be computed only when a syncer asked for one, so
    /// finished films — the ones clip cutting works on — never got the fine
    /// version and kept their retired 96 ms file. Films with a current
    /// profile are skipped, so a refresh only pays for new audio.
    SpeechProfiles {
        #[arg(long, default_value = "/data/andrep/subtitle-corpus")]
        out: PathBuf,
        /// Films profiled at once (each is one ffmpeg decode).
        #[arg(long, default_value_t = 6)]
        jobs: usize,
        /// Stop after this many films (0 = all).
        #[arg(long, default_value_t = 0)]
        limit: usize,
    },
    /// Segment every model-segmented film's subtitle into sentences in one
    /// Batch API round trip, so that everything downstream finds the answers
    /// in the cache.
    ///
    /// Japanese, Mandarin, Korean and Thai subtitles leave statements
    /// unpunctuated, so their sentence boundaries come from a language model
    /// (`movie_subtitles::llm_segment`), one request per cue. Nothing is
    /// written beside the film: tysm's response cache is the store, and
    /// transcript-check, clips and export all read from it. Without this
    /// step each of them would run a film-sized batch of its own, serially.
    /// Only transcribed films by default — nothing reads the others' sentences
    /// yet, and a request per cue adds up.
    Segment {
        #[arg(long, default_value = "/data/andrep/subtitle-corpus")]
        out: PathBuf,
        /// Every film with a subtitle, transcribed or not.
        #[arg(long)]
        all: bool,
        /// Stop after this many films (0 = all).
        #[arg(long, default_value_t = 0)]
        limit: usize,
        /// Segment this film alone, by IMDb id.
        #[arg(long)]
        imdb: Option<String>,
    },
    /// Judge every transcribed film's subtitle against what is actually
    /// said, and replace the ones that fail.
    ///
    /// Sync gates only ask *when*; this asks *what*. A rewrite into another
    /// variety, a condensed SDH track or a paraphrasing fansub can be
    /// perfectly timed and still place almost none of its sentences in the
    /// transcript — and then it is not a source of sentences for this audio.
    /// Verdicts go to `transcript-check.json` beside each film; `clips` maps
    /// only films judged verbatim. A verbatim subtitle on another clock is
    /// re-timed in place (unless it came off the disc, where the *audio* is
    /// then the suspect). For the rest, every other subtitle already on
    /// hand for the film is measured the same way, then OpenSubtitles is
    /// asked for more (spending at most `--max-downloads`), and the best
    /// one that clears the bar is adopted as the film's subtitle.
    TranscriptCheck {
        #[arg(long, default_value = "/data/andrep/subtitle-corpus")]
        out: PathBuf,
        #[arg(long, default_value = "./generate-data/data")]
        data_root: PathBuf,
        /// Stop after this many films (0 = all).
        #[arg(long, default_value_t = 0)]
        limit: usize,
        /// One film only, by IMDb id.
        #[arg(long)]
        imdb: Option<String>,
        /// Least share of eligible sentences that must place for a subtitle
        /// to count as verbatim. Default: the per-language bar
        /// (`verbatim::min_fraction`).
        #[arg(long)]
        min_verbatim: Option<f64>,
        /// Only report; never re-time, download or adopt anything.
        #[arg(long)]
        dry_run: bool,
        /// Most OpenSubtitles downloads to spend this run (0 = none; local
        /// candidates are still tried).
        #[arg(long, default_value_t = 0)]
        max_downloads: usize,
        /// Most candidates to download per film.
        #[arg(long, default_value_t = 3)]
        max_candidates: usize,
    },
    /// Align remaining subtitles by matching speech activity, not words.
    ///
    /// Complements `sync`: it reads no vocabulary, so paraphrase, archaic
    /// speech and mishearing cannot hurt it, and it weighs the whole film
    /// rather than a few sampled windows. It finds only a constant shift.
    VadSync {
        #[arg(long, default_value = "/data/andrep/subtitle-corpus")]
        out: PathBuf,
        #[arg(long, default_value = "./generate-data/data")]
        data_root: PathBuf,
        #[arg(long, default_value_t = 3)]
        jobs: usize,
        #[arg(long, default_value_t = 0)]
        limit: usize,
    },
    /// Align a downloaded subtitle against the disc's own subtitle tracks,
    /// with no audio involved.
    ///
    /// Any subtitle track on the disc — whatever its language — was authored
    /// against this exact file, so its cue *timings* are ground truth. Even a
    /// bitmap track works: only the timestamps are read, no OCR. This is
    /// exactly where both audio methods fail — a sparse, music-heavy film
    /// starves Whisper and VAD alike, but its reference track carries the same
    /// sparseness on the correct clock.
    TextSync {
        #[arg(long, default_value = "/data/andrep/subtitle-corpus")]
        out: PathBuf,
        #[arg(long, default_value = "./generate-data/data")]
        data_root: PathBuf,
        #[arg(long, default_value_t = 4)]
        jobs: usize,
        /// Stop after this many films (0 = all).
        #[arg(long, default_value_t = 0)]
        limit: usize,
    },
    /// Map every sentence in each transcribed film to the clip it is spoken
    /// in, verified by the transcript and by the phoneme model. Writes
    /// `clips.jsonl` beside the transcript; current films are skipped.
    Clips {
        #[arg(long, default_value = "/data/andrep/subtitle-corpus")]
        out: PathBuf,
        #[arg(long, default_value_t = 2)]
        jobs: usize,
        #[arg(long, default_value_t = 0)]
        limit: usize,
        /// One film only, by IMDb id.
        #[arg(long)]
        imdb: Option<String>,
        /// Comma-separated course codes (fra,spa,…); default every language
        /// with a phoneme reference.
        #[arg(long, value_delimiter = ',')]
        langs: Option<Vec<String>>,
        /// Lowest CTC log-odds ratio (per phoneme, against the model's own
        /// reading) a clip may have and still pass. Default: the calibrated
        /// per-language cut.
        #[arg(long, allow_hyphen_values = true)]
        min_ratio: Option<f64>,
    },
    /// Cut serve-ready video clips (two renditions + sidecar JSON) for every
    /// passing clip. See docs/clip-sidecar.md for the schema.
    ExportClips {
        #[arg(long, default_value = "/data/andrep/subtitle-corpus")]
        out: PathBuf,
        /// Where the clip directories land, under `<dest>/<lang>/<id>/`.
        #[arg(long, default_value = "/data/andrep/subtitle-corpus/export")]
        dest: PathBuf,
        /// Clips encoded at once (each is one ffmpeg run).
        #[arg(long, default_value_t = 4)]
        jobs: usize,
        /// Stop after this many films (0 = all).
        #[arg(long, default_value_t = 0)]
        limit: usize,
        /// One film only, by IMDb id.
        #[arg(long)]
        imdb: Option<String>,
        /// Comma-separated course codes (fra,spa,…); default every language.
        #[arg(long, value_delimiter = ',')]
        langs: Option<Vec<String>>,
    },
    /// The whole serve pipeline in order — clips, export-clips, R2 upload, export-yap —
    /// each stage resumable and skipping finished work (like refresh).
    Publish {
        #[arg(long, default_value = "/data/andrep/subtitle-corpus")]
        out: PathBuf,
        #[arg(long, default_value = "/data/andrep/subtitle-corpus/export")]
        dest: PathBuf,
        /// Root of Yap's language data, containing each course's sentence sources.
        #[arg(long, default_value = "./generate-data/data")]
        data_root: PathBuf,
        /// Clips encoded at once (each is one ffmpeg run).
        #[arg(long, default_value_t = 8)]
        jobs: usize,
        /// Comma-separated course codes (fra,spa,…); default every language.
        #[arg(long, value_delimiter = ',')]
        langs: Option<Vec<String>>,
        #[arg(long, default_value = "yap-clips")]
        bucket: String,
    },
    /// Review the previous publish's orphan candidates; dry run unless --apply.
    Prune {
        #[arg(long, default_value = "/data/andrep/subtitle-corpus/export")]
        dest: PathBuf,
        #[arg(long, default_value = "yap-clips")]
        bucket: String,
        /// Candidate manifest; defaults to <dest>/orphan-candidates.json.
        #[arg(long)]
        manifest: Option<PathBuf>,
        /// Delete the listed local clip dirs and R2 objects.
        #[arg(long)]
        apply: bool,
    },
    /// Publish finished subtitles next to their films as media-server sidecars.
    ///
    /// Writes `<video>.yap.<lang>.srt` beside each film whose corpus subtitle
    /// is verified against the file currently on disk — jellyfin shows it as
    /// a subtitle track titled "yap". Only files matching `*.yap.*.srt` are
    /// ever created or deleted, so shipped and Bazarr sidecars are untouched;
    /// stale or orphaned yap-sidecars are removed. `classify` ignores the
    /// `.yap.` namespace, so the corpus never rediscovers its own output as a
    /// source.
    ExportSidecars {
        #[arg(long, default_value = "/data/andrep/subtitle-corpus")]
        out: PathBuf,
    },
    /// Copy fresh, verbatim subtitles into Yap's raw movie sentence sources.
    /// Downloaded sources are skipped to avoid overwriting their own inputs.
    ExportYap {
        #[arg(long, default_value = "/data/andrep/subtitle-corpus")]
        out: PathBuf,
        /// Root of Yap's language data, containing each course's sentence sources.
        #[arg(long, default_value = "./generate-data/data")]
        data_root: PathBuf,
        /// Comma-separated course codes (fra,spa,…). Default: every course
        /// except eng — see [`export_yap`].
        #[arg(long, value_delimiter = ',')]
        langs: Option<Vec<String>>,
        /// Report changes without creating directories or writing files.
        #[arg(long)]
        dry_run: bool,
    },
}

/// The subtitle file a syncer should align for this movie, if any.
///
/// Usually the downloaded raw SRT in the movie's language pack. Failing that,
/// a sidecar file counts too: `extract` used to trust sidecars as
/// already-synced, but 23 of 49 turned out to be Bazarr downloads on some
/// other release's clock (*Il Mare* 3.4s out at margin 0.30) — so a sidecar
/// whose finalized output has been removed re-enters through the same sync
/// gates as any download.
/// Films whose only available subtitle was authored against a different cut,
/// so no single offset+rate can place it: Whisper finds a strong local fit
/// that a film-wide re-check then contradicts (a fresh Still Life fit of
/// +74.71s re-measured at +14.30s with 45/101 anchors). Every sync pass
/// would re-place and re-fail these forever; they wait for a replacement
/// release instead.
const DIFFERENT_CUT: &[&str] = &[
    "tt0859765", // Still Life (2006)
    "tt0209189", // Not One Less (1999)
    "tt3742378", // The Second Mother (2015) — sync locks +26.0s on 8/20 anchors, check re-measures +123.5s
];

/// Sidecars known to carry another release's clock.
///
/// `extract` trusts a sidecar's own timings, and for most that is right —
/// but a Bazarr download can be timed to a different rip of the same cut
/// (Fallen Angels −19.2s over 104/110 anchors, When Marnie Was There +23.3s
/// over 341/355, both at rate 1.0000). Deleting the finalized output is not
/// enough on its own: the next `refresh` would just re-finalize the same
/// file. Listing a film here makes `extract` leave it alone while `classify`
/// still reports the sidecar, so `subtitle_source` hands its text to the
/// syncers to be placed like any downloaded subtitle.
const SIDECAR_UNTRUSTED: &[&str] = &[
    "tt0112913", // Fallen Angels (1995)
    "tt3398268", // When Marnie Was There (2014)
    "tt1568921", // The Secret World of Arrietty (2010) — +28.5s over 257/262 anchors
    // Convicted 2026-09-01 by the subtitle-vs-transcript audit (bigram match
    // peaks far off zero; the transcript's clock is the film's by construction):
    "tt0120915", // Phantom Menace — −60s
    "tt1183252", // Chocolate (2008) — −5s
    "tt6788942", // Bad Genius (2017) — +5s
];

/// A subtitle `transcript-check` chose over the plan's source, kept beside
/// the film's output. Present, it *is* the film's subtitle source: every
/// resolver answers it first, and the stamp records it, so `inventory`
/// neither evicts it as a stranger nor re-extracts the track it replaced.
fn adopted_subtitle(dir: &std::path::Path) -> Option<PathBuf> {
    let path = dir.join("adopted.srt");
    path.exists().then_some(path)
}

fn subtitle_source(
    movie: &Movie,
    dir: &std::path::Path,
    data_root: &std::path::Path,
) -> Option<PathBuf> {
    // A film with no original-language audio can never yield a speech clip,
    // so aligning a subtitle for it is work spent making a number wrong.
    if matches!(movie.source, Source::NoOriginalAudio) {
        return None;
    }
    if let Some(adopted) = adopted_subtitle(dir) {
        return Some(adopted);
    }
    if DIFFERENT_CUT.contains(&movie.imdb_id.as_str()) {
        return None;
    }
    // The plan already ranked a sidecar above a course download (`classify`),
    // and `export-yap` writes our verified subtitle back as that download —
    // answering the raw first would make the stamp disagree with the plan and
    // have `inventory` evict and re-extract the same sidecar on every refresh.
    if let Source::Sidecar { path } = &movie.source {
        if path.exists() {
            return Some(path.clone());
        }
    }
    if let Some(course) = library::course_dir(&movie.original_language) {
        let raw = data_root
            .join(
                language_utils::Language::from_code(course)
                    .unwrap()
                    .corpus_code(),
            )
            .join("sentence-sources/movies")
            .join(format!("subtitles-raw/{}.srt", movie.imdb_id));
        if raw.exists() {
            return Some(raw);
        }
    }
    None
}

/// What a writer knows about the subtitle source it derived from.
enum StampSource<'a> {
    /// The subtitle came off the disc itself; there is no separate source.
    Disc,
    /// Derived from this subtitle file.
    File(&'a std::path::Path),
    /// Says nothing about the subtitle — a speech profile or reference cache
    /// must not erase what the last subtitle writer recorded.
    Keep,
}

fn read_stamp(dir: &std::path::Path) -> Option<FilmStamp> {
    serde_json::from_slice(&std::fs::read(dir.join("film.json")).ok()?).ok()
}

/// The subtitle file the film's output *should* currently derive from, or
/// None for disc-sourced films (their source is the video itself).
fn expected_subtitle_source(
    movie: &Movie,
    dir: &std::path::Path,
    data_root: &std::path::Path,
) -> Option<PathBuf> {
    if let Some(adopted) = adopted_subtitle(dir) {
        return Some(adopted);
    }
    match &movie.source {
        Source::DiscText { .. } | Source::DiscBitmap { .. } => None,
        _ => subtitle_source(movie, dir, data_root),
    }
}

/// The film's speech profile, from the per-film cache when present.
///
/// Decoding a feature's audio takes minutes; the profile is ~260KB. Cached
/// next to the subtitle and stamped with the film's identity so `inventory`
/// evicts it when the file changes underneath — which matters even for films
/// with no finished subtitle yet, where only the profile exists.
fn cached_speech_profile(movie: &Movie, dir: &std::path::Path) -> Result<Vec<f32>> {
    Ok(vad::bucketed(
        &fine_speech_profile(movie, dir)?,
        vad::FRAMES_PER_BUCKET,
    ))
}

/// earshot's native 16 ms profile, cached beside the film. The `-16ms` in the
/// name is the cache key: it retires the old 96 ms `speech-profile.f32` files
/// so a refresh recomputes them at full resolution rather than mistaking a
/// coarse profile for a fine one.
fn fine_speech_profile(movie: &Movie, dir: &std::path::Path) -> Result<Vec<f32>> {
    let path = dir.join("speech-profile-16ms.f32");
    if let Some(p) = vad::read_profile(&path) {
        return Ok(p);
    }
    let (media, stream) = audio_source(movie, dir)?;
    let p = vad::speech_profile(&media, stream)?;
    std::fs::create_dir_all(dir)?;
    vad::write_profile(&path, &p)?;
    write_stamp(dir, movie, StampSource::Keep);
    Ok(p)
}

/// Cue timings for one of the disc's reference tracks, cached per film.
///
/// Reading a bitmap reference means demuxing the whole film — minutes — for a
/// product that is a few KB of timestamps. Cached like the speech profile,
/// stamped with the film's identity, evicted with it when the film changes.
/// Only timings survive the cache; text-sync never reads reference *text*.
fn cached_reference_cues(
    movie: &Movie,
    out: &std::path::Path,
    stream: &library::ReferenceStream,
    scratch: &std::path::Path,
) -> Result<Vec<sync::Cue>> {
    let dir = out.join(&movie.imdb_id);
    let path = dir.join("references.json");
    let mut cache: std::collections::HashMap<u32, Vec<(i64, i64)>> = std::fs::read(&path)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    if let Some(spans) = cache.get(&stream.index) {
        return Ok(spans
            .iter()
            .map(|&(start_ms, end_ms)| sync::Cue {
                start_ms,
                end_ms,
                text: String::new(),
            })
            .collect());
    }
    let cues = reference_cues(&movie.path, stream, scratch)?;
    cache.insert(
        stream.index,
        cues.iter().map(|c| (c.start_ms, c.end_ms)).collect(),
    );
    std::fs::create_dir_all(&dir)?;
    std::fs::write(&path, serde_json::to_vec(&cache)?)?;
    write_stamp(&dir, movie, StampSource::Keep);
    Ok(cues)
}

/// Best-effort: stamp failures do not fail a sync that already succeeded.
fn write_stamp(dir: &std::path::Path, movie: &Movie, source: StampSource) {
    let Ok(mut stamp) = film_stamp(movie) else {
        return;
    };
    (stamp.subtitle, stamp.track) = match source {
        StampSource::Disc => (None, disc_track(movie)),
        StampSource::File(p) => (subtitle_stamp(p), None),
        StampSource::Keep => read_stamp(dir).map_or((None, None), |s| (s.subtitle, s.track)),
    };
    if let Ok(json) = serde_json::to_vec_pretty(&stamp) {
        let _ = std::fs::write(dir.join("film.json"), json);
    }
    // A finalized output supersedes any recorded failure.
    if !matches!(source, StampSource::Keep) {
        let _ = std::fs::remove_file(dir.join("sync-failed.json"));
    }
}

/// The inputs a failed alignment was attempted against: the video's identity
/// plus the subtitle file it tried to place.
fn sync_failure_stamp(movie: &Movie, raw: &std::path::Path) -> Option<FilmStamp> {
    let mut stamp = film_stamp(movie).ok()?;
    stamp.subtitle = subtitle_stamp(raw);
    Some(stamp)
}

/// Alignment is deterministic in its inputs: until the video or the subtitle
/// file changes, re-running the gauntlet reproduces the same failure. Every
/// refresh used to grind all ~18 hopeless films through text-sync, sync and
/// vad-sync anyway — half an hour of audio decoding and Whisper windows to
/// learn nothing. `vad-sync` (the last gate) records the failed inputs as
/// `sync-failed.json`; a matching stamp skips the film in every sync queue. A
/// new download or release stops matching and retries automatically; deleting
/// the file forces a retry by hand.
fn sync_already_failed(movie: &Movie, dir: &std::path::Path, raw: &std::path::Path) -> bool {
    let Ok(bytes) = std::fs::read(dir.join("sync-failed.json")) else {
        return false;
    };
    let Ok(recorded) = serde_json::from_slice::<FilmStamp>(&bytes) else {
        return false;
    };
    let Some(current) = sync_failure_stamp(movie, raw) else {
        return false;
    };
    recorded.matches(&current) && recorded.subtitle == current.subtitle
}

fn record_sync_failure(movie: &Movie, dir: &std::path::Path, raw: &std::path::Path) {
    let Some(stamp) = sync_failure_stamp(movie, raw) else {
        return;
    };
    if let Ok(json) = serde_json::to_vec_pretty(&stamp) {
        let _ = std::fs::create_dir_all(dir);
        let _ = std::fs::write(dir.join("sync-failed.json"), json);
    }
}

/// Queue-builder suffix explaining how many films were skipped as already
/// failed on identical inputs.
fn parked_note(parked: usize) -> String {
    if parked == 0 {
        String::new()
    } else {
        format!(" ({parked} skipped: same inputs already failed; delete sync-failed.json to retry)")
    }
}

use sync::{read_audio_stamp, AudioStamp};

/// The film's extracted original-language audio, when it is present and still
/// belongs to the file on disk. File identity only — the per-stream probe is
/// `extract-audio`'s job; a caller here just needs to trust the artifact.
fn extracted_audio(movie: &Movie, dir: &std::path::Path) -> Option<PathBuf> {
    let path = dir.join("audio.opus");
    if !path.exists() {
        return None;
    }
    let stamp = read_audio_stamp(dir)?;
    let current = film_stamp(movie).ok()?;
    let recorded = FilmStamp {
        filename: stamp.filename,
        duration_ms: stamp.duration_ms,
        subtitle: None,
        track: None,
    };
    recorded.matches(&current).then_some(path)
}

/// Where to listen for this film: the extracted opus when it is current —
/// a few hundred MB that seeks instantly, instead of pulling a lossless
/// track out of a 30GB remux on the array — else the video itself.
fn audio_source(movie: &Movie, dir: &std::path::Path) -> Result<(PathBuf, usize)> {
    if let Some(audio) = extracted_audio(movie, dir) {
        return Ok((audio, 0));
    }
    let codes = library::stream_codes(&movie.original_language);
    let stream = sync::original_audio_stream(&movie.path, codes, &rejected_streams(movie, dir))?;
    Ok((movie.path.clone(), stream))
}

/// A track the listener turned down, kept so the extractor never picks it
/// again for this file. See `Command_::AudioCheck`.
#[derive(Serialize, Deserialize)]
struct RejectedStream {
    filename: String,
    stream: sync::AudioStreamIdentity,
    verdict: audio_check::Verdict,
}

fn read_rejected(dir: &std::path::Path) -> Vec<RejectedStream> {
    std::fs::read(dir.join("audio-rejected.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

/// Audio positions the listener has rejected for the file now on disk.
fn rejected_streams(movie: &Movie, dir: &std::path::Path) -> Vec<usize> {
    let filename = film_filename(movie);
    read_rejected(dir)
        .into_iter()
        .filter(|r| r.filename == filename)
        .map(|r| r.stream.stream_index)
        .collect()
}

enum Freshness {
    /// No finished subtitle, or the stamp still matches the inputs on disk.
    Fine,
    /// An input changed underneath the output — stale artifacts evicted.
    Evicted { why: String },
}

fn freshen_output(
    movie: &Movie,
    out: &std::path::Path,
    data_root: &std::path::Path,
) -> Result<Freshness> {
    let dir = out.join(&movie.imdb_id);
    let has_subtitle = dir.join("subtitle.srt").exists();
    // A stamp with no subtitle is a cached speech profile — still worth
    // checking, since a stale profile would poison the next vad-sync.
    if !has_subtitle && !dir.join("film.json").exists() {
        return Ok(Freshness::Fine);
    }
    // No film on disk is not evidence of change — the array may be offline.
    // Evict only when a present file positively fails to match.
    let Ok(current) = film_stamp(movie) else {
        return Ok(Freshness::Fine);
    };
    let old = read_stamp(&dir).with_context(|| {
        format!(
            "Missing or invalid film.json in {}; clear this film's generated output and rebuild",
            dir.display()
        )
    })?;
    if !old.matches(&current) {
        // The video changed: everything derived from it is stale.
        let _ = std::fs::remove_file(dir.join("subtitle.srt"));
        let _ = std::fs::remove_file(dir.join("speech-profile-16ms.f32"));
        let _ = std::fs::remove_file(dir.join("references.json"));
        let _ = std::fs::remove_file(dir.join("transcript.jsonl"));
        let _ = std::fs::remove_file(dir.join("audio.opus"));
        let _ = std::fs::remove_file(dir.join("audio.json"));
        let _ = std::fs::remove_file(dir.join("audio-check.json"));
        let _ = std::fs::remove_file(dir.join("audio-rejected.json"));
        let _ = std::fs::remove_file(dir.join("film.json"));
        let _ = std::fs::remove_file(dir.join("sync-failed.json"));
        let _ = std::fs::remove_file(dir.join("adopted.srt"));
        let _ = std::fs::remove_file(subtitle_corpus::verbatim::report_path(&dir));
        return Ok(Freshness::Evicted {
            why: format!("film changed ({} → {})", old.filename, current.filename),
        });
    }
    // Video unchanged; a disc-derived subtitle is still from the track the
    // plan names? (An adopted file records itself in `subtitle` and is
    // judged below; only a stamp with no file behind it is the disc's.)
    if let (Some(index), None) = (disc_track(movie), &old.subtitle) {
        if old.track != Some(index) {
            let _ = std::fs::remove_file(dir.join("subtitle.srt"));
            write_stamp(&dir, movie, StampSource::Disc);
            if has_subtitle {
                return Ok(Freshness::Evicted {
                    why: format!("disc track changed ({:?} → {index})", old.track),
                });
            }
            return Ok(Freshness::Fine);
        }
    }
    // Is the subtitle still derived from the right source file?
    let expected_path = expected_subtitle_source(movie, &dir, data_root);
    let expected = expected_path.as_deref().and_then(subtitle_stamp);
    if old.subtitle == expected || !has_subtitle {
        return Ok(Freshness::Fine);
    }
    let _ = std::fs::remove_file(dir.join("subtitle.srt"));
    write_stamp(&dir, movie, StampSource::Disc);
    Ok(Freshness::Evicted {
        why: "subtitle source changed".into(),
    })
}

/// Run `f` over `items` on `jobs` threads, reporting progress as it goes.
///
/// The work is IO-bound on ffmpeg reading whole films off the array, so the
/// results come back out of order and are re-sorted into the input order at the
/// end rather than being written into shared slots.
fn parallel<T: Send + Sync, R: Send>(
    items: Vec<T>,
    jobs: usize,
    label: &str,
    f: impl Fn(&T) -> R + Sync,
) -> Vec<R> {
    let total = items.len();
    let next = AtomicUsize::new(0);
    let done = AtomicUsize::new(0);
    let results = Mutex::new(Vec::with_capacity(total));

    std::thread::scope(|scope| {
        for _ in 0..jobs.max(1) {
            scope.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= total {
                    break;
                }
                let value = f(&items[i]);
                results.lock().unwrap().push((i, value));
                let n = done.fetch_add(1, Ordering::Relaxed) + 1;
                if n.is_multiple_of(25) || n == total {
                    eprintln!("  {label}: {n}/{total}");
                }
            });
        }
    });

    let mut out = results.into_inner().unwrap();
    out.sort_by_key(|(i, _)| *i);
    out.into_iter().map(|(_, v)| v).collect()
}

fn inventory(library: PathBuf, data_root: PathBuf, out: PathBuf, jobs: usize) -> Result<()> {
    let movies = library::load_library(&library)?;
    println!("{} movies on disk", movies.len());

    let probed = parallel(movies, jobs, "probing", |entry| -> Result<_> {
        let source = library::classify(
            &entry.imdb_id,
            &entry.path,
            &entry.original_language,
            &data_root,
        )
        .unwrap_or(Source::Missing);
        let movie = Movie {
            imdb_id: entry.imdb_id.clone(),
            title: entry.title.clone(),
            year: entry.year,
            path: entry.path.clone(),
            original_language: entry.original_language.clone(),
            source,
        };
        let freshness = freshen_output(&movie, &out, &data_root)?;
        Ok((movie, freshness))
    })
    .into_iter()
    .collect::<Result<Vec<_>>>()?;

    for (movie, freshness) in &probed {
        match freshness {
            Freshness::Fine => {}
            Freshness::Evicted { why } => println!(
                "  ✗ {} — {why}, evicted for re-derivation",
                truncate(&movie.title, 40),
            ),
        }
    }
    let classified: Vec<Movie> = probed.into_iter().map(|(m, _)| m).collect();

    std::fs::create_dir_all(&out)?;
    std::fs::write(plan_path(&out), serde_json::to_vec_pretty(&classified)?)?;

    let mut counts: std::collections::BTreeMap<&str, usize> = Default::default();
    for m in &classified {
        *counts.entry(m.source.label()).or_default() += 1;
    }
    println!("\nsubtitle source for each movie:");
    for (label, n) in &counts {
        println!("  {label:26}{n:>5}");
    }
    let ready = counts.get("disc text").copied().unwrap_or(0)
        + counts.get("disc bitmap (OCR)").copied().unwrap_or(0);
    println!(
        "\n{ready} of {} need no synchronisation at all (the disc's own track).",
        classified.len()
    );
    println!("wrote {}", plan_path(&out).display());
    Ok(())
}

/// Publish verified subtitles as `<video>.yap.<lang>.srt` sidecars, and
/// retract the ones the corpus no longer stands behind.
fn export_sidecars(out: PathBuf) -> Result<()> {
    let plan = read_plan(&out)?;
    let (mut written, mut kept, mut removed) = (0usize, 0usize, 0usize);
    for movie in &plan {
        let Some(video_dir) = movie.path.parent() else {
            continue;
        };
        let Some(stem) = movie.path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let lang = library::stream_codes(&movie.original_language)
            .first()
            .copied()
            .unwrap_or("und");
        let expected = format!("{stem}.yap.{lang}.srt");
        let fresh = output_is_fresh(movie, &out.join(&movie.imdb_id));

        // Everything in our namespace that is not the one file we currently
        // stand behind — old video names, evicted films — gets retracted.
        for entry in std::fs::read_dir(video_dir).into_iter().flatten().flatten() {
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            if !name.contains(".yap.") || !name.ends_with(".srt") {
                continue;
            }
            if !(fresh && name == expected) {
                let _ = std::fs::remove_file(entry.path());
                removed += 1;
            }
        }

        if fresh {
            let srt = std::fs::read(out.join(&movie.imdb_id).join("subtitle.srt"))?;
            let dest = video_dir.join(&expected);
            if std::fs::read(&dest).ok().as_deref() == Some(srt.as_slice()) {
                kept += 1;
            } else {
                std::fs::write(&dest, &srt)?;
                written += 1;
            }
        }
    }
    println!("{written} sidecars written, {kept} already current, {removed} retracted");
    Ok(())
}

/// The whole pipeline, in dependency order, with each step's own defaults.
///
/// Inventory failing aborts — every later step would read a stale plan and
/// quietly do the wrong work. Any other step failing is reported and skipped
/// past: a Whisper outage is no reason not to run text-sync, and the next
/// refresh retries whatever was left undone.
fn refresh(
    library: PathBuf,
    data_root: PathBuf,
    out: PathBuf,
    transcribe_imdb: Vec<String>,
) -> Result<()> {
    println!("━━━ inventory ━━━");
    inventory(library, data_root.clone(), out.clone(), 8)?;
    let audio_imdb = transcribe_imdb.clone();

    type Step<'a> = (&'a str, Box<dyn FnOnce() -> Result<()>>);
    let steps: Vec<Step> = vec![
        ("extract", {
            let out = out.clone();
            Box::new(move || extract(out, 6, 0))
        }),
        ("ocr", {
            let out = out.clone();
            Box::new(move || ocr_all(out, "gpt-5.6-luna".into(), 0, 0))
        }),
        ("extract-audio", {
            let out = out.clone();
            Box::new(move || {
                if audio_imdb.is_empty() {
                    extract_audio(out, 6, 0, None)
                } else {
                    for imdb in audio_imdb {
                        extract_audio(out.clone(), 1, 0, Some(&imdb))?;
                    }
                    Ok(())
                }
            })
        }),
        ("audio-check", {
            let out = out.clone();
            Box::new(move || audio_check(out, 4, 0, None))
        }),
        ("speech-profiles", {
            let out = out.clone();
            Box::new(move || speech_profiles(out, 6, 0))
        }),
        // TODO: Replace text-sync, sync and vad-sync with the full-transcript
        // alignment already in verbatim::measure/transcript-check. Import raw
        // subtitles before transcription, preserve the disc-track retiming guard,
        // and require verification before exporting sidecars. Then remove reference
        // caches and failed-sync bookkeeping; retain VAD for chunking/clip boundaries.
        // Tradeoff: fewer rescued films and transcription spend on rejected sources.
        ("text-sync", {
            let (out, data_root) = (out.clone(), data_root.clone());
            Box::new(move || text_sync(out, data_root, 4, 0))
        }),
        ("sync", {
            let (out, data_root) = (out.clone(), data_root.clone());
            Box::new(move || sync_all(out, data_root, 4, 0))
        }),
        ("vad-sync", {
            let (out, data_root) = (out.clone(), data_root.clone());
            Box::new(move || vad_sync(out, data_root, 3, 0))
        }),
        ("transcribe", {
            let out = out.clone();
            Box::new(move || {
                if transcribe_imdb.is_empty() {
                    transcribe_all(out, 4, 0, None)
                } else {
                    for imdb in transcribe_imdb {
                        transcribe_all(out.clone(), 1, 0, Some(imdb))?;
                    }
                    Ok(())
                }
            })
        }),
        ("segment", {
            let out = out.clone();
            Box::new(move || segment_all(out, false, 0, None))
        }),
        ("transcript-check", {
            let (out, data_root) = (out.clone(), data_root.clone());
            Box::new(move || transcript_check(out, data_root, 0, None, None, false, 10, 3))
        }),
        ("clips", {
            let out = out.clone();
            Box::new(move || clips(out, 2, 0, None, None, None))
        }),
        ("sidecars", {
            let out = out.clone();
            Box::new(move || export_sidecars(out))
        }),
        (
            "export-yap",
            Box::new(move || export_yap(out, data_root, None, false)),
        ),
    ];

    let mut failed: Vec<&str> = Vec::new();
    for (name, run) in steps {
        println!("\n━━━ {name} ━━━");
        if let Err(e) = run() {
            println!("  ✗ {name} failed: {e:#}");
            failed.push(name);
        }
    }
    if failed.is_empty() {
        Ok(())
    } else {
        bail!("steps failed: {}", failed.join(", "))
    }
}

/// Convert one embedded text track to SRT.
///
/// Subtitle packets are interleaved through the container, so this reads the
/// whole file — it is the slow part, and why it runs in parallel.
fn extract_one(movie: &Movie, out: &std::path::Path) -> Result<usize> {
    let dir = out.join(&movie.imdb_id);
    std::fs::create_dir_all(&dir)?;
    let dest = dir.join("subtitle.srt");
    if dest.exists() {
        let text = std::fs::read_to_string(&dest)?;
        return Ok(movie_subtitles_len(&text));
    }
    let tmp = dir.join("subtitle.srt.tmp");

    // Both paths go through ffmpeg: a sidecar may be ASS/SSA, and even an SRT
    // can carry a byte-order mark or non-UTF-8 encoding that a plain copy would
    // preserve and every later stage would then have to cope with.
    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-v", "error", "-y", "-i"]);
    match &movie.source {
        Source::DiscText { index, .. } => {
            cmd.arg(&movie.path)
                .args(["-map", &format!("0:{index}"), "-f", "srt"]);
        }
        Source::Sidecar { path } => {
            cmd.arg(path).args(["-f", "srt"]);
        }
        other => bail!("{} is not a text source", other.label()),
    }
    let status = cmd.arg(&tmp).status().context("ffmpeg failed to start")?;
    if !status.success() {
        let _ = std::fs::remove_file(&tmp);
        bail!("ffmpeg exited with {status}");
    }
    let text = std::fs::read_to_string(&tmp)?;
    let cues = movie_subtitles_len(&text);
    if cues == 0 {
        let _ = std::fs::remove_file(&tmp);
        bail!("extracted track had no cues");
    }
    std::fs::rename(&tmp, &dest)?;
    match &movie.source {
        Source::Sidecar { path } => write_stamp(&dir, movie, StampSource::File(path)),
        _ => write_stamp(&dir, movie, StampSource::Disc),
    }
    Ok(cues)
}

/// Cue count, as a cheap sanity check that the extraction produced something.
fn movie_subtitles_len(srt: &str) -> usize {
    srt.lines().filter(|l| l.contains("-->")).count()
}

fn extract(out: PathBuf, jobs: usize, limit: usize) -> Result<()> {
    let plan = read_plan(&out)?;
    let mut todo: Vec<Movie> = plan
        .into_iter()
        .filter(|m| matches!(m.source, Source::DiscText { .. } | Source::Sidecar { .. }))
        .filter(|m| {
            !(matches!(m.source, Source::Sidecar { .. })
                && SIDECAR_UNTRUSTED.contains(&m.imdb_id.as_str()))
        })
        .collect();
    if limit > 0 {
        todo.truncate(limit);
    }
    println!("{} movies have a text subtitle already", todo.len());

    let results = parallel(todo, jobs, "extracting", |m| {
        (m.imdb_id.clone(), m.title.clone(), extract_one(m, &out))
    });

    let mut ok = 0usize;
    let mut cues = 0usize;
    for (imdb, title, r) in &results {
        match r {
            Ok(n) => {
                ok += 1;
                cues += n;
            }
            Err(e) => println!("  ✗ {imdb} {}: {e}", truncate(title, 40)),
        }
    }
    println!(
        "\nextracted {ok}/{} subtitles, {cues} cues total",
        results.len()
    );
    println!("into {}", out.display());
    Ok(())
}

/// What happened to one film in an `extract-audio` pass.
enum AudioOutcome {
    Extracted(sync::AudioStreamIdentity),
    Current,
    NoStream,
    Failed(anyhow::Error),
}

/// Pull the original-language audio track out of one film as `audio.opus`.
///
/// The artifact lives in the corpus, not beside the video: it is internal
/// substrate (whisper windows, VAD, future audio work), not part of the media
/// collection. Channels are kept — dialogue lives in the centre channel of a
/// surround mix, and folding it away now would close that door — and the
/// original timeline is preserved, so a timestamp in the opus *is* a timestamp
/// in the film.
fn extract_audio_one(movie: &Movie, dir: &std::path::Path) -> AudioOutcome {
    let codes = library::stream_codes(&movie.original_language);
    let Ok(stream) = sync::original_audio_stream(&movie.path, codes, &rejected_streams(movie, dir))
    else {
        return AudioOutcome::NoStream;
    };
    let identity = match sync::audio_stream_identity(&movie.path, stream) {
        Ok(i) => i,
        Err(e) => return AudioOutcome::Failed(e),
    };
    let current = match film_stamp(movie) {
        Ok(s) => s,
        Err(e) => return AudioOutcome::Failed(e),
    };
    if dir.join("audio.opus").exists() {
        if let Some(stamp) = read_audio_stamp(dir) {
            let recorded = FilmStamp {
                filename: stamp.filename.clone(),
                duration_ms: stamp.duration_ms,
                subtitle: None,
                track: None,
            };
            if recorded.matches(&current) && stamp.stream.same_track(&identity) {
                return AudioOutcome::Current;
            }
        }
    }

    let result = (|| -> Result<()> {
        std::fs::create_dir_all(dir)?;
        let tmp = dir.join("audio.opus.tmp");
        // ~64 kbps per channel is transparent-enough opus for speech work;
        // libopus rejects ffmpeg's "(side)" surround names, so aformat maps
        // each layout onto the nearest one opus can carry.
        let bitrate = (u64::from(identity.channels.max(1)) * 64_000).min(510_000);
        let status = Command::new("ffmpeg")
            .args(["-v", "error", "-y", "-i"])
            .arg(&movie.path)
            .args([
                "-map",
                &format!("0:a:{stream}"),
                "-vn",
                "-sn",
                // aresample=async=1 locks the output timeline to the film's.
                // Some source tracks drop packets across silence; ffmpeg then
                // concatenates the audio and every later sample slides earlier,
                // so the opus ends up shorter than the film and progressively
                // out of sync (Run Lola Run: 165s short, 19s off by minute 5).
                // async=1 fills each gap with silence and first_pts=0 anchors
                // the start, so a timestamp in the opus is a timestamp in the
                // film — which every downstream stage assumes. Found 2026-09-01
                // by the subtitle-vs-transcript audit.
                // aresample must run *before* aformat: a track can carry an
                // unknown channel layout (God of Cookery's 6ch AC3), which
                // aformat cannot name, and feeding that into libopus fails the
                // whole extraction. aresample normalizes it first, then aformat
                // maps to an opus-legal name.
                "-af",
                "aresample=async=1:first_pts=0,aformat=channel_layouts=7.1|6.1|5.1|5.0|quad|3.0|stereo|mono",
                "-c:a",
                "libopus",
                "-b:a",
                &bitrate.to_string(),
                "-f",
                "ogg",
            ])
            .arg(&tmp)
            .status()
            .context("ffmpeg failed to start")?;
        if !status.success() {
            let _ = std::fs::remove_file(&tmp);
            bail!("ffmpeg exited with {status}");
        }
        std::fs::rename(&tmp, dir.join("audio.opus"))?;
        let stamp = AudioStamp {
            filename: current.filename.clone(),
            duration_ms: current.duration_ms,
            stream: identity.clone(),
        };
        std::fs::write(dir.join("audio.json"), serde_json::to_vec_pretty(&stamp)?)?;
        write_stamp(dir, movie, StampSource::Keep);
        Ok(())
    })();
    match result {
        Ok(()) => AudioOutcome::Extracted(identity),
        Err(e) => AudioOutcome::Failed(e),
    }
}

fn extract_audio(out: PathBuf, jobs: usize, limit: usize, imdb: Option<&str>) -> Result<()> {
    let plan = read_plan(&out)?;
    let mut todo: Vec<Movie> = plan
        .into_iter()
        .filter(|m| imdb.is_none_or(|id| m.imdb_id == id))
        .filter(|m| m.path.exists())
        .collect();
    if limit > 0 {
        todo.truncate(limit);
    }
    println!("{} films to check for extracted audio", todo.len());

    let results = parallel(todo, jobs, "extracting audio", |m| {
        let outcome = extract_audio_one(m, &out.join(&m.imdb_id));
        if let AudioOutcome::Extracted(id) = &outcome {
            println!(
                "  {} ✓ {} {}ch → opus",
                truncate(&m.title, 40),
                id.codec,
                id.channels
            );
        }
        (m.imdb_id.clone(), m.title.clone(), outcome)
    });

    let mut extracted = 0usize;
    let mut current = 0usize;
    let mut no_stream = 0usize;
    for (imdb, title, outcome) in &results {
        match outcome {
            AudioOutcome::Extracted(_) => extracted += 1,
            AudioOutcome::Current => current += 1,
            AudioOutcome::NoStream => no_stream += 1,
            AudioOutcome::Failed(e) => println!("  ✗ {imdb} {}: {e:#}", truncate(title, 40)),
        }
    }
    println!(
        "\n{extracted} tracks extracted, {current} already current, {no_stream} with no original-language stream"
    );
    Ok(())
}

/// The listener's verdict on the track now in `audio.opus`.
#[derive(Serialize, Deserialize)]
struct AudioCheck {
    model: String,
    expected: String,
    filename: String,
    stream: sync::AudioStreamIdentity,
    verdict: audio_check::Verdict,
}

/// Whether the extracted track has already been heard by the current model,
/// judged against the variety the course teaches now. A rejected track is
/// evicted on the spot, so a verdict that still matches the artifact is an
/// acceptance.
///
/// `expected` is part of the freshness test because it is part of the
/// question: a track heard as "Portuguese" was never asked whether it was
/// Brazilian, so narrowing `audio_check::expected_language` re-opens exactly
/// the films whose verdict answered the older, broader question.
fn audio_checked(dir: &std::path::Path, expected: &str) -> bool {
    let Some(stamp) = read_audio_stamp(dir) else {
        return false;
    };
    std::fs::read(dir.join("audio-check.json"))
        .ok()
        .and_then(|b| serde_json::from_slice::<AudioCheck>(&b).ok())
        .is_some_and(|c| {
            c.model == audio_check::MODEL
                && c.expected == expected
                && c.filename == stamp.filename
                && c.stream == stamp.stream
        })
}

/// Evict the rejected track and everything the pipeline derived from it,
/// and record the rejection so the extractor passes the stream over.
fn reject_audio(
    dir: &std::path::Path,
    stamp: AudioStamp,
    verdict: audio_check::Verdict,
) -> Result<()> {
    for name in [
        "audio.opus",
        "audio.json",
        "audio-check.json",
        "speech-profile-16ms.f32",
        "transcript.jsonl",
        "clips.jsonl",
    ] {
        let _ = std::fs::remove_file(dir.join(name));
    }
    let _ = std::fs::remove_file(subtitle_corpus::verbatim::report_path(dir));
    let mut rejected = read_rejected(dir);
    rejected.push(RejectedStream {
        filename: stamp.filename,
        stream: stamp.stream,
        verdict,
    });
    std::fs::write(
        dir.join("audio-rejected.json"),
        serde_json::to_vec_pretty(&rejected)?,
    )?;
    Ok(())
}

/// What listening to one film came to: the tracks turned down along the way,
/// and the one accepted, if any survived.
struct Listened {
    rejected: Vec<audio_check::Verdict>,
    accepted: Option<audio_check::Verdict>,
}

async fn audio_check_one(
    client: &google_speech::gemini::GeminiClient,
    movie: &Movie,
    dir: &std::path::Path,
) -> Result<Listened> {
    let expected = audio_check::expected_language(&movie.original_language);
    let mut rejected = Vec::new();
    loop {
        let stamp = read_audio_stamp(dir).context("audio.json missing")?;
        let mut verdict = None;
        for points in [audio_check::SAMPLE_POINTS, audio_check::RETRY_POINTS] {
            let samples = {
                let audio = dir.join("audio.opus");
                let duration = stamp.duration_ms;
                tokio::task::spawn_blocking(move || audio_check::samples(&audio, duration, points))
                    .await??
            };
            let heard = audio_check::judge(client, expected, &samples).await?;
            let quiet = !heard.enough_dialogue;
            verdict = Some(heard);
            if !quiet {
                break;
            }
        }
        let verdict = verdict.expect("two sample sets were tried");
        let check = AudioCheck {
            model: audio_check::MODEL.to_string(),
            expected: expected.to_string(),
            filename: stamp.filename.clone(),
            stream: stamp.stream.clone(),
            verdict: verdict.clone(),
        };
        std::fs::write(
            dir.join("audio-check.json"),
            serde_json::to_vec_pretty(&check)?,
        )?;
        if verdict.accepted() {
            return Ok(Listened {
                rejected,
                accepted: Some(verdict),
            });
        }
        println!(
            "  {} ✗ stream {}: {} — {}",
            truncate(&movie.title, 34),
            stamp.stream.stream_index,
            if verdict.commentary {
                "commentary"
            } else {
                &verdict.spoken_language
            },
            verdict.notes
        );
        reject_audio(dir, stamp, verdict.clone())?;
        rejected.push(verdict);
        // The extractor sees the rejection and reaches for the next candidate.
        let outcome = {
            let (movie, dir) = (movie.clone(), dir.to_path_buf());
            tokio::task::spawn_blocking(move || extract_audio_one(&movie, &dir)).await?
        };
        match outcome {
            AudioOutcome::Extracted(id) => println!(
                "  {} → trying stream {} ({} {}ch)",
                truncate(&movie.title, 34),
                id.stream_index,
                id.codec,
                id.channels
            ),
            AudioOutcome::NoStream => {
                return Ok(Listened {
                    rejected,
                    accepted: None,
                })
            }
            AudioOutcome::Current => bail!("the extractor re-chose a rejected stream"),
            AudioOutcome::Failed(e) => return Err(e),
        }
    }
}

/// Listen to every extracted track not yet judged; see `Command_::AudioCheck`.
#[tokio::main]
async fn audio_check(out: PathBuf, jobs: usize, limit: usize, imdb: Option<String>) -> Result<()> {
    use futures::stream::StreamExt;
    use std::sync::Arc;

    let key = std::env::var("GEMINI_API_KEY").context("GEMINI_API_KEY not set")?;
    let client = Arc::new(google_speech::gemini::GeminiClient::new(key));
    let plan = read_plan(&out)?;
    let mut queue: Vec<Movie> = plan
        .into_iter()
        .filter(|m| imdb.as_deref().is_none_or(|id| m.imdb_id == id))
        .filter(|m| {
            let dir = out.join(&m.imdb_id);
            extracted_audio(m, &dir).is_some()
                && !audio_checked(&dir, audio_check::expected_language(&m.original_language))
        })
        .collect();
    if limit > 0 {
        queue.truncate(limit);
    }
    let total = queue.len();
    println!(
        "{total} extracted tracks to listen to, model {}",
        audio_check::MODEL
    );
    if total == 0 {
        return Ok(());
    }

    let out = Arc::new(out);
    let progress = AtomicUsize::new(0);
    let outcomes: Vec<Result<Listened>> = futures::stream::iter(queue)
        .map(|movie| {
            let (client, out) = (Arc::clone(&client), Arc::clone(&out));
            let progress = &progress;
            async move {
                let outcome = audio_check_one(&client, &movie, &out.join(&movie.imdb_id)).await;
                let n = progress.fetch_add(1, Ordering::Relaxed) + 1;
                let title = truncate(&movie.title, 34);
                match &outcome {
                    Ok(Listened {
                        accepted: Some(v),
                        rejected,
                    }) => println!(
                        "[{n}/{total}] {title} ✓ {} ({} confidence){}{}",
                        v.spoken_language,
                        v.confidence,
                        if v.enough_dialogue {
                            ""
                        } else {
                            " — too little dialogue in six windows to judge, accepted"
                        },
                        match rejected.len() {
                            0 => String::new(),
                            k => format!(", after rejecting {k}"),
                        }
                    ),
                    Ok(Listened { accepted: None, .. }) => {
                        println!("[{n}/{total}] {title} ✗ no usable track on the disc")
                    }
                    Err(e) => println!("[{n}/{total}] {title} ✗ {e:#}"),
                }
                outcome
            }
        })
        .buffer_unordered(jobs.max(1))
        .collect()
        .await;

    let accepted = outcomes
        .iter()
        .filter(|o| matches!(o, Ok(l) if l.accepted.is_some()))
        .count();
    let exhausted = outcomes
        .iter()
        .filter(|o| matches!(o, Ok(l) if l.accepted.is_none()))
        .count();
    let rejected: usize = outcomes
        .iter()
        .filter_map(|o| o.as_ref().ok())
        .map(|l| l.rejected.len())
        .sum();
    println!(
        "\n{accepted} tracks accepted, {rejected} rejected, {exhausted} films left with no usable track, {} failed",
        outcomes.iter().filter(|o| o.is_err()).count()
    );
    Ok(())
}

/// Build the fine speech profile for every film with extracted audio; see
/// `Command_::SpeechProfiles`.
fn speech_profiles(out: PathBuf, jobs: usize, limit: usize) -> Result<()> {
    let plan = read_plan(&out)?;
    let mut todo: Vec<Movie> = plan
        .into_iter()
        .filter(|m| {
            let dir = out.join(&m.imdb_id);
            extracted_audio(m, &dir).is_some()
                && vad::read_profile(&dir.join("speech-profile-16ms.f32")).is_none()
        })
        .collect();
    if limit > 0 {
        todo.truncate(limit);
    }
    println!(
        "{} films with audio and no 16 ms speech profile",
        todo.len()
    );
    if todo.is_empty() {
        return Ok(());
    }

    let results = parallel(todo, jobs, "profiling speech", |m| {
        let dir = out.join(&m.imdb_id);
        let r = fine_speech_profile(m, &dir).map(|p| p.len());
        if let Ok(frames) = &r {
            // The retired coarse file has nothing the fine one lacks.
            let _ = std::fs::remove_file(dir.join("speech-profile.f32"));
            println!(
                "  {} ✓ {:.0} min profiled",
                truncate(&m.title, 40),
                *frames as f64 * vad::FRAME as f64 / 16_000.0 / 60.0
            );
        }
        (m.imdb_id.clone(), m.title.clone(), r)
    });
    let failed = results.iter().filter(|(_, _, r)| r.is_err()).count();
    for (imdb, title, r) in &results {
        if let Err(e) = r {
            println!("  ✗ {imdb} {}: {e:#}", truncate(title, 40));
        }
    }
    println!(
        "\n{} profiles written, {failed} failed",
        results.len() - failed
    );
    Ok(())
}

/// Warm the sentence-segmentation cache for every model-segmented film; see
/// `Command_::Segment`.
#[tokio::main]
async fn segment_all(out: PathBuf, all: bool, limit: usize, imdb: Option<String>) -> Result<()> {
    use movie_subtitles::llm_segment;
    let films: Vec<Movie> = read_plan(&out)?
        .into_iter()
        .filter(|m| imdb.as_ref().is_none_or(|id| *id == m.imdb_id))
        .filter(|m| all || out.join(&m.imdb_id).join("transcript.jsonl").exists())
        .collect();
    let mut loaded = subtitle_corpus::clips::llm_tracks(&out, &films)?;
    if limit > 0 {
        loaded.truncate(limit);
    }
    let todo: Vec<(&Movie, language_utils::Language)> = loaded
        .iter()
        .map(|(i, language, _)| (&films[*i], *language))
        .collect();
    let lines: Vec<&Vec<movie_subtitles::SubtitleLine>> =
        loaded.iter().map(|(_, _, lines)| lines).collect();
    let tracks: Vec<(&[movie_subtitles::SubtitleLine], language_utils::Language)> = loaded
        .iter()
        .map(|(_, language, lines)| (lines.as_slice(), *language))
        .collect();
    let cues: usize = tracks.iter().map(|(t, _)| t.len()).sum();
    println!(
        "{} model-segmented films with a subtitle, {cues} cues, model {}",
        todo.len(),
        llm_segment::MODEL
    );
    if todo.is_empty() {
        return Ok(());
    }
    let client = llm_segment::client()?;
    let (splits, report) =
        llm_segment::split_tracks(&client, &tracks, llm_segment::print_progress()).await?;
    for (((m, language), lines), splits) in todo.iter().zip(&lines).zip(&splits) {
        let keyed = movie_subtitles::sentences::keyed_sentences_from_splits(
            lines.as_slice(),
            splits,
            *language,
            &m.imdb_id,
        );
        let worthy = keyed.iter().filter(|k| k.course_worthy).count();
        println!(
            "  {} ✓ {} cues → {} sentences ({worthy} course-worthy)",
            truncate(&m.title, 40),
            lines.len(),
            keyed.len()
        );
        // Asked for one film: show its opening so the cuts can be eyeballed.
        if imdb.is_some() {
            for k in keyed.iter().take(60) {
                let mark = if k.course_worthy { "keep" } else { "drop" };
                println!("      [{mark}] {}", k.sentence);
            }
        }
    }
    println!(
        "\n{} cues, {} put to the model, {} fell back to per-cue",
        report.cues, report.asked, report.fallbacks
    );
    Ok(())
}

/// Read every bitmap track in the library back into text.
///
/// Resumable at movie granularity by the finished `subtitle.srt`, and at cue
/// granularity by tysm's response cache — an interrupted run re-reads nothing
/// it already paid for.
#[tokio::main]
async fn ocr_all(out: PathBuf, model: String, films_in_flight: usize, limit: usize) -> Result<()> {
    use futures::stream::StreamExt;
    use std::sync::Arc;

    let plan = read_plan(&out)?;
    let mut queue: Vec<Movie> = plan
        .into_iter()
        .filter(|m| matches!(m.source, Source::DiscBitmap { .. }))
        .filter(|m| !out.join(&m.imdb_id).join("subtitle.srt").exists())
        .collect();
    if limit > 0 {
        queue.truncate(limit);
    }
    let total = queue.len();
    let films_in_flight = if films_in_flight == 0 {
        total.max(1)
    } else {
        films_in_flight
    };
    println!("{total} movies still need OCR, {films_in_flight} batches in flight");

    let client = Arc::new(ocr::client(&model)?);
    let fallback_client = Arc::new(ocr::client(ocr::FALLBACK_MODEL)?);
    let out = Arc::new(out);
    let progress = AtomicUsize::new(0);

    // Only the verdict per film is needed; the name is printed as it lands.
    let results: Vec<Result<(usize, usize, usize, usize)>> =
        futures::stream::iter(queue.into_iter())
        .map(|movie| {
            let client = Arc::clone(&client);
            let fallback_client = Arc::clone(&fallback_client);
            let out = Arc::clone(&out);
            let progress = &progress;
            async move {
                let outcome = ocr_one(&client, &fallback_client, &movie, &out).await;
                let n = progress.fetch_add(1, Ordering::Relaxed) + 1;
                match &outcome {
                    Ok((lines, cues, retried, rescued)) => println!(
                        "[{n}/{total}] {} ✓ {lines} lines (of {cues} cues; {retried} retried, {rescued} rescued)",
                        truncate(&movie.title, 42)
                    ),
                    Err(e) => println!("[{n}/{total}] {} ✗ {e}", truncate(&movie.title, 42)),
                }
                outcome
            }
        })
        .buffer_unordered(films_in_flight.max(1))
        .collect()
        .await;

    let done = results.iter().filter(|r| r.is_ok()).count();
    println!("\n{done} transcribed, {} failed", results.len() - done);
    if let Some(cost) = client.cost() {
        println!("spent ${cost:.2}");
    }
    Ok(())
}

/// OCR one film: extract its bitmap track, read every cue in a single batch,
/// and write the SRT with the disc's own timings.
async fn ocr_one(
    client: &tysm::chat_completions::ChatClient,
    fallback_client: &tysm::chat_completions::ChatClient,
    movie: &Movie,
    out: &std::path::Path,
) -> Result<(usize, usize, usize, usize)> {
    let Source::DiscBitmap { index, codec } = &movie.source else {
        bail!("not a bitmap source");
    };
    // A film seen for the first time has no corpus directory yet; nothing
    // upstream of OCR creates one for a bitmap source.
    std::fs::create_dir_all(out.join(&movie.imdb_id))?;
    let sup = ocr::sup_path(out, &movie.imdb_id);

    // Reading a whole film blocks its thread for minutes; keep it off the
    // async runtime so other films' batches keep progressing.
    let images = if codec == "dvd_subtitle" {
        let (video, index) = (movie.path.clone(), *index);
        tokio::task::spawn_blocking(move || ocr::vobsub_cue_images(&video, index))
            .await?
            .context("decode")?
    } else {
        let (video, index, sup_for_task) = (movie.path.clone(), *index, sup.clone());
        tokio::task::spawn_blocking(move || ocr::extract_sup(&video, index, &sup_for_task))
            .await?
            .context("extract")?;
        ocr::cue_images(&sup).context("decode")?
    };
    if images.is_empty() {
        bail!("no text cues in the bitmap track");
    }

    // One batch per film. Half the price of live requests, and a film's cues are
    // a natural unit: wanted together, finished together, and a failed batch
    // costs exactly one film's retry. A library-wide batch is not on offer
    // anyway — a film's cue images run to tens of MB and a job's input file
    // is capped at 200 MB — so the caller submits every film at once instead.
    // tysm consults the same response cache first, so cues already read are
    // never resubmitted.
    let requests: Vec<_> = images
        .iter()
        .map(|img| ocr::messages_for(&img.png))
        .collect();
    let results = client
        .batch_chat_with_messages::<ocr::Transcription>(requests, |_| {})
        .await
        .map_err(|e| anyhow::anyhow!("batch: {e}"))?;

    let ocr::ReadLines {
        lines,
        retried,
        rescued,
    } = ocr::read_lines(fallback_client, &images, results).await;

    if lines.is_empty() {
        bail!("no text recovered from {} cues", images.len());
    }

    std::fs::write(
        out.join(&movie.imdb_id).join("subtitle.srt"),
        ocr::to_srt(&lines),
    )?;
    write_stamp(&out.join(&movie.imdb_id), movie, StampSource::Disc);
    // The .sup is large and fully derived from the film; the SRT replaces it.
    let _ = std::fs::remove_file(&sup);
    Ok((lines.len(), images.len(), retried, rescued))
}

#[tokio::main]
async fn sync_all(
    out: PathBuf,
    data_root: PathBuf,
    films_in_flight: usize,
    limit: usize,
) -> Result<()> {
    use futures::stream::StreamExt;
    use std::sync::Arc;

    // Fail on missing credentials before processing any films.
    let client = Arc::new(CloudflareWhisper::from_env(reqwest::Client::new())?);

    let plan = read_plan(&out)?;
    let mut queue: Vec<(Movie, PathBuf)> = Vec::new();
    let mut parked = 0usize;
    for movie in plan {
        let dir = out.join(&movie.imdb_id);
        if dir.join("subtitle.srt").exists() {
            continue;
        }
        if let Some(raw) = subtitle_source(&movie, &dir, &data_root) {
            if movie.path.exists() {
                if sync_already_failed(&movie, &dir, &raw) {
                    parked += 1;
                } else {
                    queue.push((movie, raw));
                }
            }
        }
    }
    if limit > 0 {
        queue.truncate(limit);
    }
    let total = queue.len();
    println!(
        "{total} films have a subtitle to align, {films_in_flight} at a time{}",
        parked_note(parked)
    );

    let out = Arc::new(out);
    let progress = AtomicUsize::new(0);

    let results: Vec<bool> = futures::stream::iter(queue.into_iter())
        .map(|(movie, raw)| {
            let client = Arc::clone(&client);
            let out = Arc::clone(&out);
            let progress = &progress;
            async move {
                let outcome = sync_one(&client, &movie, &raw, &out).await;
                let n = progress.fetch_add(1, Ordering::Relaxed) + 1;
                match &outcome {
                    Ok(a) => println!(
                        "[{n}/{total}] {} ✓ {:+.2}s{} ({}/{} anchors, worst {:.0}ms)",
                        truncate(&movie.title, 34),
                        a.offset_ms / 1000.0,
                        if (a.rate - 1.0).abs() > 1e-5 {
                            format!(", rate {:.5}", a.rate)
                        } else {
                            String::new()
                        },
                        a.anchors_used,
                        a.anchors_seen,
                        a.worst_residual_ms
                    ),
                    Err(e) => println!("[{n}/{total}] {} ✗ {e}", truncate(&movie.title, 34)),
                }
                outcome.is_ok()
            }
        })
        .buffer_unordered(films_in_flight.max(1))
        .collect()
        .await;

    let done = results.iter().filter(|ok| **ok).count();
    println!("\n{done} aligned, {} left unaligned", results.len() - done);
    Ok(())
}

/// Align one film: transcribe a few windows, match lines, fit, write.
async fn sync_one(
    client: &CloudflareWhisper,
    movie: &Movie,
    raw_srt: &std::path::Path,
    out: &std::path::Path,
) -> Result<sync::Alignment> {
    let cues = sync::parse_cues(&std::fs::read_to_string(raw_srt)?);
    if cues.is_empty() {
        bail!("subtitle has no cues");
    }
    let (media, stream) = audio_source(movie, &out.join(&movie.imdb_id))?;
    let duration = sync::duration_ms(&movie.path)?;
    let language = library::course_dir(&movie.original_language)
        .and_then(Language::from_code)
        .map(whisper::language_code)
        .unwrap_or("en");

    // Spread the windows across the body of the film. Openings are logos and
    // credits, endings are credits again — neither carries much dialogue, and
    // anchors clustered at one end cannot reveal a rate.
    let mut heard = Vec::new();
    for at in sync::choose_windows(&cues, duration, 5, 60) {
        match sync::transcribe_window(client, &media, stream, at, 60, language).await {
            Ok(words) => heard.extend(words),
            // One refused window is survivable; the fit needs several anyway.
            Err(e) => eprintln!("      window at {}s failed: {e}", at / 1000),
        }
    }
    if heard.is_empty() {
        bail!("no audio could be transcribed");
    }

    // A subtitle that covers only part of the film cannot be placed reliably:
    // there is no way to tell a correctly-timed first-half subtitle from the
    // same file shifted onto the second half, and the anchors are too few to
    // arbitrate. Lust, Caution covered 46 of 158 minutes — a "CD1" subtitle —
    // and was confidently shifted 104 minutes to the end of the film with 88%
    // of its anchors agreeing. Films with genuinely long silent stretches
    // (2001 spans 61%) still clear this.
    let span = cues.iter().map(|c| c.end_ms).max().unwrap_or(0)
        - cues.iter().map(|c| c.start_ms).min().unwrap_or(0);
    if duration > 0 && (span as f64) < 0.5 * duration as f64 {
        bail!(
            "subtitle covers only {:.0}% of the film — partial, cannot be placed",
            span as f64 / duration as f64 * 100.0
        );
    }

    let anchors = sync::find_anchors(&cues, &heard, 4);
    let Some(alignment) = sync::fit(&anchors, 3000.0) else {
        bail!("only {} anchors, too few to trust", anchors.len());
    };
    // A poor fit means the anchors disagree about what the shift is, which is
    // how a wrong match or a different cut of the film shows up. Writing a
    // plausible-looking wrong alignment is worse than writing none.
    let agreement = alignment.anchors_used as f64 / alignment.anchors_seen.max(1) as f64;
    if agreement < 0.35 {
        bail!(
            "only {:.0}% of {} anchors agree on the shift",
            agreement * 100.0,
            alignment.anchors_seen
        );
    }
    if alignment.worst_residual_ms > 1500.0 {
        bail!(
            "anchors disagree by {:.0}ms (limit {:.0})",
            alignment.worst_residual_ms,
            1500.0
        );
    }

    // The aligned subtitle must fit the film it claims to describe. A fit can
    // be internally consistent and still absurd — anchors agreeing on a shift
    // that pushes the last line past the end credits — and this catches that
    // for the price of one comparison.
    let last = cues.iter().map(|c| c.end_ms).max().unwrap_or(0);
    let aligned_end = alignment.apply(last);
    if aligned_end > duration + 120_000 || alignment.apply(cues[0].start_ms) < -60_000 {
        bail!(
            "alignment puts the subtitle at {:.1}–{:.1} min of a {:.1} min film",
            alignment.apply(cues[0].start_ms) as f64 / 60_000.0,
            aligned_end as f64 / 60_000.0,
            duration as f64 / 60_000.0
        );
    }

    let shifted: Vec<sync::Cue> = cues
        .iter()
        .map(|c| sync::Cue {
            start_ms: alignment.apply(c.start_ms),
            end_ms: alignment.apply(c.end_ms),
            text: c.text.clone(),
        })
        .collect();
    let dir = out.join(&movie.imdb_id);
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join("subtitle.srt"), sync::write_cues(&shifted))?;
    write_stamp(&dir, movie, StampSource::File(raw_srt));
    Ok(alignment)
}

/// Transcribe one film in full and write it beside the subtitle.
/// Does this film need transcribing — because it has none, or because the one
/// it has was made under settings we no longer use?
///
/// Reuse the finished artifact while its settings and audio identity match.
/// This also preserves accepted partial transcripts without retrying their gaps.
fn transcript_is_stale(movie: &Movie, dir: &std::path::Path) -> bool {
    let Some(stored) = transcript::stored_provenance(&dir.join("transcript.jsonl")) else {
        // No file, or one written before provenance was recorded.
        return true;
    };
    let audio = read_audio_stamp(dir).map(|s| s.stream);
    match library::course_dir(&movie.original_language)
        .and_then(Language::from_code)
        .map(whisper::language_code)
        .map(|language| transcript::provenance(language, audio))
    {
        Some(Ok(current)) => stored.is_stale_against(&current),
        // No language, or no provenance to compare against: there is nothing
        // this run could produce, so leave what is there alone.
        _ => false,
    }
}

async fn transcribe_one(
    http: &reqwest::Client,
    account: &transcript::ScribeAccount,
    store: &osmo::Store,
    movie: &Movie,
    out: &std::path::Path,
) -> Result<usize> {
    let dir = out.join(&movie.imdb_id);
    let language = library::course_dir(&movie.original_language)
        .and_then(Language::from_code)
        .map(whisper::language_code)
        .context("no Whisper language for this film's original language")?;
    let audio = extracted_audio(movie, &dir).context("no extracted audio")?;

    // The profile has to come from the whole film in one pass — see the
    // module docs on earshot's recurrence — which `cached_speech_profile`
    // already guarantees, and caches for everyone else.
    let profile = cached_speech_profile(movie, &dir)?;
    // The audio's own duration, not the video's: it is what we slice.
    let film_ms = sync::duration_ms(&audio)?;

    let transcript = transcript::transcribe_film(
        http,
        account,
        store,
        &audio,
        &profile,
        film_ms,
        transcript::provenance(language, read_audio_stamp(&dir).map(|s| s.stream))?,
    )
    .await?;
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join("transcript.jsonl"), transcript.to_jsonl()?)?;
    write_stamp(&dir, movie, StampSource::Keep);
    Ok(transcript.words.len())
}

#[tokio::main]
async fn transcribe_all(
    out: PathBuf,
    films_in_flight: usize,
    limit: usize,
    imdb: Option<String>,
) -> Result<()> {
    use futures::stream::StreamExt;
    use std::sync::Arc;

    // Not the Whisper credentials the syncers use: transcription runs against
    // ElevenLabs. Fail here rather than once per chunk.
    let account = Arc::new(transcript::ScribeAccount::from_env()?);

    let plan = read_plan(&out)?;
    let mut queue: Vec<Movie> = plan
        .into_iter()
        .filter(|m| imdb.as_deref().is_none_or(|id| m.imdb_id == id))
        .filter(|m| out.join(&m.imdb_id).join("subtitle.srt").exists())
        .filter(|m| transcript_is_stale(m, &out.join(&m.imdb_id)))
        .filter(|m| extracted_audio(m, &out.join(&m.imdb_id)).is_some())
        .collect();
    if limit > 0 {
        queue.truncate(limit);
    }
    let total = queue.len();
    println!("{total} films to transcribe in full");
    if total == 0 {
        return Ok(());
    }

    // Opened only once there is work: this is the store generate-data fills,
    // 10M keys and 43GB of it, and opening it costs tens of seconds. A
    // refresh with every film already transcribed should cost nothing at all.
    // Sharing it is the point — a transcript outlives eviction of the
    // artifact and rides the R2 mirror to other machines.
    let store = Arc::new(osmo::Store::open("./.cache"));
    let http = Arc::new(reqwest::Client::new());
    let out = Arc::new(out);
    let progress = AtomicUsize::new(0);

    let done: Vec<bool> = futures::stream::iter(queue.into_iter())
        .map(|movie| {
            let http = Arc::clone(&http);
            let account = Arc::clone(&account);
            let store = Arc::clone(&store);
            let out = Arc::clone(&out);
            let progress = &progress;
            async move {
                let outcome = transcribe_one(&http, &account, &store, &movie, &out).await;
                let n = progress.fetch_add(1, Ordering::Relaxed) + 1;
                match &outcome {
                    Ok(words) => println!(
                        "[{n}/{total}] {} ✓ {words} words",
                        truncate(&movie.title, 34)
                    ),
                    Err(e) => println!("[{n}/{total}] {} ✗ {e:#}", truncate(&movie.title, 34)),
                }
                outcome.is_ok()
            }
        })
        .buffer_unordered(films_in_flight.max(1))
        .collect()
        .await;

    println!(
        "\n{} transcribed, {} failed",
        done.iter().filter(|ok| **ok).count(),
        done.iter().filter(|ok| !**ok).count()
    );
    Ok(())
}

/// Judge every transcribed film's subtitle against its transcript, re-time
/// the skewed ones and replace the paraphrased ones where a better
/// candidate can be found. See `Command_::TranscriptCheck`.
#[allow(clippy::too_many_arguments)]
#[tokio::main]
async fn transcript_check(
    out: PathBuf,
    data_root: PathBuf,
    limit: usize,
    imdb: Option<String>,
    min_verbatim: Option<f64>,
    dry_run: bool,
    max_downloads: usize,
    max_candidates: usize,
) -> Result<()> {
    use subtitle_corpus::verbatim::{self, Verdict};

    let plan = read_plan(&out)?;
    let mut queue: Vec<Movie> = plan
        .into_iter()
        .filter(|m| imdb.as_deref().is_none_or(|id| m.imdb_id == id))
        .filter(|m| {
            let dir = out.join(&m.imdb_id);
            dir.join("subtitle.srt").exists() && dir.join("transcript.jsonl").exists()
        })
        .collect();
    // The measure is the clip mapper's own placement test, calibrated on
    // the languages it maps; for the rest it has no ground truth and
    // `clips` would not act on the verdict anyway.
    let ungated = queue.len();
    queue.retain(|m| {
        library::course_dir(&m.original_language).is_some_and(subtitle_corpus::clips::maps)
    });
    let ungated = ungated - queue.len();
    if limit > 0 {
        queue.truncate(limit);
    }
    let total = queue.len();
    println!("{total} transcribed films to judge ({ungated} in languages without a phoneme gate left alone)");
    subtitle_corpus::clips::warm_segmentation(&out, &queue).await?;

    let client = if !dry_run && max_downloads > 0 {
        match opensubtitles_client().await {
            Ok(c) => Some(c),
            Err(e) => {
                println!("  OpenSubtitles unavailable, local candidates only: {e:#}");
                None
            }
        }
    } else {
        None
    };
    let mut downloads_left = if client.is_some() { max_downloads } else { 0 };

    let mut counts: std::collections::BTreeMap<&str, usize> = Default::default();
    let (mut retimed, mut adopted, mut failed) = (0usize, 0usize, 0usize);
    for (n, movie) in queue.iter().enumerate() {
        let n = n + 1;
        let dir = out.join(&movie.imdb_id);
        let title = truncate(&movie.title, 34);
        let (Some(code), Some(language)) = (
            library::course_dir(&movie.original_language),
            library::course_dir(&movie.original_language)
                .and_then(language_utils::Language::from_code),
        ) else {
            println!("[{n}/{total}] {title} ✗ unmapped language");
            failed += 1;
            continue;
        };
        let min_verbatim = min_verbatim.unwrap_or_else(|| verbatim::min_fraction(code));
        let mut report = match verbatim::check(&dir, language, code, min_verbatim).await {
            Ok(r) => r,
            Err(e) => {
                println!("[{n}/{total}] {title} ✗ {e:#}");
                failed += 1;
                continue;
            }
        };
        let mut note = String::new();
        let disc = matches!(
            movie.source,
            Source::DiscText { .. } | Source::DiscBitmap { .. }
        );
        if report.measure.verdict == Verdict::Skewed && !dry_run {
            if disc {
                // Authored to this very file: if it reads as skewed, the
                // transcript's clock (the audio extraction) is what moved.
                note.push_str(
                    " — disc track on another clock: audio extraction suspect, not re-timed",
                );
            } else if let Some(fit) = report.measure.aligned.as_ref() {
                // The verdict already vouches for the fit: the sentences
                // place under it. A worst-anchor residual is no extra test
                // — PAL-rate fits carry 1.5–2 s tails on hundreds of good
                // anchors and still place 55–65% of the film.
                let srt = std::fs::read_to_string(dir.join("subtitle.srt"))?;
                std::fs::write(dir.join("subtitle.srt"), verbatim::retime(&srt, fit))?;
                write_stamp(&dir, movie, StampSource::Keep);
                let _ = std::fs::remove_file(subtitle_corpus::clips::clips_path(&dir));
                note.push_str(&format!(" — re-timed {:+.1}s", fit.offset_ms / 1000.0));
                retimed += 1;
                report = verbatim::check(&dir, language, code, min_verbatim).await?;
            }
        }
        // "Empty" belongs here too: a forced track under a title the
        // classifier did not recognise reads as a film with no dialogue,
        // and the alternatives are the only way out (Day for Night, 8
        // eligible cues from "For non-French dialogue").
        if matches!(
            report.measure.verdict,
            Verdict::Paraphrase | Verdict::Skewed | Verdict::Empty
        ) && !dry_run
        {
            println!(
                "[{n}/{total}] {title}: {} — trying other subtitles",
                verbatim::describe(&report.measure)
            );
            match adopt_candidate(
                movie,
                &dir,
                &data_root,
                language,
                code,
                min_verbatim,
                report.measure.best_placed(),
                client.as_ref(),
                &mut downloads_left,
                max_candidates,
            )
            .await
            {
                Ok(Some(label)) => {
                    adopted += 1;
                    note.push_str(&format!(" — adopted {label}"));
                    report = verbatim::check(&dir, language, code, min_verbatim).await?;
                }
                Ok(None) => note.push_str(" — no better candidate"),
                Err(e) => note.push_str(&format!(" — candidates: {e:#}")),
            }
        }
        *counts.entry(report.measure.verdict.label()).or_default() += 1;
        let mark = if report.measure.verdict == Verdict::Verbatim {
            "✓"
        } else {
            "✗"
        };
        println!(
            "[{n}/{total}] {title} {mark} {} [{}]{note}",
            verbatim::describe(&report.measure),
            movie.source.label()
        );
    }

    println!();
    for (verdict, n) in &counts {
        println!("  {verdict:12}{n:>5}");
    }
    println!(
        "{retimed} re-timed, {adopted} replaced by a better subtitle, {failed} could not be judged"
    );
    if let Some(n) = counts.get("paraphrase").filter(|n| **n > 0) {
        println!(
            "{n} films keep a subtitle that is not what is said; `clips` skips them. \
             Re-run with --max-downloads to try more OpenSubtitles candidates."
        );
    }
    Ok(())
}

async fn opensubtitles_client() -> Result<opensubtitles_downloader::OpenSubtitlesClient> {
    let api_key =
        std::env::var("OPENSUBTITLES_API_KEY").context("OPENSUBTITLES_API_KEY not set")?;
    let mut client = opensubtitles_downloader::OpenSubtitlesClient::new(api_key);
    if let (Ok(user), Ok(password)) = (
        std::env::var("OPENSUBTITLES_USERNAME"),
        std::env::var("OPENSUBTITLES_PASSWORD"),
    ) {
        client
            .login(&user, &password)
            .await
            .context("OpenSubtitles login")?;
    }
    Ok(client)
}

/// Download up to `max_candidates` subtitles for the film that are not
/// already under `saved`, within the run's remaining budget. Every download
/// is kept: it cost quota, and a later re-check should not pay again.
async fn fetch_candidates(
    client: &opensubtitles_downloader::OpenSubtitlesClient,
    movie: &Movie,
    language: language_utils::Language,
    saved: &std::path::Path,
    downloads_left: &mut usize,
    max_candidates: usize,
) -> Result<()> {
    use opensubtitles_downloader::{rank_by_quality, Throttled};

    let imdb_num: u64 = movie
        .imdb_id
        .strip_prefix("tt")
        .unwrap_or(&movie.imdb_id)
        .parse()
        .with_context(|| format!("bad IMDb id {}", movie.imdb_id))?;
    let mut results = client
        .search_subtitles_for_movie(imdb_num, language.opensubtitles_languages())
        .await
        .context("OpenSubtitles search")?;
    results.retain(|s| !s.attributes.ai_translated && !s.attributes.machine_translated);
    rank_by_quality(&mut results);
    std::fs::create_dir_all(saved)?;
    let unseen = results
        .iter()
        .filter_map(|r| r.attributes.files.first())
        .filter(|f| !saved.join(format!("{}.srt", f.file_id)).exists())
        .count();
    println!(
        "      OpenSubtitles ({}): {} human-made candidates, {unseen} not yet fetched",
        language.opensubtitles_languages(),
        results.len()
    );

    let mut fetched = 0usize;
    for result in &results {
        if fetched >= max_candidates || *downloads_left == 0 {
            break;
        }
        let Some(file_id) = result.attributes.files.first().map(|f| f.file_id) else {
            continue;
        };
        let path = saved.join(format!("{file_id}.srt"));
        if path.exists() {
            continue;
        }
        match client.download_subtitle(file_id).await {
            Ok(srt) => {
                std::fs::write(&path, srt)?;
                fetched += 1;
                *downloads_left -= 1;
            }
            Err(e) => match e.downcast_ref::<Throttled>() {
                Some(Throttled::QuotaExhausted) => {
                    println!("      OpenSubtitles quota exhausted; no more downloads this run");
                    *downloads_left = 0;
                }
                Some(Throttled::TooManyRequests) => {
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                }
                None => return Err(e.context("OpenSubtitles download")),
            },
        }
    }
    Ok(())
}

/// Measure every other subtitle on hand for the film — the course's
/// downloaded SRT, a sidecar beside the video,
/// anything fetched earlier, plus fresh OpenSubtitles candidates while the
/// budget lasts — and install the best one that clears the bar and beats
/// `to_beat`. Returns what was adopted.
#[allow(clippy::too_many_arguments)]
async fn adopt_candidate(
    movie: &Movie,
    dir: &std::path::Path,
    data_root: &std::path::Path,
    language: language_utils::Language,
    code: &str,
    min_verbatim: f64,
    to_beat: usize,
    client: Option<&opensubtitles_downloader::OpenSubtitlesClient>,
    downloads_left: &mut usize,
    max_candidates: usize,
) -> Result<Option<String>> {
    use subtitle_corpus::verbatim::{self, Verdict};

    let mut candidates: Vec<(String, PathBuf)> = Vec::new();
    if let Some(course) = library::course_dir(&movie.original_language) {
        let movies = data_root
            .join(
                language_utils::Language::from_code(course)
                    .unwrap()
                    .corpus_code(),
            )
            .join("sentence-sources/movies");
        let path = movies
            .join("subtitles-raw")
            .join(format!("{}.srt", movie.imdb_id));
        if path.exists() {
            candidates.push(("course download".to_string(), path));
        }
    }
    if let Some(path) =
        library::sidecar(&movie.path, library::stream_codes(&movie.original_language))
    {
        candidates.push(("sidecar".to_string(), path));
    }
    let saved = dir.join("candidates");
    if let Some(client) = client {
        if *downloads_left > 0 {
            if let Err(e) = fetch_candidates(
                client,
                movie,
                language,
                &saved,
                downloads_left,
                max_candidates,
            )
            .await
            {
                println!("      {e:#}");
            }
        }
    }
    for entry in std::fs::read_dir(&saved).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "srt") {
            let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned());
            candidates.push((format!("opensubtitles {}", stem.unwrap_or_default()), path));
        }
    }
    if candidates.is_empty() {
        return Ok(None);
    }

    let transcript = subtitle_corpus::cues::load_transcript(&dir.join("transcript.jsonl"))?;
    let mut best: Option<(String, String, verbatim::Measure)> = None;
    for (label, path) in candidates {
        let text = String::from_utf8_lossy(&std::fs::read(&path)?).into_owned();
        let measure = verbatim::measure(
            &text,
            &transcript,
            language,
            code,
            &movie.imdb_id,
            min_verbatim,
        )
        .await?;
        println!("      {label:24} {}", verbatim::describe(&measure));
        let bar = best.as_ref().map_or(to_beat, |b| b.2.best_placed());
        if matches!(measure.verdict, Verdict::Verbatim | Verdict::Skewed)
            && measure.best_placed() > bar
        {
            best = Some((label, text, measure));
        }
    }
    let Some((label, text, measure)) = best else {
        return Ok(None);
    };

    // The subtitle being displaced may be the only copy of an OCR spend or
    // a disc extraction; keep the first one displaced under a fixed name
    // (later adoptions replace only what an earlier adoption installed).
    let replaced = dir.join("subtitle.replaced.srt");
    if !replaced.exists() {
        let _ = std::fs::rename(dir.join("subtitle.srt"), &replaced);
    }
    let adopted = dir.join("adopted.srt");
    std::fs::write(&adopted, &text)?;
    let timed = match &measure.aligned {
        Some(fit) if fit.fraction > measure.fraction => verbatim::retime(&text, fit),
        _ => sync::write_cues(&sync::parse_cues(&text)),
    };
    std::fs::write(dir.join("subtitle.srt"), timed)?;
    write_stamp(dir, movie, StampSource::File(&adopted));
    let _ = std::fs::remove_file(subtitle_corpus::clips::clips_path(dir));
    let _ = std::fs::remove_file(verbatim::report_path(dir));
    Ok(Some(label))
}

#[tokio::main]
async fn clips(
    out: PathBuf,
    jobs: usize,
    limit: usize,
    imdb: Option<String>,
    langs: Option<Vec<String>>,
    min_ratio: Option<f64>,
) -> Result<()> {
    let gate = subtitle_corpus::clips::Gate {
        min_ratio,
        ..Default::default()
    };
    subtitle_corpus::clips::clips_all(out, jobs, limit, imdb, langs, gate)
        .await
        .map(drop)
}

#[tokio::main]
async fn export_clips(
    out: PathBuf,
    dest: PathBuf,
    jobs: usize,
    limit: usize,
    imdb: Option<String>,
    langs: Option<Vec<String>>,
) -> Result<()> {
    subtitle_corpus::export::export_clips(out, dest, jobs, limit, imdb, langs, Default::default())
        .await
        .map(drop)
}

#[tokio::main]
async fn publish(
    out: PathBuf,
    dest: PathBuf,
    data_root: PathBuf,
    jobs: usize,
    langs: Option<Vec<String>>,
    bucket: String,
) -> Result<()> {
    subtitle_corpus::export::publish(out, dest, data_root, jobs, langs, bucket).await
}

#[tokio::main]
async fn prune(dest: PathBuf, bucket: String, manifest: PathBuf, apply: bool) -> Result<()> {
    subtitle_corpus::export::prune(dest, bucket, manifest, apply).await
}

/// Align by speech activity the films that word-matching could not place.
fn vad_sync(out: PathBuf, data_root: PathBuf, jobs: usize, limit: usize) -> Result<()> {
    let plan = read_plan(&out)?;
    let mut queue: Vec<(Movie, PathBuf)> = Vec::new();
    let mut parked = 0usize;
    for movie in plan {
        let dir = out.join(&movie.imdb_id);
        if dir.join("subtitle.srt").exists() {
            continue;
        }
        if let Some(raw) = subtitle_source(&movie, &dir, &data_root) {
            if movie.path.exists() {
                if sync_already_failed(&movie, &dir, &raw) {
                    parked += 1;
                } else {
                    queue.push((movie, raw));
                }
            }
        }
    }
    if limit > 0 {
        queue.truncate(limit);
    }
    println!(
        "{} films left for speech-activity alignment{}",
        queue.len(),
        parked_note(parked)
    );

    let out = &out;
    let results = parallel(queue, jobs, "aligning", move |(movie, raw)| {
        let outcome = (|| -> Result<vad::VadOffset> {
            let cues = sync::parse_cues(&std::fs::read_to_string(raw)?);
            if cues.is_empty() {
                bail!("no cues");
            }
            let duration = sync::duration_ms(&movie.path)?;
            // A subtitle that covers only part of the film cannot be placed reliably:
            // there is no way to tell a correctly-timed first-half subtitle from the
            // same file shifted onto the second half, and the anchors are too few to
            // arbitrate. Lust, Caution covered 46 of 158 minutes — a "CD1" subtitle —
            // and was confidently shifted 104 minutes to the end of the film with 88%
            // of its anchors agreeing. Films with genuinely long silent stretches
            // (2001 spans 61%) still clear this.
            let span = cues.iter().map(|c| c.end_ms).max().unwrap_or(0)
                - cues.iter().map(|c| c.start_ms).min().unwrap_or(0);
            if duration > 0 && (span as f64) < 0.5 * duration as f64 {
                bail!(
                    "subtitle covers only {:.0}% of the film — partial, cannot be placed",
                    span as f64 / duration as f64 * 100.0
                );
            }

            let speech = cached_speech_profile(movie, &out.join(&movie.imdb_id))?;
            let subtitle = vad::subtitle_profile(&cues, speech.len());
            let found = vad::find_offset(&speech, &subtitle, 120_000);
            if found.agreement < 0.15 {
                bail!("speech matches weakly ({:.2})", found.agreement);
            }
            if found.margin() < 0.08 {
                bail!(
                    "no clear peak: {:.2} vs {:.2} elsewhere",
                    found.agreement,
                    found.runner_up
                );
            }
            // Same sanity as the word-matching path: the result has to fit the
            // film it describes.
            let last = cues.iter().map(|c| c.end_ms).max().unwrap_or(0) + found.offset_ms;
            if last > duration + 120_000 || cues[0].start_ms + found.offset_ms < -60_000 {
                bail!("shift puts the subtitle outside the film");
            }
            let shifted: Vec<sync::Cue> = cues
                .iter()
                .map(|c| sync::Cue {
                    start_ms: c.start_ms + found.offset_ms,
                    end_ms: c.end_ms + found.offset_ms,
                    text: c.text.clone(),
                })
                .collect();
            let dir = out.join(&movie.imdb_id);
            std::fs::create_dir_all(&dir)?;
            std::fs::write(dir.join("subtitle.srt"), sync::write_cues(&shifted))?;
            write_stamp(&dir, movie, StampSource::File(raw));
            Ok(found)
        })();
        match &outcome {
            Ok(v) => println!(
                "  {} ✓ {:+.2}s (agree {:.2}, margin {:.2})",
                truncate(&movie.title, 34),
                v.offset_ms as f64 / 1000.0,
                v.agreement,
                v.margin()
            ),
            Err(e) => {
                println!("  {} ✗ {e}", truncate(&movie.title, 34));
                // vad-sync is the last gate: in refresh order the film has
                // just failed text-sync and sync too, so these inputs are a
                // proven dead end until one of them changes.
                record_sync_failure(movie, &out.join(&movie.imdb_id), raw);
            }
        }
        outcome.is_ok()
    });
    let done = results.iter().filter(|ok| **ok).count();
    println!(
        "\n{done} aligned by speech activity, {} still unaligned",
        results.len() - done
    );
    Ok(())
}

/// Read one reference track's cues — parsed text for a text stream, bare
/// timestamps for a bitmap one.
fn reference_cues(
    video: &std::path::Path,
    stream: &library::ReferenceStream,
    scratch: &std::path::Path,
) -> Result<Vec<sync::Cue>> {
    if stream.is_text {
        let tmp = scratch.with_extension("srt");
        let status = Command::new("ffmpeg")
            .args(["-v", "error", "-y", "-i"])
            .arg(video)
            .args(["-map", &format!("0:{}", stream.index)])
            .arg(&tmp)
            .status()
            .context("ffmpeg failed to start")?;
        if !status.success() {
            bail!("ffmpeg could not convert stream {}", stream.index);
        }
        let cues = sync::parse_cues(&std::fs::read_to_string(&tmp)?);
        let _ = std::fs::remove_file(&tmp);
        Ok(cues)
    } else {
        let bitmaps = if stream.codec == "dvd_subtitle" {
            vobsub::cues(video, stream.index)?
        } else {
            let tmp = scratch.with_extension("sup");
            let _ = std::fs::remove_file(&tmp);
            ocr::extract_sup(video, stream.index, &tmp)?;
            let data = std::fs::read(&tmp)?;
            let _ = std::fs::remove_file(&tmp);
            pgs::cues(&data)
        };
        Ok(bitmaps
            .into_iter()
            .map(|c| sync::Cue {
                start_ms: c.start_ms as i64,
                end_ms: c.end_ms as i64,
                text: String::new(),
            })
            .collect())
    }
}

fn text_sync(out: PathBuf, data_root: PathBuf, jobs: usize, limit: usize) -> Result<()> {
    let plan = read_plan(&out)?;
    let mut queue: Vec<(Movie, PathBuf)> = Vec::new();
    let mut parked = 0usize;
    for movie in plan {
        let dir = out.join(&movie.imdb_id);
        if dir.join("subtitle.srt").exists() {
            continue;
        }
        if let Some(raw) = subtitle_source(&movie, &dir, &data_root) {
            if movie.path.exists() {
                if sync_already_failed(&movie, &dir, &raw) {
                    parked += 1;
                } else {
                    queue.push((movie, raw));
                }
            }
        }
    }
    if limit > 0 {
        queue.truncate(limit);
    }
    println!(
        "{} films left to align against their discs' own tracks{}",
        queue.len(),
        parked_note(parked)
    );

    let out = &out;
    let results = parallel(queue, jobs, "aligning", move |(movie, raw)| {
        let outcome = (|| -> Result<(String, f64, vad::VadOffset, usize)> {
            let cues = sync::parse_cues(&std::fs::read_to_string(raw)?);
            if cues.is_empty() {
                bail!("no cues");
            }
            let duration = sync::duration_ms(&movie.path)?;
            // Same guard as the audio path: a partial subtitle cannot be
            // placed, only mistaken for a placed one.
            let span = cues.iter().map(|c| c.end_ms).max().unwrap_or(0)
                - cues.iter().map(|c| c.start_ms).min().unwrap_or(0);
            if duration > 0 && (span as f64) < 0.5 * duration as f64 {
                bail!(
                    "subtitle covers only {:.0}% of the film — partial, cannot be placed",
                    span as f64 / duration as f64 * 100.0
                );
            }
            let refs = library::reference_subtitle_streams(&movie.path)?;
            if refs.is_empty() {
                bail!("disc carries no usable subtitle track");
            }

            let buckets = (duration / vad::BUCKET_MS) as usize;
            // A subtitle authored against a PAL (25fps), cinema (24fps) or
            // NTSC-film (23.976fps) transfer of the same film runs fast or
            // slow by a constant factor. A pure shift search sees that as a
            // wide, flat peak — high agreement, no margin — so each rate is
            // searched as its own hypothesis and the sharpest peak wins. The
            // 0.1% cinema/NTSC pair matters despite its size: it is 5.4s of
            // drift across a feature (measured on Delicatessen), enough to
            // flatten the peak and drag the compromise offset seconds off at
            // both ends.
            const RATES: &[f64] = &[
                23.976 / 25.0,
                23.976 / 24.0,
                1.0,
                24.0 / 23.976,
                25.0 / 23.976,
            ];
            let candidates: Vec<(f64, Vec<f32>)> = RATES
                .iter()
                .map(|&rate| {
                    let scaled: Vec<sync::Cue> = cues
                        .iter()
                        .map(|c| sync::Cue {
                            start_ms: (rate * c.start_ms as f64) as i64,
                            end_ms: (rate * c.end_ms as f64) as i64,
                            text: String::new(),
                        })
                        .collect();
                    (rate, vad::subtitle_profile(&scaled, buckets))
                })
                .collect();
            // Each reference votes independently; they were all authored
            // against this file, so they must agree with each other. One
            // reference passing the gates is an answer, two agreeing is a
            // cross-check no single-method threshold can give.
            let mut votes: Vec<(String, f64, vad::VadOffset)> = Vec::new();
            for (n, stream) in refs.iter().take(4).enumerate() {
                let scratch = std::env::temp_dir().join(format!(
                    "textsync-{}-{}-{n}",
                    std::process::id(),
                    movie.imdb_id
                ));
                let Ok(ref_cues) = cached_reference_cues(movie, out, stream, &scratch) else {
                    continue;
                };
                let ref_span = ref_cues.iter().map(|c| c.end_ms).max().unwrap_or(0)
                    - ref_cues.iter().map(|c| c.start_ms).min().unwrap_or(0);
                // An unflagged forced track looks exactly like a real one
                // until you count its cues.
                if ref_cues.len() < 50 || (ref_span as f64) < 0.4 * duration as f64 {
                    continue;
                }
                let reference = vad::subtitle_profile(&ref_cues, buckets);
                // The best hypothesis is the one that *correlates* best. Margin
                // cannot arbitrate here: cue-vs-cue peaks are broad (a cue
                // lasts seconds, so a 2s-away shift still overlaps most of it),
                // leaving every hypothesis's margin near zero — and choosing
                // among near-ties by margin picks noise.
                let (rate, found) = candidates
                    .iter()
                    .map(|(rate, profile)| (*rate, vad::find_offset(&reference, profile, 300_000)))
                    .max_by(|a, b| a.1.agreement.total_cmp(&b.1.agreement))
                    .expect("RATES is never empty");
                let label = format!(
                    "{}:{}",
                    if stream.language.is_empty() {
                        "und"
                    } else {
                        &stream.language
                    },
                    if stream.is_text { "text" } else { "pgs" }
                );
                votes.push((label, rate, found));
            }
            // Two ways to believe an answer. A single reference is enough when
            // its peak stands clear of every rival (margin). But cue-vs-cue
            // peaks are broad — a cue lasts seconds, so near shifts correlate
            // almost as well and margin stays near zero even when the answer
            // is right. What margin cannot supply, *independent agreement*
            // can: two tracks, authored separately against this same file,
            // naming the same offset within half a second is not chance.
            let confident: Vec<_> = votes
                .iter()
                .filter(|(_, _, v)| v.agreement >= 0.25 && v.margin() >= 0.10)
                .cloned()
                .collect();
            let chosen = if !confident.is_empty() {
                confident
            } else {
                /// Correlation each track must reach before its vote counts
                /// toward a consensus that overrides the margin gate.
                const CONSENSUS_AGREEMENT: f32 = 0.4;
                let strong: Vec<_> = votes
                    .iter()
                    .filter(|(_, _, v)| v.agreement >= CONSENSUS_AGREEMENT)
                    .cloned()
                    .collect();
                let within = |a: &vad::VadOffset, b: &vad::VadOffset| {
                    (a.offset_ms - b.offset_ms).abs() <= 500
                };
                let consensus: Vec<_> = strong
                    .iter()
                    .filter(|(_, r1, v1)| {
                        strong
                            .iter()
                            .filter(|(_, r2, v2)| r1 == r2 && within(v1, v2))
                            .count()
                            >= 2
                    })
                    .cloned()
                    .collect();
                if consensus.is_empty() {
                    bail!("no reference track produced a confident offset");
                }
                consensus
            };
            let (label, rate, best) = chosen
                .iter()
                .max_by(|a, b| a.2.agreement.total_cmp(&b.2.agreement))
                .cloned()
                .expect("chosen is non-empty");
            // The rate is a property of the candidate file, so two references
            // concluding different rates is as damning as two different
            // offsets.
            if chosen.iter().any(|(_, r, _)| *r != rate) {
                bail!("references disagree on the playback rate");
            }
            let spread = chosen
                .iter()
                .map(|(_, _, v)| v.offset_ms)
                .fold((i64::MAX, i64::MIN), |(lo, hi), o| (lo.min(o), hi.max(o)));
            if chosen.len() > 1 && spread.1 - spread.0 > 500 {
                bail!(
                    "references disagree: offsets span {:.1}s across {} tracks",
                    (spread.1 - spread.0) as f64 / 1000.0,
                    chosen.len()
                );
            }
            let votes = chosen;
            let place = |ms: i64| (rate * ms as f64) as i64 + best.offset_ms;
            let last = place(cues.iter().map(|c| c.end_ms).max().unwrap_or(0));
            if last > duration + 120_000 || place(cues[0].start_ms) < -60_000 {
                bail!("shift puts the subtitle outside the film");
            }
            let shifted: Vec<sync::Cue> = cues
                .iter()
                .map(|c| sync::Cue {
                    start_ms: place(c.start_ms),
                    end_ms: place(c.end_ms),
                    text: c.text.clone(),
                })
                .collect();
            let dir = out.join(&movie.imdb_id);
            std::fs::create_dir_all(&dir)?;
            std::fs::write(dir.join("subtitle.srt"), sync::write_cues(&shifted))?;
            write_stamp(&dir, movie, StampSource::File(raw));
            Ok((label, rate, best, votes.len()))
        })();
        match &outcome {
            Ok((label, rate, v, n)) => println!(
                "  {} ✓ {:+.2}s ×{rate:.4} via {label} ({n} track(s) agree, agree {:.2}, margin {:.2})",
                truncate(&movie.title, 34),
                v.offset_ms as f64 / 1000.0,
                v.agreement,
                v.margin()
            ),
            Err(e) => println!("  {} ✗ {e}", truncate(&movie.title, 34)),
        }
        outcome.is_ok()
    });
    let done = results.iter().filter(|ok| **ok).count();
    println!(
        "\n{done} aligned against disc tracks, {} still unaligned",
        results.len() - done
    );
    Ok(())
}

fn main() -> Result<()> {
    // The Cloudflare and OpenAI keys live in the repo's `.env`, which nothing
    // in the environment exports: `.envrc` is only `use flake`, and the flake's
    // shellHook handles R2 and GCP alone. Without this, every invocation
    // outside an interactive shell that happened to have them is one missing
    // variable away from a run that quietly verifies nothing.
    dotenvy::dotenv().ok();
    match Args::parse().command {
        Command_::Proofread(options) => subtitle_corpus::proofread::run(options),
        Command_::WordCheck {
            out,
            language,
            imdb,
            limit,
            dry_run,
            eval,
        } => subtitle_corpus::word_check::run(out, language, imdb, limit, dry_run, eval),
        Command_::Inventory {
            library,
            data_root,
            out,
            jobs,
        } => inventory(library, data_root, out, jobs),
        Command_::Refresh {
            library,
            data_root,
            out,
            transcribe_imdb,
        } => refresh(library, data_root, out, transcribe_imdb),
        Command_::Extract { out, jobs, limit } => extract(out, jobs, limit),
        Command_::ExtractAudio {
            out,
            jobs,
            limit,
            imdb,
        } => extract_audio(out, jobs, limit, imdb.as_deref()),
        Command_::AudioCheck {
            out,
            jobs,
            limit,
            imdb,
        } => audio_check(out, jobs, limit, imdb),
        Command_::Ocr {
            out,
            model,
            films_in_flight,
            limit,
        } => ocr_all(out, model, films_in_flight, limit),
        Command_::Sync {
            out,
            data_root,
            films_in_flight,
            limit,
        } => sync_all(out, data_root, films_in_flight, limit),
        Command_::Segment {
            out,
            all,
            limit,
            imdb,
        } => segment_all(out, all, limit, imdb),
        Command_::Transcribe {
            out,
            films_in_flight,
            limit,
            imdb,
        } => transcribe_all(out, films_in_flight, limit, imdb),
        Command_::SpeechProfiles { out, jobs, limit } => speech_profiles(out, jobs, limit),
        Command_::TranscriptCheck {
            out,
            data_root,
            limit,
            imdb,
            min_verbatim,
            dry_run,
            max_downloads,
            max_candidates,
        } => transcript_check(
            out,
            data_root,
            limit,
            imdb,
            min_verbatim,
            dry_run,
            max_downloads,
            max_candidates,
        ),
        Command_::VadSync {
            out,
            data_root,
            jobs,
            limit,
        } => vad_sync(out, data_root, jobs, limit),
        Command_::TextSync {
            out,
            data_root,
            jobs,
            limit,
        } => text_sync(out, data_root, jobs, limit),
        Command_::Clips {
            out,
            jobs,
            limit,
            imdb,
            langs,
            min_ratio,
        } => clips(out, jobs, limit, imdb, langs, min_ratio),
        Command_::ExportClips {
            out,
            dest,
            jobs,
            limit,
            imdb,
            langs,
        } => export_clips(out, dest, jobs, limit, imdb, langs),
        Command_::Publish {
            out,
            dest,
            data_root,
            jobs,
            langs,
            bucket,
        } => publish(out, dest, data_root, jobs, langs, bucket),
        Command_::Prune {
            dest,
            bucket,
            manifest,
            apply,
        } => {
            let manifest = manifest.unwrap_or_else(|| dest.join("orphan-candidates.json"));
            prune(dest, bucket, manifest, apply)
        }
        Command_::ExportSidecars { out } => export_sidecars(out),
        Command_::ExportYap {
            out,
            data_root,
            langs,
            dry_run,
        } => export_yap(out, data_root, langs, dry_run),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sidecar_outranks_the_course_raw_it_was_exported_as() {
        let dir = tempfile::tempdir().unwrap();
        let sidecar = dir.path().join("film.fr.srt");
        std::fs::write(&sidecar, "1\n00:00:01,000 --> 00:00:02,000\nBonjour.\n\n").unwrap();
        let raw_dir = dir
            .path()
            .join("data/fra/sentence-sources/movies/subtitles-raw");
        std::fs::create_dir_all(&raw_dir).unwrap();
        std::fs::write(raw_dir.join("tt0000001.srt"), "exported copy").unwrap();
        let movie = Movie {
            imdb_id: "tt0000001".into(),
            title: "Film".into(),
            year: Some(1981),
            path: dir.path().join("film.mkv"),
            original_language: "French".into(),
            source: Source::Sidecar {
                path: sidecar.clone(),
            },
        };
        let out = dir.path().join("corpus/tt0000001");
        assert_eq!(
            subtitle_source(&movie, &out, &dir.path().join("data")),
            Some(sidecar)
        );
    }
}
