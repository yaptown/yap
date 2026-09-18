//! Verify human-recorded audio against ground-truth IPA using a wav2vec2
//! phoneme model hosted on Modal.
//!
//! For each clip we:
//! 1. Hash the original WAV bytes and check the shared response cache. Default
//!    keys depend on content, not the producer; model-varying evaluations opt
//!    into model+audio keys, and the corpus supplies WAV-hash+label keys.
//! 2. On cache miss, decode WAV → f32 mono 16kHz via ffmpeg and send it to
//!    the Modal batch endpoint (lexide `pronunciation/modal/PRONUNCIATION_BATCHING.md`),
//!    pooled with whatever other clips are in flight, then persist the
//!    response.
//! 3. Strip suprasegmental markers from the model's predicted tokens and
//!    diff against the ground-truth IPA assembled from
//!    `word_to_pronunciation`. Ground truth has no suprasegmentals yet, so
//!    we normalize defensively on both sides.
//! 4. Report a [`ClipVerification`] with the verdict; caller decides whether
//!    to drop the clip and/or log it to the per-language failures jsonl.
//!
//! The batch endpoint is configured via `WAV2VEC2_BATCH_ENDPOINT_URL` with a
//! default to the prod URL.

pub mod wav2vec2;

use anyhow::{Context, Result};
use base64::Engine;
pub use g2p::Phoneme;
use language_utils::{Language, PhonemeLabelSource};
use lexide::pronunciation::remote::decode_audio_bytes as decode_wav_to_f32;
#[cfg(test)]
use lexide::pronunciation::{DECODER_VERSION, PredictRequest};
use lexide::pronunciation::{
    PredictResponse as ModalResponse, cache_version, remote::PhonemizerClient,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(test)]
use std::io::Write;
use std::path::Path;
use std::sync::LazyLock;
use xxhash_rust::xxh3::xxh3_64;

pub use lexide::pronunciation::{
    AlignedPhoneme, DecodedPath, FrameMatrix, FrameMatrixPayload, LegacyFrameMatrixPayload,
    ModelIdentity, PhoneRun, TargetScore, decode_path, is_phone_token,
};

fn expected_deploy_marker() -> Option<String> {
    std::env::var("WAV2VEC2_EXPECTED_DEPLOY_MARKER")
        .ok()
        .filter(|s| !s.is_empty())
}

/// Identity of the target renderer. Persist alongside model identity when caching scores.
pub fn model_target_identity() -> String {
    g2p::identity()
}

/// Pronunciation target supplied by g2p. `None` for languages without
/// validated model labels (see `Language::g2p_lang`).
pub fn model_target(text: &str, language: Language) -> Option<Result<g2p::Phonemized, g2p::Error>> {
    let lang = language.g2p_lang()?;
    Some(g2p::phonemize_lang(lang, text))
}

/// One lossless per-clip artifact: untouched selected item and every raw batch
/// envelope value, never sibling matrices. Typed views are derived on read.
pub use lexide::pronunciation::RawPrediction;
pub use lexide::pronunciation::remote::{AudioClip, AudioInput, audio_cache_key};

use lexide::pronunciation::remote::response_identity;

/// Producer recorded by the returned matrix, never the context's intention.
pub fn frame_identity(frames: &FrameMatrix) -> Option<ModelIdentity> {
    let p = frames.producer.as_ref()?;
    Some(ModelIdentity {
        model_id: p.model_id.clone(),
        model_revision: p.model_revision.clone(),
        decoder_version: Some(p.decoder_version.clone()),
        deploy_marker: Some(p.deploy_marker.clone()),
    })
}

pub use lexide::pronunciation::AlignmentOp;

/// Outcome of verifying one clip. `failure_reason` is `None` iff the clip
/// passed and should be kept.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipVerification {
    /// Producer observed in the response; absent when rejected before inference
    /// or when a historical response did not report its identity.
    #[serde(default)]
    pub cache_version: Option<String>,
    #[serde(default)]
    pub model: Option<ModelIdentity>,
    pub actor: String,
    pub text: String,
    pub wav_path: String,
    /// Raw phoneme tokens straight from the model.
    pub predicted_raw: Vec<String>,
    /// Phonemes after stripping suprasegmental markers.
    pub predicted_normalized: Vec<Phoneme>,
    /// Ground-truth phonemes assembled from `word_to_pronunciation`. `None`
    /// when some word in `text` isn't in our pronunciation map. When a word
    /// has alternate accepted pronunciations, this is the variant that
    /// matched the model's prediction *most closely* — we pass if any
    /// variant is within threshold, and report the closest one for the
    /// alignment.
    pub expected: Option<Vec<Phoneme>>,
    /// Number of alternate (word, variant) combinations considered for
    /// `expected`. 1 when no variants exist; >1 when at least one word in
    /// the phrase has accepted alternates.
    #[serde(default)]
    pub variants_considered: usize,
    pub edit_distance: Option<usize>,
    /// `edit_distance / max(len(predicted), len(expected))`, `None` when
    /// `expected` is None.
    pub edit_distance_pct: Option<f64>,
    /// Step-by-step alignment between predicted and expected. `None` when
    /// `expected` is None.
    pub alignment: Option<Vec<AlignmentOp>>,
    /// `None` if the clip passed the threshold.
    pub failure_reason: Option<String>,
}

impl ClipVerification {
    pub fn passed(&self) -> bool {
        self.failure_reason.is_none()
    }
}

#[derive(Clone)]
pub struct VerifyContext<'a> {
    pub http: &'a reqwest::Client,
    /// Shared response store. Keys default to original WAV content, independent
    /// of the producing model or G2P version.
    store: osmo::Store,
    /// Optional extra cache identity. Lexide always hashes the audio itself.
    cache_context: Option<String>,
    client: PhonemizerClient,
    /// word (lowercase) → accepted IPA pronunciations (main + alternates).
    /// The verifier passes a clip if the model's prediction is within
    /// threshold of *any* of these variants — alternates exist because
    /// wikipron lists multiple valid pronunciations for many French words
    /// (e.g. `mes` is both /me/ and /mɛ/) and our LLM picks one
    /// arbitrarily for "main"; we shouldn't penalize audio that matches a
    /// documented alternate.
    pub word_to_pronunciation: &'a HashMap<String, language_utils::Pronunciations>,
    /// Reject clips whose `edit_distance_pct` exceeds this fraction. 0.3
    /// (30%) is a starting threshold; tune via the env var
    /// `AUDIO_VERIFY_THRESHOLD`.
    pub mismatch_threshold: f64,
    /// Target language — drives per-language phoneme canonicalization (e.g.
    /// collapsing r↔ʁ for French).
    pub target_language: Language,
}

impl<'a> VerifyContext<'a> {
    /// Add expected labels or another discriminator to lexide's audio identity.
    pub fn with_cache_context(mut self, context: impl Into<String>) -> Self {
        self.cache_context = Some(context.into());
        self
    }

    #[cfg(test)]
    fn response_key(&self, hash: u64) -> String {
        lexide::pronunciation::remote::audio_cache_key(hash, self.cache_context.as_deref())
    }

    /// Model-varying evaluations are deliberately not production content keys.
    /// A deployment marker validates live responses, not checkpoint content.
    pub fn key_by_model(&mut self, identity: &ModelIdentity) {
        self.client = self.client.clone().with_expected_identity(identity.clone());
        let model = serde_json::to_string(&(
            &identity.model_id,
            &identity.model_revision,
            &identity.decoder_version,
        ))
        .expect("model identity is serializable");
        self.cache_context = Some(format!("model:{model}"));
    }

    /// Production constructor: discover the live identity lazily on a cache miss.
    /// Cache hits retain their own producer and do not trigger freshness checks.
    pub fn new(
        http: &'a reqwest::Client,
        store: osmo::Store,
        word_to_pronunciation: &'a HashMap<String, language_utils::Pronunciations>,
        target_language: Language,
    ) -> Result<Self> {
        let threshold = std::env::var("AUDIO_VERIFY_THRESHOLD")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.3);
        let expected_deploy_marker = expected_deploy_marker();
        let mut ctx = Self::with_overrides(
            http,
            store,
            word_to_pronunciation,
            target_language,
            threshold,
            expected_deploy_marker,
        )?;
        ctx.client = ctx.client.with_identity_check();
        Ok(ctx)
    }

    /// Explicit, no-probe constructor for local/cache-only callers and tests.
    /// Evaluations also call `key_by_model` with their full resolved identity.
    pub fn with_overrides(
        http: &'a reqwest::Client,
        store: osmo::Store,
        word_to_pronunciation: &'a HashMap<String, language_utils::Pronunciations>,
        target_language: Language,
        mismatch_threshold: f64,
        expected_deploy_marker: Option<String>,
    ) -> Result<Self> {
        // Fail closed on the label source. The model is trained on labels from
        // exactly one G2P engine per language; scoring against a different
        // one silently compares incompatible phoneme inventories, and every
        // downstream number still *looks* fine. Hindi is the worked example:
        // scored against espeak `hi` it measured as our worst language by a
        // wide margin, because its real labels come from `schwa-stress-hin`,
        // which emits aspiration bound to its consonant rather than as the
        // standalone `ʰ` espeak produces.
        //
        // Refusing here rather than in the caller matters, because the
        // fallback is not "no targets" — `ground_truth_phoneme_variants`
        // would quietly fall back to wikipron, which is a *third* inventory.
        match target_language.phoneme_label_source() {
            PhonemeLabelSource::Espeak(_)
            | PhonemeLabelSource::Hindi
            | PhonemeLabelSource::Mandarin
            | PhonemeLabelSource::Japanese
            | PhonemeLabelSource::Korean
            | PhonemeLabelSource::Thai => {}
            PhonemeLabelSource::Unvalidated => anyhow::bail!(
                "{:?} has no validated phoneme label source — refusing to \
                 verify audio against an unchecked reference. See lexide's \
                 PHONEME_BACKENDS.md.",
                target_language
            ),
        }
        let mut client = BATCH_CLIENT
            .as_ref()
            .map_err(|error| anyhow::anyhow!("{error:#}"))?
            .clone()
            .with_cache(store.clone());
        if let Some(marker) = expected_deploy_marker {
            client = client.with_expected_deploy_marker(marker);
        }
        if cache_only() {
            client = client.with_cached_only();
        }
        Ok(Self {
            http,
            store,
            cache_context: None,
            client,
            word_to_pronunciation,
            mismatch_threshold,
            target_language,
        })
    }
}

/// Compute the accepted phoneme sequence(s) for a clip.
///
/// If `override_transcription` is `Some`, uses that single whitespace-
/// separated IPA string as the sole ground truth — encodes a speaker's
/// connected-speech realization that may differ from citation form (e.g.
/// French `de` produced as /dø/). Otherwise derives from wikipron's
/// per-word variants (cross-product, capped at MAX_VARIANT_COMBINATIONS)
/// plus the espeak phrase-level rendering.
///
/// Returns `None` only when no source produced any candidates — e.g. a
/// word missing from wikipron AND a language with no espeak support.
pub fn expected_phoneme_variants(
    ctx: &VerifyContext<'_>,
    text: &str,
    override_transcription: Option<&str>,
) -> Option<Vec<g2p::Phonemized>> {
    if let Some(s) = override_transcription {
        let target = import_ipa(s)?;
        return (!target.phonemes.is_empty()).then_some(vec![target]);
    }
    ground_truth_phoneme_variants(text, ctx.word_to_pronunciation, ctx.target_language)
}

