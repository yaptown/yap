//! Shared source discovery for the book builder and publisher.
use std::path::{Path, PathBuf};

use anyhow::{Context, ensure};
use language_utils::book::Book;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BuildOptions {
    /// Manuscripts are translated; canonical public-domain books retain their original edition.
    translate: bool,
}

pub struct Source {
    pub directory: PathBuf,
    pub slug: String,
    pub translate: bool,
}

pub fn discover(root: &Path) -> anyhow::Result<Vec<Source>> {
    let mut sources = vec![];
    for entry in std::fs::read_dir(root)? {
        let directory = entry?.path();
        let options = directory.join("metadata.json");
        let translate = if directory.join("source.json").is_file() {
            false
        } else if options.is_file() {
            serde_json::from_slice::<BuildOptions>(&std::fs::read(options)?)?.translate
        } else {
            continue;
        };
        sources.push(Source {
            slug: directory
                .file_name()
                .unwrap()
                .to_str()
                .context("book slug is not UTF-8")?
                .into(),
            translate,
            directory,
        });
    }
    sources.sort_by(|a, b| a.slug.cmp(&b.slug));
    ensure!(
        !sources.is_empty(),
        "no book sources found in {}",
        root.display()
    );
    Ok(sources)
}

impl Source {
    pub fn build(&self) -> PathBuf {
        self.directory.join("build")
    }

    pub fn validate(&self, book: &Book) -> anyhow::Result<()> {
        ensure!(
            book.slug == self.slug,
            "{}: source slug differs from directory",
            self.slug
        );
        ensure!(!book.chapters.is_empty(), "{}: empty book", self.slug);
        for (position, chapter) in book.chapters.iter().enumerate() {
            ensure!(
                chapter.position == position
                    && chapter.language == book.original_language
                    && !chapter.machine_translated
                    && !chapter.paragraphs.is_empty(),
                "{}: invalid original chapter {position}",
                self.slug
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use language_utils::book::{BookChapter, BookParagraph};

    #[test]
    fn public_domain_originals_cover_course_languages() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../books");
        let mut languages = std::collections::BTreeSet::new();
        for source in discover(&root).unwrap() {
            if source.translate {
                continue;
            }
            let book: Book = serde_json::from_slice(
                &std::fs::read(source.directory.join("source.json")).unwrap(),
            )
            .unwrap();
            source.validate(&book).unwrap();
            let language = language_utils::Language::from_code(&book.original_language).unwrap();
            for chapter in &book.chapters {
                assert!(!chapter.title.trim().is_empty());
                assert_eq!(
                    chapter.title,
                    language_utils::text_cleanup::cleanup_sentence(chapter.title.clone(), language)
                );
                for paragraph in &chapter.paragraphs {
                    if let BookParagraph::Paragraph(text) = paragraph {
                        assert!(!text.trim().is_empty());
                        assert_eq!(
                            text,
                            &language_utils::text_cleanup::cleanup_sentence(text.clone(), language)
                        );
                        for artifact in [
                            "Template:",
                            "Public domainPublic domain",
                            "一作「",
                            "शीर्ष पर वापस",
                            "�",
                        ] {
                            assert!(
                                !text.contains(artifact),
                                "{} contains {artifact}",
                                book.slug
                            );
                        }
                    }
                }
            }
            languages.insert(book.original_language);
        }
        for course in language_utils::COURSES {
            assert!(
                languages.contains(course.target_language.code()),
                "missing original {}",
                course.target_language.code()
            );
        }
    }

    #[test]
    fn discovers_canonical_books_and_manuscripts() {
        let root = tempfile::tempdir().unwrap();
        let directory = root.path().join("sample");
        std::fs::create_dir(&directory).unwrap();
        std::fs::create_dir(root.path().join("ignored")).unwrap();
        let mut book = Book {
            slug: "sample".into(),
            title: "Sample".into(),
            author: "Author".into(),
            original_language: "fra".into(),
            copyright: Some("1900; public domain".into()),
            chapters: vec![BookChapter {
                position: 0,
                part: None,
                language: "fra".into(),
                title: "Chapter".into(),
                machine_translated: false,
                paragraphs: vec![BookParagraph::Paragraph("Text".into())],
            }],
        };
        std::fs::write(
            directory.join("source.json"),
            serde_json::to_vec(&book).unwrap(),
        )
        .unwrap();
        let manuscript = root.path().join("translated-manuscript");
        std::fs::create_dir(&manuscript).unwrap();
        std::fs::write(manuscript.join("metadata.json"), r#"{"translate":true}"#).unwrap();
        let sources = discover(root.path()).unwrap();
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].build(), directory.join("build"));
        assert!(!sources[0].translate);
        assert!(sources[1].translate);
        sources[0].validate(&book).unwrap();
        book.chapters[0].machine_translated = true;
        assert!(sources[0].validate(&book).is_err());
        book.chapters[0].machine_translated = false;
        book.chapters[0].position = 1;
        assert!(sources[0].validate(&book).is_err());
    }
}
