//! Evaluate the wav2vec2 phoneme model on subtitle-corpus film audio.
//!
//! The model was trained on clean single-speaker recordings; film audio (music
//! beds, room tone, compression) is out of distribution, and before any clip
//! filter is built on it we need to know whether it still *discriminates*
//! there. The trick is that we don't need phonetic ground truth to measure
//! that: the full-film ElevenLabs transcript is an independent witness, so a
//! subtitle cue whose text the transcript confirms verbatim is a known-good
//! clip and a cue the transcript contradicts (or hears nothing for) is a
//! known-bad one. If the phoneme-vs-espeak edit distance separates those two
//! populations, the model works on this material regardless of whether either
//! side is "correct" IPA.
//!
//! For each film with all three artifacts (synced `subtitle.srt`, full
//! `transcript.jsonl`, `audio.opus`) in an espeak-supported language:
//!
//! 1. Label each cue by transcript agreement (token WER between the cleaned
//!    cue text and the transcript words overlapping its span): low WER →
//!    `pos`, high WER → `neg` (split into `neg_mismatch` / `neg_silent`),
//!    anything between → unlabeled and skipped.
//! 2. Sample up to a per-film quota of each label, spread across the film.
//! 3. Cut the cue's audio span from `audio.opus`, run it through the
//!    production `verify_clip_bytes` path (Modal wav2vec2, shared cache,
//!    same normalization) against the espeak phrase-level rendering as the
//!    sole expected sequence — no wikipron, so the comparison is purely
//!    heard-phonemes vs espeak, which is the production shape for corpus
//!    text where per-word dictionaries can't be assumed complete.
//! 4. Append one JSONL record per cue; print per-language separation stats
//!    and the substitution confusion tallies at the end.
//!
//! Predictions are cached under the production cache partition (keyed by the
//! WAV bytes), so re-runs and later analysis passes cost nothing.
//!
//! Usage (from the repo root, so `.cache` resolves):
//!     cargo run --release --bin phoneme-corpus-eval -- [--langs fra,deu] [--max-films 2]

use anyhow::{Context, Result};
use clap::Parser;
use generate_data::audio_verification::{
    AlignmentOp, ClipVerification, VerifyContext, expected_phoneme_variants, verify_clip_bytes,
};
use language_utils::Language;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use subtitle_corpus::cues::{
    Candidate, CueLabel, MIN_FILM_POSITIVES, course_code_g2p as course_code, label_cues,
    load_transcript, parse_cues, sample, slice_wav, tokenization_for,
};

#[derive(Parser, Debug)]
#[command(about = "Evaluate the wav2vec2 phoneme model against transcript-labeled film cues")]
struct Args {
    /// Root of the subtitle corpus.
    #[arg(long, default_value = "/data/andrep/subtitle-corpus")]
    corpus: PathBuf,
    /// Comma-separated course codes to include (default: every espeak-supported one).
    #[arg(long)]
    langs: Option<String>,
    /// Cues sampled per film per label.
    #[arg(long, default_value_t = 40)]
    per_film: usize,
    /// Stop after this many films per language (0 = all).
    #[arg(long, default_value_t = 0)]
    max_films: usize,
    /// Concurrent Modal predictions per film.
    #[arg(long, default_value_t = 8)]
    concurrency: usize,
    /// Output JSONL (appended to; cues with matching provenance are skipped).
    #[arg(long, default_value = "out/phoneme-corpus-eval.jsonl")]
    out: PathBuf,
    /// Only print the summary of an existing output file; run nothing.
    #[arg(long, default_value_t = false)]
    summary_only: bool,
}

/// Equality, not a shortened cache key, decides whether scores are reusable.
/// Deploy markers identify containers, not model revisions or target renderers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Provenance {
    model_id: String,
    model_revision: String,
    decoder_version: String,
    g2p: String,
}

impl Provenance {
    fn new(ctx: &VerifyContext<'_>) -> Result<Self> {
        Ok(Self::from_verified_identity(ctx.verified_model_identity()?))
    }

