//! Exercise the public reexports through the same fp16 wire payload as cached clips.
//! The former local CTC unit tests are all covered by lexide's pronunciation tests.

use base64::Engine;
use phoneme_verify::{AlignedPhoneme, FrameMatrix, FrameMatrixPayload, PhoneRun, TargetScore};
use std::io::Write;

fn cached_matrix(rows: &[[f32; 4]]) -> FrameMatrix {
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    for row in rows {
        for p in row {
            encoder
                .write_all(&half::f16::from_f32(p.ln()).to_le_bytes())
                .unwrap();
        }
    }
    let payload = FrameMatrixPayload {
        shape: vec![rows.len(), 4],
        dtype: "float16".into(),
        encoding: "zlib+base64".into(),
        blank_id: 0,
        vocab: ["<pad>", "a", "b", "c"].map(String::from).to_vec(),
        data: base64::engine::general_purpose::STANDARD.encode(encoder.finish().unwrap()),
    };
    let cached = serde_json::to_vec(&payload).unwrap();
    FrameMatrix::decode(&serde_json::from_slice(&cached).unwrap()).unwrap()
}

#[test]
fn cached_split_mass_uses_shared_nonblank_first_decode() {
    let speech = [0.4, 0.24, 0.18, 0.18];
    let blank = [0.6, 0.16, 0.12, 0.12];
    let matrix = cached_matrix(&[speech, speech, blank, speech]);
    // The old joint-argmax implementation returned no phones here. Now blank
    // wins iff log P(blank) >= ln(0.5); otherwise maximize over phones only.
    assert_eq!(matrix.greedy_ids(), [1, 1]);
    assert_eq!(matrix.speech_fraction(0, 4), Some(0.75));
    let decoded: phoneme_verify::DecodedPath = matrix.decode_path().unwrap();
    assert_eq!(decoded.path, [1, 1, 0, 1]);
    assert_eq!(
        decoded.runs,
        [
            PhoneRun {
                id: 1,
                start_frame: 0,
                end_frame: 2
            },
            PhoneRun {
                id: 1,
                start_frame: 3,
                end_frame: 4
            },
        ]
    );
    let score: TargetScore = matrix.score_target(&["a".into(), "a".into()]);
    assert_eq!(score.ratio, Some(0.0));
    assert_eq!(score.free_len, 2);
    assert_eq!(score.target_len, 2);
    assert_eq!(
        serde_json::from_value::<TargetScore>(serde_json::to_value(&score).unwrap()).unwrap(),
        score
    );
    // Type identity ensures callers can use lexide directly without adapters.
    let shared: &lexide::pronunciation::FrameMatrix = &matrix;
    assert_eq!(shared.greedy_ids(), matrix.greedy_ids());
}

#[test]
fn alignment_end_stays_inclusive_while_phone_runs_are_exclusive() {
    let matrix = cached_matrix(&[[0.1, 0.7, 0.1, 0.1]; 2]);
    let spans: Vec<AlignedPhoneme> = matrix.force_align(&[matrix.id("a").unwrap()]).unwrap();
    assert_eq!(spans[0].start_frame, 0);
    assert_eq!(spans[0].end_frame, 1);
    assert_eq!(spans[0].frames, 2);
    // Existing clip timestamps add one to AlignedPhoneme.end_frame, not PhoneRun.
    assert_eq!(
        matrix.decode_path().unwrap().runs[0].end_frame,
        spans[0].end_frame + 1
    );
    assert!((spans[0].logp_mean - 0.7f64.ln()).abs() < 1e-3);
}
