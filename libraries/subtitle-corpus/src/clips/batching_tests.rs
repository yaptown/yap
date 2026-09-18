use super::*;
use base64::Engine;
use std::cell::Cell;
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
        pending.push(PendingClip {
            index: i,
            wav: save_cut(&dir, &hash.to_le_bytes()).unwrap(),
        });
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
async fn discovery_precedes_inference_and_results_route_by_slot() {
    let root = tempfile::tempdir().unwrap();
    let discovered = Cell::new(0);
    let inputs: Vec<_> = (0..3)
        .map(|index| {
            (
                index,
                Ok(FilmWork::Prepared(Box::new(film(root.path(), index, 3)))),
            )
        })
        .collect();
    let preparation =
        futures::stream::iter(inputs).inspect(|_| discovered.set(discovered.get() + 1));
    let mapped = map_staged(
        preparation,
        |pending| {
            assert_eq!(discovered.get(), 3);
            assert_eq!(pending.len(), 9);
            futures::stream::iter(pending.into_iter().rev().map(|(id, cut)| {
                assert!(cut.wav.exists(), "cut survives until inference finishes");
                let result = if id == (1, 1) {
                    Err(anyhow::anyhow!("one clip failed"))
                } else {
                    Ok(matrix(cut.hash))
                };
                (id, result)
            }))
        },
        &Gate::default(),
    )
    .await;
    for (index, work) in mapped {
        let FilmWork::Prepared(film) = work.unwrap() else {
            panic!()
        };
        for (slot, clip) in film.clips.iter().enumerate() {
            if (index, slot) == (1, 1) {
                assert!(clip.is_none());
            } else {
                assert_eq!(
                    clip.as_ref().unwrap().heard_ipa,
                    vec![format!("a{}", index * 100 + slot)]
                );
            }
        }
        assert_eq!(finish_film(FilmWork::Prepared(film)).is_ok(), index != 1);
    }
}

#[tokio::test]
async fn empty_or_resolved_selection_never_infers() {
    let inputs = vec![(0, Ok(FilmWork::Current(FilmSummary::default())))];
    map_staged(
        futures::stream::iter(inputs),
        |_| {
            panic!("nothing to infer");
            #[allow(unreachable_code)]
            futures::stream::empty()
        },
        &Gate::default(),
    )
    .await;
}

#[test]
fn incomplete_work_is_not_written_and_malformed_json_is_rejected() {
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
    std::fs::write(&path, format!("{}\nmalformed\n{}\n", lines[0], lines[2])).unwrap();
    assert!(read_clips(&path).is_err());
    assert!(stored_provenance(&path).is_none());
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
    changed.cut.preferred_clear_ms += 1;
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
        prepare_film(&store, &movie, &dir, &gate, 1).await.unwrap();
        let rows = read_clips(&clips_path(&dir)).unwrap();
        assert_eq!(rows[0].passed, passed);
        assert_eq!(rows[0].producers.g2p.as_deref(), Some("older-renderer"));
        assert_eq!(rows[1].reject.as_deref(), Some("cut: permanent failure"));
    }
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
    assert!(prepare_film(&store, &movie, &dir, &gate, 1)
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
async fn report_repair_cannot_make_a_failed_redo_look_current() {
    let root = tempfile::tempdir().unwrap();
    let mut film = film(root.path(), 0, 1);
    let dir = film.dir.clone();
    let cues: Vec<_> = (0..30)
        .map(|i| crate::sync::Cue {
            start_ms: i * 5000 + 500,
            end_ms: i * 5000 + 1700,
            text: "Bonjour mon ami.".into(),
        })
        .collect();
    std::fs::write(dir.join("subtitle.srt"), crate::sync::write_cues(&cues)).unwrap();
    let mut transcript = String::new();
    for cue in &cues {
        for (i, text) in ["Bonjour", "mon", "ami"].into_iter().enumerate() {
            transcript.push_str(
                &serde_json::to_string(&Spoken {
                    text: text.into(),
                    at_ms: cue.start_ms + i as i64 * 400,
                    until_ms: cue.start_ms + i as i64 * 400 + 300,
                    kind: Kind::Word,
                    speaker: None,
                    logprob: None,
                })
                .unwrap(),
            );
            transcript.push('\n');
        }
    }
    std::fs::write(dir.join("transcript.jsonl"), transcript).unwrap();
    std::fs::write(dir.join("audio.opus"), b"must not be decoded").unwrap();
    std::fs::write(
        dir.join("audio.json"),
        serde_json::to_vec(&crate::sync::AudioStamp {
            filename: "fixture.mkv".into(),
            duration_ms: 200000,
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
    let gate = Gate::default();
    film.provenance = current_provenance(&dir, Language::French, "fra", &gate).unwrap();
    let original = film.provenance.clone();
    film.clips[0].as_mut().unwrap().passed = true;
    finish_film(FilmWork::Prepared(Box::new(film))).unwrap();
    let movie = Movie {
        imdb_id: "0".into(),
        title: "fixture".into(),
        year: None,
        path: dir.clone(),
        original_language: "French".into(),
        source: crate::library::Source::Missing,
    };
    let store = osmo::Store::open(root.path().join("cache"));
    assert_eq!(
        existing_work(&dir, &original).0,
        Work::Redo("verbatim measurement missing or stale")
    );
    // First run really regenerates a matching verbatim report, then fails
    // preparation at the absent speech profile. The second is an ordinary retry.
    for _ in 0..2 {
        let error = prepare_film(&store, &movie, &dir, &gate, 1)
            .await
            .err()
            .unwrap();
        assert!(format!("{error:#}").contains("speech profile"), "{error:#}");
        assert_eq!(
            crate::verbatim::stored(&dir).unwrap().measure.verdict,
            crate::verbatim::Verdict::Verbatim
        );
        assert!(!clips_path(&dir).exists());
        assert!(matches!(existing_work(&dir, &original).0, Work::Redo(_)));
    }
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
        prepare_film(&store, &movie, &dir, &Gate::default(), 1)
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
    prepare_film(&store, &movie, &dir, &Gate::default(), 1)
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
    prepare_film(&store, &movie, &dir, &Gate::default(), 1)
        .await
        .unwrap();
    assert!(read_clips(&clips_path(&dir)).unwrap()[0].passed);
}
