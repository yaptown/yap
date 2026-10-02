use anyhow::{Context, ensure};
use language_utils::{Course, Gram, Language, PhrasebookDefinitionEntry};
use std::path::Path;

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Override {
    meaning: Option<String>,
    additional_notes: Option<String>,
    target_language_example: Option<String>,
    native_language_example: Option<String>,
}

/// Files live beside custom_definitions.jsonl, but are scoped to the native
/// language: custom_phrasebook_for_eng.jsonl contains English glosses only.
/// Each line is ["display text", {"meaning": "...", "additional_notes": "..."}],
/// with all fields optional. Examples can also be replaced via
/// target_language_example and native_language_example when a corrected gloss
/// needs a different example. All grams sharing the display text are updated.
pub fn apply(
    source_data_path: &Path,
    course: Course,
    phrasebook: &mut [(Gram<String>, PhrasebookDefinitionEntry)],
) -> anyhow::Result<()> {
    let path = source_data_path.join(format!(
        "custom_phrasebook_for_{}.jsonl",
        course.native_language.code()
    ));
    if path.exists() {
        let jsonl = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        apply_jsonl(&jsonl, course.target_language, phrasebook)
            .with_context(|| format!("Failed to apply {}", path.display()))?;
    }
    Ok(())
}

fn apply_jsonl(
    jsonl: &str,
    language: Language,
    phrasebook: &mut [(Gram<String>, PhrasebookDefinitionEntry)],
) -> anyhow::Result<()> {
    for (index, line) in jsonl.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let (text, patch): (String, Override) = serde_json::from_str(line)
            .with_context(|| format!("Invalid phrasebook override on line {}", index + 1))?;
        let mut matched = false;
        for (gram, entry) in phrasebook.iter_mut() {
            if gram.to_display_string(language) == text {
                matched = true;
                if let Some(meaning) = &patch.meaning {
                    entry.meaning.clone_from(meaning);
                }
                if let Some(notes) = &patch.additional_notes {
                    entry.additional_notes.clone_from(notes);
                }
                if let Some(example) = &patch.target_language_example {
                    entry.target_language_example.clone_from(example);
                }
                if let Some(example) = &patch.native_language_example {
                    entry.native_language_example.clone_from(example);
                }
            }
        }
        ensure!(matched, "Phrasebook override {text:?} matches no gram");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use language_utils::{Atom, OtherWord, OtherWordType, Word, WordType};

    fn fixture() -> (Gram<String>, PhrasebookDefinitionEntry) {
        let gram = Gram::new(
            ["je", "dois"]
                .into_iter()
                .map(|text| {
                    Atom::Tok(Word {
                        text: text.to_owned(),
                        word_type: WordType::Other(OtherWord {
                            other_tag: OtherWordType::X,
                        }),
                    })
                })
                .collect(),
        );
        let entry = PhrasebookDefinitionEntry {
            target_language_multi_word_term: "je dois".into(),
            meaning: "owe, must".into(),
            additional_notes: "Original notes".into(),
            target_language_example: "Je dois partir.".into(),
            native_language_example: "I must leave.".into(),
            informal: false,
            compositional: true,
            cognate: false,
            false_cognate: false,
            can_be_translated_literally: true,
        };
        (gram, entry)
    }

    #[test]
    fn partial_overrides_update_all_matching_grams() {
        let (gram, original) = fixture();
        let mut other_gram = gram.clone();
        if let Atom::Tok(word) = &mut other_gram.0[0] {
            word.word_type = WordType::Other(OtherWord {
                other_tag: OtherWordType::Propn,
            });
        }
        let mut entries = vec![(gram, original.clone()), (other_gram, original.clone())];
        apply_jsonl(
            "\n[\"je dois\",{\"meaning\":\"I must; I owe\"}]\n",
            Language::French,
            &mut entries,
        )
        .unwrap();
        let mut expected = original;
        expected.meaning = "I must; I owe".into();
        for (_, entry) in &entries {
            assert_eq!(entry, &expected);
        }
        apply_jsonl(
            r#"["je dois",{"additional_notes":"","target_language_example":"Je dois dix euros.","native_language_example":"I owe ten euros."}]"#,
            Language::French,
            &mut entries,
        )
        .unwrap();
        expected.additional_notes.clear();
        expected.target_language_example = "Je dois dix euros.".into();
        expected.native_language_example = "I owe ten euros.".into();
        for (_, entry) in &entries {
            assert_eq!(entry, &expected);
        }
    }

    #[test]
    fn rejects_unmatched_text_and_unknown_fields() {
        let mut entries = vec![fixture()];
        let error = apply_jsonl(
            r#"["typo",{"meaning":"I must"}]"#,
            Language::French,
            &mut entries,
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "Phrasebook override \"typo\" matches no gram"
        );
        assert!(
            apply_jsonl(
                r#"["je dois",{"meanings":["I must"]}]"#,
                Language::French,
                &mut entries,
            )
            .is_err()
        );
    }

    #[test]
    fn loader_scopes_overrides_to_native_language() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("custom_phrasebook_for_eng.jsonl"),
            r#"["je dois",{"meaning":"I must; I owe"}]"#,
        )
        .unwrap();
        let mut entries = vec![fixture()];
        let mut course = Course {
            target_language: Language::French,
            native_language: Language::Italian,
        };
        apply(dir.path(), course, &mut entries).unwrap();
        assert_eq!(entries[0].1.meaning, "owe, must");
        course.native_language = Language::English;
        apply(dir.path(), course, &mut entries).unwrap();
        assert_eq!(entries[0].1.meaning, "I must; I owe");
    }
}
