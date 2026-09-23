//! Stroke-order centerlines for the Thai script.
//!
//! Conventions follow the Thai Ministry of Education handwriting guide
//! (ตัวอักษรแบบกระทรวงศึกษาธิการ, "หัวกลมตัวมน"): a letter with a head loop (หัว)
//! starts inside the head, goes around it and continues to the end of the
//! letter without lifting the pen; letters without a head start where the
//! guide starts them. Stroke order and direction were checked against
//! ActiveThai's start/end-dot practice sheets.
//!
//! Glyphs are drawn in "writing units": the Thai writing grid with the
//! baseline at y=0, the top line of a consonant body at about y=560 and y
//! pointing up. Stems sit at y≈36 (bottom) and y≈515 (top) because they are
//! centerlines, and a normal head loop has radius 66. Combining marks are drawn
//! relative to the end of their base consonant, x=0, as a font does; for
//! placement they are set on an imaginary อ whose ink is centred 300 units to
//! the left of that point.
//!
//! A stroke is a sequence of waypoints the pen passes through, smoothed with a
//! centripetal Catmull-Rom spline. [`CORNER`] between waypoints makes a sharp
//! turn there instead of a smooth one. [`head`] gives the waypoints of a head
//! loop and [`ring`] those of a closed loop in the middle of a stroke.
use super::{
    CORNER, Draw, Ends, Item, Pt, Strokes, centripetal, collect, corner_runs, glyph, items, pts,
};
use crate::{Forms, StrokeStandard};

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

pub fn glyphs() -> Forms {
    let consonants: [(char, Draw); 46] = [
        ('ก', ko_kai),
        ('ข', kho_khai),
        ('ฃ', kho_khuat),
        ('ค', kho_khwai),
        ('ฅ', kho_khon),
        ('ฆ', kho_rakhang),
        ('ง', ngo_ngu),
        ('จ', cho_chan),
        ('ฉ', cho_ching),
        ('ช', cho_chang),
        ('ซ', so_so),
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
        ('ป', || bo_baimai(715)),
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
        ('ษ', so_ruesi),
        ('ส', so_suea),
        ('ห', ho_hip),
        ('ฬ', lo_chula),
        ('อ', o_ang),
        ('ฮ', ho_nokhuk),
    ];
    let others: [(char, Draw); 21] = [
        ('ฯ', paiyannoi),
        ('ะ', sara_a),
        ('า', || sara_aa(40)),
        ('ำ', sara_am),
        ('เ', || sara_e(0)),
        ('แ', sara_ae),
        ('โ', sara_o),
        ('ใ', sara_ai_maimuan),
        ('ไ', sara_ai_maimalai),
        ('ๅ', || sara_aa(-190)),
        ('ๆ', mai_yamok),
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
    ];
    let marks: [(char, Draw); 8] = [
        ('ั', mai_han_akat),
        ('ิ', sara_i),
        ('ี', sara_ii),
        ('ึ', sara_ue),
        ('ื', sara_uee),
        ('ุ', sara_u),
        ('ู', sara_uu),
        ('็', mai_taikhu),
    ];
    let tone_marks: [(char, Draw); 5] = [
        ('่', mai_ek),
        ('้', mai_tho),
        ('๊', mai_tri),
        ('๋', mai_chattawa),
        ('์', thanthakhat),
    ];
    let to_box = |dx: f64| move |(x, y): Pt| (500.0 + SCALE * (x + dx), BASELINE - SCALE * y);
    // Spacing characters are centred on their own ink.
    let spacing = consonants.into_iter().chain(others).map(|(c, draw)| {
        let strokes = draw();
        let xs = strokes.iter().flatten().map(|p| p.0);
        let min = xs.clone().fold(f64::INFINITY, f64::min);
        let max = xs.fold(f64::NEG_INFINITY, f64::max);
        (
            c,
            glyph(StrokeStandard::Thai, strokes, to_box(-(min + max) / 2.0)),
        )
    });
    let marks = marks.map(|(c, draw)| {
        (
            c,
            glyph(StrokeStandard::Thai, draw(), to_box(MARK_BASE_CENTRE)),
        )
    });
    let tone_marks = tone_marks.map(|(c, draw)| {
        let strokes = draw()
            .into_iter()
            .map(|s| shift(s, 0.0, TONE_LIFT))
            .collect();
        (
            c,
            glyph(StrokeStandard::Thai, strokes, to_box(MARK_BASE_CENTRE)),
        )
    });
    collect(spacing.chain(marks).chain(tone_marks))
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

/// shared by ฎ ฏ: head, notch, arch, right stem down below the line
fn cha_da_body() -> Vec<Item> {
    items![
        head((122, 94), 125, CW),
        (175, 245), (205, 300), (250, 332), (292, 350),
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

/// ห: head, down, diagonal up into the right loop, down the right
fn ho_hip() -> Strokes {
    vec![
        stroke![
            head((120, 466), -15, CW),
            (181, 400), (181, 300), (181, 40),
            CORNER,
            (235, 160), (295, 295), (355, 385), (405, 430),
            ring((469, 466), 205, -240),
            (514, 350), (514, 40),
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

/// the small circle of ำ, anticlockwise from the top
fn nikhahit() -> Vec<Pt> {
    stroke![ring_r((-130, 727), 90, 360, 70)]
}

/// ำ: circle over the consonant, then า
fn sara_am() -> Strokes {
    [vec![nikhahit()], sara_aa(40)].concat()
}

/// ◌ิ is written from the right end of its base (the consonant's back line)
/// leftwards, then up and over the arch. ◌ี ◌ื add strokes that come down onto
/// the arch's end; ◌ึ starts at its head on the right.
fn arch() -> Vec<Pt> {
    pts![(-478, 720), (-445, 760), (-380, 785), (-300, 792)]
}

/// ิ
fn sara_i() -> Strokes {
    vec![stroke![(-135, 665), (-485, 665), CORNER, arch(), (-210, 778), (-160, 752), (-138, 705), (-134, 668)]]
}

fn sara_ii_arch() -> Vec<Pt> {
    stroke![(-150, 665), (-485, 665), CORNER, arch(), (-225, 775), (-185, 745), (-152, 718)]
}

/// ี: ิ, then a stroke down onto its end
fn sara_ii() -> Strokes {
    vec![sara_ii_arch(), stroke![(-126, 805), (-126, 700)]]
}

/// ึ: head on the right, along the base, arch back to the head
fn sara_ue() -> Strokes {
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

/// ็: head on the right, zigzag, round up the left, wavy top
fn mai_taikhu() -> Strokes {
    vec![
        stroke![
            head_r((-179, 713), 190, CCW, 48),
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

/// ๋: across, then down
fn mai_chattawa() -> Strokes {
    vec![stroke![(-265, 750), (-15, 750)], stroke![(-140, 855), (-140, 645)]]
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
