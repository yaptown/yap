//! Translation autograde eval: grades translation attempts with several
//! candidate models through the exact backend grading code
//! (`autograde_core::grade_translation`) and compares them.
//!
//! Two sets of cases:
//!
//! - **Ambiguity set** (`translation_eval_ambiguity_set.json`, hand-checked):
//!   film lines whose lone sentence has several valid readings while the
//!   scene picks one ("Pero ve." is "But go." alone, "But she sees." in the
//!   film). Each answer carries the label its key word should get, so this set
//!   has gold by construction. The request is built the way production builds
//!   it: the challenge from `Deck::translation_challenge_for_sentence`, the
//!   context from the film's subtitles.
//! - **Real set**: mistranslations users actually submitted, pulled from
//!   Supabase and replayed through the event log, with film context attached
//!   where the sentence comes from a film. The reference grade comes from
//!   [`REFERENCE`], run through the same grading code.
//!
//! Run (from the repo root, with `.env` sourced):
//!   cargo test -p yap-frontend-rs translation_autograde_eval -- --ignored --nocapture
//!
//! Env: GEMINI_API_KEY, ANTHROPIC_API_KEY, SUPABASE_SERVICE_ROLE_KEY
//! (required); EVAL_REAL_CASES (default 100), EVAL_AMBIGUITY_ITEMS (default
//! all), EVAL_MAX_USERS (default 400), EVAL_CONCURRENCY (default 16).
//!
//! Responses are cached in `../out/translation-eval/cache` by exact input, so
//! reruns are free; cache hits are left out of the latency figures.
//!
//! Output: `../out/translation-eval/{eval.json, eval.md}`.

use crate::*;
use autograde::{AutoGradeTranslationRequest, AutoGradeTranslationResponse, GraderContext};
use language_utils::SentenceGram;
use movie_subtitles::SubtitleLine;
use tysm::chat_completions::ChatClient;
use weapon::data_model::Event as _;

/// Candidate models, all at the effort production would use.
const CANDIDATES: &[(&str, &str)] = &[
    ("gemini-3.5-flash", "low"),
    ("gemini-4-hydrogen", "low"),
    ("gemini-4-neon", "low"),
    ("claude-sonnet-5-5", "low"),
    ("claude-opus-5-5", "low"),
];

/// The model whose grades stand in for gold on the real set.
const REFERENCE: (&str, &str) = ("claude-fable-5-2", "high");

/// Calls faster than this were served from tysm's cache.
const CACHE_HIT_MS: u128 = 100;

/// Subtitle lines on each side of the sentence, about what a clip's padding
/// holds.
const CONTEXT_LINES: usize = 2;

const OUT_DIR: &str = "../out/translation-eval";

/// Minimal `.env` loader (repo root) so the eval works without sourcing.
fn load_dotenv_if_present() {
    for path in ["../.env", ".env"] {
        if let Ok(contents) = std::fs::read_to_string(path) {
            for line in contents.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = line.split_once('=') {
                    let k = k.trim();
                    let v = v.trim().trim_matches('"').trim_matches('\'');
                    if std::env::var(k).is_err() {
                        unsafe { std::env::set_var(k, v) };
                    }
                }
            }
        }
    }
}

