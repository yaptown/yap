//! Hand-authored Devanagari (Hindi) stroke order.
//!
//! No usable published dataset exists, so every letter below is drawn by hand
//! as directed pen centerlines, in the order Indian schools teach (TuitMob swar
//! and vyanjan worksheets were the primary reference): body first, left to
//! right and top to bottom, the vertical stem after the part left of it, and
//! the shirorekha (headline) last, left to right. Segments a worksheet numbers
//! separately but that the pen draws without lifting (the "3" of अ, the S of
//! इ) are one stroke here.
//!
//! Letters are authored in design units: Noto Sans Devanagari font units with
//! y flipped, so x runs from the glyph origin, the headline centre is y=150
//! and the baseline (bottom of a stem) is y=740. [`Fit`] maps a letter into
//! the 1000-unit box: centred horizontally, headline kept at y=150, shrunk
//! only if wider than the box allows. Marks keep their position relative to
//! the consonant instead of being centred: they are drawn around a notional प
//! whose origin is x=0, placed where a centred प would sit.
//!
//! # Accepted letterforms
//!
//! A letter listed twice in [`letters`] has an accepted alternative form
//! (index 0 is the taught, Noto-like form):
//! - अ आ ओ औ ऑ: the headline over the whole letter, the "3" below it, as
//!   many hands write it (Noto keeps it over the stem only). थ ध भ get no
//!   such form on purpose: their headline gap is what tells them from य घ म.
//! - ख: the left part closed into a loop where it turns (vs Noto's open
//!   hook).
//! - झ: the older Uttara (northern) form from Wikipedia's "Jha (Indic)" /
//!   Commons `Devanagari_jh_old.svg`: a hooked left stroke with a bar to the
//!   stem, and a क-like hook right of the stem.
//! - छ: the headline over the right bowl only, leaving the upper loop open.
//!
//! A review against more references added these. Sources: TM = TuitMob
//! worksheets; OP, JP, SM = the Wikimedia Commons stroke-order sets by
//! Opiaterein (CC BY 3.0), JackPotte (CC BY-SA 3.0) and Saurmandal (CC BY-SA
//! 3.0); UT = UT Austin Hindi Urdu Flagship's handwriting video of the vowel
//! signs. Each rests on one or two sources, so the taught forms stay:
//! - ई's hook and ऐ's sign after the headline (TM).
//! - ऐ ओ औ with their signs drawn up from the headline (SM, UT).
//! - ऊ with its tail after the headline (JP).
//! - श with the headline over the stem only (OP, JP).
//! - ग: the knob first, then the short stem up to the headline (OP).
//! - ल: the stem first, then the body from it round to the tail (JP).
//! - ष: the diagonal after the stem (JP).
//!
//! Vowel signs and signs above have their own alternatives ([`mark`]):
//! - ि in one stroke, up the stem from its foot into the hook (UT).
//! - ी in one stroke, the hook from over the consonant's stem, then on down
//!   the stem of ी (UT).
//! - ु from its left end, sweeping right and curling up under the stem (UT).
//! - ू in one stroke, round the loop and out along the tail (UT).
//! - े ै ो ौ drawn up from the headline (SM, UT).
//! - ं ः before the headline (TM), and ँ with them (arc and dot, as one sign).
//!
//! The review also claimed the loops of थ ध भ turn the other way in OP and JP;
//! frame by frame both turn them as we do (थ भ clockwise, ध anticlockwise) and
//! differ only in where the loop starts, so no form was added for that.
//!
//! Nukta letters take every form of their base. An akshara's forms are the
//! product of its letters' forms and then its marks' forms, the first letter
//! varying slowest, so index 0 is every taught form.
//!
//! ळ (Marathi, also Konkani and Rajasthani) is drawn against Noto Sans
//! Devanagari: a stub from the headline into a figure-eight lying on its
//! side, no stem; ऩ ऱ ऴ are न र ळ with the nukta.
//!
//! # The writable unit: the akshara
//!
//! Hindi is written in aksharas, not letters: [`segment`] splits text into
//! consonant clusters (`C(्C)*`, each consonant optionally with a nukta) or
//! independent vowels, digits and signs, each with its following combining
//! marks. [`Devanagari::glyph`] composes any such unit from the structural
//! letter model below; a single letter or a lone mark is just the smallest
//! akshara, and composes to exactly what was drawn and reviewed on its own.
//!
//! # Structural model
//!
//! A [`Letter`] is its body strokes, an optional full-height stem, the strokes
//! drawn after the stem (the hook of क and फ, the signs of ओ औ ऑ), its
//! headline extent (which leaves the gaps of अ थ ध भ) and, for nukta letters,
//! the dot. The full letter is body, stem, tail, headline, nukta, the order the
//! worksheets teach; an alternative form may draw its stem first or put
//! strokes after the headline (before the nukta). Marks are anchored to a [`Frame`] measured from what they
//! attach to: the left edge (ि), the right stem (े ै ं ँ), the right edge
//! (ा ी ो ौ ॉ ः) and the foot of the last consonant (ु ू ृ ्).
//!
//! # Conjuncts
//!
//! Conjuncts follow the rule the Cambridge Introduction to Sanskrit (CIS,
//! chapter 1) teaches: a consonant with a stem drops it to form its half form,
//! and the next consonant follows in full to its right. क and फ keep a short
//! stem with their hook. Half forms are packed left to right by ink, [`GAP`]
//! apart; chains (स्त्र) are the same rule repeated. One headline is drawn
//! across the cluster after all bodies (its gaps survive: स्थ keeps थ's),
//! then the marks. The whole akshara is scaled down about the headline if it
//! is wider than the box or reaches too far below it.
//!
//! None of the references teach a stroke order for conjuncts, so the order
//! below extends the single-letter rule (bodies left to right, headline after
//! the bodies). The review confirmed the letter bodies against TM, OP, JP and
//! SM; the structure of the vowel signs rests on UT alone; the digits, the
//! nukta letters, ळ and most conjuncts are still unverified by any source.
//! Judgment calls, from the CIS list of forms to memorise and from Noto Sans
//! Devanagari as a check on the shapes:
//!
//! - **Ligatures** drawn as their own letters: क्ष त्र ज्ञ श्र द्ध द्व द्य
//!   ह्म त्त. क्त and च्छ are taught and printed as the half-form rule, so
//!   they compose; so do श्व and श्च (the half श, not the loop variant some
//!   fonts use).
//! - **Stemless first consonants** (ट ठ ड ढ द ह छ ङ र ळ, and the stemless
//!   nukta letters) stack only in ट्ट ट्ठ ड्ड ड्ढ (the lower letter, smaller
//!   and without its headline, hangs under the upper one's stub). Every other
//!   stemless first consonant outside the ligatures is written in full with an
//!   explicit virama under it, drawn right after its body, which is how the
//!   Central Hindi Directorate's standard spelling writes them (विद्‌या,
//!   चिह्‌न); द्भ द्म द्द, stacks in Sanskrit type, are written that way too.
//!   ZWNJ after a virama asks for the same with any consonant.
//! - **Reph** (र before a consonant) is a hook rising from the headline over
//!   the akshara's rightmost stem (a ा ी ो ौ stem if there is one), drawn
//!   after every other mark except ं and ँ, which move right of it and come
//!   last.
//! - **Rakar** (्र after a consonant) is drawn after that consonant's body and
//!   stem and before the headline: a diagonal down-left from the stem, or from
//!   द's tail as Noto draws it, and a ^ under the lowest point of the other
//!   stemless letters (ट्र ड्र).
//! - **Mark order**: consonant bodies, stems and rakar; then the strokes of
//!   marks that hang from the headline (the stems and hooks of ि ा ी ो ौ ॉ,
//!   in the order the single marks were drawn; ि after the cluster although
//!   it sits left of it, consonant before vowel sign as the akshara is spelled
//!   and read, since no reference shows ि's order on a letter); then the one
//!   headline; then nuktas, the marks above and below in text order, reph, and
//!   ं ँ.
//! - रु and रू attach to the middle of र, as every Hindi primer shows them.
use super::{Ends, Pt, Strokes, bounds, centripetal, glyph, group, strokes};
use language_utils::{StrokeGlyph, StrokeStandard};
use rustc_hash::FxHashMap;

const HEADLINE: i32 = 150;
const BASELINE: i32 = 740;
const MAX_WIDTH: f64 = 860.0;
/// The lowest an akshara may reach in the box before [`Fit`] shrinks it about
/// the headline; no single letter comes near it.
const MAX_BOTTOM: f64 = 975.0;
/// Where the notional प (ink 0..583) sits when centred in the box.
const MARK_ORIGIN_X: f64 = 209.0;
/// The stem of ा and the marks built on it, relative to the notional प.
const AA_STEM: i32 = 700;
/// Ink gap between consecutive consonants of a cluster.
const GAP: f64 = 40.0;
/// Headline pieces closer than this are drawn as one stroke; the gaps over
/// the loops of थ ध भ are wider.
const HEAD_GAP: f64 = 200.0;
/// How much a stack squeezes its upper letter vertically and shrinks the
/// lower one.
const STACK_SCALE: f64 = 0.72;

