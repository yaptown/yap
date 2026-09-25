//! Stroke-order centerlines for the Thai script.
//!
//! Conventions follow the Thai Ministry of Education handwriting guide
//! (ตัวอักษรแบบกระทรวงศึกษาธิการ, "หัวกลมตัวมน"): a letter with a head loop (หัว)
//! starts inside the head, goes around it and continues to the end of the
//! letter without lifting the pen; letters without a head start where the
//! guide starts them. Stroke order and direction were reviewed against
//! thai-notes.com's recorded pen trajectories (nearly every character),
//! ActiveThai's numbered start/end-dot worksheets, the Ministry's handwriting
//! handbook (ลายมือสวย) rules, and a published list of which heads run
//! clockwise. Where they disagree, the forms the others use are kept as
//! alternatives.
//!
//! # Alternatives
//!
//! Accepted forms after the taught one (see [`super`]'s "Accepted
//! letterforms"):
//!
//! - ศ ส: the tail drawn inward, from its tip (thai-notes; the taught
//!   outward one is ActiveThai's).
//! - บ ป ษ: a pen lift at the bottom-left corner (ActiveThai).
//! - ฆ ซ: thai-notes' letterform with the head low on the left and two humps
//!   on top. Its ฑ trace has ฆ's loop, so ฑ gets no such form.
//! - ๆ: the tail kinked left partway down (thai-notes).
//! - ๙: a ๑-style head spiralling out from inside the bowl (thai-notes).
//! - ำ ํ: the circle anticlockwise (taught clockwise, per thai-notes).
//! - ิ: drawn the other way, from the arch's right end (ActiveThai).
//! - ี: one stroke, ิ then back up into the tick (ActiveThai).
//! - ึ: taught as ิ going on into an anticlockwise circle (thai-notes); also
//!   circle first, then base leftwards and the arch back to it, and
//!   ActiveThai's circle, arch leftwards, then the base.
//! - ็: the head anticlockwise (taught clockwise, per thai-notes).
//! - ๋: the bar first (taught vertical first, per thai-notes).
//!
//! A composed unit takes every combination of its parts' forms.
//!
//! Glyphs are drawn in "writing units": the Thai writing grid with the
//! baseline at y=0, the top line of a consonant body at about y=560 and y
//! pointing up. Stems sit at y≈36 (bottom) and y≈515 (top) because they are
//! centerlines, and a normal head loop has radius 66. Combining marks are drawn
//! relative to the end of their base consonant, x=0, as a font does; drawn on
//! their own, they are set on an imaginary อ whose ink is centred 300 units to
//! the left of that point, tone marks and ์ a tier up ([`TONE_LIFT`]).
//!
//! # The writable unit: a consonant with its marks
//!
//! [`segment`] splits text into a consonant (ก–ฮ, ฤ ฦ included) with the run
//! of combining marks after it (above-vowels ั ิ ี ึ ื ็, below-vowels ุ ู,
//! tone marks ่ ้ ๊ ๋, ์, ํ, ๎ and ฺ), and every other character on its own:
//! spacing vowels (ะ า ำ เ แ โ ใ ไ ๅ; ำ follows the cluster, so น้ำ is น้
//! then ำ), digits, ฯ ๆ, spaces, and a mark with no consonant before it. There
//! is no word segmentation: the units are the syllable's written pieces.
//! Single characters keep exactly the glyphs they were drawn and reviewed
//! with; [`Thai::glyphs`] composes the rest. ฺ and ๎ are not drawn, so a unit
//! carrying one has no glyph (neither occurs in the course corpus).
//!
//! # Composition
//!
//! A unit is its consonant, placed exactly where it sits on its own (centred
//! on its ink), then each mark in text order: consonant, vowel, tone mark, the
//! order Thai schools teach. The rules, checked against Noto Sans Thai Looped
//! shaped by HarfBuzz:
//!
//! - **Anchor.** A font hangs marks off the end of the consonant, so they sit
//!   over its right side (over the right half of the wide ญ ฌ ณ). Marks move
//!   by the difference between the consonant's right edge and the notional
//!   อ's; the right edge is the rightmost ink in the lower half of the body
//!   ([`ANCHOR_BAND`]), so the tails of ศ ส ษ ฮ and the เชิง of ญ do not pull
//!   marks right. On อ itself a mark lands exactly where it does alone.
//! - **Tiers.** Above-vowels (and ํ) sit on the body, below-vowels under the
//!   line. A tone mark or ์ sits on the body when there is no above-vowel, and
//!   [`TONE_LIFT`] higher, over the vowel, when there is one.
//! - **Ascenders.** Where the low ink of the above-marks would meet the stem of
//!   ป ฝ ฟ, the marks move left of it together, at their normal height, as
//!   Noto sets them (fonts do not raise them). If that would push them past
//!   the consonant's left edge (the wide ึ over ฝ, anything over ฬ's loop),
//!   they rise over the ascender instead. Noto swaps in a narrower ึ and a
//!   short ฬ there; this pack has one form of each.
//! - **Descenders.** ญ and ฐ are written without their เชิง (their last
//!   stroke) over a below-vowel, which then sits as under any consonant, as
//!   Thai writes them and Noto sets them ([`FOOTED`]). Under ฎ ฏ ฤ ฦ, which
//!   keep their descenders, it drops below the descender.
//! - **ื.** A tone mark on ื sits midway between its two ticks
//!   ([`UEE_TONE_DX`]).
//! - **Clearance.** Moved marks keep [`CLEARANCE`] from the ink they clear,
//!   measured between centerlines.
//! - **Box.** A composed unit that would leave the box (a descender stack, a
//!   lifted ์ over ิ) is moved back in, [`INSET`] from the edge, and scaled
//!   uniformly only if it is too tall or wide to fit at all. None of the
//!   corpus units need scaling.
//!
//! A stroke is a sequence of waypoints the pen passes through, smoothed with a
//! centripetal Catmull-Rom spline. [`CORNER`] between waypoints makes a sharp
//! turn there instead of a smooth one. [`head`] gives the waypoints of a head
//! loop and [`ring`] those of a closed loop in the middle of a stroke.
use super::{
    CORNER, Draw, Ends, Item, Pt, Strokes, bounds, centripetal, collect, corner_runs, glyph, group, items, pts,
};
use crate::Forms;
use language_utils::{StrokeGlyph, StrokeStandard};
use rustc_hash::FxHashMap;

/// Writing units -> 1000-unit box.
const SCALE: f64 = 0.7;
/// Box y of the writing baseline.
const BASELINE: f64 = 780.0;
/// Combining marks: base consonant ink centre is 300 left of x=0.
const MARK_BASE_CENTRE: f64 = 300.0;
/// Tone marks sit above the above-vowels.
const TONE_LIFT: f64 = 210.0;
/// Radius of a consonant head loop.
const HEAD: i32 = 66;
/// Bottom of a stem that runs below the line (ฤ ฦ): the font's descender
/// (-235) inset by the same 36 as a stem's foot on the baseline.
const DESCENDER: i32 = -199;
const CW: bool = true;
const CCW: bool = false;
/// A consonant's right edge, where its marks anchor, is its rightmost ink
/// between the baseline and this height, so tails and flicks that leave the
/// body (ศ ส ษ ฮ ฬ ญ) do not count.
const ANCHOR_BAND: f64 = 250.0;
/// Ink above this rises past the body (the stems of ป ฝ ฟ, the loop of ฬ);
/// ink below this hangs under the line (ฎ ฏ ฐ ญ ฤ ฦ).
const ASCENDER: f64 = 600.0;
const DESCENDS: f64 = -40.0;
/// The space kept between a mark and consonant ink it has to clear.
const CLEARANCE: f64 = 100.0;
/// A composed unit is kept this far inside the 1000-unit box.
const INSET: f64 = 20.0;
/// The consonants written without their เชิง (their last stroke) over a
/// below-vowel.
const FOOTED: [char; 2] = ['ญ', 'ฐ'];
/// A tone mark on ื moves this far left, to midway between its ticks
/// (OBEC's rule 19); on the plain tier it sits over the right tick.
const UEE_TONE_DX: f64 = -32.0;

/// Where a combining mark stacks.
#[derive(Clone, Copy, PartialEq)]
enum Tier {
    /// ั ิ ี ึ ื ็ ํ, on top of the consonant.
    Above,
    /// ุ ู, under it.
    Below,
    /// ่ ้ ๊ ๋ ์, on top of the consonant, or of an above-vowel if there is one.
    Tone,
}

