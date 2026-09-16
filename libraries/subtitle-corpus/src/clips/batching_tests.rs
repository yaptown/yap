use super::*;
use base64::Engine;
use std::cell::{Cell, RefCell};
use std::io::Write;

fn matrix(hash: u64) -> FrameMatrix {
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    // fp16 -2 and -1: a single nonblank frame with a recognizable label.
    encoder.write_all(&[0x00, 0xc0, 0x00, 0xbc]).unwrap();
    FrameMatrix::decode(&phoneme_verify::FrameMatrixPayload::Legacy(
        phoneme_verify::LegacyFrameMatrixPayload {
            shape: vec![1, 2],
            dtype: "float16".into(),
            encoding: "zlib+base64".into(),
            blank_id: 0,
            vocab: vec!["<pad>".into(), format!("a{hash}")],
            data: base64::engine::general_purpose::STANDARD.encode(encoder.finish().unwrap()),
        },
    ))
    .unwrap()
}

fn clip(film: usize, index: usize, hash: u64) -> Clip {
    Clip {
        producers: Producers::default(),
        measured: false,
        audio_hash: Some(hash),
        sentence: format!("film {film} clip {index}"),
        imdb_id: film.to_string(),
        start_ms: (index as i64 + 1) * 1000,
        // Deliberately scrambled durations, unrelated to persisted clip order.
        end_ms: (index as i64 + 1) * 1000 + 600 + ((index * 17 + film * 7) % 25) as i64 * 100,
        pad_before_ms: 0,
        pad_after_ms: 0,
        repaired_before_ms: 0,
        repaired_after_ms: 0,
        words: Vec::new(),
        speaker: None,
        transcript_wer: 0.0,
        audio_event_overlap: false,
        clear_before_ms: 300,
        clear_after_ms: 300,
        target_ipa: vec![format!("a{hash}")],
        oov: Vec::new(),
        ratio: None,
        logp_target_per_phoneme: None,
        edge_logp_start: None,
        edge_logp_end: None,
        lead_speech: None,
        tail_speech: None,
        lead_rms: None,
        voiced: Some(1.0),
        heard_ipa: Vec::new(),
        passed: false,
        reject: None,
    }
}

fn film(root: &Path, index: usize, count: usize) -> PreparedFilm {
    let dir = root.join(index.to_string());
    std::fs::create_dir_all(&dir).unwrap();
    let gate = Gate::default();
    let (language, code, min_ratio) = if index.is_multiple_of(2) {
        (Language::French, "fra", -1000.0)
    } else {
        (Language::German, "deu", 1000.0)
    };
    let mut clips = Vec::new();
    let mut pending = Vec::new();
    for i in 0..count {
        // Same WAV key across films; logical slots must not collapse.
        let hash = if i == 5 { 42 } else { (index * 100 + i) as u64 };
        let mut clip = clip(index, i, hash);
        if index == 7 && i == 5 {
            // The unclamped duration is 600 ms, but the actual cut is 550.
            // Put it late in discovery to catch a sort that forgets clamping.
            clip.start_ms = 50;
            clip.end_ms = 550;
            clip.pad_before_ms = 100;
        }
        clips.push(Some(clip));
        pending.push(PendingClip { index: i, hash });
    }
    PreparedFilm {
        dir,
        language,
        provenance: Provenance {
            inputs: Inputs {
                format: FORMAT_VERSION,
                subtitle_digest: "subtitle".into(),
                transcript_digest: "transcript".into(),
                segmentation: "test-segmentation".into(),
                language: code.into(),
                audio: AudioInput {
                    filename: "film.mkv".into(),
                    stream_index: 0,
                    duration_ms: 100000,
                },
            },
            cut: Cut {
                preferred_clear_ms: gate.preferred_clear_ms,
                min_clear_ms: gate.min_clear_ms,
                speech_threshold: gate.speech_threshold,
            },
            gate: GateCuts {
                min_ratio: Some(min_ratio),
                min_edge_logp: gate.min_edge_logp,
                max_pad_speech: gate.max_pad_speech,
                max_lead_rms: gate.max_lead_rms,
                min_voiced: gate.min_voiced,
                min_verbatim: crate::verbatim::min_fraction(code),
            },
        },
        summary: FilmSummary {
            sentences: count,
            aligned: count,
            ..Default::default()
        },
        clips,
        pending,
    }
}

