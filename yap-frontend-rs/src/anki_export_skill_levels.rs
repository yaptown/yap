//! Real-pack, real-manifest coverage check for YAP-104. Fetch fixtures first:
//! ```sh
//! mkdir -p /tmp/yap-104
//! for lang in fra spa deu jpn; do
//!   curl --fail -sS -H 'Authorization: Bearer anonymous' \
//!     "https://yap-ai-backend.fly.dev/clip/$lang/sentences" \
//!     -o /tmp/yap-104/manifest_$lang.json
//! done
//! ```
//! Then run `cargo test -p yap-frontend-rs --release anki_skill_levels --
//! --ignored --nocapture --test-threads=1` (serial for comparable timings).
//! Each course is independent so a stale pack doesn't hide other results.
//! Reports JSON lines; no network access, account writes, or media generation.
use super::*;
use crate::{Context, DeckState};
use std::{path::Path, sync::Arc, time::Instant};
use weapon::{AppState, data_model::Timestamped};

fn word_count(pack: &LanguagePack, sentence: Spur, language: Language) -> usize {
    pack.sentence_to_literals(&sentence, language)
        .unwrap()
        .iter()
        .filter(|literal| literal.word.text.chars().any(char::is_alphanumeric))
        .count()
}

/// Take the actual three-round placement test as a learner who knows the N
/// easiest distinct placement-eligible spellings. Ease (not raw frequency) is
/// the regression's axis. Report the resulting known gram count too: sampling,
/// smoothing, phrases, and senses mean it needn't equal N.
fn placed_deck(seed: &Deck, words: &[String], count: usize) -> Deck {
    if count == 0 {
        return seed.clone();
    }
    let known: FxHashSet<_> = words[..count].iter().collect();
    let mut session = seed.start_placement_session();
    while !crate::get_placement_session_info(session.clone()).finished {
        session.selected_words = session
            .words
            .iter()
            .filter(|word| known.contains(&word.word))
            .map(|word| word.word.clone())
            .collect();
        session = seed.advance_placement_session(session);
    }
    let event = Timestamped {
        timestamp: chrono::DateTime::from_timestamp_millis(1_700_000_000_000).unwrap(),
        within_device_events_index: 0,
        timezone: seed.context.timezone,
        event: seed.complete_placement_test(session.known_words, session.unknown_words),
    };
    Deck::finalize(
        Deck::process_event(DeckState::new(), &seed.context, &event),
        &seed.context,
    )
}

#[test]
#[ignore = "requires the French out/ pack and /tmp/yap-104/manifest_fra.json"]
fn anki_skill_levels_french() {
    check_skill_levels(Language::French);
}

#[test]
#[ignore = "requires the Spanish out/ pack and /tmp/yap-104/manifest_spa.json"]
fn anki_skill_levels_spanish() {
    check_skill_levels(Language::SpanishLatinAmerican);
}

#[test]
#[ignore = "requires the German out/ pack and /tmp/yap-104/manifest_deu.json"]
fn anki_skill_levels_german() {
    check_skill_levels(Language::German);
}

#[test]
#[ignore = "requires the Japanese out/ pack and /tmp/yap-104/manifest_jpn.json"]
fn anki_skill_levels_japanese() {
    check_skill_levels(Language::Japanese);
}