const NUKTA: char = '\u{93c}';
const VIRAMA: char = '\u{94d}';
const ZWNJ: char = '\u{200c}';
const ZWJ: char = '\u{200d}';

fn is_consonant(c: char) -> bool {
    matches!(c, '\u{915}'..='\u{939}' | '\u{958}'..='\u{95f}')
}

/// Combining marks: signs above, vowel signs, nukta, virama, stress marks and
/// the vocalic ॢ ॣ (not ऽ, which is a letter).
fn is_mark(c: char) -> bool {
    matches!(c, '\u{900}'..='\u{903}' | '\u{93a}'..='\u{93c}' | '\u{93e}'..='\u{94f}' | '\u{951}'..='\u{957}' | '\u{962}' | '\u{963}')
}

/// Devanagari characters that carry marks without being consonants:
/// independent vowels, digits, danda and ॐ.
fn is_other_base(c: char) -> bool {
    matches!(c, '\u{904}'..='\u{914}' | '\u{950}' | '\u{960}' | '\u{961}' | '\u{964}'..='\u{96f}' | '\u{972}'..='\u{97f}')
}

/// Splits text into its writable units in order, skipping nothing: aksharas,
/// and every other character (spaces, Latin, a stray ZWJ) on its own.
pub fn segment(text: &str) -> Vec<&str> {
    let mut units = Vec::new();
    let mut rest = text;
    while let Some(first) = rest.chars().next() {
        let mut chars = rest.char_indices().peekable();
        chars.next();
        if is_consonant(first) {
            chars.next_if(|&(_, c)| c == NUKTA);
            loop {
                // virama, an optional ZWJ/ZWNJ, then a consonant continues the cluster
                let mut look = chars.clone();
                if look.next().map(|(_, c)| c) != Some(VIRAMA) {
                    break;
                }
                look.next_if(|&(_, c)| c == ZWJ || c == ZWNJ);
                if look.next().is_none_or(|(_, c)| !is_consonant(c)) {
                    break;
                }
                look.next_if(|&(_, c)| c == NUKTA);
                chars = look;
            }
        }
        if is_consonant(first) || is_other_base(first) {
            while chars.next_if(|&(_, c)| is_mark(c)).is_some() {}
        }
        let len = chars.peek().map_or(rest.len(), |&(i, _)| i);
        units.push(&rest[..len]);
        rest = &rest[len..];
    }
    units
}

/// The Devanagari letters, built once; aksharas are composed from them on
/// demand.
pub struct Devanagari {
    /// Every accepted form of each letter, the taught form first.
    letters: FxHashMap<char, Vec<Letter>>,
    ligatures: FxHashMap<(char, char), Vec<Letter>>,
}

impl Default for Devanagari {
    fn default() -> Self {
        let mut letters = group(letters());
        for (c, base, x, y) in NUKTA_LETTERS {
            let forms = letters[&base].iter().map(|l| l.clone().nukta(x, y)).collect();
            assert!(letters.insert(c, forms).is_none());
        }
        let mut ligatures: FxHashMap<(char, char), Vec<Letter>> = ligatures().into_iter().map(|(pair, l)| (pair, vec![l])).collect();
        for (top, bottom) in STACKS {
            let forms = letters[&top].iter().flat_map(|t| letters[&bottom].iter().map(|b| stack(t, b))).collect();
            assert!(ligatures.insert((top, bottom), forms).is_none());
        }
        Devanagari { letters, ligatures }
    }
}

/// Nukta letters: the base letter, then the dot below (added once the
/// letter, headline included, is complete).
const NUKTA_LETTERS: [(char, char, i32, i32); 11] = [
    ('\u{929}', 'न', 220, 800), // ऩ
    ('\u{931}', 'र', 150, 800), // ऱ
    ('\u{934}', 'ळ', 400, 800), // ऴ
    ('\u{958}', 'क', 150, 800), // क़
    ('\u{959}', 'ख', 120, 800), // ख़
    ('\u{95a}', 'ग', 160, 800), // ग़
    ('\u{95b}', 'ज', 260, 800), // ज़
    ('\u{95c}', 'ड', 280, 820), // ड़
    ('\u{95d}', 'ढ', 290, 820), // ढ़
    ('\u{95e}', 'फ', 165, 800), // फ़
    ('\u{95f}', 'य', 250, 800), // य़
];

/// Consonant pairs written as a stack, upper letter first.
const STACKS: [(char, char); 4] = [('ट', 'ट'), ('ट', 'ठ'), ('ड', 'ड'), ('ड', 'ढ')];

/// One consonant, ligature or stack of a cluster, or the base of any other
/// akshara, with every form it may be written in.
struct Slot<'a> {
    forms: &'a [Letter],
    consonant: Option<char>,
    /// The next consonant follows a virama + ZWNJ.
    before_zwnj: bool,
}

/// A slot written in one of its forms.
struct Part<'a> {
    letter: &'a Letter,
    /// The plain consonant, for the shapes that depend on it (रु, द्र).
    consonant: Option<char>,
    /// Drawn in full with a virama under it instead of as a half form.
    virama: bool,
    rakar: bool,
}

impl Devanagari {
    /// Every accepted form of one unit from [`segment`], the taught form
    /// first; empty if it is not a Devanagari akshara this pack can draw.
    /// ZWJ is ignored; ZWNJ after a virama keeps the virama visible.
    pub fn glyphs(&self, unit: &str) -> Vec<StrokeGlyph> {
        let forms = self.draw(unit).unwrap_or_default();
        forms.into_iter().map(|(strokes, fit)| glyph(StrokeStandard::Devanagari, strokes, |p| fit.to_box(p))).collect()
    }

    /// Each form of the unit in design units, and where it goes in the box.
    /// An akshara's forms are every combination of its letters' forms, the
    /// first letter's choice varying slowest; the first is all taught forms.
    fn draw(&self, unit: &str) -> Option<Vec<(Strokes, Fit)>> {
        let chars: Vec<char> = unit.chars().filter(|&c| c != ZWJ).collect();
        if let [c] = chars[..]
            && let Some(forms) = mark(c, &Frame::NOTIONAL_PA)
        {
            let forms = forms.into_iter().map(|(pre, head, post)| (strokes![..pre, ..head.map(|(a, b)| vec![(a, 150.0), (b, 150.0)]), ..post], Fit::MARK));
            return Some(forms.collect());
        }
        let mut chars = chars.into_iter().peekable();
        let first = chars.next()?;
        let mut slots = Vec::new();
        let mut reph = false;
        let mut rakar = false;
        if is_consonant(first) {
            // (consonant, preceded by virama + ZWNJ)
            let mut cluster = vec![(with_nukta(first, &mut chars)?, false)];
            loop {
                let mut look = chars.clone();
                if look.next() != Some(VIRAMA) {
                    break;
                }
                let zwnj = look.next_if_eq(&ZWNJ).is_some();
                match look.next() {
                    Some(c) if is_consonant(c) => {
                        chars = look;
                        cluster.push((with_nukta(c, &mut chars)?, zwnj));
                    }
                    _ => break,
                }
            }
            if cluster.len() > 1 && cluster[0].0 == 'र' && !cluster[1].1 {
                reph = true;
                cluster.remove(0);
            }
            let n = cluster.len();
            rakar = n > 1 && cluster[n - 1] == ('र', false) && !self.ligatures.contains_key(&(cluster[n - 2].0, 'र'));
            if rakar {
                cluster.pop();
            }
            let mut i = 0;
            while i < cluster.len() {
                let (c, _) = cluster[i];
                let ligature = cluster.get(i + 1).filter(|next| !next.1).and_then(|next| self.ligatures.get(&(c, next.0)));
                let forms = match ligature {
                    Some(l) => l,
                    None => self.letters.get(&c)?,
                };
                i += if ligature.is_some() { 2 } else { 1 };
                slots.push(Slot { forms, consonant: ligature.is_none().then_some(c), before_zwnj: cluster.get(i).is_some_and(|next| next.1) });
            }
        } else {
            slots.push(Slot { forms: self.letters.get(&first)?, consonant: None, before_zwnj: false });
        }
        let marks: Vec<char> = chars.filter(|&c| c != ZWNJ).collect();
        let choices = slots.iter().fold(vec![vec![]], |choices: Vec<Vec<usize>>, slot| {
            choices.into_iter().flat_map(|choice| (0..slot.forms.len()).map(move |k| [choice.as_slice(), &[k]].concat())).collect()
        });
        let forms = choices
            .into_iter()
            .map(|choice| {
                let parts: Vec<Part> = slots
                    .iter()
                    .zip(choice)
                    .enumerate()
                    .map(|(i, (slot, k))| {
                        let letter = &slot.forms[k];
                        let last = i + 1 == slots.len();
                        Part { letter, consonant: slot.consonant, virama: !last && (letter.stem.is_none() || slot.before_zwnj), rakar: last && rakar }
                    })
                    .collect();
                compose(&parts, reph, &marks)
            })
            .collect::<Option<Vec<_>>>()?;
        Some(forms.into_iter().flatten().map(|strokes| {
            let fit = Fit::new(&strokes);
            (strokes, fit)
        }).collect())
    }

    /// Every single-character unit this pack draws: letters and marks.
    #[cfg(test)]
    fn atomic_units(&self) -> Vec<char> {
        let marks = "ािीुूृेैोौॉॅंःँ़्".chars();
        self.letters.keys().copied().chain(marks).collect()
    }
}

