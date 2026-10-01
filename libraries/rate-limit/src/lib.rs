//! Client-side pacing against a provider's per-window request quota.
//!
//! A quota such as "1,000 requests per minute" is best honoured right in
//! front of the HTTP call: callers can then fan out as wide as they like and
//! the queue forms here, instead of in a hand-tuned concurrency bound at
//! every call site. Clients that are built per call share one limiter
//! through a `static`.

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Duration;

use tokio::time::Instant;

/// Sliding window: at most `max_requests` sends in any `window`.
pub struct RateLimiter {
    max_requests: usize,
    window: Duration,
    sent_at: Mutex<VecDeque<Instant>>,
}

impl RateLimiter {
    pub const fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            sent_at: Mutex::new(VecDeque::new()),
        }
    }

    /// Stay a safe margin inside a published quota: the provider's counting
    /// is not aligned with ours, and a limiter that never trips a 429 is
    /// cheaper than the retry that follows one.
    pub const fn per_minute_with_headroom(quota: usize, headroom_percent: usize) -> Self {
        Self::new(quota * headroom_percent / 100, Duration::from_secs(60))
    }

    /// Wait for a slot, then take it.
    pub async fn acquire(&self) {
        loop {
            let wait = {
                let mut sent_at = self.sent_at.lock().unwrap();
                let now = Instant::now();
                while sent_at
                    .front()
                    .is_some_and(|&first| now.duration_since(first) >= self.window)
                {
                    sent_at.pop_front();
                }
                if sent_at.len() < self.max_requests {
                    sent_at.push_back(now);
                    return;
                }
                // Full: wake when the oldest send leaves the window.
                self.window - now.duration_since(sent_at[0])
            };
            tokio::time::sleep(wait).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(start_paused = true)]
    async fn paces_to_the_window() {
        let limiter = RateLimiter::new(3, Duration::from_secs(60));
        let start = Instant::now();
        for _ in 0..3 {
            limiter.acquire().await;
        }
        assert_eq!(start.elapsed(), Duration::ZERO);

        // The fourth send waits for the first to leave the window.
        limiter.acquire().await;
        assert_eq!(start.elapsed(), Duration::from_secs(60));

        // Then the window slides: two more go straight away, the next waits
        // for the fourth send to age out.
        limiter.acquire().await;
        limiter.acquire().await;
        assert_eq!(start.elapsed(), Duration::from_secs(60));
        limiter.acquire().await;
        assert_eq!(start.elapsed(), Duration::from_secs(120));
    }

    #[test]
    fn headroom_is_applied() {
        assert_eq!(
            RateLimiter::per_minute_with_headroom(1_000, 70).max_requests,
            700
        );
    }
}
