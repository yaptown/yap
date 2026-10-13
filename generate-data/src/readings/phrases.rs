//! Phrase readings are projections of this occurrence, never of its lemma key.
use super::{Dictionary, atom_readings, inflections};
use language_utils::readings::{
    canonical_kana, is_han, is_ruby_base, ruby_segments, supports_readings,
};
use language_utils::{
    Atom, Gram, Language, OtherWordType, ReadingSentence, SentenceGram, Word, WordType,
};
use std::collections::{BTreeMap, BTreeSet};
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Stats {
    pub grams: usize,
    pub without_reading: usize,
}

fn literal_reading(words: &[&Word<String>], language: Language) -> Option<String> {
    let mut result = String::new();
    for word in words {
        if matches!(&word.word_type, WordType::Other(o) if o.other_tag == OtherWordType::Punct)
            && !word.text.chars().any(|c| is_ruby_base(c, language))
        {
            continue;
        }
        if language != Language::Japanese || word.text.chars().any(|c| is_ruby_base(c, language)) {
            return None;
        }
        let kana: String = word.text.nfkc().map(canonical_kana).collect();
        if !kana
            .chars()
            .all(|c| ('ぁ'..='ゖ').contains(&c) || c == 'ー')
        {
            return None;
        }
        result.push_str(&kana);
    }
    Some(result)
}

/// Crop only at proven pronunciation boundaries. Ruby resolves kana islands;
/// dictionary atom options resolve adjacent Han words against the *actual*
/// segment reading. Ambiguous alignments must agree on the selected output.
fn crop_japanese(
    words: &[&Word<String>],
    reading: &str,
    selected: &BTreeSet<usize>,
    dictionary: &Dictionary,
    inflections: &inflections::Index,
) -> Option<String> {
    let text: String = words.iter().map(|w| w.text.as_str()).collect();
    let mut offset = 0;
    let mut wanted = BTreeSet::new();
    for (i, word) in words.iter().enumerate() {
        let end = offset + word.text.chars().count();
        if selected.contains(&i) {
            wanted.extend(offset..end);
        }
        offset = end;
    }
    let mut offset = 0;
    let mut ruby = Some(String::new());
    for segment in ruby_segments(&text, reading, Language::Japanese) {
        let end = offset + segment.base.chars().count();
        let hits = wanted.range(offset..end).count();
        if hits > 0 {
            if let Some(r) = segment.ruby {
                if hits != end - offset {
                    ruby = None;
                    break;
                }
                ruby.as_mut().unwrap().push_str(&r);
            } else {
                ruby.as_mut().unwrap().extend(
                    segment
                        .base
                        .chars()
                        .enumerate()
                        .filter(|(i, _)| wanted.contains(&(offset + i)))
                        .map(|(_, c)| canonical_kana(c))
                        .filter(|c| ('ぁ'..='ゖ').contains(c) || *c == 'ー'),
                );
            }
        }
        offset = end;
    }
    if ruby.is_some() {
        return ruby;
    }
    let options: Vec<_> = words
        .iter()
        .enumerate()
        .map(|(i, word)| {
            atom_readings(word, Language::Japanese, dictionary, inflections)
                .into_iter()
                .map(|option| {
                    let output = if selected.contains(&i) {
                        option.reading.clone()
                    } else {
                        String::new()
                    };
                    (option.reading, output)
                })
                .collect()
        })
        .collect();
    align_atoms(reading, &options)
}

/// Align exact atom inputs. Different outputs at the same input position stay
/// ambiguous: appending an identical suffix cannot make them equal again.
fn align_unique<T: Clone + Eq>(
    reading: &str,
    options: &[Vec<(String, String)>],
    initial: T,
    append: impl Fn(&T, &str) -> T,
) -> Option<T> {
    let mut states = BTreeMap::from([(0, Some(initial))]);
    for options in options {
        let mut next = BTreeMap::new();
        for (position, output) in states {
            for (input, replacement) in options {
                if reading[position..].starts_with(input) {
                    let value = output.as_ref().map(|out| append(out, replacement));
                    let existing = next
                        .entry(position + input.len())
                        .or_insert_with(|| value.clone());
                    if *existing != value {
                        *existing = None;
                    }
                }
            }
        }
        states = next;
    }
    states.remove(&reading.len()).flatten()
}

/// Projection and lemma policy need a unique final string, even if several atom
/// alignments agree on that string.
pub(super) fn align_atoms(reading: &str, options: &[Vec<(String, String)>]) -> Option<String> {
    align_unique(reading, options, String::new(), |out, part| {
        format!("{out}{part}")
    })
}

