//! Stroke order for the 66 Russian print letters.
//!
//! Each letter is one function returning its strokes in writing order; each
//! stroke is a list of (x, y) points in pen direction, in a 1000-unit box with
//! y pointing down. To fix a letter, edit its function.
//!
//! Formation follows Russian primary-school print (печатные буквы), applied
//! consistently:
//! - stems first, top to bottom, left to right;
//! - then bars and connectors, left to right;
//! - strokes follow handwriting, not typography (single-storey а);
//! - bowls leave the stem and turn clockwise back to it (Б В Р Ь Ъ Ы);
//! - ovals start at the top and run anticlockwise (О Ю, the bowl of б);
//! - tails (Д Ц Щ), dots (Ё) and breves (Й) last.
//!
//! Lowercase letters that are small capitals in print (в г д ж з и й к л м н
//! п т х ц ч ш щ ъ ы ь э ю я) reuse the capital's function at x-height.
use super::{Pt, Strokes, collect, glyph};
use crate::{Glyphs, StrokeStandard};

// Writing frame (y down).
/// Ascender line: б flag, ф stem, lowercase dots and breve.
const ASC: f64 = 150.0;
/// Cap height.
const CAP: f64 = 150.0;
/// x-height.
const XH: f64 = 450.0;
/// Baseline.
const BASE: f64 = 850.0;
/// Descender: Д Ц Щ tails, р у ф.
const DESC: f64 = 980.0;
/// Centre line of the dots and breve over capitals.
const MARK: f64 = 70.0;
/// Every letter is centred horizontally.
const CX: f64 = 500.0;

pub fn glyphs() -> Glyphs {
    collect(
        letters()
            .into_iter()
            .map(|(c, strokes)| (c, glyph(StrokeStandard::Cyrillic, strokes, |p| p))),
    )
}

fn letters() -> Vec<(char, Strokes)> {
    vec![
        ('А', cap(a_cap, 540.0)),
        ('Б', cap(be, 420.0)),
        ('В', cap(ve, 420.0)),
        ('Г', cap(ghe, 340.0)),
        ('Д', de(CAP, BASE, 540.0, DESC)),
        ('Е', cap(ie, 380.0)),
        ('Ё', [cap(ie, 380.0), dots(MARK)].concat()),
        ('Ж', cap(zhe, 680.0)),
        ('З', cap(ze, 400.0)),
        ('И', cap(i, 460.0)),
        ('Й', [cap(i, 460.0), breve(MARK, 200.0)].concat()),
        ('К', cap(ka, 440.0)),
        ('Л', cap(el, 460.0)),
        ('М', cap(em, 560.0)),
        ('Н', cap(en, 460.0)),
        ('О', cap(o, 580.0)),
        ('П', cap(pe, 460.0)),
        ('Р', cap(er_cap, 400.0)),
        ('С', cap(es, 500.0)),
        ('Т', cap(te, 480.0)),
        ('У', u(CAP, BASE, 500.0, BASE)),
        ('Ф', ef(CAP + 90.0, BASE - 90.0, 620.0, CAP, BASE)),
        ('Х', cap(ha, 500.0)),
        ('Ц', tse(CAP, BASE, 480.0, DESC)),
        ('Ч', cap(che, 420.0)),
        ('Ш', cap(sha, 620.0)),
        ('Щ', shcha(CAP, BASE, 620.0, DESC)),
        ('Ъ', cap(hard_sign, 520.0)),
        ('Ы', cap(yeru, 580.0)),
        ('Ь', cap(soft_sign, 400.0)),
        ('Э', cap(e, 480.0)),
        ('Ю', cap(yu, 680.0)),
        ('Я', cap(ya, 420.0)),
        ('а', a_small(380.0)),
        ('б', be_small(400.0)),
        ('в', small(ve, 340.0)),
        ('г', small(ghe, 280.0)),
        ('д', de(XH, BASE, 440.0, DESC)),
        ('е', ie_small(400.0)),
        ('ё', [ie_small(400.0), dots(XH - 110.0)].concat()),
        ('ж', small(zhe, 560.0)),
        ('з', small(ze, 340.0)),
        ('и', small(i, 380.0)),
        ('й', [small(i, 380.0), breve(XH - 110.0, 180.0)].concat()),
        ('к', small(ka, 360.0)),
        ('л', small(el, 380.0)),
        ('м', small(em, 460.0)),
        ('н', small(en, 380.0)),
        ('о', small(o, 420.0)),
        ('п', small(pe, 380.0)),
        ('р', er_small(380.0)),
        ('с', small(es, 380.0)),
        ('т', small(te, 400.0)),
        ('у', u(XH, BASE, 420.0, DESC)),
        ('ф', ef(XH, BASE, 500.0, ASC, DESC)),
        ('х', small(ha, 400.0)),
        ('ц', tse(XH, BASE, 400.0, DESC)),
        ('ч', small(che, 360.0)),
        ('ш', small(sha, 520.0)),
        ('щ', shcha(XH, BASE, 520.0, DESC)),
        ('ъ', small(hard_sign, 440.0)),
        ('ы', small(yeru, 500.0)),
        ('ь', small(soft_sign, 340.0)),
        ('э', small(e, 380.0)),
        ('ю', small(yu, 560.0)),
        ('я', small(ya, 360.0)),
    ]
}

