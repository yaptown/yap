//! Sentence postings by reading-gram key, encoded as sequential Elias–Fano lists.
//! Byte offsets delimit each list: a little-endian u32 count, packed low bits,
//! then unary high bits (value >> low) + ordinal. Empty lists occupy no bytes.
use lasso::Key;

use crate::{SentenceSpur, sentence_codec::GramKey};

#[derive(Debug, Clone, Default, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct SentenceIndex {
    universe: u32,
    offsets: Vec<u32>,
    data: Vec<u8>,
}

impl SentenceIndex {
    /// Lists must contain sorted, distinct sentence slots below `universe`.
    pub fn new(universe: usize, lists: &[Vec<u32>]) -> Self {
        let universe = u32::try_from(universe).unwrap();
        let mut data = Vec::new();
        let mut offsets = Vec::with_capacity(lists.len() + 1);
        for list in lists {
            offsets.push(u32::try_from(data.len()).unwrap());
            if list.is_empty() {
                continue;
            }
            assert!(list.windows(2).all(|w| w[0] < w[1]));
            assert!(*list.last().unwrap() < universe);
            let count = u32::try_from(list.len()).unwrap();
            let low = low_bits(universe, count);
            data.extend_from_slice(&count.to_le_bytes());
            let start = data.len();
            let high_start = list.len() * low;
            let bits = high_start + (*list.last().unwrap() as usize >> low) + list.len();
            data.resize(start + bits.div_ceil(8), 0);
            let bytes = &mut data[start..];
            for (i, &value) in list.iter().enumerate() {
                for bit in 0..low {
                    if value & (1 << bit) != 0 {
                        set_bit(bytes, i * low + bit);
                    }
                }
                set_bit(bytes, high_start + (value as usize >> low) + i);
            }
        }
        offsets.push(u32::try_from(data.len()).unwrap());
        data.shrink_to_fit();
        Self {
            universe,
            offsets,
            data,
        }
    }
}

impl ArchivedSentenceIndex {
    pub fn sentences(&self, key: GramKey) -> impl Iterator<Item = SentenceSpur> + '_ {
        let slot = key.index() as usize;
        let bytes = self.offsets.get(slot..slot + 2).map_or(&[][..], |range| {
            &self.data[range[0].to_native() as usize..range[1].to_native() as usize]
        });
        let (count, bytes) = if bytes.is_empty() {
            (0, bytes)
        } else {
            (
                u32::from_le_bytes(bytes[..4].try_into().unwrap()),
                &bytes[4..],
            )
        };
        let low = low_bits(self.universe.to_native(), count);
        let high_start = count as usize * low;
        let mut high_position = 0;
        (0..count as usize).map(move |i| {
            let mut value = 0;
            for bit in 0..low {
                value |= usize::from(get_bit(bytes, i * low + bit)) << bit;
            }
            while !get_bit(bytes, high_start + high_position) {
                high_position += 1;
            }
            value |= (high_position - i) << low;
            high_position += 1;
            SentenceSpur::try_from_usize(value).unwrap()
        })
    }
}

fn low_bits(universe: u32, count: u32) -> usize {
    (universe / count.max(1)).checked_ilog2().unwrap_or(0) as usize
}

fn set_bit(bytes: &mut [u8], bit: usize) {
    bytes[bit / 8] |= 1 << (bit % 8);
}

fn get_bit(bytes: &[u8], bit: usize) -> bool {
    bytes[bit / 8] & (1 << (bit % 8)) != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        for (universe, lists) in [
            (0, vec![vec![]]),
            (
                257,
                vec![
                    vec![],
                    vec![0],
                    (0..257).collect(),
                    vec![1, 17, 128, 256],
                    vec![256],
                ],
            ),
            (u32::MAX, vec![vec![0, 12345, u32::MAX - 1]]),
        ] {
            let index = SentenceIndex::new(universe as usize, &lists);
            let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&index).unwrap();
            let index = rkyv::access::<ArchivedSentenceIndex, rkyv::rancor::Error>(&bytes).unwrap();
            for (key, expected) in lists.iter().enumerate() {
                let actual: Vec<_> = index
                    .sentences(GramKey::try_from_usize(key).unwrap())
                    .map(|s| s.into_usize() as u32)
                    .collect();
                assert_eq!(&actual, expected);
            }
        }
    }
}
