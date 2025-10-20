use language_utils::{Language, NlpAnalyzedSentence, PartOfSpeech};
use tysm::chat_completions::ChatClient;

/// Classification result for a sentence
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SentenceClassification {
    /// Sentence has no known issues
    Unknown,
    /// Sentence plausibly has an issue that should be reviewed
    #[allow(unused)]
    Suspicious { reason: String },
}

/// Result of word correction
#[derive(Debug, Clone)]
pub struct CorrectionResult {
    /// Whether any corrections were made
    pub corrected: bool,
    /// Description of what was corrected (if anything)
    #[allow(unused)]
    pub corrections: Vec<String>,
}

/// Trait for language-specific sentence classification rules
pub trait SentenceClassifier {
    /// Classify a sentence as Unknown or Suspicious
    fn classify(&self, sentence: &NlpAnalyzedSentence) -> SentenceClassification;
}

/// Trait for language-specific word correction rules
pub trait WordCorrector {
    /// Correct tokens in a sentence, returning whether any corrections were made
    fn correct(&self, sentence: &mut NlpAnalyzedSentence) -> CorrectionResult;
}

/// Get the classifier for a given language
pub fn get_classifier(language: Language) -> Box<dyn SentenceClassifier> {
    match language {
        Language::French => Box::new(FrenchClassifier),
        Language::German => Box::new(GermanClassifier),
        Language::Spanish => Box::new(SpanishClassifier),
        _ => Box::new(DefaultClassifier),
    }
}

/// Get the corrector for a given language
pub fn get_corrector(language: Language) -> Box<dyn WordCorrector> {
    match language {
        Language::French => Box::new(FrenchCorrector),
        Language::German => Box::new(GermanCorrector),
        Language::Spanish => Box::new(SpanishCorrector),
        _ => Box::new(DefaultCorrector),
    }
}

/// Default classifier that marks everything as Unknown
struct DefaultClassifier;

impl SentenceClassifier for DefaultClassifier {
    fn classify(&self, _sentence: &NlpAnalyzedSentence) -> SentenceClassification {
        SentenceClassification::Unknown
    }
}

/// Default corrector that makes no changes
struct DefaultCorrector;

impl WordCorrector for DefaultCorrector {
    fn correct(&self, _sentence: &mut NlpAnalyzedSentence) -> CorrectionResult {
        CorrectionResult {
            corrected: false,
            corrections: vec![],
        }
    }
}

/// Spanish-specific classifier
struct SpanishClassifier;

impl SentenceClassifier for SpanishClassifier {
    fn classify(&self, sentence: &NlpAnalyzedSentence) -> SentenceClassification {
        // Check for Space tokens which indicate NLP parsing issues
        for token in &sentence.doc {
            if token.pos == PartOfSpeech::Space {
                return SentenceClassification::Suspicious {
                    reason: format!("Contains Space token: '{}'", sentence.sentence),
                };
            }

            let text_lower = token.text.to_lowercase();

            // Check for lemmas containing spaces (parsing error)
            if token.lemma.contains(' ') {
                return SentenceClassification::Suspicious {
                    reason: format!("'{}' has lemma with space: '{}'", token.text, token.lemma),
                };
            }

            // Check for object/reflexive pronouns with subject pronoun lemmas
            if (text_lower == "me" && token.lemma == "yo")
                || (text_lower == "te" && token.lemma == "tú")
                || (text_lower == "lo" && token.lemma == "él")
                || (text_lower == "la" && token.lemma == "él")
                || (text_lower == "le" && token.lemma == "él")
                || (text_lower == "se" && token.lemma == "él")
                || (text_lower == "nos" && token.lemma == "yo")
                || (text_lower == "nosotros" && token.lemma == "yo")
                || (text_lower == "nosotras" && token.lemma == "yo")
            {
                return SentenceClassification::Suspicious {
                    reason: format!(
                        "Pronoun '{}' has incorrect lemma '{}'",
                        token.text, token.lemma
                    ),
                };
            }
        }

        SentenceClassification::Unknown
    }
}

/// Spanish-specific corrector
struct SpanishCorrector;

impl WordCorrector for SpanishCorrector {
    fn correct(&self, sentence: &mut NlpAnalyzedSentence) -> CorrectionResult {
        let mut corrected = false;
        let mut corrections = Vec::new();

        for token in &mut sentence.doc {
            let text_lower = token.text.to_lowercase();

            // Fix "ella" lemma - should always be "ella", not "él"
            if text_lower == "ella" && token.lemma == "él" {
                corrections.push(format!("Fixed '{}' lemma from 'él' to 'ella'", token.text));
                token.lemma = "ella".to_string();
                corrected = true;
            }
        }

        CorrectionResult {
            corrected,
            corrections,
        }
    }
}

/// French-specific classifier
struct FrenchClassifier;

