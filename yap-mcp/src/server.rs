//! The yap MCP server: state management and tool implementations.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Context as _;
use chrono::Utc;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::wrapper::Parameters,
    model::{
        CallToolResult, ContentBlock, ListResourcesResult, ListToolsResult, PaginatedRequestParams,
        ReadResourceRequestParams, ReadResourceResult, Resource, ResourceContents,
        ServerCapabilities, ServerInfo,
    },
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use language_utils::SentenceSpur;
use language_utils::{
    Course, Gram, Language, SpurGram, TaggedGram, TtsProvider, TtsRequest, autograde,
    dictionary_entry_slug,
    language_pack::LanguagePack,
    text_cleanup::{cleanup_sentence, find_closest_match},
};
use weapon::data_model::EventStore;
use yap_frontend_rs::{
    CardContext, CardIndicator, CardSummary, Challenge, Context, Deck, DeckEvent, LanguageEvent,
    LanguageEventContent, Rating, TranslateComprehensibleSentence, autograde_translation,
    dictionary::DictionaryWord, translation_is_perfect,
};

use knowledge_graph::{KnowledgeEvent, KnowledgeGraph};

use crate::deck::{PackCache, build_deck, build_knowledge, detect_course, insert_rows, new_store};
use crate::output::*;
use crate::sync::{
    DEVICE_ID, KNOWLEDGE_STREAM, REVIEWS_STREAM, SupabaseAuth, WRITTEN_STREAMS, fetch_events,
    find_user_id, upload_events,
};
use crate::verbatim::Verbatim;

/// A card as tool input and output: the deck's own indicator with grams and
/// patterns spelled out as strings.
pub type Card = CardIndicator<Gram<String>, String>;

/// How long fetched events stay fresh before a tool call triggers a re-fetch.
const REFRESH_INTERVAL: Duration = Duration::from_secs(20);

pub struct Config {
    pub supabase: SupabaseAuth,
    pub email: String,
    pub out_dir: PathBuf,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let email = std::env::var("YAP_USER_EMAIL")
            .context("YAP_USER_EMAIL env var required (the account to operate on)")?;
        let service_role_key = std::env::var("SUPABASE_SERVICE_ROLE_KEY")
            .context("SUPABASE_SERVICE_ROLE_KEY env var required (in the repo .env)")?;
        let url = std::env::var("SUPABASE_URL")
            .unwrap_or_else(|_| "https://eearwzqotpfoderpfrqx.supabase.co".to_string());
        let out_dir = std::env::var("YAP_OUT_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../out")));
        Ok(Config {
            supabase: SupabaseAuth::service_role(url, service_role_key),
            email,
            out_dir,
        })
    }
}

/// Whether a recorded review event reached the server or is still pending a
/// retry. Either way the event is recorded in the local working store;
/// `PendingSync` just means the server copy will catch up on a later call.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum SyncStatus {
    Synced,
    PendingSync,
}

/// Result of an idempotency-token lookup.
enum Replay {
    /// Seen before for this presentation — replay the cached response.
    Hit(CallToolResult),
    /// Seen before for a *different* presentation — reject (one nonce = one
    /// operation).
    IdentityConflict,
    /// Not seen — record normally.
    Miss,
}

/// How many recent idempotency tokens to remember. A review session is a
/// few dozen actions; this is a generous ceiling before the oldest are
/// dropped wholesale.
const IDEMPOTENCY_CACHE_MAX: usize = 256;

/// Returned when a nonce is reused for a different card/sentence — the real
/// widget never does this (each presentation mints its own nonce).
const IDEMPOTENCY_CONFLICT_MSG: &str = "that idempotency token was already used for a different card. Each token belongs \
     to one challenge, so the token from the current challenge works, or it can be left out";

pub struct YapState {
    pub(crate) supabase: SupabaseAuth,
    pub(crate) http: reqwest::Client,
    pub(crate) user_id: String,
    context: Context,
    store: EventStore<String, String>,
    last_fetched_id: Option<i64>,
    last_fetch: Instant,
    /// Set when an upload failed, to make the next sync fetch regardless of
    /// the throttle: a re-fetch echo-confirms any event whose ack we lost and
    /// advances `uploaded` past it, so we stop re-sending a committed
    /// index (which the unique constraint would reject).
    force_fetch: bool,
    deck: Option<Deck>,
    knowledge: Option<KnowledgeGraph>,
    /// Per written stream, how many of our device's events the server has
    /// confirmed received.
    uploaded: HashMap<&'static str, usize>,
    /// Responses already served for a given idempotency token, so a retried
    /// review (widget resubmit, or a call whose response was lost in transit
    /// after the event was recorded) replays the original outcome instead of
    /// recording a second event. The token is a per-presentation nonce; each
    /// entry is bound to the presentation's immutable identity (card /
    /// language / sentence), so a retry — even with an edited answer — replays
    /// that one review, while a nonce paired with a *different* presentation
    /// records the real request instead of replaying a stale one.
    idempotency_cache: HashMap<String, (String, CallToolResult)>,
}

impl YapState {
    /// stdio mode: resolve the account by email using the service role key.
    pub async fn load(config: Config) -> anyhow::Result<Self> {
        let http = reqwest::Client::new();
        log::info!("looking up user {}", config.email);
        let user_id = find_user_id(&http, &config.supabase, &config.email).await?;
        let packs = PackCache::new(config.out_dir);
        Self::for_user(config.supabase, user_id, &packs).await
    }

    /// Fetch a user's events, detect their course, and replay their deck.
    pub async fn for_user(
        supabase: SupabaseAuth,
        user_id: String,
        packs: &PackCache,
    ) -> anyhow::Result<Self> {
        let http = reqwest::Client::new();

        log::info!("fetching events for {user_id}");
        let (rows, last_fetched_id) = fetch_events(&http, &supabase, &user_id, None).await?;
        log::info!("fetched {} events", rows.len());

        let mut store = new_store();
        insert_rows(&mut store, rows);

        let (course, study_goal) = detect_course(&store)?;
        log::info!(
            "detected course: {} for {} speakers",
            course.target_language,
            course.native_language
        );

        let language_pack = packs.get(&course).await?;
        let timezone = *chrono::Local::now().offset();
        let context = Context {
            study_goal,
            language_pack,
            course,
            timezone,
        };

        let mut state = YapState {
            supabase,
            http,
            user_id,
            context,
            store,
            last_fetched_id,
            last_fetch: Instant::now(),
            force_fetch: false,
            deck: None,
            knowledge: None,
            uploaded: HashMap::new(),
            idempotency_cache: HashMap::new(),
        };
        // Everything fetched is by definition already on the server.
        for stream in WRITTEN_STREAMS {
            let len = state.my_events_len(stream);
            state.uploaded.insert(stream, len);
        }
        let deck = state.deck();
        log::info!(
            "deck ready: {} cards, {} reviews",
            deck.num_cards_added(),
            deck.stats().total_reviews
        );
        Ok(state)
    }

    /// Swap in the bearer token from the current request (remote mode: user
    /// access tokens rotate on refresh).
    pub fn set_bearer(&mut self, bearer: String) {
        self.supabase.bearer = bearer;
    }

    fn pack(&self) -> &LanguagePack {
        &self.context.language_pack
    }

    fn deck(&mut self) -> &Deck {
        if self.deck.is_none() {
            self.deck = Some(build_deck(&self.store, &self.context));
        }
        self.deck.as_ref().expect("just built")
    }

    pub(crate) fn knowledge(&mut self) -> &KnowledgeGraph {
        if self.knowledge.is_none() {
            self.knowledge = Some(build_knowledge(&self.store));
        }
        self.knowledge.as_ref().expect("just built")
    }

    /// How many of a stream's events exist under our device id.
    fn my_events_len(&self, stream: &str) -> usize {
        self.store
            .get_raw(stream.to_string())
            .and_then(|store| {
                store
                    .num_events_per_device()
                    .get(&DEVICE_ID.to_string())
                    .copied()
            })
            .unwrap_or(0)
    }

    /// Pull down events written by other devices since our last fetch, and
    /// flush any locally-recorded reviews the server hasn't confirmed yet.
    /// Called at the top of every tool handler (including read-only ones),
    /// since a pending upload should reach Supabase on the user's next
    /// action rather than waiting for their next write.
    pub(crate) async fn sync(&mut self) -> anyhow::Result<()> {
        if self.force_fetch || self.last_fetch.elapsed() >= REFRESH_INTERVAL {
            let (new_rows, max_id) = fetch_events(
                &self.http,
                &self.supabase,
                &self.user_id,
                self.last_fetched_id,
            )
            .await?;
            self.last_fetch = Instant::now();
            self.force_fetch = false;
            self.last_fetched_id = max_id.or(self.last_fetched_id);

            // Echoes of our own uploads confirm the server has them (the store
            // itself dedups the events). This also advances `uploaded` past
            // any event whose upload ack we lost, so the flush below won't
            // pointlessly re-send an index the server already has.
            for row in &new_rows {
                if row.device_id != DEVICE_ID {
                    continue;
                }
                if let Some(count) = self.uploaded.get_mut(row.stream_id.as_str()) {
                    *count = (*count).max(row.event.within_device_events_index + 1);
                }
            }
            if insert_rows(&mut self.store, new_rows) > 0 {
                self.deck = None;
                self.knowledge = None;
            }
        }

        // Retry any locally-recorded reviews the server hasn't confirmed yet,
        // so a review whose upload failed still reaches Supabase on the user's
        // next action (even a read-only one) rather than waiting for their
        // next write.
        self.flush_pending_uploads().await;
        Ok(())
    }

    /// Record a review event locally and attempt to sync it.
    ///
    /// The local event store is the working source of truth (this mirrors the
    /// app's offline-first model), so recording always succeeds and the
    /// upload is best-effort: a failure leaves the event pending for the next
    /// `flush_pending_uploads` to retry rather than surfacing an error. That
    /// matters because the review widgets let the user retry on error — if
    /// this returned `Err`, a retry would record a *second* event (a fresh
    /// per-device index), double-reviewing the card. Returning
    /// [`SyncStatus`] instead means the widget always sees success and never
    /// retries; the pending event self-heals on the next call.
    ///
    /// Caveat: this store is in-memory (rebuilt from Supabase on start), so a
    /// pending event is lost if the process exits or the per-user state is
    /// evicted before a later call flushes it — a rare, low-harm edge (the
    /// card simply comes due again) that the old error-returning path shared,
    /// minus the double-record.
    async fn append_event(&mut self, content: LanguageEventContent) -> SyncStatus {
        let deck_event = DeckEvent::Language(LanguageEvent {
            target_language: self.context.course.target_language,
            native_language: self.context.course.native_language,
            content,
        });
        self.store.add_raw_event(
            REVIEWS_STREAM.to_string(),
            DEVICE_ID.to_string(),
            deck_event,
            None,
            self.context.timezone,
        );
        self.deck = None;
        self.flush_pending_uploads().await
    }