/// The consonant, as its precomposed nukta letter if a nukta follows.
fn with_nukta(c: char, chars: &mut std::iter::Peekable<impl Iterator<Item = char>>) -> Option<char> {
    if chars.next_if_eq(&NUKTA).is_none() {
        return Some(c);
    }
    NUKTA_LETTERS.iter().find(|n| n.1 == c).map(|n| n.0)
}

/// Lays out an akshara's parts and marks in design units, in stroke order:
/// one stroke list per combination of the marks' forms.
fn compose(parts: &[Part], reph: bool, marks: &[char]) -> Option<Vec<Strokes>> {
    let mut before_head = Strokes::new();
    let mut heads = Vec::new();
    let mut after_head = Strokes::new();
    let mut right_ink: Option<f64> = None;
    let mut last = None;
    for (i, part) in parts.iter().enumerate() {
        let half = i + 1 < parts.len() && !part.virama;
        let mut strokes = if half { part.letter.half() } else { part.letter.before_head() };
        let dx = right_ink.map_or(0.0, |r| r + GAP - bounds(&strokes).0);
        let letter = part.letter.shifted(dx);
        shift(&mut strokes, dx, 0.0);
        if part.rakar {
            strokes.push(rakar(&letter, part.consonant));
        }
        if part.virama {
            strokes.extend(mark(VIRAMA, &letter.frame())?.swap_remove(0).2);
        }
        let ink = bounds(&strokes);
        right_ink = Some(ink.1);
        if let Some((h0, h1)) = letter.head {
            heads.push((h0, if half { ink.1 } else { h1 }));
        }
        after_head.extend(letter.above.clone());
        after_head.extend(letter.nukta.map(|(x, y)| dot(x, y)));
        before_head.extend(strokes);
        last = Some(letter);
    }
    let last = last?;
    let mut frame = last.frame();
    frame.left = heads.iter().map(|h| h.0).fold(bounds(&before_head).0, f64::min);
    if parts.len() == 1 && parts[0].consonant == Some('र') && !parts[0].rakar {
        frame.ra = true;
    }
    // One draft per combination of mark forms so far, earlier marks varying
    // slowest.
    let mut drafts = vec![(before_head, heads, after_head)];
    let (dots, marks): (Vec<char>, Vec<char>) = marks.iter().partition(|&&m| m == 'ं' || m == 'ँ');
    for m in marks {
        drafts = fan(drafts, &mark(m, &frame)?);
        frame.advance(m);
    }
    if reph {
        let x = frame.reph;
        let reph = curve_f(&[(x + 5.0, 150.0), (x - 30.0, 105.0), (x - 42.0, 60.0), (x - 22.0, 28.0), (x + 22.0, 18.0)]);
        drafts = fan(drafts, &[(vec![], None, vec![reph])]);
        frame.top = frame.top.max(x + 110.0);
    }
    for m in dots {
        drafts = fan(drafts, &mark(m, &frame)?);
    }
    let finish = |(before_head, mut heads, after_head): Draft| {
        heads.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut merged: Vec<(f64, f64)> = Vec::new();
        for (a, b) in heads {
            match merged.last_mut() {
                Some(last) if a <= last.1 + HEAD_GAP => last.1 = last.1.max(b),
                _ => merged.push((a, b)),
            }
        }
        let head_lines = merged.into_iter().map(|(a, b)| vec![(a, 150.0), (b, 150.0)]);
        strokes![..before_head, ..head_lines, ..after_head]
    };
    Some(drafts.into_iter().map(finish).collect())
}

/// An akshara being composed: strokes before the headline, the headline
/// pieces, strokes after it.
type Draft = (Strokes, Vec<(f64, f64)>, Strokes);

/// Every draft written with each of a mark's forms.
fn fan(drafts: Vec<Draft>, forms: &[MarkStrokes]) -> Vec<Draft> {
    drafts
        .into_iter()
        .flat_map(|(before, heads, after)| {
            forms.iter().map(move |(pre, head, post)| {
                (strokes![..before.clone(), ..pre.clone()], heads.iter().copied().chain(*head).collect(), strokes![..after.clone(), ..post.clone()])
            })
        })
        .collect()
}

/// Maps design units into the 1000 box: `cx` goes to the box centre and
/// everything scales by `scale` about the headline, which stays at y=150.
#[derive(Debug, Clone, Copy)]
struct Fit {
    cx: f64,
    scale: f64,
}

impl Fit {
    /// A single mark keeps its place beside the notional प, which sits at
    /// [`MARK_ORIGIN_X`] when centred.
    const MARK: Fit = Fit { cx: 500.0 - MARK_ORIGIN_X, scale: 1.0 };

    /// Centred, uniformly shrunk if wider than [`MAX_WIDTH`] or reaching below
    /// [`MAX_BOTTOM`].
    fn new(strokes: &[Vec<Pt>]) -> Fit {
        let (min, max, _, bottom) = bounds(strokes);
        let headline = f64::from(HEADLINE);
        let scale = (MAX_WIDTH / (max - min).max(MAX_WIDTH)).min((MAX_BOTTOM - headline) / (bottom - headline).max(MAX_BOTTOM - headline));
        Fit { cx: (min + max) / 2.0, scale }
    }

    fn to_box(self, (x, y): Pt) -> Pt {
        let headline = f64::from(HEADLINE);
        (500.0 + (x - self.cx) * self.scale, headline + (y - headline) * self.scale)
    }
}

fn shift(strokes: &mut Strokes, dx: f64, dy: f64) {
    for p in strokes.iter_mut().flatten() {
        *p = (p.0 + dx, p.1 + dy);
    }
}

fn shifted(mut strokes: Strokes, dx: f64, dy: f64) -> Strokes {
    shift(&mut strokes, dx, dy);
    strokes
}

// --- structural model --------------------------------------------------------

/// A letter in design units: see the module docs.
#[derive(Clone, Default)]
struct Letter {
    body: Strokes,
    /// x of the full-height stem, drawn after the body.
    stem: Option<f64>,
    /// Drawn after the stem, before the headline.
    tail: Strokes,
    head: Option<(f64, f64)>,
    /// Drawn after the headline (ई's hook, ऊ's tail in some hands).
    above: Strokes,
    /// The stem is drawn before the body instead of after it.
    stem_first: bool,
    /// क and फ: where the stem stops in the half form, which keeps the hook.
    half_stem: Option<f64>,
    nukta: Option<Pt>,
}

fn letter(body: Strokes) -> Letter {
    Letter { body, ..Default::default() }
}

impl Letter {
    fn stem(self, x: i32) -> Self {
        Letter { stem: Some(x.into()), ..self }
    }
    fn tail(self, tail: Strokes) -> Self {
        Letter { tail, ..self }
    }
    fn head(self, x0: i32, x1: i32) -> Self {
        Letter { head: Some((x0.into(), x1.into())), ..self }
    }
    fn above(self, above: Strokes) -> Self {
        Letter { above, ..self }
    }
    fn stem_first(self) -> Self {
        Letter { stem_first: true, ..self }
    }
    fn half_stem(self, y: i32) -> Self {
        Letter { half_stem: Some(y.into()), ..self }
    }
    fn nukta(self, x: i32, y: i32) -> Self {
        Letter { nukta: Some((x.into(), y.into())), ..self }
    }

    /// Body, stem and tail: everything drawn before the headline.
    fn before_head(&self) -> Strokes {
        let stem = self.stem.map(|x| vec![(x, 150.0), (x, 740.0)]);
        if self.stem_first {
            strokes![..stem, ..self.body.clone(), ..self.tail.clone()]
        } else {
            strokes![..self.body.clone(), ..stem, ..self.tail.clone()]
        }
    }

    /// The form a consonant takes before another: no stem, or for क and फ a
    /// short one that still carries the hook. Other tail strokes that hang
    /// from the stem go with it; the rest (ष's diagonal) stay.
    fn half(&self) -> Strokes {
        match (self.stem, self.half_stem) {
            (Some(x), Some(y)) => strokes![..self.body.clone(), vec![(x, 150.0), (x, y)], ..self.tail.clone()],
            (x, _) => {
                let off_stem = self.tail.iter().filter(|s| x.is_none_or(|x| (s[0].0 - x).abs() > 10.0)).cloned();
                strokes![..self.body.clone(), ..off_stem]
            }
        }
    }

    fn shifted(&self, dx: f64) -> Letter {
        Letter {
            body: shifted(self.body.clone(), dx, 0.0),
            stem: self.stem.map(|x| x + dx),
            tail: shifted(self.tail.clone(), dx, 0.0),
            head: self.head.map(|(a, b)| (a + dx, b + dx)),
            above: shifted(self.above.clone(), dx, 0.0),
            stem_first: self.stem_first,
            half_stem: self.half_stem,
            nukta: self.nukta.map(|(x, y)| (x + dx, y)),
        }
    }