fn check_skill_levels(language: Language) {
    let mut failures = Vec::new();
    let code = language.code();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let pack = Arc::new(
        language_utils::language_pack::load_split_dir(&root.join(format!("out/{code}_for_eng")))
            .unwrap_or_else(|error| panic!("cannot load {code} pack: {error:?}")),
    );
    let rows: Vec<clips::ClipRow> = serde_json::from_slice(
        &std::fs::read(format!("/tmp/yap-104/manifest_{code}.json")).unwrap(),
    )
    .unwrap();
    let manifest_rows = rows.len();
    clips::publish_manifest(language, rows);
    let context = Context {
        study_goal: None,
        language_pack: pack.clone(),
        course: Course {
            target_language: language,
            native_language: Language::English,
        },
        timezone: chrono::FixedOffset::east_opt(0).unwrap(),
    };
    let seed = Deck::finalize(DeckState::new(), &context);
    let mut words: Vec<_> = pack
        .words_to_heteronyms
        .keys()
        .filter_map(|spur| {
            let text = pack.string_rodeo.resolve(spur);
            let (heteronym, frequency) = context.lookup_word(text)?;
            context
                .is_word_good_for_placement_test(&heteronym)
                .then(|| (text.to_owned(), frequency.ease))
        })
        .collect();
    words.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    let words: Vec<_> = words.into_iter().map(|(text, _)| text).collect();
    assert!(
        words.len() >= 5000,
        "{code}: only {} placement words",
        words.len()
    );
    for level in [0, 100, 500, 2000, 5000] {
        let deck = placed_deck(&seed, &words, level);
        let initially_known = deck.get_comprehensible_written_grams(false).iter().count();
        eprintln!("{code}/{level}: starting with {initially_known} known grams");
        let started = Instant::now();
        let planner = AnkiDeckPlanner::new(
            &deck,
            AnkiDeckOptions {
                card_types: AnkiCardTypes::Both,
                word_cards: true,
            },
            DECK_SIZE,
            "skill-level-test".into(),
            1_700_000_000_000.0,
        )
        .unwrap();
        let mut days = 0;
        let mut first_introduced = Vec::new();
        loop {
            let had_day = planner.state.borrow().day.is_some();
            let step = planner.step();
            let state = planner.state.borrow();
            if had_day && state.day.is_none() {
                days += 1;
                if days % 25 == 0 {
                    eprintln!(
                        "{code}/{level}: day {days}, {} sentences, {:.1}s",
                        state.used_sentences.len(),
                        started.elapsed().as_secs_f64()
                    );
                }
                if first_introduced.len() < 100 {
                    first_introduced.extend(
                        state
                            .simulation
                            .as_ref()
                            .unwrap()
                            .last_introduced()
                            .iter()
                            .filter_map(|c| c.written_gram().copied()),
                    );
                    first_introduced.truncate(100);
                }
            }
            if step.done {
                break;
            }
        }
        let wall_seconds = started.elapsed().as_secs_f64();
        let state = planner.state.borrow();
        let cutoff = state.empty_days >= 60;
        let lengths: Vec<_> = state
            .clip_sentence_ranks
            .keys()
            .map(|s| word_count(&pack, *s, language))
            .collect();
        let with_clip = first_introduced
            .iter()
            .filter(|g| state.clip_sentences_of.contains_key(g))
            .count();
        let sentences: Vec<_> = state
            .notes
            .iter()
            .filter_map(|note| match note {
                AnkiNote::Sentence {
                    sentence,
                    target_word,
                    ..
                } => Some(serde_json::json!({
                    "sentence": sentence, "target": target_word,
                    "words": word_count(&pack, pack.string_rodeo.get(sentence).unwrap(), language),
                })),
                _ => None,
            })
            .collect();
        let average_words = sentences
            .iter()
            .map(|s| s["words"].as_u64().unwrap())
            .sum::<u64>() as f64
            / sentences.len().max(1) as f64;
        let finish = planner.finish();
        println!(
            "YAP104 {}",
            serde_json::json!({
                "course": code, "placement_known_words": level, "initially_known_grams": initially_known,
                "manifest_rows": manifest_rows, "indexed_sentences": lengths.len(),
                "clip_lengths_le": ([1,2,3,5,10].map(|n| (n, lengths.iter().filter(|&&l| l <= n).count()))),
                "first_introduced": first_introduced.len(), "first_introduced_with_clip": with_clip,
                "sentences": sentences.len(), "word_notes": state.used_words.len(),
                "days": days, "empty_day_cutoff": cutoff, "wall_seconds": wall_seconds,
                "average_words": average_words, "first_15": sentences.iter().take(15).collect::<Vec<_>>(),
                "finish_message": finish.as_ref().ok().map(|p| &p.finish_message),
                "error": finish.as_ref().err().map(|e| format!("{e:?}")),
            })
        );
        if sentences.len() != DECK_SIZE {
            failures.push((code, level, sentences.len()));
        }
    }
    assert!(failures.is_empty(), "incomplete decks {failures:?}");
}
