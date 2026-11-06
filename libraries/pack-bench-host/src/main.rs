//! From the workspace root (use a separate target directory during pack regeneration):
//! CARGO_TARGET_DIR=target-bench CARGO_PROFILE_RELEASE_LTO=true cargo build --release -p pack-bench --target wasm32-unknown-unknown
//! CARGO_TARGET_DIR=target-bench cargo run --release -p pack-bench-host -- target-bench/wasm32-unknown-unknown/release/pack_bench.wasm out/fra_for_eng
//!
//! The guest path can point into another checkout. Keep the host/Wasmtime version,
//! compiler, optimization flags and input pack bytes fixed for comparisons.
//! Fuel is Wasmtime's deterministic abstract work metric, not CPU instructions:
//! host I/O/copies and some bulk-memory work are not charged per byte. "Read"
//! phases include guest allocation/zeroing and a host copy, not WASI filesystem work.
//! Memory is linear-memory high-water size (MiB), not live allocations or host RSS.
//! Browser imports are traps except the fixed Date clock needed by simulation:
//! DeckState::new → FSRS Seed::default, and NextCardsIterator's new FSRS cards.
//! Date construction/getTime/drop are pinned to 2024-01-01 UTC, never host time.
//! Smoke fixture (never writes out/):
//! CARGO_TARGET_DIR=target-bench cargo run --release -p pack-bench-host --example fixture -- /tmp/pack-bench-fixture
//!
//! Whole-app deterministic 365-day simulation (directory basename selects the course):
//! CARGO_TARGET_DIR=target-bench CARGO_PROFILE_RELEASE_LTO=true cargo build --release -p pack-bench --example simulation --target wasm32-unknown-unknown
//! CARGO_TARGET_DIR=target-bench cargo run --release -p pack-bench-host -- target-bench/wasm32-unknown-unknown/release/examples/simulation.wasm out/fra_for_eng
//! Run twice and compare fuel, memory and challenges (not wall time). Load includes
//! guest allocation and validation; finalize includes context setup; 365 days uses
//! 2024-01-01 UTC. Total includes all teardown: the explicit drop mark and the
//! initial deck's implicit drop on return (simulate_usage borrows/clones it).
//! Linear memory cannot shrink, so the final size captures its peak between marks.

use language_utils::language_pack::{CORE_FILENAME, SENTENCES_FILENAME};
use std::{path::Path, time::Instant};
use wasmtime::{Caller, Config, Engine, Linker, Module, Store};

const FUEL: u64 = 1_000_000_000_000_000;

struct State {
    parts: [Vec<u8>; 2],
    previous_fuel: u64,
    previous_time: Instant,
    peak_bytes: usize,
}

fn main() -> wasmtime::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: pack-bench-host GUEST.wasm PACK_DIRECTORY");
        std::process::exit(2);
    }
    let dir = Path::new(&args[2]);
    let parts = [
        std::fs::read(dir.join(CORE_FILENAME))?,
        std::fs::read(dir.join(SENTENCES_FILENAME))?,
    ];
    println!("guest: {}\npack: {}", args[1], dir.display());
    println!(
        "{:<30} {:>18} {:>12} {:>12}",
        "phase", "fuel", "memory MiB", "wall ms"
    );
    let mut config = Config::new();
    config.consume_fuel(true);
    let engine = Engine::new(&config)?;
    let module = Module::from_file(&engine, &args[1])?;
    let mut linker = Linker::new(&engine);
    linker.func_wrap(
        "bench",
        "part_len",
        |caller: Caller<'_, State>, part: u32| -> u32 {
            caller.data().parts[part as usize].len().try_into().unwrap()
        },
    )?;
    linker.func_wrap(
        "bench",
        "read_part",
        |mut caller: Caller<'_, State>, part: u32, ptr: u32, len: u32| {
            let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
            let (memory, state) = memory.data_and_store_mut(&mut caller);
            let bytes = &state.parts[part as usize];
            assert_eq!(len as usize, bytes.len());
            memory[ptr as usize..ptr as usize + len as usize].copy_from_slice(bytes);
        },
    )?;
    linker.func_wrap(
        "bench",
        "mark",
        |mut caller: Caller<'_, State>, ptr: u32, len: u32| -> wasmtime::Result<()> {
            let now = Instant::now();
            let fuel = caller.get_fuel()?;
            let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
            let size = memory.data_size(&caller);
            let label = std::str::from_utf8(
                &memory.data(&caller)[ptr as usize..ptr as usize + len as usize],
            )?
            .to_owned();
            let state = caller.data_mut();
            println!(
                "{:<30} {:>18} {:>12.3} {:>12.3}",
                label,
                state.previous_fuel - fuel,
                size as f64 / 1048576.0,
                now.duration_since(state.previous_time).as_secs_f64() * 1000.0
            );
            state.peak_bytes = state.peak_bytes.max(size);
            state.previous_fuel = fuel;
            // Do not bill printing this row to the following phase's wall time.
            state.previous_time = Instant::now();
            Ok(())
        },
    )?;
    let course_index = language_utils::COURSES.iter().position(|course| {
        let name = format!(
            "{}_for_{}",
            course.target_language.code(),
            course.native_language.code()
        );
        Some(name.as_str()) == dir.file_name().and_then(|s| s.to_str())
    });
    linker.func_wrap("bench", "course_index", move || -> wasmtime::Result<u32> {
        Ok(course_index
            .ok_or_else(|| wasmtime::Error::msg("pack directory must be a course code"))?
            as u32)
    })?;
    linker.func_wrap("bench", "challenges", |count: u32| {
        println!("challenges: {count}");
    })?;
    // Exact imports for the pinned js-sys version, not a general JS emulator.
    // Only our singleton Date handle may be read or dropped; other objects trap.
    const DATE: u32 = 132;
    linker.func_wrap(
        "__wbindgen_placeholder__",
        "__wbg_new_0_445c13a750296eb6",
        || DATE,
    )?;
    linker.func_wrap(
        "__wbindgen_placeholder__",
        "__wbg_getTime_00b3f7db575e4ef5",
        |date: u32| -> wasmtime::Result<f64> {
            wasmtime::ensure!(date == DATE, "getTime called on an unknown JS object");
            Ok(1_704_067_200_000.0)
        },
    )?;
    linker.func_wrap(
        "__wbindgen_placeholder__",
        "__wbindgen_object_drop_ref",
        |object: u32| -> wasmtime::Result<()> {
            wasmtime::ensure!(object == DATE, "drop called on an unknown JS object");
            Ok(())
        },
    )?;
    linker.define_unknown_imports_as_traps(&module)?;
    let mut store = Store::new(
        &engine,
        State {
            parts,
            previous_fuel: FUEL,
            previous_time: Instant::now(),
            peak_bytes: 0,
        },
    );
    store.set_fuel(FUEL)?;
    let instance = linker.instantiate(&mut store, &module)?;
    let run = instance.get_typed_func::<(), ()>(&mut store, "run")?;
    store.data_mut().previous_time = Instant::now();
    run.call(&mut store, ())?;
    let memory = instance.get_memory(&mut store, "memory").unwrap();
    let peak = store.data().peak_bytes.max(memory.data_size(&store));
    println!(
        "peak linear memory: {:.3} MiB ({} pages)",
        peak as f64 / 1048576.0,
        peak / 65536
    );
    println!("total fuel: {}", FUEL - store.get_fuel()?);
    Ok(())
}
