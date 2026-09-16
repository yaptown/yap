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
use language_utils::{Language, PhonemeLabelSource};
use lexide::pronunciation::{
    DECODER_VERSION, PhonemeAlternative as RawPhonemeAlt, PredictRequest,
    PredictResponse as ModalResponse, cache_version, remote::PhonemizerClient,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{LazyLock, OnceLock};
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

/// The live identity is discovered once; it does not partition production data.
static PRODUCTION_MODEL: OnceLock<Result<ModelIdentity, String>> = OnceLock::new();

fn resolve_model(probe: impl FnOnce() -> Result<ModelIdentity>) -> Result<ModelIdentity> {
    let identity = probe()?;
    anyhow::ensure!(
        !identity.model_id.trim().is_empty() && !identity.model_revision.trim().is_empty(),
        "phonemizer probe returned empty model identity"
    );
    check_decoder(identity.decoder_version.as_deref())?;
    Ok(identity)
}

fn resolved_model_once(
    cell: &OnceLock<Result<ModelIdentity, String>>,
    initialize: impl FnOnce() -> Result<ModelIdentity>,
) -> Result<&ModelIdentity> {
    cell.get_or_init(|| {
        initialize().map_err(|error| format!("resolving phonemizer identity: {error:#}"))
    })
    .as_ref()
    .map_err(|error| anyhow::anyhow!("{error}"))
}

fn production_model() -> Result<&'static ModelIdentity> {
    resolved_model_once(&PRODUCTION_MODEL, || resolve_model(probe_identity))
}

fn probe_identity() -> Result<ModelIdentity> {
    // Constructors are synchronous and may run inside a current-thread Tokio
    // runtime. This thread owns and drops its runtime and HTTP client; never
    // block_on a caller's runtime.
    std::thread::spawn(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
            .block_on(async {
                let http = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(300))
                    .build()?;
                let client = wav2vec2::identity_client(http)?;
                match expected_deploy_marker() {
                    Some(marker) => client.check_identity(&marker).await,
                    None => client.identity().await,
                }
            })
    })
    .join()
    .map_err(|_| anyhow::anyhow!("phonemizer identity probe thread panicked"))?
}

/// Identity of the target renderer. Persist alongside model identity when caching scores.
pub fn model_target_identity() -> String {
    g2p::identity()
}

/// The scoring target for `text` in `language`, using g2p's training-label
/// source. `None` for languages without validated model labels (see
/// `Language::g2p_lang`).
pub fn model_target(text: &str, language: Language) -> Option<Result<g2p::Phonemized, g2p::Error>> {
    // TODO(g2p): expose a disable-language-switch option, then use it here.
    // French words such as polyuréthane and Giverny can currently switch to English.
    let lang = language.g2p_lang()?;
    Some(g2p::phonemize_lang(lang, text))
}

/// Full renderer output and the renderer that actually produced it. The key
/// uses literal G2P inputs plus clip identity, never a renderer version.
#[derive(Serialize, Deserialize)]
pub struct CachedTarget {
    pub phonemized: g2p::Phonemized,
    pub renderer: String,
}

pub async fn cached_model_target(
    store: &osmo::Store,
    language: Language,
    text: &str,
    clip_hash: u64,
    refresh: bool,
) -> Result<CachedTarget> {
    let lang = language.g2p_lang().context("no model label source")?;
    let inputs = serde_json::to_vec(&(lang, text, clip_hash))?;
    let key = format!("phoneme-target/{:016x}", xxh3_64(&inputs));
    if !refresh
        && let Some(bytes) = store.read(&key).await
        && let Ok(target) = serde_json::from_slice::<CachedTarget>(&bytes)
    {
        return Ok(target);
    }
    let target = CachedTarget {
        phonemized: g2p::phonemize_lang(lang, text)?,
        renderer: model_target_identity(),
    };
    store.write(&key, &serde_json::to_vec(&target)?).await?;
    Ok(target)
}

/// One artifact for every inference consumer. The original WAV hash belongs
/// in the key (with exact target labels for corpus clips), not in this value.
/// TODO(raw-response): temporary lossy retention until lexide publishes its raw
/// client. Replace `response` here with untouched item + all envelope RawValues;
/// typed serialization currently discards unknown wire fields. Do not run a
/// corpus rewrite against this transitional representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedResponse {
    response: ModalResponse,
}

fn response_identity(response: &ModalResponse) -> Option<ModelIdentity> {
    if let Some(FrameMatrixPayload::V1(payload)) = &response.frame_matrix {
        let producer = &payload.producer;
        return Some(ModelIdentity {
            model_id: producer.model_id.clone(),
            model_revision: producer.model_revision.clone(),
            decoder_version: Some(producer.decoder_version.clone()),
            deploy_marker: Some(producer.deploy_marker.clone()),
        });
    }
    Some(ModelIdentity {
        model_id: response.model_id.clone()?,
        model_revision: response.model_revision.clone()?,
        decoder_version: response.decoder_version.clone(),
        deploy_marker: response.deploy_marker.clone(),
    })
}

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

fn response_frames(response: &ModalResponse) -> Result<FrameMatrix> {
    FrameMatrix::decode(
        response
            .frame_matrix
            .as_ref()
            .context("response has no frame matrix")?,
    )
}

/// One step of the optimal alignment between predicted and expected phoneme
/// sequences. Read in order, the ops reconstruct both sequences and show
/// exactly where they disagree.
///
/// `Sub`/`Extra` carry probabilities sourced from the model's top-k at the
/// predicted position. `expected_prob` is `None` when the expected phoneme
/// wasn't in the model's top-k for that position — i.e. the model
/// effectively assigned zero probability to the correct answer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum AlignmentOp {
    /// Same phoneme on both sides.
    Match { phoneme: String, probability: f64 },
    /// Different phoneme — predicted has one thing, expected has another.
    Sub {
        expected: String,
        predicted: String,
        predicted_prob: f64,
        expected_prob: Option<f64>,
    },
    /// Predicted has a phoneme the expected sequence doesn't.
    Extra {
        predicted: String,
        predicted_prob: f64,
    },
    /// Expected has a phoneme the model didn't output. We have no model
    /// position for this gap, so no probability is available.
    Missing { expected: String },
}

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
    pub predicted_normalized: Vec<String>,
    /// Ground-truth phonemes assembled from `word_to_pronunciation`. `None`
    /// when some word in `text` isn't in our pronunciation map. When a word
    /// has alternate accepted pronunciations, this is the variant that
    /// matched the model's prediction *most closely* — we pass if any
    /// variant is within threshold, and report the closest one for the
    /// alignment.
    pub expected: Option<Vec<String>>,
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
    /// A caller may key a clip structurally instead of by WAV hash. Holding a
    /// producer constant while varying content calls for content keys (corpus:
    /// WAV hash plus labels). The minority deliberately varying producers (model
    /// comparisons/evaluations) must include full model identity plus audio.
    cache_key: Option<std::sync::Arc<dyn Fn(u64) -> String + Send + Sync>>,
    expected_identity: Option<ModelIdentity>,
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
    /// When set (`WAV2VEC2_EXPECTED_DEPLOY_MARKER`, by the eval harness), every
    /// endpoint response must report this exact deploy marker or we bail rather
    /// than cache a possibly-contaminated prediction. `None` in production.
    expected_deploy_marker: Option<String>,
}

impl<'a> VerifyContext<'a> {
    /// Supply a per-clip complete cache-key builder. Default keys use WAV hash;
    /// corpus callers add exact labels, model-varying evaluations use key_by_model.
    pub fn with_cache_key(mut self, key: impl Fn(u64) -> String + Send + Sync + 'static) -> Self {
        self.cache_key = Some(std::sync::Arc::new(key));
        self
    }

    fn response_key(&self, hash: u64) -> String {
        self.cache_key
            .as_ref()
            .map_or_else(|| format!("phoneme-response/{hash:016x}"), |key| key(hash))
    }

    /// Model-varying evaluations are deliberately not production content keys.
    /// A deployment marker validates live responses, not checkpoint content.
    pub fn key_by_model(&mut self, identity: &ModelIdentity) {
        self.expected_identity = Some(identity.clone());
        let model = serde_json::to_string(&(
            &identity.model_id,
            &identity.model_revision,
            &identity.decoder_version,
        ))
        .expect("model identity is serializable");
        self.cache_key = Some(std::sync::Arc::new(move |hash| {
            format!(
                "phoneme-response/model/{:016x}/{hash:016x}",
                xxh3_64(model.as_bytes())
            )
        }));
    }

