//! Renders a text's writable units as an SVG that draws each stroke in
//! order, so a pack can be eyeballed without any host app.
//!
//! cargo run -p stroke-order --example svg -- <cache-dir> <out.svg> <lang code> <text>...
//!
//! Each argument after the language code is one line. Strokes are numbered at
//! their start and animate in writing order; opening the SVG in a browser
//! plays it, a rasteriser shows the static numbered view. A unit with
//! accepted alternative forms shows them in extra rows under its line.
use anyhow::{Context, Result, ensure};
use language_utils::Language;
use std::{fmt::Write, path::PathBuf};

const COLORS: [&str; 8] = [
    "#d62728", "#1f77b4", "#2ca02c", "#e6a100", "#9467bd", "#17becf", "#8c564b", "#e377c2",
];

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    ensure!(
        args.len() >= 4,
        "usage: svg <cache-dir> <out.svg> <lang> <text>..."
    );
    let cache = PathBuf::from(&args[0]);
    std::fs::create_dir_all(&cache)?;
    let language = Language::from_code(&args[2]).context("unknown language code")?;
    let pack = stroke_order::load(language, |url| {
        let path = cache.join(url.rsplit('/').next().unwrap());
        async move {
            if path.exists() {
                return Ok(std::fs::read(path)?);
            }
            let bytes = reqwest::get(url).await?.error_for_status()?.bytes().await?;
            std::fs::write(&path, &bytes)?;
            Ok(bytes.to_vec())
        }
    })
    .await?;

    let lines = &args[3..];
    let cell = 100.0;
    let width = lines
        .iter()
        .map(|l| pack.segment(l).len())
        .max()
        .unwrap_or(0) as f64
        * cell;
    let mut svg = String::new();
    let mut style = String::from(
        "<style>path{fill:none;stroke-linecap:round;stroke-linejoin:round;stroke-width:5}",
    );
    let mut t = 0.0;
    let mut row = 0;
    for line in lines {
        let units: Vec<_> = pack
            .segment(line)
            .into_iter()
            .map(|unit| pack.glyphs(unit))
            .collect();
        let forms = units.iter().map(Vec::len).max().unwrap_or(0).max(1);
        let cells = units.iter().enumerate().flat_map(|(col, glyphs)| {
            glyphs
                .iter()
                .enumerate()
                .map(move |(form, glyph)| (col, row + form, glyph))
        });
        for (col, row, glyph) in cells {
            let (ox, oy) = (col as f64 * cell, row as f64 * cell);
            write!(
                svg,
                r##"<rect x="{ox}" y="{oy}" width="{cell}" height="{cell}" fill="none" stroke="#ddd"/>"##
            )?;
            for (i, stroke) in glyph.strokes.iter().enumerate() {
                let color = COLORS[i % COLORS.len()];
                let d: String = stroke
                    .points
                    .iter()
                    .enumerate()
                    .map(|(k, (x, y))| {
                        let cmd = if k == 0 { 'M' } else { 'L' };
                        format!(
                            "{cmd}{:.1},{:.1}",
                            ox + f64::from(*x) * cell,
                            oy + f64::from(*y) * cell
                        )
                    })
                    .collect();
                let id = format!("s{row}_{col}_{i}");
                let dur = 0.5;
                write!(
                    svg,
                    r##"<path id="{id}" d="{d}" stroke="{color}" pathLength="1"/>"##
                )?;
                write!(
                    style,
                    "#{id}{{stroke-dasharray:1;animation:draw {dur}s {t:.2}s linear both}}"
                )?;
                t += dur;
                let (x, y) = stroke.points[0];
                write!(
                    svg,
                    r##"<circle cx="{:.1}" cy="{:.1}" r="6" fill="{color}"/><text x="{:.1}" y="{:.1}" font-size="8" fill="#fff" text-anchor="middle" font-family="sans-serif">{}</text>"##,
                    ox + f64::from(x) * cell,
                    oy + f64::from(y) * cell,
                    ox + f64::from(x) * cell,
                    oy + f64::from(y) * cell + 3.0,
                    i + 1
                )?;
            }
        }
        row += forms;
    }
    style.push_str("@keyframes draw{from{stroke-dashoffset:1}to{stroke-dashoffset:0}}</style>");
    let height = row as f64 * cell;
    std::fs::write(
        &args[1],
        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}">{style}<rect width="100%" height="100%" fill="#fff"/>{svg}</svg>"##
        ),
    )?;
    Ok(())
}
