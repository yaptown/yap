//! Orthographic readings, deliberately separate from event-serialized words.
use crate::Language;

#[bridgerton::bridge(transparent)]
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct RubySegment {
    pub base: String,
    pub ruby: Option<String>,
}

/// A sentence occurrence's pronunciation, separate from its written-card identity.
/// The reading is whole-gram text, never a frequency-dependent numeric index.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct ReadingTaggedGram<G, S> {
    pub sense: crate::TaggedGram<G>,
    pub reading: Option<S>,
}

impl<G, S> ReadingTaggedGram<G, S> {
    pub fn without_reading(sense: crate::TaggedGram<G>) -> Self {
        Self {
            sense,
            reading: None,
        }
    }
}

impl ReadingTaggedGram<crate::SpurGram, lasso::Spur> {
    pub fn resolve(
        &self,
        grams: &lasso::RodeoReader<crate::Gram<lasso::Spur>>,
    ) -> ReadingTaggedGram<crate::Gram<lasso::Spur>, lasso::Spur> {
        ReadingTaggedGram {
            sense: self.sense.resolve(grams),
            reading: self.reading,
        }
    }
}

impl ReadingTaggedGram<crate::Gram<String>, String> {
    pub fn get_interned(
        &self,
        strings: &lasso::RodeoReader,
    ) -> Option<ReadingTaggedGram<crate::Gram<lasso::Spur>, lasso::Spur>> {
        Some(ReadingTaggedGram {
            sense: self.sense.get_interned(strings)?,
            reading: match &self.reading {
                Some(s) => Some(strings.get(s)?),
                None => None,
            },
        })
    }
}

impl ReadingTaggedGram<crate::Gram<lasso::Spur>, lasso::Spur> {
    pub fn get_interned(
        &self,
        grams: &lasso::RodeoReader<crate::Gram<lasso::Spur>>,
    ) -> Option<ReadingTaggedGram<crate::SpurGram, lasso::Spur>> {
        Some(ReadingTaggedGram {
            sense: self.sense.get_interned(grams)?,
            reading: self.reading,
        })
    }

    pub fn resolve(
        &self,
        strings: &lasso::RodeoReader,
    ) -> ReadingTaggedGram<crate::Gram<String>, String> {
        ReadingTaggedGram {
            sense: self.sense.resolve(strings),
            reading: self.reading.map(|s| strings.resolve(&s).to_owned()),
        }
    }
}

pub fn has_han(text: &str) -> bool {
    text.chars().any(is_han)
}

pub fn is_han(c: char) -> bool {
    matches!(c as u32, 0x3400..=0x4dbf | 0x4e00..=0x9fff | 0xf900..=0xfaff | 0x20000..=0x323af)
}

/// Japanese counter abbreviations stand for 箇, and iteration repeats a Han
/// reading; none of these symbols is fixed-pronunciation kana.
pub fn is_ruby_base(c: char, language: Language) -> bool {
    match language {
        Language::Japanese => is_han(c) || matches!(c, '々' | 'ヶ' | 'ヵ'),
        Language::ChineseSimplified => is_han(c),
        _ => false,
    }
}

pub fn supports_readings(language: Language) -> bool {
    matches!(language, Language::Japanese | Language::ChineseSimplified)
}

/// The same kana folding is used by reading validation and ruby alignment.
/// Wave dashes mark the same elongation as ー in stylized kana spellings.
pub fn canonical_kana(c: char) -> char {
    if matches!(c, '〜' | '～' | '~') {
        'ー'
    } else if ('\u{30a1}'..='\u{30f6}').contains(&c) {
        char::from_u32(c as u32 - 0x60).unwrap()
    } else {
        c
    }
}

