//! Run from the repo root: cargo run -p stroke-order --example diagnostics --
//! <cache-dir> <samples-out-dir>
use anyhow::{Context, Result, ensure};
use language_utils::Language;
use std::{
    collections::BTreeMap,
    fmt::Write,
    io::{Cursor, Read},
    path::{Path, PathBuf},
};
use stroke_order::{ANIMCJK_URL, Glyphs, KANJIVG_URL, MMAH_URL, SCRIBING_URL};
use stroke_order::{StrokeGlyph, StrokeStandard};

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    ensure!(
        args.len() == 2,
        "usage: diagnostics <cache-dir> <samples-out-dir>"
    );
    let cache = PathBuf::from(&args[0]);
    let samples_out = PathBuf::from(&args[1]);
    std::fs::create_dir_all(&cache)?;
    let fetch = |name: String, url: &'static str| {
        let path = cache.join(name);
        async move {
            if path.exists() {
                return Ok::<_, anyhow::Error>(std::fs::read(path)?);
            }
            let bytes = reqwest::get(url)
                .await?
                .error_for_status()?
                .bytes()
                .await?
                .to_vec();
            std::fs::write(path, &bytes)?;
            Ok(bytes)
        }
    };
    let mut maps = Vec::new();
    for (name, url, standard) in [
        ("kanjivg", KANJIVG_URL, StrokeStandard::Japan),
        ("mmah", MMAH_URL, StrokeStandard::Prc),
        ("animcjk", ANIMCJK_URL, StrokeStandard::Taiwan),
        ("scribing", SCRIBING_URL, StrokeStandard::Korea),
    ] {
        let filename = format!(
            "{name}-{}.bin",
            match standard {
                StrokeStandard::Japan => stroke_order::KANJIVG_COMMIT,
                StrokeStandard::Prc => stroke_order::MMAH_COMMIT,
                StrokeStandard::Taiwan => stroke_order::ANIMCJK_COMMIT,
                StrokeStandard::Korea => stroke_order::SCRIBING_COMMIT,
                StrokeStandard::Devanagari
                | StrokeStandard::Thai
                | StrokeStandard::Latin
                | StrokeStandard::Cyrillic => unreachable!(),
            }
        );
        let bytes = fetch(filename, url).await?;
        let map = if standard == StrokeStandard::Japan {
            stroke_order::parse_kanjivg(&bytes)?
        } else if standard == StrokeStandard::Korea {
            stroke_order::parse_scribing(&bytes)?
        } else {
            stroke_order::parse_medians(&bytes, standard)?
        };
        println!(
            "{name}: {} glyphs, {} strokes; all points finite/in box, all strokes >=2 points",
            map.len(),
            map.values().map(|g| g.strokes.len()).sum::<usize>()
        );
        if standard == StrokeStandard::Korea {
            ensure!(map.len() == 40, "Korean jamo");
            // Syllables are composed on demand: check a spread of them.
            let pack =
                stroke_order::load(Language::Korean, |_| std::future::ready(Ok(bytes.clone())))
                    .await?;
            let sampled: Vec<char> = ('가'..='힣').step_by(97).chain(['힣']).collect();
            for &c in &sampled {
                let glyph = pack
                    .glyphs(&c.to_string())
                    .into_iter()
                    .next()
                    .context("syllable")?;
                stroke_order::validate(&glyph).with_context(|| format!("{c}"))?;
            }
            println!(
                "scribing: 40 jamo; {} sampled syllables compose and validate; Unihan comparison skipped",
                sampled.len()
            );
            samples(&samples_out, name, |c| {
                pack.glyphs(&c.to_string()).into_iter().next()
            })?;
        } else {
            geometry(name, &map)?;
            samples(&samples_out, name, |c| map.get(&c).cloned())?;
            maps.push((name, map));
        }
    }
    let unihan = fetch(
        "Unihan-18.0.0.zip".into(),
        "https://www.unicode.org/Public/18.0.0/ucd/Unihan.zip",
    )
    .await?;
    let mut zip = zip::ZipArchive::new(Cursor::new(unihan))?;
    let mut text = String::new();
    zip.by_name("Unihan_IRGSources.txt")?
        .read_to_string(&mut text)?;
    let counts: BTreeMap<_, Vec<usize>> = text
        .lines()
        .filter_map(|line| {
            let mut fields = line.split('\t');
            let c = fields.next()?.strip_prefix("U+")?;
            if fields.next()? != "kTotalStrokes" {
                return None;
            }
            Some((
                char::from_u32(u32::from_str_radix(c, 16).unwrap()).unwrap(),
                fields
                    .next()?
                    .split_whitespace()
                    .map(|s| s.parse().unwrap())
                    .collect(),
            ))
        })
        .collect();
    for (name, map) in &maps {
        let mut matched = 0;
        let mut checked = 0;
        let mut mismatches = Vec::new();
        for (&c, glyph) in map {
            if let Some(counts) = counts.get(&c) {
                checked += 1;
                if counts.contains(&glyph.strokes.len()) {
                    matched += 1;
                } else {
                    mismatches.push(c);
                }
            }
        }
        mismatches.sort();
        println!(
            "{name} Unihan18 kTotalStrokes: {matched}/{checked} ({:.2}%) match any listed count; {} differences; {} without Unihan count (diagnostic, not a gate). First differences: {}",
            100.0 * matched as f64 / checked as f64,
            checked - matched,
            map.len() - checked,
            mismatches.iter().take(30).collect::<String>()
        );
    }
    Ok(())
}