/// A letter at cap height; letters take the top and bottom of their body and
/// its width.
fn cap(f: fn(f64, f64, f64) -> Strokes, w: f64) -> Strokes {
    f(CAP, BASE, w)
}

/// A letter at x-height.
fn small(f: fn(f64, f64, f64) -> Strokes, w: f64) -> Strokes {
    f(XH, BASE, w)
}

// --- drawing toolkit -------------------------------------------------------

/// Elliptical arc from angle a0 to a1 in degrees; 0 is right, 90 is up.
/// Increasing angles run anticlockwise on screen.
fn arc(cx: f64, cy: f64, rx: f64, ry: f64, a0: f64, a1: f64) -> Vec<Pt> {
    let n = ((a1 - a0).abs() / 15.0).round_ties_even().max(4.0) as usize;
    super::arc((cx, cy), rx, -ry, a0, a1, n)
}

/// Quadratic Bezier from p0 to p2 pulled towards p1.
fn curve(p0: Pt, p1: Pt, p2: Pt, n: usize) -> Vec<Pt> {
    (0..=n)
        .map(|i| {
            let t = i as f64 / n as f64;
            let f = |a: f64, b: f64, c: f64| {
                (1.0 - t).powf(2.0) * a + 2.0 * (1.0 - t) * t * b + t.powf(2.0) * c
            };
            (f(p0.0, p1.0, p2.0), f(p0.1, p1.1, p2.1))
        })
        .collect()
}

/// Join point lists into one stroke, dropping repeated junction points.
fn path<const N: usize>(segments: [Vec<Pt>; N]) -> Vec<Pt> {
    super::join(segments, 1.0)
}

fn line<const N: usize>(points: [Pt; N]) -> Vec<Pt> {
    points.to_vec()
}

/// Clockwise bowl leaving a stem at (x, top) and returning at (x, bottom).
fn bowl(x: f64, top: f64, bottom: f64, right: f64) -> Vec<Pt> {
    let ry = (bottom - top) / 2.0;
    let rx = ry.min((right - x) * 0.7);
    path([
        line([(x, top), (right - rx, top)]),
        arc(right - rx, top + ry, rx, ry, 90.0, -90.0),
        line([(right - rx, bottom), (x, bottom)]),
    ])
}

fn lerp(p: Pt, q: Pt, t: f64) -> Pt {
    (p.0 + (q.0 - p.0) * t, p.1 + (q.1 - p.1) * t)
}

/// The two dots of Ё, left then right.
fn dots(y: f64) -> Strokes {
    vec![
        line([(CX - 90.0, y - 18.0), (CX - 90.0, y + 18.0)]),
        line([(CX + 90.0, y - 18.0), (CX + 90.0, y + 18.0)]),
    ]
}

/// The breve of Й: a cup drawn left to right.
fn breve(y: f64, w: f64) -> Strokes {
    vec![arc(CX, y - 25.0, w / 2.0, 50.0, 190.0, 350.0)]
}

// --- letters ---------------------------------------------------------------

fn a_cap(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let apex = (CX, t);
    vec![
        line([apex, (l, b)]),
        line([apex, (r, b)]),
        line([lerp(apex, (l, b), 0.66), lerp(apex, (r, b), 0.66)]),
    ]
}

fn be(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let m = t + (b - t) * 0.42;
    vec![
        line([(l, t), (l, b)]),
        line([(l, t), (r - w * 0.05, t)]),
        bowl(l, m, b, r),
    ]
}

fn ve(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let m = t + (b - t) * 0.47;
    vec![
        line([(l, t), (l, b)]),
        bowl(l, t, m, r - w * 0.1),
        bowl(l, m, b, r),
    ]
}

fn ghe(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    vec![line([(l, t), (l, b)]), line([(l, t), (r, t)])]
}

/// Л-shaped body, then one stroke up the left foot, along the base and down
/// the right foot.
fn de(t: f64, b: f64, w: f64, foot: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let stem = r - w * 0.12;
    let leg_top = l + w * 0.3;
    let leg_bottom = l + w * 0.1;
    vec![
        curve(
            (leg_top, t),
            (leg_top - w * 0.02, b - (b - t) * 0.4),
            (leg_bottom, b),
            12,
        ),
        line([(stem, t), (stem, b)]),
        line([(leg_top, t), (stem, t)]),
        line([(l, foot), (l, b), (r, b), (r, foot)]),
    ]
}

