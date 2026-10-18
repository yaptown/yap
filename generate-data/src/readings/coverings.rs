//! Exhaustive CEDICT coverings, independent of the tokenizer's boundaries.
use super::*;

// Exceeding these bounds is an explicit error, never silent semantic pruning.
const MAX_CHARACTERS: usize = 64;
const MAX_COVERINGS: usize = 16_384;

/// A dictionary span retains its independently citable source senses. A covering
/// is just a sequence of these parts; it has no synthetic dictionary-source ID.
#[derive(Clone, Serialize)]
pub(super) struct Part {
    pub start: usize,
    pub end: usize,
    pub text: String,
    #[serde(flatten)]
    pub entry: DictionaryReading,
}
pub(super) type Covering = Vec<Part>;

pub(super) fn readings(surface: &str, dictionary: &Dictionary) -> BTreeMap<String, Vec<Covering>> {
    let mut boundaries: Vec<_> = surface.char_indices().map(|(i, _)| i).collect();
    boundaries.push(surface.len());
    assert!(
        boundaries.len() <= MAX_CHARACTERS + 1,
        "CEDICT covering exceeds {MAX_CHARACTERS} characters: {surface}"
    );
    let mut suffixes: Vec<Vec<Covering>> = vec![vec![]; boundaries.len()];
    suffixes.last_mut().unwrap().push(vec![]);
    static SILENT: std::sync::LazyLock<regex::Regex> =
        std::sync::LazyLock::new(|| regex::Regex::new(r"\A[\p{P}\s]\z").unwrap());
    for start in (0..boundaries.len() - 1).rev() {
        let character = &surface[boundaries[start]..boundaries[start + 1]];
        if SILENT.is_match(character) {
            // Silent separators need no dictionary entry. Keep the original
            // offsets: skipping a boundary must not invent a joined headword.
            suffixes[start] = suffixes[start + 1].clone();
        }
        for end in start + 1..boundaries.len() {
            let span = &surface[boundaries[start]..boundaries[end]];
            let Some(entries) = dictionary.get(span) else {
                continue;
            };
            for entry in normalized_entries(span, Language::ChineseSimplified, entries) {
                let mut primary_groups: BTreeMap<String, Vec<DictionaryEvidence>> = BTreeMap::new();
                for evidence in &entry.evidence {
                    primary_groups
                        .entry(evidence.primary.clone())
                        .or_default()
                        .push(evidence.clone());
                }
                if primary_groups.is_empty() {
                    primary_groups.insert(entry.reading.clone(), vec![]);
                }
                for evidence in primary_groups.into_values() {
                    for tail in suffixes[end].clone() {
                        let mut parts = vec![Part {
                            start: boundaries[start],
                            end: boundaries[end],
                            text: span.into(),
                            entry: DictionaryReading {
                                evidence: evidence.clone(),
                                ..entry.clone()
                            },
                        }];
                        parts.extend(tail);
                        suffixes[start].push(parts);
                        assert!(
                            suffixes[start].len() <= MAX_COVERINGS,
                            "CEDICT covering exceeds {MAX_COVERINGS} alternatives: {surface}"
                        );
                    }
                }
            }
        }
    }
    let mut result = BTreeMap::<String, Vec<Covering>>::new();
    for parts in std::mem::take(&mut suffixes[0]) {
        let reading = parts
            .iter()
            .map(|p| p.entry.reading.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        if let Some(reading) =
            canonical::checked_surface_reading(Language::ChineseSimplified, surface, &reading)
        {
            result.entry(reading).or_default().push(parts);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn silent_boundaries_preserve_spans_without_joining_dictionary_headwords() {
        let dictionary: Dictionary = [
            ("不了", "bù le"),
            ("不", "bù"),
            ("了", "liǎo"),
            ("我", "wǒ"),
        ]
        .into_iter()
        .map(|(word, reading)| (word.into(), vec![reading.into()]))
        .collect();
        let paths = readings("不了，我", &dictionary);
        let refusal = &paths["bù le wǒ"][0];
        assert_eq!(
            refusal
                .iter()
                .map(|p| (p.start, p.end, p.text.as_str()))
                .collect::<Vec<_>>(),
            [(0, 6, "不了"), (9, 12, "我")]
        );
        for surface in ["不，了", "不 了", "不。了"] {
            let paths = readings(surface, &dictionary);
            assert_eq!(paths.into_keys().collect::<Vec<_>>(), ["bù liǎo"]);
        }
        let surrounded = readings("「不了！」", &dictionary);
        assert_eq!(surrounded["bù le"][0][0].start, "「".len());
        assert_eq!(surrounded["bù le"][0][0].end, "「不了".len());
        for unknown in ["不了X我", "不了猫我", "不了の我", "不了3我", "不了+我"] {
            assert!(readings(unknown, &dictionary).is_empty());
        }
    }

    #[test]
    fn keeps_every_complete_covering_inside_unknown_tokens() {
        let dictionary: Dictionary = [
            ("留", "liú"),
            ("留下", "liú xià"),
            ("下来", "xià lai"),
            ("来", "lái"),
            ("一个人", "yī gè rén"),
            ("一个", "yī ge"),
            ("人", "rén"),
        ]
        .into_iter()
        .map(|(word, reading)| (word.into(), vec![reading.into()]))
        .collect();
        let forms = |text| {
            readings(text, &dictionary)
                .into_keys()
                .collect::<BTreeSet<_>>()
        };
        assert_eq!(
            forms("留下来"),
            BTreeSet::from(["liú xià lái".into(), "liú xià lai".into()])
        );
        assert_eq!(
            forms("一个人"),
            BTreeSet::from(["yī ge rén".into(), "yī gè rén".into()])
        );
        assert!(forms("留下未知").is_empty());
        let paths = readings("留下来", &dictionary);
        let parts = &paths["liú xià lai"][0];
        assert_eq!(
            parts
                .iter()
                .map(|p| (p.start, p.end, p.text.as_str()))
                .collect::<Vec<_>>(),
            [(0, 3, "留"), (3, 9, "下来")]
        );
    }
}
