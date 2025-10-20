use language_utils::{Language, NlpAnalyzedSentence, PartOfSpeech};
use tysm::chat_completions::ChatClient;

/// Classification result for a sentence
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SentenceClassification {
    /// Sentence has no known issues
    Unknown,
    /// Sentence plausibly has an issue that should be reviewed
    #[allow(unused)]
    Suspicious { reasons: Vec<String> },
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
        Language::Korean => Box::new(KoreanClassifier),
        _ => Box::new(DefaultClassifier),
    }
}

/// Get the corrector for a given language
pub fn get_corrector(language: Language) -> Box<dyn WordCorrector> {
    match language {
        Language::French => Box::new(FrenchCorrector),
        Language::German => Box::new(GermanCorrector),
        Language::Spanish => Box::new(SpanishCorrector),
        Language::Korean => Box::new(KoreanCorrector),
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
        let mut reasons = Vec::new();

        // Check for Space tokens which indicate NLP parsing issues
        for token in &sentence.doc {
            if token.pos == PartOfSpeech::Space {
                reasons.push(format!("Contains Space token: '{}'", sentence.sentence));
            }

            let text_lower = token.text.to_lowercase();

            // Check for lemmas containing spaces (parsing error)
            if token.lemma.contains(' ') {
                reasons.push(format!(
                    "'{}' has lemma with space: '{}'",
                    token.text, token.lemma
                ));
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
                reasons.push(format!(
                    "Pronoun '{}' has incorrect lemma '{}'",
                    token.text, token.lemma
                ));
            }
        }

        if reasons.is_empty() {
            SentenceClassification::Unknown
        } else {
            SentenceClassification::Suspicious { reasons }
        }
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

/// Korean-specific classifier
struct KoreanClassifier;

impl SentenceClassifier for KoreanClassifier {
    fn classify(&self, sentence: &NlpAnalyzedSentence) -> SentenceClassification {
        let mut reasons = Vec::new();

        // Check for Space tokens which indicate NLP parsing issues
        for token in &sentence.doc {
            if token.pos == PartOfSpeech::Space {
                reasons.push(format!("Contains Space token: '{}'", sentence.sentence));
            }

            // Check for X (unknown) POS tags
            if token.pos == PartOfSpeech::X {
                reasons.push(format!("Token '{}' has unknown POS (X)", token.text));
            }

            // Check for verbs/auxiliaries with themselves as lemma (no morphological analysis)
            // Properly analyzed Korean should have lemmas with "+" morpheme boundaries
            if (token.pos == PartOfSpeech::Verb || token.pos == PartOfSpeech::Aux)
                && token.text == token.lemma
                && !token.lemma.contains('+')
            {
                reasons.push(format!(
                    "Verb/Aux '{}' has itself as lemma (no morphological analysis)",
                    token.text
                ));
            }
        }

        if reasons.is_empty() {
            SentenceClassification::Unknown
        } else {
            SentenceClassification::Suspicious { reasons }
        }
    }
}

/// Korean-specific corrector
struct KoreanCorrector;

impl WordCorrector for KoreanCorrector {
    fn correct(&self, _sentence: &mut NlpAnalyzedSentence) -> CorrectionResult {
        CorrectionResult {
            corrected: false,
            corrections: vec![],
        }
    }
}

/// French-specific classifier
struct FrenchClassifier;

impl SentenceClassifier for FrenchClassifier {
    fn classify(&self, sentence: &NlpAnalyzedSentence) -> SentenceClassification {
        let mut reasons = Vec::new();

        // Check for Space tokens which indicate NLP parsing issues
        for token in &sentence.doc {
            if token.pos == PartOfSpeech::Space {
                reasons.push(format!("Contains Space token: '{}'", sentence.sentence));
            }

            let text_lower = token.text.to_lowercase();

            // Check for hyphen being parsed incorrectly (indicates parsing error)
            if text_lower == "-"
                && (token.pos == PartOfSpeech::Pron || token.pos == PartOfSpeech::X)
            {
                reasons.push(format!("Hyphen parsed as {:?}", token.pos));
            }

            // Check for "lui" pronoun with lemma "luire"
            if text_lower == "lui" && token.lemma == "luire" && token.pos == PartOfSpeech::Pron {
                reasons.push("'lui' pronoun has lemma 'luire'".to_string());
            }

            // Check for "eux" with lemma "lui"
            if text_lower == "eux" && token.lemma == "lui" {
                reasons.push("'eux' has lemma 'lui'".to_string());
            }
        }

        if reasons.is_empty() {
            SentenceClassification::Unknown
        } else {
            SentenceClassification::Suspicious { reasons }
        }
    }
}

/// German-specific classifier
struct GermanClassifier;

impl SentenceClassifier for GermanClassifier {
    fn classify(&self, sentence: &NlpAnalyzedSentence) -> SentenceClassification {
        let mut reasons = Vec::new();

        // Check for Space tokens which indicate NLP parsing issues
        for token in &sentence.doc {
            if token.pos == PartOfSpeech::Space {
                reasons.push(format!("Contains Space token: '{}'", sentence.sentence));
            }

            let text_lower = token.text.to_lowercase();

            // Check for "will" which is often miscategorized
            // In German, "will" is a form of "wollen" (to want), but often gets confused
            if text_lower == "will" {
                reasons.push(
                    "Contains 'will' which is often miscategorized as it has multiple meanings ('werden', 'wollen', the name, etc)"
                        .to_string(),
                );
            }

            // Check for reflexive pronouns with lemma "sich"
            if (text_lower == "mich" || text_lower == "dich")
                && token.lemma == "sich"
                && token.pos == PartOfSpeech::Pron
            {
                reasons.push(format!("'{}' has lemma 'sich'", token.text));
            }

            // Check for "den" article with incorrect lemma "die"
            // Could be wrong (should be "der" for masc. acc.) or correct (dative plural)
            if text_lower == "den" && token.lemma == "die" && token.pos == PartOfSpeech::Det {
                reasons.push(
                    "'den' has lemma 'die' (could be wrong if accusative masculine)".to_string(),
                );
            }

            // Check for words that should be pronouns but are tagged as nouns
            // Common indefinite pronouns: alles, jemand, jemanden, jemandem, niemand, etc.
            if token.pos == PartOfSpeech::Noun {
                let indefinite_pronouns = [
                    "alles",
                    "etwas",
                    "nichts",
                    "jemand",
                    "jemanden",
                    "jemandem",
                    "jemands",
                    "niemand",
                    "niemanden",
                    "niemandem",
                    "niemands",
                ];
                if indefinite_pronouns.contains(&text_lower.as_str()) {
                    reasons.push(format!(
                        "'{}' tagged as NOUN but should likely be PRON",
                        token.text
                    ));
                }
            }

            // Check for capitalized lemma on non-nouns (nouns are capitalized in German)
            if token.pos != PartOfSpeech::Noun
                && token.pos != PartOfSpeech::Propn
                && token.pos != PartOfSpeech::Punct
            {
                if let Some(first_char) = token.lemma.chars().next() {
                    if first_char.is_uppercase() {
                        reasons.push(format!(
                            "Non-noun '{}' has capitalized lemma '{}'",
                            token.text, token.lemma
                        ));
                    }
                }
            }

            // Check for nouns with lowercase lemmas (nouns are capitalized in German)
            if token.pos == PartOfSpeech::Noun || token.pos == PartOfSpeech::Propn {
                if let Some(first_char) = token.lemma.chars().next() {
                    if first_char.is_lowercase() {
                        reasons.push(format!(
                            "Noun '{}' has lowercase lemma '{}'",
                            token.text, token.lemma
                        ));
                    }
                }
            }
        }

        if reasons.is_empty() {
            SentenceClassification::Unknown
        } else {
            SentenceClassification::Suspicious { reasons }
        }
    }
}

/// German-specific corrector
struct GermanCorrector;

impl WordCorrector for GermanCorrector {
    fn correct(&self, sentence: &mut NlpAnalyzedSentence) -> CorrectionResult {
        let mut corrected = false;
        let mut corrections = Vec::new();

        for token in &mut sentence.doc {
            let text_lower = token.text.to_lowercase();

            // Fix personal pronouns that aren't properly lemmatized
            if token.pos == PartOfSpeech::Pron {
                // 2nd person plural: euch → ihr
                if text_lower == "euch" && token.lemma != "ihr" {
                    corrections.push(format!(
                        "Fixed '{}' lemma from '{}' to 'ihr'",
                        token.text, token.lemma
                    ));
                    token.lemma = "ihr".to_string();
                    corrected = true;
                }

                // 2nd person singular: dir, dich → du
                if (text_lower == "dir" || text_lower == "dich") && token.lemma != "du" {
                    corrections.push(format!(
                        "Fixed '{}' lemma from '{}' to 'du'",
                        token.text, token.lemma
                    ));
                    token.lemma = "du".to_string();
                    corrected = true;
                }

                // 1st person singular: mir, mich → ich
                if (text_lower == "mir" || text_lower == "mich") && token.lemma != "ich" {
                    corrections.push(format!(
                        "Fixed '{}' lemma from '{}' to 'ich'",
                        token.text, token.lemma
                    ));
                    token.lemma = "ich".to_string();
                    corrected = true;
                }
            }

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
    #[serde(rename = "2. tokens")]
    pub corrected_tokens: Vec<SimplifiedToken>,
}

/// Dependency relation types (Universal Dependencies)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum DependencyRelation {
    #[serde(rename = "acl")]
    Acl,
    #[serde(rename = "acl:relcl")]
    AclRelcl,
    #[serde(rename = "advcl")]
    Advcl,
    #[serde(rename = "advcl:relcl")]
    AdvclRelcl,
    #[serde(rename = "advmod")]
    Advmod,
    #[serde(rename = "advmod:emph")]
    AdvmodEmph,
    #[serde(rename = "advmod:lmod")]
    AdvmodLmod,
    #[serde(rename = "amod")]
    Amod,
    #[serde(rename = "appos")]
    Appos,
    #[serde(rename = "aux")]
    Aux,
    #[serde(rename = "aux:pass")]
    AuxPass,
    #[serde(rename = "case")]
    Case,
    #[serde(rename = "cc")]
    Cc,
    #[serde(rename = "cc:preconj")]
    CcPreconj,
    #[serde(rename = "ccomp")]
    Ccomp,
    #[serde(rename = "clf")]
    Clf,
    #[serde(rename = "compound")]
    Compound,
    #[serde(rename = "compound:lvc")]
    CompoundLvc,
    #[serde(rename = "compound:prt")]
    CompoundPrt,
    #[serde(rename = "compound:redup")]
    CompoundRedup,
    #[serde(rename = "compound:svc")]
    CompoundSvc,
    #[serde(rename = "conj")]
    Conj,
    #[serde(rename = "cop")]
    Cop,
    #[serde(rename = "csubj")]
    Csubj,
    #[serde(rename = "csubj:outer")]
    CsubjOuter,
    #[serde(rename = "csubj:pass")]
    CsubjPass,
    #[serde(rename = "dep")]
    Dep,
    #[serde(rename = "det")]
    Det,
    #[serde(rename = "det:numgov")]
    DetNumgov,
    #[serde(rename = "det:nummod")]
    DetNummod,
    #[serde(rename = "det:poss")]
    DetPoss,
    #[serde(rename = "discourse")]
    Discourse,
    #[serde(rename = "dislocated")]
    Dislocated,
    #[serde(rename = "expl")]
    Expl,
    #[serde(rename = "expl:impers")]
    ExplImpers,
    #[serde(rename = "expl:pass")]
    ExplPass,
    #[serde(rename = "expl:pv")]
    ExplPv,
    #[serde(rename = "fixed")]
    Fixed,
    #[serde(rename = "flat")]
    Flat,
    #[serde(rename = "flat:foreign")]
    FlatForeign,
    #[serde(rename = "flat:name")]
    FlatName,
    #[serde(rename = "goeswith")]
    Goeswith,
    #[serde(rename = "iobj")]
    Iobj,
    #[serde(rename = "list")]
    List,
    #[serde(rename = "mark")]
    Mark,
    #[serde(rename = "nmod")]
    Nmod,
    #[serde(rename = "nmod:poss")]
    NmodPoss,
    #[serde(rename = "nmod:tmod")]
    NmodTmod,
    #[serde(rename = "nsubj")]
    Nsubj,
    #[serde(rename = "nsubj:outer")]
    NsubjOuter,
    #[serde(rename = "nsubj:pass")]
    NsubjPass,
    #[serde(rename = "nummod")]
    Nummod,
    #[serde(rename = "nummod:gov")]
    NummodGov,
    #[serde(rename = "obj")]
    Obj,
    #[serde(rename = "obl")]
    Obl,
    #[serde(rename = "obl:agent")]
    OblAgent,
    #[serde(rename = "obl:arg")]
    OblArg,
    #[serde(rename = "obl:lmod")]
    OblLmod,
    #[serde(rename = "obl:tmod")]
    OblTmod,
    #[serde(rename = "orphan")]
    Orphan,
    #[serde(rename = "parataxis")]
    Parataxis,
    #[serde(rename = "punct")]
    Punct,
    #[serde(rename = "reparandum")]
    Reparandum,
    #[serde(rename = "root")]
    Root,
    #[serde(rename = "vocative")]
    Vocative,
    #[serde(rename = "xcomp")]
    Xcomp,
}

/// A single token with its dependency information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct TokenDependency {
    pub index: usize,
    pub word: String,
    pub dependency: DependencyRelation,
    pub head: usize,
}

/// Response from the LLM for dependency parsing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct DependencyParseResponse {
    #[serde(rename = "1. thoughts")]
    pub thoughts: String,
    #[serde(rename = "2. dependencies")]
    pub dependencies: Vec<TokenDependency>,
}

/// Response from the LLM for multiword term validation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct MultiwordTermValidationResponse {
    #[serde(rename = "1. thoughts")]
    pub thoughts: String,
    #[serde(rename = "2. validated_multiword_terms")]
    pub validated_multiword_terms: Vec<String>,
}

