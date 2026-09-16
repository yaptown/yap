//! Per-run HTTP activity, excluding request preparation, retry backoff, and
//! response matrix decoding/cache writes. This measures client-side idle time,
//! not GPU utilization: an outstanding HTTP request may still be queued remotely.

use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct RequestActivity {
    started: Instant,
    state: Mutex<ActivityState>,
}

impl Default for RequestActivity {
    fn default() -> Self {
        Self {
            started: Instant::now(),
            state: Mutex::new(ActivityState::default()),
        }
    }
}

pub struct RequestActivitySnapshot {
    pub elapsed: Duration,
    pub without_request: Duration,
    pub active_requests: usize,
    pub peak_requests: usize,
    pub attempts: usize,
    pub retries: usize,
}

impl RequestActivity {
    pub fn snapshot(&self) -> RequestActivitySnapshot {
        self.state
            .lock()
            .expect("request activity lock poisoned")
            .snapshot(self.started.elapsed())
    }

    /// The guard covers one actual HTTP attempt, including response-body
    /// parsing. Drop it before retry backoff, even on errors or cancellation.
    pub(crate) fn begin(&self, retry: bool) -> RequestGuard<'_> {
        self.state
            .lock()
            .expect("request activity lock poisoned")
            .begin(self.started.elapsed(), retry);
        RequestGuard(self)
    }
}

pub(crate) struct RequestGuard<'a>(&'a RequestActivity);

impl Drop for RequestGuard<'_> {
    fn drop(&mut self) {
        self.0
            .state
            .lock()
            .expect("request activity lock poisoned")
            .end(self.0.started.elapsed());
    }
}

#[derive(Default)]
struct ActivityState {
    active: usize,
    peak: usize,
    attempts: usize,
    retries: usize,
    idle_since: Duration,
    idle: Duration,
}

impl ActivityState {
    fn begin(&mut self, now: Duration, retry: bool) {
        if self.active == 0 {
            self.idle += now - self.idle_since;
        }
        self.active += 1;
        self.peak = self.peak.max(self.active);
        self.attempts += 1;
        self.retries += usize::from(retry);
    }

    fn end(&mut self, now: Duration) {
        self.active -= 1;
        if self.active == 0 {
            self.idle_since = now;
        }
    }

    fn snapshot(&self, now: Duration) -> RequestActivitySnapshot {
        RequestActivitySnapshot {
            elapsed: now,
            without_request: self.idle
                + if self.active == 0 {
                    now - self.idle_since
                } else {
                    Duration::ZERO
                },
            active_requests: self.active,
            peak_requests: self.peak,
            attempts: self.attempts,
            retries: self.retries,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlapping_attempts_initial_fill_retry_gap_and_tail_are_accounted_once() {
        let mut state = ActivityState::default();
        let sec = Duration::from_secs;
        state.begin(sec(2), false);
        state.begin(sec(3), false);
        state.end(sec(5));
        assert_eq!(state.snapshot(sec(6)).without_request, sec(2));
        state.end(sec(7));
        assert_eq!(state.snapshot(sec(10)).without_request, sec(5));
        // A retry after four seconds of backoff is a new attempt, not a
        // continuous outstanding request obscuring those idle seconds.
        state.begin(sec(11), true);
        state.end(sec(13));
        let snapshot = state.snapshot(sec(15));
        assert_eq!(snapshot.elapsed, sec(15));
        assert_eq!(snapshot.without_request, sec(8));
        assert_eq!(snapshot.active_requests, 0);
        assert_eq!(snapshot.peak_requests, 2);
        assert_eq!(snapshot.attempts, 3);
        assert_eq!(snapshot.retries, 1);
    }

    #[test]
    fn guard_drop_closes_the_attempt() {
        let activity = RequestActivity::default();
        let guard = activity.begin(false);
        assert_eq!(activity.snapshot().active_requests, 1);
        drop(guard);
        assert_eq!(activity.snapshot().active_requests, 0);
        assert_eq!(activity.snapshot().attempts, 1);
        assert_eq!(activity.snapshot().retries, 0);
    }
}
