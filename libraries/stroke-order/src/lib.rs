//! Build-time conversion of pinned, attributed stroke-order sources.
mod authored;
mod korean;
pub use korean::parse_scribing;
mod types;
pub use types::{Stroke, StrokeGlyph, StrokeStandard};

use anyhow::{Context, Result, bail, ensure};
use language_utils::Language;
use rustc_hash::FxHashMap;
use std::{
    future::Future,
    io::{Cursor, Read},
};

pub const KANJIVG_COMMIT: &str = "422b5538595676da918c288a4230cb5e22a1ee7e";
pub const KANJIVG_URL: &str =
    "https://codeload.github.com/KanjiVG/kanjivg/zip/422b5538595676da918c288a4230cb5e22a1ee7e";
pub const MMAH_COMMIT: &str = "bddc96d41bef78427ed0e034e9f7e31d71fd1b92";
pub const MMAH_URL: &str = "https://raw.githubusercontent.com/skishore/makemeahanzi/bddc96d41bef78427ed0e034e9f7e31d71fd1b92/graphics.txt";
pub const ANIMCJK_COMMIT: &str = "ec5e17cca76c87587790bcbce5ea0b4d4fb753d6";
pub const ANIMCJK_URL: &str = "https://raw.githubusercontent.com/parsimonhi/animCJK/ec5e17cca76c87587790bcbce5ea0b4d4fb753d6/graphicsZhHant.txt";

pub const SCRIBING_COMMIT: &str = "fbd28a4de6bbeb3ee07ecbb0517d0054b1919791";
pub const SCRIBING_URL: &str = "https://raw.githubusercontent.com/xiaolai/scribing/fbd28a4de6bbeb3ee07ecbb0517d0054b1919791/packs/generated/korean-textbook.json";

/// One glyph per character, as the downloaded sources publish them.
pub type Glyphs = FxHashMap<char, StrokeGlyph>;
/// Every accepted form of each character, the taught form first.
pub type Forms = FxHashMap<char, Vec<StrokeGlyph>>;

/// A language's stroke order: how its text splits into writable units, and
/// the strokes of each unit.
pub struct StrokePack {
    script: Pack,
    /// What every language's text contains besides its script: Latin letters,
    /// digits and punctuation. Consulted when the script has no glyph.
    common: Forms,
}

enum Pack {
    /// The glyphs of each character.
    Chars(Forms),
    /// The 40 Korean jamo; syllables are composed from them on demand.
    Hangul(Glyphs),
    Devanagari(authored::devanagari::Devanagari),
    Thai(authored::thai::Thai),
}

impl StrokePack {
    /// The writable units of `text`, in order and skipping nothing: whitespace
    /// and characters the pack cannot draw are units too, with no glyph. One
    /// character per unit, except Hindi, whose unit is the akshara (a
    /// consonant cluster or vowel with its marks), and Thai, whose unit is a
    /// consonant with its stacked marks.
    pub fn segment<'a>(&self, text: &'a str) -> Vec<&'a str> {
        match &self.script {
            Pack::Devanagari(_) => authored::devanagari::segment(text),
            Pack::Thai(_) => authored::thai::segment(text),
            Pack::Chars(_) | Pack::Hangul(_) => text
                .char_indices()
                .map(|(i, c)| &text[i..i + c.len_utf8()])
                .collect(),
        }
    }

    /// Every accepted way to write one unit from [`Self::segment`]: the
    /// taught form first, then the alternatives a learner may write instead.
    /// Empty if the pack cannot draw the unit.
    pub fn glyphs(&self, unit: &str) -> Vec<StrokeGlyph> {
        let mut chars = unit.chars();
        let single = chars.next().filter(|_| chars.next().is_none());
        let found = match (&self.script, single) {
            (Pack::Chars(forms), Some(c)) => forms.get(&c).cloned().unwrap_or_default(),
            (Pack::Hangul(jamo), Some(c)) => korean::glyph(c, jamo).into_iter().collect(),
            (Pack::Devanagari(devanagari), _) => devanagari.glyphs(unit),
            (Pack::Thai(thai), _) => thai.glyphs(unit),
            (Pack::Chars(_) | Pack::Hangul(_), None) => Vec::new(),
        };
        if !found.is_empty() {
            return found;
        }
        let Some(c) = single else {
            return Vec::new();
        };
        if let Some(forms) = self.common.get(&c) {
            return forms.clone();
        }
        // A compatibility form (fullwidth Ａ, halfwidth ｱ, ｢) is written like
        // the character it is compatible with.
        match fold(c) {
            Some(base) if base != c => self.glyphs(base.encode_utf8(&mut [0; 4])),
            _ => Vec::new(),
        }
    }
}

