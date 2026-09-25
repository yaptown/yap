//! Gemini's native generateContent client and Interactions TTS, the sibling of
//! the Cloud Text-to-Speech client in the crate root. Unlike Cloud TTS it has
//! no per-language voice list — one voice speaks every language, picked from
//! the text itself — no SSML, and no `speakingRate` knob. Speech metadata steers
//! delivery separately from the transcript, which is spoken verbatim.

use anyhow::{Context, Result};
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// One input part. Binary data is base64-encoded only at the wire boundary.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Part {
    Text(String),
    InlineData {
        #[serde(rename = "mimeType")]
        mime_type: String,
        #[serde(serialize_with = "serialize_base64")]
        data: Vec<u8>,
    },
}

fn serialize_base64<S: serde::Serializer>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&base64::engine::general_purpose::STANDARD.encode(bytes))
}

/// The desired output; callers cannot mix incompatible generation settings.
#[derive(Debug, Clone)]
pub enum Output {
    /// Plain text.
    Text,
    /// JSON conforming to a Gemini response schema.
    Json { schema: Value },
}

impl Serialize for Output {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Text => json!({}),
            Self::Json { schema } => json!({
                "responseMimeType": "application/json",
                "responseSchema": schema,
            }),
        }
        .serialize(serializer)
    }
}

/// A single user turn. The model belongs in the URL, not the JSON body.
#[derive(Debug, Clone)]
pub struct GenerateContent {
    pub model: String,
    pub parts: Vec<Part>,
    pub output: Output,
}

impl Serialize for GenerateContent {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        json!({
            "contents": [{ "role": "user", "parts": self.parts }],
            "generationConfig": self.output,
        })
        .serialize(serializer)
    }
}

/// Parts from the first candidate; wire-level response details stay private.
#[derive(Debug, Deserialize)]
#[serde(from = "ResponseEnvelope")]
pub struct GeminiResponse {
    parts: Vec<ResponsePart>,
}

#[derive(Debug, Deserialize)]
struct ResponseEnvelope {
    #[serde(default)]
    candidates: Vec<ResponseCandidate>,
}

#[derive(Debug, Deserialize)]
struct ResponseCandidate {
    #[serde(default)]
    content: ResponseContent,
}

#[derive(Debug, Default, Deserialize)]
struct ResponseContent {
    #[serde(default)]
    parts: Vec<WireResponsePart>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WireResponsePart {
    text: Option<String>,
    inline_data: Option<InlineData>,
}

#[derive(Debug)]
enum ResponsePart {
    Text(String),
    InlineData { mime_type: String, data: Vec<u8> },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InlineData {
    mime_type: String,
    #[serde(deserialize_with = "deserialize_base64")]
    data: Vec<u8>,
}

fn deserialize_base64<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<u8>, D::Error> {
    let encoded = String::deserialize(deserializer)?;
    base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|error| {
            serde::de::Error::custom(format!("Gemini inline data was not valid base64: {error}"))
        })
}

impl From<ResponseEnvelope> for GeminiResponse {
    fn from(response: ResponseEnvelope) -> Self {
        Self {
            parts: response
                .candidates
                .into_iter()
                .next()
                .into_iter()
                .flat_map(|candidate| candidate.content.parts)
                .flat_map(|part| {
                    part.text
                        .map(ResponsePart::Text)
                        .into_iter()
                        .chain(part.inline_data.map(|inline| ResponsePart::InlineData {
                            mime_type: inline.mime_type,
                            data: inline.data,
                        }))
                })
                .collect(),
        }
    }
}

impl GeminiResponse {
    /// The first text part, if the model supplied one.
    pub fn text(&self) -> Option<&str> {
        self.parts.iter().find_map(|part| match part {
            ResponsePart::Text(text) => Some(text.as_str()),
            ResponsePart::InlineData { .. } => None,
        })
    }

