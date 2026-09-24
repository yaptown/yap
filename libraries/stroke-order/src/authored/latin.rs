//! Print-letter stroke order for the Latin alphabet.
//!
//! Covers A-Z, a-z, the digits and every accented letter used by the English,
//! French, Spanish, German, Portuguese and Italian courses. Letters are the
//! single-storey "ball and stick" print forms taught in primary school (close
//! to Zaner-Bloser), drawn as pen-direction centerlines. Accented letters are
//! composed from their Unicode decomposition: base letter first, then the
//! marks, the way a learner dots an i.
//!
//! Accepted alternatives (index 0 is always the ball-and-stick form above),
//! drawn in the same frame after the common print and typographic forms:
//! - a: double-storey, one stroke (hook, down the stem, round the bowl back
//!   to the stem, push up and pull down the stem);
//! - g: double-storey (the bowl, then the stem into the closed lower loop);
//! - y: curved tail, one stroke (a u whose right stem runs into g's tail);
//! - t: stem curving right at the foot, then the crossbar;
//! - k, K: three strokes (stem, arm, leg from the middle of the arm);
//! - M: stem, V, stem as three strokes; and one zigzag up from the left foot;
//! - N: stem, diagonal, stem as three strokes; and one zigzag up from the
//!   left foot;
//! - I: the bare stem; J: with a top bar; z, Z, 7: with a crossbar;
//! - Q: a tail hanging below the circle;
//! - ß: one stroke from the baseline, up the stem and down through the bumps;
//! - 1: the bare stroke, and flag and foot;
//! - 4: open top, the down stroke and bar apart from the stem.
//!
//! After those comes the cursive-derived print family, what writers schooled
//! in France, Spain, Portugal, Italy and Latin America produce: the Lund
//! University block-letter allograph study's "Lower-3" (one arc per letter,
//! with connectors protruding from the ends) and "Upper-2" (the capital's
//! stem drawn down and back up before the rest of the letter). The letters
//! stay unjoined but keep their connectors:
//! - entry strokes into i ı j m n r t u w y and exit tails out of a d h i l m
//!   n t u; c e o v w start or end in a hook or knot;
//! - looped ascenders drawn in one stroke on b d f h k l (b and d as in
//!   a handwriting sample), looped descenders on f g j y and a looped q;
//! - a from its flat top with a plain stem, and with the stem as a small
//!   loop; the cursive loop e; k with a knot at the waist; the French p (the
//!   stem from half-way up the ascender, the bowl open at the bottom); the
//!   cursive r and s; x as two crossed arcs; z as ʒ and the italic z, with
//!   and without a bar; f with a looped top and a straight stem and bar;
//!   crossed q; the print r with an entry stroke;
//! - capitals B D E F M N P R with the retraced stem, H and K retracing to
//!   the bar or joint, A in one stroke from the lower left, L and Z with a
//!   loop at the foot (Z also barred), and the hooked I, J, S and T.
//!
//! The family's references are the school handwriting models documented by
//! Primarium (<https://primarium.info/handwriting-models/>, CC BY-SA 4.0):
//! Écriture A and B and Méthode Dumont (France), Cuadernos Rubio and
//! Santillana (Spain), Porto Editora (Portugal), Letra Brasileira (Brazil),
//! Corsivo tradizionale and Italica (Italy); and Wikipedia's "Regional
//! handwriting variation" for the French p, crossed q, looped k and the Z
//! with a bottom loop. The geometry is drawn here; none is copied. Each
//! form's comment names the models it follows.
//!
//! Accented letters take every form of their base (à has all four a's, ÿ all
//! three y's); ª and º keep the taught form.
//!
//! Frame (1000 em, y down): ascender 150, capitals and digits 190..850 (a
//! little below the ascender to leave room for accents), x-height 450,
//! baseline 850, descender 975. Arc angles are in degrees,
//! counterclockwise-positive as on paper: 0 is 3 o'clock, 90 is 12 o'clock, so
//! increasing angles draw a counterclockwise ("circle back") curve and
//! decreasing ones a clockwise ("circle forward") curve.
use super::{Coord, Pt, collect, dist, glyph, pt};
use crate::Forms;
use language_utils::StrokeStandard;
use unicode_normalization::UnicodeNormalization;

pub(super) const ASC: i32 = 150;
pub(super) const CAP: i32 = 190;
pub(super) const X: i32 = 450;
pub(super) const BASE: i32 = 850;
pub(super) const DESC: i32 = 975;
pub(super) const CAP_MID: i32 = (CAP + BASE) / 2; // 520
pub(super) const X_MID: i32 = (X + BASE) / 2; // 650

/// The accented letters of the course languages, then the ones their texts
/// borrow from other Latin orthographies and transliterations (Polish,
/// Czech, Romanian, Hungarian, Lithuanian, Turkish, Maltese, romanised
/// Japanese and Indic): every letter whose decomposition is a drawn base and
/// drawn marks.
const ACCENTED: &str = "àáâäãåçèéêëìíîïñòóôöõùúûüÿÀÁÂÄÃÅÇÈÉÊËÌÍÎÏÑÒÓÔÖÕÙÚÛÜŸ\
āēīōūĀĒĪŌŪăĕğĭŏŭĂĔĞĬŎŬċėġżĊĖĠŻąęįųĄĘĮŲčďěňřšťžľČĎĚŇŘŠŤŽĽőűŐŰćńśźĆŃŚŹ\
ḍḥḷṃṇṛṣṭẓḌḤḶṂṆṚṢṬẒșțȘȚģķļņĢĶĻŅýỳŷÝỲŶỹỸẽẼĩĨũŨ";

pub fn glyphs() -> Forms {
    let bases = BASES
        .iter()
        .filter(|(c, _)| *c != 'ı')
        .map(|&(c, draw)| (c, draw()));
    let accented = ACCENTED
        .chars()
        .flat_map(|c| compose(c).into_iter().map(move |form| (c, form)));
    collect(bases.chain(accented).map(|(c, strokes)| {
        let strokes = strokes.into_iter().map(|pen| pen.0).collect();
        (c, glyph(StrokeStandard::Latin, strokes, |p| p))
    }))
}

/// One pen-down motion. Each method continues from the current point.
pub(super) struct Pen(pub(super) Vec<Pt>);

type Draw = fn() -> Vec<Pen>;

impl Pen {
    pub(super) fn new() -> Self {
        Pen(Vec::new())
    }

    pub(super) fn at(start: (impl Coord, impl Coord)) -> Self {
        Pen(vec![pt(start)])
    }

    pub(super) fn line<A: Coord, B: Coord, const N: usize>(mut self, pts: [(A, B); N]) -> Self {
        self.0.extend(pts.map(pt));
        self
    }

