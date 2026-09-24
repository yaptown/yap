//! Stroke order for the 66 Russian print letters.
//!
//! Each letter is one function returning its strokes in writing order; each
//! stroke is a list of (x, y) points in pen direction, in a 1000-unit box with
//! y pointing down. To fix a letter, edit its function.
//!
//! Formation follows Russian primary-school print (печатные буквы), as charted
//! in Безруких's print-letter direction charts and defectologiya.pro numbered
//! charts, applied consistently:
//! - stems first, top to bottom, left to right, except that a diagonal
//!   joining two stems comes between them (И) and a bar that hangs a stem
//!   comes before it (Т);
//! - then bars, diagonals and connectors, left to right (М's diagonals each
//!   from the top of its stem down to the valley);
//! - strokes follow handwriting, not typography (single-storey а);
//! - bowls leave the stem and turn clockwise back to it (Б В Р Ь Ъ Ы);
//! - ovals start at the top and run anticlockwise (О Ю, the bowl of б);
//! - tails and feet (Д Ц Щ), dots (Ё) and breves (Й) last; Д's left foot is
//!   its own short downstroke, then the base runs left to right into the
//!   right foot.
//!
//! Lowercase letters that are small capitals in print (в г д ж з и й к л м н
//! п т х ц ч ш щ ъ ы ь э ю я) reuse the capital's function at x-height.
//!
//! Accepted alternatives, each the other form common in Russian print and
//! handwriting (index 0 is the formation above):
//! - а: two-storey, as in typeset text, one stroke like the Latin one;
//! - б: the bowl first, then the flag rising right as its own stroke;
//! - Д д: a straight upright left leg (the Ц-like print form) instead of the
//!   curved one;
//! - Ж ж: five strokes (stem, then each arm on its own);
//! - К к: three strokes (stem, arm, then the leg from the same point).
//!
//! Then the other orders and shapes Russian preschool print materials teach
//! (Безруких's print-letter direction charts, the Радуга preschool workbook,
//! defectologiya.pro numbered charts):
//! - М м: the V in one stroke, down the left diagonal and up the right, after
//!   both stems (Радуга);
//! - Л л: the older school Λ, each side from the apex down, left first; Д д
//!   on the same Λ with the feet as in the taught form (Безруких, Радуга);
//! - У у: the long right arm first, then the short left arm;
//! - О о: two halves from the top, the left anticlockwise, then the right
//!   clockwise;
//! - К к: three strokes with the arm drawn outward, from the junction up and
//!   out, then the leg from the junction down and out;
//! - Ф ф: the stem, then the whole bowl as one anticlockwise oval;
//! - Ъ ъ: the flag before the stem, by analogy with Т.
//!
//! After those comes the cursive-derived print family. Russian schools teach
//! only cursive (прописи), so an adult's "print" handwriting is mostly the
//! cursive letters written unjoined, keeping their entry hooks and exit tails.
//! A letter gets a form here when its cursive shape differs visibly from
//! print:
//! - т written like a Latin m, and the same with a bar over it (т̅); ш as ɯ,
//!   and with a bar under it (ш̲); the bars are how writers tell т from ш;
//! - the minim letters, each one stroke with an entry hook: и й like u, п
//!   like n, ц щ with a looped tail, у with a looped descender, ч from a
//!   shallow cup back down its stem, л м as a pointed ʌ hooked at the foot;
//! - д like ꝺ/g (a looped descender) and like ∂; б as an oval with the stroke
//!   rising from its right side into the flag; в with a looped top over the
//!   x-height; г as ɿ; the loop е; з as ʒ, looped below the baseline; ж and х
//!   built from ")" and "("; р ь ъ ы with the stem retraced in one stroke;
//! - capitals where cursive differs: Д as a script D looped at the foot, Е Ё
//!   as Ɛ, Т as three stems under a wavy bar, and Ж З И Й Л М У Х Ц Ч Ш Щ as
//!   the lowercase cursive forms at cap height; Б В Р Ъ Ы Ь in one stroke with
//!   the stem retraced.
//!
//! The references are the standard Russian school cursive (школьные прописи,
//! as charted in Wikipedia's "Russian cursive" and Wikimedia Commons'
//! "Russian alphabet - cursive"), and that article's notes on т as 𝑚 with a
//! bar over it, ш with a bar under it, д as ꝺ or ∂, and п like a cursive n.
//! The geometry is drawn here; none is copied. Letters whose cursive shape
//! differs from print only by joining strokes (а к н о с ф э ю я, Г П) have
//! no form of their own.
use super::{Pt, Strokes, collect, glyph};
use crate::{Forms, StrokeStandard};

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

