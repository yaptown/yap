//! Yap's endpoint configuration, audio padding, and retry policy for lexide's client.

use anyhow::{Context, Result};
use lexide::pronunciation::{PredictRequest, RawBatchResponse, remote::PhonemizerClient};

mod activity;
pub use activity::{RequestActivity, RequestActivitySnapshot};

use super::{RawPrediction, check_decoder};

const MODAL_PREDICT_URL_DEFAULT: &str =
    "https://anchpop--wav2vec2-phoneme-wav2vec2phoneme-predict.modal.run";

/// The batch endpoint every clip goes through: `WAV2VEC2_BATCH_ENDPOINT_URL`,
/// else the batch sibling of `WAV2VEC2_ENDPOINT_URL`, else production.
/// This client is for batch inference only; identity discovery uses predict.
pub fn batch_client(http: reqwest::Client) -> Result<PhonemizerClient> {
    configured_batch_client(
        http,
        std::env::var("WAV2VEC2_ENDPOINT_URL").ok().as_deref(),
        std::env::var("WAV2VEC2_BATCH_ENDPOINT_URL").ok().as_deref(),
    )
}

fn configured_batch_client(
    http: reqwest::Client,
    predict: Option<&str>,
    batch: Option<&str>,
) -> Result<PhonemizerClient> {
    if let Some(batch) = batch {
        // This client only sends batches. Reuse the explicit URL for its unused
        // predict endpoint so an arbitrary batch URL needs no predict URL.
        PhonemizerClient::with_endpoints(http, batch, batch)
    } else {
        Ok(
            PhonemizerClient::new(predict.unwrap_or(MODAL_PREDICT_URL_DEFAULT))
                .context("set WAV2VEC2_BATCH_ENDPOINT_URL for a custom single-clip endpoint")?
                .with_http_client(http),
        )
    }
}

pub(crate) fn identity_client(http: reqwest::Client) -> Result<PhonemizerClient> {
    let batch = std::env::var("WAV2VEC2_BATCH_ENDPOINT_URL").ok();
    let predict = identity_predict_url(
        std::env::var("WAV2VEC2_ENDPOINT_URL").ok().as_deref(),
        batch.as_deref(),
    )?;
    // Only identity() / check_identity() are used here, not batch inference.
    PhonemizerClient::with_endpoints(http, &predict, &predict)
}

fn identity_predict_url(predict: Option<&str>, batch: Option<&str>) -> Result<String> {
    if let Some(predict) = predict {
        return Ok(predict.to_owned());
    }
    match batch {
        Some(batch) => batch
            .strip_suffix("-predict-batch.modal.run")
            .map(|prefix| format!("{prefix}-predict.modal.run"))
            .context("set WAV2VEC2_ENDPOINT_URL for identity discovery on a custom batch endpoint"),
        None => Ok(MODAL_PREDICT_URL_DEFAULT.to_owned()),
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
}

impl Clip<'_> {
    /// Pad short clips at their native rate, then let lexide encode the samples.
    pub fn into_request(self) -> PredictRequest {
        let samples = pad_to_min_length(self.samples.to_vec(), min_samples(self.sample_rate));
        PredictRequest {
            sample_rate: self.sample_rate,
            top_k: self.top_k,
            return_frame_matrix: true,
            return_all_heads: true,
            ..PredictRequest::from_samples(&samples)
        }
    }
}

/// How many times to attempt a single Modal prediction before giving up.
/// Transient failures (cold-start 408s, rate limits, selected 5xx) are retried
/// with a linear backoff; a non-transient status fails immediately.
const MAX_ATTEMPTS: usize = 5;

/// lexide preserves reqwest's status beneath its response-body context.
/// Transport/parse failures without an HTTP status remain retryable.
fn is_transient_error(error: &anyhow::Error) -> bool {
    error
        .chain()
        .filter_map(|cause| cause.downcast_ref::<reqwest::Error>())
        .find_map(reqwest::Error::status)
        .is_none_or(|status| matches!(status.as_u16(), 408 | 425 | 429 | 500 | 502 | 503 | 504))
}

