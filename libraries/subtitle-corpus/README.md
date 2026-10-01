# Subtitle corpus

`subtitle-corpus training-export` is how film clips reach lexide training.

```sh
subtitle-corpus training-export --dry-run
subtitle-corpus training-export --langs spa,por --replace-legacy
subtitle-corpus training-export --imdb tt8291806 --out-root /tmp/training-export-trial
```

The source defaults to `/data/andrep/subtitle-corpus` (`--out`); the destination
is `/data/coding/lexide/pronunciation/data/audio` (`--out-root`). Every `passed`
clip is cut from `audio.opus` at its scored bounds and padding into a 16 kHz mono
PCM WAV. There is no cap. Existing manifest filenames are skipped. Dry runs
report counts and estimated WAV bytes without making directories or cutting audio.
Repeated IMDb/start/end cuts share a filename: identical-text duplicates export
once using the first stored passing row; groups with differing sentence text
(including punctuation differences) are excluded entirely. The summary separately
counts duplicates and differing-text groups. Previously exported conflicts are
not removed. Real exports record duration from the WAV samples.

`--jobs N` controls concurrent audio cuts (default 12). Each language's manifest
rows are appended in selection order after its WAVs are persisted, with progress
reported every 2,000 cuts.

`--replace-legacy` removes only selected films' old `film_<imdb>_<5 digits>.wav`
manifest rows and their otherwise-unreferenced WAVs. It never edits phoneme or
VAD outputs: rerun lexide's full preprocessing with `--skip-narrowing` (the
merged-label aligner is not compatible yet), or rebuild labels then VAD, in
that order. Running VAD alone first would read stale phoneme rows referencing
deleted WAVs. Do not train with stale `--use-narrowed` outputs.
Speaker clusters preserve ElevenLabs' chunk-scoped `speaker_N@chunk` identity.

A film's course variety is derived from the reviewed table in
`src/library/varieties.rs`, then a named variety in the current audio-check
verdict, then its original-language default. Spanish and Portuguese always
share corpus directories/keys (`spa`, `por`); `variety` is an attribute, never
a storage directory. Training manifests carry the corresponding `espeak_voice`
only for these two languages. European Portuguese clips use audio-only gates
until the pronunciation model is validated for that variety.

Publish refreshes variety metadata in sidecars and language indexes. Variety
is not part of the media stamp, so unchanged cuts reuse both video renditions.