    /// Each inline part's MIME type and decoded bytes, in response order.
    /// Base64 is validated and decoded while parsing the response.
    pub fn inline_data(&self) -> impl Iterator<Item = (&str, &[u8])> {
        self.parts.iter().filter_map(|part| match part {
            ResponsePart::InlineData { mime_type, data } => {
                Some((mime_type.as_str(), data.as_slice()))
            }
            ResponsePart::Text(_) => None,
        })
    }
}

/// The model every TTS caller uses. Preview models come and go; keeping the
/// name in one place makes the swap a one-line change.
pub const GEMINI_TTS_MODEL: &str = "gemini-3.8-flash-tts";

/// The voice used unless a caller asks for another. Achernar reads cues and
/// sentences cleanly across every course language; Zephyr is the older
/// house voice.
pub const DEFAULT_VOICE: &str = "Achernar";

#[derive(Debug, Clone)]
pub struct GeminiTtsRequest {
    /// Delivery tone and pace, separate from the verbatim transcript.
    pub style: String,
    /// The words to voice verbatim.
    pub text: String,
    /// A prebuilt voice name, e.g. [`DEFAULT_VOICE`].
    pub voice: String,
}

impl Serialize for GeminiTtsRequest {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut content = json!({"type": "text", "text": self.text});
        if !self.style.is_empty() {
            content["annotations"] = json!([{"type": "speech_metadata", "style": self.style}]);
        }
        json!({
            "model": GEMINI_TTS_MODEL,
            "input": [{"type": "user_input", "content": [content]}],
            "response_format": {"type": "audio", "mime_type": "audio/l16", "sample_rate": 24000},
            "generation_config": {"speech_config": [{"voice": self.voice}]},
        })
        .serialize(serializer)
    }
}

#[derive(Debug, Deserialize)]
struct InteractionResponse {
    #[serde(default)]
    steps: Vec<InteractionStep>,
}

#[derive(Debug, Deserialize)]
struct InteractionStep {
    #[serde(default)]
    content: Vec<InteractionContent>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum InteractionContent {
    #[serde(rename = "audio")]
    Audio {
        mime_type: String,
        sample_rate: u32,
        #[serde(deserialize_with = "deserialize_base64")]
        data: Vec<u8>,
    },
    #[serde(other)]
    Other,
}

impl InteractionResponse {
    /// The audio parts, joined. We ask for raw PCM, and anything else is an
    /// error rather than something to decode as PCM: when the model's
    /// default quietly became WAV, its header and trailing C2PA chunk played
    /// as a click and a burst of static.
    fn audio(self) -> Result<Option<GeminiTtsAudio>> {
        let mut audio: Option<GeminiTtsAudio> = None;
        for content in self.steps.into_iter().flat_map(|step| step.content) {
            let InteractionContent::Audio {
                mime_type,
                sample_rate,
                data,
            } = content
            else {
                continue;
            };
            anyhow::ensure!(
                mime_type.starts_with("audio/l16"),
                "Unexpected Gemini audio MIME type: {mime_type}"
            );
            audio
                .get_or_insert_with(|| GeminiTtsAudio {
                    samples: Vec::new(),
                    sample_rate,
                })
                .samples
                .extend(
                    data.chunks_exact(2)
                        .map(|pair| i16::from_le_bytes([pair[0], pair[1]])),
                );
        }
        Ok(audio.filter(|audio| !audio.samples.is_empty()))
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
        audio_codec::pcm_to_wav(&self.samples, self.sample_rate)
    }

    /// The samples as an Ogg Opus stream: what Cloud TTS returns, so audio
    /// from either provider is stored and played the same way.
    pub fn to_ogg_opus(&self) -> Result<Vec<u8>> {
        audio_codec::encode_ogg_opus(&self.samples, self.sample_rate)
    }
}

#[derive(Debug, Clone)]
pub struct GeminiClient {
    api_key: String,
    http: reqwest::Client,
}

/// Why a generation failed.
#[derive(Debug)]
pub enum GeminiError {
    /// The API answered 400 for this prompt. In practice that is the model
    /// declining the request rather than a malformed one — the same prompt
    /// with another draw or wording usually goes through — so it is an
    /// outcome for this attempt, not an infrastructure failure.
    Declined(String),
    /// Transport failure, or a rate limit / server error that outlived the
    /// retry budget.
    Other(anyhow::Error),
}

impl std::fmt::Display for GeminiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GeminiError::Declined(body) => write!(f, "Gemini declined the prompt: {body}"),
            GeminiError::Other(e) => write!(f, "{e:#}"),
        }
    }
}

impl std::error::Error for GeminiError {}

/// Transport errors, rate limits and server errors retry with exponential
/// backoff, like the Cloud TTS client; a 400 is not (see [`GeminiError::Declined`]).
const TRANSIENT_ATTEMPTS: u32 = 6;

/// A 400 that is about the caller, not the prompt: a bad or missing API key
/// comes back as `INVALID_ARGUMENT` too, and must not be remembered as the
/// model declining this prompt. Google marks those with an `API_KEY_*` reason
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

impl GeminiClient {
    pub fn new(api_key: String) -> Self {
        Self::with_http(api_key, reqwest::Client::new())
    }

    pub fn with_http(api_key: String, http: reqwest::Client) -> Self {
        Self { api_key, http }
    }