/// Use GPT to clean/correct an NLP analyzed sentence
pub async fn clean_sentence_with_llm(
    language: Language,
    sentence: &NlpAnalyzedSentence,
    suspicious_reason: Option<String>,
    chat_client: &ChatClient,
) -> anyhow::Result<NlpCorrectionResponse> {
    let suspicion_context = if let Some(reason) = suspicious_reason {
        format!(
            "\n\nWe flagged this sentence as potentially suspicious because: {reason}. There may be additional issues that are not listed here."
        )
    } else {
        String::new()
    };

    let system_prompt = format!(
        r#"You are an expert in {language} NLP analysis. Your task is to review and potentially correct an automatically-generated NLP analysis of a {language} sentence.

The analysis consists of tokens, where each token has:
- text: the word as it appears
- whitespace: any whitespace after the word
- pos: part of speech (e.g., Noun, Verb, Adj, Adv, Det, Pron, Propn, etc.)
- lemma: the dictionary/base/standardized form of the word. 

Common issues to avoid:
- Lemmas that are incorrect (e.g., pronouns with wrong base forms)
- Part of speech tags that don't match the word
- Capitalized words getting confused for proper nouns just because they are capitalized
- Capitalization issues in lemmas (especially for German nouns)
- Lemmas that contain spaces (usually errors)
- Lemmas that do not convert the word to its dictionary form
- Contractions with themselves as lemmas (e.g., "l'" with lemma "l'" instead of "le")
The text of the word should always be the same as it appears in the sentence (including hyphens, apostrophes, etc.) The goal is that you can concatenate the tokens + whitespace in the order they appear in your output to get the original sentence.

Review the analysis carefully. If you find errors, correct them. If the analysis is already correct, return it unchanged. In either case, you will return all tokens in the sentence. You are the ultimate authority on the correct analysis of the sentence, and your response should stand alone.{suspicion_context} 

Think through your analysis, and finally provide the corrected token list. Remember, the provided analysis likely has errors. If it was likely to be good, we would not need you!"#
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

/// Use GPT to parse dependency relations for a sentence
pub async fn parse_dependencies_with_llm(
    language: Language,
    sentence: &str,
    tokens: &[SimplifiedToken],
    chat_client: &ChatClient,
) -> anyhow::Result<DependencyParseResponse> {
    let system_prompt = format!(
        r#"You are an expert in {language} syntax and dependency grammar (Universal Dependencies). Your task is to analyze the dependency structure of a {language} sentence.

For each token in the sentence, you need to identify:
1. Its dependency relation (e.g., nsubj, obj, det, etc.)
2. Its head (the index of the token it depends on, or 0 for the root)

Universal Dependencies relation types include:
acl, acl:relcl, advcl, advcl:relcl, advmod, advmod:emph, advmod:lmod, amod, appos, aux, aux:pass, case, cc, cc:preconj, ccomp, clf, compound, compound:lvc, compound:prt, compound:redup, compound:svc, conj, cop, csubj, csubj:outer, csubj:pass, dep, det, det:numgov, det:nummod, det:poss, discourse, dislocated, expl, expl:impers, expl:pass, expl:pv, fixed, flat, flat:foreign, flat:name, goeswith, iobj, list, mark, nmod, nmod:poss, nmod:tmod, nsubj, nsubj:outer, nsubj:pass, nummod, nummod:gov, obj, obl, obl:agent, obl:arg, obl:lmod, obl:tmod, orphan, parataxis, punct, reparandum, root, vocative, xcomp

Important rules:
- Exactly one token should have "root" as its dependency and 0 as its head
- All other tokens should have a head pointing to another token's index (1-based)
- The dependency structure should form a valid tree

Think through the sentence structure, then provide the dependency analysis for each token."#
    );

    // Build the indexed token list
    let mut indexed_tokens = String::new();
    for (i, token) in tokens.iter().enumerate() {
        indexed_tokens.push_str(&format!("{}. {}\n", i + 1, token.text));
    }

    let user_prompt = format!(
        "Sentence: \"{sentence}\"\n\nTokens:\n{indexed_tokens}\n\nProvide the dependency analysis for each token."
    );

    let response: DependencyParseResponse = chat_client
        .chat_with_system_prompt(system_prompt, user_prompt)
        .await?;

    Ok(response)
}

/// Use GPT to validate and normalize multiword terms in a sentence
#[allow(unused)] // not needed for now
pub async fn validate_multiword_terms_with_llm(
    language: Language,
    sentence: &str,
    high_confidence_terms: &[String],
    low_confidence_terms: &[String],
    chat_client: &ChatClient,
) -> anyhow::Result<MultiwordTermValidationResponse> {
    let system_prompt = format!(
        r#"You are an expert in {language} linguistics and multiword expressions. Your task is to validate and identify multiword terms (collocations, idioms, phrasal constructions, etc.) in a {language} sentence.

You will be given:
1. A sentence
2. Medium-confidence multiword term candidates (more likely correct)
3. Low-confidence multiword term candidates (may or may not be correct)

Your job is to:
1. Review all the candidate terms and determine which ones actually appear in the sentence
2. Identify any additional multiword terms that were missed
3. Return ALL multiword terms in their INFINITIVE/BASE FORM (not conjugated)

CRITICAL RULE ABOUT BASE FORMS:
- All multiword terms MUST be in their infinitive/dictionary form
- If a verb appears in the sentence conjugated, return it in infinitive form
- For example:
  * If the sentence has "he needs to", return "need to" (not "needs to")
  * If the sentence has "we're going", return "be going" (not "we're going" or "are going")
  * If the sentence has "ont besoin de" (French), return "avoir besoin de" (not "ont besoin de")
  * If the sentence has "hace falta" (Spanish), return "hacer falta" (not "hace falta")

What counts as a multiword term:
- Phrasal verbs (e.g., "look up", "give in")
- Idiomatic expressions (e.g., "break the ice", "piece of cake")
- Fixed collocations (e.g., "pay attention", "take care")
- Common verb + particle/preposition combinations
- Compound structures that function as a unit

What does NOT count:
- Random word sequences
- Temporary grammatical constructions
- Proper nouns (unless they're fixed expressions)

Think carefully about whether each candidate is a genuine multiword term, consider if there are additional multiword terms that were missed and should be added, then provide your final list of validated terms in their base forms."#
    );

    let mut user_prompt = format!("Sentence: \"{sentence}\"\n\n");

    if !high_confidence_terms.is_empty() {
        user_prompt.push_str("Medium-confidence multiword term candidates:\n");
        for term in high_confidence_terms {
            user_prompt.push_str(&format!("- {term}\n"));
        }
        user_prompt.push('\n');
    }

    if !low_confidence_terms.is_empty() {
        user_prompt.push_str("Low-confidence multiword term candidates:\n");
        for term in low_confidence_terms {
            user_prompt.push_str(&format!("- {term}\n"));
        }
        user_prompt.push('\n');
    }

    user_prompt.push_str("Please validate these candidates and identify any additional multiword terms, returning all in their base/infinitive forms.");

    let response: MultiwordTermValidationResponse = chat_client
        .chat_with_system_prompt(system_prompt, user_prompt)
        .await?;

    Ok(response)
}
