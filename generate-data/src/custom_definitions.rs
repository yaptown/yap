use anyhow::Context;
use language_utils::{Course, DictionaryDefinition, Heteronym, Language};
use std::{collections::BTreeMap, path::Path};

/// Hand-written dictionary entries that replace the generated ones. Each line
/// is `[heteronym, {"English": definition, "French": …}]`, one definition per
/// native language, and every course built from the target language must have
/// its native language there: an override that silently applied to one course
/// and not another would be a bug. A sense can set `"note_is_curated": true`
/// to have flashcards show its note.
pub fn load(
    source_data_path: &Path,
    course: Course,
) -> anyhow::Result<BTreeMap<Heteronym<String>, DictionaryDefinition>> {
    let path = source_data_path.join("custom_definitions.jsonl");
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let jsonl = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    jsonl
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let (heteronym, mut by_native): (
                Heteronym<String>,
                BTreeMap<Language, DictionaryDefinition>,
            ) = serde_json::from_str(line)
                .with_context(|| format!("Failed to parse {}", path.display()))?;
            let definition = by_native.remove(&course.native_language).with_context(|| {
                format!(
                    "{}: {} has no {:?} definition",
                    path.display(),
                    heteronym.word,
                    course.native_language
                )
            })?;
            Ok((heteronym, definition))
        })
        .collect()
}
