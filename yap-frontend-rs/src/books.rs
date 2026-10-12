//! Official books and device-local reading state, independent of deck events.
use crate::persistent;
use bridgerton::Error;
use language_utils::{
    Language,
    book::{Book, BookParagraph},
};
use opfs::{DirectoryHandle as _, FileHandle as _, WritableFileStream as _};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BooksCopy {
    pub title: String,
    pub home: String,
    pub contents: String,
    pub loading: String,
    pub empty: String,
    pub failed: String,
    pub previous: String,
    pub next: String,
    pub text_smaller: String,
    pub text_larger: String,
}
#[bridgerton::bridge]
pub fn books_copy() -> BooksCopy {
    BooksCopy {
        title: yap_i18n::tr!("books-title"),
        home: yap_i18n::tr!("common-home"),
        contents: yap_i18n::tr!("books-contents"),
        loading: yap_i18n::tr!("books-loading"),
        empty: yap_i18n::tr!("books-empty"),
        failed: yap_i18n::tr!("books-failed"),
        previous: yap_i18n::tr!("books-previous"),
        next: yap_i18n::tr!("books-next"),
        text_smaller: yap_i18n::tr!("books-text-smaller"),
        text_larger: yap_i18n::tr!("books-text-larger"),
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct CatalogRow {
    slug: String,
    title: String,
    author: String,
    cover: Option<String>,
    teaser: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Structure {
    position: usize,
    part_start: Option<usize>,
    part: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Edition {
    book: Book,
    structure: Vec<Structure>,
}
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BookCardView {
    pub slug: String,
    pub title: String,
    pub author: String,
    pub teaser: Option<String>,
    /// The cover image's URL, when the book has one; hosts fall back to a drawn tile.
    pub cover: Option<String>,
}
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BooksScreenView {
    pub copy: BooksCopy,
    pub books: Vec<BookCardView>,
}
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct BookPosition {
    pub chapter: usize,
    pub paragraph: usize,
}
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BookTocItem {
    pub position: usize,
    pub title: String,
    pub enabled: bool,
}
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BookPartView {
    pub title: Option<String>,
    pub chapters: Vec<BookTocItem>,
}
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BookBlockView {
    pub text: Option<String>,
}
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BookReaderView {
    pub title: String,
    pub author: String,
    pub copyright: Option<String>,
    pub parts: Vec<BookPartView>,
    pub position: BookPosition,
    pub chapter_title: Option<String>,
    /// The author, plus a machine-translation note on translated chapters.
    pub byline: String,
    /// BCP 47 tag for the text, so hosts hyphenate and shape it as this language.
    pub lang: String,
    pub paragraphs: Vec<BookBlockView>,
    pub previous: Option<usize>,
    pub next: Option<usize>,
}
/// Text sizes the reader steps through, in points (CSS pixels on web).
const TEXT_SIZES: [f64; 6] = [16.0, 18.0, 20.0, 22.0, 25.0, 28.0];
const DEFAULT_TEXT_SIZE: u32 = 2;

/// How the reader lays a chapter out on a screen, so web and iOS paginate alike.
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReaderLayout {
    /// Pages shown side by side: a two-page spread when the screen fits one.
    pub columns: u32,
    pub page_width: f64,
    pub page_height: f64,
    /// Between the pages of a spread.
    pub gap: f64,
    /// From the top of the screen to the first line, leaving room for the reader's bar.
    pub top: f64,
    pub font_size: f64,
    /// Line height as a multiple of the font size.
    pub line_height: f64,
    /// Justifying a phone-width column leaves rivers of space; ragged right reads better.
    pub justified: bool,
    /// The text size step this layout uses, and the neighbouring steps if any.
    pub text_size: u32,
    pub smaller: Option<u32>,
    pub larger: Option<u32>,
}

/// `text_size` is a step the host saved earlier, or `None` for the default.
#[bridgerton::bridge]
pub fn reader_layout(width: f64, height: f64, text_size: Option<u32>) -> ReaderLayout {
    let step = text_size
        .unwrap_or(DEFAULT_TEXT_SIZE)
        .min(TEXT_SIZES.len() as u32 - 1);
    let gap = 64.0;
    let (columns, margin, max_width) = if width >= 900.0 {
        (2, 48.0, 1180.0)
    } else {
        (1, 24.0, 620.0)
    };
    let spread = (width - 2.0 * margin).min(max_width);
    let page_width = (spread - gap * (columns as f64 - 1.0)) / columns as f64;
    ReaderLayout {
        columns,
        page_width,
        page_height: height - 144.0,
        gap,
        top: 80.0,
        font_size: TEXT_SIZES[step as usize],
        line_height: 1.6,
        justified: page_width >= 480.0,
        text_size: step,
        smaller: step.checked_sub(1),
        larger: Some(step + 1).filter(|&s| (s as usize) < TEXT_SIZES.len()),
    }
}

fn reader(edition: Edition, language: Language, position: BookPosition) -> BookReaderView {
    let book = edition.book;
    let chapter = book
        .chapters
        .iter()
        .find(|c| c.position == position.chapter)
        .or(book.chapters.first());
    let position = chapter
        .map(|c| BookPosition {
            chapter: c.position,
            paragraph: if c.position == position.chapter {
                position.paragraph.min(c.paragraphs.len().saturating_sub(1))
            } else {
                0
            },
        })
        .unwrap_or_default();
    let mut parts: Vec<BookPartView> = vec![];
    let mut previous_part = None;
    for item in &edition.structure {
        if parts.is_empty() || previous_part != item.part_start {
            parts.push(BookPartView {
                title: item.part.clone(),
                chapters: vec![],
            });
        }
        previous_part = item.part_start;
        let text = book.chapters.iter().find(|c| c.position == item.position);
        parts.last_mut().unwrap().chapters.push(BookTocItem {
            position: item.position,
            title: text.map(|c| c.title.clone()).unwrap_or_else(|| {
                yap_i18n::tr!("books-section-unavailable", number = item.position + 1)
            }),
            enabled: text.is_some(),
        });
    }
    BookReaderView {
        title: book.title.clone(),
        author: book.author.clone(),
        copyright: book.copyright.clone(),
        parts,
        chapter_title: chapter.map(|c| c.title.clone()),
        byline: match chapter.filter(|c| c.machine_translated) {
            Some(_) => format!(
                "{} · {}",
                book.author,
                yap_i18n::tr!("books-machine-translated")
            ),
            None => book.author.clone(),
        },
        lang: match language {
            Language::ChineseSimplified => "zh-Hans".into(),
            Language::ChineseTraditional => "zh-Hant".into(),
            _ => language.iso_639_1().into(),
        },
        paragraphs: chapter
            .map(|c| {
                c.paragraphs
                    .iter()
                    .map(|p| BookBlockView {
                        text: match p {
                            BookParagraph::Paragraph(text) => Some(text.clone()),
                            BookParagraph::SceneBreak => None,
                        },
                    })
                    .collect()
            })
            .unwrap_or_default(),
        previous: book
            .chapters
            .iter()
            .filter(|c| c.position < position.chapter)
            .map(|c| c.position)
            .max(),
        next: book
            .chapters
            .iter()
            .filter(|c| c.position > position.chapter)
            .map(|c| c.position)
            .min(),
        position,
    }
}
fn error(e: impl std::fmt::Debug) -> Error {
    Error::new(format!("{e:?}"))
}
async fn directory() -> Result<persistent::DirectoryHandle, Error> {
    persistent::app_specific_dir()
        .await
        .map_err(error)?
        .get_directory_handle_with_options(
            "books",
            &opfs::GetDirectoryHandleOptions { create: true },
        )
        .await
        .map_err(error)
}
async fn read<T: DeserializeOwned>(key: &str) -> Option<T> {
    let bytes = directory()
        .await
        .ok()?
        .get_file_handle_with_options(key, &opfs::GetFileHandleOptions { create: false })
        .await
        .ok()?
        .read()
        .await
        .ok()?;
    serde_json::from_slice(&bytes).ok()
}
async fn write<T: Serialize>(key: &str, value: &T) -> Result<(), Error> {
    let mut file = directory()
        .await?
        .get_file_handle_with_options(key, &opfs::GetFileHandleOptions { create: true })
        .await
        .map_err(error)?;
    let mut stream = file
        .create_writable_with_options(&opfs::CreateWritableOptions {
            keep_existing_data: false,
        })
        .await
        .map_err(error)?;
    stream
        .write_at_cursor_pos(&serde_json::to_vec(value).map_err(error)?)
        .await
        .map_err(error)?;
    stream.close().await.map_err(error)
}
fn key(language: Language, slug: &str, kind: &str) -> String {
    format!("{}-{slug}-{kind}.json", language.code())
}
async fn rpc<T: DeserializeOwned>(name: &str, body: serde_json::Value) -> Result<T, Error> {
    let config = crate::supabase::supabase_config();
    let response = fetch_happen::Client
        .post(format!("{}/rest/v1/rpc/{name}", config.supabase_url))
        .header("apikey", &config.supabase_anon_key)
        .header(
            "Authorization",
            format!("Bearer {}", config.supabase_anon_key),
        )
        .json(&body)
        .map_err(error)?
        .send()
        .await
        .map_err(error)?;
    if !response.ok() {
        return Err(Error::new(format!("Books HTTP {}", response.status())));
    }
    response.json().await.map_err(error)
}
/// The catalog and an opened book are fetched fresh, so newly published chapters and
/// corrections show up, with the cached copy serving offline. Moving between chapters
/// of an open book reads the cache.
async fn cached<T: DeserializeOwned + Serialize>(
    key: &str,
    prefer_cache: bool,
    name: &str,
    body: serde_json::Value,
) -> Result<T, Error> {
    if prefer_cache && let Some(value) = read(key).await {
        return Ok(value);
    }
    match rpc(name, body).await {
        Ok(value) => {
            write(key, &value).await?;
            Ok(value)
        }
        Err(e) => read(key).await.ok_or(e),
    }
}
#[bridgerton::bridge]
pub async fn load_books(
    language: Language,
    native_language: Language,
) -> Result<BooksScreenView, Error> {
    let rows: Vec<CatalogRow> = cached(
        &format!("{}-{}-catalog.json", language.code(), native_language.code()),
        false,
        "book_catalog",
        serde_json::json!({"requested_language": language.code(), "native_language": native_language.code()}),
    )
    .await?;
    Ok(BooksScreenView {
        copy: books_copy(),
        books: rows
            .into_iter()
            .map(|r| BookCardView {
                slug: r.slug,
                title: r.title,
                author: r.author,
                teaser: r.teaser,
                cover: r
                    .cover
                    .map(|key| format!("{}/{key}", crate::utils::packs_origin())),
            })
            .collect(),
    })
}
#[bridgerton::bridge]
pub async fn load_book(
    language: Language,
    slug: String,
    position: Option<BookPosition>,
) -> Result<BookReaderView, Error> {
    let edition = cached(
        &key(language, &slug, "edition"),
        position.is_some(),
        "book_edition",
        serde_json::json!({"requested_language": language.code(), "requested_slug": slug}),
    )
    .await?;
    let position = match position {
        Some(p) => p,
        None => read(&key(language, &slug, "position"))
            .await
            .unwrap_or_default(),
    };
    Ok(reader(edition, language, position))
}
#[bridgerton::bridge]
pub async fn save_book_position(
    language: Language,
    slug: String,
    position: BookPosition,
) -> Result<(), Error> {
    write(&key(language, &slug, "position"), &position).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use language_utils::book::BookChapter;
    fn edition() -> Edition {
        Edition {
            book: Book {
                slug: "test".into(),
                title: "Test".into(),
                author: "George Phillies".into(),
                original_language: "eng".into(),
                copyright: None,
                chapters: [0, 2]
                    .into_iter()
                    .map(|position| BookChapter {
                        position,
                        part: None,
                        language: "fra".into(),
                        title: format!("Chapter {position}"),
                        machine_translated: true,
                        paragraphs: vec![
                            BookParagraph::Paragraph("Text".into()),
                            BookParagraph::SceneBreak,
                        ],
                    })
                    .collect(),
            },
            structure: (0..3)
                .map(|position| Structure {
                    position,
                    part_start: None,
                    part: None,
                })
                .collect(),
        }
    }
    #[test]
    fn edition_part_titles_are_retained() {
        let mut edition = edition();
        for chapter in &mut edition.book.chapters {
            chapter.part = Some("Première partie".into());
        }
        for item in &mut edition.structure {
            item.part = Some("Première partie".into());
            item.part_start = Some(0);
        }
        let view = reader(edition, Language::French, BookPosition::default());
        assert_eq!(view.parts.len(), 1);
        assert_eq!(view.parts[0].title.as_deref(), Some("Première partie"));
        assert!(!view.parts[0].chapters[1].enabled);
        assert_eq!(
            view.parts[0].chapters[1].title,
            "Section 2 — not yet available"
        );
    }
    #[test]
    fn partial_navigation_and_scene_break() {
        let view = reader(edition(), Language::French, BookPosition::default());
        assert!(!view.parts[0].chapters[1].enabled);
        assert_eq!(view.next, Some(2));
        assert_eq!(view.previous, None);
        assert_eq!(view.paragraphs[1].text, None);
        assert!(view.byline.ends_with("machine translated."));
    }
    #[test]
    fn layout_spreads_on_wide_screens_and_clamps_text_size() {
        let phone = reader_layout(390.0, 844.0, None);
        assert_eq!((phone.columns, phone.justified), (1, false));
        assert_eq!(phone.font_size, 20.0);
        let desktop = reader_layout(1280.0, 800.0, Some(99));
        assert_eq!((desktop.columns, desktop.justified), (2, true));
        assert_eq!((desktop.smaller, desktop.larger), (Some(4), None));
        assert_eq!(reader_layout(390.0, 844.0, Some(0)).smaller, None);
    }
    #[test]
    fn position_clamped_and_missing_chapter_recovers() {
        let view = reader(
            edition(),
            Language::French,
            BookPosition {
                chapter: 2,
                paragraph: 99,
            },
        );
        assert_eq!(view.position.paragraph, 1);
        assert_eq!(view.previous, Some(0));
        assert_eq!(view.next, None);
        let recovered = reader(
            edition(),
            Language::French,
            BookPosition {
                chapter: 1,
                paragraph: 99,
            },
        );
        assert_eq!(recovered.position.chapter, 0);
        assert_eq!(recovered.position.paragraph, 0);
    }
    #[test]
    fn zero_availability_retains_structure() {
        let mut edition = edition();
        edition.book.chapters.clear();
        edition.structure[2].part_start = Some(2);
        let view = reader(edition, Language::French, BookPosition::default());
        assert_eq!(view.parts.len(), 2);
        assert_eq!(view.parts[0].chapters.len(), 2);
        assert_eq!(view.parts[1].chapters.len(), 1);
        assert!(view.parts.iter().all(|part| part.title.is_none()));
        assert!(view.chapter_title.is_none());
        assert_eq!(view.next, None);
    }
}