fn is_consonant(c: char) -> bool {
    matches!(c, '\u{e01}'..='\u{e2e}')
}

/// The combining marks: above- and below-vowels, phinthu, tone marks,
/// thanthakhat, nikhahit and yamakkan.
fn is_mark(c: char) -> bool {
    matches!(c, '\u{e31}' | '\u{e34}'..='\u{e3a}' | '\u{e47}'..='\u{e4e}')
}

/// Splits text into its writable units in order, skipping nothing: a
/// consonant with the combining marks that follow it, and every other
/// character (spacing vowels, digits, spaces, a mark with no consonant) on its
/// own. Units are syllable pieces, not words: ครับ is ค, รั, บ.
pub fn segment(text: &str) -> Vec<&str> {
    let mut units = Vec::new();
    let mut rest = text;
    while let Some(first) = rest.chars().next() {
        let mut len = first.len_utf8();
        if is_consonant(first) {
            len += rest[len..].chars().take_while(|&c| is_mark(c)).map(char::len_utf8).sum::<usize>();
        }
        units.push(&rest[..len]);
        rest = &rest[len..];
    }
    units
}

/// The Thai letters and marks, built once; a consonant with marks is composed
/// from them on demand.
pub struct Thai {
    /// Every character drawn on its own: spacing letters centred on their ink,
    /// a lone mark on the notional อ.
    pub(crate) units: Forms,
    /// Every accepted form of each consonant, in writing units.
    consonants: FxHashMap<char, Vec<Strokes>>,
    /// Every accepted form of each combining mark, in writing units, drawn
    /// relative to the end of its base (tone marks unlifted).
    marks: FxHashMap<char, (Tier, Vec<Strokes>)>,
    /// The notional อ's anchor, relative to its ink centre.
    notional_anchor: f64,
}

impl Default for Thai {
    fn default() -> Self {
        let consonants = group(consonants().into_iter().map(|(c, draw)| (c, draw())));
        let marks: FxHashMap<char, (Tier, Vec<Strokes>)> = group(marks().into_iter().map(|(c, tier, draw)| (c, (tier, draw()))))
            .into_iter()
            .map(|(c, forms)| (c, (forms[0].0, forms.into_iter().map(|f| f.1).collect())))
            .collect();
        // Spacing characters are centred on their own ink.
        let spacing = consonants.iter().flat_map(|(&c, forms)| forms.iter().map(move |s| (c, s.clone()))).chain(others().into_iter().map(|(c, draw)| (c, draw()))).map(|(c, strokes)| {
            let dx = -centre(&strokes);
            (c, glyph(StrokeStandard::Thai, strokes, to_box(dx)))
        });
        let lone_marks = marks.iter().flat_map(|(&c, (tier, forms))| {
            let lift = if *tier == Tier::Tone { TONE_LIFT } else { 0.0 };
            forms.iter().map(move |form| {
                let strokes = form.iter().map(|s| shift(s.clone(), 0.0, lift)).collect();
                (c, glyph(StrokeStandard::Thai, strokes, to_box(MARK_BASE_CENTRE)))
            })
        });
        let units = collect(spacing.chain(lone_marks));
        let o = &consonants[&'อ'][0];
        let notional_anchor = anchor(o) - centre(o);
        Thai { units, consonants, marks, notional_anchor }
    }
}

impl Thai {
    /// Every accepted form of one unit from [`segment`], the taught form
    /// first; empty if the pack cannot draw it.
    pub fn glyphs(&self, unit: &str) -> Vec<StrokeGlyph> {
        let mut chars = unit.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => self.units.get(&c).cloned().unwrap_or_default(),
            _ => self.compose(unit),
        }
    }

    /// A consonant with its marks, in every combination of its parts' forms,
    /// the taught forms first.
    fn compose(&self, unit: &str) -> Vec<StrokeGlyph> {
        let mut chars = unit.chars();
        let Some((c, bases)) = chars.next().and_then(|c| Some((c, self.consonants.get(&c)?))) else {
            return Vec::new();
        };
        let Some(marks) = chars.map(|m| self.marks.get(&m).map(|(tier, forms)| (m, *tier, forms))).collect::<Option<Vec<_>>>() else {
            return Vec::new();
        };
        let mut combos: Vec<Vec<&Strokes>> = bases.iter().map(|b| vec![b]).collect();
        for (_, _, forms) in &marks {
            combos = combos.into_iter().flat_map(|combo| forms.iter().map(move |f| [combo.clone(), vec![f]].concat())).collect();
        }
        combos.into_iter().map(|combo| self.place(c, &marks, &combo)).collect()
    }

    /// One combination: the consonant where it sits on its own, then each
    /// mark in text order, anchored to the consonant's right edge and stacked
    /// (see the module docs).
    fn place(&self, consonant: char, marks: &[(char, Tier, &Vec<Strokes>)], forms: &[&Strokes]) -> StrokeGlyph {
        let (base, mark_forms) = forms.split_first().unwrap();
        let lift = if marks.iter().any(|m| m.1 == Tier::Above) { TONE_LIFT } else { 0.0 };
        let tone_dx = if marks.iter().any(|m| m.0 == 'ื') { UEE_TONE_DX } else { 0.0 };
        let c = centre(base);
        let footless = FOOTED.contains(&consonant) && marks.iter().any(|m| m.1 == Tier::Below);
        let base: Strokes = base[..base.len() - usize::from(footless)].iter().map(|s| shift(s.clone(), -c, 0.0)).collect();
        // Exactly MARK_BASE_CENTRE on อ, so its marks land where they do alone.
        let dx = MARK_BASE_CENTRE + (anchor(&base) - self.notional_anchor);
        let mut placed: Vec<(bool, Strokes)> = marks
            .iter()
            .zip(mark_forms)
            .map(|(&(_, tier, _), strokes)| {
                let (tx, dy) = if tier == Tier::Tone { (tone_dx, lift) } else { (0.0, 0.0) };
                (tier != Tier::Below, strokes.iter().map(|s| shift(s.clone(), dx + tx, dy)).collect())
            })
            .collect();
        for above in [true, false] {
            let group: Strokes = placed.iter().filter(|m| m.0 == above).flat_map(|m| m.1.clone()).collect();
            if group.is_empty() {
                continue;
            }
            let (mx, my) = if above { clear_above(&base, &group) } else { (0.0, clear_below(&base, &group)) };
            for (_, strokes) in placed.iter_mut().filter(|m| m.0 == above) {
                *strokes = strokes.iter().map(|s| shift(s.clone(), mx, my)).collect();
            }
        }
        let strokes: Strokes = base.into_iter().chain(placed.into_iter().flat_map(|m| m.1)).collect();
        let fit = Fit::new(&strokes);
        glyph(StrokeStandard::Thai, strokes, |p| fit.apply(to_box(0.0)(p)))
    }
}

/// Writing units, `dx` shifted, -> the 1000-unit box.
fn to_box(dx: f64) -> impl Fn(Pt) -> Pt {
    move |(x, y)| (500.0 + SCALE * (x + dx), BASELINE - SCALE * y)
}

/// The centre of the ink, horizontally.
fn centre(strokes: &Strokes) -> f64 {
    let (min, max, _, _) = bounds(strokes);
    (min + max) / 2.0
}

/// The right edge of a consonant's body; see [`ANCHOR_BAND`].
fn anchor(strokes: &Strokes) -> f64 {
    strokes.iter().flatten().filter(|p| (0.0..=ANCHOR_BAND).contains(&p.1)).map(|p| p.0).fold(f64::NEG_INFINITY, f64::max)
}

/// How far the above-marks move, together, to clear an ascender their low ink
/// (below its top plus [`CLEARANCE`]; a lifted tone mark is already clear)
/// would touch: left of it if they then still sit over the consonant (ป ฝ ฟ,
/// as Noto Sans Thai sets them), otherwise up over it (ฬ, and the wide ึ).
fn clear_above(base: &Strokes, marks: &Strokes) -> Pt {
    let ascender: Vec<Pt> = base.iter().flatten().copied().filter(|p| p.1 > ASCENDER).collect();
    let (left, right, _, top) = bounds(&[ascender]);
    let low: Vec<Pt> = marks.iter().flatten().copied().filter(|p| p.1 < top + CLEARANCE).collect();
    let (x0, x1, y0, _) = bounds(&[low]);
    // Also covers no ascender or no low ink: the bounds are then infinite.
    if !(x1 > left - CLEARANCE && x0 < right + CLEARANCE) {
        return (0.0, 0.0);
    }
    let dx = left - CLEARANCE - x1;
    if bounds(marks).0 + dx >= bounds(base).0 - CLEARANCE {
        (dx, 0.0)
    } else {
        (0.0, top + CLEARANCE - y0)
    }
}