fn env_knob(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

// ---------------------------------------------------------------------------
// Language packs and film context
// ---------------------------------------------------------------------------

/// A course's language pack plus a display-string → gram lookup (to turn the
/// phrase strings stored in old events back into grams).
struct Pack {
    pack: Arc<LanguagePack>,
    display_to_gram: std::collections::HashMap<String, Gram<String>>,
}

#[derive(Default)]
struct Packs(BTreeMap<Course, Option<Pack>>);

impl Packs {
    fn get(&mut self, course: Course) -> Option<&Pack> {
        self.0
            .entry(course)
            .or_insert_with(|| {
                let dir = format!(
                    "../out/{}_for_{}",
                    course.target_language.code(),
                    course.native_language.code()
                );
                let dir = std::path::Path::new(&dir);
                if !dir
                    .join(language_utils::language_pack::CORE_FILENAME)
                    .exists()
                {
                    return None;
                }
                let pack: LanguagePack = language_utils::language_pack::load_split_dir(dir).ok()?;
                let display_to_gram = pack
                    .gram_rodeo
                    .strings()
                    .map(|spur| {
                        let gram: Gram<String> = spur.resolve(&pack.string_rodeo);
                        (gram.to_display_string(course.target_language), gram)
                    })
                    .collect();
                Some(Pack {
                    pack: Arc::new(pack),
                    display_to_gram,
                })
            })
            .as_ref()
    }
}

/// Cleaned subtitle lines per (language, film), loaded on first use.
#[derive(Default)]
struct Subtitles(BTreeMap<(Language, String), Option<Vec<SubtitleLine>>>);

impl Subtitles {
    fn get(&mut self, language: Language, imdb: &str) -> Option<&[SubtitleLine]> {
        self.0
            .entry((language, imdb.to_string()))
            .or_insert_with(|| {
                // Regional variants (por-pt, spa-es) have no subtitle dir of
                // their own; their films live under the base language.
                let code = language.code();
                [code, code.split('-').next().unwrap()]
                    .into_iter()
                    .find_map(|code| {
                        let dir = format!("../generate-data/data/{code}/sentence-sources/movies");
                        movie_subtitles::load(std::path::Path::new(&dir), imdb, language)
                            .unwrap()
                            .map(|(lines, _)| lines)
                    })
            })
            .as_deref()
    }
}

/// Lowercase letters and digits, everything else a single space, padded so
/// `contains` matches whole words.
fn match_key(text: &str) -> String {
    let words: Vec<String> = text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect();
    format!(" {} ", words.join(" "))
}

/// Strip the dialogue dashes and spaces subtitles put around a turn.
fn trim_turn(text: &str) -> &str {
    text.trim_matches(|c: char| c.is_whitespace() || matches!(c, '-' | '–' | '—'))
}

/// The film's dialogue around `sentence`: [`CONTEXT_LINES`] cues on each
/// side, plus whatever else shares the sentence's own cue ("–Pero habla.
/// –No habla. –Pero ve." puts the first two turns before "Pero ve."). None
/// when the sentence isn't found in the film.
fn dialogue_around(lines: &[SubtitleLine], sentence: &str) -> Option<(Vec<String>, Vec<String>)> {
    let needle = match_key(sentence);
    // A sentence spanning two cues only counts once no single cue holds it,
    // or "Pero ve." would match the pair ending in the cue that holds it.
    let (first, last) = (0..lines.len())
        .find(|&i| match_key(&lines[i].sentence).contains(&needle))
        .map(|i| (i, i))
        .or_else(|| {
            (0..lines.len().saturating_sub(1)).find_map(|i| {
                let joined = format!("{} {}", lines[i].sentence, lines[i + 1].sentence);
                match_key(&joined).contains(&needle).then_some((i, i + 1))
            })
        })?;

    let mut before: Vec<String> = lines[first.saturating_sub(CONTEXT_LINES)..first]
        .iter()
        .map(|l| l.sentence.clone())
        .collect();
    let mut after: Vec<String> = lines[last + 1..(last + 1 + CONTEXT_LINES).min(lines.len())]
        .iter()
        .map(|l| l.sentence.clone())
        .collect();
    if first == last
        && let Some((prefix, suffix)) = lines[first].sentence.split_once(sentence)
    {
        if !trim_turn(prefix).is_empty() {
            before.push(trim_turn(prefix).to_string());
        }
        if !trim_turn(suffix).is_empty() {
            after.insert(0, trim_turn(suffix).to_string());
        }
    }
    Some((before, after))
}

/// The grader context production would send for a corpus sentence: the
/// film's title and the dialogue around the line. `imdb` pins the film (the
/// same line can occur in several); otherwise the first of the sentence's
/// films whose subtitles contain it. None when the sentence isn't from a film
/// we have subtitles for.
fn film_context(
    pack: &LanguagePack,
    language: Language,
    sentence: &str,
    imdb: Option<&str>,
    subtitles: &mut Subtitles,
) -> Option<GraderContext> {
    let key = pack.sentence_key(sentence)?;
    let films: Vec<String> = match imdb {
        Some(imdb) => vec![imdb.to_string()],
        None => pack
            .source_of(key)
            .movie_ids()
            .map(str::to_string)
            .collect(),
    };
    films.into_iter().find_map(|film| {
        let (dialogue_before, dialogue_after) =
            dialogue_around(subtitles.get(language, &film)?, sentence)?;
        Some(GraderContext {
            movie_title: pack
                .sentences
                .movies
                .get(film.as_str())
                .map(|m| m.title.to_string()),
            dialogue_before,
            dialogue_after,
        })
    })
}

// ---------------------------------------------------------------------------
// Ambiguity set
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct AmbiguityItem {
    /// Target language code; the native language is English.
    language: String,
    /// The film the line is taken from, which supplies the context.
    imdb: String,
    sentence: String,
    /// The ambiguous word, as it appears in the sentence.
    word: String,
    /// The readings, and which one the scene means.
    note: String,
    answers: Vec<AmbiguityAnswer>,
}

#[derive(serde::Deserialize)]
struct AmbiguityAnswer {
    text: String,
    /// The grade `word` should get.
    expected: autograde::Remembered,
}

/// One grading request, plus what to score in its response.
struct Case {
    set: &'static str,
    language: Language,
    /// Anonymous label for where the case came from (user number or note).
    origin: String,
    request: AutoGradeTranslationRequest,
    /// Ambiguity set: the key word's literal position and its gold grade.
    gold: Option<(usize, autograde::Remembered)>,
    /// Real set: the grades stored when the user reviewed it.
    production_literal_grades: Vec<Option<bool>>,
}

/// Build the production challenge for each ambiguity item, one case per
/// answer.
fn ambiguity_cases(packs: &mut Packs, subtitles: &mut Subtitles, limit: usize) -> Vec<Case> {
    let items: Vec<AmbiguityItem> =
        serde_json::from_str(include_str!("translation_eval_ambiguity_set.json")).unwrap();
    let mut cases = Vec::new();
    for item in items.into_iter().take(limit) {
        let language = Language::from_code(&item.language).unwrap();
        let course = Course {
            target_language: language,
            native_language: Language::English,
        };
        let pack = packs.get(course).unwrap().pack.clone();
        let sentence = pack
            .sentence_key(&item.sentence)
            .unwrap_or_else(|| panic!("{:?} is not in the {language} pack", item.sentence));
        let context = Context {
            study_goal: None,
            language_pack: pack.clone(),
            course,
            timezone: chrono::FixedOffset::east_opt(0).unwrap(),
        };
        let deck = <Deck as weapon::AppState>::finalize(DeckState::new(), &context);

        // The key word's gram, found while flattening the sentence the same
        // way the challenge builder does.
        let grams = comprehensible_sentence_from_spur(&pack, sentence)
            .unwrap()
            .target_language_sentence_grams
            .to_literals(&pack.string_rodeo, &pack.gram_rodeo, language);
        let gram = grams
            .into_iter()
            .find_map(|g| {
                let (SentenceGram::Learnable((gram, literals))
                | SentenceGram::Obvious((gram, literals))) = g;
                literals
                    .iter()
                    .any(|l| l.word.text.to_lowercase() == item.word.to_lowercase())
                    .then_some(gram)
            })
            .unwrap_or_else(|| panic!("no {:?} in {:?}", item.word, item.sentence));
        let challenge = deck
            .translation_challenge_for_sentence(gram, sentence)
            .unwrap();
        let position = challenge
            .target_language_literals
            .iter()
            .position(|l| l.word.text.to_lowercase() == item.word.to_lowercase())
            .unwrap();
        assert!(
            challenge.target_language_literals[position]
                .word
                .heteronym()
                .is_some(),
            "{:?} in {:?} is not gradable",
            item.word,
            item.sentence
        );
        let grader_context =
            film_context(&pack, language, &item.sentence, Some(&item.imdb), subtitles)
                .unwrap_or_else(|| panic!("{:?} not found in {}", item.sentence, item.imdb));
        for answer in item.answers {
            cases.push(Case {
                set: "ambiguity",
                language,
                origin: item.note.clone(),
                request: AutoGradeTranslationRequest {
                    course,
                    challenge_sentence: challenge.target_language.clone(),
                    user_sentence: answer.text,
                    literals: challenge.target_language_literals.clone(),
                    phrases: challenge.unique_target_language_phrases.clone(),
                    primary_expression: challenge.primary_expression.clone(),
                    primary_literal_indices: challenge.primary_literal_indices.clone(),
                    context: grader_context.clone(),
                },
                gold: Some((position, answer.expected)),
                production_literal_grades: vec![],
            });
        }
    }
    cases
}

// ---------------------------------------------------------------------------
// Real set: mistranslations from Supabase
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct RawCase {
    user: usize,
    course: Course,
    challenge: String,
    submission: String,
    literals: Vec<(Literal<String>, Option<current::LiteralResult>)>,
    phrases: Vec<(String, Option<bool>)>,
}

/// Fetch every event row for a user (paginated).
async fn fetch_user_rows(
    http: &reqwest::Client,
    supabase_url: &str,
    key: &str,
    user_id: &str,
) -> Vec<serde_json::Value> {
    let mut all = Vec::new();
    let page_size = 1000;
    let mut offset = 0;
    loop {
        let page: Vec<serde_json::Value> = match http
            .get(format!(
                "{supabase_url}/rest/v1/events?user_id=eq.{user_id}&order=id.asc&limit={page_size}&offset={offset}"
            ))
            .header("apikey", key)
            .header("Authorization", format!("Bearer {key}"))
            .send()
            .await
            .and_then(|r| r.error_for_status())
        {
            Ok(r) => r.json().await.unwrap_or_default(),
            Err(_) => break,
        };
        let count = page.len();
        all.extend(page);
        if count < page_size {
            break;
        }
        offset += page_size;
    }
    all
}

/// Replay a user's events (migrating old events via the real
/// `from_versioned` path) and extract every graded translation where at
/// least one literal or phrase was forgotten.
fn extract_mistranslations(
    rows: &[serde_json::Value],
    user: usize,
    packs: &mut Packs,
) -> Vec<RawCase> {
    let mut grouped: BTreeMap<
        String,
        BTreeMap<String, Vec<Timestamped<weapon::data_model::RawJson>>>,
    > = BTreeMap::new();
    for row in rows {
        let stream_id = row["stream_id"].as_str().unwrap_or("reviews").to_string();
        let device_id = row["device_id"].as_str().unwrap_or("unknown").to_string();
        let event_value = match &row["event"] {
            serde_json::Value::String(s) => match serde_json::from_str(s) {
                Ok(v) => v,
                Err(_) => continue,
            },
            v => v.clone(),
        };
        let Ok(timestamped) =
            serde_json::from_value::<Timestamped<weapon::data_model::RawJson>>(event_value)
        else {
            continue;
        };
        grouped
            .entry(stream_id)
            .or_default()
            .entry(device_id)
            .or_default()
            .push(timestamped);
    }

    let mut store: EventStore<String, String> = EventStore::default();
    store.get_or_insert_default::<EventType<DeckEvent>>(REVIEWS_STREAM.to_string(), None);
    store.get_or_insert_default::<EventType<DeckSelectionEvent>>(
        DECK_SELECTION_STREAM.to_string(),
        None,
    );
    for (stream_id, devices) in grouped {
        for (device_id, events) in devices {
            store.add_device_events_jsons(stream_id.clone(), device_id, events, None);
        }
    }

    // The deck-selection course is only needed to migrate old events.
    let deck_selection = store
        .get::<EventType<DeckSelectionEvent>>(DECK_SELECTION_STREAM.to_string())
        .map(|s| {
            s.state(
                DeckSelectionPartial {
                    target_language: None,
                    native_language: None,
                    onboarding_selections: BTreeMap::new(),
                    selected_languages: BTreeSet::new(),
                    heard_about: None,
                },
                &(),
            )
        });
    let Some(target) = deck_selection
        .as_ref()
        .and_then(|ds: &DeckSelection| ds.target_language)
    else {
        return vec![];
    };
    let native = deck_selection
        .as_ref()
        .and_then(|ds: &DeckSelection| ds.native_language)
        .unwrap_or(Language::English);
    let course = Course {
        target_language: target,
        native_language: native,
    };
    let Some(pack) = packs.get(course) else {
        return vec![];
    };
    let context = Context {
        study_goal: None,
        language_pack: pack.pack.clone(),
        course,
        timezone: chrono::FixedOffset::east_opt(0).unwrap(),
    };

    let Some(stream) = store.get::<EventType<DeckEvent>>(REVIEWS_STREAM.to_string()) else {
        return vec![];
    };

    let mut out = Vec::new();
    for t in stream.iter() {
        let EventType::User(versioned) = &t.event else {
            continue;
        };
        let Some(DeckEvent::Language(le)) = DeckEvent::from_versioned(versioned, &context) else {
            continue;
        };
        // A user may have studied several courses; the event knows its own.
        let event_course = Course {
            target_language: le.target_language,
            native_language: le.native_language,
        };
        let LanguageEventContent::TranslationChallenge { review, .. } = le.content else {
            continue;
        };
        let current::SentenceReviewResult::Graded {
            challenge,
            submission,
            literals,
            phrases,
        } = review
        else {
            continue; // Perfect submissions are not mistranslations
        };
        let any_literal_forgot = literals
            .iter()
            .any(|(_, r)| matches!(r, Some(lr) if lr.remembered == Some(false)));
        let any_phrase_forgot = phrases.iter().any(|(_, g)| *g == Some(false));
        if !any_literal_forgot && !any_phrase_forgot {
            continue;
        }
        out.push(RawCase {
            user,
            course: event_course,
            challenge,
            submission,
            literals,
            phrases: phrases
                .into_iter()
                .map(|(g, r)| (g.to_display_string(event_course.target_language), r))
                .collect(),
        });
    }
    out
}

fn gradable_count(case: &RawCase) -> usize {
    case.literals
        .iter()
        .filter(|(l, _)| l.word.heteronym().is_some())
        .count()
}

/// Heuristic "hardness" of a mistranslation: favors long sentences with many
/// gradable words, multiple forgotten items, and real idiomatic phrases.
fn hardness(case: &RawCase) -> usize {
    let forgotten_lits = case
        .literals
        .iter()
        .filter(|(_, r)| matches!(r, Some(lr) if lr.remembered == Some(false)))
        .count();
    // "Real" phrases are the ones the grader actually graded (Some), not the
    // liberal false-positive candidates (None).
    let real_phrases = case.phrases.iter().filter(|(_, g)| g.is_some()).count();
    let forgotten_phrases = case
        .phrases
        .iter()
        .filter(|(_, g)| *g == Some(false))
        .count();
    case.challenge.split_whitespace().count()
        + 2 * gradable_count(case)
        + 3 * (forgotten_lits + forgotten_phrases)
        + 2 * real_phrases
}

/// Rebuild the grading request a stored mistranslation was graded with. Old
/// events don't record the primary expression, so the first forgotten phrase
/// or word stands in for it.
fn real_request(
    case: &RawCase,
    pack: &Pack,
    context: GraderContext,
) -> AutoGradeTranslationRequest {
    let phrases: Vec<Gram<String>> = case
        .phrases
        .iter()
        .filter_map(|(disp, _)| pack.display_to_gram.get(disp).cloned())
        .collect();
    let (primary_expression, primary_literal_indices) = case
        .phrases
        .iter()
        .find(|(_, g)| *g == Some(false))
        .and_then(|(disp, _)| pack.display_to_gram.get(disp).cloned())
        .map(|gram| (gram, vec![]))
        .or_else(|| {
            case.literals
                .iter()
                .enumerate()
                .find(|(_, (l, r))| {
                    l.word.heteronym().is_some()
                        && matches!(r, Some(lr) if lr.remembered == Some(false))
                })
                .or_else(|| {
                    case.literals
                        .iter()
                        .enumerate()
                        .find(|(_, (l, _))| l.word.heteronym().is_some())
                })
                .map(|(index, (literal, _))| {
                    (Gram(vec![Atom::Tok(literal.word.clone())]), vec![index])
                })
        })
        .unwrap_or((Gram(vec![]), vec![]));
    AutoGradeTranslationRequest {
        course: case.course,
        challenge_sentence: case.challenge.clone(),
        user_sentence: case.submission.clone(),
        literals: case.literals.iter().map(|(l, _)| l.clone()).collect(),
        phrases: phrases
            .into_iter()
            .map(|gram| TaggedGram { gram, sense: None })
            .collect(),
        primary_expression: TaggedGram {
            gram: primary_expression,
            sense: None,
        },
        primary_literal_indices,
        context,
    }
}

/// Pull mistranslations from every user (up to `max_users`), attach film
/// context, and pick `sample` of them round-robin across languages, cases
/// with context first, hardest first.
async fn real_cases(
    packs: &mut Packs,
    subtitles: &mut Subtitles,
    sample: usize,
    max_users: usize,
) -> Vec<Case> {
    let service_role_key = std::env::var("SUPABASE_SERVICE_ROLE_KEY").unwrap();
    let supabase_url = std::env::var("SUPABASE_URL")
        .unwrap_or_else(|_| "https://eearwzqotpfoderpfrqx.supabase.co".to_string());
    let http = reqwest::Client::new();

    let mut user_ids: Vec<String> = Vec::new();
    for page in 1..=100 {
        let resp: serde_json::Value = http
            .get(format!(
                "{supabase_url}/auth/v1/admin/users?page={page}&per_page=50"
            ))
            .header("apikey", &service_role_key)
            .header("Authorization", format!("Bearer {service_role_key}"))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let users = resp["users"].as_array().cloned().unwrap_or_default();
        if users.is_empty() {
            break;
        }
        user_ids.extend(
            users
                .iter()
                .filter_map(|u| u["id"].as_str().map(String::from)),
        );
    }
    println!("Discovered {} users", user_ids.len());

    let mut seen = BTreeSet::new();
    let mut by_lang: BTreeMap<Language, Vec<(bool, RawCase, GraderContext)>> = BTreeMap::new();
    for (user, id) in user_ids.iter().take(max_users).enumerate() {
        let rows = fetch_user_rows(&http, &supabase_url, &service_role_key, id).await;
        for case in extract_mistranslations(&rows, user, packs) {
            if gradable_count(&case) < 2
                || !seen.insert((case.challenge.clone(), case.submission.clone()))
            {
                continue;
            }
            let Some(pack) = packs.get(case.course) else {
                continue;
            };
            let pack = pack.pack.clone();
            let context = film_context(
                &pack,
                case.course.target_language,
                &case.challenge,
                None,
                subtitles,
            )
            .unwrap_or_default();
            by_lang
                .entry(case.course.target_language)
                .or_default()
                .push((!context.is_empty(), case, context));
        }
    }
    for (language, bucket) in &mut by_lang {
        bucket.sort_by_key(|(has_context, case, _)| {
            std::cmp::Reverse((*has_context, hardness(case)))
        });
        println!(
            "  {language}: {} mistranslations, {} with film context",
            bucket.len(),
            bucket.iter().filter(|(c, _, _)| *c).count()
        );
    }

    let mut buckets: Vec<std::collections::VecDeque<_>> =
        by_lang.into_values().map(Into::into).collect();
    let mut cases = Vec::new();
    while cases.len() < sample && buckets.iter().any(|b| !b.is_empty()) {
        for bucket in &mut buckets {
            if cases.len() == sample {
                break;
            }
            let Some((_, case, context)) = bucket.pop_front() else {
                continue;
            };
            let pack = packs.get(case.course).unwrap();
            cases.push(Case {
                set: "real",
                language: case.course.target_language,
                origin: format!("user {}", case.user),
                request: real_request(&case, pack, context),
                gold: None,
                production_literal_grades: case
                    .literals
                    .iter()
                    .map(|(_, r)| r.as_ref().and_then(|lr| lr.remembered))
                    .collect(),
            });
        }
    }
    cases
}

// ---------------------------------------------------------------------------
// Grading and scoring
// ---------------------------------------------------------------------------

struct Grader {
    model: &'static str,
    effort: &'static str,
    client: ChatClient,
}

impl Grader {
    /// Each provider's native API, configured the way production would.
    fn new((model, effort): (&'static str, &'static str)) -> Self {
        let client = if model.starts_with("claude-") {
            ChatClient::from_env(model)
                .unwrap()
                .with_reasoning_effort(effort)
                .with_extra_body(serde_json::json!({ "max_tokens": 16000 }))
        } else {
            ChatClient::new(std::env::var("GEMINI_API_KEY").unwrap(), model)
                .with_url("https://generativelanguage.googleapis.com/v1beta/openai/")
                .with_reasoning_effort(effort)
        }
        // A small store of its own: opening the shared 300 GB `.cache` takes
        // minutes and would land inside the first calls' latencies.
        .with_cache_directory(format!("{OUT_DIR}/cache"))
        // The eval's own limiter bounds concurrency, before the timer starts,
        // so tysm's semaphore never queues inside the timed region.
        .with_max_concurrent_requests(1024);
        Self {
            model,
            effort,
            client,
        }
    }

    fn label(&self) -> String {
        format!("{} ({})", self.model, self.effort)
    }
}

#[derive(serde::Serialize)]
struct GradeOut {
    model: String,
    error: Option<String>,
    response: Option<AutoGradeTranslationResponse>,
    latency_ms: u128,
    cached: bool,
}

impl GradeOut {
    fn literal(&self, position: usize) -> Option<Option<autograde::Remembered>> {
        self.response
            .as_ref()?
            .literal_grades
            .get(position)
            .cloned()
    }

    /// The grade of the challenge's primary expression: its literals' grades,
    /// or for a phrase whether it was listed remembered or forgot.
    fn primary(&self, request: &AutoGradeTranslationRequest) -> Option<String> {
        let response = self.response.as_ref()?;
        let primary = &request.primary_expression;
        if request.phrases.contains(primary) {
            Some(
                if response.phrases_forgot.contains(primary) {
                    "Forgot"
                } else if response.phrases_remembered.contains(primary) {
                    "Remembered"
                } else {
                    "-"
                }
                .to_string(),
            )
        } else {
            Some(
                request
                    .primary_literal_indices
                    .iter()
                    .map(|&p| grade_str(response.literal_grades.get(p).cloned().flatten()))
                    .collect::<Vec<_>>()
                    .join("/"),
            )
        }
    }
}

#[derive(serde::Serialize)]
struct CaseOut {
    set: &'static str,
    language: String,
    origin: String,
    challenge: String,
    submission: String,
    context: GraderContext,
    primary_expression: String,
    /// (literal position, text) of each gradable literal.
    gradable_words: Vec<(usize, String)>,
    gold_word: Option<String>,
    gold: Option<String>,
    production_literal_grades: Vec<Option<bool>>,
    grades: Vec<GradeOut>,
}

fn grade_str(grade: Option<autograde::Remembered>) -> &'static str {
    match grade {
        Some(autograde::Remembered::Remembered) => "Remembered",
        Some(autograde::Remembered::Forgot) => "Forgot",
        None => "null",
    }
}