/// Send one batch with yap's retry policy. A cold-start 408 typically means
/// a short backoff lets the container finish warming up. The outer `Err`
/// is the whole request failing; an inner `Err` rejects just one clip and
/// is not retried. lexide owns the wire format and batch-size validation.
pub async fn predict_batch(
    client: &PhonemizerClient,
    requests: &[PredictRequest],
    activity: Option<&RequestActivity>,
) -> Result<Vec<Result<RawPrediction>>> {
    let mut last_err: Option<anyhow::Error> = None;
    for attempt in 1..=MAX_ATTEMPTS {
        let response = {
            let _attempt = activity.map(|activity| activity.begin(attempt > 1));
            client.predict_batch_raw(requests).await
        };
        match response {
            Ok(batch) => return split_batch(batch),
            Err(error) if !is_transient_error(&error) => return Err(error),
            Err(error) => last_err = Some(error),
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

/// Validate item views without rewriting their raw bytes or dropping unknown
/// envelope metadata. Per-item errors keep their original request positions.
pub(crate) fn split_batch(raw: RawBatchResponse) -> Result<Vec<Result<RawPrediction>>> {
    check_decoder(raw.batch.decoder_version.as_deref())?;
    Ok(raw
        .batch
        .results
        .into_iter()
        .map(|item| {
            let response = RawPrediction {
                item,
                envelope: raw.envelope.clone(),
            };
            response.decode()?;
            Ok(response)
        })
        .collect())
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
pub(crate) mod tests {
    use super::*;
    use base64::Engine;
    use std::io::{BufRead, BufReader, Read, Write};
    use std::time::Duration;

    #[test]
    fn clip_request_pads_at_native_rate_and_preserves_options() {
        for sample_rate in [16_000, 24_000, 44_100, 48_000] {
            let samples = [1.0, -0.5];
            let request = Clip {
                samples: &samples,
                sample_rate,
                top_k: 5,
            }
            .into_request();
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(&request.audio_f32_b64)
                .unwrap();
            let expected_len = sample_rate as usize * 6 / 10;
            assert_eq!(bytes.len(), expected_len * 4);
            let lead = (expected_len - samples.len()) / 2;
            assert!(bytes[..lead * 4].iter().all(|byte| *byte == 0));
            assert_eq!(&bytes[lead * 4..lead * 4 + 4], &1.0_f32.to_le_bytes());
            assert_eq!(
                &bytes[lead * 4 + 4..lead * 4 + 8],
                &(-0.5_f32).to_le_bytes()
            );
            assert!(bytes[lead * 4 + 8..].iter().all(|byte| *byte == 0));
            assert_eq!(request.sample_rate, sample_rate);
            assert_eq!(request.top_k, 5);
            assert!(request.return_frame_matrix);
            assert!(request.return_all_heads);
            assert!(!request.return_frames);
            assert!(request.language.is_none());
            assert!(request.target_phonemes.is_none());
        }
    }

    #[test]
    fn audio_transport_is_little_endian_float32() {
        assert_eq!(
            PredictRequest::from_samples(&[0.0, 1.0, -0.5]).audio_f32_b64,
            "AAAAAAAAgD8AAAC/"
        );
    }

    type RecordedRequest = (String, serde_json::Value);

    /// Serve local requests and retain the path and body for wire assertions.
    pub(crate) fn test_server(
        responses: Vec<(u16, impl ToString + Send + 'static)>,
    ) -> (String, std::thread::JoinHandle<Vec<RecordedRequest>>) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/batch", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let mut requests = Vec::new();
            for (status, response) in responses {
                let (mut socket, _) = listener.accept().unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut reader = BufReader::new(&socket);
                let mut request_line = String::new();
                reader.read_line(&mut request_line).unwrap();
                let mut length = None;
                loop {
                    let mut line = String::new();
                    assert_ne!(reader.read_line(&mut line).unwrap(), 0);
                    if line == "\r\n" {
                        break;
                    }
                    let (key, value) = line.trim().split_once(':').unwrap();
                    if key.eq_ignore_ascii_case("content-length") {
                        length = Some(value.trim().parse::<usize>().unwrap());
                    }
                }
                let mut body = vec![0; length.unwrap()];
                reader.read_exact(&mut body).unwrap();
                let request = serde_json::from_slice(&body).unwrap();
                let response = response.to_string();
                write!(socket, "HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}", response.len()).unwrap();
                requests.push((request_line, request));
            }
            requests
        });
        (url, server)
    }

    fn local_http() -> reqwest::Client {
        reqwest::Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn retry_classification_preserves_status_through_anyhow_context() {
        for (status, retry) in [
            (400, false),
            (401, false),
            (403, false),
            (404, false),
            (422, false),
            (501, false),
            (408, true),
            (425, true),
            (429, true),
            (500, true),
            (502, true),
            (503, true),
            (504, true),
            // A malformed success has a parse error without an HTTP status.
            (200, true),
        ] {
            let (url, server) = test_server(vec![(status, serde_json::json!([]))]);
            let client = configured_batch_client(local_http(), None, Some(&url)).unwrap();
            let error = client
                .predict_batch(&[PredictRequest::from_samples(&[0.0])])
                .await
                .unwrap_err()
                .context("inner")
                .context("outer");
            assert_eq!(
                is_transient_error(&error),
                retry,
                "status {status}: {error:#}"
            );
            server.join().unwrap();
        }
        let transport = local_http().get("not a URL").send().await.unwrap_err();
        assert!(is_transient_error(
            &anyhow::Error::new(transport).context("transport")
        ));
        assert!(is_transient_error(&anyhow::anyhow!("unknown failure")));
    }

    #[tokio::test]
    async fn fatal_statuses_fail_without_backoff_and_use_explicit_batch_endpoint() {
        for status in [400, 401, 403, 404, 422, 501] {
            let (url, server) =
                test_server(vec![(status, serde_json::json!({"detail": "bad clip"}))]);
            let client = configured_batch_client(
                local_http(),
                Some("invalid unused predict URL"),
                Some(&url),
            )
            .unwrap();
            let request = PredictRequest::from_samples(&[0.0, 1.0, -0.5]);
            let error = tokio::time::timeout(
                Duration::from_secs(2),
                predict_batch(&client, std::slice::from_ref(&request), None),
            )
            .await
            .expect("fatal status must not enter the five-second backoff")
            .unwrap_err();
            assert!(!is_transient_error(&error));
            assert!(format!("{error:#}").contains("bad clip"));
            let requests = server.join().unwrap();
            assert_eq!(requests[0].0, "POST /batch HTTP/1.1\r\n");
            assert_eq!(requests[0].1, serde_json::json!({"requests": [request]}));
        }
    }

    #[tokio::test]
    async fn transient_failure_retries_then_preserves_item_errors_and_metadata() {
        let (url, server) = test_server(vec![
            (503, serde_json::json!({"detail": "warming up"})),
            (
                200,
                serde_json::json!({
                    "deploy_marker": "fresh", "results": [
                        {"phonemes": []},
                        {"error": {"type": "ValueError", "message": "bad clip"}},
                    ],
                }),
            ),
        ]);
        let client = configured_batch_client(local_http(), None, Some(&url)).unwrap();
        let requests = [
            PredictRequest::from_samples(&[0.0]),
            PredictRequest::from_samples(&[1.0]),
        ];
        let start = std::time::Instant::now();
        let activity = RequestActivity::default();
        let results = tokio::time::timeout(
            Duration::from_secs(10),
            predict_batch(&client, &requests, Some(&activity)),
        )
        .await
        .unwrap()
        .unwrap();
        assert!(start.elapsed() >= Duration::from_secs(5));
        let snapshot = activity.snapshot();
        assert_eq!(snapshot.attempts, 2);
        assert_eq!(snapshot.retries, 1);
        assert_eq!(snapshot.active_requests, 0);
        assert_eq!(snapshot.peak_requests, 1);
        assert!(
            snapshot.without_request >= Duration::from_secs(5),
            "retry backoff is not HTTP activity"
        );
        assert_eq!(
            results[0]
                .as_ref()
                .unwrap()
                .decode()
                .unwrap()
                .deploy_marker
                .as_deref(),
            Some("fresh")
        );
        assert!(
            results[1]
                .as_ref()
                .unwrap_err()
                .to_string()
                .contains("ValueError: bad clip")
        );
        let wire = server.join().unwrap();
        assert_eq!(wire.len(), 2);
        assert_eq!(wire[0], wire[1]);
        assert_eq!(wire[0].1, serde_json::json!({"requests": requests}));
    }

    #[tokio::test]
    async fn client_rejects_wrong_batch_result_count() {
        let (url, server) = test_server(vec![(200, serde_json::json!({"results": []}))]);
        let client = configured_batch_client(local_http(), None, Some(&url)).unwrap();
        let error = client
            .predict_batch(&[PredictRequest::from_samples(&[0.0])])
            .await
            .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("batch returned 0 results for 1 clips")
        );
        server.join().unwrap();
    }

    #[tokio::test]
    async fn probe_and_prediction_marker_errors_match() {
        for reported in [None, Some("stale")] {
            let (url, server) = test_server(vec![(
                200,
                serde_json::json!({
                    "model_id": "m", "model_revision": "r", "deploy_marker": reported,
                }),
            )]);
            let predict = url.replace("/batch", "/predict");
            let client = PhonemizerClient::with_endpoints(local_http(), &predict, &url).unwrap();
            assert_eq!(
                client
                    .check_identity("fresh")
                    .await
                    .unwrap_err()
                    .to_string(),
                crate::check_marker(Some("fresh"), reported)
                    .unwrap_err()
                    .to_string(),
            );
            let requests = server.join().unwrap();
            assert_eq!(requests[0].0, "POST /predict HTTP/1.1\r\n");
            assert_eq!(requests[0].1, serde_json::json!({"marker_only": true}));
        }
    }

    #[tokio::test]
    async fn identity_rejects_missing_fields_and_load_error() {
        for response in [
            serde_json::json!({"deploy_marker": "fresh"}),
            serde_json::json!({"model_id": "m", "model_revision": "r", "load_error": "bad weights"}),
        ] {
            let (url, server) = test_server(vec![(200, response)]);
            let client = PhonemizerClient::with_endpoints(local_http(), &url, &url).unwrap();
            assert!(client.identity().await.is_err());
            server.join().unwrap();
        }
    }

    #[test]
    fn endpoint_configuration_preserves_defaults_and_reverse_discovery() {
        let http = local_http();
        assert!(configured_batch_client(http.clone(), None, None).is_ok());
        assert!(
            configured_batch_client(http.clone(), Some("https://x-predict.modal.run"), None)
                .is_ok()
        );
        let error = configured_batch_client(http.clone(), Some("http://localhost/single"), None)
            .unwrap_err();
        assert!(error.to_string().contains("WAV2VEC2_BATCH_ENDPOINT_URL"));
        assert!(configured_batch_client(http, None, Some("not a URL")).is_err());
        assert_eq!(
            identity_predict_url(None, None).unwrap(),
            MODAL_PREDICT_URL_DEFAULT
        );
        assert_eq!(
            identity_predict_url(None, Some("https://x-predict-batch.modal.run")).unwrap(),
            "https://x-predict.modal.run",
        );
        let error = identity_predict_url(None, Some("http://localhost/batch")).unwrap_err();
        assert!(error.to_string().contains("WAV2VEC2_ENDPOINT_URL"));
        assert_eq!(
            identity_predict_url(
                Some("http://localhost/single"),
                Some("http://localhost/batch")
            )
            .unwrap(),
            "http://localhost/single",
        );
    }

    // Talks to the production batch endpoint. Explicit opt-in only.
    #[tokio::test]
    #[ignore = "contacts the live Modal endpoint"]
    async fn batch_endpoint_round_trip() {
        let client = batch_client(reqwest::Client::new()).unwrap();
        let silence = Clip {
            samples: &[],
            sample_rate: 16_000,
            top_k: 10,
        }
        .into_request();
        let results = predict_batch(&client, &[silence.clone(), silence], None)
            .await
            .unwrap();
        assert_eq!(results.len(), 2);
        for result in results {
            assert!(
                result.unwrap().decode().unwrap().deploy_marker.is_some(),
                "batch marker not applied"
            );
        }
    }

    #[test]
    fn pad_to_min_length_pads_symmetrically() {
        assert_eq!(
            pad_to_min_length(vec![1.0, 2.0, 3.0], 5),
            vec![0.0, 1.0, 2.0, 3.0, 0.0]
        );
        let samples = vec![1.0, 2.0, 3.0];
        assert_eq!(pad_to_min_length(samples.clone(), 3), samples);
        assert_eq!(pad_to_min_length(samples.clone(), 2), samples);
        assert_eq!(pad_to_min_length(vec![1.0], 4), vec![0.0, 1.0, 0.0, 0.0]);
    }
}
