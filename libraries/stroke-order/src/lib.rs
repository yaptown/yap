//! Runtime stroke lookup, authored glyphs, and writable-unit segmentation.
mod authored;

use anyhow::{Result, ensure};
use language_utils::{StrokeGlyph, StrokeTable, WritingSystem};
use rustc_hash::FxHashMap;

/// Every accepted form of each authored character, the taught form first.
pub(crate) type Forms = FxHashMap<char, Vec<StrokeGlyph>>;

/// The writable units of `text`, in order, skipping nothing (whitespace and
/// undrawable characters are units too). One character per unit, except
/// Devanagari (the akshara) and Thai (a consonant with its stacked marks).
pub fn segment(system: WritingSystem, text: &str) -> Vec<&str> {
    match system {
        WritingSystem::Devanagari => authored::devanagari::segment(text),
        WritingSystem::Thai => authored::thai::segment(text),
        _ => text
            .char_indices()
            .map(|(i, c)| &text[i..i + c.len_utf8()])
            .collect(),
    }
}

/// A language's downloaded and authored forms, with its writable-unit composer.
pub struct StrokePack {
    forms: StrokeTable,
    composer: Composer,
    system: WritingSystem,
}

enum Composer {
    None,
    Devanagari(authored::devanagari::Devanagari),
    Thai(authored::thai::Thai),
}

impl StrokePack {
    /// `table` is the language pack's stroke table (empty for scripts drawn in code).
    pub fn new(system: WritingSystem, table: StrokeTable) -> Self {
        let mut forms = table;
        if system == WritingSystem::Cyrillic {
            forms.extend(
                authored::cyrillic::glyphs()
                    .into_iter()
                    .map(|(c, glyphs)| (c.to_string(), glyphs)),
            );
        }
        for (c, glyphs) in authored::common() {
            forms.entry(c.to_string()).or_insert(glyphs);
        }
        let composer = match system {
            WritingSystem::Devanagari => Composer::Devanagari(Default::default()),
            WritingSystem::Thai => Composer::Thai(Default::default()),
            _ => Composer::None,
        };
        Self {
            forms,
            composer,
            system,
        }
    }

    /// The writable units of `text`, including whitespace and undrawable units.
    pub fn segment<'a>(&self, text: &'a str) -> Vec<&'a str> {
        segment(self.system, text)
    }

    /// Every accepted form of a unit, taught form first; empty if undrawable.
    pub fn glyphs(&self, unit: &str) -> Vec<StrokeGlyph> {
        if let Some(forms) = self.forms.get(unit) {
            return forms.clone();
        }
        let found = match &self.composer {
            Composer::Devanagari(composer) => composer.glyphs(unit),
            Composer::Thai(composer) => composer.glyphs(unit),
            Composer::None => Vec::new(),
        };
        if !found.is_empty() {
            return found;
        }
        let mut chars = unit.chars();
        if let Some(c) = chars.next().filter(|_| chars.next().is_none())
            && let Some(base) = fold(c).filter(|&base| base != c)
        {
            return self.glyphs(base.encode_utf8(&mut [0; 4]));
        }
        Vec::new()
    }
}

/// The base character of a compatibility character, if it has exactly one.
pub fn fold(c: char) -> Option<char> {
    use unicode_normalization::UnicodeNormalization;
    let mut folded = c.nfkc();
    folded.next().filter(|_| folded.next().is_none())
}