fn percentile(sorted: &[u128], p: f64) -> u128 {
    if sorted.is_empty() {
        return 0;
    }
    sorted[((sorted.len() - 1) as f64 * p).round() as usize]
}

fn pct(n: usize, d: usize) -> String {
    if d == 0 {
        "-".to_string()
    } else {
        format!("{n}/{d} ({:.0}%)", 100.0 * n as f64 / d as f64)
    }
}

#[tokio::test]
#[ignore]
async fn translation_autograde_eval() {
    load_dotenv_if_present();
    for key in [
        "GEMINI_API_KEY",
        "ANTHROPIC_API_KEY",
        "SUPABASE_SERVICE_ROLE_KEY",
    ] {
        assert!(std::env::var(key).is_ok_and(|k| !k.is_empty()), "set {key}");
    }

    let mut packs = Packs::default();
    let mut subtitles = Subtitles::default();
    let mut cases = ambiguity_cases(
        &mut packs,
        &mut subtitles,
        env_knob("EVAL_AMBIGUITY_ITEMS", usize::MAX),
    );
    let ambiguity_count = cases.len();
    cases.extend(
        real_cases(
            &mut packs,
            &mut subtitles,
            env_knob("EVAL_REAL_CASES", 100),
            env_knob("EVAL_MAX_USERS", 400),
        )
        .await,
    );
    let real_count = cases.len() - ambiguity_count;
    let real_with_context = cases
        .iter()
        .filter(|c| c.set == "real" && !c.request.context.is_empty())
        .count();
    println!(
        "{ambiguity_count} ambiguity cases, {real_count} real cases ({real_with_context} with film context)"
    );
    for set in ["ambiguity", "real"] {
        if let Some(case) = cases
            .iter()
            .find(|c| c.set == set && !c.request.context.is_empty())
        {
            println!(
                "\nExample {set} request:\n{}",
                serde_json::to_string_pretty(&case.request).unwrap()
            );
        }
    }

    std::fs::create_dir_all(format!("{OUT_DIR}/cache")).unwrap();
    let graders: Vec<Grader> = std::iter::once(REFERENCE)
        .chain(CANDIDATES.iter().copied())
        .map(Grader::new)
        .collect();

    // A global limiter keeps latencies real single-call latencies: the permit
    // is acquired before the timer starts.
    let concurrency = env_knob("EVAL_CONCURRENCY", 16);
    let limiter = Arc::new(tokio::sync::Semaphore::new(concurrency));
    println!(
        "\nRunning {} grading calls, {concurrency} at a time...",
        cases.len() * graders.len()
    );
    let mut tasks = Vec::new();
    for (ci, case) in cases.iter().enumerate() {
        for (gi, grader) in graders.iter().enumerate() {
            let limiter = limiter.clone();
            tasks.push(async move {
                let _permit = limiter.acquire_owned().await.unwrap();
                let start = std::time::Instant::now();
                let result = autograde_core::grade_translation(&grader.client, &case.request).await;
                let latency_ms = start.elapsed().as_millis();
                if let Err(e) = &result {
                    println!("  {} error on case {ci}: {e}", grader.label());
                }
                (ci, gi, result, latency_ms)
            });
        }
    }
    let mut grid: Vec<Vec<Option<GradeOut>>> = cases
        .iter()
        .map(|_| graders.iter().map(|_| None).collect())
        .collect();
    for (ci, gi, result, latency_ms) in futures::future::join_all(tasks).await {
        let (response, error) = match result {
            Ok(r) => (Some(r), None),
            Err(e) => (None, Some(e.to_string())),
        };
        grid[ci][gi] = Some(GradeOut {
            model: graders[gi].label(),
            error,
            response,
            latency_ms,
            cached: latency_ms < CACHE_HIT_MS,
        });
    }

    let cases_out: Vec<CaseOut> = cases
        .iter()
        .zip(grid)
        .map(|(case, grades)| {
            let request = &case.request;
            let target = request.course.target_language;
            CaseOut {
                set: case.set,
                language: target.code().to_string(),
                origin: case.origin.clone(),
                challenge: request.challenge_sentence.clone(),
                submission: request.user_sentence.clone(),
                context: request.context.clone(),
                primary_expression: request.primary_expression.to_display_string(target),
                gradable_words: request
                    .literals
                    .iter()
                    .enumerate()
                    .filter(|(_, l)| l.word.heteronym().is_some())
                    .map(|(p, l)| (p, l.word.text.clone()))
                    .collect(),
                gold_word: case
                    .gold
                    .as_ref()
                    .map(|(p, _)| request.literals[*p].word.text.clone()),
                gold: case
                    .gold
                    .clone()
                    .map(|(_, g)| grade_str(Some(g)).to_string()),
                production_literal_grades: case.production_literal_grades.clone(),
                grades: grades.into_iter().map(Option::unwrap).collect(),
            }
        })
        .collect();

    std::fs::create_dir_all(OUT_DIR).unwrap();
    std::fs::write(
        format!("{OUT_DIR}/eval.json"),
        serde_json::to_string_pretty(&cases_out).unwrap(),
    )
    .unwrap();

    let md = report(&cases, &cases_out, &graders);
    std::fs::write(format!("{OUT_DIR}/eval.md"), &md).unwrap();
    println!("\n{md}");
    println!("Wrote {OUT_DIR}/eval.json and {OUT_DIR}/eval.md");
}

