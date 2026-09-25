//! Build-time conversion of pinned, attributed sources into a pack’s StrokeTable.
//! Licenses live in ../stroke-order/licenses.
mod korean;
pub use korean::parse_scribing;

use anyhow::{Context, Result, bail, ensure};
use language_utils::{Language, Stroke, StrokeGlyph, StrokeStandard, StrokeTable};
use rustc_hash::FxHashMap;
use std::{
    future::Future,
    io::{Cursor, Read},
};
use stroke_order::{fold, segment, validate};
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

/// Sources in priority order; each glyph keeps its source's national standard.
fn chain(sources: impl IntoIterator<Item = Glyphs>) -> Glyphs {
    let mut glyphs = Glyphs::default();
    for source in sources {
        for (c, glyph) in source {
            glyphs.entry(c).or_insert(glyph);
        }
    }
    glyphs
}

/// The stroke table for a course: every writable unit of `texts` that the
/// language's downloaded sources draw. Empty for scripts drawn in code.
/// The caller owns networking and caching; the source URLs are immutable
/// and make good cache keys.
pub async fn table<'a, F, Fut>(
    language: Language,
    texts: impl IntoIterator<Item = &'a str>,
    mut fetch: F,
) -> Result<StrokeTable>
where
    F: FnMut(&'static str) -> Fut,
    Fut: Future<Output = Result<Vec<u8>>>,
{
    let source = match language {
        Language::Japanese => {
            let japan = parse_kanjivg(&fetch(KANJIVG_URL).await?)?;
            let prc = parse_medians(&fetch(MMAH_URL).await?, StrokeStandard::Prc)?;
            chain([japan, prc])
        }
        Language::ChineseSimplified => {
            let prc = parse_medians(&fetch(MMAH_URL).await?, StrokeStandard::Prc)?;
            let taiwan = parse_medians(&fetch(ANIMCJK_URL).await?, StrokeStandard::Taiwan)?;
            let japan = parse_kanjivg(&fetch(KANJIVG_URL).await?)?;
            chain([prc, taiwan, japan])
        }
        Language::ChineseTraditional => {
            let taiwan = parse_medians(&fetch(ANIMCJK_URL).await?, StrokeStandard::Taiwan)?;
            let prc = parse_medians(&fetch(MMAH_URL).await?, StrokeStandard::Prc)?;
            let japan = parse_kanjivg(&fetch(KANJIVG_URL).await?)?;
            chain([taiwan, prc, japan])
        }
        Language::Korean => parse_scribing(&fetch(SCRIBING_URL).await?)?,
        _ => return Ok(StrokeTable::default()),
    };
    let mut table = StrokeTable::default();
    for text in texts {
        for unit in segment(language.writing_system(), text) {
            let mut chars = unit.chars();
            let Some(c) = chars.next().filter(|_| chars.next().is_none()) else {
                continue;
            };
            for c in std::iter::once(c).chain(fold(c).filter(|&base| base != c)) {
                table.entry(c.to_string()).or_insert_with(|| {
                    let glyph = if language == Language::Korean {
                        korean::glyph(c, &source)
                    } else {
                        source.get(&c).cloned()
                    };
                    glyph.into_iter().collect()
                });
            }
        }
    }
    table.retain(|_, forms| !forms.is_empty());
    Ok(table)
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

    /// A minimal KanjiVG archive with `一` and `ア`.
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
        zip.start_file(
            format!("kanjivg-{KANJIVG_COMMIT}/kanji/030a2.svg"),
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
        let strokes = table(Language::ChineseTraditional, ["一二三"], |url| {
            urls.push(url);
            source(url)
        })
        .await
        .unwrap();
        assert_eq!(urls, [ANIMCJK_URL, MMAH_URL, KANJIVG_URL]);
        let map =
            stroke_order::StrokePack::new(Language::ChineseTraditional.writing_system(), strokes);
        assert_eq!(map.glyphs("二")[0].standard, StrokeStandard::Taiwan);
        assert_eq!(map.glyphs("三")[0].standard, StrokeStandard::Prc);
        assert_eq!(map.glyphs("一")[0].standard, StrokeStandard::Japan);
        assert_eq!(map.segment("一二 x"), ["一", "二", " ", "x"]);
        assert!(map.glyphs("一二").is_empty());
        assert!(map.glyphs(" ").is_empty());
        let strokes = table(Language::ChineseSimplified, ["一二"], source)
            .await
            .unwrap();
        assert!(
            !strokes.contains_key("三"),
            "units absent from texts are not stored"
        );
        let map =
            stroke_order::StrokePack::new(Language::ChineseSimplified.writing_system(), strokes);
        assert_eq!(map.glyphs("二")[0].standard, StrokeStandard::Prc);
        assert_eq!(map.glyphs("一")[0].standard, StrokeStandard::Japan);
        let strokes = table(Language::Japanese, ["一二"], source).await.unwrap();
        let map = stroke_order::StrokePack::new(Language::Japanese.writing_system(), strokes);
        assert_eq!(map.glyphs("一")[0].standard, StrokeStandard::Japan);
        assert_eq!(map.glyphs("二")[0].standard, StrokeStandard::Prc);
        // Text in any script contains Latin letters and digits, and
        // compatibility forms are written like their base character.
        assert_eq!(map.glyphs("A")[0].standard, StrokeStandard::Latin);
        assert_eq!(map.glyphs("Ａ"), map.glyphs("A"));
        assert_eq!(map.glyphs("３"), map.glyphs("3"));
        assert!(map.glyphs("ｱ").is_empty());
        let strokes = table(Language::Japanese, ["ｱｱ"], source).await.unwrap();
        assert_eq!(strokes.len(), 1);
        assert_eq!(strokes["ア"][0].standard, StrokeStandard::Japan);
        assert!(!strokes.contains_key("一"));
        let map = stroke_order::StrokePack::new(Language::Japanese.writing_system(), strokes);
        assert_eq!(map.glyphs("ｱ"), map.glyphs("ア"));
        let strokes = table(Language::English, ["abc"], |_| async {
            panic!("authored packs are embedded, not fetched")
        })
        .await
        .unwrap();
        assert!(strokes.is_empty());
        let map = stroke_order::StrokePack::new(Language::English.writing_system(), strokes);
        assert_eq!(map.glyphs("a")[0].standard, StrokeStandard::Latin);
        let strokes = table(Language::Hindi, ["नमस्ते"], |_| async {
            panic!("authored packs are embedded, not fetched")
        })
        .await
        .unwrap();
        assert!(strokes.is_empty());
        let map = stroke_order::StrokePack::new(Language::Hindi.writing_system(), strokes);
        assert_eq!(map.segment("नमस्ते"), ["न", "म", "स्ते"]);
        assert_eq!(map.glyphs("स्ते")[0].standard, StrokeStandard::Devanagari);
        assert_eq!(map.glyphs("7")[0].standard, StrokeStandard::Latin);
        let strokes = table(Language::Thai, ["ครับ"], |_| async {
            panic!("authored packs are embedded, not fetched")
        })
        .await
        .unwrap();
        assert!(strokes.is_empty());
        let map = stroke_order::StrokePack::new(Language::Thai.writing_system(), strokes);
        assert_eq!(map.segment("ครับ"), ["ค", "รั", "บ"]);
        assert_eq!(map.glyphs("รั").len(), 1);
        assert_eq!(map.glyphs("รั")[0].standard, StrokeStandard::Thai);
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
