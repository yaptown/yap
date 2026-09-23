//! Serve-ready clip export: every passing clip becomes a directory of
//! `hi.mp4` + `lo.mp4` + `meta.json`, cut generously from the source film
//! (neighboring subtitle lines as context) with the sidecar — not the file
//! boundary — defining what the clip *is*. Schema and rationale:
//! `docs/clip-sidecar.md`.
//!
//! Everything in the sidecar comes from artifacts already on disk; the
//! forced alignment only reads cached responses, so export neither probes
//! the model nor spends inference, including when an alignment is missing.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::{bail, Context, Result};
use futures::{stream, TryStreamExt};
use language_utils::Language;
use md5::{Digest as _, Md5};
use movie_subtitles::cleanup_subtitle_text;
use movie_subtitles::segment::SubtitleSegmenter;
use serde_json::json;
use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;

use crate::clips::{
    clips_path, read_file as read_clips_with_provenance, subtitle_sentences, Clip, Provenance,
};
use crate::cues::{load_transcript, parse_cues, repair_latin_homoglyphs};
use crate::library::{
    course_dir, current_verdict, output_is_fresh, read_plan, truncate, Movie, Source,
};
use crate::sync::{AudioStreamIdentity, Cue};
use crate::transcript::{Kind, Spoken};
use movie_subtitles::sentences::KeyedSentence;

/// Loudness target for the served clips, measured over the critical span.
const TARGET_I: f64 = -18.0;
/// True-peak ceiling; caps the gain on quiet-but-peaky clips.
const TP_CEIL: f64 = -1.5;
/// A neighboring subtitle line joins the cut only when the silence between
/// it and the clip is at most this — a longer gap usually means a scene
/// change, and unrelated footage is worse than no context.
const CTX_GAP_MS: i64 = 2_000;
/// The cut never exceeds this; context is dropped (furthest line first)
/// before the scored span ever is.
const CTX_CAP_MS: i64 = 15_000;
/// Breathing room past a context line's cue stamps.
const CTX_PAD_MS: i64 = 150;
/// Minimum gap between speech runs for a widened context cut. Not part of the recipe:
/// it only moves the cut bounds, which the media stamp already carries, so
/// changing it re-encodes exactly the clips whose context actually changes.
const CTX_CLEAN_MS: i64 = 200;
/// Maximum outward search from a padded context bound. Like CTX_CLEAN_MS,
/// this is represented by the cut bounds in the media stamp, not the recipe.
const CTX_WIDEN_MS: i64 = 1_500;
/// The hi rendition is never upscaled and never taller than this.
const MAX_HEIGHT: i64 = 1440;
const LO_HEIGHT: i64 = 480;
/// NVENC constant-quality target for the hi rendition. Clips are nearly
/// always played well under full screen, so the cut is set for a small file
/// rather than for pixel-peeping: cq 29 measured ~3.3 Mbps against ~7.2 at
/// cq 23 on 1440p, halving the served bytes. The 1440p cap stays because a
/// few people do watch on 5K displays.
const HI_CQ: u32 = 29;
const HI_GPU_PRESET: &str = "p4";
const HI_CRF: u32 = 19;
const HI_PRESET: &str = "medium";
const HI_AAC: &str = "160k";
const LO_CRF: u32 = 27;
const LO_PRESET: &str = "veryfast";
const LO_AAC: &str = "96k";
/// R2 puts in flight at once during upload (see `upload_lang`).
const UPLOAD_JOBS: usize = 64;

/// Sidecar `format` field.
const SIDECAR_FORMAT: u32 = 4;

/// Everything that shapes the rendered files, in one comparable string.
/// Built from the constants so no tweak can be forgotten; anything that
/// changes the output must appear here or in the per-film stamp.
fn encode_recipe() -> String {
    format!(
        "gpu hi h264_nvenc cq{HI_CQ} {HI_GPU_PRESET} forced-idr1 scale_cuda | \
         gpu tonemap_cuda hable desat0 yuv420p matrix/primaries/transfer bt709 | \
         fallback cpu libx264 crf{HI_CRF} {HI_PRESET}, then cpu core_only1 | \
         cpu tonemap zscale linear npl100 gbrpf32le bt709 hable desat0 tv lanczos yuv420p | \
         hi aac{HI_AAC} le{MAX_HEIGHT}p | \
         lo libx264 crf{LO_CRF} {LO_PRESET} aac{LO_AAC} le{LO_HEIGHT}p | \
         loudnorm I{TARGET_I} TP{TP_CEIL} critical linear stereo 48000Hz | \
         keyframe@critical faststart"
    )
}

/// Where one clip is cut from the film: the scored span (what the gates
/// heard) and the generous cut around it with neighbouring subtitle lines.
struct Cut {
    scored_start: i64,
    scored_end: i64,
    cut_start: i64,
    cut_end: i64,
    ctx_before: usize,
    ctx_after: usize,
}

impl Cut {
    /// The scored span, clip-relative: the keyframe lands on its start and
    /// the loudness is measured over it.
    fn critical(&self) -> (i64, i64) {
        (
            self.scored_start - self.cut_start,
            self.scored_end - self.cut_start,
        )
    }
}

fn plan_cut(clip: &Clip, cues: &[Cue], transcript: &[Spoken]) -> Cut {
    let scored_start = (clip.start_ms - clip.pad_before_ms).max(0);
    let scored_end = clip.end_ms + clip.pad_after_ms;
    let (cut_start, cut_end, ctx_before, ctx_after) = context_bounds(
        clip.start_ms,
        clip.end_ms,
        scored_start,
        scored_end,
        cues,
        transcript,
    );
    Cut {
        scored_start,
        scored_end,
        cut_start,
        cut_end,
        ctx_before,
        ctx_after,
    }
}

/// Everything that decides the bytes of a clip's two renditions: the recipe,
/// the source video's identity, and the cut. Written into the sidecar as
/// `media.stamp`; on resume the renditions are reused when the stored stamp
/// equals the one computed now, and re-rendered otherwise. Deliberately
/// *not* the clips provenance: a re-map under a new segmenter or gate that
/// lands on the same span must not cost a day of re-encoding (it did,
/// twice, in 2026-09). The sidecar itself is always rewritten — it is cheap
/// and carries the provenance.
fn media_stamp(
    movie: &Movie,
    video: &VideoProbe,
    audio_stream: u32,
    cut: &Cut,
) -> serde_json::Value {
    let video_bytes = std::fs::metadata(&movie.path).map(|m| m.len()).unwrap_or(0);
    let critical = cut.critical();
    json!({
        "recipe": encode_recipe(),
        "video": {
            "filename": movie.path.file_name().and_then(|f| f.to_str()),
            "bytes": video_bytes,
            "duration_ms": video.duration_ms,
            "height": video.height,
            "hdr": video.hdr,
            "audio_stream": audio_stream,
        },
        "cut": {
            "start_ms": cut.cut_start,
            "end_ms": cut.cut_end,
            "critical_start_ms": critical.0,
            "critical_end_ms": critical.1,
        },
    })
}

