//! A monotone prediction is a suffix of the pack's ease order. Only cards in
//! the deck need overrides; finalizing never materializes the predicted suffix.
use std::{collections::BTreeSet, hash::Hash};

use language_utils::{
    SpurGram, TaggedGram,
    language_pack::{EaseOrder, LanguagePack},
};
use pav_regression::SmoothRegression;
use rustc_hash::FxHashMap;

use crate::{CardData, CardIndicator, Regressions};

#[derive(Clone, Copy, Debug)]
enum CardKnowledge {
    Now,
    Planned,
    Excluded,
}

impl CardKnowledge {
    fn contains(self, planned: bool) -> bool {
        matches!(self, Self::Now) || planned && matches!(self, Self::Planned)
    }
}

#[derive(Clone, Debug)]
struct ComprehensibleGrams<K> {
    threshold_rank: Option<u32>,
    cards: FxHashMap<K, CardKnowledge>,
}

impl<K: Copy + Eq + Hash + Ord> ComprehensibleGrams<K> {
    fn new(order: &EaseOrder<K>, regression: Option<&SmoothRegression<f32>>) -> Self {
        Self {
            // Isotonic regression and box smoothing preserve monotonicity.
            threshold_rank: regression.map(|regression| {
                order.partition_point(|ease| {
                    !regression.interpolate(ease).is_some_and(|p| p >= 0.80)
                })
            }),
            cards: FxHashMap::default(),
        }
    }

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

    fn insert(&mut self, order: &EaseOrder<K>, key: K, card: &CardData) {
        if order.rank(&key).is_none() {
            return;
        }
        let knowledge = match card {
            CardData::Added { fsrs_card } | CardData::Ghost { fsrs_card }
                if fsrs_card.state == rs_fsrs::State::Review =>
            {
                CardKnowledge::Now
            }
            CardData::Added { .. } => CardKnowledge::Planned,
            CardData::Ghost { .. } => CardKnowledge::Excluded,
        };
        self.cards.insert(key, knowledge);
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CachedComprehensibleGrams {
    written: ComprehensibleGrams<TaggedGram<SpurGram>>,
    listening: ComprehensibleGrams<SpurGram>,
}

impl CachedComprehensibleGrams {
    pub(crate) fn new<'a>(
        pack: &LanguagePack,
        regressions: &Regressions,
        cards: impl Iterator<Item = (&'a CardIndicator<SpurGram, lasso::Spur>, &'a CardData)>,
    ) -> Self {
        let mut result = Self {
            written: ComprehensibleGrams::new(
                &pack.written_ease_order,
                regressions.target_language_regression.as_ref(),
            ),
            listening: ComprehensibleGrams::new(
                &pack.listening_ease_order,
                regressions.listening_regression.as_ref(),
            ),
        };
        for (indicator, card) in cards {
            match indicator {
                CardIndicator::WrittenGram { gram } => {
                    result.written.insert(&pack.written_ease_order, *gram, card)
                }
                CardIndicator::ListeningGram { gram } => {
                    result
                        .listening
                        .insert(&pack.listening_ease_order, *gram, card)
                }
                CardIndicator::LetterPronunciation { .. } => {}
            }
        }
        result
    }

    pub(crate) fn written<'a>(&'a self, pack: &'a LanguagePack, planned: bool) -> WrittenGrams<'a> {
        WrittenGrams {
            cache: &self.written,
            order: &pack.written_ease_order,
            planned,
        }
    }

    pub(crate) fn listening<'a>(
        &'a self,
        pack: &'a LanguagePack,
        planned: bool,
    ) -> ListeningGrams<'a> {
        ListeningGrams {
            cache: &self.listening,
            pack,
            planned,
        }
    }
}

/// Borrowed membership view of written entries, including predicted knowledge.
#[derive(Clone, Copy)]
pub struct WrittenGrams<'a> {
    cache: &'a ComprehensibleGrams<TaggedGram<SpurGram>>,
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
        for (key, knowledge) in &self.cache.cards {
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
    cache: &'a ComprehensibleGrams<SpurGram>,
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
