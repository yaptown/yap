//! Spelling matching shared by course generation and pronunciation-card display.
use crate::{Language, PatternPosition, WritingSystem};
use std::ops::Range;
use unicode_normalization::UnicodeNormalization;

pub fn normalize_word(word: &str, language: Language) -> String {
    if language.writing_system() == WritingSystem::Hangul {
        word.nfkd().collect()
    } else {
        word.to_lowercase().nfc().collect()
    }
}

/// Compatibility consonants can be either initial or final jamo. Preserve both
/// readings, including cross-syllable combinations such as ㄱㅇ in 먹어.
pub fn normalize_pattern(pattern: &str, language: Language) -> Vec<String> {
    if language.writing_system() != WritingSystem::Hangul {
        return vec![normalize_word(pattern, language)];
    }
    let mut variants = vec![String::new()];
    for ch in pattern.chars() {
        let mut choices = Vec::new();
        for (letters, base) in [
            ("ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ", 0x1100),
            (
                "ㄱㄲㄳㄴㄵㄶㄷㄹㄺㄻㄼㄽㄾㄿㅀㅁㅂㅄㅅㅆㅇㅈㅊㅋㅌㅍㅎ",
                0x11a8,
            ),
            ("ㅏㅐㅑㅒㅓㅔㅕㅖㅗㅘㅙㅚㅛㅜㅝㅞㅟㅠㅡㅢㅣ", 0x1161),
        ] {
            if let Some(index) = letters.chars().position(|letter| letter == ch) {
                choices.push(char::from_u32(base + index as u32).unwrap().to_string());
            }
        }
        if choices.is_empty() {
            choices.push(ch.nfkd().collect());
        }
        variants = variants
            .iter()
            .flat_map(|prefix| {
                choices
                    .iter()
                    .map(move |choice| format!("{prefix}{choice}"))
            })
            .take(64)
            .collect();
    }
    variants
}

pub fn matches_normalized(word: &str, variants: &[String], position: PatternPosition) -> bool {
    variants.iter().any(|pattern| {
        !pattern.is_empty()
            && match position {
                PatternPosition::Beginning => word.starts_with(pattern),
                PatternPosition::End => word.ends_with(pattern),
                PatternPosition::Anywhere => word.contains(pattern),
            }
    })
}

