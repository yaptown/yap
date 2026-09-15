//! Wire types, audio payloads, and retrying transport for Modal's wav2vec2 endpoint.

use anyhow::Result;
use base64::Engine;
use lexide::pronunciation::{BatchResponse as ModalBatchResponse, BatchResult};
pub use lexide::pronunciation::{EmittedPhoneme, PhonemeAlternative, PredictResponse};

use super::{check_decoder, merge_metadata};

pub const MODAL_BATCH_URL_DEFAULT: &str =
    "https://anchpop--wav2vec2-phoneme-wav2vec2phoneme-predict-batch.modal.run";

/// The batch endpoint every clip goes through: `WAV2VEC2_BATCH_ENDPOINT_URL`,
/// else the batch sibling of a `WAV2VEC2_ENDPOINT_URL` single-clip endpoint
/// (the same env var the AI backend uses), else production.
pub fn batch_url() -> Result<String> {
    if let Ok(url) = std::env::var("WAV2VEC2_BATCH_ENDPOINT_URL") {
        return Ok(url);
    }
    match std::env::var("WAV2VEC2_ENDPOINT_URL") {
        Ok(single) => batch_endpoint(&single),
        Err(_) => Ok(MODAL_BATCH_URL_DEFAULT.to_string()),
    }
}

/// Modal names a class method's endpoint after the method, so the batch
/// endpoint sits beside the single-clip one.
pub fn batch_endpoint(single: &str) -> Result<String> {
    match single.strip_suffix("-predict.modal.run") {
        Some(prefix) => Ok(format!("{prefix}-predict-batch.modal.run")),
        None => anyhow::bail!(
            "set WAV2VEC2_BATCH_ENDPOINT_URL for the custom single-clip endpoint {single}"
        ),
    }
}

/// Minimum seconds we send to Modal. wav2vec2's convolutional encoder needs
/// roughly one full stride window of context (~320 samples at 16 kHz, but
/// in practice the Modal endpoint 500s on anything below several hundred
/// ms), so we pad with leading/trailing silence to stay safely above that
/// floor. 0.6 s is comfortably past every minimum we've observed.
const MIN_SECONDS: f64 = 0.6;

/// Shared with frame timing so it subtracts exactly the silence we send.
pub(crate) fn min_samples(sample_rate: u32) -> usize {
    (f64::from(sample_rate) * MIN_SECONDS).ceil() as usize
}

/// One clip for the endpoint, at its native sample rate.
pub struct Clip<'a> {
    pub samples: &'a [f32],
    pub sample_rate: u32,
    pub top_k: usize,
    pub return_frame_matrix: bool,
}

/// Pad short clips and encode the wire payload. Leave the frame-matrix flag
/// absent unless requested, just like the prediction-only cache population.
pub fn clip_payload(clip: &Clip<'_>) -> serde_json::Value {
    let min_len = min_samples(clip.sample_rate);
    let samples = pad_to_min_length(clip.samples.to_vec(), min_len);
    let mut payload = serde_json::json!({
        "audio_f32_b64": encode_audio_f32(&samples),
        "sample_rate": clip.sample_rate,
        "top_k": clip.top_k,
    });
    if clip.return_frame_matrix {
        payload["return_frame_matrix"] = true.into();
    }
    payload
}

/// How many times to attempt a single Modal prediction before giving up.
/// Transient failures (cold-start 408s, rate limits, 5xx) are retried with a
/// linear backoff; a non-transient status fails immediately.
const MAX_ATTEMPTS: usize = 5;

/// Whether an HTTP status from the Modal endpoint is worth retrying.
fn is_transient_status(status: reqwest::StatusCode) -> bool {
    matches!(status.as_u16(), 408 | 425 | 429 | 500 | 502 | 503 | 504)
}