    /// Elliptical arc; draws a straight join from the current point to the
    /// arc's start if they differ (e.g. the push-up retrace in b, h, n).
    pub(super) fn arc(
        mut self,
        cx: impl Coord,
        cy: impl Coord,
        rx: impl Coord,
        ry: impl Coord,
        a0: impl Coord,
        a1: impl Coord,
    ) -> Self {
        let (a0, a1) = (a0.f64(), a1.f64());
        let steps = ((a1 - a0).abs() / 10.0).ceil().max(4.0) as usize;
        let arc = super::arc(pt((cx, cy)), rx.f64(), -ry.f64(), a0, a1, steps);
        for (i, p) in arc.into_iter().enumerate() {
            if i > 0 || self.0.last().is_none_or(|&last| dist(p, last) > 1.0) {
                self.0.push(p);
            }
        }
        self
    }

    /// Catmull-Rom spline from the current point through `pts`.
    pub(super) fn curve<A: Coord, B: Coord, const N: usize>(mut self, pts: [(A, B); N]) -> Self {
        let start = *self.0.last().unwrap();
        let ctrl: Vec<Pt> = [start, start]
            .into_iter()
            .chain(pts.map(pt))
            .chain(std::iter::once(pt(pts[N - 1])))
            .collect();
        for w in ctrl.windows(4) {
            for s in 1..=6 {
                let t = f64::from(s) / 6.0;
                let f = |a: f64, b: f64, c: f64, d: f64| {
                    0.5 * (2.0 * b
                        + (c - a) * t
                        + (2.0 * a - 5.0 * b + 4.0 * c - d) * t * t
                        + (3.0 * b - a - 3.0 * c + d) * t.powf(3.0))
                };
                self.0.push((
                    f(w[0].0, w[1].0, w[2].0, w[3].0),
                    f(w[0].1, w[1].1, w[2].1, w[3].1),
                ));
            }
        }
        self
    }
}

pub(super) fn line<A: Coord, B: Coord, const N: usize>(pts: [(A, B); N]) -> Pen {
    Pen::new().line(pts)
}

/// A dot is a tap: a stroke too short to see as a line.
pub(super) fn dot(x: impl Coord, y: impl Coord) -> Pen {
    let (x, y) = pt((x, y));
    line([(x, y - 6.0), (x, y + 6.0)])
}

/// The circle of a/d/g/q, ending at 3 o'clock where the stick is drawn.
fn ball(cx: i32, rx: i32) -> Pen {
    Pen::new().arc(cx, X_MID, rx, 200, 30, 360)
}

/// An arch from a stem at x=left over the x-height and down to x=right.
fn hump(pen: Pen, left: i32, right: i32) -> Pen {
    let r = f64::from(right - left) / 2.0;
    pen.arc(f64::from(left) + r, f64::from(X) + r, r, r, 180, 0)
        .line([(right, BASE)])
}

/// The entry stroke of the cursive-derived forms: from low on the left up to
/// the top of a stem at `(x, top)`.
fn entry(x: i32, top: i32) -> Pen {
    Pen::at((x - 130, top + 190)).curve([(x - 55, top + 80), (x, top)])
}

/// The rounded arch of the cursive-derived h, m and n: from the foot of the
/// stem at `left` back up it, over, and down onto a leg at `right`, ending
/// part-way down the leg.
fn arch(pen: Pen, left: i32, right: i32) -> Pen {
    pen.curve([
        (left + 5, 640),
        (left + 60, 490),
        ((left + right) / 2, 455),
        (right - 45, 490),
        (right, 580),
        (right, 660),
    ])
}

/// Down a stem at `x` and out along the baseline in the exit tail.
fn tail(pen: Pen, x: i32) -> Pen {
    pen.line([(x, BASE - 70)])
        .curve([(x + 25, BASE - 12), (x + 80, BASE), (x + 140, BASE - 40)])
}

/// A looped ascender in one stroke: up from the lower left across the stem
/// line, up the right side of a narrow loop, over the top and down its left
/// side, which is the stem at `x`. Ends on the stem at x-height.
fn looped_ascender(x: i32) -> Pen {
    Pen::at((x - 90, 820)).curve([
        (x + 5, 650),
        (x + 55, 450),
        (x + 65, 280),
        (x + 35, ASC + 5),
        (x - 5, 200),
        (x - 10, 330),
        (x, X),
    ])
}

/// A looped descender closing on the stem at `x` near the baseline: down the
/// stem, round to the left and back up, then out to the right.
fn looped_descender(pen: Pen, x: i32) -> Pen {
    pen.line([(x, 915)]).curve([
        (x - 25, 966),
        (x - 80, 978),
        (x - 130, 950),
        (x - 135, 885),
        (x - 85, 830),
        (x, 805),
        (x + 70, 790),
        (x + 130, 760),
    ])
}

/// The "Upper-2" capital: the stem drawn down and back up the same line, then
/// on into the rest of the letter without lifting. `base(c)`'s second stroke
/// must start at the top of its stem.
fn retraced(c: char) -> Vec<Pen> {
    let mut strokes = base(c).into_iter();
    let (stem, rest) = (strokes.next().unwrap(), strokes.next().unwrap());
    assert_eq!(
        stem.0[0], rest.0[0],
        "{c}: the stroke after the stem starts elsewhere"
    );
    let back = stem.0.iter().rev().skip(1).copied();
    let merged = stem
        .0
        .iter()
        .copied()
        .chain(back)
        .chain(rest.0.into_iter().skip(1));
    std::iter::once(Pen(merged.collect()))
        .chain(strokes)
        .collect()
}

/// ª and º: a small raised letter over an underline.
fn ordinal(c: char) -> Vec<Pen> {
    let small = base(c).into_iter().map(|pen| {
        Pen(pen
            .0
            .into_iter()
            .map(|(x, y)| (500.0 + (x - 490.0) * 0.5, 170.0 + (y - f64::from(X)) * 0.5))
            .collect())
    });
    small.chain([line([(395, 460), (605, 460)])]).collect()
}

/// Every accepted form of a letter in [`BASES`], the taught one first.
fn forms(c: char) -> impl Iterator<Item = Vec<Pen>> {
    BASES
        .iter()
        .filter(move |(b, _)| *b == c)
        .map(|(_, draw)| draw())
}

/// The taught form of a letter in [`BASES`].
fn base(c: char) -> Vec<Pen> {
    forms(c).next().unwrap()
}

/// Form `n` of a letter in [`BASES`].
fn form(c: char, n: usize) -> Vec<Pen> {
    forms(c).nth(n).unwrap()
}

