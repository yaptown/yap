//! Synthesis for a deck being built, unauthenticated like the other /tts routes.
//! Unlike them it is a GET returning the audio bytes, and it checks clips with
//! the burst-friendly Cloudflare-only transcriber.

use axum::{
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use axum_extra::extract::Query;
use language_utils::{Language, TtsProvider, TtsRequest, audio_mime_type};
use serde::Deserialize;

use crate::{HTTP, synthesize_checked_bytes, tts_verify::Transcribers};

#[derive(Debug, Deserialize)]
pub struct Params {
    language: Language,
    text: String,
    #[serde(default)]
    hint: Vec<String>,
}

impl Params {
    fn request(self) -> TtsRequest {
        TtsRequest::plain(self.language, &self.text, self.hint)
    }
}

fn cache_control(verified: bool) -> &'static str {
    if verified {
        "public, max-age=31536000, immutable"
    } else {
        "no-store"
    }
}

pub async fn tts(Query(params): Query<Params>) -> Result<Response, StatusCode> {
    let request = params.request();
    let http = &*HTTP;
    // A deck export fires hundreds of these in minutes; see `Transcribers`.
    // ElevenLabs is the only fallback: Google's voice would be a step down
    // for a card the learner keeps.
    let synthesized = synthesize_checked_bytes(
        http,
        &request,
        TtsProvider::Gemini,
        &[TtsProvider::ElevenLabs],
        Transcribers::CloudflareOnly,
    )
    .await?;
    Ok((
        [
            (header::CONTENT_TYPE, audio_mime_type(&synthesized.audio)),
            (header::CACHE_CONTROL, cache_control(synthesized.verified)),
        ],
        synthesized.audio,
    )
        .into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hints_round_trip_through_repeated_query_keys() {
        for hints in [vec![], vec!["Élodie"], vec!["Élodie", "Paris", "Élodie"]] {
            let mut url = reqwest::Url::parse("https://example.com/anki/tts").unwrap();
            {
                let mut query = url.query_pairs_mut();
                let language = serde_json::to_value(Language::French).unwrap();
                query.append_pair("language", language.as_str().unwrap());
                query.append_pair("text", "Élodie voit Paris.");
                for hint in &hints {
                    query.append_pair("hint", hint);
                }
            }
            let uri = url.as_str().parse().unwrap();
            let Query(params) = Query::<Params>::try_from_uri(&uri).unwrap();
            let request = params.request();
            assert_eq!(request.text, "Élodie voit Paris.");
            assert_eq!(request.language, Language::French);
            assert_eq!(request.verification_hints, hints);
        }
    }

    #[test]
    fn only_verified_audio_is_cacheable() {
        assert_eq!(cache_control(true), "public, max-age=31536000, immutable");
        assert_eq!(cache_control(false), "no-store");
    }
}