    /// Record knowledge-graph events (already validated against the graph at
    /// `at`) and attempt to sync them, as one upload. Same offline-first
    /// contract as [`Self::append_event`].
    pub(crate) async fn append_knowledge(
        &mut self,
        events: Vec<KnowledgeEvent>,
        at: chrono::DateTime<Utc>,
    ) -> SyncStatus {
        for event in events {
            self.store.add_raw_event_at(
                KNOWLEDGE_STREAM.to_string(),
                DEVICE_ID.to_string(),
                event,
                None,
                at,
                self.context.timezone,
            );
        }
        self.knowledge = None;
        self.flush_pending_uploads().await
    }

    /// Upload every locally-recorded event the server hasn't confirmed yet,
    /// per stream and in order so the per-device index sequence never has
    /// gaps. Best-effort: on failure the events stay pending (`uploaded`
    /// isn't advanced) and the next call retries them. A committed index that
    /// we think is still pending (a lost upload ack) would be rejected by the
    /// unique constraint on re-send, so a failure sets `force_fetch`: the next
    /// sync fetches, echo-confirms that index, and advances past it before
    /// we'd re-send.
    async fn flush_pending_uploads(&mut self) -> SyncStatus {
        let mut status = SyncStatus::Synced;
        for stream in WRITTEN_STREAMS {
            let pending = self
                .store
                .get_raw(stream.to_string())
                .expect("written streams registered in new_store")
                .jsons(&DEVICE_ID.to_string(), self.uploaded[stream]);
            if pending.is_empty() {
                continue;
            }
            match upload_events(
                &self.http,
                &self.supabase,
                &self.user_id,
                stream,
                DEVICE_ID,
                &pending,
            )
            .await
            {
                Ok(()) => {
                    *self.uploaded.get_mut(stream).expect("seeded in for_user") += pending.len();
                }
                Err(e) => {
                    log::warn!(
                        "upload failed, {} {stream} event(s) recorded locally and pending retry: {e:#}",
                        pending.len()
                    );
                    // Force the next sync to fetch: if this failure was a lost
                    // ack (the server actually has the event), the echo-confirm
                    // there trims it from pending so we stop hitting the unique
                    // constraint on re-send.
                    self.force_fetch = true;
                    status = SyncStatus::PendingSync;
                }
            }
        }
        status
    }

    /// Look up a prior response for this tool's idempotency token. A nonce
    /// identifies exactly one presentation, so a hit must also match that
    /// presentation's identity; a nonce seen with a *different* identity is a
    /// conflict (a misbehaving client — the real widget always pairs a nonce
    /// with its own challenge). Tokens are per-account (this state) and
    /// per-server-lifetime — the retry window we care about is a live widget
    /// (seconds to minutes), so a token lost to restart or the 256-entry
    /// bound just misses and records normally.
    fn replay_idempotent(&self, tool: &str, token: Option<&str>, identity: &str) -> Replay {
        let Some(token) = token else {
            return Replay::Miss;
        };
        match self.idempotency_cache.get(&idempotency_key(tool, token)) {
            Some((id, result)) if id == identity => Replay::Hit(result.clone()),
            Some(_) => Replay::IdentityConflict,
            None => Replay::Miss,
        }
    }

    /// Remember a successful response under its idempotency token and
    /// presentation identity so a later retry replays it. Errors aren't
    /// cached — a retry should be free to succeed. Keyed by tool so a token
    /// can't cross tools.
    fn remember_idempotent(
        &mut self,
        tool: &str,
        token: Option<&str>,
        identity: &str,
        result: &CallToolResult,
    ) {
        let Some(token) = token else { return };
        if result.is_error.unwrap_or(false) {
            return;
        }
        if self.idempotency_cache.len() >= IDEMPOTENCY_CACHE_MAX {
            self.idempotency_cache.clear();
        }
        self.idempotency_cache.insert(
            idempotency_key(tool, token),
            (identity.to_string(), result.clone()),
        );
    }

    /// The course's target language, serialized the way tool outputs carry it.
    fn target_language_value(&self) -> serde_json::Value {
        serde_json::to_value(self.context.course.target_language).expect("Language serializes")
    }

    /// Reject references whose language doesn't match the course this server
    /// is connected to.
    fn check_language(&self, language: &str) -> Result<(), String> {
        let expected = self.context.course.target_language;
        let parsed: Option<Language> =
            serde_json::from_value(serde_json::Value::String(language.to_string())).ok();
        if parsed != Some(expected) {
            return Err(format!(
                "this server is connected to a {expected} course, but got language '{language}'. \
                 The language field returned by search_dictionary and get_due_cards is the one \
                 that matches."
            ));
        }
        Ok(())
    }

    /// Parse a gram (the word/lemma/part-of-speech token sequence that uniquely
    /// and sense number identify a dictionary entry) from tool input, requiring it to name a
    /// gram that actually exists in this course; missing senses use the most frequent sense.
    fn resolve_gram(
        &self,
        gram: &TaggedGram<Gram<String>>,
    ) -> Result<TaggedGram<SpurGram>, String> {
        match self.pack().resolve_entry(gram) {
            Some(interned) => Ok(interned),
            None => Err(format!(
                "no dictionary entry in this course matches that gram for '{}'. Entries are \
                 keyed by the full word/lemma/part-of-speech sequence, so a gram only matches \
                 when every token does (a missing or stale sense number just selects the most \
                 frequent sense). search_dictionary returns the exact grams of real entries.",
                gram.to_display_string(self.context.course.target_language)
            )),
        }
    }

    /// Resolve a written-gram card plus an optional explicit sentence into
    /// the full translation-challenge payload. Explicit sentences are
    /// verified against the corpus (a model- or widget-supplied sentence is
    /// only accepted after spacing normalization); omitted ones use the
    /// app's own selection.
    fn translation_challenge_payload(
        &mut self,
        card: &Card,
        sentence: Option<&str>,
    ) -> Result<(SentenceSpur, TranslateComprehensibleSentence), String> {
        let CardIndicator::WrittenGram { gram } = card else {
            return Err(
                "translation challenges are built for written gram cards; listening and \
                 pronunciation cards have their own exercise in present_card"
                    .to_string(),
            );
        };
        let display = gram.to_display_string(self.context.course.target_language);
        let Some(interned) = self.pack().resolve_entry(gram) else {
            return Err("that card's gram isn't a dictionary entry in this course".to_string());
        };
        let spur = match sentence {
            Some(text) => {
                let text = cleanup_sentence(text.to_string(), self.context.course.target_language);
                let pack = self.pack();
                let Some(spur) = pack.sentence_key(&text) else {
                    return Err(
                        "that sentence isn't in yap's corpus. Grading needs a real corpus \
                         sentence (for its word links and reference translation), so one copied \
                         from get_sentences works, or leaving it out lets the server pick"
                            .to_string(),
                    );
                };
                let contains = pack
                    .sentences_containing(&interned)
                    .any(|candidate| candidate == spur);
                if !contains {
                    return Err(format!(
                        "that sentence doesn't contain «{display}». get_sentences for this word \
                         lists ones that do, or leaving it out lets the server pick"
                    ));
                }
                spur
            }
            None => match self.deck().pick_translation_sentence(&interned) {
                Some(spur) => spur,
                None => {
                    return Err(format!(
                        "no comprehensible sentence contains «{display}» yet: every sentence \
                         with it still has other words the user hasn't learned. present_card \
                         works well for it in the meantime."
                    ));
                }
            },
        };
        match self
            .deck()
            .translation_challenge_for_sentence(interned, spur)
        {
            Some(challenge) => Ok((spur, challenge)),
            None => Err("could not build a translation challenge for that sentence".to_string()),
        }
    }

    /// Human-readable provenance for a sentence.
    fn sentence_sources(&self, sentence: &SentenceSpur) -> Vec<String> {
        let pack = self.pack();
        let source = pack.source_of(*sentence);
        let mut out = Vec::new();
        if source.from_anki {
            out.push("Anki deck".to_string());
        }
        if source.from_tatoeba {
            out.push("Tatoeba".to_string());
        }
        if source.from_manual {
            out.push("manually curated".to_string());
        }
        if source.from_song {
            out.push("song lyrics".to_string());
        }
        for movie in source
            .movie_ids()
            .filter_map(|id| pack.sentences.movies.get(id))
        {
            match movie.year.as_ref() {
                Some(year) => out.push(format!("movie: {} ({year})", movie.title)),
                None => out.push(format!("movie: {}", movie.title)),
            }
        }
        out
    }

    /// Sample sentences containing an interned gram: up to `count` composed
    /// only of words the user already knows, and up to `count` more from the
    /// whole corpus, each rendered with translations and attribution.
    fn sample_sentences(
        &mut self,
        interned: TaggedGram<SpurGram>,
        count: usize,
    ) -> SampledSentences {
        let comprehensible_pool = {
            let deck = self.deck();
            let comprehensible = deck.comprehensible_written_grams(false).rank_bitset();
            deck.context()
                .language_pack
                .comprehensible_sentences(Some(&interned), |rank| comprehensible.contains(rank))
        };

        use rand::seq::IndexedRandom as _;
        let chosen_comprehensible: Vec<SentenceSpur> = comprehensible_pool
            .choose_multiple(&mut rand::rng(), count)
            .copied()
            .collect();

        let pack = self.pack();
        let all_containing: Vec<_> = pack.sentences_containing(&interned).collect();
        let other_pool: Vec<SentenceSpur> = all_containing
            .iter()
            .copied()
            .filter(|sentence| !chosen_comprehensible.contains(sentence))
            .collect();
        let chosen_other: Vec<SentenceSpur> = other_pool
            .choose_multiple(&mut rand::rng(), count)
            .copied()
            .collect();

        let render = |sentence: &SentenceSpur| SentenceOut {
            text: pack.sentence_text(*sentence),
            translations: pack
                .translations_of(*sentence)
                .iter()
                .map(|t| pack.sentences.translation_rodeo.resolve_string(t))
                .collect(),
            sources: self.sentence_sources(sentence),
        };
        SampledSentences {
            comprehensible: chosen_comprehensible.iter().map(render).collect(),
            other: chosen_other.iter().map(render).collect(),
            total_comprehensible: comprehensible_pool.len(),
            total_containing: all_containing.len(),
        }
    }
}

struct SampledSentences {
    comprehensible: Vec<SentenceOut>,
    other: Vec<SentenceOut>,
    total_comprehensible: usize,
    total_containing: usize,
}

/// Success result carrying the value both as readable text and as MCP
/// structured content.
fn ok_json_structured(value: serde_json::Value) -> CallToolResult {
    let mut result = CallToolResult::success(vec![ContentBlock::text(
        serde_json::to_string_pretty(&value).expect("json serializes"),
    )]);
    result.structured_content = Some(value);
    result
}

/// Success result from a typed output (see [`crate::output`]): the same
/// value becomes both the readable text and the structured content, so the
/// payload always matches the tool's declared outputSchema.
pub(crate) fn ok_typed<T: Serialize>(value: &T) -> CallToolResult {
    ok_json_structured(serde_json::to_value(value).expect("output serializes"))
}

/// The public dictionary, canonical regardless of where the server runs.
const DICTIONARY_BASE_URL: &str = "https://yap.town/d";

fn dictionary_sense_gram(pack: &LanguagePack, index: usize) -> TaggedGram<Gram<String>> {
    let (spur, _) = pack
        .sense_frequencies
        .get_index(index)
        .expect("dictionary senses come from this pack's frequency index");
    spur.map(|gram| pack.resolve_gram(&gram))
}