/// Synthetic changes need proven individual atom boundaries, not merely an
/// agreed whole reading. An ambiguous segmentation therefore stays unexpanded.
pub(super) fn aligned_atom_readings(reading: &str, options: &[Vec<String>]) -> Option<Vec<String>> {
    let options: Vec<_> = options
        .iter()
        .map(|readings| readings.iter().map(|r| (r.clone(), r.clone())).collect())
        .collect();
    align_unique(reading, &options, Vec::new(), |out, part| {
        let mut out = out.clone();
        out.push(part.into());
        out
    })
}

fn joined(
    sentence: &ReadingSentence<Gram<String>, String>,
    indices: &[u16],
    language: Language,
    dictionary: &Dictionary,
    inflections: &inflections::Index,
) -> Option<String> {
    if indices.is_empty() || !supports_readings(language) {
        return None;
    }
    let wanted: BTreeSet<_> = indices.iter().map(|&i| usize::from(i)).collect();
    if wanted.len() != indices.len() || !indices.windows(2).all(|pair| pair[0] < pair[1]) {
        return None;
    }
    let mut surface = String::new();
    let mut offset = 0;
    let mut parts = Vec::new();
    for segment in &sentence.grams {
        let (SentenceGram::Learnable(segment) | SentenceGram::Obvious(segment)) = segment;
        let words: Vec<_> = segment
            .sense
            .gram
            .iter()
            .filter_map(|atom| match atom {
                Atom::Tok(word) => Some(word),
                _ => None,
            })
            .collect();
        let start = offset;
        offset += words.len();
        let selected = wanted.range(start..offset).count();
        if selected == 0 {
            continue;
        }
        for (i, word) in words.iter().enumerate() {
            if wanted.contains(&(start + i)) {
                surface.push_str(&word.text);
            }
        }
        // Written kana needs no unavailable reading of an unselected neighbour.
        if segment.reading.is_none() {
            let selected_words: Vec<_> = words
                .iter()
                .enumerate()
                .filter(|(i, _)| wanted.contains(&(start + i)))
                .map(|(_, w)| *w)
                .collect();
            parts.push(literal_reading(&selected_words, language)?);
            continue;
        }
        // A missing Han reading is not guessed. Pure kana and punctuation still
        // have their directly-written pronunciation even without ruby metadata.
        let text: String = words.iter().map(|w| w.text.as_str()).collect();
        let reading = super::canonical::checked_surface_reading(
            language,
            &text,
            segment.reading.as_deref()?,
        )?;
        if selected == words.len() {
            if !reading.is_empty() {
                parts.push(reading);
            }
        } else {
            if language == Language::Japanese {
                let selected = wanted.range(start..offset).map(|i| i - start).collect();
                parts.push(crop_japanese(
                    &words,
                    &reading,
                    &selected,
                    dictionary,
                    inflections,
                )?);
                continue;
            }
            let syllables: Vec<_> = reading.split_whitespace().collect();
            let han: Vec<_> = words
                .iter()
                .map(|w| w.text.chars().filter(|&c| is_han(c)).count())
                .collect();
            if han.iter().sum::<usize>() != syllables.len() {
                return None;
            }
            let mut position = 0;
            for (i, n) in han.into_iter().enumerate() {
                if wanted.contains(&(start + i)) && n > 0 {
                    parts.push(syllables[position..position + n].join(" "));
                }
                position += n;
            }
        }
    }
    parts.retain(|part| !part.is_empty());
    if wanted.last().is_some_and(|&i| i >= offset) || parts.is_empty() {
        return None;
    }
    // Source occurrences already carry meaning-bound lexical tones. Preserve
    // those choices; only presentation/surface checks apply to the projection.
    super::canonical::checked_surface_reading(
        language,
        &surface,
        &parts.join(if language == Language::Japanese {
            ""
        } else {
            " "
        }),
    )
}

