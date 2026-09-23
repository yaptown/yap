//! A host-independent Anki recipe. Hosts package these notes and media, not learning logic.
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use bridgerton::Error;
use language_utils::{
    CLIPS_ORIGIN, Course, GramDefinition, Language, Literal, SentenceGram, SpurGram, TaggedGram,
    dictionary_entry_slug, language_pack::LanguagePack,
};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use unicode_normalization::UnicodeNormalization;
use xxhash_rust::xxh3::xxh3_64;

use crate::{Deck, clips, human_audio, utils};

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
    pub size: u32,
    pub card_types: AnkiCardTypes,
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
    pub deck_id: i64,
    pub sentence_model_id: i64,
    pub word_model_id: i64,
    pub notes: Vec<AnkiNote>,
    pub bundled: Vec<AnkiBundledMedia>,
    pub stats: AnkiDeckStats,
}

#[bridgerton::bridge(transparent)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnkiDeckStats {
    pub level: u32,
    pub total_levels: u32,
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
        tts: AnkiAudio,
        include_reading: bool,
        include_listening: bool,
    },
    Word {
        guid: String,
        note_id: i64,
        card_id: i64,
        word: String,
        definition: String,
        audio: AnkiAudio,
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
#[serde(tag = "type")]
pub enum AnkiAudio {
    Bundled { filename: String },
    Streamed { url: String },
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
    pub needs_placement: bool,
    pub level_line: String,
    pub level: u32,
    pub total_levels: u32,
    pub too_advanced: bool,
    pub too_advanced_message: Option<String>,
    pub clips_loaded: bool,
    pub clip_sentence_count: u32,
    pub language_name: String,
    pub download_label: String,
}

fn validate(options: &AnkiDeckOptions) -> Result<(), Error> {
    if options.size == 0 {
        return Err(Error::new("Choose at least one sentence."));
    }
    Ok(())
}

#[bridgerton::bridge]
pub async fn mint_anki_deck(
    options: AnkiDeckOptions,
    access_token: Option<String>,
) -> Result<MintedAnkiDeck, Error> {
    validate(&options)?;
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
        AnkiAudio::Bundled { filename }
    } else {
        AnkiAudio::Streamed {
            url: tts_url(course.target_language, word, &[], token),
        }
    };
    AnkiNote::Word {
        guid: guid(course, "word", word),
        note_id: id(course, "word-note", word),
        card_id: id(course, "word-card", word),
        word: word.to_owned(),
        definition,
        audio,
    }
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

const TITLE: &str = "Sentence mining, already done";
const TOO_ADVANCED: &str = "You're past the top level. The deck will still catch gaps, but Yap's app will serve you better.";
const CLIPS_LOADING: &str = "Movie clips are still loading. Please try again.";
const NO_SENTENCES: &str = "No comprehensible movie-clip sentences were found at your level.";

#[bridgerton::bridge]
impl Deck {
    pub fn anki_export_view(&self) -> AnkiExportView {
        let pack = &self.context.language_pack;
        let language = self.context.course.target_language;
        let tier = (!pack.gram_frequencies.entries.is_empty()).then(|| self.get_current_tier());
        let (level, total_levels) = tier.map_or((1, 1), |tier| (tier.level, tier.total_levels));
        let too_advanced = !pack.gram_frequencies.entries.is_empty()
            && Self::percent_known_in(
                &pack.gram_frequencies,
                self.get_comprehensible_written_grams(true),
                self.get_comprehensible_listening_grams(true),
            )
            .all_available_learned;
        let clip_sentence_count = pack
            .comprehensible_sentences(None, |_| true)
            .into_iter()
            .filter(|sentence| {
                clips::sentence_has_clip(language, pack.string_rodeo.resolve(sentence))
            })
            .count() as u32;
        AnkiExportView {
            title: TITLE.into(),
            needs_placement: !self.has_taken_placement_test() && self.num_cards_added() < 3,
            level_line: format!("Level {level} of {total_levels}"),
            level,
            total_levels,
            too_advanced,
            too_advanced_message: too_advanced.then(|| TOO_ADVANCED.into()),
            clips_loaded: clips::manifest_loaded(language),
            clip_sentence_count,
            language_name: language.to_string(),
            download_label: format!("Download the {language} deck"),
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

    pub fn anki_deck_plan(
        &self,
        options: AnkiDeckOptions,
        token: String,
        timestamp_ms: f64,
    ) -> Result<AnkiDeckPlan, Error> {
        validate(&options)?;
        let course = self.context.course;
        let language = course.target_language;
        if !clips::manifest_loaded(language) {
            return Err(Error::new(CLIPS_LOADING));
        }
        let view = self.anki_export_view();
        if view.clip_sentence_count == 0 {
            return Err(Error::new(NO_SENTENCES));
        }
        let pack = &self.context.language_pack;
        let display = |gram: TaggedGram<SpurGram>| {
            gram.resolve(&pack.gram_rodeo)
                .resolve(&pack.string_rodeo)
                .to_display_string(language)
        };
        // What the learner already understands going in; everything else a
        // sentence needs gets a word note first.
        let known = self.get_comprehensible_written_grams(false).clone();
        let mut notes = Vec::new();
        let mut bundled = Vec::new();
        let mut used_sentences = BTreeSet::new();
        let mut used_words = BTreeSet::new();
        let mut posters = BTreeSet::new();
        if !timestamp_ms.is_finite() {
            return Err(Error::new("Invalid export time."));
        }
        let now = chrono::DateTime::from_timestamp_millis(timestamp_ms as i64)
            .ok_or_else(|| Error::new("Invalid export time."))?;
        let mut simulation = self.simulate_usage(now).with_new_cards_per_day(20);
        // Words the simulator has introduced that have no usable sentence yet.
        // They are retried every day: a sentence's other words only become
        // comprehensible once the simulated learner has reviewed them.
        let mut pending: Vec<TaggedGram<SpurGram>> = Vec::new();
        let mut empty_days = 0;
        while used_sentences.len() < options.size as usize && empty_days < 60 {
            let mut day = simulation.next_day();
            for _ in day.by_ref() {}
            simulation = day.finish_day();
            pending.extend(
                simulation
                    .last_introduced()
                    .iter()
                    .filter_map(|indicator| indicator.written_gram().copied()),
            );
            let deck = simulation.deck();
            let before = used_sentences.len();
            let mut still_pending = Vec::new();
            for gram in pending {
                let word = display(gram);
                let mut candidates = pack.comprehensible_sentences(Some(&gram), |g| {
                    deck.get_comprehensible_written_grams(false).contains(g)
                });
                candidates.retain(|s| {
                    !used_sentences.contains(s)
                        && clips::sentence_has_clip(language, pack.string_rodeo.resolve(s))
                });
                candidates.sort_by_key(|s| {
                    (
                        deck.stats.sentences_reviewed.get(s).copied().unwrap_or(0),
                        pack.string_rodeo.resolve(s).chars().count(),
                        pack.string_rodeo.resolve(s),
                    )
                });
                let Some((sentence, challenge)) = candidates.into_iter().find_map(|s| {
                    deck.translation_challenge_for_sentence(gram, s)
                        .map(|c| (s, c))
                }) else {
                    still_pending.push(gram);
                    continue;
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
                    let word = display(prerequisite);
                    if used_words.insert(word.clone()) {
                        notes.push(word_note(
                            pack,
                            course,
                            prerequisite,
                            &word,
                            &token,
                            &mut bundled,
                        ));
                    }
                }
                let text = challenge.target_language;
                let clip = clips::clip_for_sentence(language, &text).unwrap();
                let imdb = clip.clip_id.split('-').next().unwrap().to_owned();
                let movie = pack.movies.get(&imdb);
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
                    &token,
                );
                let tts = if used_sentences.len() < 50 {
                    let filename = format!("yap-sentence-{}.mp3", guid(course, "sentence", &text));
                    bundled.push(AnkiBundledMedia {
                        filename: filename.clone(),
                        source: AnkiMediaSource::Tts { url },
                    });
                    AnkiAudio::Bundled { filename }
                } else {
                    AnkiAudio::Streamed { url }
                };
                let glosses = sentence_glosses(
                    &challenge.target_language_literals,
                    &challenge.literal_gram_indices,
                    &challenge.gram_definitions_for_lookup,
                    self.context.course,
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
                        component(&token)
                    ),
                    tts,
                    include_reading: !matches!(options.card_types, AnkiCardTypes::Listening),
                    include_listening: !matches!(options.card_types, AnkiCardTypes::Reading),
                });
                used_sentences.insert(sentence);
                if used_sentences.len() == options.size as usize {
                    break;
                }
            }
            pending = still_pending;
            empty_days = if before == used_sentences.len() {
                empty_days + 1
            } else {
                0
            };
        }
        if used_sentences.is_empty() {
            return Err(Error::new(NO_SENTENCES));
        }
        let sentence_count = used_sentences.len() as u32;
        let word_count = used_words.len() as u32;
        Ok(AnkiDeckPlan {
            language,
            course_code: course_code(course),
            deck_name: if course.native_language == Language::English {
                format!("Yap • {language}")
            } else {
                format!("Yap • {language} ({})", course.native_language)
            },
            deck_id: id(course, "deck", ""),
            sentence_model_id: id(course, "sentence-model", ""),
            word_model_id: id(course, "word-model", ""),
            notes,
            bundled,
            stats: AnkiDeckStats {
                level: view.level,
                total_levels: view.total_levels,
                sentence_count,
                word_count,
                card_count: word_count
                    + sentence_count
                        * if matches!(options.card_types, AnkiCardTypes::Both) {
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
    use language_utils::{
        Atom, ConsolidatedLanguageData, Course, DictionaryEntry, Gram, GramFrequencyEntry,
        GramFrequencyList, GramVocabEntry, Heteronym, PartOfSpeech, SentenceGram, SentenceGrams,
        TaggedGram, TargetToNativeWord, Word, WordType, language_pack::LanguagePack,
    };
    use std::sync::Arc;
    use weapon::AppState;

    fn options(size: u32) -> AnkiDeckOptions {
        AnkiDeckOptions {
            size,
            card_types: AnkiCardTypes::Both,
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
            .anki_deck_plan(options(55), "a+b&雪".into(), 1_700_000_000_000.0)
            .unwrap();
        let b = deck
            .anki_deck_plan(options(55), "a+b&雪".into(), 1_700_000_000_000.0)
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
        assert_eq!(
            a.bundled
                .iter()
                .filter(|m| matches!(m.source, AnkiMediaSource::Tts { .. }))
                .count(),
            50
        );
        let mut sentences = BTreeSet::new();
        let mut ids = BTreeSet::new();
        for pair in a.notes.chunks_exact(2) {
            let AnkiNote::Word { word, .. } = &pair[0] else {
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
            let url = match tts {
                AnkiAudio::Bundled { filename } => {
                    let entry = a.bundled.iter().find(|m| &m.filename == filename).unwrap();
                    let AnkiMediaSource::Tts { url } = &entry.source else {
                        panic!("bundled sentence audio is fetched TTS")
                    };
                    url
                }
                AnkiAudio::Streamed { url } => url,
            };
            assert!(url.contains("language=English&text="));
            assert!(!url.contains("&hint="));
        }
        let reading = deck
            .anki_deck_plan(
                AnkiDeckOptions {
                    size: 55,
                    card_types: AnkiCardTypes::Reading,
                },
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
    fn anki_errors_and_view() {
        let deck = fixture();
        assert!(
            deck.anki_deck_plan(options(0), "token".into(), 1_700_000_000_000.0)
                .is_err()
        );
        // A different language has no manifest in this test's thread-local mirror.
        assert!(!deck.anki_export_view().clips_loaded);
        assert!(
            deck.anki_deck_plan(options(1), "token".into(), 1_700_000_000_000.0)
                .is_err()
        );
        clips::publish_manifest(Language::English, vec![]);
        assert!(deck.anki_export_view().clips_loaded);
        assert_eq!(deck.anki_export_view().clip_sentence_count, 0);
        assert!(deck.anki_export_view().needs_placement);
        assert!(
            deck.anki_deck_plan(options(1), "token".into(), 1_700_000_000_000.0)
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
            .anki_deck_plan(options(3), "token".into(), 1_700_000_000_000.0)
            .unwrap();
        assert_eq!(
            plan.bundled
                .iter()
                .filter(|m| m.filename.starts_with("yap-poster-"))
                .count(),
            1
        );
        let filename = human_filename(deck.context.course, "word00");
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
        let grams: BTreeSet<_> = deck
            .context
            .language_pack
            .gram_frequencies
            .entries
            .keys()
            .copied()
            .collect();
        deck.comprehensible.written.now_and_planned = grams.clone();
        deck.comprehensible.listening.now_and_planned = grams;
        // The corpus denominator includes grams outside this pack's teachable inventory.
        Arc::get_mut(&mut deck.context.language_pack)
            .unwrap()
            .gram_frequencies
            .total_count = 1_000_000;
        assert!(deck.get_percent_of_words_known() < 1.0);
        assert!(deck.anki_export_view().too_advanced);
        assert!(deck.anki_export_view().too_advanced_message.is_some());
    }

    #[test]
    fn anki_french_pack_sample() {
        // Real pack and simulator; an offline manifest fixture gives every pack sentence a clip.
        // This exercises selection, not the availability of the live published corpus.
        let deck = Deck::default();
        publish(&deck.context.language_pack, deck.context.course);
        let plan = deck
            .anki_deck_plan(options(10), "offline-test".into(), 1_700_000_000_000.0)
            .unwrap();
        assert_eq!(plan.stats.sentence_count, 10);
        println!("FIRST TEN ANKI NOTES (real French pack, offline manifest fixture):");
        for note in plan.notes.iter().take(10) {
            println!("{}", serde_json::to_string(note).unwrap());
        }
    }
}
