use lasso::Spur;
use rustc_hash::FxHashMap;

#[derive(Clone, Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct MinimalPairs {
    /// Key is the unordered phoneme pair, canonicalized so the resolved
    /// string of `key.0` sorts before `key.1`. Within each `(word_a, word_b)`
    /// value, `word_a` carries `key.0` at the distinguishing position and
    /// `word_b` carries `key.1`.
    pub by_phoneme_pair: FxHashMap<(Spur, Spur), Vec<(Spur, Spur)>>,
    /// For each word that's part of any minimal pair, the list of words one
    /// phoneme substitution away. Sorted by `word_frequency` desc (most
    /// common standalone-usage neighbors first), with Spur identity as a
    /// tiebreaker for a stable archive.
    pub by_word: FxHashMap<Spur, Vec<Spur>>,
}