    /// Identity enforced on live responses, not attributed to cached outputs.
    pub fn expected_model_identity(&self) -> Result<&ModelIdentity> {
        self.expected_identity.as_ref().context("resolved model identity required; use VerifyContext::new or an explicitly resolved evaluation identity")
    }

    /// Production constructor: discover and enforce the live model identity.
    /// Cache hits retain their own producer and do not trigger freshness checks.
    pub fn new(
        http: &'a reqwest::Client,
        store: osmo::Store,
        word_to_pronunciation: &'a HashMap<String, language_utils::Pronunciations>,
        target_language: Language,
    ) -> Result<Self> {
        let model = production_model()?;
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
        ctx.expected_identity = Some(model.clone());
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
        Ok(Self {
            http,
            store,
            cache_key: None,
            expected_identity: None,
            word_to_pronunciation,
            mismatch_threshold,
            target_language,
            expected_deploy_marker,
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
) -> Option<Vec<Reading>> {
    if let Some(s) = override_transcription {
        let normalized: Vec<String> = s
            .split_whitespace()
            .flat_map(|t| normalize_phonemes(t, ctx.target_language))
            .collect();
        return if normalized.is_empty() {
            None
        } else {
            Some(vec![vec![normalized]])
        };
    }
    ground_truth_phoneme_variants(text, ctx.word_to_pronunciation, ctx.target_language)
}

/// One accepted reading of a phrase: its words in order, each a phoneme
/// sequence. The boundaries let the verifier tell a word the model never
/// heard from a phrase that merely drifted; see [`unheard_word`].
pub type Reading = Vec<Vec<String>>;

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
    expected: Option<Vec<Reading>>,
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
    expected: Option<Vec<Reading>>,
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
    let raw_top_k = response
        .phonemes
        .iter()
        .map(|p| p.top_k.clone())
        .collect::<Vec<_>>();
    let (predicted_normalized, predicted_top_k) =
        normalize_with_topk(&raw_phonemes, &raw_top_k, ctx.target_language);

    // Caller supplies the accepted phoneme sequence(s) — see
    // [`expected_phoneme_variants`] for how to build them from wikipron +
    // espeak (or a per-clip override).
    let variant_candidates = expected;

    let (
        expected,
        variants_considered,
        edit_distance,
        edit_distance_pct,
        alignment,
        failure_reason,
    ) = match variant_candidates {
        // No ground truth available — both wikipron and espeak came up
        // empty. Reject the clip rather than silently passing: shipping
        // unverified audio is worse than dropping a clip we can fix by
        // adding a wikipron entry or extending espeak coverage.
        None => (
            None,
            0,
            None,
            None,
            None,
            Some(
                "no ground truth available (word missing from wikipron and espeak unsupported \
                 for this language)"
                    .to_string(),
            ),
        ),
        Some(variants) => {
            let n_variants = variants.len();
            // Score each variant by edit distance against the prediction;
            // pick the closest one. Ties broken by variant order (main first).
            let mut best: Option<(usize, Reading, Vec<AlignmentOp>)> = None;
            for reading in variants {
                let (dist, ops) = align(&predicted_normalized, &predicted_top_k, &reading.concat());
                if best.as_ref().is_none_or(|(d, _, _)| dist < *d) {
                    best = Some((dist, reading, ops));
                }
            }
            let (dist, reading, ops) = best.unwrap();
            let exp = reading.concat();
            let max_len = predicted_normalized.len().max(exp.len()).max(1);
            let pct = dist as f64 / max_len as f64;
            let reason = if predicted_normalized.is_empty() {
                Some("model returned no phonemes".to_string())
            } else if let Some(word) = unheard_word(&reading, &ops) {
                Some(format!(
                    "word /{}/ not heard ({dist} edits over max-len {max_len} = {:.0}%)",
                    word.join(" "),
                    pct * 100.0
                ))
            } else if pct > ctx.mismatch_threshold {
                Some(format!(
                    "phoneme mismatch ({dist} edits over max-len {max_len} = {:.0}%, \
                         best of {n_variants} variant(s))",
                    pct * 100.0
                ))
            } else {
                None
            };
            (
                Some(exp),
                n_variants,
                Some(dist),
                Some(pct),
                Some(ops),
                reason,
            )
        }
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
const MODAL_TOP_K: usize = 10;

/// Clips per HTTP request the endpoint accepts, not its GPU microbatch size.
pub const MODAL_BATCH_SIZE: usize = 64;

/// How long the batch worker waits after the first queued clip for the
/// other in-flight callers to enqueue theirs, so a batch carries the whole
/// concurrent front rather than one clip.
const MODAL_BATCH_LINGER: std::time::Duration = std::time::Duration::from_millis(200);

struct BatchItem {
    payload: PredictRequest,
    reply: tokio::sync::oneshot::Sender<Result<ModalResponse>>,
}

/// The process-wide queue feeding the batch worker. Spawned on first use,
/// which is always inside the caller's tokio runtime.
static BATCH_QUEUE: LazyLock<tokio::sync::mpsc::UnboundedSender<BatchItem>> = LazyLock::new(|| {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(batch_worker(rx));
    tx
});

/// Drain the queue into requests of up to [`MODAL_BATCH_SIZE`] clips and
/// hand each caller its own result. The endpoint groups similar lengths
/// into GPU batches itself; this only pools what concurrent callers submit.
async fn batch_worker(mut rx: tokio::sync::mpsc::UnboundedReceiver<BatchItem>) {
    let client = wav2vec2::batch_client(reqwest::Client::new());
    while let Some(first) = rx.recv().await {
        let mut batch = vec![first];
        tokio::time::sleep(MODAL_BATCH_LINGER).await;
        while batch.len() < MODAL_BATCH_SIZE {
            match rx.try_recv() {
                Ok(item) => batch.push(item),
                Err(_) => break,
            }
        }
        let payloads: Vec<PredictRequest> = batch
            .iter_mut()
            .map(|item| std::mem::take(&mut item.payload))
            .collect();
        let results = match &client {
            Ok(client) => wav2vec2::predict_batch(client, &payloads, None).await,
            Err(e) => Err(anyhow::anyhow!("{e:#}")),
        };
        match results {
            Ok(results) => {
                for (item, result) in batch.into_iter().zip(results) {
                    let _ = item.reply.send(result);
                }
            }
            Err(e) => {
                let message = format!("{e:#}");
                for item in batch {
                    let _ = item.reply.send(Err(anyhow::anyhow!("{message}")));
                }
            }
        }
    }
}

/// Send one clip through the batch worker and refuse any response from a
/// container whose deploy marker is not the one expected — so nothing from
/// a stale/contaminated container is ever cached under the wrong model's
/// key.
async fn post_modal(ctx: &VerifyContext<'_>, payload: PredictRequest) -> Result<ModalResponse> {
    let (reply, result) = tokio::sync::oneshot::channel();
    BATCH_QUEUE
        .send(BatchItem { payload, reply })
        .map_err(|_| anyhow::anyhow!("the Modal batch worker is gone"))?;
    let modal = result
        .await
        .context("the Modal batch worker dropped the request")??;
    check_response_identity(ctx, &modal)?;
    Ok(modal)
}

fn check_decoder(decoder: Option<&str>) -> Result<()> {
    if let Some(decoder) = decoder {
        anyhow::ensure!(
            decoder == DECODER_VERSION,
            "decoder mismatch: endpoint reported {decoder:?}, expected {DECODER_VERSION:?}"
        );
    }
    Ok(())
}

/// Never mask an item/envelope disagreement with the other layer's metadata.
/// After merging, checking an item against its context also checks the envelope.
fn merge_metadata(name: &str, item: &mut Option<String>, envelope: &Option<String>) -> Result<()> {
    if let Some(envelope) = envelope {
        if let Some(item) = item.as_ref() {
            anyhow::ensure!(
                item == envelope,
                "batch {name} mismatch: item {item:?}, envelope {envelope:?}"
            );
        } else {
            *item = Some(envelope.clone());
        }
    }
    Ok(())
}

/// Per-response freshness check: the one-shot marker_only probe only proves
/// the *first* request hit a fresh container. Verifying the marker on every
/// response guarantees no later request was routed to a stale/contaminated
/// warm container and silently cached under the wrong model's key.
fn check_response_identity(ctx: &VerifyContext<'_>, modal: &ModalResponse) -> Result<()> {
    // V1's matrix producer is authoritative, but must agree with any envelope
    // metadata too. Never label a matrix using the context's desired producer.
    let observed = response_identity(modal);
    if let Some(observed) = &observed {
        for (name, outer, inner) in [
            (
                "model id",
                modal.model_id.as_deref(),
                Some(observed.model_id.as_str()),
            ),
            (
                "model revision",
                modal.model_revision.as_deref(),
                Some(observed.model_revision.as_str()),
            ),
            (
                "decoder",
                modal.decoder_version.as_deref(),
                observed.decoder_version.as_deref(),
            ),
            (
                "deploy-marker",
                modal.deploy_marker.as_deref(),
                observed.deploy_marker.as_deref(),
            ),
        ] {
            anyhow::ensure!(
                outer.is_none() || inner == outer,
                "response {name} disagrees with matrix producer"
            );
        }
        check_decoder(observed.decoder_version.as_deref())?;
        check_marker(
            ctx.expected_deploy_marker.as_deref(),
            observed.deploy_marker.as_deref(),
        )?;
        if let Some(expected) = &ctx.expected_identity {
            anyhow::ensure!(
                observed.model_id == expected.model_id
                    && observed.model_revision == expected.model_revision,
                "matrix producer does not match expected model identity"
            );
        }
    }
    check_decoder(modal.decoder_version.as_deref())?;
    if let Some(identity) = &ctx.expected_identity {
        for (name, reported, expected) in [
            ("model id", modal.model_id.as_ref(), &identity.model_id),
            (
                "model revision",
                modal.model_revision.as_ref(),
                &identity.model_revision,
            ),
        ] {
            if let Some(reported) = reported {
                anyhow::ensure!(
                    reported == expected,
                    "{name} mismatch: endpoint reported {reported:?}, expected {expected:?} — refusing to cache prediction"
                );
            }
        }
    }
    check_marker(
        ctx.expected_deploy_marker.as_deref(),
        observed
            .as_ref()
            .and_then(|identity| identity.deploy_marker.as_deref())
            .or(modal.deploy_marker.as_deref()),
    )
}

fn check_marker(expected: Option<&str>, reported: Option<&str>) -> Result<()> {
    if let Some(expected) = expected {
        anyhow::ensure!(
            reported == Some(expected),
            "deploy-marker mismatch: endpoint reported {reported:?}, expected {expected:?}"
        );
    }
    Ok(())
}

async fn cached_response(ctx: &VerifyContext<'_>, key: &str) -> Option<Result<ModalResponse>> {
    let bytes = ctx.store.read(key).await?;
    Some(
        serde_json::from_slice::<CachedResponse>(&bytes)
            .context("cached response")
            .map(|cached| cached.response),
    )
}

/// Validate before writing: a malformed response never becomes a reusable hit.
async fn cache_response(
    ctx: &VerifyContext<'_>,
    hash: u64,
    response: ModalResponse,
) -> Result<FrameMatrix> {
    check_response_identity(ctx, &response)?;
    let frames = response_frames(&response)?;
    ctx.store
        .write(
            &ctx.response_key(hash),
            &serde_json::to_vec(&CachedResponse { response })?,
        )
        .await?;
    Ok(frames)
}

/// Cache-only lookup by the caller's complete key, without cutting audio.
/// Missing and malformed values never trigger inference in this reader.
pub async fn cached_frame_matrix(
    ctx: &VerifyContext<'_>,
    key: &str,
) -> Option<Result<FrameMatrix>> {
    cached_response(ctx, key)
        .await
        .map(|response| response.and_then(|r| response_frames(&r)))
}

async fn prediction_response(
    ctx: &VerifyContext<'_>,
    wav: &[u8],
) -> Result<(ModalResponse, FrameMatrix)> {
    let hash = xxh3_64(wav);
    if let Some(Ok(response)) = cached_response(ctx, &ctx.response_key(hash)).await
        && let Ok(frames) = response_frames(&response)
    {
        return Ok((response, frames));
    }
    anyhow::ensure!(
        !cache_only(),
        "response cache miss for {hash:016x}; cache-only mode is enabled"
    );
    let request = prepare_frame_request(wav.to_vec()).await?;
    let response = post_modal(ctx, request.payload).await?;
    let frames = cache_response(ctx, hash, response.clone()).await?;
    Ok((response, frames))
}

/// A decoded, padded request paired with the original WAV's cache identity.
pub struct PreparedFrameRequest {
    hash: u64,
    payload: PredictRequest,
}

/// Prepare audio without inference. Batch callers count only successful
/// preparations toward the endpoint's request limit.
pub async fn prepare_frame_request(wav: Vec<u8>) -> Result<PreparedFrameRequest> {
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

/// Submit one prepared batch, preserving each clip's identity checks and cache
/// destination. Languages may differ: the model hears audio, not the target.
/// Requests must already be cache misses. Results retain input order, and an
/// item failure does not discard its neighbors. Transport/protocol failures
/// affect this request only; callers may continue with later batches.
pub async fn infer_frame_batch(
    items: Vec<(&VerifyContext<'_>, PreparedFrameRequest)>,
    activity: Option<&wav2vec2::RequestActivity>,
) -> Vec<Result<FrameMatrix>> {
    if cache_only() {
        return items
            .iter()
            .map(|(_, request)| {
                Err(anyhow::anyhow!(
                    "frame-matrix cache miss for {:016x}; cache-only mode is enabled",
                    request.hash
                ))
            })
            .collect();
    }
    let Some((ctx, _)) = items.first() else {
        return Vec::new();
    };
    let client = wav2vec2::batch_client(ctx.http.clone());
    infer_frame_batch_at(items, &client, activity).await
}

async fn infer_frame_batch_at(
    items: Vec<(&VerifyContext<'_>, PreparedFrameRequest)>,
    client: &Result<PhonemizerClient>,
    activity: Option<&wav2vec2::RequestActivity>,
) -> Vec<Result<FrameMatrix>> {
    let (destinations, requests): (Vec<_>, Vec<_>) = items
        .into_iter()
        .map(|(ctx, request)| ((ctx, request.hash), request.payload))
        .unzip();
    let response = match client {
        Ok(client) => wav2vec2::predict_batch(client, &requests, activity).await,
        Err(error) => Err(anyhow::anyhow!("{error:#}")),
    };
    let responses = match response {
        Ok(responses) => responses,
        Err(error) => {
            return destinations
                .iter()
                .map(|_| Err(anyhow::anyhow!("{error:#}")))
                .collect();
        }
    };
    let mut results = Vec::with_capacity(destinations.len());
    for ((ctx, hash), response) in destinations.into_iter().zip(responses) {
        results.push(async { cache_response(ctx, hash, response?).await }.await);
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
        if let Some(Ok(frames)) = cached_frame_matrix(ctx, &ctx.response_key(hash)).await {
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
        if !requests.is_empty() && (requests.len() == MODAL_BATCH_SIZE || pending.peek().is_none())
        {
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
    let mut ids = Vec::new();
    let mut spans = Vec::with_capacity(segments.len());
    for segment in segments {
        // Raw CTC uses only the single default-voice target, not accepted-variant
        // readings (including Spanish seseo). Seseo-pronounced Spanish therefore
        // scores worse here than on the edit-distance path.
        let phonemized = model_target(segment, ctx.target_language)
            .ok_or_else(|| anyhow::anyhow!("no g2p for {:?}", ctx.target_language))?
            .with_context(|| format!("phonemizing {segment:?}"))?;
        let start = ids.len();
        ids.extend(phonemized.phonemes.iter().filter_map(|p| matrix.id(p)));
        if ids.len() == start {
            anyhow::bail!("no phonemes the model knows for {segment:?}");
        }
        spans.push((start, ids.len()));
    }
    let aligned = matrix.force_align(&ids).ok_or_else(|| {
        anyhow::anyhow!(
            "no alignment of {} phonemes over {} frames",
            ids.len(),
            matrix.frames
        )
    })?;
    let ms = |frame: usize| (frame as u32 * FRAME_MS).saturating_sub(pad_ms);
    Ok(spans
        .iter()
        .map(|&(start, end)| {
            (
                ms(aligned[start].start_frame),
                ms(aligned[end - 1].end_frame + 1),
            )
        })
        .collect())
}

/// Decode WAV bytes to mono f32 samples at 16 kHz by piping through ffmpeg.
/// 16 kHz is the standard rate wav2vec2 phoneme models expect.
fn decode_wav_to_f32(wav_bytes: &[u8]) -> Result<Vec<f32>> {
    let mut child = Command::new("ffmpeg")
        .args([
            "-loglevel",
            "error",
            "-i",
            "pipe:0",
            "-f",
            "f32le",
            "-ar",
            "16000",
            "-ac",
            "1",
            "pipe:1",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn ffmpeg for WAV decoding")?;

    // Write stdin from its own thread: ffmpeg streams output while it
    // still has input left to read, so once the input is larger than the
    // OS pipe buffers, writing it all before draining stdout deadlocks —
    // ffmpeg blocks writing stdout, we block writing stdin. Short clips
    // never hit this; a 12-second movie cue does. Same pattern as
    // `subtitle-corpus`'s `encode_opus`.
    let mut stdin = child.stdin.take().context("ffmpeg stdin not captured")?;
    let owned_bytes = wav_bytes.to_vec();
    let writer = std::thread::spawn(move || stdin.write_all(&owned_bytes));

    let output = child
        .wait_with_output()
        .context("Failed to wait for ffmpeg")?;
    writer
        .join()
        .map_err(|_| anyhow::anyhow!("ffmpeg stdin writer thread panicked"))?
        .context("Failed to write WAV bytes to ffmpeg stdin")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffmpeg decode failed ({}): {stderr}", output.status);
    }

    if output.stdout.len() % 4 != 0 {
        anyhow::bail!(
            "ffmpeg output is not a multiple of 4 bytes ({} bytes)",
            output.stdout.len()
        );
    }
    Ok(output
        .stdout
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect())
}

/// Expand a raw IPA token into the deployed model's comparable token sequence.
/// Shared by expected readings, predictions, and top-k alternatives.
///
/// TODO(g2p): the pending pin update must reconcile this split-token
/// normalization with the deployed model's merged-token labels.
/// Only explicit tie bars split tokens: preserve untied diphthongs/diacritics.
pub fn normalize_phonemes(token: &str, language: Language) -> Vec<String> {
    token
        .split(['\u{0361}', '\u{035c}'])
        .filter_map(|component| normalize_phoneme(component, language))
        .collect()
}

/// Normalize a single IPA component into a canonical comparable form. Returns
/// `None` for tokens that consist entirely of non-phonemic markers.
///
/// Three layers of cleanup, applied in order:
///
/// 1. **Universal stripping** — remove characters that aren't phonemic in
///    any language: suprasegmental stress (`ˈ`/`ˌ`), syllable boundary
///    (`.`), length marks (`ː`/`ˑ`), the liaison/elision marker (`‿`),
///    ASCII digits (the multilingual wav2vec2 model leaks Mandarin tone
///    numbers like `y5`, `i5`, `a5` into French output), and `^`, an
///    espeak artifact the Russian voice leaves on a word-final palatalized
///    consonant (`царь` → `tsɑrɪ^`) that lexide blacklists at preprocess
///    time, so no model can emit it.
/// 2. **Internal whitespace strip** — `f a ɪ` is already split on
///    whitespace by the caller, but defensive in case a token slipped
///    through with embedded whitespace.
/// 3. **Per-language canonicalization** — collapses phoneme equivalence
///    classes so model conventions and ground-truth conventions agree:
///    for French, `{r, ʀ}` collapse to `ʁ`, `{ts, tɕ, tɕh}` to `t`,
///    `ɥ → y` (the model doesn't reliably distinguish the rounded palatal
///    approximant from `y`).
///
/// Combining diacritics inside the phoneme (e.g. the tilde on `ã`) are NOT
/// touched — those are phonemic (except German non-syllabic U+032F).
fn normalize_phoneme(token: &str, language: Language) -> Option<String> {
    let stripped: String = token
        .chars()
        .filter(|c| {
            !matches!(*c, 'ˈ' | 'ˌ' | '.' | 'ː' | 'ˑ' | '‿' | '^')
                && !c.is_ascii_digit()
                && !c.is_whitespace()
        })
        .collect();
    let canonical = canonicalize_for_language(&stripped, language);
    (!canonical.is_empty()).then_some(canonical)
}

/// Map a phoneme token onto its canonical form for the given target
/// language. Symmetric across predicted/expected — when both sides go
/// through this, equivalence-class members compare equal.
fn canonicalize_for_language(token: &str, language: Language) -> String {
    match language {
        // German inventory: 43,267 rows / 1,079,676 tokens in lexide's
        // pronunciation/data/audio/deu/phonemes.jsonl contain neither ʔ nor
        // U+032F. Strip these reference-only marks, not vowels or diphthongs.
        Language::German => token
            .chars()
            .filter(|c| !matches!(c, 'ʔ' | '\u{032f}'))
            .collect(),
        Language::French => match token {
            // R variants: ground truth uses ʁ (uvular fricative); the
            // multilingual model emits any of: r (alveolar trill, common
            // across many languages), ʀ (uvular trill, French stage variant),
            // ɾ (alveolar tap, Spanish/Italian-flavored), x (voiceless velar
            // fricative, sometimes emitted for the devoiced uvular allophone
            // /χ/ that occurs phrase-finally in French). All collapse to ʁ.
            "r" | "ʀ" | "ʁ" | "ɾ" | "x" => "ʁ".to_string(),
            // /t/ affricate variants observed in the model output for
            // French. Not phonemic in French; collapse to plain /t/.
            "ts" | "tɕ" | "tɕh" | "tɕʰ" => "t".to_string(),
            // The model conflates the rounded palatal approximant with /y/.
            "ɥ" => "y".to_string(),
            // Front /a/ vs back /ɑ/ is no longer phonemically distinguished
            // in modern Parisian French; both wikipron and the model use
            // them inconsistently. Collapse to a.
            "ɑ" | "a" => "a".to_string(),
            _ => token.to_string(),
        },
        _ => token.to_string(),
    }
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
) -> Option<Vec<Reading>> {
    // Collect per-word phoneme-sequence variants in phrase order.
    // `complete` flips to false when a word is missing from the dictionary
    // and g2p can't name it either; we then skip the cross-product and rely
    // on the phrase-level g2p variant (if available) as the sole ground truth.
    let mut per_word: Vec<Vec<Vec<String>>> = Vec::new();
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
            let g2p_word: Vec<String> = match model_target(&cleaned, language) {
                Some(Ok(phonemized)) => phonemized
                    .phonemes
                    .iter()
                    .flat_map(|p| normalize_phonemes(p, language))
                    .collect(),
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
        let mut word_variants: Vec<Vec<String>> = accepted
            .all()
            .map(|ipa| {
                ipa.split_whitespace()
                    .flat_map(|p| normalize_phonemes(p, language))
                    .collect::<Vec<String>>()
            })
            // Distinct sequences only — after normalization, different raw
            // wikipron entries can collapse to the same phoneme sequence.
            .fold(Vec::new(), |mut acc, v| {
                if !acc.contains(&v) {
                    acc.push(v);
                }
                acc
            });
        add_spanish_dialect_word(&cleaned, language, &mut word_variants);
        per_word.push(word_variants);
    }

    let mut candidates: Vec<Reading> = if !complete || per_word.is_empty() {
        // No per-word candidates — the phrase-level g2p below is our only shot.
        Vec::new()
    } else {
        let mut acc: Vec<Reading> = vec![Vec::new()];
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
                // Words come from g2p's own spans (a backend without them
                // yields one word), which is what lets a phrase-level reading
                // still say which word the audio skipped.
                let spans = if phonemized.word_spans.is_empty() {
                    vec![(0, phonemized.phonemes.len())]
                } else {
                    phonemized.word_spans.clone()
                };
                let g2p_reading: Reading = spans
                    .iter()
                    .map(|&(start, end)| {
                        phonemized.phonemes[start..end]
                            .iter()
                            .flat_map(|p| normalize_phonemes(p, language))
                            .collect::<Vec<String>>()
                    })
                    .filter(|word| !word.is_empty())
                    .collect();
                let flat = g2p_reading.concat();
                if !flat.is_empty() && !candidates.iter().any(|c| c.concat() == flat) {
                    candidates.push(g2p_reading);
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

/// Seseo is an accepted Spanish dialect reading, not a θ/s equivalence:
/// keep the training-contract `es` target and both phones unchanged.
fn spanish_dialect_target(
    text: &str,
    language: Language,
) -> Option<Result<g2p::Phonemized, g2p::Error>> {
    (language == Language::Spanish).then(|| g2p::phonemize(text, "es-419"))
}

fn add_spanish_dialect_word(text: &str, language: Language, variants: &mut Vec<Vec<String>>) {
    if let Some(Ok(target)) = spanish_dialect_target(text, language) {
        let phones: Vec<String> = target
            .phonemes
            .iter()
            .flat_map(|p| normalize_phonemes(p, language))
            .collect();
        if !phones.is_empty() && !variants.contains(&phones) {
            variants.push(phones);
        }
    }
}

/// Normalize the model's raw phoneme output AND the parallel top-k
/// alternatives at the same time. Returns the two lists with matching
/// length, parallel by position.
///
/// Empty normalized tokens drop their entire position. A tied affricate
/// expands into component positions; alternatives with the same component
/// count contribute probability only to their corresponding position. Other
/// lengths are omitted, since a per-position top-k cannot represent them.
/// Canonical equivalents at each position merge by summing probability, then
/// sort by descending probability. No probability is renormalized.
fn normalize_with_topk(
    raw_phonemes: &[String],
    raw_top_k: &[Vec<RawPhonemeAlt>],
    language: Language,
) -> (Vec<String>, Vec<Vec<(String, f64)>>) {
    let mut normalized = Vec::with_capacity(raw_phonemes.len());
    let mut normalized_top_k: Vec<Vec<(String, f64)>> = Vec::with_capacity(raw_phonemes.len());

    for (i, raw) in raw_phonemes.iter().enumerate() {
        let components = normalize_phonemes(raw, language);
        let mut merged = vec![HashMap::<String, f64>::new(); components.len()];
        if let Some(alts) = raw_top_k.get(i) {
            for alt in alts {
                let alt_components = normalize_phonemes(&alt.phoneme, language);
                // Alternatives must span the same number of component positions.
                // Do not credit a single /t/ with a whole /t͡ʃ/, or duplicate an
                // atomic alternative across both positions of a split affricate.
                if alt_components.len() == components.len() {
                    for (position, component) in merged.iter_mut().zip(alt_components) {
                        *position.entry(component).or_default() += alt.probability;
                    }
                }
            }
        }
        normalized.extend(components);
        for position in merged {
            let mut alts: Vec<_> = position.into_iter().collect();
            alts.sort_by(|a, b| {
                b.1.partial_cmp(&a.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.0.cmp(&b.0))
            });
            normalized_top_k.push(alts);
        }
    }

    (normalized, normalized_top_k)
}

/// Look up `phoneme`'s probability in a normalized top-k list. Returns
/// `None` when the phoneme wasn't in the top-k — i.e. the model
/// effectively assigned ~0 probability to it.
fn prob_of(phoneme: &str, top_k: &[(String, f64)]) -> Option<f64> {
    top_k
        .iter()
        .find(|(p, _)| p == phoneme)
        .map(|(_, prob)| *prob)
}

/// Levenshtein alignment with per-position probability annotations. Same
/// algorithm as before; ops now carry the model's confidence at the
/// predicted position so the JSONL output can show how close the model
/// was to the correct answer.
/// The first word of `reading` with two or more phonemes that the alignment
/// leaves entirely unheard: every phoneme `Missing`, none matched or
/// substituted. A whole word gone is a letter or word the audio skipped —
/// a voice that silently drops "œ" — however small the edit distance looks
/// beside a long example. One-phoneme words are exempt: the model does
/// swallow a lone schwa.
fn unheard_word<'a>(reading: &'a Reading, ops: &[AlignmentOp]) -> Option<&'a [String]> {
    let mut consumed = ops
        .iter()
        .filter(|op| !matches!(op, AlignmentOp::Extra { .. }));
    for word in reading {
        let heard = consumed
            .by_ref()
            .take(word.len())
            .filter(|op| !matches!(op, AlignmentOp::Missing { .. }))
            .count();
        if word.len() >= 2 && heard == 0 {
            return Some(word);
        }
    }
    None
}

fn align(
    predicted: &[String],
    predicted_top_k: &[Vec<(String, f64)>],
    expected: &[String],
) -> (usize, Vec<AlignmentOp>) {
    let (m, n) = (predicted.len(), expected.len());

    // Helper for the chosen phoneme's prob at predicted position i. Falls
    // back to 1.0 if the position has no top-k (shouldn't happen, but
    // defensive — e.g. for callers in tests that pass empty top-k).
    let pred_prob = |i: usize| -> f64 {
        predicted_top_k
            .get(i)
            .and_then(|alts| alts.first())
            .map(|(_, p)| *p)
            .unwrap_or(1.0)
    };
    // Helper for prob of the expected phoneme at predicted position i.
    let exp_prob_at = |i: usize, exp_ph: &str| -> Option<f64> {
        predicted_top_k
            .get(i)
            .and_then(|alts| prob_of(exp_ph, alts))
    };

    // dp[i][j] = min cost to align predicted[..i] with expected[..j].
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for (i, row) in dp.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in dp[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=m {
        for j in 1..=n {
            let cost = if predicted[i - 1] == expected[j - 1] {
                0
            } else {
                1
            };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }

    // Backtrack from (m, n) to (0, 0), preferring match > sub > extra/missing
    // on ties so the alignment is readable. `i-1` is the predicted position
    // we're emitting an op for; we use it to look up probabilities.
    let mut ops: Vec<AlignmentOp> = Vec::with_capacity(m.max(n));
    let (mut i, mut j) = (m, n);
    while i > 0 || j > 0 {
        let here = dp[i][j];
        if i > 0 && j > 0 {
            let cost = if predicted[i - 1] == expected[j - 1] {
                0
            } else {
                1
            };
            if dp[i - 1][j - 1] + cost == here {
                if cost == 0 {
                    ops.push(AlignmentOp::Match {
                        phoneme: predicted[i - 1].clone(),
                        probability: pred_prob(i - 1),
                    });
                } else {
                    ops.push(AlignmentOp::Sub {
                        expected: expected[j - 1].clone(),
                        predicted: predicted[i - 1].clone(),
                        predicted_prob: pred_prob(i - 1),
                        expected_prob: exp_prob_at(i - 1, &expected[j - 1]),
                    });
                }
                i -= 1;
                j -= 1;
                continue;
            }
        }
        if i > 0 && dp[i - 1][j] + 1 == here {
            ops.push(AlignmentOp::Extra {
                predicted: predicted[i - 1].clone(),
                predicted_prob: pred_prob(i - 1),
            });
            i -= 1;
            continue;
        }
        // Remaining case: dp[i][j-1] + 1 == here
        ops.push(AlignmentOp::Missing {
            expected: expected[j - 1].clone(),
        });
        j -= 1;
    }
    ops.reverse();
    (dp[m][n], ops)
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
mod tests {
    #[test]
    fn tied_affricates_expand_symmetrically_in_every_language() {
        for language in [
            Language::German,
            Language::Spanish,
            Language::Italian,
            Language::French,
            Language::English,
            Language::Russian,
            Language::Hindi,
            Language::Portuguese,
            Language::Korean,
            Language::ChineseSimplified,
            Language::ChineseTraditional,
            Language::Japanese,
            Language::Thai,
        ] {
            for (raw, components) in [
                ("ˈt͡ʃː", ["t", "ʃ"]),
                ("t͜ʃ", ["t", "ʃ"]),
                ("d͡ʒ", ["d", "ʒ"]),
                ("d͜ʒ", ["d", "ʒ"]),
                ("t͡s", ["t", "s"]),
                ("t͜s", ["t", "s"]),
                ("d͡z", ["d", "z"]),
                ("d͜z", ["d", "z"]),
            ] {
                let expected = normalize_phonemes(raw, language);
                assert_eq!(expected, word(&components));
                let (predicted, topk) = normalize_with_topk(&word(&[raw]), &[], language);
                assert_eq!(predicted, expected);
                assert_eq!(topk.len(), 2);
                assert_eq!(align(&predicted, &topk, &expected).0, 0);
            }
        }
        assert_eq!(normalize_phonemes("aɪ", Language::German), word(&["aɪ"]));
        assert_eq!(normalize_phonemes("ɪ̯", Language::German), word(&["ɪ"]));
        assert_eq!(normalize_phonemes("ɐ̯", Language::German), word(&["ɐ"]));
        assert_eq!(normalize_phonemes("ɪ̯", Language::Spanish), word(&["ɪ̯"]));
        assert_eq!(
            normalize_phonemes("ʔ", Language::German),
            Vec::<String>::new()
        );
        assert_eq!(normalize_phonemes("ʔ", Language::English), word(&["ʔ"]));
        assert_eq!(normalize_phonemes("ã", Language::German), word(&["ã"]));
    }

    #[test]
    fn expanded_topk_is_component_aligned_without_spurious_atomic_matches() {
        let alt = |phoneme: &str, probability| RawPhonemeAlt {
            phoneme: phoneme.into(),
            probability,
        };
        let raw = word(&["ʔ", "t͡ʃ", "a", "t"]);
        let topk = vec![
            vec![alt("ʔ", 1.0)],
            vec![
                alt("t͡ʃ", 0.5),
                alt("t͜ʃ", 0.2),
                alt("d͡ʒ", 0.1),
                alt("t", 0.2),
            ],
            vec![alt("a", 0.9)],
            vec![alt("t", 0.6), alt("t͡ʃ", 0.4)],
        ];
        let (tokens, normalized) = normalize_with_topk(&raw, &topk, Language::German);
        assert_eq!(tokens, word(&["t", "ʃ", "a", "t"]));
        assert_eq!(normalized.len(), tokens.len());
        assert_eq!(prob_of("t", &normalized[0]), Some(0.7));
        assert_eq!(prob_of("ʃ", &normalized[1]), Some(0.7));
        assert_eq!(prob_of("d", &normalized[0]), Some(0.1));
        assert_eq!(prob_of("ʒ", &normalized[1]), Some(0.1));
        assert_eq!(prob_of("t", &normalized[1]), None);
        assert_eq!(prob_of("a", &normalized[2]), Some(0.9));
        assert_eq!(prob_of("t", &normalized[3]), Some(0.6));
        assert_eq!(prob_of("ʃ", &normalized[3]), None);
    }

    #[test]
    fn systemic_cues_normalize_references_without_hiding_spoken_errors() {
        // Exact examples extracted from the September 14 deployed verification
        // reports under /tmp/lexide-deploy-verify/{deu,spa,ita}-systemic.json.
        let fixtures: Vec<serde_json::Value> =
            serde_json::from_str(include_str!("../tests/fixtures/systemic-cues.json")).unwrap();
        for fixture in fixtures {
            let language = match fixture["language"].as_str().unwrap() {
                "deu" => Language::German,
                "spa" => Language::Spanish,
                "ita" => Language::Italian,
                _ => unreachable!(),
            };
            let raw: Vec<String> = serde_json::from_value(fixture["raw"].clone()).unwrap();
            let original: Vec<String> =
                serde_json::from_value(fixture["expected"].clone()).unwrap();
            let expected: Vec<_> = original
                .iter()
                .flat_map(|p| normalize_phonemes(p, language))
                .collect();
            let (heard, topk) = normalize_with_topk(&raw, &[], language);
            assert_eq!(
                expected,
                original
                    .iter()
                    .flat_map(|p| {
                        if language == Language::German {
                            match p.as_str() {
                                "ʔ" => vec![],
                                "ʏ̯" => word(&["ʏ"]),
                                _ => vec![p.clone()],
                            }
                        } else {
                            match p.as_str() {
                                "t͡ʃ" => word(&["t", "ʃ"]),
                                "d͡ʒ" => word(&["d", "ʒ"]),
                                _ => vec![p.clone()],
                            }
                        }
                    })
                    .collect::<Vec<_>>(),
                "{}",
                fixture["text"]
            );
            assert_eq!(
                heard,
                serde_json::from_value::<Vec<String>>(fixture["heard"].clone()).unwrap()
            );
            assert!(
                align(&heard, &topk, &expected).0 > 0,
                "do not erase genuine errors: {}",
                fixture["text"]
            );
        }
    }

    #[tokio::test]
    async fn german_normalization_and_provenance_cover_every_verification_path() {
        let dir = tempfile::tempdir().unwrap();
        let http = reqwest::Client::new();
        let mut dictionary = HashMap::new();
        dictionary.insert("test".into(), ap("ʔ t͡ʃ ɪ̯", &[]));
        let ctx = VerifyContext {
            http: &http,
            store: osmo::Store::open(dir.path()),
            cache_key: None,
            expected_identity: None,
            word_to_pronunciation: &dictionary,
            mismatch_threshold: 0.3,
            target_language: Language::German,
            expected_deploy_marker: None,
        };
        let expected = expected_phoneme_variants(&ctx, "test", Some("ʔ t͡ʃ ɪ̯")).unwrap();
        assert_eq!(expected, vec![vec![word(&["t", "ʃ", "ɪ"])]]);
        assert!(
            expected_phoneme_variants(&ctx, "test", None)
                .unwrap()
                .contains(&expected[0])
        );
        // Invalid bytes intentionally bypass ffmpeg's defect gate; both model
        // payloads below are cached, so there is no inference/network request.
        let audio = b"cached normalization regression";
        let hash = xxh3_64(audio);
        let response = serde_json::from_value(serde_json::json!({
            "phonemes": [{"phoneme": "ʔ"}, {"phoneme": "t͜ʃ"}, {"phoneme": "ɪ̯"}],
            "frame_matrix": batch_test_payload(1),
            "model_id": "actual-model", "model_revision": "actual-revision"
        }))
        .unwrap();
        cache_response(&ctx, hash, response).await.unwrap();
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
        cache_response(&ctx, hash, response.clone()).await.unwrap();
        ctx.expected_identity = Some(test_identity()); // different from cached producer
        assert_eq!(frame_matrix(&ctx, wav).await.unwrap().frames, 3);
        let batch = frame_matrices_at(&ctx, &[wav], Err(anyhow::anyhow!("offline")), true).await;
        assert_eq!(batch[0].as_ref().unwrap().frames, 3);
        assert_eq!(
            response_identity(
                &cached_response(&ctx, &ctx.response_key(hash))
                    .await
                    .unwrap()
                    .unwrap()
            )
            .unwrap()
            .model_id,
            "actual"
        );
        ctx.expected_identity = None;
        ctx = ctx.with_cache_key(|hash| format!("caller/{hash:016x}"));
        assert!(
            cached_frame_matrix(&ctx, &ctx.response_key(hash))
                .await
                .is_none()
        );
        cache_response(&ctx, hash, response).await.unwrap();
        assert!(
            cached_frame_matrix(&ctx, &ctx.response_key(hash))
                .await
                .unwrap()
                .is_ok()
        );
        assert!(
            cached_frame_matrix(&ctx, &ctx.response_key(hash + 1))
                .await
                .is_none()
        );
        ctx.store
            .write(&ctx.response_key(hash), b"corrupt")
            .await
            .unwrap();
        assert!(
            cached_frame_matrix(&ctx, &ctx.response_key(hash))
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
        cache_response(&ctx, 42, response.clone()).await.unwrap();
        let cached = cached_response(&ctx, &ctx.response_key(42))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            serde_json::to_value(&cached).unwrap(),
            serde_json::to_value(&response).unwrap()
        );
        let frames = cached_frame_matrix(&ctx, &ctx.response_key(42))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(frames.heads.len(), 4);
        assert_eq!(frame_identity(&frames), Some(test_identity()));
        let mut conflicting = response;
        conflicting.model_revision = Some("not the matrix revision".into());
        assert!(cache_response(&ctx, 43, conflicting).await.is_err());
        assert!(cached_response(&ctx, &ctx.response_key(43)).await.is_none());
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

    #[test]
    fn verified_identity_rejects_unchecked_cache_overrides() {
        let dir = tempfile::tempdir().unwrap();
        let http = reqwest::Client::new();
        let words = HashMap::new();
        let mut ctx = VerifyContext::with_overrides(
            &http,
            osmo::Store::open(dir.path()),
            &words,
            Language::French,
            0.3,
            None,
        )
        .unwrap();
        assert!(ctx.expected_model_identity().is_err());
        // The explicit constructor is local; only a resolved identity can pin it.
        ctx.expected_identity = Some(resolve_model(|| Ok(test_identity())).unwrap());
        assert_eq!(ctx.expected_model_identity().unwrap(), &test_identity());
    }

    fn test_identity() -> ModelIdentity {
        ModelIdentity {
            model_id: "test/model".into(),
            model_revision: "1234567890abcdef".into(),
            decoder_version: Some(DECODER_VERSION.into()),
            deploy_marker: Some("fresh".into()),
        }
    }

    #[test]
    fn resolved_state_is_shared_and_failure_is_not_reprobed() {
        let cell = OnceLock::new();
        let first = resolved_model_once(&cell, || resolve_model(|| Ok(test_identity()))).unwrap();
        let second = resolved_model_once(&cell, || panic!("must resolve only once")).unwrap();
        assert!(std::ptr::eq(first, second));
        assert_eq!(first, second);
        let failed = OnceLock::new();
        assert!(resolved_model_once(&failed, || anyhow::bail!("offline")).is_err());
        assert!(resolved_model_once(&failed, || panic!("must not retry")).is_err());
    }

    #[test]
    fn discovery_fails_closed() {
        assert_eq!(
            resolve_model(|| Ok(test_identity())).unwrap(),
            test_identity()
        );
        assert!(resolve_model(|| anyhow::bail!("offline")).is_err());
        for field in ["model_id", "model_revision", "decoder_version"] {
            let mut identity = test_identity();
            match field {
                "model_id" => identity.model_id.clear(),
                "model_revision" => identity.model_revision.clear(),
                _ => identity.decoder_version = Some("wrong".into()),
            }
            assert!(resolve_model(|| Ok(identity)).is_err());
        }
        assert!(check_marker(Some("fresh"), Some("stale")).is_err());
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
        ctx.expected_identity = Some(test_identity());
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
            let split = wav2vec2::split_batch(serde_json::from_value(bad).unwrap());
            assert!(split.is_err() || split.unwrap().remove(0).is_err());
            // Missing per-item metadata inherits the envelope and is checked.
            let mut envelope = good.clone();
            envelope[field] = serde_json::json!("wrong");
            envelope["results"] = serde_json::json!([{"phonemes": []}]);
            match wav2vec2::split_batch(serde_json::from_value(envelope).unwrap()) {
                Err(_) => {}
                Ok(mut items) => {
                    assert!(check_response_identity(&ctx, &items.remove(0).unwrap()).is_err())
                }
            }
        }
        let mut envelope = good.clone();
        envelope["results"] = serde_json::json!([{"phonemes": []}]);
        let item = wav2vec2::split_batch(serde_json::from_value(envelope).unwrap())
            .unwrap()
            .remove(0)
            .unwrap();
        assert!(check_response_identity(&ctx, &item).is_ok());
        assert_eq!(item.model_revision, Some(test_identity().model_revision));
        assert!(check_response_identity(&ctx, &response(serde_json::json!({}))).is_err());
        ctx.expected_deploy_marker = None;
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
            "response": {"phonemes": [], "frame_matrix": payload}
        }))
        .unwrap();
        ctx.store.write(&key, &bytes).await.unwrap();
        let cached = cached_frame_matrix(&ctx, &ctx.response_key(hash))
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
        let ctx = VerifyContext {
            http: &http,
            store,
            cache_key: None,
            expected_identity: None,
            word_to_pronunciation: &empty,
            mismatch_threshold: 0.3,
            target_language: Language::English,
            expected_deploy_marker: Some("test".into()),
        };
        let cached = b"cached without decoding".to_vec();
        let response = serde_json::from_value(serde_json::json!({
            "phonemes": [], "frame_matrix": batch_test_payload(2), "deploy_marker": "test"
        }))
        .unwrap();
        cache_response(&ctx, xxh3_64(&cached), response)
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

    #[test]
    fn normalize_strips_suprasegmentals() {
        let lang = Language::English;
        assert_eq!(normalize_phoneme("ˈa", lang), Some("a".to_string()));
        assert_eq!(normalize_phoneme("ˌb", lang), Some("b".to_string()));
        assert_eq!(normalize_phoneme("iː", lang), Some("i".to_string()));
        assert_eq!(normalize_phoneme(".", lang), None);
        assert_eq!(normalize_phoneme("ˈ", lang), None);
        // Combining diacritics inside the phoneme are preserved.
        assert_eq!(normalize_phoneme("ã", lang), Some("ã".to_string()));
        // espeak's Russian `^` artifact is not a phone anywhere.
        assert_eq!(
            normalize_phoneme("ɪ^", Language::Russian),
            Some("ɪ".to_string())
        );
        assert_eq!(normalize_phoneme("^", Language::Russian), None);
    }

    #[test]
    fn normalize_strips_tone_digits_and_liaison() {
        let lang = Language::French;
        // Mandarin tone digits leak through the multilingual wav2vec2 model.
        assert_eq!(normalize_phoneme("y5", lang), Some("y".to_string()));
        assert_eq!(normalize_phoneme("i5", lang), Some("i".to_string()));
        assert_eq!(normalize_phoneme("a5", lang), Some("a".to_string()));
        // Liaison marker has no phonetic content.
        assert_eq!(normalize_phoneme("‿", lang), None);
        assert_eq!(normalize_phoneme("a‿", lang), Some("a".to_string()));
    }

    #[test]
    fn french_canonicalization() {
        let lang = Language::French;
        assert_eq!(normalize_phoneme("r", lang), Some("ʁ".to_string()));
        assert_eq!(normalize_phoneme("ʁ", lang), Some("ʁ".to_string()));
        assert_eq!(normalize_phoneme("ts", lang), Some("t".to_string()));
        assert_eq!(normalize_phoneme("tɕ", lang), Some("t".to_string()));
        assert_eq!(normalize_phoneme("ɥ", lang), Some("y".to_string()));
        // Non-French langs: no canonicalization, just pass-through.
        assert_eq!(
            normalize_phoneme("r", Language::English),
            Some("r".to_string())
        );
    }

    /// Readings flattened to phoneme sequences, for tests about which
    /// sequences are accepted rather than where their words fall.
    fn flat_variants(
        text: &str,
        wp: &HashMap<String, language_utils::Pronunciations>,
        language: Language,
    ) -> Option<Vec<Vec<String>>> {
        ground_truth_phoneme_variants(text, wp, language)
            .map(|readings| readings.iter().map(|r| r.concat()).collect())
    }

    fn word(phonemes: &[&str]) -> Vec<String> {
        phonemes.iter().map(|p| p.to_string()).collect()
    }

    #[test]
    fn readings_keep_word_boundaries() {
        let mut wp = HashMap::new();
        wp.insert("bonjour".to_string(), ap("b ɔ̃ ʒ u ʁ", &[]));
        wp.insert("madame".to_string(), ap("m a d a m", &[]));
        let readings =
            ground_truth_phoneme_variants("Bonjour madame", &wp, Language::ChineseTraditional)
                .unwrap();
        assert_eq!(
            readings,
            vec![vec![
                word(&["b", "ɔ̃", "ʒ", "u", "ʁ"]),
                word(&["m", "a", "d", "a", "m"])
            ]]
        );
    }

    #[test]
    fn unheard_word_is_one_with_no_phoneme_matched_or_substituted() {
        let reading: Reading = vec![word(&["o", "ʊ"]), word(&["æ", "z"]), word(&["ə"])];
        let missing = |p: &str| AlignmentOp::Missing {
            expected: p.to_string(),
        };
        let matched = |p: &str| AlignmentOp::Match {
            phoneme: p.to_string(),
            probability: 1.0,
        };
        // "œ" (o ʊ) skipped entirely, the rest heard: the first word is unheard.
        let ops = vec![
            missing("o"),
            missing("ʊ"),
            matched("æ"),
            matched("z"),
            matched("ə"),
        ];
        assert_eq!(
            unheard_word(&reading, &ops),
            Some(word(&["o", "ʊ"]).as_slice())
        );
        // A substitution counts as heard; extras don't consume expected phonemes.
        let ops = vec![
            AlignmentOp::Extra {
                predicted: "h".into(),
                predicted_prob: 1.0,
            },
            AlignmentOp::Sub {
                expected: "o".into(),
                predicted: "ɔ".into(),
                predicted_prob: 1.0,
                expected_prob: None,
            },
            missing("ʊ"),
            matched("æ"),
            matched("z"),
            missing("ə"),
        ];
        // The lone schwa is exempt even though it went unheard.
        assert_eq!(unheard_word(&reading, &ops), None);
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
            vec!["b", "ɔ̃", "ʒ", "u", "ʁ", "m", "a", "d", "a", "m"]
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
        assert!(variants.contains(&vec!["m".to_string(), "e".to_string()]));
        assert!(variants.contains(&vec!["m".to_string(), "ɛ".to_string()]));
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
        let cyrano = ["s", "i", "ʁ", "a", "n", "o"].map(str::to_string);
        for accepted in [
            ["k", "ɔ", "ɲ", "a", "k"].map(str::to_string),
            ["k", "o", "ɲ", "a", "k"].map(str::to_string),
        ] {
            let expected: Vec<String> = cyrano.iter().chain(accepted.iter()).cloned().collect();
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
        let wp = HashMap::from([("a".to_string(), ap("X", &["Y"]))]);
        for count in [5, 100] {
            let text = vec!["a"; count].join(" ");
            let variants = flat_variants(&text, &wp, Language::ChineseTraditional).unwrap();
            assert_eq!(variants.len(), MAX_VARIANT_COMBINATIONS);
            for (index, variant) in variants.iter().enumerate() {
                let expected: Vec<&str> = (0..count)
                    .map(|position| {
                        if position < 4 && index & (1 << position) != 0 {
                            "Y"
                        } else {
                            "X"
                        }
                    })
                    .collect();
                assert_eq!(*variant, expected);
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
            assert!(readings.iter().any(|r| r.concat() == expected));
            assert!(readings.iter().all(|r| r.len() == 5));
            let phrase = model_target(text, Language::Portuguese).unwrap().unwrap();
            assert!(readings.iter().any(|r| r.concat() == phrase.phonemes));
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
                PhonemeLabelSource::Espeak(voice) => Some(g2p::LabelSource::Espeak(voice)),
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
        assert_eq!(normalize_phoneme("θ", Language::Spanish), Some("θ".into()));
        assert_eq!(normalize_phoneme("s", Language::Spanish), Some("s".into()));
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
        let mut wp = HashMap::from([("nombre".to_string(), ap("X", &[]))]);
        for dictionary_cinco in [false, true] {
            if dictionary_cinco {
                wp.insert("cinco".to_string(), ap("θ i n k o", &[]));
            }
            let readings =
                ground_truth_phoneme_variants("cinco nombre", &wp, Language::Spanish).unwrap();
            assert!(readings.contains(&vec![word(&["s", "i", "n", "k", "o"]), word(&["X"])]));
        }
        // The cap must not prevent a pure seseo phrase reading when later
        // words' per-word alternates are truncated.
        let text = ["cinco"; 6].join(" ");
        let readings = ground_truth_phoneme_variants(&text, &empty, Language::Spanish).unwrap();
        assert_eq!(readings.len(), MAX_VARIANT_COMBINATIONS + 1);
        assert!(readings.contains(&vec![word(&["s", "i", "n", "k", "o"]); 6]));
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
            let wp = HashMap::from([(text.to_string(), ap("X", &[]))]);
            let readings =
                ground_truth_phoneme_variants(&format!("‘{text}।’"), &wp, Language::Hindi).unwrap();
            assert_eq!(readings[0], vec![word(&["X"])]);
            let expected: Vec<String> = model_target(text, Language::Hindi)
                .unwrap()
                .unwrap()
                .phonemes
                .iter()
                .flat_map(|p| normalize_phonemes(p, Language::Hindi))
                .collect();
            let readings =
                ground_truth_phoneme_variants(text, &HashMap::new(), Language::Hindi).unwrap();
            assert_eq!(readings, vec![vec![expected]]);
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
            variants.contains(&vec!["ɔ̃".to_string(), "n".to_string(), "ɛ".to_string()]),
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
                .flat_map(|p| normalize_phonemes(p, Language::French))
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
            ["j", "eː", "ʃ", "ɛː", "ɦ", "ɛː", "ɾ", "ɡ", "j", "aː", "n"]
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
        assert_eq!(target.phonemes, ["ɡ", "a", "kː", "o", "o"]);
        assert!(target.pitch.iter().flatten().count() > 0);
    }

    #[test]
    fn mandarin_targets_come_from_the_g2pm_port() {
        let target = model_target("你好", Language::ChineseSimplified)
            .expect("Mandarin is g2p-labeled")
            .expect("mandarin chain runs");
        assert_eq!(target.phonemes, ["n", "i", "x", "au̯"]);
        assert_eq!(target.tone, [None, Some(3), None, Some(3)]);
        // Digits would be spoken but unlabeled: refused, not silently cut.
        assert!(matches!(
            model_target("我有2个", Language::ChineseSimplified),
            Some(Err(g2p::Error::Unlabelable(_)))
        ));
    }

    /// Build a degenerate top-k where each predicted phoneme is the only
    /// alternative with probability 1.0. Lets the older alignment tests
    /// keep using the simple `align(predicted, expected)` shape.
    fn fake_topk(predicted: &[String]) -> Vec<Vec<(String, f64)>> {
        predicted.iter().map(|p| vec![(p.clone(), 1.0)]).collect()
    }

    #[test]
    fn align_basics() {
        let s = |slice: &[&str]| slice.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let cases: &[(&[&str], &[&str], usize)] = &[
            (&["a", "b", "c"], &["a", "b", "c"], 0),
            (&["a", "b", "c"], &["a", "x", "c"], 1),
            (&["a", "b"], &["a", "b", "c"], 1),
            (&[], &["a", "b"], 2),
        ];
        for (predicted, expected, want_cost) in cases {
            let predicted_v = s(predicted);
            let expected_v = s(expected);
            let topk = fake_topk(&predicted_v);
            assert_eq!(align(&predicted_v, &topk, &expected_v).0, *want_cost);
        }
    }

    #[test]
    fn align_produces_readable_ops() {
        let s = |slice: &[&str]| slice.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        // predicted: a b c d e
        // expected:  a x c   e
        // Optimal: match a, sub b→x, match c, extra d, match e  (cost 2)
        let predicted = s(&["a", "b", "c", "d", "e"]);
        let topk = fake_topk(&predicted);
        let (cost, ops) = align(&predicted, &topk, &s(&["a", "x", "c", "e"]));
        assert_eq!(cost, 2);
        let mut reconstructed_predicted = Vec::new();
        let mut reconstructed_expected = Vec::new();
        for op in &ops {
            match op {
                AlignmentOp::Match { phoneme, .. } => {
                    reconstructed_predicted.push(phoneme.clone());
                    reconstructed_expected.push(phoneme.clone());
                }
                AlignmentOp::Sub {
                    expected,
                    predicted,
                    ..
                } => {
                    reconstructed_predicted.push(predicted.clone());
                    reconstructed_expected.push(expected.clone());
                }
                AlignmentOp::Extra { predicted, .. } => {
                    reconstructed_predicted.push(predicted.clone());
                }
                AlignmentOp::Missing { expected } => {
                    reconstructed_expected.push(expected.clone());
                }
            }
        }
        assert_eq!(reconstructed_predicted, vec!["a", "b", "c", "d", "e"]);
        assert_eq!(reconstructed_expected, vec!["a", "x", "c", "e"]);
    }

    #[test]
    fn sub_carries_probabilities() {
        // Predicted "i" with prob 0.6 at position 0; "e" was the model's
        // runner-up at prob 0.3. Expected is "e", so the Sub op should
        // report predicted_prob = 0.6 and expected_prob = Some(0.3).
        let predicted = vec!["i".to_string()];
        let topk = vec![vec![("i".to_string(), 0.6), ("e".to_string(), 0.3)]];
        let expected = vec!["e".to_string()];
        let (_, ops) = align(&predicted, &topk, &expected);
        match &ops[0] {
            AlignmentOp::Sub {
                predicted_prob,
                expected_prob,
                ..
            } => {
                assert!((predicted_prob - 0.6).abs() < 1e-9);
                assert_eq!(*expected_prob, Some(0.3));
            }
            other => panic!("expected Sub, got {other:?}"),
        }
    }

    #[test]
    fn sub_reports_none_when_expected_not_in_topk() {
        let predicted = vec!["i".to_string()];
        let topk = vec![vec![("i".to_string(), 0.95)]]; // only one alt
        let expected = vec!["œ".to_string()];
        let (_, ops) = align(&predicted, &topk, &expected);
        match &ops[0] {
            AlignmentOp::Sub { expected_prob, .. } => {
                assert_eq!(*expected_prob, None);
            }
            other => panic!("expected Sub, got {other:?}"),
        }
    }

    #[test]
    fn normalize_with_topk_merges_canonical_equivalents() {
        // Raw top-k has both `r` and `ʁ`; canonicalization collapses them
        // to `ʁ`, and the merged top-k should have one `ʁ` entry with the
        // summed probability.
        let raw_phonemes = vec!["r".to_string()];
        let raw_topk = vec![vec![
            RawPhonemeAlt {
                phoneme: "r".to_string(),
                probability: 0.4,
            },
            RawPhonemeAlt {
                phoneme: "ʁ".to_string(),
                probability: 0.3,
            },
            RawPhonemeAlt {
                phoneme: "a".to_string(),
                probability: 0.2,
            },
        ]];
        let (norm, norm_topk) = normalize_with_topk(&raw_phonemes, &raw_topk, Language::French);
        assert_eq!(norm, vec!["ʁ".to_string()]);
        assert_eq!(norm_topk[0][0].0, "ʁ");
        assert!((norm_topk[0][0].1 - 0.7).abs() < 1e-9);
        assert_eq!(norm_topk[0][1].0, "a");
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
            .flat_map(|p| normalize_phonemes(p, language))
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
                .flat_map(|p| normalize_phonemes(p, language))
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
            "u ʊ m l a ʊ t v i ɪ n y b ɜ"
        );
        assert_eq!(
            phonemes(Language::German, "ß", "Straße"),
            "ɛ s t s ɛ t v i ɪ n ʃ t ɾ ɑ s ə"
        );
        assert_eq!(
            phonemes(Language::Spanish, "ñ", "niño"),
            "e ɲ e k o m o e n n i ɲ o"
        );
        assert_eq!(
            phonemes(Language::Portuguese, "ã", "pão"),
            "a t ʃ i ʊ k o m w e\u{303} j p ɐ\u{303} ʊ\u{303}"
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
            "t s ɛ k ɑ k f t s ɑ r ɪ"
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