/// How far the below-marks drop to clear a descender above them.
fn clear_below(base: &Strokes, marks: &Strokes) -> f64 {
    let (x0, x1, _, y1) = bounds(marks);
    let over: Vec<Pt> = base.iter().flatten().copied().filter(|p| p.1 < DESCENDS && p.0 > x0 - CLEARANCE && p.0 < x1 + CLEARANCE).collect();
    if over.is_empty() {
        return 0.0;
    }
    (bounds(&[over]).2 - CLEARANCE - y1).min(0.0)
}

/// Keeps a composed unit [`INSET`] inside the box: moved back in if it
/// leaves it, and scaled uniformly about the box centre only if it is too
/// big to fit at all. A unit that fits is left exactly where it is.
struct Fit {
    scale: f64,
    dx: f64,
    dy: f64,
}

impl Fit {
    fn new(strokes: &Strokes) -> Fit {
        let boxed: Strokes = strokes.iter().map(|s| s.iter().map(|&p| to_box(0.0)(p)).collect()).collect();
        let (x0, x1, y0, y1) = bounds(&boxed);
        let room = 1000.0 - 2.0 * INSET;
        let scale = (room / (x1 - x0)).min(room / (y1 - y0)).min(1.0);
        let into = |a: f64, b: f64| {
            let (a, b) = (500.0 + (a - 500.0) * scale, 500.0 + (b - 500.0) * scale);
            (INSET - a).max(0.0) + (1000.0 - INSET - b).min(0.0)
        };
        Fit { scale, dx: into(x0, x1), dy: into(y0, y1) }
    }

    fn apply(&self, (x, y): Pt) -> Pt {
        if self.scale == 1.0 && self.dx == 0.0 && self.dy == 0.0 {
            return (x, y);
        }
        (500.0 + (x - 500.0) * self.scale + self.dx, 500.0 + (y - 500.0) * self.scale + self.dy)
    }
}

fn consonants() -> [(char, Draw); 53] {
    [
        ('ก', ko_kai),
        ('ข', kho_khai),
        ('ฃ', kho_khuat),
        ('ค', kho_khwai),
        ('ฅ', kho_khon),
        ('ฆ', kho_rakhang),
        ('ฆ', kho_rakhang_humps),
        ('ง', ngo_ngu),
        ('จ', cho_chan),
        ('ฉ', cho_ching),
        ('ช', cho_chang),
        ('ซ', so_so),
        ('ซ', so_so_humps),
        ('ฌ', cho_choe),
        ('ญ', yo_ying),
        ('ฎ', do_chada),
        ('ฏ', to_patak),
        ('ฐ', tho_than),
        ('ฑ', tho_montho),
        ('ฒ', tho_phuthao),
        ('ณ', no_nen),
        ('ด', do_dek),
        ('ต', to_tao),
        ('ถ', || tho_thung(40)),
        ('ท', tho_thahan),
        ('ธ', tho_thong),
        ('น', no_nu),
        ('บ', || bo_baimai(515)),
        ('บ', || lift_at(bo_baimai(515), (196, 36))),
        ('ป', || bo_baimai(715)),
        ('ป', || lift_at(bo_baimai(715), (196, 36))),
        ('ผ', || pho_phueng(515)),
        ('ฝ', || pho_phueng(715)),
        ('พ', || pho_phan(515)),
        ('ฟ', || pho_phan(717)),
        ('ภ', || pho_samphao(40)),
        ('ม', mo_ma),
        ('ย', yo_yak),
        ('ร', ro_ruea),
        ('ฤ', || tho_thung(DESCENDER)),
        ('ล', lo_ling),
        ('ฦ', || pho_samphao(DESCENDER)),
        ('ว', wo_waen),
        ('ศ', so_sala),
        ('ศ', || reversed(so_sala(), 1)),
        ('ษ', so_ruesi),
        ('ษ', || lift_at(so_ruesi(), (192, 36))),
        ('ส', so_suea),
        ('ส', || reversed(so_suea(), 1)),
        ('ห', ho_hip),
        ('ฬ', lo_chula),
        ('อ', o_ang),
        ('ฮ', ho_nokhuk),
    ]
}

fn others() -> [(char, Draw); 24] {
    [
        ('ฯ', paiyannoi),
        ('ะ', sara_a),
        ('า', || sara_aa(40)),
        ('ำ', || sara_am(CW)),
        ('ำ', || sara_am(CCW)),
        ('เ', || sara_e(0)),
        ('แ', sara_ae),
        ('โ', sara_o),
        ('ใ', sara_ai_maimuan),
        ('ไ', sara_ai_maimalai),
        ('ๅ', || sara_aa(-190)),
        ('ๆ', mai_yamok),
        ('ๆ', mai_yamok_kinked),
        ('๐', digit_0),
        ('๑', digit_1),
        ('๒', digit_2),
        ('๓', digit_3),
        ('๔', digit_4),
        ('๕', digit_5),
        ('๖', digit_6),
        ('๗', digit_7),
        ('๘', digit_8),
        ('๙', digit_9),
        ('๙', digit_9_spiral),
    ]
}

fn marks() -> [(char, Tier, Draw); 21] {
    use Tier::*;
    [
        ('ั', Above, mai_han_akat),
        ('ิ', Above, sara_i),
        ('ิ', Above, || reversed(sara_i(), 0)),
        ('ี', Above, sara_ii),
        ('ี', Above, sara_ii_one_stroke),
        ('ึ', Above, sara_ue),
        ('ึ', Above, sara_ue_circle_first),
        ('ึ', Above, sara_ue_circle_arch_base),
        ('ื', Above, sara_uee),
        ('็', Above, || mai_taikhu(CW)),
        ('็', Above, || mai_taikhu(CCW)),
        ('ํ', Above, || vec![nikhahit(CW)]),
        ('ํ', Above, || vec![nikhahit(CCW)]),
        ('ุ', Below, sara_u),
        ('ู', Below, sara_uu),
        ('่', Tone, mai_ek),
        ('้', Tone, mai_tho),
        ('๊', Tone, mai_tri),
        ('๋', Tone, mai_chattawa),
        ('๋', Tone, || mai_chattawa().into_iter().rev().collect()),
        ('์', Tone, thanthakhat),
    ]
}

// --- drawing toolkit -------------------------------------------------------

/// Points on a circle (or ellipse) from angle `a0` to `a1` in degrees, about
/// every 30 degrees. Angles are measured anticlockwise from +x, with y up, so
/// `a1 < a0` runs clockwise as seen on the page.
fn arc_points((cx, cy): (i32, i32), r: i32, a0: i32, a1: i32, ry: i32) -> Vec<Pt> {
    let n = (f64::from((a1 - a0).abs()) / 30.0)
        .round_ties_even()
        .max(1.0) as usize;
    let f = f64::from;
    super::arc((f(cx), f(cy)), f(r), f(ry), f(a0), f(a1), n)
}

/// A [`super::head_loop`] of the normal radius.
fn head(c: (i32, i32), exit: i32, cw: bool) -> Vec<Pt> {
    head_r(c, exit, cw, HEAD)
}

fn head_r((cx, cy): (i32, i32), exit: i32, cw: bool, r: i32) -> Vec<Pt> {
    super::head_loop((cx.into(), cy.into()), exit.into(), cw, r.into())
}

/// A loop in the middle of a stroke: from angle `a0`, `turn` degrees around
/// (negative = clockwise).
fn ring(c: (i32, i32), a0: i32, turn: i32) -> Vec<Pt> {
    ring_r(c, a0, turn, HEAD)
}

fn ring_r(c: (i32, i32), a0: i32, turn: i32, r: i32) -> Vec<Pt> {
    arc_points(c, r, a0, a0 + turn, r)
}

/// An elliptical [`ring`].
fn oval(c: (i32, i32), a0: i32, turn: i32, rx: i32, ry: i32) -> Vec<Pt> {
    arc_points(c, rx, a0, a0 + turn, ry)
}

/// Centripetal Catmull-Rom through the waypoints, sampled roughly every 12
/// units.
fn catmull_rom(pts: &[Pt]) -> Vec<Pt> {
    centripetal(pts, Ends::Reflect, 1e-3, |length| {
        (length / 12.0).round_ties_even().clamp(2.0, 16.0) as usize
    })
}

