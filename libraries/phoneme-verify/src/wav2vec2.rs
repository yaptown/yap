//! Yap's endpoint configuration for lexide's pronunciation client.

use anyhow::{Context, Result};
#[cfg(test)]
use lexide::pronunciation::PredictRequest;
use lexide::pronunciation::remote::PhonemizerClient;

pub use lexide::pronunciation::remote::{RequestActivity, RequestActivitySnapshot, min_samples};

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

#[cfg(test)]
pub struct Clip<'a> {
    pub samples: &'a [f32],
    pub sample_rate: u32,
    pub top_k: usize,
}

#[cfg(test)]
impl Clip<'_> {
    pub fn into_request(self) -> PredictRequest {
        lexide::pronunciation::remote::request_from_samples(
            self.samples,
            self.sample_rate,
            self.top_k,
        )
    }
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
}
