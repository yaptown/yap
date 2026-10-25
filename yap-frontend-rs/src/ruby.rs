//! Display readings shared by challenges, dictionary entries, and Anki.
use language_utils::{
    Language, Literal, SentenceGram, language_pack::LanguagePack, readings::RubySegment,
};
use lasso::Spur;

pub(crate) fn word_ruby(
    text: &str,
    reading: Option<&str>,
    language: Language,
    segmentations: &std::collections::BTreeMap<(String, String), Vec<RubySegment>>,
) -> Vec<RubySegment> {
    let segments = reading
        .map(|reading| {
            segmentations
                .get(&(text.to_owned(), reading.to_owned()))
                .cloned()
                .unwrap_or_else(|| language_utils::readings::ruby_segments(text, reading, language))
        })
        .unwrap_or_default();
    if segments.iter().any(|segment| segment.ruby.is_some()) {
        segments
    } else {
        vec![]
    }
}

pub(crate) struct SentenceRubyGroup<'a> {
    pub literals: &'a [Literal<String>],
    pub text: String,
    pub ruby: Vec<RubySegment>,
}

/// Segment each occurrence once, preserving internal but not trailing whitespace.
/// Anki can render the whole group; app views need the per-literal projection below.
pub(crate) fn sentence_ruby_groups<'a>(
    pack: &LanguagePack,
    language: Language,
    sentence: Spur,
    literals: &'a [Literal<String>],
    indices: &[usize],
) -> Vec<SentenceRubyGroup<'a>> {
    let grams = &pack.encoded_sentences[&sentence].grams;
    let mut start = 0;
    indices
        .chunk_by(|a, b| a == b)
        .map(|group| {
            let (SentenceGram::Learnable(gram) | SentenceGram::Obvious(gram)) = &grams[group[0]];
            let literals = &literals[start..start + group.len()];
            start += group.len();
            let mut text: String = literals
                .iter()
                .flat_map(|l| [l.word.text.as_str(), l.whitespace.as_str()])
                .collect();
            text.truncate(text.len() - literals.last().unwrap().whitespace.as_str().len());
            let ruby = word_ruby(
                &text,
                gram.reading.map(|s| pack.string_rodeo.resolve(&s)),
                language,
                &pack.ruby_segmentations,
            );
            SentenceRubyGroup {
                literals,
                text,
                ruby,
            }
        })
        .collect()
}

/// Each nonempty entry spells exactly that literal's word, without whitespace.
pub(crate) fn sentence_ruby(
    pack: &LanguagePack,
    language: Language,
    sentence: Spur,
    literals: &[Literal<String>],
    indices: &[usize],
) -> Vec<Vec<RubySegment>> {
    sentence_ruby_groups(pack, language, sentence, literals, indices)
        .into_iter()
        .flat_map(|group| {
            let words: Vec<_> = group
                .literals
                .iter()
                .map(|l| (l.word.text.as_str(), l.whitespace.as_str()))
                .collect();
            distribute(&words, group.ruby)
        })
        .collect()
}

/// Plain segments may cross boundaries; a reading must belong to one word.
fn distribute(words: &[(&str, &str)], segments: Vec<RubySegment>) -> Vec<Vec<RubySegment>> {
    let empty = || vec![vec![]; words.len()];
    if segments.is_empty() {
        return empty();
    }
    let mut position = 0;
    let ranges: Vec<_> = words
        .iter()
        .map(|(word, whitespace)| {
            let start = position;
            let end = start + word.chars().count();
            position = end + whitespace.chars().count();
            start..end
        })
        .collect();
    let mut result = empty();
    let mut start = 0;
    for segment in segments {
        let end = start + segment.base.chars().count();
        if segment.ruby.is_some() {
            let Some(i) = ranges
                .iter()
                .position(|range| range.start <= start && end <= range.end)
            else {
                return empty();
            };
            result[i].push(segment);
        } else {
            for (i, range) in ranges.iter().enumerate() {
                let left = start.max(range.start);
                let right = end.min(range.end);
                if left < right {
                    result[i].push(RubySegment {
                        base: segment
                            .base
                            .chars()
                            .skip(left - start)
                            .take(right - left)
                            .collect(),
                        ruby: None,
                    });
                }
            }
        }
        start = end;
    }
    for segments in &mut result {
        if !segments.iter().any(|s| s.ruby.is_some()) {
            segments.clear();
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn word_ruby(text: &str, reading: Option<&str>, language: Language) -> Vec<RubySegment> {
        super::word_ruby(text, reading, language, &Default::default())
    }
    #[test]
    fn split_reading_survives_literal_projection() {
        let segments = vec![
            RubySegment {
                base: "東".into(),
                ruby: Some("とう".into()),
            },
            RubySegment {
                base: "京".into(),
                ruby: Some("きょう".into()),
            },
        ];
        let lookup = std::collections::BTreeMap::from([(
            ("東京".into(), "とうきょう".into()),
            segments.clone(),
        )]);
        let ruby = super::word_ruby("東京", Some("とうきょう"), Language::Japanese, &lookup);
        assert_eq!(
            distribute(&[("東", ""), ("京", "")], ruby),
            vec![vec![segments[0].clone()], vec![segments[1].clone()]]
        );
    }
    #[test]
    fn literals_okurigana_and_whitespace() {
        let words = [("食べる", " "), ("猫", "")];
        let result = distribute(
            &words,
            word_ruby("食べる 猫", Some("たべるねこ"), Language::Japanese),
        );
        assert_eq!(
            result
                .iter()
                .map(|s| s.iter().map(|s| s.base.as_str()).collect::<String>())
                .collect::<Vec<_>>(),
            ["食べる", "猫"]
        );
        assert_eq!(result[0][0].ruby.as_deref(), Some("た"));
    }
    #[test]
    fn straddling_reading_drops_whole_group() {
        for words in [vec![("東", ""), ("京", "")], vec![("東", " "), ("京", "")]] {
            let text: String = words.iter().flat_map(|(w, s)| [*w, *s]).collect();
            assert_eq!(
                distribute(
                    &words,
                    vec![RubySegment {
                        base: text,
                        ruby: Some("とうきょう".into())
                    }]
                ),
                vec![vec![], vec![]]
            );
        }
    }
    #[test]
    fn plain_segments_cross_literals_without_readings_leaking() {
        let result = distribute(
            &[("食べ", ""), ("る", " "), ("猫", "")],
            word_ruby("食べる 猫", Some("たべるねこ"), Language::Japanese),
        );
        assert_eq!(
            result[0]
                .iter()
                .map(|s| s.base.as_str())
                .collect::<String>(),
            "食べ"
        );
        assert!(result[1].is_empty());
        assert_eq!(
            result[2],
            vec![RubySegment {
                base: "猫".into(),
                ruby: Some("ねこ".into())
            }]
        );
    }

    #[test]
    fn no_reading_is_empty() {
        assert!(word_ruby("猫", None, Language::Japanese).is_empty());
        assert!(word_ruby("かな", Some("かな"), Language::Japanese).is_empty());
        assert_eq!(distribute(&[("猫", "")], vec![]), vec![vec![]]);
    }
}
