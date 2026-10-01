//! Process-wide pacing for the Gemini API. A [`GeminiClient`] is built per
//! call in several places, so the budget is a static shared by all of them
//! rather than a field on the client.
//!
//! [`GeminiClient`]: crate::gemini::GeminiClient

use rate_limit::RateLimiter;

/// Google counts 1,000 requests per minute for `gemini-3.8-flash-tts` on
/// paid tier 3 (Cloud Console, 2026-10-01). A 429 still retries with
/// backoff, but the limiter means it should not happen.
static TTS: RateLimiter = RateLimiter::per_minute_with_headroom(1_000, 70);

/// Wait for a slot to send one request to `model`. Models without a known
/// quota are not paced; they are low-volume judges rather than bulk callers.
pub(crate) async fn acquire(model: &str) {
    if model == crate::gemini::GEMINI_TTS_MODEL {
        TTS.acquire().await;
    }
}
