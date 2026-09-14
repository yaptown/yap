//! Anki's audio tags cannot send a JWT. A deck token instead guards new
//! synthesis spend; playback from the public cache needs no permission.

use axum::{
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use axum_extra::extract::Query;
use language_utils::{
    Language, TtsProvider, TtsRequest, audio_mime_type, tts_cache_filename, tts_cache_url,
};
use postgrest::Postgrest;
use serde::Deserialize;

use crate::{deck_token, service_role_client, synthesize_checked_bytes, tts_cache};

#[derive(Debug, Deserialize)]
pub struct Params {
    language: Language,
    text: String,
    #[serde(default)]
    hint: Vec<String>,
    d: String,
}

impl Params {
    // Match sentence cards exactly, including the hints: otherwise an Anki
    // fetch would pay for a clip the app has already put in the shared cache.
    fn request(self) -> TtsRequest {
        TtsRequest {
            text: self.text,
            language: self.language,
            is_ssml: false,
            instructions: None,
            speed: 1.0,
            verification_hints: self.hint,
        }
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
    let token = params.d.clone();
    let request = params.request();
    let cache_filename = tts_cache_filename(&request, &TtsProvider::ElevenLabs);
    let http = reqwest::Client::new();
    if tts_cache::exists(&http, &cache_filename).await {
        return Ok((
            StatusCode::FOUND,
            [(header::LOCATION, tts_cache_url(&cache_filename))],
        )
            .into_response());
    }

    // Playback from the cache above needed no secret; only new spend does.
    if !deck_token::configured() {
        eprintln!("anki TTS: DECK_TOKEN_SECRET is not set, refusing to synthesize");
        return Err(StatusCode::NOT_IMPLEMENTED);
    }
    let deck_id = deck_token::verify(&token).ok_or(StatusCode::FORBIDDEN)?;
    let client = service_role_client()?;
    let response = client
        .from("anki_decks")
        .select("revoked_at")
        .eq("id", deck_id.to_string())
        .execute()
        .await
        .map_err(|e| {
            eprintln!("anki TTS: failed to look up deck: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    if !response.status().is_success() {
        eprintln!(
            "anki TTS: failed to look up deck: {:?}",
            response.text().await
        );
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    #[derive(Deserialize)]
    struct Deck {
        revoked_at: Option<String>,
    }
    let decks: Vec<Deck> = response.json().await.map_err(|e| {
        eprintln!("anki TTS: failed to decode deck: {e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    if decks.first().is_none_or(|deck| deck.revoked_at.is_some()) {
        return Err(StatusCode::FORBIDDEN);
    }

    let synthesized = synthesize_checked_bytes(&http, &request, TtsProvider::ElevenLabs).await?;
    // Another request may have populated the bucket between our HEAD and the
    // pipeline's lookup. That playback did not spend anything for this deck.
    if synthesized.synthesized {
        record_in_background(client, deck_id, cache_filename, synthesized.verified);
    }
    Ok((
        [
            (header::CONTENT_TYPE, audio_mime_type(&synthesized.audio)),
            (header::CACHE_CONTROL, cache_control(synthesized.verified)),
        ],
        synthesized.audio,
    )
        .into_response())
}

/// Accounting must not delay a learner's audio. Like the shared-cache upload,
/// an unavailable database is logged rather than turning a good clip into an
/// error; both verified and salvage clips have already incurred spend.
fn record_in_background(
    client: Postgrest,
    deck_id: uuid::Uuid,
    cache_filename: String,
    verified: bool,
) {
    tokio::spawn(async move {
        let row = serde_json::json!({
            "deck_id": deck_id,
            "cache_filename": cache_filename,
            "verified": verified,
        });
        match client
            .from("anki_deck_tts_synths")
            .insert(row.to_string())
            .execute()
            .await
        {
            Ok(response) if response.status().is_success() => {}
            Ok(response) => eprintln!(
                "anki TTS: failed to record synthesis for {deck_id}/{cache_filename}: {:?}",
                response.text().await
            ),
            Err(e) => eprintln!(
                "anki TTS: failed to record synthesis for {deck_id}/{cache_filename}: {e:?}"
            ),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentence_request_shares_the_app_cache_key() {
        let params = Params {
            language: Language::French,
            text: "Élodie voit Paris.".into(),
            hint: vec!["Élodie".into(), "Paris".into()],
            d: "token".into(),
        };
        let direct = TtsRequest {
            text: "Élodie voit Paris.".into(),
            language: Language::French,
            is_ssml: false,
            instructions: None,
            speed: 1.0,
            verification_hints: vec!["Élodie".into(), "Paris".into()],
        };
        assert_eq!(
            tts_cache_filename(&params.request(), &TtsProvider::ElevenLabs),
            tts_cache_filename(&direct, &TtsProvider::ElevenLabs),
        );
    }

    #[test]
    fn hints_round_trip_through_repeated_query_keys() {
        for hints in [vec![], vec!["Élodie"], vec!["Élodie", "Paris", "Élodie"]] {
            let mut url = reqwest::Url::parse("https://example.com/anki/tts").unwrap();
            {
                let mut query = url.query_pairs_mut();
                let language = serde_json::to_value(Language::French).unwrap();
                query.append_pair("language", language.as_str().unwrap());
                query.append_pair("text", "Élodie voit Paris.");
                query.append_pair("d", "token");
                for hint in &hints {
                    query.append_pair("hint", hint);
                }
            }
            let uri = url.as_str().parse().unwrap();
            let Query(params) = Query::<Params>::try_from_uri(&uri).unwrap();
            assert_eq!(params.d, "token");
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
