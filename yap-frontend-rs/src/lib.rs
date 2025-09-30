mod audio;
mod deck_selection;
mod directories;
mod language_pack;
mod next_cards;
mod notifications;
pub mod opfs_test;
pub mod simulation;
mod supabase;
mod utils;

pub use simulation::DailySimulationIterator;

use chrono::{DateTime, Utc};
use deck_selection::DeckSelectionEvent;
use futures::StreamExt;
use imdex_map::IndexMap;
use language_utils::ConsolidatedLanguageDataWithCapacity;
use language_utils::Literal;
use language_utils::TtsProvider;
use language_utils::TtsRequest;
use language_utils::autograde;
use language_utils::transcription_challenge;
use language_utils::{Course, Language};
use language_utils::{
    DictionaryEntry, Heteronym, Lexeme, PatternPosition, PhrasebookEntry, PronunciationGuide,
    TargetToNativeWord,
};
use lasso::Spur;
use opfs::persistent::{self};
use pav_regression::{IsotonicRegression, Point};
use rs_fsrs::FSRS;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;
use std::sync::LazyLock;
use wasm_bindgen::prelude::*;
use weapon::PartialAppState as _;
use weapon::data_model::Event;
use weapon::data_model::{EventStore, EventType, ListenerKey, Timestamped};

use crate::deck_selection::DeckSelection;
use crate::directories::Directories;
use crate::utils::hit_ai_server;
use next_cards::NextCardsIterator;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn get_available_courses() -> Vec<language_utils::Course> {
    language_utils::COURSES.to_vec()
}

#[wasm_bindgen]
pub struct Weapon {
    // todo: move these into a type in `weapon`
    // btw, we should never hold a borrow across an .await. by avoiding this, we guarantee the absence of "borrow while locked" panics
    store: RefCell<EventStore<String, String>>,
    user_id: Option<String>,
    device_id: String,

    // not this ofc
    language_pack: RefCell<BTreeMap<Course, Arc<LanguagePack>>>,
    directories: Directories,
}

