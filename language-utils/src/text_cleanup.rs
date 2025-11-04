//! Language-specific text cleanup utilities
//!
//! This module provides functions for cleaning up and normalizing text
//! according to language-specific typographic rules.

use crate::Language;

/// Clean up text according to language-specific rules
///
/// Currently supported languages:
/// - French: Fixes spacing before high punctuation marks
pub fn cleanup_sentence(sentence: String, language: Language) -> String {
    match language {
        Language::French => cleanup_french_sentence(sentence),
        _ => sentence,
    }
}

/// Clean up French sentence punctuation spacing
///
/// In French typography, high punctuation marks (! ?) should be preceded
/// by a thin non-breaking space (U+202F). This function ensures proper spacing:
/// - Converts regular spaces before high punctuation to thin non-breaking spaces
/// - Inserts thin non-breaking spaces if they're missing entirely
pub fn cleanup_french_sentence(sentence: String) -> String {
    const THIN_NBSP: char = '\u{202F}'; // Thin non-breaking space
    const NBSP: char = '\u{00A0}'; // Regular non-breaking space
    const HIGH_PUNCTUATION: &[char] = &['!', '?'];

    let mut result = String::with_capacity(sentence.len() + 10);
    let chars: Vec<char> = sentence.chars().collect();

    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];

        // Check if this is high punctuation
        if HIGH_PUNCTUATION.contains(&ch) {
            // Check what comes before
            if i > 0 {
                let prev_char = chars[i - 1];
                if prev_char == ' ' || prev_char == NBSP {
                    // Replace the previous space with thin nbsp
                    result.pop(); // Remove the space we just added
                    result.push(THIN_NBSP);
                } else if prev_char != THIN_NBSP {
                    // No space at all, insert thin nbsp
                    result.push(THIN_NBSP);
                }
                // If it's already a thin nbsp, do nothing
            }
            result.push(ch);
        } else {
            result.push(ch);
        }

        i += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_french_cleanup_regular_space() {
        // Regular space should be converted to thin nbsp
        let input = "Bonjour !".to_string();
        let expected = "Bonjour\u{202F}!";
        assert_eq!(cleanup_french_sentence(input), expected);
    }

    #[test]
    fn test_french_cleanup_no_space() {
        // No space should insert thin nbsp
        let input = "Bonjour!".to_string();
        let expected = "Bonjour\u{202F}!";
        assert_eq!(cleanup_french_sentence(input), expected);
    }

    #[test]
    fn test_french_cleanup_already_correct() {
        // Already correct thin nbsp should remain unchanged
        let input = format!("Bonjour\u{202F}!");
        let expected = format!("Bonjour\u{202F}!");
        assert_eq!(cleanup_french_sentence(input), expected);
    }

    #[test]
    fn test_french_cleanup_regular_nbsp() {
        // Regular nbsp should be converted to thin nbsp
        let input = format!("Bonjour\u{00A0}!");
        let expected = format!("Bonjour\u{202F}!");
        assert_eq!(cleanup_french_sentence(input), expected);
    }

    #[test]
    fn test_french_cleanup_all_punctuation() {
        // Test all high punctuation marks (! and ?)
        let input = "Question ? Exclamation ! Colon : Semicolon ;".to_string();
        let expected = "Question\u{202F}? Exclamation\u{202F}! Colon : Semicolon ;";
        assert_eq!(cleanup_french_sentence(input), expected);
    }

    #[test]
    fn test_french_cleanup_multiple_punctuation() {
        // Test multiple punctuation in one sentence
        let input = "What ?! Really !".to_string();
        let expected = "What\u{202F}?\u{202F}! Really\u{202F}!";
        assert_eq!(cleanup_french_sentence(input), expected);
    }

    #[test]
    fn test_french_cleanup_no_change_needed() {
        // Sentence without high punctuation should be unchanged
        let input = "Bonjour, comment allez-vous.".to_string();
        let expected = "Bonjour, comment allez-vous.";
        assert_eq!(cleanup_french_sentence(input), expected);
    }

    #[test]
    fn test_french_cleanup_punctuation_at_start() {
        // Edge case: punctuation at the very start (shouldn't happen in real text)
        let input = "!Wow".to_string();
        let expected = "!Wow";
        assert_eq!(cleanup_french_sentence(input), expected);
    }

    #[test]
    fn test_cleanup_sentence_french() {
        // Test the general cleanup_sentence function with French
        let input = "Bonjour !".to_string();
        let expected = "Bonjour\u{202F}!";
        assert_eq!(cleanup_sentence(input, Language::French), expected);
    }

    #[test]
    fn test_cleanup_sentence_english() {
        // Test the general cleanup_sentence function with English (no changes)
        let input = "Hello!".to_string();
        let expected = "Hello!";
        assert_eq!(cleanup_sentence(input, Language::English), expected);
    }
}