/// Align kana islands to their reading, keeping okurigana outside ruby. Only a
/// unique alignment is trustworthy; compounds and ambiguous islands use group ruby.
pub fn ruby_segments(text: &str, reading: &str, language: Language) -> Vec<RubySegment> {
    let whole = |ruby: Option<&str>| {
        vec![RubySegment {
            base: text.into(),
            ruby: ruby.map(str::to_owned),
        }]
    };
    if !text.chars().any(|c| is_ruby_base(c, language)) || reading.is_empty() {
        return whole(None);
    }
    if language == Language::ChineseSimplified {
        let syllables: Vec<_> = reading.split_whitespace().collect();
        let chars: Vec<_> = text.chars().collect();
        // Fused erhua (花儿 huār) deliberately stays grouped; 儿 ér is ordinary.
        if chars.iter().filter(|&&c| is_han(c)).count() == syllables.len() {
            let mut syllables = syllables.into_iter();
            return chars
                .iter()
                .map(|&c| RubySegment {
                    base: c.to_string(),
                    ruby: is_han(c).then(|| syllables.next().unwrap().to_owned()),
                })
                .collect();
        }
        return whole(Some(reading));
    }
    let mut runs: Vec<(bool, String)> = Vec::new();
    for c in text.chars() {
        let han = is_ruby_base(c, language);
        if let Some((previous, base)) = runs.last_mut()
            && *previous == han
        {
            base.push(c);
        } else {
            runs.push((han, c.to_string()));
        }
    }
    let kana: Vec<_> = reading.chars().map(canonical_kana).collect();
    fn align(
        runs: &[(bool, String)],
        kana: &[char],
        path: &mut Vec<RubySegment>,
        solutions: &mut Vec<Vec<RubySegment>>,
    ) {
        if solutions.len() > 1 {
            return;
        }
        let Some(((han, base), rest)) = runs.split_first() else {
            if kana.is_empty() {
                solutions.push(path.clone());
            }
            return;
        };
        if *han {
            for end in 1..=kana.len() {
                path.push(RubySegment {
                    base: base.clone(),
                    ruby: Some(kana[..end].iter().collect()),
                });
                align(rest, &kana[end..], path, solutions);
                path.pop();
                if solutions.len() > 1 {
                    break;
                }
            }
        } else {
            // Reading keys omit punctuation and spaces; keep those in the base.
            let literal: Vec<_> = base
                .chars()
                .map(canonical_kana)
                .filter(|c| c.is_alphanumeric() || *c == 'ー')
                .collect();
            if kana.starts_with(&literal) {
                path.push(RubySegment {
                    base: base.clone(),
                    ruby: None,
                });
                align(rest, &kana[literal.len()..], path, solutions);
                path.pop();
            }
        }
    }
    let mut solutions = Vec::new();
    align(&runs, &kana, &mut Vec::new(), &mut solutions);
    if solutions.len() == 1 {
        solutions.pop().unwrap()
    } else {
        whole(Some(reading))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn pairs(text: &str, reading: &str, language: Language) -> Vec<(String, Option<String>)> {
        let segments = ruby_segments(text, reading, language);
        assert_eq!(
            segments.iter().map(|s| s.base.as_str()).collect::<String>(),
            text
        );
        segments.into_iter().map(|s| (s.base, s.ruby)).collect()
    }
    fn expected(values: &[(&str, Option<&str>)]) -> Vec<(String, Option<String>)> {
        values
            .iter()
            .map(|(a, b)| (a.to_string(), b.map(str::to_owned)))
            .collect()
    }
    #[test]
    fn japanese_okurigana() {
        for (text, reading, result) in [
            ("食べ", "たべ", vec![("食", Some("た")), ("べ", None)]),
            (
                "取り消す",
                "とりけす",
                vec![
                    ("取", Some("と")),
                    ("り", None),
                    ("消", Some("け")),
                    ("す", None),
                ],
            ),
            (
                "ああ、神様",
                "ああかみさま",
                vec![("ああ、", None), ("神様", Some("かみさま"))],
            ),
            ("X線", "えっくすせん", vec![("X線", Some("えっくすせん"))]),
            ("お茶", "おちゃ", vec![("お", None), ("茶", Some("ちゃ"))]),
            ("行った", "いった", vec![("行", Some("い")), ("った", None)]),
            ("三ヶ月", "さんかげつ", vec![("三ヶ月", Some("さんかげつ"))]),
            ("三ヵ月", "さんかげつ", vec![("三ヵ月", Some("さんかげつ"))]),
            ("ヶ", "か", vec![("ヶ", Some("か"))]),
            ("人々", "ひとびと", vec![("人々", Some("ひとびと"))]),
            ("食べ", "ショク", vec![("食べ", Some("ショク"))]),
            ("食ベ", "たべ", vec![("食", Some("た")), ("ベ", None)]),
            (
                "食べてま〜す",
                "たべてまーす",
                vec![("食", Some("た")), ("べてま〜す", None)],
            ),
            ("甲あ乙", "ああああ", vec![("甲あ乙", Some("ああああ"))]),
            ("かな", "かな", vec![("かな", None)]),
            ("食べ", "", vec![("食べ", None)]),
            ("", "", vec![("", None)]),
        ] {
            assert_eq!(pairs(text, reading, Language::Japanese), expected(&result));
        }
    }
    #[test]
    fn chinese_syllables_and_erhua() {
        for (text, reading, result) in [
            (
                "银行",
                "yín háng",
                vec![("银", Some("yín")), ("行", Some("háng"))],
            ),
            (
                "孩子",
                "hái zi",
                vec![("孩", Some("hái")), ("子", Some("zi"))],
            ),
            ("花儿", "huār", vec![("花儿", Some("huār"))]),
            (
                "儿子",
                "ér zi",
                vec![("儿", Some("ér")), ("子", Some("zi"))],
            ),
            ("A股", "gǔ", vec![("A", None), ("股", Some("gǔ"))]),
            (
                "你，好！",
                "nǐ hǎo",
                vec![
                    ("你", Some("nǐ")),
                    ("，", None),
                    ("好", Some("hǎo")),
                    ("！", None),
                ],
            ),
            ("A股", "A gǔ", vec![("A股", Some("A gǔ"))]),
        ] {
            assert_eq!(
                pairs(text, reading, Language::ChineseSimplified),
                expected(&result)
            );
        }
        assert!(has_han("𠮷"));
        assert!(!has_han("カタカナ"));
        assert_eq!(
            pairs("字", "zì", Language::French),
            expected(&[("字", None)])
        );
    }
}