/// Every letter drawn directly, in writing order. A letter listed again right
/// after itself is an accepted alternative form (see the module docs).
///
/// Capitals: stems first, bars top to bottom, round letters start at 2 o'clock
/// (O and Q at 12 o'clock, as taught for the capital circle).
///
/// Lowercase: a, d, g, q circle back from 2 o'clock, push up, pull down. b, h,
/// m, n, p, r pull down, push up (retracing the stem), then the curve.
///
/// Digits: capital height, one stroke unless the pen must lift.
const BASES: &[(char, Draw)] = &[
    ('A', || {
        vec![
            line([(500, CAP), (270, BASE)]),
            line([(500, CAP), (730, BASE)]),
            line([(345, 640), (655, 640)]),
        ]
    }),
    // One stroke from the lower left: up, down the right leg, back up it to
    // the bar and across (Lund "Upper-2" family; Écriture A, Cuadernos Rubio).
    ('A', || {
        vec![line([
            (270, BASE),
            (500, CAP),
            (730, BASE),
            (657, 640),
            (343, 640),
        ])]
    }),
    ('B', || {
        let bumps = Pen::at((300, CAP))
            .line([(480, CAP)])
            .arc(480, 355, 150, 165, 90, -90)
            .line([(300, CAP_MID), (510, CAP_MID)])
            .arc(510, 685, 170, 165, 90, -90)
            .line([(300, BASE)]);
        vec![line([(300, CAP), (300, BASE)]), bumps]
    }),
    // The stem down and back up, then the bumps (Upper-2; Écriture A, Rubio).
    ('B', || retraced('B')),
    ('C', || {
        vec![Pen::new().arc(520, CAP_MID, 250, 330, 45, 318)]
    }),
    ('D', || {
        vec![
            line([(300, CAP), (300, BASE)]),
            Pen::at((300, CAP))
                .line([(420, CAP)])
                .arc(420, CAP_MID, 280, 330, 90, -90)
                .line([(300, BASE)]),
        ]
    }),
    // The stem down and back up, then the bowl (Upper-2; Écriture A, Rubio).
    ('D', || retraced('D')),
    ('E', || {
        vec![
            line([(310, CAP), (310, BASE)]),
            line([(310, CAP), (690, CAP)]),
            line([(310, CAP_MID), (640, CAP_MID)]),
            line([(310, BASE), (690, BASE)]),
        ]
    }),
    // The stem down and back up into the top bar (Upper-2; Écriture A, Rubio).
    ('E', || retraced('E')),
    ('F', || {
        vec![
            line([(320, CAP), (320, BASE)]),
            line([(320, CAP), (690, CAP)]),
            line([(320, CAP_MID), (640, CAP_MID)]),
        ]
    }),
    // The stem down and back up into the top bar (Upper-2; Écriture A, Rubio).
    ('F', || retraced('F')),
    ('G', || {
        vec![
            Pen::new()
                .arc(520, CAP_MID, 250, 330, 45, 335)
                .line([(748, 560), (560, 560)]),
        ]
    }),
    ('H', || {
        vec![
            line([(290, CAP), (290, BASE)]),
            line([(710, CAP), (710, BASE)]),
            line([(290, CAP_MID), (710, CAP_MID)]),
        ]
    }),
    // The left stem down and back up to the bar, across, then the right stem
    // (Upper-2; Écriture A, Rubio).
    ('H', || {
        vec![
            line([(290, CAP), (290, BASE), (290, CAP_MID), (710, CAP_MID)]),
            line([(710, CAP), (710, BASE)]),
        ]
    }),
    ('I', || {
        vec![
            line([(500, CAP), (500, BASE)]),
            line([(370, CAP), (630, CAP)]),
            line([(370, BASE), (630, BASE)]),
        ]
    }),
    ('I', || vec![line([(500, CAP), (500, BASE)])]),
    // One stroke: a hook into the top of the stem, down, and a curl to the
    // left at the foot (Italica, Porto Editora).
    ('I', || {
        vec![Pen::at((330, 300)).curve([
            (360, 215),
            (450, CAP),
            (510, 245),
            (510, 500),
            (500, 760),
            (460, 840),
            (380, 850),
            (330, 800),
        ])]
    }),
    ('J', || {
        vec![
            Pen::at((640, CAP))
                .line([(640, 680)])
                .arc(470, 680, 170, 170, 0, -175),
        ]
    }),
    ('J', || {
        let mut j = base('J');
        j.push(line([(480, CAP), (780, CAP)]));
        j
    }),
    // One stroke: a hook into the top bar, then down the stem into the hook
    // at the foot (Italica, Porto Editora).
    ('J', || {
        vec![
            Pen::at((400, 280))
                .curve([(410, 215), (470, CAP)])
                .line([(640, CAP), (640, 680)])
                .arc(470, 680, 170, 170, 0, -175),
        ]
    }),
    ('K', || {
        vec![
            line([(300, CAP), (300, BASE)]),
            line([(700, CAP), (305, 570), (710, BASE)]),
        ]
    }),
    // Stem, arm, then the leg from the middle of the arm.
    ('K', || {
        vec![
            line([(300, CAP), (300, BASE)]),
            line([(700, CAP), (305, 570)]),
            line([(440, 440), (710, BASE)]),
        ]
    }),
    // The stem down and back up to the joint, the leg, then the arm (Upper-2;
    // Écriture A, Rubio).
    ('K', || {
        vec![
            line([(300, CAP), (300, BASE), (300, 570), (710, BASE)]),
            line([(700, CAP), (305, 570)]),
        ]
    }),
    ('L', || vec![line([(310, CAP), (310, BASE), (680, BASE)])]),
    // One stroke: the stem down into a small loop at the corner, then the
    // foot (Écriture A, Méthode Dumont).
    ('L', || {
        vec![Pen::at((330, CAP)).line([(330, 700)]).curve([
            (315, 790),
            (265, 848),
            (205, 835),
            (205, 780),
            (275, 770),
            (360, 820),
            (460, 850),
            (570, 850),
            (690, 850),
        ])]
    }),
    ('M', || {
        vec![
            line([(240, CAP), (240, BASE)]),
            line([(240, CAP), (500, BASE), (760, CAP), (760, BASE)]),
        ]
    }),
    ('M', || {
        vec![
            line([(240, CAP), (240, BASE)]),
            line([(240, CAP), (500, BASE), (760, CAP)]),
            line([(760, CAP), (760, BASE)]),
        ]
    }),
    // One zigzag, up from the foot of the left stem.
    ('M', || {
        vec![line([
            (240, BASE),
            (240, CAP),
            (500, BASE),
            (760, CAP),
            (760, BASE),
        ])]
    }),
    // The left stem down and back up, then on through the V and right stem
    // (Upper-2; Écriture A, Rubio).
    ('M', || retraced('M')),
    ('N', || {
        vec![
            line([(280, CAP), (280, BASE)]),
            line([(280, CAP), (720, BASE), (720, CAP)]),
        ]
    }),
    ('N', || {
        vec![
            line([(280, CAP), (280, BASE)]),
            line([(280, CAP), (720, BASE)]),
            line([(720, CAP), (720, BASE)]),
        ]
    }),
    // One zigzag, up from the foot of the left stem.
    ('N', || {
        vec![line([(280, BASE), (280, CAP), (720, BASE), (720, CAP)])]
    }),
    // The left stem down and back up, then the diagonal and right stem
    // (Upper-2; Écriture A, Rubio).
    ('N', || retraced('N')),
    ('O', || {
        vec![Pen::new().arc(500, CAP_MID, 280, 330, 90, 450)]
    }),
    ('P', || {
        vec![
            line([(310, CAP), (310, BASE)]),
            Pen::at((310, CAP))
                .line([(470, CAP)])
                .arc(470, 370, 210, 180, 90, -90)
                .line([(310, 550)]),
        ]
    }),
    // The stem down and back up, then the bowl (Upper-2; Écriture A, Rubio).
    ('P', || retraced('P')),
    ('Q', || {
        vec![
            Pen::new().arc(500, CAP_MID, 280, 330, 90, 450),
            line([(590, 720), (770, 900)]),
        ]
    }),
    // A tail hanging below the circle.
    ('Q', || {
        vec![
            Pen::new().arc(500, CAP_MID, 280, 330, 90, 450),
            Pen::at((470, 820)).curve([(560, 900), (660, 945), (770, 945)]),
        ]
    }),
    ('R', || {
        let bowl = Pen::at((310, CAP))
            .line([(470, CAP)])
            .arc(470, 370, 210, 180, 90, -90)
            .line([(310, 550), (710, BASE)]);
        vec![line([(310, CAP), (310, BASE)]), bowl]
    }),
    // The stem down and back up, then the bowl and leg (Upper-2; Écriture A,
    // Rubio).
    ('R', || retraced('R')),
    ('S', || {
        vec![Pen::at((700, 280)).curve([
            (600, 200),
            (470, 195),
            (350, 250),
            (330, 360),
            (420, 455),
            (560, 510),
            (670, 590),
            (690, 720),
            (610, 825),
            (470, 855),
            (350, 820),
            (295, 760),
        ])]
    }),
    // Narrow S with a curl at each end, one stroke (Porto Editora, Écriture
    // A).
    ('S', || {
        vec![Pen::at((560, 300)).curve([
            (625, 240),
            (590, CAP),
            (490, 200),
            (410, 255),
            (405, 355),
            (480, 445),
            (590, 525),
            (645, 645),
            (615, 775),
            (515, 850),
            (395, 845),
            (330, 790),
            (345, 735),
            (410, 745),
        ])]
    }),
    ('T', || {
        vec![
            line([(500, CAP), (500, BASE)]),
            line([(270, CAP), (730, CAP)]),
        ]
    }),
    // The top bar from a hook at its left end, then the stem with a curl to
    // the left at the foot (Italica).
    ('T', || {
        vec![
            Pen::at((300, 290)).curve([(280, 225), (330, CAP), (450, 212), (580, 200), (730, CAP)]),
            Pen::at((500, 205))
                .line([(500, 770)])
                .curve([(470, 840), (400, 850), (340, 810)]),
        ]
    }),
    ('U', || {
        vec![
            Pen::at((280, CAP))
                .line([(280, 630)])
                .arc(500, 630, 220, 220, 180, 360)
                .line([(720, CAP)]),
        ]
    }),
    ('V', || vec![line([(260, CAP), (500, BASE), (740, CAP)])]),
    ('W', || {
        vec![line([
            (160, CAP),
            (330, BASE),
            (500, CAP),
            (670, BASE),
            (840, CAP),
        ])]
    }),
    ('X', || {
        vec![
            line([(290, CAP), (710, BASE)]),
            line([(710, CAP), (290, BASE)]),
        ]
    }),
    ('Y', || {
        vec![
            line([(270, CAP), (500, 540)]),
            line([(730, CAP), (500, 540), (500, BASE)]),
        ]
    }),
    ('Z', || {
        vec![line([(290, CAP), (710, CAP), (290, BASE), (710, BASE)])]
    }),
    ('Z', || {
        let mut z = base('Z');
        z.push(line([(380, CAP_MID), (620, CAP_MID)]));
        z
    }),
    // One stroke, the top bar and diagonal running into a small loop at the
    // bottom left before the foot (Écriture A, France).
    ('Z', || {
        vec![Pen::at((290, CAP)).line([(710, CAP), (360, 760)]).curve([
            (315, 818),
            (265, 850),
            (220, 830),
            (225, 785),
            (290, 780),
            (370, 825),
            (470, 850),
            (580, 850),
            (710, 845),
        ])]
    }),
    // The same with a crossbar (Écriture A, Italica, Cuadernos Rubio).
    ('Z', || {
        let mut z = form('Z', 2);
        z.push(line([(380, CAP_MID), (620, CAP_MID)]));
        z
    }),
    ('a', || vec![ball(480, 170).line([(650, X), (650, BASE)])]),
    // Double-storey a, one stroke: the hook over the top, down the stem to
    // the bowl, round the bowl back to the stem, push up the stem and pull
    // down to the baseline.
    ('a', || {
        vec![
            Pen::new()
                .arc(500, 575, 145, 120, 150, 0)
                .line([(645, 635)])
                .curve([
                    (520, 625),
                    (395, 650),
                    (345, 740),
                    (385, 825),
                    (495, 852),
                    (595, 832),
                    (645, 790),
                ])
                .line([(645, 635), (645, BASE)]),
        ]
    }),
    // Cursive-derived, one stroke: the flat top of the bowl drawn leftwards,
    // round it, up to the top and down the stem into the exit tail (a handwriting
    // sample; Écriture A, Cuadernos Rubio).
    ('a', || {
        let bowl = Pen::at((630, 460)).curve([
            (530, 452),
            (420, 465),
            (345, 540),
            (330, 665),
            (370, 790),
            (465, 850),
            (570, 835),
            (635, 765),
            (655, 620),
            (655, X),
        ]);
        vec![tail(bowl, 655)]
    }),
    // The same with the stem drawn as a small loop at the top, crossing back
    // down into the tail (a handwriting sample).
    ('a', || {
        vec![Pen::at((600, 458)).curve([
            (500, 452),
            (400, 470),
            (340, 550),
            (330, 670),
            (370, 790),
            (460, 848),
            (560, 835),
            (630, 770),
            (650, 650),
            (650, 530),
            (670, 462),
            (712, 470),
            (722, 530),
            (700, 625),
            (648, 745),
            (672, 820),
            (740, 850),
            (810, 830),
        ])]
    }),
    ('b', || {
        vec![
            Pen::at((330, ASC))
                .line([(330, BASE)])
                .arc(500, X_MID, 170, 200, 165, -165),
        ]
    }),
    // Looped ascender, one stroke: up the loop, down the stem, round the bowl
    // back to the stem (a handwriting sample; Écriture A, Rubio).
    ('b', || {
        vec![looped_ascender(340).line([(340, 770)]).curve([
            (380, 835),
            (480, 852),
            (590, 815),
            (645, 710),
            (620, 580),
            (530, 500),
            (420, 510),
            (345, 570),
        ])]
    }),
    ('c', || vec![Pen::new().arc(510, X_MID, 170, 200, 45, 318)]),
    // A hooked start at the top, round, and out in the exit tail (Rubio,
    // Porto Editora).
    ('c', || {
        vec![Pen::at((565, 545)).curve([
            (615, 530),
            (630, 475),
            (590, 452),
            (530, 452),
            (420, 470),
            (350, 550),
            (335, 680),
            (380, 800),
            (480, 850),
            (600, 840),
            (700, 790),
        ])]
    }),
    ('d', || vec![ball(480, 170).line([(650, ASC), (650, BASE)])]),
    // The bar sits above the centre so the eye is the smaller upper part, and
    // the sweep stops short of the bar's height so the tail stays open.
    // One stroke: the bowl from its flat top, up into a tall narrow loop and
    // down it into the exit tail (a handwriting sample; Écriture A, Rubio).
    ('d', || {
        vec![Pen::at((630, 460)).curve([
            (520, 452),
            (410, 465),
            (345, 540),
            (330, 670),
            (375, 800),
            (470, 850),
            (570, 830),
            (630, 770),
            (640, 640),
            (625, 450),
            (610, 280),
            (625, 165),
            (655, 155),
            (680, 240),
            (680, 450),
            (662, 650),
            (648, 765),
            (690, 835),
            (760, 850),
            (820, 820),
        ])]
    }),
    ('e', || {
        vec![
            Pen::at((355, 610))
                .line([(677, 610)])
                .arc(510, X_MID, 170, 200, 11.5, 300),
        ]
    }),
    // The cursive loop e: up from the lower left, over the loop and round
    // into the exit tail (Écriture A, Rubio, Porto Editora).
    ('e', || {
        vec![Pen::at((330, 700)).curve([
            (460, 665),
            (590, 600),
            (635, 520),
            (580, 458),
            (480, 462),
            (400, 530),
            (365, 650),
            (395, 780),
            (490, 848),
            (610, 845),
            (710, 790),
        ])]
    }),
    ('f', || {
        vec![
            Pen::new()
                .arc(540, 270, 120, 120, 30, 180)
                .line([(420, BASE)]),
            line([(320, X), (620, X)]),
        ]
    }),
    // One stroke: looped ascender, the stem down into a loop below the
    // baseline that closes on the stem and leaves to the right (Rubio,
    // Écriture A).
    ('f', || vec![looped_descender(looped_ascender(445), 445)]),
    // Looped ascender and a straight stem to the descender, then the
    // crossbar (Italica).
    ('f', || {
        vec![
            looped_ascender(445).line([(445, DESC)]),
            line([(330, X), (600, X)]),
        ]
    }),
    ('g', || {
        vec![
            ball(480, 170)
                .line([(650, X), (650, 880)])
                .arc(495, 880, 155, 95, 0, -170),
        ]
    }),
    // Double-storey g: the bowl, then from its foot the stem down into the
    // lower loop, round and closed.
    ('g', || {
        vec![
            Pen::new().arc(490, 575, 140, 125, 30, 390),
            Pen::at((450, 700)).line([(410, 760)]).curve([
                (560, 760),
                (660, 800),
                (680, 880),
                (600, 955),
                (460, 965),
                (340, 925),
                (330, 850),
                (410, 760),
            ]),
        ]
    }),
    // The ball and stem, the stem running into a loop below the baseline
    // (Écriture A, Rubio, Porto Editora).
    ('g', || {
        vec![looped_descender(ball(480, 170).line([(650, X)]), 650)]
    }),
    ('h', || {
        vec![hump(Pen::at((330, ASC)).line([(330, BASE)]), 330, 670)]
    }),
    // Dotless i: the base of i and of ì í î ï.
    // Looped ascender down to the baseline, back up into the arch and out in
    // the exit tail (Écriture A, Rubio).
    ('h', || {
        vec![tail(
            arch(looped_ascender(330).line([(330, BASE)]), 330, 650),
            650,
        )]
    }),
    ('ı', || vec![line([(500, X), (500, BASE)])]),
    // The entry stroke, down and out in the exit tail (Rubio, Écriture A).
    ('ı', || vec![tail(entry(500, X), 500)]),
    ('i', || {
        base('ı').into_iter().chain([dot(500, 335)]).collect()
    }),
    ('i', || {
        form('ı', 1).into_iter().chain([dot(500, 335)]).collect()
    }),
    ('j', || {
        vec![
            Pen::at((560, X))
                .line([(560, 880)])
                .arc(410, 880, 150, 95, 0, -170),
            dot(560, 335),
        ]
    }),
    // The entry stroke, down into a loop below the baseline, then the dot
    // (Écriture A, Rubio).
    ('j', || {
        vec![looped_descender(entry(520, X), 520), dot(520, 335)]
    }),
    ('k', || {
        vec![
            line([(330, ASC), (330, BASE)]),
            line([(640, X), (335, 670), (660, BASE)]),
        ]
    }),
    // Stem, arm, then the leg from the middle of the arm.
    ('k', || {
        vec![
            line([(330, ASC), (330, BASE)]),
            line([(640, X), (335, 670)]),
            line([(470, 585), (660, BASE)]),
        ]
    }),
    // One stroke: looped ascender to the baseline, back up into a small knot
    // at the waist and out along the leg (Écriture A, Méthode Dumont).
    ('k', || {
        vec![
            looped_ascender(330)
                .line([(330, BASE)])
                .curve([
                    (335, 660),
                    (410, 510),
                    (510, 460),
                    (590, 500),
                    (580, 590),
                    (480, 650),
                    (380, 665),
                ])
                .curve([(470, 690), (560, 790), (620, 845), (690, 850), (750, 815)]),
        ]
    }),
    ('l', || vec![line([(500, ASC), (500, BASE)])]),
    // Looped ascender and exit tail, one stroke (Écriture A, Rubio).
    ('l', || vec![tail(looped_ascender(470), 470)]),
    ('m', || {
        let pen = hump(Pen::at((250, X)).line([(250, BASE)]), 250, 500);
        vec![hump(pen, 500, 750)]
    }),
    // The entry stroke, the stem, two rounded arches and the exit tail
    // (Écriture A, Italica).
    ('m', || {
        let first = arch(entry(250, X).line([(250, BASE)]), 250, 500);
        vec![tail(arch(first.line([(500, BASE)]), 500, 745), 745)]
    }),
    ('n', || {
        vec![hump(Pen::at((330, X)).line([(330, BASE)]), 330, 670)]
    }),
    // The entry stroke, the stem, a rounded arch and the exit tail (Écriture
    // A, Italica).
    ('n', || {
        vec![tail(arch(entry(340, X).line([(340, BASE)]), 340, 655), 655)]
    }),
    ('o', || vec![Pen::new().arc(500, X_MID, 180, 200, 30, 390)]),
    // Round from the top, closed with a small knot and out to the right at
    // the top (Rubio, Porto Editora).
    ('o', || {
        vec![Pen::at((580, 462)).curve([
            (480, 452),
            (380, 500),
            (335, 630),
            (370, 780),
            (480, 850),
            (600, 815),
            (655, 690),
            (635, 550),
            (580, 470),
            (540, 482),
            (580, 510),
            (670, 500),
            (750, 462),
        ])]
    }),
    ('p', || {
        vec![
            Pen::at((330, X))
                .line([(330, DESC)])
                .arc(500, X_MID, 170, 200, 165, -165),
        ]
    }),
    // French p: the stem starts half-way up the ascender and runs to the
    // descender, back up into an arch whose bowl stays open at the bottom
    // (Écriture A, Méthode Dumont; Wikipedia).
    ('p', || {
        vec![entry(360, 300).line([(360, DESC), (360, 580)]).curve([
            (420, 480),
            (520, 455),
            (620, 500),
            (655, 620),
            (630, 760),
            (560, 840),
            (470, 850),
            (405, 820),
        ])]
    }),
    ('q', || vec![ball(480, 170).line([(650, X), (650, DESC)])]),
    // Crossed descender (block letters across Europe; Wikipedia).
    ('q', || {
        let mut q = base('q');
        q.push(line([(570, 915), (730, 915)]));
        q
    }),
    // The stem ends in a loop to the right that closes near the baseline and
    // leaves to the right (Écriture A, Méthode Dumont).
    ('q', || {
        vec![ball(480, 170).line([(650, X), (650, 900)]).curve([
            (670, 960),
            (715, 975),
            (745, 940),
            (725, 885),
            (670, 850),
            (740, 830),
            (810, 800),
        ])]
    }),
    ('r', || {
        vec![
            Pen::at((370, X))
                .line([(370, BASE)])
                .arc(520, X + 150, 150, 150, 180, 40),
        ]
    }),
    // Cursive r: up from the lower left, a small shoulder across the top,
    // down and out in the exit tail (Écriture A, Porto Editora).
    ('r', || {
        vec![Pen::at((300, 720)).curve([
            (370, 570),
            (425, 430),
            (470, 462),
            (520, 470),
            (575, 440),
            (580, 560),
            (575, 720),
            (605, 830),
            (675, 850),
            (740, 815),
        ])]
    }),
    // The print r with the entry stroke (Rubio, Italica).
    ('r', || {
        vec![
            entry(370, X)
                .line([(370, BASE)])
                .arc(520, X + 150, 150, 150, 180, 40),
        ]
    }),
    ('s', || {
        vec![Pen::at((655, 510)).curve([
            (590, 455),
            (490, 450),
            (385, 480),
            (360, 555),
            (430, 630),
            (560, 670),
            (645, 730),
            (635, 810),
            (540, 852),
            (430, 850),
            (340, 800),
        ])]
    }),
    // Cursive s: up from the lower left to a point, down round the belly and
    // back to the left, then out along the baseline (Écriture A, Rubio).
    ('s', || {
        vec![Pen::at((290, 790)).curve([
            (400, 650),
            (480, 450),
            (540, 520),
            (620, 630),
            (640, 750),
            (580, 835),
            (480, 852),
            (390, 825),
            (370, 780),
            (420, 800),
            (530, 845),
            (650, 830),
            (720, 790),
        ])]
    }),
    ('t', || {
        vec![line([(490, 230), (490, BASE)]), line([(370, X), (620, X)])]
    }),
    // The stem curves right at the foot, then the crossbar.
    ('t', || {
        vec![
            Pen::at((490, 230))
                .line([(490, 770)])
                .arc(570, 770, 80, 80, 180, 300),
            line([(370, X), (620, X)]),
        ]
    }),
    // The entry stroke up to the top, down the stem into the exit tail, then
    // the crossbar (Écriture A, Rubio).
    ('t', || {
        vec![tail(entry(490, 230), 490), line([(370, X), (620, X)])]
    }),
    ('u', || {
        vec![
            Pen::at((330, X))
                .line([(330, 680)])
                .arc(500, 680, 170, 170, 180, 360)
                .line([(670, X), (670, BASE)]),
        ]
    }),
    // The entry stroke, round the bottom, up and down the right stem into the
    // exit tail (Écriture A, Rubio).
    ('u', || {
        let cup = entry(340, X).line([(340, 700)]).curve([
            (385, 815),
            (490, 850),
            (590, 810),
            (645, 700),
            (655, X),
        ]);
        vec![tail(cup, 655)]
    }),
    ('v', || vec![line([(320, X), (500, BASE), (680, X)])]),
    // Rounded at the bottom, from an entry hook to a small knot and exit at
    // the top (Rubio, Porto Editora).
    ('v', || {
        vec![Pen::at((260, 560)).curve([
            (320, X),
            (370, 580),
            (440, 800),
            (500, 850),
            (560, 800),
            (630, 590),
            (660, X),
            (635, 485),
            (665, 505),
            (770, 470),
        ])]
    }),
    ('w', || {
        vec![line([
            (210, X),
            (355, BASE),
            (500, X),
            (645, BASE),
            (790, X),
        ])]
    }),
    // The entry stroke, two rounded valleys, and a knot and exit at the top
    // (Rubio, Italica).
    ('w', || {
        vec![
            entry(230, X)
                .curve([
                    (240, 650),
                    (290, 810),
                    (360, 850),
                    (440, 810),
                    (490, 650),
                    (500, X),
                ])
                .curve([
                    (505, 650),
                    (555, 810),
                    (630, 850),
                    (710, 810),
                    (760, 640),
                    (770, X),
                    (745, 490),
                    (780, 510),
                    (860, 470),
                ]),
        ]
    }),
    ('x', || {
        vec![line([(330, X), (670, BASE)]), line([(670, X), (330, BASE)])]
    }),
    // Two arcs back to back, crossing at the waist (Italica, Écriture A).
    ('x', || {
        vec![
            Pen::at((330, 480)).curve([
                (430, 452),
                (510, 530),
                (525, 650),
                (500, 770),
                (430, 845),
                (330, 830),
            ]),
            Pen::at((690, 480)).curve([
                (590, 452),
                (505, 530),
                (490, 650),
                (515, 770),
                (590, 845),
                (690, 830),
            ]),
        ]
    }),
    ('y', || {
        vec![line([(320, X), (500, BASE)]), line([(680, X), (445, DESC)])]
    }),
    // Curved-tail y, one stroke: a u whose right stem runs on into g's tail.
    ('y', || {
        vec![
            Pen::at((330, X))
                .line([(330, 680)])
                .arc(500, 680, 170, 170, 180, 360)
                .line([(670, X), (670, 880)])
                .arc(515, 880, 155, 95, 0, -170),
        ]
    }),
    // The entry stroke, round the bottom, up the right stem and down into a
    // loop below the baseline (Écriture A, Rubio).
    ('y', || {
        let cup = entry(340, X).line([(340, 700)]).curve([
            (385, 815),
            (490, 850),
            (590, 810),
            (645, 700),
            (655, X),
        ]);
        vec![looped_descender(cup, 655)]
    }),
    ('z', || {
        vec![line([(330, X), (670, X), (330, BASE), (670, BASE)])]
    }),
    ('z', || {
        let mut z = base('z');
        z.push(line([(410, X_MID), (590, X_MID)]));
        z
    }),
    // Letters without a decomposition.
    // Cursive z as ʒ: over the top into a knot at the waist, then the tail
    // curling below the baseline (Porto Editora, Écriture A).
    ('z', || {
        vec![
            Pen::at((340, 480))
                .curve([(400, 452), (520, 460), (630, 455)])
                .line([(450, 680)])
                .curve([
                    (550, 685),
                    (625, 750),
                    (630, 850),
                    (580, 940),
                    (490, 978),
                    (410, 955),
                    (385, 905),
                ]),
        ]
    }),
    // Italic z: across the top, down the diagonal and out in a curved foot
    // (Cuadernos Rubio).
    ('z', || {
        vec![
            Pen::at((340, 470))
                .curve([(400, 450), (500, 460), (560, 455), (640, 450)])
                .line([(360, 830)])
                .curve([(430, 810), (520, 845), (600, 850), (670, 820)]),
        ]
    }),
    // Italic z with a crossbar (Italica).
    ('z', || {
        let mut z = form('z', 3);
        z.push(line([(410, X_MID), (590, X_MID)]));
        z
    }),
    ('Æ', || {
        // The middle bar is the A's crossbar and the E's middle bar in one
        // stroke, starting on the left diagonal.
        let bar = f64::from(CAP_MID + 60);
        let bar_start = 560.0 - 330.0 * (bar - f64::from(CAP)) / f64::from(BASE - CAP);
        vec![
            line([(560, CAP), (230, BASE)]),
            line([(560, CAP), (560, BASE)]),
            line([(560, CAP), (800, CAP)]),
            line([(bar_start, bar), (760.0, bar)]),
            line([(560, BASE), (800, BASE)]),
        ]
    }),
    ('æ', || {
        let a = ball(335, 135).line([(470, X), (470, BASE)]);
        let e = Pen::at((470, X_MID))
            .line([(790, X_MID)])
            .arc(630, X_MID, 160, 200, 0, 318);
        vec![a, e]
    }),
    ('Œ', || {
        vec![
            Pen::at((550, CAP))
                .line([(440, CAP)])
                .arc(440, CAP_MID, 250, 330, 90, 270)
                .line([(550, BASE)]),
            line([(550, CAP), (550, BASE)]),
            line([(550, CAP), (800, CAP)]),
            line([(550, CAP_MID), (760, CAP_MID)]),
            line([(550, BASE), (800, BASE)]),
        ]
    }),
    ('œ', || {
        let o = Pen::new().arc(340, X_MID, 140, 200, 30, 390);
        let e = Pen::at((480, X_MID))
            .line([(800, X_MID)])
            .arc(640, X_MID, 160, 200, 0, 318);
        vec![o, e]
    }),
    ('ß', || {
        // Like B: the stem, then from its top the arch and two bumps.
        let bumps = Pen::at((330, 310)).arc(465, 310, 135, 160, 180, 0).curve([
            (570, 410),
            (490, 470),
            (610, 510),
            (675, 620),
            (665, 760),
            (575, 845),
            (450, 845),
        ]);
        vec![line([(330, 310), (330, BASE)]), bumps]
    }),
    // One stroke from the baseline: up the stem, over, down through the bumps.
    ('ß', || {
        vec![
            Pen::at((330, BASE))
                .line([(330, 310)])
                .arc(465, 310, 135, 160, 180, 0)
                .curve([
                    (570, 410),
                    (490, 470),
                    (610, 510),
                    (675, 620),
                    (665, 760),
                    (575, 845),
                    (450, 845),
                ]),
        ]
    }),
    ('ẞ', || {
        let bowl = Pen::at((320, CAP)).line([(680, CAP), (490, 470)]).curve([
            (630, 500),
            (700, 610),
            (690, 750),
            (600, 840),
            (450, 850),
        ]);
        vec![line([(320, CAP), (320, BASE)]), bowl]
    }),
    ('ª', || ordinal('a')),
    ('º', || ordinal('o')),
    ('0', || {
        vec![Pen::new().arc(500, CAP_MID, 200, 330, 90, 450)]
    }),
    ('1', || vec![line([(390, 310), (540, CAP), (540, BASE)])]),
    ('1', || vec![line([(500, CAP), (500, BASE)])]),
    ('1', || {
        let mut one = base('1');
        one.push(line([(400, BASE), (680, BASE)]));
        one
    }),
    ('2', || {
        vec![
            Pen::at((330, 320))
                .curve([
                    (400, 220),
                    (510, 190),
                    (620, 230),
                    (660, 330),
                    (630, 440),
                    (540, 560),
                    (330, BASE),
                ])
                .line([(690, BASE)]),
        ]
    }),
    ('3', || {
        vec![
            Pen::at((330, 270))
                .curve([
                    (430, 198),
                    (560, 195),
                    (645, 260),
                    (645, 365),
                    (580, 450),
                    (460, 505),
                ])
                .curve([
                    (600, 530),
                    (680, 620),
                    (680, 750),
                    (590, 840),
                    (440, 855),
                    (320, 800),
                ]),
        ]
    }),
    ('4', || {
        vec![
            line([(590, CAP), (290, 650), (720, 650)]),
            line([(590, CAP), (590, BASE)]),
        ]
    }),
    // Open top: the down stroke and bar, then the stem apart from it.
    ('4', || {
        vec![
            line([(360, CAP), (300, 650), (720, 650)]),
            line([(600, CAP), (600, BASE)]),
        ]
    }),
    ('5', || {
        let bowl = Pen::at((365, CAP)).line([(345, 480)]).curve([
            (460, 440),
            (590, 450),
            (670, 540),
            (690, 670),
            (630, 800),
            (510, 855),
            (400, 845),
            (320, 790),
        ]);
        vec![bowl, line([(365, CAP), (670, CAP)])]
    }),
    ('6', || {
        vec![Pen::at((650, 240)).curve([
            (560, 192),
            (450, 205),
            (365, 300),
            (325, 460),
            (325, 640),
            (370, 790),
            (490, 855),
            (610, 820),
            (670, 710),
            (650, 590),
            (560, 520),
            (440, 525),
            (340, 610),
        ])]
    }),
    ('7', || vec![line([(310, CAP), (690, CAP), (430, BASE)])]),
    ('7', || {
        let mut seven = base('7');
        seven.push(line([(440, CAP_MID), (680, CAP_MID)]));
        seven
    }),
    ('8', || {
        vec![Pen::at((640, 280)).curve([
            (560, 198),
            (440, 198),
            (360, 265),
            (365, 375),
            (460, 450),
            (560, 505),
            (665, 600),
            (670, 755),
            (580, 845),
            (420, 845),
            (330, 755),
            (340, 610),
            (440, 505),
            (540, 450),
            (630, 375),
            (640, 280),
        ])]
    }),
    ('9', || {
        vec![
            Pen::new()
                .arc(495, 370, 180, 180, 30, 360)
                .line([(675, BASE)]),
        ]
    }),
];

