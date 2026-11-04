//! Measure how a language pack's bytes are distributed across its fields.
//!
//! Serializes each field of `LanguagePack` on its own and reports the byte
//! count, including the separate shared, sentence, and translation interners.
//!
//!   cargo run --release --example pack_sizes -- out/fra_for_eng

use language_utils::language_pack::LanguagePack;

fn size_of<T>(value: &T) -> usize
where
    T: for<'a> rkyv::Serialize<
            rkyv::api::high::HighSerializer<
                rkyv::util::AlignedVec,
                rkyv::ser::allocator::ArenaHandle<'a>,
                rkyv::rancor::Error,
            >,
        >,
{
    rkyv::to_bytes::<rkyv::rancor::Error>(value)
        .map(|b| b.len())
        .unwrap_or(0)
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

    // "sentence-side" = the fields that only exist to serve corpus sentences.
    let mut rows: Vec<(&str, bool, usize)> = vec![
        ("translations", true, size_of(&pack.sentences.translations)),
        (
            "encoded_sentences",
            true,
            size_of(&pack.sentences.encoded_sentences),
        ),
        (
            "sentences_containing_gram_index",
            true,
            size_of(&pack.sentences.sentences_containing_gram_index),
        ),
        (
            "sentence_sources",
            true,
            size_of(&pack.sentences.sentence_sources),
        ),
        ("movies", true, size_of(&pack.sentences.movies)),
        ("books", true, size_of(&pack.sentences.books)),
        ("human_audio", true, size_of(&pack.sentences.human_audio)),
        (
            "pronunciation_audio",
            false,
            size_of(&pack.pronunciation_audio),
        ),
        (
            "words_to_heteronyms",
            false,
            size_of(&pack.words_to_heteronyms),
        ),
        (
            "source_gram_frequencies",
            false,
            size_of(&pack.sentences.source_gram_frequencies),
        ),
        (
            "word_to_pronunciation",
            false,
            size_of(&pack.word_to_pronunciation),
        ),
        (
            "pronunciation_to_words",
            false,
            size_of(&pack.pronunciation_to_words),
        ),
        ("minimal_pairs", false, size_of(&pack.minimal_pairs)),
        (
            "pronunciation_data",
            false,
            size_of(&pack.pronunciation_data),
        ),
        (
            "pattern_frequency_map",
            false,
            size_of(&pack.pattern_frequency_map),
        ),
        (
            "homophone_practice",
            false,
            size_of(&pack.sentences.homophone_practice),
        ),
        (
            "pronunciation_max_freq_cache",
            false,
            size_of(&pack.pronunciation_max_freq_cache),
        ),
        (
            "proper_noun_definitions",
            false,
            size_of(&pack.proper_noun_definitions),
        ),
        ("gram_frequencies", false, size_of(&pack.gram_frequencies)),
        ("gram_definitions", false, size_of(&pack.gram_definitions)),
        (
            "heteronym_to_grams",
            false,
            size_of(&pack.heteronym_to_grams),
        ),
        ("string_to_grams", false, size_of(&pack.string_to_grams)),
        ("morphemes", false, size_of(&pack.morphemes)),
    ];

    rows.push(("gram_rodeo", false, size_of(&pack.gram_rodeo)));
    rows.push(("string_rodeo", false, size_of(&pack.string_rodeo)));
    rows.push((
        "translation_rodeo",
        true,
        size_of(&pack.sentences.translation_rodeo),
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