/// One pen-down motion through waypoints; lists are spliced in and
/// [`CORNER`] marks a sharp turn.
fn stroke(items: Vec<Item>) -> Vec<Pt> {
    let mut pts: Vec<Pt> = Vec::new();
    for run in corner_runs(items) {
        let skip = usize::from(!pts.is_empty());
        pts.extend(catmull_rom(&run).into_iter().skip(skip));
    }
    pts
}

macro_rules! stroke {
    ($($item:expr),* $(,)?) => { stroke(items![$($item),*]) };
}

fn shift(pts: Vec<Pt>, dx: f64, dy: f64) -> Vec<Pt> {
    pts.into_iter().map(|(x, y)| (x + dx, y + dy)).collect()
}

/// The same letter written with a pen lift at `corner`, a [`CORNER`]
/// waypoint of one of its strokes: that stroke ends there and the next
/// starts there.
fn lift_at(strokes: Strokes, corner: (i32, i32)) -> Strokes {
    let corner = super::pt(corner);
    strokes
        .into_iter()
        .flat_map(|s| match s.iter().position(|&p| super::dist(p, corner) < 0.5) {
            Some(i) => vec![s[..=i].to_vec(), s[i..].to_vec()],
            None => vec![s],
        })
        .collect()
}

/// The same letter with stroke `i` drawn the other way.
fn reversed(mut strokes: Strokes, i: usize) -> Strokes {
    strokes[i].reverse();
    strokes
}

// --- glyphs ----------------------------------------------------------------
// Each function returns the strokes in writing order. CW/CCW is as seen on the
// page. Consonants are listed in alphabetical order.

/// The ก-style notched top-left shared by ก ฌ ญ ฎ ฏ ถ ภ: up the left side,
/// out to the notch tip, back to the left point, then up into the arch.
fn arch_after_notch(right_x: i32, bottom_y: i32) -> Vec<Item> {
    items![
        CORNER,
        (72, 377),
        CORNER,
        (100, 440),
        (175, 505),
        (290, 531),
        (right_x - 90, 512),
        (right_x - 25, 455),
        (right_x - 4, 360),
        (right_x, bottom_y),
    ]
}

/// ก: starts bottom-left, notch, arch, down the right
fn ko_kai() -> Strokes {
    vec![stroke![(128, 40), (128, 200), (150, 262), (200, 305), (262, 340), arch_after_notch(484, 40)]]
}

/// ข
fn kho_khai() -> Strokes {
    vec![
        stroke![
            head((142, 465), 0, CW),
            (212, 420), (190, 355), (163, 295), (152, 220), (150, 36),
            CORNER,
            (385, 36), (414, 48), (425, 90), (426, 515),
        ]
    ]
}

/// ฃ: ข with a notched head
fn kho_khuat() -> Strokes {
    vec![
        stroke![
            head((114, 470), 20, CW),
            (205, 466), (245, 492), (272, 532),
            CORNER,
            (262, 470), (232, 400), (192, 320), (166, 240), (161, 150), (161, 36),
            CORNER,
            (395, 36), (428, 48), (438, 90), (439, 515),
        ]
    ]
}

/// ค: head inside, down to the left point, round the top, down the right
fn kho_khwai() -> Strokes {
    vec![
        stroke![
            head((314, 270), 180, CCW),
            (225, 190), (165, 105), (138, 40),
            CORNER,
            (116, 150), (104, 280), (122, 410), (185, 492), (300, 531), (410, 512), (480, 450), (505, 350), (508, 40),
        ]
    ]
}

/// ค-style arch with the ฅ/ต dip in the middle of the top.
fn notched_top(left_hump_x: i32, dip: (i32, i32), right_x: i32) -> Vec<Item> {
    items![
        (112, 150), (100, 290), (115, 410), (155, 482), (left_hump_x, 518), dip,
        CORNER,
        (dip.0 + 105, 520), (right_x - 38, 496), (right_x - 8, 435), (right_x, 340), (right_x, 40),
    ]
}

/// ฅ
fn kho_khon() -> Strokes {
    vec![
        stroke![
            head((308, 270), 180, CCW),
            (222, 190), (162, 105), (133, 40),
            CORNER,
            notched_top(200, (300, 468), 500),
        ]
    ]
}

/// ฆ: notched head, down into a clockwise loop, diagonal, up the right
fn kho_rakhang() -> Strokes {
    vec![
        stroke![
            head((131, 470), 20, CW),
            (218, 466), (255, 490), (285, 532),
            CORNER,
            (272, 450), (238, 360), (210, 270), (203, 180),
            ring_r((138, 89), 12, -350, 64),
            (270, 118), (370, 78), (445, 50), (488, 40),
            CORNER,
            (494, 120), (494, 515),
        ]
    ]
}

/// ฆ as thai-notes writes it: head low on the left, up into two humps, down
/// a middle stem into a clockwise loop, diagonal, up the right
fn kho_rakhang_humps() -> Strokes {
    vec![
        stroke![
            head((211, 357), 180, CW),
            (158, 440), (190, 482), (250, 527),
            CORNER,
            (299, 484),
            CORNER,
            (342, 529), (400, 500), (433, 444), (431, 390), (400, 335), (375, 280), (372, 200),
            ring_r((300, 112), 0, -300, 72),
            (440, 190), (540, 125), (610, 64), (646, 40),
            CORNER,
            (650, 120), (650, 515),
        ]
    ]
}

/// ง: head top-right, down, along the bottom, up the diagonal
fn ngo_ngu() -> Strokes {
    vec![
        stroke![
            head((331, 466), -15, CW),
            (393, 380), (391, 300), (390, 150), (378, 85), (340, 45), (272, 36), (205, 38),
            CORNER,
            (48, 400),
        ]
    ]
}

/// จ: head, down, round the bottom and up the right, end top-left
fn cho_chan() -> Strokes {
    vec![
        stroke![
            head((141, 284), -5, CW),
            (206, 230), (205, 150), (203, 36),
            CORNER,
            (272, 36), (365, 65), (425, 150), (440, 270), (418, 400), (350, 490), (220, 530), (95, 515), (45, 470),
        ]
    ]
}

/// ฉ: จ with a clockwise tail loop at the bottom right
fn cho_ching() -> Strokes {
    vec![
        stroke![
            head((139, 278), -5, CW),
            (204, 220), (203, 130), (200, 38),
            CORNER,
            (270, 70), (345, 115),
            ring((469, 89), 150, -360),
            (462, 180), (482, 250), (486, 350), (461, 450), (394, 525), (283, 533), (180, 525), (95, 497), (58, 452),
        ]
    ]
}

/// ช
fn cho_chang() -> Strokes {
    vec![
        stroke![
            head((142, 465), 0, CW),
            (210, 420), (188, 355), (160, 295), (154, 220), (154, 36),
            CORNER,
            (380, 36), (412, 48), (426, 90), (428, 330), (425, 375), (445, 440), (460, 505), (476, 542),
        ]
    ]
}

/// ซ: ช with a notched head
fn so_so() -> Strokes {
    vec![
        stroke![
            head((114, 470), 20, CW),
            (205, 466), (240, 490), (268, 532),
            CORNER,
            (256, 470), (236, 395), (196, 310), (168, 230), (164, 150), (164, 36),
            CORNER,
            (390, 36), (422, 48), (436, 90), (437, 330), (434, 375), (455, 440), (472, 505), (489, 542),
        ]
    ]
}

/// ซ as thai-notes writes it: head low on the left, up into two humps, down
/// the left stem, along the bottom, up the right, flick
fn so_so_humps() -> Strokes {
    vec![
        stroke![
            head((100, 366), 180, CW),
            (48, 455), (80, 500), (130, 529),
            CORNER,
            (179, 480),
            CORNER,
            (220, 528), (270, 514), (297, 458), (298, 400), (265, 343), (237, 290), (236, 200), (236, 36),
            CORNER,
            (385, 36), (412, 48), (420, 90), (421, 340), (400, 400), (350, 450),
            CORNER,
            (392, 505), (440, 542), (481, 590),
        ]
    ]
}

/// ฌ: head bottom-left, ก-like left half, loop, diagonal, up the right
fn cho_choe() -> Strokes {
    vec![
        stroke![
            head((183, 86), 170, CW),
            (112, 200), (135, 280), (190, 325), (262, 348),
            CORNER, (70, 378), CORNER,
            (100, 440), (175, 505), (290, 530), (400, 510), (452, 452), (466, 350), (466, 200), (478, 150),
            ring((439, 86), 35, -360),
            (570, 105), (650, 70), (734, 42),
            CORNER,
            (740, 120), (740, 515),
        ]
    ]
}