// Diacritics, each drawn in a box above (or below) the letter. `x` is the
// letter's centre; `top`/`bottom` bound the mark vertically.

fn grave(x: f64, top: f64, bottom: f64) -> Vec<Pen> {
    vec![line([(x - 45.0, top), (x + 45.0, bottom)])]
}

fn acute(x: f64, top: f64, bottom: f64) -> Vec<Pen> {
    vec![line([(x + 45.0, top), (x - 45.0, bottom)])]
}

fn circumflex(x: f64, top: f64, bottom: f64) -> Vec<Pen> {
    vec![line([(x - 85.0, bottom), (x, top), (x + 85.0, bottom)])]
}

fn diaeresis(x: f64, top: f64, bottom: f64) -> Vec<Pen> {
    let y = (top + bottom) / 2.0;
    vec![dot(x - 80.0, y), dot(x + 80.0, y)]
}

fn tilde(x: f64, top: f64, bottom: f64) -> Vec<Pen> {
    let y = (top + bottom) / 2.0;
    let wave = (-100..=100).step_by(12).map(|t| {
        let t = f64::from(t);
        (x + t, y - 28.0 * (std::f64::consts::PI * t / 100.0).sin())
    });
    vec![Pen(wave.collect())]
}

fn ring(x: f64, top: f64, bottom: f64) -> Vec<Pen> {
    let r = (bottom - top) / 2.0;
    vec![Pen::new().arc(x, top + r, r * 0.9, r, 90, 450)]
}