#[tokio::test]
async fn global_batches_sort_compact_route_and_prefetch_after_discovery() {
    let root = tempfile::tempdir().unwrap();
    let gate = Gate::default();
    let mut inputs = Vec::new();
    for index in 0..8 {
        let mut film = film(root.path(), index, 25);
        // Discovery has already resolved a frame-cache hit and a gate reject.
        let cached = film.clips[0].as_mut().unwrap();
        score_clip(
            cached,
            &matrix((index * 100) as u64),
            film.provenance.gate.min_ratio.unwrap(),
            &gate,
        );
        film.clips[1].as_mut().unwrap().reject = Some("cut: bad audio".into());
        film.pending.retain(|p| p.index >= 2);
        inputs.push((index, Ok(FilmWork::Prepared(Box::new(film)))));
    }
    let current = film(root.path(), 8, 0);
    let current_path = clips_path(&current.dir);
    let current_summary = finish_film(FilmWork::Prepared(Box::new(current))).unwrap();
    let current_bytes = std::fs::read(&current_path).unwrap();
    inputs.push((8, Ok(FilmWork::Current(current_summary))));
    inputs.push((9, Err(anyhow::anyhow!("film preparation failed"))));

    let discovered = Cell::new(0);
    let active = Cell::new(0);
    let peak = Cell::new(0);
    let request_count = Cell::new(0);
    let lookahead_ready = tokio::sync::Notify::new();
    let second_request_finished = tokio::sync::Notify::new();
    let completion_order = RefCell::new(Vec::new());
    let requests_active = Cell::new(0);
    let requests_peak = Cell::new(0);
    let batches = RefCell::new(Vec::new());
    let durations = RefCell::new(Vec::new());
    let preparation = futures::stream::iter(inputs).then(|item| async {
        tokio::task::yield_now().await;
        discovered.set(discovered.get() + 1);
        item
    });
    let mapped = map_staged(
        preparation,
        async |cut| {
            assert_eq!(
                discovered.get(),
                10,
                "request preparation crossed discovery barrier"
            );
            active.set(active.get() + 1);
            peak.set(peak.get().max(active.get()));
            tokio::task::yield_now().await;
            active.set(active.get() - 1);
            match cut.start / 1000 - 1 {
                2 => bail!("invalid WAV"),
                3 => return Ok(FrameInput::Cached(Box::new(matrix(cut.hash)))),
                4 => bail!("cached matrix cannot decode"),
                _ => {}
            }
            request_count.set(request_count.get() + 1);
            if request_count.get() == 128 {
                lookahead_ready.notify_one();
            }
            Ok(FrameInput::Request((
                cut.hash,
                cut.end + cut.after - (cut.start - cut.before).max(0),
            )))
        },
        async |requests: Vec<(u64, i64)>, _activity| {
            assert_eq!(discovered.get(), 10, "inference crossed discovery barrier");
            let ordinal = batches.borrow().len();
            requests_active.set(requests_active.get() + 1);
            requests_peak.set(requests_peak.get().max(requests_active.get()));
            batches.borrow_mut().push(requests.len());
            durations
                .borrow_mut()
                .extend(requests.iter().map(|(_, duration)| *duration));
            if ordinal == 0 {
                // Prove preparation overlap and two in-flight requests, with
                // the second completing first, not just two queued futures.
                tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    lookahead_ready.notified(),
                )
                .await
                .unwrap();
                tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    second_request_finished.notified(),
                )
                .await
                .unwrap();
            }
            let results = requests
                .into_iter()
                .map(|(hash, _)| {
                    if hash == 107 {
                        Err(anyhow::anyhow!("one endpoint item failed"))
                    } else {
                        Ok(matrix(hash))
                    }
                })
                .collect();
            requests_active.set(requests_active.get() - 1);
            completion_order.borrow_mut().push(ordinal);
            if ordinal == 1 {
                second_request_finished.notify_one();
            }
            results
        },
        &gate,
    )
    .await;
    assert_eq!(*batches.borrow(), [64, 64, 32]);
    assert_eq!(requests_peak.get(), INFERENCE_REQUESTS_IN_FLIGHT);
    assert_eq!(requests_active.get(), 0);
    assert_eq!(
        completion_order.borrow()[0],
        1,
        "test must complete the second request first"
    );
    assert!(durations.borrow().windows(2).all(|pair| pair[0] <= pair[1]));
    assert!(
        (2..=8).contains(&peak.get()),
        "bounded concurrent preparation: {}",
        peak.get()
    );
    assert_eq!(request_count.get(), 160);
    for (index, outcome) in mapped {
        if index == 9 {
            assert!(outcome.is_err());
            assert!(!clips_path(&root.path().join("9")).exists());
            continue;
        }
        let work = outcome.unwrap();
        if let FilmWork::Prepared(film) = &work {
            for (i, clip) in film.clips.iter().enumerate() {
                if i == 2 || i == 4 || (index == 1 && i == 7) {
                    assert!(clip.is_none());
                    continue;
                }
                let clip = clip.as_ref().unwrap();
                assert_eq!(clip.sentence, format!("film {index} clip {i}"));
                if i == 1 {
                    assert_eq!(clip.reject.as_deref(), Some("cut: bad audio"));
                } else {
                    assert_eq!(clip.heard_ipa, clip.target_ipa, "misrouted matrix");
                    assert_eq!(clip.passed, index % 2 == 0, "lost film-specific threshold");
                }
            }
        }
        let path = clips_path(&root.path().join(index.to_string()));
        if index < 8 {
            assert!(finish_film(work).is_err());
            assert!(!path.exists(), "partial success must not publish a header");
        } else {
            finish_film(work).unwrap();
            assert!(stored_provenance(&path).is_some());
        }
    }
    assert_eq!(std::fs::read(current_path).unwrap(), current_bytes);
}