/// ญ: body, then the เชิง as a second stroke from its loop
fn yo_ying() -> Strokes {
    vec![
        stroke![
            head((182, 87), 170, CW),
            (112, 200), (135, 280), (190, 325), (261, 348),
            arch_after_notch(463, 40),
            CORNER,
            (690, 36), (725, 48), (738, 90), (739, 515),
        ],
        stroke![head_r((489, -135), -90, CCW, 56), (580, -205), (680, -165), (745, -120), (772, -67)],
    ]
}

/// shared by ฎ ฏ: head (anticlockwise, like ภ), up from its right side,
/// notch, arch, right stem down below the line
fn cha_da_body() -> Vec<Item> {
    items![
        head((122, 94), 20, CCW),
        (190, 205), (205, 285), (240, 325), (292, 350),
        arch_after_notch(495, -150),
        (488, -188),
    ]
}

/// ฎ: foot loops anticlockwise and ends in the spike
fn do_chada() -> Strokes {
    vec![
        stroke![
            cha_da_body(),
            CORNER,
            (430, -170), (360, -120),
            ring((215, -139), 30, 330),
            (300, -80), (318, -12),
        ]
    ]
}

/// ฏ: foot zigzags before the loop
fn to_patak() -> Strokes {
    vec![
        stroke![
            cha_da_body(),
            CORNER,
            (445, -172), (410, -130), (388, -100),
            CORNER,
            (350, -190),
            CORNER,
            (312, -140),
            ring((188, -140), 35, 330),
            (270, -80), (285, -12),
        ]
    ]
}

/// ฐ: body from the head; foot from its own head at the right
fn tho_than() -> Strokes {
    vec![
        stroke![
            head((152, 240), -10, CW),
            (218, 170), (220, 36),
            CORNER,
            (385, 36), (451, 50), (495, 82), (522, 125), (527, 211), (516, 289), (490, 335), (440, 362), (275, 383), (104, 411),
            CORNER,
            (93, 445), (104, 489), (130, 517), (198, 529), (297, 528), (385, 517), (456, 517), (489, 556),
        ],
        stroke![
            head_r((392, -106), -15, CW, 56),
            (450, -175), (440, -222),
            CORNER, (396, -208), (335, -172),
            CORNER, (286, -228),
            CORNER,
            (250, -170), (200, -145), (140, -148), (80, -170), (55, -212), (75, -260), (120, -277), (170, -260), (190, -215), (215, -150), (235, -85),
        ],
    ]
}

/// ฑ: notched head, stem down, back up the diagonal, arch, down
fn tho_montho() -> Strokes {
    vec![
        stroke![
            head((110, 470), 20, CW),
            (200, 466), (240, 490), (268, 534),
            CORNER,
            (255, 470), (230, 400), (190, 330), (162, 260), (158, 150), (158, 36),
            CORNER,
            (215, 190), (265, 330), (305, 430), (350, 492), (400, 524), (458, 528), (485, 505), (497, 440), (499, 40),
        ]
    ]
}

/// ฒ: ต-like left half, loop, diagonal, up the right
fn tho_phuthao() -> Strokes {
    vec![
        stroke![
            head((293, 294), -45, CW),
            (290, 170), (230, 82), (182, 38), (155, 40),
            CORNER,
            (120, 150), (98, 300), (110, 420), (150, 490), (193, 519), (226, 508), (304, 469),
            CORNER,
            (370, 508), (408, 519), (460, 495), (490, 440), (499, 350), (499, 215), (510, 160),
            ring((481, 86), 40, -380),
            (610, 102), (700, 60), (767, 40),
            CORNER,
            (773, 120), (773, 511),
        ]
    ]
}

/// ณ: ญ-like body, then the น tail: diagonal, clockwise loop, up
fn no_nen() -> Strokes {
    vec![
        stroke![
            head((182, 87), 170, CW),
            (112, 200), (135, 280), (190, 325), (261, 348),
            arch_after_notch(465, 40),
            CORNER,
            (540, 72), (610, 112), (660, 122),
            ring((726, 89), 150, -360),
            (705, 170), (735, 240), (740, 330), (740, 511),
        ]
    ]
}

/// ด: head, down-left to the point, up, round, down the right
fn do_dek() -> Strokes {
    vec![
        stroke![
            head((296, 294), -45, CW),
            (290, 170), (230, 82), (188, 38), (155, 40),
            CORNER,
            (118, 150), (98, 300), (112, 420), (160, 490), (250, 525), (300, 531), (400, 515), (470, 460), (500, 380), (507, 300), (507, 40),
        ]
    ]
}

/// ต: ด with the dipped top
fn to_tao() -> Strokes {
    vec![
        stroke![
            head((296, 294), -45, CW),
            (290, 170), (230, 82), (188, 38), (155, 40),
            CORNER,
            (118, 150), (98, 300), (110, 420), (150, 490), (193, 519), (226, 508), (302, 469),
            CORNER,
            (370, 508), (408, 519), (462, 495), (495, 430), (505, 340), (507, 40),
        ]
    ]
}

/// ถ: head bottom-left, up, notch, arch, down the right (and ฤ, whose right
/// side runs on below the line)
fn tho_thung(bottom: i32) -> Strokes {
    vec![stroke![head((182, 87), 165, CW), (125, 190), (135, 262), (180, 305), (262, 348), arch_after_notch(474, bottom)]]
}

/// ท: head, stem down, back up the diagonal, arch, down
fn tho_thahan() -> Strokes {
    vec![
        stroke![
            head((120, 466), -10, CW),
            (184, 400), (180, 322), (180, 150), (180, 36),
            CORNER,
            (220, 150), (260, 280), (302, 420), (351, 489), (400, 516), (452, 518), (492, 480), (514, 400), (516, 40),
        ]
    ]
}

/// ธ: no head; starts at the top of the inner stem
fn tho_thong() -> Strokes {
    vec![
        stroke![
            (139, 250), (139, 150), (139, 36),
            CORNER,
            (380, 36), (420, 50), (442, 90), (448, 180), (445, 280), (425, 330), (390, 352), (250, 368), (94, 392),
            CORNER,
            (98, 440), (115, 490), (156, 520), (228, 530), (300, 522), (400, 510), (467, 517), (500, 556),
        ]
    ]
}

/// น: head, down, diagonal up, clockwise tail loop, up the right
fn no_nu() -> Strokes {
    vec![
        stroke![
            head((119, 466), -10, CW),
            (182, 400), (177, 330), (177, 150), (177, 36),
            CORNER,
            (243, 62), (330, 112), (385, 125),
            ring((468, 89), 150, -360),
            (455, 180), (478, 250), (481, 330), (481, 511),
        ]
    ]
}

/// บ (and ป with a tall right stem)
fn bo_baimai(top: i32) -> Strokes {
    vec![
        stroke![
            head((132, 466), -15, CW),
            (195, 400), (196, 320), (196, 150), (196, 36),
            CORNER,
            (440, 36), (485, 45), (505, 80), (512, 150), (512, top),
        ]
    ]
}

/// ผ (and ฝ): head on the right of the stem, anticlockwise
fn pho_phueng(top: i32) -> Strokes {
    vec![
        stroke![
            head((193, 466), 180, CCW),
            (131, 330), (131, 150), (131, 40),
            CORNER, (322, 225),
            CORNER, (494, 40),
            CORNER, (511, 120), (511, top),
        ]
    ]
}

/// พ (and ฟ)
fn pho_phan(top: i32) -> Strokes {
    vec![
        stroke![
            head((120, 466), -15, CW),
            (182, 400), (182, 300), (182, 40),
            CORNER, (378, 440),
            CORNER, (540, 40),
            CORNER, (571, 120), (571, top),
        ]
    ]
}

/// ภ: head bottom-left, anticlockwise, up from its right side (and ฦ, whose
/// right side runs on below the line)
fn pho_samphao(bottom: i32) -> Strokes {
    vec![
        stroke![
            head((122, 87), 20, CCW),
            (190, 200), (205, 280), (240, 322), (290, 348),
            CORNER, (90, 380), CORNER,
            (118, 440), (190, 505), (300, 531), (400, 512), (465, 455), (495, 360), (499, bottom),
        ]
    ]
}

/// ม: head, down into a clockwise loop, diagonal, up the right
fn mo_ma() -> Strokes {
    vec![
        stroke![
            head((146, 466), -10, CW),
            (206, 410), (205, 330), (205, 200),
            ring((153, 89), 30, -380),
            (285, 120), (370, 90), (440, 55), (498, 40),
            CORNER,
            (500, 120), (500, 511),
        ]
    ]
}

