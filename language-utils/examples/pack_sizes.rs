//! Measure how a language pack's bytes are distributed across its fields.
//!
//! Serializes each field of `LanguagePack` on its own and reports the byte
//! count, including the separate shared, sentence, and translation interners.
//!
//!   cargo run --release --example pack_sizes -- out/fra_for_eng

use language_utils::language_pack::LanguagePack;

fn size_of<P, T>(value: &rkyv::Archived<T>, _: fn(&P) -> &T) -> usize
where
    T: rkyv::Archive
        + for<'a> rkyv::Serialize<
            rkyv::api::high::HighSerializer<
                rkyv::util::AlignedVec,
                rkyv::ser::allocator::ArenaHandle<'a>,
                rkyv::rancor::Error,
            >,
        >,
    rkyv::Archived<T>: rkyv::Deserialize<T, rkyv::api::high::HighDeserializer<rkyv::rancor::Error>>,
{
    let value = rkyv::deserialize::<T, rkyv::rancor::Error>(value).unwrap();
    rkyv::to_bytes::<rkyv::rancor::Error>(&value).unwrap().len()
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .expect("usage: pack_sizes <course dir, e.g. out/fra_for_eng>");
    let dir = std::path::PathBuf::from(dir);
    let total: u64 = [
        language_utils::language_pack::CORE_FILENAME,
        language_utils::language_pack::SENTENCES_FILENAME,
    ]
    .iter()
    .map(|f| std::fs::metadata(dir.join(f)).map(|m| m.len()).unwrap_or(0))
    .sum();
    println!("{}: {:.1} MB total\n", dir.display(), total as f64 / 1e6);

    let pack: LanguagePack = language_utils::language_pack::load_split_dir(&dir).unwrap();

    println!(
        "core: {} bytes; sentences: {} bytes",
        pack.core.bytes().len(),
        pack.sentences.bytes().len()
    );

    // The selector infers the native field type without reconstructing a whole pack.
    macro_rules! rows {
        ($native:ty, $part:expr, $sentence:expr; $($field:ident),* $(,)?) => {
            vec![$((stringify!($field), $sentence, size_of(&$part.$field, |part: &$native| &part.$field))),*]
        };
    }
    let mut rows: Vec<(&str, bool, usize)> = rows!(
        language_utils::language_pack::LanguagePackCore, pack.core, false;
        spur_space_fingerprint, string_rodeo, gram_rodeo, written_ease_order,
        listening_ease_order, reading_keys, senses, words_to_heteronyms,
        word_to_pronunciation, pronunciation_to_words, minimal_pairs,
        pronunciation_data, pattern_frequency_map, pronunciation_max_freq_cache,
        proper_noun_definitions, gram_frequencies, definition_texts, gram_definitions,
        definition_lists, definition_morphology, definition_segments,
        heteronym_to_grams, string_to_grams, morphemes,
    );
    rows.extend(rows!(
        language_utils::language_pack::LanguagePackSentences, pack.sentences, true;
        gram_readings, ruby_segmentations, redundant_senses, prefixed_text,
        grams_with_prefixed_text, strokes, spur_space_fingerprint, target_language,
        translation_rodeo, translations, source_gram_frequencies, homophone_practice,
        movies, books, sentence_sources, encoded_sentences, sentence_index,
        human_audio, pronunciation_audio,
    ));
    rows.sort_by_key(|&(_, _, n)| std::cmp::Reverse(n));

    let sum: usize = rows.iter().map(|&(_, _, n)| n).sum();
    let sentence_sum: usize = rows.iter().filter(|r| r.1).map(|r| r.2).sum();
    for (name, is_sentence, n) in &rows {
        println!(
            "{:>10.1} MB  {:>5.1}%  {} {name}",
            *n as f64 / 1e6,
            *n as f64 / sum as f64 * 100.0,
            if *is_sentence { "S" } else { " " },
        );
    }
    println!(
        "\n{:>10.1} MB  sum of fields (archive is {:.1} MB)",
        sum as f64 / 1e6,
        total as f64 / 1e6
    );
    println!(
        "{:>10.1} MB  sentence-side fields, including their interners ({:.0}%)",
        sentence_sum as f64 / 1e6,
        sentence_sum as f64 / sum as f64 * 100.0
    );
}