#[tokio::test]
async fn all_inference_failures_leave_no_current_header() {
    let root = tempfile::tempdir().unwrap();
    let mut broken = film(root.path(), 0, 1);
    // Failed candidates must never produce a current header.
    broken.dir = root.path().join("missing-parent").join("film");
    let good = film(root.path(), 1, 1);
    let mapped = map_staged(
        futures::stream::iter(vec![
            (0, Ok(FilmWork::Prepared(Box::new(broken)))),
            (1, Ok(FilmWork::Prepared(Box::new(good)))),
        ]),
        async |cut| Ok(FrameInput::Request(cut.hash)),
        async |requests: Vec<u64>, _activity| {
            requests
                .into_iter()
                .map(|_| Err(anyhow::anyhow!("request-wide failure")))
                .collect()
        },
        &Gate::default(),
    )
    .await;
    let results: Vec<_> = mapped
        .into_iter()
        .map(|(_, outcome)| outcome.and_then(finish_film))
        .collect();
    assert!(results.iter().all(Result::is_err));
    assert!(!clips_path(&root.path().join("1")).exists());
}

#[tokio::test]
async fn empty_or_resolved_selection_never_prepares_or_infers() {
    let root = tempfile::tempdir().unwrap();
    for inputs in [
        Vec::new(),
        vec![
            (0, Ok(FilmWork::Current(FilmSummary::default()))),
            (1, Ok(FilmWork::Prepared(Box::new(film(root.path(), 1, 0))))),
            (2, Err(anyhow::anyhow!("preflight"))),
        ],
    ] {
        map_staged(
            futures::stream::iter(inputs),
            async |_| -> Result<FrameInput<()>> { panic!("nothing to prepare") },
            async |_, _activity| panic!("nothing to infer"),
            &Gate::default(),
        )
        .await;
    }
}