/// POST one batch to the endpoint, retrying transient failures — cold-start
/// timeouts (408), rate limits (429), and 5xx — which are otherwise fatal
/// to a long run. A 408 typically means the container was mid-cold-start;
/// a short backoff lets it finish and the retry lands on the now-warm
/// container. The outer `Err` is the whole request failing; an inner `Err`
/// is the endpoint rejecting one clip (its `error` item), which is not
/// retried.
pub async fn predict_batch(
    http: &reqwest::Client,
    url: &str,
    payloads: Vec<serde_json::Value>,
) -> Result<Vec<Result<PredictResponse>>> {
    let count = payloads.len();
    let body = serde_json::json!({ "requests": payloads });
    let mut last_err: Option<anyhow::Error> = None;
    for attempt in 1..=MAX_ATTEMPTS {
        match http.post(url).json(&body).send().await {
            Err(e) => {
                last_err = Some(anyhow::Error::new(e).context("Modal request transport error"));
            }
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    match response.json::<ModalBatchResponse>().await {
                        Ok(batch) => return split_batch(batch, count),
                        Err(e) => {
                            last_err = Some(
                                anyhow::Error::new(e)
                                    .context("Failed to parse Modal wav2vec2 batch response"),
                            )
                        }
                    }
                } else if is_transient_status(status) {
                    let text = response.text().await.unwrap_or_default();
                    last_err = Some(anyhow::anyhow!("Modal wav2vec2 transient {status}: {text}"));
                } else {
                    // Non-retryable (e.g. 422): fail immediately.
                    let text = response.text().await.unwrap_or_default();
                    anyhow::bail!("Modal wav2vec2 error ({status}): {text}");
                }
            }
        }
        if attempt < MAX_ATTEMPTS {
            let delay = std::time::Duration::from_secs(5 * attempt as u64);
            log::warn!(
                "Modal call failed (attempt {attempt}/{MAX_ATTEMPTS}), retrying in {}s",
                delay.as_secs()
            );
            tokio::time::sleep(delay).await;
        }
    }
    Err(last_err
        .unwrap_or_else(|| anyhow::anyhow!("Modal call failed"))
        .context(format!(
            "Modal wav2vec2 endpoint failed after {MAX_ATTEMPTS} attempts"
        )))
}

/// One result per submitted clip, in order. The batch response stamps the
/// deploy marker once; each item gets it so the per-context check applies.
pub(crate) fn split_batch(
    batch: ModalBatchResponse,
    count: usize,
) -> Result<Vec<Result<PredictResponse>>> {
    if batch.results.len() != count {
        anyhow::bail!(
            "Modal wav2vec2 batch returned {} results for {count} clips",
            batch.results.len()
        );
    }
    check_decoder(batch.decoder_version.as_deref())?;
    Ok(batch
        .results
        .into_iter()
        .map(|item| {
            let mut modal = match item {
                BatchResult::Error { error } => anyhow::bail!(
                    "Modal wav2vec2 rejected the clip: {}: {}",
                    error.error_type,
                    error.message
                ),
                BatchResult::Prediction(modal) => modal,
            };
            merge_metadata("model id", &mut modal.model_id, &batch.model_id)?;
            merge_metadata(
                "model revision",
                &mut modal.model_revision,
                &batch.model_revision,
            )?;
            merge_metadata(
                "decoder",
                &mut modal.decoder_version,
                &batch.decoder_version,
            )?;
            merge_metadata(
                "deploy-marker",
                &mut modal.deploy_marker,
                &batch.deploy_marker,
            )?;
            Ok(modal)
        })
        .collect())
}