/// Verify a clip against an explicit accepted-phoneme-sequence set.
///
/// The caller is responsible for computing `expected` (see
/// [`expected_phoneme_variants`]). Splitting derivation from verification
/// keeps this function single-purpose: feed it audio + accepted
/// sequences, get back a verdict.
pub async fn verify_clip(
    ctx: &VerifyContext<'_>,
    actor: &str,
    text: &str,
    wav_path: &Path,
    expected: Option<Vec<g2p::Phonemized>>,
) -> Result<ClipVerification> {
    let wav_bytes = std::fs::read(wav_path)
        .with_context(|| format!("Failed to read wav {}", wav_path.display()))?;
    verify_clip_bytes(
        ctx,
        actor,
        text,
        &wav_path.display().to_string(),
        &wav_bytes,
        expected,
    )
    .await
}

/// Bytes-based variant of [`verify_clip`] for callers that already have the
/// audio in memory (e.g. Google TTS output, where we don't need a file at
/// all). `source_label` is recorded in the result purely for diagnostics.
pub async fn verify_clip_bytes(
    ctx: &VerifyContext<'_>,
    actor: &str,
    text: &str,
    source_label: &str,
    audio_bytes: &[u8],
    expected: Option<Vec<g2p::Phonemized>>,
) -> Result<ClipVerification> {
    // Early defect check: if the audio is too quiet or truncated, the
    // wav2vec2 prediction is unreliable (and on near-silent input the
    // model may still emit *something*, which gives misleading "matches
    // expected" results). Skip verification entirely and surface the
    // defect as the failure reason. Mirrors the same check applied to
    // Google TTS output in `verify_with_google_tts`.
    if let Ok(samples) = decode_wav_to_f32(audio_bytes)
        && let Some(defect) = audio_codec::samples_defect(&samples, MODAL_SAMPLE_RATE)
    {
        return Ok(ClipVerification {
            cache_version: None,
            model: None,
            actor: actor.to_string(),
            text: text.to_string(),
            wav_path: source_label.to_string(),
            predicted_raw: Vec::new(),
            predicted_normalized: Vec::new(),
            expected: None,
            variants_considered: 0,
            edit_distance: None,
            edit_distance_pct: None,
            alignment: None,
            failure_reason: Some(format!("audio defect: {defect}")),
        });
    }

    let (response, _) = prediction_response(ctx, audio_bytes)
        .await
        .with_context(|| format!("Failed to predict phonemes for {source_label}"))?;
    let model = response_identity(&response);
    let raw_phonemes = response
        .phonemes
        .iter()
        .map(|p| p.phoneme.clone())
        .collect::<Vec<_>>();
    let score = response.score(
        expected.as_deref().unwrap_or_default(),
        scoring_language(ctx.target_language),
    )?;
    let (
        predicted_normalized,
        expected,
        variants_considered,
        edit_distance,
        edit_distance_pct,
        alignment,
        failure_reason,
    ) = if let Some(score) = score {
        let reason = score.failure_reason(ctx.mismatch_threshold);
        (
            score.predicted,
            Some(score.expected),
            score.variants_considered,
            Some(score.edit_distance),
            Some(score.edit_distance_ratio),
            Some(score.alignment),
            reason,
        )
    } else {
        let predicted = response
            .phonemes()?
            .into_iter()
            .flat_map(|p| normalize_phonemes(p, ctx.target_language))
            .collect();
        let reason = "no ground truth available (word missing from wikipron and espeak unsupported for this language)";
        (predicted, None, 0, None, None, None, Some(reason.into()))
    };

    Ok(ClipVerification {
        cache_version: model
            .as_ref()
            .filter(|model| model.decoder_version.is_some())
            .map(cache_version),
        model,
        actor: actor.to_string(),
        text: text.to_string(),
        wav_path: source_label.to_string(),
        predicted_raw: raw_phonemes,
        predicted_normalized,
        expected,
        variants_considered,
        edit_distance,
        edit_distance_pct,
        alignment,
        failure_reason,
    })
}

/// Sample rate we always send to Modal — matches wav2vec2's training rate.
const MODAL_SAMPLE_RATE: u32 = 16_000;

/// How many alternatives we ask Modal for at each position. Larger = more
/// chance of catching the correct phoneme in the top-k for failure
/// analysis; trades off cache size linearly. 10 is comfortable for French.
#[cfg(test)]
const MODAL_TOP_K: usize = 10;

/// Individual callers share lexide's coalescing queue and request limits.
static BATCH_CLIENT: LazyLock<Result<PhonemizerClient>> = LazyLock::new(|| {
    wav2vec2::batch_client(
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()?,
    )
});

/// Cache-only lookup by the caller's complete key, without cutting audio.
pub async fn cached_frame_matrix(store: &osmo::Store, key: &str) -> Option<Result<FrameMatrix>> {
    let client = BATCH_CLIENT
        .as_ref()
        .ok()?
        .clone()
        .with_cache(store.clone())
        .with_cached_only();
    client
        .cached(key)
        .await
        .map(|raw| raw.and_then(|raw| raw.frames()))
}

async fn prediction_response(
    ctx: &VerifyContext<'_>,
    wav: &[u8],
) -> Result<(ModalResponse, FrameMatrix)> {
    let raw = ctx
        .client
        .predict_audio(
            AudioInput::Bytes(wav.to_vec()),
            ctx.cache_context.as_deref(),
        )
        .await?;
    Ok((raw.decode()?, raw.frames()?))
}

#[cfg(test)]
fn check_response_identity(ctx: &VerifyContext<'_>, response: &ModalResponse) -> Result<()> {
    ctx.client.validate_response(response)
}

#[cfg(test)]
async fn cached_response(store: &osmo::Store, key: &str) -> Option<Result<ModalResponse>> {
    let client = BATCH_CLIENT
        .as_ref()
        .ok()?
        .clone()
        .with_cache(store.clone());
    client
        .cached(key)
        .await
        .map(|raw| raw.and_then(|raw| raw.decode()))
}

#[cfg(test)]
async fn cache_response(
    ctx: &VerifyContext<'_>,
    hash: u64,
    raw: RawPrediction,
) -> Result<(ModalResponse, FrameMatrix)> {
    let raw = ctx
        .client
        .cache_response(Some(&ctx.response_key(hash)), raw)
        .await?;
    Ok((raw.decode()?, raw.frames()?))
}

/// A decoded, padded request paired with the original WAV's cache identity.
#[cfg(test)]
struct PreparedFrameRequest {
    hash: u64,
    payload: PredictRequest,
}

/// Prepare audio without inference. Batch callers count only successful
/// preparations toward the endpoint's request limit.
#[cfg(test)]
async fn prepare_frame_request(wav: Vec<u8>) -> Result<PreparedFrameRequest> {
    let hash = xxh3_64(&wav);
    let samples = tokio::task::spawn_blocking(move || decode_wav_to_f32(&wav))
        .await
        .context("audio decoding task failed")?
        .context("decoding audio for batch")?;
    Ok(PreparedFrameRequest {
        hash,
        payload: wav2vec2::Clip {
            samples: &samples,
            sample_rate: MODAL_SAMPLE_RATE,
            top_k: MODAL_TOP_K,
        }
        .into_request(),
    })
}

