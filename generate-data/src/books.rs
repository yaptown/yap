//! Book corpora have two explicit uses: legacy `books/` is gold-training prose;
//! `reader-books/` ships in courses after the ordinary sentence-quality filters.
//! Reader attribution is derived from canonical reader editions, never copied metadata.
use std::{collections::HashSet, path::Path};

use anyhow::{Context, ensure};
use language_utils::{BookMetadata, Language, SentenceSource, text_cleanup::cleanup_sentence};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

/// Legacy training-book extraction, produced by `translate_book`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookSentence {
    pub sentence: String,
    pub book: String,
    pub chunk_id: String,
}

#[derive(Default)]
pub struct BookCatalog {
    pub metadata: FxHashMap<String, BookMetadata>,
    reader_ids: HashSet<String>,
}

impl BookCatalog {
    /// A reader attribution is a shipping source. Legacy book-only prose stays
    /// training-only; merging it with any shipping provenance keeps existing behavior.
    pub fn is_training_only(&self, source: &SentenceSource) -> bool {
        source.is_book_only()
            && !source
                .book_ids
                .iter()
                .any(|id| self.reader_ids.contains(id))
    }
}

pub struct BookCorpus {
    pub sentences: Vec<(String, SentenceSource)>,
    pub catalog: BookCatalog,
}

fn source(book: String) -> SentenceSource {
    let mut source = SentenceSource::none();
    source.book_ids.push(book);
    source
}

impl BookCorpus {
    pub fn load(data_root: &Path, reader_root: &Path, language: Language) -> anyhow::Result<Self> {
        let mut corpus = Self {
            sentences: vec![],
            catalog: BookCatalog::default(),
        };
        // Legacy training sources intentionally share the dialect corpus directories.
        let training = data_root
            .join(language.corpus_code())
            .join("sentence-sources/books");
        if training.is_dir() {
            for series in std::fs::read_dir(training)? {
                let series = series?.path();
                if !series.is_dir() {
                    continue;
                }
                for entry in std::fs::read_dir(&series)? {
                    let path = entry?.path();
                    if path.extension().is_none_or(|e| e != "jsonl") {
                        continue;
                    }
                    let content = std::fs::read_to_string(&path)?;
                    for line in content.lines().filter(|line| !line.trim().is_empty()) {
                        if path
                            .file_name()
                            .is_some_and(|name| name == "metadata.jsonl")
                        {
                            let book: BookMetadata = serde_json::from_str(line)
                                .context("parse training-book metadata")?;
                            ensure!(
                                series.file_name().is_some_and(|name| *name == *book.series),
                                "book {:?} declares series {:?} but lives in {}",
                                book.id,
                                book.series,
                                series.display()
                            );
                            corpus.catalog.metadata.insert(book.id.clone(), book);
                        } else {
                            let row: BookSentence = serde_json::from_str(line)?;
                            if !row.sentence.trim().is_empty() {
                                // No new length/quality filter on the gold-training corpus.
                                corpus
                                    .sentences
                                    .push((row.sentence.trim().to_owned(), source(row.book)));
                            }
                        }
                    }
                }
            }
        }
        for metadata in crate::reader_books::load_catalog(reader_root, language)? {
            let path = crate::reader_book_sentences::directory(data_root, language)
                .join(format!("{}.jsonl", metadata.id));
            if path.is_file() {
                for row in crate::reader_book_sentences::read_records(&path)? {
                    ensure!(
                        row.book == metadata.id,
                        "reader sentence book differs from filename: {}",
                        path.display()
                    );
                    let text = cleanup_sentence(row.sentence, language);
                    let text = crate::reader_book_sentences::strip_dialogue_dash(&text).to_owned();
                    if movie_subtitles::sentences::should_include_sentence(&text, language) {
                        corpus.sentences.push((text, source(row.book)));
                    }
                }
            }
            let id = metadata.id.clone();
            ensure!(
                !corpus.catalog.metadata.contains_key(&id),
                "reader/training book id collision: {id}"
            );
            corpus.catalog.reader_ids.insert(id.clone());
            corpus.catalog.metadata.insert(id, metadata);
        }
        Ok(corpus)
    }
}

