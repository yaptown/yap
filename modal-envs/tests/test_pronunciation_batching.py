"""CPU contract tests; real GPU/HTTP validation lives in the benchmark script."""
import base64
from pathlib import Path
import sys
from types import SimpleNamespace
import zlib

import pytest
import torch
from fastapi import FastAPI
from fastapi.testclient import TestClient
from transformers import Wav2Vec2FeatureExtractor

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import wav2vec2_phoneme as service
from pronunciation_batching import length_batches

Service = service.Wav2Vec2Phoneme._get_user_cls()
batch_method = Service.transcribe_batch._get_raw_f()
batch_endpoint = Service.predict_batch._get_raw_f()


@pytest.fixture
def worker():
    obj = Service()
    obj.load_error = None
    obj._pool_number = 0
    obj._label_cache = {}
    obj.blank_id = 0
    obj.backbone = SimpleNamespace(
        config=SimpleNamespace(feat_extract_norm="layer"),
        _get_feat_extract_output_lengths=lambda n: (n - 400) // 320 + 1,
    )
    extractor = Wav2Vec2FeatureExtractor()
    obj.processor = extractor
    obj.processor.decode = lambda token: {1: "a", 2: "b"}[token]
    obj.processor.tokenizer = SimpleNamespace(get_vocab=lambda: {"<pad>": 0, "a": 1, "b": 2})

    def forward(items):
        lengths = [obj.backbone._get_feat_extract_output_lengths(x.numel()) for _, _, x in items]
        b, t = len(items), max(lengths)
        lp = torch.tensor([.1, .8, .1]).log().expand(b, t, 3).clone()
        stress = torch.tensor([1., 0., 0.]).expand(b, t, 3)
        aux = [{"tone": torch.ones(t, dtype=torch.long)} if req.get("language") == "tha"
               else {} for _, req, _ in items]
        return lp, stress, torch.full((b, t), .9), aux, lengths

    obj._forward_requests = forward
    return obj