fn entry_title(entry: &DictionaryWord) -> String {
    let gloss = entry.gloss();
    if gloss.is_empty() {
        entry.display_text()
    } else {
        format!("{} — {}", entry.display_text(), gloss)
    }
}

/// Stable id for the search/fetch pair, e.g. "french-to-english:42".
fn entry_id(course: &Course, entry: &DictionaryWord) -> String {
    format!("{}:{}", course.dictionary_slug(), entry.frequency_index())
}

/// Best-effort public URL of the entry's dictionary page. Entries whose
/// display text collides with another's get a numeric suffix at site build
/// time that we can't reproduce here, so rare homographs may 404.
fn entry_url(course: &Course, display_text: &str) -> String {
    format!(
        "{DICTIONARY_BASE_URL}/{}/{}/",
        course.dictionary_slug(),
        dictionary_entry_slug(display_text)
    )
}

pub(crate) fn error(message: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(message.into())])
}

/// Resolved audio, cached process-wide: clips aren't user-specific, and the
/// widget re-requests the same sentences often. Bounded by wholesale clears.
#[derive(Clone)]
struct CachedAudio {
    bytes: Vec<u8>,
    voice_actor: Option<yap_frontend_rs::VoiceActorInfo>,
}

static AUDIO_CACHE: std::sync::Mutex<Option<std::collections::HashMap<String, CachedAudio>>> =
    std::sync::Mutex::new(None);
const AUDIO_CACHE_MAX: usize = 256;

fn audio_cache_get(key: &str) -> Option<CachedAudio> {
    AUDIO_CACHE
        .lock()
        .expect("audio cache poisoned")
        .as_ref()
        .and_then(|cache| cache.get(key).cloned())
}

fn audio_cache_put(key: &str, audio: CachedAudio) {
    let mut guard = AUDIO_CACHE.lock().expect("audio cache poisoned");
    let cache = guard.get_or_insert_with(Default::default);
    if cache.len() >= AUDIO_CACHE_MAX {
        cache.clear();
    }
    cache.insert(key.to_string(), audio);
}

fn audio_result(audio: CachedAudio) -> CallToolResult {
    use base64::Engine as _;
    let mime_type = yap_frontend_rs::audio_mime_type(&audio.bytes);
    let source = if audio.voice_actor.is_some() {
        "human_recording"
    } else {
        "tts"
    };
    let out = GetAudioOut {
        audio_base64: base64::engine::general_purpose::STANDARD.encode(&audio.bytes),
        mime_type: mime_type.to_string(),
        voice_actor: audio.voice_actor,
        source: source.to_string(),
    };
    // The widget reads structuredContent; the text block is only narration,
    // so a summary beats duplicating the whole base64 payload.
    let mut result = CallToolResult::success(vec![ContentBlock::text(format!(
        "{} audio, {} bytes ({source}) — in structuredContent for the widget",
        out.mime_type,
        audio.bytes.len(),
    ))]);
    result.structured_content = Some(serde_json::to_value(&out).expect("output serializes"));
    result
}

/// The card shape shared by get_due_cards and unlock_cards: everything the
/// LLM needs to quiz the user and pass the card back to log_review.
fn card_summary_out(language: Language, summary: &CardSummary, locked: bool) -> CardSummaryOut {
    let indicator = summary.card_indicator();
    CardSummaryOut {
        language,
        card: serde_json::to_value(&indicator).expect("card serializes"),
        text: summary.card_text(),
        subtitle: summary.card_subtitle(),
        kind: card_kind(&indicator).to_string(),
        fsrs_state: summary.state().to_string(),
        due: chrono::DateTime::from_timestamp_millis(summary.due_timestamp_ms() as i64)
            .map(|d| d.to_rfc3339()),
        locked,
    }
}

pub(crate) fn card_kind(card: &Card) -> &'static str {
    match card {
        CardIndicator::WrittenGram { .. } => "written",
        CardIndicator::ListeningGram { .. } => "listening",
        CardIndicator::LetterPronunciation { .. } => "pronunciation",
    }
}

/// A fresh random nonce identifying one card/challenge presentation. The
/// widget echoes it back as the review's idempotency token, so a retried
/// submit — including one after the widget iframe reloads and replays the
/// same tool result — records the review exactly once.
fn presentation_nonce() -> String {
    format!("{:032x}", rand::random::<u128>())
}

/// Namespace an idempotency token by the tool that issued it, so the same
/// token can never replay another tool's (or payload's) cached response.
fn idempotency_key(tool: &str, token: &str) -> String {
    format!("{tool}:{token}")
}