impl SentenceClassifier for FrenchClassifier {
    fn classify(&self, sentence: &NlpAnalyzedSentence) -> SentenceClassification {
        // Check for Space tokens which indicate NLP parsing issues
        for token in &sentence.doc {
            if token.pos == PartOfSpeech::Space {
                return SentenceClassification::Suspicious {
                    reason: format!("Contains Space token: '{}'", sentence.sentence),
                };
            }

            let text_lower = token.text.to_lowercase();

            // Check for hyphen being parsed incorrectly (indicates parsing error)
            if text_lower == "-"
                && (token.pos == PartOfSpeech::Pron || token.pos == PartOfSpeech::X)
            {
                return SentenceClassification::Suspicious {
                    reason: format!("Hyphen parsed as {:?}", token.pos),
                };
            }

            // Check for "lui" pronoun with lemma "luire"
            if text_lower == "lui" && token.lemma == "luire" && token.pos == PartOfSpeech::Pron {
                return SentenceClassification::Suspicious {
                    reason: "'lui' pronoun has lemma 'luire'".to_string(),
                };
            }

            // Check for "eux" with lemma "lui"
            if text_lower == "eux" && token.lemma == "lui" {
                return SentenceClassification::Suspicious {
                    reason: "'eux' has lemma 'lui'".to_string(),
                };
            }
        }

        // For now, everything else is unknown unless we find a specific issue
        SentenceClassification::Unknown
    }
}

/// German-specific classifier
struct GermanClassifier;

impl SentenceClassifier for GermanClassifier {
    fn classify(&self, sentence: &NlpAnalyzedSentence) -> SentenceClassification {
        // Check for Space tokens which indicate NLP parsing issues
        for token in &sentence.doc {
            if token.pos == PartOfSpeech::Space {
                return SentenceClassification::Suspicious {
                    reason: format!("Contains Space token: '{}'", sentence.sentence),
                };
            }

            let text_lower = token.text.to_lowercase();

            // Check for reflexive pronouns with lemma "sich"
            if (text_lower == "mich" || text_lower == "dich")
                && token.lemma == "sich"
                && token.pos == PartOfSpeech::Pron
            {
                return SentenceClassification::Suspicious {
                    reason: format!("'{}' has lemma 'sich'", token.text),
                };
            }

            // Check for capitalized lemma on non-nouns (nouns are capitalized in German)
            if token.pos != PartOfSpeech::Noun
                && token.pos != PartOfSpeech::Propn
                && token.pos != PartOfSpeech::Punct
            {
                if let Some(first_char) = token.lemma.chars().next() {
                    if first_char.is_uppercase() {
                        return SentenceClassification::Suspicious {
                            reason: format!(
                                "Non-noun '{}' has capitalized lemma '{}'",
                                token.text, token.lemma
                            ),
                        };
                    }
                }
            }

            // Check for nouns with lowercase lemmas (nouns are capitalized in German)
            if token.pos == PartOfSpeech::Noun || token.pos == PartOfSpeech::Propn {
                if let Some(first_char) = token.lemma.chars().next() {
                    if first_char.is_lowercase() {
                        return SentenceClassification::Suspicious {
                            reason: format!(
                                "Noun '{}' has lowercase lemma '{}'",
                                token.text, token.lemma
                            ),
                        };
                    }
                }
            }
        }

        SentenceClassification::Unknown
    }
}

/// German-specific corrector
struct GermanCorrector;

impl WordCorrector for GermanCorrector {
    fn correct(&self, sentence: &mut NlpAnalyzedSentence) -> CorrectionResult {
        let mut corrected = false;
        let mut corrections = Vec::new();

        for token in &mut sentence.doc {
            // Fix punctuation with lemma "--"
            if token.pos == PartOfSpeech::Punct && token.lemma == "--" {
                corrections.push(format!(
                    "Fixed punctuation '{}' lemma from '--' to itself",
                    token.text
                ));
                token.lemma = token.text.clone();
                corrected = true;
            }
        }

        CorrectionResult {
            corrected,
            corrections,
        }
    }
}

/// French-specific corrector
struct FrenchCorrector;

