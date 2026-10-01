//! This is how film clips reach lexide training: `subtitle-corpus training-export`.
//! Reads existing passing clips only; never remaps or modifies the source corpus.
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::{ensure, Context, Result};
use language_utils::Language;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    clips,
    cues::slice_wav_padded,
    library::{read_plan, Movie},
};

#[derive(Debug, clap::Args)]
pub struct Options {
    #[arg(long, default_value = "/data/andrep/subtitle-corpus")]
    out: PathBuf,
    /// Shared corpus codes, comma-separated (spa and por, not variety directories).
    #[arg(long, value_delimiter = ',')]
    langs: Option<Vec<String>>,
    #[arg(long)]
    imdb: Option<String>,
    #[arg(long, default_value = "/data/coding/lexide/pronunciation/data/audio")]
    out_root: PathBuf,
    /// Count exports, legacy removals and estimated WAV bytes; no writes or cuts.
    #[arg(long)]
    dry_run: bool,
    /// Remove selected films' old five-digit cue exports and their manifest rows.
    /// Never touches phonemes.jsonl, vad.jsonl, or unrelated rows/files.
    #[arg(long)]
    replace_legacy: bool,
    /// Number of concurrent audio cuts.
    #[arg(long, default_value = "12")]
    jobs: std::num::NonZeroUsize,
}

/// Training consumes recorded verdicts and cuts, not the mapper's evolving
/// provenance/scoring schema. This also reads historical passing clip rows.
#[derive(Deserialize)]
struct Clip {
    sentence: String,
    imdb_id: String,
    start_ms: i64,
    end_ms: i64,
    pad_before_ms: i64,
    pad_after_ms: i64,
    speaker: Option<String>,
    transcript_wer: f64,
    ratio: Option<f64>,
    passed: bool,
}

/// The audio track the clips were scored against, from the provenance header.
#[derive(Deserialize)]
struct ScoredAudio {
    filename: String,
    stream_index: usize,
    duration_ms: i64,
}

impl ScoredAudio {
    /// Whether `audio.opus` on disk is still the track these timestamps belong to.
    fn matches(&self, stamp: &crate::sync::AudioStamp) -> bool {
        self.filename == stamp.filename
            && self.stream_index == stamp.stream.stream_index
            && self.duration_ms == stamp.duration_ms
    }
}

fn read_clips(path: &Path) -> Result<(Option<ScoredAudio>, Vec<Clip>)> {
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut lines = text.lines();
    // Header shape has changed across mapper generations, not the scored cuts.
    let header: serde_json::Value =
        serde_json::from_str(lines.next().context("missing clip header")?)?;
    let audio = serde_json::from_value(header["inputs"]["audio"].clone()).ok();
    let clips = lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            serde_json::from_str(line).with_context(|| format!("clip row in {}", path.display()))
        })
        .collect::<Result<_>>()?;
    Ok((audio, clips))
}

#[derive(Serialize)]
struct Row<'a> {
    file: String,
    sentence: &'a str,
    lang: &'static str,
    source: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    espeak_voice: Option<&'static str>,
    speaker_cluster: Option<String>,
    imdb_id: &'a str,
    title: &'a str,
    start_ms: i64,
    end_ms: i64,
    pad_before_ms: i64,
    pad_after_ms: i64,
    duration_sec: f64,
    transcript_wer: f64,
    ratio: Option<f64>,
}

fn filename(imdb: &str, start: i64, end: i64) -> String {
    let hash = Sha256::digest(format!("{imdb}:{start}:{end}"));
    let hash: String = hash[..6].iter().map(|byte| format!("{byte:02x}")).collect();
    format!("film_{imdb}_{hash}.wav")
}

fn voice(language: Language) -> Option<&'static str> {
    match language {
        Language::SpanishPeninsular => Some("es"),
        Language::SpanishLatinAmerican => Some("es-419"),
        Language::PortugueseBrazilian => Some("pt-br"),
        Language::PortugueseEuropean => Some("pt"),
        _ => None,
    }
}