fn parse_rating(rating: &str) -> Result<Rating, String> {
    match rating {
        "again" => Ok(Rating::Again),
        "hard" => Ok(Rating::Hard),
        "good" => Ok(Rating::Good),
        "easy" => Ok(Rating::Easy),
        "remembered" => Ok(Rating::Remembered),
        other => Err(format!(
            "invalid rating '{other}': expected one of again, hard, good, easy, remembered"
        )),
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SearchDictionaryParams {
    /// Text to search for: a target-language word/phrase or a native-language meaning.
    /// Accent-insensitive.
    query: String,
    /// Maximum number of results (default 20).
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct AddCardsParams {
    /// The target language the grams belong to, e.g. "French". The server is
    /// connected to one course, so this is that course's language; it's
    /// there as a check that the grams came from the right dictionary.
    language: String,
    /// The grams (words/phrases) to add, each the exact gram JSON returned by
    /// search_dictionary: a sequence of tokens with word, lemma, and part of
    /// speech, wrapped as {gram, sense}.
    grams: Vec<Verbatim<TaggedGram<Gram<String>>>>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GetDueCardsParams {
    /// Maximum number of due cards to return (default 10).
    #[serde(default)]
    limit: Option<usize>,
    /// Also list due cards that are set aside in lockup, marked locked
    /// (default false). Reviewing a locked card unlocks it.
    #[serde(default)]
    include_locked: bool,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ReviewInput {
    /// Exact card identity from the presented meaning's cards (or a singleton listening/pronunciation card).
    card: Verbatim<Card>,
    /// again, hard, good, easy, or remembered.
    rating: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct LogReviewParams {
    /// The target language of the cards, e.g. "French".
    language: String,
    /// Every card the user was actually tested on, with its rating. A
    /// written front can carry several meanings, each its own card, so this
    /// lists one entry per meaning row the quiz covered.
    reviews: Vec<ReviewInput>,
    /// Optional idempotency key for one review. Supplied by the review
    /// widget so that a retried submit (or a call whose response was lost
    /// after the event was recorded) records the review only once.
    /// Conversational reviews can leave it out.
    #[serde(default)]
    idempotency_token: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GetSentencesParams {
    /// The target language of the gram, e.g. "French".
    language: String,
    /// The exact gram JSON returned by search_dictionary or get_due_cards.
    gram: Verbatim<TaggedGram<Gram<String>>>,
    /// How many sentences of each kind to return (default 5, max 20).
    #[serde(default)]
    count: Option<usize>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct PresentCardParams {
    /// The target language of the card, e.g. "French".
    language: String,
    /// The card object exactly as returned by get_due_cards or unlock_cards.
    card: Verbatim<Card>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct PresentTranslationParams {
    /// The target language of the card, e.g. "French".
    language: String,
    /// The written-gram card being reviewed, exactly as returned by
    /// get_due_cards or unlock_cards.
    card: Verbatim<Card>,
    /// Optional: the exact text of a corpus sentence containing this card's
    /// gram, chosen from get_sentences. comprehensible_sentences fit best,
    /// since the grade reviews every word in the sentence, not just this
    /// card's, and the user should already know the others. Left out, the
    /// server picks the least-reviewed comprehensible sentence, as the yap
    /// app does. Only real corpus sentences work here, because grading needs
    /// each word's dictionary entry and the sentence's reference translation.
    #[serde(default)]
    sentence: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GradeTranslationParams {
    /// The target language of the challenge, e.g. "French".
    language: String,
    /// The card object exactly as given in the widget data.
    card: Verbatim<Card>,
    /// The challenge sentence exactly as given in the widget data.
    sentence: String,
    /// The user's typed translation.
    submission: String,
    /// Optional idempotency key for one submission, supplied by the review
    /// widget so a retried submit (or a lost response after the review was
    /// recorded) grades and records only once — and doesn't re-run the LLM.
    #[serde(default)]
    idempotency_token: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct GetAudioParams {
    /// The TTS request exactly as provided in widget data: text, language,
    /// and optional speed/is_ssml/instructions.
    request: Verbatim<TtsRequest>,
    /// The TTS provider to synthesize with when no human recording exists,
    /// exactly as provided in widget data (e.g. "ElevenLabs").
    provider: TtsProvider,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SearchParams {
    /// What to look up: a word or phrase in the language being learned, or a
    /// native-language meaning. Accent-insensitive.
    query: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct FetchParams {
    /// A result id exactly as returned by the search tool, e.g.
    /// "french-to-english:42".
    id: String,
}

/// Where a tool call's deck state comes from.
#[derive(Clone)]
enum StateSource {
    /// stdio mode: one account, resolved at startup.
    Single(Arc<tokio::sync::Mutex<YapState>>),
    /// Remote mode: per-user state, resolved from the request's bearer token.
    PerUser(Arc<crate::remote::RemoteApp>),
}

/// Base URI of the review widget, the interactive card UI hosts render
/// inline (MCP Apps extension). The URI hosts serve carries a content hash
/// suffix ([`Widget::uri`]) so a rebuilt widget gets a fresh URI — hosts
/// (claude.ai) cache `ui://` resources by URI and would otherwise keep
/// serving a stale iframe after every widget change.
pub const WIDGET_URI_BASE: &str = "ui://yap/review.html";
const WIDGET_MIME: &str = "text/html;profile=mcp-app";

/// The built review widget together with its content-addressed URI.
pub struct Widget {
    pub html: String,
    /// `ui://yap/review.html@<12 hex of sha256(html)>`.
    pub uri: String,
}

/// Content-addressed URI for a widget build: base + first 12 hex of the
/// HTML's sha256. Distinct HTML ⇒ distinct URI ⇒ hosts refetch it.
fn versioned_widget_uri(html: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(html.as_bytes());
    let hex: String = digest.iter().take(6).map(|b| format!("{b:02x}")).collect();
    format!("{WIDGET_URI_BASE}@{hex}")
}

/// Load the built review widget. Absent (not built) is fine: widget tools
/// degrade to text and the resource simply isn't listed.
pub fn load_widget_html() -> Option<Arc<Widget>> {
    let path = std::env::var("YAP_WIDGET_HTML")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/widget/dist/index.html"
            ))
        });
    match std::fs::read_to_string(&path) {
        Ok(html) => {
            let uri = versioned_widget_uri(&html);
            log::info!(
                "review widget loaded from {} ({} bytes) as {uri}",
                path.display(),
                html.len()
            );
            Some(Arc::new(Widget { html, uri }))
        }
        Err(e) => {
            log::warn!(
                "review widget not available ({}: {e}) — widget tools degrade to text",
                path.display()
            );
            None
        }
    }
}

fn meta_from(value: serde_json::Value) -> rmcp::model::Meta {
    rmcp::model::Meta(
        value
            .as_object()
            .expect("meta literals are objects")
            .clone(),
    )
}

/// `_meta` for the widget resource: a fully closed CSP (the widget talks to
/// the world only through the host bridge) plus theming and the legacy
/// OpenAI aliases.
fn widget_resource_meta() -> rmcp::model::Meta {
    meta_from(json!({
        "ui": {
            "csp": { "connectDomains": [], "resourceDomains": [] },
            "prefersBorder": false,
        },
        "openai/widgetDescription": "An interactive yap flashcard: plays the audio, reveals the answer, and records the user's own grade.",
        "openai/widgetCSP": { "connect_domains": [], "resource_domains": [] },
    }))
}

/// Per-tool `_meta` when the widget is available: template linkage for
/// present_card, widget-only visibility for get_audio, and widget
/// callability for the tools the iframe invokes.
fn widget_tool_meta(tool_name: &str, widget_uri: &str) -> Option<rmcp::model::Meta> {
    match tool_name {
        "present_card" => Some(meta_from(json!({
            "ui": { "resourceUri": widget_uri },
            "openai/outputTemplate": widget_uri,
            "openai/toolInvocation/invoking": "Dealing a card…",
            "openai/toolInvocation/invoked": "Card ready",
        }))),
        "present_translation" => Some(meta_from(json!({
            "ui": { "resourceUri": widget_uri },
            "openai/outputTemplate": widget_uri,
            "openai/toolInvocation/invoking": "Dealing a sentence…",
            "openai/toolInvocation/invoked": "Challenge ready",
        }))),
        "grade_translation" => Some(meta_from(json!({
            "ui": { "visibility": ["app"] },
            "openai/widgetAccessible": true,
        }))),
        "get_audio" => Some(meta_from(json!({
            "ui": { "visibility": ["app"] },
            "openai/widgetAccessible": true,
        }))),
        "log_review" => Some(meta_from(json!({
            "openai/widgetAccessible": true,
        }))),
        _ => None,
    }
}

#[derive(Clone)]
pub struct YapMcp {
    source: StateSource,
    widget: Option<Arc<Widget>>,
}

impl YapMcp {
    pub fn new(state: YapState, widget: Option<Arc<Widget>>) -> Self {
        Self {
            source: StateSource::Single(Arc::new(tokio::sync::Mutex::new(state))),
            widget,
        }
    }

    pub fn new_remote(app: Arc<crate::remote::RemoteApp>) -> Self {
        Self {
            widget: app.widget.clone(),
            source: StateSource::PerUser(app),
        }
    }

    /// Resolve the deck state for this call. In remote mode the authenticated
    /// user travels in the propagated HTTP request parts.
    pub(crate) async fn state_slot(
        &self,
        ctx: &RequestContext<RoleServer>,
    ) -> Result<Arc<tokio::sync::Mutex<YapState>>, String> {
        match &self.source {
            StateSource::Single(state) => Ok(state.clone()),
            StateSource::PerUser(app) => {
                let user = ctx
                    .extensions
                    .get::<http::request::Parts>()
                    .and_then(|parts| parts.extensions.get::<crate::remote::AuthedUser>())
                    .ok_or("unauthenticated: no user on this request")?
                    .clone();
                app.state_for_user(&user)
                    .await
                    .map_err(|e| format!("failed to load your yap account: {e:#}"))
            }
        }
    }
}

#[tool_router]
impl YapMcp {
    #[tool(
        title = "Search the dictionary",
        output_schema = rmcp::handler::server::common::schema_for_type::<crate::output::SearchDictionaryOut>(),
        description = "Search the yap dictionary for words and phrases, in the target language or by native-language meaning. Results are grouped by word, with each sense listed separately, because yap schedules senses independently: \"bank\" the river edge and \"bank\" the institution are different things to learn. Each sense carries its exact gram (token sequence of word + lemma + part of speech, plus sense number), which is what the deck tools use to find the entry, its sentences and its audio. Copying the gram straight from here is the reliable way to get one; a hand-built gram rarely matches an entry.",
        annotations(
            title = "Search the dictionary",
            read_only_hint = true,
            destructive_hint = false,
            open_world_hint = false
        )
    )]
    async fn search_dictionary(
        &self,
        ctx: RequestContext<RoleServer>,
        Parameters(params): Parameters<SearchDictionaryParams>,
    ) -> CallToolResult {
        let query = params.query.trim().to_string();
        if query.is_empty() {
            return error("query must not be empty");
        }
        let limit = params.limit.unwrap_or(20).min(100);

        let slot = match self.state_slot(&ctx).await {
            Ok(slot) => slot,
            Err(e) => return error(e),
        };
        let mut state = slot.lock().await;
        if let Err(e) = state.sync().await {
            log::warn!("sync failed, using possibly-stale events: {e:#}");
        }
        let language = state.context.course.target_language;
        let deck = state.deck();
        let pack = &deck.context().language_pack;
        let entries = deck.get_gram_dictionary_entries(Some(query.clone()), limit);
        let results: Vec<DictionaryMatchOut> = entries
            .iter()
            .map(|entry| DictionaryMatchOut {
                language,
                display_text: entry.display_text(),
                is_phrase: entry.is_phrase(),
                senses: entry
                    .senses()
                    .into_iter()
                    .map(|sense| DictionarySenseMatchOut {
                        gram: dictionary_sense_gram(pack, sense.frequency_index),
                        gloss: sense.gloss,
                        frequency_rank: sense.frequency_index + 1,
                        in_deck: sense.is_in_deck,
                        definition: sense.definition,
                    })
                    .collect(),
            })
            .collect();
        ok_typed(&SearchDictionaryOut {
            query,
            results,
            note: "frequency_rank 1 is the most common sense in the course; common senses are usually the most useful to learn first. A sense's gram, together with the word's language, works directly with add_cards and get_sentences.".to_string(),
        })
    }

    #[tool(
        title = "Add flashcards",
        output_schema = rmcp::handler::server::common::schema_for_type::<crate::output::AddCardsOut>(),
        description = "Add words or phrases to the user's yap deck as new flashcards, from (language, gram) pairs as returned by search_dictionary. A gram that doesn't name a real dictionary entry is turned away, and the whole call is rejected so nothing half-adds. Each new card becomes part of the user's daily reviews from then on, in the app as well as here, so it's worth checking with the user which words they want before adding.",
        annotations(
            title = "Add flashcards",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn add_cards(
        &self,
        ctx: RequestContext<RoleServer>,
        Parameters(params): Parameters<AddCardsParams>,
    ) -> CallToolResult {
        if params.grams.is_empty() {
            return error("grams must not be empty");
        }
        let slot = match self.state_slot(&ctx).await {
            Ok(slot) => slot,
            Err(e) => return error(e),
        };
        let mut state = slot.lock().await;
        if let Err(e) = state.sync().await {
            log::warn!("sync failed, using possibly-stale events: {e:#}");
        }
        if let Err(e) = state.check_language(&params.language) {
            return error(e);
        }

        // Validate everything before touching the deck: reject the whole call
        // on any unknown gram rather than adding a best guess.
        let target_language = state.context.course.target_language;
        let mut resolved = Vec::new();
        let mut errors = Vec::new();
        for Verbatim(gram) in params.grams {
            match state.resolve_gram(&gram) {
                Ok(entry) => resolved.push(entry.map(|gram| state.pack().resolve_gram(&gram))),
                Err(e) => errors.push(e),
            }
        }
        if !errors.is_empty() {
            return error(format!("no cards were added:\n{}", errors.join("\n")));
        }

        let candidates: Vec<(Card, String)> = resolved
            .into_iter()
            .map(|gram| {
                let display = gram.to_display_string(target_language);
                (CardIndicator::WrittenGram { gram }, display)
            })
            .collect();

        let deck = state.deck();
        let mut to_add = Vec::new();
        let mut already_in_deck = Vec::new();
        for (card, display) in candidates {
            if deck.find_card_summary(&card).is_some() {
                already_in_deck.push(display);
            } else {
                to_add.push((card, display));
            }
        }

        let added: Vec<String> = to_add.iter().map(|(_, display)| display.clone()).collect();
        let mut synced = SyncStatus::Synced;
        if !to_add.is_empty() {
            let sentence_list = deck.current_sentence_list();
            let content = LanguageEventContent::AddCards {
                cards: to_add.into_iter().map(|(card, _)| card).collect(),
                sentence_list,
            };
            synced = state.append_event(content).await;
        }

        let deck = state.deck();
        ok_typed(&AddCardsOut {
            added,
            already_in_deck,
            total_cards_in_deck: deck.num_cards_added(),
            synced: synced == SyncStatus::Synced,
        })
    }

    #[tool(
        title = "List due flashcards",
        output_schema = rmcp::handler::server::common::schema_for_type::<crate::output::GetDueCardsOut>(),
        description = "List the user's due yap flashcards, most overdue first, with the due cards broken down by kind and FSRS state, and how many locked cards are due now versus later. Each entry carries its language and card object, which present_card, present_translation and log_review take as they are. A gram card can also be quizzed in conversation using a sentence from get_sentences. Lockup is yap's way of keeping a big backlog manageable: when more than 20 cards are due, the 15 most overdue stay active and the rest are set aside, so the user isn't faced with a wall of reviews. include_locked also lists the locked cards that are due.",
        annotations(
            title = "List due flashcards",
            read_only_hint = true,
            destructive_hint = false,
            open_world_hint = false
        )
    )]
    async fn get_due_cards(
        &self,
        ctx: RequestContext<RoleServer>,
        Parameters(params): Parameters<GetDueCardsParams>,
    ) -> CallToolResult {
        let limit = params.limit.unwrap_or(10).min(50);
        let slot = match self.state_slot(&ctx).await {
            Ok(slot) => slot,
            Err(e) => return error(e),
        };
        let mut state = slot.lock().await;
        if let Err(e) = state.sync().await {
            log::warn!("sync failed, using possibly-stale events: {e:#}");
        }
        let language = state.context.course.target_language;
        let now = Utc::now();
        let deck = state.deck();
        let breakdown = DeckBreakdownOut::new(deck, now);
        let cards: Vec<CardSummaryOut> = deck
            .card_summaries_with_lockup(now.timestamp_millis() as f64)
            .iter()
            .filter(|(summary, locked)| {
                summary.due_timestamp_ms <= now.timestamp_millis() as f64
                    && (params.include_locked || !locked)
            })
            .take(limit)
            .map(|(summary, locked)| card_summary_out(language, summary, *locked))
            .collect();

        ok_typed(&GetDueCardsOut {
            total_due: breakdown.due_now_active.total,
            showing: cards.len(),
            cards,
            due_breakdown: breakdown.due_now_active,
            lockup: breakdown.lockup,
            note: "kind 'written' means recognizing the written word; 'listening' is normally an audio card (present_card plays it, and in plain text you can quiz it as best you can); 'pronunciation' is a letter-sound pattern. A card's gram works with get_sentences. Locked cards that are due are a backlog the user hasn't reached yet; unlock_cards brings back the next batch when they want more, and reviewing a locked card unlocks it too.".to_string(),
        })
    }

    #[tool(
        title = "Unlock cards",
        output_schema = rmcp::handler::server::common::schema_for_type::<crate::output::UnlockCardsOut>(),
        description = "Release the next batch of set-aside cards from lockup, putting them back in the due queue. Lockup is how yap keeps sessions manageable: when a big backlog is due at once, it keeps a handful of the most overdue cards active and sets the rest aside for later, so the user can work through it at their own pace. Unlocking adds to today's workload, which is the user's call, so it's good to ask whether they'd like more before calling. Reviewing a locked card also unlocks it.",
        annotations(
            title = "Unlock cards",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn unlock_cards(&self, ctx: RequestContext<RoleServer>) -> CallToolResult {
        let slot = match self.state_slot(&ctx).await {
            Ok(slot) => slot,
            Err(e) => return error(e),
        };
        let mut state = slot.lock().await;
        if let Err(e) = state.sync().await {
            log::warn!("sync failed, using possibly-stale events: {e:#}");
        }
        let language = state.context.course.target_language;

        let (content, unlocked) = {
            let deck = state.deck();
            match deck.get_release_offer(vec![], Utc::now().timestamp_millis() as f64) {
                None => {
                    let lockup = DeckBreakdownOut::new(deck, Utc::now()).lockup;
                    let note = if lockup.total == 0 {
                        "no cards are in lockup"
                    } else {
                        "locked cards exist but none are due yet; try again once they're due"
                    };
                    return ok_typed(&UnlockCardsOut {
                        unlocked: vec![],
                        lockup,
                        note: note.to_string(),
                        synced: None,
                    });
                }
                Some(offer) => {
                    let unlocked: Vec<CardSummaryOut> = offer
                        .release_preview()
                        .iter()
                        .map(|summary| card_summary_out(language, summary, false))
                        .collect();
                    let DeckEvent::Language(event) = offer.unlock_event();
                    (event.content, unlocked)
                }
            }
        };
        let synced = state.append_event(content).await;

        ok_typed(&UnlockCardsOut {
            unlocked,
            lockup: DeckBreakdownOut::new(state.deck(), Utc::now()).lockup,
            note: "These cards are back in the due queue and now show up in get_due_cards."
                .to_string(),
            synced: Some(synced == SyncStatus::Synced),
        })
    }

    #[tool(
        title = "Present a card",
        output_schema = rmcp::handler::server::common::schema_for_type::<crate::output::PresentOut>(),
        description = "Present one card to the user as an interactive widget: a written card quizzes all added meanings sharing its prefixed text; a listening card quizzes the spoken gram, with audio (a human recording when the course has one) and a reveal; a pronunciation card shows the letter-sound pattern with example words and their audio. The user grades themselves in the widget, and the widget records that review on their account and reports the outcome back to you; since it's already counted, logging it again would review the card twice. For a written card the user has seen before, present_translation usually makes the better exercise (it's what the yap app does), because translating a whole sentence practices every word in it, not just this one.",
        annotations(
            title = "Present a card",
            read_only_hint = true,
            destructive_hint = false,
            open_world_hint = false
        )
    )]
    async fn present_card(
        &self,
        ctx: RequestContext<RoleServer>,
        Parameters(params): Parameters<PresentCardParams>,
    ) -> CallToolResult {
        if self.widget.is_none() {
            return error(
                "the interactive review widget isn't available on this server build — quiz the user in conversation instead (get_due_cards + get_sentences + log_review)",
            );
        }
        let slot = match self.state_slot(&ctx).await {
            Ok(slot) => slot,
            Err(e) => return error(e),
        };
        let mut state = slot.lock().await;
        if let Err(e) = state.sync().await {
            log::warn!("sync failed, using possibly-stale events: {e:#}");
        }
        if let Err(e) = state.check_language(&params.language) {
            return error(e);
        }
        let Verbatim(card) = params.card;
        let target_language = state.context.course.target_language;

        // Pronunciation cards have no gram: build the app's own
        // PronunciationChallenge (guide + per-example audio) and ship that.
        if let CardIndicator::LetterPronunciation { pattern, position } = &card {
            let now_ms = Utc::now().timestamp_millis() as f64;
            let challenge = {
                let deck = state.deck();
                let not_in_deck = || {
                    error(
                        "that card isn't in the user's deck — pass a card exactly as returned by get_due_cards",
                    )
                };
                let Some(pattern_spur) = deck.context().language_pack.string_rodeo.get(pattern)
                else {
                    return not_in_deck();
                };
                let interned = CardIndicator::LetterPronunciation {
                    pattern: pattern_spur,
                    position: *position,
                };
                let Some(card_ctx) = CardContext::new(deck, interned) else {
                    return not_in_deck();
                };
                let review_info = deck.get_review_info(vec![], now_ms);
                match review_info.pronunciation_challenge(deck, &card_ctx, pattern_spur, *position)
                {
                    Some(challenge) => challenge,
                    None => {
                        return error(
                            "no pronunciation guide exists for that pattern in this course — quiz it in conversation instead",
                        );
                    }
                }
            };
            let Challenge::PronunciationChallenge {
                pattern,
                guide,
                cues,
                is_new,
                times_type_seen,
                ..
            } = challenge
            else {
                unreachable!("pronunciation_challenge builds a PronunciationChallenge");
            };
            let language = state.target_language_value();
            let native_language = serde_json::to_value(state.context.course.native_language)
                .expect("Language serializes");
            let structured = json!({
                "challenge": {
                    "type": "pronunciation",
                    "nonce": presentation_nonce(),
                    "language": language,
                    "native_language": native_language,
                    "card": card,
                    "view": yap_frontend_rs::pronunciation_view(pattern.clone(), guide, cues, is_new, times_type_seen, state.context.course.target_language),
                },
            });
            let mut result = CallToolResult::success(vec![ContentBlock::text(format!(
                "Presented the pronunciation pattern «{pattern}» to the user as an interactive card. The widget plays the example audio and records the user's own grade via log_review — do not call log_review for this card yourself; the outcome will be reported back here."
            ))]);
            result.structured_content = Some(structured);
            return result;
        }

        let gram = match &card {
            CardIndicator::WrittenGram { gram } => gram.clone(),
            CardIndicator::ListeningGram { gram } => TaggedGram {
                gram: gram.clone(),
                sense: None,
            },
            CardIndicator::LetterPronunciation { .. } => unreachable!("returned above"),
        };
        let display = gram.to_display_string(target_language);
        let kind = card_kind(&card);
        let Some(interned) = state.pack().resolve_entry(&gram) else {
            return error("that card's gram isn't a dictionary entry in this course");
        };

        // Mint the exact FlashCard the app renders — same builders over the same
        // deck + language pack — so the widget can render the app's Flashcard
        // component verbatim (full card back, morphology, homophone grid) with
        // no widget-specific projection of the content.
        let now_ms = Utc::now().timestamp_millis() as f64;
        let (flashcard, view) = {
            let deck = state.deck();
            let review_info = deck.get_review_info(vec![], now_ms);
            let interned_indicator = match &card {
                CardIndicator::WrittenGram { .. } => CardIndicator::WrittenGram { gram: interned },
                CardIndicator::ListeningGram { .. } => CardIndicator::ListeningGram {
                    gram: interned.gram,
                },
                CardIndicator::LetterPronunciation { .. } => unreachable!("returned above"),
            };
            let Some(ctx) = CardContext::new(deck, interned_indicator) else {
                return error(
                    "that card isn't in the user's deck — pass a card exactly as returned by get_due_cards",
                );
            };
            let flashcard = match &card {
                CardIndicator::WrittenGram { .. } => {
                    review_info.written_gram_flashcard(deck, interned)
                }
                CardIndicator::ListeningGram { .. } => {
                    review_info.listening_gram_flashcard(deck, interned.gram)
                }
                CardIndicator::LetterPronunciation { .. } => unreachable!("returned above"),
            };
            let view = yap_frontend_rs::flashcard_view(
                flashcard.clone(),
                ctx.is_new,
                review_info.total_count(),
                ctx.times_type_seen,
                target_language,
                state.context.course.native_language,
            );
            (flashcard, view)
        };

        let language = state.target_language_value();
        let structured = json!({
            "challenge": {
                "type": "flashcard",
                "nonce": presentation_nonce(),
                "language": language,
                "kind": kind,
                "view": view,
                "card": card,
                "content": serde_json::to_value(&flashcard.content).expect("content serializes"),
                "audio": serde_json::to_value(&flashcard.audio).expect("audio serializes"),
            },
        });
        let mut result = CallToolResult::success(vec![ContentBlock::text(format!(
            "Presented «{display}» to the user as an interactive card. The widget plays the audio and records the user's own grade via log_review — do not call log_review for this card yourself; the outcome will be reported back here."
        ))]);
        result.structured_content = Some(structured);
        result
    }

    #[tool(
        title = "Present a translation challenge",
        output_schema = rmcp::handler::server::common::schema_for_type::<crate::output::PresentOut>(),
        description = "Present a sentence-translation challenge for a written-gram card, as an interactive widget. The user reads a real sentence containing the word and types a translation, and the server grades it and logs the review, just like translation challenges in the yap app. This is yap's core exercise for words the user has already met: understanding a whole sentence is the skill being built, and every word in it gets reviewed, so one challenge does the work of several flashcards. For a brand-new word, present_card is the gentler first exposure. The widget records the review and reports the outcome back to you, so there's nothing further to log (log_review or grade_translation would count it a second time). You can choose the sentence by passing the exact text of one from get_sentences; comprehensible_sentences fit best, since the user should know every other word in it. If you leave it out, the server picks the least-reviewed comprehensible sentence, as the app does.",
        annotations(
            title = "Present a translation challenge",
            read_only_hint = true,
            destructive_hint = false,
            open_world_hint = false
        )
    )]
    async fn present_translation(
        &self,
        ctx: RequestContext<RoleServer>,
        Parameters(params): Parameters<PresentTranslationParams>,
    ) -> CallToolResult {
        if self.widget.is_none() {
            return error(
                "the interactive review widget isn't available on this server build — quiz the user in conversation instead (get_due_cards + get_sentences + log_review)",
            );
        }
        let slot = match self.state_slot(&ctx).await {
            Ok(slot) => slot,
            Err(e) => return error(e),
        };
        let mut state = slot.lock().await;
        if let Err(e) = state.sync().await {
            log::warn!("sync failed, using possibly-stale events: {e:#}");
        }
        if let Err(e) = state.check_language(&params.language) {
            return error(e);
        }
        let Verbatim(card) = params.card;
        let is_new = {
            let Some(summary) = state.deck().find_card_summary(&card) else {
                return error(
                    "that card isn't in the user's deck — pass a card exactly as returned by get_due_cards",
                );
            };
            summary.state() == "New"
        };
        let (spur, challenge) =
            match state.translation_challenge_payload(&card, params.sentence.as_deref()) {
                Ok(payload) => payload,
                Err(e) => return error(e),
            };

        let target_language = state.context.course.target_language;
        let display = challenge
            .primary_expression
            .to_display_string(target_language);
        // Send the full literals (word + word_type + whitespace) so the widget
        // renders the app's shared ChallengeSentence verbatim, including the
        // heteronym-aware colouring.
        let literals =
            serde_json::to_value(&challenge.target_language_literals).expect("literals serialize");

        let structured = json!({
            "challenge": {
                "type": "translation",
                "nonce": presentation_nonce(),
                "language": state.target_language_value(),
                "card": card,
                "is_new": is_new,
                "sentence": {
                    "text": challenge.target_language,
                    "literals": literals,
                    "chunks": language_utils::sentence_chunks(&challenge.target_language_literals.iter().map(|literal| (literal.word.text.as_str(), literal.whitespace)).collect::<Vec<_>>(), target_language),
                    "sources": state.sentence_sources(&spur),
                },
                "audio": serde_json::to_value(&challenge.audio).expect("audio serializes"),
            },
        });
        let mut result = CallToolResult::success(vec![ContentBlock::text(format!(
            "Presented a translation challenge for «{display}»: the user is translating «{}». The widget autogrades the submission and logs the review itself — do not call log_review or grade_translation for it; the outcome will be reported back here.",
            challenge.target_language
        ))]);
        result.structured_content = Some(structured);
        result
    }

    #[tool(
        title = "Grade a translation",
        output_schema = rmcp::handler::server::common::schema_for_type::<crate::output::GradeTranslationOut>(),
        description = "Autograde the user's typed translation of a presented challenge sentence and log the review, which reviews every word in the sentence. The present_translation widget calls this when the user submits. It exists for the widget rather than for conversation: the grade is meant to reflect the user's own typed answer, so a translation written or relayed by anyone else would give their schedule a misleading signal.",
        annotations(
            title = "Grade a translation",
            read_only_hint = false,
            destructive_hint = false,
            open_world_hint = false
        )
    )]
    async fn grade_translation(
        &self,
        ctx: RequestContext<RoleServer>,
        Parameters(params): Parameters<GradeTranslationParams>,
    ) -> CallToolResult {
        let slot = match self.state_slot(&ctx).await {
            Ok(slot) => slot,
            Err(e) => return error(e),
        };
        const TOOL: &str = "grade_translation";
        // The presentation's immutable identity — the mutable submission is
        // deliberately excluded so an edited retry still replays.
        let identity = json!([&*params.card, &params.language, &params.sentence]).to_string();
        let mut state = slot.lock().await;
        // Sync first, so a retry after a failed upload drives the pending
        // flush before we short-circuit on the cached response.
        if let Err(e) = state.sync().await {
            log::warn!("sync failed, using possibly-stale events: {e:#}");
        }
        // A retry of this presentation (same nonce) replays the first grade —
        // no second review event, and no second (paid, nondeterministic) LLM
        // grading — even if the user edited their answer before retrying.
        match state.replay_idempotent(TOOL, params.idempotency_token.as_deref(), &identity) {
            Replay::Hit(cached) => return cached,
            Replay::IdentityConflict => return error(IDEMPOTENCY_CONFLICT_MSG),
            Replay::Miss => {}
        }
        if let Err(e) = state.check_language(&params.language) {
            return error(e);
        }
        let Verbatim(card) = params.card;
        let submission = params.submission.trim().to_string();
        if submission.is_empty() {
            return error("the submission is empty — the user needs to type a translation first");
        }
        let (_, challenge) =
            match state.translation_challenge_payload(&card, Some(&params.sentence)) {
                Ok(payload) => payload,
                Err(e) => return error(e),
            };

        let course = state.context.course;
        let native_language = course.native_language;
        // The exact same grading the app runs (perfect-match short-circuit,
        // AI backend via fetch-happen, heuristic fallback) — fetch-happen's
        // native transport lets us call it directly instead of reimplementing
        // it. Anonymous token, like the widget's TTS path.
        let response = autograde_translation(
            challenge.target_language.clone(),
            submission.clone(),
            challenge.native_translations.clone(),
            challenge.target_language_literals.clone(),
            challenge.unique_target_language_phrases.clone(),
            None,
            course,
            autograde::GramDefinitions(challenge.gram_definitions_for_lookup.clone()),
            challenge.literal_gram_indices.clone(),
            autograde::GramDefinitions(challenge.phrase_definitions.clone()),
            challenge.primary_expression.clone(),
            challenge.primary_literal_indices.clone(),
            autograde::MovieTitles(challenge.movie_titles.clone()),
        )
        .await;

        // The promotion rule is shared Rust — see translation_is_perfect.
        let perfect =
            translation_is_perfect(challenge.target_language_literals.clone(), response.clone());

        let event = if perfect {
            state
                .deck()
                .translate_sentence_perfect(vec![], challenge.target_language.clone())
        } else {
            state.deck().translate_sentence_wrong(
                challenge.target_language.clone(),
                submission.clone(),
                autograde::LiteralGrades(response.literal_grades.clone()),
                vec![],
                response.phrases_remembered.clone(),
                response.phrases_forgot.clone(),
            )
        };
        let Some(DeckEvent::Language(event)) = event else {
            return error("could not build the review event for that sentence");
        };
        let synced = state.append_event(event.content).await;

        let remaining_due = state
            .deck()
            .due_card_summaries(Utc::now().timestamp_millis() as f64)
            .len();
        let target_language = course.target_language;
        let correct_translation =
            find_closest_match(&submission, &challenge.native_translations, native_language)
                .or_else(|| challenge.native_translations.first().cloned());

        let result = ok_typed(&GradeTranslationOut {
            perfect,
            correct_translation,
            encouragement: response.encouragement,
            explanation: response.explanation,
            autograding_error: response.autograding_error,
            literal_grades: response
                .literal_grades
                .iter()
                .map(|grade| match grade {
                    Some(autograde::Remembered::Remembered) => Some(LiteralGradeOut::Remembered),
                    Some(autograde::Remembered::Forgot) => Some(LiteralGradeOut::Forgot),
                    None => None,
                })
                .collect(),
            phrases_remembered: response
                .phrases_remembered
                .iter()
                .map(|phrase| phrase.to_display_string(target_language))
                .collect(),
            phrases_forgot: response
                .phrases_forgot
                .iter()
                .map(|phrase| phrase.to_display_string(target_language))
                .collect(),
            remaining_due,
            synced: synced == SyncStatus::Synced,
        });
        state.remember_idempotent(
            TOOL,
            params.idempotency_token.as_deref(),
            &identity,
            &result,
        );
        result
    }

    #[tool(
        title = "Get audio",
        output_schema = rmcp::handler::server::common::schema_for_type::<crate::output::GetAudioOut>(),
        description = "Fetch pronunciation audio for a widget: the course's human voice-actor recording when one exists, otherwise synthesized speech — the same resolution the yap app uses. Returns base64 audio. It's here for the review widget to play and isn't much use in conversation.",
        annotations(
            title = "Get audio",
            read_only_hint = true,
            destructive_hint = false,
            open_world_hint = false
        )
    )]
    async fn get_audio(
        &self,
        ctx: RequestContext<RoleServer>,
        Parameters(params): Parameters<GetAudioParams>,
    ) -> CallToolResult {
        let GetAudioParams {
            request: Verbatim(request),
            provider,
        } = params;

        // Authenticated per-user state must exist before we spend TTS money,
        // and loading it registers the course pack's human recordings so the
        // lookup below can find them.
        if let Err(e) = self.state_slot(&ctx).await {
            return error(e);
        }

        let cache_key = format!(
            "{}|{}",
            serde_json::to_string(&request).expect("request serializes"),
            serde_json::to_string(&provider).expect("provider serializes"),
        );
        if let Some(cached) = audio_cache_get(&cache_key) {
            return audio_result(cached);
        }

        let resolved = if yap_frontend_rs::human_audio_applies(&request)
            && let Some(human) =
                yap_frontend_rs::lookup_human_audio(request.language, &request.text)
        {
            CachedAudio {
                bytes: human.bytes,
                voice_actor: Some(yap_frontend_rs::VoiceActorInfo {
                    name: human.actor_name,
                    compensation: human.compensation,
                }),
            }
        } else {
            match yap_frontend_rs::fetch_tts(&request, &provider, None).await {
                Ok(bytes) => CachedAudio {
                    bytes,
                    voice_actor: None,
                },
                Err(e) => return error(format!("audio synthesis failed: {e}")),
            }
        };
        audio_cache_put(&cache_key, resolved.clone());
        audio_result(resolved)
    }

    #[tool(
        title = "Log a review",
        output_schema = rmcp::handler::server::common::schema_for_type::<crate::output::LogReviewOut>(),
        description = "Record the ratings for one challenge you quizzed the user on in conversation (the widgets log their own). Each rating reschedules the card with FSRS on the user's real account, so the schedule is only as good as the ratings: an honest rating after a real quiz keeps their intervals accurate, while a guessed or generous one teaches the scheduler something false. Ratings: again (forgot), hard, good, easy, or remembered (a simple yes, scheduled as easy for a new card and good otherwise). One written front can carry several meanings, so a review lists a card for each meaning row the user was actually tested on; listening and pronunciation reviews have a single card.",
        annotations(
            title = "Log a review",
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn log_review(
        &self,
        ctx: RequestContext<RoleServer>,
        Parameters(params): Parameters<LogReviewParams>,
    ) -> CallToolResult {
        let mut reviews = Vec::new();
        for review in &params.reviews {
            let rating = match parse_rating(&review.rating) {
                Ok(r) => r,
                Err(e) => return error(e),
            };
            reviews.push(yap_frontend_rs::CardReview {
                card: (*review.card).clone(),
                rating,
            });
        }
        if reviews.is_empty() {
            return error("reviews must not be empty");
        }
        let slot = match self.state_slot(&ctx).await {
            Ok(slot) => slot,
            Err(e) => return error(e),
        };
        const TOOL: &str = "log_review";
        // Submitted card identities define the presentation; ratings are excluded
        // so a retry replays the first review, even when some cards are filtered out.
        let cards: Vec<_> = params.reviews.iter().map(|review| &*review.card).collect();
        let identity = json!([cards, &params.language]).to_string();
        let mut state = slot.lock().await;
        // Sync first, so a retry after a failed upload drives the pending
        // flush before we short-circuit on the cached response.
        if let Err(e) = state.sync().await {
            log::warn!("sync failed, using possibly-stale events: {e:#}");
        }
        // A retried log of the same presentation (same nonce) replays the
        // first result instead of recording the card as reviewed twice.
        match state.replay_idempotent(TOOL, params.idempotency_token.as_deref(), &identity) {
            Replay::Hit(cached) => return cached,
            Replay::IdentityConflict => return error(IDEMPOTENCY_CONFLICT_MSG),
            Replay::Miss => {}
        }
        if let Err(e) = state.check_language(&params.language) {
            return error(e);
        }
        let deck = state.deck();
        let Some(yap_frontend_rs::DeckEvent::Language(event)) = deck.review_cards(reviews) else {
            return error("no reviewed cards are present in the deck");
        };
        let LanguageEventContent::ReviewCards { reviews } = &event.content else {
            unreachable!()
        };
        let first = &reviews[0];
        let card = first.card.clone();
        let rating = format!("{:?}", first.rating).to_lowercase();
        let before = deck.find_card_summary(&card);
        let reviewed = before
            .as_ref()
            .map(|summary| summary.card_text())
            .unwrap_or_else(|| match &card {
                CardIndicator::WrittenGram { gram } => {
                    gram.to_display_string(state.context.course.target_language)
                }
                CardIndicator::ListeningGram { gram } => {
                    gram.to_display_string(state.context.course.target_language)
                }
                CardIndicator::LetterPronunciation { pattern, .. } => pattern.clone(),
            });
        let now_ms = Utc::now().timestamp_millis() as f64;
        let was_due = before
            .as_ref()
            .is_some_and(|summary| summary.due_timestamp_ms() <= now_ms);

        let synced = state.append_event(event.content).await;

        let deck = state.deck();
        let after = deck.find_card_summary(&card);
        let remaining_due = deck
            .due_card_summaries(Utc::now().timestamp_millis() as f64)
            .len();
        let result = ok_typed(&LogReviewOut {
            reviewed,
            rating,
            was_due,
            fsrs_state: after.as_ref().map(|a| a.state().to_string()),
            next_due: after.and_then(|a| {
                chrono::DateTime::from_timestamp_millis(a.due_timestamp_ms() as i64)
                    .map(|d| d.to_rfc3339())
            }),
            remaining_due,
            total_reviews: deck.stats().total_reviews,
            synced: synced == SyncStatus::Synced,
        });
        state.remember_idempotent(
            TOOL,
            params.idempotency_token.as_deref(),
            &identity,
            &result,
        );
        result
    }

    #[tool(
        title = "Get example sentences",
        output_schema = rmcp::handler::server::common::schema_for_type::<crate::output::GetSentencesOut>(),
        description = "Get random example sentences containing a gram, with translations and source attribution. They come in two lists. In comprehensible_sentences, every other word is one the user already knows, so the gram is the only new thing and a quiz on one tests exactly that word; these are the best ones for quizzing. other_sentences are sampled from everything containing the gram, whatever its difficulty, and are better for showing the word in its natural range of use.",
        annotations(
            title = "Get example sentences",
            read_only_hint = true,
            destructive_hint = false,
            open_world_hint = false
        )
    )]
    async fn get_sentences(
        &self,
        ctx: RequestContext<RoleServer>,
        Parameters(params): Parameters<GetSentencesParams>,
    ) -> CallToolResult {
        let count = params.count.unwrap_or(5).clamp(1, 20);
        let slot = match self.state_slot(&ctx).await {
            Ok(slot) => slot,
            Err(e) => return error(e),
        };
        let mut state = slot.lock().await;
        if let Err(e) = state.sync().await {
            log::warn!("sync failed, using possibly-stale events: {e:#}");
        }
        if let Err(e) = state.check_language(&params.language) {
            return error(e);
        }
        let Verbatim(gram) = params.gram;
        let interned = match state.resolve_gram(&gram) {
            Ok(interned) => interned,
            Err(e) => return error(e),
        };
        let display = gram.to_display_string(state.context.course.target_language);

        let sampled = state.sample_sentences(interned, count);
        ok_typed(&GetSentencesOut {
            word: display,
            comprehensible_sentences: sampled.comprehensible,
            total_comprehensible: sampled.total_comprehensible,
            other_sentences: sampled.other,
            total_containing_word: sampled.total_containing,
            note: "comprehensible_sentences use only words the user already knows besides this one; other_sentences can include unfamiliar words, and some may lack translations.".to_string(),
        })
    }

    #[tool(
        title = "Get learning stats",
        output_schema = rmcp::handler::server::common::schema_for_type::<crate::output::GetStatsOut>(),
        description = "Get the user's yap stats: streak, XP, review counts, comprehension tier, recent daily activity, and a breakdown of the deck — every card by kind and FSRS state, what's due now, lockup (locked cards due now vs not yet due), a due forecast for the next 1/7/30 days, and leeches.",
        annotations(
            title = "Get learning stats",
            read_only_hint = true,
            destructive_hint = false,
            open_world_hint = false
        )
    )]
    async fn get_stats(&self, ctx: RequestContext<RoleServer>) -> CallToolResult {
        let slot = match self.state_slot(&ctx).await {
            Ok(slot) => slot,
            Err(e) => return error(e),
        };
        let mut state = slot.lock().await;
        if let Err(e) = state.sync().await {
            log::warn!("sync failed, using possibly-stale events: {e:#}");
        }
        let course = state.context.course;
        let deck = state.deck();
        let stats = deck.stats();
        let tier = deck.get_current_tier();

        // History is retained in full; this endpoint still shows the latest study week.
        let first_day = stats
            .past_days
            .last_key_value()
            .map_or(0, |(&day, _)| day - 6);
        let past_days: Vec<DayOut> = stats
            .past_days
            .range(first_day..)
            .rev()
            .map(|(day, summary)| DayOut {
                date: chrono::NaiveDate::from_num_days_from_ce_opt(*day as i32)
                    .map(|d| d.to_string()),
                reviews: summary.reviews,
                time_spent_seconds: summary.time_spent_seconds,
                new_cards: summary.new_cards,
                learned_cards: summary.learned_cards,
            })
            .collect();

        ok_typed(&GetStatsOut {
            course: format!(
                "{} for {} speakers",
                course.target_language, course.native_language
            ),
            daily_streak: deck.get_daily_streak(),
            xp: stats.xp,
            total_reviews: stats.total_reviews,
            started: stats.start_time.map(|t| t.to_rfc3339()),
            cards: DeckBreakdownOut::new(deck, Utc::now()),
            tier: TierOut {
                name: tier.name,
                tier: tier.tier,
                level: tier.level,
                total_levels: tier.total_levels,
                percent_known: tier.percent_known,
            },
            recent_days: past_days,
        })
    }

    #[tool(
        title = "Search dictionary pages",
        output_schema = rmcp::handler::server::common::schema_for_type::<crate::output::SearchOut>(),
        description = "Search the dictionary of the user's yap course. Returns matching entry pages as results with id, title, and url (a citable yap.town dictionary link); an id passed to fetch returns the full entry. This pair is for looking things up and citing them; the deck tools need exact grams, which come from search_dictionary.",
        annotations(
            title = "Search dictionary pages",
            read_only_hint = true,
            destructive_hint = false,
            open_world_hint = false
        )
    )]
    async fn search(
        &self,
        ctx: RequestContext<RoleServer>,
        Parameters(params): Parameters<SearchParams>,
    ) -> CallToolResult {
        let query = params.query.trim().to_string();
        if query.is_empty() {
            return error("query must not be empty");
        }
        let slot = match self.state_slot(&ctx).await {
            Ok(slot) => slot,
            Err(e) => return error(e),
        };
        let mut state = slot.lock().await;
        if let Err(e) = state.sync().await {
            log::warn!("sync failed, using possibly-stale events: {e:#}");
        }
        let course = state.context.course;
        let results: Vec<SearchResultOut> = state
            .deck()
            .get_gram_dictionary_entries(Some(query), 10)
            .iter()
            .map(|entry| SearchResultOut {
                id: entry_id(&course, entry),
                title: entry_title(entry),
                url: entry_url(&course, &entry.display_text()),
            })
            .collect();
        ok_typed(&SearchOut { results })
    }

    #[tool(
        title = "Fetch a dictionary page",
        output_schema = rmcp::handler::server::common::schema_for_type::<crate::output::FetchOut>(),
        description = "Fetch a dictionary entry page by an id returned from the search tool: definitions, frequency rank, whether it's in the user's deck, and example sentences with translations and source attribution. Returns readable text plus a citable url.",
        annotations(
            title = "Fetch a dictionary page",
            read_only_hint = true,
            destructive_hint = false,
            open_world_hint = false
        )
    )]
    async fn fetch(
        &self,
        ctx: RequestContext<RoleServer>,
        Parameters(params): Parameters<FetchParams>,
    ) -> CallToolResult {
        let parsed = params
            .id
            .rsplit_once(':')
            .and_then(|(slug, idx)| Some((slug.to_string(), idx.parse::<usize>().ok()?)));
        let Some((course_slug, index)) = parsed else {
            return error(
                "invalid id — pass an id exactly as returned by the search tool, e.g. \"french-to-english:42\"",
            );
        };

        let slot = match self.state_slot(&ctx).await {
            Ok(slot) => slot,
            Err(e) => return error(e),
        };
        let mut state = slot.lock().await;
        if let Err(e) = state.sync().await {
            log::warn!("sync failed, using possibly-stale events: {e:#}");
        }
        let course = state.context.course;
        if course_slug != course.dictionary_slug() {
            return error(format!(
                "that id belongs to the '{course_slug}' dictionary, but this account's course is '{}'",
                course.dictionary_slug()
            ));
        }
        let Some(entry) = state.deck().gram_dictionary_entry(index) else {
            return error(
                "no dictionary entry with that id — use an id returned by the search tool",
            );
        };

        let interned: TaggedGram<SpurGram> = *state
            .pack()
            .sense_frequencies
            .get_index(entry.frequency_index())
            .expect("index valid: entry was just built from it")
            .0;
        let senses = entry.senses();
        let in_deck = senses.iter().any(|sense| sense.is_in_deck);
        let language = state.context.course.target_language;
        let sampled = state.sample_sentences(interned, 3);

        let display = entry.display_text();
        let url = entry_url(&course, &display);
        use std::fmt::Write as _;
        let mut text = String::new();
        let _ = writeln!(text, "{display}");
        for (index, sense) in senses.iter().enumerate() {
            let _ = writeln!(
                text,
                "\nSense {} (frequency rank {}):",
                index + 1,
                sense.frequency_index + 1
            );
            if !sense.definition.morphology_label.is_empty() {
                let _ = writeln!(text, "{}", sense.definition.morphology_label);
            }
            for meaning in &sense.definition.senses {
                let _ = write!(text, "• {}", meaning.meaning);
                if let Some(note) = &meaning.note {
                    let _ = write!(text, " ({note})");
                }
                let _ = writeln!(text);
                if let Some(example) = &meaning.example {
                    let _ = writeln!(text, "  e.g. “{}” — “{}”", example.target, example.native);
                }
            }
        }
        let _ = writeln!(
            text,
            "\nFrequency rank {} (1 = most common in this course). {}",
            entry.frequency_index() + 1,
            if in_deck {
                "Already in the user's deck."
            } else {
                "Not in the user's deck."
            }
        );
        for (label, list) in [
            (
                "Example sentences the user can already fully understand:",
                &sampled.comprehensible,
            ),
            (
                "More sentences containing it (may use words the user hasn't learned):",
                &sampled.other,
            ),
        ] {
            if list.is_empty() {
                continue;
            }
            let _ = writeln!(text, "\n{label}");
            for sentence in list {
                let _ = write!(text, "• “{}”", sentence.text);
                if let Some(translation) = sentence.translations.first() {
                    let _ = write!(text, " — “{translation}”");
                }
                if !sentence.sources.is_empty() {
                    let _ = write!(text, " (source: {})", sentence.sources.join(", "));
                }
                let _ = writeln!(text);
            }
        }
        let _ = writeln!(text, "\nDictionary page: {url}");

        ok_typed(&FetchOut {
            id: entry_id(&course, &entry),
            title: entry_title(&entry),
            text,
            url,
            metadata: FetchMetadataOut {
                language,
                senses: senses
                    .into_iter()
                    .map(|sense| FetchSenseOut {
                        gram: dictionary_sense_gram(state.pack(), sense.frequency_index),
                        gloss: sense.gloss,
                        frequency_rank: sense.frequency_index + 1,
                        in_deck: sense.is_in_deck,
                    })
                    .collect(),
                frequency_rank: entry.frequency_index() + 1,
                in_deck,
                is_phrase: entry.is_phrase(),
            },
        })
    }
}