pub(super) fn annotate(
    sentences: &mut [(String, ReadingSentence<Gram<String>, String>)],
    language: Language,
    dictionary: &Dictionary,
) -> Stats {
    let inflections = inflections::Index::default();
    let mut grams = BTreeMap::<_, bool>::new();
    for (_, sentence) in sentences {
        let readings: Vec<_> = sentence
            .multiword_terms
            .iter()
            .chain(&sentence.low_confidence_multiword_terms)
            .map(|term| {
                joined(
                    sentence,
                    &term.matched_word_indices,
                    language,
                    dictionary,
                    &inflections,
                )
            })
            .collect();
        for (term, reading) in sentence
            .multiword_terms
            .iter_mut()
            .chain(&mut sentence.low_confidence_multiword_terms)
            .zip(readings)
        {
            *grams.entry(term.gram.sense.gram.clone()).or_default() |= reading.is_some();
            term.gram.reading = reading;
        }
    }
    Stats {
        grams: grams.len(),
        without_reading: grams.values().filter(|present| !**present).count(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use language_utils::{
        Heteronym, MultiwordTermMatch, PartOfSpeech, ReadingTaggedGram, SentenceGrams, TaggedGram,
    };
    fn annotate(
        sentences: &mut [(String, ReadingSentence<Gram<String>, String>)],
        language: Language,
    ) -> Stats {
        super::annotate(
            sentences,
            language,
            &BTreeMap::from([
                ("入院".into(), vec!["にゅういん".into()]),
                ("中".into(), vec!["ちゅう".into()]),
            ]),
        )
    }
    fn joined(
        sentence: &ReadingSentence<Gram<String>, String>,
        indices: &[u16],
        language: Language,
    ) -> Option<String> {
        super::joined(
            sentence,
            indices,
            language,
            &BTreeMap::new(),
            &inflections::Index::default(),
        )
    }
    fn gram(words: &[&str]) -> Gram<String> {
        Gram(
            words
                .iter()
                .map(|w| {
                    Atom::Tok(Word {
                        text: (*w).into(),
                        word_type: WordType::Heteronym(Heteronym {
                            word: (*w).into(),
                            lemma: (*w).into(),
                            pos: PartOfSpeech::Noun,
                        }),
                    })
                })
                .collect(),
        )
    }
    fn sentence(
        groups: &[(&[&str], Option<&str>)],
        phrase: &[&str],
        indices: &[u16],
        low: bool,
    ) -> ReadingSentence<Gram<String>, String> {
        let term = MultiwordTermMatch {
            gram: ReadingTaggedGram {
                sense: TaggedGram {
                    gram: gram(phrase),
                    sense: std::num::NonZeroU32::new(3),
                },
                reading: None,
            },
            matched_word_indices: indices.to_vec(),
        };
        SentenceGrams {
            grams: groups
                .iter()
                .map(|(words, r)| {
                    SentenceGram::Learnable(ReadingTaggedGram {
                        sense: TaggedGram {
                            gram: gram(words),
                            sense: None,
                        },
                        reading: r.map(str::to_owned),
                    })
                })
                .collect(),
            capitalize_first: false,
            multiword_terms: if low { vec![] } else { vec![term.clone()] },
            low_confidence_multiword_terms: if low { vec![term] } else { vec![] },
        }
    }
    #[test]
    fn surface_readings_crop_gaps_and_do_not_read_unselected_unknown_words() {
        for (groups, indices, expected) in [
            (
                vec![(&["食べ", "て", "いる"][..], Some("たべている"))],
                vec![0, 2],
                "たべいる",
            ),
            (
                vec![(&["ホテル", "に", "泊まって"][..], None)],
                vec![0, 1],
                "ほてるに",
            ),
        ] {
            // Canonical lemma identity is intentionally unrelated to the surface.
            let mut rows = vec![("s".into(), sentence(&groups, &["食べる"], &indices, false))];
            annotate(&mut rows, Language::Japanese);
            assert_eq!(
                rows[0].1.multiword_terms[0].gram.reading.as_deref(),
                Some(expected)
            );
        }
    }
    #[test]
    fn atom_alignment_must_agree_on_selected_output() {
        let words = gram(&["甲", "乙", "丙"]);
        let words: Vec<_> = words
            .iter()
            .filter_map(|a| match a {
                Atom::Tok(w) => Some(w),
                _ => None,
            })
            .collect();
        let dictionary = BTreeMap::from([
            ("甲".into(), vec!["あ".into(), "あい".into()]),
            ("乙".into(), vec!["いう".into(), "う".into()]),
            ("丙".into(), vec!["え".into()]),
        ]);
        assert_eq!(
            crop_japanese(
                &words,
                "あいうえ",
                &BTreeSet::from([2]),
                &dictionary,
                &inflections::Index::default()
            ),
            Some("え".into())
        );
        assert_eq!(
            crop_japanese(
                &words,
                "あいうえ",
                &BTreeSet::from([0]),
                &dictionary,
                &inflections::Index::default()
            ),
            None
        );
    }
    #[test]
    fn unavailable_readings_and_unknown_positions_do_not_supply_candidates() {
        let sentence = sentence(&[(&["ヶ"], None)], &["ヶ"], &[0], false);
        assert_eq!(joined(&sentence, &[0], Language::Japanese), None);
        assert_eq!(joined(&sentence, &[], Language::Japanese), None);
        assert_eq!(joined(&sentence, &[1], Language::Japanese), None);
        let punctuation = self::sentence(&[(&["。"], None)], &["。"], &[0], false);
        assert_eq!(joined(&punctuation, &[0], Language::Japanese), None);
    }

    #[test]
    fn projections_validate_whole_segments_and_preserve_meaning_tones() {
        for (language, words, reading, expected) in [
            (
                Language::ChineseSimplified,
                vec!["走", "啊"],
                "zǒu à",
                Some("zǒu a"),
            ),
            (
                Language::ChineseSimplified,
                vec!["东西"],
                "dōng xī",
                Some("dōng xī"),
            ),
            (
                Language::ChineseSimplified,
                vec!["东西"],
                "dōng xi",
                Some("dōng xi"),
            ),
            (
                Language::ChineseSimplified,
                vec!["朋友"],
                "péng you",
                Some("péng you"),
            ),
            (
                Language::ChineseSimplified,
                vec!["其中", "一个"],
                "qí zhōng yī",
                None,
            ),
            (Language::Japanese, vec!["食べ", "て"], "たべる", None),
            (
                Language::Japanese,
                vec!["には", "入って"],
                "ににはいって",
                None,
            ),
        ] {
            let indices: Vec<_> = (0..words.len() as u16).collect();
            let row = sentence(&[(&words, Some(reading))], &words, &indices, false);
            assert_eq!(
                joined(&row, &indices, language).as_deref(),
                expected,
                "{words:?}"
            );
        }
        let row = sentence(
            &[(&["猫", "は"], Some("ねこは"))],
            &["猫", "は"],
            &[0, 1],
            false,
        );
        assert!(joined(&row, &[0, 0], Language::Japanese).is_none());
        assert!(joined(&row, &[1, 0], Language::Japanese).is_none());
    }

    #[test]
    fn matches_keep_their_own_reading() {
        let mut sentences = vec![
            (
                "a".into(),
                sentence(
                    &[(&["何"], Some("なん")), (&["を"], None)],
                    &["何", "を"],
                    &[0, 1],
                    false,
                ),
            ),
            (
                "b".into(),
                sentence(
                    &[(&["何"], Some("なに")), (&["を"], None)],
                    &["何", "を"],
                    &[0, 1],
                    true,
                ),
            ),
        ];
        assert_eq!(
            annotate(&mut sentences, Language::Japanese),
            Stats {
                grams: 1,
                without_reading: 0
            }
        );
        for ((_, s), expected) in sentences.iter().zip(["なんを", "なにを"]) {
            for term in s
                .multiword_terms
                .iter()
                .chain(&s.low_confidence_multiword_terms)
            {
                assert_eq!(term.gram.reading.as_deref(), Some(expected));
                assert_eq!(term.gram.sense.sense, std::num::NonZeroU32::new(3));
            }
        }
        sentences.push((
            "c".into(),
            sentence(
                &[(&["何"], Some("なん")), (&["を"], None)],
                &["何", "を"],
                &[0, 1],
                false,
            ),
        ));
        annotate(&mut sentences, Language::Japanese);
        assert_eq!(
            sentences[1].1.low_confidence_multiword_terms[0]
                .gram
                .reading
                .as_deref(),
            Some("なにを")
        );
        assert_eq!(
            sentences[2].1.multiword_terms[0].gram.reading.as_deref(),
            Some("なんを")
        );
    }
    #[test]
    fn partial_segments_crop_the_actual_reading() {
        let mut jpn = vec![(
            "j".into(),
            sentence(
                &[(&["入院", "中"], Some("にゅういんちゅう")), (&["だ"], None)],
                &["中", "だ"],
                &[1, 2],
                false,
            ),
        )];
        assert_eq!(annotate(&mut jpn, Language::Japanese).without_reading, 0);
        assert_eq!(
            jpn[0].1.multiword_terms[0].gram.reading.as_deref(),
            Some("ちゅうだ")
        );
        jpn[0].1.multiword_terms[0].matched_word_indices = vec![0, 1, 2];
        jpn[0].1.multiword_terms[0].gram.sense.gram = gram(&["入院", "中", "だ"]);
        annotate(&mut jpn, Language::Japanese);
        assert_eq!(
            jpn[0].1.multiword_terms[0].gram.reading.as_deref(),
            Some("にゅういんちゅうだ")
        );
        let mut zh = vec![(
            "z".into(),
            sentence(
                &[
                    (&["银行", "里"], Some("yín háng lǐ")),
                    (&["有"], Some("yǒu")),
                ],
                &["银行", "有"],
                &[0, 2],
                false,
            ),
        )];
        assert_eq!(
            annotate(&mut zh, Language::ChineseSimplified).without_reading,
            0
        );
        assert_eq!(
            zh[0].1.multiword_terms[0].gram.reading.as_deref(),
            Some("yín háng yǒu")
        );
    }
}
