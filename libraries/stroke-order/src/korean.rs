use crate::{Glyphs, Stroke, StrokeGlyph, StrokeStandard, validate};
use anyhow::{Context, Result, ensure};
use serde::Deserialize;

const INITIALS: &str = "ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ";
const MEDIALS: &str = "ㅏㅐㅑㅒㅓㅔㅕㅖㅗㅘㅙㅚㅛㅜㅝㅞㅟㅠㅡㅢㅣ";
const FINALS: [&str; 28] = [
    "", "ㄱ", "ㄲ", "ㄱㅅ", "ㄴ", "ㄴㅈ", "ㄴㅎ", "ㄷ", "ㄹ", "ㄹㄱ", "ㄹㅁ", "ㄹㅂ", "ㄹㅅ",
    "ㄹㅌ", "ㄹㅍ", "ㄹㅎ", "ㅁ", "ㅂ", "ㅂㅅ", "ㅅ", "ㅆ", "ㅇ", "ㅈ", "ㅊ", "ㅋ", "ㅌ", "ㅍ",
    "ㅎ",
];

#[derive(Deserialize)]
struct Pack {
    units: std::collections::BTreeMap<char, Unit>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Unit {
    coordinates: Coordinates,
    motor_strokes: Vec<MotorStroke>,
    plans: Vec<Plan>,
    default_plan_id: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Coordinates {
    em: u32,
    y_axis: String,
}
#[derive(Deserialize)]
struct MotorStroke {
    id: String,
    points: Vec<(f32, f32)>,
}
#[derive(Deserialize)]
struct Plan {
    id: String,
    steps: Vec<Step>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Step {
    stroke_id: String,
}

/// Parses the Scribing pack: `units` keyed by character, each with 1000-em
/// y-down `motorStrokes` and a `defaultPlanId` giving the stroke order.
fn parse_pack(bytes: &[u8]) -> Result<Glyphs> {
    let pack: Pack = serde_json::from_slice(bytes)?;
    let mut glyphs = Glyphs::default();
    for (c, unit) in pack.units {
        ensure!(
            unit.coordinates.em == 1000 && unit.coordinates.y_axis == "down",
            "unexpected coordinates for {c}"
        );
        let mut strokes = std::collections::BTreeMap::new();
        for stroke in unit.motor_strokes {
            ensure!(
                strokes.insert(stroke.id, stroke.points).is_none(),
                "duplicate stroke for {c}"
            );
        }
        let mut plans = unit.plans.iter().filter(|p| p.id == unit.default_plan_id);
        let plan = plans.next().context("missing default plan")?;
        ensure!(plans.next().is_none(), "duplicate default plan");
        let glyph = StrokeGlyph {
            standard: StrokeStandard::Korea,
            strokes: plan
                .steps
                .iter()
                .map(|step| {
                    let points = strokes
                        .remove(&step.stroke_id)
                        .context("missing or repeated stroke")?;
                    Ok(Stroke {
                        points: points
                            .into_iter()
                            .map(|(x, y)| (x / 1000.0, y / 1000.0))
                            .collect(),
                    })
                })
                .collect::<Result<_>>()?,
        };
        ensure!(strokes.is_empty(), "unplanned strokes for {c}");
        validate(&glyph).with_context(|| format!("Scribing {c}"))?;
        glyphs.insert(c, glyph);
    }
    Ok(glyphs)
}

/// The 40 modern jamo of the Scribing pack; syllables are composed from them
/// by [`glyph`].
pub fn parse_scribing(bytes: &[u8]) -> Result<Glyphs> {
    let glyphs = parse_pack(bytes)?;
    ensure!(
        glyphs.len() == 40
            && INITIALS
                .chars()
                .chain(MEDIALS.chars())
                .all(|c| glyphs.contains_key(&c)),
        "expected 40 modern jamo"
    );
    Ok(glyphs)
}

/// A jamo as drawn, or a precomposed Hangul syllable fitted from its jamo.
pub fn glyph(c: char, jamo: &Glyphs) -> Option<StrokeGlyph> {
    match c {
        '가'..='힣' => Some(compose(c as u32 - 0xAC00, jamo)),
        _ => jamo.get(&c).cloned(),
    }
}

// (left, right, top, bottom). Mixed vowels wrap the initial: keep it above
// even the high ㅜ bar in ㅝ/ㅞ/ㅟ. Finals compress the top's 0.10–0.90 to 0.08–0.60.
type Region = (f32, f32, f32, f32);
const REGIONS: [(Region, Region); 3] = [
    ((0.10, 0.50, 0.15, 0.85), (0.56, 0.90, 0.10, 0.90)),
    ((0.20, 0.80, 0.10, 0.48), (0.10, 0.90, 0.55, 0.90)),
    ((0.12, 0.48, 0.10, 0.32), (0.10, 0.90, 0.10, 0.90)),
];

fn fit(glyph: &StrokeGlyph, (left, right, top, bottom): Region) -> Vec<Stroke> {
    let (mut xmin, mut xmax, mut ymin, mut ymax) = (1.0_f32, 0.0_f32, 1.0_f32, 0.0_f32);
    for &(x, y) in glyph.strokes.iter().flat_map(|s| &s.points) {
        xmin = xmin.min(x);
        xmax = xmax.max(x);
        ymin = ymin.min(y);
        ymax = ymax.max(y);
    }
    // ㅡ and ㅣ have a zero-sized axis: center it rather than dividing by zero.
    let map = |v, min, max, start, end| {
        if min == max {
            (start + end) / 2.0
        } else {
            start + (v - min) / (max - min) * (end - start)
        }
    };
    glyph
        .strokes
        .iter()
        .map(|s| Stroke {
            points: s
                .points
                .iter()
                .map(|&(x, y)| {
                    (
                        map(x, xmin, xmax, left, right),
                        map(y, ymin, ymax, top, bottom),
                    )
                })
                .collect(),
        })
        .collect()
}

fn compose(index: u32, jamo: &Glyphs) -> StrokeGlyph {
    let initial = INITIALS.chars().nth((index / 588) as usize).unwrap();
    let medial = MEDIALS.chars().nth(((index % 588) / 28) as usize).unwrap();
    let finals = FINALS[(index % 28) as usize];
    let class = if "ㅏㅐㅑㅒㅓㅔㅕㅖㅣ".contains(medial) {
        0
    } else if "ㅗㅛㅜㅠㅡ".contains(medial) {
        1
    } else {
        2
    };
    let (mut initial_region, mut medial_region) = REGIONS[class];
    if !finals.is_empty() {
        for r in [&mut initial_region, &mut medial_region] {
            r.2 = 0.08 + (r.2 - 0.10) * 0.65;
            r.3 = 0.08 + (r.3 - 0.10) * 0.65;
        }
    }
    let mut strokes = fit(&jamo[&initial], initial_region);
    strokes.extend(fit(&jamo[&medial], medial_region));
    for (i, c) in finals.chars().enumerate() {
        let region = if finals.chars().count() == 1 {
            (0.15, 0.85, 0.66, 0.92)
        } else if i == 0 {
            (0.10, 0.48, 0.66, 0.92)
        } else {
            (0.52, 0.90, 0.66, 0.92)
        };
        strokes.extend(fit(&jamo[&c], region));
    }
    StrokeGlyph {
        standard: StrokeStandard::Korea,
        strokes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const JAMO: &str = r#"{"units":{
        "ㅏ":{"coordinates":{"em":1000,"yAxis":"down"},"motorStrokes":[{"id":"b","points":[[400,500],[700,500]]},{"id":"a","points":[[400,200],[400,800]]}],"plans":[{"id":"unused","steps":[]},{"id":"default","steps":[{"strokeId":"a"},{"strokeId":"b"}]}],"defaultPlanId":"default"},
        "ㅎ":{"coordinates":{"em":1000,"yAxis":"down"},"motorStrokes":[{"id":"a","points":[[400,200],[600,200]]},{"id":"b","points":[[200,400],[800,400]]},{"id":"c","points":[[400,600],[600,800]]}],"plans":[{"id":"default","steps":[{"strokeId":"a"},{"strokeId":"b"},{"strokeId":"c"}]}],"defaultPlanId":"default"},
        "ㄴ":{"coordinates":{"em":1000,"yAxis":"down"},"motorStrokes":[{"id":"a","points":[[200,200],[200,800],[800,800]]}],"plans":[{"id":"default","steps":[{"strokeId":"a"}]}],"defaultPlanId":"default"}
    }}"#;

    #[test]
    fn default_plan_direction_and_composition() {
        let mut jamo = parse_pack(JAMO.as_bytes()).unwrap();
        assert_eq!(jamo[&'ㅏ'].strokes[0].points, [(0.4, 0.2), (0.4, 0.8)]);
        assert_eq!(jamo[&'ㅏ'].strokes[1].points, [(0.4, 0.5), (0.7, 0.5)]);
        let han = compose('한' as u32 - 0xAC00, &jamo);
        assert_eq!(
            han.strokes.len(),
            "ㅎㅏㄴ"
                .chars()
                .map(|c| jamo[&c].strokes.len())
                .sum::<usize>()
        );
        assert_eq!(
            han.strokes[0],
            fit(
                &jamo[&'ㅎ'],
                (
                    0.10,
                    0.50,
                    0.08 + (0.15 - 0.10) * 0.65,
                    0.08 + (0.85 - 0.10) * 0.65
                )
            )[0]
        );
        validate(&han).unwrap();
        // Reuse synthetic geometry: this checks decomposition and directed ordering,
        // not the upstream shapes (the diagnostics validate the actual pack).
        for (c, source) in [('ㄷ', 'ㅎ'), ('ㄹ', 'ㅏ'), ('ㄱ', 'ㄴ')] {
            jamo.insert(c, jamo[&source].clone());
        }
        let dak = glyph('닭', &jamo).unwrap();
        assert_eq!(glyph('ㄴ', &jamo), Some(jamo[&'ㄴ'].clone()));
        assert_eq!(glyph('a', &jamo), None);
        let expected: Vec<_> = [
            ('ㄷ', (0.10, 0.50, 0.1125, 0.5675)),
            ('ㅏ', (0.56, 0.90, 0.08, 0.60)),
            ('ㄹ', (0.10, 0.48, 0.66, 0.92)),
            ('ㄱ', (0.52, 0.90, 0.66, 0.92)),
        ]
        .into_iter()
        .flat_map(|(c, r)| fit(&jamo[&c], r))
        .collect();
        assert_eq!(dak.strokes.len(), expected.len());
        for (actual, expected) in dak.strokes.iter().zip(expected) {
            for (a, b) in actual.points.iter().zip(expected.points) {
                assert!((a.0 - b.0).abs() < 1e-6 && (a.1 - b.1).abs() < 1e-6);
            }
        }
        validate(&dak).unwrap();
    }

    #[test]
    fn line_jamo_fit_and_incomplete_pack() {
        assert!(parse_scribing(JAMO.as_bytes()).is_err());
        for points in [vec![(0.2, 0.5), (0.8, 0.5)], vec![(0.5, 0.2), (0.5, 0.8)]] {
            let glyph = StrokeGlyph {
                standard: StrokeStandard::Korea,
                strokes: vec![Stroke { points }],
            };
            let fitted = StrokeGlyph {
                standard: StrokeStandard::Korea,
                strokes: fit(&glyph, (0.1, 0.9, 0.2, 0.8)),
            };
            validate(&fitted).unwrap();
            assert!(fitted.strokes[0].points[0].0 == 0.5 || fitted.strokes[0].points[0].1 == 0.5);
        }
    }
}
