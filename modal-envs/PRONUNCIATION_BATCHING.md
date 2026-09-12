# Explicit pronunciation batch API

`Wav2Vec2Phoneme.predict_batch` accepts one HTTP POST containing **1–64 clips**.
`Wav2Vec2Phoneme.transcribe_batch` provides the same operation over Modal RPC.
Production URL: https://anchpop--wav2vec2-phoneme-wav2vec2phoneme-predict-batch.modal.run

The service resamples and normalizes each clip separately, sorts by length,
runs similar-length microbatches, trims padding from the outputs, and restores
the caller's order. Each item supports the existing single-request fields:
`audio_f32_b64`, `sample_rate`, `language`, `top_k`, `return_frames`,
`return_frame_matrix`, and `target_phonemes`.

```python
import base64
import numpy as np
import requests

# Pack mono samples as little-endian IEEE-754 float32 (no WAV header).
def encode_audio(samples):
    return base64.b64encode(np.asarray(samples, dtype="<f4").tobytes()).decode()
response = requests.post(batch_url, json={
    "requests": [
        {"audio_f32_b64": encode_audio(audio_a), "sample_rate": 16000, "language": "eng"},
        {"audio_f32_b64": encode_audio(audio_b), "sample_rate": 16000, "language": "tha",
         "return_frame_matrix": True},
    ],
}, timeout=180)
response.raise_for_status()
results = response.json()["results"]
```

Both single and batch HTTP endpoints accept this compact format. Legacy
`audio` float arrays are also accepted during caller rollout. Compact input
takes precedence if both fields are supplied; invalid compact input is rejected.

The response is `{"results": [...], "deploy_marker": "..."}`. Each successful
item has the existing pronunciation response fields. An invalid item receives
`{"error": {"type": "ValueError", "message": "..."}}` at its original index;
healthy items still run. An invalid batch envelope/count receives HTTP 422;
a model load failure receives HTTP 503. Check each result for `error` even
when HTTP returns 200.

This batches only the clips explicitly submitted together. It adds no
cross-request collection window. The existing single-clip endpoint remains
available. A request completes after its microbatches finish; outputs do not
stream. The 64-item limit is a request size, not 64 concurrent GPU forwards.

## Limits and deployment configuration

| Environment variable | Default | Meaning |
| --- | --- | --- |
| `WAV2VEC2_BATCH_SIZE` | `8` | Maximum clips per GPU forward |
| `WAV2VEC2_MAX_LENGTH_RATIO` | `1.25` | Longest/shortest duration within a microbatch |
| `WAV2VEC2_MAX_PADDED_SECONDS` | `120` | Longest duration × clip count per microbatch |
| `WAV2VEC2_GPU` | `L40S` | GPU choice |
| `WAV2VEC2_MAX_CONTAINERS` | Modal default | Optional container cap |
| `WAV2VEC2_BATCH_DIAGNOSTICS` | `0` | Add batching/sample-count diagnostics to results |

An individual clip exceeding the padded budget runs alone. GPU OOM splits a
microbatch recursively; failure of a singleton is reported for that clip.
Group-normalized checkpoints use singleton forwards because time padding
changes their normalization. No audio is truncated to satisfy a budget.
Decoded label strings are cached, and GPU outputs move to CPU in bulk before
frame decoding to remove repeated GPU synchronization.

Batch shape can change FP16 probabilities and predictions even with correct
masking. The 64-clip production-path test preserved phonemes in its sample;
this is not a guarantee of singleton-equivalent output on every clip.

Run local tests with `python -m pytest modal-envs/tests/test_pronunciation_batching.py -q`.