impl WordCorrector for FrenchCorrector {
    fn correct(&self, sentence: &mut NlpAnalyzedSentence) -> CorrectionResult {
        let mut corrected = false;
        let mut corrections = Vec::new();

        for token in &mut sentence.doc {
            let text_lower = token.text.to_lowercase();

            // Fix "elle" lemma - should always be "elle"
            if text_lower == "elle" && token.lemma != "elle" {
                corrections.push(format!(
                    "Fixed '{}' lemma from '{}' to 'elle'",
                    token.text, token.lemma
                ));
                token.lemma = "elle".to_string();
                corrected = true;
            }

            // Fix contractions with themselves as lemma
            if text_lower == "j'" && token.lemma == "j'" {
                corrections.push(format!("Fixed '{}' lemma from 'j'' to 'je'", token.text));
                token.lemma = "je".to_string();
                corrected = true;
            }

            if text_lower == "l'" && token.lemma == "l'" {
                // Default to "le" if we can't determine gender
                corrections.push(format!("Fixed '{}' lemma from 'l'' to 'le'", token.text));
                token.lemma = "le".to_string();
                corrected = true;
            }

            // Fix "-ce" (in "qu'est-ce que" etc.) with itself as lemma
            if text_lower == "-ce" && token.lemma == "-ce" {
                corrections.push(format!("Fixed '{}' lemma from '-ce' to 'ce'", token.text));
                token.lemma = "ce".to_string();
                corrected = true;
            }

            // Fix "-là" (in "celles-là", "celui-là", etc.) with itself as lemma
            if text_lower == "-là" && token.lemma == "-là" {
                corrections.push(format!("Fixed '{}' lemma from '-là' to 'là'", token.text));
                token.lemma = "là".to_string();
                corrected = true;
            }
        }

        CorrectionResult {
            corrected,
            corrections,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use language_utils::PartOfSpeech;
    use std::collections::BTreeMap;

    #[test]
    fn test_french_elle_correction() {
        use language_utils::{DocToken, MultiwordTerms};

        let mut sentence = NlpAnalyzedSentence {
            sentence: "Elle parle".to_string(),
            multiword_terms: MultiwordTerms {
                high_confidence: vec![],
                low_confidence: vec![],
            },
            doc: vec![
                DocToken {
                    text: "Elle".to_string(),
                    whitespace: " ".to_string(),
                    pos: PartOfSpeech::Pron,
                    lemma: "lui".to_string(), // Wrong lemma
                    morph: BTreeMap::new(),
                },
                DocToken {
                    text: "parle".to_string(),
                    whitespace: "".to_string(),
                    pos: PartOfSpeech::Verb,
                    lemma: "parler".to_string(),
                    morph: BTreeMap::new(),
                },
            ],
        };

        let corrector = FrenchCorrector;
        let result = corrector.correct(&mut sentence);

        assert!(result.corrected);
        assert_eq!(result.corrections.len(), 1);
        assert_eq!(sentence.doc[0].lemma, "elle");
    }

    #[test]
    fn test_default_classifier() {
        use language_utils::MultiwordTerms;

        let sentence = NlpAnalyzedSentence {
            sentence: "Test".to_string(),
            multiword_terms: MultiwordTerms {
                high_confidence: vec![],
                low_confidence: vec![],
            },
            doc: vec![],
        };

        let classifier = DefaultClassifier;
        let result = classifier.classify(&sentence);

        assert_eq!(result, SentenceClassification::Unknown);
    }
}

/// Simplified token representation for LLM correction (without morphology)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct SimplifiedToken {
    pub text: String,
    pub whitespace: String,
    pub pos: PartOfSpeech,
    pub lemma: String,
}

/// Response from the LLM for NLP sentence correction
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct NlpCorrectionResponse {
    #[serde(rename = "1. thoughts")]
    pub thoughts: String,
    #[serde(rename = "2. corrected_tokens")]
    pub corrected_tokens: Vec<SimplifiedToken>,
}

/// Use GPT to clean/correct an NLP analyzed sentence
pub async fn clean_sentence_with_llm(
    language: Language,
    sentence: &NlpAnalyzedSentence,
    suspicious_reason: Option<String>,
    chat_client: &ChatClient,
) -> anyhow::Result<NlpCorrectionResponse> {
    let suspicion_context = if let Some(reason) = suspicious_reason {
        format!("\n\nWe flagged this sentence as potentially suspicious because: {reason}")
    } else {
        String::new()
    };

    let system_prompt = format!(
        r#"You are an expert in {language} NLP analysis. Your task is to review and potentially correct an automatically-generated NLP analysis of a {language} sentence.

The analysis consists of tokens, where each token has:
- text: the word as it appears
- whitespace: any whitespace after the word
- pos: part of speech (e.g., Noun, Verb, Adj, Adv, Det, Pron, Propn, etc.)
- lemma: the dictionary/base form of the word

Common issues to look for:
- Lemmas that are incorrect (e.g., pronouns with wrong base forms)
- Part of speech tags that don't match the word
- Capitalization issues in lemmas (especially for German nouns)
- Lemmas that contain spaces (usually errors)
- Contractions with themselves as lemmas (e.g., "l'" with lemma "l'" instead of "le")

Review the analysis carefully. If you find errors, correct them. If the analysis is already correct, return it unchanged.{suspicion_context}

Think through your analysis, then indicate whether you made corrections, and finally provide the corrected token list."#
    );

    // Convert DocTokens to SimplifiedTokens for the prompt
    let simplified_tokens: Vec<SimplifiedToken> = sentence
        .doc
        .iter()
        .map(|token| SimplifiedToken {
            text: token.text.clone(),
            whitespace: token.whitespace.clone(),
            pos: token.pos,
            lemma: token.lemma.clone(),
        })
        .collect();

    let user_prompt = format!(
        "Sentence: \"{}\"\n\nCurrent NLP analysis:\n{}",
        sentence.sentence,
        serde_json::to_string_pretty(&simplified_tokens)?
    );

    let response: NlpCorrectionResponse = chat_client
        .chat_with_system_prompt(system_prompt, user_prompt)
        .await?;

    Ok(response)
}