def clip(n, **kwargs):
    return {"audio": [0.1, -0.2] * (n // 2), **kwargs}


def test_planner_reduces_padding_for_64_unsorted_clips():
    lengths = [16000 * (1 + (i * 17) % 12) for i in range(64)]
    batches = list(length_batches(lengths))
    assert sorted(i for batch in batches for i in batch) == list(range(64))
    for batch in batches:
        values = [lengths[i] for i in batch]
        assert len(batch) <= 8
        assert max(values) <= min(values) * 1.25
        assert max(values) * len(batch) <= 1_920_000
    padded = sum(max(lengths[i] for i in batch) * len(batch) for batch in batches)
    assert padded < max(lengths) * len(lengths) * .65
    assert list(length_batches([4_000_000, 16000])) == [[1], [0]]


@pytest.mark.parametrize("lengths,kwargs", [([0], {}), ([1], {"max_batch_size": 0}),
                                             ([1], {"max_length_ratio": .9})])
def test_invalid_planner_limits(lengths, kwargs):
    with pytest.raises(ValueError):
        list(length_batches(lengths, **kwargs))


def test_preprocessing_is_per_clip_normalization(worker):
    request = clip(1600)
    samples = worker._prepare_audio(request)
    assert abs(samples.mean().item()) < 1e-6
    assert abs(samples.var(unbiased=False).item() - 1) < 1e-4
    torch.testing.assert_close(samples, worker.processor(request["audio"], sampling_rate=16000,
                                                       return_tensors="pt").input_values[0])


@pytest.mark.parametrize("payload", [{}, {"audio": []}, {"audio": [[1., 2.]]},
                                     {"audio": [float("nan")]}, clip(1600, sample_rate=0), clip(20)])
def test_invalid_audio(worker, payload):
    with pytest.raises(ValueError):
        worker._prepare_audio(payload)


def test_order_trimming_language_and_options(worker):
    requests = [clip(n, return_frames=True, return_frame_matrix=True, top_k=k, language=lang)
                for n, k, lang in [(2000, 1, "tha"), (1600, 2, "eng"), (1800, 3, "tha")]]
    results = batch_method(worker, requests)
    for request, result in zip(requests, results):
        n = worker.backbone._get_feat_extract_output_lengths(len(request["audio"]))
        assert len(result["frames"]) == n
        assert result["frame_matrix"]["shape"] == [n, 3]
        assert len(zlib.decompress(base64.b64decode(result["frame_matrix"]["data"]))) == n * 3 * 2
        assert len(result["frames"][0]["top_k"]) == request["top_k"]
        assert ("tone" in result["frames"][0]) == (request["language"] == "tha")
    assert set(batch_method(worker, [clip(1600)])[0]) == {"phonemes"}


def test_bad_item_does_not_drop_healthy_items(worker):
    results = batch_method(worker, [clip(1600), {"audio": []}, clip(2000), clip(1600, top_k="bad")])
    assert ["error" in x for x in results] == [False, True, False, True]
    assert results[1]["error"]["type"] == "ValueError"


def test_oom_splits_and_singleton_failure_is_isolated(worker):
    forward = worker._forward_requests
    calls = []

    def limited(items):
        calls.append(len(items))
        if len(items) > 2 or any(req.get("fail") for _, req, _ in items):
            raise torch.cuda.OutOfMemoryError("simulated capacity")
        return forward(items)

    worker._forward_requests = limited
    results = batch_method(worker, [clip(1600, fail=i == 3) for i in range(8)])
    assert calls[0] == 8 and 1 in calls
    assert [i for i, x in enumerate(results) if "error" in x] == [3]


def test_group_norm_keeps_singletons(worker):
    worker.backbone.config.feat_extract_norm = "group"
    forward = worker._forward_requests
    calls = []
    worker._forward_requests = lambda items: (calls.append(len(items)), forward(items))[1]
    batch_method(worker, [clip(1600) for _ in range(5)])
    assert calls == [1] * 5


def test_http_envelope_and_limit(worker):
    worker.transcribe_batch = SimpleNamespace(local=lambda requests: batch_method(worker, requests))
    app = FastAPI()

    @app.post("/batch")
    def endpoint(request: dict):
        return batch_endpoint(worker, request)

    with TestClient(app) as client:
        response = client.post("/batch", json={"requests": [clip(2000), {}, clip(1600)]})
        assert response.status_code == 200
        assert ["error" in item for item in response.json()["results"]] == [False, True, False]
        assert response.json()["deploy_marker"] == service.DEPLOY_MARKER
        assert client.post("/batch", json={"requests": [clip(1600)] * 64}).status_code == 200
        for payload in [{}, {"requests": []}, {"requests": [{}] * 65}, {"requests": "bad"}, []]:
            assert client.post("/batch", json=payload).status_code == 422
        worker.load_error = "failed to load test model"
        assert client.post("/batch", json={"requests": [{}]}).status_code == 503


def test_mixed_languages_select_distinct_heads_once(worker):
    class Head(torch.nn.Module):
        def __init__(self, label):
            super().__init__()
            self.label, self.calls = label, 0

        def forward(self, h):
            self.calls += 1
            result = torch.zeros(*h.shape[:2], 4)
            result[..., self.label] = 1
            return result

    worker.regularized = worker.mel_sidechannel = False
    worker.backbone = lambda values, **kwargs: SimpleNamespace(last_hidden_state=values.unsqueeze(-1))
    worker.nonblank_head = torch.nn.Linear(1, 1)
    worker.phoneme_head = torch.nn.Linear(1, 3)
    worker.stress_head = torch.nn.Linear(1, 3)
    worker.masked_slots = [0]
    worker.language_heads = {"thai": Head(1), "mandarin": Head(2), "japanese": Head(3)}
    worker.language_head_specs = {
        "thai": {"lang": "tha", "target": "tone"},
        "mandarin": {"lang": "zho-hans", "target": "tone"},
        "japanese": {"lang": "jpn", "target": "pitch_accent"},
    }
    _, _, _, aux = worker._forward(torch.zeros(5, 6), language=["tha", "zho-hans", "jpn", "tha", None])
    assert [dict((key, value.unique().item()) for key, value in item.items()) for item in aux] == [
        {"tone": 1}, {"tone": 2}, {"pitch_accent": 3}, {"tone": 1}, {}]
    assert all(head.calls == 1 for head in worker.language_heads.values())
