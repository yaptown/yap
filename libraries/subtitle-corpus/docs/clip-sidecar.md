# Clip sidecar schema (format 3)

One JSON per served video clip, stored next to the mp4 (`<id>.json` beside
`<id>.mp4`), immutable. Cut generously (neighbor sentences as context when the
gap is ≤ ~2s, total ≤ ~15s); the sidecar — not the file boundary — defines what
the clip *is*. The app seeks to `critical.start_ms` by default and offers the
padding as opt-in context.

Principles:

- **Two time domains.** Everything the app consumes is clip-relative ms; the
  `source` block keeps film-absolute times + provenance digests so any future
  re-cut (different padding/resolution/codec) regenerates without re-running
  alignment or transcription.
- **Include everything cheap.** The whole `Clip` verdict block goes in
  verbatim, uses imagined or not — a few hundred bytes against a megabyte of
  video. Omitting is the only expensive mistake.
- **Both witnesses, independently timed.** Official subtitle cues with their
  own timings, and the full Scribe transcript with nested character spans.
  (Scribe is requested at character granularity; char spans live in the cached
  raw responses and are recovered by re-parse, not re-transcription — see
  `transcript.rs`.)
- **Context is display-only.** Neighbor cues never passed the gates:
  `context_verified: false` is categorical.
- Phoneme alignment reads the shared response cache using the mapper's stored
  WAV hash and exact `target_ipa` token vector; it never re-cuts audio or calls
  the model. Producer versions do not key production responses. Missing/broken
  expected alignment entries are counted in an aggregate export warning;
  audio-only, unscored, and no-alignable-path clips are not cache failures.
- Loudnorm is measured over the critical span, applied to the whole file, so a
  loud neighbor line can't duck the target sentence. Force a keyframe at or
  just before `critical.start_ms` at encode time (seeking is keyframe-accurate).
- **Id**: `imdb_id - sha256(NFC sentence text)[:8] - occurrence index`.
  The index counts occurrences of the sentence in the *official subtitle
  file*, in cue order — all occurrences, aligned or not — so it is fixed by
  `film.subtitle_digest` alone. Never index over passing clips: a gate tweak
  would renumber later occurrences and silently reuse ids for different audio.
  Gaps in the served set are fine. (Timing-based ids were rejected: repair /
  alignment changes shift `start_ms` across rebuilds.)

Caching: `clips.jsonl` format 12 distinguishes **inputs**, **cut settings**,
**gate thresholds**, and per-row **observed producers**. Inputs (subtitle and
transcript digests, segmentation, language, and the extracted-audio stamp's
filename/stream index/duration) or cut changes require remapping. Gate-only
changes reuse stored measurements, including a matching `transcript-check.json`
for `min_verbatim`, without G2P, model discovery, segmentation, or new cuts.
Film-level verbatim rejection preserves scored rows, so loosening the gate is
also cheap. Pre-gate failures remain failures. A missing audio stamp or
missing/stale verbatim measurement is not a current cheap hit.

A current header also requires `completion: "complete"`, its
`expected_candidates` count, and a valid verdict on every row. Inference failures
leave the film unfinished; cached successes survive retry. Old, malformed, or
truncated manifests are not read through compatibility fallbacks.

Model and G2P identities record what **actually produced each row**; producer
changes alone never invalidate it. Producer bugs require a deliberate refresh.
`clips --refresh-g2p` regenerates selected gated films' cached full G2P targets
(keyed by literal language selector, exact text, and clip WAV hash). Identical
labels retain the audio response key; changed token vectors infer afresh. This
flag does **not** refresh model-only outputs, and Korean stays audio-only. Before
refresh, the old rows are retained with `completion: "refresh_g2p"`; any subsequent
run resumes that interrupted refresh, even without the flag. `clip-models`
reports actual producing-model row counts and separately counts no-model rows
and unreadable/old-format/incomplete files, without network calls.

The one response artifact retains the selected item's untouched JSON and every
raw envelope field value, including unknown fields, number representations,
escaping, and whitespace inside those values. It never duplicates sibling
matrices. Envelope key order/inter-field whitespace are not retained. Consumers
decode a typed view on read. Requests always include the full frame matrix and
all heads; live responses are validated before the original raw values are
stored. `trained_against_g2p` is retained as reported, not used here as a generic
renderer-compatibility assertion.

An exported clip's renditions are reused only
when the sidecar's `media.stamp` (encode recipe, source-video identity —
filename, bytes, runtime, height, HDR, audio stream — and the cut: window
and critical span) equals the one computed now — else delete and re-render.
The stamp is deliberately not the clips provenance: a re-map under a new
segmenter or gate that lands on the same span costs a sidecar rewrite, not
a day of re-encoding. The sidecar itself is always regenerated and left
untouched when it comes out byte-identical; full-language runs sweep
orphaned ids. Upload markers store the three files' content hashes and
re-upload each file whose bytes changed. Better to recalculate than to
trust a cache whose inputs may have moved.