    fn from_verified_identity(model: &lexide::pronunciation::ModelIdentity) -> Self {
        Self {
            model_id: model.model_id.clone(),
            model_revision: model.model_revision.clone(),
            // Production validates a reported decoder against this version,
            // and uses it for cache keys even when legacy endpoints omit it.
            decoder_version: lexide::pronunciation::DECODER_VERSION.into(),
            g2p: phoneme_verify::model_target_identity(),
        }
    }
}

/// One evaluated cue, as a line of the output JSONL.
#[derive(Serialize, Deserialize)]
struct EvalRecord {
    /// Missing on historical rows: readable, but never reusable as current.
    #[serde(default)]
    provenance: Option<Provenance>,
    imdb_id: String,
    title: String,
    lang: String,
    cue_index: usize,
    start_ms: i64,
    end_ms: i64,
    text: String,
    cleaned_text: String,
    label: CueLabel,
    /// Symmetric token WER between subtitle and transcript window.
    agreement_wer: f64,
    /// Symmetric WER between the cue and everything spoken in the clip span.
    #[serde(default)]
    exact_wer: f64,
    heard_text: String,
    /// An `[audio_event]` (music, laughter…) overlaps the cue span.
    audio_event_overlap: bool,
    /// Transcript speech within `NEIGHBOR_MARGIN_MS` outside the cue span.
    neighbor_speech: bool,
    /// Nested (not flattened): `ClipVerification` also carries a `text`
    /// field, and duplicate keys break the JSONL round-trip.
    verification: ClipVerification,
    /// CTC score of espeak's raw rendering against the frame matrix — the
    /// signal `subtitle-corpus clips` gates on. Absent on records written
    /// before it existed.
    #[serde(default)]
    ctc: Option<phoneme_verify::TargetScore>,
}

fn completed_cues<'a>(
    records: &'a [EvalRecord],
    lang: &str,
    provenance: &Provenance,
) -> HashSet<(&'a str, usize)> {
    records
        .iter()
        .filter(|r| r.lang == lang && r.provenance.as_ref() == Some(provenance))
        .map(|r| (r.imdb_id.as_str(), r.cue_index))
        .collect()
}