#[cfg(test)]
async fn infer_frame_batch_at(
    items: Vec<(&VerifyContext<'_>, PreparedFrameRequest)>,
    client: &Result<PhonemizerClient>,
    _activity: Option<&wav2vec2::RequestActivity>,
) -> Vec<Result<FrameMatrix>> {
    let (destinations, requests): (Vec<_>, Vec<_>) = items
        .into_iter()
        .map(|(ctx, request)| ((ctx, request.hash), request.payload))
        .unzip();
    let responses = match client {
        Ok(client) => {
            use futures::StreamExt;
            let mut indexed = client
                .predict_many(
                    requests
                        .into_iter()
                        .enumerate()
                        .map(|(id, request)| AudioClip {
                            cache_context: None,
                            id,
                            duration: std::time::Duration::ZERO,
                            audio: AudioInput::Request(request),
                        })
                        .collect(),
                )
                .collect::<Vec<_>>()
                .await;
            indexed.sort_by_key(|(id, _)| *id);
            indexed
                .into_iter()
                .map(|(_, response)| response)
                .collect::<Vec<_>>()
        }
        Err(error) => destinations
            .iter()
            .map(|_| Err(anyhow::anyhow!("{error:#}")))
            .collect(),
    };
    let mut results = Vec::with_capacity(destinations.len());
    for ((ctx, hash), response) in destinations.into_iter().zip(responses) {
        results.push(
            async {
                cache_response(ctx, hash, response?)
                    .await
                    .map(|(_, frames)| frames)
            }
            .await,
        );
    }
    results
}

/// The model's frame distribution, decoded from the same response artifact as
/// greedy predictions. All heads are requested even if this caller uses only CTC.
pub async fn frame_matrix(ctx: &VerifyContext<'_>, wav_bytes: &[u8]) -> Result<FrameMatrix> {
    Ok(prediction_response(ctx, wav_bytes).await?.1)
}

// A test harness for cache-miss compaction and explicit-batch routing.
#[cfg(test)]
async fn frame_matrices_at(
    ctx: &VerifyContext<'_>,
    wavs: &[&[u8]],
    client: Result<PhonemizerClient>,
    only_cache: bool,
) -> Vec<Result<FrameMatrix>> {
    let mut results: Vec<Option<Result<FrameMatrix>>> = (0..wavs.len()).map(|_| None).collect();
    let mut pending = Vec::new();
    for (index, wav) in wavs.iter().enumerate() {
        let hash = xxh3_64(wav);
        if let Some(Ok(frames)) = cached_frame_matrix(&ctx.store, &ctx.response_key(hash)).await {
            results[index] = Some(Ok(frames));
            continue;
        }
        if only_cache {
            results[index] = Some(Err(anyhow::anyhow!(
                "frame-matrix cache miss for {hash:016x}; cache-only mode is enabled"
            )));
            continue;
        }
        pending.push(index);
    }
    let mut pending = pending.into_iter().peekable();
    let mut requests = Vec::new();
    let mut indices = Vec::new();
    while let Some(index) = pending.next() {
        match prepare_frame_request(wavs[index].to_vec()).await {
            Ok(request) => {
                requests.push((ctx, request));
                indices.push(index);
            }
            Err(error) => results[index] = Some(Err(error)),
        }
        if !requests.is_empty() && (pending.peek().is_none()) {
            let frames = infer_frame_batch_at(std::mem::take(&mut requests), &client, None).await;
            for (index, frames) in indices.drain(..).zip(frames) {
                results[index] = Some(frames);
            }
        }
    }
    results
        .into_iter()
        .map(|result| result.expect("every batch item resolved"))
        .collect()
}

/// The deployed wav2vec2 encoder's feature extractor: a 400-sample
/// receptive field advancing 320 samples (20 ms at 16 kHz) per frame, so a
/// clip of `n` samples yields `(n - 400) / 320 + 1` frames. Timings depend
/// on this; [`segment_timings`] checks it against the matrix it gets.
const FRAME_RECEPTIVE_SAMPLES: usize = 400;
const FRAME_STRIDE_SAMPLES: usize = 320;
const FRAME_MS: u32 = 20;

/// Where each of `segments` — spoken text, in order, together making up the
/// clip's transcript — is said, as `[start_ms, end_ms)`. Each segment is
/// phonemized with the model's own g2p labels and the concatenation is
/// Viterbi-aligned to the clip's cached frame matrix; a segment spans its
/// first phoneme's first frame to its last phoneme's last frame. CTC
/// emissions are peaky (a phoneme's frames sit near its onset), so ends run
/// early. Errors when a segment has no phonemes the model knows or the
/// alignment is impossible; the caller decides what a clip without timings
/// is worth.
pub async fn segment_timings(
    ctx: &VerifyContext<'_>,
    audio_bytes: &[u8],
    segments: &[&str],
) -> Result<Vec<(u32, u32)>> {
    let matrix = frame_matrix(ctx, audio_bytes).await?;
    // A clip below the endpoint's length floor was padded symmetrically
    // before inference; report times against the audio as it is.
    let samples = decode_wav_to_f32(audio_bytes)
        .context("decoding audio to count samples")?
        .len();
    let padded = samples.max(wav2vec2::min_samples(MODAL_SAMPLE_RATE));
    let expected_frames = padded.saturating_sub(FRAME_RECEPTIVE_SAMPLES) / FRAME_STRIDE_SAMPLES + 1;
    if matrix.frames.abs_diff(expected_frames) > 1 {
        anyhow::bail!(
            "frame matrix has {} frames for {padded} samples, expected {expected_frames}: the \
             encoder's stride is not what the timings assume",
            matrix.frames
        );
    }
    let pad_ms = ((wav2vec2::min_samples(MODAL_SAMPLE_RATE).saturating_sub(samples) / 2) * 1000
        / MODAL_SAMPLE_RATE as usize) as u32;
    let targets = segments
        .iter()
        .map(|segment| {
            model_target(segment, ctx.target_language)
                .ok_or_else(|| anyhow::anyhow!("no g2p for {:?}", ctx.target_language))?
                .with_context(|| format!("phonemizing {segment:?}"))
        })
        .collect::<Result<Vec<_>>>()?;
    let spans = matrix.align_segments(&targets)?;
    let ms = |frame: usize| (frame as u32 * FRAME_MS).saturating_sub(pad_ms);
    Ok(spans
        .iter()
        .map(|&(start, end)| (ms(start), ms(end)))
        .collect())
}

/// Normalize a label for the deployed model's comparison rules.
pub fn normalize_phonemes(token: Phoneme, language: Language) -> Vec<Phoneme> {
    lexide::pronunciation::normalize_phonemes(token, scoring_language(language))
}

fn scoring_language(language: Language) -> Option<g2p::Language> {
    g2p::Language::from_code(language.code())
}

/// Hard cap on combinatorial expansion of word-level pronunciation
/// variants into phrase-level candidates. A phrase with 5 words each
/// having 2 variants generates 32 phrase candidates; we don't want to
/// expand a long sentence with many multi-variant words into thousands.
const MAX_VARIANT_COMBINATIONS: usize = 16;

/// Build the set of accepted phoneme sequences for a target-language
/// phrase. Each word in the phrase contributes its main pronunciation
/// plus any accepted alternates; the result is the (capped) cross
/// product across words.
///
/// A word wikipron lacks (a proper noun, a spelled-out letter name) gets
/// its own g2p phonemization (including Spanish's `es-419` dialect
/// alternate), so the rest of the phrase keeps its wikipron variants instead
/// of the whole phrase falling
/// back to g2p — which reads "cognac" as /konjak/ and would have rejected a
/// good clip over one unknown word beside it.
///
/// Returns `None` only if we couldn't produce *any* candidate — i.e. some
/// word is missing from wikipron AND g2p doesn't support this language. As
/// long as one source (wikipron cross-product OR g2p) produces something,
/// we return it.
///
/// Keep at most `MAX_VARIANT_COMBINATIONS` per-word combinations, with
/// earlier words varying fastest: cue-initial letter-name alternates take
/// priority over later example-word alternates. The main-only reading is
/// always first; phrase-level g2p candidates are added outside this cap.
///
/// The espeak phrase-level IPA, when available, is *always* added as an
/// extra candidate independent of the wikipron path. Espeak applies
/// connected-speech rules (liaison, elision, reduction) that the
/// word-by-word wikipron expansion misses. The motivating example:
/// "on est" — wikipron has "on"=`/ɔ̃/` or `/ɔ.n‿/` and "est"=`/ɛ/`, but
/// in actual speech (and TTS) it surfaces as `/ɔ̃ n ɛ/` (nasal vowel
/// preserved + liaison /n/) which neither combination produces. As a
/// side effect, espeak also rescues phrases that include words missing
/// from wikipron (`est-ce que`, `peut-être`) from silently being skipped.
fn ground_truth_phoneme_variants(
    text: &str,
    word_to_pronunciation: &HashMap<String, language_utils::Pronunciations>,
    language: Language,
) -> Option<Vec<g2p::Phonemized>> {
    // Collect per-word phoneme-sequence variants in phrase order.
    // `complete` flips to false when a word is missing from the dictionary
    // and g2p can't name it either; we then skip the cross-product and rely
    // on the phrase-level g2p variant (if available) as the sole ground truth.
    let mut per_word: Vec<Vec<Vec<Phoneme>>> = Vec::new();
    let mut complete = true;
    // `is_alphabetic` excludes some combining signs (notably Hindi nukta
    // and virama). Keep them attached in dictionary keys and g2p input,
    // along with decomposed accents and Russian stress marks.
    let is_word_char = |c: char| {
        c.is_alphabetic()
            || matches!(c, '\u{0300}'..='\u{036f}' | '\u{0900}'..='\u{0903}'
                | '\u{093a}'..='\u{094f}' | '\u{0951}'..='\u{0957}'
                | '\u{0962}'..='\u{0963}' | '\u{200c}' | '\u{200d}')
    };
    for word in text.split(|c: char| !is_word_char(c) && !matches!(c, '\'' | '-' | 'ʼ')) {
        let cleaned = word.trim_matches(|c: char| !is_word_char(c)).to_lowercase();
        if !cleaned.chars().any(char::is_alphabetic) {
            continue;
        }
        let Some(accepted) = word_to_pronunciation.get(&cleaned) else {
            let g2p_word: Vec<Phoneme> = match model_target(&cleaned, language) {
                Some(Ok(phonemized)) => phonemized.phonemes,
                _ => Vec::new(),
            };
            if g2p_word.is_empty() {
                complete = false;
            } else {
                let mut variants = vec![g2p_word];
                add_spanish_dialect_word(&cleaned, language, &mut variants);
                per_word.push(variants);
            }
            continue;
        };
        let mut word_variants: Vec<Vec<Phoneme>> = accepted
            .all()
            .filter_map(|ipa| import_ipa(ipa).map(|target| target.phonemes))
            // Distinct sequences only — after normalization, different raw
            // wikipron entries can collapse to the same phoneme sequence.
            .fold(Vec::new(), |mut acc, v| {
                if !acc.iter().any(|existing: &Vec<Phoneme>| {
                    existing
                        .iter()
                        .flat_map(|p| normalize_phonemes(*p, language))
                        .eq(v.iter().flat_map(|p| normalize_phonemes(*p, language)))
                }) {
                    acc.push(v);
                }
                acc
            });
        add_spanish_dialect_word(&cleaned, language, &mut word_variants);
        per_word.push(word_variants);
    }

    let candidates: Vec<Vec<Vec<Phoneme>>> = if !complete || per_word.is_empty() {
        // No per-word candidates — the phrase-level g2p below is our only shot.
        Vec::new()
    } else {
        let mut acc: Vec<Vec<Vec<Phoneme>>> = vec![Vec::new()];
        for word_variants in &per_word {
            let mut next = Vec::with_capacity(MAX_VARIANT_COMBINATIONS);
            // Alternates outside, prefixes inside: earlier words vary
            // fastest and survive truncation. Never compute the total
            // product or allocate more than the cap at any expansion.
            'variants: for var in word_variants {
                for prefix in &acc {
                    let mut extended = prefix.clone();
                    extended.push(var.clone());
                    next.push(extended);
                    if next.len() == MAX_VARIANT_COMBINATIONS {
                        break 'variants;
                    }
                }
            }
            acc = next;
        }
        acc
    };

    // These word-by-word dictionary combinations have no phrase prosody.
    let mut candidates: Vec<g2p::Phonemized> = candidates
        .into_iter()
        .map(g2p::Phonemized::from_words)
        .collect();

    // Add the phrase-level g2p variant in the model's own label space, for
    // languages the crate labels (`model_target` is `None` for the rest —
    // Python-backend or unvalidated — and reaching for espeak there would
    // score against the wrong phoneme inventory). Runs in-process, a few
    // ms. Failures are *important*: without this variant the verifier loses
    // all phrase-level ground truth for text wikipron can't decompose, so
    // log the first error per process rather than letting it vanish.
    for target in model_target(text, language)
        .into_iter()
        .chain(spanish_dialect_target(text, language))
    {
        match target {
            Ok(phonemized) => {
                let flat = comparable(&phonemized, language);
                if !flat.is_empty() && !candidates.iter().any(|c| comparable(c, language) == flat) {
                    candidates.push(phonemized);
                }
            }
            Err(e) => {
                static WARNED: std::sync::Once = std::sync::Once::new();
                WARNED.call_once(|| {
                    log::warn!("g2p phonemization failed (first occurrence shown only): {e:#}");
                });
            }
        }
    }

    if candidates.is_empty() {
        None
    } else {
        Some(candidates)
    }
}

fn import_ipa(ipa: &str) -> Option<g2p::Phonemized> {
    match g2p::Phonemized::from_ipa_tokens(ipa) {
        Ok(target) => Some(target),
        Err(error) => {
            log::warn!("cannot import pronunciation {ipa:?}: {error}");
            None
        }
    }
}

fn comparable(target: &g2p::Phonemized, language: Language) -> Vec<Phoneme> {
    target
        .phonemes
        .iter()
        .flat_map(|p| normalize_phonemes(*p, language))
        .collect()
}

/// Seseo is an accepted Spanish dialect reading, not a θ/s equivalence:
/// keep the training-contract `es` target and both phones unchanged.
fn spanish_dialect_target(
    text: &str,
    language: Language,
) -> Option<Result<g2p::Phonemized, g2p::Error>> {
    (language == Language::Spanish)
        .then(|| g2p::phonemize(g2p::Language::SpanishLatinAmerica, text))
}

fn add_spanish_dialect_word(text: &str, language: Language, variants: &mut Vec<Vec<Phoneme>>) {
    if let Some(Ok(target)) = spanish_dialect_target(text, language) {
        let phones = target.phonemes;
        if !phones.is_empty()
            && !variants.iter().any(|v| {
                v.iter()
                    .flat_map(|p| normalize_phonemes(*p, language))
                    .eq(phones.iter().flat_map(|p| normalize_phonemes(*p, language)))
            })
        {
            variants.push(phones);
        }
    }
}

// --- Google-TTS-driven fallback verification ---------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedTts {
    /// The original text that produced this audio. Recorded so the cache is
    /// debuggable from filename → text without re-running. `Option` so older
    /// cache entries (written before this field existed) still load.
    #[serde(default)]
    text: Option<String>,
    /// OGG/Opus audio bytes, base64-encoded.
    audio_base64: String,
    attempts: usize,
    /// `passed` indicates the audio passed Google's defect check;
    /// otherwise we returned the last attempt and recorded the defect.
    passed: bool,
    last_defect: Option<String>,
}

/// Voice and language metadata for the Google TTS call. Caller maps their
/// own Language enum into this.
#[derive(Debug, Clone, Copy)]
pub struct TtsVoice {
    pub language_code: &'static str,
    pub voice_name: &'static str,
}

/// Map a target language to Google TTS voice metadata. Currently only the
/// languages we ship audio for. Returns `None` for unsupported languages.
pub fn default_voice_for(language: Language) -> Option<TtsVoice> {
    Some(match language {
        Language::French => TtsVoice {
            language_code: "fr-FR",
            voice_name: "fr-FR-Chirp3-HD-Achernar",
        },
        Language::Spanish => TtsVoice {
            language_code: "es-US",
            voice_name: "es-US-Chirp3-HD-Achernar",
        },
        Language::English => TtsVoice {
            language_code: "en-US",
            voice_name: "en-US-Chirp3-HD-Achernar",
        },
        Language::German => TtsVoice {
            language_code: "de-DE",
            voice_name: "de-DE-Chirp3-HD-Achernar",
        },
        Language::Italian => TtsVoice {
            language_code: "it-IT",
            voice_name: "it-IT-Chirp3-HD-Achernar",
        },
        Language::Portuguese => TtsVoice {
            language_code: "pt-BR",
            voice_name: "pt-BR-Chirp3-HD-Achernar",
        },
        Language::Russian => TtsVoice {
            language_code: "ru-RU",
            voice_name: "ru-RU-Chirp3-HD-Aoede",
        },
        Language::Korean => TtsVoice {
            language_code: "ko-KR",
            voice_name: "ko-KR-Chirp3-HD-Achernar",
        },
        _ => return None,
    })
}

