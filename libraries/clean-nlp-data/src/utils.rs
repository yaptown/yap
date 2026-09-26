use crate::classify::SimplifiedTokenPrime;
use language_utils::{Language, Whitespace};

/// The final gold writer and proposal validation share the constructor boundary.
pub fn validate_gold_output(output: &serde_json::Value) -> anyhow::Result<()> {
    let row: generate_data::gold::CleanedSentence = serde_json::from_value(output.clone())?;
    generate_data::gold::to_lexide(row.sentence, row.tokens)?;
    Ok(())
}

#[derive(Debug)]
pub enum ValidationResult {
    Valid,
    AutoFixed,
    Invalid {
        original: String,
        reconstructed: String,
    },
}

/// Derive gaps from the original sentence, then validate the corrected analysis.
/// Token text is never rewritten to make an alignment succeed.
pub fn validate_and_fix_whitespace(
    original: &str,
    corrected_tokens: &mut Vec<SimplifiedTokenPrime>,
    language: Language,
) -> ValidationResult {
    let invalid = |tokens: &[SimplifiedTokenPrime]| ValidationResult::Invalid {
        original: original.to_owned(),
        reconstructed: tokens
            .iter()
            .map(|t| format!("{}{}", t.text, t.whitespace))
            .collect(),
    };
    let Some(gaps) = aligned_gaps(original, corrected_tokens) else {
        return invalid(corrected_tokens);
    };
    let mut auto_fixed = false;
    for (token, gap) in corrected_tokens.iter_mut().zip(gaps) {
        auto_fixed |= token.whitespace != gap;
        token.whitespace = gap;
    }
    // Spacing-sensitive corrections see the source's gaps, not the model's hints.
    token_corrections::correct_tokens(language, corrected_tokens);
    if language == Language::French {
        for token in corrected_tokens.iter_mut() {
            if let Some(word) = token
                .lemma
                .strip_prefix("se ")
                .or_else(|| token.lemma.strip_prefix("s'"))
            {
                token.lemma = word.to_owned();
            }
        }
    }
    let tokens = corrected_tokens
        .iter()
        .map(|token| generate_data::gold::CleanedToken {
            text: token.text.clone(),
            whitespace: token.whitespace.into(),
            pos: token.pos,
            lemma: token.lemma.clone(),
            dep: "root".into(),
            head: 0,
        })
        .collect();
    if generate_data::gold::to_lexide(original.to_owned(), tokens).is_ok() {
        if auto_fixed {
            ValidationResult::AutoFixed
        } else {
            ValidationResult::Valid
        }
    } else {
        invalid(corrected_tokens)
    }
}

