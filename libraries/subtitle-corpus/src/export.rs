//! Serve-ready clip export: every passing clip becomes a directory of
//! `hi.mp4` + `lo.mp4` + `meta.json`, cut generously from the source film
//! (neighboring subtitle lines as context) with the sidecar — not the file
//! boundary — defining what the clip *is*. Schema and rationale:
//! `docs/clip-sidecar.md`.
//!
//! Everything in the sidecar comes from artifacts already on disk; the
//! forced alignment re-reads the cached frame matrices under
//! [`phoneme_verify::set_cache_only`], so an export run spends no inference.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use anyhow::{bail, Context, Result};
use language_utils::Language;
use movie_subtitles::cleanup_subtitle_text;
use movie_subtitles::segment::SubtitleSegmenter;
use phoneme_verify::VerifyContext;
use serde_json::json;
use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;

use crate::clips::{clips_path, read_clips, subtitle_sentences, Clip, Provenance};
use crate::cues::{load_transcript, parse_cues, repair_latin_homoglyphs, slice_wav_padded};
use crate::library::{course_dir, read_plan, truncate, Movie};
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
/// The hi rendition is never upscaled and never taller than this.
const MAX_HEIGHT: i64 = 1440;
const LO_HEIGHT: i64 = 480;
const HI_CRF: u32 = 19;
const HI_PRESET: &str = "medium";
const HI_AAC: &str = "160k";
const LO_CRF: u32 = 27;
const LO_PRESET: &str = "veryfast";
const LO_AAC: &str = "96k";
/// R2 puts in flight at once during upload (see `upload_lang`).
const UPLOAD_JOBS: usize = 8;

/// Sidecar `format` field.
const SIDECAR_FORMAT: u32 = 2;