fn read_records(out: &Path) -> Result<Vec<EvalRecord>> {
    let text = std::fs::read_to_string(out)
        .with_context(|| format!("no eval output at {}", out.display()))?;
    Ok(text
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    if args.summary_only {
        print_summary(&args.out)?;
        return Ok(());
    }

    let wanted: Option<HashSet<String>> = args
        .langs
        .as_ref()
        .map(|s| s.split(',').map(|x| x.trim().to_string()).collect());

    let plan = subtitle_corpus::library::read_plan(&args.corpus)?;

    // Old model/target scores remain in the append-only file, but cannot
    // suppress a fresh evaluation. Summary uses the same provenance equality.
    let existing = if args.out.exists() {
        read_records(&args.out)?
    } else {
        Vec::new()
    };
    if let Some(parent) = args.out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut out_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&args.out)?;

    // A hard per-request deadline: without one, a single hung Modal
    // connection wedges the whole buffered stream forever (observed: 40
    // minutes frozen mid-film). 120s comfortably covers a cold start; the
    // caller's retry loop handles the resulting timeout errors.
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;
    let empty_pronunciations: HashMap<String, language_utils::Pronunciations> = HashMap::new();

    let mut films_per_lang: HashMap<&'static str, usize> = HashMap::new();
    for entry in &plan {
        let Some(code) = course_code(&entry.original_language) else {
            continue;
        };
        if let Some(w) = &wanted
            && !w.contains(code)
        {
            continue;
        }
        let dir = args.corpus.join(&entry.imdb_id);
        let (srt, transcript_path, audio) = (
            dir.join("subtitle.srt"),
            dir.join("transcript.jsonl"),
            dir.join("audio.opus"),
        );
        if !(srt.exists() && transcript_path.exists() && audio.exists()) {
            continue;
        }
        if args.max_films > 0 && films_per_lang.get(code).copied().unwrap_or(0) >= args.max_films {
            continue;
        }
        *films_per_lang.entry(code).or_default() += 1;

        let language = Language::from_code(code).context("unreachable: unmapped course code")?;
        let cues = parse_cues(&std::fs::read_to_string(&srt)?);
        let transcript = load_transcript(&transcript_path)?;
        // Character units for the space-less scripts: under word tokens a
        // Japanese or Mandarin cue is one token and never matches anything.
        let candidates = label_cues(&cues, &transcript, tokenization_for(code));
        let count = |l: CueLabel| candidates.iter().filter(|c| c.label == l).count();
        let (n_pos, n_extra, n_mismatch, n_silent) = (
            count(CueLabel::Pos),
            count(CueLabel::NegExtraSpeech),
            count(CueLabel::NegMismatch),
            count(CueLabel::NegSilent),
        );
        if n_pos < MIN_FILM_POSITIVES {
            println!(
                "{} {} [{code}]: skipped — only {n_pos} verbatim positives ({} cues); subtitle \
                 likely desynced, a different cut, or forced-only",
                entry.imdb_id,
                entry.title,
                cues.len()
            );
            continue;
        }

        let ctx = VerifyContext::new(
            &http,
            generate_data::cache_remote::store(),
            &empty_pronunciations,
            language,
        )?;
        // Fail closed if the unsafe cache override bypassed discovery. A cache
        // namespace alone must never masquerade as verified model provenance.
        let provenance = Provenance::new(&ctx)?;
        let done = completed_cues(&existing, code, &provenance);

        let picked: Vec<&Candidate> = sample(&candidates, CueLabel::Pos, args.per_film)
            .into_iter()
            .chain(sample(
                &candidates,
                CueLabel::NegExtraSpeech,
                args.per_film / 2,
            ))
            .chain(sample(
                &candidates,
                CueLabel::NegMismatch,
                args.per_film / 2,
            ))
            .chain(sample(&candidates, CueLabel::NegSilent, args.per_film / 2))
            .filter(|c| !done.contains(&(entry.imdb_id.as_str(), c.cue_index)))
            .collect();

        println!(
            "{} {} [{code}]: {} cues → {n_pos} pos / {n_extra} extra-speech / {n_mismatch} mismatch / {n_silent} silent, {} to run",
            entry.imdb_id,
            entry.title,
            cues.len(),
            picked.len()
        );
        if picked.is_empty() {
            continue;
        }

        use futures::StreamExt;
        let results: Vec<Option<EvalRecord>> = futures::stream::iter(picked.into_iter().map(|c| {
            let ctx = &ctx;
            let provenance = &provenance;
            let audio = audio.clone();
            let entry_id = entry.imdb_id.clone();
            let entry_title = entry.title.clone();
            async move {
                let wav = match tokio::task::spawn_blocking({
                    let audio = audio.clone();
                    let (s, e) = (c.start_ms, c.end_ms);
                    move || slice_wav(&audio, s, e)
                })
                .await
                .expect("slice task panicked")
                {
                    Ok(w) => w,
                    Err(e) => {
                        eprintln!("  cue {}: {e:#}", c.cue_index);
                        return None;
                    }
                };
                // Empty pronunciation map ⇒ the espeak phrase-level rendering
                // is the sole expected sequence.
                let expected = expected_phoneme_variants(ctx, &c.cleaned_text, None);
                let verification = match verify_clip_bytes(
                    ctx,
                    &entry_id,
                    &c.cleaned_text,
                    &format!("{entry_id}#{}", c.cue_index),
                    &wav,
                    expected,
                )
                .await
                {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("  cue {}: {e:#}", c.cue_index);
                        return None;
                    }
                };
                // The CTC ratio scores the raw g2p sequence (the model's
                // own label space).
                let ctc = match phoneme_verify::model_target(&c.cleaned_text, language) {
                    Some(Ok(p)) if !p.phonemes.is_empty() => {
                        let target = p.phonemes;
                        match phoneme_verify::frame_matrix(ctx, &wav).await {
                            Ok(frames) => Some(frames.score_target(&target)),
                            Err(e) => {
                                eprintln!("  cue {}: frame matrix: {e:#}", c.cue_index);
                                None
                            }
                        }
                    }
                    _ => None,
                };
                Some(EvalRecord {
                    provenance: Some(provenance.clone()),
                    imdb_id: entry_id,
                    title: entry_title,
                    lang: code.to_string(),
                    cue_index: c.cue_index,
                    start_ms: c.start_ms,
                    end_ms: c.end_ms,
                    text: c.text.clone(),
                    cleaned_text: c.cleaned_text.clone(),
                    label: c.label,
                    agreement_wer: c.agreement_wer,
                    exact_wer: c.exact_wer,
                    heard_text: c.heard_text.clone(),
                    audio_event_overlap: c.audio_event_overlap,
                    neighbor_speech: c.neighbor_speech,
                    verification,
                    ctc,
                })
            }
        }))
        .buffered(args.concurrency.max(1))
        .collect()
        .await;

        for record in results.into_iter().flatten() {
            writeln!(out_file, "{}", serde_json::to_string(&record)?)?;
        }
        out_file.flush()?;
    }

    print_summary(&args.out)
}