/// ย: head, down the left, jog out to the right, bowl, up the right
fn yo_yak() -> Strokes {
    vec![
        stroke![
            head((183, 468), 165, CCW),
            (100, 420), (112, 372), (148, 335), (200, 300), (300, 285),
            CORNER,
            (192, 266), (142, 232), (118, 175), (118, 110), (145, 55), (205, 36), (400, 36), (450, 48), (475, 90), (480, 150), (480, 515),
        ]
    ]
}

/// ร: head at the bottom, up the right, back along the bar, up, wavy top
fn ro_ruea() -> Strokes {
    vec![
        stroke![
            head((281, 87), 25, CCW),
            (345, 180), (345, 256), (323, 322), (279, 350), (186, 367), (77, 392),
            CORNER,
            (70, 440), (85, 489), (131, 522), (208, 529), (284, 517), (350, 511), (394, 520), (418, 556),
        ]
    ]
}

/// ล: head, inner arch, down the stem and back up it, over the top
fn lo_ling() -> Strokes {
    vec![
        stroke![
            head((166, 87), 165, CW),
            (108, 200), (118, 290), (150, 325), (206, 341), (278, 333), (339, 300), (385, 255), (441, 200), (441, 40),
            CORNER,
            (441, 200), (441, 340), (430, 420), (395, 470), (311, 522), (178, 528), (89, 500), (61, 450),
        ]
    ]
}

/// ว: head at the bottom right, up, over to the left
fn wo_waen() -> Strokes {
    vec![
        stroke![
            head((290, 87), 25, CCW),
            (348, 160), (349, 222), (349, 385), (330, 470), (272, 520), (170, 530), (80, 505), (45, 455),
        ]
    ]
}

/// ศ: ค-like body, then the tail flick
fn so_sala() -> Strokes {
    vec![
        stroke![
            head((314, 270), 180, CCW),
            (225, 190), (165, 105), (139, 40),
            CORNER,
            (115, 150), (100, 280), (115, 400), (165, 480), (250, 522), (330, 531), (417, 506), (455, 450), (490, 400), (500, 340), (500, 40),
        ],
        stroke![(488, 440), (525, 500), (565, 550), (617, 583)],
    ]
}

/// ษ: บ-like outline, then the inner loop and its crossing tail
fn so_ruesi() -> Strokes {
    vec![
        stroke![
            head((132, 466), -15, CW),
            (192, 400), (192, 300), (192, 36),
            CORNER,
            (480, 36), (520, 50), (533, 90), (535, 515),
        ],
        stroke![head_r((363, 307), -80, CCW, 58), (470, 246), (570, 300), (630, 330), (657, 381)],
    ]
}

/// ส: ล, then the tail flick
fn so_suea() -> Strokes {
    vec![
        stroke![
            head((166, 87), 165, CW),
            (110, 210), (132, 294), (182, 333), (248, 341), (330, 322), (385, 262), (443, 200), (443, 40),
            CORNER,
            (443, 200), (443, 330), (432, 400), (395, 470), (340, 515), (270, 532), (160, 522), (94, 500), (66, 450),
        ],
        stroke![(410, 430), (440, 480), (470, 532), (510, 562), (556, 583)],
    ]
}

/// ห: head, down, diagonal up to just left of the stem, then a closed
/// anticlockwise loop (up its right side, over, down its left, back right
/// through the crossing), down the right stem
fn ho_hip() -> Strokes {
    vec![
        stroke![
            head((120, 466), -15, CW),
            (181, 400), (181, 300), (181, 40),
            CORNER,
            (235, 160), (295, 295), (350, 375), (400, 418), (455, 425), (505, 462), (500, 522), (448, 544), (396, 515), (388, 462), (425, 428), (480, 400), (514, 340), (514, 40),
        ]
    ]
}

/// ฬ: พ, then up into an anticlockwise loop and the flick
fn lo_chula() -> Strokes {
    vec![
        stroke![
            head((120, 466), -15, CW),
            (182, 400), (182, 40),
            CORNER, (375, 395),
            CORNER, (524, 40),
            CORNER, (573, 120), (573, 470), (565, 535), (530, 570),
            oval((419, 616), -20, 360, 105, 72),
            (555, 650), (600, 715), (650, 756),
        ]
    ]
}

/// อ: head, down the left, round the bottom, up, over, end top-left
fn o_ang() -> Strokes {
    vec![
        stroke![
            head((184, 295), 190, CCW),
            (110, 220), (100, 150), (110, 80), (150, 40), (220, 34), (390, 34), (435, 50), (452, 100), (452, 420), (430, 480), (380, 520), (270, 531), (160, 520), (94, 500), (67, 456),
        ]
    ]
}

/// ฮ: อ whose top becomes a loop and a flick
fn ho_nokhuk() -> Strokes {
    vec![
        stroke![
            head((184, 286), 190, CCW),
            (110, 215), (100, 150), (110, 80), (150, 40), (220, 34), (400, 34), (445, 50), (461, 100), (461, 370), (452, 425),
            oval((262, 471), -15, 355, 160, 65),
            (470, 500), (506, 556), (556, 583),
        ]
    ]
}

// --- vowels, signs and marks ------------------------------------------------

/// ะ: two small heads with tails, top then bottom
fn sara_a() -> Strokes {
    let one = stroke![head_r((123, 432), -70, CCW, 58), (200, 380), (250, 386), (300, 410), (345, 443), (368, 515)];
    vec![one.clone(), shift(one, 0.0, -306.0)]
}

/// า (and ๅ with a long stem)
fn sara_aa(bottom: i32) -> Strokes {
    vec![stroke![(25, 500), (90, 522), (167, 530), (240, 512), (280, 460), (289, 400), (289, bottom)]]
}

/// the small circle of ำ, from the top, clockwise (taught) or anticlockwise
fn nikhahit(cw: bool) -> Vec<Pt> {
    stroke![ring_r((-130, 727), 90, if cw { -360 } else { 360 }, 70)]
}

/// ำ: circle over the consonant, then า
fn sara_am(cw: bool) -> Strokes {
    [vec![nikhahit(cw)], sara_aa(40)].concat()
}

/// ◌ิ is written from the right end of its base (the consonant's back line)
/// leftwards, then up and over the arch. ◌ี ◌ื add strokes that come down onto
/// the arch's end; ◌ึ goes on from ◌ิ into a circle on the arch's end.
fn arch() -> Vec<Pt> {
    pts![(-478, 720), (-445, 760), (-380, 785), (-300, 792)]
}

/// ิ, from the base's right end round to where it started
fn sara_i_items() -> Vec<Item> {
    items![(-135, 665), (-485, 665), CORNER, arch(), (-210, 778), (-160, 752), (-138, 705), (-134, 668)]
}

/// ิ
fn sara_i() -> Strokes {
    vec![stroke(sara_i_items())]
}

fn sara_ii_arch() -> Vec<Pt> {
    stroke![(-150, 665), (-485, 665), CORNER, arch(), (-225, 775), (-185, 745), (-152, 718)]
}

/// ี: ิ, then a stroke down onto its end
fn sara_ii() -> Strokes {
    vec![sara_ii_arch(), stroke![(-126, 805), (-126, 700)]]
}

/// ี in one stroke: ิ, then straight back up into the tick
fn sara_ii_one_stroke() -> Strokes {
    vec![stroke![sara_i_items(), CORNER, (-126, 805)]]
}

/// ึ: ิ, then back up into an anticlockwise circle on the arch's end
fn sara_ue() -> Strokes {
    vec![stroke![sara_i_items(), CORNER, ring_r((-112, 795), -115, 360, 62)]]
}

/// ึ: circle on the right, over the arch leftwards, then the base rightwards
fn sara_ue_circle_arch_base() -> Strokes {
    vec![
        stroke![
            head_r((-136, 735), 90, CCW, 60),
            (-220, 788), (-300, 792), (-380, 785), (-445, 760), (-478, 720),
            CORNER,
            (-485, 665), (-135, 665),
        ]
    ]
}

/// ึ: head on the right, along the base, arch back to the head
fn sara_ue_circle_first() -> Strokes {
    vec![
        stroke![
            head_r((-136, 735), -100, CW, 60),
            (-300, 665), (-485, 665),
            CORNER,
            arch(), (-240, 780), (-205, 750),
        ]
    ]
}