#[tokio::test]
async fn all_request_preparations_fail_without_an_empty_inference_request() {
    let root = tempfile::tempdir().unwrap();
    let mapped = map_staged(
        futures::stream::iter(vec![(
            0,
            Ok(FilmWork::Prepared(Box::new(film(root.path(), 0, 4)))),
        )]),
        async |_| -> Result<FrameInput<()>> { bail!("cached matrix or WAV failed to decode") },
        async |_, _activity| panic!("failed preparations cannot trigger inference"),
        &Gate::default(),
    )
    .await;
    let (_, outcome) = mapped.into_iter().next().unwrap();
    assert!(finish_film(outcome.unwrap()).is_err());
    assert!(!clips_path(&root.path().join("0")).exists());
}

#[test]
fn completeness_and_strict_reading_are_independent_of_provenance() {
    let root = tempfile::tempdir().unwrap();
    let unresolved = film(root.path(), 0, 1);
    let path = clips_path(&unresolved.dir);
    assert!(finish_film(FilmWork::Prepared(Box::new(unresolved))).is_err());
    assert!(!path.exists(), "pending Some(Clip) is not a verdict");

    let mut complete = film(root.path(), 0, 2);
    complete.clips[0].as_mut().unwrap().passed = true;
    complete.clips[1].as_mut().unwrap().reject = Some("cut: explicit failure".into());
    finish_film(FilmWork::Prepared(Box::new(complete))).unwrap();
    let original = std::fs::read_to_string(&path).unwrap();
    assert_eq!(read_clips(&path).unwrap().len(), 2);
    let lines: Vec<_> = original.lines().collect();
    for corrupt in [
        lines[..2].join("\n"),
        format!("{}\nmalformed\n{}\n", lines[0], lines[2]),
        format!(
            "{}\n{}\n{}\n",
            lines[0],
            lines[1],
            lines[1].replace("\"passed\":true", "\"passed\":false")
        ),
    ] {
        std::fs::write(&path, corrupt).unwrap();
        assert!(read_clips(&path).is_err());
        assert!(stored_provenance(&path).is_none());
    }
}

