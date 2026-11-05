use language_utils::{
    Heteronym, OtherWord, OtherWordType, PartOfSpeech, WordType, atoms_to_literals,
    literals_to_atoms, literals_to_text, predict_whitespace,
};
use language_utils::{Language, Literal, Whitespace, Word};

fn make_word(text: &str) -> Word<String> {
    Word {
        text: text.to_string(),
        word_type: WordType::Heteronym(Heteronym {
            word: text.to_string(),
            lemma: text.to_string(),
            pos: PartOfSpeech::Noun,
        }),
    }
}

fn make_punct(text: &str) -> Word<String> {
    Word {
        text: text.to_string(),
        word_type: WordType::Other(OtherWord {
            other_tag: OtherWordType::Punct,
        }),
    }
}

fn make_literal(text: &str, whitespace: Whitespace) -> Literal<String> {
    Literal {
        word: make_word(text),
        whitespace,
    }
}

fn make_punct_literal(text: &str, whitespace: Whitespace) -> Literal<String> {
    Literal {
        word: make_punct(text),
        whitespace,
    }
}

#[test]
fn test_predict_whitespace_apostrophe() {
    let l = make_word("l'");
    let amour = make_word("amour");
    assert_eq!(
        predict_whitespace(&l, Some(&amour), Language::French),
        Whitespace::None
    );
}

#[test]
fn test_predict_whitespace_french_punct() {
    let word = make_word("bonjour");
    let exclaim = make_punct("!");
    assert_eq!(
        predict_whitespace(&word, Some(&exclaim), Language::French),
        Whitespace::NarrowNbsp
    );
}

#[test]
fn test_predict_whitespace_english_punct() {
    let word = make_word("hello");
    let exclaim = make_punct("!");
    // English should NOT have narrow nbsp before punctuation
    assert_eq!(
        predict_whitespace(&word, Some(&exclaim), Language::English),
        Whitespace::None
    );
}

#[test]
fn test_predict_whitespace_period() {
    let word = make_word("fin");
    let period = make_punct(".");
    assert_eq!(
        predict_whitespace(&word, Some(&period), Language::French),
        Whitespace::None
    );
}

#[test]
fn test_predict_whitespace_default() {
    let hello = make_word("hello");
    let world = make_word("world");
    assert_eq!(
        predict_whitespace(&hello, Some(&world), Language::English),
        Whitespace::Space
    );
}

#[test]
fn test_round_trip_simple() {
    let literals = vec![
        make_literal("Hello", Whitespace::Space),
        make_literal("world", Whitespace::None),
    ];

    let (atoms, capitalize) = literals_to_atoms(&literals, Language::English);
    let mut reconstructed = atoms_to_literals(&atoms, Language::English);
    // Manually apply capitalization for round-trip
    if capitalize && !reconstructed.is_empty() {
        reconstructed[0].word.text =
            language_utils::capitalize_first_letter(&reconstructed[0].word.text);
    }
    let original_text = literals_to_text(&literals);
    let reconstructed_text = literals_to_text(&reconstructed);

    assert_eq!(original_text, reconstructed_text);
}

#[test]
fn test_round_trip_french_apostrophe() {
    let literals = vec![
        make_literal("L'", Whitespace::None),
        make_literal("amour", Whitespace::Space),
        make_literal("est", Whitespace::Space),
        make_literal("beau", Whitespace::None),
    ];

    let (atoms, capitalize) = literals_to_atoms(&literals, Language::French);
    let mut reconstructed = atoms_to_literals(&atoms, Language::French);
    if capitalize && !reconstructed.is_empty() {
        reconstructed[0].word.text =
            language_utils::capitalize_first_letter(&reconstructed[0].word.text);
    }
    let original_text = literals_to_text(&literals);
    let reconstructed_text = literals_to_text(&reconstructed);

    assert_eq!(original_text, reconstructed_text);
}

#[test]
fn test_round_trip_french_punct() {
    let literals = vec![
        make_literal("Bonjour", Whitespace::NarrowNbsp),
        make_punct_literal("!", Whitespace::Space),
        make_literal("Comment", Whitespace::Space),
        make_literal("ça", Whitespace::Space),
        make_literal("va", Whitespace::NarrowNbsp),
        make_punct_literal("?", Whitespace::None),
    ];

    let (atoms, capitalize) = literals_to_atoms(&literals, Language::French);
    let mut reconstructed = atoms_to_literals(&atoms, Language::French);
    if capitalize && !reconstructed.is_empty() {
        reconstructed[0].word.text =
            language_utils::capitalize_first_letter(&reconstructed[0].word.text);
    }
    let original_text = literals_to_text(&literals);
    let reconstructed_text = literals_to_text(&reconstructed);

    assert_eq!(original_text, reconstructed_text);
}
