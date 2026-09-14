"""CPU contract tests for pronunciation batching and wire formats."""
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
    obj.masked_slots = [0]
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


def compact(request):
    import numpy as np
    request = dict(request)
    request["audio_f32_b64"] = base64.b64encode(
        np.asarray(request.pop("audio"), dtype="<f4").tobytes()
    ).decode()
    return request


def test_compact_audio_matches_legacy_sample_values_and_response(worker):
    legacy = clip(1600, return_frames=True, return_frame_matrix=True, language="tha")
    encoded = compact(legacy)
    torch.testing.assert_close(worker._prepare_audio(encoded), worker._prepare_audio(legacy),
                               rtol=0, atol=0)
    assert batch_method(worker, [encoded]) == batch_method(worker, [legacy])


@pytest.mark.parametrize("encoded", [None, [], "!", "", "AA==", "AAAAAAA=", "AACAfw=="])
def test_invalid_compact_audio_is_isolated(worker, encoded):
    results = batch_method(worker, [compact(clip(1600)), {"audio_f32_b64": encoded}])
    assert "phonemes" in results[0]
    assert results[1]["error"]["type"] == "ValueError"


def test_matrix_bytes_match_old_encoder_for_trimmed_tensor(worker):
    values = torch.arange(60, dtype=torch.float32).reshape(2, 10, 3) / 7
    values[1, 2, 0] = float("-inf")
    trimmed = values[1:2, :5]
    expected = trimmed[0].half().contiguous()
    matrix = worker._frame_matrix(trimmed)
    assert matrix["shape"] == [5, 3]
    assert base64.b64decode(matrix["data"]) == zlib.compress(bytes(expected.untyped_storage()), 6)


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
        assert len(result["frames"][0]["top_k"]) == min(request["top_k"], 2)
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
        response = client.post("/batch", json={"requests": [compact(clip(2000)), {}, compact(clip(1600))]})
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


@pytest.fixture
def headed_worker(worker):
    """Run the real factorized forward with deterministic CPU head logits."""
    worker.regularized = worker.mel_sidechannel = False
    worker.masked_slots = [0, 2, 4, 6]
    vocab = {"<pad>": 0, "a": 1, "<s>": 2, "b": 3, "</s>": 4, "c": 5, "<unk>": 6}
    worker.processor.tokenizer = SimpleNamespace(get_vocab=lambda: vocab)
    worker.processor.decode = lambda token: next(k for k, v in vocab.items() if v == token)

    def forward(nb, phones=None, stress=None, tone=None):
        t = len(nb)
        phones = torch.tensor(phones if phones is not None else [[.4, .3, .3]] * t)
        phone_logits = torch.full((1, t, 7), 100.)  # Specials would win without masking.
        phone_logits[..., [1, 3, 5]] = phones.log()
        worker.backbone = lambda values, **kwargs: SimpleNamespace(last_hidden_state=values.unsqueeze(-1))
        worker.nonblank_head = lambda h: torch.logit(torch.tensor(nb)).reshape(1, t, 1)
        worker.phoneme_head = lambda h: phone_logits
        worker.stress_head = lambda h: torch.nn.functional.one_hot(
            torch.tensor([stress if stress is not None else [0] * t]), 3).float()
        worker.language_heads = {"thai": lambda h: torch.nn.functional.one_hot(
            torch.tensor([tone if tone is not None else [0] * t]), 4).float()}
        worker.language_head_specs = {"thai": {"lang": "tha", "target": "tone"}}
        return worker._forward(torch.zeros(1, t), language="tha")

    return worker, forward


