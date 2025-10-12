use futures::StreamExt as _;
use language_utils::{
    ArchivedConsolidatedLanguageDataWithCapacity, ConsolidatedLanguageDataWithCapacity, Course,
    Language,
};
use opfs::{
    DirectoryHandle as _, FileHandle as _, WritableFileStream as _,
    persistent::{self, DirectoryHandle},
};
use std::{collections::BTreeMap, sync::LazyLock};
use xxhash_rust::const_xxh3::xxh3_64 as const_xxh3;

use crate::{LanguagePack, utils::hit_ai_server};

static LANGUAGE_DATA_HASHES: LazyLock<BTreeMap<Course, &'static str>> = LazyLock::new(|| {
    let mut hashes = BTreeMap::new();
    hashes.insert(
        Course {
            native_language: Language::English,
            target_language: Language::French,
        },
        include_str!("../../out/fra_for_eng/language_data.hash"),
    );
    hashes.insert(
        Course {
            native_language: Language::French,
            target_language: Language::English,
        },
        include_str!("../../out/eng_for_fra/language_data.hash"),
    );
    hashes.insert(
        Course {
            native_language: Language::English,
            target_language: Language::Spanish,
        },
        include_str!("../../out/spa_for_eng/language_data.hash"),
    );
    hashes.insert(
        Course {
            native_language: Language::English,
            target_language: Language::Korean,
        },
        include_str!("../../out/kor_for_eng/language_data.hash"),
    );
    hashes.insert(
        Course {
            native_language: Language::English,
            target_language: Language::German,
        },
        include_str!("../../out/deu_for_eng/language_data.hash"),
    );
    hashes
});

fn language_data_hash_for_course(course: Course) -> Option<&'static str> {
    LANGUAGE_DATA_HASHES.get(&course).copied()
}

fn course_directory_slug(course: Course) -> String {
    format!(
        "{}_for_{}",
        course.target_language.iso_639_3(),
        course.native_language.iso_639_3()
    )
}