/// The base character of a compatibility character, if it has exactly one.
fn fold(c: char) -> Option<char> {
    use unicode_normalization::UnicodeNormalization;
    let mut folded = c.nfkc();
    folded.next().filter(|_| folded.next().is_none())
}

/// Sources in priority order as one map: a character's glyph comes from the
/// first source that has it. Later sources fill gaps in the national one
/// (old forms in subtitles, characters only another standard covers), each
/// glyph keeping the [`StrokeStandard`] it was drawn to.
fn chain(sources: impl IntoIterator<Item = Glyphs>) -> Forms {
    let mut forms = Forms::default();
    for source in sources {
        for (c, glyph) in source {
            forms.entry(c).or_insert_with(|| vec![glyph]);
        }
    }
    forms
}

/// The caller owns networking and caching; immutable source URLs are cache keys.
pub async fn load<F, Fut>(language: Language, mut fetch: F) -> Result<StrokePack>
where
    F: FnMut(&'static str) -> Fut,
    Fut: Future<Output = Result<Vec<u8>>>,
{
    let script = match language {
        Language::Korean => Pack::Hangul(parse_scribing(&fetch(SCRIBING_URL).await?)?),
        Language::Hindi => Pack::Devanagari(Default::default()),
        Language::Thai => Pack::Thai(Default::default()),
        Language::Russian => Pack::Chars(authored::cyrillic::glyphs()),
        Language::French
        | Language::English
        | Language::SpanishLatinAmerican
        | Language::SpanishPeninsular
        | Language::German
        | Language::PortugueseBrazilian
        | Language::PortugueseEuropean
        | Language::Italian => Pack::Chars(authored::latin::glyphs()),
        Language::Japanese => {
            let japan = parse_kanjivg(&fetch(KANJIVG_URL).await?)?;
            let prc = parse_medians(&fetch(MMAH_URL).await?, StrokeStandard::Prc)?;
            Pack::Chars(chain([japan, prc]))
        }
        Language::ChineseSimplified => {
            let prc = parse_medians(&fetch(MMAH_URL).await?, StrokeStandard::Prc)?;
            let taiwan = parse_medians(&fetch(ANIMCJK_URL).await?, StrokeStandard::Taiwan)?;
            let japan = parse_kanjivg(&fetch(KANJIVG_URL).await?)?;
            Pack::Chars(chain([prc, taiwan, japan]))
        }
        Language::ChineseTraditional => {
            let taiwan = parse_medians(&fetch(ANIMCJK_URL).await?, StrokeStandard::Taiwan)?;
            let prc = parse_medians(&fetch(MMAH_URL).await?, StrokeStandard::Prc)?;
            let japan = parse_kanjivg(&fetch(KANJIVG_URL).await?)?;
            Pack::Chars(chain([taiwan, prc, japan]))
        }
    };
    Ok(StrokePack {
        script,
        common: authored::common(),
    })
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

pub fn parse_medians(bytes: &[u8], standard: StrokeStandard) -> Result<Glyphs> {
    #[derive(serde::Deserialize)]
    struct Record {
        character: char,
        medians: Vec<Vec<(f32, f32)>>,
    }
    let mut glyphs = Glyphs::default();
    for line in std::str::from_utf8(bytes)?
        .lines()
        .filter(|s| !s.is_empty())
    {
        let record: Record = serde_json::from_str(line)?;
        let glyph = StrokeGlyph {
            standard,
            strokes: record
                .medians
                .into_iter()
                .map(|points| Stroke {
                    points: points
                        .into_iter()
                        .map(|(x, y)| (x / 1024.0, (900.0 - y) / 1024.0))
                        .collect(),
                })
                .collect(),
        };
        validate(&glyph).with_context(|| format!("{standard:?} {:?}", record.character))?;
        ensure!(
            glyphs.insert(record.character, glyph).is_none(),
            "duplicate character"
        );
    }
    ensure!(!glyphs.is_empty(), "source contains no glyphs");
    Ok(glyphs)
}

pub fn parse_kanjivg(bytes: &[u8]) -> Result<Glyphs> {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes))?;
    let prefix = format!("kanjivg-{KANJIVG_COMMIT}/kanji/");
    let mut glyphs = Glyphs::default();
    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let Some(hex) = file
            .name()
            .strip_prefix(&prefix)
            .and_then(|s| s.strip_suffix(".svg"))
        else {
            continue;
        };
        if !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            continue;
        }
        let character = char::from_u32(u32::from_str_radix(hex, 16)?).context("invalid scalar")?;
        let mut svg = String::new();
        file.read_to_string(&mut svg)?;
        let glyph = parse_svg(&svg).with_context(|| format!("KanjiVG {character}"))?;
        glyphs.insert(character, glyph);
    }
    ensure!(!glyphs.is_empty(), "source contains no glyphs");
    Ok(glyphs)
}

