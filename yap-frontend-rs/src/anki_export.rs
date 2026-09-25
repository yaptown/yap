//! A host-independent Anki recipe. Hosts package these notes and media, not learning logic.
#[cfg(test)]
#[path = "anki_export_skill_levels.rs"]
mod skill_levels;
use crate::{
    comprehensible::{GramMembership, RankBitset},
    simulation::{DailySimulationIterator, DayChallengeIterator},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use bridgerton::Error;
use language_utils::{
    Atom, CLIPS_ORIGIN, Course, GramDefinition, Language, Literal, MovieMetadataBasic, OtherWord,
    OtherWordType, SentenceGram, SpurGram, TaggedGram, Word, WordType, dictionary_entry_slug,
    language_pack::LanguagePack,
};
use lasso::Spur;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{BTreeSet, VecDeque},
};
use unicode_normalization::UnicodeNormalization;
use xxhash_rust::xxh3::xxh3_64;

use crate::{Deck, clips, get_language_metadata, human_audio, utils};

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AnkiCardTypes {
    Reading,
    Listening,
    Both,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnkiDeckOptions {
    pub card_types: AnkiCardTypes,
    /// Introduce each new word on a card of its own before its sentence.
    pub word_cards: bool,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MintedAnkiDeck {
    pub deck_id: String,
    pub token: String,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnkiDeckPlan {
    pub language: Language,
    /// `{target}-{native}` language codes, e.g. `fra-eng`; names the note
    /// types, tags, and the package file.
    pub course_code: String,
    pub deck_name: String,
    /// Anki shows this on the deck's overview screen; the only place the
    /// deck itself can point back at Yap.
    pub deck_description: String,
    pub deck_id: i64,
    pub sentence_model_id: i64,
    pub word_model_id: i64,
    pub notes: Vec<AnkiNote>,
    pub bundled: Vec<AnkiBundledMedia>,
    /// How much of everyday language (the Essential bar's measure) the
    /// learner will understand once they know every word the deck teaches;
    /// only when that's a gain worth mentioning.
    pub finish_message: Option<String>,
    pub stats: AnkiDeckStats,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnkiDeckStats {
    pub sentence_count: u32,
    pub word_count: u32,
    pub card_count: u32,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum AnkiNote {
    Sentence {
        guid: String,
        note_id: i64,
        card_id: i64,
        sentence: String,
        translation: String,
        target_word: String,
        target_gloss: String,
        glosses: Vec<AnkiGloss>,
        source: AnkiSource,
        clip_url: String,
        tts: String,
        include_reading: bool,
        include_listening: bool,
        tags: Vec<String>,
    },
    Word {
        guid: String,
        note_id: i64,
        card_id: i64,
        word: String,
        definition: String,
        audio: String,
        tags: Vec<String>,
    },
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnkiGloss {
    pub text: String,
    pub gloss: Option<String>,
    pub url: Option<String>,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnkiSource {
    pub title: String,
    pub year: Option<u16>,
    pub imdb_id: String,
    pub poster_filename: Option<String>,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnkiBundledMedia {
    pub filename: String,
    pub source: AnkiMediaSource,
}

/// Where a bundled file's bytes come from: the language pack (via
/// `Deck::anki_bundled_media`) or a fetch the host performs.
#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum AnkiMediaSource {
    Poster { imdb_id: String },
    HumanAudio { text: String },
    Tts { url: String },
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnkiExportView {
    pub title: String,
    /// Films the deck's clips come from, most clips first; only films with
    /// a poster, since the page shows them as a strip of posters.
    pub films: Vec<MovieMetadataBasic>,
    pub needs_placement: bool,
    pub placement_intro: String,
    /// "Essential French": what `percent_known` is a percentage of. The deck
    /// deliberately says nothing about levels.
    pub progress_label: String,
    /// How much of the whole frequency list the learner knows, weighted by
    /// frequency, as the app's curriculum bars count it.
    pub percent_known: f64,
    pub too_advanced: bool,
    pub too_advanced_message: Option<String>,
    pub clips_loaded: bool,
    pub clip_sentence_count: u32,
    /// The course pill: tapping it picks another course for the deck.
    pub course_flag: String,
    pub course_label: String,
    pub card_types_label: String,
    pub reading_label: String,
    pub listening_label: String,
    pub word_cards_label: String,
    pub download_label: String,
    /// Replaces the placement test's "Begin Learning": finishing it here
    /// leads to a deck, not to the app.
    pub placement_complete_label: String,
    /// Why the deck exists, read while it builds.
    pub backstory: String,
    pub backstory_signature: String,
    /// Once the deck is downloaded, signed-in learners are invited into the
    /// app, which already has their placement test.
    pub try_yap_heading: String,
    pub try_yap_body: String,
    pub try_yap_label: String,
    pub enjoy_deck: String,
    /// Signed-out visitors are asked to save their deck instead: one call to
    /// action, not two.
    pub save_deck_heading: String,
    pub save_deck_body: String,
    pub sign_up_label: String,
    /// The deck downloads by itself once built; this saves it again.
    pub download_again_label: String,
}

/// Sentence notes per deck. Not a choice yet: one good default beats a
/// number nobody knows how to pick.
const DECK_SIZE: usize = 300;

/// The option descriptions and intro, which follow what's selected: with
/// both card types on, each describes "some" cards; with one, it describes
/// all of them.
#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnkiOptionsCopy {
    /// Sits between the posters and the progress bar, which it introduces.
    pub intro: String,
    pub reading_description: String,
    pub listening_description: String,
    pub word_cards_description: String,
}

#[bridgerton::bridge]
pub fn anki_options_copy(reading: bool, listening: bool, word_cards: bool) -> AnkiOptionsCopy {
    AnkiOptionsCopy {
        intro: if word_cards {
            "At first, the cards will introduce new words, and then they'll be reinforced with cards showing clips from these films. We'll pick words that are right for your knowledge level:"
        } else {
            "Each card teaches a new word with a clip from one of these films. We'll pick words that are right for your knowledge level:"
        }
        .into(),
        reading_description: if reading && !listening {
            "Cards show the subtitle, and you try to translate."
        } else {
            "Some cards show the subtitle, and you try to translate."
        }
        .into(),
        listening_description: if listening && !reading {
            "Cards have no subtitles, and you try to understand what you hear."
        } else {
            "Some cards have no subtitles, and you try to understand what you hear."
        }
        .into(),
        word_cards_description:
            "Introduce words as standalone cards before using them in sentences.".into(),
    }
}

#[bridgerton::bridge]
pub async fn mint_anki_deck(
    options: AnkiDeckOptions,
    access_token: Option<String>,
) -> Result<MintedAnkiDeck, Error> {
    let response = utils::hit_ai_server(
        fetch_happen::Method::POST,
        "/anki/deck",
        Some(serde_json::json!({"options": options})),
        access_token.as_ref(),
    )
    .await
    .map_err(|e| Error::new(e.to_string()))?
    .error_for_status()
    .map_err(|e| Error::new(e.to_string()))?;
    response.json().await.map_err(|e| Error::new(e.to_string()))
}

// Identities are scoped to the whole course: the same target language for
// two native languages must not share deck, model, or note identities, or
// Anki would merge the decks and overwrite one language's translations with
// the other's.
fn course_code(course: Course) -> String {
    format!(
        "{}-{}",
        course.target_language.code(),
        course.native_language.code()
    )
}
// Reserve the low two bits for template ordinals, staying strictly inside JS's exact range.
fn id(course: Course, kind: &str, text: &str) -> i64 {
    let hash = xxh3_64(format!("yap.anki.v1|{}|{kind}|{text}", course_code(course)).as_bytes());
    (((hash % ((1_u64 << 51) - 1)) + 1) * 4) as i64
}
fn guid(course: Course, kind: &str, text: &str) -> String {
    URL_SAFE_NO_PAD
        .encode(xxh3_64(format!("{}|{kind}|{text}", course_code(course)).as_bytes()).to_be_bytes())
}

/// The word note the deck introduces a word with; it precedes the first
/// sentence that uses the word.
fn word_note(
    pack: &LanguagePack,
    course: Course,
    gram: TaggedGram<SpurGram>,
    word: &str,
    token: &str,
    bundled: &mut Vec<AnkiBundledMedia>,
) -> AnkiNote {
    let definition = pack
        .gram_definitions
        .get(&gram)
        .map(definition)
        .unwrap_or_default();
    let audio = if pack
        .human_audio
        .values()
        .any(|clips| clips.contains_key(word))
    {
        let filename = human_filename(course, word);
        bundled.push(AnkiBundledMedia {
            filename: filename.clone(),
            source: AnkiMediaSource::HumanAudio {
                text: word.to_owned(),
            },
        });
        filename
    } else {
        let filename = format!("yap-word-{}.mp3", guid(course, "word", word));
        bundled.push(AnkiBundledMedia {
            filename: filename.clone(),
            source: AnkiMediaSource::Tts {
                url: tts_url(course.target_language, word, &[], token),
            },
        });
        filename
    };
    let mut tags = note_tags(course, "word");
    let resolved = gram.resolve(&pack.gram_rodeo);
    match resolved.gram.0.as_slice() {
        [
            Atom::Tok(Word {
                word_type: WordType::Heteronym(heteronym),
                ..
            }),
        ] => {
            let pos = serde_json::to_value(heteronym.pos).unwrap();
            tags.push(format!(
                "yap::pos::{}",
                pos.as_str().unwrap().to_lowercase()
            ));
        }
        [_, _, ..] => tags.push("yap::pos::phrase".into()),
        _ => {}
    }
    // gram_frequencies is sorted most frequent first.
    let band = match pack.gram_frequencies.entries.get_index_of(&gram) {
        Some(rank) if rank < 100 => "top-100",
        Some(rank) if rank < 500 => "top-500",
        Some(rank) if rank < 1000 => "top-1000",
        Some(rank) if rank < 2000 => "top-2000",
        Some(rank) if rank < 5000 => "top-5000",
        Some(rank) if rank < 10000 => "top-10000",
        _ => "rare",
    };
    tags.push(format!("yap::frequency::{band}"));
    AnkiNote::Word {
        guid: guid(course, "word", word),
        note_id: id(course, "word-note", word),
        card_id: id(course, "word-card", word),
        word: word.to_owned(),
        definition,
        audio,
        tags,
    }
}

/// Tags every note gets: the source, the course, and the note kind, so a
/// learner can filter or suspend by any of them in Anki's browser.
fn note_tags(course: Course, kind: &str) -> Vec<String> {
    vec![
        "yap".into(),
        format!("yap::{}", course_code(course)),
        format!("yap::{kind}"),
    ]
}

/// Anki tags are space-separated and `::` nests them.
fn tag_segment(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join("_")
        .replace("::", ":")
}

// RFC 3986 unreserved characters stay literal; everything else (including
// UTF-8 bytes and '+') is percent-escaped, as a query component, not a form.
const COMPONENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');
fn component(value: &str) -> String {
    utf8_percent_encode(value, COMPONENT).to_string()
}
fn tts_url(language: Language, text: &str, hints: &[String], token: &str) -> String {
    let name = serde_json::to_value(language).unwrap();
    let mut url = format!(
        "{}/anki/tts?language={}&text={}&d={}",
        utils::ai_server_url(),
        component(name.as_str().unwrap()),
        component(&text.nfc().collect::<String>()),
        component(token)
    );
    for hint in hints {
        url.push_str(&format!("&hint={}", component(hint)));
    }
    url
}
fn definition(def: &GramDefinition) -> String {
    crate::definition_view(def.clone())
        .senses
        .into_iter()
        .map(|sense| sense.meaning)
        .collect::<Vec<_>>()
        .join("; ")
}
fn sentence_glosses(
    literals: &[Literal<String>],
    indices: &[usize],
    definitions: &[Option<GramDefinition>],
    course: Course,
) -> Vec<AnkiGloss> {
    let mut positions = vec![None; definitions.len()];
    let mut glosses: Vec<AnkiGloss> = Vec::new();
    for (literal, &group) in literals.iter().zip(indices) {
        let Some(def) = &definitions[group] else {
            continue;
        };
        let position = *positions[group].get_or_insert_with(|| {
            let position = glosses.len();
            let display = crate::definition_view(def.clone()).headword;
            glosses.push(AnkiGloss {
                text: String::new(),
                gloss: Some(definition(def)),
                url: Some(format!(
                    "https://yap.town/d/{}/{}/",
                    course.dictionary_slug(),
                    dictionary_entry_slug(&display)
                )),
            });
            position
        });
        glosses[position].text.push_str(&literal.word.text);
        glosses[position].text.push_str(&literal.whitespace);
    }
    for gloss in &mut glosses {
        gloss.text = gloss.text.trim_end().to_owned();
    }
    glosses
}

fn human_filename(course: Course, text: &str) -> String {
    format!("yap-word-{}.ogg", guid(course, "word", text))
}
fn poster_filename(imdb: &str) -> String {
    format!("yap-poster-{imdb}.jpg")
}

// The page never talks about "your level": Yap's level system is due a
// rethink, so the deck is described by the words it teaches instead.
/// How many posters the page shows.
const FILM_COUNT: usize = 24;
const PLACEMENT_INTRO: &str = "First, tell us which words you know. No account needed.";
const TOO_ADVANCED: &str = "You already know nearly every word Yap teaches. The deck will still catch gaps, but Yap's app will serve you better.";
const CLIPS_LOADING: &str = "Movie clips are still loading. Please try again.";
const NO_SENTENCES: &str = "No movie-clip sentences use only words you know yet.";
const PLACEMENT_COMPLETE_LABEL: &str = "Generate Anki deck";
// Shown while the deck is being built; the page has a minute or two to fill.
const BACKSTORY: &str = "I wanted language learning to be easier, so I made Yap. I still think the app is the best way to learn, but the same technology makes a really good Anki deck too. Yours is being built right now.";
const TRY_YAP_BODY: &str = "My goal with Yap was to combine spaced repetition with comprehensible input. Your placement test is saved, so if you start using Yap it'll pick up where you left off.";
const SAVE_DECK_BODY: &str = "If you create an account, I'll save your placement test so you can easily come back here and modify your deck (or update it with new cards).";
const DECK_DESCRIPTION: &str = "Made with Yap (https://yap.town/anki): real movie lines built from words you know, each with its clip and a recording.\n\nWhen these run out, Yap keeps going at https://yap.town.";

#[bridgerton::bridge]
impl Deck {
    pub fn anki_export_view(&self, starting_fresh: Option<bool>) -> AnkiExportView {
        let pack = &self.context.language_pack;
        let language = self.context.course.target_language;
        let known = Self::percent_known_in(
            &pack.gram_frequencies,
            self.get_comprehensible_written_grams(true),
            self.get_comprehensible_listening_grams(true),
        );
        let too_advanced = !pack.gram_frequencies.entries.is_empty() && known.all_available_learned;
        // Walk the clip manifest, not every translated sentence in the pack.
        let clip_sentence_count = clips::map_clip_sentences(language, |text| {
            let sentence = pack.string_rodeo.get(text)?;
            pack.sentence_is_comprehensible(sentence, None, &|_| true)
                .then_some(())
        })
        .len() as u32;
        let films = self
            .get_movie_metadata(clips::films_by_clip_count(language))
            .into_iter()
            .filter(|film| {
                pack.movies
                    .get(&film.id)
                    .is_some_and(|m| m.poster_bytes.is_some())
            })
            .take(FILM_COUNT)
            .collect();
        AnkiExportView {
            title: format!(
                "Build a custom Anki deck that reinforces your {language} with clips from famous movies."
            ),
            films,
            needs_placement: starting_fresh != Some(true)
                && !self.has_taken_placement_test()
                && self.num_cards_added() < 3,
            placement_intro: PLACEMENT_INTRO.into(),
            progress_label: format!("Essential {}", get_language_metadata(language).common_name),
            percent_known: known.percent_known,
            too_advanced,
            too_advanced_message: too_advanced.then(|| TOO_ADVANCED.into()),
            clips_loaded: clips::manifest_loaded(language),
            clip_sentence_count,
            course_flag: get_language_metadata(language).flag.clone(),
            course_label: language.to_string(),
            card_types_label: "Card types".into(),
            reading_label: "Reading".into(),
            listening_label: "Listening".into(),
            word_cards_label: "Word cards".into(),
            download_label: format!("Generate {language} deck"),
            placement_complete_label: PLACEMENT_COMPLETE_LABEL.into(),
            backstory: BACKSTORY.into(),
            backstory_signature: "Andre".into(),
            try_yap_heading: "If you like the deck, consider trying Yap".into(),
            try_yap_body: TRY_YAP_BODY.into(),
            try_yap_label: "Go to Yap".into(),
            enjoy_deck: "In the meantime, enjoy your Anki deck!".into(),
            save_deck_heading: "Save your deck".into(),
            save_deck_body: SAVE_DECK_BODY.into(),
            sign_up_label: "Create an account".into(),
            download_again_label: "Download again".into(),
        }
    }

    /// Bytes for a bundled file that lives in the language pack. TTS is not
    /// in the pack; the host fetches it from the manifest's URL instead.
    pub fn anki_bundled_media(&self, source: AnkiMediaSource) -> Option<Vec<u8>> {
        let language = self.context.course.target_language;
        match source {
            AnkiMediaSource::Poster { imdb_id } => self.get_movie_poster(imdb_id),
            AnkiMediaSource::HumanAudio { text } => {
                human_audio::lookup(language, &text).map(|audio| audio.bytes)
            }
            AnkiMediaSource::Tts { .. } => None,
        }
    }

    pub fn start_anki_deck_plan(
        &self,
        options: AnkiDeckOptions,
        token: String,
        timestamp_ms: f64,
    ) -> Result<AnkiDeckPlanner, Error> {
        AnkiDeckPlanner::new(self, options, DECK_SIZE, token, timestamp_ms)
    }
}

#[cfg(test)]
impl Deck {
    /// The synchronous test helper uses exactly the host's stepped path.
    fn plan_anki_deck(
        &self,
        options: AnkiDeckOptions,
        size: usize,
        token: String,
        timestamp_ms: f64,
    ) -> Result<AnkiDeckPlan, Error> {
        let planner = AnkiDeckPlanner::new(self, options, size, token, timestamp_ms)?;
        while !planner.step().done {}
        planner.finish()
    }
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnkiDeckStep {
    /// Exactly the new notes, including prerequisites, in final deck order.
    pub notes: Vec<AnkiNote>,
    pub sentences_chosen: u32,
    pub target_size: u32,
    pub done: bool,
}

/// A paused planner. Hosts yield between short steps; no learner state changes.
#[bridgerton::bridge(opaque)]
pub struct AnkiDeckPlanner {
    state: RefCell<PlannerState>,
}

struct PlannerState {
    seed: Deck,
    options: AnkiDeckOptions,
    token: String,
    size: usize,
    unindexed: Vec<Spur>,
    clip_sentence_ranks: FxHashMap<Spur, Vec<u32>>,
    /// Proper nouns per clip sentence. They count as known, so a beginner's
    /// shortest candidates are often a word plus a name ("De Niort ?").
    clip_sentence_names: FxHashMap<Spur, usize>,
    clip_sentences_of: FxHashMap<TaggedGram<SpurGram>, Vec<Spur>>,
    simulation: Option<DailySimulationIterator>,
    day: Option<DayChallengeIterator>,
    known_now: Option<RankBitset>,
    pending: VecDeque<TaggedGram<SpurGram>>,
    still_pending: VecDeque<TaggedGram<SpurGram>>,
    before: usize,
    empty_days: usize,
    done: bool,
    notes: Vec<AnkiNote>,
    bundled: Vec<AnkiBundledMedia>,
    used_sentences: BTreeSet<Spur>,
    used_words: BTreeSet<String>,
    /// Every gram a word note or sentence teaches, for the coverage figure.
    taught: FxHashSet<TaggedGram<SpurGram>>,
    posters: BTreeSet<String>,
}

impl AnkiDeckPlanner {
    fn new(
        deck: &Deck,
        options: AnkiDeckOptions,
        size: usize,
        token: String,
        timestamp_ms: f64,
    ) -> Result<Self, Error> {
        let language = deck.context.course.target_language;
        if !clips::manifest_loaded(language) {
            return Err(Error::new(CLIPS_LOADING));
        }
        if !timestamp_ms.is_finite() {
            return Err(Error::new("Invalid export time."));
        }
        let now = chrono::DateTime::from_timestamp_millis(timestamp_ms as i64)
            .ok_or_else(|| Error::new("Invalid export time."))?;
        let pack = &deck.context.language_pack;
        let unindexed = clips::map_clip_sentences(language, |text| pack.string_rodeo.get(text));
        Ok(Self {
            state: RefCell::new(PlannerState {
                seed: deck.clone(),
                options,
                token,
                size,
                unindexed,
                clip_sentence_ranks: FxHashMap::default(),
                clip_sentence_names: FxHashMap::default(),
                clip_sentences_of: FxHashMap::default(),
                simulation: None,
                day: Some(
                    deck.simulate_usage(now)
                        .with_new_cards_per_day(20)
                        .next_day(),
                ),
                known_now: None,
                pending: VecDeque::new(),
                still_pending: VecDeque::new(),
                before: 0,
                empty_days: 0,
                done: size == 0,
                notes: Vec::new(),
                bundled: Vec::new(),
                used_sentences: BTreeSet::new(),
                used_words: BTreeSet::new(),
                taught: FxHashSet::default(),
                posters: BTreeSet::new(),
            }),
        })
    }
}

#[bridgerton::bridge]
impl AnkiDeckPlanner {
    /// Index a small batch, answer one simulated challenge, or try one pending
    /// gram. A simulated day is deliberately not an atomic unit of work.
    pub fn step(&self) -> AnkiDeckStep {
        let mut state = self.state.borrow_mut();
        let before = state.notes.len();
        if !state.done {
            state.advance();
        }
        AnkiDeckStep {
            notes: state.notes[before..].to_vec(),
            sentences_chosen: state.used_sentences.len() as u32,
            target_size: state.size as u32,
            done: state.done,
        }
    }

    /// Finish only after `step` reports done; this never runs remaining work.
    pub fn finish(&self) -> Result<AnkiDeckPlan, Error> {
        let state = self.state.borrow();
        if !state.done {
            return Err(Error::new("Sentence selection is not finished."));
        }
        state.plan()
    }
}

/// A deck that barely moves the number isn't worth bragging about: an
/// advanced learner's new words are rare ones, so 300 sentences can add a
/// single point.
fn finish_message(before: f64, after: f64, name: &str) -> Option<String> {
    (after - before >= 3.0).then(|| {
        format!(
            "Once you finish this deck, you'll understand {after}% of everyday {name}, up from {before}%."
        )
    })
}

/// Known going in, or taught by the deck: every word note ships audio, so a
/// taught word counts for listening as well as reading.
#[derive(Clone, Copy)]
struct KnownOrTaught<'a, M> {
    known: M,
    taught: &'a FxHashSet<TaggedGram<SpurGram>>,
}
impl<M: GramMembership> GramMembership for KnownOrTaught<'_, M> {
    fn contains(self, gram: &TaggedGram<SpurGram>) -> bool {
        self.taught.contains(gram) || self.known.contains(gram)
    }
}

impl PlannerState {
    fn index_sentences(&mut self) {
        let pack = &self.seed.context.language_pack;
        let order = &pack.written_ease_order;
        for _ in 0..128 {
            let Some(sentence) = self.unindexed.pop() else {
                break;
            };
            if pack
                .translations
                .get(&sentence)
                .is_none_or(|t| t.is_empty())
            {
                continue;
            }
            let Some(encoded) = pack.encoded_sentences.get(&sentence) else {
                continue;
            };
            let learnable = encoded.grams.iter().filter_map(|g| match g {
                SentenceGram::Learnable(g) => Some(g),
                SentenceGram::Obvious(_) => None,
            });
            let multiword = encoded
                .multiword_terms
                .iter()
                .chain(&encoded.low_confidence_multiword_terms)
                .map(|m| &m.gram);
            let grams: Vec<&TaggedGram<SpurGram>> = learnable.chain(multiword).collect();
            let Some(ranks) = grams
                .iter()
                .map(|g| order.rank(g))
                .collect::<Option<Vec<u32>>>()
            else {
                continue;
            };
            for gram in grams {
                self.clip_sentences_of
                    .entry(*gram)
                    .or_default()
                    .push(sentence);
            }
            self.clip_sentence_ranks.insert(sentence, ranks);
            let names = encoded
                .grams
                .iter()
                .filter(|g| match g {
                    SentenceGram::Obvious(g) => {
                        g.resolve(&pack.gram_rodeo).gram.iter().any(|atom| {
                            matches!(
                                atom,
                                Atom::Tok(Word {
                                    word_type: WordType::Other(OtherWord {
                                        other_tag: OtherWordType::Propn
                                    }),
                                    ..
                                })
                            )
                        })
                    }
                    SentenceGram::Learnable(_) => false,
                })
                .count();
            self.clip_sentence_names.insert(sentence, names);
        }
        if self.unindexed.is_empty() && self.clip_sentence_ranks.is_empty() {
            self.done = true;
        }
    }

    fn advance(&mut self) {
        if !self.unindexed.is_empty() {
            self.index_sentences();
            return;
        }
        if self.clip_sentence_ranks.is_empty() {
            self.done = true;
            return;
        }
        if let Some(day) = &mut self.day {
            if day.next().is_none() {
                let simulation = self.day.take().unwrap().finish_day();
                self.pending.extend(
                    simulation
                        .last_introduced()
                        .iter()
                        .filter_map(|indicator| indicator.written_gram().copied()),
                );
                self.known_now = Some(
                    simulation
                        .deck()
                        .get_comprehensible_written_grams(false)
                        .rank_bitset(),
                );
                self.simulation = Some(simulation);
                self.before = self.used_sentences.len();
            }
            return;
        }
        if let Some(gram) = self.pending.pop_front() {
            self.choose_sentence(gram);
            self.done = self.used_sentences.len() == self.size;
            return;
        }
        self.pending = std::mem::take(&mut self.still_pending);
        self.empty_days = if self.before == self.used_sentences.len() {
            self.empty_days + 1
        } else {
            0
        };
        self.done = self.empty_days >= 60;
        if !self.done {
            self.day = Some(self.simulation.take().unwrap().next_day());
        }
    }

    fn choose_sentence(&mut self, gram: TaggedGram<SpurGram>) {
        let course = self.seed.context.course;
        let language = course.target_language;
        let pack = &self.seed.context.language_pack;
        let display = |gram: TaggedGram<SpurGram>| {
            gram.resolve(&pack.gram_rodeo)
                .resolve(&pack.string_rodeo)
                .to_display_string(language)
        };
        let known = self.seed.get_comprehensible_written_grams(false);
        let deck = self.simulation.as_ref().unwrap().deck();
        let known_now = self.known_now.as_ref().unwrap();
        let Self {
            clip_sentences_of,
            clip_sentence_ranks,
            clip_sentence_names,
            used_sentences,
            used_words,
            taught,
            notes,
            bundled,
            posters,
            options,
            token,
            still_pending,
            ..
        } = self;
        let word = display(gram);
        let Some(gram_rank) = pack.written_ease_order.rank(&gram) else {
            return;
        };
        // A candidate is a clip sentence containing the gram whose
        // every other gram the learner already knows.
        let mut candidates: Vec<Spur> = clip_sentences_of
            .get(&gram)
            .into_iter()
            .flatten()
            .copied()
            .filter(|s| !used_sentences.contains(s))
            .filter(|s| {
                clip_sentence_ranks[s]
                    .iter()
                    .all(|&r| r == gram_rank || known_now.contains(r))
            })
            .collect();
        candidates.sort_by_key(|s| {
            (
                deck.stats.sentences_reviewed.get(s).copied().unwrap_or(0),
                clip_sentence_names[s],
                pack.string_rodeo.resolve(s).chars().count(),
                pack.string_rodeo.resolve(s),
            )
        });
        let Some((sentence, challenge)) = candidates.into_iter().find_map(|s| {
            deck.translation_challenge_for_sentence(gram, s)
                .map(|c| (s, c))
        }) else {
            still_pending.push_back(gram);
            return;
        };
        let target_gloss = pack
            .gram_definitions
            .get(&gram)
            .map(definition)
            .unwrap_or_default();
        // Word notes come before the sentence: the target word, plus
        // every other word the learner did not know at export time
        // that the deck has not presented yet (whether the simulator
        // introduced it or it was already added but unlearned). One
        // word note per spelling; a later sense of the same spelling
        // still gets its sentence, whose own gloss carries that sense.
        let mut prerequisites = vec![gram];
        if let Some(encoded) = pack.encoded_sentences.get(&sentence) {
            let learnable = encoded.grams.iter().filter_map(|g| match g {
                SentenceGram::Learnable(g) => Some(*g),
                SentenceGram::Obvious(_) => None,
            });
            let multiword = encoded
                .multiword_terms
                .iter()
                .chain(&encoded.low_confidence_multiword_terms)
                .map(|m| m.gram);
            prerequisites.extend(
                learnable
                    .chain(multiword)
                    .filter(|g| *g != gram && !known.contains(g)),
            );
        }
        for prerequisite in prerequisites {
            taught.insert(prerequisite);
            let word = display(prerequisite);
            if options.word_cards && used_words.insert(word.clone()) {
                notes.push(word_note(pack, course, prerequisite, &word, token, bundled));
            }
        }
        let text = challenge.target_language;
        let clip = clips::clip_for_sentence(language, &text).unwrap();
        let imdb = clips::clip_film(&clip.clip_id).to_owned();
        let movie = pack.movies.get(&imdb);
        let film_tag = match movie {
            Some(movie) => match movie.year {
                Some(year) => format!("{} {year}", movie.title),
                None => movie.title.clone(),
            },
            None => imdb.clone(),
        };
        let poster = movie
            .and_then(|m| m.poster_bytes.as_ref())
            .map(|_| poster_filename(&imdb));
        if let Some(filename) = &poster
            && posters.insert(filename.clone())
        {
            bundled.push(AnkiBundledMedia {
                filename: filename.clone(),
                source: AnkiMediaSource::Poster {
                    imdb_id: imdb.clone(),
                },
            });
        }
        let url = tts_url(
            language,
            &text,
            &challenge.audio.request.verification_hints,
            token,
        );
        // Every sentence ships its recording: a Listening card cannot
        // be answered without it, and the Translate card shares the
        // same file, so there is nothing to save by streaming.
        let filename = format!("yap-sentence-{}.mp3", guid(course, "sentence", &text));
        bundled.push(AnkiBundledMedia {
            filename: filename.clone(),
            source: AnkiMediaSource::Tts { url },
        });
        let tts = filename;
        let include_listening = !matches!(options.card_types, AnkiCardTypes::Reading);
        let glosses = sentence_glosses(
            &challenge.target_language_literals,
            &challenge.literal_gram_indices,
            &challenge.gram_definitions_for_lookup,
            course,
        );
        notes.push(AnkiNote::Sentence {
            guid: guid(course, "sentence", &text),
            note_id: id(course, "sentence-note", &text),
            card_id: id(course, "sentence-card", &text),
            sentence: text,
            translation: challenge.native_translations.join(" / "),
            target_word: word.clone(),
            target_gloss,
            glosses,
            source: AnkiSource {
                title: movie.map_or_else(|| imdb.clone(), |m| m.title.clone()),
                year: movie.and_then(|m| m.year),
                imdb_id: imdb,
                poster_filename: poster,
            },
            clip_url: format!(
                "{CLIPS_ORIGIN}/{}/{}/lo.mp4?d={}",
                language.code(),
                component(&clip.clip_id),
                component(token)
            ),
            tts,
            include_reading: !matches!(options.card_types, AnkiCardTypes::Listening),
            include_listening,
            tags: {
                let mut tags = note_tags(course, "sentence");
                tags.push(format!("yap::film::{}", tag_segment(&film_tag)));
                tags
            },
        });
        used_sentences.insert(sentence);
    }

    fn plan(&self) -> Result<AnkiDeckPlan, Error> {
        let course = self.seed.context.course;
        let language = course.target_language;
        if self.used_sentences.is_empty() {
            return Err(Error::new(NO_SENTENCES));
        }
        let sentence_count = self.used_sentences.len() as u32;
        let word_count = self.used_words.len() as u32;
        let pack = &self.seed.context.language_pack;
        let everyday = |taught| {
            let written = KnownOrTaught {
                known: self.seed.get_comprehensible_written_grams(true),
                taught,
            };
            let listening = KnownOrTaught {
                known: self.seed.get_comprehensible_listening_grams(true),
                taught,
            };
            Deck::percent_known_in(&pack.gram_frequencies, written, listening)
                .percent_known
                .round()
        };
        let (before, after) = (everyday(&FxHashSet::default()), everyday(&self.taught));
        let finish_message =
            finish_message(before, after, &get_language_metadata(language).common_name);
        Ok(AnkiDeckPlan {
            language,
            course_code: course_code(course),
            deck_name: if course.native_language == Language::English {
                format!("Yap • {language}")
            } else {
                format!("Yap • {language} ({})", course.native_language)
            },
            deck_description: DECK_DESCRIPTION.into(),
            deck_id: id(course, "deck", ""),
            sentence_model_id: id(course, "sentence-model", ""),
            word_model_id: id(course, "word-model", ""),
            notes: self.notes.clone(),
            bundled: self.bundled.clone(),
            finish_message,
            stats: AnkiDeckStats {
                sentence_count,
                word_count,
                card_count: word_count
                    + sentence_count
                        * if matches!(self.options.card_types, AnkiCardTypes::Both) {
                            2
                        } else {
                            1
                        },
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CardIndicator;
    use language_utils::{
        Atom, ConsolidatedLanguageData, Course, DictionaryEntry, Gram, GramFrequencyEntry,
        GramFrequencyList, GramVocabEntry, Heteronym, PartOfSpeech, SentenceGram, SentenceGrams,
        TaggedGram, TargetToNativeWord, Word, WordType, language_pack::LanguagePack,
    };
    use std::sync::Arc;
    use weapon::AppState;

    fn options() -> AnkiDeckOptions {
        AnkiDeckOptions {
            card_types: AnkiCardTypes::Both,
            word_cards: true,
        }
    }
    fn row(sentence: String, clip_id: String) -> clips::ClipRow {
        clips::ClipRow {
            sentence,
            clip_id,
            duration_ms: 1000,
            critical: clips::ClipCritical {
                start_ms: 0,
                end_ms: 1000,
            },
            clear_before_ms: None,
            clear_after_ms: None,
            pad_before_ms: None,
            pad_after_ms: None,
            aspect_ratio: None,
            lo_bytes: 0,
        }
    }
    fn fixture() -> Deck {
        let course = Course {
            target_language: Language::English,
            native_language: Language::French,
        };
        let entries: Vec<_> = (0..64)
            .map(|i| {
                let text = format!("word{i:02}");
                TaggedGram {
                    gram: Gram(vec![Atom::Tok(Word {
                        text: text.clone(),
                        word_type: WordType::Heteronym(Heteronym {
                            word: text.clone(),
                            lemma: text,
                            pos: PartOfSpeech::Noun,
                        }),
                    })]),
                    sense: None,
                }
            })
            .collect();
        let sentences: Vec<_> = entries
            .iter()
            .flat_map(|gram| {
                let text = gram.to_display_string(Language::English);
                [1, 2].map(|count| {
                    (
                        std::iter::repeat_n(text.as_str(), count)
                            .collect::<Vec<_>>()
                            .join(" "),
                        SentenceGrams {
                            grams: vec![SentenceGram::Learnable(gram.clone()); count],
                            capitalize_first: false,
                            multiword_terms: vec![],
                            low_confidence_multiword_terms: vec![],
                        },
                    )
                })
            })
            .collect();
        let data = ConsolidatedLanguageData {
            target_language_sentences: sentences.iter().map(|(text, _)| text.clone()).collect(),
            translations: sentences
                .iter()
                .map(|(text, _)| (text.clone(), vec![format!("translation {text}")]))
                .collect(),
            encoded_sentences: sentences,
            gram_vocabulary: entries
                .iter()
                .map(|g| GramVocabEntry {
                    atoms: g.gram.clone(),
                    frequency: 100,
                })
                .collect(),
            gram_frequencies: GramFrequencyList {
                entries: entries
                    .iter()
                    .enumerate()
                    .map(|(i, g)| GramFrequencyEntry {
                        gram: g.clone(),
                        count: 100 - i as u32,
                        direct_count: 100 - i as u32,
                        disambiguation_key: 0,
                    })
                    .collect(),
                total_count: 4384,
            },
            gram_dictionary: entries
                .iter()
                .map(|g| {
                    (
                        g.clone(),
                        DictionaryEntry {
                            target_language_word: g.to_display_string(Language::English),
                            definitions: vec![TargetToNativeWord {
                                native: "meaning".into(),
                                note: None,
                                example_sentence_target_language: String::new(),
                                example_sentence_native_language: String::new(),
                                cognate: false,
                                false_cognate: false,
                            }],
                            morphology: vec![],
                            segments: vec![],
                        },
                    )
                })
                .collect(),
            ..Default::default()
        };
        let context = crate::Context {
            study_goal: None,
            language_pack: Arc::new(LanguagePack::new(data, course)),
            course,
            timezone: chrono::FixedOffset::east_opt(0).unwrap(),
        };
        Deck::finalize(crate::DeckState::new(), &context)
    }
    fn publish(pack: &LanguagePack, course: Course) {
        let language = course.target_language;
        clips::publish_manifest(
            language,
            pack.encoded_sentences
                .keys()
                .map(|s| {
                    let text = pack.string_rodeo.resolve(s).to_owned();
                    let imdb = pack
                        .sentence_sources
                        .get(s)
                        .and_then(|source| source.movie_ids.first())
                        .map(String::as_str)
                        .unwrap_or("tt0000001");
                    row(
                        text.clone(),
                        format!("{imdb}-{}-0", guid(course, "sentence", &text)),
                    )
                })
                .collect(),
        );
    }
    #[test]
    fn anki_glosses_group_interleaved_literals_in_first_appearance_order() {
        let definition = |word: &str| {
            Some(GramDefinition::Dictionary(DictionaryEntry {
                target_language_word: word.into(),
                definitions: vec![TargetToNativeWord {
                    native: format!("meaning of {word}"),
                    note: None,
                    example_sentence_target_language: String::new(),
                    example_sentence_native_language: String::new(),
                    cognate: false,
                    false_cognate: false,
                }],
                morphology: vec![],
                segments: vec![],
            }))
        };
        let literals: Vec<_> = [("take", " "), ("it", " "), ("off", "  "), ("!", "")]
            .into_iter()
            .map(|(text, whitespace)| Literal {
                word: Word {
                    text: text.into(),
                    word_type: WordType::Other(language_utils::OtherWord {
                        other_tag: if text == "!" {
                            language_utils::OtherWordType::Punct
                        } else {
                            language_utils::OtherWordType::X
                        },
                    }),
                },
                whitespace: whitespace.into(),
            })
            .collect();
        let glosses = sentence_glosses(
            &literals,
            &[2, 0, 2, 1],
            &[definition("it"), None, definition("take off")],
            Course {
                target_language: Language::English,
                native_language: Language::French,
            },
        );
        assert_eq!(glosses.len(), 2);
        assert_eq!(glosses[0].text, "take off");
        assert_eq!(glosses[0].gloss.as_deref(), Some("meaning of take off"));
        assert_eq!(glosses[1].text, "it");
        assert_eq!(glosses[1].gloss.as_deref(), Some("meaning of it"));
        assert!(
            glosses[0]
                .url
                .as_ref()
                .unwrap()
                .ends_with(&format!("/{}/", dictionary_entry_slug("take off")))
        );
    }

    #[test]
    fn anki_plan_is_deterministic_and_does_not_change_seed() {
        let deck = fixture();
        publish(&deck.context.language_pack, deck.context.course);
        let a = deck
            .plan_anki_deck(options(), 55, "a+b&雪".into(), 1_700_000_000_000.0)
            .unwrap();
        let b = deck
            .plan_anki_deck(options(), 55, "a+b&雪".into(), 1_700_000_000_000.0)
            .unwrap();
        assert_eq!(
            serde_json::to_value(&a).unwrap(),
            serde_json::to_value(&b).unwrap()
        );
        assert_eq!(deck.num_cards_added(), 0);
        assert_eq!(deck.stats.total_reviews, 0);
        assert_eq!(a.stats.sentence_count, 55);
        assert_eq!(a.stats.word_count, 55);
        assert_eq!(a.stats.card_count, 165);
        // Every sentence and word recording is bundled, whatever the card types.
        assert_eq!(
            a.bundled
                .iter()
                .filter(|m| matches!(m.source, AnkiMediaSource::Tts { .. }))
                .count(),
            110
        );
        let mut sentences = BTreeSet::new();
        let mut ids = BTreeSet::new();
        for pair in a.notes.chunks_exact(2) {
            let AnkiNote::Word { word, audio, .. } = &pair[0] else {
                panic!("word first")
            };
            let AnkiNote::Sentence {
                sentence,
                clip_url,
                tts,
                note_id,
                card_id,
                guid,
                glosses,
                ..
            } = &pair[1]
            else {
                panic!("sentence second")
            };
            assert!(sentences.insert(sentence));
            assert_eq!(word, sentence, "shortest sentence wins");
            let entry = a.bundled.iter().find(|m| &m.filename == audio).unwrap();
            assert_eq!(
                audio,
                &format!(
                    "yap-word-{}.mp3",
                    super::guid(deck.context.course, "word", word)
                )
            );
            assert!(
                matches!(&entry.source, AnkiMediaSource::Tts { url } if url == &tts_url(Language::English, word, &[], "a+b&雪"))
            );
            assert!(clip_url.starts_with("https://clips.yap.town/eng/"));
            assert!(clip_url.ends_with("?d=a%2Bb%26%E9%9B%AA"));
            assert!(!guid.contains(['+', '/', '=']));
            for id in [*note_id, *card_id, *card_id + 1] {
                assert!(id > 0 && id < 1_i64 << 53);
                assert!(ids.insert(id));
            }
            assert_eq!(glosses[0].gloss.as_deref(), Some("meaning"));
            assert!(
                glosses[0]
                    .url
                    .as_ref()
                    .unwrap()
                    .starts_with("https://yap.town/d/")
            );
            let entry = a.bundled.iter().find(|m| &m.filename == tts).unwrap();
            let AnkiMediaSource::Tts { url } = &entry.source else {
                panic!("bundled sentence audio is fetched TTS")
            };
            assert!(url.contains("language=English&text="));
            assert!(!url.contains("&hint="));
        }
        let reading = deck
            .plan_anki_deck(
                AnkiDeckOptions {
                    card_types: AnkiCardTypes::Reading,
                    word_cards: true,
                },
                55,
                "new token".into(),
                1_700_000_000_000.0,
            )
            .unwrap();
        for (a, b) in a.notes.iter().zip(reading.notes) {
            if let (
                AnkiNote::Sentence {
                    guid: ga,
                    note_id: na,
                    card_id: ca,
                    ..
                },
                AnkiNote::Sentence {
                    guid: gb,
                    note_id: nb,
                    card_id: cb,
                    include_reading,
                    include_listening,
                    ..
                },
            ) = (a, b)
            {
                assert_eq!(ga, &gb);
                assert_eq!(*na, nb);
                assert_eq!(*ca, cb);
                assert!(include_reading);
                assert!(!include_listening);
            }
        }
    }
    #[test]
    fn finish_message_only_for_real_gains() {
        assert_eq!(finish_message(90.0, 92.0, "French"), None);
        assert_eq!(
            finish_message(0.0, 24.0, "French").as_deref(),
            Some(
                "Once you finish this deck, you'll understand 24% of everyday French, up from 0%."
            )
        );
    }

    #[test]
    fn anki_notes_are_tagged() {
        assert_eq!(
            tag_segment("Star Wars: Episode  IV::X 1977"),
            "Star_Wars:_Episode_IV:X_1977"
        );
        let deck = fixture();
        publish(&deck.context.language_pack, deck.context.course);
        let plan = deck
            .plan_anki_deck(options(), 55, "token".into(), 1_700_000_000_000.0)
            .unwrap();
        for note in &plan.notes {
            let (tags, kind) = match note {
                AnkiNote::Sentence { tags, .. } => (tags, "yap::sentence"),
                AnkiNote::Word { tags, .. } => (tags, "yap::word"),
            };
            assert!(
                tags.iter().all(|tag| !tag.contains(char::is_whitespace)),
                "{tags:?}"
            );
            assert!(tags.contains(&"yap::eng-fra".into()), "{tags:?}");
            assert!(tags.contains(&kind.into()), "{tags:?}");
            let specific = match note {
                AnkiNote::Sentence { .. } => "yap::film::",
                AnkiNote::Word { .. } => "yap::frequency::",
            };
            assert!(tags.iter().any(|tag| tag.starts_with(specific)), "{tags:?}");
        }
    }

    #[test]
    fn anki_without_word_cards_has_only_sentences() {
        let deck = fixture();
        publish(&deck.context.language_pack, deck.context.course);
        let plan = deck
            .plan_anki_deck(
                AnkiDeckOptions {
                    word_cards: false,
                    ..options()
                },
                55,
                "token".into(),
                1_700_000_000_000.0,
            )
            .unwrap();
        assert!(plan.stats.sentence_count > 0);
        assert_eq!(plan.stats.word_count, 0);
        assert!(
            plan.notes
                .iter()
                .all(|note| matches!(note, AnkiNote::Sentence { .. }))
        );
    }

    #[test]
    fn anki_steps_preserve_note_order_and_plan() {
        let deck = fixture();
        publish(&deck.context.language_pack, deck.context.course);
        let planner =
            AnkiDeckPlanner::new(&deck, options(), 55, "token".into(), 1_700_000_000_000.0)
                .unwrap();
        assert!(planner.finish().is_err());
        let mut notes = Vec::new();
        let mut chosen = 0;
        loop {
            let step = planner.step();
            assert_eq!(step.target_size, 55);
            assert!(step.sentences_chosen >= chosen);
            assert!(step.sentences_chosen <= chosen + 1);
            chosen = step.sentences_chosen;
            notes.extend(step.notes);
            if step.done {
                break;
            }
        }
        let plan = planner.finish().unwrap();
        assert_eq!(
            serde_json::to_value(notes).unwrap(),
            serde_json::to_value(&plan.notes).unwrap()
        );
        let done = planner.step();
        assert!(done.done);
        assert!(done.notes.is_empty());
        assert_eq!(done.sentences_chosen, 55);
        assert_eq!(deck.num_cards_added(), 0);
    }

    #[test]
    fn anki_errors_and_view() {
        let deck = fixture();
        // A different language has no manifest in this test's thread-local mirror.
        assert!(!deck.anki_export_view(None).clips_loaded);
        assert!(
            deck.plan_anki_deck(options(), 1, "token".into(), 1_700_000_000_000.0)
                .is_err()
        );
        clips::publish_manifest(Language::English, vec![]);
        assert!(deck.anki_export_view(None).clips_loaded);
        assert_eq!(deck.anki_export_view(None).clip_sentence_count, 0);
        assert!(deck.anki_export_view(None).needs_placement);
        assert!(deck.anki_export_view(Some(false)).needs_placement);
        assert!(!deck.anki_export_view(Some(true)).needs_placement);
        assert!(
            deck.plan_anki_deck(options(), 1, "token".into(), 1_700_000_000_000.0)
                .is_err()
        );
        assert!(
            deck.anki_bundled_media(AnkiMediaSource::HumanAudio {
                text: "not-media".into()
            })
            .is_none()
        );
    }
    #[test]
    fn anki_tts_normalizes_and_preserves_hint_order() {
        let url = tts_url(
            Language::French,
            "E\u{301}lodie & Paris?",
            &["Élodie".into(), "Paris".into(), "Élodie".into()],
            "a+b",
        );
        assert!(url.ends_with("language=French&text=%C3%89lodie%20%26%20Paris%3F&d=a%2Bb&hint=%C3%89lodie&hint=Paris&hint=%C3%89lodie"));
    }
    #[test]
    fn anki_bundled_human_audio_and_poster() {
        let mut deck = fixture();
        let pack = Arc::get_mut(&mut deck.context.language_pack).unwrap();
        pack.human_audio.insert(
            language_utils::VoiceActor {
                name: "Fixture actor".into(),
                compensation: language_utils::Compensation::Volunteer,
            },
            [(
                "word00".into(),
                language_utils::Audio {
                    bytes: b"OggSfixture".to_vec(),
                },
            )]
            .into_iter()
            .collect(),
        );
        pack.movies.insert(
            "tt0000001".into(),
            language_utils::MovieMetadata {
                id: "tt0000001".into(),
                title: "Fixture movie".into(),
                year: Some(2026),
                original_language: Some("en".into()),
                rotten_tomatoes_score: None,
                poster_bytes: Some(vec![1, 2, 3]),
            },
        );
        human_audio::register(Language::English, &deck.context.language_pack);
        publish(&deck.context.language_pack, deck.context.course);
        let plan = deck
            .plan_anki_deck(options(), 3, "token".into(), 1_700_000_000_000.0)
            .unwrap();
        assert_eq!(
            plan.bundled
                .iter()
                .filter(|m| m.filename.starts_with("yap-poster-"))
                .count(),
            1
        );
        let filename = human_filename(deck.context.course, "word00");
        assert!(plan.notes.iter().any(|note| matches!(note, AnkiNote::Word { word, audio, .. } if word == "word00" && audio == &filename)));
        assert!(plan.bundled.iter().any(|m| m.filename == filename
            && matches!(&m.source, AnkiMediaSource::HumanAudio { text } if text == "word00")));
        assert_eq!(
            deck.anki_bundled_media(AnkiMediaSource::HumanAudio {
                text: "word00".into()
            }),
            Some(b"OggSfixture".to_vec())
        );
        assert_eq!(
            deck.anki_bundled_media(AnkiMediaSource::Poster {
                imdb_id: "tt0000001".into()
            }),
            Some(vec![1, 2, 3])
        );
        assert!(plan.notes.iter().any(|note| matches!(note, AnkiNote::Sentence { source, .. } if source.title == "Fixture movie" && source.year == Some(2026))));
    }

    #[test]
    fn anki_advanced_means_all_available_not_all_unfiltered_words() {
        let mut deck = fixture();
        let mut state = crate::DeckState::new();
        for gram in deck.context.language_pack.gram_frequencies.entries.keys() {
            for indicator in [
                CardIndicator::WrittenGram { gram: *gram },
                CardIndicator::ListeningGram { gram: gram.gram },
            ] {
                state.cards.insert(
                    indicator,
                    crate::CardData::Added {
                        fsrs_card: rs_fsrs::Card::new(chrono::Utc::now()),
                    },
                );
            }
        }
        deck = Deck::finalize(state, &deck.context);
        // The corpus denominator includes grams outside this pack's teachable inventory.
        Arc::get_mut(&mut deck.context.language_pack)
            .unwrap()
            .gram_frequencies
            .total_count = 1_000_000;
        assert!(deck.get_percent_of_words_known() < 1.0);
        assert!(deck.anki_export_view(None).too_advanced);
        assert!(deck.anki_export_view(None).too_advanced_message.is_some());
    }

    fn assert_comprehensible_matches_brute_force(deck: &Deck) {
        let pack = &deck.context.language_pack;
        for planned in [false, true] {
            let written = deck.get_comprehensible_written_grams(planned);
            let listening = deck.get_comprehensible_listening_grams(planned);
            let mut expected_written = BTreeSet::new();
            let mut expected_listening = BTreeSet::new();
            for gram in pack.gram_frequencies.entries.keys() {
                for (indicator, expected, actual) in [
                    (
                        crate::CardIndicator::WrittenGram { gram: *gram },
                        &mut expected_written,
                        written.contains(gram),
                    ),
                    (
                        crate::CardIndicator::ListeningGram { gram: gram.gram },
                        &mut expected_listening,
                        listening.contains(gram),
                    ),
                ] {
                    let known = deck.context.is_comprehensible(
                        &indicator,
                        deck.cards.get(&indicator),
                        &deck.regressions,
                        planned,
                    );
                    assert_eq!(actual, known, "{indicator:?}, planned={planned}");
                    if known {
                        expected.insert(*gram);
                    }
                }
            }
            let written_items: Vec<_> = written.iter().collect();
            let listening_items: Vec<_> = listening.iter().collect();
            assert_eq!(
                written_items.len(),
                expected_written.len(),
                "no duplicate written entries"
            );
            assert_eq!(
                listening_items.len(),
                expected_listening.len(),
                "no duplicate listening senses"
            );
            assert_eq!(
                written_items.into_iter().collect::<BTreeSet<_>>(),
                expected_written
            );
            assert_eq!(
                listening_items.into_iter().collect::<BTreeSet<_>>(),
                expected_listening
            );
        }
    }

    fn exercise_comprehensibility(deck: Deck) {
        use crate::{CardData, CardIndicator, DeckState, PlacementTest};
        use lasso::Key;

        // No regression means no predictions, not an all-known suffix.
        assert!(deck.regressions.target_language_regression.is_none());
        assert!(deck.regressions.listening_regression.is_none());
        assert_comprehensible_matches_brute_force(&deck);
        let pack = &deck.context.language_pack;
        let order: Vec<_> = pack.written_ease_order.iter_from(0).collect();
        let mut state = DeckState::new();
        let mut placement = PlacementTest {
            known_words: vec![],
            unknown_words: vec![],
        };
        for (i, gram) in order.iter().enumerate().step_by((order.len() / 100).max(1)) {
            let text = pack
                .resolve_gram(&gram.gram)
                .to_display_string(deck.context.course.target_language);
            if i < order.len() / 2 {
                placement.unknown_words.push(text);
            } else {
                placement.known_words.push(text);
            }
        }
        // The synthetic Anki fixture has no NLP word lookup. Reviewed cards
        // also train the real finalize path directly from their gram eases.
        for (i, gram) in order.iter().enumerate().step_by(3) {
            let mut fsrs_card = rs_fsrs::Card::new(chrono::Utc::now());
            fsrs_card.state = rs_fsrs::State::Review;
            fsrs_card.early_lapses = i32::from(i < order.len() / 2);
            let card = CardData::Added { fsrs_card };
            state
                .cards
                .insert(CardIndicator::WrittenGram { gram: *gram }, card.clone());
            state
                .cards
                .insert(CardIndicator::ListeningGram { gram: gram.gram }, card);
        }
        state.placement_test_results = Some(placement);
        let predicted = Deck::finalize(state.clone(), &deck.context);
        assert_comprehensible_matches_brute_force(&predicted);
        assert!(
            predicted
                .get_comprehensible_written_grams(false)
                .iter()
                .any(|gram| !predicted
                    .cards
                    .contains_key(&CardIndicator::WrittenGram { gram })),
            "exercise a nonempty prediction suffix"
        );
        assert!(
            predicted
                .get_comprehensible_listening_grams(false)
                .iter()
                .any(|gram| !predicted
                    .cards
                    .contains_key(&CardIndicator::ListeningGram { gram: gram.gram }))
        );
        let regression = predicted
            .regressions
            .target_language_regression
            .as_ref()
            .unwrap();
        let probabilities: Vec<_> = order
            .iter()
            .map(|gram| {
                let rank = pack.written_ease_order.rank(gram).unwrap();
                regression
                    .interpolate(pack.written_ease_order.ease_at(rank).unwrap())
                    .unwrap()
            })
            .collect();
        for pair in probabilities.windows(2) {
            // f32 box integration can introduce rounding at the last bit.
            assert!(
                pair[0] <= pair[1] + 1e-6,
                "nonmonotone interpolation: {pair:?}"
            );
        }

        // Every FSRS state, both Added and Ghost, at both ends of the ease order.
        // In particular a present New/Ghost card overrides a positive prediction.
        for (i, gram) in order
            .iter()
            .take(8)
            .chain(order.iter().rev().take(8))
            .enumerate()
        {
            let mut fsrs_card = rs_fsrs::Card::new(chrono::Utc::now());
            fsrs_card.state = [
                rs_fsrs::State::New,
                rs_fsrs::State::Learning,
                rs_fsrs::State::Relearning,
                rs_fsrs::State::Review,
            ][i % 4];
            let card = if i % 8 < 4 {
                CardData::Added { fsrs_card }
            } else {
                CardData::Ghost { fsrs_card }
            };
            state
                .cards
                .insert(CardIndicator::WrittenGram { gram: *gram }, card.clone());
            state
                .cards
                .insert(CardIndicator::ListeningGram { gram: gram.gram }, card);
        }
        let absent = TaggedGram {
            gram: language_utils::SpurGram::try_from_usize(1_000_000).unwrap(),
            sense: None,
        };
        let stale_sense = TaggedGram {
            gram: order[0].gram,
            sense: std::num::NonZeroU32::new(u32::MAX),
        };
        for indicator in [
            CardIndicator::WrittenGram { gram: absent },
            CardIndicator::ListeningGram { gram: absent.gram },
            CardIndicator::WrittenGram { gram: stale_sense },
            CardIndicator::LetterPronunciation {
                pattern: lasso::Spur::try_from_usize(1_000_000).unwrap(),
                position: language_utils::PatternPosition::Anywhere,
            },
        ] {
            state.cards.insert(
                indicator,
                CardData::Added {
                    fsrs_card: rs_fsrs::Card::new(chrono::Utc::now()),
                },
            );
        }
        let deck = Deck::finalize(state, &deck.context);
        assert_comprehensible_matches_brute_force(&deck);
        for planned in [false, true] {
            for absent in [absent, stale_sense] {
                assert!(
                    !deck
                        .get_comprehensible_written_grams(planned)
                        .contains(&absent)
                );
                assert!(
                    !deck
                        .get_comprehensible_listening_grams(planned)
                        .contains(&absent)
                );
            }
        }
    }

    #[test]
    fn comprehensible_prediction_threshold_is_inclusive() {
        use pav_regression::{IsotonicRegression, Point, SmoothRegression, UnitWeight};
        let mut deck = fixture();
        for probability in [0.0, 0.799, 0.80, 1.0] {
            let points =
                [-10.0, 20.0].map(|ease| Point::new_with_weight(ease, probability, UnitWeight));
            let regression = SmoothRegression::from_regression(
                IsotonicRegression::new_ascending(&points).unwrap(),
                1.0,
            );
            deck.regressions = crate::Regressions {
                target_language_regression: Some(regression.clone()),
                listening_regression: Some(regression),
            };
            deck.comprehensible = crate::CachedComprehensibleGrams::new(
                &deck.context.language_pack,
                &deck.regressions,
                deck.cards.iter(),
            );
            assert_comprehensible_matches_brute_force(&deck);
            let expected = if probability >= 0.80 {
                deck.context.language_pack.gram_frequencies.entries.len()
            } else {
                0
            };
            for planned in [false, true] {
                assert_eq!(
                    deck.get_comprehensible_written_grams(planned)
                        .iter()
                        .count(),
                    expected
                );
                assert_eq!(
                    deck.get_comprehensible_listening_grams(planned)
                        .iter()
                        .count(),
                    expected
                );
            }
        }
    }

    #[test]
    fn comprehensible_views_match_synthetic_anki_pack() {
        let mut deck = fixture();
        // Add senses with different eases and an equal-ease tie. Build the runtime
        // indices through the same from_parts path used by real pack loading.
        let pack = Arc::try_unwrap(deck.context.language_pack).unwrap();
        let (mut core, sentences) = pack.split();
        // Give the tiny fixture a broad ease range so its fitted regressions
        // predict both known and unknown unadded entries after smoothing.
        for i in 0..core.gram_frequencies.entries.len() {
            let (_, frequency) = core.gram_frequencies.entries.get_index_mut(i).unwrap();
            frequency.ease = frequency.count as f32 / 5.0;
        }
        let (first, frequency) = core.gram_frequencies.entries.get_index(0).unwrap();
        let mut frequency = *frequency;
        let mut sense = *first;
        sense.sense = std::num::NonZeroU32::new(1);
        frequency.count = 1;
        frequency.direct_count = 1;
        frequency.ease += 2.0;
        core.gram_frequencies.entries.insert(sense, frequency);
        sense.sense = std::num::NonZeroU32::new(2);
        core.gram_frequencies.entries.insert(sense, frequency);
        let pack = LanguagePack::from_parts(core, Some(sentences));
        assert_eq!(
            pack.gram_frequency_total(sense.gram).unwrap().ease,
            frequency.ease
        );
        assert_eq!(
            pack.listening_ease_order
                .ease_at(pack.listening_ease_order.rank(&sense.gram).unwrap()),
            Some(frequency.ease)
        );
        deck.context.language_pack = Arc::new(pack);
        deck = Deck::finalize(crate::DeckState::new(), &deck.context);
        exercise_comprehensibility(deck);
    }

    #[test]
    fn comprehensible_views_match_real_french_pack() {
        exercise_comprehensibility(Deck::default());
    }

    #[test]
    fn anki_french_pack_sample() {
        // Real pack and simulator; an offline manifest fixture gives every pack sentence a clip.
        // This exercises selection, not the availability of the live published corpus.
        let deck = Deck::default();
        publish(&deck.context.language_pack, deck.context.course);
        let plan = deck
            .plan_anki_deck(options(), 10, "offline-test".into(), 1_700_000_000_000.0)
            .unwrap();
        assert_eq!(plan.stats.sentence_count, 10);
        println!("FIRST TEN ANKI NOTES (real French pack, offline manifest fixture):");
        for note in plan.notes.iter().take(10) {
            println!("{}", serde_json::to_string(note).unwrap());
        }
    }
}
