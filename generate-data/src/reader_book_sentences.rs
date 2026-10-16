//! Reader sentence records and local sentence-start partitioning.
use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use anyhow::Context;
use language_utils::{Language, text_cleanup::cleanup_sentence};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SentenceRecord {
    pub sentence: String,
    pub book: String,
    pub chapter: usize,
    pub paragraph: usize,
}

/// Edition codes intentionally differ from shared NLP corpus codes.
pub fn directory(data_root: &Path, language: Language) -> PathBuf {
    data_root
        .join(language.code())
        .join("sentence-sources/reader-books")
}

pub fn read_records(path: &Path) -> anyhow::Result<Vec<SentenceRecord>> {
    let text = std::fs::read_to_string(path)?;
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).map_err(Into::into))
        .collect()
}

/// Remove one edge quote only when the emitted sentence has odd quote parity.
/// This deliberately treats every quote style as one class, not directional pairs.
pub fn strip_dangling_quote(text: &str) -> &str {
    let is_quote = |c: char| movie_subtitles::sentences::DOUBLE_QUOTES.contains(&c);
    if text.chars().filter(|&c| is_quote(c)).count() % 2 == 0 {
        text
    } else if let Some(rest) = text.strip_prefix(is_quote) {
        rest.trim_start()
    } else if let Some(rest) = text.strip_suffix(is_quote) {
        rest.trim_end()
    } else {
        text
    }
}

/// Drop the dash that opens a line of dash-style dialogue (—¿Dónde está?), but
/// only when no dash follows: `—No —dijo él.` keeps its dash, so the sentence
/// filter rejects it rather than learners seeing a half-punctuated speech tag.
/// Subtitles never come through here; their dashes mark speaker turns.
pub fn strip_dialogue_dash(text: &str) -> &str {
    let is_dash = |c: char| matches!(c, '—' | '–');
    match text.strip_prefix(['—', '–', '-']) {
        Some(rest) if !rest.contains(is_dash) => rest.trim_start(),
        _ => text,
    }
}

/// Only local input drops these invisible characters; Thai is reproduced exactly.
fn normalize_input(text: &str) -> String {
    text.chars()
        .filter(|c| !matches!(c, '\u{2060}' | '\u{feff}' | '\u{200b}' | '\u{00ad}'))
        .collect()
}

fn has_text(text: &str) -> bool {
    static TEXT: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"[\p{L}\p{N}]").unwrap());
    TEXT.is_match(text)
}

/// Starts partition the whole input: prefix joins the first piece, gaps join the
/// preceding piece, and no starts means one piece. Model ends are never used.
fn partition(text: &str, mut starts: Vec<usize>, language: Language) -> Vec<String> {
    if let Some(first) = starts.first_mut() {
        *first = 0;
    } else {
        starts.push(0);
    }
    starts.push(text.len());
    starts
        .windows(2)
        .map(|bounds| cleanup_sentence(text[bounds[0]..bounds[1]].trim().into(), language))
        .filter(|piece| has_text(piece))
        .collect()
}

pub fn segment_local(
    model: Option<&lexide::Segmenter>,
    text: &str,
    language: Language,
) -> anyhow::Result<Vec<String>> {
    let text = normalize_input(text);
    let starts = if language == Language::ChineseSimplified {
        let mut cursor = 0;
        crate::books::segment_chinese(&text)
            .into_iter()
            .map(|sentence| {
                let start = cursor + text[cursor..].find(&sentence).unwrap();
                cursor = start + sentence.len();
                start
            })
            .collect()
    } else {
        let offsets: Vec<_> = text
            .char_indices()
            .map(|(index, _)| index)
            .chain(std::iter::once(text.len()))
            .collect();
        let hint = match language {
            Language::French => lexide::Language::French,
            Language::English => lexide::Language::English,
            Language::German => lexide::Language::German,
            Language::Hindi => lexide::Language::Hindi,
            Language::Italian => lexide::Language::Italian,
            Language::Japanese => lexide::Language::Japanese,
            Language::Korean => lexide::Language::Korean,
            Language::Russian => lexide::Language::Russian,
            Language::PortugueseBrazilian | Language::PortugueseEuropean => {
                lexide::Language::PortugueseBrazil
            }
            Language::SpanishLatinAmerican | Language::SpanishPeninsular => {
                lexide::Language::SpanishEuro
            }
            _ => anyhow::bail!("unsupported local reader segmentation language: {language}"),
        };
        model
            .context("non-Chinese segmentation needs a local model")?
            .segment_detailed(&text, Some(hint))
            .into_iter()
            .map(|sentence| offsets[sentence.start])
            .collect()
    };
    Ok(partition(&text, starts, language))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_untagged_dash_dialogue_loses_its_dash() {
        for (input, expected) in [
            ("—¿Dónde está?", "¿Dónde está?"),
            ("— Bonjour, dit-il.", "Bonjour, dit-il."),
            ("– Нет.", "Нет."),
            ("—No —dijo él.", "—No —dijo él."),
            ("— Нет, — сказал он.", "— Нет, — сказал он."),
            ("Il partit.", "Il partit."),
        ] {
            assert_eq!(strip_dialogue_dash(input), expected, "{input}");
        }
    }

    #[test]
    fn dangling_quotes_are_stripped_only_at_an_edge_with_odd_parity() {
        for (input, expected) in [
            ("“ Hello", "Hello"),
            ("Hello ”", "Hello"),
            ("\"Hello", "Hello"),
            ("« bonjour", "bonjour"),
            ("bonjour »", "bonjour"),
            ("» Hallo", "Hallo"),
            ("Hallo «", "Hallo"),
            ("„Hallo", "Hallo"),
            ("‟Hello", "Hello"),
            ("‹ hello", "hello"),
            ("hello ›", "hello"),
            ("「こんにちは", "こんにちは"),
            ("こんにちは」", "こんにちは"),
            ("『こんにちは", "こんにちは"),
            ("こんにちは』", "こんにちは"),
            ("“hello “world”", "hello “world”"),
            ("เขาพูดว่า “สวัสดี", "เขาพูดว่า “สวัสดี"),
            ("”hello”", "”hello”"),
            ("»Hallo«", "»Hallo«"),
            ("'hello", "'hello"),
            ("l’homme", "l’homme"),
            (" “hello", " “hello"),
        ] {
            assert_eq!(strip_dangling_quote(input), expected, "{input}");
        }
    }

    #[test]
    fn reader_partition_keeps_prefix_and_gaps_without_model_ends() {
        let text = "prefix α. β!";
        assert_eq!(
            partition(
                text,
                vec![text.find('α').unwrap(), text.find('β').unwrap()],
                Language::English
            ),
            ["prefix α.", "β!"]
        );
        assert_eq!(partition(text, vec![], Language::English), [text]);
        assert_eq!(
            normalize_input("a\u{2060}b\u{feff}c\u{200b}d\u{00ad}e"),
            "abcde"
        );
        assert!(partition("...", vec![], Language::English).is_empty());
        assert!(has_text("논"));
        assert!(has_text("१"));
    }
}
