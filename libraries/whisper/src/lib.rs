//! Hosted Whisper transcription, without application-specific gating or racing.

use anyhow::{Context, Result, bail};
use base64::Engine;
use language_utils::Language;
use serde::Deserialize;

/// `large-v3-turbo` is the variant that exposes `initial_prompt`, and it reads
/// proper nouns markedly better than OpenAI's hosted `whisper-1` — on our
/// fixture it transcribed "Baxter" correctly with no conditioning at all,
/// where `whisper-1` produced "backstairs". Cloudflare serves it as
/// `@cf/openai/` + this name, Groq under the bare name.
pub const MODEL: &str = "whisper-large-v3-turbo";

/// Whisper takes ISO-639-1 codes, including one code for both Chinese scripts.
pub fn language_code(language: Language) -> &'static str {
    match language {
        Language::English => "en",
        Language::French => "fr",
        Language::German => "de",
        Language::Spanish => "es",
        Language::Italian => "it",
        Language::Portuguese => "pt",
        Language::Russian => "ru",
        Language::Japanese => "ja",
        Language::Korean => "ko",
        Language::Thai => "th",
        Language::Hindi => "hi",
        Language::ChineseSimplified | Language::ChineseTraditional => "zh",
    }
}

pub struct TranscribeRequest<'a> {
    pub audio: &'a [u8],
    pub language: &'a str,
    /// Vocabulary, never a transcript: Cloudflare's `initial_prompt` / Groq's
    /// `prompt` supplies names without dictating what was said. `prefix` would
    /// force the output to start with our text, hiding sentence-initial errors.
    pub vocabulary: &'a [String],
    /// False for isolated utterances, where previous text invites hallucination
    /// loops; true for continuous film windows (Whisper's default).
    /// Groq's plain transcription endpoint does not expose this option.
    pub condition_on_previous_text: bool,
}

#[derive(Debug, PartialEq)]
pub struct Transcript {
    pub text: String,
    pub words: Vec<Word>,
}

#[derive(Debug, PartialEq)]
pub struct Word {
    pub text: String,
    pub start_s: f64,
    pub end_s: f64,
}

/// The Workers AI account to transcribe against.
///
/// Read once, at the top of a run, because the alternative is worse than it
/// looks: reading the environment per window turns a missing variable into a
/// per-window failure, every window of a film fails, and the film is recorded
/// `undecided` — a durable "we could not verify this" written by a typo.
/// Holding the credentials in a value the transcriber cannot be called
/// without makes that unrepresentable.
#[derive(Clone)]
pub struct CloudflareWhisper {
    account_id: String,
    token: String,
    http: reqwest::Client,
}

impl CloudflareWhisper {
    pub fn new(account_id: String, token: String, http: reqwest::Client) -> Self {
        Self {
            account_id,
            token,
            http,
        }
    }

    pub fn from_env(http: reqwest::Client) -> Result<Self> {
        Ok(Self::new(
            std::env::var("CLOUDFLARE_ACCOUNT_ID").context("CLOUDFLARE_ACCOUNT_ID not set")?,
            std::env::var("CLOUDFLARE_API_TOKEN").context("CLOUDFLARE_API_TOKEN not set")?,
            http,
        ))
    }

    pub async fn transcribe(&self, request: &TranscribeRequest<'_>) -> Result<Transcript> {
        let mut body = serde_json::json!({
            "audio": base64::engine::general_purpose::STANDARD.encode(request.audio),
            "task": "transcribe",
            "language": request.language,
            "condition_on_previous_text": request.condition_on_previous_text,
        });
        if !request.vocabulary.is_empty() {
            body["initial_prompt"] = request.vocabulary.join(", ").into();
        }
        let response = self
            .http
            .post(format!(
                "https://api.cloudflare.com/client/v4/accounts/{}/ai/run/@cf/openai/{MODEL}",
                self.account_id
            ))
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await?;
        let envelope: CloudflareEnvelope = response_json(response).await?;
        Ok(envelope.into())
    }
}

#[derive(Deserialize)]
struct CloudflareEnvelope {
    result: CloudflareResult,
}

#[derive(Deserialize)]
struct CloudflareResult {
    text: String,
    #[serde(default)]
    segments: Vec<Segment>,
}