    /// Where marks attach to this letter written in full.
    fn frame(&self) -> Frame {
        let strokes = self.before_head();
        let (left, ink_right, _, _) = bounds(&strokes);
        let right = self.head.map_or(ink_right, |h| h.1);
        let stem = self.stem.unwrap_or(right - 144.0);
        // Stemless letters take marks below at their lowest point.
        let below = match self.stem {
            Some(x) => (x, 740.0),
            None => strokes.iter().flatten().copied().fold((0.0, f64::NEG_INFINITY), |a, p| if p.1 > a.1 { p } else { a }),
        };
        let left = self.head.map_or(left, |h| left.min(h.0));
        Frame { left, stem, right, below, top: stem + 2.0, reph: stem, ra: false }
    }
}

/// The squeezed upper letter over the smaller lower one, whose headline end
/// hangs under the upper letter's stub, neither with a stem of its own.
fn stack(top: &Letter, bottom: &Letter) -> Letter {
    let k = STACK_SCALE;
    let mut upper = top.before_head();
    for p in upper.iter_mut().flatten() {
        p.1 = 150.0 + (p.1 - 150.0) * k;
    }
    let (_, _, _, upper_bottom) = bounds(&upper);
    let mut lower = bottom.before_head();
    let hang = lower[0][0];
    let anchor = (upper[0][0].0, upper_bottom + 15.0);
    for p in lower.iter_mut().flatten() {
        *p = (anchor.0 + (p.0 - hang.0) * k, anchor.1 + (p.1 - hang.1) * k);
    }
    Letter { body: strokes![..upper, ..lower], head: top.head, ..Default::default() }
}

/// ्र: a diagonal down-left from the stem (or द's tail), else a ^ under the
/// letter's lowest point.
fn rakar(letter: &Letter, consonant: Option<char>) -> Vec<Pt> {
    match (letter.stem, consonant) {
        (Some(x), _) => vec![(x, 500.0), (x - 330.0, 690.0)],
        (None, consonant) => {
            let (x, y) = letter.frame().below;
            if consonant == Some('द') {
                vec![(x - 40.0, 640.0), (x - 350.0, 760.0)]
            } else {
                vec![(x - 150.0, y + 150.0), (x, y + 15.0), (x + 150.0, y + 150.0)]
            }
        }
    }
}

/// Where marks attach, in design units.
#[derive(Clone, Copy)]
struct Frame {
    /// Left edge of the ink: ि stands left of it.
    left: f64,
    /// The right stem (or where it would be): ि ी hooks reach it, े ै sit on it.
    stem: f64,
    /// Right end of the headline: ा ी ो ौ ॉ stand right of it, then ः.
    right: f64,
    /// The foot of the last consonant: ु ू ृ ् hang from it.
    below: Pt,
    /// x of ं ँ.
    top: f64,
    /// x of the reph.
    reph: f64,
    /// The akshara is a bare र, which takes ु ू at its middle.
    ra: bool,
}

impl Frame {
    /// The notional प that single marks are drawn around.
    const NOTIONAL_PA: Frame = Frame { left: 0.0, stem: 438.0, right: 582.0, below: (438.0, 740.0), top: 440.0, reph: 438.0, ra: false };

    /// Moves the anchors past a mark just placed.
    fn advance(&mut self, m: char) {
        match m {
            'ा' | 'ी' | 'ो' | 'ौ' | 'ॉ' => {
                let stem = self.right + 118.0;
                self.right += 178.0;
                self.reph = stem;
                self.top = stem - 40.0;
            }
            'े' | 'ै' => self.top = self.top.max(self.stem + 90.0),
            _ => {}
        }
    }
}

/// A mark's strokes drawn before the headline, its piece of the headline,
/// and its strokes drawn after.
type MarkStrokes = (Strokes, Option<(f64, f64)>, Strokes);

/// Every accepted form of a dependent mark around `f`, the taught form first
/// (see the module docs for the alternatives).
fn mark(c: char, f: &Frame) -> Option<Vec<MarkStrokes>> {
    // The templates are drawn around the notional प; anchors move them.
    let right = f.right - 582.0;
    let on_stem = f.stem - 438.0;
    let (bx, by) = (f.below.0 - 438.0, f.below.1 - 740.0);
    let top = f.top - 440.0;
    let head = |a: i32, b: i32, dx: f64| Some((f64::from(a) + dx, f64::from(b) + dx));
    // Signs hanging from the headline: as taught, and drawn up from it.
    let hanging = |signs: Strokes, pre: Strokes, head: Option<(f64, f64)>, dx: f64| -> Vec<MarkStrokes> {
        let up: Strokes = signs.iter().cloned().map(rev).collect();
        [signs, up].map(|signs| {
            let signs = shifted(signs, dx, 0.0);
            if pre.is_empty() { (vec![], None, signs) } else { (strokes![..pre.clone(), ..signs], head, vec![]) }
        }).into()
    };
    // Signs above: after the headline as taught, or before it.
    let above = |signs: Strokes| vec![(vec![], None, signs.clone()), (signs, None, vec![])];
    Some(match c {
        'ा' => vec![(shifted(strokes![stem(AA_STEM)], right, 0.0), head(560, 760, right), vec![])],
        'ि' => {
            // stem left of everything, hook stretched to reach the right stem
            let (s, t) = (f.left - 120.0, f.stem + 2.0);
            let stretch = |x: f64| s + (x + 120.0) * ((t - s) / 560.0);
            let hook = curve(&[(-120, 150), (-140, 90), (-100, 40), (0, 20), (150, 20), (300, 45), (400, 95), (440, 150)]);
            let hook: Vec<Pt> = hook.into_iter().map(|(x, y)| (stretch(x), y)).collect();
            let stem = vec![(s, 150.0), (s, 740.0)];
            vec![
                (strokes![stem.clone(), hook.clone()], head(-150, 20, f.left), vec![]),
                // one stroke: up the stem from its foot and on into the hook
                (vec![join([rev(stem), hook])], head(-150, 20, f.left), vec![]),
            ]
        }
        'ी' => {
            // hook from over the right stem to the stem of ी
            let (s, t) = (f.stem - 38.0, f.right + 118.0);
            let stretch = |x: f64| s + (x - 400.0) * ((t - s) / 300.0);
            let hook = curve(&[(400, 150), (385, 95), (420, 40), (500, 20), (600, 35), (670, 85), (AA_STEM, 150)]);
            let hook: Vec<Pt> = hook.into_iter().map(|(x, y)| (stretch(x), y)).collect();
            let stem = vec![(t, 150.0), (t, 740.0)];
            vec![
                (strokes![stem.clone(), hook.clone()], head(560, 760, right), vec![]),
                // one stroke: the hook, then straight on down the stem
                (vec![join([hook, stem])], head(560, 760, right), vec![]),
            ]
        }
        'ु' if f.ra => below_both(curve(&[(318, 420), (380, 360), (450, 345), (500, 390), (500, 460), (455, 525)])),
        'ू' if f.ra => vec![(vec![], None, vec![curve(&[(318, 420), (390, 365), (480, 330), (580, 350), (625, 420), (610, 500), (560, 540)])])],
        // as taught, and from the left end sweeping right to curl up under the stem
        'ु' => below_both(shifted(strokes![curve(&[(380, 770), (470, 760), (545, 805), (545, 885), (470, 940), (340, 945), (220, 905), (130, 840)])], bx, by).remove(0)),
        'ू' => vec![
            (vec![], None, shifted(strokes![
                curve(&[(440, 745), (360, 770), (300, 830), (310, 910), (380, 960), (480, 960)]),
                curve(&[(440, 745), (540, 830), (620, 915), (660, 965)]),
            ], bx, by)),
            // one stroke: round the loop, then out along the tail
            (vec![], None, shifted(strokes![join([
                curve(&[(440, 745), (360, 770), (300, 830), (310, 910), (380, 960), (480, 960), (535, 910), (530, 840)]),
                curve(&[(530, 840), (600, 895), (660, 965)]),
            ])], bx, by)),
        ],
        'ृ' => vec![(vec![], None, shifted(strokes![curve(&[(460, 760), (390, 790), (345, 850), (370, 925), (460, 950), (575, 915)])], bx, by))],
        'े' => hanging(strokes![curve(&[(250, 50), (310, 25), (370, 45), (415, 95), (440, 150)])], vec![], None, on_stem),
        'ै' => hanging(strokes![
            curve(&[(190, 105), (250, 85), (310, 100), (360, 122), (400, 150)]),
            curve(&[(250, 40), (320, 18), (385, 50), (420, 100), (440, 150)]),
        ], vec![], None, on_stem),
        'ो' => hanging(strokes![curve(&[(480, 50), (545, 25), (610, 50), (660, 95), (AA_STEM, 150)])], shifted(strokes![stem(AA_STEM)], right, 0.0), head(560, 760, right), right),
        'ौ' => hanging(strokes![
            curve(&[(430, 105), (495, 85), (560, 100), (615, 125), (655, 150)]),
            curve(&[(480, 40), (555, 18), (625, 50), (670, 100), (AA_STEM, 150)]),
        ], shifted(strokes![stem(AA_STEM)], right, 0.0), head(560, 760, right), right),
        'ॉ' => vec![(shifted(strokes![stem(AA_STEM), chandra(650, 35, 112, 90)], right, 0.0), head(560, 760, right), vec![])],
        // candra e (बॅंक): ँ's arc without the dot, where े would be
        'ॅ' => vec![(vec![], None, shifted(strokes![chandra(440, 30, 115, 110)], on_stem, 0.0))],
        'ं' => above(shifted(strokes![dot(440, 65)], top, 0.0)),
        'ः' => above(shifted(strokes![dot(668, 357), dot(668, 605)], right, 0.0)),
        'ँ' => above(shifted(strokes![chandra(440, 30, 115, 110), dot(440, 52)], top, 0.0)),
        '़' => vec![(vec![], None, strokes![nukta(170, 800)])],
        VIRAMA => vec![(vec![], None, shifted(strokes![curve(&[(410, 790), (480, 790), (540, 830), (620, 930)])], bx, by))],
        _ => return None,
    })
}

