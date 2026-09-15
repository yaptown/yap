//! Process-local synthesis request counts; nothing is persisted or sent elsewhere.

use std::sync::atomic::{AtomicU64, Ordering};

/// Cumulative HTTP synthesis attempts across all clients in this process.
///
/// Includes failed requests, transient retries, defect retries, and caller-driven
/// fallbacks. Cache hits do not reach the HTTP call sites and are not counted.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RequestCounts {
    pub gemini: u64,
    /// Requests through the Cloud Text-to-Speech client (normally Chirp3 voices).
    pub chirp3: u64,
}

#[derive(Default)]
struct Counters {
    gemini: AtomicU64,
    chirp3: AtomicU64,
}

impl Counters {
    const fn new() -> Self {
        Self {
            gemini: AtomicU64::new(0),
            chirp3: AtomicU64::new(0),
        }
    }

    fn snapshot(&self) -> RequestCounts {
        RequestCounts {
            gemini: self.gemini.load(Ordering::Relaxed),
            chirp3: self.chirp3.load(Ordering::Relaxed),
        }
    }

    fn record(&self, backend: Backend) {
        let (name, counter) = match backend {
            Backend::Gemini => ("gemini", &self.gemini),
            Backend::Chirp3 => ("chirp3", &self.chirp3),
        };
        let count = counter.fetch_add(1, Ordering::Relaxed) + 1;
        log::info!("google-speech: synthesis request backend={name} count={count}");
    }
}

pub(crate) enum Backend {
    Gemini,
    Chirp3,
}

static COUNTERS: Counters = Counters::new();

/// Snapshot process-wide request counts. Subtract an earlier snapshot from a
/// later one, field by field, to report a run without resetting shared counters.
///
/// Counts advance immediately before sending, including transport failures, not
/// upon successful synthesis. The two fields are read independently; take the
/// final snapshot after all synthesis tasks finish for exact run totals. Other
/// concurrent synthesis in this process is included as well.
pub fn request_counts() -> RequestCounts {
    COUNTERS.snapshot()
}

pub(crate) fn record_request(backend: Backend) {
    COUNTERS.record(backend);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshots_are_cumulative_and_independent_per_backend() {
        let counters = Counters::new();
        assert_eq!(counters.snapshot(), RequestCounts::default());
        counters.record(Backend::Gemini);
        let before = counters.snapshot();
        counters.record(Backend::Gemini);
        counters.record(Backend::Gemini);
        counters.record(Backend::Chirp3);
        let after = counters.snapshot();
        assert_eq!(
            before,
            RequestCounts {
                gemini: 1,
                chirp3: 0
            }
        );
        assert_eq!(after.gemini - before.gemini, 2);
        assert_eq!(after.chirp3 - before.chirp3, 1);
        assert_eq!(counters.snapshot(), after);
    }

    #[test]
    fn concurrent_requests_do_not_lose_increments() {
        let counters = Counters::new();
        std::thread::scope(|scope| {
            for _ in 0..4 {
                scope.spawn(|| {
                    for _ in 0..100 {
                        counters.record(Backend::Gemini);
                        counters.record(Backend::Chirp3);
                    }
                });
            }
        });
        assert_eq!(
            counters.snapshot(),
            RequestCounts {
                gemini: 400,
                chirp3: 400
            }
        );
    }
}
