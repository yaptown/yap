//! Directed stroke centerlines in a unit square, with the origin at the top left.

#[bridgerton::bridge(transparent)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub enum StrokeStandard {
    Japan,
    Prc,
    Taiwan,
    Korea,
    /// Scripts with one common print form and no competing national
    /// standards; the pack is authored in this repository.
    Devanagari,
    Thai,
    Latin,
    Cyrillic,
}

#[bridgerton::bridge(transparent)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct Stroke {
    pub points: Vec<(f32, f32)>,
}

#[bridgerton::bridge(transparent)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct StrokeGlyph {
    pub standard: StrokeStandard,
    pub strokes: Vec<Stroke>,
}

/// The stroke data a language pack carries: each writable unit (see
/// `stroke_order::segment`) that a downloaded source draws, with every
/// accepted form, the taught form first. Scripts drawn in code (Devanagari,
/// Thai, Latin, Cyrillic, punctuation) are not stored; `stroke_order`
/// regenerates them.
pub type StrokeTable = rustc_hash::FxHashMap<String, Vec<StrokeGlyph>>;