#[derive(Default, Debug)]
struct CtcStats {
    ratios: Vec<f64>,
    oov_rejects: usize,
    unscorable_rejects: usize,
    missing: usize,
}

impl CtcStats {
    fn collect<'a>(records: impl Iterator<Item = &'a EvalRecord>) -> Self {
        let mut stats = Self::default();
        for r in records {
            match &r.ctc {
                None => stats.missing += 1,
                Some(c) if !c.oov.is_empty() => stats.oov_rejects += 1,
                Some(c) => match c.ratio.filter(|r| r.is_finite()) {
                    Some(ratio) => stats.ratios.push(ratio),
                    None => stats.unscorable_rejects += 1,
                },
            }
        }
        stats.ratios.sort_by(f64::total_cmp);
        stats
    }

    fn evaluated(&self) -> usize {
        self.ratios.len() + self.oov_rejects + self.unscorable_rejects
    }

    fn kept(&self, cut: f64) -> usize {
        self.ratios.iter().filter(|&&r| r >= cut).count()
    }
}

/// CTC-only separation and candidate cuts, not the other production acoustic
/// gates (edges, voiced speech, padding, etc.) or full production clip yield.
fn print_ctc_summary(by_lang: &BTreeMap<&str, Vec<&EvalRecord>>) {
    println!("\n=== CTC log-odds ratio (per phoneme, target vs free decode) by label ===");
    println!(
        "CTC gate only, not other production acoustic gates. OOV/unscorable scores are hard \
         rejects; missing CTC scores are unevaluated. Quantiles/AUC exclude all three."
    );
    println!(
        "{:<6} {:<14} {:>5} {:>5} {:>10} {:>7}  {:>6} {:>6} {:>6} {:>6} {:>6}",
        "lang", "label", "n", "OOV", "unscorable", "missing", "p10", "p25", "p50", "p75", "p90"
    );
    for (lang, rs) in by_lang {
        for label in [
            CueLabel::Pos,
            CueLabel::NegExtraSpeech,
            CueLabel::NegMismatch,
            CueLabel::NegSilent,
        ] {
            let stats = CtcStats::collect(rs.iter().copied().filter(|r| r.label == label));
            let xs = &stats.ratios;
            let q = |p: f64| {
                if xs.is_empty() {
                    "n/a".to_string()
                } else {
                    format!("{:.2}", xs[((xs.len() - 1) as f64 * p) as usize])
                }
            };
            println!(
                "{:<6} {:<14} {:>5} {:>5} {:>10} {:>7}  {:>6} {:>6} {:>6} {:>6} {:>6}",
                lang,
                format!("{label:?}"),
                xs.len(),
                stats.oov_rejects,
                stats.unscorable_rejects,
                stats.missing,
                q(0.10),
                q(0.25),
                q(0.50),
                q(0.75),
                q(0.90),
            );
        }
        let pos = CtcStats::collect(rs.iter().copied().filter(|r| r.label == CueLabel::Pos));
        let neg = CtcStats::collect(rs.iter().copied().filter(|r| r.label != CueLabel::Pos));
        if !pos.ratios.is_empty() && !neg.ratios.is_empty() {
            let mut wins = 0f64;
            for p in &pos.ratios {
                for n in &neg.ratios {
                    wins += if p > n {
                        1.0
                    } else if p == n {
                        0.5
                    } else {
                        0.0
                    };
                }
            }
            println!(
                "{lang:<6} AUC(pos>neg) = {:.3}  ({} pos vs {} neg; in-vocabulary only)",
                wins / (pos.ratios.len() * neg.ratios.len()) as f64,
                pos.ratios.len(),
                neg.ratios.len()
            );
        }
        print!("{lang:<6} CTC cut ≥ (kept/evaluated, including hard rejects):");
        for cut in [-2.0, -1.5, -1.0, -0.75, -0.5, -0.35, -0.25, -0.15] {
            print!(
                "  {cut:>5.2} → pos {}/{} neg {}/{}",
                pos.kept(cut),
                pos.evaluated(),
                neg.kept(cut),
                neg.evaluated()
            );
        }
        println!();
    }
}