/// A sign below, as drawn and in reverse.
fn below_both(sign: Vec<Pt>) -> Vec<MarkStrokes> {
    vec![(vec![], None, vec![sign.clone()]), (vec![], None, vec![rev(sign)])]
}

/// The same path in the other pen direction.
fn rev(mut points: Vec<Pt>) -> Vec<Pt> {
    points.reverse();
    points
}

// --- drawing toolkit -------------------------------------------------------

/// Straight segments through the points.
fn line(pts: &[(i32, i32)]) -> Vec<Pt> {
    pts.iter().map(|&(x, y)| (x.into(), y.into())).collect()
}

/// Smooth curve through the points (centripetal Catmull-Rom).
fn curve(pts: &[(i32, i32)]) -> Vec<Pt> {
    curve_f(&line(pts))
}

fn curve_f(pts: &[Pt]) -> Vec<Pt> {
    centripetal(pts, Ends::Repeat, 1e-6, |_| 10)
}

/// Elliptical arc; angles in degrees, 0 = right, -90 = top (y is down).
/// Decreasing angles run counter-clockwise as seen on screen.
fn arc(cx: i32, cy: i32, rx: i32, ry: i32, start: i32, end: i32, n: usize) -> Vec<Pt> {
    let f = f64::from;
    super::arc((f(cx), f(cy)), f(rx), f(ry), f(start), f(end), n)
}

/// A dot, drawn as a tiny counter-clockwise circle from the top.
fn dot<T: Into<f64>>(x: T, y: T) -> Vec<Pt> {
    super::arc((x.into(), y.into()), 20.0, 20.0, -90.0, -450.0, 10)
}

/// One pen stroke made of consecutive parts (corners where they meet).
fn join<const N: usize>(parts: [Vec<Pt>; N]) -> Vec<Pt> {
    super::join(parts, 1e-6)
}

fn stem(x: i32) -> Vec<Pt> {
    line(&[(x, HEADLINE), (x, BASELINE)])
}

fn nukta(x: i32, y: i32) -> Vec<Pt> {
    dot(x, y)
}

// --- shared letter parts ---------------------------------------------------

/// अ without its stem and headline: the "3" and the connector.
fn a_body() -> Strokes {
    vec![
        join([
            curve(&[(155, 190), (230, 135), (310, 120), (375, 165), (385, 245), (340, 320), (230, 395)]),
            curve(&[(230, 395), (340, 420), (430, 480), (465, 565), (440, 645), (360, 690), (260, 685), (160, 615), (95, 505), (55, 395)]),
        ]),
        curve(&[(345, 415), (460, 425), (560, 410), (632, 395)]),
    ]
}

/// An अ-family letter with its headline over the whole letter instead of
/// only the stem: the "3" and connector (the first two strokes) squeezed
/// down from the headline, their foot kept.
fn full_head(l: Letter) -> Letter {
    let mut body = l.body;
    for p in body[..2].iter_mut().flatten() {
        p.1 = 690.0 - (690.0 - p.1) * (690.0 - 190.0) / (690.0 - 120.0);
    }
    Letter { body, head: l.head.map(|(_, x1)| (0.0, x1)), ..l }
}

/// The right-hand bowl of ख, from its top end round to the stem.
fn kha_bowl() -> Vec<Pt> {
    curve(&[(610, 300), (520, 285), (430, 300), (385, 370), (400, 440), (470, 470), (580, 450), (686, 400)])
}

/// छ without its headline: upper loop, lower bowl curling in, the stub.
fn chha_body() -> Strokes {
    vec![
        curve(&[(290, 272), (200, 262), (130, 292), (115, 352), (160, 420), (230, 465), (300, 478)]),
        curve(&[(200, 480), (140, 530), (120, 610), (170, 690), (290, 725), (430, 700), (560, 620), (625, 500), (620, 390), (570, 320), (495, 300), (440, 330), (425, 400), (460, 470), (515, 505)]),
        line(&[(494, HEADLINE), (494, 300)]),
    ]
}

/// इ without its headline: short stem, bar, S, loop and tail in one stroke.
fn i_body() -> Vec<Pt> {
    join([
        line(&[(364, HEADLINE), (364, 314), (170, 314)]),
        curve(&[(170, 314), (100, 335), (68, 395), (95, 465), (150, 505), (260, 480), (370, 485), (435, 545), (430, 630), (360, 695), (250, 718), (150, 702), (88, 668), (80, 622), (118, 603), (158, 640), (175, 700), (240, 790), (310, 880)]),
    ])
}

/// उ without its headline: from the headline down into the "3".
fn u_body() -> Vec<Pt> {
    join([
        curve(&[(380, 150), (440, 205), (455, 285), (400, 350), (300, 390), (235, 400)]),
        curve(&[(235, 400), (380, 420), (470, 500), (485, 610), (420, 700), (310, 735), (190, 700), (110, 600), (55, 420)]),
    ])
}

/// ए without its headline.
fn e_body() -> Strokes {
    vec![
        join([line(&[(120, HEADLINE), (120, 390)]), curve(&[(120, 390), (150, 470), (240, 540), (380, 620), (455, 700), (450, 780), (410, 830)])]),
        join([line(&[(432, HEADLINE), (432, 330)]), curve(&[(432, 330), (410, 420), (360, 475), (310, 500)])]),
    ]
}

/// ड (and ङ) without headline: short stem, bar, S ending in the left tail.
fn da_body() -> Vec<Pt> {
    join([
        line(&[(404, HEADLINE), (404, 318), (240, 318)]),
        curve(&[(240, 318), (150, 340), (100, 400), (135, 470), (200, 505), (330, 485), (450, 540), (470, 630), (410, 705), (290, 730), (160, 700), (40, 600)]),
    ])
}

/// Right-hand hook of क and फ, leaving the stem at x.
fn ka_hook(x: i32) -> Vec<Pt> {
    curve(&[(x, 470), (x + 62, 420), (x + 142, 400), (x + 232, 430), (x + 287, 500), (x + 287, 580), (x + 247, 650), (x + 192, 700)])
}

/// The loop of व and ब: from its top end round to the stem at x.
fn va_loop(x: i32) -> Vec<Pt> {
    curve(&[(355, 290), (250, 285), (140, 320), (80, 400), (85, 500), (150, 580), (260, 605), (350, 575), (x, 500)])
}

/// The chandra arc, left to right.
fn chandra(cx: i32, top: i32, bottom: i32, half: i32) -> Vec<Pt> {
    curve(&[(cx - half, top), (cx - half + 30, bottom - 20), (cx, bottom), (cx + half - 30, bottom - 20), (cx + half, top)])
}

/// म/भ/न: clockwise knob hanging left of (x, y), then the bar to the stem.
fn knob_bar(x: i32, y: i32, bar_to: i32) -> Vec<Pt> {
    join([curve(&[(x, y), (x - 2, y + 80), (x - 37, y + 125), (x - 87, y + 100), (x - 97, y + 40), (x - 57, y + 5), (x, y)]), line(&[(x, y), (bar_to, y)])])
}

// --- letters (design units) ------------------------------------------------