#[tokio::test]
async fn freshness_tiers_regate_without_probes_and_preserve_failures() {
    let root = tempfile::tempdir().unwrap();
    let mut film = film(root.path(), 0, 2);
    let dir = film.dir.clone();
    for name in ["subtitle.srt", "transcript.jsonl", "audio.opus"] {
        std::fs::write(
            dir.join(name),
            b"not decodable; cheap paths must not read contents",
        )
        .unwrap();
    }
    let stamp = crate::sync::AudioStamp {
        filename: "source.mkv".into(),
        duration_ms: 100000,
        stream: crate::sync::AudioStreamIdentity {
            stream_index: 2,
            codec: "opus".into(),
            channels: 2,
            channel_layout: "stereo".into(),
        },
    };
    std::fs::write(dir.join("audio.json"), serde_json::to_vec(&stamp).unwrap()).unwrap();
    let gate = Gate::default();
    film.provenance = current_provenance(&dir, Language::French, "fra", &gate).unwrap();
    let original = film.provenance.clone();
    score_clip(film.clips[0].as_mut().unwrap(), &matrix(0), -2.0, &gate);
    film.clips[0].as_mut().unwrap().producers.g2p = Some("older-renderer".into());
    film.clips[1].as_mut().unwrap().reject = Some("cut: permanent failure".into());
    let report = crate::verbatim::Report {
        format: crate::verbatim::FORMAT,
        subtitle_digest: original.inputs.subtitle_digest.clone(),
        transcript_digest: original.inputs.transcript_digest.clone(),
        min_fraction: 0.25,
        measure: crate::verbatim::Measure {
            eligible: 100,
            placed: 50,
            fraction: 0.5,
            aligned: None,
            verdict: crate::verbatim::Verdict::Verbatim,
        },
    };
    std::fs::write(
        crate::verbatim::report_path(&dir),
        serde_json::to_vec(&report).unwrap(),
    )
    .unwrap();
    finish_film(FilmWork::Prepared(Box::new(film))).unwrap();
    assert_eq!(existing_work(&dir, &original).0, Work::Nothing);
    let mut changed = original.clone();
    changed.inputs.audio.stream_index += 1;
    assert_eq!(original.work(&changed), Work::Redo("audio changed"));
    changed = original.clone();
    changed.inputs.transcript_digest.push('x');
    assert_eq!(original.work(&changed), Work::Redo("transcript changed"));
    changed = original.clone();
    changed.inputs.segmentation.push('x');
    assert_eq!(original.work(&changed), Work::Redo("inputs changed"));
    changed = original.clone();
    changed.cut.min_clear_ms += 1;
    assert_eq!(original.work(&changed), Work::Redo("cut changed"));
    changed = original.clone();
    changed.gate.max_lead_rms += 1.0;
    assert_eq!(original.work(&changed), Work::Regate);
    let movie = Movie {
        imdb_id: "0".into(),
        title: "fixture".into(),
        year: None,
        path: dir.clone(),
        original_language: "French".into(),
        source: crate::library::Source::Missing,
    };
    let store = osmo::Store::open(root.path().join("cache"));
    // Tightening and relaxing a film gate retain the exact scored rows. No
    // valid transcript, audio/profile or endpoint exists in this fixture.
    for (threshold, passed) in [(0.25, true), (0.75, false), (0.25, true)] {
        let gate = Gate {
            min_verbatim: Some(threshold),
            ..Gate::default()
        };
        prepare_film(&store, &movie, &dir, &gate, 1, false)
            .await
            .unwrap();
        let rows = read_clips(&clips_path(&dir)).unwrap();
        assert_eq!(rows[0].passed, passed);
        assert_eq!(rows[0].producers.g2p.as_deref(), Some("older-renderer"));
        assert_eq!(rows[1].reject.as_deref(), Some("cut: permanent failure"));
    }
    let mut rejected_report = report;
    rejected_report.measure.fraction = 0.0;
    rejected_report.measure.placed = 0;
    std::fs::write(
        crate::verbatim::report_path(&dir),
        serde_json::to_vec(&rejected_report).unwrap(),
    )
    .unwrap();
    for requested in [true, false] {
        assert!(prepare_film(&store, &movie, &dir, &gate, 1, requested)
            .await
            .is_err());
        assert!(
            interrupted_refresh(&dir),
            "film-level preflight must not erase refresh intent"
        );
        assert_eq!(read_manifest(&clips_path(&dir)).unwrap().1.len(), 2);
    }
    let (_, rows) = read_manifest(&clips_path(&dir)).unwrap();
    write_clips(&dir, original.clone(), &rows, FilmSummary::default()).unwrap();
    std::fs::remove_file(crate::verbatim::report_path(&dir)).unwrap();
    assert_eq!(
        existing_work(&dir, &original).0,
        Work::Redo("verbatim measurement missing or stale")
    );
    std::fs::remove_file(dir.join("audio.json")).unwrap();
    assert_eq!(
        existing_work(&dir, &original).0,
        Work::Redo("audio stamp missing")
    );
    assert!(prepare_film(&store, &movie, &dir, &gate, 1, false)
        .await
        .err()
        .unwrap()
        .to_string()
        .contains("audio stamp missing"));
}

#[test]
fn clip_keys_use_audio_and_exact_token_boundaries_not_producers() {
    let mut clip = clip(0, 0, 42);
    clip.target_ipa = vec!["t".into(), "ʃ".into()];
    let key = clip_key(42, &clip.target_ipa);
    clip.producers.g2p = Some("different renderer".into());
    assert_eq!(clip_key(42, &clip.target_ipa), key);
    assert_ne!(clip_key(43, &clip.target_ipa), key);
    assert_ne!(clip_key(42, &["tʃ".into()]), key);
}