fn macron(x: f64, top: f64, bottom: f64) -> Vec<Pen> {
    let y = (top + bottom) / 2.0;
    vec![line([(x - 90.0, y), (x + 90.0, y)])]
}

fn breve(x: f64, top: f64, bottom: f64) -> Vec<Pen> {
    let r = (bottom - top) / 2.0;
    vec![Pen::new().arc(x, top + r * 0.6, r * 1.4, r, 180, 360)]
}

fn caron(x: f64, top: f64, bottom: f64) -> Vec<Pen> {
    vec![line([(x - 85.0, top), (x, bottom), (x + 85.0, top)])]
}

fn double_acute(x: f64, top: f64, bottom: f64) -> Vec<Pen> {
    vec![
        line([(x + 5.0, top), (x - 85.0, bottom)]),
        line([(x + 105.0, top), (x + 15.0, bottom)]),
    ]
}

fn dot_above(x: f64, top: f64, bottom: f64) -> Vec<Pen> {
    vec![dot(x, (top + bottom) / 2.0)]
}

/// A tail curling left from the foot of the letter at `(x, top)`, as on ą ę.
fn ogonek(x: f64, top: f64) -> Vec<Pen> {
    vec![Pen::at((x, top)).arc(x + 40.0, top + 45.0, 40, 45, 180, 340)]
}