/// The markdown report: summary tables, then every case where a candidate
/// missed the gold (ambiguity) or disagreed with the reference on the primary
/// expression (real).
fn report(cases: &[Case], out: &[CaseOut], graders: &[Grader]) -> String {
    let languages: BTreeSet<Language> = cases.iter().map(|c| c.language).collect();
    let ambiguity: Vec<usize> = (0..cases.len())
        .filter(|&i| cases[i].set == "ambiguity")
        .collect();
    let real: Vec<usize> = (0..cases.len())
        .filter(|&i| cases[i].set == "real")
        .collect();
    let real_languages: BTreeSet<Language> = real.iter().map(|&i| cases[i].language).collect();
    let ambiguity_items: BTreeSet<(&str, &str)> = ambiguity
        .iter()
        .map(|&i| (out[i].language.as_str(), out[i].challenge.as_str()))
        .collect();

    let mut md = String::from("# Translation autograde eval\n\n");
    md.push_str(&format!(
        "Ambiguity set: {} answers to {} sentences. Real set: {} cases in {} languages, {} with film context. Reference: {}.\n\n",
        ambiguity.len(),
        ambiguity_items.len(),
        real.len(),
        real_languages.len(),
        real.iter().filter(|&&i| !cases[i].request.context.is_empty()).count(),
        graders[0].label(),
    ));

    // Ambiguity accuracy, overall and per language.
    md.push_str("## Ambiguity set: key word graded as expected\n\nOver completed calls; failed calls are in the errors column below.\n\n| model | overall |");
    let amb_languages: Vec<Language> = languages
        .iter()
        .copied()
        .filter(|l| ambiguity.iter().any(|&i| cases[i].language == *l))
        .collect();
    for l in &amb_languages {
        md.push_str(&format!(" {} |", l.code()));
    }
    md.push_str(" expected Remembered | expected Forgot |\n|---|---|");
    for _ in &amb_languages {
        md.push_str("---|");
    }
    md.push_str("---|---|\n");
    for (gi, grader) in graders.iter().enumerate() {
        let score = |filter: &dyn Fn(&Case) -> bool| {
            let (mut right, mut total) = (0, 0);
            for &i in &ambiguity {
                // Failed calls are counted in the errors column, not as misses.
                if !filter(&cases[i]) || out[i].grades[gi].response.is_none() {
                    continue;
                }
                let (position, gold) = cases[i].gold.clone().unwrap();
                total += 1;
                if out[i].grades[gi].literal(position) == Some(Some(gold)) {
                    right += 1;
                }
            }
            pct(right, total)
        };
        md.push_str(&format!("| {} | {} |", grader.label(), score(&|_| true)));
        for l in &amb_languages {
            md.push_str(&format!(" {} |", score(&|c| c.language == *l)));
        }
        md.push_str(&format!(
            " {} | {} |\n",
            score(&|c| matches!(c.gold, Some((_, autograde::Remembered::Remembered)))),
            score(&|c| matches!(c.gold, Some((_, autograde::Remembered::Forgot)))),
        ));
    }

    // Real-set agreement with the reference, latency and cost.
    md.push_str(&format!(
        "\n## Real set vs {}, latency, cost\n\nPer-word agreement counts every gradable literal (null included) where both graded. Latency is over uncached calls in both sets.\n\n| model | per-word agreement | primary expression matches | errors | uncached calls | p50 ms | p90 ms | $/grade | tokens in/out per grade |\n|---|---|---|---|---|---|---|---|---|\n",
        graders[0].label()
    ));
    for (gi, grader) in graders.iter().enumerate() {
        let (mut word_agree, mut words, mut primary_agree, mut primaries) = (0, 0, 0, 0);
        for &i in &real {
            let (reference, candidate) = (&out[i].grades[0], &out[i].grades[gi]);
            if reference.response.is_none() || candidate.response.is_none() {
                continue;
            }
            for &(p, _) in &out[i].gradable_words {
                words += 1;
                if reference.literal(p) == candidate.literal(p) {
                    word_agree += 1;
                }
            }
            primaries += 1;
            if reference.primary(&cases[i].request) == candidate.primary(&cases[i].request) {
                primary_agree += 1;
            }
        }
        let errors = out.iter().filter(|c| c.grades[gi].error.is_some()).count();
        let mut latencies: Vec<u128> = out
            .iter()
            .map(|c| &c.grades[gi])
            .filter(|g| !g.cached && g.error.is_none())
            .map(|g| g.latency_ms)
            .collect();
        latencies.sort_unstable();
        let uncached = latencies.len().max(1) as f64;
        let usage = grader.client.usage();
        md.push_str(&format!(
            "| {} | {} | {} | {errors} | {} | {} | {} | {} | {:.0}/{:.0} |\n",
            grader.label(),
            pct(word_agree, words),
            pct(primary_agree, primaries),
            latencies.len(),
            percentile(&latencies, 0.5),
            percentile(&latencies, 0.9),
            grader
                .client
                .cost()
                .map(|c| format!("{:.4}", c / uncached))
                .unwrap_or_else(|| "unpriced".to_string()),
            usage.prompt_tokens as f64 / uncached,
            usage.completion_tokens as f64 / uncached,
        ));
    }
    md.push_str(&format!(
        "\nSpend this run (cached calls are free): {}\n\n",
        tysm::chat_completions::process_spend()
    ));

    md.push_str("## Ambiguity misses\n\n");
    for &i in &ambiguity {
        let (position, gold) = cases[i].gold.clone().unwrap();
        let misses: Vec<String> = graders
            .iter()
            .enumerate()
            .filter_map(|(gi, g)| {
                // Failed calls are counted in the errors column, not listed.
                let got = out[i].grades[gi].literal(position)?;
                (got != Some(gold.clone())).then(|| format!("{} → {}", g.model, grade_str(got)))
            })
            .collect();
        if misses.is_empty() {
            continue;
        }
        let c = &out[i];
        md.push_str(&format!(
            "- **{}** {:?} → {:?}: *{}* should be {} ({}). Misses: {}\n",
            c.language,
            c.challenge,
            c.submission,
            c.gold_word.as_deref().unwrap(),
            grade_str(Some(gold)),
            c.origin,
            misses.join(", ")
        ));
    }

    md.push_str(&format!(
        "\n## Real cases where a candidate's primary grade differs from {}\n\n",
        graders[0].model
    ));
    for &i in &real {
        let request = &cases[i].request;
        // Only cases both sides graded; failed calls are in the errors column.
        let Some(reference) = out[i].grades[0].primary(request) else {
            continue;
        };
        let differing: Vec<String> = graders
            .iter()
            .enumerate()
            .skip(1)
            .filter_map(|(gi, g)| {
                let got = out[i].grades[gi].primary(request)?;
                (got != reference).then(|| format!("{} → {got}", g.model))
            })
            .collect();
        if differing.is_empty() {
            continue;
        }
        let c = &out[i];
        md.push_str(&format!(
            "### {} [{}] {:?}\n\nUser: {:?}. Primary: *{}*. {} → {}; {}\n\n",
            c.language,
            i,
            c.challenge,
            c.submission,
            c.primary_expression,
            graders[0].model,
            reference,
            differing.join(", ")
        ));
        if !c.context.is_empty() {
            md.push_str(&format!(
                "Context: {} | before: {:?} | after: {:?}\n\n",
                c.context.movie_title.as_deref().unwrap_or("?"),
                c.context.dialogue_before,
                c.context.dialogue_after
            ));
        }
        for g in &c.grades {
            if let Some(r) = &g.response {
                md.push_str(&format!(
                    "> **{}**: {} {}\n>\n",
                    g.model,
                    r.encouragement.as_deref().unwrap_or(""),
                    r.explanation.as_deref().unwrap_or("")
                ));
            }
        }
        md.push('\n');
    }
    md
}
