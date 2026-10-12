//! Reader-ready books, independent of either frontend or the sentence corpus.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Book {
    pub slug: String,
    pub title: String,
    pub author: String,
    pub original_language: String,
    /// Original legal notice, never translated.
    pub copyright: Option<String>,
    pub chapters: Vec<BookChapter>,
}

/// A complete chapter in one language. Editions may contain only some chapters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookChapter {
    /// Stable, zero-based position in the original, including front matter.
    pub position: usize,
    /// Part label in this chapter’s edition language.
    pub part: Option<String>,
    pub language: String,
    pub title: String,
    pub machine_translated: bool,
    pub paragraphs: Vec<BookParagraph>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "text", rename_all = "snake_case")]
pub enum BookParagraph {
    Paragraph(String),
    SceneBreak,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scene_break_is_explicit_and_roundtrips() {
        let blocks = vec![
            BookParagraph::Paragraph("Hello".into()),
            BookParagraph::SceneBreak,
        ];
        let json = serde_json::to_string(&blocks).unwrap();
        assert!(json.contains("scene_break"));
        assert_eq!(
            serde_json::from_str::<Vec<BookParagraph>>(&json).unwrap(),
            blocks
        );
    }
}