fn row<'a>(movie: &'a Movie, clip: &'a Clip, language: Language) -> Result<Row<'a>> {
    ensure!(
        clip.imdb_id == movie.imdb_id,
        "clip IMDb does not match {}",
        movie.imdb_id
    );
    ensure!(
        clip.start_ms >= 0
            && clip.end_ms > clip.start_ms
            && clip.pad_before_ms >= 0
            && clip.pad_after_ms >= 0,
        "invalid clip bounds"
    );
    let duration_ms = clip.end_ms + clip.pad_after_ms - (clip.start_ms - clip.pad_before_ms).max(0);
    Ok(Row {
        file: filename(&movie.imdb_id, clip.start_ms, clip.end_ms),
        sentence: &clip.sentence,
        lang: language.corpus_code(),
        source: "film",
        espeak_voice: voice(language),
        speaker_cluster: clip
            .speaker
            .as_ref()
            .map(|speaker| format!("film:{}:{speaker}", movie.imdb_id)),
        imdb_id: &movie.imdb_id,
        title: &movie.title,
        start_ms: clip.start_ms,
        end_ms: clip.end_ms,
        pad_before_ms: clip.pad_before_ms,
        pad_after_ms: clip.pad_after_ms,
        duration_sec: duration_ms as f64 / 1000.0,
        transcript_wer: clip.transcript_wer,
        ratio: clip.ratio,
    })
}

/// Only exact, basename-only legacy names belonging to selected films qualify.
fn legacy(row: &serde_json::Value, films: &HashSet<&str>) -> bool {
    if row["source"] != "film" {
        return false;
    }
    let Some(file) = row["file"].as_str() else {
        return false;
    };
    let Some((imdb, index)) = file
        .strip_prefix("film_")
        .and_then(|s| s.strip_suffix(".wav"))
        .and_then(|s| s.split_once('_'))
    else {
        return false;
    };
    films.contains(imdb) && index.len() == 5 && index.bytes().all(|b| b.is_ascii_digit())
}

struct Manifest {
    retained: String,
    files: HashSet<String>,
    removed_files: HashSet<String>,
    removed_rows: usize,
}

fn manifest(path: &Path, films: &HashSet<&str>, replace: bool) -> Result<Manifest> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e).with_context(|| format!("read {}", path.display())),
    };
    let mut result = Manifest {
        retained: String::new(),
        files: HashSet::new(),
        removed_files: HashSet::new(),
        removed_rows: 0,
    };
    for line in text.split_inclusive('\n') {
        if line.trim().is_empty() {
            result.retained.push_str(line);
            continue;
        }
        let row: serde_json::Value = serde_json::from_str(line).context("invalid manifest row")?;
        let file = row["file"].as_str().context("manifest row has no file")?;
        if replace && legacy(&row, films) {
            result.removed_files.insert(file.to_owned());
            result.removed_rows += 1;
        } else {
            result.files.insert(file.to_owned());
            result.retained.push_str(line);
        }
    }
    // A retained row may still reference a legacy filename. Never delete its audio.
    result
        .removed_files
        .retain(|file| !result.files.contains(file));
    Ok(result)
}