/// A short comma under the letter, as on ș ț.
fn comma_below(x: f64, top: f64) -> Vec<Pen> {
    vec![line([(x + 15.0, top + 40.0), (x - 15.0, top + 95.0)])]
}

/// Hangs from the bottom of the letter's curve at `(x, top)`.
fn cedilla(x: f64, top: f64) -> Vec<Pen> {
    vec![
        Pen::at((x, top))
            .line([(x, top + 30.0)])
            .arc(x + 5.0, top + 70.0, 42, 40, 90, -150),
    ]
}

/// Base letter first, then its marks, once for every form of the base letter.
fn compose(c: char) -> Vec<Vec<Pen>> {
    let mut chars = c.nfd();
    let letter = chars.next().unwrap();
    let marks: Vec<char> = chars.collect();
    // A mark above replaces the dot of i; a mark below (į) leaves it.
    let below = |m: &char| matches!(m, '\u{323}' | '\u{326}' | '\u{327}' | '\u{328}');
    let dotless = letter == 'i' && !marks.iter().all(below);
    forms(if dotless { 'ı' } else { letter })
        .map(|strokes| add_marks(c, letter, strokes, &marks))
        .collect()
}

fn add_marks(c: char, letter: char, mut strokes: Vec<Pen>, marks: &[char]) -> Vec<Pen> {
    let pts: Vec<Pt> = strokes.iter().flat_map(|pen| pen.0.clone()).collect();
    let xs = pts.iter().map(|p| p.0);
    let centre =
        (xs.clone().fold(f64::INFINITY, f64::min) + xs.fold(f64::NEG_INFINITY, f64::max)) / 2.0;
    // Marks above sit over the x-height, or over the ascender line for
    // capitals and for lowercase letters that reach above the x-height.
    let reaches_up = pts.iter().any(|p| p.1 < f64::from(X) - 60.0);
    let (top, bottom) = if letter.is_uppercase() || reaches_up {
        (40.0, 150.0)
    } else {
        (275.0, 385.0)
    };
    // The first lowest point of the letter, where marks below attach.
    let lowest = pts
        .iter()
        .fold(pts[0], |a, &p| if p.1 > a.1 { p } else { a });
    for &mark in marks {
        strokes.extend(match mark {
            '\u{300}' => grave(centre, top, bottom),
            '\u{301}' => acute(centre, top, bottom),
            '\u{302}' => circumflex(centre, top, bottom),
            '\u{303}' => tilde(centre, top, bottom),
            '\u{304}' => macron(centre, top, bottom),
            '\u{306}' => breve(centre, top, bottom),
            '\u{307}' => dot_above(centre, top, bottom),
            '\u{308}' => diaeresis(centre, top, bottom),
            '\u{30a}' => ring(centre, top, bottom),
            '\u{30b}' => double_acute(centre, top, bottom),
            '\u{30c}' => caron(centre, top, bottom),
            '\u{323}' => vec![dot(centre, f64::from(BASE) + 70.0)],
            '\u{326}' => comma_below(lowest.0, f64::from(BASE)),
            '\u{327}' => cedilla(lowest.0, f64::from(BASE)),
            '\u{328}' => ogonek(lowest.0, f64::from(BASE)),
            _ => panic!("no drawing for mark U+{:04X} in {c}", u32::from(mark)),
        });
    }
    strokes
}
