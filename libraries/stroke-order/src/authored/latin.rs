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
//! Accented letters take every form of their base (à has both a's, ÿ both
//! y's); ª and º keep the taught form.
//!
//! Frame (1000 em, y down): ascender 150, capitals and digits 190..850 (a
//! little below the ascender to leave room for accents), x-height 450,
//! baseline 850, descender 975. Arc angles are in degrees,
//! counterclockwise-positive as on paper: 0 is 3 o'clock, 90 is 12 o'clock, so
//! increasing angles draw a counterclockwise ("circle back") curve and
//! decreasing ones a clockwise ("circle forward") curve.
use super::{Coord, Pt, collect, dist, glyph, pt};
use crate::{Forms, StrokeStandard};
use unicode_normalization::UnicodeNormalization;

const ASC: i32 = 150;
const CAP: i32 = 190;
const X: i32 = 450;
const BASE: i32 = 850;
const DESC: i32 = 975;
const CAP_MID: i32 = (CAP + BASE) / 2; // 520
const X_MID: i32 = (X + BASE) / 2; // 650

const ACCENTED: &str = "àáâäãåçèéêëìíîïñòóôöõùúûüÿÀÁÂÄÃÅÇÈÉÊËÌÍÎÏÑÒÓÔÖÕÙÚÛÜŸ";

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
struct Pen(Vec<Pt>);

type Draw = fn() -> Vec<Pen>;

impl Pen {
    fn new() -> Self {
        Pen(Vec::new())
    }

    fn at(start: (impl Coord, impl Coord)) -> Self {
        Pen(vec![pt(start)])
    }

    fn line<A: Coord, B: Coord, const N: usize>(mut self, pts: [(A, B); N]) -> Self {
        self.0.extend(pts.map(pt));
        self
    }