/// Export clips with at most `jobs` concurrent clips (at least one). Each GPU
/// attempt uses one NVENC session; the roughly 12 sessions available on this
/// host are shared with Jellyfin, so leave headroom when choosing `jobs`.
/// CPU fallbacks and loudness measurement also share this concurrency budget.
pub async fn export_clips(
    out: PathBuf,
    dest: PathBuf,
    jobs: usize,
    limit: usize,
    imdb: Option<String>,
    langs: Option<Vec<String>>,
) -> Result<()> {
    let plan = read_plan(&out)?;
    let mut queue: Vec<Movie> = plan
        .into_iter()
        .filter(|m| imdb.as_deref().is_none_or(|id| m.imdb_id == id))
        .filter(|m| {
            langs.as_ref().is_none_or(|l| {
                course_dir(&m.original_language).is_some_and(|c| l.iter().any(|x| x == c))
            })
        })
        .filter(|m| clips_path(&out.join(&m.imdb_id)).exists())
        .collect();
    if limit > 0 {
        queue.truncate(limit);
    }
    println!("{} films with clips to export", queue.len());

    let store = osmo::Store::open("./.cache");
    let alignment_cache_failures = AtomicUsize::new(0);
    let (mut rendered, mut refreshed, mut unchanged) = (0usize, 0usize, 0usize);
    let mut failed: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut valid: std::collections::HashMap<String, std::collections::HashSet<String>> =
        std::collections::HashMap::new();
    for movie in &queue {
        let title = truncate(&movie.title, 34);
        match export_film(&store, movie, &out, &dest, jobs, &alignment_cache_failures).await {
            Ok(f) => {
                rendered += f.rendered;
                refreshed += f.refreshed;
                unchanged += f.unchanged;
                println!(
                    "{title} ✓ {} rendered, {} sidecars refreshed, {} current",
                    f.rendered, f.refreshed, f.unchanged
                );
                valid.entry(f.code).or_default().extend(f.ids);
            }
            Err(e) => {
                failed.insert(&movie.imdb_id);
                println!("{title} ✗ {e:#}");
            }
        }
    }
    println!("\n{rendered} clips rendered, {refreshed} sidecars refreshed, {unchanged} current");
    let missing = alignment_cache_failures.load(Ordering::Relaxed);
    if missing > 0 {
        eprintln!("warning: {missing} expected phoneme alignments omitted due to missing or invalid cached responses");
    }

    // Orphan sweep: a clip dir whose id no longer exists (sentence re-keyed,
    // gate change, film dropped from the plan or its clips evicted) must not
    // linger looking servable. Only on unfiltered runs — a partial run
    // cannot know the full id set — and judged film by film: a film that
    // failed this run has an unknown id set, so its dirs are kept, but one
    // failure must not shield every other film's leftovers (2026-09-08:
    // seven stale films kept 99 orphans in the served index for a week).
    if imdb.is_none() && limit == 0 {
        for (code, ids) in &valid {
            let lang_dir = dest.join(code);
            let mut swept = 0usize;
            for entry in std::fs::read_dir(&lang_dir).into_iter().flatten().flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().into_owned();
                let film = name.split('-').next().unwrap_or_default();
                if path.is_dir() && !ids.contains(&name) && !failed.contains(film) {
                    std::fs::remove_dir_all(&path)?;
                    swept += 1;
                }
            }
            if swept > 0 {
                println!("swept {swept} orphaned clip dirs from {code}");
            }
        }
    }

    // The index is rebuilt from the sidecars on every run, so resumed and
    // partial runs still leave it whole.
    for lang in queue
        .iter()
        .filter_map(|m| course_dir(&m.original_language))
        .collect::<std::collections::BTreeSet<_>>()
    {
        let n = write_index(&dest.join(lang))?;
        println!("index: {lang} {n} clips");
    }
    Ok(())
}

async fn export_film(
    store: &osmo::Store,
    movie: &Movie,
    out: &Path,
    dest: &Path,
    jobs: usize,
    alignment_cache_failures: &AtomicUsize,
) -> Result<FilmExport> {
    let code = course_dir(&movie.original_language).context("unmapped language")?;
    let language = Language::from_code(code).context("unmapped course code")?;
    let dir = out.join(&movie.imdb_id);
    if !movie.path.exists() {
        bail!("video missing: {}", movie.path.display());
    }

    let (provenance, clips) = read_clips_with_provenance(&clips_path(&dir))?;
    let srt = std::fs::read_to_string(dir.join("subtitle.srt"))?;
    let segmenter = SubtitleSegmenter::for_language(language)?;
    let sentences = subtitle_sentences(&srt, language, &segmenter).await?;
    let cues: Vec<Cue> = parse_cues(&srt)
        .into_iter()
        .filter_map(|c| {
            let text = repair_latin_homoglyphs(&cleanup_subtitle_text(&c.text));
            (!text.is_empty()).then_some(Cue { text, ..c })
        })
        .collect();
    let transcript = load_transcript(&dir.join("transcript.jsonl"))?;
    let video = probe_video(&movie.path)?;
    let audio_stream = audio_stream_index(&dir.join("audio.json"), &movie.path, &video)?;

    // Audio-only films have no phoneme alignment; no context or model probe
    // exists on this path, only the cache store's read interface.
    let cache = (!crate::clips::audio_only(code)).then_some(store);

    let lang_dir = dest.join(code);
    let passing: Vec<&Clip> = clips.iter().filter(|c| c.passed).collect();
    let rendered = AtomicUsize::new(0);
    let refreshed = AtomicUsize::new(0);
    let unchanged = AtomicUsize::new(0);
    let total = passing.len();

    use futures::StreamExt;
    let results: Vec<Result<()>> = futures::stream::iter(passing.iter().map(|clip| {
        let (cache, movie, provenance, clips, sentences, cues, transcript) = (
            cache,
            movie,
            &provenance,
            &clips,
            &sentences,
            &cues,
            &transcript,
        );
        let (lang_dir, video) = (&lang_dir, &video);
        let (rendered, refreshed, unchanged) = (&rendered, &refreshed, &unchanged);
        async move {
            let id = clip_id(&movie.imdb_id, clip, sentences, clips);
            let clip_dir = lang_dir.join(&id);
            let cut = plan_cut(clip, cues, transcript);
            let stamp = media_stamp(movie, video, audio_stream, &cut);
            // Renditions on disk are reused when they were cut from the same
            // bytes to the same recipe; the loudness they were normalised
            // with is a function of those same inputs, so it comes along.
            let old = std::fs::read(clip_dir.join("meta.json")).ok();
            let reuse = old
                .as_deref()
                .and_then(|b| serde_json::from_slice::<serde_json::Value>(b).ok())
                .filter(|m| {
                    m["media"]["stamp"] == stamp
                        && clip_dir.join("hi.mp4").exists()
                        && clip_dir.join("lo.mp4").exists()
                })
                .and_then(|m| {
                    let l = &m["media"]["loudnorm"];
                    Some(Loudness {
                        measured_i: l["measured_i"].as_f64()?,
                        measured_tp: l["measured_tp"].as_f64()?,
                    })
                });
            if reuse.is_none() {
                let _ = std::fs::remove_dir_all(&clip_dir);
            }
            let course_sentence = sentences
                .iter()
                .any(|k| k.course_worthy && k.sentence == clip.sentence);
            let r = export_one(
                cache,
                movie,
                provenance,
                clip,
                &id,
                course_sentence,
                &cut,
                stamp,
                reuse,
                old.as_deref(),
                cues,
                transcript,
                alignment_cache_failures,
                video,
                audio_stream,
                &clip_dir,
            )
            .await;
            match &r {
                Ok(Outcome::Unchanged) => {
                    unchanged.fetch_add(1, Ordering::Relaxed);
                }
                Ok(Outcome::Refreshed) => {
                    refreshed.fetch_add(1, Ordering::Relaxed);
                }
                Ok(Outcome::Rendered) => {
                    let n = rendered.fetch_add(1, Ordering::Relaxed) + 1;
                    println!("  [{n}/{total}] {id} ✓");
                }
                Err(e) => println!("  {id} ✗ {e:#}"),
            }
            r.map(|_| ())
        }
    }))
    .buffer_unordered(jobs.max(1))
    .collect()
    .await;

    let failed = results.iter().filter(|r| r.is_err()).count();
    if failed > 0 {
        bail!("{failed} of {total} clips failed");
    }
    let ids = passing
        .iter()
        .map(|clip| clip_id(&movie.imdb_id, clip, &sentences, &clips))
        .collect();
    Ok(FilmExport {
        rendered: rendered.load(Ordering::Relaxed),
        refreshed: refreshed.load(Ordering::Relaxed),
        unchanged: unchanged.load(Ordering::Relaxed),
        code: code.to_string(),
        ids,
    })
}

/// What one film's export produced, for totals and the orphan sweep.
struct FilmExport {
    /// Clip dirs whose renditions were (re-)encoded.
    rendered: usize,
    /// Renditions reused, sidecar rewritten.
    refreshed: usize,
    /// Nothing to do: renditions reused and the sidecar came out identical.
    unchanged: usize,
    code: String,
    ids: Vec<String>,
}

enum Outcome {
    Rendered,
    Refreshed,
    Unchanged,
}

/// EBU R128 numbers the renditions were normalised with; carried over from
/// the old sidecar when the renditions are reused.
struct Loudness {
    measured_i: f64,
    measured_tp: f64,
}