#[tokio::test]
async fn interrupted_refresh_preserves_rows_but_regenerates_old_target_values() {
    let root = tempfile::tempdir().unwrap();
    let mut film = film(root.path(), 0, 1);
    film.clips[0].as_mut().unwrap().passed = true;
    let dir = film.dir.clone();
    let provenance = film.provenance.clone();
    finish_film(FilmWork::Prepared(Box::new(film))).unwrap();
    let store = osmo::Store::open(root.path().join("cache"));
    let mut old =
        phoneme_verify::cached_model_target(&store, Language::French, "bonjour", 42, false)
            .await
            .unwrap();
    old.renderer = "old-renderer".into();
    old.phonemized.phonemes = vec!["old-label".into()];
    let key = format!(
        "phoneme-target/{:016x}",
        xxhash_rust::xxh3::xxh3_64(
            &serde_json::to_vec(&(Language::French.g2p_lang().unwrap(), "bonjour", 42_u64))
                .unwrap()
        )
    );
    store
        .write(&key, &serde_json::to_vec(&old).unwrap())
        .await
        .unwrap();
    assert_eq!(
        phoneme_verify::cached_model_target(&store, Language::French, "bonjour", 42, false)
            .await
            .unwrap()
            .renderer,
        "old-renderer"
    );
    begin_refresh(&dir, &provenance).unwrap();
    assert!(read_file(&clips_path(&dir)).is_err());
    assert_eq!(read_manifest(&clips_path(&dir)).unwrap().1.len(), 1);
    // A normal subsequent run sees the marker even without the CLI flag.
    let refreshed = phoneme_verify::cached_model_target(
        &store,
        Language::French,
        "bonjour",
        42,
        interrupted_refresh(&dir),
    )
    .await
    .unwrap();
    assert_eq!(refreshed.renderer, phoneme_verify::model_target_identity());
    assert_ne!(refreshed.phonemized.phonemes, old.phonemized.phonemes);
    let again = phoneme_verify::cached_model_target(&store, Language::French, "bonjour", 42, false)
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(again).unwrap(),
        serde_json::to_value(refreshed).unwrap()
    );
    std::fs::write(clips_path(&dir), "corrupt").unwrap();
    assert!(
        !interrupted_refresh(&dir),
        "corruption is not a refresh request"
    );
}

#[test]
fn model_inventory_counts_observed_rows_and_unreadable_files() {
    let root = tempfile::tempdir().unwrap();
    let mut film = film(root.path(), 0, 3);
    for clip in film.clips.iter_mut().flatten() {
        clip.passed = true;
    }
    for clip in film.clips[..2].iter_mut().flatten() {
        clip.producers.model = Some(phoneme_verify::ModelIdentity {
            model_id: "actual/model".into(),
            model_revision: "full-revision".into(),
            decoder_version: Some("decoder".into()),
            deploy_marker: Some(clip.sentence.clone()),
        });
    }
    let path = clips_path(&film.dir);
    finish_film(FilmWork::Prepared(Box::new(film))).unwrap();
    let old = root.path().join("old.jsonl");
    let corrupt = root.path().join("corrupt.jsonl");
    std::fs::write(&old, "{\"format\":11,\"model\":\"not a row producer\"}\n").unwrap();
    std::fs::write(&corrupt, "broken").unwrap();
    let counts = count_models([path, old, corrupt]);
    assert_eq!(
        counts.models["actual/model@full-revision decoder=decoder"],
        2
    );
    assert_eq!(
        counts.models.len(),
        1,
        "deploy markers do not split a checkpoint"
    );
    assert_eq!(counts.no_model, 1);
    assert_eq!(counts.unreadable_files, 2);
}

#[test]
fn inference_metrics_use_logical_fill_and_completed_clips_over_phase_time() {
    let empty = InferenceProgress::default();
    assert_eq!(empty.fill_rate(), 0.0);
    assert_eq!(empty.clips_per_minute(std::time::Duration::ZERO), 0.0);
    let progress = InferenceProgress {
        requests: 3,
        submitted: 160,
        completed: 128,
        failed: 2,
    };
    assert_eq!(progress.fill_rate(), 160.0 / 192.0);
    // Completed failed attempts count toward throughput and are also reported
    // separately. In-flight submissions are not yet completed clips.
    assert_eq!(
        progress.clips_per_minute(std::time::Duration::from_secs(120)),
        64.0
    );
}

