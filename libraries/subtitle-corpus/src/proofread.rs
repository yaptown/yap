//! Orthography review of cues, then conservative coherence review of course sentences.

mod prompts;

use std::{
    collections::{BTreeMap, BTreeSet},
    io::{BufRead, Write},
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use futures::{stream, StreamExt};
use language_utils::Language;
use movie_subtitles::{
    corrections::{self, Correction},
    llm_segment::batch_jobs::SMALL_BATCH_THRESHOLD,
    segment::SubtitleSegmenter,
    sentences::KeyedSentence,
    SubtitleLine,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tysm::chat_completions::{ChatClient, ChatMessage};

const MODEL: &str = "gpt-6-luna";
const REASONING_EFFORT: &str = "low";
const CHUNK_ITEMS: usize = 30;
const CONTEXT_ITEMS: usize = 2;
// tysm sends 16 requests per call, under a client-wide cap of 100.
const LIVE_CALLS: usize = 6;
fn transport(client: ChatClient, live: bool) -> ChatClient {
    if live {
        client.with_no_batch()
    } else {
        client.with_small_batch_threshold(SMALL_BATCH_THRESHOLD)
    }
}

#[derive(Debug, clap::Args)]
pub struct Options {
    #[arg(long, default_value = "/data/andrep/subtitle-corpus")]
    out: PathBuf,
    #[arg(long, default_value = "./generate-data/data")]
    data_root: PathBuf,
    #[arg(long)]
    language: Option<String>,
    #[arg(long)]
    imdb: Option<String>,
    /// Maximum distinct (language, film) pairs; 0 means all.
    #[arg(long, default_value_t = 0)]
    limit: usize,
    /// Count unique cues and orthography requests only; no API calls or writes.
    #[arg(long)]
    dry_run: bool,
    /// Run labeled cues or sentences through production course chunks, without merging.
    #[arg(long)]
    eval: Option<PathBuf>,
    /// Use live requests instead of the Batch API, including segmentation.
    #[arg(long)]
    live: bool,
    /// Audit both passes without modifying correction overlays.
    #[arg(long)]
    no_merge: bool,
}

#[derive(Debug, Deserialize)]
struct Sample {
    language: String,
    imdb: String,
    #[serde(flatten)]
    expectation: Expectation,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Expectation {
    Orthography { cue: String, expected: String },
    Coherence { sentence: String, incoherent: bool },
}

struct Track {
    language: Language,
    imdb: String,
    raw: Vec<SubtitleLine>,
}

#[derive(Debug, Serialize)]
struct Item {
    index: usize,
    text: String,
    read_only: bool,
    #[serde(skip)]
    key: String,
}

#[derive(Debug, Serialize)]
struct Chunk {
    language: String,
    imdb: String,
    before_read_only: Vec<String>,
    items: Vec<Item>,
    after_read_only: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
struct Change {
    index: usize,
    corrected: String,
    reason: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct Answer {
    changes: Vec<Change>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct Flag {
    index: usize,
    reason: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct CoherenceAnswer {
    flags: Vec<Flag>,
}

#[derive(Debug, Serialize)]
struct Audited {
    language: String,
    imdb: String,
    index: usize,
    cue: Option<String>,
    input: Option<String>,
    corrected: String,
    reason: String,
    accepted: bool,
    rejection_reason: Option<String>,
}

#[derive(Debug, Serialize)]
struct AuditedFlag {
    language: String,
    imdb: String,
    sentence: Option<String>,
    index: usize,
    reason: String,
    accepted: bool,
    rejection_reason: Option<String>,
}

/// Keep original track order and boundaries. Repeated/non-course items remain
/// read-only context, and entirely read-only chunks never become requests.
fn chunks(
    language: Language,
    imdb: &str,
    items: Vec<Item>,
    seen: &mut BTreeSet<String>,
) -> Vec<Chunk> {
    let mut result = Vec::new();
    for start in (0..items.len()).step_by(CHUNK_ITEMS) {
        let end = (start + CHUNK_ITEMS).min(items.len());
        let focus: Vec<_> = items[start..end]
            .iter()
            .enumerate()
            .map(|(index, item)| Item {
                index,
                text: item.text.clone(),
                key: item.key.clone(),
                read_only: item.read_only || !seen.insert(item.key.clone()),
            })
            .collect();
        if focus.iter().all(|item| item.read_only) {
            continue;
        }
        result.push(Chunk {
            language: language.code().into(),
            imdb: imdb.into(),
            items: focus,
            before_read_only: items[start.saturating_sub(CONTEXT_ITEMS)..start]
                .iter()
                .map(|item| item.text.clone())
                .collect(),
            after_read_only: items[end..(end + CONTEXT_ITEMS).min(items.len())]
                .iter()
                .map(|item| item.text.clone())
                .collect(),
        });
    }
    result
}

fn cue_chunks(tracks: &[Track], samples: Option<&[Sample]>) -> Result<Vec<Chunk>> {
    let mut seen = BTreeMap::<(Language, &str), BTreeSet<String>>::new();
    let mut result = Vec::new();
    for track in tracks {
        let mut input = track.raw.clone();
        corrections::apply_spelling(&mut input, track.language, &track.imdb);
        let items = input
            .into_iter()
            .zip(&track.raw)
            .map(|(line, raw)| Item {
                index: 0,
                text: line.sentence,
                key: raw.sentence.clone(),
                read_only: false,
            })
            .collect();
        let film_chunks = chunks(
            track.language,
            &track.imdb,
            items,
            seen.entry((track.language, &track.imdb)).or_default(),
        );
        result.extend(film_chunks.into_iter().filter(|chunk| {
            samples.is_none_or(|rows| {
                rows.iter().any(|row| {
                    row.language == chunk.language
                        && row.imdb == chunk.imdb
                        && match &row.expectation {
                            // Sentence eval needs the film's full preceding orthography pass.
                            Expectation::Coherence { .. } => true,
                            Expectation::Orthography { cue, .. } => chunk
                                .items
                                .iter()
                                .any(|item| !item.read_only && &item.key == cue),
                        }
                })
            })
        }));
    }
    if let Some(samples) = samples {
        for sample in samples {
            if let Expectation::Orthography { cue, .. } = &sample.expectation {
                require_sample(&result, sample, cue)?;
            }
        }
    }
    Ok(result)
}

fn require_sample(chunks: &[Chunk], sample: &Sample, text: &str) -> Result<()> {
    if !chunks.iter().any(|chunk| {
        chunk.language == sample.language
            && chunk.imdb == sample.imdb
            && chunk
                .items
                .iter()
                .any(|item| !item.read_only && item.key == text)
    }) {
        bail!(
            "eval text not found among eligible selected course items: {} {} {text:?}",
            sample.language,
            sample.imdb
        );
    }
    Ok(())
}

fn sorted_paths(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = std::fs::read_dir(dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    paths.sort();
    Ok(paths)
}

fn discover(options: &Options, samples: Option<&[Sample]>) -> Result<Vec<Track>> {
    // Course tracks come first, so eval uses their exact production chunks.
    let mut tracks = BTreeMap::<(String, String), Vec<(PathBuf, bool)>>::new();
    for dir in sorted_paths(&options.data_root)? {
        let code = dir.file_name().unwrap().to_string_lossy().into_owned();
        let raw_dir = dir.join("sentence-sources/movies/subtitles-raw");
        if !raw_dir.is_dir() || Language::from_code(&code).is_none() {
            continue;
        }
        for path in sorted_paths(&raw_dir)? {
            if path.extension().is_some_and(|ext| ext == "srt") {
                let imdb = path.file_stem().unwrap().to_string_lossy().into_owned();
                tracks
                    .entry((code.clone(), imdb))
                    .or_default()
                    .push((path, true));
            }
        }
    }
    if samples.is_none() {
        let inventory = if crate::library::plan_path(&options.out).exists() {
            crate::library::read_plan(&options.out)?
                .into_iter()
                .filter_map(|film| {
                    crate::library::course_dir(&film.original_language)
                        .map(|code| (film.imdb_id, code.to_owned()))
                })
                .collect::<BTreeMap<_, _>>()
        } else {
            BTreeMap::new()
        };
        for dir in sorted_paths(&options.out)? {
            let srt = dir.join("subtitle.srt");
            if !srt.is_file() {
                continue;
            }
            let imdb = dir.file_name().unwrap().to_string_lossy().into_owned();
            if options.imdb.as_ref().is_some_and(|wanted| wanted != &imdb) {
                continue;
            }
            let code = if dir.join("clips.jsonl").exists() {
                // Read only provenance; old formats still identify the language.
                let header = std::io::BufReader::new(std::fs::File::open(dir.join("clips.jsonl"))?)
                    .lines()
                    .next()
                    .context("empty clips file")??;
                let value: serde_json::Value = serde_json::from_str(&header)?;
                value["inputs"]["language"].as_str().map(str::to_owned)
            } else {
                None
            }
            .or_else(|| inventory.get(&imdb).cloned());
            let Some(code) = code else {
                eprintln!("proofread: no language for {imdb}; skipped");
                continue;
            };
            tracks.entry((code, imdb)).or_default().push((srt, false));
        }
    }
    let mut result = Vec::new();
    let mut films = 0;
    for ((code, imdb), paths) in tracks {
        if options
            .language
            .as_ref()
            .is_some_and(|wanted| wanted != &code)
            || options.imdb.as_ref().is_some_and(|wanted| wanted != &imdb)
            || samples.is_some_and(|rows| {
                !rows
                    .iter()
                    .any(|row| row.language == code && row.imdb == imdb)
            })
        {
            continue;
        }
        if options.limit != 0 && films >= options.limit {
            break;
        }
        films += 1;
        let language = Language::from_code(&code)
            .with_context(|| format!("unknown language {code} for {imdb}"))?;
        for (path, course) in paths {
            let text = std::fs::read_to_string(&path)?;
            let raw = if course {
                movie_subtitles::parse_srt(&text)
                    .with_context(|| format!("parsing {}", path.display()))?
            } else {
                crate::clips::uncorrected_subtitle_lines(&text)
            };
            result.push(Track {
                language,
                imdb: imdb.clone(),
                raw,
            });
        }
    }
    Ok(result)
}

fn index_rejection(
    chunk: &Chunk,
    index: usize,
    counts: &BTreeMap<usize, usize>,
) -> Option<&'static str> {
    if counts[&index] != 1 {
        Some("duplicate index changes index count")
    } else if let Some(item) = chunk.items.get(index) {
        item.read_only.then_some("read-only context index")
    } else {
        Some("index outside chunk")
    }
}

/// Preserve sentence boundaries and speech rhythm, treating a run of two or
/// more adjacent dots as one ellipsis. Collapse in the original text, not after
/// filtering: the periods in two separate sentences must remain two periods.
/// Each protected mark with its position: how many skeleton letters precede
/// it. Comparing positions, not just the sequence, keeps a period from moving
/// ("Non. Je viens" → "Non je viens.") while spacing and accents stay free.
fn punctuation_marks(text: &str, language: Language) -> Vec<(usize, char)> {
    let mut marks = Vec::new();
    let mut chars = text.char_indices().peekable();
    while let Some((at, ch)) = chars.next() {
        let position = || corrections::skeleton(&text[..at], language).chars().count();
        if ch == '.' && chars.peek().is_some_and(|&(_, next)| next == '.') {
            while chars.peek().is_some_and(|&(_, next)| next == '.') {
                chars.next();
            }
            marks.push((position(), '…'));
            continue;
        }
        if let Some(class) = corrections::sentence_mark(ch) {
            marks.push((position(), class));
        }
    }
    marks
}

/// Preserve the cue's unambiguous house style without rejecting a real fix.
fn restore_quote_style(input: &str, corrected: &str) -> String {
    let mut corrected = corrected.to_owned();
    if input.contains('’') && !input.contains('\'') {
        corrected = corrected.replace('\'', "’");
    } else if input.contains('\'') && !input.contains('’') {
        corrected = corrected.replace('’', "'");
    }
    if input.contains('"') && !input.contains(['“', '”']) {
        corrected = corrected.replace(['“', '”'], "\"");
    } else if input.contains(['“', '”']) && !input.contains('"') {
        // Keep original opening/closing roles, including a cue that contains
        // only the closing quote of speech begun in the previous cue.
        let mut roles = input.chars().filter(|ch| matches!(ch, '“' | '”'));
        let mut opening = true;
        corrected = corrected
            .chars()
            .map(|ch| {
                if matches!(ch, '"' | '“' | '”') {
                    let role = roles.next().unwrap_or(if opening { '“' } else { '”' });
                    opening = role == '”';
                    if ch == '"' {
                        role
                    } else {
                        ch
                    }
                } else {
                    ch
                }
            })
            .collect();
    }
    corrected
}

fn audit(chunk: &Chunk, answer: Answer) -> Vec<Audited> {
    let mut counts = BTreeMap::<usize, usize>::new();
    for change in &answer.changes {
        *counts.entry(change.index).or_default() += 1;
    }
    let language = Language::from_code(&chunk.language).unwrap();
    answer
        .changes
        .into_iter()
        .map(|mut change| {
            let item = chunk.items.get(change.index);
            let mut rejection = index_rejection(chunk, change.index, &counts).or_else(|| {
                let input = &item.unwrap().text;
                if change.corrected.trim().is_empty() {
                    Some("empty correction")
                } else if change.corrected == *input {
                    Some("unchanged text")
                } else if corrections::skeleton(&change.corrected, language)
                    != corrections::skeleton(input, language)
                {
                    Some("letters changed (skeleton mismatch)")
                } else if punctuation_marks(&change.corrected, language)
                    != punctuation_marks(input, language)
                {
                    Some("punctuation marks changed or moved")
                } else {
                    None
                }
            });
            if rejection.is_none() {
                let input = &item.unwrap().text;
                change.corrected = restore_quote_style(input, &change.corrected);
                if &change.corrected == input {
                    rejection = Some("unchanged text after restoring quote style");
                }
            }
            Audited {
                language: chunk.language.clone(),
                imdb: chunk.imdb.clone(),
                index: change.index,
                cue: item.map(|item| item.key.clone()),
                input: item.map(|item| item.text.clone()),
                corrected: change.corrected,
                reason: change.reason,
                accepted: rejection.is_none(),
                rejection_reason: rejection.map(str::to_owned),
            }
        })
        .collect()
}

fn audit_flags(chunk: &Chunk, answer: CoherenceAnswer) -> Vec<AuditedFlag> {
    let mut counts = BTreeMap::<usize, usize>::new();
    for flag in &answer.flags {
        *counts.entry(flag.index).or_default() += 1;
    }
    answer
        .flags
        .into_iter()
        .map(|flag| {
            let rejection = index_rejection(chunk, flag.index, &counts);
            AuditedFlag {
                language: chunk.language.clone(),
                imdb: chunk.imdb.clone(),
                index: flag.index,
                sentence: chunk.items.get(flag.index).map(|item| item.text.clone()),
                reason: flag.reason,
                accepted: rejection.is_none(),
                rejection_reason: rejection.map(str::to_owned),
            }
        })
        .collect()
}

fn judge_messages(chunk: &Chunk, systems: &BTreeMap<&str, String>) -> Vec<ChatMessage> {
    vec![
        ChatMessage::system(&systems[chunk.language.as_str()]),
        ChatMessage::user(serde_json::to_string(chunk).expect("serializable chunk")),
    ]
}

/// Shared scheduling, but separate schemas and system prompts for the two passes.
async fn judge<T: DeserializeOwned + schemars::JsonSchema>(
    chunks: &[Chunk],
    system_prompt: fn(&str) -> String,
    live: bool,
) -> Result<(Vec<(&Chunk, T)>, usize)> {
    if chunks.is_empty() {
        return Ok((Vec::new(), 0));
    }
    let client = transport(
        ChatClient::from_env(MODEL)?
            .with_cache_directory("./.cache")
            .with_reasoning_effort(REASONING_EFFORT),
        live,
    );
    let mut systems = BTreeMap::new();
    for chunk in chunks {
        systems
            .entry(chunk.language.as_str())
            .or_insert_with(|| system_prompt(&chunk.language));
    }
    let messages = |chunk: &Chunk| judge_messages(chunk, &systems);
    // Pool languages without changing any request's shared language prefix.
    let jobs = if live {
        chunks.chunks(chunks.len().div_ceil(LIVE_CALLS)).collect()
    } else {
        movie_subtitles::llm_segment::batch_jobs::partition::<_, T>(&client, chunks, messages)?
    };
    let submit = |i: usize| {
        let group = jobs[i];
        let client = &client;
        async move {
            let mut last = None;
            client
                .batch_chat_with_messages_fn::<_, T>(group, messages, |batch| {
                    let now = (
                        format!("{:?}", batch.status),
                        batch.request_counts.completed,
                        batch.request_counts.failed,
                    );
                    if last.as_ref() != Some(&now) {
                        eprintln!(
                            "proofread batch {}: {} {}/{} completed, {} failed",
                            batch.id, now.0, now.1, batch.request_counts.total, now.2
                        );
                        last = Some(now);
                    }
                })
                .await
        }
    };
    let results = if live {
        stream::iter((0..jobs.len()).map(submit))
            .buffered(LIVE_CALLS)
            .map(|result| result.map_err(anyhow::Error::from))
            .collect::<Vec<_>>()
            .await
    } else {
        let indices: Vec<_> = (0..jobs.len()).collect();
        movie_subtitles::llm_segment::batch_jobs::run(&indices, |&i| submit(i)).await
    };
    let mut rows = Vec::new();
    let mut failed = 0;
    for (group, result) in jobs.iter().zip(results) {
        let count = group.len();
        match result {
            Ok(answers) => {
                for (chunk, answer) in answers {
                    match answer {
                        Ok(answer) => rows.push((chunk, answer)),
                        // An off-schema answer or a refusal proposes nothing,
                        // which is always safe: this chunk simply goes
                        // unchanged, as if the model had found no errors.
                        Err(error) if movie_subtitles::llm_segment::unusable_answer(&error) => {
                            eprintln!(
                                "proofread answer unusable, chunk left as is: {} {}: {error:#}",
                                chunk.language, chunk.imdb
                            );
                        }
                        Err(error) => {
                            failed += 1;
                            eprintln!(
                                "proofread request failed: {} {}: {error:#}",
                                chunk.language, chunk.imdb
                            );
                        }
                    }
                }
            }
            Err(error) => {
                failed += count;
                eprintln!("proofread call failed: {error:#}");
            }
        }
    }
    Ok((rows, failed))
}

fn corrected_lines(
    track: &Track,
    changes: &BTreeMap<(&str, &str, &str), &str>,
) -> Vec<SubtitleLine> {
    let mut lines = track.raw.clone();
    corrections::apply(&mut lines, track.language, &track.imdb);
    // Accepted changes are applied before segmentation even on --no-merge/eval.
    // Existing proofreads remain in force when this run proposes no replacement,
    // exactly as merge_file preserves them on disk.
    for (raw, line) in track.raw.iter().zip(&mut lines) {
        if let Some(corrected) = changes.get(&(track.language.code(), &track.imdb, &raw.sentence)) {
            line.sentence = (*corrected).to_owned();
        }
    }
    lines
}

fn sentence_items(sentences: Vec<KeyedSentence>) -> Vec<Item> {
    sentences
        .into_iter()
        .map(|sentence| Item {
            index: 0,
            key: sentence.sentence.clone(),
            text: sentence.sentence,
            read_only: !sentence.course_worthy,
        })
        .collect()
}

async fn sentence_chunks(
    tracks: &[Track],
    rows: &[Audited],
    samples: Option<&[Sample]>,
    live: bool,
) -> Result<Vec<Chunk>> {
    let changes = rows
        .iter()
        .filter(|row| row.accepted)
        .map(|row| {
            (
                (
                    row.language.as_str(),
                    row.imdb.as_str(),
                    row.cue.as_deref().unwrap(),
                ),
                row.corrected.as_str(),
            )
        })
        .collect();
    let selected: Vec<_> = tracks
        .iter()
        .filter(|track| {
            samples.is_none_or(|samples| {
                samples.iter().any(|sample| {
                    sample.language == track.language.code()
                        && sample.imdb == track.imdb
                        && matches!(sample.expectation, Expectation::Coherence { .. })
                })
            })
        })
        .collect();
    let prepared: Vec<_> = selected
        .iter()
        .map(|track| {
            let lines = corrected_lines(track, &changes);
            if movie_subtitles::llm_segment::uses_llm(track.language) {
                movie_subtitles::sentences::prepared_lines(&lines)
            } else {
                lines
            }
        })
        .collect();
    let llm_tracks: Vec<_> = selected
        .iter()
        .zip(&prepared)
        .filter(|(track, _)| movie_subtitles::llm_segment::uses_llm(track.language))
        .map(|(track, lines)| (lines.as_slice(), track.language))
        .collect();
    let mut splits = if llm_tracks.is_empty() {
        Vec::new()
    } else {
        let client = transport(movie_subtitles::llm_segment::batch_client()?, live);
        let (splits, report) = movie_subtitles::llm_segment::split_tracks(
            &client,
            &llm_tracks,
            movie_subtitles::llm_segment::print_progress(),
        )
        .await
        .context("proofread segmentation failed; no overlays merged")?;
        println!(
            "proofread segmentation: {} cues, {} requests, {} fallbacks",
            report.cues, report.asked, report.fallbacks
        );
        splits
    }
    .into_iter();
    // Rule segmentation is CPU-bound (parsley takes tens of seconds on a long
    // film) and there are thousands of tracks, so it runs across every core,
    // as generate-data does; one core took most of a night.
    let mut rules = BTreeMap::new();
    for track in &selected {
        if !movie_subtitles::llm_segment::uses_llm(track.language)
            && !rules.contains_key(&track.language)
        {
            let SubtitleSegmenter::Rules(segmenter) =
                SubtitleSegmenter::for_language(track.language)?
            else {
                unreachable!("rule languages get a rule segmenter")
            };
            rules.insert(track.language, segmenter);
        }
    }
    let mut ruled: Vec<Option<Vec<KeyedSentence>>> = {
        use rayon::prelude::*;
        selected
            .par_iter()
            .zip(&prepared)
            .map(|(track, lines)| {
                rules.get(&track.language).map(|segmenter| {
                    movie_subtitles::sentences::keyed_sentences_by_rules(
                        lines,
                        track.language,
                        &track.imdb,
                        segmenter,
                    )
                })
            })
            .collect()
    };
    let mut seen = BTreeMap::<(Language, &str), BTreeSet<String>>::new();
    let mut result = Vec::new();
    for ((track, lines), ruled) in selected.into_iter().zip(&prepared).zip(&mut ruled) {
        let sentences = match ruled.take() {
            Some(sentences) => sentences,
            None => movie_subtitles::sentences::keyed_sentences_from_splits(
                lines,
                &splits.next().expect("one split result per LLM track"),
                track.language,
                &track.imdb,
            ),
        };
        result.extend(
            chunks(
                track.language,
                &track.imdb,
                sentence_items(sentences),
                seen.entry((track.language, &track.imdb)).or_default(),
            )
            .into_iter()
            .filter(|chunk| {
                samples.is_none_or(|rows| {
                    rows.iter().any(|row| {
                        row.language == chunk.language
                            && row.imdb == chunk.imdb
                            && match &row.expectation {
                                Expectation::Coherence { sentence, .. } => chunk
                                    .items
                                    .iter()
                                    .any(|item| !item.read_only && &item.key == sentence),
                                _ => false,
                            }
                    })
                })
            }),
        );
    }
    if let Some(samples) = samples {
        for sample in samples {
            if let Expectation::Coherence { sentence, .. } = &sample.expectation {
                require_sample(&result, sample, sentence)?;
            }
        }
    }
    Ok(result)
}

fn print_counts(label: &str, chunks: &[Chunk]) {
    let mut counts = BTreeMap::<&str, (usize, usize)>::new();
    for chunk in chunks {
        let (items, requests) = counts.entry(&chunk.language).or_default();
        *items += chunk.items.iter().filter(|item| !item.read_only).count();
        *requests += 1;
    }
    for (code, (items, requests)) in counts {
        println!("{label} {code}: {items} items, {requests} requests");
    }
}

fn write_jsonl<T: Serialize>(path: &Path, rows: &[T]) -> Result<()> {
    let mut file = std::io::BufWriter::new(std::fs::File::create(path)?);
    for row in rows {
        serde_json::to_writer(&mut file, row)?;
        writeln!(file)?;
    }
    file.flush()?;
    Ok(())
}

fn evaluate(samples: &[Sample], chunks: &[Chunk], rows: &[Audited], flags: &[AuditedFlag]) {
    let (mut passed, mut total, mut tp, mut fp, mut missed) = (0, 0, 0, 0, 0);
    for sample in samples {
        match &sample.expectation {
            Expectation::Orthography { cue, expected } => {
                let input = chunks
                    .iter()
                    .filter(|chunk| chunk.language == sample.language && chunk.imdb == sample.imdb)
                    .flat_map(|chunk| &chunk.items)
                    .find(|item| !item.read_only && &item.key == cue)
                    .unwrap();
                let changed = rows.iter().find(|row| {
                    row.accepted
                        && row.language == sample.language
                        && row.imdb == sample.imdb
                        && row.cue.as_deref() == Some(cue)
                });
                let actual = changed.map_or(input.text.as_str(), |row| row.corrected.as_str());
                let matches = actual == expected;
                passed += usize::from(matches);
                total += 1;
                println!(
                    "{}",
                    serde_json::json!({"language":sample.language,"imdb":sample.imdb,"cue":cue,"changed":changed.is_some(),"actual":actual,"expected":expected,"matches":matches})
                );
            }
            Expectation::Coherence {
                sentence,
                incoherent,
            } => {
                let flag = flags.iter().find(|row| {
                    row.accepted
                        && row.language == sample.language
                        && row.imdb == sample.imdb
                        && row.sentence.as_deref() == Some(sentence)
                });
                tp += usize::from(flag.is_some() && *incoherent);
                fp += usize::from(flag.is_some() && !incoherent);
                missed += usize::from(flag.is_none() && *incoherent);
                println!(
                    "{}",
                    serde_json::json!({"language":sample.language,"imdb":sample.imdb,"sentence":sentence,"flagged":flag.is_some(),"expected_incoherent":incoherent,"reason":flag.map(|row| &row.reason)})
                );
            }
        }
    }
    if total != 0 {
        println!("orthography eval: {passed}/{total} match expected");
    }
    if samples
        .iter()
        .any(|sample| matches!(sample.expectation, Expectation::Coherence { .. }))
    {
        if tp + fp == 0 {
            println!("coherence precision: undefined (no positive predictions); {missed} missed");
        } else {
            println!(
                "coherence precision: {tp}/{} ({:.1}%); {fp} false positives, {missed} missed",
                tp + fp,
                100.0 * tp as f64 / (tp + fp) as f64
            );
        }
    }
}

#[tokio::main]
pub async fn run(options: Options) -> Result<()> {
    let samples: Option<Vec<Sample>> = options
        .eval
        .as_ref()
        .map(|path| -> Result<_> {
            std::fs::read_to_string(path)?
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| Ok(serde_json::from_str(line)?))
                .collect()
        })
        .transpose()?;
    let tracks = discover(&options, samples.as_deref())?;
    let cues = cue_chunks(&tracks, samples.as_deref())?;
    print_counts("orthography cues", &cues);
    if options.dry_run {
        println!("Coherence requests depend on corrected sentence segmentation; dry-run makes no API calls.");
        return Ok(());
    }
    let audit_dir = options.out.join("proofread");
    std::fs::create_dir_all(&audit_dir)?;
    let prefix = if samples.is_some() { "eval-" } else { "" };
    let (answers, failed) = judge::<Answer>(&cues, prompts::orthography, options.live).await?;
    let mut rows: Vec<_> = answers
        .into_iter()
        .flat_map(|(chunk, answer)| audit(chunk, answer))
        .collect();
    rows.sort_by(|a, b| {
        (&a.language, &a.imdb, &a.cue, a.index).cmp(&(&b.language, &b.imdb, &b.cue, b.index))
    });
    write_jsonl(&audit_dir.join(format!("{prefix}changes.jsonl")), &rows)?;
    let mut totals: BTreeMap<_, (usize, usize)> = cues
        .iter()
        .map(|chunk| (chunk.language.as_str(), (0, 0)))
        .collect();
    for row in &rows {
        let (accepted, rejected) = totals.get_mut(row.language.as_str()).unwrap();
        if row.accepted {
            *accepted += 1;
        } else {
            *rejected += 1;
        }
    }
    for (code, (accepted, rejected)) in totals {
        println!("orthography {code}: {accepted} accepted, {rejected} rejected");
    }
    for row in rows.iter().filter(|row| !row.accepted).take(10) {
        println!("rejected: {}", serde_json::to_string(row)?);
    }
    if failed != 0 {
        bail!(
            "{failed} orthography requests failed; audit saved, no overlays merged or eval scored"
        );
    }

    let sentences = sentence_chunks(&tracks, &rows, samples.as_deref(), options.live).await?;
    print_counts("coherence sentences", &sentences);
    let (answers, failed) =
        judge::<CoherenceAnswer>(&sentences, prompts::coherence, options.live).await?;
    let mut flags: Vec<_> = answers
        .into_iter()
        .flat_map(|(chunk, answer)| audit_flags(chunk, answer))
        .collect();
    flags.sort_by(|a, b| {
        (&a.language, &a.imdb, &a.sentence, a.index).cmp(&(
            &b.language,
            &b.imdb,
            &b.sentence,
            b.index,
        ))
    });
    write_jsonl(&audit_dir.join(format!("{prefix}incoherent.jsonl")), &flags)?;
    for flag in &flags {
        println!("incoherent: {}", serde_json::to_string(flag)?);
    }
    if failed != 0 {
        bail!("{failed} coherence requests failed; audit saved, no overlays merged or eval scored");
    }
    if let Some(samples) = samples {
        evaluate(&samples, &cues, &rows, &flags);
        return Ok(());
    }
    if options.no_merge {
        return Ok(());
    }
    let mut additions = BTreeMap::<String, Vec<Correction>>::new();
    for row in rows.into_iter().filter(|row| row.accepted) {
        additions
            .entry(row.language)
            .or_default()
            .push(Correction::Proofread {
                imdb: row.imdb,
                cue: row.cue.unwrap(),
                input: row.input.unwrap(),
                corrected: row.corrected,
                reason: row.reason,
            });
    }
    for flag in flags.into_iter().filter(|flag| flag.accepted) {
        additions
            .entry(flag.language)
            .or_default()
            .push(Correction::Incoherent {
                imdb: flag.imdb,
                sentence: flag.sentence.unwrap(),
                reason: flag.reason,
            });
    }
    for (code, entries) in additions {
        println!(
            "merging {} entries for {code}; rebuild to embed them",
            entries.len()
        );
        corrections::merge_file(
            Path::new(corrections::CORRECTIONS_DIR),
            Language::from_code(&code).unwrap(),
            entries,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pooled_languages_keep_their_prefixes_and_original_order() {
        let mut input = Vec::new();
        for (language, text) in [
            (Language::French, "À bientôt."),
            (Language::German, "Guten Tag."),
            (Language::French, "Bonsoir."),
        ] {
            input.extend(chunks(
                language,
                "test",
                items(&[text]),
                &mut BTreeSet::new(),
            ));
        }
        let systems: BTreeMap<_, _> = ["fra", "deu"]
            .into_iter()
            .map(|code| (code, prompts::orthography(code)))
            .collect();
        let client = ChatClient::new("not-a-real-key", "test-model");
        let jobs = movie_subtitles::llm_segment::batch_jobs::partition::<_, Answer>(
            &client,
            &input,
            |chunk| judge_messages(chunk, &systems),
        )
        .unwrap();
        assert_eq!(jobs.len(), 1, "small languages belong to one pooled job");
        assert_eq!(
            jobs[0]
                .iter()
                .map(|chunk| chunk.language.as_str())
                .collect::<Vec<_>>(),
            ["fra", "deu", "fra"]
        );
        for (actual, expected) in jobs[0].iter().zip(&input) {
            assert!(std::ptr::eq(actual, expected));
            let messages = serde_json::to_value(judge_messages(actual, &systems)).unwrap();
            assert_eq!(
                messages[0]["content"][0]["text"],
                prompts::orthography(&actual.language)
            );
            assert_eq!(
                messages[1]["content"][0]["text"],
                serde_json::to_string(actual).unwrap()
            );
        }
    }

    #[test]
    fn live_scheduling_stays_at_six_calls() {
        for count in [1_usize, 16, 17, 4_999, 5_000, 5_001, 40_001] {
            let requests = vec![(); count];
            assert!(requests.chunks(count.div_ceil(LIVE_CALLS)).count() <= LIVE_CALLS);
        }
    }

    #[test]
    fn live_is_an_explicit_opt_in() {
        #[derive(clap::Parser)]
        struct Cli {
            #[command(flatten)]
            options: Options,
        }
        use clap::Parser;
        assert!(!Cli::parse_from(["proofread"]).options.live);
        assert!(Cli::parse_from(["proofread", "--live"]).options.live);
        let client = ChatClient::new("not-a-real-key", "test-model");
        let client = transport(client, false);
        assert_eq!(client.small_batch_threshold, 16);
        assert_eq!(transport(client, true).small_batch_threshold, usize::MAX);
    }

    fn items(texts: &[&str]) -> Vec<Item> {
        texts
            .iter()
            .map(|text| Item {
                index: 0,
                text: (*text).into(),
                key: (*text).into(),
                read_only: false,
            })
            .collect()
    }

    #[test]
    fn accepted_fixes_keep_apostrophe_and_quote_style() {
        for (input, corrected, expected) in [
            ("qu’a", "qu'à", "qu’à"),
            ("qu'a", "qu’à", "qu'à"),
            ("Il dit “a”.", "Il dit \"à\".", "Il dit “à”."),
            ("Il dit \"a\".", "Il dit “à”.", "Il dit \"à\"."),
            ("a”.", "à\".", "à”."),
        ] {
            let chunk = chunks(
                Language::French,
                "test",
                items(&[input]),
                &mut BTreeSet::new(),
            )
            .remove(0);
            let rows = audit(
                &chunk,
                Answer {
                    changes: vec![Change {
                        index: 0,
                        corrected: corrected.into(),
                        reason: "accent".into(),
                    }],
                },
            );
            assert!(rows[0].accepted);
            assert_eq!(rows[0].corrected, expected);
        }
    }

    #[test]
    fn punctuation_guard_preserves_boundaries_but_allows_orthography() {
        for (input, corrected, accepted) in [
            ("Au revoir Perrin.", "Au revoir, Perrin.", false),
            ("Où est elle ?", "Où est-elle ?", true),
            ("Bens....", "Bens...", true),
            ("Bens..", "Bens…", true),
            ("Ça va allez.", "Ça va, allez.", false),
            ("Un. Deux.", "Un deux...", false),
            ("Cómo?", "¿Cómo?", true),
            ("# Sono pronto #", "Sono pronto", false),
            ("♪ Sono pronto ♫", "Sono pronto", false),
            // Non-Latin marks are boundaries too; only their width is free.
            ("いくらするの？", "いくらするの。", false),
            ("いくらするの?", "いくらするの？", true),
            ("我不知道，你呢？", "我不知道你呢？", false),
            ("हम यहाँ हैं। चलो", "हम यहाँ हैं चलो", false),
            // A mark may not move, even when the sequence stays the same.
            ("Non. Je viens", "Non je viens.", false),
            ("Tout à l heure .", "Tout à l'heure.", true),
            // Symbols carry meaning and are frozen like letters.
            ("Growth rate is at 27%.", "Growth rate is at 27.", false),
            ("12.000€!", "12.000$!", false),
        ] {
            let chunk = chunks(
                Language::French,
                "test",
                items(&[input]),
                &mut BTreeSet::new(),
            )
            .remove(0);
            let rows = audit(
                &chunk,
                Answer {
                    changes: vec![Change {
                        index: 0,
                        corrected: corrected.into(),
                        reason: "test".into(),
                    }],
                },
            );
            assert_eq!(rows[0].accepted, accepted, "{input} → {corrected}");
        }
    }

    #[test]
    fn guard_rejects_wrong_duplicate_and_context_indices() {
        let chunk = chunks(
            Language::French,
            "test",
            items(&["fais tu ?", "A tout", "A tout"]),
            &mut BTreeSet::new(),
        )
        .remove(0);
        let answer = Answer {
            changes: vec![
                Change {
                    index: 0,
                    corrected: "À tout".into(),
                    reason: "wrong cue".into(),
                },
                Change {
                    index: 1,
                    corrected: "À tout".into(),
                    reason: "accent".into(),
                },
                Change {
                    index: 2,
                    corrected: "À tout".into(),
                    reason: "context".into(),
                },
                Change {
                    index: 30,
                    corrected: "À tout".into(),
                    reason: "outside".into(),
                },
            ],
        };
        let rows = audit(&chunk, answer);
        assert_eq!(
            rows.iter().map(|row| row.accepted).collect::<Vec<_>>(),
            [false, true, false, false]
        );
        let rows = audit(
            &chunk,
            Answer {
                changes: vec![
                    Change {
                        index: 0,
                        corrected: "fais-tu ?".into(),
                        reason: "one".into(),
                    },
                    Change {
                        index: 0,
                        corrected: "Fais-tu ?".into(),
                        reason: "two".into(),
                    },
                ],
            },
        );
        assert!(rows.iter().all(|row| !row.accepted));
        let flags = audit_flags(
            &chunk,
            CoherenceAnswer {
                flags: vec![Flag {
                    index: 30,
                    reason: "wrong index".into(),
                }],
            },
        );
        assert!(!flags[0].accepted);
    }

    #[test]
    fn dedup_preserves_track_order_and_read_only_neighbours() {
        let texts: Vec<_> = (0..65).map(|i| format!("cue {i}")).collect();
        let refs: Vec<_> = texts.iter().map(String::as_str).collect();
        let mut seen = BTreeSet::new();
        let first = chunks(Language::French, "test", items(&refs), &mut seen);
        assert_eq!(first.len(), 3);
        assert_eq!(first[1].before_read_only, ["cue 28", "cue 29"]);
        assert_eq!(first[1].after_read_only, ["cue 60", "cue 61"]);
        assert!(chunks(Language::French, "test", items(&refs), &mut seen).is_empty());
        let second = chunks(
            Language::French,
            "test",
            items(&["cue 10", "new", "cue 9"]),
            &mut seen,
        );
        assert_eq!(
            second[0]
                .items
                .iter()
                .map(|item| item.read_only)
                .collect::<Vec<_>>(),
            [true, false, true]
        );
        assert_eq!(second[0].items[2].text, "cue 9");
    }

    #[test]
    fn coherence_only_judges_worthy_sentences_but_retains_context() {
        let sentences = vec![
            KeyedSentence {
                sentence: "Not for the course".into(),
                start_ms: 0,
                end_ms: 1,
                course_worthy: false,
            },
            KeyedSentence {
                sentence: "A sentence.".into(),
                start_ms: 1,
                end_ms: 2,
                course_worthy: true,
            },
        ];
        let chunk = chunks(
            Language::English,
            "test",
            sentence_items(sentences),
            &mut BTreeSet::new(),
        )
        .remove(0);
        assert!(chunk.items[0].read_only);
        assert!(!chunk.items[1].read_only);
        let flags = audit_flags(
            &chunk,
            CoherenceAnswer {
                flags: vec![Flag {
                    index: 0,
                    reason: "read-only".into(),
                }],
            },
        );
        assert!(!flags[0].accepted);
    }
}