/// `imdb - sha256(NFC sentence)[..8] - occurrence index`, the occurrence
/// counted over the segmented subtitle sentences in cue order (all of them,
/// aligned or not), so the id is fixed by the subtitle file alone.
fn clip_id(imdb: &str, clip: &Clip, sentences: &[KeyedSentence], all: &[Clip]) -> String {
    let normalized: String = clip.sentence.nfc().collect();
    let digest = Sha256::digest(normalized.as_bytes());
    let hash: String = digest[..4].iter().map(|b| format!("{b:02x}")).collect();

    // Occurrences of this sentence, in subtitle order, with passage spans.
    let occ: Vec<(i64, i64)> = sentences
        .iter()
        .filter(|k| k.sentence == clip.sentence)
        .map(|k| (i64::from(k.start_ms), i64::from(k.end_ms)))
        .collect();
    // Pick the occurrence whose passage span sits closest to a clip's audio
    // span. Cue stamps are display times, so exact containment can miss;
    // nearest midpoint is unambiguous when the same line recurs minutes
    // apart.
    let nearest = |c: &Clip| -> usize {
        let mid = (c.start_ms + c.end_ms) / 2;
        occ.iter()
            .enumerate()
            .min_by_key(|(_, (a, b))| ((a + b) / 2 - mid).abs())
            .map(|(i, _)| i)
            .unwrap_or(0)
    };
    // Two clips of the same sentence must never land on one id: exported
    // concurrently, they would render into and delete the same directory
    // (Pee Mak, 2026-09-07). When the twins of this sentence do not all pick
    // distinct occurrences — more clips than occurrences, or two clips
    // nearest the same one — every twin falls back to its position among
    // the twins in `all` (`clip` is one of them), unique by construction.
    let twins: Vec<&Clip> = all.iter().filter(|c| c.sentence == clip.sentence).collect();
    let picks: std::collections::HashSet<usize> = twins.iter().map(|c| nearest(c)).collect();
    let index = if picks.len() == twins.len() {
        nearest(clip)
    } else {
        twins
            .iter()
            .position(|c| std::ptr::eq(*c, clip))
            .expect("clip is drawn from `all`")
    };
    format!("{imdb}-{hash}-{index}")
}

#[allow(clippy::too_many_arguments)]
async fn export_one(
    cache: Option<&osmo::Store>,
    movie: &Movie,
    provenance: &Provenance,
    clip: &Clip,
    id: &str,
    course_sentence: bool,
    cut: &Cut,
    stamp: serde_json::Value,
    reuse: Option<Loudness>,
    old_sidecar: Option<&[u8]>,
    cues: &[Cue],
    transcript: &[Spoken],
    alignment_cache_failures: &AtomicUsize,
    video: &VideoProbe,
    audio_stream: u32,
    clip_dir: &Path,
) -> Result<Outcome> {
    let Cut {
        scored_start,
        scored_end,
        cut_start,
        cut_end,
        ctx_before,
        ctx_after,
    } = *cut;
    let critical = cut.critical();

    // The mapper's stored WAV hash and exact labels name the response; no
    // audio cut or identity probe is needed. Missing alignments are counted.
    let alignment = align_phonemes(
        cache,
        alignment_cache_failures,
        clip,
        scored_start - cut_start,
    )
    .await;

    let reused = reuse.is_some();
    let Loudness {
        measured_i,
        measured_tp,
    } = match reuse {
        Some(l) => l,
        None => {
            // Loudness of the critical span through the same stereo downmix
            // the encode uses; gain capped by the true-peak ceiling.
            let (measured_i, measured_tp) = tokio::task::spawn_blocking({
                let path = movie.path.clone();
                move || {
                    measure_loudness(&path, audio_stream, scored_start, scored_end - scored_start)
                }
            })
            .await??;
            Loudness {
                measured_i,
                measured_tp,
            }
        }
    };
    let gain_db = (TARGET_I - measured_i).min(TP_CEIL - measured_tp);

    if !reused {
        std::fs::create_dir_all(clip_dir)?;
        let encode = tokio::task::spawn_blocking({
            let (path, clip_dir) = (movie.path.clone(), clip_dir.to_path_buf());
            let video = video.clone();
            let crit_s = critical.0 as f64 / 1000.0;
            move || {
                encode_renditions(
                    &path,
                    audio_stream,
                    &video,
                    cut_start,
                    cut_end,
                    gain_db,
                    crit_s,
                    &clip_dir,
                )
            }
        })
        .await?;
        if let Err(e) = encode {
            // A half-written directory must not read as done on resume.
            let _ = std::fs::remove_dir_all(clip_dir);
            return Err(e);
        }
    }

    let rel = |ms: i64| ms - cut_start;
    let sidecar = json!({
        "format": SIDECAR_FORMAT,
        "id": id,
        "language": provenance.inputs.language,
        "film": {
            "imdb_id": movie.imdb_id,
            "title": movie.title,
            "year": movie.year,
            "subtitle_digest": provenance.inputs.subtitle_digest,
            "transcript_digest": provenance.inputs.transcript_digest,
        },
        "source": {
            "sentence_start_ms": clip.start_ms,
            "sentence_end_ms": clip.end_ms,
            "cut_start_ms": cut_start,
            "cut_end_ms": cut_end,
            "pad_before_ms": clip.pad_before_ms,
            "pad_after_ms": clip.pad_after_ms,
            "repaired_before_ms": clip.repaired_before_ms,
            "repaired_after_ms": clip.repaired_after_ms,
        },
        "critical": { "start_ms": critical.0, "end_ms": critical.1 },
        "sentence": {
            "text": clip.sentence,
            "course_sentence": course_sentence,
            "speaker": clip.speaker,
            "words": clip.words.iter().map(|w| json!({
                "text": w.text, "at_ms": rel(w.at_ms), "until_ms": rel(w.until_ms),
            })).collect::<Vec<_>>(),
        },
        "subtitles": cues.iter()
            .filter(|c| c.end_ms > cut_start && c.start_ms < cut_end)
            .map(|c| {
                let role = if c.end_ms <= clip.start_ms { "context-before" }
                    else if c.start_ms >= clip.end_ms { "context-after" }
                    else { "sentence" };
                json!({
                    "text": c.text,
                    "at_ms": rel(c.start_ms), "until_ms": rel(c.end_ms),
                    "role": role,
                })
            }).collect::<Vec<_>>(),
        "context_verified": false,
        "context_lines": { "before": ctx_before, "after": ctx_after },
        "transcript": {
            "words": transcript.iter()
                .filter(|w| w.until_ms > cut_start && w.at_ms < cut_end)
                .map(|w| json!({
                    "text": w.text,
                    "at_ms": rel(w.at_ms), "until_ms": rel(w.until_ms),
                    "kind": if w.kind == Kind::AudioEvent { "audio_event" } else { "word" },
                    "speaker": w.speaker, "logprob": w.logprob,
                })).collect::<Vec<_>>(),
        },
        "phonemes": {
            "target_ipa": clip.target_ipa,
            "heard_ipa": clip.heard_ipa,
            "oov": clip.oov,
            "alignment": alignment.unwrap_or(serde_json::Value::Null),
        },
        "verification": {
            "measured": clip.measured,
            "audio_hash": clip.audio_hash,
            "passed": clip.passed,
            "reject": clip.reject,
            "transcript_wer": clip.transcript_wer,
            "ratio": clip.ratio,
            "logp_target_per_phoneme": clip.logp_target_per_phoneme,
            "edge_logp_start": clip.edge_logp_start,
            "edge_logp_end": clip.edge_logp_end,
            "lead_speech": clip.lead_speech,
            "tail_speech": clip.tail_speech,
            "lead_rms": clip.lead_rms,
            "voiced": clip.voiced,
            "audio_event_overlap": clip.audio_event_overlap,
            "clear_before_ms": clip.clear_before_ms,
            "clear_after_ms": clip.clear_after_ms,
            "pad_before_ms": clip.pad_before_ms,
            "pad_after_ms": clip.pad_after_ms,
            "provenance": {
                "inputs": provenance.inputs,
                "cut": provenance.cut,
                "gate": provenance.gate,
                "producers": clip.producers,
            },
        },
        "media": {
            "stamp": stamp,
            "duration_ms": cut_end - cut_start,
            "width": video.width,
            "height": video.height,
            "aspect_ratio": video.aspect_ratio,
            "loudnorm": {
                "measured_i": measured_i,
                "measured_tp": measured_tp,
                "gain_db": gain_db,
                "measured_over": "critical",
            },
            "keyframe_at_critical": true,
            "renditions": {
                "hi": rendition_info(&clip_dir.join("hi.mp4"), video.height.min(MAX_HEIGHT)),
                "lo": rendition_info(&clip_dir.join("lo.mp4"), video.height.min(LO_HEIGHT)),
            },
        },
    });
    let bytes = serde_json::to_vec_pretty(&sidecar)?;
    // Reuse is keyed only by the media stamp. New sidecar-only fields change
    // these bytes, so an old export is refreshed without re-encoding media.
    if reused && old_sidecar == Some(bytes.as_slice()) {
        return Ok(Outcome::Unchanged);
    }
    // Write-then-rename so `meta.json exists` really means the clip is whole.
    let tmp = clip_dir.join("meta.json.tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, clip_dir.join("meta.json"))?;
    Ok(if reused {
        Outcome::Refreshed
    } else {
        Outcome::Rendered
    })
}

/// The full serve pipeline, [`crate::clips`]-style resumable at every stage:
/// re-map clips (skips films whose provenance is current), export videos
/// (skips clip dirs with a finished sidecar), upload to R2 over S3 (skips matching `.uploaded`
/// hashes and verifies stale files against bucket ETags), then export subtitles into yap.
/// The yap export runs last so its freshness/verbatim gates see the verdicts
/// recomputed by the clip re-map, not those from the previous run.
/// Safe to re-run after any interruption.
/// `jobs` is the export concurrency / NVENC session budget (see [`export_clips`]);
/// it does not change re-map or upload concurrency.
pub async fn publish(
    out: PathBuf,
    dest: PathBuf,
    data_root: PathBuf,
    jobs: usize,
    langs: Option<Vec<String>>,
    bucket: String,
) -> Result<()> {
    println!("=== stage 1: clips re-map ===");
    let gate = crate::clips::Gate::default();
    crate::clips::clips_all(out.clone(), 4, 0, None, langs.clone(), gate).await?;

    println!("=== stage 2: video export ===");
    export_clips(out.clone(), dest.clone(), jobs, 0, None, langs.clone()).await?;

    println!("=== stage 3: upload to {bucket} ===");
    let codes: Vec<String> = match &langs {
        Some(l) => l.clone(),
        None => std::fs::read_dir(&dest)?
            .flatten()
            .filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().into_string().ok())
            .collect(),
    };
    let r2 = crate::r2::R2::from_env(&bucket).await?;
    for code in codes {
        upload_lang(&dest.join(&code), &code, &r2).await?;
    }

    println!("=== stage 4: subtitles into yap ===");
    export_yap(out, data_root, langs, false)?;
    Ok(())
}