pub fn pattern_matches(
    word: &str,
    pattern: &str,
    position: PatternPosition,
    language: Language,
) -> bool {
    matches_normalized(
        &normalize_word(word, language),
        &normalize_pattern(pattern, language),
        position,
    )
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HighlightedRun {
    pub text: String,
    pub highlighted: bool,
}

/// Match normalized spelling, but return original text. NFD keeps accents attached
/// to their source characters; NFKD Hangul matches map back to whole syllables.
pub fn highlighted_runs(
    word: &str,
    pattern: &str,
    position: PatternPosition,
    language: Language,
) -> Vec<HighlightedRun> {
    let hangul = language.writing_system() == WritingSystem::Hangul;
    let normalize = |text: &str| -> String {
        if hangul {
            text.nfkd().collect()
        } else {
            text.to_lowercase().nfd().collect()
        }
    };
    let mut normalized = String::new();
    let mut sources: Vec<(Range<usize>, Range<usize>)> = Vec::new();
    for (start, ch) in word.char_indices() {
        let text = normalize(&ch.to_string());
        let offset = normalized.len();
        normalized.push_str(&text);
        sources.push((offset..normalized.len(), start..start + ch.len_utf8()));
    }
    let variants = normalize_pattern(pattern, language);
    let mut matches = Vec::new();
    for variant in variants {
        let variant = normalize(&variant);
        if variant.is_empty() {
            continue;
        }
        for (start, _) in normalized.char_indices() {
            let end = start + variant.len();
            if normalized[start..].starts_with(&variant)
                && match position {
                    PatternPosition::Beginning => start == 0,
                    PatternPosition::End => end == normalized.len(),
                    PatternPosition::Anywhere => true,
                }
                // Do not match plain e inside é (including decomposed input).
                && (hangul || normalized[end..].chars().next().is_none_or(|c| !unicode_normalization::char::is_combining_mark(c)))
            {
                matches.push(start..end);
            }
        }
    }
    let mut runs: Vec<HighlightedRun> = Vec::new();
    for (normalized_range, original_range) in sources {
        let highlighted = matches
            .iter()
            .any(|m| m.start < normalized_range.end && normalized_range.start < m.end);
        if let Some(last) = runs.last_mut()
            && last.highlighted == highlighted
        {
            last.text.push_str(&word[original_range]);
        } else {
            runs.push(HighlightedRun {
                text: word[original_range].into(),
                highlighted,
            });
        }
    }
    runs
}

/// The visual cue may explain a jamo, while identity and spoken text stay intact.
pub fn display_pattern(pattern: &str, language: Language) -> String {
    if language != Language::Korean {
        return pattern.into();
    }
    let romanization = match pattern {
        "ㄱ" => "g/k",
        "ㄲ" => "kk",
        "ㄴ" => "n",
        "ㄷ" => "d/t",
        "ㄸ" => "tt",
        "ㄹ" => "r/l",
        "ㅁ" => "m",
        "ㅂ" => "b/p",
        "ㅃ" => "pp",
        "ㅅ" => "s",
        "ㅆ" => "ss",
        "ㅇ" => "ng",
        "ㅈ" => "j",
        "ㅉ" => "jj",
        "ㅊ" => "ch",
        "ㅋ" => "k",
        "ㅌ" => "t",
        "ㅍ" => "p",
        "ㅎ" => "h",
        "ㅏ" => "a",
        "ㅐ" => "ae",
        "ㅑ" => "ya",
        "ㅒ" => "yae",
        "ㅓ" => "eo",
        "ㅔ" => "e",
        "ㅕ" => "yeo",
        "ㅖ" => "ye",
        "ㅗ" => "o",
        "ㅘ" => "wa",
        "ㅙ" => "wae",
        "ㅚ" => "oe",
        "ㅛ" => "yo",
        "ㅜ" => "u",
        "ㅝ" => "wo",
        "ㅞ" => "we",
        "ㅟ" => "wi",
        "ㅠ" => "yu",
        "ㅡ" => "eu",
        "ㅢ" => "ui",
        "ㅣ" => "i",
        _ => return pattern.into(),
    };
    format!("{pattern} ({romanization})")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn korean_syllables_and_positional_consonants() {
        for word in ["극", "글"] {
            assert_eq!(
                highlighted_runs(word, "ㅡ", PatternPosition::Anywhere, Language::Korean),
                vec![HighlightedRun {
                    text: word.into(),
                    highlighted: true
                }]
            );
        }
        assert!(pattern_matches(
            "국",
            "ㄱ",
            PatternPosition::End,
            Language::Korean
        ));
        assert!(!pattern_matches(
            "김치",
            "ㄱ",
            PatternPosition::End,
            Language::Korean
        ));
        assert!(pattern_matches(
            "먹어",
            "ㄱㅇ",
            PatternPosition::Anywhere,
            Language::Korean
        ));
        assert_eq!(display_pattern("ㅡ", Language::Korean), "ㅡ (eu)");
    }
    #[test]
    fn spelling_and_display_agree_on_accents_and_case() {
        for (word, pattern, expected) in [
            ("CAFÉ", "é", true),
            ("cafe\u{301}", "é", true),
            ("cafe\u{301}", "e", false),
            ("chef", "ch", true),
        ] {
            assert_eq!(
                highlighted_runs(word, pattern, PatternPosition::Anywhere, Language::French)
                    .iter()
                    .any(|run| run.highlighted),
                expected
            );
            assert_eq!(
                pattern_matches(word, pattern, PatternPosition::Anywhere, Language::French),
                expected
            );
        }
    }
}
