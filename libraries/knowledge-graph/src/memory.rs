//! Math Academy's per-concept memory model.
//!
//! A concept carries a repetition count `reps` (how far along spaced
//! repetition it is, fractional) and a `memory` level that halves every
//! interval, where the interval grows geometrically with `reps`. A review
//! moves both by its credit:
//!
//! ```text
//! reps   -> max(0, reps + credit)
//! memory -> clamp(memory_now + credit, 0, 1)
//! ```
//!
//! The concept comes due when memory decays to [`DUE_MEMORY`], which is one
//! interval after a perfect review. Positive credit earned early (while memory
//! is still above that level) is discounted, because a too-early repetition
//! does little for long-term retention. The learning-speed factor from Math
//! Academy's formula is left out for now (it would multiply `credit`).

use chrono::{DateTime, Utc};

/// Memory level at which a concept comes due.
pub const DUE_MEMORY: f64 = 0.5;

/// Reviews scoring at least this pass.
pub const PASSING_SCORE: f64 = 0.5;

/// Interval growth per repetition.
const INTERVAL_GROWTH: f64 = 2.5;

#[derive(Clone, Debug, PartialEq)]
pub struct Memory {
    pub reps: f64,
    /// Memory level as of `updated_at`.
    pub memory: f64,
    pub updated_at: DateTime<Utc>,
}

/// The raw credit of an explicit review: passing scores earn 0.5..=1 of a
/// repetition, failing ones lose up to a full repetition.
pub fn review_credit(score: f64) -> f64 {
    if score >= PASSING_SCORE {
        score
    } else {
        score - 1.0
    }
}

impl Memory {
    /// A concept that has just been introduced, before its first review.
    pub fn fresh(at: DateTime<Utc>) -> Self {
        Memory {
            reps: 0.0,
            memory: 0.0,
            updated_at: at,
        }
    }

    /// Ideal days between this repetition and the next: half a day before the
    /// first review, then one day, 2.5, about 6, 16, 39, 98, ...
    pub fn interval_days(&self) -> f64 {
        INTERVAL_GROWTH.powf(self.reps - 1.0).max(0.4)
    }

    pub fn memory_at(&self, now: DateTime<Utc>) -> f64 {
        let days = (now - self.updated_at).num_milliseconds().max(0) as f64 / 86_400_000.0;
        self.memory * 0.5f64.powf(days / self.interval_days())
    }

    /// When memory decays to [`DUE_MEMORY`].
    pub fn due_at(&self) -> DateTime<Utc> {
        if self.memory <= DUE_MEMORY {
            return self.updated_at;
        }
        let days = self.interval_days() * (self.memory / DUE_MEMORY).log2();
        self.updated_at + chrono::Duration::milliseconds((days * 86_400_000.0) as i64)
    }

    /// Apply a repetition's credit (negative for a failure). Returns the
    /// credit actually applied after the early-repetition discount.
    pub fn credit(&mut self, credit: f64, at: DateTime<Utc>) -> f64 {
        let now = self.memory_at(at);
        let applied = if credit > 0.0 {
            credit * ((1.0 - now) / (1.0 - DUE_MEMORY)).clamp(0.0, 1.0)
        } else {
            credit
        };
        self.reps = (self.reps + applied).max(0.0);
        self.memory = (now + applied).clamp(0.0, 1.0);
        self.updated_at = at;
        applied
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect_reviews_at_due_time_grow_the_interval() {
        let start = DateTime::<Utc>::UNIX_EPOCH;
        let mut memory = Memory::fresh(start);
        memory.credit(1.0, start);
        assert_eq!(memory.due_at(), start + chrono::Duration::days(1));

        let due = memory.due_at();
        memory.credit(1.0, due);
        let gap = memory.due_at() - due;
        assert_eq!(gap.num_hours(), 60);
    }

    #[test]
    fn reviewing_again_immediately_earns_nothing() {
        let start = DateTime::<Utc>::UNIX_EPOCH;
        let mut memory = Memory::fresh(start);
        memory.credit(1.0, start);
        assert_eq!(memory.credit(1.0, start), 0.0);
        assert_eq!(memory.reps, 1.0);
    }

    #[test]
    fn a_failure_makes_the_concept_due_now() {
        let start = DateTime::<Utc>::UNIX_EPOCH;
        let mut memory = Memory::fresh(start);
        memory.credit(1.0, start);
        memory.credit(1.0, memory.due_at());
        let later = memory.updated_at + chrono::Duration::hours(1);
        memory.credit(review_credit(0.0), later);
        assert_eq!(memory.due_at(), later);
        assert!(memory.reps < 1.5);
    }
}