/// Upload one language's exported clips over S3, with SDK-managed retries.
/// A `.uploaded` marker records each file's xxh3 hash once all three objects
/// land. Matching markers skip work; stale files already matching bucket MD5
/// ETags need no put. Objects are immutable by id and get a forever cache;
/// the index gets a short one.
async fn upload_lang(lang_dir: &Path, code: &str, r2: &crate::r2::R2) -> Result<()> {
    const IMMUTABLE: &str = "public, max-age=31536000, immutable";
    let etags = r2.list_etags(&format!("{code}/")).await?;
    println!("{code}: listed {} keys in bucket", etags.len());
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(lang_dir)?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    let mut uploaded = 0;
    let mut verified = 0;
    let mut skipped = 0;
    // No detached tasks: the first error drops the remaining futures. Markers
    // only land after all three files are up, so interrupted dirs are retried.
    let mut uploads = stream::iter(dirs.iter().map(|dir| {
        let etags = &etags;
        Ok::<_, anyhow::Error>(async move {
            let id = dir.file_name().and_then(|s| s.to_str()).unwrap_or_default();
            if !dir.join("meta.json").exists() {
                return Ok(None); // half-written export
            }
            let files = [
                ("hi", "hi.mp4", "video/mp4"),
                ("lo", "lo.mp4", "video/mp4"),
                ("meta", "meta.json", "application/json"),
            ];
            // Hashing reads ~9 MB per dir; keep it off the stream's task so
            // dirs hash in parallel instead of one after another.
            let hashes = {
                let dir = dir.clone();
                tokio::task::spawn_blocking(move || -> Result<serde_json::Map<_, _>> {
                    let mut hashes = serde_json::Map::new();
                    for (key, name, _) in files {
                        hashes.insert(key.into(), file_hash(&dir.join(name))?.into());
                    }
                    Ok(hashes)
                })
                .await??
            };
            let marked = tokio::fs::read(dir.join(".uploaded"))
                .await
                .ok()
                .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
                .unwrap_or_default();
            let stale: Vec<_> = files
                .iter()
                .filter(|(key, _, _)| marked[*key] != hashes[*key])
                .collect();
            if stale.is_empty() {
                return Ok(Some(UploadOutcome::Skipped));
            }
            let mut did_upload = false;
            for (_, name, content_type) in stale {
                let key = format!("{code}/{id}/{name}");
                let bytes = tokio::fs::read(dir.join(name))
                    .await
                    .with_context(|| format!("reading {key}"))?;
                if !etag_matches(etags.get(&key).map(String::as_str), &bytes) {
                    r2.put(&key, bytes, content_type, IMMUTABLE).await?;
                    did_upload = true;
                }
            }
            tokio::fs::write(dir.join(".uploaded"), serde_json::to_vec(&hashes)?).await?;
            Ok(Some(if did_upload {
                UploadOutcome::Uploaded
            } else {
                UploadOutcome::Verified
            }))
        })
    }))
    .try_buffer_unordered(UPLOAD_JOBS);
    while let Some(outcome) = uploads.try_next().await? {
        match outcome {
            Some(UploadOutcome::Uploaded) => uploaded += 1,
            Some(UploadOutcome::Verified) => verified += 1,
            Some(UploadOutcome::Skipped) => skipped += 1,
            None => continue,
        }
        let done: usize = uploaded + verified + skipped;
        if done.is_multiple_of(250) {
            println!("  {done} processed: {uploaded} uploaded, {verified} verified, {skipped} already up");
        }
    }
    let index = lang_dir.join("index.jsonl");
    if index.exists() {
        r2.put(
            &format!("{code}/index.jsonl"),
            tokio::fs::read(&index).await?,
            "application/x-ndjson",
            "public, max-age=60",
        )
        .await?;
    }
    println!("{code}: {uploaded} clip dirs uploaded, {verified} verified in bucket, {skipped} already up");
    Ok(())
}

/// Counts are per directory: any put makes it uploaded, otherwise all stale
/// files were verified against the listing (or the marker already matched).
enum UploadOutcome {
    Uploaded,
    Verified,
    Skipped,
}

fn etag_matches(etag: Option<&str>, bytes: &[u8]) -> bool {
    etag.is_some_and(|etag| {
        !etag.contains('-')
            && etag
                .bytes()
                .map(|b| b.to_ascii_lowercase())
                .eq(Md5::digest(bytes).iter().flat_map(|b| {
                    let hex = |n: u8| b"0123456789abcdef"[n as usize];
                    [hex(b >> 4), hex(b & 0xf)]
                }))
    })
}

/// Extend the scored cut to neighboring subtitle lines within [`CTX_GAP_MS`].
/// Display times are not speech times: keep the lines and move dirty cuts
/// outward to silence, pulling in any newly overlapping cues until stable.
/// Taxi (`tt0152930`) placed a padded cut 29 ms into "c'est"; widening into
/// the preceding gap now keeps its context lines. Drop an outermost cue only
/// if no gap is within [`CTX_WIDEN_MS`] or the widened cut exceeds [`CTX_CAP_MS`].
/// Scored bounds without context are accepted as-is: the sentence gate already
/// measured their margins. Returns film-absolute bounds and context counts.
fn context_bounds(
    clip_start: i64,
    clip_end: i64,
    scored_start: i64,
    scored_end: i64,
    cues: &[Cue],
    transcript: &[Spoken],
) -> (i64, i64, usize, usize) {
    // Keep all candidates, including those beyond the initial gap limit, so
    // widening can pull them in. Truncating on a drop permanently excludes
    // that cue and everything further out, preventing trim/re-add cycles.
    let mut before: Vec<&Cue> = cues
        .iter()
        .rev()
        .filter(|c| c.end_ms <= clip_start)
        .collect();
    let mut after: Vec<&Cue> = cues.iter().filter(|c| c.start_ms >= clip_end).collect();
    let mut edge = clip_start;
    let mut nb = 0;
    for cue in &before {
        if edge - cue.end_ms > CTX_GAP_MS {
            break;
        }
        edge = cue.start_ms;
        nb += 1;
    }
    let mut edge = clip_end;
    let mut na = 0;
    for cue in &after {
        if cue.start_ms - edge > CTX_GAP_MS {
            break;
        }
        edge = cue.end_ms;
        na += 1;
    }
    loop {
        let s = before[..nb].last().map_or(scored_start, |c| {
            (c.start_ms - CTX_PAD_MS).min(scored_start).max(0)
        });
        let e = after[..na]
            .last()
            .map_or(scored_end, |c| (c.end_ms + CTX_PAD_MS).max(scored_end));
        let s = if nb == 0 {
            Some(s)
        } else {
            widen_context_bound(s, transcript, true)
        };
        let e = if na == 0 {
            Some(e)
        } else {
            widen_context_bound(e, transcript, false)
        };
        if s.is_none() {
            nb -= 1;
            before.truncate(nb);
        }
        if e.is_none() {
            na -= 1;
            after.truncate(na);
        }
        let (Some(s), Some(e)) = (s, e) else { continue };
        let old_counts = (nb, na);
        if nb > 0 {
            while nb < before.len() && before[nb].end_ms > s {
                nb += 1;
            }
        }
        if na > 0 {
            while na < after.len() && after[na].start_ms < e {
                na += 1;
            }
        }
        if (nb, na) != old_counts {
            continue;
        }
        if e - s <= CTX_CAP_MS || (nb == 0 && na == 0) {
            return (s, e, nb, na);
        }
        // Drop whichever outermost line sits furthest from the sentence.
        let d_before = before[..nb].last().map(|c| clip_start - c.start_ms);
        let d_after = after[..na].last().map(|c| c.end_ms - clip_end);
        if d_before >= d_after {
            nb -= 1;
            before.truncate(nb);
        } else {
            na -= 1;
            after.truncate(na);
        }
    }
}

