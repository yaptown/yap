//! Run with pack-bench-host and any directory named for a language_utils::COURSES entry.
#![cfg(target_arch = "wasm32")]

use chrono::{TimeZone, Utc};
use language_utils::language_pack::{LanguagePack, PackCore, PackSentences};
use std::sync::Arc;
use yap_frontend_rs::{Context, Deck, DeckState};

#[link(wasm_import_module = "bench")]
unsafe extern "C" {
    fn mark(ptr: *const u8, len: usize);
    fn part_len(part: u32) -> usize;
    fn read_part(part: u32, ptr: *mut u8, len: usize);
    fn course_index() -> u32;
    fn challenges(count: u32);
}

fn phase(label: &str) {
    unsafe { mark(label.as_ptr(), label.len()) }
}

fn read(part: u32) -> rkyv::util::AlignedVec {
    let mut bytes = rkyv::util::AlignedVec::new();
    bytes.resize(unsafe { part_len(part) }, 0);
    unsafe { read_part(part, bytes.as_mut_ptr(), bytes.len()) };
    bytes
}

#[unsafe(no_mangle)]
pub extern "C" fn run() {
    let core = PackCore::from_bytes(read(0)).unwrap();
    let sentences = PackSentences::from_bytes(read(1)).unwrap();
    let language_pack = Arc::new(LanguagePack::from_parts(core, Some(sentences)));
    phase("load");
    let context = Context {
        study_goal: None,
        language_pack,
        course: language_utils::COURSES[unsafe { course_index() } as usize],
        timezone: chrono::FixedOffset::east_opt(0).unwrap(),
    };
    let deck = <Deck as weapon::AppState>::finalize(DeckState::new(), &context);
    phase("finalize");
    let fixed_time = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let mut simulator = deck.simulate_usage(fixed_time);
    let mut count = 0;
    for _ in 0..365 {
        let mut day = simulator.next_day();
        count += day.by_ref().count();
        simulator = day.finish_day();
    }
    phase("365 days");
    unsafe { challenges(count.try_into().unwrap()) };
    assert!(count > 1000);
    drop(simulator);
    drop(context);
    phase("drop");
}
