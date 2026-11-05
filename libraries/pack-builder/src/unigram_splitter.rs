use lasso::Splitter;
use omnigram::unigram::{UnigramModel, UnigramTrainer, UnigramTrainerConfig};

// French full-corpus sweep: 32k saved only 6.8 KB over 16k but trained
// longer. Eight-character candidates avoid the extra cost of longer runs.
const TARGET_MULTIWORD_TOKENS: usize = 16_000;
const MAX_PIECE_LENGTH: usize = 8;

pub(crate) struct UnigramSplitter {
    model: UnigramModel<char>,
}

impl Splitter<str> for UnigramSplitter {
    type Value<'r> = &'r str;

    fn train<'r>(rows: impl Iterator<Item = &'r str> + Clone) -> Self {
        let config = UnigramTrainerConfig {
            target_multiword_tokens: TARGET_MULTIWORD_TOKENS,
            max_piece_length: MAX_PIECE_LENGTH,
            ..Default::default()
        };
        // HashSet iteration must not influence EM accumulation or vocabulary ties.
        let mut rows: Vec<_> = rows.collect();
        rows.sort_unstable();
        let corpus: Vec<Vec<char>> = rows.into_iter().map(|row| row.chars().collect()).collect();
        let model = if corpus.iter().all(Vec::is_empty) {
            UnigramModel::new(Vec::new(), config.merge_alpha)
        } else {
            UnigramTrainer::new(config).train(&corpus, &[], |_| true)
        };
        Self { model }
    }

    fn split<'r>(&self, row: &'r str, emit: &mut impl FnMut(&'r str)) {
        let chars: Vec<_> = row.chars().collect();
        let segments = self
            .model
            .segment(&chars)
            .expect("training covers every input character");
        let mut start = 0;
        for segment in segments {
            let end = start + segment.iter().map(|ch| ch.len_utf8()).sum::<usize>();
            emit(&row[start..end]);
            start = end;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_raw_text_and_empty_rows() {
        type Reader = lasso::CompressedRodeoReader<lasso::Spur, lasso::RunTable, lasso::Text>;
        let texts = [
            "",
            "hello, world!",
            "hello,  world!",
            "é漢字 🙂",
            "e\u{301}\t\n",
        ];
        let reader =
            Reader::build::<UnigramSplitter>(texts.iter().map(|text| text.to_string()).collect());
        for text in texts {
            assert_eq!(reader.resolve_string(&reader.get(&(), text).unwrap()), text);
        }
        assert!(Reader::build::<UnigramSplitter>(Default::default()).is_empty());
        let empty = Reader::build::<UnigramSplitter>([String::new()].into_iter().collect());
        assert!(empty.resolve(&empty.get(&(), "").unwrap()).next().is_none());
    }
}