pub fn run(options: Options) -> Result<()> {
    if let Some(langs) = &options.langs {
        for code in langs {
            ensure!(
                Language::from_code(code).is_some_and(|l| l.corpus_code() == code),
                "use shared corpus codes for --langs, not {code}"
            );
        }
    }
    let mut films: BTreeMap<&str, Vec<(Movie, Language)>> = BTreeMap::new();
    for movie in read_plan(&options.out)? {
        if options.imdb.as_ref().is_some_and(|id| id != &movie.imdb_id) {
            continue;
        }
        let Some(language) = movie.course(&options.out) else {
            continue;
        };
        let code = language.corpus_code();
        if options
            .langs
            .as_ref()
            .is_some_and(|ls| !ls.iter().any(|l| l == code))
        {
            continue;
        }
        films.entry(code).or_default().push((movie, language));
    }
    ensure!(!films.is_empty(), "no films match the selection");
    if options.dry_run {
        println!("DRY RUN — no writes or audio cuts");
    }
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(options.jobs.get())
        .build()?;
    for (code, films) in films {
        let dir = options.out_root.join(code);
        let path = dir.join("manifest.jsonl");
        let ids = films.iter().map(|(m, _)| m.imdb_id.as_str()).collect();
        let manifest = manifest(&path, &ids, options.replace_legacy)?;
        let removed_wavs = manifest
            .removed_files
            .iter()
            .filter(|file| dir.join(file).is_file())
            .count();
        if !options.dry_run && manifest.removed_rows > 0 {
            // Commit the new manifest before unlinking only its unreferenced legacy WAVs.
            let mut tmp = tempfile::NamedTempFile::new_in(&dir)?;
            tmp.write_all(manifest.retained.as_bytes())?;
            tmp.persist(&path)?;
            for file in &manifest.removed_files {
                match fs::remove_file(dir.join(file)) {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => return Err(e.into()),
                }
            }
        }
        let (mut passed, mut skipped) = (0usize, 0usize);
        let mut film_clips = Vec::new();
        let mut selected_files = HashMap::new();
        let mut conflicting_cuts = HashSet::new();
        let mut duplicates = 0;
        for (movie, language) in &films {
            let source = options.out.join(&movie.imdb_id);
            let clips_path = clips::clips_path(&source);
            if !clips_path.exists() {
                continue;
            }
            let (scored, clips) = read_clips(&clips_path)?;
            // Timestamps belong to the track they were scored on. An evicted
            // track has no audio at all; a re-extracted one needs a remap first.
            let current = crate::sync::read_audio_stamp(&source)
                .zip(scored)
                .is_some_and(|(stamp, scored)| scored.matches(&stamp));
            if !current || !source.join("audio.opus").exists() {
                eprintln!(
                    "{}: audio.opus is not the track the clips were scored on, skipping",
                    movie.imdb_id
                );
                continue;
            }
            film_clips.push((movie, language, source, clips));
        }
        let mut selected = Vec::new();
        for (movie, language, source, clips) in &film_clips {
            for clip in clips.iter().filter(|clip| clip.passed) {
                passed += 1;
                let row = row(movie, clip, **language)?;
                if manifest.files.contains(&row.file) {
                    skipped += 1;
                    continue;
                }
                if let Some(first_sentence) = selected_files.get(&row.file) {
                    if first_sentence != row.sentence {
                        conflicting_cuts.insert(row.file.clone());
                    }
                    duplicates += 1;
                    continue;
                }
                // Keep the first row only if the whole group agrees on its text.
                selected_files.insert(row.file.clone(), row.sentence.to_owned());
                selected.push((source, row));
            }
        }
        selected.retain(|(_, row)| !conflicting_cuts.contains(&row.file));
        let exported = selected.len();
        let bytes: u64 = selected
            .iter()
            .map(|(_, row)| (row.duration_sec * 32_000.0).round() as u64 + 44)
            .sum();
        if !options.dry_run && !selected.is_empty() {
            fs::create_dir_all(&dir)?;
            let completed = AtomicUsize::new(0);
            // Indexed collection preserves selection order; workers persist WAVs,
            // then a single writer appends the rows after every cut succeeds.
            let rows = pool.install(|| {
                selected
                    .into_par_iter()
                    .map(|(source, mut row)| -> Result<_> {
                        let wav = slice_wav_padded(
                            &source.join("audio.opus"),
                            row.start_ms,
                            row.end_ms,
                            row.pad_before_ms,
                            row.pad_after_ms,
                        )?;
                        // Opus seeking/resampling can differ from requested millisecond
                        // bounds by a few samples; duration describes the actual WAV.
                        row.duration_sec =
                            clips::wav_samples(&wav).context("invalid cut WAV")?.len() as f64
                                / 16_000.0;
                        let target = dir.join(&row.file);
                        if target.exists() {
                            ensure!(
                                fs::read(&target)? == wav,
                                "refusing to overwrite different audio at {}",
                                target.display()
                            );
                        } else {
                            let mut temp = tempfile::NamedTempFile::new_in(&dir)?;
                            temp.write_all(&wav)?;
                            temp.persist_noclobber(&target)?;
                        }
                        let done = completed.fetch_add(1, Ordering::Relaxed) + 1;
                        if done.is_multiple_of(2000) || done == exported {
                            eprintln!("{code}: cut {done}/{exported}");
                        }
                        Ok(row)
                    })
                    .collect::<Result<Vec<_>>>()
            })?;
            let mut file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)?;
            if !manifest.retained.is_empty() && !manifest.retained.ends_with('\n') {
                writeln!(file)?;
            }
            for row in rows {
                serde_json::to_writer(&mut file, &row)?;
                writeln!(file)?;
            }
        }
        let conflicts = conflicting_cuts.len();
        println!("{code}: {passed} passed, {skipped} already in manifest, {duplicates} duplicate cuts ({conflicts} differing-text groups), {exported} {} (estimated {bytes} bytes); legacy: {} rows, {removed_wavs} WAVs {}",
            if options.dry_run { "would export" } else { "exported" }, manifest.removed_rows,
            if options.dry_run { "would remove" } else { "removed" });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn stable_names_and_voice_contract() {
        let file = filename("tt123", 1000, 2000);
        assert_eq!(file.len(), "film_tt123_".len() + 12 + 4);
        assert_eq!(file, "film_tt123_33181e35d03f.wav");
        assert_ne!(file, filename("tt123", 1001, 2000));
        assert_eq!(voice(Language::SpanishPeninsular), Some("es"));
        assert_eq!(voice(Language::SpanishLatinAmerican), Some("es-419"));
        assert_eq!(voice(Language::PortugueseBrazilian), Some("pt-br"));
        assert_eq!(voice(Language::PortugueseEuropean), Some("pt"));
        assert_eq!(voice(Language::Japanese), None);
    }

    #[test]
    fn legacy_cleanup_is_exact_and_preserves_unrelated_bytes_and_references() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("manifest.jsonl");
        let keep = " {\"file\":\"film_tt1_00001.wav\",\"source\":\"tts\",\"extra\":42}\n\n";
        fs::write(
            &path,
            format!(
                "{}\n{keep}",
                json!({"file":"film_tt1_00001.wav","source":"film"})
            ),
        )
        .unwrap();
        let selected = HashSet::from(["tt1"]);
        let result = manifest(&path, &selected, true).unwrap();
        assert_eq!(result.retained, keep);
        assert_eq!(result.removed_rows, 1);
        assert!(result.removed_files.is_empty());
        assert_eq!(manifest(&path, &selected, false).unwrap().removed_rows, 0);
        for file in [
            "film_tt2_00001.wav",
            "film_tt1_0001.wav",
            "film_tt1_abcdef012345.wav",
            "../film_tt1_00001.wav",
            "film_tt1_00001.wav.bak",
        ] {
            assert!(!legacy(&json!({"file":file,"source":"film"}), &selected));
        }
    }
}