/// ื: ี, then one more stroke inside
fn sara_uee() -> Strokes {
    vec![sara_ii_arch(), stroke![(-126, 805), (-126, 700)], stroke![(-228, 818), (-205, 742)]]
}

/// ุ
fn sara_u() -> Strokes {
    vec![stroke![head_r((-196, -153), -10, CW, 58), (-112, -200), (-115, -240), (-135, -272)]]
}

/// ู
fn sara_uu() -> Strokes {
    vec![
        stroke![
            head_r((-347, -143), -10, CW, 55),
            (-290, -185), (-285, -240), (-265, -278), (-200, -283), (-150, -270), (-128, -230), (-126, -110),
        ]
    ]
}

/// เ: head at the bottom, up the stem
fn sara_e(dx: i32) -> Strokes {
    vec![stroke![head_r((209 + dx, 87), 165, CW, 60), (150 + dx, 170), (148 + dx, 240), (148 + dx, 514)]]
}

/// แ: two เ, left first
fn sara_ae() -> Strokes {
    [sara_e(0), sara_e(286)].concat()
}

/// โ: head at the bottom, up the stem, hook left, wavy cap
fn sara_o() -> Strokes {
    vec![
        stroke![
            head_r((245, 87), 165, CW, 60),
            (183, 200), (183, 600), (175, 690), (140, 740), (90, 762), (52, 772),
            CORNER,
            (58, 830), (90, 870), (154, 885), (269, 879), (337, 874), (371, 908),
        ]
    ]
}

/// ใ: head, up, round anticlockwise and curl in
fn sara_ai_maimuan() -> Strokes {
    vec![
        stroke![
            head_r((245, 87), 165, CW, 60),
            (185, 200), (185, 540), (205, 580), (250, 610), (292, 665), (313, 760), (300, 840), (260, 880), (190, 892), (115, 875), (60, 835),
            (34, 770), (40, 712), (70, 683), (114, 676), (160, 695), (176, 745), (155, 790), (115, 800), (85, 788),
        ]
    ]
}

/// ไ: head, up, round, down to the point, flick up-left
fn sara_ai_maimalai() -> Strokes {
    vec![
        stroke![
            head_r((227, 87), 165, CW, 60),
            (162, 200), (162, 540), (182, 575), (239, 605), (285, 660), (298, 740), (290, 808), (256, 860), (205, 890), (165, 875), (130, 820), (100, 758),
            CORNER,
            (-25, 918),
        ]
    ]
}

/// ั
fn mai_han_akat() -> Strokes {
    vec![stroke![head_r((-282, 737), -75, CCW, 58), (-200, 672), (-106, 700), (-35, 745), (-5, 812)]]
}

/// ็: head on the right, zigzag, round up the left, wavy top. Taught with the
/// head clockwise, leaving from its bottom heading left.
fn mai_taikhu(cw: bool) -> Strokes {
    let head = if cw { head_r((-179, 713), -100, CW, 48) } else { head_r((-179, 713), 190, CCW, 48) };
    vec![
        stroke![
            head,
            (-265, 676),
            CORNER, (-320, 742), CORNER,
            (-360, 672), (-410, 665), (-440, 720), (-440, 780), (-405, 850), (-315, 872), (-213, 866), (-124, 872), (-101, 900),
        ]
    ]
}

/// ่
fn mai_ek() -> Strokes {
    vec![stroke![(-145, 810), (-145, 675)]]
}

/// ้: head, down, flick up to the right
fn mai_tho() -> Strokes {
    vec![
        stroke![
            head_r((-307, 805), 0, CW, 55),
            (-258, 750), (-282, 702), (-298, 665),
            CORNER,
            (-222, 667), (-144, 680), (-90, 770), (-44, 855),
        ]
    ]
}

/// ๊: head, up, dipped top, down, flick up to the right
fn mai_tri() -> Strokes {
    vec![
        stroke![
            head_r((-350, 709), 160, CW, 48),
            (-395, 790), (-356, 850), (-300, 848), (-244, 828),
            CORNER,
            (-200, 850), (-165, 810), (-157, 740), (-150, 680), (-128, 672), (-72, 722), (-44, 800), (-33, 835),
        ]
    ]
}

/// ๋: down, then across
fn mai_chattawa() -> Strokes {
    vec![stroke![(-140, 855), (-140, 645)], stroke![(-265, 750), (-15, 750)]]
}

/// ์: head, flick up to the right
fn thanthakhat() -> Strokes {
    vec![stroke![head_r((-190, 714), 150, CW, 52), (-225, 800), (-156, 812), (-104, 845), (-46, 881)]]
}

/// ฯ: head, tail up to the top of the stem, down the stem
fn paiyannoi() -> Strokes {
    vec![
        stroke![
            head((106, 468), -80, CCW),
            (200, 400), (295, 444), (345, 480), (356, 515),
            CORNER,
            (356, 40),
        ]
    ]
}

/// ๆ: head, up the left, dipped top, down the long stem
fn mai_yamok() -> Strokes {
    vec![
        stroke![
            head((169, 292), 165, CW),
            (100, 370), (86, 420), (100, 470), (150, 518), (191, 510), (248, 469),
            CORNER,
            (320, 508), (360, 520), (400, 492), (420, 420), (421, -185),
        ]
    ]
}

/// ๆ with the tail kinked left partway down (thai-notes)
fn mai_yamok_kinked() -> Strokes {
    vec![
        stroke![
            head((169, 292), 165, CW),
            (100, 370), (86, 420), (100, 470), (150, 518), (191, 510), (248, 469),
            CORNER,
            (320, 508), (360, 520), (400, 492), (420, 420), (421, 200), (405, 145), (340, 40), (292, -45), (276, -110), (275, -185),
        ]
    ]
}

// --- digits ----------------------------------------------------------------

/// ๐: anticlockwise from the top
fn digit_0() -> Strokes {
    vec![stroke![oval((325, 258), 90, 360, 225, 232)]]
}

/// ๑: head, then the big clockwise sweep out to the tail
fn digit_1() -> Strokes {
    vec![
        stroke![
            head((272, 113), 225, CW),
            (160, 95), (120, 160), (100, 250), (122, 361), (189, 439), (300, 489), (411, 483), (511, 417), (556, 250), (539, 139), (489, 56), (428, -17),
        ]
    ]
}

/// ๒: head, up, dipped top, down the right, along the bottom, up the left
fn digit_2() -> Strokes {
    vec![
        stroke![
            head((401, 258), 180, CW),
            (330, 330), (335, 410), (360, 460), (400, 482), (467, 436),
            CORNER,
            (520, 470), (575, 484), (625, 460), (650, 400), (655, 300), (655, 150), (640, 80), (600, 45), (530, 36), (200, 36), (160, 45), (135, 80), (128, 150), (128, 583),
        ]
    ]
}

/// The left half of ๓ ๗ ๙: up from the head and over the first hump.
fn humps_to_stub(stub_x: i32) -> Vec<Pt> {
    pts![(125, 180), (112, 280), (133, 389), (178, 456), (228, 489), (300, 483), (340, 440), (stub_x, 394)]
}

/// ๓: head, first hump, down the short middle stem and back up, second hump
fn digit_3() -> Strokes {
    vec![
        stroke![
            head((222, 84), 170, CW),
            humps_to_stub(378), (378, 250),
            CORNER,
            (380, 390), (405, 450), (440, 480), (500, 487), (567, 467), (611, 422), (630, 333), (628, 256), (611, 144), (600, 40), (556, 2),
        ]
    ]
}

/// ๔ (and the start of ๕): head, down, along the bottom, up the left.
fn digit_4_to(top: Vec<Pt>) -> Vec<Item> {
    items![
        head((436, 254), 190, CCW),
        (383, 160), (420, 105), (460, 55), (475, 36),
        CORNER,
        (400, 28), (322, 28), (222, 50), (144, 106), (96, 206), (100, 339), (156, 417), top,
    ]
}

/// ๔
fn digit_4() -> Strokes {
    vec![stroke![digit_4_to(pts![(256, 472), (356, 481), (444, 483), (511, 494), (561, 533), (583, 589), (617, 618)])]]
}

/// ๕: ๔ whose top bar makes an anticlockwise loop on the way
fn digit_5() -> Strokes {
    vec![
        stroke![
            digit_4_to(pts![(230, 468), (300, 494)]),
            ring_r((362, 562), -90, 360, 64),
            (430, 494), (511, 500), (561, 533), (583, 590), (617, 620),
        ]
    ]
}

