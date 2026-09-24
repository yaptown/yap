//! Punctuation and common symbols, which text in every language contains.
//!
//! Western marks are drawn in the Latin frame ([`super::latin`]: capitals
//! 190..850, x-height 450, baseline 850, descender 975): `.` `,` sit on the
//! baseline, `?` `!` span the capitals, quotes hang from the cap line,
//! dashes and operators sit at mid x-height, brackets and `|` span ascender
//! to descender. Quotation marks are the short curved ticks people write,
//! not the typographic teardrops: the opening “ ‘ as little 6s, the closing
//! ” ’ „ and the comma as little 9s.
//!
//! CJK marks use the full em square the way Noto Sans CJK JP places them in
//! horizontal text: 。 、 in the lower left; an opening bracket 「 『 《 〈 【
//! 〔 in the right half, next to the text it opens, and its closing partner
//! in the left half; ― across the middle; ・ at the centre; 〝 as two ticks
//! at the top right, 〞 at the top left and 〟 at the bottom left. 。 is
//! drawn clockwise from the bottom and 、 from the top left, as KanjiVG draws
//! them (a Japanese pack takes KanjiVG's own 。 、 ・).
//!
//! The order follows the Latin letters: top to bottom, left to right, curves
//! and stems before dots (`?` `!` `;` `¿` `¡`), verticals before horizontals
//! (`+` `#`), a bar before its dots (`÷`), an arrow's shaft (drawn the way
//! it points) before its head. A bracket is one stroke from its top end.
//!
//! Drawn: ! " # $ % & ' ( ) * + , - . / : ; < = > ? @ [ \ ] ^ _ ` { | } ~
//! “ ” ‘ ’ „ « » ‹ › ¿ ¡ — – … ‥ · • × ÷ ° § € £ ¥ ′ ″ ‽ ⁈ ⁉ → ← ↑ ↓
//! 。 、 「 」 『 』 《 》 〈 〉 【 】 〔 〕 ― ・ 〝 〞 〟 ※ 〇.
//!
//! Accepted alternatives (the taught form first):
//! - … on the baseline, and at mid-height as Japanese and Chinese writers
//!   put it in the em square; ‥ the other way round (mid-height first), as
//!   it is mostly Japanese;
//! - &: the one-stroke figure from the foot of the leg, over the head loop
//!   and round the bowl to the arm; and the "Et" form, an ɛ crossed by a
//!   vertical stroke.
//!
//! Fullwidth and halfwidth forms (！ ？ （ ， ： ～ ｢ ･ ...) fold onto these by
//! NFKC (see [`crate::StrokePack::glyphs`]) and need no glyph of their own.
//! Symbols written exactly like a drawn mark that NFKC does not fold are
//! [`ALIASES`]. Pictographs (♬ ♥ ☆ ■ ● ...) are not writing and are left
//! out.
use super::latin::{ASC, BASE, CAP, DESC, Pen, X_MID, dot, line};
use super::{Pt, collect, glyph};
use crate::{Forms, StrokeStandard};

pub fn glyphs() -> Forms {
    let mut forms = collect(MARKS.iter().map(|&(c, draw)| {
        let strokes = draw().into_iter().map(|pen| pen.0).collect();
        (c, glyph(StrokeStandard::Latin, strokes, |p| p))
    }));
    for &(alias, mark) in ALIASES {
        let drawn = forms[&mark].clone();
        assert!(
            forms.insert(alias, drawn).is_none(),
            "{alias} drawn and aliased"
        );
    }
    forms
}

/// Symbols written exactly like a drawn mark, and the mark.
const ALIASES: &[(char, char)] = &[
    ('‚', ','),  // single low quotation mark
    ('‐', '-'),  // hyphen (‑ folds onto it)
    ('⸺', '—'),  // two-em dash
    ('∙', '·'),  // bullet operator
    ('∼', '~'),  // tilde operator
    ('〜', '~'), // wave dash
    ('≪', '《'), // much less-than
    ('≫', '》'), // much greater-than
];

type Draw = fn() -> Vec<Pen>;

