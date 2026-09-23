//! Stroke-order packs authored in this repository, for scripts with no
//! published stroke-order dataset: Devanagari, Thai, Latin and Cyrillic.
//!
//! Each script module draws its letters as directed pen centerlines in its own
//! design frame and maps them into a 1000-unit box (y down). [`glyph`] then
//! rounds to whole box units, drops repeated points and normalises to the unit
//! square. Rounding to integers first keeps the geometry exactly as it was
//! drawn and reviewed (the packs used to be generated as integer JSON).
//!
//! The drawing toolkit below is shared; each script keeps the conventions it
//! was drawn with (angle direction, sampling density, how joins are handled),
//! passed in explicitly rather than unified, because changing them would
//! change the reviewed geometry.
pub mod cyrillic;
// The Devanagari and Thai letter tables keep one row of waypoints per line, as
// they were drawn; rustfmt would put every point on its own line.
#[rustfmt::skip]
pub mod devanagari;
pub mod latin;
#[rustfmt::skip]
pub mod thai;

use crate::{Glyphs, Stroke, StrokeGlyph, StrokeStandard, validate};

pub type Pt = (f64, f64);
/// A letter's strokes in writing order, each a polyline in pen direction.
pub type Strokes = Vec<Vec<Pt>>;
/// Draws one letter.
pub type Draw = fn() -> Strokes;

/// Lets letter tables mix integer and fractional coordinates.
pub trait Coord: Copy {
    fn f64(self) -> f64;
}
impl Coord for i32 {
    fn f64(self) -> f64 {
        self.into()
    }
}
impl Coord for f64 {
    fn f64(self) -> f64 {
        self
    }
}

pub fn pt<A: Coord, B: Coord>((x, y): (A, B)) -> Pt {
    (x.f64(), y.f64())
}

/// A point list from coordinate tuples of any [`Coord`] type.
macro_rules! pts {
    ($($p:expr),* $(,)?) => { vec![$($crate::authored::pt($p)),*] };
}
pub(crate) use pts;

/// A letter's stroke list; `..strokes` splices in a list of strokes, like
/// `[*body(), head()]` would.
macro_rules! strokes {
    (@parts [$($part:expr),*];) => {
        std::iter::empty::<Vec<$crate::authored::Pt>>()
            $(.chain($part))*
            .collect::<Vec<_>>()
    };
    (@parts [$($part:expr),*]; .. $e:expr $(, $($rest:tt)*)?) => {
        $crate::authored::strokes!(@parts [$($part,)* $e]; $($($rest)*)?)
    };
    (@parts [$($part:expr),*]; $e:expr $(, $($rest:tt)*)?) => {
        $crate::authored::strokes!(@parts [$($part,)* [$e]]; $($($rest)*)?)
    };
    ($($t:tt)*) => { $crate::authored::strokes!(@parts []; $($t)*) };
}
pub(crate) use strokes;

pub fn dist(a: Pt, b: Pt) -> f64 {
    (a.0 - b.0).hypot(a.1 - b.1)
}

/// `n + 1` evenly spaced points on an ellipse from angle `a0` to `a1`
/// (degrees, 0 = +x), at `(cx + rx cos a, cy + ry sin a)`. Whether increasing
/// angles run clockwise depends on the frame's y direction; a frame that wants
/// "anticlockwise-positive on paper" with y down passes a negative `ry`.
pub fn arc((cx, cy): Pt, rx: f64, ry: f64, a0: f64, a1: f64, n: usize) -> Vec<Pt> {
    (0..=n)
        .map(|i| {
            let a = (a0 + (a1 - a0) * i as f64 / n as f64).to_radians();
            (cx + rx * a.cos(), cy + ry * a.sin())
        })
        .collect()
}

/// One pen stroke made of consecutive parts: points closer than `eps` to the
/// previous one (the shared junctions) are dropped.
pub fn join(parts: impl IntoIterator<Item = Vec<Pt>>, eps: f64) -> Vec<Pt> {
    let mut out: Vec<Pt> = Vec::new();
    for p in parts.into_iter().flatten() {
        if out.last().is_none_or(|&last| dist(last, p) > eps) {
            out.push(p);
        }
    }
    out
}

/// How a centripetal Catmull-Rom spline invents the neighbours of its ends.
pub enum Ends {
    /// Repeat the end point (the curve leaves its ends slowly).
    Repeat,
    /// Mirror the next point through the end (a natural end tangent).
    Reflect,
}