    /// Elliptical arc; draws a straight join from the current point to the
    /// arc's start if they differ (e.g. the push-up retrace in b, h, n).
    fn arc(
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
    fn curve<A: Coord, B: Coord, const N: usize>(mut self, pts: [(A, B); N]) -> Self {
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

fn line<A: Coord, B: Coord, const N: usize>(pts: [(A, B); N]) -> Pen {
    Pen::new().line(pts)
}

/// A dot is a tap: a stroke too short to see as a line.
fn dot(x: impl Coord, y: impl Coord) -> Pen {
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
    ('B', || {
        let bumps = Pen::at((300, CAP))
            .line([(480, CAP)])
            .arc(480, 355, 150, 165, 90, -90)
            .line([(300, CAP_MID), (510, CAP_MID)])
            .arc(510, 685, 170, 165, 90, -90)
            .line([(300, BASE)]);
        vec![line([(300, CAP), (300, BASE)]), bumps]
    }),
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
    ('E', || {
        vec![
            line([(310, CAP), (310, BASE)]),
            line([(310, CAP), (690, CAP)]),
            line([(310, CAP_MID), (640, CAP_MID)]),
            line([(310, BASE), (690, BASE)]),
        ]
    }),
    ('F', || {
        vec![
            line([(320, CAP), (320, BASE)]),
            line([(320, CAP), (690, CAP)]),
            line([(320, CAP_MID), (640, CAP_MID)]),
        ]
    }),
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
    ('I', || {
        vec![
            line([(500, CAP), (500, BASE)]),
            line([(370, CAP), (630, CAP)]),
            line([(370, BASE), (630, BASE)]),
        ]
    }),
    ('I', || vec![line([(500, CAP), (500, BASE)])]),
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
    ('L', || vec![line([(310, CAP), (310, BASE), (680, BASE)])]),
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
    ('T', || {
        vec![
            line([(500, CAP), (500, BASE)]),
            line([(270, CAP), (730, CAP)]),
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
    ('b', || {
        vec![
            Pen::at((330, ASC))
                .line([(330, BASE)])
                .arc(500, X_MID, 170, 200, 165, -165),
        ]
    }),
    ('c', || vec![Pen::new().arc(510, X_MID, 170, 200, 45, 318)]),
    ('d', || vec![ball(480, 170).line([(650, ASC), (650, BASE)])]),
    // The bar sits above the centre so the eye is the smaller upper part, and
    // the sweep stops short of the bar's height so the tail stays open.
    ('e', || {
        vec![
            Pen::at((355, 610))
                .line([(677, 610)])
                .arc(510, X_MID, 170, 200, 11.5, 300),
        ]
    }),
    ('f', || {
        vec![
            Pen::new()
                .arc(540, 270, 120, 120, 30, 180)
                .line([(420, BASE)]),
            line([(320, X), (620, X)]),
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
    ('h', || {
        vec![hump(Pen::at((330, ASC)).line([(330, BASE)]), 330, 670)]
    }),
    // Dotless i: the base of i and of ì í î ï.
    ('ı', || vec![line([(500, X), (500, BASE)])]),
    ('i', || {
        base('ı').into_iter().chain([dot(500, 335)]).collect()
    }),
    ('j', || {
        vec![
            Pen::at((560, X))
                .line([(560, 880)])
                .arc(410, 880, 150, 95, 0, -170),
            dot(560, 335),
        ]
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
    ('l', || vec![line([(500, ASC), (500, BASE)])]),
    ('m', || {
        let pen = hump(Pen::at((250, X)).line([(250, BASE)]), 250, 500);
        vec![hump(pen, 500, 750)]
    }),
    ('n', || {
        vec![hump(Pen::at((330, X)).line([(330, BASE)]), 330, 670)]
    }),
    ('o', || vec![Pen::new().arc(500, X_MID, 180, 200, 30, 390)]),
    ('p', || {
        vec![
            Pen::at((330, X))
                .line([(330, DESC)])
                .arc(500, X_MID, 170, 200, 165, -165),
        ]
    }),
    ('q', || vec![ball(480, 170).line([(650, X), (650, DESC)])]),
    ('r', || {
        vec![
            Pen::at((370, X))
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
    ('u', || {
        vec![
            Pen::at((330, X))
                .line([(330, 680)])
                .arc(500, 680, 170, 170, 180, 360)
                .line([(670, X), (670, BASE)]),
        ]
    }),
    ('v', || vec![line([(320, X), (500, BASE), (680, X)])]),
    ('w', || {
        vec![line([
            (210, X),
            (355, BASE),
            (500, X),
            (645, BASE),
            (790, X),
        ])]
    }),
    ('x', || {
        vec![line([(330, X), (670, BASE)]), line([(670, X), (330, BASE)])]
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
    ('z', || {
        vec![line([(330, X), (670, X), (330, BASE), (670, BASE)])]
    }),
    ('z', || {
        let mut z = base('z');
        z.push(line([(410, X_MID), (590, X_MID)]));
        z
    }),
    // Letters without a decomposition.
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

/// Hangs from the bottom of the letter's curve at `(x, top)`.
fn cedilla(x: f64, top: f64) -> Vec<Pen> {
    vec![
        Pen::at((x, top))
            .line([(x, top + 30.0)])
            .arc(x + 5.0, top + 70.0, 42, 40, 90, -150),
    ]
}

/// Base letter first, then its marks, once for every form of the base letter.
/// Marks above sit in a band above cap height for capitals and above the
/// x-height for lowercase.
fn compose(c: char) -> Vec<Vec<Pen>> {
    let mut chars = c.nfd();
    let letter = chars.next().unwrap();
    let marks: Vec<char> = chars.collect();
    forms(if letter == 'i' { 'ı' } else { letter })
        .map(|strokes| add_marks(c, letter, strokes, &marks))
        .collect()
}

fn add_marks(c: char, letter: char, mut strokes: Vec<Pen>, marks: &[char]) -> Vec<Pen> {
    let pts: Vec<Pt> = strokes.iter().flat_map(|pen| pen.0.clone()).collect();
    let xs = pts.iter().map(|p| p.0);
    let centre =
        (xs.clone().fold(f64::INFINITY, f64::min) + xs.fold(f64::NEG_INFINITY, f64::max)) / 2.0;
    let (top, bottom) = if letter.is_uppercase() {
        (40.0, 150.0)
    } else {
        (275.0, 385.0)
    };
    for &mark in marks {
        strokes.extend(match mark {
            '\u{300}' => grave(centre, top, bottom),
            '\u{301}' => acute(centre, top, bottom),
            '\u{302}' => circumflex(centre, top, bottom),
            '\u{303}' => tilde(centre, top, bottom),
            '\u{308}' => diaeresis(centre, top, bottom),
            '\u{30a}' => ring(centre, top, bottom),
            '\u{327}' => {
                // The first lowest point of the letter.
                let lowest = pts
                    .iter()
                    .fold(pts[0], |a, &p| if p.1 > a.1 { p } else { a });
                cedilla(lowest.0, f64::from(BASE))
            }
            _ => panic!("no drawing for mark U+{:04X} in {c}", u32::from(mark)),
        });
    }
    strokes
}