```jsonc
{
  "format": 3,
  "id": "tt0101700-3fa2c81d-0",        // imdb id + sentence hash + occurrence index (see above)
  "language": "fra",

  "film": {
    "imdb_id": "tt0101700",
    "subtitle_digest": "…",            // Provenance digests: pins exactly which
    "transcript_digest": "…"           //   subtitle + transcript this was built from
  },
  "source": {                          // FILM-ABSOLUTE milliseconds
    "sentence_start_ms": 4123500,      // Clip.start_ms / end_ms (unpadded, repaired)
    "sentence_end_ms": 4126200,
    "cut_start_ms": 4121800,           // what the mp4 actually contains, incl. context
    "cut_end_ms": 4129400,
    "pad_before_ms": 300, "pad_after_ms": 150,
    "repaired_before_ms": 0, "repaired_after_ms": 120
  },

  // ---- everything below is CLIP-RELATIVE milliseconds ----
  "critical": { "start_ms": 1400, "end_ms": 4400 },  // seek target; keeps the ~300ms lead-in

  "sentence": {
    "text": "Comme je casse tout, j'ai tout en double.",
    "course_sentence": true,             // passes should_include_sentence — in the course
    "speaker": "speaker_1@247",        // Clip.speaker (chunk-scoped id, film-local only)
    "words": [                         // ClipWord stamps, shifted clip-relative
      { "text": "Comme", "at_ms": 1700, "until_ms": 1950 }
    ]
  },

  "subtitles": [                       // ALL official cues overlapping the cut, timed
    { "text": "…", "at_ms": 0, "until_ms": 1500, "role": "context-before" },
    { "text": "Comme je casse tout…", "at_ms": 1500, "until_ms": 4300, "role": "sentence" },
    { "text": "…", "at_ms": 4600, "until_ms": 7200, "role": "context-after" }
  ],
  "context_verified": false,

  "transcript": {                      // ElevenLabs Scribe, full detail
    "words": [
      { "text": "Comme", "at_ms": 1700, "until_ms": 1950,
        "speaker": "speaker_1@247", "logprob": -0.02,
        "chars": [ { "c": "C", "at_ms": 1700, "until_ms": 1760 } ] }
    ]
  },

  "phonemes": {
    "target_ipa": ["k", "ɔ", "m"],     // exact cached G2P tokens, boundaries preserved
    "heard_ipa":  ["k", "ɔ", "m"],     // model's free reading
    "oov": [],
    "alignment": [                     // forced alignment from the cached frame matrix
      { "ph": "k", "at_ms": 1710, "until_ms": 1780, "logp": -0.11 }
    ]
  },

  "verification": {                    // the Clip verdict block, verbatim
    "passed": true, "reject": null, "measured": true, "audio_hash": 123456789,
    "transcript_wer": 0.0, "ratio": 1.9, "logp_target_per_phoneme": -0.4,
    "edge_logp_start": -0.3, "edge_logp_end": -0.5,
    "lead_speech": 0.02, "tail_speech": 0.0, "lead_rms": 0.1, "voiced": 0.61,
    "audio_event_overlap": false, "clear_before_ms": 900, "clear_after_ms": 640,
    "pad_before_ms": 300, "pad_after_ms": 150,
    "provenance": {
      "inputs": { "format": 12, "subtitle_digest": "…", "transcript_digest": "…",
                  "segmentation": "…", "language": "fra",
                  "audio": { "filename": "film.mkv", "stream_index": 2, "duration_ms": 7126875 } },
      "cut": { "preferred_clear_ms": 100, "min_clear_ms": 0, "speech_threshold": 0.7 },
      "gate": { "min_ratio": -2.0, "min_edge_logp": -4.0, "max_pad_speech": 0.25,
                "max_lead_rms": 1.0, "min_voiced": 0.25, "min_verbatim": 0.25 },
      "producers": { "model": { "model_id": "…", "model_revision": "full revision",
                                "decoder_version": "nonblank_v1", "deploy_marker": "…" },
                     "g2p": "actual cached renderer identity" }
    }
  },

  "media": {
    "stamp": {                         // the media stamp (see above) — rendition reuse key
      "recipe": "hi h264 crf19 medium …",
      "video": { "filename": "….mkv", "bytes": 31882123456, "duration_ms": 7126875,
                 "height": 1080, "hdr": false, "audio_stream": 2 },
      "cut": { "start_ms": 5231200, "end_ms": 5238800,
               "critical_start_ms": 1850, "critical_end_ms": 5600 }
    },
    "duration_ms": 7600,
    "width": 1920, "height": 1080, "aspect_ratio": 1.7777777777777777,
    "loudnorm": { "measured_i": -24.3, "measured_tp": -3.1, "gain_db": 6.3,
                  "measured_over": "critical" },
    "keyframe_at_critical": true
  }
}
```