/// Applies `f` to every point.
fn map(pens: Vec<Pen>, f: impl Fn(Pt) -> Pt) -> Vec<Pen> {
    pens.into_iter()
        .map(|pen| Pen(pen.0.into_iter().map(&f).collect()))
        .collect()
}

/// The mirror image of left-to-right strokes about the box's centre line,
/// still drawn top to bottom and left to right: the closing bracket of an
/// opening one.
fn mirror(pens: Vec<Pen>) -> Vec<Pen> {
    let mut pens = map(pens, |(x, y)| (1000.0 - x, y));
    pens.reverse();
    pens
}

/// Turned half way round the box's centre, each stroke still drawn from its
/// top: the closing corner bracket of an opening one.
fn turned(pens: Vec<Pen>) -> Vec<Pen> {
    map(pens, |(x, y)| (1000.0 - x, 1000.0 - y))
        .into_iter()
        .map(|mut pen| {
            pen.0.reverse();
            pen
        })
        .collect()
}

/// A mark narrowed to 70% about the centre line and centred on `x`, for the
/// two halves of ⁈ and ⁉.
fn narrowed(pens: Vec<Pen>, x: f64) -> Vec<Pen> {
    map(pens, |(px, py)| (x + (px - 500.0) * 0.7, py))
}

/// The curve and stem of ?, from the top left, over and down.
fn hook() -> Pen {
    Pen::at((345, 330)).curve([
        (395, 235),
        (500, 195),
        (610, 230),
        (655, 320),
        (620, 415),
        (540, 475),
        (500, 545),
        (500, 660),
    ])
}

fn question() -> Vec<Pen> {
    vec![hook(), dot(500, BASE - 15)]
}

fn exclamation() -> Vec<Pen> {
    vec![line([(500, CAP), (500, 660)]), dot(500, BASE - 15)]
}

/// A little 6, drawn from its tail at the top right down into the bowl: the
/// opening quote hanging from `(x, top)`.
fn six(x: i32, top: i32) -> Pen {
    Pen::at((x + 25, top)).curve([(x - 3, top + 45), (x - 12, top + 95), (x + 3, top + 130)])
}

/// A little 9, the comma shape: from the top down, curving off to the lower
/// left. The closing quote, and on the baseline the comma and „.
fn nine(x: i32, top: i32) -> Pen {
    Pen::at((x - 3, top)).curve([(x + 12, top + 40), (x + 7, top + 90), (x - 25, top + 130)])
}

/// One chevron pointing left (`dir` 1) or right (-1) from its tip, drawn
/// from the upper arm.
fn chevron(tip: i32, dir: i32, y: i32, w: i32, h: i32) -> Pen {
    line([(tip + dir * w, y - h), (tip, y), (tip + dir * w, y + h)])
}

/// `n` dots on a row at height `y`, `gap` apart, centred.
fn dots(n: i32, y: i32, gap: i32) -> Vec<Pen> {
    (0..n)
        .map(|i| dot(500 + (2 * i - (n - 1)) * gap / 2, y))
        .collect()
}

/// A capital-like S between `top` and `bottom`, from its upper end.
fn s(top: f64, bottom: f64) -> Pen {
    let y = |f: f64| top + (bottom - top) * f;
    Pen::at((640.0, y(0.12))).curve([
        (570.0, y(0.02)),
        (480.0, y(0.0)),
        (390.0, y(0.04)),
        (350.0, y(0.16)),
        (390.0, y(0.3)),
        (500.0, y(0.42)),
        (610.0, y(0.54)),
        (655.0, y(0.7)),
        (620.0, y(0.88)),
        (520.0, y(0.99)),
        (410.0, y(0.98)),
        (340.0, y(0.88)),
    ])
}