/// Call Google TTS for `text` (using `voice`), running the result through
/// the same wav2vec2 verifier we use for human audio. The synthesized audio
/// is cached under `google-tts/{hash}` keyed by (text, voice, speed).
///
/// `actor` is recorded on the resulting [`ClipVerification`] for logging —
/// callers typically pass something like `"google-tts"` so the synthetic
/// verifications are easy to grep out of the all-results log.
///
/// If the Google TTS retry loop hit its limit, this prepends a note to the
/// verification's `failure_reason` so the synthetic audio's outcome is
/// distinguishable from a clean pass-or-fail.
pub async fn verify_with_google_tts(
    ctx: &VerifyContext<'_>,
    actor: &str,
    text: &str,
    voice: TtsVoice,
    google_api_key: &str,
) -> Result<ClipVerification> {
    let synthesis = TtsSynthesis::Google {
        voice,
        text: text.to_string(),
    };
    let keys = TtsKeys {
        google: Some(google_api_key.to_string()),
        gemini: None,
    };
    let (_, verification) = synthesize_verified(ctx, actor, &synthesis, text, &keys).await?;
    Ok(verification)
}

/// API keys a synthesis may need. Each is optional so a cache-only run can
/// consume already populated entries; a key is required only on a miss.
#[derive(Debug, Clone, Default)]
pub struct TtsKeys {
    pub google: Option<String>,
    pub gemini: Option<String>,
}

impl TtsKeys {
    /// `GOOGLE_CLOUD_API_KEY` and `GEMINI_API_KEY`, whichever are set.
    pub fn from_env() -> Self {
        Self {
            google: std::env::var("GOOGLE_CLOUD_API_KEY").ok(),
            gemini: std::env::var("GEMINI_API_KEY").ok(),
        }
    }
}

/// One way of producing a clip of some spoken text. A cue is tried through
/// several of these in order — Gemini twice, then Cloud TTS — and the first
/// whose clip passes verification is kept.
#[derive(Debug, Clone)]
pub enum TtsSynthesis {
    /// Cloud TTS: a fixed per-language voice reading `text` literally.
    Google { voice: TtsVoice, text: String },
    /// Gemini reading `text` under `instructions`. The model is stochastic,
    /// so `attempt` distinguishes repeated draws of the same prompt in the
    /// cache — a second draw is a genuinely different clip.
    Gemini {
        voice: String,
        instructions: String,
        text: String,
        attempt: u32,
    },
}

impl TtsSynthesis {
    /// Where the clip lives in the cache: hashed from every input that
    /// determines the audio, so a change to any of them misses automatically.
    fn cache_key(&self) -> String {
        match self {
            TtsSynthesis::Google { voice, text } => {
                let speed = 1.0f64;
                let seed = format!(
                    "{text}|{}|{}|{speed}|ssml=false",
                    voice.language_code, voice.voice_name
                );
                format!("google-tts/{:016x}", xxh3_64(seed.as_bytes()))
            }
            TtsSynthesis::Gemini {
                voice,
                instructions,
                text,
                attempt,
            } => {
                let seed = format!(
                    "{}|{voice}|{instructions}|{text}|{attempt}",
                    google_speech::gemini::GEMINI_TTS_MODEL
                );
                // v2: clips encoded with the lookahead flushed; v1 entries
                // lost their last few milliseconds to the encoder delay.
                format!("gemini-tts/v2/{:016x}", xxh3_64(seed.as_bytes()))
            }
        }
    }

    /// Recorded as the verification's `wav_path`: which provider and voice
    /// produced the clip, and for Gemini which draw.
    pub fn label(&self) -> String {
        match self {
            TtsSynthesis::Google { voice, .. } => format!("google-tts://{}", voice.voice_name),
            TtsSynthesis::Gemini { voice, attempt, .. } => {
                format!("gemini-tts://{voice}#{attempt}")
            }
        }
    }
}

/// Synthesize (or load from the cache) one clip and verify it against
/// `spoken_text`: the same words the voice was given, phonemized as the
/// reference. The returned audio is Ogg Opus whichever provider made it,
/// ready to embed in a language pack.
///
/// A provider's own refusal to produce usable audio — Cloud TTS exhausting
/// its defect retries, Gemini declining a prompt or answering without audio
/// — comes back as a failed [`ClipVerification`], cached like any other
/// outcome, so the caller moves on to its next candidate. Transport errors
/// and exhausted rate-limit backoff are `Err`: the run can't tell good audio
/// from bad and should stop rather than quietly fall through.
pub async fn synthesize_verified(
    ctx: &VerifyContext<'_>,
    actor: &str,
    synthesis: &TtsSynthesis,
    spoken_text: &str,
    keys: &TtsKeys,
) -> Result<(Vec<u8>, ClipVerification)> {
    let cache_key = synthesis.cache_key();
    let label = synthesis.label();

    let (audio_bytes, tts_note) = if let Some(s) = ctx.store.read(&cache_key).await
        && let Ok(cached) = serde_json::from_slice::<CachedTts>(&s)
    {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&cached.audio_base64)
            .context("Cached TTS audio_base64 was not valid base64")?;
        let note = (!cached.passed).then(|| {
            format!(
                "{label} gave up after {} attempt(s) ({})",
                cached.attempts,
                cached.last_defect.as_deref().unwrap_or("unknown defect")
            )
        });
        (bytes, note)
    } else {
        if cache_only() {
            anyhow::bail!(
                "TTS cache miss for {spoken_text:?} via {label} ({cache_key}); cache-only mode is \
                 enabled"
            );
        }
        let (bytes, attempts, passed, last_defect): (Vec<u8>, usize, bool, Option<String>) =
            match synthesis {
                TtsSynthesis::Google { voice, text } => {
                    let api_key = keys.google.as_deref().ok_or_else(|| {
                        anyhow::anyhow!("GOOGLE_CLOUD_API_KEY is required for uncached TTS audio")
                    })?;
                    let client = google_speech::GoogleTtsClient::new(api_key.to_string())
                        .with_max_attempts(5);
                    let outcome = client
                        .synthesize(&google_speech::GoogleTtsRequest {
                            text: text.clone(),
                            language_code: voice.language_code.to_string(),
                            voice_name: voice.voice_name.to_string(),
                            speed: 1.0,
                            is_ssml: false,
                        })
                        .await
                        .with_context(|| format!("Google TTS call failed for {text:?}"))?;
                    let (passed, last_defect) = match outcome.status {
                        google_speech::TtsStatus::Passed => (true, None),
                        google_speech::TtsStatus::HitLimit { last_defect } => {
                            (false, Some(last_defect.to_string()))
                        }
                    };
                    (outcome.audio_bytes, outcome.attempts, passed, last_defect)
                }
                TtsSynthesis::Gemini {
                    voice,
                    instructions,
                    text,
                    ..
                } => {
                    let api_key = keys.gemini.as_deref().ok_or_else(|| {
                        anyhow::anyhow!("GEMINI_API_KEY is required for uncached Gemini TTS audio")
                    })?;
                    let client = google_speech::gemini::GeminiClient::with_http(
                        api_key.to_string(),
                        ctx.http.clone(),
                    );
                    let request = google_speech::gemini::GeminiTtsRequest {
                        instructions: instructions.clone(),
                        text: text.clone(),
                        voice: voice.clone(),
                    };
                    match client.synthesize(&request).await {
                        Ok(Some(audio)) => {
                            let bytes = audio
                                .to_ogg_opus()
                                .with_context(|| format!("encoding Gemini audio for {text:?}"))?;
                            let defect = audio_codec::audio_defect(&bytes);
                            (bytes, 1, defect.is_none(), defect.map(str::to_string))
                        }
                        Ok(None) => (Vec::new(), 1, false, Some("no audio in response".into())),
                        Err(google_speech::gemini::GeminiError::Declined(body)) => {
                            (Vec::new(), 1, false, Some(format!("declined: {body}")))
                        }
                        Err(google_speech::gemini::GeminiError::Other(e)) => {
                            return Err(e.context(format!("Gemini TTS call failed for {text:?}")));
                        }
                    }
                }
            };
        let to_cache = CachedTts {
            text: Some(spoken_text.to_string()),
            audio_base64: base64::engine::general_purpose::STANDARD.encode(&bytes),
            attempts,
            passed,
            last_defect: last_defect.clone(),
        };
        ctx.store
            .write(
                &cache_key,
                serde_json::to_string(&to_cache)
                    .context("serialize cached_tts")?
                    .as_bytes(),
            )
            .await
            .with_context(|| format!("Failed to write cache entry {cache_key}"))?;
        let note = (!passed).then(|| {
            format!(
                "{label} gave up after {attempts} attempt(s) ({})",
                last_defect.unwrap_or_else(|| "unknown defect".to_string())
            )
        });
        (bytes, note)
    };

    // If the provider gave up, skip verification entirely. Running wav2vec2
    // on near-silent or truncated audio invites the model to hallucinate
    // plausible phonemes (the `pas` case: TTS returned 0.19s of -50 dB
    // audio, we padded to 0.6s with zeros, model emitted `p a` matching
    // expected, edit_distance came out 0 — but the audio itself is
    // unusable). Don't trust verification on audio the provider already
    // flagged; surface the provider's note as the failure.
    if let Some(note) = tts_note {
        let verification = ClipVerification {
            cache_version: None,
            model: None,
            actor: actor.to_string(),
            text: spoken_text.to_string(),
            wav_path: label,
            predicted_raw: Vec::new(),
            predicted_normalized: Vec::new(),
            expected: None,
            variants_considered: 0,
            edit_distance: None,
            edit_distance_pct: None,
            alignment: None,
            failure_reason: Some(note),
        };
        return Ok((audio_bytes, verification));
    }

    // Synthesized clips have no per-clip transcription overrides — they were
    // made from the canonical text, so the wikipron+espeak ground truth is
    // the right reference.
    let expected = expected_phoneme_variants(ctx, spoken_text, None);
    let verification =
        verify_clip_bytes(ctx, actor, spoken_text, &label, &audio_bytes, expected).await?;
    Ok((audio_bytes, verification))
}

use std::sync::atomic::{AtomicBool, Ordering};

static CACHE_ONLY: AtomicBool = AtomicBool::new(false);

/// Refuse every network call process-wide: a phoneme prediction or TTS
/// synthesis that isn't already cached becomes an error. generate-data
/// mirrors its own `--cache-only` flag here.
pub fn set_cache_only(enabled: bool) {
    CACHE_ONLY.store(enabled, Ordering::Relaxed);
}