fn widen_context_bound(boundary: i64, transcript: &[Spoken], start: bool) -> Option<i64> {
    let clean = if start {
        clean_context_start(boundary, transcript)
    } else {
        clean_context_end(boundary, transcript)
    };
    if clean {
        return Some(boundary);
    }
    let mut words: Vec<_> = transcript
        .iter()
        .filter(|w| w.kind == Kind::Word)
        .map(|w| (w.at_ms, w.until_ms))
        .collect();
    words.sort_unstable();
    let mut end = 0;
    let mut nearest = None;
    // Taking the maximum end merges overlapping words into speech runs.
    // The sentinel also exposes the silence after the final word.
    for (next_start, next_end) in words.into_iter().chain([(i64::MAX, i64::MAX)]) {
        let gap = next_start - end;
        if gap >= CTX_CLEAN_MS {
            let pad = CTX_PAD_MS.min(gap / 2);
            let candidate = if start { next_start - pad } else { end + pad };
            let distance = if start {
                boundary - candidate
            } else {
                candidate - boundary
            };
            if (0..=CTX_WIDEN_MS).contains(&distance)
                && nearest.is_none_or(|(best, _)| distance < best)
            {
                nearest = Some((distance, candidate));
            }
        }
        end = end.max(next_end);
    }
    nearest.map(|(_, bound)| bound)
}

fn clean_context_start(boundary: i64, transcript: &[Spoken]) -> bool {
    let words = transcript.iter().filter(|word| word.kind == Kind::Word);
    !words
        .clone()
        .any(|word| word.at_ms < boundary && word.until_ms > boundary)
        && words
            .filter(|word| word.until_ms <= boundary)
            .map(|word| word.until_ms)
            .max()
            .is_none_or(|end| boundary - end >= CTX_CLEAN_MS)
}

