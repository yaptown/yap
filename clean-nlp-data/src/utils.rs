use crate::classify::NlpCorrectionResponse;

#[derive(Debug)]
pub enum ValidationResult {
    /// The response matches the original text exactly
    Valid,
    /// The response had a single-space mismatch that was auto-fixed
    AutoFixed,
    /// The response has mismatches that cannot be auto-fixed
    Invalid {
        original: String,
        reconstructed: String,
    },
}

/// Validate that an LLM correction response matches the original text.
/// If there's a single-space difference, automatically fix it.
pub fn validate_and_fix_whitespace(
    original: &str,
    correction: &mut NlpCorrectionResponse,
) -> ValidationResult {
    let reconstructed: String = correction
        .corrected_tokens
        .iter()
        .map(|token| format!("{}{}", token.text, token.whitespace))
        .collect();

    if reconstructed == original {
        return ValidationResult::Valid;
    }

    // Check if the difference is exactly one whitespace character (space, nbsp, etc.)
    let orig_no_spaces: String = original.chars().filter(|c| !c.is_whitespace()).collect();
    let recon_no_spaces: String = reconstructed.chars().filter(|c| !c.is_whitespace()).collect();

    // Use character count instead of byte length (important for UTF-8)
    if orig_no_spaces == recon_no_spaces && original.chars().count() == reconstructed.chars().count() + 1 {
        // Find where the whitespace is missing and what character it is
        let orig_chars: Vec<char> = original.chars().collect();
        let recon_chars: Vec<char> = reconstructed.chars().collect();

        let mut missing_space_pos = 0;
        let mut missing_space_char = ' ';
        for i in 0..orig_chars.len() {
            if i >= recon_chars.len() || orig_chars[i] != recon_chars[i] {
                missing_space_pos = i;
                missing_space_char = orig_chars[i];
                break;
            }
        }

        // Find which token this position falls into and add the whitespace
        let mut pos = 0;
        for token in correction.corrected_tokens.iter_mut() {
            let token_end = pos + token.text.len();

            if missing_space_pos >= pos && missing_space_pos <= token_end {
                // Whitespace should be added to this token's whitespace
                token.whitespace.push(missing_space_char);
                return ValidationResult::AutoFixed;
            }

            pos = token_end + token.whitespace.len();
        }

        // Couldn't find where to add the whitespace
        ValidationResult::Invalid {
            original: original.to_string(),
            reconstructed,
        }
    } else {
        ValidationResult::Invalid {
            original: original.to_string(),
            reconstructed,
        }
    }
}