#[cfg(test)]
mod command_tests {
    use super::*;
    use serde_json::json;

    fn fixture(root: &Path) {
        fs::create_dir_all(root.join("tt8291806")).unwrap();
        fs::write(root.join("plan.json"), json!([{"imdb_id":"tt8291806", "title":"Pain and Glory",
            "year":2019,"path":"unused.mkv","original_language":"Spanish","source":{"tier":"missing"}}]).to_string()).unwrap();
        let header = json!({"inputs":{"format":clips::FORMAT_VERSION,"subtitle_digest":"s","transcript_digest":"t",
            "segmentation":"s","language":"spa","audio":{"filename":"film.mkv","stream_index":0,"duration_ms":3000}},
            "cut":{"preferred_clear_ms":100,"speech_threshold":0.7},
            "gate":{"min_ratio":-2.0,"min_edge_logp":-4.0,"max_pad_speech":0.25,"max_lead_rms":1.0,"min_voiced":0.25,"min_verbatim":0.25}});
        let clip = json!({"model":null,"g2p":null,"measured":true,"audio_hash":null,"sentence":"Una frase de prueba.",
            "imdb_id":"tt8291806","start_ms":300,"end_ms":1000,"pad_before_ms":100,"pad_after_ms":150,
            "repaired_before_ms":0,"repaired_after_ms":0,"words":[],"speaker":"speaker_2@3","transcript_wer":0.0,
            "audio_event_overlap":false,"clear_before_ms":200,"clear_after_ms":300,"target_ipa":[],"oov":[],"ratio":-0.5,
            "logp_target_per_phoneme":null,"edge_logp_start":null,"edge_logp_end":null,"lead_speech":null,"tail_speech":null,
            "lead_rms":null,"voiced":null,"heard_ipa":[],"passed":true,"reject":null});
        let mut second = clip.clone();
        second["start_ms"] = 1200.into();
        second["end_ms"] = 1900.into();
        second["speaker"] = serde_json::Value::Null;
        let mut rejected = clip.clone();
        rejected["passed"] = false.into();
        fs::write(
            root.join("tt8291806/clips.jsonl"),
            format!("{header}\n{clip}\n{second}\n{rejected}\n"),
        )
        .unwrap();
        // Tiny PCM fixture; ffmpeg identifies the WAV header even under audio.opus.
        let samples = 3 * 16_000u32;
        let mut wav = b"RIFF".to_vec();
        wav.extend((36 + samples * 2).to_le_bytes());
        wav.extend(b"WAVEfmt ");
        wav.extend(16u32.to_le_bytes());
        wav.extend(1u16.to_le_bytes());
        wav.extend(1u16.to_le_bytes());
        wav.extend(16_000u32.to_le_bytes());
        wav.extend(32_000u32.to_le_bytes());
        wav.extend(2u16.to_le_bytes());
        wav.extend(16u16.to_le_bytes());
        wav.extend(b"data");
        wav.extend((samples * 2).to_le_bytes());
        wav.resize(wav.len() + samples as usize * 2, 0);
        fs::write(root.join("tt8291806/audio.opus"), wav).unwrap();
        // The extracted track the header was scored against.
        fs::write(
            root.join("tt8291806/audio.json"),
            json!({"filename":"film.mkv","duration_ms":3000,
                "stream":{"stream_index":0,"codec":"aac","channels":2,"channel_layout":"stereo"}})
            .to_string(),
        )
        .unwrap();
    }

