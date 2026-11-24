//! Write every phonemized sentence of a pack as JSONL: its text and, per gram,
//! the gram's words, their parts of speech, its corpus count, and its phoneme
//! sequence (format: `sentence_phonemes`).
//! Feeds TTS training in lexide/tts.
//!
//!   cargo run --release --example dump_sentence_phonemes -- out/eng_for_fra > eng.jsonl

use language_utils::{Atom, SentenceGram, WordType, language_pack::LanguagePack};
use std::io::Write;

fn main() {
    let dir = std::env::args()
        .nth(1)
        .expect("usage: dump_sentence_phonemes <course dir, e.g. out/eng_for_fra>");
    let pack: LanguagePack = language_utils::language_pack::load_split_dir(dir.as_ref()).unwrap();
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());
    let (mut written, mut skipped) = (0usize, 0usize);
    for key in pack.sentence_keys() {
        let Some((sentence, Some(phonemes))) = pack.sentence_grams_with_phonemes(key) else {
            skipped += 1;
            continue;
        };
        assert_eq!(sentence.grams.len(), phonemes.len());
        let grams: Vec<_> = sentence
            .grams
            .iter()
            .zip(&phonemes)
            .map(|(gram, sequence)| {
                let gram = match gram {
                    SentenceGram::Learnable(g) | SentenceGram::Obvious(g) => g,
                };
                let (words, pos): (Vec<String>, Vec<serde_json::Value>) = pack
                    .resolve_gram(&gram.sense.gram)
                    .0
                    .into_iter()
                    .filter_map(|atom| match atom {
                        Atom::Tok(word) => {
                            let pos = match &word.word_type {
                                WordType::Heteronym(h) => serde_json::to_value(h.pos),
                                WordType::Other(o) => serde_json::to_value(o.other_tag),
                            };
                            Some((word.text, pos.unwrap()))
                        }
                        Atom::Control(_) => None,
                    })
                    .unzip();
                // Corpus occurrences of the gram over all its senses: a rarity / surprisal prior.
                let count = pack.gram_frequency_total(gram.sense.gram).map_or(0, |f| f.count);
                serde_json::json!({"words": words, "pos": pos, "count": count, "phonemes": sequence})
            })
            .collect();
        let row = serde_json::json!({"text": pack.sentence_text(key), "grams": grams});
        serde_json::to_writer(&mut out, &row).unwrap();
        out.write_all(b"\n").unwrap();
        written += 1;
    }
    eprintln!("{written} sentences written, {skipped} without phonemes");
}