/// Every letter in writing order; a letter listed again right after itself
/// is an accepted alternative form (see the module docs).
fn letters() -> Vec<(char, Letter)> {
    // अ and the vowels built on it, which share its headline.
    let a = letter(a_body()).stem(632).head(520, 776);
    let aa = letter(strokes![..a_body(), stem(632)]).stem(890).head(520, 1030);
    let o_sign = || strokes![curve(&[(720, 50), (785, 30), (840, 65), (875, 110), (890, 150)])];
    let au_signs = || strokes![curve(&[(700, 105), (760, 88), (815, 108), (855, 150)]), curve(&[(720, 45), (790, 25), (850, 65), (890, 150)])];
    let up = |signs: Strokes| signs.into_iter().map(rev).collect::<Strokes>();
    let o = aa.clone().tail(o_sign());
    let o_up = aa.clone().tail(up(o_sign()));
    let au = aa.clone().tail(au_signs());
    let au_up = aa.clone().tail(up(au_signs()));
    let candra_o = aa.clone().tail(strokes![chandra(775, 30, 110, 95)]);
    let ii_hook = || curve(&[(380, 150), (348, 95), (365, 45), (420, 22), (470, 38), (490, 75)]);
    let uu_tail = || curve(&[(460, 520), (520, 430), (610, 410), (690, 460), (720, 560), (690, 660), (620, 730)]);
    let ai_sign = || curve(&[(200, 45), (270, 25), (345, 55), (400, 110), (425, 150)]);
    let ga = || join([line(&[(168, HEADLINE), (168, 500)]), curve(&[(168, 500), (158, 550), (115, 560), (78, 520), (82, 462), (125, 440), (165, 465)])]);
    let la = || join([
        curve(&[(270, 725), (170, 650), (90, 560), (80, 450), (125, 360), (210, 325), (285, 350), (320, 420), (325, 520)]),
        curve(&[(325, 520), (360, 420), (410, 350), (470, 320), (548, 320)]),
    ]);
    let sha = || curve(&[(250, 340), (140, 310), (85, 250), (110, 185), (200, 160), (300, 175), (360, 240), (355, 330), (300, 410), (200, 460), (110, 480), (65, 520), (90, 565), (150, 565), (220, 630), (330, 735)]);
    let ssa_body = || join([line(&[(120, HEADLINE), (120, 390)]), curve(&[(120, 390), (145, 470), (210, 530), (300, 530), (380, 490), (447, 440)])]);
    let ssa_diagonal = || line(&[(170, 200), (390, 480)]);
    vec![
        // Independent vowels
        ('अ', a.clone()),
        ('अ', full_head(a)),
        ('आ', aa.clone()),
        ('आ', full_head(aa)),
        ('इ', letter(strokes![i_body()]).head(0, 505)),
        ('ई', letter(strokes![i_body(), ii_hook()]).head(0, 505)),
        ('ई', letter(strokes![i_body()]).above(strokes![ii_hook()]).head(0, 505)),
        ('उ', letter(strokes![u_body()]).head(0, 560)),
        ('ऊ', letter(strokes![u_body(), uu_tail()]).head(0, 798)),
        ('ऊ', letter(strokes![u_body()]).above(strokes![uu_tail()]).head(0, 798)),
        ('ऋ', letter(strokes![join([curve(&[(30, 330), (120, 292), (210, 290), (300, 345), (410, 440)]), line(&[(410, 440), (70, 650)])])])
            .stem(422)
            .tail(strokes![join([
                curve(&[(425, 430), (540, 415), (620, 380), (660, 330), (640, 292), (598, 305), (600, 355), (650, 430), (750, 540)]),
                curve(&[(750, 540), (640, 560), (575, 630), (590, 720), (700, 760), (830, 730)]),
            ])])
            .head(0, 863)),
        ('ए', letter(e_body()).head(0, 568)),
        ('ऐ', letter(strokes![..e_body(), ai_sign()]).head(0, 568)),
        ('ऐ', letter(strokes![..e_body(), rev(ai_sign())]).head(0, 568)),
        ('ऐ', letter(e_body()).above(strokes![ai_sign()]).head(0, 568)),
        ('ऐ', letter(e_body()).above(strokes![rev(ai_sign())]).head(0, 568)),
        ('ओ', o.clone()),
        ('ओ', full_head(o)),
        ('ओ', o_up.clone()),
        ('ओ', full_head(o_up)),
        ('औ', au.clone()),
        ('औ', full_head(au)),
        ('औ', au_up.clone()),
        ('औ', full_head(au_up)),
        ('ऑ', candra_o.clone()),
        ('ऑ', full_head(candra_o)),
        ('ऍ', letter(e_body()).tail(strokes![chandra(440, 30, 115, 110)]).head(0, 568)),
        // Consonants
        ('क', letter(strokes![curve(&[(345, 282), (245, 270), (150, 300), (100, 380), (110, 470), (170, 545), (250, 585), (330, 560), (418, 500)])])
            .stem(418).tail(strokes![ka_hook(418)]).head(0, 783).half_stem(520)),
        ('ख', letter(strokes![
            curve(&[(255, 150), (275, 240), (245, 320), (170, 380), (100, 400), (65, 435), (80, 490), (140, 580), (240, 665), (380, 715), (530, 700), (686, 620)]),
            kha_bowl(),
        ]).stem(686).head(0, 830)),
        // The left part closed into a loop where it turns, as many hands write it.
        ('ख', letter(strokes![
            curve(&[(255, 150), (275, 240), (250, 330), (190, 390), (120, 405), (70, 370), (62, 318), (100, 295), (145, 325), (165, 395), (205, 490), (280, 590), (390, 660), (530, 665), (686, 600)]),
            kha_bowl(),
        ]).stem(686).head(0, 830)),
        ('ग', letter(strokes![ga()]).stem(435).head(0, 577)),
        ('ग', letter(strokes![rev(ga())]).stem(435).head(0, 577)),
        ('घ', letter(strokes![
            curve(&[(110, 150), (75, 210), (80, 275), (140, 325), (220, 335), (300, 330)]),
            curve(&[(150, 340), (105, 400), (100, 480), (150, 560), (240, 590), (350, 560), (461, 480)]),
        ]).stem(461).head(0, 602)),
        ('ङ', letter(strokes![da_body(), dot(562, 420)]).head(-10, 656)),
        ('च', letter(strokes![
            line(&[(40, 325), (390, 325)]),
            curve(&[(230, 330), (160, 390), (140, 470), (180, 550), (270, 595), (380, 570), (503, 480)]),
        ]).stem(503).head(0, 647)),
        ('छ', letter(chha_body()).head(0, 713)),
        // The headline only over the right bowl, leaving the upper loop open above.
        ('छ', letter(chha_body()).head(380, 713)),
        ('ज', letter(strokes![join([curve(&[(60, 330), (110, 450), (180, 580), (290, 640), (400, 615), (445, 530), (420, 440), (365, 375)]), line(&[(365, 375), (611, 375)])])]).stem(611).head(0, 755)),
        ('झ', letter(strokes![i_body(), curve(&[(330, 475), (420, 492), (520, 488), (625, 470)])]).stem(625).head(0, 771)),
        // The older (Uttara) झ: a hooked left stroke and its bar to the stem,
        // then a hook hanging right of the stem like क's.
        ('झ', letter(strokes![
            curve(&[(110, HEADLINE), (160, 250), (172, 360), (150, 450), (105, 510), (60, 535)]),
            line(&[(130, 495), (440, 495)]),
        ]).stem(440).tail(strokes![curve(&[(440, 420), (530, 405), (600, 450), (615, 540), (580, 630), (505, 690)])]).head(0, 640)),
        ('ञ', letter(strokes![
            join([curve(&[(195, 325), (270, 295), (360, 300), (435, 345), (478, 440)]), curve(&[(478, 440), (440, 535), (340, 610), (220, 600), (130, 510), (40, 390)])]),
            line(&[(478, 450), (612, 450)]),
        ]).stem(612).head(0, 755)),
        ('ट', letter(strokes![join([line(&[(357, HEADLINE), (357, 345), (270, 345)]), curve(&[(270, 345), (170, 370), (100, 440), (95, 560), (160, 660), (280, 700), (390, 680), (470, 635)])])]).head(0, 517)),
        ('ठ', letter(strokes![join([line(&[(339, HEADLINE), (339, 336)]), arc(290, 530, 225, 195, -77, -430, 40)])]).head(0, 596)),
        ('ड', letter(strokes![da_body()]).head(0, 542)),
        ('ढ', letter(strokes![join([
            line(&[(400, HEADLINE), (400, 323), (280, 323)]),
            curve(&[(280, 323), (170, 350), (85, 440), (75, 580), (150, 690), (290, 730), (430, 700), (500, 620), (490, 530), (420, 475), (330, 480), (275, 545), (290, 620), (330, 680)]),
        ])]).head(0, 575)),
        ('ण', letter(strokes![join([line(&[(82, HEADLINE), (82, 430)]), curve(&[(82, 430), (110, 540), (230, 590), (345, 540), (380, 430)]), line(&[(380, 430), (380, HEADLINE)])])]).stem(593).head(0, 737)),
        ('त', letter(strokes![join([line(&[(438, 430), (270, 430)]), curve(&[(270, 430), (160, 450), (110, 520), (110, 610), (150, 680), (210, 735)])])]).stem(438).head(0, 582)),
        ('थ', letter(strokes![join([
            curve(&[(175, 330), (110, 265), (105, 190), (160, 130), (235, 130), (275, 195), (255, 285), (180, 360), (62, 420)]),
            curve(&[(62, 420), (120, 530), (220, 610), (340, 620), (440, 560), (506, 480)]),
        ])]).stem(506).head(391, 650)),
        ('द', letter(strokes![join([
            line(&[(395, HEADLINE), (395, 320), (250, 320)]),
            curve(&[(250, 320), (140, 340), (80, 420), (75, 520), (130, 610), (240, 655), (340, 650), (420, 610), (445, 555), (410, 505), (355, 520), (350, 590), (390, 680), (450, 790)]),
        ])]).head(0, 541)),
        ('ध', letter(strokes![
            curve(&[(285, 290), (320, 215), (280, 140), (190, 115), (100, 150), (70, 230), (110, 310), (200, 370), (320, 390)]),
            curve(&[(150, 385), (110, 460), (120, 550), (200, 610), (300, 615), (400, 560), (487, 480)]),
        ]).stem(487).head(384, 629)),
        ('न', letter(strokes![knob_bar(170, 395, 424)]).stem(424).head(0, 568)),
        ('प', letter(strokes![join([line(&[(118, HEADLINE), (118, 390)]), curve(&[(118, 390), (140, 470), (200, 530), (290, 535), (370, 490), (438, 430)])])]).stem(438).head(0, 582)),
        ('फ', letter(strokes![join([line(&[(120, HEADLINE), (120, 390)]), curve(&[(120, 390), (145, 480), (210, 540), (300, 540), (370, 500), (418, 450)])])])
            .stem(418).tail(strokes![ka_hook(418)]).head(0, 785).half_stem(520)),
        ('ब', letter(strokes![va_loop(435), line(&[(170, 375), (370, 520)])]).stem(435).head(0, 584)),
        ('भ', letter(strokes![join([
            curve(&[(200, 340), (90, 290), (55, 210), (100, 135), (190, 115), (280, 150), (325, 230), (330, 330), (330, 455)]),
            knob_bar(330, 455, 573)[1..].to_vec(),
        ])]).stem(573).head(402, 717)),
        ('म', letter(strokes![join([line(&[(167, HEADLINE), (167, 450)]), knob_bar(167, 450, 467)])]).stem(467).head(0, 611)),
        ('य', letter(strokes![join([curve(&[(220, 150), (255, 220), (245, 300), (170, 360), (40, 395)]), curve(&[(40, 395), (80, 480), (160, 560), (270, 590), (370, 550), (449, 470)])])]).stem(449).head(0, 593)),
        ('र', letter(strokes![curve(&[(310, 150), (335, 240), (310, 320), (240, 375), (160, 392), (100, 368), (60, 395), (72, 450), (130, 472), (200, 545), (330, 730)])]).head(0, 424)),
        ('ल', letter(strokes![la()]).stem(548).head(0, 692)),
        ('ल', letter(strokes![rev(la())]).stem(548).stem_first().head(0, 692)),
        // ळ (Marathi): a stub down from the headline into a figure-eight lying
        // on its side, round the left loop first; stemless, like ठ.
        ('ळ', letter(strokes![join([
            line(&[(518, HEADLINE), (518, 345)]),
            curve(&[(518, 345), (460, 360), (420, 420), (400, 505), (380, 590), (340, 645), (300, 660), (255, 635), (232, 575), (232, 440), (255, 375), (300, 348), (345, 360), (380, 420), (400, 505), (420, 590), (460, 645), (505, 660), (550, 640), (578, 580), (580, 440), (555, 370), (505, 345)]),
        ])]).head(0, 775)),
        ('व', letter(strokes![va_loop(424)]).stem(424).head(0, 568)),
        ('श', letter(strokes![sha()]).stem(550).head(40, 695)),
        ('श', letter(strokes![sha()]).stem(550).head(400, 695)),
        ('ष', letter(strokes![ssa_body(), ssa_diagonal()]).stem(447).head(0, 591)),
        ('ष', letter(strokes![ssa_body()]).stem(447).tail(strokes![ssa_diagonal()]).head(0, 591)),
        ('स', letter(strokes![
            curve(&[(250, 150), (275, 240), (250, 320), (190, 370), (120, 378), (72, 395), (62, 440), (100, 470), (160, 480), (230, 570), (314, 725)]),
            curve(&[(180, 440), (270, 460), (410, 460), (546, 445)]),
        ]).stem(546).head(0, 689)),
        ('ह', letter(strokes![
            join([
                line(&[(400, HEADLINE), (400, 310), (170, 310)]),
                curve(&[(170, 310), (100, 340), (80, 400), (120, 450), (200, 465), (320, 470), (430, 510), (470, 590), (440, 660), (360, 700)]),
            ]),
            curve(&[(95, 470), (80, 560), (110, 660), (190, 760), (270, 810), (350, 850)]),
        ]).head(0, 543)),
        // Other signs
        ('।', letter(strokes![line(&[(231, 120), (231, BASELINE)])])),
        // ऽ hangs from the headline like an S: the top bowl curls back to the
        // right, the lower one sweeps out to the baseline on the left.
        ('ऽ', letter(strokes![curve(&[(400, HEADLINE), (300, 175), (210, 240), (200, 320), (270, 375), (370, 400), (430, 470), (400, 580), (300, 670), (150, 720)])]).head(0, 470)),
        // ॰ (abbreviation sign) is a small ring at mid height.
        ('॰', letter(strokes![arc(240, 480, 75, 75, -90, -450, 24)])),
        ('ॐ', letter(strokes![
            join([
                curve(&[(150, 160), (230, 125), (310, 118), (375, 160), (385, 240), (340, 315), (230, 380)]),
                curve(&[(230, 380), (345, 405), (430, 470), (455, 560), (420, 640), (330, 680), (220, 670), (130, 590), (80, 490), (42, 390)]),
            ]),
            curve(&[(420, 400), (510, 300), (620, 255), (740, 265), (830, 330), (840, 430), (790, 510), (690, 540), (600, 500)]),
            chandra(590, 25, 100, 150),
            dot(590, 52),
        ])),
        // Digits (no headline)
        ('०', letter(strokes![arc(260, 415, 178, 182, -90, -450, 40)])),
        ('१', letter(strokes![join([
            curve(&[(290, 345), (200, 320), (150, 250), (180, 170), (260, 140), (350, 170), (380, 240), (340, 320), (230, 410), (130, 480)]),
            curve(&[(130, 480), (230, 560), (330, 640), (375, 700), (340, 750)]),
        ])])),
        ('२', letter(strokes![curve(&[(105, 180), (180, 130), (265, 120), (345, 160), (385, 240), (360, 330), (280, 395), (190, 420), (120, 405), (95, 365), (130, 335), (185, 360), (230, 430), (290, 570), (350, 725)])])),
        ('३', letter(strokes![join([
            curve(&[(95, 170), (180, 125), (280, 115), (365, 160), (395, 230), (345, 300), (240, 335), (160, 340)]),
            curve(&[(160, 340), (300, 360), (400, 420), (430, 510), (380, 590), (270, 625), (170, 612), (105, 585), (90, 545), (125, 520), (170, 550), (215, 630), (300, 815)]),
        ])])),
        ('४', letter(strokes![curve(&[(100, 130), (150, 250), (258, 370), (370, 460), (405, 560), (360, 660), (258, 700), (150, 660), (110, 560), (150, 460), (258, 370), (370, 260), (420, 130)])])),
        ('५', letter(strokes![join([
            curve(&[(215, 125), (140, 200), (90, 290), (100, 380), (170, 450), (270, 455), (345, 425)]),
            curve(&[(345, 425), (380, 380), (350, 340), (305, 370), (330, 440), (390, 540), (440, 735)]),
        ])])),
        ('६', letter(strokes![join([
            curve(&[(390, 140), (280, 112), (160, 140), (100, 220), (140, 310), (250, 355)]),
            curve(&[(250, 355), (150, 390), (95, 480), (110, 590), (200, 670), (300, 675), (370, 620), (375, 560), (330, 560), (320, 610), (360, 700), (430, 810)]),
        ])])),
        ('७', letter(strokes![curve(&[(123, 123), (90, 260), (75, 411), (96, 560), (160, 660), (258, 690), (370, 650), (440, 540), (460, 420), (430, 300), (366, 249), (290, 265), (258, 330), (265, 420), (310, 470), (402, 483)])])),
        ('८', letter(strokes![curve(&[(375, 140), (250, 260), (160, 370), (130, 480), (170, 590), (250, 650), (340, 640), (430, 570)])])),
        ('९', letter(strokes![curve(&[(215, 360), (125, 300), (110, 210), (170, 135), (260, 115), (350, 150), (390, 230), (360, 320), (290, 400), (370, 500), (420, 600), (400, 700), (340, 750)])])),
    ]
}

