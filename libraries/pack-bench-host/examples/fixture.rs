//! Tiny valid pack for exercising every guest scenario while real packs regenerate.
use language_utils::{
    Atom, Course, Gram, GramVocabEntry, Heteronym, Language, PartOfSpeech,
    PhrasebookDefinitionEntry, ReadingGramFrequencyEntry, ReadingTaggedGram, SentenceGram,
    SentenceGrams, TaggedGram, Word, WordType,
};
use pack_builder::{ConsolidatedLanguageData, GramFrequencyList};

fn main() {
    let directory = std::env::args()
        .nth(1)
        .expect("usage: fixture NEW_DIRECTORY");
    // Refuse to overwrite anything, especially a real course directory.
    std::fs::create_dir(&directory).unwrap();
    let mut data = ConsolidatedLanguageData::default();
    for (i, text) in ["ne pas", "de", "bonjour", "ailleurs"]
        .into_iter()
        .enumerate()
    {
        let gram = Gram(
            text.split_whitespace()
                .map(|word| {
                    Atom::Tok(Word {
                        text: word.into(),
                        word_type: WordType::Heteronym(Heteronym {
                            word: word.into(),
                            lemma: word.into(),
                            pos: PartOfSpeech::Noun,
                        }),
                    })
                })
                .collect(),
        );
        let sense = TaggedGram {
            gram: gram.clone(),
            sense: None,
        };
        let reading = ReadingTaggedGram {
            sense: sense.clone(),
            reading: None,
        };
        data.gram_vocabulary.push(GramVocabEntry {
            atoms: gram,
            frequency: 100 - i as u32,
        });
        data.gram_frequencies
            .entries
            .push(ReadingGramFrequencyEntry {
                count: 100 - i as u32,
                direct_count: 100 - i as u32,
                disambiguation_key: i as u32,
                gram: reading.clone(),
            });
        data.target_language_sentences.push(text.into());
        data.translations
            .push((text.into(), vec![format!("translation {i}")]));
        data.encoded_sentences.push((
            text.into(),
            SentenceGrams {
                grams: vec![SentenceGram::Learnable(reading)],
                capitalize_first: false,
                multiword_terms: vec![],
                low_confidence_multiword_terms: vec![],
            },
        ));
        data.phrasebook.insert(
            sense,
            PhrasebookDefinitionEntry {
                target_language_multi_word_term: text.into(),
                meaning: format!("definition {i}"),
                additional_notes: String::new(),
                note_is_curated: false,
                target_language_example: text.into(),
                native_language_example: format!("translation {i}"),
                informal: false,
                compositional: false,
                cognate: false,
                false_cognate: false,
                can_be_translated_literally: false,
            },
        );
    }
    data.gram_frequencies = GramFrequencyList {
        total_count: 394,
        ..data.gram_frequencies
    };
    let pack = pack_builder::build(
        data,
        Course {
            target_language: Language::French,
            native_language: Language::English,
        },
    );
    language_utils::language_pack::write_split_dir(std::path::Path::new(&directory), pack).unwrap();
    println!("synthetic pack: {directory}");
}