/// Every mark drawn directly, in writing order. A mark listed again right
/// after itself is an accepted alternative (see the module docs).
const MARKS: &[(char, Draw)] = &[
    ('.', || vec![dot(500, BASE - 15)]),
    (',', || vec![nine(500, BASE - 30)]),
    (':', || vec![dot(500, 500), dot(500, BASE - 15)]),
    (';', || vec![dot(500, 500), nine(500, BASE - 30)]),
    ('?', question),
    ('!', exclamation),
    // ? turned upside down and hanging below the baseline: the curve from
    // its free end at the bottom, round and up the stem, then the dot.
    ('¿', || {
        vec![
            map(vec![hook()], |(x, y)| (1000.0 - x, 1160.0 - y))
                .pop()
                .unwrap(),
            dot(500, 1160 - BASE + 15),
        ]
    }),
    ('¡', || vec![line([(500, 500), (500, 965)]), dot(500, 325)]),
    ('‽', || {
        vec![hook(), line([(500, CAP), (500, 660)]), dot(500, BASE - 15)]
    }),
    ('⁈', || {
        narrowed(question(), 390.0)
            .into_iter()
            .chain(narrowed(exclamation(), 650.0))
            .collect()
    }),
    ('⁉', || {
        narrowed(exclamation(), 350.0)
            .into_iter()
            .chain(narrowed(question(), 610.0))
            .collect()
    }),
    ('\'', || vec![line([(500, CAP), (500, 320)])]),
    ('"', || {
        vec![
            line([(445, CAP), (445, 320)]),
            line([(555, CAP), (555, 320)]),
        ]
    }),
    ('‘', || vec![six(500, CAP)]),
    ('’', || vec![nine(500, CAP)]),
    ('“', || vec![six(440, CAP), six(560, CAP)]),
    ('”', || vec![nine(440, CAP), nine(560, CAP)]),
    ('„', || vec![nine(440, BASE - 30), nine(560, BASE - 30)]),
    ('«', || {
        vec![
            chevron(370, 1, X_MID, 110, 130),
            chevron(510, 1, X_MID, 110, 130),
        ]
    }),
    ('»', || {
        vec![
            chevron(490, -1, X_MID, 110, 130),
            chevron(630, -1, X_MID, 110, 130),
        ]
    }),
    ('‹', || vec![chevron(445, 1, X_MID, 110, 130)]),
    ('›', || vec![chevron(555, -1, X_MID, 110, 130)]),
    ('-', || vec![line([(390, X_MID), (610, X_MID)])]),
    ('–', || vec![line([(300, X_MID), (700, X_MID)])]),
    ('—', || vec![line([(120, X_MID), (880, X_MID)])]),
    ('…', || dots(3, BASE - 15, 170)),
    ('…', || dots(3, 500, 333)),
    ('‥', || dots(2, 500, 500)),
    ('‥', || dots(2, BASE - 15, 170)),
    ('·', || vec![dot(500, X_MID - 20)]),
    ('•', || {
        vec![Pen::new().arc(500, X_MID - 20, 45, 45, 90, 450)]
    }),
    ('+', || {
        vec![
            line([(500, 500), (500, 800)]),
            line([(350, 650), (650, 650)]),
        ]
    }),
    ('×', || {
        vec![
            line([(390, 540), (610, 760)]),
            line([(610, 540), (390, 760)]),
        ]
    }),
    ('÷', || {
        vec![line([(340, 650), (660, 650)]), dot(500, 530), dot(500, 770)]
    }),
    ('=', || {
        vec![
            line([(350, 590), (650, 590)]),
            line([(350, 710), (650, 710)]),
        ]
    }),
    ('<', || vec![line([(640, 500), (360, 650), (640, 800)])]),
    ('>', || vec![line([(360, 500), (640, 650), (360, 800)])]),
    ('/', || vec![line([(650, CAP), (350, BASE)])]),
    ('\\', || vec![line([(350, CAP), (650, BASE)])]),
    ('|', || vec![line([(500, ASC), (500, DESC)])]),
    ('_', || vec![line([(270, 940), (730, 940)])]),
    ('^', || vec![line([(390, 380), (500, CAP), (610, 380)])]),
    ('`', || vec![line([(450, CAP), (550, 290)])]),
    ('~', || {
        let wave = (-150..=150).step_by(15).map(|t| {
            let t = f64::from(t);
            (
                500.0 + t,
                650.0 + 40.0 * (std::f64::consts::PI * t / 150.0).sin(),
            )
        });
        vec![Pen(wave.collect())]
    }),
    ('(', paren),
    (')', || mirror(paren())),
    ('[', square_bracket),
    (']', || mirror(square_bracket())),
    ('{', brace),
    ('}', || mirror(brace())),
    ('*', || {
        vec![
            line([(500, CAP), (500, 470)]),
            line([(380, 260), (620, 400)]),
            line([(620, 260), (380, 400)]),
        ]
    }),
    // The a (bowl, then the stem), and without lifting on round the outer
    // circle.
    ('@', || {
        vec![
            Pen::new()
                .arc(480, 600, 100, 115, 10, 360)
                .line([(580, 690)])
                .curve([
                    (640, 720),
                    (710, 690),
                    (760, 610),
                    (770, 520),
                    (734, 415),
                    (635, 316),
                    (500, 280),
                    (365, 316),
                    (266, 415),
                    (230, 550),
                    (266, 685),
                    (365, 784),
                    (500, 820),
                    (640, 800),
                ]),
        ]
    }),
    ('#', || {
        vec![
            line([(450, 230), (410, 810)]),
            line([(590, 230), (550, 810)]),
            line([(330, 410), (680, 410)]),
            line([(320, 630), (670, 630)]),
        ]
    }),
    ('$', || {
        vec![s(240.0, 790.0), line([(500, 160), (500, 870)])]
    }),
    ('%', || {
        vec![
            Pen::new().arc(370, 330, 70, 80, 90, 450),
            line([(660, 210), (340, 830)]),
            Pen::new().arc(630, 710, 70, 80, 90, 450),
        ]
    }),
    // From the foot of the leg up to the head, round the head loop, down
    // round the bowl and out into the arm.
    ('&', || {
        vec![Pen::at((700, BASE)).curve([
            (560, 660),
            (440, 470),
            (395, 360),
            (415, 260),
            (485, 222),
            (550, 255),
            (560, 335),
            (500, 420),
            (400, 510),
            (330, 620),
            (335, 750),
            (420, 840),
            (530, 845),
            (620, 760),
            (670, 640),
        ])]
    }),
    // The "Et" form: an ɛ, then a vertical stroke through it.
    ('&', || {
        vec![
            Pen::at((640, 280)).curve([
                (560, 220),
                (450, 225),
                (390, 300),
                (420, 400),
                (510, 460),
                (400, 510),
                (350, 620),
                (390, 760),
                (500, 840),
                (620, 820),
                (670, 760),
            ]),
            line([(560, CAP), (560, DESC - 40)]),
        ]
    }),
    ('°', || vec![Pen::new().arc(500, 270, 60, 60, 90, 450)]),
    ('§', || vec![s(190.0, 590.0), s(450.0, 850.0)]),
    ('€', || {
        vec![
            Pen::new().arc(560, 520, 230, 330, 45, 315),
            line([(290, 440), (600, 440)]),
            line([(290, 590), (570, 590)]),
        ]
    }),
    // From the hook at the top right, over and down the stem, a kink at the
    // foot and along the base; then the bar.
    ('£', || {
        vec![
            Pen::at((650, 300))
                .curve([
                    (610, 220),
                    (520, 195),
                    (440, 240),
                    (420, 350),
                    (430, 550),
                    (420, 710),
                    (360, 840),
                ])
                .line([(680, 840)]),
            line([(320, 530), (560, 530)]),
        ]
    }),
    ('¥', || {
        vec![
            line([(320, CAP), (500, 520)]),
            line([(680, CAP), (500, 520), (500, BASE)]),
            line([(340, 580), (660, 580)]),
            line([(340, 700), (660, 700)]),
        ]
    }),
    ('′', || vec![line([(540, CAP), (480, 330)])]),
    ('″', || {
        vec![
            line([(480, CAP), (420, 330)]),
            line([(600, CAP), (540, 330)]),
        ]
    }),
    ('→', || {
        vec![
            line([(250, X_MID), (750, X_MID)]),
            line([(650, 560), (750, X_MID), (650, 740)]),
        ]
    }),
    ('←', || {
        vec![
            line([(750, X_MID), (250, X_MID)]),
            line([(350, 560), (250, X_MID), (350, 740)]),
        ]
    }),
    ('↑', || {
        vec![
            line([(500, BASE), (500, CAP)]),
            line([(410, 290), (500, CAP), (590, 290)]),
        ]
    }),
    ('↓', || {
        vec![
            line([(500, CAP), (500, BASE)]),
            line([(410, 750), (500, BASE), (590, 750)]),
        ]
    }),
    // CJK marks, in the em square.
    ('。', || {
        vec![Pen::new().arc(210, 770, 105, 105, -90, -450)]
    }),
    ('、', || {
        vec![Pen::at((110, 700)).curve([(200, 770), (290, 880)])]
    }),
    ('「', corner_bracket),
    ('」', || turned(corner_bracket())),
    ('『', white_corner_bracket),
    ('』', || turned(white_corner_bracket())),
    ('《', double_angle_bracket),
    ('》', || mirror(double_angle_bracket())),
    ('〈', angle_bracket),
    ('〉', || mirror(angle_bracket())),
    ('【', lenticular_bracket),
    ('】', || mirror(lenticular_bracket())),
    ('〔', shell_bracket),
    ('〕', || mirror(shell_bracket())),
    ('―', || vec![line([(40, 500), (960, 500)])]),
    ('・', || vec![dot(500, 500)]),
    ('〝', || {
        vec![
            line([(720, 60), (880, 160)]),
            line([(720, 220), (880, 320)]),
        ]
    }),
    ('〞', || {
        vec![
            line([(280, 60), (120, 160)]),
            line([(280, 220), (120, 320)]),
        ]
    }),
    ('〟', || {
        vec![
            line([(120, 640), (280, 740)]),
            line([(120, 800), (280, 900)]),
        ]
    }),
    // The cross, then the dots top, left, right, bottom.
    ('※', || {
        vec![
            line([(150, 150), (850, 850)]),
            line([(850, 150), (150, 850)]),
            dot(500, 180),
            dot(180, 500),
            dot(820, 500),
            dot(500, 820),
        ]
    }),
    ('〇', || vec![Pen::new().arc(500, 500, 400, 420, 90, 450)]),
];