/// Conjuncts with a shape of their own, drawn against Noto Sans Devanagari's
/// forms in the same design units.
fn ligatures() -> Vec<((char, char), Letter)> {
    vec![
        // क्ष: from the crossing round the upper loop and down into the lower
        // bowl, its curl and tail; the connector to the stem; the stem.
        (('क', 'ष'), letter(strokes![
            curve(&[(240, 400), (320, 345), (355, 260), (320, 170), (240, 140), (160, 180), (130, 265), (170, 345), (240, 400), (170, 450), (100, 520), (80, 600), (120, 670), (220, 705), (320, 690), (395, 640), (415, 585), (370, 550), (325, 585), (335, 670), (380, 740), (420, 810)]),
            curve(&[(240, 400), (350, 380), (460, 368), (588, 366)]),
        ]).stem(588).head(467, 730)),
        // त्र: the arc and diagonal of ऋ's first stroke, the stem.
        (('त', 'र'), letter(strokes![join([curve(&[(30, 330), (120, 292), (210, 290), (300, 345), (410, 440)]), line(&[(410, 440), (70, 650)])])]).stem(435).head(0, 565)),
        // ज्ञ: the bar, the S and tail, the stem.
        (('ज', 'ञ'), letter(strokes![
            line(&[(100, 335), (512, 335)]),
            curve(&[(225, 360), (310, 400), (355, 480), (330, 580), (250, 640), (150, 650), (80, 610), (70, 560), (110, 540), (160, 570), (220, 660), (300, 800)]),
        ]).stem(512).head(0, 655)),
        // श्र: from the stem side into the loop and out to the left, the stem, the rakar.
        (('श', 'र'), letter(strokes![
            curve(&[(577, 460), (420, 440), (300, 400), (200, 330), (165, 250), (200, 170), (270, 145), (340, 180), (370, 260), (330, 340), (230, 410), (110, 465), (40, 490)]),
        ]).stem(577).tail(strokes![line(&[(577, 490), (210, 690)])]).head(475, 720)),
        // द्ध: द's stub and bowl curling into the long tail, then ध's loop and bowl below.
        (('द', 'ध'), letter(strokes![
            join([
                line(&[(660, HEADLINE), (660, 320), (560, 320)]),
                curve(&[(560, 320), (450, 320), (375, 380), (360, 470), (400, 560), (500, 610), (600, 625), (690, 610), (730, 560), (710, 500), (660, 490), (625, 530), (630, 600), (665, 720), (715, 890)]),
            ]),
            curve(&[(260, 575), (250, 510), (180, 490), (115, 525), (105, 600), (150, 655), (240, 665), (360, 655)]),
            curve(&[(250, 665), (200, 740), (220, 830), (310, 885), (430, 880), (540, 820), (620, 720), (655, 640)]),
        ]).head(0, 800)),
        // द्व: द's stub, bowl, curl and tail, then व's loop below.
        (('द', 'व'), letter(strokes![
            join([
                line(&[(440, HEADLINE), (440, 330), (340, 330)]),
                curve(&[(340, 330), (230, 345), (160, 420), (150, 510), (190, 590), (270, 630), (380, 630), (460, 610), (505, 560), (490, 500), (440, 490), (410, 540), (420, 620), (460, 740), (515, 890)]),
            ]),
            curve(&[(210, 600), (140, 640), (95, 720), (125, 810), (230, 845), (350, 810), (420, 730), (440, 650)]),
        ]).head(0, 585)),
        // द्य: द's stub and small bowl, य's bar and bowl, the stem.
        (('द', 'य'), letter(strokes![
            join([line(&[(360, HEADLINE), (360, 270), (270, 270)]), curve(&[(270, 270), (200, 300), (185, 370), (230, 450)])]),
            join([line(&[(360, 470), (260, 470)]), curve(&[(260, 470), (200, 520), (200, 610), (260, 680), (350, 690), (420, 650), (478, 590)])]),
        ]).stem(478).head(0, 625)),
        // ह्म: ह's stub and bowl into म's knob and bar, ह's tail, the long stem.
        (('ह', 'म'), letter(strokes![
            join([
                line(&[(400, HEADLINE), (400, 310), (290, 310)]),
                curve(&[(290, 310), (180, 330), (120, 400), (130, 475)]),
                curve(&[(130, 475), (200, 450), (300, 440), (400, 465), (455, 530), (470, 620)]),
                knob_bar(470, 620, 668)[1..].to_vec(),
            ]),
            curve(&[(130, 475), (80, 580), (90, 690), (160, 800), (280, 890)]),
            line(&[(668, HEADLINE), (668, 858)]),
        ]).head(0, 815)),
        // त्त: the first त's bar, the second's bar and bowl, the stem.
        (('त', 'त'), letter(strokes![
            line(&[(515, 305), (40, 305)]),
            join([line(&[(515, 410), (360, 410)]), curve(&[(360, 410), (300, 440), (270, 520), (290, 610), (360, 700)])]),
        ]).stem(540).head(0, 684)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate;

    #[test]
    fn segments_aksharas() {
        let text = "नमस्ते, प्रिय कर्मों! ज़्य ज्ञान हिन्दी क्\u{200c}ष र्\u{200d}य ि x";
        assert_eq!(
            segment(text),
            ["न", "म", "स्ते", ",", " ", "प्रि", "य", " ", "क", "र्मों", "!", " ", "ज़्य", " ", "ज्ञा", "न", " ", "हि", "न्दी", " ", "क्\u{200c}ष", " ", "र्\u{200d}य", " ", "ि", " ", "x"]
        );
        assert_eq!(segment("").len(), 0);
        assert_eq!(segment("क्").len(), 1);
    }

    #[test]
    fn atomic_units_draw() {
        let d = Devanagari::default();
        let units = d.atomic_units();
        assert!(units.len() >= 86, "{}", units.len());
        for c in units {
            let forms = d.glyphs(&c.to_string());
            assert!(!forms.is_empty(), "{c}");
            for g in forms {
                validate(&g).unwrap_or_else(|e| panic!("{c}: {e}"));
            }
        }
    }

    #[test]
    fn variant_counts() {
        let d = Devanagari::default();
        for (unit, forms) in [
            ("अ", 2),
            // headline extent times sign direction
            ("ओ", 4),
            ("क", 1),
            ("ख", 2),
            ("ख़", 2),
            ("ळ", 1),
            ("ऴ", 1),
            ("ळ\u{93c}", 1),
            ("न\u{93c}", 1),
            ("क्ळ", 1),
            // ी as taught or in one stroke
            ("ळी", 2),
            // two letters with two forms each
            ("ख्झ", 4),
            ("ख्य", 2),
            // every vowel sign and sign above has a second form
            ("कि", 2),
            ("की", 2),
            ("कु", 2),
            ("कू", 2),
            ("के", 2),
            ("कै", 2),
            ("को", 2),
            ("कौ", 2),
            ("कं", 2),
            ("कः", 2),
            ("कँ", 2),
            ("रु", 2),
            ("रू", 1),
            ("कृ", 1),
            // letter forms times mark forms
            ("खों", 8),
            ("ई", 2),
            ("ऊ", 2),
            ("ऐ", 4),
            ("औ", 4),
            ("श", 2),
            ("ग", 2),
            ("ल", 2),
            ("ष", 2),
            ("थ", 1),
        ] {
            assert_eq!(d.glyphs(unit).len(), forms, "{unit}");
        }
        // Decomposed nukta is the precomposed letter, in clusters too.
        assert_eq!(d.glyphs("ळ\u{93c}"), d.glyphs("\u{934}"));
        assert_eq!(d.glyphs("क्ळ\u{93c}"), d.glyphs("क्\u{934}"));
        // The product is ordered: the first letter's choice varies slowest,
        // and index 0 is every letter's taught form.
        // Half ख (2 strokes), झ (3, or 4 with the old form's hook), headline.
        let product = d.glyphs("ख्झ");
        assert_eq!(product.iter().map(|g| g.strokes.len()).collect::<Vec<_>>(), [6, 7, 6, 7]);
        assert_ne!(product[0], product[2]);
        for unit in ["क्ळ", "ळी", "ऴ", "ऩ", "ऱ", "ऩ्य", "ऱ्य", "ळ्य", "र्ळ", "ळ्र"] {
            for g in d.glyphs(unit) {
                validate(&g).unwrap_or_else(|e| panic!("{unit}: {e}"));
            }
        }
    }

    #[test]
    fn corpus_clusters_compose() {
        // The most frequent clusters in the Hindi course's sentences, then
        // every ligature, stack, र form and explicit-virama case.
        let clusters = "क्य म्ह च्छ स्त प्र क्ष त्र त्त स्ट च्च स्व न्ह ट्र स्क प्य र्म क्त क्र ग्र ल्द ल्ल द्ध न्य व्य ल्क म्म ध्य स्थ स्प त्म र्य ष्ट ब्र प्त क्स स्स द्र र्ट ज्ञ श्व र्त न्न श्च त्य ज़्य र्व र्द ट्ट र्थ द्य ज्य र्ज न्ग क्क र्ड श्क ख्य र्क द्व न्द श्र \
            ह्म ट्ठ ड्ड ड्ढ ड्र फ़्र स्त्र ष्ट्र र्त्त र्र द्द द्भ ट्स ह्न क्\u{200c}ष";
        let d = Devanagari::default();
        let mut most = (0, String::new());
        for c in clusters.split_whitespace() {
            // bare, and under the busiest common marks
            for unit in [c.to_string(), format!("{c}ों"), format!("{c}िं"), format!("{c}ीं")] {
                let forms = d.glyphs(&unit);
                assert!(!forms.is_empty(), "{unit}");
                most = most.max((forms.len(), unit.clone()));
                for g in forms {
                    validate(&g).unwrap_or_else(|e| panic!("{unit}: {e}"));
                }
            }
        }
        // Forms multiply; keep the product small enough to grade against.
        assert!(most.0 <= 32, "{most:?}");
        println!("most forms: {most:?}");
        // Decomposed nukta is the precomposed letter.
        assert_eq!(d.glyphs("ज\u{93c}्य"), d.glyphs("\u{95b}्य"));
        assert!(d.glyphs("x").is_empty());
        assert!(d.glyphs("कऽ").is_empty());
    }

    #[test]
    fn akshara_stroke_counts() {
        let d = Devanagari::default();
        for (unit, strokes) in [
            // half स (2), त (bar-bowl, stem), headline, े
            ("स्ते", 6),
            // प (body, stem), rakar, ि (stem, hook), headline
            ("प्रि", 6),
            // म (body, stem), headline, reph
            ("र्म", 4),
            // half न, द, ी (stem, hook), headline
            ("न्दी", 5),
            // ज्ञ (bar, S, stem), ा stem, headline
            ("ज्ञा", 5),
        ] {
            assert_eq!(d.glyphs(unit)[0].strokes.len(), strokes, "{unit}");
        }
    }
}