struct SummaryGroup<'a> {
    provenance: &'a Provenance,
    // Latest appended row per (language, film, cue), within this provenance.
    records: BTreeMap<(&'a str, &'a str, usize), &'a EvalRecord>,
}

fn summary_groups(records: &[EvalRecord]) -> (Vec<SummaryGroup<'_>>, usize) {
    let mut groups: Vec<SummaryGroup<'_>> = Vec::new();
    let mut unstamped = 0;
    for r in records {
        let Some(provenance) = &r.provenance else {
            unstamped += 1;
            continue;
        };
        let index = groups.iter().position(|g| g.provenance == provenance);
        let group = match index {
            Some(i) => &mut groups[i],
            None => {
                groups.push(SummaryGroup {
                    provenance,
                    records: BTreeMap::new(),
                });
                groups.last_mut().unwrap()
            }
        };
        group.records.insert((&r.lang, &r.imdb_id, r.cue_index), r);
    }
    (groups, unstamped)
}

/// Offline summary: never discover the currently serving model or pool model
/// histories. Unstamped rows have unknown (possibly mixed) models and targets.
fn print_summary(out: &Path) -> Result<()> {
    let records = read_records(out)?;
    if records.is_empty() {
        println!("no records");
        return Ok(());
    }
    let (groups, unstamped) = summary_groups(&records);
    println!(
        "Historical/unstamped: {unstamped} rows excluded from aggregates and resume \
         (unknown model/target provenance)."
    );
    for group in groups {
        println!(
            "\n=== Recorded provenance (not asserted current): {} — {} unique cues ===",
            serde_json::to_string(group.provenance)?,
            group.records.len()
        );
        print_group_summary(group.records.values().copied());
    }
    Ok(())
}