    /// One native generateContent call, with retries for transient failures.
    pub async fn generate(
        &self,
        request: &GenerateContent,
    ) -> std::result::Result<GeminiResponse, GeminiError> {
        // The key travels in a header, not the query string, so a transport
        // error's URL never carries it into a log.
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            request.model
        );
        self.post_json(&url, request).await
    }

    async fn post_json<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        request: &impl Serialize,
    ) -> std::result::Result<T, GeminiError> {
        let mut attempt = 0;
        loop {
            attempt += 1;
            crate::telemetry::record_request(crate::telemetry::Backend::Gemini);
            let response = self
                .http
                .post(url)
                .header("x-goog-api-key", &self.api_key)
                .header("Content-Type", "application/json")
                .json(request)
                .send()
                .await;
            let error = match response {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        return response
                            .json()
                            .await
                            .context("Failed to parse Gemini response JSON")
                            .map_err(GeminiError::Other);
                    }
                    let text = response.text().await.unwrap_or_default();
                    if status == reqwest::StatusCode::BAD_REQUEST && !is_configuration_error(&text)
                    {
                        return Err(GeminiError::Declined(text));
                    }
                    let error = anyhow::anyhow!("Gemini error ({status}): {text}");
                    let transient = status == reqwest::StatusCode::TOO_MANY_REQUESTS
                        || status.is_server_error();
                    if !transient {
                        return Err(GeminiError::Other(error));
                    }
                    error
                }
                Err(error) => anyhow::Error::from(error).context("Gemini request failed"),
            };
            if attempt == TRANSIENT_ATTEMPTS {
                return Err(GeminiError::Other(error));
            }
            let delay = std::time::Duration::from_secs(1 << (attempt - 1));
            log::warn!(
                "gemini: {error:#} on attempt {attempt}/{TRANSIENT_ATTEMPTS}, retrying in {}s",
                delay.as_secs()
            );
            tokio::time::sleep(delay).await;
        }
    }

    /// One synthesis. `Ok(None)` when the response carried no audio, which
    /// the API does now and then without an error status; callers treat it
    /// like any other failed attempt.
    pub async fn synthesize(
        &self,
        request: &GeminiTtsRequest,
    ) -> std::result::Result<Option<GeminiTtsAudio>, GeminiError> {
        let response: InteractionResponse = self
            .post_json(
                "https://generativelanguage.googleapis.com/v1beta/interactions",
                request,
            )
            .await?;
        response.audio().map_err(GeminiError::Other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_text_request() {
        let request = GenerateContent {
            model: "test-model".into(),
            parts: vec![Part::Text("Hello\n世界".into())],
            output: Output::Text,
        };
        assert_eq!(
            serde_json::to_value(request).unwrap(),
            json!({
                "contents": [{"role": "user", "parts": [{"text": "Hello\n世界"}]}],
                "generationConfig": {},
            })
        );
    }

    #[test]
    fn serializes_json_request_with_binary_input() {
        let request = GenerateContent {
            model: "judge-model".into(),
            parts: vec![
                Part::Text("Listen".into()),
                Part::InlineData {
                    mime_type: "audio/ogg".into(),
                    data: vec![0, 1, 254, 255],
                },
            ],
            output: Output::Json {
                schema: json!({"type": "OBJECT", "properties": {"ok": {"type": "BOOLEAN"}}, "required": ["ok"]}),
            },
        };
        assert_eq!(
            serde_json::to_value(request).unwrap(),
            json!({
                "contents": [{"role": "user", "parts": [
                    {"text": "Listen"},
                    {"inlineData": {"mimeType": "audio/ogg", "data": "AAH+/w=="}},
                ]}],
                "generationConfig": {
                    "responseMimeType": "application/json",
                    "responseSchema": {"type": "OBJECT", "properties": {"ok": {"type": "BOOLEAN"}}, "required": ["ok"]},
                },
            })
        );
    }

    #[test]
    fn serializes_speech_metadata_separately_from_transcript() {
        let mut request = GeminiTtsRequest {
            style: "warm and welcoming".into(),
            text: "Trouvé.".into(),
            voice: "Zephyr".into(),
        };
        assert_eq!(
            serde_json::to_value(&request).unwrap(),
            json!({
                "model": GEMINI_TTS_MODEL,
                "input": [{"type": "user_input", "content": [{"type": "text", "text": "Trouvé.",
                    "annotations": [{"type": "speech_metadata", "style": "warm and welcoming"}]}]}],
                "response_format": {"type": "audio", "mime_type": "audio/l16", "sample_rate": 24000},
                "generation_config": {"speech_config": [{"voice": "Zephyr"}]},
            })
        );
        request.style.clear();
        assert_eq!(
            serde_json::to_value(&request).unwrap()["input"][0]["content"][0],
            json!({"type": "text", "text": "Trouvé."})
        );
    }

    #[test]
    fn parses_interaction_pcm() {
        let response: InteractionResponse = serde_json::from_value(json!({
            "steps": [{"type": "model_output", "content": [
                {"type": "text", "text": "ignored"},
                {"type": "audio", "data": "AAH+/w==", "channels": 1,
                 "sample_rate": 24000, "mime_type": "audio/l16; rate=24000; channels=1"}
            ]}]
        }))
        .unwrap();
        let audio = response.audio().unwrap().unwrap();
        assert_eq!(audio.sample_rate, 24000);
        assert_eq!(audio.samples, [256, -2]);
    }

    #[test]
    fn rejects_non_pcm_audio() {
        for mime in ["audio/wav", "audio/ogg"] {
            let response: InteractionResponse = serde_json::from_value(json!({
                "steps": [{"content": [{"type": "audio", "data": "AAH+/w==", "mime_type": mime, "sample_rate": 24000}]}]
            }))
            .unwrap();
            assert_eq!(
                response.audio().unwrap_err().to_string(),
                format!("Unexpected Gemini audio MIME type: {mime}")
            );
        }
    }

    #[test]
    fn interaction_without_audio_is_none() {
        for fixture in [
            json!({}),
            json!({"steps": [{"content": [{"type": "text", "text": "declined"}]}]}),
        ] {
            let response: InteractionResponse = serde_json::from_value(fixture).unwrap();
            assert!(response.audio().unwrap().is_none());
        }
    }

    #[test]
    fn serializes_empty_inline_data() {
        assert_eq!(
            serde_json::to_value(Part::InlineData {
                mime_type: "audio/ogg".into(),
                data: vec![],
            })
            .unwrap(),
            json!({"inlineData": {"mimeType": "audio/ogg", "data": ""}})
        );
    }

    #[test]
    fn decodes_response_fixture() {
        let response: GeminiResponse = serde_json::from_str(
            r#"{
            "candidates": [{
                "content": {"role": "model", "parts": [
                    {"inlineData": {"mimeType": "audio/L16;rate=24000", "data": "AAH+/w=="}},
                    {"text": "Hello 世界"},
                    {"inlineData": {"mimeType": "audio/ogg", "data": "AgM="}},
                    {"text": "another text part"}
                ]},
                "finishReason": "STOP"
            }, {
                "content": {"parts": [{"text": "ignored alternative"}]}
            }],
            "usageMetadata": {"totalTokenCount": 10}
        }"#,
        )
        .unwrap();
        assert_eq!(response.text(), Some("Hello 世界"));
        assert_eq!(
            response.inline_data().collect::<Vec<_>>(),
            vec![
                ("audio/L16;rate=24000", &[0, 1, 254, 255][..]),
                ("audio/ogg", &[2, 3][..])
            ]
        );
    }

    #[test]
    fn empty_or_blocked_responses_have_no_output() {
        for fixture in [
            json!({"promptFeedback": {"blockReason": "SAFETY"}}),
            json!({"candidates": []}),
            json!({"candidates": [{"finishReason": "SAFETY"}]}),
            json!({"candidates": [{"content": {}}]}),
            json!({"candidates": [{"content": {"parts": []}}]}),
            json!({"candidates": [{"content": {"parts": [{"functionCall": {"name": "unknown"}}]}}]}),
        ] {
            let response: GeminiResponse = serde_json::from_value(fixture).unwrap();
            assert_eq!(response.text(), None);
            assert_eq!(response.inline_data().count(), 0);
        }
    }

    #[test]
    fn invalid_inline_base64_is_an_error() {
        let error = serde_json::from_value::<GeminiResponse>(json!({
            "candidates": [{"content": {"parts": [
                {"inlineData": {"mimeType": "audio/ogg", "data": "not base64!"}}
            ]}}]
        }))
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("Gemini inline data was not valid base64")
        );
    }

    #[test]
    fn configuration_errors_are_not_prompt_declines() {
        assert!(is_configuration_error(
            r#"{"error":{"details":[{"reason":"API_KEY_INVALID"}]}}"#
        ));
        assert!(is_configuration_error("API key not valid"));
        assert!(!is_configuration_error(
            r#"{"error":{"message":"Prompt declined"}}"#
        ));
        assert!(!is_configuration_error("not JSON"));
    }
}