fn geometry(name: &str, map: &Glyphs) -> Result<()> {
    for c in ['一', '丨'] {
        let glyph = map.get(&c).with_context(|| format!("{name} missing {c}"))?;
        ensure!(glyph.strokes.len() == 1, "{name} {c}: expected one stroke");
        let points = &glyph.strokes[0].points;
        let (start, end) = (points[0], *points.last().unwrap());
        if c == '一' {
            ensure!(
                start.0 < end.0 && points.iter().all(|p| (p.1 - 0.5).abs() < 0.2),
                "{name} 一 geometry"
            );
        } else {
            ensure!(start.1 < end.1, "{name} 丨 direction");
        }
        println!("{name} {c}: {start:?} -> {end:?}");
    }
    if name == "kanjivg" {
        let glyph = &map[&'あ'];
        ensure!(
            glyph.strokes[0].points.iter().all(|p| p.1 < 0.4),
            "あ first stroke too low"
        );
        println!(
            "kanjivg kana: {} hiragana, {} katakana; あ first stroke y<0.4",
            map.keys()
                .filter(|&&c| ('ぁ'..='ゖ').contains(&c) || ('゛'..='ゞ').contains(&c))
                .count(),
            map.keys().filter(|&&c| ('ァ'..='ヾ').contains(&c)).count()
        );
    }
    Ok(())
}

fn samples(out: &Path, name: &str, glyph: impl Fn(char) -> Option<StrokeGlyph>) -> Result<()> {
    let dir = out.join(match name {
        "kanjivg" => "Japan",
        "mmah" => "Prc",
        "scribing" => "Korea",
        _ => "Taiwan",
    });
    std::fs::create_dir_all(&dir)?;
    let chars = match name {
        "kanjivg" => "一丨あア永語",
        "mmah" => "一丨永你国汉",
        "scribing" => "ㄱㅏ한글뭐왜꽃닭값앉훑쒜",
        _ => "一丨永你國漢",
    };
    for c in chars.chars() {
        let glyph = glyph(c).context("missing sample glyph")?;
        let svg = render_svg(c, &glyph)?;
        std::fs::write(dir.join(format!("{c}.svg")), svg)?;
    }
    Ok(())
}

fn render_svg(c: char, glyph: &StrokeGlyph) -> Result<String> {
    let mut svg = format!(
        r##"<!-- Converted 2026-09-23: directed centerlines normalized and numbered for Yap.Town. See libraries/stroke-order/licenses/NOTICE.md for source attribution and licenses. -->
<svg xmlns="http://www.w3.org/2000/svg" viewBox="-12 -12 280 300"><title>{c} — {:?}, numbered in pen order</title><rect x="0" y="0" width="256" height="256" fill="white" stroke="#bbb"/><g fill="none" stroke="#111" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">"##,
        glyph.standard
    );
    for stroke in &glyph.strokes {
        let points = stroke
            .points
            .iter()
            .map(|(x, y)| format!("{},{}", x * 256.0, y * 256.0))
            .collect::<Vec<_>>()
            .join(" ");
        write!(svg, r#"<polyline points="{points}"/>"#)?;
    }
    svg.push_str("</g><g fill=\"#b02020\" font-family=\"sans-serif\" font-size=\"12\">");
    for (i, stroke) in glyph.strokes.iter().enumerate() {
        let (x, y) = stroke.points[0];
        write!(
            svg,
            r#"<text x="{}" y="{}">{}</text>"#,
            x * 256.0 - 8.0,
            y * 256.0 - 3.0,
            i + 1
        )?;
    }
    write!(
        svg,
        "</g><text x=\"0\" y=\"280\" font-size=\"14\">{c} — {:?}</text></svg>",
        glyph.standard
    )?;
    Ok(svg)
}
