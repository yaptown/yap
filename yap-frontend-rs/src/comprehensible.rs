//! A monotone prediction is a suffix of the pack's ease order. Only cards in
//! the deck need overrides; finalizing never materializes the predicted suffix.
use std::{collections::BTreeSet, hash::Hash};

use isotonic::SmoothRegression;
use language_utils::{
    SpurGram, TaggedGram,
    language_pack::{EaseOrder, LanguagePack},
};
use rustc_hash::FxHashMap;

use crate::{CardData, Cards, Regressions};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CardKnowledge {
    Now,
    Planned,
    Excluded,
}

impl From<&CardData> for CardKnowledge {
    fn from(card: &CardData) -> Self {
        match card {
            CardData::Added { fsrs_card } | CardData::Ghost { fsrs_card }
                if fsrs_card.state == rs_fsrs::State::Review =>
            {
                Self::Now
            }
            CardData::Added { .. } => Self::Planned,
            CardData::Ghost { .. } => Self::Excluded,
        }
    }
}

impl CardKnowledge {
    fn contains(self, planned: bool) -> bool {
        matches!(self, Self::Now) || planned && matches!(self, Self::Planned)
    }
}

#[derive(Clone, Copy)]
struct ComprehensibleGrams<'a, K> {
    threshold_rank: Option<u32>,
    cards: &'a FxHashMap<K, CardKnowledge>,
}

impl<K: Copy + Eq + Hash + Ord> ComprehensibleGrams<'_, K> {
    fn contains(&self, order: &EaseOrder<K>, key: &K, planned: bool) -> bool {
        let Some(rank) = order.rank(key) else {
            return false;
        };
        self.cards.get(key).map_or_else(
            || {
                self.threshold_rank
                    .is_some_and(|threshold| rank >= threshold)
            },
            |knowledge| knowledge.contains(planned),
        )
    }

    fn iter<'a>(&'a self, order: &'a EaseOrder<K>, planned: bool) -> impl Iterator<Item = K> + 'a {
        self.threshold_rank
            .into_iter()
            .flat_map(|rank| order.iter_from(rank))
            .filter(|key| !self.cards.contains_key(key))
            .chain(
                self.cards.iter().filter_map(move |(key, knowledge)| {
                    knowledge.contains(planned).then_some(*key)
                }),
            )
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CachedComprehensibleGrams {
    written: Option<u32>,
    listening: Option<u32>,
}

fn threshold<K: Copy + Eq + Hash + Ord>(
    order: &EaseOrder<K>,
    regression: Option<&SmoothRegression<f32>>,
) -> Option<u32> {
    // Isotonic regression and box smoothing preserve monotonicity.
    regression.map(|regression| {
        order.partition_point(|ease| !regression.interpolate(ease).is_some_and(|p| p >= 0.80))
    })
}

impl CachedComprehensibleGrams {
    pub(crate) fn new(pack: &LanguagePack, regressions: &Regressions) -> Self {
        Self {
            written: threshold(
                &pack.written_ease_order,
                regressions.target_language_regression.as_ref(),
            ),
            listening: threshold(
                &pack.listening_ease_order,
                regressions.listening_regression.as_ref(),
            ),
        }
    }

    pub(crate) fn written<'a>(
        &self,
        pack: &'a LanguagePack,
        cards: &'a Cards,
        planned: bool,
    ) -> WrittenGrams<'a> {
        WrittenGrams {
            cache: ComprehensibleGrams {
                threshold_rank: self.written,
                cards: cards.written_knowledge(),
            },
            order: &pack.written_ease_order,
            planned,
        }
    }

    pub(crate) fn listening<'a>(
        &self,
        pack: &'a LanguagePack,
        cards: &'a Cards,
        planned: bool,
    ) -> ListeningGrams<'a> {
        ListeningGrams {
            cache: ComprehensibleGrams {
                threshold_rank: self.listening,
                cards: cards.listening_knowledge(),
            },
            pack,
            planned,
        }
    }
}

/// Borrowed membership view of written entries, including predicted knowledge.
#[derive(Clone, Copy)]
pub struct WrittenGrams<'a> {
    cache: ComprehensibleGrams<'a, TaggedGram<SpurGram>>,
    order: &'a EaseOrder<TaggedGram<SpurGram>>,
    planned: bool,
}

impl WrittenGrams<'_> {
    pub fn contains(&self, gram: &TaggedGram<SpurGram>) -> bool {
        self.cache.contains(self.order, gram, self.planned)
    }

    pub fn iter(&self) -> impl Iterator<Item = TaggedGram<SpurGram>> + '_ {
        self.cache.iter(self.order, self.planned)
    }

    /// The same membership as a bitset over ease ranks, for loops that test
    /// many grams against one snapshot: a bit test instead of two hash lookups.
    pub fn rank_bitset(&self) -> RankBitset {
        let mut bits = vec![0u64; self.order.len().div_ceil(64)];
        let mut set = |rank: u32, known: bool| {
            let (word, bit) = (rank as usize / 64, rank % 64);
            if known {
                bits[word] |= 1 << bit;
            } else {
                bits[word] &= !(1 << bit);
            }
        };
        if let Some(threshold) = self.cache.threshold_rank {
            for rank in threshold..self.order.len() as u32 {
                set(rank, true);
            }
        }
        for (key, knowledge) in self.cache.cards {
            if let Some(rank) = self.order.rank(key) {
                set(rank, knowledge.contains(self.planned));
            }
        }
        RankBitset(bits)
    }
}

/// See [`WrittenGrams::rank_bitset`].
pub struct RankBitset(Vec<u64>);

impl RankBitset {
    pub fn contains(&self, rank: u32) -> bool {
        self.0[rank as usize / 64] & (1 << (rank % 64)) != 0
    }
}

/// Listening knowledge is shared by all senses of a bare gram.
#[derive(Clone, Copy)]
pub(crate) struct ListeningGrams<'a> {
    cache: ComprehensibleGrams<'a, SpurGram>,
    pack: &'a LanguagePack,
    planned: bool,
}

impl ListeningGrams<'_> {
    pub(crate) fn contains(&self, gram: &TaggedGram<SpurGram>) -> bool {
        self.pack.written_ease_order.rank(gram).is_some()
            && self
                .cache
                .contains(&self.pack.listening_ease_order, &gram.gram, self.planned)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = TaggedGram<SpurGram>> + '_ {
        self.cache
            .iter(&self.pack.listening_ease_order, self.planned)
            .flat_map(|gram| self.pack.senses_of(gram).iter().copied())
    }
}

/// Membership-only consumers work with borrowed views or owned projections.
pub(crate) trait GramMembership: Copy {
    fn contains(self, gram: &TaggedGram<SpurGram>) -> bool;
}

impl GramMembership for WrittenGrams<'_> {
    fn contains(self, gram: &TaggedGram<SpurGram>) -> bool {
        WrittenGrams::contains(&self, gram)
    }
}
impl GramMembership for ListeningGrams<'_> {
    fn contains(self, gram: &TaggedGram<SpurGram>) -> bool {
        ListeningGrams::contains(&self, gram)
    }
}
impl GramMembership for &BTreeSet<TaggedGram<SpurGram>> {
    fn contains(self, gram: &TaggedGram<SpurGram>) -> bool {
        BTreeSet::contains(self, gram)
    }
}