fn ie(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let m = t + (b - t) * 0.48;
    vec![
        line([(l, t), (l, b)]),
        line([(l, t), (r, t)]),
        line([(l, m), (r - w * 0.08, m)]),
        line([(l, b), (r, b)]),
    ]
}

/// Central stem, then each side as one "<" in to the stem and out again.
fn zhe(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let m = t + (b - t) * 0.48;
    let (left_top, right_top) = ((l + w * 0.03, t), (r - w * 0.03, t));
    vec![
        line([(CX, t), (CX, b)]),
        line([left_top, (CX, m), (l, b)]),
        line([right_top, (CX, m), (r, b)]),
    ]
}

/// One stroke like the digit 3: upper bowl, then the larger lower bowl.
fn ze(t: f64, b: f64, w: f64) -> Strokes {
    let h = b - t;
    let m = t + h * 0.47;
    let upper = arc(
        CX - w * 0.04,
        t + (m - t) / 2.0,
        w * 0.44,
        (m - t) / 2.0,
        150.0,
        -95.0,
    );
    let lower = arc(CX, m + (b - m) / 2.0, w / 2.0, (b - m) / 2.0, 95.0, -150.0);
    vec![path([upper, lower])]
}

/// Stems, then the diagonal pushed up from the foot of the left stem.
fn i(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    vec![
        line([(l, t), (l, b)]),
        line([(r, t), (r, b)]),
        line([(l, b), (r, t)]),
    ]
}

/// Stem, then one "<" from the top right in to the stem and out.
fn ka(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let m = t + (b - t) * 0.55;
    vec![
        line([(l, t), (l, b)]),
        line([(r - w * 0.04, t), (l, m), (r, b)]),
    ]
}

fn el(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let top = l + w * 0.22;
    vec![
        curve((top, t), (top, b - (b - t) * 0.15), (l, b), 12),
        line([(r, t), (r, b)]),
        line([(top, t), (r, t)]),
    ]
}

fn em(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    vec![
        line([(l, t), (l, b)]),
        line([(r, t), (r, b)]),
        line([(l, t), (CX, b), (r, t)]),
    ]
}

fn en(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let m = t + (b - t) * 0.48;
    vec![
        line([(l, t), (l, b)]),
        line([(r, t), (r, b)]),
        line([(l, m), (r, m)]),
    ]
}

fn o(t: f64, b: f64, w: f64) -> Strokes {
    vec![arc(CX, (t + b) / 2.0, w / 2.0, (b - t) / 2.0, 90.0, 450.0)]
}

fn pe(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    vec![
        line([(l, t), (l, b)]),
        line([(r, t), (r, b)]),
        line([(l, t), (r, t)]),
    ]
}

fn er_cap(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    vec![line([(l, t), (l, b)]), bowl(l, t, t + (b - t) * 0.56, r)]
}

fn es(t: f64, b: f64, w: f64) -> Strokes {
    vec![arc(CX, (t + b) / 2.0, w / 2.0, (b - t) / 2.0, 50.0, 310.0)]
}

/// The bar hangs the stem, so it comes first.
fn te(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    vec![line([(l, t), (r, t)]), line([(CX, t), (CX, b)])]
}

/// Short left arm to the joint, then the long right arm through it.
fn u(t: f64, b: f64, w: f64, tail: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let joint_y = if tail > b { b } else { t + (b - t) * 0.7 };
    let joint = (CX + w * 0.02, joint_y);
    let end = (l + w * 0.12, tail);
    let right = path([
        line([(r, t), joint]),
        curve(joint, lerp((r, t), joint, 1.25), end, 8),
    ]);
    vec![line([(l, t), joint]), right]
}

/// Stem, then the left half-oval and the right half-oval, both from the top.
fn ef(t: f64, b: f64, w: f64, stem_top: f64, stem_bottom: f64) -> Strokes {
    let (rx, ry) = (w / 2.0, (b - t) / 2.0);
    let cy = (t + b) / 2.0;
    vec![
        line([(CX, stem_top), (CX, stem_bottom)]),
        arc(CX, cy, rx, ry, 90.0, 270.0),
        arc(CX, cy, rx, ry, 90.0, -90.0),
    ]
}

fn ha(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    vec![line([(l, t), (r, b)]), line([(r, t), (l, b)])]
}

fn tse(t: f64, b: f64, w: f64, tail: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let stem = r - w * 0.12;
    vec![
        line([(l, t), (l, b)]),
        line([(stem, t), (stem, b)]),
        line([(l, b), (r, b)]),
        line([(r, b), (r, tail)]),
    ]
}