// putting this inside LOGGER prevents us from accidentally initializing the logger more than once
#[allow(clippy::declare_interior_mutable_const)]
const LOGGER: LazyLock<()> = LazyLock::new(|| {
    utils::set_panic_hook();

    wasm_logger::init(wasm_logger::Config::default());
    log::info!("Logging initialized");
});

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Weapon {
    // Todo: I want to mostly move this into `weapon`. The one holdup is that wasm-bindgen types can't be generic, necessitating wrappers
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub async fn new(
        user_id: Option<String>,
        sync_stream: js_sys::Function,
    ) -> Result<Self, persistent::Error> {
        // used to only initialize the logger once
        #[allow(clippy::borrow_interior_mutable_const)]
        *LOGGER;

        let directories = directories::get_directories(&user_id)
            .await
            .inspect_err(|e| {
                log::error!("Error getting directories: {e:?}");
            })?;

        if user_id.is_some() {
            EventStore::<String, String>::import_logged_out_user_data(
                directories.weapon_directory_handle.clone(),
                directories.user_events_directory_handle.clone(),
                &directories.current_user_directory_handle,
            )
            .await
            .inspect_err(|e| {
                log::error!("Error importing logged out data: {e:?}");
            })?;
        }

        let device_id =
            utils::get_or_create_device_id(&directories.weapon_directory_handle, &user_id)
                .await
                .inspect_err(|e| {
                    log::error!("Error getting device ID: {e:?}");
                })?;

        // should move this into a separate function
        let mut events: EventStore<String, String> = EventStore::default();

        events.register_listener(move |listener_id, stream_id| {
            #[cfg(target_arch = "wasm32")]
            {
                let this = JsValue::null();
                let listener_js: JsValue = listener_id.into();
                let stream_js = JsValue::from_str(&stream_id);
                let _ = sync_stream.call2(&this, &listener_js, &stream_js);
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let _ = (listener_id, &sync_stream, stream_id);
            }
        });

        Ok(Self {
            store: RefCell::new(events),
            user_id,
            device_id,
            language_pack: RefCell::new(BTreeMap::new()),
            directories,
        })
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn subscribe_to_stream(
        &self,
        stream_id: String,
        callback: js_sys::Function,
    ) -> ListenerKey {
        // After sync, flush any pending notifications to JS listeners
        let _flusher = FlushLater::new(self);

        self.store
            .borrow_mut()
            .register_listener(move |_, event_stream_id| {
                if event_stream_id == stream_id {
                    let this = JsValue::null();
                    let _ = callback.call0(&this);
                }
            })
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn unsubscribe(&self, key: ListenerKey) {
        self.store.borrow_mut().unregister_listener(key)
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn request_reviews(&self) {
        let _flusher = FlushLater::new(self); // The addition of a new stream can trigger listeners, so we want to make sure to flush them after.
        self.store
            .borrow_mut()
            .get_or_insert_default::<EventType<DeckEvent>>("reviews".to_string(), None);
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn request_deck_selection(&self) {
        let _flusher = FlushLater::new(self); // The addition of a new stream can trigger listeners, so we want to make sure to flush them after.
        self.store
            .borrow_mut()
            .get_or_insert_default::<EventType<DeckSelectionEvent>>(
                "deck_selection".to_string(),
                None,
            );
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_stream_num_events(&self, stream_id: String) -> Option<usize> {
        let store = self.store.borrow();
        if !store.loaded_at_least_once(&stream_id) {
            return None;
        }
        store.get_raw(stream_id.clone()).map(|s| s.num_events())
    }

    pub fn get_deck_selection_state(&self) -> Option<DeckSelection> {
        let store = self.store.borrow();
        store
            .get::<EventType<DeckSelectionEvent>>("deck_selection".to_string())
            .map(|s| {
                s.state(DeckSelection {
                    target_language: None,
                    native_language: None,
                })
            })
    }

    pub async fn get_deck_state(&self, course: Course) -> Result<Deck, JsValue> {
        let language_pack = self
            .get_language_pack(course)
            .await
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))?;

        let target_language = course.target_language;
        let native_language = self
            .get_deck_selection_state()
            .and_then(|s| s.native_language)
            .unwrap_or(course.native_language);

        let initial_state = DeckState::new(language_pack, target_language, native_language);
        let store = self.store.borrow_mut();
        let Some(stream) = store.get::<EventType<DeckEvent>>("reviews".to_string()) else {
            return Ok(Deck::finalize(initial_state));
        };
        Ok(stream.state(initial_state))
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub async fn sync_with_supabase(
        &self,
        access_token: String,
        modifier: Option<ListenerKey>,
    ) -> Result<(), wasm_bindgen::JsValue> {
        if let Some(user_id) = &self.user_id {
            // After sync, flush any pending notifications to JS listeners
            let _flusher = FlushLater::new(self);

            EventStore::sync_with_supabase(
                &self.store,
                &access_token,
                supabase::supabase_config(),
                user_id,
                None,
                modifier,
            )
            .await?;
        }
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub async fn sync(
        &self,
        stream_id: String,
        access_token: Option<String>,
        attempt_supabase: bool,
        modifier: Option<ListenerKey>,
    ) -> Result<(), wasm_bindgen::JsValue> {
        // After sync, flush any pending notifications to JS listeners
        let _flusher = FlushLater::new(self);

        let is_initial_load = {
            let store = self.store.borrow();
            !store.loaded_at_least_once(&stream_id)
        };

        let start_time = if is_initial_load {
            web_sys::window()
                .and_then(|w| w.performance())
                .map(|p| p.now())
        } else {
            None
        };

        EventStore::load_from_local_storage(
            &self.store,
            &self.directories.current_user_directory_handle,
            stream_id.clone(),
            modifier,
        )
        .await?;

        if is_initial_load {
            if let (Some(start), Some(perf)) =
                (start_time, web_sys::window().and_then(|w| w.performance()))
            {
                log::info!(
                    "Initial load from disk for {stream_id} took {}ms",
                    perf.now() - start
                );
            }
        }

        {
            if self
                .store
                .borrow_mut()
                .mark_loaded(stream_id.clone(), modifier)
            {
                self.flush_notifications();
            }
        }

        EventStore::save_to_local_storage(
            &self.store,
            &self.directories.current_user_directory_handle,
            stream_id.clone(),
        )
        .await?;

        if attempt_supabase
            && let Some(access_token) = access_token
            && let Some(user_id) = &self.user_id
        {
            let supabase_sync_result = EventStore::sync_with_supabase(
                &self.store,
                &access_token,
                supabase::supabase_config(),
                user_id,
                Some(stream_id.clone()),
                modifier,
            )
            .await?;
            if supabase_sync_result.downloaded_from_supabase > 0 {
                EventStore::save_to_local_storage(
                    &self.store,
                    &self.directories.current_user_directory_handle,
                    stream_id,
                )
                .await?;
            }
        }

        Ok(())
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_timestamp_of_earliest_unsynced_event(
        &self,
        target: weapon::data_model::SyncTarget,
    ) -> Option<EarliestUnsyncedEvent> {
        self.store
            .borrow()
            .get_timestamp_of_earliest_unsynced_event(target)
            .map(|timestamp| EarliestUnsyncedEvent { timestamp })
    }

    #[cfg(target_arch = "wasm32")]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub async fn load_from_local_storage(
        &self,
        stream_id: String,
    ) -> Result<(), persistent::Error> {
        let _flusher = FlushLater::new(self);

        EventStore::load_from_local_storage(
            &self.store,
            &self.directories.current_user_directory_handle,
            stream_id.clone(),
            None,
        )
        .await?;

        self.store.borrow_mut().mark_loaded(stream_id, None);

        Ok(())
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_sync_state(
        &self,
        target: weapon::data_model::SyncTarget,
    ) -> weapon::data_model::SyncState<String, String> {
        self.store
            .borrow()
            .sync_state(target)
            .cloned()
            .unwrap_or_default()
    }

    /// Flush pending store/stream notifications safely, avoiding RefCell re-borrows during callbacks.
    fn flush_notifications(&self) {
        // do it like this to avoid holding the borrow while we call the callbacks
        let notifications = self.store.borrow_mut().drain_due_notifications();
        // that's important because many of these callbacks will call back into rust functions that themselves do borrow_mut()
        for notification in notifications {
            notification();
        }
    }

    // =======
    // non-obviously for JS consumption
    // =======

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn num_events(&self) -> usize {
        self.store
            .borrow()
            .vector_clock()
            .values()
            .map(|device_counts| device_counts.values().sum::<usize>())
            .sum()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn num_events_on_remote_as_of_last_sync(
        &self,
        target: weapon::data_model::SyncTarget,
    ) -> usize {
        self.store
            .borrow()
            .sync_state(target)
            .map(|state| {
                state
                    .remote_clock
                    .values()
                    .map(|device_counts| device_counts.values().sum::<usize>())
                    .sum()
            })
            .unwrap_or(0)
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn user_id(&self) -> Option<String> {
        self.user_id.clone()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn device_id(&self) -> String {
        self.device_id.clone()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn add_remote_event(
        &self,
        device_id: String,
        stream_id: String,
        event: String,
    ) -> Result<(), JsValue> {
        let event: serde_json::Value =
            serde_json::from_str(&event).map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
        let event =
            <Timestamped<EventType<DeckEvent>> as weapon::data_model::Event>::from_json(&event)
                .map_err(|e| JsValue::from_str(&format!("{e:?}")))?;

        self.store
            .borrow_mut()
            .add_device_event(stream_id, device_id, event, None);
        self.flush_notifications();
        Ok(())
    }

    // =======
    // less generic
    // =======-

    pub fn add_deck_event(&self, event: DeckEvent) {
        self.store.borrow_mut().add_raw_event(
            "reviews".to_string(),
            self.device_id.clone(),
            event,
            None,
        );
        self.flush_notifications();
    }

    pub fn add_deck_selection_event(&self, event: DeckSelectionEvent) {
        self.store.borrow_mut().add_raw_event(
            "deck_selection".to_string(),
            self.device_id.clone(),
            event,
            None,
        );
        self.flush_notifications();
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub async fn cache_language_pack(&self, course: Course) {
        let _ = self.get_language_pack(course).await;
    }
}

impl Weapon {
    pub(crate) async fn get_language_pack(
        &self,
        course: Course,
    ) -> Result<Arc<LanguagePack>, language_pack::LanguageDataError> {
        let language_pack = if let Some(language_pack) = self.language_pack.borrow().get(&course) {
            language_pack.clone()
        } else {
            let language_pack = language_pack::get_language_pack(
                &self.directories.data_directory_handle,
                course,
                &|_| {},
            )
            .await?;
            self.language_pack
                .borrow_mut()
                .insert(course, Arc::new(language_pack));

            self.language_pack
                .borrow()
                .get(&course)
                .expect("language pack must exist as we just added it")
                .clone()
        };
        Ok(language_pack)
    }
}

#[derive(Clone, Debug, tsify::Tsify, serde::Serialize, serde::Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct EarliestUnsyncedEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// A simple struct that flushes event listeners when dropped. THis is useful if you want to ensure you don't forget to flush listeners, regardless of the code path a function takes.
struct FlushLater<'a> {
    weapon: &'a Weapon,
}

impl<'a> FlushLater<'a> {
    fn new(weapon: &'a Weapon) -> Self {
        Self { weapon }
    }
}

impl<'a> Drop for FlushLater<'a> {
    fn drop(&mut self) {
        self.weapon.flush_notifications();
    }
}

#[derive(tsify::Tsify, serde::Serialize, serde::Deserialize, Debug, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct TranslateComprehensibleSentence<S> {
    pub audio: AudioRequest,
    pub target_language: S,
    pub target_language_literals: Vec<Literal<S>>,
    pub primary_expression: Lexeme<S>,
    pub unique_target_language_lexemes: Vec<Lexeme<S>>,
    pub unique_target_language_lexeme_definitions: Vec<(Lexeme<S>, Vec<TargetToNativeWord>)>,
    pub native_translations: Vec<S>,
}

impl TranslateComprehensibleSentence<Spur> {
    fn resolve(&self, rodeo: &lasso::RodeoReader) -> TranslateComprehensibleSentence<String> {
        TranslateComprehensibleSentence {
            audio: self.audio.clone(),
            target_language: rodeo.resolve(&self.target_language).to_string(),
            target_language_literals: self
                .target_language_literals
                .iter()
                .map(|l| l.resolve(rodeo))
                .collect(),
            primary_expression: self.primary_expression.resolve(rodeo),
            unique_target_language_lexemes: self
                .unique_target_language_lexemes
                .iter()
                .map(|l| l.resolve(rodeo))
                .collect(),
            unique_target_language_lexeme_definitions: self
                .unique_target_language_lexeme_definitions
                .iter()
                .map(|(l, d)| (l.resolve(rodeo), d.clone()))
                .collect(),
            native_translations: self
                .native_translations
                .iter()
                .map(|t| rodeo.resolve(t).to_string())
                .collect(),
        }
    }
}

#[derive(tsify::Tsify, serde::Serialize, serde::Deserialize, Debug, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct TranscribeComprehensibleSentence<S> {
    pub target_language: S,
    pub audio: AudioRequest,
    pub native_language: S,
    pub parts: Vec<transcription_challenge::Part>,
}

impl TranscribeComprehensibleSentence<Spur> {
    fn resolve(&self, rodeo: &lasso::RodeoReader) -> TranscribeComprehensibleSentence<String> {
        TranscribeComprehensibleSentence {
            target_language: rodeo.resolve(&self.target_language).to_string(),
            audio: self.audio.clone(),
            native_language: rodeo.resolve(&self.native_language).to_string(),
            parts: self.parts.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frequency {
    count: u32,
}

#[derive(Debug)]
pub struct LanguagePack {
    rodeo: lasso::RodeoReader,
    translations: HashMap<Spur, Vec<Spur>>,
    words_to_heteronyms: HashMap<Spur, BTreeSet<Heteronym<Spur>>>,
    sentences_containing_lexeme_index: HashMap<Lexeme<Spur>, Vec<Spur>>,
    sentences_to_literals: HashMap<Spur, Vec<Literal<Spur>>>,
    sentences_to_lexemes: HashMap<Spur, Vec<Lexeme<Spur>>>,
    sentences_to_all_lexemes: HashMap<Spur, Vec<Lexeme<Spur>>>,
    word_frequencies: IndexMap<Lexeme<Spur>, Frequency>,
    total_word_count: u64,
    dictionary: BTreeMap<Heteronym<Spur>, DictionaryEntry>,
    phrasebook: BTreeMap<Spur, PhrasebookEntry>,
    word_to_pronunciation: HashMap<Spur, Spur>,
    pronunciation_to_words: HashMap<Spur, Vec<Spur>>,
    pronunciation_data: language_utils::PronunciationData,
    pattern_frequency_map: HashMap<(Spur, language_utils::PatternPosition), u32>,
}

impl LanguagePack {
    /// Get all lexemes for words that share a pronunciation
    /// Returns an iterator over (word, lexeme) pairs
    pub fn pronunciation_to_lexemes(
        &self,
        pronunciation: &Spur,
    ) -> impl Iterator<Item = (Spur, Lexeme<Spur>)> + '_ {
        self.pronunciation_to_words
            .get(pronunciation)
            .into_iter()
            .flat_map(|words| words.iter())
            .flat_map(move |word| {
                self.words_to_heteronyms
                    .get(word)
                    .into_iter()
                    .flat_map(|heteronyms| heteronyms.iter())
                    .map(move |heteronym| (*word, Lexeme::Heteronym(*heteronym)))
            })
    }

    /// Get the maximum frequency for any word with this pronunciation
    pub fn pronunciation_max_frequency(&self, pronunciation: &Spur) -> Option<Frequency> {
        self.pronunciation_to_lexemes(pronunciation)
            .filter_map(|(_, lexeme)| self.word_frequencies.get(&lexeme).copied())
            .max()
    }

    pub fn new(language_data: ConsolidatedLanguageDataWithCapacity) -> Self {
        let rodeo = {
            let rodeo = language_data.intern();
            rodeo.into_reader()
        };

        let sentences: Vec<Spur> = {
            language_data
                .consolidated_language_data
                .target_language_sentences
                .iter()
                .map(|s| rodeo.get(s).unwrap())
                .collect()
        };

        let translations = {
            language_data
                .consolidated_language_data
                .translations
                .iter()
                .map(|(target_language, native_languages)| {
                    (
                        rodeo.get(target_language).unwrap(),
                        native_languages
                            .iter()
                            .map(|n| rodeo.get(n).unwrap())
                            .collect(),
                    )
                })
                .collect()
        };

        let words_to_heteronyms = {
            let mut map: HashMap<Spur, BTreeSet<Heteronym<Spur>>> = HashMap::new();

            for freq in &language_data.consolidated_language_data.frequencies {
                if let Lexeme::Heteronym(heteronym) = &freq.lexeme {
                    let word_spur = rodeo.get(&heteronym.word).unwrap();
                    map.entry(word_spur).or_default().insert({
                        Heteronym {
                            word: rodeo.get(&heteronym.word).unwrap(),
                            lemma: rodeo.get(&heteronym.lemma).unwrap(),
                            pos: heteronym.pos,
                        }
                    });
                }
            }

            map
        };

        let sentences_to_literals = {
            language_data
                .consolidated_language_data
                .nlp_sentences
                .iter()
                .map(|(sentence, analysis)| {
                    (
                        rodeo.get(sentence).unwrap(),
                        analysis
                            .words
                            .iter()
                            .map(|word| {
                                word.get_interned(&rodeo).unwrap_or_else(|| {
                                    panic!("word not in rodeo: {word:?} in sentence: {sentence:?}")
                                })
                            })
                            .collect(),
                    )
                })
                .collect()
        };

        let sentences_to_lexemes: HashMap<Spur, Vec<Lexeme<Spur>>> = {
            language_data
                .consolidated_language_data
                .nlp_sentences
                .iter()
                .map(|(sentence, analysis)| {
                    (
                        rodeo.get(sentence).unwrap(),
                        analysis
                            .lexemes()
                            .map(|l| l.get_interned(&rodeo).unwrap())
                            .collect(),
                    )
                })
                .collect()
        };

        let sentences_containing_lexeme_index = {
            let mut map = HashMap::new();
            for (i, sentence_spur) in sentences.iter().enumerate() {
                let _sentence = rodeo.resolve(sentence_spur);
                let Some(lexemes) = sentences_to_lexemes.get(sentence_spur) else {
                    continue;
                };
                for lexeme in lexemes.iter().cloned() {
                    map.entry(lexeme).or_insert(vec![]).push(sentences[i]);
                }
            }
            map
        };

        let sentences_to_all_lexemes = {
            language_data
                .consolidated_language_data
                .nlp_sentences
                .iter()
                .map(|(sentence, analysis)| {
                    (
                        rodeo.get(sentence).unwrap(),
                        analysis
                            .all_lexemes()
                            .map(|l| l.get_interned(&rodeo).unwrap())
                            .collect(),
                    )
                })
                .collect()
        };

        let word_frequencies = {
            let mut map = IndexMap::new();
            for freq in &language_data.consolidated_language_data.frequencies {
                map.insert(
                    freq.lexeme.get_interned(&rodeo).unwrap(),
                    Frequency { count: freq.count },
                );
            }
            map
        };

        let total_word_count = {
            language_data
                .consolidated_language_data
                .frequencies
                .iter()
                .map(|freq| freq.count as u64)
                .sum()
        };

        let dictionary = {
            language_data
                .consolidated_language_data
                .dictionary
                .iter()
                .map(|(heteronym, entry)| (heteronym.get_interned(&rodeo).unwrap(), entry.clone()))
                .collect()
        };

        let phrasebook = {
            language_data
                .consolidated_language_data
                .phrasebook
                .iter()
                .map(|(multiword_term, entry)| (rodeo.get(multiword_term).unwrap(), entry.clone()))
                .collect()
        };

        let word_to_pronunciation = {
            language_data
                .consolidated_language_data
                .word_to_pronunciation
                .iter()
                .map(|(word, pronunciation)| {
                    (
                        rodeo
                            .get(word)
                            .unwrap_or_else(|| panic!("word not in rodeo: {word:?}")),
                        rodeo.get(pronunciation).unwrap_or_else(|| {
                            panic!("pronunciation not in rodeo: {pronunciation:?}")
                        }),
                    )
                })
                .collect()
        };

        let pronunciation_to_words = {
            language_data
                .consolidated_language_data
                .pronunciation_to_words
                .iter()
                .map(|(pronunciation, words)| {
                    (
                        rodeo.get(pronunciation).unwrap(),
                        words.iter().map(|word| rodeo.get(word).unwrap()).collect(),
                    )
                })
                .collect()
        };

        let pronunciation_data = language_data
            .consolidated_language_data
            .pronunciation_data
            .clone();

        let pattern_frequency_map = {
            pronunciation_data
                .pattern_frequencies
                .iter()
                .map(|((pattern, position), freq)| {
                    ((rodeo.get(pattern).unwrap(), *position), *freq)
                })
                .collect()
        };

        Self {
            rodeo,
            translations,
            words_to_heteronyms,
            sentences_containing_lexeme_index,
            sentences_to_literals,
            sentences_to_lexemes,
            sentences_to_all_lexemes,
            word_frequencies,
            total_word_count,
            dictionary,
            phrasebook,
            word_to_pronunciation,
            pronunciation_to_words,
            pronunciation_data,
            pattern_frequency_map,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd, tsify::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum SentenceReviewResult {
    Perfect {
        #[serde(default)]
        lexemes_needed_hint: BTreeSet<Lexeme<String>>,
    },
    Wrong {
        submission: String,
        lexemes_remembered: BTreeSet<Lexeme<String>>,
        lexemes_forgotten: BTreeSet<Lexeme<String>>,
        #[serde(default)]
        lexemes_needed_hint: BTreeSet<Lexeme<String>>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd, tsify::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum CardType {
    TargetLanguage,
    Listening,
    LetterPronunciation,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd, tsify::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct AddCardOptions {
    pub smart_add: u32,
    pub manual_add: Vec<(u32, CardType)>,
}

#[derive(
    Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd, tsify::Tsify, Hash,
)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum CardIndicator<S> {
    TargetLanguage {
        lexeme: Lexeme<S>,
    },
    ListeningHomophonous {
        pronunciation: S,
    },
    ListeningLexeme {
        lexeme: Lexeme<S>,
    },
    LetterPronunciation {
        pattern: S,
        position: PatternPosition,
    },
}

impl<S> CardIndicator<S> {
    pub fn target_language(&self) -> Option<&Lexeme<S>> {
        match self {
            CardIndicator::TargetLanguage { lexeme } => Some(lexeme),
            _ => None,
        }
    }

    pub fn listening_homophonous(&self) -> Option<&S> {
        match self {
            CardIndicator::ListeningHomophonous { pronunciation } => Some(pronunciation),
            _ => None,
        }
    }

    pub fn listening_lexeme(&self) -> Option<&Lexeme<S>> {
        match self {
            CardIndicator::ListeningLexeme { lexeme } => Some(lexeme),
            _ => None,
        }
    }

    pub fn letter_pronunciation(&self) -> Option<&S> {
        match self {
            CardIndicator::LetterPronunciation { pattern, .. } => Some(pattern),
            _ => None,
        }
    }
}

impl CardIndicator<String> {
    pub fn get_interned(&self, rodeo: &lasso::RodeoReader) -> Option<CardIndicator<Spur>> {
        Some(match self {
            CardIndicator::TargetLanguage { lexeme } => CardIndicator::TargetLanguage {
                lexeme: lexeme.get_interned(rodeo)?,
            },
            CardIndicator::ListeningHomophonous { pronunciation } => {
                CardIndicator::ListeningHomophonous {
                    pronunciation: rodeo.get(pronunciation)?,
                }
            }
            CardIndicator::ListeningLexeme { lexeme } => CardIndicator::ListeningLexeme {
                lexeme: lexeme.get_interned(rodeo)?,
            },
            CardIndicator::LetterPronunciation { pattern, position } => {
                CardIndicator::LetterPronunciation {
                    pattern: rodeo.get(pattern)?,
                    position: *position,
                }
            }
        })
    }
}

impl CardIndicator<Spur> {
    pub fn resolve(&self, rodeo: &lasso::RodeoReader) -> CardIndicator<String> {
        match self {
            CardIndicator::TargetLanguage { lexeme } => CardIndicator::TargetLanguage {
                lexeme: lexeme.resolve(rodeo),
            },
            CardIndicator::ListeningHomophonous { pronunciation } => {
                CardIndicator::ListeningHomophonous {
                    pronunciation: rodeo.resolve(pronunciation).to_string(),
                }
            }
            CardIndicator::ListeningLexeme { lexeme } => CardIndicator::ListeningLexeme {
                lexeme: lexeme.resolve(rodeo),
            },
            CardIndicator::LetterPronunciation { pattern, position } => {
                CardIndicator::LetterPronunciation {
                    pattern: rodeo.resolve(pattern).to_string(),
                    position: *position,
                }
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd, tsify::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum SentenceReviewIndicator {
    TargetToNative {
        challenge_sentence: String,
        result: SentenceReviewResult,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd, tsify::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct LanguageEvent {
    #[serde(alias = "language")]
    pub target_language: Language,
    #[serde(default = "default_native_language")]
    pub native_language: Language,
    pub content: LanguageEventContent,
}

fn default_native_language() -> Language {
    Language::English
}

#[derive(
    Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd, tsify::Tsify,
)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "lowercase")]
pub enum Rating {
    Again,
    Remembered, // generic rating for when the user picked "remembered" without choosing a specific rating

    Hard,
    Good,
    Easy,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd, tsify::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum LanguageEventContent {
    AddCards {
        cards: Vec<CardIndicator<String>>,
    },
    ReviewCard {
        reviewed: CardIndicator<String>,
        rating: Rating,
    },
    #[serde(rename = "ReviewSentence")]
    TranslationChallenge {
        review: SentenceReviewIndicator,
    },
    TranscriptionChallenge {
        challenge: Vec<transcription_challenge::PartGraded>,
    },
}

// Event types
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd, tsify::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum DeckEvent {
    Language(LanguageEvent),
}
#[derive(Clone, Debug, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, tsify::Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "version")]
pub enum VersionedDeckEvent {
    V1(DeckEvent),
}

impl Event for DeckEvent {
    fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        let versioned = VersionedDeckEvent::from(self.clone());
        serde_json::to_value(versioned)
    }

    fn from_json(json: &serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<VersionedDeckEvent>(json.clone()).map(|versioned| versioned.into())
    }
}
impl From<DeckEvent> for VersionedDeckEvent {
    fn from(event: DeckEvent) -> Self {
        VersionedDeckEvent::V1(event)
    }
}
impl From<VersionedDeckEvent> for DeckEvent {
    fn from(event: VersionedDeckEvent) -> Self {
        match event {
            VersionedDeckEvent::V1(event) => event,
        }
    }
}

#[derive(Clone, Debug)]
enum CardStatus {
    Tracked(CardData),
    Unadded(Unadded),
}

impl CardStatus {
    pub(crate) fn reviewed(&self) -> Option<&CardData> {
        match self {
            CardStatus::Tracked(card_data) => Some(card_data),
            CardStatus::Unadded(_) => None,
        }
    }

    pub(crate) fn unadded(&self) -> Option<&Unadded> {
        match self {
            CardStatus::Unadded(unadded) => Some(unadded),
            CardStatus::Tracked(_) => None,
        }
    }
}

#[derive(Clone, Debug)]
struct Unadded {}

#[derive(Clone, Debug)]
enum CardData {
    /// Card that has been formally added to the deck
    Added { fsrs_card: rs_fsrs::Card },
    /// Ghost card - not formally added but has been reviewed through comprehensible sentences
    Ghost { fsrs_card: rs_fsrs::Card },
}

impl CardData {
    /// Returns positive surprise if there are no lapses, or negative surprise otherwise
    pub fn pre_existing_knowledge(&self) -> f64 {
        match self {
            CardData::Added { fsrs_card } | CardData::Ghost { fsrs_card } => {
                if fsrs_card.lapses == 0 {
                    fsrs_card.accumulated_positive_surprise
                } else {
                    -fsrs_card.accumulated_negative_surprise
                }
            }
        }
    }

    pub fn due_timestamp_ms(&self) -> f64 {
        match self {
            CardData::Added { fsrs_card } | CardData::Ghost { fsrs_card } => {
                fsrs_card.due.timestamp_millis() as f64
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct DailyStreak {
    streak_start: chrono::DateTime<chrono::Utc>,
    last_review_time: chrono::DateTime<chrono::Utc>,
}

/// Context contains the language-specific configuration
#[derive(Clone, Debug)]
pub struct Context {
    pub language_pack: Arc<LanguagePack>,
    pub target_language: Language,
    pub native_language: Language,
}

/// Stats contains review statistics and progress tracking
#[derive(Clone, Debug)]
pub struct Stats {
    pub sentences_reviewed: BTreeMap<Spur, u32>,
    pub words_listened_to: BTreeMap<Heteronym<Spur>, u32>,
    pub total_reviews: u64,
    pub xp: f64,
    pub daily_streak: Option<DailyStreak>,
    /// Track daily challenge completions for the past week
    /// Key is days since epoch, value is number of challenges completed
    pub past_week_challenges: BTreeMap<i64, u32>,
}

#[derive(Clone, Debug)]
pub struct DeckState {
    cards: HashMap<CardIndicator<Spur>, CardData>,
    fsrs: FSRS,
    stats: Stats,
    context: Context,
}

#[derive(Clone, Debug)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Deck {
    cards: HashMap<CardIndicator<Spur>, CardStatus>,
    fsrs: FSRS,
    stats: Stats,
    context: Context,
    regressions: Regressions,
}

#[derive(Clone, Debug)]
pub(crate) struct Regressions {
    target_language_regression: Option<IsotonicRegression<f64>>,
    listening_regression: Option<IsotonicRegression<f64>>,
}

struct ComprehensibleSentence {
    target_language: Spur,
    target_language_literals: Vec<Literal<Spur>>,
    unique_target_language_lexemes: Vec<Lexeme<Spur>>,
    native_languages: Vec<Spur>,
}

impl From<Deck> for DeckState {
    fn from(deck: Deck) -> Self {
        // Convert cards from CardStatus to CardData, only keeping Added cards
        let cards = deck
            .cards
            .iter()
            .filter_map(|(indicator, status)| match status {
                CardStatus::Tracked(data) => Some((*indicator, data.clone())),
                CardStatus::Unadded { .. } => None,
            })
            .collect();

        DeckState {
            cards,
            fsrs: deck.fsrs,
            stats: deck.stats,
            context: deck.context,
        }
    }
}

impl weapon::PartialAppState for Deck {
    type Event = DeckEvent;
    type Partial = DeckState;

    fn process_event(mut deck: Self::Partial, event: &Timestamped<Self::Event>) -> Self::Partial {
        let Timestamped::<DeckEvent> {
            event,
            timestamp,
            within_device_events_index: _,
        } = event;

        let DeckEvent::Language(LanguageEvent {
            target_language: event_language,
            native_language: _, // TODO: specify native_language
            content: event,
        }) = event;

        deck.update_daily_streak(timestamp);
        deck.stats.total_reviews += 1;

        if *event_language != deck.context.target_language {
            return deck;
        }

        // Track challenge completions for workload statistics
        match event {
            LanguageEventContent::TranslationChallenge { .. }
            | LanguageEventContent::TranscriptionChallenge { .. } => {
                let days_since_epoch = timestamp.timestamp() / 86400;
                *deck
                    .stats
                    .past_week_challenges
                    .entry(days_since_epoch)
                    .or_insert(0) += 1;

                // Clean up old entries (keep only last 7 days)
                let seven_days_ago = days_since_epoch - 7;
                deck.stats
                    .past_week_challenges
                    .retain(|&day, _| day > seven_days_ago);
            }
            _ => {}
        }

        match event {
            LanguageEventContent::AddCards { cards } => {
                for card in cards {
                    if let Some(card) = card.get_interned(&deck.context.language_pack.rodeo) {
                        // Make sure the card is valid and can be added
                        if !deck.context.is_card_valid(&card) {
                            continue;
                        }
                        // Add the card to the deck if it's not already in it, or transition ghost to added
                        deck.cards
                            .entry(card)
                            .and_modify(|existing| {
                                // If it's a ghost card, transition it to added
                                if let CardData::Ghost { fsrs_card } = existing {
                                    let mut new_fsrs_card = fsrs_card.clone();
                                    // Reset the due date to now when formally adding
                                    new_fsrs_card.due = *timestamp;
                                    *existing = CardData::Added {
                                        fsrs_card: new_fsrs_card,
                                    };
                                }
                            })
                            .or_insert_with(|| {
                                let mut fsrs_card = rs_fsrs::Card::new(*timestamp);
                                fsrs_card.due = *timestamp;
                                CardData::Added { fsrs_card }
                            });
                    }
                }
            }
            LanguageEventContent::ReviewCard { reviewed, rating } => {
                if let Some(reviewed) = reviewed.get_interned(&deck.context.language_pack.rodeo) {
                    deck.log_review(reviewed, *rating, *timestamp);
                }
            }
            LanguageEventContent::TranslationChallenge {
                review:
                    SentenceReviewIndicator::TargetToNative {
                        challenge_sentence,
                        result:
                            SentenceReviewResult::Perfect {
                                lexemes_needed_hint,
                            },
                    },
            } => {
                if let Some(challenge_sentence) =
                    deck.context.language_pack.rodeo.get(challenge_sentence)
                {
                    if let Some(lexemes) = deck
                        .context
                        .language_pack
                        .sentences_to_lexemes
                        .get(&challenge_sentence)
                    {
                        let sentence_review_count = deck
                            .stats
                            .sentences_reviewed
                            .entry(challenge_sentence)
                            .or_insert(0);
                        *sentence_review_count += 1;

                        let lexemes = lexemes.clone().into_iter().collect::<BTreeSet<_>>();
                        let lexemes_needed_hint = lexemes_needed_hint
                            .clone()
                            .into_iter()
                            .flat_map(|lexeme| {
                                lexeme.get_interned(&deck.context.language_pack.rodeo)
                            })
                            .collect::<BTreeSet<_>>();
                        for lexeme in lexemes.difference(&lexemes_needed_hint) {
                            deck.log_review(
                                CardIndicator::TargetLanguage { lexeme: *lexeme },
                                Rating::Remembered,
                                *timestamp,
                            );
                        }
                        for lexeme in lexemes_needed_hint {
                            deck.log_review(
                                CardIndicator::TargetLanguage { lexeme },
                                Rating::Again,
                                *timestamp,
                            );
                        }
                    }
                }
            }
            LanguageEventContent::TranslationChallenge {
                review:
                    SentenceReviewIndicator::TargetToNative {
                        challenge_sentence: _,
                        result:
                            SentenceReviewResult::Wrong {
                                submission: _,
                                lexemes_remembered,
                                lexemes_forgotten,
                                lexemes_needed_hint,
                            },
                    },
            } => {
                for lexeme in lexemes_remembered.difference(lexemes_needed_hint) {
                    if let Some(lexeme) = lexeme.get_interned(&deck.context.language_pack.rodeo) {
                        deck.log_review(
                            CardIndicator::TargetLanguage { lexeme },
                            Rating::Remembered,
                            *timestamp,
                        );
                    }
                }

                for lexeme in lexemes_forgotten.union(lexemes_needed_hint) {
                    if let Some(lexeme) = lexeme.get_interned(&deck.context.language_pack.rodeo) {
                        deck.log_review(
                            CardIndicator::TargetLanguage { lexeme },
                            Rating::Again,
                            *timestamp,
                        );
                    }
                }
            }
            LanguageEventContent::TranscriptionChallenge { challenge } => {
                let mut perfect = true;

                // Check if this is a full sentence transcription
                // (no Provided parts with heteronyms - only punctuation is provided)
                let is_full_sentence_transcription = !challenge.iter().any(|part| {
                    matches!(part, transcription_challenge::PartGraded::Provided { part } if part.heteronym.is_some())
                });

                // First pass: collect worst grade for each heteronym (word with its specific meaning)
                // Using HashMap to track worst grade per heteronym
                let mut worst_grades: std::collections::HashMap<
                    Heteronym<lasso::Spur>,
                    transcription_challenge::WordGrade,
                > = std::collections::HashMap::new();

                for part in challenge {
                    if let transcription_challenge::PartGraded::AskedToTranscribe {
                        parts, ..
                    } = part
                    {
                        for graded_part in parts {
                            if let Some(heteronym) = &graded_part.heard.heteronym
                                && let Some(heteronym) =
                                    heteronym.get_interned(&deck.context.language_pack.rodeo)
                            {
                                // Update with worse grade (remember: worse grade > better grade in Ord)
                                worst_grades
                                    .entry(heteronym)
                                    .and_modify(|existing_grade| {
                                        if graded_part.grade > *existing_grade {
                                            *existing_grade = graded_part.grade.clone();
                                        }
                                    })
                                    .or_insert_with(|| graded_part.grade.clone());
                            }
                        }
                    }
                }

                // Process each heteronym with its worst grade
                for (heteronym, grade) in worst_grades {
                    if let Some(&pronunciation) = deck
                        .context
                        .language_pack
                        .word_to_pronunciation
                        .get(&heteronym.word)
                    {
                        let listening_homophonous_card =
                            CardIndicator::ListeningHomophonous { pronunciation };
                        let listening_lexeme_card = CardIndicator::ListeningLexeme {
                            lexeme: Lexeme::Heteronym(heteronym),
                        };

                        // Map the grade to a FSRS rating
                        let rating = match grade {
                            transcription_challenge::WordGrade::Perfect {} => Rating::Remembered,
                            transcription_challenge::WordGrade::CorrectWithTypo {} => {
                                Rating::Remembered
                            }
                            transcription_challenge::WordGrade::PhoneticallyIdenticalButContextuallyIncorrect {} => {
                                Rating::Hard
                            }
                            transcription_challenge::WordGrade::PhoneticallySimilarButContextuallyIncorrect {} => {
                                Rating::Again
                            }
                            transcription_challenge::WordGrade::Incorrect {} => Rating::Again,
                            transcription_challenge::WordGrade::Missed {} => Rating::Again,
                        };

                        if rating != Rating::Again {
                            *deck.stats.words_listened_to.entry(heteronym).or_insert(0) += 1;
                        } else {
                            perfect = false;
                        }

                        // Always log review for ListeningHomophonous card
                        deck.log_review(listening_homophonous_card, rating, *timestamp);

                        if rating == Rating::Remembered {
                            if let std::collections::hash_map::Entry::Vacant(e) =
                                deck.cards.entry(listening_lexeme_card)
                            {
                                // Add the card as a new card
                                let mut fsrs_card = rs_fsrs::Card::new(*timestamp);
                                fsrs_card.due = *timestamp;
                                e.insert(CardData::Added { fsrs_card });
                            }
                        }

                        // For full sentence transcriptions with successful transcription,
                        // add or review the ListeningLexeme card
                        if is_full_sentence_transcription {
                            // Log a review for the existing card
                            deck.log_review(listening_lexeme_card, rating, *timestamp);
                        }
                    }
                }

                if perfect {
                    let challenge_sentence = challenge
                        .iter()
                        .flat_map(|part| match part {
                            transcription_challenge::PartGraded::AskedToTranscribe {
                                parts,
                                ..
                            } => parts
                                .iter()
                                .flat_map(|part| {
                                    vec![part.heard.text.clone(), part.heard.whitespace.clone()]
                                })
                                .collect::<Vec<_>>(),
                            transcription_challenge::PartGraded::Provided { part } => {
                                vec![part.text.clone(), part.whitespace.clone()]
                            }
                        })
                        .collect::<Vec<String>>()
                        .join("");
                    if let Some(challenge_sentence) =
                        deck.context.language_pack.rodeo.get(&challenge_sentence)
                    {
                        let sentence_review_count = deck
                            .stats
                            .sentences_reviewed
                            .entry(challenge_sentence)
                            .or_insert(0);
                        *sentence_review_count += 1;
                    }
                }
            }
        }

        deck
    }

    fn finalize(state: Self::Partial) -> Self {
        // Collect data points for isotonic regression
        let mut target_language_points = Vec::new();
        let mut listening_points = Vec::new();

        for (card_indicator, card_data) in state.cards.iter() {
            // Only use cards that have been reviewed (not new)
            // For regression, only use Added cards that aren't new
            match card_data {
                CardData::Added { fsrs_card } | CardData::Ghost { fsrs_card }
                    if fsrs_card.state == rs_fsrs::State::New =>
                {
                    continue;
                }
                _ => {}
            }

            if let Some(frequency) = state.context.get_card_frequency(card_indicator) {
                let pre_existing_knowledge = card_data.pre_existing_knowledge();
                let point = Point::new(frequency.sqrt_frequency(), pre_existing_knowledge);

                match card_indicator {
                    CardIndicator::TargetLanguage { .. } => {
                        target_language_points.push(point);
                    }
                    CardIndicator::ListeningHomophonous { .. }
                    | CardIndicator::ListeningLexeme { .. } => {
                        listening_points.push(point);
                    }
                    CardIndicator::LetterPronunciation { .. } => {}
                }
            }
        }

        // Add bias points at (0, -10) and (10, -10) to ensure the curve slopes down
        // This represents a word with 0 occurrences being very difficult. We'll give them a weight of 10 to ensure it's not ignored
        let bias_points = [
            Point::new_with_weight(Frequency { count: 1 }.sqrt_frequency(), -10.0, 5.0),
            Point::new_with_weight(Frequency { count: 25 }.sqrt_frequency(), 0.0, 5.0),
            Point::new_with_weight(Frequency { count: 64 }.sqrt_frequency(), 0.0, 1.0),
            Point::new_with_weight(Frequency { count: 400 }.sqrt_frequency(), 0.0, 1.0),
            Point::new_with_weight(Frequency { count: 1000 }.sqrt_frequency(), 0.0, 0.5),
            Point::new_with_weight(Frequency { count: 4000 }.sqrt_frequency(), 0.0, 0.5),
        ];

        // Create isotonic regressions (need at least 2 non-new cards)
        let target_language_regression = if target_language_points.len() >= 2 {
            target_language_points.extend_from_slice(&bias_points);
            IsotonicRegression::new_ascending(&target_language_points)
                .inspect_err(|e| log::error!("regression error: {e:?}"))
                .ok()
        } else {
            None
        };

        let listening_regression = if listening_points.len() >= 2 {
            listening_points.extend_from_slice(&bias_points);
            IsotonicRegression::new_ascending(&listening_points)
                .inspect_err(|e| log::error!("regression error: {e:?}"))
                .ok()
        } else {
            None
        };

        let regressions = Regressions {
            target_language_regression,
            listening_regression,
        };

        // Convert existing cards to CardStatus and calculate probabilities for unadded cards
        let added_cards: HashMap<CardIndicator<Spur>, CardData> = state.cards;

        // Create all cards as Unadded first, then update with Added status
        let mut all_cards: HashMap<CardIndicator<Spur>, CardStatus> = state
            .context
            .language_pack
            .word_frequencies
            .keys()
            .map(|lexeme| {
                (
                    CardIndicator::TargetLanguage { lexeme: *lexeme },
                    CardStatus::Unadded(Unadded {}),
                )
            })
            .chain(
                state
                    .context
                    .language_pack
                    .pronunciation_to_words
                    .keys()
                    .map(|pronunciation| {
                        (
                            CardIndicator::ListeningHomophonous {
                                pronunciation: *pronunciation,
                            },
                            CardStatus::Unadded(Unadded {}),
                        )
                    }),
            )
            .chain(
                // Add ListeningLexeme cards for all words
                state
                    .context
                    .language_pack
                    .word_frequencies
                    .keys()
                    .map(|lexeme| {
                        (
                            CardIndicator::ListeningLexeme { lexeme: *lexeme },
                            CardStatus::Unadded(Unadded {}),
                        )
                    }),
            )
            .chain(
                // Add pronunciation pattern cards
                state
                    .context
                    .language_pack
                    .pronunciation_data
                    .guides
                    .iter()
                    .filter_map(|guide| {
                        // Only create cards for patterns that exist in the rodeo
                        state
                            .context
                            .language_pack
                            .rodeo
                            .get(&guide.pattern)
                            .map(|pattern| {
                                (
                                    CardIndicator::LetterPronunciation {
                                        pattern,
                                        position: guide.position,
                                    },
                                    CardStatus::Unadded(Unadded {}),
                                )
                            })
                    }),
            )
            .collect();

        // Update only the cards that have been added
        for (indicator, card_data) in added_cards {
            if let Some(status) = all_cards.get_mut(&indicator) {
                *status = CardStatus::Tracked(card_data);
            } else {
                log::error!("Card {indicator:?} not found in all_cards");
            }
        }

        Deck {
            cards: all_cards,
            fsrs: state.fsrs,
            stats: state.stats,
            context: state.context,
            regressions,
        }
    }
}

impl DeckState {
    /// Create a new DeckState with the given language pack and target language
    pub fn new(
        language_pack: Arc<LanguagePack>,
        target_language: Language,
        native_language: Language,
    ) -> Self {
        Self {
            cards: HashMap::new(),
            fsrs: FSRS::new(rs_fsrs::Parameters {
                request_retention: 0.7,
                ..Default::default()
            }),
            stats: Stats {
                sentences_reviewed: BTreeMap::new(),
                words_listened_to: BTreeMap::new(),
                total_reviews: 0,
                xp: 0.0,
                daily_streak: None,
                past_week_challenges: BTreeMap::new(),
            },
            context: Context {
                language_pack,
                target_language,
                native_language,
            },
        }
    }

    fn log_review(&mut self, card: CardIndicator<Spur>, rating: Rating, timestamp: DateTime<Utc>) {
        // Make sure the card is valid before logging a review
        if !self.context.is_card_valid(&card) {
            return;
        }

        let card_data = self.cards.entry(card).or_insert_with(|| {
            // Create a ghost card if it doesn't exist
            let mut fsrs_card = rs_fsrs::Card::new(timestamp);
            fsrs_card.due = timestamp;
            CardData::Ghost { fsrs_card }
        });

        // Update the card data
        let fsrs_card = match card_data {
            CardData::Added { fsrs_card } | CardData::Ghost { fsrs_card } => fsrs_card,
        };
        let fsrs_rating = match rating {
            Rating::Again => rs_fsrs::Rating::Again,
            Rating::Remembered => {
                // for new cards, we use Easy. Otherwise, we use Good
                if fsrs_card.state == rs_fsrs::State::New {
                    rs_fsrs::Rating::Easy
                } else {
                    rs_fsrs::Rating::Good
                }
            }
            Rating::Hard => rs_fsrs::Rating::Hard,
            Rating::Good => rs_fsrs::Rating::Good,
            Rating::Easy => rs_fsrs::Rating::Easy,
        };

        let old_stability = fsrs_card.stability;
        *fsrs_card = self
            .fsrs
            .next(fsrs_card.clone(), timestamp, fsrs_rating)
            .card;
        let new_stability = fsrs_card.stability;
        self.stats.xp += (new_stability - old_stability).max(0.0) / 10.0;
    }

    fn update_daily_streak(&mut self, timestamp: &DateTime<Utc>) {
        match &self.stats.daily_streak {
            None => {
                // First review ever
                self.stats.daily_streak = Some(DailyStreak {
                    streak_start: *timestamp,
                    last_review_time: *timestamp,
                });
            }
            Some(streak) => {
                if timestamp > &streak.last_review_time {
                    // This is a newer review
                    let hours_since_last = (*timestamp - streak.last_review_time).num_hours();

                    if hours_since_last <= 30 {
                        // Within 30 hours, continue streak
                        self.stats.daily_streak = Some(DailyStreak {
                            streak_start: streak.streak_start,
                            last_review_time: *timestamp,
                        });
                    } else {
                        // More than 30 hours, start new streak
                        self.stats.daily_streak = Some(DailyStreak {
                            streak_start: *timestamp,
                            last_review_time: *timestamp,
                        });
                    }
                }
                // If timestamp <= last_review_time, it's an old event being processed, ignore
            }
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Deck {
    /// First, the frontend calls get_all_cards_summary to get a view of what cards are due and what cards are going to be due in the future.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_all_cards_summary(&self) -> Vec<CardSummary> {
        let language_pack: &LanguagePack = &self.context.language_pack;
        let rodeo = &language_pack.rodeo;
        let mut summaries: Vec<CardSummary> = self
            .cards
            .iter()
            .filter_map(|(card_indicator, card_status)| {
                if let CardStatus::Tracked(CardData::Added { fsrs_card }) = card_status {
                    let state = match fsrs_card.state {
                        rs_fsrs::State::New => "new".to_string(),
                        rs_fsrs::State::Learning => "learning".to_string(),
                        rs_fsrs::State::Review => "review".to_string(),
                        rs_fsrs::State::Relearning => "relearning".to_string(),
                    };
                    Some(CardSummary {
                        card_indicator: card_indicator.resolve(rodeo),
                        due_timestamp_ms: fsrs_card.due.timestamp_millis() as f64,
                        state,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Sort by due date
        summaries.sort_by(|a, b| a.due_timestamp_ms.partial_cmp(&b.due_timestamp_ms).unwrap());

        summaries
    }

    /// TODO: get_review_info and get_all_cards_summary can probably be combined.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_review_info(
        &self,
        banned_challenge_types: Vec<ChallengeType>,
        timestamp_ms: f64,
    ) -> ReviewInfo {
        let now =
            DateTime::<Utc>::from_timestamp_millis(timestamp_ms as i64).unwrap_or_else(Utc::now);
        let mut due_cards = vec![];
        let mut future_cards = vec![];
        let mut due_but_banned_cards = vec![];

        let no_listening_cards = banned_challenge_types.contains(&ChallengeType::Listening);
        let no_text_cards = banned_challenge_types.contains(&ChallengeType::Text);
        let no_speaking_cards = banned_challenge_types.contains(&ChallengeType::Speaking);

        for (card, card_status) in self.cards.iter() {
            if let CardStatus::Tracked(CardData::Added { fsrs_card }) = card_status {
                let due_date = fsrs_card.due;

                if due_date <= now {
                    match card {
                        CardIndicator::TargetLanguage { .. } if no_text_cards => {
                            due_but_banned_cards.push(*card);
                        }
                        CardIndicator::ListeningHomophonous { .. }
                        | CardIndicator::ListeningLexeme { .. }
                            if no_listening_cards =>
                        {
                            due_but_banned_cards.push(*card);
                        }
                        CardIndicator::LetterPronunciation { .. } if no_speaking_cards => {
                            due_but_banned_cards.push(*card);
                        }
                        CardIndicator::TargetLanguage { .. }
                        | CardIndicator::ListeningHomophonous { .. }
                        | CardIndicator::ListeningLexeme { .. }
                        | CardIndicator::LetterPronunciation { .. } => due_cards.push(*card),
                    }
                } else {
                    future_cards.push(*card);
                }
            }
        }

        // sort by due date, then by card indicator for deterministic ordering
        due_cards.sort_by_key(|card_indicator| {
            let card_status = self.cards.get(card_indicator).unwrap();
            let due_timestamp = if let CardStatus::Tracked(card_data) = card_status {
                ordered_float::NotNan::new(card_data.due_timestamp_ms()).unwrap()
            } else {
                ordered_float::NotNan::new(0.0).unwrap()
            };
            (due_timestamp, *card_indicator)
        });

        due_but_banned_cards.sort_by_key(|card_indicator| {
            let card_status = self.cards.get(card_indicator).unwrap();
            let due_timestamp = if let CardStatus::Tracked(card_data) = card_status {
                ordered_float::NotNan::new(card_data.due_timestamp_ms()).unwrap()
            } else {
                ordered_float::NotNan::new(0.0).unwrap()
            };
            (due_timestamp, *card_indicator)
        });

        future_cards.sort_by_key(|card_indicator| {
            let card_status = self.cards.get(card_indicator).unwrap();
            let due_timestamp = if let CardStatus::Tracked(card_data) = card_status {
                ordered_float::NotNan::new(card_data.due_timestamp_ms()).unwrap()
            } else {
                ordered_float::NotNan::new(0.0).unwrap()
            };
            (due_timestamp, *card_indicator)
        });

        ReviewInfo {
            due_cards,
            due_but_banned_cards,
            future_cards,
        }
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub async fn cache_challenge_audio(
        &self,
        access_token: Option<String>,
        abort_signal: Option<web_sys::AbortSignal>,
    ) {
        let mut audio_cache = match audio::AudioCache::new().await {
            Ok(cache) => cache,
            Err(e) => {
                log::error!("Failed to create audio cache: {e:?}");
                return;
            }
        };
        let access_token = access_token.as_ref();

        const SIMULATION_DAYS: u32 = 2;
        let mut requested_filenames = BTreeSet::new();
        let mut simulation_iterator = self.simulate_usage(chrono::Utc::now());
        for _ in 0..SIMULATION_DAYS {
            // Sleep for 1 second using JavaScript's setTimeout via JsFuture
            let promise = js_sys::Promise::new(&mut |resolve, _| {
                web_sys::window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 1000)
                    .unwrap();
            });
            wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

            // Check if aborted before progressing
            if let Some(ref signal) = abort_signal {
                if signal.aborted() {
                    return;
                }
            }

            let challenges;
            (simulation_iterator, challenges) = simulation_iterator.next();

            // get the audio files
            requested_filenames.extend(
                futures::stream::iter(challenges)
                    .map(|challenge| {
                        let request = challenge.audio_request();
                        let audio_cache = audio_cache.clone();
                        let abort_signal = abort_signal.clone();
                        async move {
                            // Check if aborted before processing
                            if let Some(ref signal) = abort_signal {
                                if signal.aborted() {
                                    return None;
                                }
                            }

                            // Generate the cache filename for this request
                            let cache_filename = audio::AudioCache::get_cache_filename(
                                &request.request,
                                &request.provider,
                            );

                            // Just try to fetch and cache, ignoring errors for individual requests
                            let _ = audio_cache.fetch_and_cache(&request, access_token).await;
                            Some(cache_filename)
                        }
                    })
                    .buffered(3)
                    .filter_map(|x| async { x })
                    .collect::<BTreeSet<_>>()
                    .await,
            );
            // sleep for 1 second
        }

        // Check if aborted before cleanup
        if let Some(ref signal) = abort_signal {
            if signal.aborted() {
                return;
            }
        }

        // Clean up any files that weren't in the requested set
        if let Err(e) = audio_cache.cleanup_except(requested_filenames).await {
            log::error!("Failed to clean up audio cache: {e:?}");
        }
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_percent_of_words_known(&self) -> f64 {
        let total_words_reviewed: u64 = self
            .cards
            .iter()
            .filter_map(|(card_indicator, card_status)| match card_indicator {
                CardIndicator::TargetLanguage { lexeme } => Some((lexeme, card_status)),
                CardIndicator::ListeningHomophonous { .. } => None,
                CardIndicator::ListeningLexeme { .. } => None,
                CardIndicator::LetterPronunciation { .. } => None,
            })
            .filter_map(|(lexeme, card_status)| {
                if let CardStatus::Tracked(card_data) = card_status {
                    let is_reviewed = match card_data {
                        CardData::Added { fsrs_card } => fsrs_card.state != rs_fsrs::State::New,
                        CardData::Ghost { fsrs_card } => fsrs_card.state != rs_fsrs::State::New,
                    };
                    if is_reviewed {
                        self.context.language_pack.word_frequencies.get(lexeme)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .map(|freq| freq.count as u64)
            .sum();
        total_words_reviewed as f64 / self.context.language_pack.total_word_count as f64
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_total_reviews(&self) -> u64 {
        self.stats.total_reviews
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_xp(&self) -> f64 {
        self.stats.xp
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_daily_streak(&self) -> u32 {
        match &self.stats.daily_streak {
            None => 0,
            Some(streak) => {
                let now = chrono::Utc::now();
                let hours_since_last = (now - streak.last_review_time).num_hours();

                if hours_since_last <= 30 {
                    // Streak is active (reviewed within last 30 hours)
                    (streak.last_review_time.date_naive() - streak.streak_start.date_naive())
                        .num_days() as u32
                        + 1
                } else {
                    // Streak is broken
                    0
                }
            }
        }
    }

    fn max_cards_to_add(&self) -> usize {
        let current_cards = self.num_cards();

        if current_cards < 5 {
            1
        } else if current_cards < 11 {
            2
        } else {
            5
        }
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn add_card_options(&self, banned_challenge_types: Vec<ChallengeType>) -> AddCardOptions {
        let banned_types_set = banned_challenge_types
            .into_iter()
            .collect::<std::collections::HashSet<_>>();

        let max_cards_to_add = self.max_cards_to_add();

        AddCardOptions {
            smart_add: self
                .next_unknown_cards_with_banned(None, &banned_types_set)
                .take(max_cards_to_add)
                .count() as u32,
            manual_add: vec![
                (
                    if banned_types_set.contains(&ChallengeType::Text) {
                        0
                    } else {
                        self.next_unknown_cards(Some(CardType::TargetLanguage))
                            .take(max_cards_to_add)
                            .count() as u32
                    },
                    CardType::TargetLanguage,
                ),
                (
                    if banned_types_set.contains(&ChallengeType::Listening) {
                        0
                    } else {
                        self.next_unknown_cards(Some(CardType::Listening))
                            .take(max_cards_to_add)
                            .count() as u32
                    },
                    CardType::Listening,
                ),
                (
                    if banned_types_set.contains(&ChallengeType::Speaking) {
                        0
                    } else {
                        self.next_unknown_cards(Some(CardType::LetterPronunciation))
                            .take(max_cards_to_add)
                            .count() as u32
                    },
                    CardType::LetterPronunciation,
                ),
            ],
        }
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn add_next_unknown_cards(
        &self,
        card_type: Option<CardType>,
        count: usize,
        banned_challenge_types: Vec<ChallengeType>,
    ) -> Option<DeckEvent> {
        let banned_types_set = banned_challenge_types
            .into_iter()
            .collect::<std::collections::HashSet<_>>();

        if count == 0 {
            return None;
        }
        let cards = self
            .next_unknown_cards_with_banned(card_type, &banned_types_set)
            .take(count)
            .map(|card| card.resolve(&self.context.language_pack.rodeo))
            .collect::<Vec<_>>();

        (!cards.is_empty()).then_some({
            DeckEvent::Language(LanguageEvent {
                target_language: self.context.target_language,
                native_language: self.context.native_language,
                content: LanguageEventContent::AddCards { cards },
            })
        })
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn review_card(
        &self,
        reviewed: CardIndicator<String>,
        rating: Rating,
    ) -> Option<DeckEvent> {
        let indicator = reviewed.get_interned(&self.context.language_pack.rodeo)?;
        self.cards.get(&indicator).and_then(|status| {
            matches!(status, CardStatus::Tracked(_)).then_some(DeckEvent::Language(LanguageEvent {
                target_language: self.context.target_language,
                native_language: self.context.native_language,
                content: LanguageEventContent::ReviewCard { reviewed, rating },
            }))
        })
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn translate_sentence_perfect(
        &self,
        words_tapped: Vec<Lexeme<String>>,
        challenge_sentence: String,
    ) -> Option<DeckEvent> {
        Some(DeckEvent::Language(LanguageEvent {
            target_language: self.context.target_language,
            native_language: self.context.native_language,
            content: LanguageEventContent::TranslationChallenge {
                review: SentenceReviewIndicator::TargetToNative {
                    challenge_sentence,
                    result: SentenceReviewResult::Perfect {
                        lexemes_needed_hint: words_tapped.into_iter().collect(),
                    },
                },
            },
        }))
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn translate_sentence_wrong(
        &self,
        challenge_sentence: String,
        submission: String,
        words_remembered: Vec<Lexeme<String>>,
        words_forgotten: Vec<Lexeme<String>>,
        words_tapped: Vec<Lexeme<String>>,
    ) -> Option<DeckEvent> {
        Some(DeckEvent::Language(LanguageEvent {
            target_language: self.context.target_language,
            native_language: self.context.native_language,
            content: LanguageEventContent::TranslationChallenge {
                review: SentenceReviewIndicator::TargetToNative {
                    challenge_sentence,
                    result: SentenceReviewResult::Wrong {
                        submission,
                        lexemes_remembered: words_remembered.into_iter().collect(),
                        lexemes_forgotten: words_forgotten.into_iter().collect(),
                        lexemes_needed_hint: words_tapped.into_iter().collect(),
                    },
                },
            },
        }))
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn transcribe_sentence(
        &self,
        challenge: Vec<transcription_challenge::PartGraded>,
    ) -> Option<DeckEvent> {
        Some(DeckEvent::Language(LanguageEvent {
            target_language: self.context.target_language,
            native_language: self.context.native_language,
            content: LanguageEventContent::TranscriptionChallenge { challenge },
        }))
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn num_cards(&self) -> usize {
        self.cards.values().filter_map(CardStatus::reviewed).count()
    }

    /// Get the average number of challenges completed per day in the past week
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_past_week_challenge_average(&self) -> f64 {
        let total_challenges: u32 = self.stats.past_week_challenges.values().sum();
        // Average over 7 days
        total_challenges as f64 / 7.0
    }

    /// Calculate upcoming review statistics for the next week
    /// Returns total reviews and max reviews on any single day
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_upcoming_week_review_stats(&self) -> UpcomingReviewStats {
        let now = Utc::now();
        let one_week_later = now + chrono::Duration::days(7);

        let mut daily_counts: HashMap<i64, u32> = HashMap::new();
        let mut total_reviews = 0u32;

        for (_, card_status) in self.cards.iter() {
            if let CardStatus::Tracked(CardData::Added { fsrs_card }) = card_status {
                let due_date = fsrs_card.due;

                // Skip new cards (they haven't been reviewed yet)
                if fsrs_card.state == rs_fsrs::State::New {
                    continue;
                }

                // Check if due within the next week
                if due_date > now && due_date <= one_week_later {
                    total_reviews += 1;

                    // Get the day offset from today (0 = today, 1 = tomorrow, etc.)
                    let days_from_now = (due_date - now).num_days();
                    *daily_counts.entry(days_from_now).or_insert(0) += 1;
                }
            }
        }

        let max_per_day = daily_counts.values().max().copied().unwrap_or(0);

        UpcomingReviewStats {
            total_reviews,
            max_per_day,
        }
    }

    /// Count the number of cards created within the past `hours` hours.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_cards_added_in_past_hours(&self, hours: f64) -> u32 {
        if !hours.is_finite() || hours <= 0.0 {
            return 0;
        }

        let clamped_hours = hours.min((i64::MAX as f64) / 3600.0);
        let cutoff =
            Utc::now() - chrono::Duration::seconds((clamped_hours * 3600.0).round() as i64);

        self.cards
            .values()
            .filter_map(|card_status| match card_status {
                CardStatus::Tracked(CardData::Added { fsrs_card }) => Some(fsrs_card),
                _ => None,
            })
            .filter(|fsrs_card| fsrs_card.created_at >= cutoff)
            .count() as u32
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_frequency_knowledge_chart_data(&self) -> Vec<FrequencyKnowledgePoint> {
        // Sample frequencies from 1 to 10000 on a logarithmic scale
        let target_frequencies: Vec<f64> = vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 15.0, 20.0, 30.0, 40.0, 50.0, 60.0,
            70.0, 80.0, 90.0, 100.0, 150.0, 200.0, 300.0, 400.0, 500.0, 600.0, 700.0, 800.0, 900.0,
            1000.0, 1500.0, 2000.0, 3000.0, 4000.0, 5000.0, 6000.0, 7000.0, 8000.0, 9000.0,
            10000.0,
        ];

        // Create a map to collect data for each frequency bucket
        let mut frequency_buckets: std::collections::HashMap<String, (Vec<f64>, Vec<String>)> =
            std::collections::HashMap::new();

        // Iterate through actual lexemes in the language pack and find ones matching our target frequencies
        for (lexeme, frequency) in self.context.language_pack.word_frequencies.iter() {
            let freq_value = frequency.count as f64;

            // Check if this frequency is close to one of our target frequencies
            for &target_freq in &target_frequencies {
                if (freq_value - target_freq).abs() < target_freq * 0.1 {
                    // Within 10% of target
                    let card_indicator = CardIndicator::TargetLanguage { lexeme: *lexeme };

                    // Use the regression to predict knowledge at this frequency
                    let knowledge_probability = self
                        .regressions
                        .predict_card_knowledge_probability(&card_indicator, *frequency);

                    // Get the word string for display
                    let word_str = match lexeme {
                        Lexeme::Heteronym(h) => self.context.language_pack.rodeo.resolve(&h.word),
                        Lexeme::Multiword(s) => self.context.language_pack.rodeo.resolve(s),
                    };

                    let bucket_key = format!("{target_freq}");
                    let entry = frequency_buckets
                        .entry(bucket_key)
                        .or_insert((vec![], vec![]));
                    entry.0.push(knowledge_probability);
                    if entry.1.len() < 5 {
                        // Limit to 5 example words per bucket
                        entry.1.push(word_str.to_string());
                    }

                    break;
                }
            }
        }

        // Convert buckets to final chart data
        let mut chart_data = Vec::new();
        for &target_freq in &target_frequencies {
            let bucket_key = format!("{target_freq}");
            if let Some((probabilities, words)) = frequency_buckets.get(&bucket_key) {
                if !probabilities.is_empty() {
                    let avg_probability =
                        probabilities.iter().sum::<f64>() / probabilities.len() as f64;
                    chart_data.push(FrequencyKnowledgePoint {
                        frequency: target_freq,
                        predicted_knowledge: avg_probability,
                        word_count: probabilities.len() as u32,
                        example_words: words.join(", "),
                    });
                }
            }
        }

        chart_data
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct UpcomingReviewStats {
    pub total_reviews: u32,
    pub max_per_day: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify))]
#[cfg_attr(target_arch = "wasm32", tsify(into_wasm_abi))]
pub struct FrequencyKnowledgePoint {
    pub frequency: f64,
    pub predicted_knowledge: f64,
    pub word_count: u32,
    pub example_words: String,
}

impl Deck {
    pub(crate) fn next_unknown_cards(&self, card_type: Option<CardType>) -> NextCardsIterator<'_> {
        let permitted_types = match card_type {
            Some(CardType::TargetLanguage) => vec![ChallengeType::Text],
            Some(CardType::Listening) => vec![ChallengeType::Listening],
            Some(CardType::LetterPronunciation) => vec![ChallengeType::Speaking],
            None => vec![
                ChallengeType::Text,
                ChallengeType::Listening,
                ChallengeType::Speaking,
            ],
        };
        NextCardsIterator::new(self, permitted_types)
    }

    pub(crate) fn next_unknown_cards_with_banned(
        &self,
        card_type: Option<CardType>,
        banned_types: &std::collections::HashSet<ChallengeType>,
    ) -> NextCardsIterator<'_> {
        let all_types = match card_type {
            Some(CardType::TargetLanguage) => vec![ChallengeType::Text],
            Some(CardType::Listening) => vec![ChallengeType::Listening],
            Some(CardType::LetterPronunciation) => vec![ChallengeType::Speaking],
            None => vec![
                ChallengeType::Text,
                ChallengeType::Listening,
                ChallengeType::Speaking,
            ],
        };

        let permitted_types = all_types
            .into_iter()
            .filter(|t| !banned_types.contains(t))
            .collect();
        NextCardsIterator::new(self, permitted_types)
    }

    fn get_card(&self, card_indicator: CardIndicator<Spur>) -> Option<Card> {
        let card_status = self.cards.get(&card_indicator)?;
        let card_data = match card_status {
            CardStatus::Tracked(data) => data,
            CardStatus::Unadded { .. } => return None,
        };

        let card = Card {
            content: match card_indicator {
                CardIndicator::TargetLanguage {
                    lexeme: Lexeme::Heteronym(heteronym),
                } => {
                    let Some(entry) = self
                        .context
                        .language_pack
                        .dictionary
                        .get(&heteronym)
                        .cloned()
                    else {
                        panic!(
                            "Heteronym {:?} was in the deck, but was not found in dictionary",
                            heteronym.resolve(&self.context.language_pack.rodeo)
                        );
                    };
                    CardContent::Heteronym(heteronym, entry.definitions.clone())
                }
                CardIndicator::TargetLanguage {
                    lexeme: Lexeme::Multiword(multiword_term),
                } => {
                    let Some(entry) = self
                        .context
                        .language_pack
                        .phrasebook
                        .get(&multiword_term)
                        .cloned()
                    else {
                        panic!(
                            "Multiword term {:?} was in the deck, but was not found in phrasebook",
                            self.context.language_pack.rodeo.resolve(&multiword_term)
                        );
                    };
                    CardContent::Multiword(
                        multiword_term,
                        MultiwordCardContent {
                            meaning: entry.meaning.clone(),
                            example_sentence_target_language: entry.target_language_example.clone(),
                            example_sentence_native_language: entry.native_language_example.clone(),
                        },
                    )
                }
                CardIndicator::ListeningHomophonous { pronunciation } => {
                    let Some(possible_words) = self
                        .context
                        .language_pack
                        .pronunciation_to_words
                        .get(&pronunciation)
                        .cloned()
                    else {
                        panic!(
                            "Pronunciation {:?} was in the deck, but was not found in pronunciation_to_words",
                            self.context.language_pack.rodeo.resolve(&pronunciation)
                        );
                    };
                    let possible_words = possible_words.into_iter().collect::<BTreeSet<_>>();

                    // figure out which of those words the user knows
                    let possible_words = possible_words
                        .iter()
                        .map(|word| {
                            // Check if any lexeme for this word is known
                            let word_known = self
                                .context
                                .language_pack
                                .pronunciation_to_lexemes(&pronunciation)
                                .filter(|(w, _)| w == word)
                                .any(|(_, lexeme)| {
                                    self.cards
                                        .get(&CardIndicator::TargetLanguage { lexeme })
                                        .is_some_and(|status| {
                                            matches!(status, CardStatus::Tracked(_))
                                        })
                                });
                            (word_known, *word)
                        })
                        .collect();
                    CardContent::Listening {
                        pronunciation,
                        possible_words,
                    }
                }
                CardIndicator::ListeningLexeme { lexeme } => {
                    // For ListeningLexeme, we create a listening card content similar to homophonous
                    // but specifically for the lexeme
                    match lexeme {
                        Lexeme::Heteronym(heteronym) => {
                            // Get the pronunciation from the pronunciation map if available
                            let pronunciation = self
                                .context
                                .language_pack
                                .word_to_pronunciation
                                .get(&heteronym.word)
                                .cloned()
                                .unwrap_or(heteronym.word);

                            CardContent::Listening {
                                pronunciation,
                                possible_words: vec![(true, heteronym.word)],
                            }
                        }
                        Lexeme::Multiword(word) => {
                            // For multiword, just use the word itself as pronunciation
                            CardContent::Listening {
                                pronunciation: word,
                                possible_words: vec![(true, word)],
                            }
                        }
                    }
                }
                CardIndicator::LetterPronunciation { pattern, position } => {
                    let pattern_str = self.context.language_pack.rodeo.resolve(&pattern);
                    let Some(guide) = self
                        .context
                        .language_pack
                        .pronunciation_data
                        .guides
                        .iter()
                        .find(|g| g.pattern == pattern_str && g.position == position)
                        .cloned()
                    else {
                        panic!(
                            "Pattern {pattern_str} with position {position:?} was in the deck, but was not found in pronunciation guides"
                        );
                    };
                    CardContent::LetterPronunciation { pattern, guide }
                }
            },
            fsrs_card: match card_data {
                CardData::Added { fsrs_card } => fsrs_card.clone(),
                CardData::Ghost { fsrs_card } => fsrs_card.clone(),
            },
        };
        Some(card)
    }

    fn card_known(&self, card_indicator: &CardIndicator<Spur>) -> bool {
        self.cards
            .get(card_indicator)
            .and_then(|status| status.reviewed())
            .is_some()
    }

    fn lexeme_known(&self, lexeme: &Lexeme<Spur>) -> bool {
        self.card_known(&CardIndicator::TargetLanguage { lexeme: *lexeme })
    }

    fn get_comprehensible_sentence_containing(
        &self,
        required_lexeme: Option<&Lexeme<Spur>>,
        mut comprehensible_words: BTreeSet<Lexeme<Spur>>,
        sentences_reviewed: &BTreeMap<Spur, u32>,
        language_pack: &LanguagePack,
    ) -> Option<ComprehensibleSentence> {
        // Add the target word to comprehensible words if provided
        if let Some(required_lexeme) = required_lexeme {
            comprehensible_words.insert(*required_lexeme);
        }

        // Search through all sentences - if we have a required lexeme, only look at sentences containing it
        let candidate_sentences: Vec<Spur> = if let Some(required_lexeme) = required_lexeme {
            language_pack
                .sentences_containing_lexeme_index
                .get(required_lexeme)?
                .clone()
        } else {
            // If no required lexeme, consider all sentences
            language_pack.translations.keys().cloned().collect()
        };

        let mut possible_sentences = Vec::new();

        // Warning: this loop is HOT!
        'checkSentences: for sentence in &candidate_sentences {
            let Some(lexemes) = language_pack.sentences_to_all_lexemes.get(sentence) else {
                continue;
            };

            for lexeme in lexemes {
                if !comprehensible_words.contains(lexeme) {
                    continue 'checkSentences; // Early exit!
                }
            }

            possible_sentences.push(sentence);
        }

        if !possible_sentences.is_empty() {
            possible_sentences.sort_by_key(|sentence| {
                let sentence_review_count = sentences_reviewed.get(sentence).unwrap_or(&0);
                *sentence_review_count
            });
            let target_language = **possible_sentences.first()?;

            let lexemes = language_pack
                .sentences_to_all_lexemes
                .get(&target_language)?;

            let unique_target_language_lexemes = {
                let mut unique_target_language_lexemes = vec![];
                let mut lexemes_set = BTreeSet::new();

                for lexeme in lexemes {
                    if !lexemes_set.contains(&lexeme) {
                        unique_target_language_lexemes.push(*lexeme);
                        lexemes_set.insert(lexeme);
                    }
                }
                unique_target_language_lexemes
            };

            let native_languages = language_pack
                .translations
                .get(&target_language)
                .unwrap()
                .clone();

            let target_language_literals = language_pack
                .sentences_to_literals
                .get(&target_language)
                .unwrap()
                .clone();

            return Some(ComprehensibleSentence {
                target_language,
                target_language_literals,
                unique_target_language_lexemes,
                native_languages,
            });
        }

        None
    }
}

impl Context {
    /// Check if a card is valid and can be added to the deck
    /// For lexeme cards: checks if they exist in word_frequencies (which guarantees they have definitions)
    /// For listening cards: checks if the pronunciation exists
    /// For letter pronunciation cards: checks if the pattern exists in the frequency map
    pub fn is_card_valid(&self, card: &CardIndicator<Spur>) -> bool {
        match card {
            CardIndicator::TargetLanguage { lexeme } => {
                // Check if lexeme exists in word_frequencies (which guarantees it has a definition)
                self.language_pack.word_frequencies.contains_key(lexeme)
            }
            CardIndicator::ListeningHomophonous { pronunciation } => self
                .language_pack
                .pronunciation_to_words
                .contains_key(pronunciation),
            CardIndicator::ListeningLexeme { lexeme } => {
                // Check if lexeme exists in word_frequencies (which guarantees it has a definition)
                self.language_pack.word_frequencies.contains_key(lexeme)
            }
            CardIndicator::LetterPronunciation { pattern, position } => self
                .language_pack
                .pattern_frequency_map
                .contains_key(&(*pattern, *position)),
        }
    }

    fn is_comprehensible(
        &self,
        card_indicator: &CardIndicator<Spur>,
        card_status: &CardStatus,
        regressions: &Regressions,
    ) -> bool {
        match card_status {
            // For tracked cards (both Added and Ghost), check if they're in review state
            CardStatus::Tracked(card_data) => {
                match card_data {
                    CardData::Added { fsrs_card } | CardData::Ghost { fsrs_card } => {
                        // Card is comprehensible if it's in review state (not new, learning, or relearning)
                        fsrs_card.state == rs_fsrs::State::Review
                    }
                }
            }
            // For unadded cards, use regression predictions
            CardStatus::Unadded(_) => {
                // Check if we have high confidence they would be known
                // Use 80% probability threshold for considering a card comprehensible
                // 80% was not chosen in a super scientific way, it's just a number that seemed to work well
                if let Some((knowledge_probability, _)) =
                    self.get_card_knowledge_probability(card_indicator, regressions)
                {
                    knowledge_probability >= 0.80
                } else {
                    false
                }
            }
        }
    }

    fn get_card_value(
        &self,
        card: &CardIndicator<Spur>,
        regressions: &Regressions,
    ) -> Option<ordered_float::NotNan<f64>> {
        let (knowledge_probability, frequency) =
            self.get_card_knowledge_probability(card, regressions)?;
        ordered_float::NotNan::new((1.0 - knowledge_probability) * (frequency.sqrt_frequency()))
            .ok()
    }

    fn get_card_value_with_status(
        &self,
        card: &CardIndicator<Spur>,
        status: &CardStatus,
        regressions: &Regressions,
    ) -> Option<ordered_float::NotNan<f64>> {
        let frequency = self.get_card_frequency(card)?;

        // Check if we have a reviewed card (ghost or added)
        if let CardStatus::Tracked(card_data) = status {
            // Get the FSRS card using explicit pattern match
            let fsrs_card = match card_data {
                CardData::Added { fsrs_card } | CardData::Ghost { fsrs_card } => fsrs_card,
            };

            // If it's been reviewed (not new), use the actual knowledge from FSRS
            if fsrs_card.state != rs_fsrs::State::New {
                // Get the predicted knowledge
                let predicted_knowledge = regressions.predict_card_knowledge(card, frequency)?;

                // Calculate observed knowledge from FSRS data
                let observed_knowledge = if fsrs_card.lapses == 0 {
                    fsrs_card.accumulated_positive_surprise
                } else {
                    -fsrs_card.accumulated_negative_surprise
                };

                // For ghost cards, combine observed and predicted
                // For added cards, just use observed
                let combined_knowledge = match card_data {
                    CardData::Ghost { .. } => {
                        if observed_knowledge < 0.0 {
                            // Has lapses: use whichever is lower (more pessimistic)
                            observed_knowledge.min(predicted_knowledge)
                        } else {
                            // No lapses: add positive surprisal to prediction
                            observed_knowledge + predicted_knowledge
                        }
                    }
                    CardData::Added { .. } => {
                        // Added card - use actual knowledge
                        observed_knowledge
                    }
                };

                // Convert knowledge to probability and then to value
                let probability = Regressions::knowledge_to_probability(combined_knowledge);
                return ordered_float::NotNan::new(
                    (1.0 - probability) * frequency.sqrt_frequency(),
                )
                .ok();
            }
        }

        // Fall back to regular prediction-based value for new or unadded cards
        self.get_card_value(card, regressions)
    }

    fn get_card_knowledge_probability(
        &self,
        card: &CardIndicator<Spur>,
        regressions: &Regressions,
    ) -> Option<(f64, Frequency)> {
        let frequency = self.get_card_frequency(card)?;

        let knowledge_probability = match card {
            CardIndicator::LetterPronunciation { pattern, position } => {
                // For pronunciation patterns, use the LLM's familiarity assessment
                let pattern_str = self.language_pack.rodeo.resolve(pattern);
                let guide = self
                    .language_pack
                    .pronunciation_data
                    .guides
                    .iter()
                    .find(|g| g.pattern == pattern_str && g.position == *position)?;

                // Convert familiarity to probability
                match guide.familiarity {
                    language_utils::PronunciationFamiliarity::LikelyAlreadyKnows => 0.85,
                    language_utils::PronunciationFamiliarity::MaybeAlreadyKnows => 0.50,
                    language_utils::PronunciationFamiliarity::ProbablyDoesNotKnow => 0.15,
                }
            }
            _ => regressions.predict_card_knowledge_probability(card, frequency),
        };

        Some((knowledge_probability, frequency))
    }

    /// Get the frequency count for a card (used for isotonic regression)
    fn get_card_frequency(&self, card: &CardIndicator<Spur>) -> Option<Frequency> {
        match card {
            CardIndicator::TargetLanguage { lexeme } => {
                self.language_pack.word_frequencies.get(lexeme).copied()
            }
            CardIndicator::ListeningHomophonous { pronunciation } => {
                // For listening cards, use the maximum frequency of any word it could be
                self.language_pack
                    .pronunciation_max_frequency(pronunciation)
            }
            CardIndicator::ListeningLexeme { lexeme } => {
                // For listening lexeme cards, use the same frequency as the target language card
                self.language_pack.word_frequencies.get(lexeme).copied()
            }
            CardIndicator::LetterPronunciation { pattern, position } => {
                // Look up the actual frequency of this pattern from our calculated data
                let count = self
                    .language_pack
                    .pattern_frequency_map
                    .get(&(*pattern, *position))
                    .copied()
                    .unwrap_or(0);
                Some(Frequency { count })
            }
        }
    }
}

impl Regressions {
    /// Predict the pre-existing knowledge of a card based on its frequency using isotonic regression
    /// Returns None if the card type has no regression model or frequency can't be determined
    pub(crate) fn predict_card_knowledge(
        &self,
        card: &CardIndicator<Spur>,
        frequency: Frequency,
    ) -> Option<f64> {
        let regression = match card {
            CardIndicator::TargetLanguage { .. } => self.target_language_regression.as_ref(),
            CardIndicator::ListeningHomophonous { .. } | CardIndicator::ListeningLexeme { .. } => {
                self.listening_regression.as_ref()
            }
            CardIndicator::LetterPronunciation { .. } => {
                // For pronunciation patterns, we don't use regression
                // Instead we use the LLM's familiarity assessment in predict_card_knowledge_probability
                return None;
            }
        }?;

        // Compute smoothed prediction by averaging at frequency ±20%
        let base_freq = frequency.sqrt_frequency();
        let lower_freq = base_freq * 0.8;
        let upper_freq = base_freq * 1.2;

        // Get predictions at all three points
        let predictions = [
            regression.interpolate(lower_freq),
            regression.interpolate(base_freq),
            regression.interpolate(upper_freq),
        ];

        // Average the available predictions
        let valid_predictions: Vec<f64> = predictions.into_iter().flatten().collect();
        if valid_predictions.is_empty() {
            None
        } else {
            Some(valid_predictions.iter().sum::<f64>() / valid_predictions.len() as f64)
        }
    }

    /// Get the predicted probability of knowing a card (0.0 to 1.0).
    /// Based on accumulated surprise (pre-existing knowledge) from review history.
    /// The relationship maps knowledge to probability:
    ///
    /// - Knowledge >= 3.0 = 95% chance of knowing (easy cards)
    /// - Knowledge = 0 = 50% chance of knowing (neutral)
    /// - Knowledge <= -2.0 = 10% chance of knowing (failed cards)
    /// - Linear interpolation between these points
    pub(crate) fn predict_card_knowledge_probability(
        &self,
        card: &CardIndicator<Spur>,
        frequency: Frequency,
    ) -> f64 {
        let Some(knowledge) = self.predict_card_knowledge(card, frequency) else {
            return 0.0;
        };
        Self::knowledge_to_probability(knowledge)
    }

    fn knowledge_to_probability(knowledge: f64) -> f64 {
        // With pre-existing knowledge:
        // - Positive values indicate easier cards (higher probability)
        // - Negative values indicate harder cards (lower probability)
        // - Any negative value indicates at least one lapse
        //
        // Based on latest test results:
        //   - Easy review gives ~4.6 positive surprise
        //   - Good review gives ~2.3 positive surprise initially
        //   - Initial again review gives ~0.1 negative surprise
        //   - Again after success gives ~2.4 negative surprise

        // Key insight: negative values (lapses > 0) always indicate struggling cards
        if knowledge < 0.0 {
            // Card has been failed at least once
            // New algorithm: initial failures have small negative (~0.1)
            // Failures after success have larger negative (~2.4)

            if knowledge >= -0.15 {
                // Very small negative (likely initial failure ~0.1): 10-15% probability
                // Initial failures indicate genuine lack of knowledge
                0.10 + 0.05 * ((knowledge + 0.15) / 0.15)
            } else if knowledge >= -1.0 {
                // Small to moderate negative: 5-10% probability
                let range = 1.0 - 0.15;
                0.05 + 0.05 * ((knowledge + 1.0) / range)
            } else if knowledge >= -3.0 {
                // Significant negative (failed after knowing ~2.4): 2-5% probability
                let range = 3.0 - 1.0;
                0.02 + 0.03 * ((knowledge + 3.0) / range)
            } else {
                // Deep negative surprise: cap at 2%
                0.02
            }
        } else {
            // Card has never been failed (positive knowledge)
            // Map positive surprise to higher probability
            const EASY_THRESHOLD: f64 = 4.5; // Easy review level (~4.6)
            const GOOD_THRESHOLD: f64 = 2.0; // Good review level (~2.3)

            if knowledge >= EASY_THRESHOLD {
                // Easy-level knowledge: 90-95% probability
                0.95
            } else if knowledge >= GOOD_THRESHOLD {
                // Good-level knowledge: 70-90% probability
                let range = EASY_THRESHOLD - GOOD_THRESHOLD;
                0.7 + 0.25 * (knowledge - GOOD_THRESHOLD) / range
            } else if knowledge > 0.0 {
                // Low positive knowledge: 10-70% probability
                let range = GOOD_THRESHOLD;
                0.1 + 0.6 * knowledge / range
            } else {
                // Zero knowledge (new card): 10% probability
                0.1
            }
        }
    }
}

#[derive(Debug)]
pub struct Card {
    content: CardContent<Spur>,
    fsrs_card: rs_fsrs::Card,
}

#[derive(tsify::Tsify, serde::Serialize, serde::Deserialize, Debug, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct MultiwordCardContent {
    meaning: String,
    example_sentence_target_language: String,
    example_sentence_native_language: String,
}

#[derive(tsify::Tsify, serde::Serialize, serde::Deserialize, Debug, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum CardContent<S> {
    Heteronym(Heteronym<S>, Vec<TargetToNativeWord>),
    Multiword(S, MultiwordCardContent),
    Listening {
        pronunciation: S,
        possible_words: Vec<(bool, S)>,
    },
    LetterPronunciation {
        pattern: S,
        guide: PronunciationGuide,
    },
}

impl<S> CardContent<S> {
    fn lexeme(&self) -> Option<Lexeme<S>>
    where
        S: Clone,
    {
        match self {
            CardContent::Heteronym(heteronym, _) => Some(Lexeme::Heteronym(heteronym.clone())),
            CardContent::Multiword(multiword_term, _) => {
                Some(Lexeme::Multiword(multiword_term.clone()))
            }
            CardContent::Listening { .. } => None,
            CardContent::LetterPronunciation { .. } => None,
        }
    }

    fn pronunciation(&self) -> Option<S>
    where
        S: Clone,
    {
        match self {
            CardContent::Listening { pronunciation, .. } => Some(pronunciation.clone()),
            _ => None,
        }
    }
}

impl CardContent<Spur> {
    fn resolve(&self, rodeo: &lasso::RodeoReader) -> CardContent<String> {
        match self {
            CardContent::Heteronym(heteronym, definitions) => {
                CardContent::Heteronym(heteronym.resolve(rodeo), definitions.clone())
            }
            CardContent::Multiword(multiword, content) => {
                CardContent::Multiword(rodeo.resolve(multiword).to_string(), content.clone())
            }
            CardContent::Listening {
                pronunciation,
                possible_words,
            } => CardContent::Listening {
                pronunciation: rodeo.resolve(pronunciation).to_string(),
                possible_words: possible_words
                    .iter()
                    .map(|(known, word)| (*known, rodeo.resolve(word).to_string()))
                    .collect(),
            },
            CardContent::LetterPronunciation { pattern, guide } => {
                CardContent::LetterPronunciation {
                    pattern: rodeo.resolve(pattern).to_string(),
                    guide: guide.clone(),
                }
            }
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Clone)]
pub struct ReviewInfo {
    due_cards: Vec<CardIndicator<Spur>>,
    due_but_banned_cards: Vec<CardIndicator<Spur>>,
    future_cards: Vec<CardIndicator<Spur>>,
}

#[derive(tsify::Tsify, serde::Serialize, serde::Deserialize, Debug, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "type")]
pub enum Challenge<S> {
    FlashCardReview {
        indicator: CardIndicator<S>,
        content: CardContent<S>,
        audio: AudioRequest,
        is_new: bool,
        listening_prefix: Option<String>,
    },
    TranslateComprehensibleSentence(TranslateComprehensibleSentence<S>),
    TranscribeComprehensibleSentence(TranscribeComprehensibleSentence<S>),
}

impl<S> Challenge<S> {
    fn audio_request(&self) -> AudioRequest {
        match self {
            Challenge::FlashCardReview { audio, .. } => audio.clone(),
            Challenge::TranslateComprehensibleSentence(translate_comprehensible_sentence) => {
                translate_comprehensible_sentence.audio.clone()
            }
            Challenge::TranscribeComprehensibleSentence(transcribe_comprehensible_sentence) => {
                transcribe_comprehensible_sentence.audio.clone()
            }
        }
    }
}

impl Challenge<Spur> {
    fn resolve(&self, rodeo: &lasso::RodeoReader) -> Challenge<String> {
        match self {
            Challenge::FlashCardReview {
                indicator,
                content,
                audio,
                is_new,
                listening_prefix,
            } => Challenge::FlashCardReview {
                indicator: indicator.resolve(rodeo),
                content: content.resolve(rodeo),
                audio: audio.clone(),
                is_new: *is_new,
                listening_prefix: listening_prefix.clone(),
            },
            Challenge::TranslateComprehensibleSentence(translate_comprehensible_sentence) => {
                Challenge::TranslateComprehensibleSentence(
                    translate_comprehensible_sentence.resolve(rodeo),
                )
            }
            Challenge::TranscribeComprehensibleSentence(transcribe_comprehensible_sentence) => {
                Challenge::TranscribeComprehensibleSentence(
                    transcribe_comprehensible_sentence.resolve(rodeo),
                )
            }
        }
    }
}

#[derive(
    tsify::Tsify, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize, Debug, Clone, Copy,
)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum ChallengeType {
    Text,
    Listening,
    Speaking,
}

impl ReviewInfo {
    /// Get the set of comprehensible lexemes (words that are known/in review state)
    fn get_comprehensible_written_lexemes(&self, deck: &Deck) -> BTreeSet<Lexeme<Spur>> {
        deck.cards
            .iter()
            .filter_map(|(card_indicator, card_status)| match card_indicator {
                CardIndicator::TargetLanguage { lexeme } => {
                    Some((card_indicator, *lexeme, card_status))
                }
                _ => None,
            })
            .filter(|(card_indicator, _lexeme, card_status)| {
                deck.context
                    .is_comprehensible(card_indicator, card_status, &deck.regressions)
            })
            .map(|(_card_indicator, lexeme, _card_status)| lexeme)
            .collect()
    }

    /// Find a sentence where all lexemes have ListeningLexeme cards
    fn find_listening_lexeme_sentence(
        &self,
        required_lexeme: &Lexeme<Spur>,
        deck: &Deck,
    ) -> Option<ComprehensibleSentence> {
        let language_pack = &deck.context.language_pack;
        // Get all lexemes that have ListeningLexeme cards
        let listening_lexeme_set: BTreeSet<Lexeme<Spur>> = deck
            .cards
            .keys()
            .filter_map(|card| match card {
                CardIndicator::ListeningLexeme { lexeme } => Some(*lexeme),
                _ => None,
            })
            .collect();

        // If no ListeningLexeme cards exist, return None
        if listening_lexeme_set.is_empty() {
            return None;
        }

        // Use the refactored function to find a sentence containing the required lexeme
        // where all lexemes are in the ListeningLexeme set
        deck.get_comprehensible_sentence_containing(
            Some(required_lexeme), // Pass the specific lexeme we're testing
            listening_lexeme_set,
            &deck.stats.sentences_reviewed,
            language_pack,
        )
    }

    pub fn get_challenge_for_card(
        &self,
        deck: &Deck,
        card_indicator: CardIndicator<Spur>,
    ) -> Option<Challenge<String>> {
        let card = deck.get_card(card_indicator)?;
        let language_pack = &deck.context.language_pack;

        // If we can't find a suitable challenge, we'll return a flashcard challenge. Let's construct it here
        let listening_prefix = matches!(&card.content, CardContent::Listening { .. })
            .then(|| Self::get_listening_prefix(deck.context.target_language).to_string());

        let flashcard = Challenge::<Spur>::FlashCardReview {
            audio: match &card.content {
                CardContent::Heteronym(heteronym, _) => AudioRequest {
                    request: TtsRequest {
                        text: language_pack.rodeo.resolve(&heteronym.word).to_string(),
                        language: deck.context.target_language,
                    },
                    provider: TtsProvider::Google,
                },
                CardContent::Multiword(multiword, _) => AudioRequest {
                    request: TtsRequest {
                        text: language_pack.rodeo.resolve(multiword).to_string(),
                        language: deck.context.target_language,
                    },
                    provider: TtsProvider::Google,
                },
                CardContent::Listening {
                    pronunciation: _,
                    possible_words,
                } => AudioRequest {
                    request: TtsRequest {
                        text: format!(
                            "{}... \"{}\".",
                            Self::get_listening_prefix(deck.context.target_language),
                            language_pack.rodeo.resolve(
                                &possible_words
                                    .iter()
                                    .find(|(known, _)| *known)
                                    .or(possible_words.first())
                                    .cloned()
                                    .unwrap()
                                    .1
                            )
                        ),
                        language: deck.context.target_language,
                    },
                    provider: TtsProvider::Google,
                },
                CardContent::LetterPronunciation { pattern, .. } => {
                    // For pronunciation patterns, read the pattern itself
                    AudioRequest {
                        request: TtsRequest {
                            text: language_pack.rodeo.resolve(pattern).to_string(),
                            language: deck.context.target_language,
                        },
                        provider: TtsProvider::Google,
                    }
                }
            },
            indicator: card_indicator,
            content: card.content.clone(),
            is_new: card.fsrs_card.state == rs_fsrs::State::New,
            listening_prefix: listening_prefix.clone(),
        };

        let challenge: Challenge<Spur> = if card.fsrs_card.state == rs_fsrs::State::New
            && !matches!(card_indicator, CardIndicator::ListeningLexeme { .. })
        {
            flashcard
        } else if let CardIndicator::ListeningLexeme { lexeme } = card_indicator {
            // For ListeningLexeme cards, find a sentence containing this specific lexeme
            if let Some(sentence) = self.find_listening_lexeme_sentence(&lexeme, deck) {
                // Create a transcription challenge where only words are transcribed, punctuation is provided
                // Group consecutive words together and consecutive punctuation together
                let mut parts: Vec<transcription_challenge::Part> = Vec::new();
                let mut current_words: Vec<language_utils::Literal<String>> = Vec::new();

                for literal in &sentence.target_language_literals {
                    let resolved = literal.resolve(&language_pack.rodeo);

                    if resolved.heteronym.is_some() {
                        // This is a word - add to current words group
                        current_words.push(resolved);
                    } else {
                        // This is punctuation - flush any accumulated words first
                        if !current_words.is_empty() {
                            parts.push(transcription_challenge::Part::AskedToTranscribe {
                                parts: current_words.clone(),
                            });
                            current_words.clear();
                        }
                        // Add the punctuation as provided
                        parts.push(transcription_challenge::Part::Provided { part: resolved });
                    }
                }

                // Flush any remaining words
                if !current_words.is_empty() {
                    parts.push(transcription_challenge::Part::AskedToTranscribe {
                        parts: current_words,
                    });
                }

                Challenge::TranscribeComprehensibleSentence(TranscribeComprehensibleSentence {
                    target_language: sentence.target_language,
                    native_language: *sentence.native_languages.first().unwrap(),
                    parts,
                    audio: AudioRequest {
                        request: TtsRequest {
                            text: language_pack
                                .rodeo
                                .resolve(&sentence.target_language)
                                .to_string(),
                            language: deck.context.target_language,
                        },
                        provider: TtsProvider::ElevenLabs,
                    },
                })
            } else {
                flashcard
            }
        } else if let Some(pronunciation) = card.content.pronunciation() {
            let mut heteronyms = language_pack
                .pronunciation_to_words
                .get(&pronunciation)
                .unwrap()
                .iter()
                .cloned()
                .flat_map(|word| {
                    language_pack
                        .words_to_heteronyms
                        .get(&word)
                        .unwrap()
                        .clone()
                })
                .filter(|heteronym| deck.lexeme_known(&Lexeme::Heteronym(*heteronym)))
                .collect::<Vec<_>>();
            heteronyms
                .sort_by_key(|heteronym| deck.stats.words_listened_to.get(heteronym).unwrap_or(&0));

            if let Some((target_heteronym, sentence)) = heteronyms
                .iter()
                .filter_map(|heteronym| {
                    let comprehensible_lexemes = self.get_comprehensible_written_lexemes(deck);
                    let sentence = deck.get_comprehensible_sentence_containing(
                        Some(&Lexeme::Heteronym(*heteronym)),
                        comprehensible_lexemes,
                        &deck.stats.sentences_reviewed,
                        language_pack,
                    )?;
                    Some((*heteronym, sentence))
                })
                .next()
            {
                let parts = sentence
                    .target_language_literals
                    .into_iter()
                    .map(|literal| {
                        if let Some(ref heteronym) = literal.heteronym
                            && heteronym == &target_heteronym
                        {
                            transcription_challenge::Part::AskedToTranscribe {
                                parts: vec![literal.resolve(&language_pack.rodeo)],
                            }
                        } else {
                            transcription_challenge::Part::Provided {
                                part: literal.resolve(&language_pack.rodeo),
                            }
                        }
                    })
                    .collect();
                Challenge::TranscribeComprehensibleSentence(TranscribeComprehensibleSentence {
                    target_language: sentence.target_language,
                    native_language: *sentence.native_languages.first().unwrap(),
                    parts,
                    audio: AudioRequest {
                        request: TtsRequest {
                            text: language_pack
                                .rodeo
                                .resolve(&sentence.target_language)
                                .to_string(),
                            language: deck.context.target_language,
                        },
                        provider: TtsProvider::ElevenLabs,
                    },
                })
            } else {
                flashcard
            }
        } else if let Some(lexeme) = card.content.lexeme() {
            if let Some(ComprehensibleSentence {
                target_language,
                target_language_literals,
                unique_target_language_lexemes,
                native_languages,
            }) = {
                let comprehensible_lexemes = self.get_comprehensible_written_lexemes(deck);
                deck.get_comprehensible_sentence_containing(
                    Some(&lexeme),
                    comprehensible_lexemes,
                    &deck.stats.sentences_reviewed,
                    language_pack,
                )
            } {
                let unique_target_language_lexeme_definitions = unique_target_language_lexemes
                    .iter()
                    .map(|lexeme| {
                        let definitions = match lexeme {
                            Lexeme::Heteronym(heteronym) => language_pack
                                .dictionary
                                .get(heteronym)
                                .map(|entry| entry.definitions.clone())
                                .unwrap_or_default(),
                            Lexeme::Multiword(term) => language_pack
                                .phrasebook
                                .get(term)
                                .map(|entry| {
                                    vec![TargetToNativeWord {
                                        native: entry.meaning.clone(),
                                        note: Some(entry.additional_notes.clone()),
                                        example_sentence_target_language: entry
                                            .target_language_example
                                            .clone(),
                                        example_sentence_native_language: entry
                                            .native_language_example
                                            .clone(),
                                    }]
                                })
                                .unwrap_or_default(),
                        };
                        (*lexeme, definitions)
                    })
                    .collect();

                Challenge::TranslateComprehensibleSentence(TranslateComprehensibleSentence {
                    target_language,
                    target_language_literals,
                    unique_target_language_lexemes,
                    native_translations: native_languages,
                    primary_expression: lexeme,
                    unique_target_language_lexeme_definitions,
                    audio: AudioRequest {
                        request: TtsRequest {
                            text: language_pack.rodeo.resolve(&target_language).to_string(),
                            language: deck.context.target_language,
                        },
                        provider: TtsProvider::ElevenLabs,
                    },
                })
            } else {
                flashcard
            }
        } else {
            flashcard
        };

        Some(challenge.resolve(&language_pack.rodeo))
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl ReviewInfo {
    fn get_listening_prefix(language: Language) -> &'static str {
        match language {
            Language::French => "Le mot est",
            Language::Spanish => "La palabra es",
            Language::English => "The word is",
            Language::Korean => "단어는",
            Language::German => "Das Wort ist",
        }
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_next_challenge(&self, deck: &Deck) -> Option<Challenge<String>> {
        if !self.due_cards.is_empty() {
            Some(self.get_challenge_for_card(deck, self.due_cards[0])?)
        } else {
            None
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl ReviewInfo {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn due_count(&self) -> usize {
        self.due_cards.len()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn due_but_banned_count(&self) -> usize {
        self.due_but_banned_cards.len()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn future_count(&self) -> usize {
        self.future_cards.len()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn total_count(&self) -> usize {
        self.due_cards.len() + self.future_cards.len()
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct CardSummary {
    card_indicator: CardIndicator<String>,
    due_timestamp_ms: f64,
    state: String,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl CardSummary {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn card_indicator(&self) -> CardIndicator<String> {
        self.card_indicator.clone()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn due_timestamp_ms(&self) -> f64 {
        self.due_timestamp_ms
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn state(&self) -> String {
        self.state.clone()
    }
}

#[wasm_bindgen]
pub fn test_fn(f: js_sys::Function) {
    f.call0(&JsValue::NULL).unwrap();
}

#[derive(tsify::Tsify, serde::Serialize, serde::Deserialize, Debug, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct AudioRequest {
    request: TtsRequest,
    provider: TtsProvider,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn get_audio(
    request: AudioRequest,
    access_token: Option<String>,
) -> Result<js_sys::Uint8Array, JsValue> {
    let audio_cache = audio::AudioCache::new().await?;
    let bytes = audio_cache
        .fetch_and_cache(&request, access_token.as_ref())
        .await?;
    Ok(js_sys::Uint8Array::from(&bytes[..]))
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn autograde_translation(
    challenge_sentence: String,
    user_sentence: String,
    primary_expression: Lexeme<String>,
    lexemes: Vec<Lexeme<String>>,
    access_token: Option<String>,
    language: Language,
) -> Result<autograde::AutoGradeTranslationResponse, JsValue> {
    let request = autograde::AutoGradeTranslationRequest {
        challenge_sentence,
        user_sentence,
        primary_expression: primary_expression.clone(),
        lexemes,
        language,
    };

    let response = hit_ai_server("/autograde-translation", request, access_token.as_ref())
        .await
        .map_err(|e| JsValue::from_str(&format!("Request error: {e:?}")))?;

    if !response.ok() {
        return Err(JsValue::from_str(&format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    let mut response: autograde::AutoGradeTranslationResponse = response
        .json()
        .await
        .map_err(|e| JsValue::from_str(&format!("Response parsing error: {e:?}")))?;

    // make sure the primary expression is in the appropriate array:
    if response.primary_expression_status == autograde::Remembered::Forgot
        && !response.expressions_forgot.contains(&primary_expression)
    {
        response.expressions_forgot.push(primary_expression);
    } else if response.primary_expression_status == autograde::Remembered::Remembered
        && !response
            .expressions_remembered
            .contains(&primary_expression)
    {
        response.expressions_remembered.push(primary_expression);
    }

    log::info!("Autograde response: {response:#?}");

    Ok(response)
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn autograde_transcription(
    submission: Vec<transcription_challenge::PartSubmitted>,
    access_token: Option<String>,
    language: Language,
) -> transcription_challenge::Grade {
    let _autograde_error =
        match autograde_transcription_llm(submission.clone(), access_token, language).await {
            Ok(grade) => return grade,
            Err(e) => Some(e),
        };

    // fall back to some heuristic grading
    let results = submission
        .into_iter()
        .map(|part| match part {
            transcription_challenge::PartSubmitted::AskedToTranscribe { parts, submission } => {
                let submitted_words = submission.split_whitespace().collect::<Vec<_>>();
                if submitted_words.len() != parts.len() {
                    return transcription_challenge::PartGraded::AskedToTranscribe {
                        parts: parts
                            .iter()
                            .map(|part| transcription_challenge::PartGradedPart {
                                heard: part.clone(),
                                grade: transcription_challenge::WordGrade::Missed {},
                            })
                            .collect(),
                        submission: submission.clone(),
                    };
                }

                transcription_challenge::PartGraded::AskedToTranscribe {
                    parts: parts
                        .iter()
                        .zip(submitted_words.iter())
                        .map(|(part, &submission)| {
                            let part_text = part.text.to_lowercase().trim().to_string();
                            let submission = submission.to_lowercase().trim().to_string();
                            if part_text == submission {
                                transcription_challenge::PartGradedPart {
                                    heard: part.clone(),
                                    grade: transcription_challenge::WordGrade::Perfect {},
                                }
                            } else if remove_accents(&part_text) == remove_accents(&submission) {
                                transcription_challenge::PartGradedPart {
                                    heard: part.clone(),
                                    grade: transcription_challenge::WordGrade::CorrectWithTypo {},
                                }
                            // todo: check if word entered is in the set of homophones
                            // and if so, grade is as correct PhoneticallyIdenticalButContextuallyIncorrect
                            } else {
                                transcription_challenge::PartGradedPart {
                                    heard: part.clone(),
                                    grade: transcription_challenge::WordGrade::Incorrect {},
                                }
                            }
                        })
                        .collect(),
                    submission: submission.clone(),
                }
            }
            transcription_challenge::PartSubmitted::Provided { part } => {
                transcription_challenge::PartGraded::Provided { part }
            }
        })
        .collect();

    transcription_challenge::Grade {
        explanation: None,
        results,
        compare: Vec::new(),
        autograding_error: Some("The LLM was not able to grade this transcription".to_string()),
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn autograde_transcription_llm(
    submission: Vec<transcription_challenge::PartSubmitted>,
    access_token: Option<String>,
    language: Language,
) -> Result<transcription_challenge::Grade, JsValue> {
    // Check if all answers are exactly correct (case-insensitive)
    let all_correct = submission.iter().all(|part| match part {
        transcription_challenge::PartSubmitted::AskedToTranscribe { parts, submission } => {
            let submission = submission.trim().to_lowercase();
            let parts = parts
                .iter()
                .map(|part| {
                    format!(
                        "{text}{whitespace}",
                        text = part.text.to_lowercase(),
                        whitespace = part.whitespace
                    )
                })
                .collect::<Vec<_>>();
            submission.trim() == parts.join("").trim()
        }
        transcription_challenge::PartSubmitted::Provided { .. } => true,
    });
    if all_correct {
        // Skip server call and return perfect results
        let results = submission
            .into_iter()
            .map(|part| match part {
                transcription_challenge::PartSubmitted::AskedToTranscribe { parts, submission } => {
                    let parts = parts
                        .iter()
                        .map(|part| transcription_challenge::PartGradedPart {
                            heard: part.clone(),
                            grade: transcription_challenge::WordGrade::Perfect {},
                        })
                        .collect();
                    transcription_challenge::PartGraded::AskedToTranscribe {
                        parts,
                        submission: submission.clone(),
                    }
                }
                transcription_challenge::PartSubmitted::Provided { part } => {
                    transcription_challenge::PartGraded::Provided { part }
                }
            })
            .collect();

        return Ok(transcription_challenge::Grade {
            explanation: None,
            results,
            compare: Vec::new(),
            autograding_error: None,
        });
    }

    let request = autograde::AutoGradeTranscriptionRequest {
        submission,
        language,
    };

    let response = hit_ai_server("/autograde-transcription", &request, access_token.as_ref())
        .await
        .map_err(|e| JsValue::from_str(&format!("Request error: {e:?}")))?;

    let response: transcription_challenge::Grade = response
        .json()
        .await
        .map_err(|e| JsValue::from_str(&format!("Response parsing error: {e:?}")))?;

    Ok(response)
}

fn remove_accents(s: &str) -> String {
    use unicode_normalization::UnicodeNormalization;

    s.nfd()
        .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
        .collect()
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn get_courses() -> Vec<language_utils::Course> {
    language_utils::COURSES.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Days;
    use language_utils::ArchivedConsolidatedLanguageDataWithCapacity;

    // Include the French language data at compile time for tests
    // This makes tests self-contained and doesn't include the data in production builds
    const FRENCH_LANGUAGE_DATA: &[u8] = include_bytes!("../../out/fra_for_eng/language_data.rkyv");

    impl Default for Deck {
        fn default() -> Self {
            // Parse the included bytes to create a language pack
            // Copy to an aligned vector to avoid alignment issues
            let bytes = FRENCH_LANGUAGE_DATA.to_vec();
            let archived = rkyv::access::<
                ArchivedConsolidatedLanguageDataWithCapacity,
                rkyv::rancor::Error,
            >(&bytes)
            .unwrap();
            let deserialized = rkyv::deserialize::<
                ConsolidatedLanguageDataWithCapacity,
                rkyv::rancor::Error,
            >(archived)
            .unwrap();

            let language_pack = Arc::new(LanguagePack::new(deserialized));

            let state = DeckState::new(language_pack, Language::French, Language::English);
            <Deck as weapon::PartialAppState>::finalize(state)
        }
    }

    #[test]
    fn test_fsrs() {
        use chrono::Utc;
        use rs_fsrs::{Card, FSRS, Rating};

        let fsrs = FSRS::default();
        let card = Card::new(Utc::now());

        let record_log = fsrs.repeat(card, Utc::now());
        for rating in Rating::iter() {
            let item = record_log[rating].to_owned();

            println!("{rating:#?}: {item:#?}");

            let record_log = fsrs.repeat(
                item.card,
                Utc::now().checked_add_days(Days::new(10)).unwrap(),
            );

            {
                // For any rating (Easy, Good, Hard, Again), you can compute the new card stats, which includes the next time the card should be reviewed
                let item = record_log[rating].to_owned();

                /* item = SchedulingInfo {
                    card: Card {
                        due: 2025-09-16T18:51:25.591443Z,
                        stability: 104.27451175337288,
                        difficulty: 2.24267983513529,
                        elapsed_days: 10,
                        scheduled_days: 104,
                        reps: 2,
                        lapses: 0,
                        state: Review,
                        last_review: 2025-06-04T18:51:25.591443Z,
                    },
                    review_log: ReviewLog {
                        rating: Easy,
                        elapsed_days: 10,
                        scheduled_days: 15,
                        state: Review,
                        reviewed_date: 2025-06-04T18:51:25.591443Z,
                    },
                } */
                println!("{rating:#?}+{rating:#?}: {item:#?}");
            }
        }
    }

    #[test]
    fn test_card_accumulated_surprise_after_one_easy_review() {
        use chrono::Utc;
        use rs_fsrs::{Card, FSRS, Rating};

        let fsrs = FSRS::default();
        let card = Card::new(Utc::now());

        // Do one easy review
        let record_log = fsrs.repeat(card, Utc::now());
        let after_easy = record_log[&Rating::Easy].to_owned();

        // Easy review should increase positive surprise
        assert!(
            after_easy.card.accumulated_positive_surprise > 0.0,
            "Accumulated positive surprise {} should be greater than 0 after easy review",
            after_easy.card.accumulated_positive_surprise
        );

        // Negative surprise should remain at 0 for easy review
        assert_eq!(
            after_easy.card.accumulated_negative_surprise, 0.0,
            "Accumulated negative surprise should be 0 after easy review"
        );

        println!(
            "✓ After one easy review - Positive surprise: {}, Negative surprise: {}",
            after_easy.card.accumulated_positive_surprise,
            after_easy.card.accumulated_negative_surprise
        );
    }

    #[test]
    fn test_card_accumulated_surprise_after_one_again_review() {
        use chrono::Utc;
        use rs_fsrs::{Card, FSRS, Rating};

        let fsrs = FSRS::default();
        let card = Card::new(Utc::now());

        // Do one "again" review (failed on first attempt)
        let record_log = fsrs.repeat(card, Utc::now());
        let after_again = record_log[&Rating::Again].to_owned();

        // Failed review should only have negative surprise
        assert_eq!(
            after_again.card.accumulated_positive_surprise, 0.0,
            "Positive surprise should be 0 after initial again review"
        );

        assert!(
            after_again.card.accumulated_negative_surprise > 0.0,
            "Negative surprise {} should be greater than 0 after again review",
            after_again.card.accumulated_negative_surprise
        );

        println!(
            "✓ After one again review - Positive surprise: {}, Negative surprise: {}",
            after_again.card.accumulated_positive_surprise,
            after_again.card.accumulated_negative_surprise
        );
        println!("  Lapses: {}", after_again.card.lapses);
    }

    #[test]
    fn test_card_accumulated_surprise_after_two_good_reviews() {
        use chrono::{Days, Utc};
        use rs_fsrs::{Card, FSRS, Rating};

        let fsrs = FSRS::default();
        let mut card = Card::new(Utc::now());

        // Do first good review
        let record_log = fsrs.repeat(card, Utc::now());
        card = record_log[&Rating::Good].card.clone();
        let pos_surprise_first = card.accumulated_positive_surprise;
        let neg_surprise_first = card.accumulated_negative_surprise;

        // Do second good review after 2 weeks
        let review_time = Utc::now().checked_add_days(Days::new(14)).unwrap();
        let record_log = fsrs.repeat(card, review_time);
        card = record_log[&Rating::Good].card.clone();
        let pos_surprise_second = card.accumulated_positive_surprise;
        let neg_surprise_second = card.accumulated_negative_surprise;

        println!("✓ Accumulated surprise progression with two good reviews:");
        println!(
            "  After 1st good - Positive: {pos_surprise_first}, Negative: {neg_surprise_first}"
        );
        println!(
            "  After 2nd good - Positive: {pos_surprise_second}, Negative: {neg_surprise_second}"
        );
        println!(
            "  Positive change: {}",
            pos_surprise_second - pos_surprise_first
        );
        println!(
            "  Negative change: {}",
            neg_surprise_second - neg_surprise_first
        );
        println!("  Reps: {}, Lapses: {}", card.reps, card.lapses);

        // Good reviews typically shouldn't generate much surprise in either direction
        // But the exact behavior depends on FSRS implementation
        println!("  (Good reviews are neutral, surprise accumulation depends on expectations)");
    }

    #[test]
    fn test_card_accumulated_surprise_after_one_easy_and_three_good_reviews() {
        use chrono::{Days, Utc};
        use rs_fsrs::{Card, FSRS, Rating};

        let fsrs = FSRS::default();
        let mut card = Card::new(Utc::now());

        // Do one easy review
        let record_log = fsrs.repeat(card, Utc::now());
        card = record_log[&Rating::Easy].card.clone();
        let pos_surprise_after_easy = card.accumulated_positive_surprise;
        let neg_surprise_after_easy = card.accumulated_negative_surprise;

        // Do three good reviews
        for i in 1..=3 {
            let review_time = Utc::now().checked_add_days(Days::new(i * 14)).unwrap();
            let record_log = fsrs.repeat(card, review_time);
            card = record_log[&Rating::Good].card.clone();
        }

        // Check accumulated surprise after mixed reviews
        println!("✓ Accumulated surprise after 1 easy + 3 good reviews:");
        println!(
            "  Positive: {} (started at {})",
            card.accumulated_positive_surprise, pos_surprise_after_easy
        );
        println!(
            "  Negative: {} (started at {})",
            card.accumulated_negative_surprise, neg_surprise_after_easy
        );
        println!("  Reps: {}, Lapses: {}", card.reps, card.lapses);

        // Easy review should have added positive surprise, good reviews might add less
        assert!(
            card.accumulated_positive_surprise >= pos_surprise_after_easy,
            "Positive surprise should not decrease with successful reviews"
        );
    }

    #[test]
    fn test_card_accumulated_surprise_after_one_easy_and_one_again_review() {
        use chrono::{Days, Utc};
        use rs_fsrs::{Card, FSRS, Rating};

        let fsrs = FSRS::default();
        let mut card = Card::new(Utc::now());

        // Do one easy review
        let record_log = fsrs.repeat(card, Utc::now());
        card = record_log[&Rating::Easy].card.clone();
        let pos_surprise_after_easy = card.accumulated_positive_surprise;
        let neg_surprise_after_easy = card.accumulated_negative_surprise;

        // Do one "again" review (failed review)
        let review_time = Utc::now().checked_add_days(Days::new(14)).unwrap();
        let record_log = fsrs.repeat(card, review_time);
        card = record_log[&Rating::Again].card.clone();

        // Check that negative surprise increased after the "again" review
        assert!(
            card.accumulated_negative_surprise > neg_surprise_after_easy,
            "Negative surprise {} should increase from {} after an 'again' review",
            card.accumulated_negative_surprise,
            neg_surprise_after_easy
        );

        println!("✓ Accumulated surprise after 1 easy + 1 again review:");
        println!(
            "  Positive: {} (was {} after easy)",
            card.accumulated_positive_surprise, pos_surprise_after_easy
        );
        println!(
            "  Negative: {} (was {} after easy)",
            card.accumulated_negative_surprise, neg_surprise_after_easy
        );
        println!("  Lapses: {}", card.lapses);
    }

    #[test]
    fn test_default_deck_creation() {
        use crate::Deck;

        // Test that we can create a default Deck
        let _deck = Deck::default();

        println!("✓ Default Deck created successfully");
    }

    #[test]
    fn test_default_deck_can_add_cards() {
        use crate::Deck;
        use weapon::AppState;

        let mut deck = Deck::default();

        // Test that we can add cards to the default deck
        if let Some(event) = deck.add_next_unknown_cards(None, 1, Vec::new()) {
            let ts = weapon::data_model::Timestamped {
                timestamp: chrono::Utc::now(),
                within_device_events_index: 0,
                event,
            };
            deck = deck.apply_event(&ts);

            // If language pack has data, we should have added a card
            if !deck.context.language_pack.word_frequencies.is_empty() {
                assert!(!deck.cards.is_empty());
                println!("✓ Successfully added card to default deck");
            } else {
                println!("✓ Language pack is empty, no cards to add (expected)");
            }
        } else {
            println!("✓ No cards available to add (empty language pack)");
        }
    }

    #[test]
    fn test_add_card_limits_scale_with_deck_size() {
        use crate::Deck;
        use weapon::AppState;
        use weapon::data_model::Timestamped;

        let mut deck = Deck::default();

        let assert_limits = |deck: &Deck| {
            let options = deck.add_card_options(Vec::new());
            let expected_max = if deck.num_cards() < 5 {
                1
            } else if deck.num_cards() < 11 {
                2
            } else {
                5
            } as u32;

            assert!(options.smart_add <= expected_max);
            assert!(
                options
                    .manual_add
                    .iter()
                    .all(|(count, _)| *count <= expected_max)
            );
        };

        assert_limits(&deck);

        while deck.num_cards() < 12 {
            let Some(event) = deck.add_next_unknown_cards(None, 5, Vec::new()) else {
                break;
            };

            let timestamped = Timestamped {
                timestamp: chrono::Utc::now(),
                within_device_events_index: 0,
                event,
            };

            let previous_cards = deck.num_cards();
            deck = deck.apply_event(&timestamped);
            assert!(
                deck.num_cards() <= previous_cards + 5,
                "deck should not grow by more than the requested amount"
            );

            assert_limits(&deck);
        }
    }
}