    #[test]
    fn historic_headers_do_not_change_the_recorded_training_cut() {
        let temp = tempfile::tempdir().unwrap();
        fixture(temp.path());
        let path = temp.path().join("tt8291806/clips.jsonl");
        let original = fs::read_to_string(&path).unwrap();
        let (_, rows) = original.split_once('\n').unwrap();
        let historic = format!("{{\"format\":11,\"language\":\"spa\"}}\n{rows}");
        fs::write(&path, historic).unwrap();
        let (_, clips) = read_clips(&path).unwrap();
        assert_eq!(clips.iter().filter(|clip| clip.passed).count(), 2);
        assert_eq!(clips[0].pad_before_ms, 100);
    }

    #[test]
    fn conflicting_cuts_are_excluded_even_when_the_conflict_comes_last() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        fixture(&source);
        let path = source.join("tt8291806/clips.jsonl");
        let text = fs::read_to_string(&path).unwrap();
        let first = text.lines().nth(1).unwrap();
        let mut conflict: serde_json::Value = serde_json::from_str(first).unwrap();
        conflict["sentence"] = "Different text at the same cut.".into();
        fs::write(&path, format!("{text}{first}\n{conflict}\n{first}\n")).unwrap();
        let out = temp.path().join("output");
        run(Options {
            out: source,
            out_root: out.clone(),
            langs: None,
            imdb: None,
            dry_run: false,
            replace_legacy: false,
            jobs: std::num::NonZeroUsize::new(2).unwrap(),
        })
        .unwrap();
        let dir = out.join("spa");
        let manifest = fs::read_to_string(dir.join("manifest.jsonl")).unwrap();
        let rows: Vec<serde_json::Value> = manifest
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["start_ms"], 1200);
        assert!(!dir.join(filename("tt8291806", 300, 1000)).exists());
        assert!(dir.join(filename("tt8291806", 1200, 1900)).exists());
    }

    #[test]
    fn export_is_read_only_when_dry_and_idempotent_with_precise_legacy_cleanup() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        fixture(&source);
        let clips_path = source.join("tt8291806/clips.jsonl");
        let text = fs::read_to_string(&clips_path).unwrap();
        let duplicate: serde_json::Value =
            serde_json::from_str(text.lines().nth(1).unwrap()).unwrap();
        writeln!(
            fs::OpenOptions::new()
                .append(true)
                .open(&clips_path)
                .unwrap(),
            "{duplicate}"
        )
        .unwrap();
        let out = temp.path().join("output");
        let options = |dry_run, replace_legacy| Options {
            out: source.clone(),
            out_root: out.clone(),
            langs: Some(vec!["spa".into()]),
            imdb: Some("tt8291806".into()),
            dry_run,
            replace_legacy,
            jobs: std::num::NonZeroUsize::new(2).unwrap(),
        };
        run(options(true, true)).unwrap();
        assert!(!out.exists());
        let dir = out.join("spa");
        fs::create_dir_all(&dir).unwrap();
        let keep = " {\"file\":\"other.wav\",\"source\":\"tts\",\"unknown\":42}\n";
        let original = format!(
            "{}\n{keep}",
            json!({"source":"film","file":"film_tt8291806_00001.wav"})
        );
        fs::write(dir.join("manifest.jsonl"), &original).unwrap();
        for name in [
            "film_tt8291806_00001.wav",
            "other.wav",
            "film_ttother_00001.wav",
            "phonemes.jsonl",
            "vad.jsonl",
        ] {
            fs::write(dir.join(name), b"untouched").unwrap();
        }
        run(options(true, true)).unwrap();
        assert_eq!(
            fs::read_to_string(dir.join("manifest.jsonl")).unwrap(),
            original
        );
        assert!(dir.join("film_tt8291806_00001.wav").exists());
        let source_before = fs::read(source.join("tt8291806/clips.jsonl")).unwrap();
        run(options(false, true)).unwrap();
        assert!(!dir.join("film_tt8291806_00001.wav").exists());
        let written = fs::read_to_string(dir.join("manifest.jsonl")).unwrap();
        assert!(written.starts_with(keep));
        let rows: Vec<serde_json::Value> = written
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        assert_eq!(rows.len(), 3); // Unrelated row plus two unique passing cuts.
        assert_eq!(rows[1]["sentence"], "Una frase de prueba."); // First passing row wins a shared cut.
        assert_eq!(rows[1]["espeak_voice"], "es");
        assert_eq!(rows[1]["lang"], "spa");
        assert_eq!(rows[1]["speaker_cluster"], "film:tt8291806:speaker_2@3");
        assert!(rows[2]["speaker_cluster"].is_null());
        assert_eq!(rows[1]["duration_sec"], 0.95);
        for forbidden in ["cue_index", "subtitle_text", "agreement_wer", "exact_wer"] {
            assert!(rows[1].get(forbidden).is_none());
        }
        run(options(false, true)).unwrap();
        assert_eq!(
            fs::read_to_string(dir.join("manifest.jsonl")).unwrap(),
            written
        );
        assert!(!out.join("spa-es").exists());
        for name in [
            "other.wav",
            "film_ttother_00001.wav",
            "phonemes.jsonl",
            "vad.jsonl",
        ] {
            assert_eq!(fs::read(dir.join(name)).unwrap(), b"untouched");
        }
        assert_eq!(
            fs::read(source.join("tt8291806/clips.jsonl")).unwrap(),
            source_before
        );
    }
}
