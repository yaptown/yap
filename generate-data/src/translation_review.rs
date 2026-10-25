//! Human translation decisions and the short-Japanese translation gate.
use anyhow::Context;
use language_utils::{Course, Language, Literal, OtherWordType, PartOfSpeech, WordType};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

#[derive(Default)]
pub struct Decisions {
    overrides: BTreeMap<String, Vec<String>>,
    reviewed: BTreeSet<String>,
}

impl Decisions {
    pub fn load(directory: &Path, course: Course) -> anyhow::Result<Self> {
        let mut result = Self::default();
        let path = directory.join("translation_overrides.jsonl");
        if path.exists() {
            result.overrides =
                Self::parse_overrides(&std::fs::read_to_string(&path)?, course.native_language)
                    .with_context(|| format!("Failed to load {}", path.display()))?;
        }
        let path = directory.join("translations_reviewed.jsonl");
        if path.exists() {
            for line in std::fs::read_to_string(&path)?
                .lines()
                .filter(|l| !l.is_empty())
            {
                let (text, languages): (String, Vec<Language>) = serde_json::from_str(line)
                    .with_context(|| format!("Failed to parse {}", path.display()))?;
                if languages.contains(&course.native_language) {
                    result.reviewed.insert(text);
                }
            }
        }
        Ok(result)
    }

    fn parse_overrides(
        jsonl: &str,
        native: Language,
    ) -> anyhow::Result<BTreeMap<String, Vec<String>>> {
        jsonl
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| {
                let (text, mut by_native): (String, BTreeMap<Language, Vec<String>>) =
                    serde_json::from_str(line)?;
                let translations = by_native
                    .remove(&native)
                    .with_context(|| format!("{text} has no {native:?} translations"))?;
                Ok((text, translations))
            })
            .collect()
    }

    pub fn override_for(&self, text: &str) -> Option<&[String]> {
        self.overrides.get(text).map(Vec::as_slice)
    }

    fn should_review(&self, text: &str, tokens: usize) -> bool {
        tokens <= 3 && !self.overrides.contains_key(text) && !self.reviewed.contains(text)
    }
}

pub fn token_count(words: &[Literal<String>]) -> usize {
    words
        .iter()
        .filter(|literal| match &literal.word.word_type {
            WordType::Heteronym(h) => h.pos != PartOfSpeech::Sym,
            WordType::Other(other) => other.other_tag == OtherWordType::Propn,
        })
        .count()
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct Judgment {
    /// A single replacement for the entire set, or null to keep it unchanged.
    replacement: Option<String>,
}

pub async fn review(
    course: Course,
    decisions: &Decisions,
    token_counts: &BTreeMap<String, usize>,
    translations: &mut BTreeMap<String, Vec<String>>,
    client: &tysm::chat_completions::ChatClient,
) -> anyhow::Result<()> {
    if course.target_language != Language::Japanese {
        return Ok(());
    }
    let prompts: Vec<_> = translations
        .iter()
        .filter(|(text, _)| {
            token_counts
                .get(*text)
                .is_some_and(|&count| decisions.should_review(text, count))
        })
        .map(|(text, translations)| {
            (
                text.clone(),
                format!(
                    "Sentence: {text}\nTranslations: {}",
                    serde_json::to_string(translations).unwrap()
                ),
            )
        })
        .collect();
    let system = include_str!("translation_review_prompt.txt");
    let responses = client
        .batch_chat_with_system_prompt_fn::<_, _, Judgment>(
            system,
            &prompts,
            |(_, prompt)| prompt.clone(),
            |_| {},
        )
        .await?;
    let mut replaced = 0;
    for ((text, _), result) in responses {
        if let Some(replacement) = result
            .with_context(|| format!("Reviewing translation of {text}"))?
            .replacement
        {
            anyhow::ensure!(
                !replacement.trim().is_empty(),
                "Empty translation replacement for {text}"
            );
            translations.insert(text.clone(), vec![replacement]);
            replaced += 1;
        }
    }
    println!(
        "translation review: judged {}, replaced {replaced}",
        prompts.len()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn overridden_sentences_skip_review() {
        let decisions = Decisions {
            overrides: Decisions::parse_overrides("[\"上野です\",{\"English\":[\"It's Ueno.\"]}]\n[\"unused\",{\"English\":[\"unused\"]}]", Language::English).unwrap(),
            ..Default::default()
        };
        assert!(!decisions.should_review("上野です", 2));
    }
    #[test]
    fn missing_native_is_an_error() {
        let error =
            Decisions::parse_overrides("[\"上野です\",{\"French\":[\"Ueno\"]}]", Language::English)
                .unwrap_err();
        assert_eq!(error.to_string(), "上野です has no English translations");
    }
    #[test]
    fn missing_native_is_strict_even_for_unused_non_japanese_overrides() {
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(directory.path().join("translation_overrides.jsonl"),
            "[\"used\",{\"English\":[\"hello\"],\"French\":[\"bonjour\"]}]\n[\"unused\",{\"English\":[\"unused\"]}]").unwrap();
        for target_language in [Language::PortugueseBrazilian, Language::English] {
            let result = Decisions::load(
                directory.path(),
                Course {
                    target_language,
                    native_language: Language::French,
                },
            );
            assert!(
                format!("{:#}", result.err().unwrap())
                    .contains("unused has no French translations")
            );
        }
    }

    #[test]
    fn reviewed_only_skips_judgment() {
        let decisions = Decisions {
            reviewed: BTreeSet::from(["reviewed".into()]),
            ..Default::default()
        };
        assert!(!decisions.should_review("reviewed", 1));
        assert!(decisions.should_review("other", 2));
        assert!(decisions.should_review("other", 3));
        assert!(!decisions.should_review("other", 4));
    }
    #[test]
    fn punctuation_symbols_unknowns_and_spaces_do_not_count() {
        use language_utils::{OtherWord, Word};
        let mut words = [
            OtherWordType::Punct,
            OtherWordType::Space,
            OtherWordType::X,
            OtherWordType::Propn,
        ]
        .into_iter()
        .map(|other_tag| Literal {
            word: Word {
                text: "x".into(),
                word_type: WordType::Other(OtherWord { other_tag }),
            },
            whitespace: language_utils::Whitespace::None,
        })
        .collect::<Vec<_>>();
        words.push(Literal {
            word: Word {
                text: "$".into(),
                word_type: WordType::Heteronym(language_utils::Heteronym {
                    word: "$".into(),
                    lemma: "$".into(),
                    pos: PartOfSpeech::Sym,
                }),
            },
            whitespace: language_utils::Whitespace::None,
        });
        assert_eq!(token_count(&words), 1);
    }
}
