use language_utils::language_pack::FrequencyList;
use serde::{Deserialize, Serialize};

/// Tier definitions for the frequency list. Each tier covers a range of the most frequent grams.
/// The first element is the tier name, the second is how many new words this tier adds.
type TierName = fn() -> String;

const TIERS: &[(TierName, usize)] = &[
    (|| yap_i18n::tr!("tier-elementary"), 200),
    (|| yap_i18n::tr!("tier-core"), 300),
    (|| yap_i18n::tr!("tier-basic"), 400),
    (|| yap_i18n::tr!("curriculum-essential"), 500),
    (|| yap_i18n::tr!("tier-functional"), 600),
    (|| yap_i18n::tr!("tier-intermediate"), 700),
    (|| yap_i18n::tr!("tier-proficient"), 800),
    (|| yap_i18n::tr!("tier-advanced"), 900),
    (|| yap_i18n::tr!("tier-expert"), 1000),
    (|| yap_i18n::tr!("tier-specialized"), 1100),
    (|| yap_i18n::tr!("tier-esoteric"), usize::MAX), // all remaining
];

pub(crate) const WORDS_PER_LEVEL: usize = 31;

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TierInfo {
    /// 1-indexed tier number
    pub tier: u32,
    /// Display name for this tier
    pub name: String,
    /// 1-indexed level within this tier
    pub level: u32,
    /// Total number of levels in this tier
    pub total_levels: u32,
    /// Progress through the current effort slice in equivalent words (0-100)
    pub percent_known: f64,
    /// Cumulative corpus coverage reached at the end of this level (0-100)
    pub percent_of_usage: f64,
}

pub(crate) struct EssentialPosition {
    pub(crate) tier_info: TierInfo,
    start: usize,
    end: usize,
}

impl EssentialPosition {
    /// Project through the current level, not the projected position's next level.
    pub(crate) fn projected_percent(&self, frequencies: &FrequencyList, coverage: f64) -> f64 {
        level_percent(
            equivalent_words(frequencies, coverage),
            self.start,
            self.end,
        )
    }
}

fn level_percent(words: f64, start: usize, end: usize) -> f64 {
    if start == end {
        return 100.0;
    }
    ((words - start as f64) / (end - start) as f64 * 100.0).clamp(0.0, 100.0)
}

/// Invert corpus coverage on the pack's descending-frequency curve. Keep the
/// stats denominator: total_count includes corpus mass omitted from entries.
fn equivalent_words(frequencies: &FrequencyList, coverage: f64) -> f64 {
    invert_counts(
        frequencies.entries.values().map(|f| f.count as f64),
        coverage / 100.0 * frequencies.total_count as f64,
    )
}

fn invert_counts(counts: impl Iterator<Item = f64>, mut mass: f64) -> f64 {
    let mut words = 0.0;
    for count in counts {
        if mass <= 0.0 {
            break;
        }
        if mass < count {
            return words + mass / count;
        }
        words += 1.0;
        mass -= count;
    }
    words
}

/// All Essential displays use the same coverage-derived position. Levels retain
/// their original word slices, including the short final level of each tier.
pub(crate) fn position(frequencies: &FrequencyList, coverage: f64) -> EssentialPosition {
    let words = equivalent_words(frequencies, coverage);
    let len = frequencies.entries.len();
    let mut start: usize = 0;
    for (tier_idx, &(name, size)) in TIERS.iter().enumerate() {
        let end = start.saturating_add(size).min(len);
        if words < end as f64 || end == len {
            let total_levels = (end - start).div_ceil(WORDS_PER_LEVEL).max(1);
            let level_idx =
                (((words - start as f64) / WORDS_PER_LEVEL as f64) as usize).min(total_levels - 1);
            let level_start = start + level_idx * WORDS_PER_LEVEL;
            let level_end = (level_start + WORDS_PER_LEVEL).min(end);
            let cumulative_mass: u64 = frequencies
                .entries
                .values()
                .take(level_end)
                .map(|f| f.count as u64)
                .sum();
            return EssentialPosition {
                tier_info: TierInfo {
                    tier: (tier_idx + 1) as u32,
                    name: name(),
                    level: (level_idx + 1) as u32,
                    total_levels: total_levels as u32,
                    percent_known: level_percent(words, level_start, level_end),
                    percent_of_usage: if frequencies.total_count == 0 {
                        0.0
                    } else {
                        cumulative_mass as f64 / frequencies.total_count as f64 * 100.0
                    },
                },
                start: level_start,
                end: level_end,
            };
        }
        start = end;
    }
    unreachable!("the last tier includes every remaining word")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coverage_inversion() {
        let invert = |mass| invert_counts([60.0, 30.0, 10.0].into_iter(), mass);
        assert_eq!(invert(0.0), 0.0);
        assert_eq!(invert(60.0), 1.0);
        assert_eq!(invert(75.0), 1.5);
        assert_eq!(invert(100.0), 3.0);
        assert_eq!(invert(110.0), 3.0);
        for mass in 0..100 {
            assert!(invert(mass as f64) <= invert(mass as f64 + 1.0));
        }
    }

    #[test]
    fn position_preserves_slices_and_the_corpus_denominator() {
        use language_utils::{Frequency, SpurGram, TaggedGram};
        use lasso::Key;
        let frequencies = FrequencyList {
            entries: (0..250)
                .map(|i| {
                    (
                        TaggedGram {
                            gram: SpurGram::try_from_usize(i).unwrap(),
                            sense: None,
                        },
                        Frequency {
                            count: 1,
                            direct_count: 1,
                            easy: false,
                            compositional: false,
                            ease: 0.0,
                        },
                    )
                })
                .collect(),
            total_count: 500,
        };
        let zero = position(&frequencies, 0.0);
        assert_eq!(
            (
                zero.tier_info.tier,
                zero.tier_info.level,
                zero.tier_info.percent_known
            ),
            (1, 1, 0.0)
        );
        let boundary = position(&frequencies, 6.2);
        assert_eq!(
            (boundary.tier_info.level, boundary.tier_info.percent_known),
            (2, 0.0)
        );
        let next_tier = position(&frequencies, 40.0);
        assert_eq!(
            (next_tier.tier_info.tier, next_tier.tier_info.level),
            (2, 1)
        );
        assert_eq!(next_tier.tier_info.percent_of_usage, 46.2);
        assert_eq!(zero.projected_percent(&frequencies, 40.0), 100.0);
        let all = position(&frequencies, 50.0);
        assert_eq!(
            (
                all.tier_info.tier,
                all.tier_info.level,
                all.tier_info.percent_known
            ),
            (2, 2, 100.0)
        );
        assert_eq!(all.tier_info.percent_of_usage, 50.0);
    }

    #[test]
    fn progress_uses_word_units_and_caps_at_current_level_end() {
        assert_eq!(level_percent(15.5, 0, 31), 50.0);
        assert_eq!(level_percent(31.0, 31, 62), 0.0);
        assert_eq!(level_percent(193.0, 186, 200), 50.0);
        assert_eq!(level_percent(210.0, 186, 200), 100.0);
    }
}