/// ๖: head, round the bottom and up the right, over, flick up-left
fn digit_6() -> Strokes {
    vec![
        stroke![
            head((168, 138), -120, CCW),
            (199, 39), (282, 24), (382, 44), (460, 100), (510, 222), (504, 333), (460, 422), (382, 472), (299, 489), (238, 483), (183, 456), (150, 415), (115, 398),
            CORNER,
            (85, 470), (45, 615),
        ]
    ]
}

/// ๗: ๓, then along the bottom and up the tall right stem
fn digit_7() -> Strokes {
    vec![
        stroke![
            head((222, 84), 170, CW),
            humps_to_stub(362), (362, 270),
            CORNER,
            (365, 390), (390, 440), (417, 475), (467, 489), (522, 483), (572, 444), (598, 344), (600, 222), (598, 40),
            CORNER,
            (667, 38), (730, 48), (775, 90), (800, 170), (806, 260), (806, 583),
        ]
    ]
}

/// ๘: head, down, zigzag along the bottom, up the left, over, flick
fn digit_8() -> Strokes {
    vec![
        stroke![
            head((507, 238), -10, CW),
            (575, 160), (556, 80), (478, 36), (400, 40), (345, 100), (311, 150),
            CORNER, (260, 90), (222, 44), CORNER,
            (185, 48), (140, 90), (100, 200), (100, 344), (144, 422), (222, 472), (300, 489), (400, 489), (478, 494), (533, 528), (572, 590), (622, 628),
        ]
    ]
}

/// ๙: head, hump, down the diagonal; then the second hump and flick
fn digit_9() -> Strokes {
    vec![
        stroke![head((228, 84), 170, CW), (125, 180), (111, 289), (144, 422), (183, 472), (244, 489), (300, 478), (356, 417), (389, 392), (420, 320), (533, 40), (560, 20)],
        stroke![(392, 400), (410, 440), (440, 475), (500, 489), (560, 470), (600, 430), (630, 420), CORNER, (645, 470), (656, 589), (700, 628)],
    ]
}

/// ๙ with a ๑-style head (thai-notes): spiral out clockwise from inside the
/// bowl, down its right side, round the bowl, down the diagonal; then the
/// second hump and flick
fn digit_9_spiral() -> Strokes {
    vec![
        stroke![head_r((262, 300), -20, CW, 58), (318, 210), (290, 110), (230, 45), (160, 38), (118, 100), (111, 200), (125, 330), (170, 440), (244, 489), (300, 478), (356, 417), (389, 392), (420, 320), (533, 40), (560, 20)],
        digit_9()[1].clone(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate;

    #[test]
    fn segments_syllables() {
        assert_eq!(segment("สวัสดี ครับ"), ["ส", "วั", "ส", "ดี", " ", "ค", "รั", "บ"]);
        assert_eq!(segment("น้ำ ที่ เก็บ ใหญ่"), ["น้", "ำ", " ", "ที่", " ", "เ", "ก็", "บ", " ", "ใ", "ห", "ญ่"]);
        // A mark with no consonant before it is its own unit.
        assert_eq!(segment("ัก เิ"), ["ั", "ก", " ", "เ", "ิ"]);
        assert_eq!(segment("ฤๅ ๑๒"), ["ฤ", "ๅ", " ", "๑", "๒"]);
        assert!(segment("").is_empty());
    }

    /// The box y range of some of a glyph's strokes.
    fn ys(g: &StrokeGlyph, strokes: impl std::slice::SliceIndex<[language_utils::Stroke], Output = [language_utils::Stroke]>) -> (f32, f32) {
        g.strokes[strokes].iter().flat_map(|s| &s.points).fold((1.0, 0.0), |(a, b), p| (a.min(p.1), b.max(p.1)))
    }

    fn xs(g: &StrokeGlyph, strokes: impl std::slice::SliceIndex<[language_utils::Stroke], Output = [language_utils::Stroke]>) -> (f32, f32) {
        g.strokes[strokes].iter().flat_map(|s| &s.points).fold((1.0, 0.0), |(a, b), p| (a.min(p.0), b.max(p.0)))
    }

    #[test]
    fn composes_syllables() {
        let t = Thai::default();
        // Taught stroke counts.
        for (unit, strokes) in [("รั", 2), ("ห้", 2), ("น้", 2), ("ที่", 4), ("ปั", 2), ("ญุ", 2), ("ฐู", 2), ("ฎุ", 2), ("ดื้", 5), ("ปั้", 3), ("นํ้", 3)] {
            assert_eq!(t.glyphs(unit)[0].strokes.len(), strokes, "{unit}");
        }
        // The taught form.
        let one = |unit: &str| t.glyphs(unit).remove(0);
        // The consonant is drawn first, exactly as on its own.
        assert_eq!(one("รั").strokes[..1], one("ร").strokes[..]);
        // On อ, marks land exactly where they do alone.
        assert_eq!(one("อั่").strokes, [one("อ").strokes, one("ั").strokes, one("่").strokes].concat());
        // A tone mark with no above-vowel sits a tier lower than alone.
        let (lone, on_ha) = (ys(&one("้"), ..), ys(&one("ห้"), 1..));
        assert!((on_ha.0 - lone.0 - 0.147).abs() < 0.002, "{lone:?} {on_ha:?}");
        // With one, it sits over it.
        assert!(ys(&one("ที่"), 3..).1 < ys(&one("ที่"), 1..3).0);
        assert!(ys(&one("ดื้"), 4..).1 < ys(&one("ดื้"), 1..4).0);
        // ั goes left of ป's tall stem, at its normal height.
        let pa = one("ปั");
        assert!(xs(&pa, 1..).1 < xs(&pa, ..1).1 - 0.05);
        assert_eq!(ys(&pa, 1..), ys(&one("บั"), 1..));
        // ุ drops below ฎ's foot.
        let da = one("ฎุ");
        assert!(ys(&da, 1..).0 > ys(&da, ..1).1);
        // ญ loses its เชิง instead, and ุ sits where it does under น.
        assert_eq!(one("ญุ").strokes[..1], one("ญ").strokes[..1]);
        assert_eq!(ys(&one("ญุ"), 1..), ys(&one("นุ"), 1..));
        // A tone mark on ื sits midway between its ticks.
        let (ticks, tone) = (xs(&one("ดื"), 2..), xs(&one("ดื่"), 4..));
        assert!(((ticks.0 + ticks.1) / 2.0 - tone.0).abs() < 0.005, "{ticks:?} {tone:?}");
        // Marks with alternatives multiply the forms, the taught ones first.
        let forms = t.glyphs("กํ๋");
        assert_eq!(forms.len(), 4);
        assert_eq!(forms[0].strokes[..2], one("กํ").strokes[..]);
        let width = |g: &StrokeGlyph| xs(g, 2..3).1 - xs(g, 2..3).0;
        // Taught ๋ starts with its vertical stroke; the alternative with the bar.
        assert!(width(&forms[0]) < 0.01 && width(&forms[1]) > 0.1);
        assert_ne!(forms[2].strokes[1], forms[0].strokes[1]);
    }

    #[test]
    fn lone_marks_keep_their_glyph() {
        let t = Thai::default();
        for &c in t.marks.keys() {
            let unit = c.to_string();
            assert_eq!(segment(&unit), [unit.as_str()]);
            assert_eq!(t.glyphs(&unit), t.units[&c]);
        }
        // Phinthu and yamakkan are segmented but not drawn.
        assert!(t.glyphs("ก\u{e3a}").is_empty());
    }

    #[test]
    fn every_stack_fits_the_box() {
        let t = Thai::default();
        let marks = |tier| t.marks.iter().filter(move |m| m.1.0 == tier).map(|m| *m.0);
        let above = || std::iter::once(None).chain(marks(Tier::Above).map(Some));
        let below = || std::iter::once(None).chain(marks(Tier::Below).map(Some));
        let tone = || std::iter::once(None).chain(marks(Tier::Tone).map(Some));
        for &c in t.consonants.keys() {
            for (b, a, n) in below().flat_map(|b| above().flat_map(move |a| tone().map(move |n| (b, a, n)))) {
                let unit: String = [Some(c), b, a, n].into_iter().flatten().collect();
                let forms: usize = unit.chars().skip(1).map(|m| t.marks[&m].1.len()).product::<usize>() * t.consonants[&c].len();
                let glyphs = t.glyphs(&unit);
                assert_eq!(glyphs.len(), forms, "{unit}");
                for g in &glyphs {
                    validate(g).unwrap_or_else(|e| panic!("{unit}: {e}"));
                }
            }
        }
    }
}