pub fn cache_only() -> bool {
    CACHE_ONLY.load(Ordering::Relaxed)
}
#[cfg(test)]
fn raw_prediction(response: ModalResponse) -> RawPrediction {
    RawPrediction {
        item: serde_json::value::to_raw_value(&response).unwrap(),
        envelope: Default::default(),
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn german_normalization_and_provenance_cover_every_verification_path() {
        let dir = tempfile::tempdir().unwrap();
        let http = reqwest::Client::new();
        let mut dictionary = HashMap::new();
        dictionary.insert("test".into(), ap("ʔ t͡ʃ ɪ̯", &[]));
        let ctx = VerifyContext::with_overrides(
            &http,
            osmo::Store::open(dir.path()),
            &dictionary,
            Language::German,
            0.3,
            None,
        )
        .unwrap();
        let expected = expected_phoneme_variants(&ctx, "test", Some("ʔ t͡ʃ ɪ̯")).unwrap();
        assert_eq!(
            comparable(&expected[0], Language::German),
            word(&["t", "ʃ", "ɪ"])
        );
        assert!(
            expected_phoneme_variants(&ctx, "test", None)
                .unwrap()
                .iter()
                .any(|candidate| candidate.phonemes == expected[0].phonemes
                    && candidate.word_spans == expected[0].word_spans)
        );
        // Invalid bytes intentionally bypass ffmpeg's defect gate; both model
        // payloads below are cached, so there is no inference/network request.
        let audio = b"cached normalization regression";
        let hash = xxh3_64(audio);
        let response: ModalResponse = serde_json::from_value(serde_json::json!({
            "phonemes": [{"phoneme": "ʔ"}, {"phoneme": "t͜ʃ"}, {"phoneme": "ɪ̯"}],
            "frame_matrix": batch_test_payload(1),
            "model_id": "actual-model", "model_revision": "actual-revision"
        }))
        .unwrap();
        cache_response(&ctx, hash, raw_prediction(response))
            .await
            .unwrap();
        let passed = verify_clip_bytes(&ctx, "test", "test", "cached", audio, Some(expected))
            .await
            .unwrap();
        assert!(passed.passed(), "{:?}", passed.failure_reason);
        assert_eq!(passed.edit_distance, Some(0));
        let missing = verify_clip_bytes(&ctx, "test", "test", "cached", audio, None)
            .await
            .unwrap();
        assert!(!missing.passed());
        let defective = verify_clip_bytes(&ctx, "test", "test", "silent", &batch_test_wav(0), None)
            .await
            .unwrap();
        assert!(
            defective
                .failure_reason
                .as_deref()
                .unwrap()
                .starts_with("audio defect:")
        );
        let synthesis = TtsSynthesis::Google {
            voice: default_voice_for(Language::German).unwrap(),
            text: "test".into(),
        };
        ctx.store
            .write(
                &synthesis.cache_key(),
                &serde_json::to_vec(&CachedTts {
                    text: Some("test".into()),
                    audio_base64: String::new(),
                    attempts: 5,
                    passed: false,
                    last_defect: Some("silent".into()),
                })
                .unwrap(),
            )
            .await
            .unwrap();
        let (_, rejected_tts) =
            synthesize_verified(&ctx, "test", &synthesis, "test", &TtsKeys::default())
                .await
                .unwrap();
        assert!(!rejected_tts.passed());
        for row in [passed, missing] {
            assert_eq!(row.model.as_ref().unwrap().model_id, "actual-model");
            assert!(
                row.cache_version.is_none(),
                "missing reported decoder is not today's decoder"
            );
        }
        for row in [defective, rejected_tts] {
            assert!(row.model.is_none());
            assert!(row.cache_version.is_none());
        }
    }

    use super::*;
    fn batch_test_payload(frames: usize) -> FrameMatrixPayload {
        let mut encoder =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        for _ in 0..frames {
            for value in [-2.0_f32, -0.1] {
                encoder
                    .write_all(&half::f16::from_f32(value).to_le_bytes())
                    .unwrap();
            }
        }
        FrameMatrixPayload::Legacy(LegacyFrameMatrixPayload {
            shape: vec![frames, 2],
            dtype: "float16".into(),
            encoding: "zlib+base64".into(),
            blank_id: 0,
            vocab: vec!["<pad>".into(), "a".into()],
            data: base64::engine::general_purpose::STANDARD.encode(encoder.finish().unwrap()),
        })
    }

    #[tokio::test]
    async fn response_cache_is_content_keyed_and_ignores_producer_changes() {
        let dir = tempfile::tempdir().unwrap();
        let http = reqwest::Client::new();
        let words = HashMap::new();
        let mut ctx = VerifyContext::with_overrides(
            &http,
            osmo::Store::open(dir.path()),
            &words,
            Language::English,
            0.3,
            None,
        )
        .unwrap();
        let wav = b"cached raw frames";
        let hash = xxh3_64(wav);
        let response: ModalResponse = serde_json::from_value(serde_json::json!({
            "phonemes": [], "frame_matrix": batch_test_payload(3),
            "model_id": "actual", "model_revision": "revision"
        }))
        .unwrap();
        cache_response(&ctx, hash, raw_prediction(response.clone()))
            .await
            .unwrap();
        ctx.client = ctx.client.clone().with_expected_identity(test_identity()); // different from cached producer
        assert_eq!(frame_matrix(&ctx, wav).await.unwrap().frames, 3);
        let batch = frame_matrices_at(&ctx, &[wav], Err(anyhow::anyhow!("offline")), true).await;
        assert_eq!(batch[0].as_ref().unwrap().frames, 3);
        assert_eq!(
            response_identity(
                &cached_response(&ctx.store, &ctx.response_key(hash))
                    .await
                    .unwrap()
                    .unwrap()
            )
            .unwrap()
            .model_id,
            "actual"
        );
        ctx.client = wav2vec2::batch_client(http.clone())
            .unwrap()
            .with_cache(ctx.store.clone());
        ctx = ctx.with_cache_context("caller");
        assert!(
            cached_frame_matrix(&ctx.store, &ctx.response_key(hash))
                .await
                .is_none()
        );
        cache_response(&ctx, hash, raw_prediction(response))
            .await
            .unwrap();
        assert!(
            cached_frame_matrix(&ctx.store, &ctx.response_key(hash))
                .await
                .unwrap()
                .is_ok()
        );
        assert!(
            cached_frame_matrix(&ctx.store, &ctx.response_key(hash + 1))
                .await
                .is_none()
        );
        ctx.store
            .write(&ctx.response_key(hash), b"corrupt")
            .await
            .unwrap();
        assert!(
            cached_frame_matrix(&ctx.store, &ctx.response_key(hash))
                .await
                .unwrap()
                .is_err()
        );
        let identity = ModelIdentity {
            model_id: "model/id".into(),
            model_revision: "123456789012-A".into(),
            decoder_version: Some(DECODER_VERSION.into()),
            deploy_marker: Some("first".into()),
        };
        ctx.key_by_model(&identity);
        let key = ctx.response_key(hash);
        let mut another = identity.clone();
        another.deploy_marker = Some("second".into());
        ctx.key_by_model(&another);
        assert_eq!(ctx.response_key(hash), key);
        another.model_revision = "123456789012-B".into();
        ctx.key_by_model(&another);
        assert_ne!(ctx.response_key(hash), key, "full revisions are key inputs");
    }

    #[tokio::test]
    async fn v1_response_preserves_all_typed_heads_and_observed_producer() {
        let dir = tempfile::tempdir().unwrap();
        let http = reqwest::Client::new();
        let words = HashMap::new();
        let ctx = VerifyContext::with_overrides(
            &http,
            osmo::Store::open(dir.path()),
            &words,
            Language::English,
            0.3,
            None,
        )
        .unwrap();
        let FrameMatrixPayload::Legacy(phone) = batch_test_payload(1) else {
            unreachable!()
        };
        let head = |shape: Vec<usize>, labels: Vec<&str>, semantics: &str| {
            let mut encoder =
                flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
            encoder
                .write_all(&vec![0; shape.iter().product::<usize>() * 2])
                .unwrap();
            serde_json::json!({"shape": shape, "labels": labels, "dtype": "float16", "encoding": "zlib+base64", "value_semantics": semantics,
                "data": base64::engine::general_purpose::STANDARD.encode(encoder.finish().unwrap())})
        };
        let mut tone = head(vec![1, 2], vec!["low", "high"], "probability");
        tone["language"] = "zho".into();
        tone["target"] = "tone".into();
        let response: ModalResponse = serde_json::from_value(serde_json::json!({
            "phonemes": [], "frame_matrix": {
                "schema_version": 1, "producer": test_identity(), "trained_against_g2p": "training-label-renderer",
                "sample_rate": 16000, "frame_rate_ms": 20.0,
                "heads": {
                    "phone": {"shape": phone.shape, "labels": phone.vocab, "blank_id": phone.blank_id,
                        "dtype": phone.dtype, "encoding": phone.encoding, "data": phone.data, "value_semantics": "joint_log_probability"},
                    "nonblank": head(vec![1], vec!["nonblank"], "sigmoid_probability"),
                    "stress": head(vec![1, 3], vec!["none", "primary", "secondary"], "probability"),
                    "tone_zho": tone
                }
            }
        })).unwrap();
        cache_response(&ctx, 42, raw_prediction(response.clone()))
            .await
            .unwrap();
        let cached = cached_response(&ctx.store, &ctx.response_key(42))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            serde_json::to_value(&cached).unwrap(),
            serde_json::to_value(&response).unwrap()
        );
        let frames = cached_frame_matrix(&ctx.store, &ctx.response_key(42))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(frames.heads.len(), 4);
        assert_eq!(frame_identity(&frames), Some(test_identity()));
        let mut conflicting = response;
        conflicting.model_revision = Some("not the matrix revision".into());
        assert!(
            cache_response(&ctx, 43, raw_prediction(conflicting))
                .await
                .is_err()
        );
        assert!(
            cached_response(&ctx.store, &ctx.response_key(43))
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn batch_cache_retains_exact_item_and_all_envelope_raw_values() {
        let matrix = serde_json::to_string(&batch_test_payload(1)).unwrap();
        let unknown = r#"{ "integer":9007199254740993, "decimal":1.2300e+02, "escape":"a\/b", "nested":[ 1, 2 ] }"#;
        let item =
            format!(r#"{{ "phonemes": [], "frame_matrix": {matrix}, "future_item": {unknown} }}"#);
        let sibling =
            format!(r#"{{"phonemes":[],"frame_matrix":{matrix},"other":"sibling-only"}}"#);
        let body = format!(
            r#"{{"model_id":"actual/model", "model_revision":"full-revision", "decoder_version":"nonblank_v1", "deploy_marker":"served", "future_envelope": {unknown}, "results":[{item},{sibling}]}}"#
        );
        let (url, server) = wav2vec2::tests::test_server(vec![(200, body)]);
        let dir = tempfile::tempdir().unwrap();
        let http = reqwest::Client::builder()
            .no_proxy()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap();
        let words = HashMap::new();
        let ctx = VerifyContext::with_overrides(
            &http,
            osmo::Store::open(dir.path()),
            &words,
            Language::English,
            0.3,
            Some("served".into()),
        )
        .unwrap();
        let request = || {
            wav2vec2::Clip {
                samples: &[0.0],
                sample_rate: 16000,
                top_k: 10,
            }
            .into_request()
        };
        let frames = infer_frame_batch_at(
            vec![
                (
                    &ctx,
                    PreparedFrameRequest {
                        hash: 42,
                        payload: request(),
                    },
                ),
                (
                    &ctx,
                    PreparedFrameRequest {
                        hash: 43,
                        payload: request(),
                    },
                ),
            ],
            &PhonemizerClient::with_endpoints(http.clone(), &url, &url),
            None,
        )
        .await;
        assert!(frames.iter().all(Result::is_ok));
        let wire = server.join().unwrap();
        assert!(
            wire[0].1["requests"][0]["return_frame_matrix"]
                .as_bool()
                .unwrap()
        );
        assert!(
            wire[0].1["requests"][0]["return_all_heads"]
                .as_bool()
                .unwrap()
        );
        let bytes = ctx.store.read(&ctx.response_key(42)).await.unwrap();
        let mut cached: RawPrediction = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(cached.item.get(), item);
        assert_eq!(cached.envelope["future_envelope"].get(), unknown);
        assert_eq!(cached.envelope.len(), 5);
        assert!(!cached.envelope.contains_key("results"));
        assert!(!String::from_utf8(bytes).unwrap().contains("sibling-only"));
        assert_eq!(
            cached.decode().unwrap().model_id.as_deref(),
            Some("actual/model")
        );
        assert_eq!(
            cached_frame_matrix(&ctx.store, &ctx.response_key(42))
                .await
                .unwrap()
                .unwrap()
                .frames,
            1
        );
        // Older decoder output remains readable; only live writes apply today's
        // expected decoder/model/deployment policy.
        cached.envelope.insert(
            "decoder_version".into(),
            serde_json::value::to_raw_value("older-decoder").unwrap(),
        );
        assert!(cache_response(&ctx, 44, cached.clone()).await.is_err());
        ctx.store
            .write(&ctx.response_key(42), &serde_json::to_vec(&cached).unwrap())
            .await
            .unwrap();
        assert!(
            cached_frame_matrix(&ctx.store, &ctx.response_key(42))
                .await
                .unwrap()
                .is_ok()
        );
        assert_eq!(
            cached_response(&ctx.store, &ctx.response_key(42))
                .await
                .unwrap()
                .unwrap()
                .decoder_version
                .as_deref(),
            Some("older-decoder")
        );
        // A producer expectation change cannot rewrite or reject persisted raw data.
        let mut changed = ctx;
        changed.client = changed
            .client
            .clone()
            .with_expected_identity(test_identity());
        assert_eq!(
            cached_response(&changed.store, &changed.response_key(42))
                .await
                .unwrap()
                .unwrap()
                .model_revision
                .as_deref(),
            Some("full-revision")
        );
    }

    fn batch_test_wav(seed: i16) -> Vec<u8> {
        let samples = 1600_u32;
        let mut wav = b"RIFF".to_vec();
        wav.extend_from_slice(&(36 + samples * 2).to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16_u32.to_le_bytes());
        wav.extend_from_slice(&1_u16.to_le_bytes()); // PCM
        wav.extend_from_slice(&1_u16.to_le_bytes()); // mono
        wav.extend_from_slice(&16000_u32.to_le_bytes());
        wav.extend_from_slice(&32000_u32.to_le_bytes());
        wav.extend_from_slice(&2_u16.to_le_bytes());
        wav.extend_from_slice(&16_u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&(samples * 2).to_le_bytes());
        for _ in 0..samples {
            wav.extend_from_slice(&seed.to_le_bytes());
        }
        wav
    }

    #[test]
    fn model_target_identity_is_the_g2p_identity() {
        assert_eq!(model_target_identity(), g2p::identity());
    }

    fn test_identity() -> ModelIdentity {
        ModelIdentity {
            model_id: "test/model".into(),
            model_revision: "1234567890abcdef".into(),
            decoder_version: Some(DECODER_VERSION.into()),
            deploy_marker: Some("fresh".into()),
        }
    }

    fn raw_batch(value: serde_json::Value) -> lexide::pronunciation::RawBatchResponse {
        let text = value.to_string();
        let batch = serde_json::from_str(&text).unwrap();
        let mut envelope: std::collections::BTreeMap<String, Box<serde_json::value::RawValue>> =
            serde_json::from_str(&text).unwrap();
        envelope.remove("results");
        lexide::pronunciation::RawBatchResponse { batch, envelope }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn response_and_envelope_identity_guards() {
        let dir = tempfile::tempdir().unwrap();
        let http = reqwest::Client::new();
        let words = HashMap::new();
        // Explicit contexts never discover production identity.
        let mut ctx = VerifyContext::with_overrides(
            &http,
            osmo::Store::open(dir.path()),
            &words,
            Language::English,
            0.3,
            Some("fresh".into()),
        )
        .unwrap();
        let response = |metadata: serde_json::Value| {
            let mut value = metadata;
            value["phonemes"] = serde_json::json!([]);
            serde_json::from_value::<ModalResponse>(value).unwrap()
        };
        let good = serde_json::to_value(test_identity()).unwrap();
        assert!(check_response_identity(&ctx, &response(good.clone())).is_ok());
        // Overrides still enforce decoder and marker; no identity pin is inferred.
        for (field, value) in [("decoder_version", "wrong"), ("deploy_marker", "stale")] {
            let mut bad = good.clone();
            bad[field] = serde_json::json!(value);
            assert!(check_response_identity(&ctx, &response(bad)).is_err());
        }
        ctx.client = ctx.client.clone().with_expected_identity(test_identity());
        for field in [
            "model_id",
            "model_revision",
            "decoder_version",
            "deploy_marker",
        ] {
            let mut bad = good.clone();
            bad[field] = serde_json::json!("wrong");
            assert!(check_response_identity(&ctx, &response(bad.clone())).is_err());
            // A good per-item claim must not mask a bad envelope.
            bad["results"] = serde_json::json!([response(good.clone())]);
            let split = raw_batch(bad).into_predictions();
            assert!(split.is_err() || split.unwrap().remove(0).is_err());
            // Missing per-item metadata inherits the envelope and is checked.
            let mut envelope = good.clone();
            envelope[field] = serde_json::json!("wrong");
            envelope["results"] = serde_json::json!([{"phonemes": []}]);
            match raw_batch(envelope).into_predictions() {
                Err(_) => {}
                Ok(mut items) => {
                    assert!(
                        check_response_identity(&ctx, &items.remove(0).unwrap().decode().unwrap())
                            .is_err()
                    )
                }
            }
        }
        let mut envelope = good.clone();
        envelope["results"] = serde_json::json!([{"phonemes": []}]);
        let item = raw_batch(envelope)
            .into_predictions()
            .unwrap()
            .remove(0)
            .unwrap();
        let item = item.decode().unwrap();
        assert!(check_response_identity(&ctx, &item).is_ok());
        assert_eq!(item.model_revision, Some(test_identity().model_revision));
        assert!(check_response_identity(&ctx, &response(serde_json::json!({}))).is_err());
        ctx.client = wav2vec2::batch_client(http.clone())
            .unwrap()
            .with_cache(ctx.store.clone());
        assert!(check_response_identity(&ctx, &response(serde_json::json!({}))).is_ok());
    }

    #[tokio::test]
    async fn cache_only_reader_reports_corrupt_matrix_without_inference() {
        let dir = tempfile::tempdir().unwrap();
        let http = reqwest::Client::new();
        let empty = HashMap::new();
        let ctx = VerifyContext::with_overrides(
            &http,
            osmo::Store::open(dir.path()),
            &empty,
            Language::English,
            0.3,
            None,
        )
        .unwrap();
        let wav = b"not audio: neither decoding nor inference should run";
        let hash = xxh3_64(wav);
        let key = ctx.response_key(hash);
        let mut payload = batch_test_payload(1);
        let FrameMatrixPayload::Legacy(legacy) = &mut payload else {
            unreachable!("fixture uses the legacy wire format")
        };
        legacy.encoding = "broken".into();
        let bytes = serde_json::to_vec(&serde_json::json!({
            "envelope": {}, "item": {"phonemes": [], "frame_matrix": payload}
        }))
        .unwrap();
        ctx.store.write(&key, &bytes).await.unwrap();
        let cached = cached_frame_matrix(&ctx.store, &ctx.response_key(hash))
            .await
            .expect("not a cache miss");
        assert!(
            cached
                .unwrap_err()
                .to_string()
                .contains("unsupported frame matrix format")
        );
        let results = frame_matrices_at(
            &ctx,
            &[wav],
            Err(anyhow::anyhow!("inference must not run")),
            true,
        )
        .await;
        assert!(
            results[0]
                .as_ref()
                .unwrap_err()
                .to_string()
                .contains("cache-only mode")
        );
        assert_eq!(ctx.store.read(&key).await.unwrap(), bytes);
    }

    #[tokio::test]
    async fn batch_cache_misses_are_bounded_ordered_and_isolated() {
        use std::io::Read;
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            for (batch, count) in [64, 64, 1].into_iter().enumerate() {
                let (mut socket, _) = listener.accept().unwrap();
                socket
                    .set_read_timeout(Some(std::time::Duration::from_secs(30)))
                    .unwrap();
                let mut bytes = Vec::new();
                let (header_end, length) = loop {
                    let mut block = [0; 8192];
                    let n = socket.read(&mut block).unwrap();
                    assert!(n > 0);
                    bytes.extend_from_slice(&block[..n]);
                    if let Some(end) = bytes.windows(4).position(|s| s == b"\r\n\r\n") {
                        let headers = String::from_utf8_lossy(&bytes[..end]);
                        let length = headers
                            .lines()
                            .find_map(|line| {
                                let (key, value) = line.split_once(':')?;
                                key.eq_ignore_ascii_case("content-length")
                                    .then(|| value.trim().parse::<usize>().unwrap())
                            })
                            .unwrap();
                        break (end + 4, length);
                    }
                };
                while bytes.len() < header_end + length {
                    let mut block = [0; 8192];
                    let n = socket.read(&mut block).unwrap();
                    assert!(n > 0);
                    bytes.extend_from_slice(&block[..n]);
                }
                let request: serde_json::Value =
                    serde_json::from_slice(&bytes[header_end..header_end + length]).unwrap();
                let items = request["requests"].as_array().unwrap();
                assert_eq!(items.len(), count);
                assert!(
                    items.iter().all(
                        |item| item["audio_f32_b64"].is_string() && item.get("audio").is_none()
                    )
                );
                if batch == 1 {
                    // A fatal request-wide error fails these 64 clips, not
                    // the subsequent tail. It must not enter retry backoff.
                    write!(
                        socket,
                        "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    )
                    .unwrap();
                    continue;
                }
                let results: Vec<_> = (0..count).map(|i| {
                    if batch == 0 {
                        match i {
                            1 => return serde_json::json!({"error": {"type": "ValueError", "message": "bad clip"}}),
                            2 => return serde_json::json!({"phonemes": [], "frame_matrix": batch_test_payload(1), "deploy_marker": "wrong"}),
                            3 => return serde_json::json!({"phonemes": []}),
                            4 => {
                                let mut invalid = batch_test_payload(1);
                                let FrameMatrixPayload::Legacy(legacy) = &mut invalid else {
                                    unreachable!("fixture uses the legacy wire format")
                                };
                                legacy.encoding = "invalid".into();
                                return serde_json::json!({"phonemes": [], "frame_matrix": invalid});
                            }
                            _ => {}
                        }
                    }
                    serde_json::json!({"phonemes": [], "frame_matrix": batch_test_payload(1)})
                }).collect();
                let body = serde_json::to_vec(
                    &serde_json::json!({"results": results, "deploy_marker": "test"}),
                )
                .unwrap();
                write!(socket, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).unwrap();
                socket.write_all(&body).unwrap();
            }
        });
        let dir = tempfile::tempdir().unwrap();
        let store = osmo::Store::open(dir.path());
        let http = reqwest::Client::new();
        let empty = HashMap::new();
        let ctx = VerifyContext::with_overrides(
            &http,
            store,
            &empty,
            Language::English,
            0.3,
            Some("test".into()),
        )
        .unwrap();
        let cached = b"cached without decoding".to_vec();
        let response: ModalResponse = serde_json::from_value(serde_json::json!({
            "phonemes": [], "frame_matrix": batch_test_payload(2), "deploy_marker": "test"
        }))
        .unwrap();
        cache_response(&ctx, xxh3_64(&cached), raw_prediction(response))
            .await
            .unwrap();
        // An early decode failure must be compacted out before chunking;
        // it must not shrink the first otherwise-full request to 63 items.
        let mut wavs = vec![cached, b"invalid WAV".to_vec()];
        wavs.extend((0..129).map(batch_test_wav));
        // Discovery treats corrupt artifacts as misses and replaces them on a
        // successful inference, rather than leaving a permanent failed slot.
        ctx.store
            .write(&ctx.response_key(xxh3_64(&wavs[2])), b"corrupt")
            .await
            .unwrap();
        let refs: Vec<_> = wavs.iter().map(Vec::as_slice).collect();
        let client = PhonemizerClient::with_endpoints(http.clone(), &url, &url);
        let results = frame_matrices_at(&ctx, &refs, client, false).await;
        server.join().unwrap();
        assert_eq!(results.len(), 131);
        assert_eq!(results[0].as_ref().unwrap().frames, 2);
        let expected_errors: Vec<_> = [1, 3, 4, 5, 6].into_iter().chain(66..130).collect();
        assert_eq!(
            results
                .iter()
                .enumerate()
                .filter_map(|(i, r)| r.is_err().then_some(i))
                .collect::<Vec<_>>(),
            expected_errors
        );
        assert_eq!(results[130].as_ref().unwrap().frames, 1);
        // No endpoint required for cache hits; cache-only misses never make HTTP calls.
        let cached =
            frame_matrices_at(&ctx, &refs, Err(anyhow::anyhow!("no endpoint")), true).await;
        assert_eq!(cached.iter().filter(|r| r.is_ok()).count(), 62);
        let hash = xxh3_64(&wavs[2]);
        assert!(ctx.store.read(&ctx.response_key(hash)).await.is_some());
    }

    /// Readings flattened to phoneme sequences, for tests about which
    /// sequences are accepted rather than where their words fall.
    fn flat_variants(
        text: &str,
        wp: &HashMap<String, language_utils::Pronunciations>,
        language: Language,
    ) -> Option<Vec<Vec<Phoneme>>> {
        ground_truth_phoneme_variants(text, wp, language)
            .map(|readings| readings.iter().map(|r| comparable(r, language)).collect())
    }

    fn word(phonemes: &[&str]) -> Vec<Phoneme> {
        phonemes.iter().map(|p| p.parse().unwrap()).collect()
    }

    #[test]
    fn readings_keep_word_boundaries() {
        let mut wp = HashMap::new();
        wp.insert("bonjour".to_string(), ap("b ɔ̃ ʒ u ʁ", &[]));
        wp.insert("madame".to_string(), ap("m a d a m", &[]));
        let readings =
            ground_truth_phoneme_variants("Bonjour madame", &wp, Language::ChineseTraditional)
                .unwrap();
        assert_eq!(readings.len(), 1);
        assert_eq!(readings[0].word_spans, vec![(0, 5), (5, 10)]);
        assert_eq!(
            readings[0].phonemes,
            word(&["b", "ɔ̃", "ʒ", "u", "ʁ", "m", "a", "d", "a", "m"])
        );
    }

    fn ap(main: &str, others: &[&str]) -> language_utils::Pronunciations {
        language_utils::Pronunciations {
            main: main.to_string(),
            others: others.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn ground_truth_concatenates_per_word_single_variant() {
        let mut wp = HashMap::new();
        wp.insert("bonjour".to_string(), ap("b ɔ̃ ʒ u ʁ", &[]));
        wp.insert("madame".to_string(), ap("m a d a m", &[]));
        // Use Traditional Mandarin, which has no validated g2p source, so
        // the test isolates the wikipron-only path; the espeak path is
        // covered by `ground_truth_includes_espeak_variant_for_supported_languages`.
        let variants =
            flat_variants("Bonjour, madame!", &wp, Language::ChineseTraditional).unwrap();
        assert_eq!(variants.len(), 1);
        assert_eq!(
            variants[0],
            word(&["b", "ɔ̃", "ʒ", "u", "ʁ", "m", "a", "d", "a", "m"])
        );
    }

    #[test]
    fn ground_truth_enumerates_per_word_variants() {
        // `mes` has both /me/ and /mɛ/ in wikipron — phrase "mes" alone
        // should produce both candidate sequences. Use Traditional Mandarin to skip
        // the espeak addition (which would inject a third candidate).
        let mut wp = HashMap::new();
        wp.insert("mes".to_string(), ap("m e", &["m ɛ"]));
        let variants = flat_variants("mes", &wp, Language::ChineseTraditional).unwrap();
        assert_eq!(variants.len(), 2);
        assert!(variants.contains(&word(&["m", "e"])));
        assert!(variants.contains(&word(&["m", "ɛ"])));
    }

    #[test]
    fn ground_truth_cross_product_across_words() {
        let mut wp = HashMap::new();
        wp.insert("mes".to_string(), ap("m e", &["m ɛ"]));
        wp.insert("amis".to_string(), ap("a m i", &["a m i z"]));
        let variants = flat_variants("mes amis", &wp, Language::ChineseTraditional).unwrap();
        // 2 × 2 = 4 phrase candidates
        assert_eq!(variants.len(), 4);
    }

    #[test]
    fn ground_truth_returns_none_on_missing_word() {
        let mut wp = HashMap::new();
        wp.insert("bonjour".to_string(), ap("b ɔ̃ ʒ u ʁ", &[]));
        assert!(flat_variants("bonjour madame", &wp, Language::ChineseTraditional).is_none());
    }

    // One word wikipron lacks must not cost the phrase its wikipron variants
    // for the other words: g2p names just that word.
    #[test]
    fn ground_truth_fills_missing_words_from_g2p() {
        let mut wp = HashMap::new();
        wp.insert("cognac".to_string(), ap("k ɔ ɲ a k", &["k o ɲ a k"]));
        let variants = flat_variants("Cyrano cognac", &wp, Language::French).unwrap();
        let cyrano = ["s", "i", "ʁ", "a", "n", "o"].map(|s| s.parse::<Phoneme>().unwrap());
        for accepted in [
            ["k", "ɔ", "ɲ", "a", "k"].map(|s| s.parse::<Phoneme>().unwrap()),
            ["k", "o", "ɲ", "a", "k"].map(|s| s.parse::<Phoneme>().unwrap()),
        ] {
            let expected: Vec<Phoneme> = cyrano.iter().chain(accepted.iter()).cloned().collect();
            assert!(
                variants.contains(&expected),
                "missing {expected:?} in {variants:?}"
            );
        }
    }

    #[test]
    fn ground_truth_caps_combinations_preserving_earlier_alternates() {
        // Traditional Mandarin isolates the dictionary path. 2^100 would overflow usize;
        // both phrases must instead keep the same bounded prefix choices.
        let wp = HashMap::from([("a".to_string(), ap("ʘ", &["ǀ"]))]);
        for count in [5, 100] {
            let text = vec!["a"; count].join(" ");
            let variants = flat_variants(&text, &wp, Language::ChineseTraditional).unwrap();
            assert_eq!(variants.len(), MAX_VARIANT_COMBINATIONS);
            for (index, variant) in variants.iter().enumerate() {
                let expected: Vec<&str> = (0..count)
                    .map(|position| {
                        if position < 4 && index & (1 << position) != 0 {
                            "ǀ"
                        } else {
                            "ʘ"
                        }
                    })
                    .collect();
                assert_eq!(*variant, word(&expected));
            }
            assert_eq!(
                variants,
                flat_variants(&text, &wp, Language::ChineseTraditional).unwrap()
            );
        }
    }

    #[test]
    fn portuguese_cerveja_cue_keeps_letter_name_alternates() {
        // Actual out/por/word_to_pronunciation.jsonl entries. The raw cue
        // has 18 combinations and used to lose both /e/ and /ɛ/ to the cap.
        let wp = HashMap::from([
            ("e".to_string(), ap("i", &["e", "ɛ"])),
            ("é".to_string(), ap("ɛ", &[])),
            ("como".to_string(), ap("k o m u", &["k u m u"])),
            ("em".to_string(), ap("ɐ̃ j̃", &[])),
            (
                "cerveja".to_string(),
                ap("s ɨ ɾ v e ʒ ɐ", &["s ɨ ɾ b e ʒ ɐ", "s ɨ ɾ v ɐ j ʒ ɐ"]),
            ),
        ]);
        let spoken = language_utils::pronunciation_challenge_spoken_text(
            Language::Portuguese,
            "ce",
            "cerveja",
        );
        assert_eq!(spoken, "c é como em cerveja");
        for (text, count) in [(spoken.as_str(), 6), ("c e como em cerveja", 16)] {
            let readings = ground_truth_phoneme_variants(text, &wp, Language::Portuguese).unwrap();
            assert_eq!(readings.len(), count + 1); // phrase g2p is outside the cap
            let expected = word(&[
                "s", "e", "ɛ", "k", "o", "m", "u", "ɐ̃", "j̃", "s", "ɨ", "ɾ", "v", "e", "ʒ", "ɐ",
            ]);
            assert!(
                readings
                    .iter()
                    .any(|r| comparable(r, Language::Portuguese) == expected)
            );
            assert!(readings.iter().all(|r| r.word_spans.len() == 5));
            let phrase = model_target(text, Language::Portuguese).unwrap().unwrap();
            assert!(
                readings
                    .iter()
                    .any(|r| comparable(r, Language::Portuguese) == phrase.phonemes)
            );
        }
    }

    #[test]
    fn phoneme_label_source_mirror_matches_g2p() {
        // Generate the list and an exhaustive identity match together: adding a
        // Language variant must fail to compile rather than escape this check.
        macro_rules! languages {
            ($($language:ident),* $(,)?) => {
                [$(Language::$language),*].map(|language| match language {
                    $(Language::$language => Language::$language),*
                })
            };
        }
        for language in languages![
            French,
            English,
            Spanish,
            Korean,
            German,
            ChineseSimplified,
            ChineseTraditional,
            Japanese,
            Russian,
            Portuguese,
            Italian,
            Hindi,
            Thai,
        ] {
            let mirror = language.phoneme_label_source();
            let lang = language.g2p_lang();
            assert_eq!(
                mirror == PhonemeLabelSource::Unvalidated,
                lang.is_none(),
                "{language:?}: label-source mirror/support gate drifted from g2p; \
                 fix Language::phoneme_label_source and Language::g2p_lang together"
            );
            let expected = match mirror {
                PhonemeLabelSource::Espeak(_) => Some(g2p::LabelSource::Espeak),
                PhonemeLabelSource::Hindi => Some(g2p::LabelSource::Hindi),
                PhonemeLabelSource::Mandarin => Some(g2p::LabelSource::Mandarin),
                PhonemeLabelSource::Japanese => Some(g2p::LabelSource::Japanese),
                PhonemeLabelSource::Korean => Some(g2p::LabelSource::Korean),
                PhonemeLabelSource::Thai => Some(g2p::LabelSource::Thai),
                PhonemeLabelSource::Unvalidated => None,
            };
            assert_eq!(
                expected,
                g2p::label_source(language.code()),
                "{language:?}: label-source mirror drifted from g2p::label_source; \
                 fix Language::phoneme_label_source to match the pinned g2p table"
            );
        }
    }

    #[test]
    fn spanish_variety_labels_match_pre_upgrade_fixture() {
        // Captured from g2p 3ae99aa's phonemize(text, "es-419") before the API bump.
        for (text, expected) in [
            ("cinco", vec!["s", "i", "n", "k", "o"]),
            ("caza", vec!["k", "a", "s", "a"]),
            (
                "Gracias por la cerveza.",
                vec![
                    "ɡ", "ɾ", "a", "s", "j", "a", "s", "p", "o", "ɾ", "l", "a", "s", "e", "ɾ", "β",
                    "e", "s", "a",
                ],
            ),
        ] {
            assert_eq!(
                spanish_dialect_target(text, Language::Spanish)
                    .unwrap()
                    .unwrap()
                    .phonemes,
                word(&expected)
            );
        }
    }

    #[test]
    fn spanish_accepts_seseo_without_changing_training_labels() {
        assert!(matches!(
            Language::Spanish.phoneme_label_source(),
            PhonemeLabelSource::Espeak("es")
        ));
        assert_eq!(
            model_target("cinco", Language::Spanish)
                .unwrap()
                .unwrap()
                .phonemes,
            word(&["θ", "i", "n", "k", "o"])
        );
        assert_eq!(
            normalize_phonemes(Phoneme::Theta, Language::Spanish),
            word(&["θ"])
        );
        assert_eq!(
            normalize_phonemes(Phoneme::S, Language::Spanish),
            word(&["s"])
        );
        let empty = HashMap::new();
        let variants = flat_variants("cinco", &empty, Language::Spanish).unwrap();
        assert_eq!(
            variants,
            vec![
                word(&["θ", "i", "n", "k", "o"]),
                word(&["s", "i", "n", "k", "o"])
            ]
        );
        // Latin American per-word g2p also combines with dictionary-only
        // pronunciations, not merely a whole-phrase fallback.
        let mut wp = HashMap::from([("nombre".to_string(), ap("ʘ", &[]))]);
        for dictionary_cinco in [false, true] {
            if dictionary_cinco {
                wp.insert("cinco".to_string(), ap("θ i n k o", &[]));
            }
            let readings =
                ground_truth_phoneme_variants("cinco nombre", &wp, Language::Spanish).unwrap();
            assert!(
                readings
                    .iter()
                    .any(|r| r.phonemes == word(&["s", "i", "n", "k", "o", "ʘ"])
                        && r.word_spans == [(0, 5), (5, 6)])
            );
        }
        // The cap must not prevent a pure seseo phrase reading when later
        // words' per-word alternates are truncated.
        let text = ["cinco"; 6].join(" ");
        let readings = ground_truth_phoneme_variants(&text, &empty, Language::Spanish).unwrap();
        assert_eq!(readings.len(), MAX_VARIANT_COMBINATIONS + 1);
        assert!(readings.iter().any(|r| r.phonemes
            == vec![word(&["s", "i", "n", "k", "o"]); 6].concat()
            && r.word_spans.len() == 6));
        assert_eq!(
            flat_variants("niño", &empty, Language::Spanish)
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn ground_truth_preserves_hindi_combining_signs() {
        for text in ["क्", "क़", "क्ष", "हिंदी", "क्\u{200d}ष"] {
            // A dictionary sentinel proves lookup sees the entire grapheme,
            // while an empty dictionary exercises g2p on the intact word.
            let wp = HashMap::from([(text.to_string(), ap("ʘ", &[]))]);
            let readings =
                ground_truth_phoneme_variants(&format!("‘{text}।’"), &wp, Language::Hindi).unwrap();
            assert_eq!(readings[0].phonemes, word(&["ʘ"]));
            let expected: Vec<Phoneme> = model_target(text, Language::Hindi)
                .unwrap()
                .unwrap()
                .phonemes
                .iter()
                .flat_map(|p| normalize_phonemes(*p, Language::Hindi))
                .collect();
            let readings =
                ground_truth_phoneme_variants(text, &HashMap::new(), Language::Hindi).unwrap();
            assert_eq!(
                readings
                    .iter()
                    .map(|r| comparable(r, Language::Hindi))
                    .collect::<Vec<_>>(),
                vec![expected]
            );
        }
    }

    // The liaison output depends on our espeak-ng fork's French patches;
    // g2p embeds that build, so this runs everywhere, CI included.
    #[test]
    fn ground_truth_includes_espeak_variant_for_supported_languages() {
        // For French (espeak-supported), the candidate list should include
        // an espeak-derived variant in addition to the wikipron cross-product.
        // `on est` is the motivating case — espeak produces /ɔ̃ n ɛ/
        // (nasal vowel preserved + liaison /n/), which neither of the
        // wikipron per-word combinations captures.
        let mut wp = HashMap::new();
        wp.insert("on".to_string(), ap("ɔ̃", &["ɔ . n ‿"]));
        wp.insert("est".to_string(), ap("ɛ", &["e"]));
        let variants = flat_variants("on est", &wp, Language::French).unwrap();
        // Must include the espeak liaison candidate that the per-word
        // cross-product can't reach.
        assert!(
            variants.contains(&word(&["ɔ̃", "n", "ɛ"])),
            "expected espeak liaison candidate /ɔ̃ n ɛ/ in candidates: {variants:?}"
        );
    }

    // The pronunciation-challenge transcript spells the pattern with letter
    // names; espeak must phonemize those names as the voice says them, not
    // as the words the bare letters would be ("y" the adverb, "à" the
    // preposition).
    #[test]
    fn french_letter_names_phonemize_as_spoken() {
        let phonemes = |pattern: &str, example: &str| {
            let text = language_utils::pronunciation_challenge_spoken_text(
                Language::French,
                pattern,
                example,
            );
            model_target(&text, Language::French)
                .unwrap()
                .unwrap()
                .phonemes
                .iter()
                .flat_map(|p| normalize_phonemes(*p, Language::French))
                .collect::<Vec<_>>()
                .join(" ")
        };
        assert_eq!(phonemes("y", "pays"), "i ɡ ʁ ɛ k k ɔ m d ɑ̃ p ɛ i");
        assert_eq!(
            phonemes("à", "voilà"),
            "a a k s ɑ̃ ɡ ʁ a v k ɔ m d ɑ̃ v w a l a"
        );
        assert_eq!(
            phonemes("ô", "côte"),
            "o a k s ɑ̃ s i ʁ k ɔ̃ f l ɛ k s k ɔ m d ɑ̃ k o t"
        );
        assert_eq!(phonemes("cy", "cycle"), "s e i ɡ ʁ ɛ k k ɔ m d ɑ̃ s i k l");
    }

    #[test]
    fn hindi_targets_use_g2p_training_labels() {
        let target = model_target("यह शहर ज्ञान", Language::Hindi)
            .expect("Hindi is g2p-labeled")
            .expect("hindi chain runs");
        assert_eq!(
            target.phonemes,
            word(&["j", "eː", "ʃ", "ɛː", "ɦ", "ɛː", "ɾ", "ɡ", "j", "aː", "n"])
        );
        assert_eq!(target.word_spans, [(0, 2), (2, 7), (7, 11)]);
        // Refuse targets with holes where the recording still contains speech.
        for text in ["19 वीं", "AOL अपनी"] {
            assert!(model_target(text, Language::Hindi).unwrap().is_err());
        }
        assert!(model_target("國家", Language::ChineseTraditional).is_none());
    }

    #[test]
    fn korean_verification_accepts_the_g2p_label_source() {
        assert_eq!(Language::Korean.g2p_lang(), Some("kor"));
        assert_eq!(
            Language::Korean.phoneme_label_source(),
            PhonemeLabelSource::Korean
        );
        let dir = tempfile::tempdir().unwrap();
        let http = reqwest::Client::new();
        let words = HashMap::new();
        VerifyContext::with_overrides(
            &http,
            osmo::Store::open(dir.path()),
            &words,
            Language::Korean,
            0.3,
            None,
        )
        .expect("Korean labels are supported without probing the endpoint");
    }

    #[test]
    fn japanese_targets_come_from_jpreprocess() {
        let target = model_target("学校", Language::Japanese)
            .expect("Japanese is g2p-labeled")
            .expect("japanese chain runs");
        // Sokuon becomes length on the following obstruent, not a token.
        assert_eq!(target.phonemes, word(&["ɡ", "a", "kː", "o", "o"]));
        assert!(target.pitch.iter().flatten().count() > 0);
    }

    #[test]
    fn mandarin_targets_come_from_the_g2pm_port() {
        let target = model_target("你好", Language::ChineseSimplified)
            .expect("Mandarin is g2p-labeled")
            .expect("mandarin chain runs");
        assert_eq!(target.phonemes, word(&["n", "i", "x", "au̯"]));
        assert_eq!(target.tone, [None, Some(3), None, Some(3)]);
        // Digits would be spoken but unlabeled: refused, not silently cut.
        assert!(matches!(
            model_target("我有2个", Language::ChineseSimplified),
            Some(Err(g2p::Error::Unlabelable(_)))
        ));
    }
}

#[cfg(test)]
mod letter_name_tests {
    use super::*;

    fn phonemes(language: Language, pattern: &str, example: &str) -> String {
        let text = language_utils::pronunciation_challenge_spoken_text(language, pattern, example);
        model_target(&text, language)
            .unwrap()
            .unwrap()
            .phonemes
            .iter()
            .flat_map(|p| normalize_phonemes(*p, language))
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn japanese_and_hindi_segments_use_the_models_backend() {
        for (language, pattern, expected) in [
            (Language::Japanese, "は", "h a"),
            (Language::Japanese, "へ", "h e"),
            (Language::Japanese, "ー", "tɕ o o o ɴ p ɯᵝ"),
            (Language::Japanese, "っ", "tɕ i i s a i ts ɯᵝ"),
            (Language::Japanese, "ッ", "tɕ i i s a i ts ɯᵝ"),
            (Language::Hindi, "़", "n ʊ k t̪ a"),
            (Language::Hindi, "्", "ɦ ə l ə n t̪"),
            (Language::Hindi, "ड़", "ɽ ə"),
            (Language::Hindi, "क्ष", "k ʃ"),
        ] {
            let segments = language_utils::pronunciation_challenge_segments(language, pattern, "");
            assert_eq!(segments[0].display, pattern);
            let target = model_target(&segments[0].spoken, language)
                .unwrap()
                .unwrap();
            let phones = target
                .phonemes
                .iter()
                .flat_map(|p| normalize_phonemes(*p, language))
                .collect::<Vec<_>>()
                .join(" ");
            assert_eq!(phones, expected, "{language:?} {pattern}");
        }
    }

    // The letter-name table is only useful if espeak reads each name as the
    // voice will say it: a name that phonemizes as something else would
    // fail every clip of that letter however well it was spoken.
    #[test]
    fn letter_names_phonemize_as_spoken() {
        assert_eq!(
            phonemes(Language::German, "ü", "über"),
            "u ʊ m l aʊ t v i ɪ n y b ɜ"
        );
        assert_eq!(
            phonemes(Language::German, "ß", "Straße"),
            "ɛ s ts ɛ t v i ɪ n ʃ t ɾ ɑ s ə"
        );
        assert_eq!(
            phonemes(Language::Spanish, "ñ", "niño"),
            "e ɲ e k o m o e n n i ɲ o"
        );
        assert_eq!(
            phonemes(Language::Portuguese, "ã", "pão"),
            "a tʃ i ʊ k o m w e\u{303} j p ɐ\u{303}ʊ\u{303}"
        );
        assert_eq!(
            phonemes(Language::Russian, "щ", "борщ"),
            "ɕ ɑ k ɑ k v b o r ɕ"
        );
        // Bare letters that espeak would read as words: Portuguese "e" is
        // the conjunction /i/ and "o" the article /u/; a bare Russian "о"
        // reduces to /ʌ/. Named, they are the letters.
        assert_eq!(
            phonemes(Language::Portuguese, "e", "cerveja"),
            "ɛ k o m w e\u{303} j s e ɾ v e ʒ ɐ"
        );
        assert_eq!(
            phonemes(Language::Portuguese, "o", "ovo"),
            "ɔ k o m w e\u{303} j o v ʊ"
        );
        assert_eq!(
            phonemes(Language::Russian, "о", "окно"),
            "o k ɑ k v ʌ k n o"
        );
        // The `^` espeak leaves on царь is stripped, not scored.
        assert_eq!(
            phonemes(Language::Russian, "ц", "царь"),
            "ts ɛ k ɑ k f ts ɑ r ɪ"
        );
        assert_eq!(
            phonemes(Language::Russian, "ь", "соль"),
            "mʲ ɑ x kʲ i j z n ɑ k k ɑ k f s o ɫ"
        );
        assert_eq!(
            phonemes(Language::French, "ç", "garçon"),
            "s e s e d i j k ɔ m d ɑ\u{303} ɡ a ʁ s ɔ\u{303}"
        );
        // A lone sokuon has no reading at all; its name does.
        assert_eq!(
            phonemes(Language::Japanese, "っ", "カップ"),
            "tɕ i i s a i ts ɯᵝ n o j o o n i k a p ɯᵝ"
        );
    }
}