#[test]
fn recut_hash_must_match_discovery() {
    let wav = b"the exact original WAV";
    let hash = xxhash_rust::xxh3::xxh3_64(wav);
    check_recut_hash(wav, hash).unwrap();
    assert!(check_recut_hash(b"a different cut", hash).is_err());
}

#[tokio::test]
async fn audio_only_current_film_skips_model_and_regates_film_verbatim() {
    use crate::verbatim::{Measure, Report, Verdict};
    let root = tempfile::tempdir().unwrap();
    let mut film = film(root.path(), 0, 1);
    let clip = film.clips[0].as_mut().unwrap();
    clip.measured = true;
    clip.passed = true;
    clip.target_ipa.clear();
    film.language = Language::Korean;
    film.provenance.inputs.language = "kor".into();
    film.provenance.gate.min_ratio = None;
    film.provenance.inputs.segmentation = movie_subtitles::segment::provenance(Language::Korean);
    let dir = film.dir.clone();
    std::fs::write(
        dir.join("audio.json"),
        serde_json::to_vec(&crate::sync::AudioStamp {
            filename: film.provenance.inputs.audio.filename.clone(),
            duration_ms: film.provenance.inputs.audio.duration_ms,
            stream: crate::sync::AudioStreamIdentity {
                stream_index: 0,
                codec: "opus".into(),
                channels: 2,
                channel_layout: "stereo".into(),
            },
        })
        .unwrap(),
    )
    .unwrap();
    for name in ["subtitle.srt", "transcript.jsonl", "audio.opus"] {
        std::fs::write(dir.join(name), b"fixture; must not be decoded").unwrap();
    }
    film.provenance.inputs.subtitle_digest =
        crate::transcript::source_digest(&dir.join("subtitle.srt")).unwrap();
    film.provenance.inputs.transcript_digest =
        crate::transcript::source_digest(&dir.join("transcript.jsonl")).unwrap();
    let mut report = Report {
        format: crate::verbatim::FORMAT,
        subtitle_digest: film.provenance.inputs.subtitle_digest.clone(),
        transcript_digest: film.provenance.inputs.transcript_digest.clone(),
        min_fraction: crate::verbatim::min_fraction("kor"),
        measure: Measure {
            eligible: 30,
            placed: 30,
            fraction: 1.0,
            aligned: None,
            verdict: Verdict::Verbatim,
        },
    };
    std::fs::write(
        crate::verbatim::report_path(&dir),
        serde_json::to_vec(&report).unwrap(),
    )
    .unwrap();
    finish_film(FilmWork::Prepared(Box::new(film))).unwrap();
    let movie = Movie {
        imdb_id: "0".into(),
        title: "fixture".into(),
        year: None,
        path: dir.clone(),
        original_language: "Korean".into(),
        source: crate::library::Source::Missing,
    };
    let store = osmo::Store::open(root.path().join("cache"));
    assert!(matches!(
        prepare_film(&store, &movie, &dir, &Gate::default(), 1, false)
            .await
            .unwrap(),
        FilmWork::Current(_)
    ));
    report.measure.fraction = 0.0;
    report.measure.placed = 0;
    report.measure.verdict = Verdict::Paraphrase;
    std::fs::write(
        crate::verbatim::report_path(&dir),
        serde_json::to_vec(&report).unwrap(),
    )
    .unwrap();
    prepare_film(&store, &movie, &dir, &Gate::default(), 1, false)
        .await
        .unwrap();
    let clips = read_clips(&clips_path(&dir)).unwrap();
    assert!(!clips[0].passed);
    assert!(clips[0]
        .reject
        .as_deref()
        .unwrap()
        .contains("film-level verbatim"));
    report.measure.fraction = 1.0;
    report.measure.placed = 30;
    std::fs::write(
        crate::verbatim::report_path(&dir),
        serde_json::to_vec(&report).unwrap(),
    )
    .unwrap();
    prepare_film(&store, &movie, &dir, &Gate::default(), 1, false)
        .await
        .unwrap();
    assert!(read_clips(&clips_path(&dir)).unwrap()[0].passed);
}
