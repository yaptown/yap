//! Dictionary-backed Japanese display segmentation (not reading verification).
//! JmdictFurigana 2.3.1+2026-10-25, CC BY-SA (same data license as JMdict):
//! https://github.com/Doublevil/JmdictFurigana#licence
use super::kanjidic::{self, Verifier};
use anyhow::Result;
use language_utils::{
    Language, SentenceGram,
    language_pack::LanguagePack,
    readings::{RubySegment, canonical_kana, ruby_segments},
};
use std::{collections::BTreeSet, path::Path};

const DIRECTORY: &str = ".cache/readings/jmdict-furigana-2.3.1-2026-10-25";
const URL: &str = "https://github.com/Doublevil/JmdictFurigana/releases/download/2.3.1%2B2026-10-25/JmdictFurigana.json";

#[derive(serde::Deserialize)]
struct Entry {
    text: String,
    reading: String,
    furigana: Vec<Segment>,
}
#[derive(serde::Deserialize)]
struct Segment {
    ruby: String,
    rt: Option<String>,
}

pub struct Segmenter(Verifier);
impl Segmenter {
    pub async fn load() -> Result<Self> {
        let path = Path::new(DIRECTORY).join("JmdictFurigana.json");
        if !path.exists() {
            std::fs::create_dir_all(DIRECTORY)?;
            let bytes = reqwest::get(URL).await?.error_for_status()?.bytes().await?;
            std::fs::write(&path, bytes)?;
        }
        let read = |name: &str| std::fs::read_to_string(format!(".cache/readings/{name}"));
        let mut verifier = Verifier::new(
            serde_json::from_str(&read("jpn-kanjidic.json")?)?,
            &serde_json::from_str(&read("jpn-dictionary.json")?)?,
            serde_json::from_str(&read("jpn-reading-stems.json")?)?,
        );
        let json = std::fs::read_to_string(path)?;
        let entries: Vec<Entry> = serde_json::from_str(json.trim_start_matches('\u{feff}'))?;
        for entry in entries {
            let segments: Vec<_> = entry
                .furigana
                .into_iter()
                .map(|s| RubySegment {
                    base: s.ruby,
                    ruby: s.rt.map(|rt| rt.chars().map(canonical_kana).collect()),
                })
                .collect();
            let text: String = segments.iter().map(|s| s.base.as_str()).collect();
            let reading: String = segments
                .iter()
                .flat_map(|s| {
                    s.ruby
                        .as_deref()
                        .unwrap_or(&s.base)
                        .chars()
                        .map(canonical_kana)
                })
                .collect();
            // Upstream data is evidence, not permission to alter the visible text
            // or silently change its pronunciation.
            if text == entry.text
                && reading
                    == entry
                        .reading
                        .chars()
                        .map(canonical_kana)
                        .collect::<String>()
                && segments.iter().all(|s| {
                    s.ruby.is_none()
                        || s.base
                            .chars()
                            .all(|c| language_utils::readings::is_ruby_base(c, Language::Japanese))
                })
            {
                verifier.add_furigana(entry.text, reading, segments);
            }
        }
        Ok(Self(verifier))
    }

    pub fn segment(&self, text: &str, reading: &str) -> Option<Vec<RubySegment>> {
        kanjidic::ruby(text, reading, &self.0)
    }

    pub fn populate(&self, pack: &mut LanguagePack) {
        pack.ruby_segmentations = pairs(pack)
            .into_iter()
            .filter_map(|(text, reading)| {
                let segments = self.segment(&text, &reading)?;
                (segments != ruby_segments(&text, &reading, Language::Japanese))
                    .then_some(((text, reading), segments))
            })
            .collect();
    }
}

/// Exact word/card forms and exact occurrence forms, including internal whitespace.
pub fn pairs(pack: &LanguagePack) -> BTreeSet<(String, String)> {
    let mut pairs = BTreeSet::new();
    for (gram, readings) in &pack.gram_readings {
        let text = pack
            .resolve_gram(&gram.gram)
            .to_display_string(Language::Japanese);
        for reading in readings {
            pairs.insert((text.clone(), pack.string_rodeo.resolve(reading).into()));
        }
    }
    for sentence in pack.encoded_sentences.values() {
        let literals = sentence.clone().without_readings().to_literals(
            &pack.string_rodeo,
            &pack.gram_rodeo,
            Language::Japanese,
        );
        for (gram, literals) in sentence.grams.iter().zip(literals) {
            let (SentenceGram::Learnable(gram) | SentenceGram::Obvious(gram)) = gram;
            let (SentenceGram::Learnable((_, literals)) | SentenceGram::Obvious((_, literals))) =
                literals;
            if let Some(reading) = gram.reading {
                let mut text: String = literals
                    .iter()
                    .flat_map(|l| [l.word.text.as_str(), l.whitespace.as_str()])
                    .collect();
                if let Some(last) = literals.last() {
                    text.truncate(text.len() - last.whitespace.as_str().len());
                }
                pairs.insert((text, pack.string_rodeo.resolve(&reading).into()));
            }
        }
    }
    pairs
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    #[ignore = "requires cached Japanese dictionaries and downloaded JmdictFurigana"]
    async fn dictionary_segmentations() {
        std::env::set_current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()).unwrap();
        let segmenter = Segmenter::load().await.unwrap();
        let ruby = |base: &str, reading: &str| RubySegment {
            base: base.into(),
            ruby: Some(reading.into()),
        };
        let plain = |base: &str| RubySegment {
            base: base.into(),
            ruby: None,
        };
        for (text, reading, expected) in [
            ("上野", "うえの", vec![ruby("上", "うえ"), ruby("野", "の")]),
            (
                "学校",
                "がっこう",
                vec![ruby("学", "がっ"), ruby("校", "こう")],
            ),
            ("手紙", "てがみ", vec![ruby("手", "て"), ruby("紙", "がみ")]),
            ("今日", "きょう", vec![ruby("今日", "きょう")]),
            (
                "今日中",
                "きょうじゅう",
                vec![ruby("今日", "きょう"), ruby("中", "じゅう")],
            ),
            ("大人", "おとな", vec![ruby("大人", "おとな")]),
            ("一人", "ひとり", vec![ruby("一人", "ひとり")]),
            ("上手く", "うまく", vec![ruby("上手", "うま"), plain("く")]),
            (
                "お土産",
                "おみやげ",
                vec![plain("お"), ruby("土産", "みやげ")],
            ),
            (
                "承る",
                "うけたまわる",
                vec![ruby("承", "うけたまわ"), plain("る")],
            ),
            (
                "東 京",
                "とうきょう",
                vec![ruby("東", "とう"), plain(" "), ruby("京", "きょう")],
            ),
        ] {
            assert_eq!(segmenter.segment(text, reading), Some(expected), "{text}");
        }
        for (text, reading) in [("食べる", "たべる"), ("取り消す", "とりけす")] {
            assert_eq!(
                segmenter.segment(text, reading).unwrap(),
                ruby_segments(text, reading, Language::Japanese)
            );
        }
    }
}