/// Deterministic sentence splitter for Chinese: split after runs of 。！？…
/// (carrying along any closing quotes/brackets) and at line breaks. Lossless —
/// the concatenation of the output is the input minus whitespace.
pub fn segment_chinese(text: &str) -> Vec<String> {
    const TERMINALS: [char; 4] = ['。', '！', '？', '…'];
    const CLOSERS: [char; 5] = ['”', '』', '」', '）', '"'];
    let mut sentences = Vec::new();
    let mut current = String::new();
    let mut flush = |current: &mut String| {
        let sentence = current.trim();
        if !sentence.is_empty() {
            sentences.push(sentence.to_string());
        }
        current.clear();
    };
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\n' {
            flush(&mut current);
            continue;
        }
        current.push(c);
        if TERMINALS.contains(&c) {
            while chars
                .peek()
                .is_some_and(|c| TERMINALS.contains(c) || CLOSERS.contains(c))
            {
                current.push(chars.next().unwrap());
            }
            flush(&mut current);
        }
    }
    flush(&mut current);
    sentences
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader_book_sentences::SentenceRecord;
    use language_utils::book::{Book, BookChapter, BookParagraph};

    fn original(root: &Path, slug: &str, language: Language) {
        let path = root.join(slug);
        std::fs::create_dir_all(&path).unwrap();
        let book = Book {
            slug: slug.into(),
            title: format!("Title {slug}"),
            author: "Author".into(),
            original_language: language.code().into(),
            copyright: None,
            chapters: vec![BookChapter {
                position: 0,
                part: None,
                language: language.code().into(),
                title: "Chapter".into(),
                machine_translated: false,
                paragraphs: vec![BookParagraph::Paragraph("A small sentence.".into())],
            }],
        };
        std::fs::write(path.join("source.json"), serde_json::to_vec(&book).unwrap()).unwrap();
    }
    fn reader_rows(data: &Path, language: Language, slug: &str, texts: &[String]) {
        let path = crate::reader_book_sentences::directory(data, language);
        std::fs::create_dir_all(&path).unwrap();
        let rows = texts
            .iter()
            .map(|sentence| {
                serde_json::to_string(&SentenceRecord {
                    sentence: sentence.clone(),
                    book: slug.into(),
                    chapter: 0,
                    paragraph: 0,
                })
                .unwrap()
            })
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(path.join(format!("{slug}.jsonl")), rows).unwrap();
    }
    #[test]
    fn reader_books_ship_but_training_only_prose_does_not() {
        let root = tempfile::tempdir().unwrap();
        let data = root.path().join("data");
        let books = root.path().join("books");
        original(&books, "reader", Language::English);
        reader_rows(
            &data,
            Language::English,
            "reader",
            &[
                "A small sentence.".into(),
                format!("{}.", "word ".repeat(30)),
            ],
        );
        let training = data.join("eng/sentence-sources/books/legacy");
        std::fs::create_dir_all(&training).unwrap();
        let long = format!("{}.", "training ".repeat(40));
        std::fs::write(
            training.join("legacy.jsonl"),
            serde_json::to_string(&BookSentence {
                sentence: long.clone(),
                book: "legacy".into(),
                chunk_id: "c0".into(),
            })
            .unwrap(),
        )
        .unwrap();
        let corpus = BookCorpus::load(&data, &books, Language::English).unwrap();
        assert_eq!(corpus.sentences.len(), 2); // Reader too-long row rejected; training long row retained.
        let reader = corpus
            .sentences
            .iter()
            .find(|(_, source)| source.book_ids == ["reader"])
            .unwrap()
            .clone();
        assert!(!reader.1.from_manual && !reader.1.from_tatoeba);
        let training_source = source("legacy".into());
        let mut overlap = training_source.clone();
        overlap.from_tatoeba = true;
        let mut reader_overlap = training_source.clone();
        reader_overlap.merge(&reader.1);
        let (training, app) = crate::target_sentences::partition_book_sentences(
            vec![
                (reader.0, None, reader.1),
                (long, None, training_source),
                ("Existing overlap.".into(), None, overlap),
                ("Reader overlap.".into(), None, reader_overlap),
            ],
            &corpus.catalog,
        );
        assert_eq!(training.len(), 1);
        assert_eq!(training[0].2.book_ids, ["legacy"]);
        assert_eq!(app.len(), 3);
        assert_eq!(app[0].2.book_ids, ["reader"]);
        assert_eq!(app[2].2.book_ids, ["legacy", "reader"]);
        assert_eq!(corpus.catalog.metadata["reader"].title, "Title reader");
    }
    #[test]
    fn reader_edition_directories_do_not_mix_sibling_dialects() {
        for (a, b) in [
            (Language::SpanishLatinAmerican, Language::SpanishPeninsular),
            (Language::PortugueseBrazilian, Language::PortugueseEuropean),
        ] {
            let root = tempfile::tempdir().unwrap();
            let data = root.path().join("data");
            let books = root.path().join("books");
            original(&books, "first", a);
            original(&books, "second", b);
            reader_rows(&data, a, "first", &["Hola amigo.".into()]);
            reader_rows(&data, b, "second", &["Hola amigo.".into()]);
            // Fresh-checkout attribution needs only manuscript metadata, never builds.
            let manuscript = books.join("translated");
            std::fs::create_dir_all(&manuscript).unwrap();
            std::fs::write(manuscript.join("metadata.json"),r#"{"translate":true,"title":"Translated book","author":"Author","original_language":"eng"}"#).unwrap();
            reader_rows(&data, a, "translated", &["Primera frase.".into()]);
            reader_rows(&data, b, "translated", &["Segunda frase.".into()]);
            let a_books = BookCorpus::load(&data, &books, a).unwrap();
            let b_books = BookCorpus::load(&data, &books, b).unwrap();
            assert!(a_books.catalog.reader_ids.contains("first"));
            assert!(!a_books.catalog.reader_ids.contains("second"));
            assert!(b_books.catalog.reader_ids.contains("second"));
            assert!(!b_books.catalog.reader_ids.contains("first"));
            assert_eq!(a_books.sentences[0].1.book_ids, ["first"]);
            assert_eq!(b_books.sentences[0].1.book_ids, ["second"]);
            assert_eq!(
                a_books
                    .sentences
                    .iter()
                    .find(|(_, s)| s.book_ids == ["translated"])
                    .unwrap()
                    .0,
                "Primera frase."
            );
            assert_eq!(
                b_books
                    .sentences
                    .iter()
                    .find(|(_, s)| s.book_ids == ["translated"])
                    .unwrap()
                    .0,
                "Segunda frase."
            );
            assert_ne!(
                crate::reader_book_sentences::directory(&data, a),
                crate::reader_book_sentences::directory(&data, b)
            );
        }
    }
    #[test]
    fn chinese_splitter_keeps_terminals_closers_and_text() {
        let text = "“方源，交出春秋蝉！”他大喝。这里早已布下天罗地网……\n\n在他看来——\n\n无非是早死、晚死的区别罢了。";
        let parts = segment_chinese(text);
        assert_eq!(
            parts,
            vec![
                "“方源，交出春秋蝉！”",
                "他大喝。",
                "这里早已布下天罗地网……",
                "在他看来——",
                "无非是早死、晚死的区别罢了。"
            ]
        );
        let strip = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
        assert_eq!(strip(&parts.concat()), strip(text));
    }
}