pub fn glyphs() -> Forms {
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
        ('Б', cap(be_retraced, 420.0)),
        ('В', cap(ve, 420.0)),
        ('В', cap(ve_retraced, 420.0)),
        ('Г', cap(ghe, 340.0)),
        ('Д', de(CAP, BASE, 540.0, DESC)),
        ('Д', de_straight(CAP, BASE, 540.0, DESC)),
        ('Д', cap(de_script, 540.0)),
        ('Д', de_lambda(CAP, BASE, 540.0, DESC)),
        ('Е', cap(ie, 380.0)),
        ('Е', cap(ie_script, 380.0)),
        ('Ё', [cap(ie, 380.0), dots(MARK)].concat()),
        ('Ё', [cap(ie_script, 380.0), dots(MARK)].concat()),
        ('Ж', cap(zhe, 680.0)),
        ('Ж', cap(zhe_five, 680.0)),
        ('Ж', cap(zhe_cursive, 680.0)),
        ('З', cap(ze, 400.0)),
        ('З', cap(ze_cursive, 400.0)),
        ('И', cap(i, 460.0)),
        ('И', cap(i_cursive, 460.0)),
        ('Й', [cap(i, 460.0), breve(MARK, 200.0)].concat()),
        ('Й', [cap(i_cursive, 460.0), breve(MARK, 200.0)].concat()),
        ('К', cap(ka, 440.0)),
        ('К', cap(ka_three, 440.0)),
        ('К', cap(ka_out, 440.0)),
        ('Л', cap(el, 460.0)),
        ('Л', cap(el_cursive, 460.0)),
        ('Л', cap(el_lambda, 460.0)),
        ('М', cap(em, 560.0)),
        ('М', cap(em_v, 560.0)),
        ('М', cap(em_cursive, 560.0)),
        ('Н', cap(en, 460.0)),
        ('О', cap(o, 580.0)),
        ('О', cap(o_halves, 580.0)),
        ('П', cap(pe, 460.0)),
        ('Р', cap(er_cap, 400.0)),
        ('Р', cap(er_retraced, 400.0)),
        ('С', cap(es, 500.0)),
        ('Т', cap(te, 480.0)),
        ('Т', cap(te_script, 540.0)),
        ('У', u(CAP, BASE, 500.0, BASE)),
        ('У', cap(u_cursive, 500.0)),
        ('У', u_long_first(CAP, BASE, 500.0, BASE)),
        ('Ф', ef(CAP + 90.0, BASE - 90.0, 620.0, CAP, BASE)),
        ('Ф', ef_oval(CAP + 90.0, BASE - 90.0, 620.0, CAP, BASE)),
        ('Х', cap(ha, 500.0)),
        ('Х', cap(ha_cursive, 500.0)),
        ('Ц', tse(CAP, BASE, 480.0, DESC)),
        ('Ц', cap(tse_cursive, 480.0)),
        ('Ч', cap(che, 420.0)),
        ('Ч', cap(che_cursive, 420.0)),
        ('Ш', cap(sha, 620.0)),
        ('Ш', cap(sha_cursive, 620.0)),
        ('Щ', shcha(CAP, BASE, 620.0, DESC)),
        ('Щ', cap(shcha_cursive, 620.0)),
        ('Ъ', cap(hard_sign, 520.0)),
        ('Ъ', cap(hard_sign_cursive, 520.0)),
        ('Ъ', cap(hard_sign_flag_first, 520.0)),
        ('Ы', cap(yeru, 580.0)),
        ('Ы', cap(yeru_cursive, 580.0)),
        ('Ь', cap(soft_sign, 400.0)),
        ('Ь', cap(soft_sign_cursive, 400.0)),
        ('Э', cap(e, 480.0)),
        ('Ю', cap(yu, 680.0)),
        ('Я', cap(ya, 420.0)),
        ('а', a_small(380.0)),
        ('а', a_two_storey(380.0)),
        ('б', be_small(400.0)),
        ('б', be_small_flag(400.0)),
        ('б', be_cursive(380.0)),
        ('в', small(ve, 340.0)),
        ('в', ve_cursive(340.0)),
        ('г', small(ghe, 280.0)),
        ('г', ghe_cursive(300.0)),
        ('д', de(XH, BASE, 440.0, DESC)),
        ('д', de_straight(XH, BASE, 440.0, DESC)),
        ('д', de_g(420.0)),
        ('д', de_delta(420.0)),
        ('д', de_lambda(XH, BASE, 440.0, DESC)),
        ('е', ie_small(400.0)),
        ('е', ie_loop(380.0)),
        ('ё', [ie_small(400.0), dots(XH - 110.0)].concat()),
        ('ё', [ie_loop(380.0), dots(XH - 110.0)].concat()),
        ('ж', small(zhe, 560.0)),
        ('ж', small(zhe_five, 560.0)),
        ('ж', small(zhe_cursive, 560.0)),
        ('з', small(ze, 340.0)),
        ('з', small(ze_cursive, 340.0)),
        ('и', small(i, 380.0)),
        ('и', small(i_cursive, 360.0)),
        ('й', [small(i, 380.0), breve(XH - 110.0, 180.0)].concat()),
        (
            'й',
            [small(i_cursive, 360.0), breve(XH - 110.0, 180.0)].concat(),
        ),
        ('к', small(ka, 360.0)),
        ('к', small(ka_three, 360.0)),
        ('к', small(ka_out, 360.0)),
        ('л', small(el, 380.0)),
        ('л', small(el_cursive, 360.0)),
        ('л', small(el_lambda, 380.0)),
        ('м', small(em, 460.0)),
        ('м', small(em_v, 460.0)),
        ('м', small(em_cursive, 460.0)),
        ('н', small(en, 380.0)),
        ('о', small(o, 420.0)),
        ('о', small(o_halves, 420.0)),
        ('п', small(pe, 380.0)),
        ('п', pe_n(360.0)),
        ('р', er_small(380.0)),
        ('р', er_cursive(380.0)),
        ('с', small(es, 380.0)),
        ('т', small(te, 400.0)),
        ('т', te_m(460.0)),
        ('т', te_m_barred(460.0)),
        ('у', u(XH, BASE, 420.0, DESC)),
        ('у', small(u_cursive, 380.0)),
        ('у', u_long_first(XH, BASE, 420.0, DESC)),
        ('ф', ef(XH, BASE, 500.0, ASC, DESC)),
        ('ф', ef_oval(XH, BASE, 500.0, ASC, DESC)),
        ('х', small(ha, 400.0)),
        ('х', small(ha_cursive, 400.0)),
        ('ц', tse(XH, BASE, 400.0, DESC)),
        ('ц', small(tse_cursive, 360.0)),
        ('ч', small(che, 360.0)),
        ('ч', small(che_cursive, 340.0)),
        ('ш', small(sha, 520.0)),
        ('ш', small(sha_cursive, 500.0)),
        ('ш', sha_barred(500.0)),
        ('щ', shcha(XH, BASE, 520.0, DESC)),
        ('щ', small(shcha_cursive, 500.0)),
        ('ъ', small(hard_sign, 440.0)),
        ('ъ', small(hard_sign_cursive, 440.0)),
        ('ъ', small(hard_sign_flag_first, 440.0)),
        ('ы', small(yeru, 500.0)),
        ('ы', small(yeru_cursive, 500.0)),
        ('ь', small(soft_sign, 340.0)),
        ('ь', small(soft_sign_cursive, 340.0)),
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

/// The feet of Д: the left foot down on its own, then the base left to right
/// running on down the right foot.
fn de_feet(l: f64, r: f64, b: f64, foot: f64) -> Strokes {
    vec![line([(l, b), (l, foot)]), line([(l, b), (r, b), (r, foot)])]
}

/// Л-shaped body, then the feet.
fn de(t: f64, b: f64, w: f64, foot: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let stem = r - w * 0.12;
    let leg_top = l + w * 0.3;
    let leg_bottom = l + w * 0.1;
    [
        vec![
            curve(
                (leg_top, t),
                (leg_top - w * 0.02, b - (b - t) * 0.4),
                (leg_bottom, b),
                12,
            ),
            line([(stem, t), (stem, b)]),
            line([(leg_top, t), (stem, t)]),
        ],
        de_feet(l, r, b, foot),
    ]
    .concat()
}

/// Д with a straight upright left leg (the Ц-like print form), then the same
/// stem, roof and feet.
fn de_straight(t: f64, b: f64, w: f64, foot: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let stem = r - w * 0.12;
    let leg = l + w * 0.18;
    [
        vec![
            line([(leg, t), (leg, b)]),
            line([(stem, t), (stem, b)]),
            line([(leg, t), (stem, t)]),
        ],
        de_feet(l, r, b, foot),
    ]
    .concat()
}

/// The older school Д on a Λ: each side from the apex down, then the feet.
fn de_lambda(t: f64, b: f64, w: f64, foot: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    [el_lambda(t, b, w * 0.76), de_feet(l, r, b, foot)].concat()
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

/// Central stem, then each of the four arms on its own: the left arm in to
/// the stem and the left leg out, then the right pair.
fn zhe_five(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let m = t + (b - t) * 0.48;
    let (left_top, right_top) = ((l + w * 0.03, t), (r - w * 0.03, t));
    vec![
        line([(CX, t), (CX, b)]),
        line([left_top, (CX, m)]),
        line([(CX, m), (l, b)]),
        line([right_top, (CX, m)]),
        line([(CX, m), (r, b)]),
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

/// Left stem, the diagonal pushed up from its foot, then the right stem.
fn i(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    vec![
        line([(l, t), (l, b)]),
        line([(l, b), (r, t)]),
        line([(r, t), (r, b)]),
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

/// Stem, the arm in to it, then the leg out from the same point.
fn ka_three(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let m = t + (b - t) * 0.55;
    vec![
        line([(l, t), (l, b)]),
        line([(r - w * 0.04, t), (l, m)]),
        line([(l, m), (r, b)]),
    ]
}

/// Stem, then the arm out from the junction up to the right, then the leg out
/// from the same point.
fn ka_out(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let m = t + (b - t) * 0.55;
    vec![
        line([(l, t), (l, b)]),
        line([(l, m), (r - w * 0.04, t)]),
        line([(l, m), (r, b)]),
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

/// The older school Л, a Λ: each side from the apex down, left first.
fn el_lambda(t: f64, b: f64, w: f64) -> Strokes {
    let apex = (CX, t);
    vec![
        line([apex, (CX - w / 2.0, b)]),
        line([apex, (CX + w / 2.0, b)]),
    ]
}

/// Both stems, then each diagonal from the top of its stem down to the
/// valley.
fn em(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    vec![
        line([(l, t), (l, b)]),
        line([(r, t), (r, b)]),
        line([(l, t), (CX, b)]),
        line([(r, t), (CX, b)]),
    ]
}

/// Both stems, then the V in one stroke, down to the valley and up again.
fn em_v(t: f64, b: f64, w: f64) -> Strokes {
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

/// О as two halves from the top: the left anticlockwise, then the right
/// clockwise.
fn o_halves(t: f64, b: f64, w: f64) -> Strokes {
    let (cy, rx, ry) = ((t + b) / 2.0, w / 2.0, (b - t) / 2.0);
    vec![
        arc(CX, cy, rx, ry, 90.0, 270.0),
        arc(CX, cy, rx, ry, 90.0, -90.0),
    ]
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

/// У with the long right arm first, then the short left arm.
fn u_long_first(t: f64, b: f64, w: f64, tail: f64) -> Strokes {
    let mut strokes = u(t, b, w, tail);
    strokes.reverse();
    strokes
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

/// Stem, then the whole bowl as one oval, anticlockwise from the top.
fn ef_oval(t: f64, b: f64, w: f64, stem_top: f64, stem_bottom: f64) -> Strokes {
    vec![
        line([(CX, stem_top), (CX, stem_bottom)]),
        arc(CX, (t + b) / 2.0, w / 2.0, (b - t) / 2.0, 90.0, 450.0),
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

/// Ъ with the flag first, as the bar of Т: flag, stem, bowl.
fn hard_sign_flag_first(t: f64, b: f64, w: f64) -> Strokes {
    let mut strokes = hard_sign(t, b, w);
    strokes.swap(0, 1);
    strokes
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

/// Two-storey а, one stroke: the hook over the top, down the stem, round
/// the bowl anticlockwise from the stem back to it, down to the baseline.
fn a_two_storey(w: f64) -> Strokes {
    let r = CX + w * 0.42;
    let (brx, bry) = (w * 0.4, 110.0);
    let by = BASE - bry;
    vec![path([
        arc(CX, XH + 125.0, w * 0.42, 120.0, 150.0, 0.0),
        line([(r, XH + 125.0), (r, by - bry * 20f64.to_radians().sin())]),
        arc(r - brx, by, brx, bry, 20.0, 360.0),
        line([(r, by), (r, BASE)]),
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

/// The bowl and its left side as one stroke from the top, then the flag
/// rising to the right as a stroke of its own.
fn be_small_flag(w: f64) -> Strokes {
    let l = CX - w / 2.0;
    let (cy, ry) = ((XH + BASE) / 2.0, (BASE - XH) / 2.0);
    vec![
        path([
            line([(l, ASC + 60.0), (l, cy)]),
            arc(CX, cy, w / 2.0, ry, 180.0, 530.0),
        ]),
        curve(
            (l, ASC + 60.0),
            (l + w * 0.3, ASC + 20.0),
            (l + w * 0.75, ASC),
            8,
        ),
    ]
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

// --- cursive-derived print family --------------------------------------------

/// A smooth stroke through waypoints (a centripetal Catmull-Rom spline).
fn smooth(points: &[Pt]) -> Vec<Pt> {
    super::centripetal(points, super::Ends::Reflect, 1.0, |len| {
        (len / 20.0).ceil().max(1.0) as usize
    })
}

/// How big a letter's connectors are: its body height, capped at x-height
/// size so capitals do not get giant hooks.
fn reach(t: f64, b: f64) -> f64 {
    (b - t).min(BASE - XH)
}

/// The entry hook of a cursive letter: from low on the left up into the top
/// of the first stem at `(x, t)`.
fn entry(x: f64, t: f64, e: f64) -> Vec<Pt> {
    smooth(&[
        (x - 0.2 * e, t + 0.3 * e),
        (x - 0.07 * e, t + 0.08 * e),
        (x, t),
    ])
}

/// The exit tail of a cursive letter: a stem ending at `(x, b - 0.2e)` bends
/// out along the baseline and up to the right.
fn exit(x: f64, b: f64, e: f64) -> Vec<Pt> {
    smooth(&[
        (x, b - 0.2 * e),
        (x + 0.02 * e, b - 0.06 * e),
        (x + 0.1 * e, b),
        (x + 0.22 * e, b - 0.07 * e),
    ])
}

/// A minim and its foot: down a stem at `x0` from `t`, round the bottom and
/// up the next stem at `x1` to `t` again (the cursive и, ш, у).
fn cup(x0: f64, x1: f64, t: f64, b: f64) -> Vec<Pt> {
    let rx = (x1 - x0) / 2.0;
    let ry = rx.min((b - t) * 0.35);
    path([
        line([(x0, t), (x0, b - ry)]),
        arc(x0 + rx, b - ry, rx, ry, 180.0, 360.0),
        line([(x1, b - ry), (x1, t)]),
    ])
}

/// An arch: from the foot of a stem at `x0` back up it, over the top and
/// down the next stem at `x1` to `end` (the cursive т and п).
fn arch(x0: f64, x1: f64, t: f64, b: f64, end: f64) -> Vec<Pt> {
    let rx = (x1 - x0) / 2.0;
    let ry = rx.min((b - t) * 0.35);
    path([
        line([(x0, b), (x0, t + ry)]),
        arc(x0 + rx, t + ry, rx, ry, 180.0, 0.0),
        line([(x1, t + ry), (x1, end)]),
    ])
}

/// A looped descender off a stem at `x`: down from the baseline, round to
/// the left and back up across the stem, out to the right.
fn loop_tail(x: f64, b: f64) -> Vec<Pt> {
    smooth(&[
        (x, b),
        (x, DESC - 50.0),
        (x - 25.0, DESC - 5.0),
        (x - 75.0, DESC - 2.0),
        (x - 100.0, DESC - 35.0),
        (x - 80.0, b + 40.0),
        (x - 20.0, b + 5.0),
        (x + 60.0, b - 20.0),
    ])
}

/// The entry, a row of minims joined at their feet, the last stem down.
fn minims(stems: &[f64], t: f64, b: f64, e: f64) -> Vec<Vec<Pt>> {
    let mut parts = vec![entry(stems[0], t, e)];
    parts.extend(stems.windows(2).map(|s| cup(s[0], s[1], t, b)));
    parts
}

/// Cursive и: like a Latin u, one stroke.
fn i_cursive(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r, e) = (CX - w / 2.0, CX + w / 2.0, reach(t, b));
    let mut parts = minims(&[l, r], t, b, e);
    parts.extend([line([(r, t), (r, b - 0.2 * e)]), exit(r, b, e)]);
    vec![super::join(parts, 1.0)]
}

/// Cursive ш: three minims (ɯ), one stroke.
fn sha_cursive(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r, e) = (CX - w / 2.0, CX + w / 2.0, reach(t, b));
    let mut parts = minims(&[l, CX, r], t, b, e);
    parts.extend([line([(r, t), (r, b - 0.2 * e)]), exit(r, b, e)]);
    vec![super::join(parts, 1.0)]
}

/// Cursive ц: the и with a looped tail off the last stem.
fn tse_cursive(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r, e) = (CX - w / 2.0, CX + w / 2.0, reach(t, b));
    let mut parts = minims(&[l, r], t, b, e);
    parts.extend([line([(r, t), (r, b)]), loop_tail(r, b)]);
    vec![super::join(parts, 1.0)]
}

/// Cursive щ: the ш with a looped tail off the last stem.
fn shcha_cursive(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r, e) = (CX - w / 2.0, CX + w / 2.0, reach(t, b));
    let mut parts = minims(&[l, CX, r], t, b, e);
    parts.extend([line([(r, t), (r, b)]), loop_tail(r, b)]);
    vec![super::join(parts, 1.0)]
}

/// Cursive у: a cup, then the right stem down into a looped descender.
fn u_cursive(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r, e) = (CX - w / 2.0, CX + w / 2.0, reach(t, b));
    let foot = if b - t > BASE - XH {
        t + (b - t) * 0.55
    } else {
        b
    };
    vec![path([
        entry(l, t, e),
        cup(l, r, t, foot),
        line([(r, t), (r, b)]),
        loop_tail(r, b),
    ])]
}

/// Cursive ч: a shallow cup, then back down the right stem, one stroke.
fn che_cursive(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r, e) = (CX - w / 2.0, CX + w / 2.0, reach(t, b));
    vec![path([
        entry(l, t, e),
        cup(l, r, t, t + (b - t) * 0.58),
        line([(r, t), (r, b - 0.2 * e)]),
        exit(r, b, e),
    ])]
}

/// The hook and rising curve that start cursive л and м, ending at the top
/// of a stem at `(x, t)`.
fn el_rise(l: f64, x: f64, t: f64, b: f64, e: f64) -> Vec<Pt> {
    smooth(&[
        (l - 0.05 * e, b - 0.14 * e),
        (l + 0.02 * e, b - 0.01 * e),
        (l + 0.12 * e, b - 0.06 * e),
        (
            lerp((l, b), (x, t), 0.55).0,
            lerp((l, b), (x, t), 0.55).1 + 0.05 * e,
        ),
        (x - 0.08 * e, t + 0.1 * e),
        (x, t),
    ])
}

/// Cursive л: a pointed ʌ, hooked at the foot, the right side a stem.
fn el_cursive(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r, e) = (CX - w / 2.0, CX + w / 2.0, reach(t, b));
    vec![path([
        el_rise(l, r, t, b, e),
        line([(r, t), (r, b - 0.2 * e)]),
        exit(r, b, e),
    ])]
}

/// Cursive м: the л, then up again to a second stem.
fn em_cursive(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r, e) = (CX - w / 2.0, CX + w / 2.0, reach(t, b));
    let mid = CX + w * 0.02;
    vec![path([
        el_rise(l, mid, t, b, e),
        line([(mid, t), (mid, b), (r, t), (r, b - 0.2 * e)]),
        exit(r, b, e),
    ])]
}

/// Cursive х: ")" then "(" touching back to back.
fn ha_cursive(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let (cy, ry) = ((t + b) / 2.0, (b - t) / 2.0);
    vec![
        arc(l, cy, w / 2.0, ry, 110.0, -110.0),
        arc(r, cy, w / 2.0, ry, 70.0, 290.0),
    ]
}

/// Cursive ж: ")" into the stem, the stem, then "(".
fn zhe_cursive(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let (cy, ry) = ((t + b) / 2.0, (b - t) / 2.0);
    vec![
        arc(l, cy, w / 2.0, ry, 110.0, -110.0),
        line([(CX, t), (CX, b)]),
        arc(r, cy, w / 2.0, ry, 70.0, 290.0),
    ]
}

/// Cursive з: a small top bowl, then the lower bowl running on into a
/// looped descender (ʒ).
fn ze_cursive(t: f64, b: f64, w: f64) -> Strokes {
    let h = b - t;
    let m = t + h * 0.42;
    let x = CX + w * 0.1;
    let upper = arc(CX, t + (m - t) / 2.0, w * 0.4, (m - t) / 2.0, 150.0, -90.0);
    let lower = smooth(&[
        (CX, m),
        (CX + w * 0.3, m + h * 0.07),
        (CX + w * 0.48, m + (b - m) * 0.45),
        (CX + w * 0.35, b - (b - m) * 0.12),
        (x, b),
    ]);
    vec![path([upper, lower, loop_tail(x, b)])]
}

/// Cursive ь in one stroke: down the stem, back up it, round the bowl.
fn soft_sign_cursive(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let m = t + (b - t) * 0.42;
    vec![path([line([(l, t), (l, b), (l, m)]), bowl(l, m, b, r)])]
}

/// Cursive ъ in one stroke: the tick into the stem, then the ь.
fn hard_sign_cursive(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let stem = l + w * 0.28;
    let m = t + (b - t) * 0.42;
    vec![path([
        line([(l, t + (b - t) * 0.08), (stem, t), (stem, b), (stem, m)]),
        bowl(stem, m, b, r),
    ])]
}

/// Cursive ы: the one-stroke ь, then the stem.
fn yeru_cursive(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let m = t + (b - t) * 0.42;
    vec![
        path([line([(l, t), (l, b), (l, m)]), bowl(l, m, b, l + w * 0.62)]),
        line([(r, t), (r, b)]),
    ]
}

// Capitals only.

/// Б in one stroke: the bar drawn right to left into the stem, down, back
/// up to the waist and round the bowl.
fn be_retraced(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let m = t + (b - t) * 0.42;
    vec![path([
        line([(r - w * 0.05, t), (l, t), (l, b), (l, m)]),
        bowl(l, m, b, r),
    ])]
}

/// В in one stroke: the stem down and back up, then both bowls.
fn ve_retraced(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let m = t + (b - t) * 0.47;
    vec![path([
        line([(l, t), (l, b), (l, t)]),
        bowl(l, t, m, r - w * 0.1),
        bowl(l, m, b, r),
    ])]
}

/// Р in one stroke: the stem down and back up, then the bowl.
fn er_retraced(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    vec![path([
        line([(l, t), (l, b), (l, t)]),
        bowl(l, t, t + (b - t) * 0.56, r),
    ])]
}

/// Cursive Д, like a script D: the stem down, a small loop at its foot,
/// along the baseline, up round the bowl and over the top to a curl.
fn de_script(t: f64, b: f64, w: f64) -> Strokes {
    let (h, x) = (b - t, |f: f64| CX + w * f);
    vec![smooth(&[
        (x(0.04), t + h * 0.03),
        (x(-0.02), t + h * 0.5),
        (x(-0.1), b - h * 0.07),
        (x(-0.2), b),
        (x(-0.33), b - h * 0.03),
        (x(-0.34), b - h * 0.08),
        (x(-0.25), b - h * 0.09),
        (x(-0.05), b),
        (x(0.2), b - h * 0.02),
        (x(0.42), b - h * 0.2),
        (x(0.5), t + h * 0.5),
        (x(0.42), t + h * 0.15),
        (x(0.15), t),
        (x(-0.2), t + h * 0.03),
        (x(-0.4), t + h * 0.12),
        (x(-0.35), t + h * 0.2),
    ])]
}

/// Cursive Е (Ɛ), one stroke: the small upper bowl anticlockwise from the
/// top right, then the larger lower bowl.
fn ie_script(t: f64, b: f64, w: f64) -> Strokes {
    let h = b - t;
    let m = t + h * 0.46;
    vec![path([
        arc(
            CX + w * 0.04,
            (t + m) / 2.0,
            w * 0.4,
            (m - t) / 2.0,
            35.0,
            280.0,
        ),
        arc(
            CX + w * 0.04,
            (m + b) / 2.0,
            w * 0.5,
            (b - m) / 2.0,
            80.0,
            325.0,
        ),
    ])]
}

/// Cursive Т: three stems, then the wavy bar over them.
fn te_script(t: f64, b: f64, w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let top = t + 50.0;
    vec![
        line([(l, top), (l, b)]),
        line([(CX, top), (CX, b)]),
        line([(r, top), (r, b)]),
        smooth(&[
            (l - w * 0.08, t + 20.0),
            (CX - w * 0.25, t - 15.0),
            (CX + w * 0.2, t + 5.0),
            (r + w * 0.08, t - 25.0),
        ]),
    ]
}

// Lowercase only.

/// Cursive т like a Latin m: the entry, down the first stem, then two arches.
fn te_m(w: f64) -> Strokes {
    let (l, r, e) = (CX - w / 2.0, CX + w / 2.0, reach(XH, BASE));
    vec![path([
        entry(l, XH, e),
        line([(l, XH), (l, BASE)]),
        arch(l, CX, XH, BASE, BASE),
        arch(CX, r, XH, BASE, BASE - 0.2 * e),
        exit(r, BASE, e),
    ])]
}

/// The m-shaped т with the bar above it that tells it from ш.
fn te_m_barred(w: f64) -> Strokes {
    let y = XH - 70.0;
    [te_m(w), vec![line([(CX - w / 2.0, y), (CX + w / 2.0, y)])]].concat()
}

/// The ɯ-shaped ш with the bar under it that tells it from т.
fn sha_barred(w: f64) -> Strokes {
    let y = BASE + 70.0;
    [
        small(sha_cursive, w),
        vec![line([(CX - w / 2.0, y), (CX + w / 2.0, y)])],
    ]
    .concat()
}

/// Cursive п like a Latin n.
fn pe_n(w: f64) -> Strokes {
    let (l, r, e) = (CX - w / 2.0, CX + w / 2.0, reach(XH, BASE));
    vec![path([
        entry(l, XH, e),
        line([(l, XH), (l, BASE)]),
        arch(l, r, XH, BASE, BASE - 0.2 * e),
        exit(r, BASE, e),
    ])]
}

/// Cursive г: a hook curling over from the left into the stem (ɿ).
fn ghe_cursive(w: f64) -> Strokes {
    let x = |f: f64| CX + w * f;
    vec![smooth(&[
        (x(-0.5), XH + 110.0),
        (x(-0.3), XH + 15.0),
        (x(0.0), XH),
        (x(0.2), XH + 50.0),
        (x(0.22), XH + 200.0),
        (x(0.2), BASE - 40.0),
        (x(0.1), BASE),
    ])]
}

/// The oval that starts cursive а-like letters: anticlockwise from two
/// o'clock back round to a stem at the right; returns the stem's x.
fn oval(w: f64) -> (Vec<Pt>, f64) {
    let (cy, ry) = ((XH + BASE) / 2.0, (BASE - XH) / 2.0);
    let (ox, rx) = (CX - w * 0.1, w * 0.36);
    (arc(ox, cy, rx, ry, 30.0, 360.0), ox + rx)
}

/// Cursive д like ꝺ/g: the oval, up to the x-height, then down the stem into
/// a looped descender.
fn de_g(w: f64) -> Strokes {
    let (oval, x) = oval(w);
    vec![path([oval, line([(x, XH), (x, BASE)]), loop_tail(x, BASE)])]
}

/// Cursive д like ∂: the oval, then on up a tall stem curling back left.
fn de_delta(w: f64) -> Strokes {
    let (oval, x) = oval(w);
    let cy = (XH + BASE) / 2.0;
    vec![path([
        oval,
        smooth(&[
            (x, cy),
            (x - 5.0, XH),
            (x - w * 0.12, ASC + 80.0),
            (x - w * 0.35, ASC),
            (x - w * 0.6, ASC + 30.0),
        ]),
    ])]
}

/// Cursive б: the oval from the top right anticlockwise, then up from its
/// right side into the flag, which turns right.
fn be_cursive(w: f64) -> Strokes {
    let (cy, ry) = ((XH + BASE) / 2.0, (BASE - XH) / 2.0);
    let rx = w / 2.0;
    let oval = arc(CX, cy, rx, ry, 20.0, 380.0);
    let start = *oval.last().unwrap();
    vec![path([
        oval,
        smooth(&[
            start,
            (CX + rx * 0.95, XH - 30.0),
            (CX + w * 0.36, ASC + 110.0),
            (CX + w * 0.44, ASC + 25.0),
            (CX + w * 0.62, ASC),
        ]),
    ])]
}

/// Cursive в: up a narrow loop over the x-height, down its left side as the
/// stem, round the bowl and back in to the stem at the waist.
fn ve_cursive(w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let x = |f: f64| l + w * f;
    vec![smooth(&[
        (x(0.12), XH + 70.0),
        (x(0.3), XH - 100.0),
        (x(0.22), ASC + 90.0),
        (x(0.06), ASC + 70.0),
        (x(-0.02), ASC + 150.0),
        (l, XH),
        (l, BASE - 60.0),
        (x(0.15), BASE),
        (x(0.62), BASE),
        (r, BASE - 100.0),
        (x(0.65), XH + 170.0),
        (x(0.08), XH + 150.0),
        (x(0.2), XH + 110.0),
    ])]
}

/// The cursive loop е: up from the middle into the loop, over the top and
/// round anticlockwise.
fn ie_loop(w: f64) -> Strokes {
    let (l, r) = (CX - w / 2.0, CX + w / 2.0);
    let h = BASE - XH;
    let x = |f: f64| l + w * f;
    vec![smooth(&[
        (x(0.05), XH + h * 0.58),
        (x(0.5), XH + h * 0.52),
        (x(0.9), XH + h * 0.35),
        (x(0.82), XH + h * 0.06),
        (CX, XH),
        (x(0.1), XH + h * 0.2),
        (l, XH + h * 0.55),
        (x(0.18), BASE - h * 0.04),
        (x(0.6), BASE),
        (r, BASE - h * 0.15),
    ])]
}

/// Cursive р in one stroke: the entry, the stem down and back up, the bowl.
fn er_cursive(w: f64) -> Strokes {
    let (l, r, e) = (CX - w / 2.0, CX + w / 2.0, reach(XH, BASE));
    vec![path([
        entry(l, XH, e),
        line([(l, XH), (l, DESC), (l, XH)]),
        bowl(l, XH, BASE, r),
    ])]
}
