# Archived clarity labels

Human audio-quality labels collected for film clips in August 2026.
The labeling UI and server have been retired from this repository; these files
are preserved unchanged for reference. The original live directory was
`/data/andrep/subtitle-corpus/clarity-labels/` on the NixOS box.

- `manifest.json` — the 300-clip labeling set. Hidden band composition
  (labelers don't see this): pass-random 90, pass-borderline-voice 70,
  whisper-reject 55, ratio-marginal 55, pad-reject 30.
- `labels-greg.json` — merged labels per labeler; keys are clip ids.
- `labels-greg.log.jsonl` — append-only journal of every POST (recovery).
- `corrections-greg.json` — analysis-time overrides; apply LAST, they win over
  the labels file (clients can re-post stale cached labels).
- `backups/` — point-in-time snapshots.

Label semantics: the `overall` axis (beginner/advanced/hard) is **audio-based
student suitability only** — linguistic difficulty must not lower it. Labels
from greg before the evening of 2026-08-31 may mix in word difficulty.
`easy`/`hard` values predate the beginner/advanced split; `noise` predates the
minor-noise/bothersome-noise split. `volume_db` records the playback gain the
labeler had set when labeling.

Audio files are not included. The original clips lived alongside the live
directory in `clips-orig/`.
