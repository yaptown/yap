//! Canonical reader sources and manuscript metadata, shared by publishing and corpora.
use std::path::{Path, PathBuf};

use anyhow::{Context, ensure};
use language_utils::{
    BookMetadata, Language,
    book::{Book, BookChapter, BookParagraph},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ManuscriptMetadata {
    pub translate: bool,
    pub title: String,
    pub author: String,
    pub original_language: String,
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
            serde_json::from_slice::<ManuscriptMetadata>(&std::fs::read(options)?)?.translate
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

    pub fn manuscript_metadata(&self) -> anyhow::Result<ManuscriptMetadata> {
        Ok(serde_json::from_slice(&std::fs::read(
            self.directory.join("metadata.json"),
        )?)?)
    }

    fn read_original(&self) -> anyhow::Result<Book> {
        let book: Book =
            serde_json::from_slice(&std::fs::read(self.directory.join("source.json"))?)?;
        self.validate(&book)?;
        Ok(book)
    }

    fn built_languages(&self) -> anyhow::Result<Vec<Language>> {
        let mut languages = vec![];
        for entry in std::fs::read_dir(self.build())? {
            let path = entry?.path();
            if path.is_dir() {
                languages.push(
                    Language::from_code(path.file_name().unwrap().to_str().unwrap())
                        .context("unknown reader edition directory")?,
                );
            }
        }
        languages.sort_by_key(|language| language.code());
        Ok(languages)
    }

    fn built_chapters(&self, language: Language) -> anyhow::Result<Vec<BookChapter>> {
        let directory = self.build().join(language.code());
        if !directory.is_dir() {
            return Ok(vec![]);
        }
        let mut chapters = vec![];
        for entry in std::fs::read_dir(directory)? {
            let path = entry?.path();
            if path
                .extension()
                .is_some_and(|extension| extension == "json")
            {
                chapters.push(serde_json::from_slice::<BookChapter>(&std::fs::read(
                    path,
                )?)?);
            }
        }
        chapters.sort_by_key(|chapter| chapter.position);
        ensure!(
            chapters
                .windows(2)
                .all(|pair| pair[0].position < pair[1].position),
            "duplicate reader chapter positions: {}",
            self.slug
        );
        ensure!(
            chapters
                .iter()
                .all(|chapter| chapter.language == language.code()),
            "reader edition language mismatch: {}",
            self.slug
        );
        Ok(chapters)
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

pub struct Edition {
    pub language: Language,
    pub metadata: BookMetadata,
    pub chapters: Vec<BookChapter>,
}

impl Edition {
    pub fn slug(&self) -> &str {
        &self.metadata.id
    }
    /// Keep source positions, including skipped scene-break slots.
    pub fn paragraphs(&self) -> impl Iterator<Item = (usize, usize, &str)> {
        self.chapters.iter().flat_map(|chapter| {
            chapter
                .paragraphs
                .iter()
                .enumerate()
                .filter_map(move |(index, paragraph)| match paragraph {
                    BookParagraph::Paragraph(text) => {
                        Some((chapter.position, index, text.as_str()))
                    }
                    BookParagraph::SceneBreak => None,
                })
        })
    }
}

/// Segmentation needs manuscript chapter text. Attribution alone never needs builds.
pub fn load_editions(root: &Path, only: Option<Language>) -> anyhow::Result<Vec<Edition>> {
    let mut editions = vec![];
    for source in discover(root)? {
        if source.translate {
            let info = source.manuscript_metadata()?;
            let languages = if let Some(language) = only {
                vec![language]
            } else {
                source.built_languages()?
            };
            for language in languages {
                let chapters = source.built_chapters(language)?;
                if !chapters.is_empty() {
                    editions.push(Edition {
                        language,
                        metadata: metadata(
                            &source.slug,
                            &info.title,
                            &info.author,
                            &info.original_language,
                            language,
                        )?,
                        chapters,
                    });
                }
            }
        } else {
            let book = source.read_original()?;
            let language = Language::from_code(&book.original_language)
                .context("unknown original book language")?;
            if only.is_none_or(|only| only == language) {
                editions.push(Edition {
                    language,
                    metadata: metadata(
                        &source.slug,
                        &book.title,
                        &book.author,
                        &book.original_language,
                        language,
                    )?,
                    chapters: book.chapters,
                });
            }
        }
    }
    Ok(editions)
}

fn metadata(
    slug: &str,
    title: &str,
    author: &str,
    original_language: &str,
    edition_language: Language,
) -> anyhow::Result<BookMetadata> {
    let original =
        Language::from_code(original_language).context("unknown original reader language")?;
    Ok(BookMetadata {
        id: slug.into(),
        series: slug.into(),
        title: title.into(),
        author: author.into(),
        original_language: Some(original.iso_639_1().into()),
        machine_translated: edition_language != original,
        translator: None,
        source_url: None,
    })
}

/// PD attribution comes only from source.json; manuscript attribution comes only
/// from metadata.json. Translations are editions in a different language from the original.
pub fn load_catalog(root: &Path, language: Language) -> anyhow::Result<Vec<BookMetadata>> {
    let mut catalog = vec![];
    for source in discover(root)? {
        if source.translate {
            let info = source.manuscript_metadata()?;
            catalog.push(metadata(
                &source.slug,
                &info.title,
                &info.author,
                &info.original_language,
                language,
            )?);
        } else {
            let book = source.read_original()?;
            if book.original_language == language.code() {
                catalog.push(metadata(
                    &source.slug,
                    &book.title,
                    &book.author,
                    &book.original_language,
                    language,
                )?);
            }
        }
    }
    Ok(catalog)
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
    fn reader_metadata_loads_without_ignored_builds() {
        let root = tempfile::tempdir().unwrap();
        let directory = root.path().join("manuscript");
        std::fs::create_dir(&directory).unwrap();
        std::fs::write(directory.join("metadata.json"),r#"{"translate":true,"title":"Original title","author":"Original author","original_language":"eng"}"#).unwrap();
        let english = load_catalog(root.path(), Language::English).unwrap();
        let french = load_catalog(root.path(), Language::French).unwrap();
        assert_eq!(english[0].title, "Original title");
        assert_eq!(english[0].author, "Original author");
        assert_eq!(english[0].original_language.as_deref(), Some("en"));
        assert!(!english[0].machine_translated);
        assert!(french[0].machine_translated);
        assert!(!directory.join("build").exists());
        assert!(english[0].translator.is_none());
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
        std::fs::write(manuscript.join("metadata.json"), r#"{"translate":true,"title":"Manuscript","author":"Author","original_language":"eng"}"#).unwrap();
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