#[derive(Deserialize)]
struct Segment {
    #[serde(default)]
    words: Vec<CloudflareWord>,
}

#[derive(Deserialize)]
struct CloudflareWord {
    word: String,
    start: f64,
    #[serde(default)]
    end: Option<f64>,
}

impl From<CloudflareEnvelope> for Transcript {
    fn from(envelope: CloudflareEnvelope) -> Self {
        Self {
            text: envelope.result.text,
            words: envelope
                .result
                .segments
                .into_iter()
                .flat_map(|s| s.words)
                .map(|w| Word {
                    text: w.word,
                    start_s: w.start,
                    end_s: w.end.unwrap_or(w.start).max(w.start),
                })
                .collect(),
        }
    }
}

#[derive(Clone)]
pub struct GroqWhisper {
    api_key: String,
    http: reqwest::Client,
}

impl GroqWhisper {
    pub fn new(api_key: String, http: reqwest::Client) -> Self {
        Self { api_key, http }
    }

    pub fn from_env(http: reqwest::Client) -> Result<Self> {
        Ok(Self::new(
            std::env::var("GROQ_API_KEY").context("GROQ_API_KEY not set")?,
            http,
        ))
    }

    /// Groq's plain JSON response has no word timestamps; `words` is empty.
    pub async fn transcribe(&self, request: &TranscribeRequest<'_>) -> Result<Transcript> {
        // Groq sniffs the container from the part's filename extension, so it has
        // to match the bytes: Google returns Ogg Opus, Gemini WAV, the rest MP3.
        let filename = if request.audio.starts_with(b"RIFF") {
            "audio.wav"
        } else if request.audio.starts_with(b"OggS") {
            "audio.ogg"
        } else {
            "audio.mp3"
        };
        let mut form = reqwest::multipart::Form::new()
            .part(
                "file",
                reqwest::multipart::Part::bytes(request.audio.to_vec()).file_name(filename),
            )
            .text("model", MODEL)
            .text("language", request.language.to_owned())
            .text("response_format", "json");
        if !request.vocabulary.is_empty() {
            form = form.text("prompt", request.vocabulary.join(", "));
        }
        let response = self
            .http
            .post("https://api.groq.com/openai/v1/audio/transcriptions")
            .bearer_auth(&self.api_key)
            .multipart(form)
            .send()
            .await?;
        let result: GroqResult = response_json(response).await?;
        Ok(Transcript {
            text: result.text,
            words: vec![],
        })
    }
}

#[derive(Deserialize)]
struct GroqResult {
    text: String,
}

async fn response_json<T: serde::de::DeserializeOwned>(response: reqwest::Response) -> Result<T> {
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await?;
        bail!("Whisper returned {status}: {body}");
    }
    response
        .json()
        .await
        .context("Whisper response was not JSON")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cloudflare_words() {
        let envelope: CloudflareEnvelope = serde_json::from_str(r#"{"result":{"text":"Bonjour ici","segments":[{"words":[{"word":"Bonjour","start":1.2,"end":1.8},{"word":"ici","start":2.0},{"word":"!","start":3.0,"end":2.5}]},{}]}}"#).unwrap();
        let transcript = Transcript::from(envelope);
        assert_eq!(transcript.text, "Bonjour ici");
        assert_eq!(
            transcript.words,
            vec![
                Word {
                    text: "Bonjour".into(),
                    start_s: 1.2,
                    end_s: 1.8
                },
                Word {
                    text: "ici".into(),
                    start_s: 2.0,
                    end_s: 2.0
                },
                Word {
                    text: "!".into(),
                    start_s: 3.0,
                    end_s: 3.0
                },
            ]
        );
    }

    #[test]
    fn cloudflare_text_only() {
        let envelope: CloudflareEnvelope =
            serde_json::from_str(r#"{"result":{"text":"Bonjour"}}"#).unwrap();
        assert_eq!(
            Transcript::from(envelope),
            Transcript {
                text: "Bonjour".into(),
                words: vec![]
            }
        );
    }

    #[test]
    fn chinese_scripts_share_language_code() {
        assert_eq!(language_code(Language::ChineseSimplified), "zh");
        assert_eq!(language_code(Language::ChineseTraditional), "zh");
    }
}