fn aligned_gaps(original: &str, tokens: &[SimplifiedTokenPrime]) -> Option<Vec<Whitespace>> {
    let mut remaining = original;
    let gaps = tokens
        .iter()
        .map(|token| {
            let after_text = remaining.strip_prefix(token.text.as_str())?;
            remaining = after_text.trim_start_matches(char::is_whitespace);
            let gap = &after_text[..after_text.len() - remaining.len()];
            // This is an ingestion boundary: accept only literal gaps from the source.
            gap.parse::<lexide_types::Whitespace>().ok().map(Into::into)
        })
        .collect::<Option<Vec<_>>>()?;
    remaining.is_empty().then_some(gaps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use language_utils::{PartOfSpeechTag, Whitespace};

    fn token(text: &str, whitespace: Whitespace) -> SimplifiedTokenPrime {
        SimplifiedTokenPrime {
            text: text.into(),
            whitespace,
            pos: PartOfSpeechTag::Noun,
            lemma: text.into(),
        }
    }

    #[test]
    fn broken_cleaner_rows_are_rejected() {
        for (original, text, whitespace) in [
            ("oui, non", "oui", ", non"),
            ("hello", "wrong", ""),
            ("two words", " two words", ""),
        ] {
            let output = serde_json::json!({"sentence":original,"tokens":[{"text":text,"whitespace":whitespace,"pos":"INTJ","lemma":text,"dep":"root","head":0}]});
            assert!(validate_gold_output(&output).is_err());
        }
    }

    #[test]
    fn valid_supported_gaps_are_preserved() {
        for gap in [
            Whitespace::None,
            Whitespace::Space,
            Whitespace::Nbsp,
            Whitespace::NarrowNbsp,
        ] {
            let original = format!("Hello{gap}world");
            let mut tokens = vec![token("Hello", gap), token("world", Whitespace::None)];
            assert!(matches!(
                validate_and_fix_whitespace(&original, &mut tokens, Language::English),
                ValidationResult::Valid
            ));
            assert_eq!(tokens[0].whitespace, gap);
        }
    }

    #[test]
    fn multiword_token_text_is_preserved_but_edge_whitespace_is_rejected() {
        for text in ["New York", "pommes de terre", "वास्तव में"] {
            let mut tokens = vec![token(text, Whitespace::None)];
            assert!(matches!(
                validate_and_fix_whitespace(text, &mut tokens, Language::English),
                ValidationResult::Valid
            ));
            assert_eq!(tokens[0].text, text);
        }
        for text in [" two words", "two words ", ""] {
            let mut tokens = vec![token(text, Whitespace::None)];
            assert!(matches!(
                validate_and_fix_whitespace(text, &mut tokens, Language::English),
                ValidationResult::Invalid { .. }
            ));
        }
    }
    #[test]
    fn test_narrow_nbsp_replaced_with_regular_space() {
        // Original: "Hello" + narrow non-breaking space + "world"
        let original = "Hello\u{202F}world";

        // LLM output: "Hello" + regular space + "world"
        let mut tokens = vec![
            token("Hello", Whitespace::Space),
            token("world", Whitespace::None),
        ];

        let result = validate_and_fix_whitespace(original, &mut tokens, Language::French);

        // Should auto-fix to use narrow non-breaking space
        assert!(matches!(result, ValidationResult::AutoFixed));
        assert_eq!(tokens[0].whitespace.as_str(), "\u{202F}");

        // Verify reconstruction matches original
        let reconstructed: String = tokens
            .iter()
            .map(|t| format!("{}{}", t.text, t.whitespace.as_str()))
            .collect();
        assert_eq!(reconstructed, original);
    }

    #[test]
    fn test_missing_narrow_nbsp() {
        // Original: "Hello" + narrow non-breaking space + "world"
        let original = "Hello\u{202F}world";

        // LLM output: "Hello" + no space + "world"
        let mut tokens = vec![
            token("Hello", Whitespace::None),
            token("world", Whitespace::None),
        ];

        let result = validate_and_fix_whitespace(original, &mut tokens, Language::French);

        // Should auto-fix by adding the narrow non-breaking space
        assert!(matches!(result, ValidationResult::AutoFixed));
        assert_eq!(tokens[0].whitespace.as_str(), "\u{202F}");

        // Verify reconstruction matches original
        let reconstructed: String = tokens
            .iter()
            .map(|t| format!("{}{}", t.text, t.whitespace.as_str()))
            .collect();
        assert_eq!(reconstructed, original);
    }

    #[test]
    fn test_multiple_tokens_with_mixed_whitespace() {
        // Original: "A" + nbsp + "B" + regular space + "C"
        let original = "A\u{00A0}B C";

        // LLM output: "A" + regular space + "B" + regular space + "C"
        let mut tokens = vec![
            token("A", Whitespace::Space),
            token("B", Whitespace::Space),
            token("C", Whitespace::None),
        ];

        let result = validate_and_fix_whitespace(original, &mut tokens, Language::French);

        // Should auto-fix
        assert!(matches!(result, ValidationResult::AutoFixed));
        assert_eq!(tokens[0].whitespace.as_str(), "\u{00A0}");
        assert_eq!(tokens[1].whitespace.as_str(), " ");

        // Verify reconstruction matches original
        let reconstructed: String = tokens
            .iter()
            .map(|t| format!("{}{}", t.text, t.whitespace.as_str()))
            .collect();
        assert_eq!(reconstructed, original);
    }

    #[test]
    fn test_already_valid() {
        let original = "Hello world";

        let mut tokens = vec![
            token("Hello", Whitespace::Space),
            token("world", Whitespace::None),
        ];

        let result = validate_and_fix_whitespace(original, &mut tokens, Language::French);

        assert!(matches!(result, ValidationResult::Valid));
    }

    #[test]
    fn test_narrow_nbsp_bug_reproduction() {
        // Original: three tokens with narrow nbsp between first two
        // "A" + narrow nbsp + "B" + regular space + "C"
        let original = "A\u{202F}B C";

        // LLM output: missing the narrow nbsp
        let mut tokens = vec![
            token("A", Whitespace::None),
            token("B", Whitespace::Space),
            token("C", Whitespace::None),
        ];

        let result = validate_and_fix_whitespace(original, &mut tokens, Language::French);

        // Should auto-fix
        assert!(matches!(result, ValidationResult::AutoFixed));

        // Check that we don't get narrow nbsp FOLLOWED by regular space
        assert_eq!(tokens[0].whitespace.as_str(), "\u{202F}");
        assert_eq!(tokens[1].whitespace.as_str(), " ");

        // Verify reconstruction matches original exactly
        let reconstructed: String = tokens
            .iter()
            .map(|t| format!("{}{}", t.text, t.whitespace.as_str()))
            .collect();
        assert_eq!(
            reconstructed,
            original,
            "Reconstructed should match original. Got: {:?}, Expected: {:?}",
            reconstructed.chars().collect::<Vec<_>>(),
            original.chars().collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_real_world_bug_nbsp_becomes_narrow_nbsp_plus_space() {
        // Real bug: Original has regular nbsp (\u{a0}), LLM returns regular space
        // Original: "faire" + nbsp + "?"
        let original = "faire\u{a0}?";

        // LLM output: "faire" + regular space + "?"
        let mut tokens = vec![
            token("faire", Whitespace::Space),
            token("?", Whitespace::None),
        ];

        let result = validate_and_fix_whitespace(original, &mut tokens, Language::French);

        // Should auto-fix
        assert!(matches!(result, ValidationResult::AutoFixed));

        // Should have nbsp, NOT narrow nbsp + space
        assert_eq!(
            tokens[0].whitespace.as_str(),
            "\u{a0}",
            "Expected regular nbsp, got: {:?}",
            tokens[0].whitespace.as_str().chars().collect::<Vec<_>>()
        );

        // Verify reconstruction matches original exactly
        let reconstructed: String = tokens
            .iter()
            .map(|t| format!("{}{}", t.text, t.whitespace.as_str()))
            .collect();
        assert_eq!(
            reconstructed,
            original,
            "Reconstructed should match original. Got: {:?}, Expected: {:?}",
            reconstructed.chars().collect::<Vec<_>>(),
            original.chars().collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_exact_bug_reproduction() {
        // Exact bug from the error message
        // Original sentence has narrow nbsp (\u{202f})
        let original = "C'est pour quoi faire\u{202f}?";

        // LLM returns WITH regular nbsp (\u{a0}) instead of narrow nbsp
        let mut tokens = vec![
            token("C'", Whitespace::None),
            token("est", Whitespace::Space),
            token("pour", Whitespace::Space),
            token("quoi", Whitespace::Space),
            token("faire", Whitespace::Nbsp), // Regular nbsp instead of narrow nbsp!
            token("?", Whitespace::None),
        ];

        println!(
            "Before: {:?}",
            tokens
                .iter()
                .map(|t| format!("{:?}", t.whitespace.as_str().chars().collect::<Vec<_>>()))
                .collect::<Vec<_>>()
        );
        let result = validate_and_fix_whitespace(original, &mut tokens, Language::French);
        println!(
            "After: {:?}",
            tokens
                .iter()
                .map(|t| format!("{:?}", t.whitespace.as_str().chars().collect::<Vec<_>>()))
                .collect::<Vec<_>>()
        );

        // Should auto-fix
        assert!(matches!(result, ValidationResult::AutoFixed));

        // Should have narrow nbsp, NOT narrow nbsp + space
        assert_eq!(
            tokens[4].whitespace.as_str(),
            "\u{202f}",
            "Expected narrow nbsp only, got: {:?}",
            tokens[4].whitespace.as_str().chars().collect::<Vec<_>>()
        );

        // Verify reconstruction matches original exactly
        let reconstructed: String = tokens
            .iter()
            .map(|t| format!("{}{}", t.text, t.whitespace.as_str()))
            .collect();
        assert_eq!(
            reconstructed,
            original,
            "Reconstructed should match original. Got: {:?}, Expected: {:?}",
            reconstructed.chars().collect::<Vec<_>>(),
            original.chars().collect::<Vec<_>>()
        );
    }
    #[test]
    fn whitespace_alignment_never_inserts_letters() {
        let mut tokens = vec![token("a", Whitespace::Space), token("bc", Whitespace::None)];
        assert!(matches!(
            validate_and_fix_whitespace("ab c ", &mut tokens, Language::English),
            ValidationResult::Invalid { .. }
        ));
        assert_eq!(tokens[0].text, "a");
        assert_eq!(tokens[1].text, "bc");
        assert_eq!(tokens[0].whitespace, Whitespace::Space);
    }

    #[test]
    fn unsupported_source_gaps_and_unaligned_text_are_rejected() {
        for original in ["a  b", "a\tb", "a\u{2009}b", "a, b", " a b", "a b extra"] {
            let mut tokens = vec![token("a", Whitespace::Space), token("b", Whitespace::None)];
            assert!(
                matches!(
                    validate_and_fix_whitespace(original, &mut tokens, Language::English),
                    ValidationResult::Invalid { .. }
                ),
                "{original:?}"
            );
        }
    }

    #[test]
    fn displaced_and_multibyte_gaps_come_from_the_source() {
        let mut tokens = vec![
            token("हाँ", Whitespace::None),
            token("जाओ", Whitespace::Space),
        ];
        assert!(matches!(
            validate_and_fix_whitespace("हाँ\u{202f}जाओ", &mut tokens, Language::Hindi),
            ValidationResult::AutoFixed
        ));
        assert_eq!(tokens[0].whitespace, Whitespace::NarrowNbsp);
        assert_eq!(tokens[1].whitespace, Whitespace::None);
        assert!(matches!(
            validate_and_fix_whitespace("हाँ\u{202f}जाओ", &mut tokens, Language::Hindi),
            ValidationResult::Valid
        ));
    }
}