fn paren() -> Vec<Pen> {
    vec![Pen::at((590, ASC)).curve([(480, 300), (440, 560), (480, 820), (590, DESC)])]
}

fn square_bracket() -> Vec<Pen> {
    vec![line([(590, ASC), (430, ASC), (430, DESC), (590, DESC)])]
}

/// From the upper end, down to a point at mid-height, and on down.
fn brace() -> Vec<Pen> {
    vec![
        Pen::at((600, ASC))
            .curve([(530, 175), (510, 250), (510, 450), (480, 540), (420, 562)])
            .curve([(480, 585), (510, 675), (510, 875), (530, 950), (600, DESC)]),
    ]
}

/// 「 in one stroke: leftwards along the top into the corner, then down.
fn corner_bracket() -> Vec<Pen> {
    vec![line([(940, 60), (680, 60), (680, 680)])]
}

/// 『 as two nested 「, the outer one first.
fn white_corner_bracket() -> Vec<Pen> {
    vec![
        line([(950, 50), (600, 50), (600, 690)]),
        line([(950, 170), (720, 170), (720, 690)]),
    ]
}

fn angle_bracket() -> Vec<Pen> {
    vec![chevron(620, 1, 500, 320, 450)]
}

/// 《 as two 〈, the outer one first.
fn double_angle_bracket() -> Vec<Pen> {
    vec![
        chevron(550, 1, 500, 250, 450),
        chevron(700, 1, 500, 250, 450),
    ]
}

fn shell_bracket() -> Vec<Pen> {
    vec![line([(940, 40), (760, 150), (760, 850), (940, 960)])]
}

/// 【 as its outline in one stroke: the outer bracket from the top, then
/// back up the inner curve.
fn lenticular_bracket() -> Vec<Pen> {
    vec![line([(960, 40), (680, 40), (680, 960), (960, 960)]).curve([
        (880, 870),
        (800, 720),
        (780, 500),
        (800, 280),
        (880, 130),
        (960, 40),
    ])]
}