pub fn validate(glyph: &StrokeGlyph) -> Result<()> {
    ensure!(!glyph.strokes.is_empty(), "glyph has no strokes");
    for stroke in &glyph.strokes {
        ensure!(stroke.points.len() >= 2, "stroke has fewer than two points");
        for point in &stroke.points {
            ensure!(
                [point.0, point.1]
                    .iter()
                    .all(|v| v.is_finite() && (0.0..=1.0).contains(v)),
                "point outside unit square: {point:?}"
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use language_utils::{Language, StrokeStandard};
    #[test]
    fn authored_packs_build() {
        for (glyphs, standard, min) in [
            (
                authored::thai::Thai::default().units,
                StrokeStandard::Thai,
                81,
            ),
            (authored::latin::glyphs(), StrokeStandard::Latin, 122),
            (authored::cyrillic::glyphs(), StrokeStandard::Cyrillic, 66),
            (authored::punctuation::glyphs(), StrokeStandard::Latin, 94),
        ] {
            assert!(glyphs.len() >= min, "{standard:?}: {} glyphs", glyphs.len());
            for (c, forms) in &glyphs {
                for glyph in forms {
                    assert_eq!(glyph.standard, standard);
                    validate(glyph).unwrap_or_else(|e| panic!("{standard:?} {c}: {e}"));
                }
            }
        }
    }

    #[test]
    fn accepted_forms() {
        let embedded =
            |language: Language| StrokePack::new(language.writing_system(), StrokeTable::default());
        let latin = embedded(Language::French);
        let cyrillic = embedded(Language::Russian);
        let hindi = embedded(Language::Hindi);
        for (pack, unit, forms) in [
            (&latin, "a", 4),
            (&latin, "b", 2),
            (&latin, "e", 2),
            (&latin, "f", 3),
            (&latin, "i", 2),
            (&latin, "q", 3),
            (&latin, "z", 5),
            (&latin, "à", 4),
            (&latin, "ã", 4),
            (&latin, "é", 2),
            (&latin, "ç", 2),
            (&latin, "ñ", 2),
            (&latin, "ü", 2),
            (&latin, "í", 2),
            (&latin, "ÿ", 3),
            (&latin, "4", 2),
            (&latin, "1", 3),
            (&latin, "A", 2),
            (&latin, "C", 1),
            (&latin, "M", 4),
            (&latin, "Z", 4),
            (&cyrillic, "а", 2),
            (&cyrillic, "о", 2),
            (&cyrillic, "т", 3),
            (&cyrillic, "ш", 3),
            (&cyrillic, "д", 5),
            (&cyrillic, "Д", 4),
            (&cyrillic, "б", 3),
            (&cyrillic, "Т", 2),
            (&latin, "ō", 2),
            (&latin, "ď", 2),
            (&latin, "ș", 2),
            (&hindi, "अ", 2),
            (&hindi, "क्ळ", 1),
            (&hindi, "ऴ", 1),
            (&latin, "?", 1),
            (&latin, "…", 2),
            (&latin, "‥", 2),
            (&latin, "&", 2),
            (&latin, "。", 1),
            (&cyrillic, "«", 1),
            (&hindi, "!", 1),
        ] {
            let glyphs = pack.glyphs(unit);
            assert_eq!(glyphs.len(), forms, "{unit}");
            for glyph in &glyphs {
                validate(glyph).unwrap_or_else(|e| panic!("{unit}: {e}"));
            }
        }
        // Aliases are written exactly like the mark they name, and fullwidth
        // and halfwidth punctuation folds onto the drawn marks.
        for (alias, mark) in [
            ("≪", "《"),
            ("⸺", "—"),
            ("〜", "~"),
            ("‑", "-"),
            ("！", "!"),
            ("？", "?"),
            ("（", "("),
            ("，", ","),
            ("～", "~"),
            ("｢", "「"),
            ("･", "・"),
        ] {
            assert!(!latin.glyphs(mark).is_empty(), "{mark}");
            assert_eq!(latin.glyphs(alias), latin.glyphs(mark), "{alias}");
        }
        // An accent composes onto every form of its letter, the taught one first.
        let (a, grave) = (latin.glyphs("a"), latin.glyphs("à"));
        for (a, grave) in a.iter().zip(&grave) {
            assert_eq!(a.strokes[..], grave.strokes[..a.strokes.len()]);
        }
    }
}