@pytest.mark.parametrize("nb,emits", [(.6, True), (.4, False), (.5, False)])
def test_nonblank_first_real_forward(headed_worker, nb, emits):
    worker, forward = headed_worker
    lp, stress, p_nb, aux = forward([nb])
    assert lp[0, 0].argmax().item() == worker.blank_id  # Original failure.
    torch.testing.assert_close(lp[0, 0, [0, 1, 3, 5]].exp(),
                               torch.tensor([1 - nb, nb * .4, nb * .3, nb * .3]))
    result = worker._decode_with_confidence(lp, stress, p_nb, aux, 100)
    assert [p["phoneme"] for p in result] == (["a"] if emits else [])
    if emits:
        assert result[0]["confidence"] == .24
        assert {x["phoneme"] for x in result[0]["top_k"]} == {"a", "b", "c"}
    frames = worker._frames_topk(lp, stress, p_nb, aux, 100)
    assert frames[0]["p_nonblank"] == nb
    assert {x["phoneme"] for x in frames[0]["top_k"]} == {"a", "b", "c"}
    assert sum(x["probability"] for x in frames[0]["top_k"]) == pytest.approx(nb)
    score = worker._score_target(lp, p_nb, ["a"])
    assert score["logp_target"] == pytest.approx(torch.tensor(nb * .4).log().item())
    assert score["free_len"] == int(emits)
    assert score["ratio"] == (0.0 if emits else None)


def test_collapse_onset_labels_and_free_reference(headed_worker):
    worker, forward = headed_worker
    lp, stress, nb, aux = forward([.6, .7, .5, .6, .8],
                                  stress=[1, 2, 0, 2, 1], tone=[1, 2, 0, 3, 2])
    result = worker._decode_with_confidence(lp, stress, nb, aux)
    assert [p["phoneme"] for p in result] == ["ˈa", "ˌa"]
    assert [p["tone"] for p in result] == [1, 3]
    score = worker._score_target(lp, nb, ["a", "a"])
    expected = -torch.nn.functional.ctc_loss(
        lp.transpose(0, 1), torch.tensor([[1, 1]]), torch.tensor([5]),
        torch.tensor([2]), blank=0, reduction="none")[0].item()
    assert score["free_len"] == len(result) == 2
    assert score["logp_free"] == pytest.approx(expected)
    assert score["logp_target"] == pytest.approx(expected)
    assert score["ratio"] == 0
    other = worker._score_target(lp, nb, ["b"])
    assert other["logp_free"] == score["logp_free"]
    assert other["ratio"] == pytest.approx(other["logp_target"] - expected)

    # Exercise the actual response builder to catch missing p_nonblank plumbing.
    worker._forward_requests = lambda items: (lp, stress, nb, [aux], [5])
    _, response = next(worker._process_microbatch(
        [(0, {"target_phonemes": ["a", "a"]}, torch.zeros(1680))], 1, 1))
    assert response["phonemes"] == result
    assert response["target_score"] == score


def test_confidence_is_product_of_means(headed_worker):
    worker, forward = headed_worker
    lp, stress, nb, aux = forward([.6, .9], phones=[[.4, .3, .3], [.8, .1, .1]])
    result = worker._decode_with_confidence(lp, stress, nb, aux, 100)
    assert len(result) == 1
    assert result[0]["phoneme"] == "a"
    assert result[0]["confidence"] == .45  # (.4 + .8)/2 * (.6 + .9)/2, not .48.
    assert {x["phoneme"]: x["probability"] for x in result[0]["top_k"]} == {
        "a": .45, "b": .15, "c": .15}


def test_tied_topk_does_not_change_winning_id(headed_worker):
    worker, forward = headed_worker
    lp, stress, nb, aux = forward([.6], phones=[[1/3, 1/3, 1/3]])
    assert worker._decode_with_confidence(lp, stress, nb, aux, 100)[0]["phoneme"] == "a"


def test_conditional_softmax_survives_zero_sigmoid_and_excludes_slots(headed_worker):
    worker, forward = headed_worker
    lp, stress, nb, aux = forward([.6])
    # Finite log-sigmoid at very negative logits, even though sigmoid underflows.
    lp[..., [1, 3, 5]] -= 1000
    lp[..., worker.masked_slots] = 100  # Explicit exclusion, not just -inf from forward.
    nb.zero_()
    predicted, conditional, ids = worker._phone_frames(lp, nb)
    assert predicted.tolist() == [worker.blank_id]
    assert ids.tolist() == [1, 3, 5]
    torch.testing.assert_close(conditional, torch.tensor([[.4, .3, .3]]), atol=2e-5, rtol=0)
    frames = worker._frames_topk(lp, stress, nb, aux, 100)
    assert all(x["probability"] == 0 for x in frames[0]["top_k"])
    nb.fill_(.6)
    assert worker._decode_with_confidence(lp, stress, nb, aux)[0]["phoneme"] == "a"