fn parse_svg(svg: &str) -> Result<StrokeGlyph> {
    // KanjiVG contains a DTD but needs no external entities. Namespace matching
    // uses the SVG URI, never a prefix (kvg's metadata prefix is unrelated).
    let doc = roxmltree::Document::parse_with_options(
        svg,
        roxmltree::ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        },
    )?;
    let group = doc
        .descendants()
        .find(|n| {
            n.has_tag_name(("http://www.w3.org/2000/svg", "g"))
                && n.attribute("id")
                    .is_some_and(|id| id.starts_with("kvg:StrokePaths_"))
        })
        .context("missing StrokePaths")?;
    let strokes = group
        .descendants()
        .filter(|n| n.has_tag_name(("http://www.w3.org/2000/svg", "path")))
        .map(|n| flatten(n.attribute("d").context("missing path data")?))
        .collect::<Result<_>>()?;
    let glyph = StrokeGlyph {
        standard: StrokeStandard::Japan,
        strokes,
    };
    validate(&glyph)?;
    Ok(glyph)
}

/// Twelve directed samples per cubic, with both exact endpoints retained.
fn flatten(path: &str) -> Result<Stroke> {
    use svgtypes::PathSegment;
    let mut points = Vec::new();
    let mut current = (0.0_f64, 0.0_f64);
    let mut control = None;
    for segment in svgtypes::PathParser::from(path) {
        let segment = segment?;
        let absolute = |abs, x, y| {
            if abs {
                (x, y)
            } else {
                (current.0 + x, current.1 + y)
            }
        };
        let (a, b, end) = match segment {
            PathSegment::MoveTo { abs, x, y } => {
                ensure!(points.is_empty(), "multiple subpaths in stroke");
                current = absolute(abs, x, y);
                points.push(current);
                control = None;
                continue;
            }
            PathSegment::CurveTo {
                abs,
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => (
                absolute(abs, x1, y1),
                absolute(abs, x2, y2),
                absolute(abs, x, y),
            ),
            PathSegment::SmoothCurveTo { abs, x2, y2, x, y } => {
                let a = control.map_or(current, |c: (f64, f64)| {
                    (2.0 * current.0 - c.0, 2.0 * current.1 - c.1)
                });
                (a, absolute(abs, x2, y2), absolute(abs, x, y))
            }
            other => bail!("unsupported KanjiVG path command: {other:?}"),
        };
        ensure!(!points.is_empty(), "curve before move");
        for i in 1..12 {
            let t = f64::from(i) / 12.0;
            let u = 1.0 - t;
            let sample = |start, a, b, end| {
                u * u * u * start + 3.0 * u * u * t * a + 3.0 * u * t * t * b + t * t * t * end
            };
            points.push((
                sample(current.0, a.0, b.0, end.0),
                sample(current.1, a.1, b.1, end.1),
            ));
        }
        points.push(end);
        current = end;
        control = Some(b);
    }
    let mut points: Vec<_> = points
        .into_iter()
        .map(|(x, y)| ((x / 109.0) as f32, (y / 109.0) as f32))
        .collect();
    // Remove redundant interior samples, never the directed endpoints.
    if points.len() > 2 {
        let end = points.pop().unwrap();
        points.dedup_by(|a, b| (a.0 - b.0).hypot(a.1 - b.1) < 1e-6);
        while points.len() > 1
            && (points.last().unwrap().0 - end.0).hypot(points.last().unwrap().1 - end.1) < 1e-6
        {
            points.pop();
        }
        points.push(end);
    }
    Ok(Stroke { points })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cubic_direction_and_exact_endpoints() {
        let stroke = flatten("m10,20 c10,0 20,0 30,0 s20,0 30,0").unwrap();
        assert_eq!(stroke.points.len(), 25);
        assert_eq!(stroke.points[0], (10.0 / 109.0, 20.0 / 109.0));
        assert_eq!(*stroke.points.last().unwrap(), (70.0 / 109.0, 20.0 / 109.0));
        assert!(stroke.points.windows(2).all(|w| w[0].0 < w[1].0));
        assert_eq!(
            stroke,
            flatten("M10,20 C20,20 30,20 40,20 S60,20 70,20").unwrap()
        );
    }
    #[test]
    fn median_direction_and_geometry() {
        let map = parse_medians(
            r#"{"character":"一","medians":[[[100,388],[900,388]]]}"#.as_bytes(),
            StrokeStandard::Prc,
        )
        .unwrap();
        let p = &map[&'一'].strokes[0].points;
        assert!(p[0].0 < p[1].0);
        assert_eq!(p[0].1, 0.5);
        let map = parse_medians(
            r#"{"character":"丨","medians":[[[512,800],[512,0]]]}"#.as_bytes(),
            StrokeStandard::Taiwan,
        )
        .unwrap();
        let p = &map[&'丨'].strokes[0].points;
        assert!(p[0].1 < p[1].1);
    }
    #[test]
    fn namespace_and_stroke_paths_only() {
        let svg = r#"<s:svg xmlns:s="http://www.w3.org/2000/svg"><s:g id="kvg:StrokePaths_03042"><s:path d="M31,33c1,1 20,0 40,-3"/></s:g><s:g id="kvg:StrokeNumbers_03042"><s:path d="garbage"/></s:g></s:svg>"#;
        let glyph = parse_svg(svg).unwrap();
        assert_eq!(glyph.strokes.len(), 1);
        assert!(glyph.strokes[0].points.iter().all(|p| p.1 < 0.4));
    }
    #[test]
    fn smooth_reflection_and_duplicate_samples() {
        assert_eq!(
            flatten("M10,10 C10,20 20,30 30,30 S50,40 60,50").unwrap(),
            flatten("M10,10 C10,20 20,30 30,30 C40,30 50,40 60,50").unwrap()
        );
        let stroke = flatten("M10,10 C10,10 10,10 10,10 C10,10 20,20 30,30").unwrap();
        assert_eq!(stroke.points[0], (10.0 / 109.0, 10.0 / 109.0));
        assert_eq!(*stroke.points.last().unwrap(), (30.0 / 109.0, 30.0 / 109.0));
        assert!(stroke.points.windows(2).all(|p| p[0] != p[1]));
    }

    #[test]
    fn canonical_zip_entries_only() {
        use std::io::Write;
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><g id="kvg:StrokePaths_04e00"><path d="M10,50 C30,50 60,50 90,50"/></g></svg>"#;
        for (file, contents) in [
            ("kanji/04e00.svg", svg),
            ("kanji/04e00-Kaisho.svg", "invalid"),
            ("other/04e01.svg", "invalid"),
        ] {
            zip.start_file(
                format!("kanjivg-{KANJIVG_COMMIT}/{file}"),
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
            zip.write_all(contents.as_bytes()).unwrap();
        }
        let map = parse_kanjivg(&zip.finish().unwrap().into_inner()).unwrap();
        assert_eq!(map.len(), 1);
        assert_eq!(map[&'一'].standard, StrokeStandard::Japan);
    }

    /// A KanjiVG archive with the one character `一`.
    fn kanjivg_zip() -> Vec<u8> {
        use std::io::Write;
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><g id="kvg:StrokePaths_04e00"><path d="M10,50 C30,50 60,50 90,50"/></g></svg>"#;
        zip.start_file(
            format!("kanjivg-{KANJIVG_COMMIT}/kanji/04e00.svg"),
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
        zip.write_all(svg.as_bytes()).unwrap();
        zip.finish().unwrap().into_inner()
    }

    #[tokio::test]
    async fn national_sources_and_fallback_tags() {
        let prc = r#"{"character":"二","medians":[[[100,388],[900,388]]]}
{"character":"三","medians":[[[100,388],[900,388]]]}"#;
        let taiwan = r#"{"character":"二","medians":[[[110,388],[910,388]]]}"#;
        let source = |url: &str| {
            std::future::ready(Ok(match url {
                MMAH_URL => prc.as_bytes().to_vec(),
                ANIMCJK_URL => taiwan.as_bytes().to_vec(),
                _ => kanjivg_zip(),
            }))
        };
        let mut urls = Vec::new();
        let map = load(Language::ChineseTraditional, |url| {
            urls.push(url);
            source(url)
        })
        .await
        .unwrap();
        assert_eq!(urls, [ANIMCJK_URL, MMAH_URL, KANJIVG_URL]);
        assert_eq!(map.glyphs("二")[0].standard, StrokeStandard::Taiwan);
        assert_eq!(map.glyphs("三")[0].standard, StrokeStandard::Prc);
        assert_eq!(map.glyphs("一")[0].standard, StrokeStandard::Japan);
        assert_eq!(map.segment("一二 x"), ["一", "二", " ", "x"]);
        assert!(map.glyphs("一二").is_empty());
        assert!(map.glyphs(" ").is_empty());
        let map = load(Language::ChineseSimplified, source).await.unwrap();
        assert_eq!(map.glyphs("二")[0].standard, StrokeStandard::Prc);
        assert_eq!(map.glyphs("一")[0].standard, StrokeStandard::Japan);
        let map = load(Language::Japanese, source).await.unwrap();
        assert_eq!(map.glyphs("一")[0].standard, StrokeStandard::Japan);
        assert_eq!(map.glyphs("二")[0].standard, StrokeStandard::Prc);
        // Text in any script contains Latin letters and digits, and
        // compatibility forms are written like their base character.
        assert_eq!(map.glyphs("A")[0].standard, StrokeStandard::Latin);
        assert_eq!(map.glyphs("Ａ"), map.glyphs("A"));
        assert_eq!(map.glyphs("３"), map.glyphs("3"));
        assert!(map.glyphs("ｱ").is_empty());
        let map = load(Language::English, |_| async {
            panic!("authored packs are embedded, not fetched")
        })
        .await
        .unwrap();
        assert_eq!(map.glyphs("a")[0].standard, StrokeStandard::Latin);
        let map = load(Language::Hindi, |_| async {
            panic!("authored packs are embedded, not fetched")
        })
        .await
        .unwrap();
        assert_eq!(map.segment("नमस्ते"), ["न", "म", "स्ते"]);
        assert_eq!(map.glyphs("स्ते")[0].standard, StrokeStandard::Devanagari);
        assert_eq!(map.glyphs("7")[0].standard, StrokeStandard::Latin);
        let map = load(Language::Thai, |_| async {
            panic!("authored packs are embedded, not fetched")
        })
        .await
        .unwrap();
        assert_eq!(map.segment("ครับ"), ["ค", "รั", "บ"]);
        assert_eq!(map.glyphs("รั").len(), 1);
        assert_eq!(map.glyphs("รั")[0].standard, StrokeStandard::Thai);
    }

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

    #[tokio::test]
    async fn accepted_forms() {
        let embedded = |language| {
            load(language, |_| async {
                panic!("authored packs are embedded, not fetched")
            })
        };
        let latin = embedded(Language::French).await.unwrap();
        let cyrillic = embedded(Language::Russian).await.unwrap();
        let hindi = embedded(Language::Hindi).await.unwrap();
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

    #[test]
    fn rejects_invalid_geometry() {
        for medians in ["[]", "[[[1,1]]]", "[[[1,1],[2000,1]]]"] {
            assert!(
                parse_medians(
                    format!(r#"{{"character":"一","medians":{medians}}}"#).as_bytes(),
                    StrokeStandard::Prc
                )
                .is_err()
            );
        }
    }
}