/// Mono float32 samples in little-endian order, encoded for the Modal API.
fn encode_audio_f32(samples: &[f32]) -> String {
    let bytes: Vec<u8> = samples
        .iter()
        .flat_map(|sample| sample.to_le_bytes())
        .collect();
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// Pad `samples` symmetrically with zeros to reach at least `min_len`.
/// wav2vec2 normally sees speech surrounded by silence, so leading/trailing
/// zero-padding is a no-op for phoneme prediction on the speech we *do*
/// care about — and it lifts very short clips (e.g. 0.2 s synthetic-TTS
/// renders of single syllables) above the Modal endpoint's minimum-length
/// floor.
fn pad_to_min_length(samples: Vec<f32>, min_len: usize) -> Vec<f32> {
    if samples.len() >= min_len {
        return samples;
    }
    let needed = min_len - samples.len();
    let lead = needed / 2;
    let trail = needed - lead;
    let mut padded = Vec::with_capacity(min_len);
    padded.resize(lead, 0.0);
    padded.extend_from_slice(&samples);
    padded.resize(padded.len() + trail, 0.0);
    padded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_payload_pads_at_native_rate_and_preserves_optional_flag() {
        for sample_rate in [16_000, 24_000, 44_100, 48_000] {
            let samples = [1.0, -0.5];
            let mut clip = Clip {
                samples: &samples,
                sample_rate,
                top_k: 5,
                return_frame_matrix: false,
            };
            let payload = clip_payload(&clip);
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(payload["audio_f32_b64"].as_str().unwrap())
                .unwrap();
            let expected_len = sample_rate as usize * 6 / 10;
            assert_eq!(bytes.len(), expected_len * 4);
            let lead = (expected_len - samples.len()) / 2;
            assert_eq!(&bytes[lead * 4..lead * 4 + 4], &1.0_f32.to_le_bytes());
            assert_eq!(payload["sample_rate"], sample_rate);
            assert_eq!(payload["top_k"], 5);
            assert!(payload.get("return_frame_matrix").is_none());
            clip.return_frame_matrix = true;
            assert_eq!(clip_payload(&clip)["return_frame_matrix"], true);
        }
    }

    #[test]
    fn batch_envelope_checks_count_marker_and_endpoint() {
        let response = || ModalBatchResponse {
            results: vec![BatchResult::Prediction(
                serde_json::from_value(serde_json::json!({"phonemes": []})).unwrap(),
            )],
            model_id: None,
            model_revision: None,
            decoder_version: None,
            deploy_marker: Some("new".into()),
        };
        assert!(split_batch(response(), 2).is_err());
        let item = split_batch(response(), 1).unwrap().remove(0).unwrap();
        assert_eq!(item.deploy_marker.as_deref(), Some("new"));
        assert_eq!(
            batch_endpoint("https://x-predict.modal.run").unwrap(),
            "https://x-predict-batch.modal.run"
        );
        assert!(batch_endpoint("http://localhost/single").is_err());
    }

    #[test]
    fn audio_transport_is_little_endian_float32() {
        assert_eq!(encode_audio_f32(&[0.0, 1.0, -0.5]), "AAAAAAAAgD8AAAC/");
    }

    // Talks to the production batch endpoint: run with `--ignored` after
    // changing the wire format.
    #[tokio::test]
    #[ignore]
    async fn batch_endpoint_round_trip() {
        let http = reqwest::Client::new();
        let silence = clip_payload(&Clip {
            samples: &[],
            sample_rate: 16_000,
            top_k: 10,
            return_frame_matrix: false,
        });
        let results = predict_batch(&http, &batch_url().unwrap(), vec![silence.clone(), silence])
            .await
            .unwrap();
        assert_eq!(results.len(), 2);
        for result in results {
            let modal = result.unwrap();
            assert!(modal.deploy_marker.is_some(), "batch marker not applied");
        }
    }

    #[test]
    fn pad_to_min_length_pads_symmetrically() {
        let s = vec![1.0_f32, 2.0, 3.0];
        // 5 needed: 1 lead, 1 trail (3 + 2 = 5)
        let padded = pad_to_min_length(s, 5);
        assert_eq!(padded, vec![0.0, 1.0, 2.0, 3.0, 0.0]);
        // No-op when already at length.
        let s = vec![1.0_f32, 2.0, 3.0];
        let padded = pad_to_min_length(s.clone(), 3);
        assert_eq!(padded, s);
        // Odd needed: extra sample goes on the trailing side.
        let s = vec![1.0_f32];
        let padded = pad_to_min_length(s, 4);
        assert_eq!(padded, vec![0.0, 1.0, 0.0, 0.0]);
    }
}