/// Left hook curving into the bar, then the full right stem.
fn che(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let h = b - t;
    let bar = t + h * 0.58;
    let rr = (h * 0.22).min(w * 0.4);
    vec![
        path([
            line([(l, t), (l, bar - rr)]),
            arc(l + rr, bar - rr, rr, rr, 180.0, 270.0),
            line([(l + rr, bar), (r, bar)]),
        ]),
        line([(r, t), (r, b)]),
    ]
}

fn sha(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    vec![
        line([(l, t), (l, b)]),
        line([(CX, t), (CX, b)]),
        line([(r, t), (r, b)]),
        line([(l, b), (r, b)]),
    ]
}

fn shcha(t: f64, b: f64, w: f64, tail: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let tail_x = r + w * 0.1;
    vec![
        line([(l, t), (l, b)]),
        line([(CX, t), (CX, b)]),
        line([(r, t), (r, b)]),
        line([(l, b), (tail_x, b)]),
        line([(tail_x, b), (tail_x, tail)]),
    ]
}

fn hard_sign(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let stem = l + w * 0.28;
    let m = t + (b - t) * 0.42;
    vec![
        line([(stem, t), (stem, b)]),
        line([(l, t), (stem, t)]),
        bowl(stem, m, b, r),
    ]
}

fn yeru(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let m = t + (b - t) * 0.42;
    vec![
        line([(l, t), (l, b)]),
        bowl(l, m, b, l + w * 0.62),
        line([(r, t), (r, b)]),
    ]
}

fn soft_sign(t: f64, b: f64, w: f64) -> Strokes {
    let l = CX - w / 2.0;
    let r = CX + w / 2.0;
    let m = t + (b - t) * 0.42;
    vec![line([(l, t), (l, b)]), bowl(l, m, b, r)]
}

/// Mirrored С drawn clockwise, then the bar out to the curve.
fn e(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let cy = (t + b) / 2.0;
    vec![
        arc(CX, cy, w / 2.0, (b - t) / 2.0, 130.0, -130.0),
        line([(l + w * 0.3, cy), (r, cy)]),
    ]
}

fn yu(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let cy = (t + b) / 2.0;
    let rx = w * 0.34;
    let ox = r - rx;
    vec![
        line([(l, t), (l, b)]),
        line([(l, cy), (ox - rx, cy)]),
        arc(ox, cy, rx, (b - t) / 2.0, 90.0, 450.0),
    ]
}

/// Right stem, bowl turning anticlockwise back to it, then the leg.
fn ya(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let m = t + (b - t) * 0.54;
    let ry = (m - t) / 2.0;
    let rx = ry.min(w * 0.6);
    let left = l + w * 0.05;
    let joint = (left + rx * 0.8, m);
    vec![
        line([(r, t), (r, b)]),
        path([
            line([(r, t), (left + rx, t)]),
            arc(left + rx, t + ry, rx, ry, 90.0, 270.0),
            line([(left + rx, m), (r, m)]),
        ]),
        line([joint, (l, b)]),
    ]
}

// Lowercase forms that are not small capitals.

/// Handwritten single-storey а, one stroke: circle anticlockwise from two
/// o'clock, push up to the x-height, pull the stem down.
fn a_small(w: f64) -> Strokes {
    let r = CX + w / 2.0;
    let (cy, ry) = ((XH + BASE) / 2.0, (BASE - XH) / 2.0);
    let rx = w * 0.45;
    vec![path([
        arc(r - rx, cy, rx, ry, 30.0, 360.0),
        line([(r, cy), (r, XH), (r, BASE)]),
    ])]
}

/// One stroke like the digit 6: a short flag down into the left side, then
/// the bowl anticlockwise.
fn be_small(w: f64) -> Strokes {
    let l = CX - w / 2.0;
    let (cy, ry) = ((XH + BASE) / 2.0, (BASE - XH) / 2.0);
    vec![path([
        curve(
            (l + w * 0.6, ASC),
            (l + w * 0.02, ASC + 10.0),
            (l, ASC + 170.0),
            12,
        ),
        line([(l, ASC + 170.0), (l, cy)]),
        arc(CX, cy, w / 2.0, ry, 180.0, 530.0),
    ])]
}

/// Bar left to right, then over the top and round anticlockwise.
fn ie_small(w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let cy = (XH + BASE) / 2.0 + 10.0;
    vec![path([
        line([(l, cy), (r, cy)]),
        arc(
            CX,
            (XH + BASE) / 2.0,
            w / 2.0,
            (BASE - XH) / 2.0,
            0.0,
            310.0,
        ),
    ])]
}

fn er_small(w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    vec![line([(l, XH), (l, DESC)]), bowl(l, XH, BASE, r)]
}
