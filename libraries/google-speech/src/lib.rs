//! Google speech APIs: Cloud Text-to-Speech and Gemini.
//!
//! Google's TTS occasionally returns silent or truncated audio for short
//! utterances. This crate wraps the synthesize call with a retry loop and
//! reports whether the final audio passed defect checks or whether all
//! attempts were defective and we returned the last one anyway.
//!
//! The crate is application-agnostic — callers map their own language enums
//! into the `language_code` + `voice_name` strings the Google API takes.

use anyhow::{Context, Result};
use audio_codec::audio_defect;
use base64::Engine;
use serde::{Deserialize, Serialize};

pub mod gemini;
mod telemetry;

pub use telemetry::{RequestCounts, request_counts};

#[derive(Debug, Clone)]
pub struct GoogleTtsRequest {
    pub text: String,
    /// BCP-47 language tag, e.g. `"fr-FR"`.
    pub language_code: String,
    /// Google TTS voice name, e.g. `"fr-FR-Chirp3-HD-Achernar"`.
    pub voice_name: String,
    /// Playback speed multiplier (1.0 = normal).
    pub speed: f64,
    /// Treat `text` as SSML rather than plain text.
    pub is_ssml: bool,
}

/// What we got back from the API after the retry loop, plus a status
/// indicating whether defect checks passed. We *always* return audio bytes if
/// the API call succeeded — `status` lets the caller decide what to do when
/// every attempt was flagged as defective.
#[derive(Debug, Clone)]
pub struct GoogleTtsOutcome {
    /// OGG/Opus-encoded audio bytes (the encoding we always request).
    pub audio_bytes: Vec<u8>,
    /// Total attempts made, including the one whose audio we returned.
    pub attempts: usize,
    /// `Passed` if the returned audio passed defect checks. `HitLimit` if
    /// every attempt was flagged and we returned the last one regardless.
    pub status: TtsStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TtsStatus {
    /// Returned audio passed `audio_defect` after `attempts` tries.
    Passed,
    /// Hit the retry limit; the returned audio is whatever the last attempt
    /// produced, and `last_defect` is what flagged it.
    HitLimit { last_defect: &'static str },
}

impl TtsStatus {
    pub fn passed(&self) -> bool {
        matches!(self, TtsStatus::Passed)
    }
}

#[derive(Debug, Clone)]
pub struct GoogleTtsClient {
    api_key: String,
    http: reqwest::Client,
    max_attempts: usize,
}

impl GoogleTtsClient {
    pub fn new(api_key: String) -> Self {
        Self::with_http(api_key, reqwest::Client::new())
    }

    pub fn with_http(api_key: String, http: reqwest::Client) -> Self {
        Self {
            api_key,
            http,
            max_attempts: 5,
        }
    }

    /// Override the retry budget. Defaults to 5.
    pub fn with_max_attempts(mut self, n: usize) -> Self {
        self.max_attempts = n.max(1);
        self
    }

    /// Call Google TTS with the retry-on-defect loop. Returns once we either
    /// get audio that passes [`audio_defect`] or exhaust `max_attempts`.
    pub async fn synthesize(&self, request: &GoogleTtsRequest) -> Result<GoogleTtsOutcome> {
        let url = "https://texttospeech.googleapis.com/v1beta1/text:synthesize";

        let input = if request.is_ssml {
            GoogleTtsInput {
                text: None,
                ssml: Some(request.text.clone()),
            }
        } else {
            GoogleTtsInput {
                text: Some(request.text.clone()),
                ssml: None,
            }
        };
        let payload = GoogleTtsRequestBody {
            input,
            voice: GoogleTtsVoice {
                language_code: request.language_code.clone(),
                name: request.voice_name.clone(),
            },
            audio_config: GoogleTtsAudioConfig {
                audio_encoding: "OGG_OPUS".to_string(),
                speaking_rate: request.speed,
            },
        };

        let mut last_bytes: Vec<u8> = Vec::new();
        let mut last_defect: Option<&'static str> = None;
        let mut attempts = 0usize;

        while attempts < self.max_attempts {
            attempts += 1;
            let body = self.post(url, &payload).await?;
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(&body.audio_content)
                .context("Google TTS audio_content was not valid base64")?;

            match audio_defect(&bytes) {
                None => {
                    return Ok(GoogleTtsOutcome {
                        audio_bytes: bytes,
                        attempts,
                        status: TtsStatus::Passed,
                    });
                }
                Some(defect) => {
                    log::warn!(
                        "google-speech: defective audio ({defect}) on attempt {attempts}/{}, \
                         retrying",
                        self.max_attempts
                    );
                    last_defect = Some(defect);
                    last_bytes = bytes;
                }
            }
        }

        Ok(GoogleTtsOutcome {
            audio_bytes: last_bytes,
            attempts,
            status: TtsStatus::HitLimit {
                last_defect: last_defect.unwrap_or("unknown"),
            },
        })
    }
}

/// Rate limits and server errors are retried with exponential backoff (about
/// two minutes in total) and don't count against the defect budget: a
/// per-minute quota blip must not kill a run that has already synthesized
/// hundreds of clips.
const TRANSIENT_ATTEMPTS: u32 = 8;

impl GoogleTtsClient {
    async fn post(
        &self,
        url: &str,
        payload: &GoogleTtsRequestBody,
    ) -> Result<GoogleTtsResponseBody> {
        for attempt in 1..=TRANSIENT_ATTEMPTS {
            // The key travels in a header, not the query string, so a
            // transport error's URL never carries it into a log.
            telemetry::record_request(telemetry::Backend::Chirp3);
            let response = self
                .http
                .post(url)
                .header("X-Goog-Api-Key", &self.api_key)
                .header("Content-Type", "application/json")
                .json(payload)
                .send()
                .await
                .context("Google TTS request failed")?;
            let status = response.status();
            if status.is_success() {
                return response
                    .json()
                    .await
                    .context("Failed to parse Google TTS response JSON");
            }
            let body = response.text().await.unwrap_or_default();
            let transient =
                status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error();
            if !transient || attempt == TRANSIENT_ATTEMPTS {
                anyhow::bail!("Google TTS error ({status}): {body}");
            }
            let delay = std::time::Duration::from_secs(1 << (attempt - 1));
            log::warn!(
                "google-speech: {status} on attempt {attempt}/{TRANSIENT_ATTEMPTS}, retrying in {}s",
                delay.as_secs()
            );
            tokio::time::sleep(delay).await;
        }
        unreachable!("the last attempt returns or bails")
    }
}

// --- request/response wire types (private — exposed via GoogleTtsRequest) ---

#[derive(Serialize)]
struct GoogleTtsRequestBody {
    input: GoogleTtsInput,
    voice: GoogleTtsVoice,
    #[serde(rename = "audioConfig")]
    audio_config: GoogleTtsAudioConfig,
}

#[derive(Serialize)]
struct GoogleTtsInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ssml: Option<String>,
}

#[derive(Serialize)]
struct GoogleTtsVoice {
    #[serde(rename = "languageCode")]
    language_code: String,
    name: String,
}

#[derive(Serialize)]
struct GoogleTtsAudioConfig {
    #[serde(rename = "audioEncoding")]
    audio_encoding: String,
    #[serde(rename = "speakingRate")]
    speaking_rate: f64,
}

#[derive(Deserialize)]
struct GoogleTtsResponseBody {
    #[serde(rename = "audioContent")]
    audio_content: String,
}
