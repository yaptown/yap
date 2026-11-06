// The guest only exists as WebAssembly; its host imports have no native definitions.
#![cfg(target_arch = "wasm32")]

use language_utils::language_pack::{CoreBytes, LanguagePack, PackCore, PackSentences};
use std::hint::black_box;

#[link(wasm_import_module = "bench")]
unsafe extern "C" {
    fn mark(ptr: *const u8, len: usize);
    fn part_len(part: u32) -> usize;
    fn read_part(part: u32, ptr: *mut u8, len: usize);
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

/// Inputs and iteration order are fixed; no wall clocks or random sampling in the guest.
#[unsafe(no_mangle)]
pub extern "C" fn run() {
    phase("start");
    let bytes = read(0);
    phase("core: read");
    let core = CoreBytes::from_bytes(bytes).unwrap();
    phase("core: validate");
    let core = PackCore::from(core);
    phase("core: sense frequencies");
    let core = LanguagePack::from_parts(core, None);
    phase("core: assemble");

    let bytes = read(1);
    phase("sentences: read");
    let sentences = PackSentences::from_bytes(bytes).unwrap();
    phase("sentences: validate");
    let pack = core.with_sentences(sentences);
    drop(core);
    phase("assemble pack");

    let senses: Vec<_> = pack.sense_frequencies.keys().copied().collect();
    let keys: Vec<_> = pack.sentence_keys().collect();
    assert!(!senses.is_empty() && !keys.is_empty());
    let named = |text: &str| {
        senses
            .iter()
            .copied()
            .find(|sense| {
                pack.resolve_gram(&sense.gram)
                    .to_display_string(pack.sentences.target_language)
                    == text
            })
            .expect("benchmark requires ne pas and de senses")
    };
    let required = [
        named("ne pas"),
        named("de"),
        senses[senses.len() / 2],
        senses[senses.len() * 9 / 10],
    ];
    phase("select inputs");
    for (sense, label) in required.iter().zip([
        "comprehensible: ne pas",
        "comprehensible: de",
        "comprehensible: middle",
        "comprehensible: low",
    ]) {
        black_box(pack.comprehensible_sentences(Some(sense), |rank| rank < 2000));
        phase(label);
    }
    let texts: Vec<_> = (0..1000)
        .map(|i| pack.sentence_text(keys[i * 997 % keys.len()]))
        .collect();
    black_box(&texts);
    phase("sentence_text x1000");
    for (i, text) in texts.iter().enumerate() {
        assert_eq!(
            black_box(pack.sentence_key(text)),
            Some(keys[i * 997 % keys.len()])
        );
    }
    phase("sentence_key x1000");
    for i in 0..1000 {
        black_box(
            pack.sentences_containing(&senses[i * 997 % senses.len()])
                .count(),
        );
    }
    phase("sentences_containing x1000");
    for i in 0..1000 {
        black_box(pack.gram_definition(&senses[i * 997 % senses.len()]));
    }
    phase("gram_definition x1000");
    drop((texts, keys, senses, pack));
    phase("drop pack");
}