/// Everything that shapes the rendered files, in one comparable string.
/// Built from the constants so no tweak can be forgotten; anything that
/// changes the output must appear here or in the per-film stamp.
fn encode_recipe() -> String {
    format!(
        "hi h264 crf{HI_CRF} {HI_PRESET} aac{HI_AAC} le{MAX_HEIGHT}p | \
         lo crf{LO_CRF} {LO_PRESET} aac{LO_AAC} le{LO_HEIGHT}p | \
         loudnorm I{TARGET_I} TP{TP_CEIL} critical linear | \
         ctx gap{CTX_GAP_MS} cap{CTX_CAP_MS} pad{CTX_PAD_MS} | \
         keyframe@critical | tonemap zscale hable bt709 | lanczos yuv420p"
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

fn plan_cut(clip: &Clip, cues: &[Cue]) -> Cut {
    let scored_start = (clip.start_ms - clip.pad_before_ms).max(0);
    let scored_end = clip.end_ms + clip.pad_after_ms;
    let (cut_start, cut_end, ctx_before, ctx_after) =
        context_bounds(clip, scored_start, scored_end, cues);
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

/// Sidecars written before `media.stamp` existed (2026-09) state the same
/// facts under `export`, `source` and `critical` — height, HDR and runtime
/// follow from the same file. Honouring them lets the first publish after
/// the change reuse ~19k renditions instead of re-encoding them. Remove
/// once every served sidecar carries `media.stamp`.
fn legacy_stamp_matches(old: &serde_json::Value, stamp: &serde_json::Value) -> bool {
    let (e, v) = (&old["export"], &old["export"]["video"]);
    e["recipe"] == stamp["recipe"]
        && v["filename"] == stamp["video"]["filename"]
        && v["bytes"] == stamp["video"]["bytes"]
        && v["audio_stream"] == stamp["video"]["audio_stream"]
        && old["source"]["cut_start_ms"] == stamp["cut"]["start_ms"]
        && old["source"]["cut_end_ms"] == stamp["cut"]["end_ms"]
        && old["critical"]["start_ms"] == stamp["cut"]["critical_start_ms"]
        && old["critical"]["end_ms"] == stamp["cut"]["critical_end_ms"]
}

pub async fn export_clips(
    out: PathBuf,
    dest: PathBuf,
    jobs: usize,
    limit: usize,
    imdb: Option<String>,
    langs: Option<Vec<String>>,
) -> Result<()> {
    // Cache misses must fail the clip's alignment block, never call Modal.
    phoneme_verify::set_cache_only(true);
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
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;
    let (mut rendered, mut refreshed, mut unchanged) = (0usize, 0usize, 0usize);
    let mut failed: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut valid: std::collections::HashMap<String, std::collections::HashSet<String>> =
        std::collections::HashMap::new();
    for movie in &queue {
        let title = truncate(&movie.title, 34);
        match export_film(&http, &store, movie, &out, &dest, jobs).await {
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
    http: &reqwest::Client,
    store: &osmo::Store,
    movie: &Movie,
    out: &Path,
    dest: &Path,
    jobs: usize,
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
    let audio_opus = dir.join("audio.opus");

    let empty = std::collections::HashMap::new();
    // An audio-only language (Korean) has no phoneme model to align with;
    // its clips ship without the alignment block rather than not at all.
    let ctx = (!crate::clips::audio_only(code))
        .then(|| VerifyContext::new(http, store.clone(), &empty, language))
        .transpose()?;

    let lang_dir = dest.join(code);
    let passing: Vec<&Clip> = clips.iter().filter(|c| c.passed).collect();
    let rendered = AtomicUsize::new(0);
    let refreshed = AtomicUsize::new(0);
    let unchanged = AtomicUsize::new(0);
    let total = passing.len();

    use futures::StreamExt;
    let results: Vec<Result<()>> = futures::stream::iter(passing.iter().map(|clip| {
        let (ctx, movie, provenance, clips, sentences, cues, transcript) = (
            ctx.as_ref(),
            movie,
            &provenance,
            &clips,
            &sentences,
            &cues,
            &transcript,
        );
        let (lang_dir, audio_opus, video) = (&lang_dir, &audio_opus, &video);
        let (rendered, refreshed, unchanged) = (&rendered, &refreshed, &unchanged);
        async move {
            let id = clip_id(&movie.imdb_id, clip, sentences, clips);
            let clip_dir = lang_dir.join(&id);
            let cut = plan_cut(clip, cues);
            let stamp = media_stamp(movie, video, audio_stream, &cut);
            // Renditions on disk are reused when they were cut from the same
            // bytes to the same recipe; the loudness they were normalised
            // with is a function of those same inputs, so it comes along.
            let old = std::fs::read(clip_dir.join("meta.json")).ok();
            let reuse = old
                .as_deref()
                .and_then(|b| serde_json::from_slice::<serde_json::Value>(b).ok())
                .filter(|m| {
                    (m["media"]["stamp"] == stamp || legacy_stamp_matches(m, &stamp))
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
                ctx,
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
                audio_opus,
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
    ctx: Option<&VerifyContext<'_>>,
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
    audio_opus: &Path,
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

    // Forced alignment from the cached frame matrix — same wav bytes the
    // gates scored, so this is a cache hit unless the cut code drifted.
    // Failure loses the alignment block, never the clip.
    let alignment = align_phonemes(ctx, audio_opus, clip, scored_start - cut_start).await;

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
        "language": provenance.language,
        "film": {
            "imdb_id": movie.imdb_id,
            "title": movie.title,
            "year": movie.year,
            "subtitle_digest": provenance.subtitle_digest,
            "transcript_digest": provenance.transcript_digest,
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
            "provenance": {
                "format": provenance.format,
                "model": provenance.model,
                "min_ratio": provenance.min_ratio,
                "min_clear_ms": provenance.min_clear_ms,
                "min_edge_logp": provenance.min_edge_logp,
                "max_pad_speech": provenance.max_pad_speech,
                "max_lead_rms": provenance.max_lead_rms,
                "min_voiced": provenance.min_voiced,
            },
        },
        "media": {
            "stamp": stamp,
            "duration_ms": cut_end - cut_start,
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
/// (skips clip dirs with a finished sidecar), upload to R2 (skips `.uploaded`
/// markers). Safe to re-run after any interruption.
pub async fn publish(
    out: PathBuf,
    dest: PathBuf,
    jobs: usize,
    langs: Option<Vec<String>>,
    bucket: String,
) -> Result<()> {
    println!("=== stage 1: clips re-map ===");
    let gate = crate::clips::Gate::default();
    crate::clips::clips_all(out.clone(), 4, 0, None, langs.clone(), gate).await?;

    println!("=== stage 2: video export ===");
    export_clips(out, dest.clone(), jobs, 0, None, langs.clone()).await?;

    println!("=== stage 3: upload to {bucket} ===");
    let codes: Vec<String> = match &langs {
        Some(l) => l.clone(),
        None => std::fs::read_dir(&dest)?
            .flatten()
            .filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().into_string().ok())
            .collect(),
    };
    for code in codes {
        upload_lang(&dest.join(&code), &code, &bucket)?;
    }
    Ok(())
}

/// Upload one language's exported clips via wrangler. A `.uploaded` marker is
/// written per clip dir once all three objects land; marked dirs are skipped,
/// so re-runs only pay for what's new. Objects are immutable by id and get a
/// forever cache; the index gets a short one.
fn upload_lang(lang_dir: &Path, code: &str, bucket: &str) -> Result<()> {
    const IMMUTABLE: &str = "public, max-age=31536000, immutable";
    // Cloudflare's API answers a small fraction of puts with a 502; over the
    // ~50k objects of a full publish one such blip is near-certain, and a
    // failed put costs the whole run (2026-09-06: died at 1,575 of 18k clips).
    // Retry with growing waits before giving up on a key.
    const RETRY_WAITS: [u64; 4] = [5, 30, 120, 300];
    let put = |file: &Path, key: &str, content_type: &str, cache: &str| -> Result<()> {
        let mut attempt = 0;
        loop {
            let status = Command::new("wrangler")
                .args(["r2", "object", "put", &format!("{bucket}/{key}")])
                .arg("--file")
                .arg(file)
                .args(["--content-type", content_type, "--cache-control", cache])
                .arg("--remote")
                .stdout(std::process::Stdio::null())
                .status()
                .context("wrangler failed to start")?;
            if status.success() {
                return Ok(());
            }
            let Some(wait) = RETRY_WAITS.get(attempt) else {
                bail!("upload failed for {key} after {} attempts", attempt + 1);
            };
            eprintln!(
                "  upload of {key} failed (attempt {}), retrying in {wait}s",
                attempt + 1
            );
            std::thread::sleep(std::time::Duration::from_secs(*wait));
            attempt += 1;
        }
    };
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(lang_dir)?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    let uploaded = AtomicUsize::new(0);
    let skipped = AtomicUsize::new(0);
    let upload_dir = |dir: &Path| -> Result<()> {
        let id = dir.file_name().and_then(|s| s.to_str()).unwrap_or_default();
        if !dir.join("meta.json").exists() {
            return Ok(()); // half-written export
        }
        // The marker records what was uploaded, not that something was:
        // each file goes up only when it no longer hashes to what went up.
        // A refreshed sidecar costs one small put, a re-rendered clip all
        // three; a stale marker never wins.
        let files = [
            ("hi", "hi.mp4", "video/mp4"),
            ("lo", "lo.mp4", "video/mp4"),
            ("meta", "meta.json", "application/json"),
        ];
        let mut hashes = serde_json::Map::new();
        for (key, name, _) in files {
            hashes.insert(key.into(), file_hash(&dir.join(name))?.into());
        }
        let marked = std::fs::read(dir.join(".uploaded"))
            .ok()
            .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
            .unwrap_or_default();
        let stale: Vec<_> = files
            .iter()
            .filter(|(key, _, _)| marked[*key] != hashes[*key])
            .collect();
        if stale.is_empty() {
            skipped.fetch_add(1, Ordering::Relaxed);
            return Ok(());
        }
        for (_, name, content_type) in stale {
            put(
                &dir.join(name),
                &format!("{code}/{id}/{name}"),
                content_type,
                IMMUTABLE,
            )?;
        }
        std::fs::write(dir.join(".uploaded"), serde_json::to_vec(&hashes)?)?;
        let n = uploaded.fetch_add(1, Ordering::Relaxed) + 1;
        if n.is_multiple_of(25) {
            println!("  {n} uploaded (last: {id})");
        }
        Ok(())
    };
    // Each put is a whole `wrangler` process — mostly Node start-up, ~2s
    // for a few MB — so one at a time meant ~700 clips/h and a day per
    // publish. Workers take clip dirs in order; a dir's marker lands only
    // after all three objects do, so a killed run redoes at most UPLOAD_JOBS
    // dirs. The first failure stops the rest at their next dir.
    let next = AtomicUsize::new(0);
    let failed = AtomicBool::new(false);
    std::thread::scope(|scope| -> Result<()> {
        let workers: Vec<_> = (0..UPLOAD_JOBS)
            .map(|_| {
                scope.spawn(|| -> Result<()> {
                    loop {
                        if failed.load(Ordering::Relaxed) {
                            return Ok(());
                        }
                        let Some(dir) = dirs.get(next.fetch_add(1, Ordering::Relaxed)) else {
                            return Ok(());
                        };
                        if let Err(e) = upload_dir(dir) {
                            failed.store(true, Ordering::Relaxed);
                            return Err(e);
                        }
                    }
                })
            })
            .collect();
        let mut first_error = None;
        for worker in workers {
            if let Err(e) = worker.join().expect("upload worker panicked") {
                first_error.get_or_insert(e);
            }
        }
        first_error.map_or(Ok(()), Err)
    })?;
    let uploaded = uploaded.into_inner();
    let skipped = skipped.into_inner();
    let index = lang_dir.join("index.jsonl");
    if index.exists() {
        put(
            &index,
            &format!("{code}/index.jsonl"),
            "application/x-ndjson",
            "public, max-age=60",
        )?;
    }
    println!("{code}: {uploaded} clip dirs uploaded, {skipped} already up");
    Ok(())
}

/// Extend the scored cut to neighboring subtitle lines within [`CTX_GAP_MS`],
/// then trim (furthest line first) back under [`CTX_CAP_MS`]. Returns the cut
/// bounds (film-absolute) and how many lines survived on each side.
fn context_bounds(
    clip: &Clip,
    scored_start: i64,
    scored_end: i64,
    cues: &[Cue],
) -> (i64, i64, usize, usize) {
    let mut before: Vec<&Cue> = Vec::new();
    let mut edge = clip.start_ms;
    for cue in cues.iter().rev().filter(|c| c.end_ms <= clip.start_ms) {
        if edge - cue.end_ms > CTX_GAP_MS {
            break;
        }
        edge = cue.start_ms;
        before.push(cue);
    }
    let mut after: Vec<&Cue> = Vec::new();
    let mut edge = clip.end_ms;
    for cue in cues.iter().filter(|c| c.start_ms >= clip.end_ms) {
        if cue.start_ms - edge > CTX_GAP_MS {
            break;
        }
        edge = cue.end_ms;
        after.push(cue);
    }
    let bounds = |before: &[&Cue], after: &[&Cue]| {
        let s = before
            .last()
            .map_or(scored_start, |c| {
                (c.start_ms - CTX_PAD_MS).min(scored_start)
            })
            .max(0);
        let e = after
            .last()
            .map_or(scored_end, |c| (c.end_ms + CTX_PAD_MS).max(scored_end));
        (s, e)
    };
    let (mut s, mut e) = bounds(&before, &after);
    while e - s > CTX_CAP_MS && (!before.is_empty() || !after.is_empty()) {
        // Drop whichever outermost line sits furthest from the sentence.
        let d_before = before.last().map(|c| clip.start_ms - c.start_ms);
        let d_after = after.last().map(|c| c.end_ms - clip.end_ms);
        if d_before >= d_after {
            before.pop();
        } else {
            after.pop();
        }
        (s, e) = bounds(&before, &after);
    }
    (s, e, before.len(), after.len())
}

/// Per-phoneme spans in clip-relative ms, from the cached frame matrix.
async fn align_phonemes(
    ctx: Option<&VerifyContext<'_>>,
    audio_opus: &Path,
    clip: &Clip,
    scored_offset_ms: i64,
) -> Option<serde_json::Value> {
    let ctx = ctx?;
    let wav = slice_wav_padded(
        audio_opus,
        clip.start_ms,
        clip.end_ms,
        clip.pad_before_ms,
        clip.pad_after_ms,
    )
    .ok()?;
    let frames = phoneme_verify::frame_matrix(ctx, &wav).await.ok()?;
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
    height: i64,
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
            "stream=height,color_transfer:format=duration",
            "-of",
            "json",
        ])
        .arg(path)
        .output()
        .context("ffprobe failed to start")?;
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).context("ffprobe output")?;
    let stream = v["streams"].get(0).context("no video stream")?;
    let height = stream["height"].as_i64().context("no height")?;
    let transfer = stream["color_transfer"].as_str().unwrap_or("");
    let duration_ms = v["format"]["duration"]
        .as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .map(|s| (s * 1000.0).round() as i64)
        .context("no container duration")?;
    Ok(VideoProbe {
        height,
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
    let tonemap = if video.hdr {
        "zscale=t=linear:npl=100,format=gbrpf32le,zscale=p=bt709,tonemap=hable:desat=0,\
         zscale=t=bt709:m=bt709:r=tv,"
    } else {
        ""
    };
    let filter = format!(
        "[0:v:0]{tonemap}scale=-2:'min({MAX_HEIGHT},ih)':flags=lanczos,format=yuv420p,\
         split=2[vh][v0];[v0]scale=-2:'min({LO_HEIGHT},ih)'[vl];\
         [0:a:{audio_stream}]aformat=channel_layouts=stereo,volume={gain_db:.2}dB,\
         aresample=48000,asplit=2[ah][al]"
    );
    let key = format!("{crit_s:.3}");
    let run = |core_only: bool| -> Result<bool> {
        let mut cmd = Command::new("ffmpeg");
        cmd.args(["-v", "error", "-y"]);
        if core_only {
            cmd.args(["-core_only", "1"]);
        }
        cmd.arg("-ss")
            .arg(format!("{:.3}", cut_start as f64 / 1000.0))
            .args([
                "-t",
                &format!("{:.3}", (cut_end - cut_start) as f64 / 1000.0),
            ])
            .arg("-i")
            .arg(path)
            .args(["-filter_complex", &filter])
            .args(["-map", "[vh]", "-map", "[ah]"])
            .args(["-c:v", "libx264", "-crf", "19", "-preset", "medium"])
            .args(["-c:a", "aac", "-b:a", "160k"])
            .args(["-force_key_frames", &key, "-movflags", "+faststart"])
            .arg(clip_dir.join("hi.mp4"))
            .args(["-map", "[vl]", "-map", "[al]"])
            .args(["-c:v", "libx264", "-crf", "27", "-preset", "veryfast"])
            .args(["-c:a", "aac", "-b:a", "96k"])
            .args(["-force_key_frames", &key, "-movflags", "+faststart"])
            .arg(clip_dir.join("lo.mp4"));
        Ok(cmd
            .status()
            .context("ffmpeg (encode) failed to start")?
            .success())
    };
    if run(false)? {
        return Ok(());
    }
    // A DTS-HD MA frame with a bad XLL sync word decodes as its lossy 5.1
    // core; that one frame arriving in a filtergraph configured for 7.1 is a
    // property change ffmpeg cannot reinit a complex graph for, and it can
    // sit in the keyframe pre-roll before the cut. Decoding the core only
    // gives every frame the same layout; after the stereo AAC downmix the
    // lossless extension made no difference. (Non-DTS decoders ignore the
    // option.)
    eprintln!(
        "  retrying with core-only audio decode: {}",
        clip_dir.display()
    );
    if run(true)? {
        return Ok(());
    }
    bail!("ffmpeg encode failed for {}", clip_dir.display());
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

fn read_clips_with_provenance(path: &Path) -> Result<(Provenance, Vec<Clip>)> {
    let text = std::fs::read_to_string(path)?;
    let first = text.lines().next().context("empty clips.jsonl")?;
    let provenance: Provenance = serde_json::from_str(first).context("no provenance line")?;
    Ok((provenance, read_clips(path)?))
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