/// A centripetal Catmull-Rom spline through `pts`: `ends` picks the phantom
/// end points, `min_knot` floors each knot distance before its square root,
/// and `samples(span_length)` is the number of points emitted per span.
/// Zero-length spans are skipped.
pub fn centripetal(
    pts: &[Pt],
    ends: Ends,
    min_knot: f64,
    samples: impl Fn(f64) -> usize,
) -> Vec<Pt> {
    let Some((&first, &last)) = pts.first().zip(pts.last()) else {
        return Vec::new();
    };
    if pts.len() < 2 {
        return vec![first];
    }
    let reflect = |p: Pt, q: Pt| (2.0 * p.0 - q.0, 2.0 * p.1 - q.1);
    let (before, after) = match ends {
        Ends::Repeat => (first, last),
        Ends::Reflect => (reflect(first, pts[1]), reflect(last, pts[pts.len() - 2])),
    };
    let ext: Vec<Pt> = std::iter::once(before)
        .chain(pts.iter().copied())
        .chain(std::iter::once(after))
        .collect();
    let mut out = vec![first];
    for w in ext.windows(4) {
        let [p0, p1, p2, p3] = [w[0], w[1], w[2], w[3]];
        let length = dist(p1, p2);
        if length < 1e-6 {
            continue;
        }
        let n = samples(length);
        let knot = |t: f64, a: Pt, b: Pt| t + dist(a, b).max(min_knot).powf(0.5);
        let t0 = 0.0;
        let t1 = knot(t0, p0, p1);
        let t2 = knot(t1, p1, p2);
        let t3 = knot(t2, p2, p3);
        for k in 1..=n {
            let t = t1 + (t2 - t1) * k as f64 / n as f64;
            let lerp = |a: Pt, b: Pt, ta: f64, tb: f64| {
                let (wa, wb) = ((tb - t) / (tb - ta), (t - ta) / (tb - ta));
                (wa * a.0 + wb * b.0, wa * a.1 + wb * b.1)
            };
            let a1 = lerp(p0, p1, t0, t1);
            let a2 = lerp(p1, p2, t1, t2);
            let a3 = lerp(p2, p3, t2, t3);
            let b1 = lerp(a1, a2, t0, t2);
            let b2 = lerp(a2, a3, t1, t3);
            out.push(lerp(b1, b2, t1, t2));
        }
    }
    out
}

/// A head loop (Thai หัว): start inside it, spiral out, go once around and
/// leave at angle `exit` (degrees, y up), moving clockwise on the page if
/// `cw`. The pen leaves tangentially, so `exit` is where the loop joins the
/// rest of the letter.
pub fn head_loop((cx, cy): Pt, exit: f64, cw: bool, r: f64) -> Vec<Pt> {
    let sign = if cw { -1.0 } else { 1.0 };
    let start = exit - sign * 390.0;
    (0..14)
        .map(|i| {
            let t = f64::from(i) / 13.0;
            let a = (start + sign * 390.0 * t).to_radians();
            let rr = r * 1.0_f64.min(0.45 + 0.55 * (390.0 * t) / 90.0);
            (cx + rr * a.cos(), cy + rr * a.sin())
        })
        .collect()
}

/// A waypoint list for a smoothed stroke: points, spliced point lists, and
/// [`CORNER`]s, which make the pen turn sharply instead of smoothly.
pub enum Item {
    Point(Pt),
    Points(Vec<Pt>),
    Items(Vec<Item>),
    Corner,
}
pub const CORNER: Item = Item::Corner;

impl<A: Coord, B: Coord> From<(A, B)> for Item {
    fn from(p: (A, B)) -> Self {
        Item::Point(pt(p))
    }
}
impl From<Vec<Pt>> for Item {
    fn from(points: Vec<Pt>) -> Self {
        Item::Points(points)
    }
}
impl From<Vec<Item>> for Item {
    fn from(items: Vec<Item>) -> Self {
        Item::Items(items)
    }
}

/// A list of [`Item`]s from anything convertible into one.
macro_rules! items {
    ($($item:expr),* $(,)?) => { vec![$($crate::authored::Item::from($item)),*] };
}
pub(crate) use items;

/// Splits waypoints into runs at each [`CORNER`]; a new run starts at the
/// previous run's last waypoint.
pub fn corner_runs(items: Vec<Item>) -> Strokes {
    fn walk(item: Item, runs: &mut Strokes) {
        match item {
            Item::Point(p) => runs.last_mut().unwrap().push(p),
            Item::Points(ps) => runs.last_mut().unwrap().extend(ps),
            Item::Items(items) => items.into_iter().for_each(|i| walk(i, runs)),
            Item::Corner => {
                let last = *runs.last().unwrap().last().unwrap();
                runs.push(vec![last]);
            }
        }
    }
    let mut runs = vec![Vec::new()];
    items.into_iter().for_each(|i| walk(i, &mut runs));
    runs
}

/// A letter's strokes, mapped into the 1000-unit box by `to_box`, rounded to
/// whole units (ties to even) with repeats dropped, and scaled to the unit
/// square.
pub fn glyph(standard: StrokeStandard, strokes: Strokes, to_box: impl Fn(Pt) -> Pt) -> StrokeGlyph {
    let strokes = strokes
        .into_iter()
        .map(|s| {
            let mut points: Vec<(i32, i32)> = s
                .into_iter()
                .map(|p| {
                    let (x, y) = to_box(p);
                    (x.round_ties_even() as i32, y.round_ties_even() as i32)
                })
                .collect();
            points.dedup();
            Stroke {
                points: points
                    .into_iter()
                    .map(|(x, y)| (x as f32 / 1000.0, y as f32 / 1000.0))
                    .collect(),
            }
        })
        .collect();
    StrokeGlyph { standard, strokes }
}

/// Collects `(char, glyph)` pairs. Panics on a duplicate character or a
/// stroke that collapses or leaves the box: the letters are constants, and the
/// tests build every pack.
pub fn collect(glyphs: impl IntoIterator<Item = (char, StrokeGlyph)>) -> Glyphs {
    let mut out = Glyphs::default();
    for (c, g) in glyphs {
        validate(&g).unwrap_or_else(|e| panic!("{:?} {c}: {e}", g.standard));
        assert!(out.insert(c, g).is_none(), "{c} drawn twice");
    }
    out
}