pub(crate) async fn get_language_pack(
    data_directory_handle: &DirectoryHandle,
    course: Course,
    set_loading_state: &impl Fn(&str),
) -> Result<LanguagePack, LanguageDataError> {
    let course_directory = course_directory_slug(course);
    let mut language_directory = data_directory_handle
        .get_directory_handle_with_options(
            &course_directory,
            &opfs::GetDirectoryHandleOptions { create: true },
        )
        .await
        .map_err(LanguageDataError::Persistent)?;

    let language_data_hash = language_data_hash_for_course(course)
        .ok_or(LanguageDataError::UnsupportedCourse(course))?;
    log::info!(
        "expected language_data_hash for {:?}->{:?}: {language_data_hash}",
        course.native_language,
        course.target_language
    );
    let language_data_hash_file = language_directory
        .get_file_handle_with_options(
            &format!("language_data_{language_data_hash}.rkyv"),
            &opfs::GetFileHandleOptions { create: false },
        )
        .await;

    let bytes = if let Ok(language_data_hash_file) = language_data_hash_file {
        // Cache hit - read from local storage
        log::info!("reading language data from local storage");
        let bytes = language_data_hash_file
            .read()
            .await
            .map_err(LanguageDataError::Persistent)?;
        let computed_hash = const_xxh3(&bytes);
        let expected_hash: u64 = language_data_hash.parse().unwrap();
        if computed_hash != expected_hash {
            log::warn!(
                "Language data hash mismatch! Expected: {expected_hash}, Got: {computed_hash}"
            );
            download_and_cache_language_data(
                &mut language_directory,
                course,
                language_data_hash,
                set_loading_state,
            )
            .await?
        } else {
            log::info!("Language data from local storage hash matches expectation");
            bytes
        }
    } else {
        download_and_cache_language_data(
            &mut language_directory,
            course,
            language_data_hash,
            set_loading_state,
        )
        .await?
    };

    set_loading_state("Deserializing language data");
    // Common deserialization logic for both cache hit and miss
    let archived = rkyv::access::<ArchivedConsolidatedLanguageDataWithCapacity, rkyv::rancor::Error>(
        &bytes[..],
    );

    let deserialized = match archived {
        Ok(archived) => {
            rkyv::deserialize::<ConsolidatedLanguageDataWithCapacity, rkyv::rancor::Error>(archived)
                .inspect_err(|e| {
                    log::error!("Error deserializing language data: {e:?}");
                })
                .unwrap()
        }
        Err(e) => {
            log::error!("Error when accessing language data: {e}\nre-downloading language data");
            let bytes = download_and_cache_language_data(
                &mut language_directory,
                course,
                language_data_hash,
                set_loading_state,
            )
            .await?;
            let archived = rkyv::access::<
                ArchivedConsolidatedLanguageDataWithCapacity,
                rkyv::rancor::Error,
            >(&bytes[..])
            .inspect_err(|e| {
                log::error!("2nd error accessing language data: {e:?}");
            })
            .map_err(LanguageDataError::Rkyv)?;
            rkyv::deserialize::<ConsolidatedLanguageDataWithCapacity, rkyv::rancor::Error>(archived)
                .inspect_err(|e| {
                    log::error!("Error deserializing language data: {e:?}");
                })
                .map_err(LanguageDataError::Rkyv)?
        }
    };

    set_loading_state("Storing language data in memory");
    let language_pack = LanguagePack::new(deserialized);

    Ok(language_pack)
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum LanguageDataError {
    #[error("OPFS error: {0:?}")]
    Persistent(persistent::Error),

    #[error("Rkyv error")]
    Rkyv(#[source] rkyv::rancor::Error),

    #[error("AI server error:")]
    AiServer(#[source] fetch_happen::Error),

    #[error("Unsupported course: {0:?}")]
    UnsupportedCourse(Course),
}

async fn download_and_cache_language_data(
    language_directory_handle: &mut DirectoryHandle,
    course: Course,
    language_data_hash: &'static str,
    set_loading_state: &impl Fn(&str),
) -> Result<Vec<u8>, LanguageDataError> {
    set_loading_state(&format!(
        "Downloading {:?}->{:?} language data",
        course.native_language, course.target_language
    ));
    log::info!(
        "Language data cache miss for {:?}->{:?}, fetching from server...",
        course.native_language,
        course.target_language
    );
    let response = hit_ai_server(
        fetch_happen::Method::POST,
        "/language-data",
        Some(course),
        None,
    )
    .await
    .map_err(LanguageDataError::AiServer)?;

    if !response.ok() {
        log::info!("Server returned error: {}", response.status());
        panic!("Server returned error: {}", response.status());
    }
    let bytes = response
        .bytes()
        .await
        .map_err(LanguageDataError::AiServer)?;

    set_loading_state("Verifying language data");
    let language_data_hash = {
        let computed_hash = const_xxh3(&bytes);
        let expected_hash: u64 = language_data_hash.parse().unwrap();

        if computed_hash != expected_hash {
            log::warn!(
                "Language data hash mismatch! Expected: {expected_hash}, Got: {computed_hash}. Proceeding anyway..."
            );
        } else {
            log::info!("Language data hash verified.");
        }
        computed_hash
    };
    let mut language_data_file = language_directory_handle
        .get_file_handle_with_options(
            &format!("language_data_{language_data_hash}.rkyv"),
            &opfs::GetFileHandleOptions { create: true },
        )
        .await
        .map_err(LanguageDataError::Persistent)?;
    let mut writable = language_data_file
        .create_writable_with_options(&opfs::CreateWritableOptions {
            keep_existing_data: false,
        })
        .await
        .map_err(LanguageDataError::Persistent)?;
    writable
        .write_at_cursor_pos(bytes.clone())
        .await
        .map_err(LanguageDataError::Persistent)?;
    writable
        .close()
        .await
        .map_err(LanguageDataError::Persistent)?;

    set_loading_state("Cleaning up old language data files");
    // Clean up old language data files
    let files_to_delete = {
        let current_filename = format!("language_data_{language_data_hash}.rkyv");
        let mut entries = language_directory_handle
            .entries()
            .await
            .map_err(LanguageDataError::Persistent)?;
        let mut files_to_delete = Vec::new();

        // Collect filenames to delete first
        while let Some(Ok((filename, _))) = entries.next().await {
            if filename.starts_with("language_data_")
                && filename.ends_with(".hash")
                && filename != current_filename
            {
                files_to_delete.push(filename);
            }
        }

        files_to_delete
    };

    // Now delete the collected files
    for filename in files_to_delete {
        log::info!("Removing old language data file: {filename}");
        if let Err(e) = language_directory_handle.remove_entry(&filename).await {
            log::warn!("Failed to remove old language data file {filename}: {e:?}");
        }
    }

    log::info!("Language data successfully loaded and cached!");
    Ok(bytes)
}
