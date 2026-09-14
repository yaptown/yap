//! Gemini's prompt-driven TTS, the sibling of the Cloud Text-to-Speech client
//! in the crate root. Unlike Cloud TTS it has no per-language voice list — one
//! voice speaks every language, picked from the text itself — no SSML, and no
//! `speakingRate` knob, so delivery is steered entirely by the direction
//! line that precedes the text in the prompt.

use anyhow::{Context, Result};
use base64::Engine;

/// The model every caller uses. Preview models come and go; keeping the
/// name in one place makes the swap a one-line change.
pub const GEMINI_TTS_MODEL: &str = "gemini-3.1-flash-tts-preview";

/// The voice used unless a caller asks for another. Achernar reads cues and
/// sentences cleanly across every course language; Zephyr is the older
/// house voice.
pub const DEFAULT_VOICE: &str = "Achernar";

#[derive(Debug, Clone)]
pub struct GeminiTtsRequest {
    /// Direction for the model ("Read aloud in a warm welcoming tone"),
    /// spoken by nobody; the text follows it on its own line.
    pub instructions: String,
    /// The words to voice.
    pub text: String,
    /// A prebuilt voice name, e.g. [`DEFAULT_VOICE`].
    pub voice: String,
}

impl GeminiTtsRequest {
    /// The prompt as sent: direction, newline, text.
    pub fn prompt(&self) -> String {
        format!("{}\n{}", self.instructions, self.text)
    }
}

/// Raw audio as Gemini returns it: 16-bit linear PCM, mono, 24 kHz.
#[derive(Debug, Clone)]
pub struct GeminiTtsAudio {
    pub samples: Vec<i16>,
    pub sample_rate: u32,
}

impl GeminiTtsAudio {
    /// The samples wrapped in a WAV header, for callers that want a file.
    pub fn to_wav(&self) -> Vec<u8> {
        crate::pcm_to_wav(&self.samples, self.sample_rate)
    }

    /// The samples as an Ogg Opus stream: what Cloud TTS returns, so audio
    /// from either provider is stored and played the same way.
    pub fn to_ogg_opus(&self) -> Result<Vec<u8>> {
        crate::encode_ogg_opus(&self.samples, self.sample_rate)
    }
}

#[derive(Debug, Clone)]
pub struct GeminiTtsClient {
    api_key: String,
    http: reqwest::Client,
}

/// Why a synthesis produced nothing.
#[derive(Debug)]
pub enum GeminiTtsError {
    /// The API answered 400 for this prompt. In practice that is the model
    /// declining the request rather than a malformed one — the same prompt
    /// with another draw or wording usually goes through — so it is an
    /// outcome for this attempt, not an infrastructure failure.
    Declined(String),
    /// Transport failure, or a rate limit / server error that outlived the
    /// retry budget.
    Other(anyhow::Error),
}

impl std::fmt::Display for GeminiTtsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GeminiTtsError::Declined(body) => write!(f, "Gemini TTS declined the prompt: {body}"),
            GeminiTtsError::Other(e) => write!(f, "{e:#}"),
        }
    }
}

impl std::error::Error for GeminiTtsError {}

/// Rate limits and server errors are retried with exponential backoff, like
/// the Cloud TTS client; a 400 is not (see [`GeminiTtsError::Declined`]).
const TRANSIENT_ATTEMPTS: u32 = 6;

/// A 400 that is about the caller, not the prompt: a bad or missing API key
/// comes back as `INVALID_ARGUMENT` too, and must not be remembered as the
/// model declining this cue. Google marks those with an `API_KEY_*` reason
/// in the error details.
fn is_configuration_error(body: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| {
            v.pointer("/error/details")?.as_array().map(|details| {
                details.iter().any(|d| {
                    d.get("reason")
                        .and_then(|r| r.as_str())
                        .is_some_and(|r| r.starts_with("API_KEY"))
                })
            })
        })
        .unwrap_or(false)
        || body.contains("API key not valid")
}

impl GeminiTtsClient {
    pub fn new(api_key: String) -> Self {
        Self::with_http(api_key, reqwest::Client::new())
    }

    pub fn with_http(api_key: String, http: reqwest::Client) -> Self {
        Self { api_key, http }
    }

    /// One synthesis. `Ok(None)` when the response carried no audio, which
    /// the API does now and then without an error status; callers treat it
    /// like any other failed attempt.
    pub async fn synthesize(
        &self,
        request: &GeminiTtsRequest,
    ) -> std::result::Result<Option<GeminiTtsAudio>, GeminiTtsError> {
        let body = serde_json::json!({
            "contents": [{
                "role": "user",
                "parts": [{ "text": request.prompt() }]
            }],
            "generationConfig": {
                "responseModalities": ["audio"],
                "temperature": 1,
                "speech_config": {
                    "voice_config": {
                        "prebuilt_voice_config": { "voice_name": request.voice }
                    }
                }
            }
        });
        // The key travels in a header, not the query string, so a transport
        // error's URL never carries it into a log.
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{GEMINI_TTS_MODEL}:generateContent"
        );

        let response: serde_json::Value = {
            let mut attempt = 0;
            loop {
                attempt += 1;
                let response = self
                    .http
                    .post(&url)
                    .header("x-goog-api-key", &self.api_key)
                    .header("Content-Type", "application/json")
                    .json(&body)
                    .send()
                    .await
                    .context("Gemini TTS request failed")
                    .map_err(GeminiTtsError::Other)?;
                let status = response.status();
                if status.is_success() {
                    break response
                        .json()
                        .await
                        .context("Failed to parse Gemini TTS response JSON")
                        .map_err(GeminiTtsError::Other)?;
                }
                let text = response.text().await.unwrap_or_default();
                if status == reqwest::StatusCode::BAD_REQUEST && !is_configuration_error(&text) {
                    return Err(GeminiTtsError::Declined(text));
                }
                let transient =
                    status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error();
                if !transient || attempt == TRANSIENT_ATTEMPTS {
                    return Err(GeminiTtsError::Other(anyhow::anyhow!(
                        "Gemini TTS error ({status}): {text}"
                    )));
                }
                let delay = std::time::Duration::from_secs(1 << (attempt - 1));
                log::warn!(
                    "gemini-tts: {status} on attempt {attempt}/{TRANSIENT_ATTEMPTS}, retrying in {}s",
                    delay.as_secs()
                );
                tokio::time::sleep(delay).await;
            }
        };

        // Audio arrives as one or more inlineData parts of raw linear16 PCM.
        let mut pcm = Vec::new();
        let mut sample_rate = 24_000;
        for part in response
            .pointer("/candidates/0/content/parts")
            .and_then(|v| v.as_array())
            .map(Vec::as_slice)
            .unwrap_or_default()
        {
            let Some(data) = part.pointer("/inlineData/data").and_then(|v| v.as_str()) else {
                continue;
            };
            if let Some(rate) = part
                .pointer("/inlineData/mimeType")
                .and_then(|v| v.as_str())
                .and_then(|mime| mime.split("rate=").nth(1))
                .and_then(|rate| rate.parse::<u32>().ok())
            {
                sample_rate = rate;
            }
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(data)
                .context("Gemini TTS audio was not valid base64")
                .map_err(GeminiTtsError::Other)?;
            pcm.extend(
                bytes
                    .chunks_exact(2)
                    .map(|pair| i16::from_le_bytes([pair[0], pair[1]])),
            );
        }
        if pcm.is_empty() {
            return Ok(None);
        }
        Ok(Some(GeminiTtsAudio {
            samples: pcm,
            sample_rate,
        }))
    }
}
