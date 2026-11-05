use language_utils::{Language, Literal, Whitespace, predict_whitespace};

/// A single whitespace prediction error for diagnostics
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WhitespacePredictionError {
    /// The left token text
    pub left_text: String,
    /// The right token text (if any)
    pub right_text: Option<String>,
    /// What was predicted
    pub predicted: Whitespace,
    /// What the actual whitespace was
    pub actual: Whitespace,
    /// Full sentence context
    pub sentence: String,
}

/// Analyze whitespace prediction accuracy for a set of literals.
/// Returns a list of prediction errors with context.
pub fn analyze_whitespace_predictions(
    literals: &[Literal<String>],
    sentence_text: &str,
    language: Language,
) -> Vec<WhitespacePredictionError> {
    let mut errors = Vec::new();

    for (i, literal) in literals.iter().enumerate() {
        let next_word = literals.get(i + 1).map(|l| &l.word);
        let predicted = predict_whitespace(&literal.word, next_word, language);
        let actual = literal.whitespace;

        if predicted != actual {
            errors.push(WhitespacePredictionError {
                left_text: literal.word.text.clone(),
                right_text: next_word.map(|w| w.text.clone()),
                predicted,
                actual,
                sentence: sentence_text.to_string(),
            });
        }
    }

    errors
}

/// Summary of whitespace prediction errors grouped by pattern
#[derive(Debug, Clone, Default)]
pub struct WhitespacePredictionSummary {
    /// Errors grouped by (left_text, right_text, predicted, actual)
    /// Using BTreeMap for stable ordering in output
    pub by_pattern:
        std::collections::BTreeMap<(String, Option<String>, Whitespace, Whitespace), Vec<String>>,
    /// Total errors
    pub total_errors: usize,
    /// Total predictions made
    pub total_predictions: usize,
}

impl WhitespacePredictionSummary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add errors from analyzing a sentence
    pub fn add_sentence(
        &mut self,
        literals: &[Literal<String>],
        sentence_text: &str,
        language: Language,
    ) {
        self.total_predictions += literals.len();

        let errors = analyze_whitespace_predictions(literals, sentence_text, language);
        self.total_errors += errors.len();

        for error in errors {
            let key = (
                error.left_text,
                error.right_text,
                error.predicted,
                error.actual,
            );
            self.by_pattern.entry(key).or_default().push(error.sentence);
        }
    }

    /// Generate a diagnostic report as a string
    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        let accuracy = if self.total_predictions > 0 {
            100.0 * (1.0 - self.total_errors as f64 / self.total_predictions as f64)
        } else {
            100.0
        };

        report.push_str(&format!(
            "# Whitespace Prediction Diagnostics\n\n\
             Total predictions: {}\n\
             Total errors: {}\n\
             Accuracy: {:.2}%\n\n\
             ## Error Patterns (sorted by frequency)\n\n",
            self.total_predictions, self.total_errors, accuracy
        ));

        // Sort patterns by frequency (most common first)
        let mut patterns: Vec<_> = self.by_pattern.iter().collect();
        patterns.sort_by_key(|b| std::cmp::Reverse(b.1.len()));

        for ((left, right, predicted, actual), sentences) in patterns {
            let right_str = right.as_deref().unwrap_or("<END>");
            report.push_str(&format!(
                "### \"{}\" + \"{}\" ({} occurrences)\n",
                left,
                right_str,
                sentences.len()
            ));
            report.push_str(&format!(
                "- Predicted: {predicted:?}\n- Actual: {actual:?}\n"
            ));
            report.push_str("- Examples:\n");
            for sentence in sentences.iter().take(3) {
                report.push_str(&format!("  - {sentence}\n"));
            }
            report.push('\n');
        }

        report
    }
}