impl YapMcp {
    fn all_tools() -> rmcp::handler::server::router::tool::ToolRouter<Self> {
        Self::tool_router() + Self::knowledge_router()
    }
}

#[tool_handler(router = Self::all_tools())]
impl ServerHandler for YapMcp {
    /// Hand-written (the macro defers to it): injects the MCP Apps `_meta`
    /// onto widget-related tools, only when the widget is actually built.
    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let mut tools = Self::all_tools().list_all();
        if let Some(widget) = &self.widget {
            for tool in &mut tools {
                if let Some(meta) = widget_tool_meta(&tool.name, &widget.uri) {
                    tool.meta = Some(meta);
                }
            }
        }
        Ok(ListToolsResult {
            tools,
            ..Default::default()
        })
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        let mut result = ListResourcesResult::default();
        if let Some(widget) = &self.widget {
            let mut resource = Resource::new(widget.uri.clone(), "yap review card");
            resource.title = Some("yap review card".to_string());
            resource.description =
                Some("Interactive flashcard widget: audio, reveal, and grading.".to_string());
            resource.mime_type = Some(WIDGET_MIME.to_string());
            resource.size = Some(widget.html.len() as u64);
            resource.meta = Some(widget_resource_meta());
            result.resources.push(resource);
        }
        Ok(result)
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        // Any version of the widget URI (bare, current, or a stale hash) gets
        // the current build. Hosts cache tool metadata across deploys — after
        // a widget rebuild, ChatGPT still asks for the previous hash, and a
        // not-found here surfaces as "Failed to fetch template" on every
        // present_* call until the host refreshes the schema. Serving current
        // content for a stale URI is strictly better than an error; the hash
        // still does its cache-busting job by changing the URI hosts fetch.
        let is_widget_uri = request.uri == WIDGET_URI_BASE
            || request
                .uri
                .strip_prefix(WIDGET_URI_BASE)
                .is_some_and(|rest| rest.starts_with('@'));
        match &self.widget {
            Some(widget) if is_widget_uri => {
                let mut result = ReadResourceResult::new(vec![]);
                result.contents = vec![ResourceContents::TextResourceContents {
                    // Echo the requested URI: hosts key the fetched content
                    // by the URI they asked for.
                    uri: request.uri.clone(),
                    mime_type: Some(WIDGET_MIME.to_string()),
                    text: widget.html.clone(),
                    meta: Some(widget_resource_meta()),
                }];
                Ok(result)
            }
            _ => Err(McpError::resource_not_found(
                format!("unknown resource: {}", request.uri),
                None,
            )),
        }
    }

    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.capabilities = ServerCapabilities::builder()
            .enable_tools()
            .enable_resources()
            .build();
        info.instructions = Some(
            "This connects you to the user's yap.town account, which holds two things: their \
             language-learning deck (flashcards, spaced-repetition reviews, dictionary, \
             stats), and (mostly unrelated to language learning) a knowledge graph, which is a \
             study map on any subject that you help them build and review. (These are the \
             knowledge_* tools, described at the end). The deck is the same one the yap web \
             and iOS apps use, so anything you change shows up there too. The knowledge graph \
             has no website yet, but might get one eventually.\n\
             \n\
             How yap teaches: words are learned and practiced inside real sentences (from \
             films, books and other corpora) that the user can almost fully understand. Each \
             review reschedules a card with FSRS spaced repetition, so honest ratings keep the \
             user's intervals accurate, and a review that happens counts as real practice on \
             their account. A typical session runs get_due_cards, quizzes each card (ideally \
             through the widgets below), and records the outcome.\n\
             \n\
             Words are identified by (language, gram). A gram is the exact token sequence \
             (word + lemma + part of speech) plus a sense number, and it's how the deck links a \
             card to its dictionary entry, sentences and audio. Because of that, grams work \
             best copied from search_dictionary or get_due_cards rather than written by hand: \
             a hand-built gram usually doesn't match any entry, and the server turns it away. \
             Multi-word phrases are learned as chunks on purpose, even transparent ones like \
             \"I don't know\", since fluent speakers process them as a unit. To add words, a \
             good flow is to search_dictionary, show the user what you found, and add what \
             they pick with add_cards.\n\
             \n\
             search and fetch are a standard browse-and-cite pair over the public dictionary at \
             yap.town/d/, handy for looking up and linking entries. search_dictionary is the \
             one that returns grams for the deck tools.\n\
             \n\
             When the interactive widgets are available, they make the nicest quiz, and \
             choosing the one the yap app would use keeps the experience consistent. \
             present_translation suits a written card the user has seen before: they translate \
             a sentence containing the word, the server grades it, and every word in that \
             sentence gets reviewed. present_card suits new cards, listening and pronunciation \
             cards, and words present_translation reports have no comprehensible sentence yet; \
             there the user grades themselves. Either way the widget records the review itself \
             and reports the outcome back to you, so the review is already counted, and \
             logging it again would count it twice.\n\
             \n\
             The knowledge graph is for anything the user wants to learn and keep, outside \
             the language course: a field they're studying, a paper, a codebase, a concept. A \
             concept node in the graph is basically something that the user is guaranteed to \
             revisit and practice. You and the user shape it together. The user tells you what \
             they want to learn, and you break it into atomic concepts they can study. You add \
             concepts as they come up, link them, split ones that turn out to be too broad, \
             and mark ones the user says they already know. It's meant to stay very mutable, \
             and the user is counting on you to keep it up to date. You can also add concepts \
             proactively, including things the user didn't name, for example when they unify, \
             connect or extend what's there, so their understanding ends up as a complete and \
             connected web rather than a list of isolated facts. Maintaining the graph is your \
             part of the collaboration: the user will rarely ask for specific changes (and \
             having to would pull them out of learning) so you shape it proactively and mostly \
             quietly. For example, when a question only reaches part of a concept, or the user \
             has an insight, splitting or adding and linking right then keeps the map \
             accurate. The graph itself follows Math Academy's design. The most important \
             thing is that concepts should be atomic. Like, ridiculously atomic. However atomic \
             you're thinking, even more atomic than that. It's better that way. It is totally \
             fine, even expected, to make a graph with a ton of concepts in it, even for \
             something that seems simple. For example, addition laws would not be one concept, \
             you would have a separate concept for each law. Newton's laws of motion and its \
             applications would not either - each law of motion would have a concept just \
             asking the user to recall the law, and another to apply it in a new case. That \
             said, it's also fine to ask the user what they want to learn and focus on before \
             you make the graph, as it may inform you. For example, a law student sharing a \
             case might want to remember many details about the case, or maybe only remember a \
             rule put forward by the judge.\n\
             \n\
             The atomicity of concepts of course leads to a large number of concepts being \
             created to explain simple things - but this isn't an issue, because if you find \
             out the user already knows a concept during pretesting (more on that later), you \
             can just mark it as already known, and it won't be scheduled for review. And \
             because you inform the system of the prerequisites between concepts, the system \
             knows how to propagate the user's reviews to its prerequisites. (Not to mention, \
             you can and should do that manually as well, if the system ever fails to \
             recognize the user's understanding of something they obviously know.)\n\
             \n\
             Those prerequisite links decide what's ready to learn next (the frontier), and \
             each one has an \"encompassing\" score that says how much practicing that concept \
             exercises the prerequisite. This allows one good review of the advanced concept \
             to give partial credit to its foundations, so fewer explicit reviews are needed. \
             This works best when each concept is small enough for one question to test, so a \
             review score means one clear thing.\n\
             \n\
             The review history stores the question, the user's answer and your notes, which \
             keeps later questions fresh and lets the next session pick up where this one left \
             off. Tags on concepts record both the material (`machine-learning`) and the \
             context the user is learning it in (`for-work`), so later sessions can find \
             things and focus on one goal. Plain-text documents (notes, papers, chapters) can \
             be saved and linked to concepts as source material. knowledge_next plans a \
             session, with each \
             concept's description and recent questions; knowledge_concept shows one concept \
             in full; knowledge_apply records every change, reviews included."
                .to_string(),
        );
        info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every tool parameter must advertise a real JSON type: claude.ai's
    /// client stringifies untyped parameters before they reach the server.
    /// The verbatim parameters must also match the shape their producers
    /// emit (a gram is an array, a card an object), or the client rejects the
    /// echoed value.
    #[test]
    fn every_tool_parameter_has_its_real_type() {
        fn schema<T: JsonSchema>() -> serde_json::Value {
            serde_json::to_value(schemars::schema_for!(T)).unwrap()
        }
        fn is_typed(prop: &serde_json::Value) -> bool {
            ["type", "$ref", "anyOf", "oneOf"]
                .iter()
                .any(|key| prop.get(key).is_some())
        }
        fn check<T: JsonSchema>() -> serde_json::Value {
            let schema = schema::<T>();
            for (name, prop) in schema["properties"].as_object().unwrap() {
                assert!(
                    is_typed(prop),
                    "{}::{name} is untyped: {prop}",
                    std::any::type_name::<T>()
                );
                if let Some(items) = prop.get("items") {
                    assert!(
                        is_typed(items),
                        "{}::{name} items untyped: {items}",
                        std::any::type_name::<T>()
                    );
                }
            }
            schema
        }
        check::<SearchDictionaryParams>();
        check::<GetDueCardsParams>();
        check::<LogReviewParams>();
        check::<PresentCardParams>();
        check::<PresentTranslationParams>();
        check::<GradeTranslationParams>();
        check::<GetAudioParams>();
        check::<SearchParams>();
        check::<FetchParams>();
        check::<crate::knowledge::GraphParams>();
        check::<crate::knowledge::NextParams>();
        check::<crate::knowledge::ConceptParams>();
        check::<crate::knowledge::ReviewsParams>();
        check::<crate::knowledge::DocumentParams>();
        check::<crate::knowledge::SaveDocumentParams>();
        let apply = check::<crate::knowledge::ApplyParams>();
        assert_eq!(
            apply["properties"]["ops"]["items"]["$ref"],
            "#/$defs/KnowledgeEvent"
        );
        for op in apply["$defs"]["KnowledgeEvent"]["oneOf"]
            .as_array()
            .unwrap()
        {
            for (name, prop) in op["properties"].as_object().unwrap() {
                assert!(
                    is_typed(prop),
                    "knowledge op field {name} is untyped: {prop}"
                );
            }
        }

        let sentences = check::<GetSentencesParams>();
        assert_eq!(sentences["properties"]["gram"]["type"], "object");
        let add = check::<AddCardsParams>();
        assert_eq!(add["properties"]["grams"]["items"]["type"], "object");

        let card = &check::<PresentCardParams>()["properties"]["card"];
        assert_eq!(card["type"], "object");
        assert_eq!(card["oneOf"].as_array().unwrap().len(), 3);
    }
}