fn print_group_summary<'a>(records: impl Iterator<Item = &'a EvalRecord>) {
    let mut by_lang: BTreeMap<&str, Vec<&EvalRecord>> = BTreeMap::new();
    for r in records {
        by_lang.entry(r.lang.as_str()).or_default().push(r);
    }

    print_ctc_summary(&by_lang);

    println!("\n=== phoneme-vs-espeak edit-distance % by label ===");
    println!(
        "{:<6} {:<14} {:>5}  {:>6} {:>6} {:>6} {:>6} {:>6}",
        "lang", "label", "n", "p10", "p25", "p50", "p75", "p90"
    );
    for (lang, rs) in &by_lang {
        for label in [
            CueLabel::Pos,
            CueLabel::NegExtraSpeech,
            CueLabel::NegMismatch,
            CueLabel::NegSilent,
        ] {
            let mut pcts: Vec<f64> = rs
                .iter()
                .filter(|r| r.label == label)
                .filter_map(|r| r.verification.edit_distance_pct)
                .collect();
            if pcts.is_empty() {
                continue;
            }
            pcts.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let q = |p: f64| pcts[((pcts.len() - 1) as f64 * p) as usize];
            println!(
                "{:<6} {:<14} {:>5}  {:>5.0}% {:>5.0}% {:>5.0}% {:>5.0}% {:>5.0}%",
                lang,
                format!("{label:?}"),
                pcts.len(),
                q(0.10) * 100.0,
                q(0.25) * 100.0,
                q(0.50) * 100.0,
                q(0.75) * 100.0,
                q(0.90) * 100.0,
            );
        }
        // AUC of pct as a pos-vs-neg discriminator (rank-sum estimate):
        // probability a random negative scores higher than a random positive.
        let pos: Vec<f64> = rs
            .iter()
            .filter(|r| r.label == CueLabel::Pos)
            .filter_map(|r| r.verification.edit_distance_pct)
            .collect();
        let neg: Vec<f64> = rs
            .iter()
            .filter(|r| r.label != CueLabel::Pos)
            .filter_map(|r| r.verification.edit_distance_pct)
            .collect();
        if !pos.is_empty() && !neg.is_empty() {
            let mut wins = 0f64;
            for p in &pos {
                for n in &neg {
                    wins += if n > p {
                        1.0
                    } else if n == p {
                        0.5
                    } else {
                        0.0
                    };
                }
            }
            println!(
                "{lang:<6} AUC(neg>pos) = {:.3}  ({} pos vs {} neg)",
                wins / (pos.len() * neg.len()) as f64,
                pos.len(),
                neg.len()
            );
        }
    }

    println!("\n=== top substitutions among POSITIVES (systematic model↔espeak disagreement) ===");
    for (lang, rs) in &by_lang {
        let mut subs: HashMap<(String, String), usize> = HashMap::new();
        let mut extra: HashMap<String, usize> = HashMap::new();
        let mut missing: HashMap<String, usize> = HashMap::new();
        let mut total_ops = 0usize;
        for r in rs.iter().filter(|r| r.label == CueLabel::Pos) {
            for op in r.verification.alignment.iter().flatten() {
                total_ops += 1;
                match op {
                    AlignmentOp::Sub {
                        expected,
                        predicted,
                        ..
                    } => {
                        *subs
                            .entry((expected.clone(), predicted.clone()))
                            .or_default() += 1
                    }
                    AlignmentOp::Extra { predicted, .. } => {
                        *extra.entry(predicted.clone()).or_default() += 1
                    }
                    AlignmentOp::Missing { expected } => {
                        *missing.entry(expected.clone()).or_default() += 1
                    }
                    AlignmentOp::Match { .. } => {}
                }
            }
        }
        let mut subs: Vec<_> = subs.into_iter().collect();
        subs.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
        let mut extra: Vec<_> = extra.into_iter().collect();
        extra.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
        let mut missing: Vec<_> = missing.into_iter().collect();
        missing.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
        println!("\n[{lang}] ({total_ops} alignment ops in positives)");
        print!("  subs (espeak→model): ");
        for ((e, p), n) in subs.iter().take(20) {
            print!("{e}→{p}:{n}  ");
        }
        print!("\n  model-extra: ");
        for (p, n) in extra.iter().take(12) {
            print!("{p}:{n}  ");
        }
        print!("\n  model-missing: ");
        for (e, n) in missing.iter().take(12) {
            print!("{e}:{n}  ");
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provenance() -> Provenance {
        Provenance {
            model_id: "test/model".into(),
            model_revision: "1234567890ab-full-revision".into(),
            decoder_version: lexide::pronunciation::DECODER_VERSION.into(),
            g2p: phoneme_verify::model_target_identity(),
        }
    }

    fn record(provenance: Option<Provenance>) -> EvalRecord {
        // A historical record deliberately omits provenance, cache_version,
        // exact_wer and ctc. These all predate their respective stamps.
        let mut r: EvalRecord = serde_json::from_value(serde_json::json!({
            "imdb_id": "tt1", "title": "Film", "lang": "fra", "cue_index": 7,
            "start_ms": 10, "end_ms": 20, "text": "bonjour", "cleaned_text": "bonjour",
            "label": "pos", "agreement_wer": 0.0, "heard_text": "bonjour",
            "audio_event_overlap": false, "neighbor_speech": false,
            "verification": {
                "actor": "tt1", "text": "bonjour", "wav_path": "tt1#7",
                "predicted_raw": [], "predicted_normalized": [], "expected": null,
                "edit_distance": null, "edit_distance_pct": null,
                "alignment": null, "failure_reason": null
            }
        }))
        .unwrap();
        r.provenance = provenance;
        r
    }

    fn score(ratio: Option<f64>, oov: &[&str]) -> phoneme_verify::TargetScore {
        phoneme_verify::TargetScore {
            logp_target: None,
            logp_target_per_phoneme: None,
            logp_free: None,
            ratio,
            target_len: 2,
            free_len: 2,
            oov: oov.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn resume_requires_full_model_decoder_and_target_identity() {
        let current = provenance();
        let records = vec![record(Some(current.clone()))];
        assert!(completed_cues(&records, "fra", &current).contains(&("tt1", 7)));
        assert!(completed_cues(&records, "hin", &current).is_empty());
        for field in ["model", "revision", "decoder", "g2p"] {
            let mut changed = current.clone();
            match field {
                "model" => changed.model_id.push_str("-new"),
                // Same 12-character cache prefix is not the same full revision.
                "revision" => changed.model_revision.push_str("-new"),
                "decoder" => changed.decoder_version = "new-decoder".into(),
                "g2p" => changed.g2p.push_str("-new"),
                _ => unreachable!(),
            }
            assert!(
                completed_cues(&records, "fra", &changed).is_empty(),
                "{field}"
            );
        }
        let mut missing_decoder = serde_json::to_value(&current).unwrap();
        missing_decoder
            .as_object_mut()
            .unwrap()
            .remove("decoder_version");
        assert!(serde_json::from_value::<Provenance>(missing_decoder).is_err());
    }

    #[test]
    fn effective_decoder_is_stamped_even_when_endpoint_omits_it() {
        let mut model = lexide::pronunciation::ModelIdentity {
            model_id: "test/model".into(),
            model_revision: "revision".into(),
            decoder_version: None,
            deploy_marker: Some("deployment-a".into()),
        };
        let stamp = Provenance::from_verified_identity(&model);
        assert_eq!(
            stamp.decoder_version,
            lexide::pronunciation::DECODER_VERSION
        );
        model.decoder_version = Some(lexide::pronunciation::DECODER_VERSION.into());
        model.deploy_marker = Some("deployment-b".into());
        assert_eq!(stamp, Provenance::from_verified_identity(&model));
        let records = vec![record(Some(stamp.clone()))];
        let mut next_decoder = stamp;
        next_decoder.decoder_version.push_str("-next");
        assert!(completed_cues(&records, "fra", &next_decoder).is_empty());
    }

    #[test]
    fn hindi_target_change_invalidates_old_resume() {
        let current = provenance();
        assert_eq!(current.g2p, phoneme_verify::model_target_identity());
        let mut r = record(Some(current.clone()));
        r.lang = "hin".into();
        let records = vec![r];
        assert_eq!(completed_cues(&records, "hin", &current).len(), 1);
        let mut historical = current;
        historical.g2p.push_str(" hindi=Legacy");
        assert!(completed_cues(&records, "hin", &historical).is_empty());
    }

    #[test]
    fn legacy_rows_are_readable_but_never_current() {
        let records = vec![record(None)];
        assert!(records[0].ctc.is_none());
        assert!(records[0].verification.cache_version.is_none());
        assert!(completed_cues(&records, "fra", &provenance()).is_empty());
        let (groups, unstamped) = summary_groups(&records);
        assert!(groups.is_empty());
        assert_eq!(unstamped, 1);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("eval.jsonl");
        std::fs::write(&path, serde_json::to_string(&records[0]).unwrap()).unwrap();
        print_summary(&path).unwrap(); // Pure file read, no discovery/client.
    }

    #[test]
    fn summaries_separate_provenance_and_use_latest_matching_row() {
        let first = provenance();
        let mut other = first.clone();
        other.model_revision.push_str("-other");
        let mut old = record(Some(first.clone()));
        old.ctc = Some(score(Some(-2.0), &[]));
        let mut latest = record(Some(first.clone()));
        latest.ctc = Some(score(Some(-0.1), &[]));
        let mut another_language = record(Some(first.clone()));
        another_language.lang = "deu".into();
        let records = vec![
            old,
            record(Some(other)),
            record(None),
            latest,
            another_language,
        ];
        let (groups, unstamped) = summary_groups(&records);
        assert_eq!(unstamped, 1);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].records.len(), 2);
        assert_eq!(groups[1].records.len(), 1);
        assert_eq!(
            groups[0].records[&("fra", "tt1", 7)]
                .ctc
                .as_ref()
                .unwrap()
                .ratio,
            Some(-0.1)
        );
        assert_eq!(completed_cues(&records, "fra", &first).len(), 1);
        // All printed summaries, not just CTC, receive these isolated groups.
        for group in groups {
            print_group_summary(group.records.values().copied());
        }
    }

    #[test]
    fn oov_ratios_never_enter_statistics_or_pass_the_ctc_gate() {
        let mut records: Vec<_> = (0..6).map(|_| record(None)).collect();
        records[0].ctc = Some(score(Some(-0.2), &[]));
        records[1].ctc = Some(score(Some(-2.0), &[]));
        records[2].ctc = Some(score(Some(0.0), &["unknown"]));
        records[3].ctc = Some(score(None, &["unknown"]));
        records[4].ctc = Some(score(None, &[]));
        let stats = CtcStats::collect(records.iter());
        assert_eq!(stats.ratios, vec![-2.0, -0.2]);
        assert_eq!(stats.oov_rejects, 2);
        assert_eq!(stats.unscorable_rejects, 1);
        assert_eq!(stats.missing, 1);
        assert_eq!(stats.evaluated(), 5);
        assert_eq!(stats.kept(-0.5), 1);
        assert_eq!(stats.kept(-3.0), 2);
        let all_oov = CtcStats::collect(records[2..4].iter());
        assert!(all_oov.ratios.is_empty());
        assert_eq!(all_oov.evaluated(), 2);
        assert_eq!(all_oov.kept(-100.0), 0);
    }

    #[test]
    fn explicit_cache_override_cannot_stamp_verified_provenance() {
        let dir = tempfile::tempdir().unwrap();
        let http = reqwest::Client::new();
        let words = HashMap::new();
        let ctx = VerifyContext::with_overrides(
            &http,
            osmo::Store::open(dir.path()),
            &words,
            Language::French,
            "looks-like-production".into(),
            0.3,
            Some("deploy-marker".into()),
        )
        .unwrap();
        assert!(
            Provenance::new(&ctx)
                .unwrap_err()
                .to_string()
                .contains("verified model identity required")
        );
    }

    #[test]
    fn obsolete_model_marker_option_is_rejected() {
        assert!(Args::try_parse_from(["eval", "--model-marker", "old-revision"]).is_err());
        assert!(
            Args::try_parse_from(["eval", "--summary-only"])
                .unwrap()
                .summary_only
        );
    }
}