fn clean_context_end(boundary: i64, transcript: &[Spoken]) -> bool {
    let words = transcript.iter().filter(|word| word.kind == Kind::Word);
    !words
        .clone()
        .any(|word| word.at_ms < boundary && word.until_ms > boundary)
        && words
            .filter(|word| word.at_ms >= boundary)
            .map(|word| word.at_ms)
            .min()
            .is_none_or(|start| start - boundary >= CTX_CLEAN_MS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn alignment_uses_shared_key_without_audio_and_counts_only_cache_failures() {
        let root = tempfile::tempdir().unwrap();
        let store = osmo::Store::open(root.path());
        let mut clip: Clip = serde_json::from_value(json!({
            "model": null, "g2p": "old-renderer", "audio_hash": 42, "measured": true,
            "sentence": "fixture", "imdb_id": "tt0", "start_ms": 0, "end_ms": 1000,
            "pad_before_ms": 0, "pad_after_ms": 0, "repaired_before_ms": 0, "repaired_after_ms": 0,
            "words": [], "speaker": null, "transcript_wer": 0.0, "audio_event_overlap": false,
            "clear_before_ms": 0, "clear_after_ms": 0, "target_ipa": ["a"], "oov": [],
            "ratio": 0.0, "logp_target_per_phoneme": -1.0, "edge_logp_start": -1.0, "edge_logp_end": -1.0,
            "lead_speech": null, "tail_speech": null, "lead_rms": null, "voiced": 1.0,
            "heard_ipa": ["a"], "passed": true, "reject": null
        })).unwrap();
        let failures = AtomicUsize::new(0);
        assert!(align_phonemes(None, &failures, &clip, 0).await.is_none());
        assert_eq!(failures.load(Ordering::Relaxed), 0);
        assert!(align_phonemes(Some(&store), &failures, &clip, 0)
            .await
            .is_none());
        assert_eq!(failures.load(Ordering::Relaxed), 1);
        let key = crate::clips::clip_key(42, &clip.target_ipa);
        store.write(&key, b"corrupt").await.unwrap();
        assert!(align_phonemes(Some(&store), &failures, &clip, 0)
            .await
            .is_none());
        assert_eq!(failures.load(Ordering::Relaxed), 2);
        use base64::Engine;
        use std::io::Write;
        let mut encoder =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(&[0x00, 0xc0, 0x00, 0xbc]).unwrap();
        let response = json!({"envelope": {}, "item": {"phonemes": [], "frame_matrix": {
            "shape": [1, 2], "dtype": "float16", "encoding": "zlib+base64", "blank_id": 0,
            "vocab": ["<pad>", "a"], "data": base64::engine::general_purpose::STANDARD.encode(encoder.finish().unwrap())
        }}});
        store
            .write(&key, &serde_json::to_vec(&response).unwrap())
            .await
            .unwrap();
        let alignment = align_phonemes(Some(&store), &failures, &clip, 10)
            .await
            .unwrap();
        assert_eq!(alignment[0]["ph"], "a");
        assert_eq!(failures.load(Ordering::Relaxed), 2);
        // A valid matrix with no alignable path is not a cache problem.
        clip.target_ipa = vec!["a".into(), "a".into()];
        store
            .write(
                &crate::clips::clip_key(42, &clip.target_ipa),
                &serde_json::to_vec(&response).unwrap(),
            )
            .await
            .unwrap();
        assert!(align_phonemes(Some(&store), &failures, &clip, 0)
            .await
            .is_none());
        clip.ratio = None;
        assert!(align_phonemes(Some(&store), &failures, &clip, 0)
            .await
            .is_none());
        assert_eq!(failures.load(Ordering::Relaxed), 2);
    }

    fn example_args(mode: EncodeMode, hdr: bool) -> Vec<OsString> {
        rendition_args(
            mode,
            Path::new("/film with spaces.mkv"),
            2,
            hdr,
            1234,
            6789,
            -2.345,
            0.456,
            Path::new("/clips/test"),
        )
    }

    #[test]
    fn rendition_commands_preserve_cpu_and_order_gpu_fallbacks() {
        assert_eq!(
            ENCODE_MODES,
            [EncodeMode::Gpu, EncodeMode::Cpu, EncodeMode::CpuCoreOnly]
        );
        for hdr in [false, true] {
            for mode in ENCODE_MODES {
                let mut expected = vec!["-v", "error", "-y"];
                match mode {
                    EncodeMode::Gpu => {
                        expected.extend(["-hwaccel", "cuda", "-hwaccel_output_format", "cuda"])
                    }
                    EncodeMode::Cpu => {}
                    EncodeMode::CpuCoreOnly => expected.extend(["-core_only", "1"]),
                }
                let tonemap = match (mode, hdr) {
                    (EncodeMode::Gpu, true) => "tonemap_cuda=tonemap=hable:desat=0:format=yuv420p:matrix=bt709:primaries=bt709:transfer=bt709,",
                    (_, true) => "zscale=t=linear:npl=100,format=gbrpf32le,zscale=p=bt709,tonemap=hable:desat=0,zscale=t=bt709:m=bt709:r=tv,",
                    (_, false) => "",
                };
                let scaling = if mode == EncodeMode::Gpu {
                    "scale_cuda=w=-2:h='min(1440,ih)':format=yuv420p,split=2[vh][v0];[v0]scale_cuda=w=-2:h='min(480,ih)',hwdownload,format=yuv420p[vl]"
                } else {
                    "scale=-2:'min(1440,ih)':flags=lanczos,format=yuv420p,split=2[vh][v0];[v0]scale=-2:'min(480,ih)'[vl]"
                };
                let filter = format!("[0:v:0]{tonemap}{scaling};[0:a:2]aformat=channel_layouts=stereo,volume=-2.35dB,aresample=48000,asplit=2[ah][al]");
                expected.extend([
                    "-ss",
                    "1.234",
                    "-t",
                    "5.555",
                    "-i",
                    "/film with spaces.mkv",
                    "-filter_complex",
                    &filter,
                    "-map",
                    "[vh]",
                    "-map",
                    "[ah]",
                ]);
                let cq = HI_CQ.to_string();
                if mode == EncodeMode::Gpu {
                    expected.extend([
                        "-c:v",
                        "h264_nvenc",
                        "-cq",
                        &cq,
                        "-preset",
                        HI_GPU_PRESET,
                        "-forced-idr",
                        "1",
                    ]);
                } else {
                    expected.extend(["-c:v", "libx264", "-crf", "19", "-preset", "medium"]);
                }
                expected.extend([
                    "-c:a",
                    "aac",
                    "-b:a",
                    "160k",
                    "-force_key_frames",
                    "0.456",
                    "-movflags",
                    "+faststart",
                    "/clips/test/hi.mp4",
                    "-map",
                    "[vl]",
                    "-map",
                    "[al]",
                    "-c:v",
                    "libx264",
                    "-crf",
                    "27",
                    "-preset",
                    "veryfast",
                    "-c:a",
                    "aac",
                    "-b:a",
                    "96k",
                    "-force_key_frames",
                    "0.456",
                    "-movflags",
                    "+faststart",
                    "/clips/test/lo.mp4",
                ]);
                assert_eq!(
                    example_args(mode, hdr),
                    expected.into_iter().map(OsString::from).collect::<Vec<_>>(),
                    "{mode:?}, hdr={hdr}"
                );
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn rendition_paths_preserve_non_utf8() {
        use std::os::unix::ffi::OsStringExt;
        let source = PathBuf::from(OsString::from_vec(b"/film-\xff.mkv".to_vec()));
        let dest = PathBuf::from(OsString::from_vec(b"/clips-\xfe".to_vec()));
        let args = rendition_args(EncodeMode::Gpu, &source, 0, false, 0, 1000, 0.0, 0.0, &dest);
        let input = args.iter().position(|arg| arg == "-i").unwrap();
        assert_eq!(args[input + 1], source.as_os_str());
        assert_eq!(args.last().unwrap(), dest.join("lo.mp4").as_os_str());
    }

    fn cue(start_ms: i64, end_ms: i64) -> Cue {
        Cue {
            start_ms,
            end_ms,
            text: String::new(),
        }
    }

    fn word(at_ms: i64, until_ms: i64) -> Spoken {
        Spoken {
            text: String::new(),
            at_ms,
            until_ms,
            kind: Kind::Word,
            speaker: None,
            logprob: None,
        }
    }

    #[test]
    fn context_widens_taxi_shape_cut_inside_word() {
        let cues = [cue(6_000, 7_000)];
        // The padded boundary is 5_850, 29 ms after this word began.
        let transcript = [word(5_000, 5_571), word(5_821, 5_901)];

        assert_eq!(
            context_bounds(8_000, 9_000, 7_700, 9_150, &cues, &transcript),
            (5_696, 9_150, 1, 0)
        );
    }

    #[test]
    fn context_keeps_boundary_after_clean_gap() {
        let cues = [cue(6_000, 7_000)];
        let transcript = [word(5_200, 5_350)];

        assert_eq!(
            context_bounds(8_000, 9_000, 7_700, 9_150, &cues, &transcript),
            (5_850, 9_150, 1, 0)
        );
    }

    #[test]
    fn context_drops_dirty_trailing_cue_without_readding_it() {
        let cues = [cue(2_500, 3_000), cue(3_500, 4_000), cue(6_100, 6_500)];
        // Speech continues beyond the 1,500 ms budget after 4,150.
        let transcript = [word(4_250, 6_000)];

        assert_eq!(
            context_bounds(1_000, 2_000, 700, 2_150, &cues, &transcript),
            (700, 3_150, 0, 1)
        );
    }

    #[test]
    fn context_widening_pulls_in_overlapping_neighbour() {
        // The second cue is initially beyond CTX_GAP_MS, but the widened
        // end (starting from the scored end) overlaps it. Including it then
        // moves the padded end to 5,550.
        let cues = [cue(2_500, 3_000), cue(5_050, 5_400)];
        let transcript = [word(3_900, 4_500), word(4_600, 5_100)];
        assert_eq!(
            context_bounds(1_000, 2_000, 700, 4_000, &cues, &transcript),
            (700, 5_550, 0, 2)
        );
    }
}

/// Per-phoneme spans in clip-relative ms, from the cached frame matrix.
async fn align_phonemes(
    cache: Option<&osmo::Store>,
    alignment_cache_failures: &AtomicUsize,
    clip: &Clip,
    scored_offset_ms: i64,
) -> Option<serde_json::Value> {
    let store = cache?;
    if !clip.measured || clip.ratio.is_none() || clip.target_ipa.is_empty() {
        return None;
    }
    let cached = match clip.audio_hash {
        Some(hash) => {
            phoneme_verify::cached_frame_matrix(
                store,
                &crate::clips::clip_key(hash, &clip.target_ipa),
            )
            .await
        }
        None => None,
    };
    let frames = match cached {
        Some(Ok(frames)) => frames,
        _ => {
            alignment_cache_failures.fetch_add(1, Ordering::Relaxed);
            return None;
        }
    };
    let present: Vec<&String> = clip
        .target_ipa
        .iter()
        .filter(|t| frames.id(t).is_some())
        .collect();
    let ids: Vec<usize> = present.iter().filter_map(|t| frames.id(t)).collect();
    let spans = frames.force_align(&ids)?;
    // Frames cover the wav after symmetric zero-padding to the model's
    // 0.6s minimum; undo that padding to place frames in wav time.
    let wav_ms = clip.end_ms + clip.pad_after_ms - (clip.start_ms - clip.pad_before_ms).max(0);
    let padded_ms = wav_ms.max(600);
    let lead_pad_ms = (padded_ms - wav_ms) / 2;
    let ms_per_frame = padded_ms as f64 / frames.frames as f64;
    Some(
        spans
            .iter()
            .zip(&present)
            .map(|(s, ph)| {
                let at = (s.start_frame as f64 * ms_per_frame) as i64 - lead_pad_ms;
                let until = ((s.end_frame + 1) as f64 * ms_per_frame) as i64 - lead_pad_ms;
                json!({
                    "ph": ph,
                    "at_ms": at + scored_offset_ms,
                    "until_ms": until + scored_offset_ms,
                    "logp": s.logp_mean,
                })
            })
            .collect(),
    )
}

#[derive(Debug, Clone)]
struct VideoProbe {
    width: i64,
    height: i64,
    /// Display width divided by display height, including anamorphic SAR.
    aspect_ratio: f64,
    /// PQ / HLG sources are tonemapped to SDR bt709 for h264 playback.
    hdr: bool,
    /// Container runtime; part of the media stamp so a different cut of the
    /// film under the same name and size still reads as a new source.
    duration_ms: i64,
}

fn probe_video(path: &Path) -> Result<VideoProbe> {
    let out = Command::new("ffprobe")
        .args(["-v", "error", "-select_streams", "v:0", "-show_entries"])
        .args([
            "stream=width,height,sample_aspect_ratio,display_aspect_ratio,color_transfer:format=duration",
            "-of",
            "json",
        ])
        .arg(path)
        .output()
        .context("ffprobe failed to start")?;
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).context("ffprobe output")?;
    let stream = v["streams"].get(0).context("no video stream")?;
    let width = stream["width"].as_i64().context("no width")?;
    let height = stream["height"].as_i64().context("no height")?;
    let ratio = |name: &str| {
        let raw = stream[name].as_str()?;
        let (numerator, denominator) = raw.split_once(':')?;
        let numerator = numerator.parse::<f64>().ok()?;
        let denominator = denominator.parse::<f64>().ok()?;
        (numerator.is_finite() && denominator.is_finite() && numerator > 0.0 && denominator > 0.0)
            .then_some(numerator / denominator)
    };
    let aspect_ratio = ratio("display_aspect_ratio")
        .or_else(|| ratio("sample_aspect_ratio").map(|sar| width as f64 * sar / height as f64))
        .unwrap_or(width as f64 / height as f64);
    let transfer = stream["color_transfer"].as_str().unwrap_or("");
    let duration_ms = v["format"]["duration"]
        .as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .map(|s| (s * 1000.0).round() as i64)
        .context("no container duration")?;
    Ok(VideoProbe {
        width,
        height,
        aspect_ratio,
        hdr: matches!(transfer, "smpte2084" | "arib-std-b67"),
        duration_ms,
    })
}

/// The audio-relative stream index (`-map 0:a:N`) recorded at extraction,
/// verified against the file on disk — a remux can reorder audio tracks
/// under an unchanged filename, and the wrong language track must fail loud.
/// An in-place transcode of the same track is fine
/// ([`AudioStreamIdentity::same_track`]).
fn audio_stream_index(audio_json: &Path, video: &Path, probe: &VideoProbe) -> Result<u32> {
    let v: serde_json::Value = serde_json::from_slice(&std::fs::read(audio_json)?)?;
    // Same film first: the extraction names the file and its runtime, and a
    // stream check means nothing against a different cut under the same name
    // (the refresh evicts such films, but publish must not trust that).
    let filename = video
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or_default();
    let recorded_ms = v["duration_ms"].as_i64().unwrap_or_default();
    if v["filename"] != filename || (recorded_ms - probe.duration_ms).abs() > 250 {
        bail!(
            "extraction was from {} ({}ms), the film is now {filename} ({}ms)",
            v["filename"].as_str().unwrap_or_default(),
            recorded_ms,
            probe.duration_ms
        );
    }
    let recorded: AudioStreamIdentity =
        serde_json::from_value(v["stream"].clone()).context("audio.json has no stream")?;
    let now = crate::sync::audio_stream_identity(video, recorded.stream_index)?;
    if !recorded.same_track(&now) {
        bail!(
            "audio stream a:{} is now {}/{}ch, extraction saw {}/{}ch — remux changed under {}",
            recorded.stream_index,
            now.codec,
            now.channels,
            recorded.codec,
            recorded.channels,
            video.display()
        );
    }
    Ok(recorded.stream_index as u32)
}

/// EBU R128 integrated loudness + true peak of one span, through the same
/// stereo downmix the encode applies.
fn measure_loudness(
    path: &Path,
    audio_stream: u32,
    start_ms: i64,
    dur_ms: i64,
) -> Result<(f64, f64)> {
    let out = Command::new("ffmpeg")
        .args(["-nostats", "-hide_banner", "-ss"])
        .arg(format!("{:.3}", start_ms as f64 / 1000.0))
        .args(["-t", &format!("{:.3}", dur_ms as f64 / 1000.0), "-i"])
        .arg(path)
        .args(["-map", &format!("0:a:{audio_stream}")])
        .args(["-af", "aformat=channel_layouts=stereo,ebur128=peak=true"])
        .args(["-f", "null", "-"])
        .output()
        .context("ffmpeg (loudness) failed to start")?;
    let text = String::from_utf8_lossy(&out.stderr);
    let summary = text
        .rsplit("Summary:")
        .next()
        .context("no ebur128 summary")?;
    let grab = |label: &str| -> Option<f64> {
        summary
            .lines()
            .find(|l| l.trim_start().starts_with(label))?
            .split_whitespace()
            .find_map(|t| t.parse::<f64>().ok())
    };
    let i = grab("I:").context("no integrated loudness in summary")?;
    // "Peak:" appears under both "Sample peak" and "True peak"; the last one
    // is the true peak.
    let tp = summary
        .lines()
        .filter(|l| l.trim_start().starts_with("Peak:"))
        .filter_map(|l| l.split_whitespace().find_map(|t| t.parse::<f64>().ok()))
        .next_back()
        .context("no true peak in summary")?;
    Ok((i, tp))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EncodeMode {
    Gpu,
    Cpu,
    CpuCoreOnly,
}

// First keep decode, tonemap and hi scaling/encoding on CUDA, downloading only
// the small lo branch. Unsupported inputs or exhausted GPU sessions fall back
// to the original full CPU encode. Finally retry that CPU graph with DTS core
// audio: corrupt DTS-HD XLL frames can change 7.1 to 5.1 in the keyframe pre-roll,
// which a complex graph cannot reinitialize. Core-only keeps a stable layout;
// non-DTS decoders ignore it. A clip fails only after all three stages fail.
const ENCODE_MODES: [EncodeMode; 3] = [EncodeMode::Gpu, EncodeMode::Cpu, EncodeMode::CpuCoreOnly];

/// One decode of the source segment, two encodes: `hi.mp4` (≤1440p, quality)
/// and `lo.mp4` (≤480p, fast first paint). Both get a keyframe at the
/// critical start and faststart moov.
#[allow(clippy::too_many_arguments)]
fn encode_renditions(
    path: &Path,
    audio_stream: u32,
    video: &VideoProbe,
    cut_start: i64,
    cut_end: i64,
    gain_db: f64,
    crit_s: f64,
    clip_dir: &Path,
) -> Result<()> {
    let mut failures = Vec::new();
    for mode in ENCODE_MODES {
        match mode {
            EncodeMode::Gpu => {}
            EncodeMode::Cpu => eprintln!(
                "  retrying with CPU encode: {} ({})",
                clip_dir.display(),
                failures.last().expect("GPU attempted")
            ),
            EncodeMode::CpuCoreOnly => eprintln!(
                "  retrying with core-only audio decode: {} ({})",
                clip_dir.display(),
                failures.last().expect("CPU attempted")
            ),
        }
        let args = rendition_args(
            mode,
            path,
            audio_stream,
            video.hdr,
            cut_start,
            cut_end,
            gain_db,
            crit_s,
            clip_dir,
        );
        match Command::new("ffmpeg").args(args).output() {
            Ok(output) if output.status.success() => return Ok(()),
            Ok(output) => failures.push(format!(
                "{mode:?} ({}): {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            )),
            Err(error) => failures.push(format!("{mode:?} failed to start: {error}")),
        }
    }
    bail!(
        "ffmpeg encode failed for {} after GPU, CPU and CPU core-only attempts:\n{}",
        clip_dir.display(),
        failures.join("\n")
    );
}

/// Pure command construction keeps fallback order and every output option
/// testable without a GPU, ffmpeg, or a source film. Paths remain lossless.
#[allow(clippy::too_many_arguments)]
fn rendition_args(
    mode: EncodeMode,
    path: &Path,
    audio_stream: u32,
    hdr: bool,
    cut_start: i64,
    cut_end: i64,
    gain_db: f64,
    crit_s: f64,
    clip_dir: &Path,
) -> Vec<OsString> {
    let tonemap = match (mode, hdr) {
        (EncodeMode::Gpu, true) => "tonemap_cuda=tonemap=hable:desat=0:format=yuv420p:matrix=bt709:primaries=bt709:transfer=bt709,",
        (_, true) => "zscale=t=linear:npl=100,format=gbrpf32le,zscale=p=bt709,tonemap=hable:desat=0,zscale=t=bt709:m=bt709:r=tv,",
        (_, false) => "",
    };
    let video_filter = if mode == EncodeMode::Gpu {
        // `format=yuv420p` on the hi scale is what makes the lo branch's
        // `hwdownload` legal: a hardware frame can only be downloaded in the
        // format it already holds, and the decoder hands us nv12/p010. HDR
        // clips get it from `tonemap_cuda` too, but SDR clips have no tonemap
        // stage, so without this every SDR film fell back to the CPU encode.
        format!("[0:v:0]{tonemap}scale_cuda=w=-2:h='min({MAX_HEIGHT},ih)':format=yuv420p,split=2[vh][v0];[v0]scale_cuda=w=-2:h='min({LO_HEIGHT},ih)',hwdownload,format=yuv420p[vl]")
    } else {
        format!("[0:v:0]{tonemap}scale=-2:'min({MAX_HEIGHT},ih)':flags=lanczos,format=yuv420p,split=2[vh][v0];[v0]scale=-2:'min({LO_HEIGHT},ih)'[vl]")
    };
    let filter = format!("{video_filter};[0:a:{audio_stream}]aformat=channel_layouts=stereo,volume={gain_db:.2}dB,aresample=48000,asplit=2[ah][al]");
    let mut args: Vec<OsString> = ["-v", "error", "-y"].map(OsString::from).into();
    match mode {
        EncodeMode::Gpu => {
            args.extend(["-hwaccel", "cuda", "-hwaccel_output_format", "cuda"].map(OsString::from))
        }
        EncodeMode::Cpu => {}
        EncodeMode::CpuCoreOnly => args.extend(["-core_only", "1"].map(OsString::from)),
    }
    args.extend([
        OsString::from("-ss"),
        format!("{:.3}", cut_start as f64 / 1000.0).into(),
        "-t".into(),
        format!("{:.3}", (cut_end - cut_start) as f64 / 1000.0).into(),
        "-i".into(),
        path.as_os_str().to_owned(),
        "-filter_complex".into(),
        filter.into(),
    ]);
    let key = format!("{crit_s:.3}");
    for (hi, video_label, audio_label, filename, aac) in [
        (true, "[vh]", "[ah]", "hi.mp4", HI_AAC),
        (false, "[vl]", "[al]", "lo.mp4", LO_AAC),
    ] {
        args.extend(["-map", video_label, "-map", audio_label].map(OsString::from));
        if hi && mode == EncodeMode::Gpu {
            args.extend(
                [
                    "-c:v",
                    "h264_nvenc",
                    "-cq",
                    &HI_CQ.to_string(),
                    "-preset",
                    HI_GPU_PRESET,
                    "-forced-idr",
                    "1",
                ]
                .map(OsString::from),
            );
        } else {
            let crf = if hi { HI_CRF } else { LO_CRF }.to_string();
            let preset = if hi { HI_PRESET } else { LO_PRESET };
            args.extend(["-c:v", "libx264", "-crf", &crf, "-preset", preset].map(OsString::from));
        }
        args.extend(
            [
                "-c:a",
                "aac",
                "-b:a",
                aac,
                "-force_key_frames",
                &key,
                "-movflags",
                "+faststart",
            ]
            .map(OsString::from),
        );
        args.push(clip_dir.join(filename).into_os_string());
    }
    args
}

fn file_hash(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path).with_context(|| format!("hashing {}", path.display()))?;
    Ok(format!("{:016x}", xxhash_rust::xxh3::xxh3_64(&bytes)))
}

fn rendition_info(file: &Path, height: i64) -> serde_json::Value {
    json!({
        "file": file.file_name().and_then(|s| s.to_str()),
        "height": height,
        "bytes": std::fs::metadata(file).map(|m| m.len()).unwrap_or(0),
    })
}

/// One line per exported clip, rebuilt from the sidecars.
fn write_index(lang_dir: &Path) -> Result<usize> {
    // A language whose films all failed or passed nothing still gets its
    // (empty) index: the queue named it, so the served listing must too.
    std::fs::create_dir_all(lang_dir)?;
    let mut rows: Vec<(String, serde_json::Value)> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(lang_dir) {
        for entry in entries.flatten() {
            let meta = entry.path().join("meta.json");
            let Ok(bytes) = std::fs::read(&meta) else {
                continue;
            };
            let m: serde_json::Value = serde_json::from_slice(&bytes)?;
            rows.push((
                m["id"].as_str().unwrap_or_default().to_string(),
                json!({
                    "id": m["id"],
                    "imdb_id": m["film"]["imdb_id"],
                    "title": m["film"]["title"],
                    "sentence": m["sentence"]["text"],
                    "course_sentence": m["sentence"]["course_sentence"],
                    "duration_ms": m["media"]["duration_ms"],
                    "critical": m["critical"],
                    "clear_before_ms": m["verification"]["clear_before_ms"],
                    "clear_after_ms": m["verification"]["clear_after_ms"],
                    "pad_before_ms": m["verification"]["pad_before_ms"],
                    "pad_after_ms": m["verification"]["pad_after_ms"],
                    "aspect_ratio": m["media"]["aspect_ratio"],
                    "hi_bytes": m["media"]["renditions"]["hi"]["bytes"],
                    "lo_bytes": m["media"]["renditions"]["lo"]["bytes"],
                }),
            ));
        }
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0));
    let n = rows.len();
    let body: String = rows.into_iter().map(|(_, v)| format!("{v}\n")).collect();
    std::fs::write(lang_dir.join("index.jsonl"), body)?;
    Ok(n)
}

#[cfg(test)]
mod upload_tests {
    use super::etag_matches;

    #[test]
    fn listing_etag_matches_only_identical_single_part_bytes() {
        let etag = "900150983cd24fb0d6963f7d28e17f72";
        assert!(etag_matches(Some(etag), b"abc"));
        assert!(!etag_matches(Some(etag), b"changed"));
        assert!(!etag_matches(None, b"abc"));
        assert!(!etag_matches(Some(&format!("{etag}-1")), b"abc"));
        assert!(!etag_matches(Some(""), b"abc"));
    }

    #[test]
    fn md5_keeps_leading_zeroes() {
        assert!(etag_matches(Some("0cc175b9c0f1b6a831c399e269772661"), b"a"));
    }
}

/// Missing files are expected on first export; other read failures must not
/// masquerade as missing metadata or an outdated subtitle.
fn read_optional(path: &std::path::Path) -> Result<Option<Vec<u8>>> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e).with_context(|| format!("reading {}", path.display())),
    }
}

/// Append only missing IDs, retaining every existing byte (including unknown
/// fields and blank lines). Return whether a row was/would be appended.
fn append_movie_metadata(
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

/// The corpus subtitle as yap must receive it: re-serialised from the cues the
/// corpus itself reads. Disc and sidecar tracks are copied verbatim into
/// `subtitle.srt`, and some carry a blank line inside a two-speaker cue;
/// [`parse_cues`] drops the orphaned half, so no clip was ever cut for it,
/// but yap's stricter parser fails the whole file — and a raw SRT that
/// fails to parse takes the entire course build down with it.
fn yap_srt(srt: &str) -> Result<String> {
    let normalized = crate::sync::write_cues(&crate::sync::parse_cues(srt));
    movie_subtitles::parse_srt(&normalized)
        .context("normalised subtitle is not parseable by yap")?;
    Ok(normalized)
}

pub fn export_yap(
    out: PathBuf,
    data_root: PathBuf,
    langs: Option<Vec<String>>,
    dry_run: bool,
) -> Result<()> {
    use crate::verbatim::Verdict;

    // Every sentence handed to yap is paid for again when generate-data
    // rebuilds that course, and the English course has no learners
    // (Andre, 2026-09-19) — so eng is opt-in rather than default.
    let wanted = |course: &str| match &langs {
        Some(langs) => langs.iter().any(|l| l == course),
        None => course != "eng",
    };

    let (mut written, mut kept, mut downloaded, mut unverified, mut rows) = (0, 0, 0, 0, 0);
    for movie in read_plan(&out)? {
        let Some(course) = course_dir(&movie.original_language) else {
            continue;
        };
        if !wanted(course) {
            continue;
        }
        // Writing a synced download back to its raw input invalidates its
        // source stamp on the next inventory, causing perpetual re-syncing.
        if matches!(movie.source, Source::Downloaded { .. }) {
            downloaded += 1;
            continue;
        }
        let dir = out.join(&movie.imdb_id);
        // Same test `clips` applies: a report about the current subtitle and
        // transcript, at the course's bar — a replaced subtitle or a fresh
        // transcript leaves a stale verdict that must not authorise an export.
        let verified = output_is_fresh(&movie, &dir)
            && current_verdict(&dir, course) == Some(Verdict::Verbatim);
        if !verified {
            unverified += 1;
            continue;
        }
        let language = Language::from_code(course).context("unknown course language")?;
        let movies = data_root
            .join(language.corpus_code())
            .join("sentence-sources/movies");
        let dest = movies
            .join("subtitles-raw")
            .join(format!("{}.srt", movie.imdb_id));
        let srt = yap_srt(&std::fs::read_to_string(dir.join("subtitle.srt"))?)
            .with_context(|| format!("{}: subtitle.srt", movie.imdb_id))?;
        let identical = read_optional(&dest)?.as_deref() == Some(srt.as_bytes());
        let metadata = language_utils::MovieMetadataBasic {
            id: movie.imdb_id.clone(),
            title: movie.title.clone(),
            year: movie.year,
            original_language: Some(language.iso_639_1().to_owned()),
            rotten_tomatoes_score: None,
        };
        let appended = append_movie_metadata(&movies.join("metadata.jsonl"), &metadata, dry_run)?;
        rows += usize::from(appended);
        if identical {
            kept += 1;
        } else {
            if !dry_run {
                std::fs::create_dir_all(dest.parent().context("subtitle destination parent")?)?;
                std::fs::write(&dest, srt)?;
            }
            written += 1;
        }
        if dry_run {
            println!(
                "  {} {} ({}) — {} {}; metadata {}",
                movie.imdb_id,
                movie.title,
                course,
                if identical { "keep" } else { "write" },
                dest.display(),
                if appended { "append" } else { "keep" },
            );
        }
    }
    println!(
        "{}{written} subtitles written, {kept} kept, {downloaded} downloaded skipped, {unverified} unverified skipped, {rows} metadata rows appended",
        if dry_run { "dry-run (would): " } else { "" },
    );
    Ok(())
}

#[cfg(test)]
mod export_yap_tests {
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
    fn yap_srt_drops_orphan_blocks_and_parses() {
        // A disc track with a blank line inside a two-speaker cue: the corpus
        // reads two cues and an orphan; yap's parser rejects the file outright.
        let raw = "1\n00:00:01,000 --> 00:00:02,000\n-Tu as de la fièvre.\n\n-Non, j'ai chaud.\n\n2\n00:00:03,000 --> 00:00:04,000\nBonjour tout le monde.\n\n";
        assert!(movie_subtitles::parse_srt(raw).is_err());
        let normalized = yap_srt(raw).unwrap();
        assert_eq!(normalized, "1\n00:00:01,000 --> 00:00:02,000\n-Tu as de la fièvre.\n\n2\n00:00:03,000 --> 00:00:04,000\nBonjour tout le monde.\n\n");
        assert_eq!(movie_subtitles::parse_srt(&normalized).unwrap().len(), 2);
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
}
