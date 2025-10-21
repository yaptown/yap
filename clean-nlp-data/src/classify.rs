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

            // Check for words that can be either DET or PRON depending on context
            // Rule: If it modifies a noun directly → DET. If it stands alone replacing a noun → PRON.
            let det_or_pron_words = [
                // Demonstratives
                "este", "esta", "estos", "estas", "ese", "esa", "esos", "esas", "aquel", "aquella",
                "aquellos", "aquellas", // Possessives (some forms can be both)
                "nuestro", "nuestra", "nuestros", "nuestras", "vuestro", "vuestra", "vuestros",
                "vuestras", // Indefinites/Quantifiers
                "uno", "una", "unos", "unas", "alguno", "alguna", "algunos", "algunas", "ninguno",
                "ninguna", "todo", "toda", "todos", "todas", "otro", "otra", "otros", "otras",
                "mucho", "mucha", "muchos", "muchas", "poco", "poca", "pocos", "pocas", "varios",
                "varias", "cierto", "cierta", "ciertos", "ciertas", "mismo", "misma", "mismos",
                "mismas", "tal", "tales", // Articles (can sometimes be pronouns)
                "el", "la", "los", "las",
            ];

            if det_or_pron_words.contains(&text_lower.as_str())
                && (token.pos == PartOfSpeech::Det || token.pos == PartOfSpeech::Pron)
            {
                reasons.push(format!(
                    "'{}' can be either DET or PRON depending on context (Rule: modifies noun → DET, stands alone → PRON)",
                    token.text
                ));
            }

            // Check common past-tense verbs are lemmatized to infinitive
            if token.pos == PartOfSpeech::Verb || token.pos == PartOfSpeech::Aux {
                let expected_lemmas: Vec<(&str, &str)> = vec![
                    ("era", "ser"),
                    ("eran", "ser"),
                    ("estaba", "estar"),
                    ("estaban", "estar"),
                    ("tenía", "tener"),
                    ("tenían", "tener"),
                    ("hacía", "hacer"),
                    ("hacían", "hacer"),
                    ("decía", "decir"),
                    ("decían", "decir"),
                    ("iba", "ir"),
                    ("iban", "ir"),
                    ("venía", "venir"),
                    ("venían", "venir"),
                    ("veía", "ver"),
                    ("veían", "ver"),
                    ("podía", "poder"),
                    ("podían", "poder"),
                    ("quería", "querer"),
                    ("querían", "querer"),
                    ("sabía", "saber"),
                    ("sabían", "saber"),
                ];

                for (past_form, expected_infinitive) in expected_lemmas {
                    if text_lower == past_form && token.lemma != expected_infinitive {
                        reasons.push(format!(
                            "Past-tense verb '{}' has lemma '{}', but the dictionary form is '{}', look at the context to determine which is rigbt",
                            token.text, token.lemma, expected_infinitive
                        ));
                    }
                }
            }

            // Check for haber conjugations which can be either AUX or VERB depending on context
            // Rule: AUX when forming compound tenses (e.g., "he comido")
            //       VERB in impersonal constructions (e.g., "hay que ir")
            let haber_forms = [
                // Present
                "he",
                "has",
                "ha",
                "hemos",
                "habéis",
                "han",
                "hay", // Imperfect
                "había",
                "habías",
                "habíamos",
                "habíais",
                "habían", // Preterite
                "hube",
                "hubiste",
                "hubo",
                "hubimos",
                "hubisteis",
                "hubieron", // Future
                "habré",
                "habrás",
                "habrá",
                "habremos",
                "habréis",
                "habrán", // Conditional
                "habría",
                "habrías",
                "habríamos",
                "habríais",
                "habrían",
            ];

            let deber_forms = [
                // Present
                "debo",
                "debes",
                "debe",
                "debemos",
                "debéis",
                "deben", // Imperfect
                "debía",
                "debías",
                "debíamos",
                "debíais",
                "debían", // Preterite
                "debí",
                "debiste",
                "debió",
                "debimos",
                "debisteis",
                "debieron", // Future
                "deberé",
                "deberás",
                "deberá",
                "deberemos",
                "deberéis",
                "deberán", // Conditional
                "debería",
                "deberías",
                "deberíamos",
                "deberíais",
                "deberían",
            ];

            let poder_forms = [
                // Present
                "puedo",
                "puedes",
                "puede",
                "podemos",
                "podéis",
                "pueden", // Imperfect
                "podía",
                "podías",
                "podíamos",
                "podíais",
                "podían", // Preterite
                "pude",
                "pudiste",
                "pudo",
                "pudimos",
                "pudisteis",
                "pudieron", // Future
                "podré",
                "podrás",
                "podrá",
                "podremos",
                "podréis",
                "podrán", // Conditional
                "podría",
                "podrías",
                "podríamos",
                "podríais",
                "podrían",
            ];

            let saber_forms = [
                // Present
                "sé",
                "sabes",
                "sabe",
                "sabemos",
                "sabéis",
                "saben", // Imperfect
                "sabía",
                "sabías",
                "sabíamos",
                "sabíais",
                "sabían", // Preterite
                "supe",
                "supiste",
                "supo",
                "supimos",
                "supisteis",
                "supieron", // Future
                "sabré",
                "sabrás",
                "sabrá",
                "sabremos",
                "sabréis",
                "sabrán", // Conditional
                "sabría",
                "sabrías",
                "sabríamos",
                "sabríais",
                "sabrían",
            ];

            if haber_forms.contains(&text_lower.as_str())
                && (token.pos == PartOfSpeech::Verb || token.pos == PartOfSpeech::Aux)
                && token.lemma == "haber"
            {
                reasons.push(format!(
                    "'{}' (haber) can be either AUX or VERB depending on context. Rule: AUX when forming compound tenses (e.g., 'he comido'), VERB in impersonal constructions (e.g., 'hay que ir', 'había mucha gente')",
                    token.text
                ));
            }

            if deber_forms.contains(&text_lower.as_str())
                && (token.pos == PartOfSpeech::Verb || token.pos == PartOfSpeech::Aux)
                && token.lemma == "deber"
            {
                reasons.push(format!(
                    "'{}' (deber) can be either AUX or VERB depending on context. Rule: AUX when expressing obligation with infinitive (e.g., 'debo ir'), VERB when expressing owing (e.g., 'me debe dinero')",
                    token.text
                ));
            }

            if poder_forms.contains(&text_lower.as_str())
                && (token.pos == PartOfSpeech::Verb || token.pos == PartOfSpeech::Aux)
                && token.lemma == "poder"
            {
                reasons.push(format!(
                    "'{}' (poder) can be either AUX or VERB depending on context. Rule: AUX when expressing ability/possibility with infinitive (e.g., 'puedo hacerlo'), VERB when used standalone or as a noun",
                    token.text
                ));
            }

            if saber_forms.contains(&text_lower.as_str())
                && (token.pos == PartOfSpeech::Verb || token.pos == PartOfSpeech::Aux)
                && token.lemma == "saber"
            {
                reasons.push(format!(
                    "'{}' (saber) can be either AUX or VERB depending on context. Rule: AUX when expressing ability with infinitive (e.g., 'sé nadar'), VERB when expressing knowledge of facts (e.g., 'sé la respuesta')",
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
                reasons.push("Contains Space token, which is usually not necessary due to the `whitespace` field".to_string());
            }
            if token.pos == PartOfSpeech::Propn {
                reasons.push(format!(
                    "Contains '{}' classified as a proper noun, but the legacy NLP pipeline often over-classifies things as proper nouns",
                    token.text
                ));
            }

            let text_lower = token.text.to_lowercase();

            // Check for hyphen being parsed incorrectly (indicates parsing error)
            if text_lower == "-"
                && (token.pos == PartOfSpeech::Pron || token.pos == PartOfSpeech::X)
            {
                reasons.push(format!("Hyphen parsed as {:?}", token.pos));
            }

            // Check for "lui" pronoun with lemma "luire"
            if text_lower == "lui" && token.lemma == "luire" {
                reasons
                    .push("'lui' has lemma 'luire' - is that right in this context?".to_string());
            }

            // Check for "eux" with lemma "lui"
            if text_lower == "eux" && token.lemma == "lui" {
                reasons.push("'eux' has lemma 'lui'".to_string());
            }

            // Check for words that can be either DET or PRON depending on context
            // Rule: If it modifies a noun directly → DET. If it stands alone replacing a noun → PRON.
            let det_or_pron_words = [
                // Quantifiers/Indefinites that can be both
                "tout",
                "toute",
                "tous",
                "toutes",
                "certain",
                "certains",
                "certaine",
                "certaines",
                "aucun",
                "aucune",
                "plusieurs",
                "autre",
                "autres",
                "même",
                "mêmes",
                "tel",
                "telle",
                "tels",
                "telles",
                "chacun",
                "chacune",
                // Articles (can sometimes be pronouns in certain constructions)
                "le",
                "la",
                "les",
                "l'",
            ];

            if det_or_pron_words.contains(&text_lower.as_str())
                && (token.pos == PartOfSpeech::Det || token.pos == PartOfSpeech::Pron)
            {
                reasons.push(format!(
                    "'{}' can be either DET or PRON depending on context (Rule: modifies noun → DET, stands alone → PRON)",
                    token.text
                ));
            }

            // Check common past-tense verbs are lemmatized to infinitive
            if token.pos == PartOfSpeech::Verb || token.pos == PartOfSpeech::Aux {
                let expected_lemmas: Vec<(&str, &str)> = vec![
                    ("était", "être"),
                    ("étaient", "être"),
                    ("avait", "avoir"),
                    ("avaient", "avoir"),
                    ("faisait", "faire"),
                    ("faisaient", "faire"),
                    ("disait", "dire"),
                    ("disaient", "dire"),
                    ("allait", "aller"),
                    ("allaient", "aller"),
                    ("venait", "venir"),
                    ("venaient", "venir"),
                    ("voyait", "voir"),
                    ("voyaient", "voir"),
                    ("pouvait", "pouvoir"),
                    ("pouvaient", "pouvoir"),
                    ("voulait", "vouloir"),
                    ("voulaient", "vouloir"),
                    ("savait", "savoir"),
                    ("savaient", "savoir"),
                ];

                for (past_form, expected_infinitive) in expected_lemmas {
                    if text_lower == past_form && token.lemma != expected_infinitive {
                        reasons.push(format!(
                            "Past-tense verb '{}' has lemma '{}', but the dictionary form is '{}', look at the context to determine which is rigbt",
                            token.text, token.lemma, expected_infinitive
                        ));
                    }
                }
            }

            // Check for avoir conjugations which can be either AUX or VERB depending on context
            // Rule: AUX when forming compound tenses with past participles (e.g., "j'ai mangé")
            //       VERB when expressing possession or other meanings (e.g., "j'ai un livre", "il a faim")
            let avoir_forms = [
                // Present
                "ai", "as", "a", "avons", "avez", "ont", // Imperfect
                "avais", "avait", "avions", "aviez", "avaient", // Future
                "aurai", "auras", "aura", "aurons", "aurez", "auront", // Conditional
                "aurais", "aurait", "aurions", "auriez", "auraient", // Passé simple
                "eus", "eut", "eûmes", "eûtes", "eurent",
            ];

            let devoir_forms = [
                // Present
                "dois",
                "doit",
                "devons",
                "devez",
                "doivent", // Imperfect
                "devais",
                "devait",
                "devions",
                "deviez",
                "devaient", // Future
                "devrai",
                "devras",
                "devra",
                "devrons",
                "devrez",
                "devront", // Conditional
                "devrais",
                "devrait",
                "devrions",
                "devriez",
                "devraient", // Passé simple
                "dus",
                "dut",
                "dûmes",
                "dûtes",
                "durent",
            ];

            let pouvoir_forms = [
                // Present
                "peux",
                "peut",
                "pouvons",
                "pouvez",
                "peuvent", // Imperfect
                "pouvais",
                "pouvait",
                "pouvions",
                "pouviez",
                "pouvaient", // Future
                "pourrai",
                "pourras",
                "pourra",
                "pourrons",
                "pourrez",
                "pourront", // Conditional
                "pourrais",
                "pourrait",
                "pourrions",
                "pourriez",
                "pourraient", // Passé simple
                "pus",
                "put",
                "pûmes",
                "pûtes",
                "purent",
            ];

            let savoir_forms = [
                // Present
                "sais",
                "sait",
                "savons",
                "savez",
                "savent", // Imperfect
                "savais",
                "savait",
                "savions",
                "saviez",
                "savaient", // Future
                "saurai",
                "sauras",
                "saura",
                "saurons",
                "saurez",
                "sauront", // Conditional
                "saurais",
                "saurait",
                "saurions",
                "sauriez",
                "sauraient", // Passé simple
                "sus",
                "sut",
                "sûmes",
                "sûtes",
                "surent",
            ];

            let falloir_forms = [
                // Present
                "faut",     // Imperfect
                "fallait",  // Future
                "faudra",   // Conditional
                "faudrait", // Passé simple
                "fallut",
            ];

            if avoir_forms.contains(&text_lower.as_str())
                && (token.pos == PartOfSpeech::Verb || token.pos == PartOfSpeech::Aux)
                && token.lemma == "avoir"
            {
                reasons.push(format!(
                    "'{}' (avoir) can be either AUX or VERB depending on context. Rule: AUX when forming compound tenses with past participles (e.g., 'j'ai mangé'), VERB when expressing possession or other meanings (e.g., 'j'ai un livre', 'il a faim', 'on n'a pas beaucoup de temps', etc.)",
                    token.text
                ));
            }

            if devoir_forms.contains(&text_lower.as_str())
                && (token.pos == PartOfSpeech::Verb || token.pos == PartOfSpeech::Aux)
                && token.lemma == "devoir"
            {
                reasons.push(format!(
                    "'{}' (devoir) can be either AUX or VERB depending on context. Rule: AUX when expressing obligation/necessity with infinitive (e.g., 'je dois partir'), VERB when used standalone or with other complements (e.g., 'il me doit de l'argent')",
                    token.text
                ));
            }

            if pouvoir_forms.contains(&text_lower.as_str())
                && (token.pos == PartOfSpeech::Verb || token.pos == PartOfSpeech::Aux)
                && token.lemma == "pouvoir"
            {
                reasons.push(format!(
                    "'{}' (pouvoir) can be either AUX or VERB depending on context. Rule: AUX when expressing ability/possibility with infinitive (e.g., 'je peux venir'), VERB when used standalone or as a noun",
                    token.text
                ));
            }

            if savoir_forms.contains(&text_lower.as_str())
                && (token.pos == PartOfSpeech::Verb || token.pos == PartOfSpeech::Aux)
                && token.lemma == "savoir"
            {
                reasons.push(format!(
                    "'{}' (savoir) can be either AUX or VERB depending on context. Rule: AUX when expressing ability/knowledge with infinitive (e.g., 'je sais nager'), VERB when expressing knowledge of facts (e.g., 'je sais la réponse')",
                    token.text
                ));
            }

            if falloir_forms.contains(&text_lower.as_str())
                && (token.pos == PartOfSpeech::Verb || token.pos == PartOfSpeech::Aux)
                && token.lemma == "falloir"
            {
                reasons.push(format!(
                    "'{}' (falloir) can be either AUX or VERB depending on context. Rule: AUX when expressing necessity with infinitive (e.g., 'il faut partir'), VERB when used with noun complements (e.g., 'il faut du temps')",
                    token.text
                ));
            }

            // Check for "du" which can be partitive article OR contraction of "de + le"
            // Partitive article: Je bois du café → lemma should be "du"
            // Contraction of "de + le": Je viens du marché → lemma should be "de"
            // If "du" appears after a verb that takes "de" as preposition → likely contraction → lemma "de"
            if text_lower == "du" {
                reasons.push(format!(
                    "'du' can be: (1) Partitive article meaning 'some/any' (e.g., 'Je bois du café') → lemma 'du', OR (2) Contraction of 'de + le' preposition (e.g., 'Je viens du marché') → lemma 'de'. Current lemma: '{}'. Rule: If 'du' appears after a verb that takes 'de' as a preposition → likely contraction → lemmatize to 'de'",
                    token.lemma
                ));
            }

            // Check for "des" which can be indefinite article, partitive, OR contraction of "de + les"
            // Indefinite article: J'ai vu des oiseaux → lemma should be "un"
            // Partitive article: Je mange des pommes → lemma should be "des"
            // Contraction of "de + les": Je parle des enfants → lemma should be "de"
            if text_lower == "des" {
                reasons.push(format!(
                    "'des' can be: (1) Indefinite article/plural (e.g., 'J'ai vu des oiseaux') → lemma 'des', (2) Partitive article (e.g., 'Je mange des pommes') → lemma 'des', OR (3) Contraction of 'de + les' (e.g., 'Je parle des enfants') → lemma 'de'. Current lemma: '{}'. Rule: If 'des' appears before a noun without a preceding preposition → likely indefinite article → lemmatize to 'un'",
                    token.lemma
                ));
            }

            if text_lower == "bois" {
                reasons.push(format!(
                    "'bois' can be: (1) Verb 'boire' (e.g., 'Je bois du café') → lemma 'boire', OR (2) Noun 'bois' (e.g., 'Le bois est dur') → lemma 'bois'. Current lemma: '{}'.",
                    token.lemma
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

/// German-specific classifier
struct GermanClassifier;

impl SentenceClassifier for GermanClassifier {
    fn classify(&self, sentence: &NlpAnalyzedSentence) -> SentenceClassification {
        let mut reasons = Vec::new();

        // Check for Space tokens which indicate NLP parsing issues
        for (idx, token) in sentence.doc.iter().enumerate() {
            let is_first_word = idx == 0;
            let _is_last_word = idx == sentence.doc.len() - 1;

            if token.pos == PartOfSpeech::Space {
                reasons.push("Contains SPACE token, but the `whitespace` field should be used instead (SPACE tokens are not usually necessary)".to_string());
            }
            if token.pos == PartOfSpeech::Propn {
                reasons.push(format!(
                    "Contains '{}' classified as a proper noun, but the legacy NLP pipeline often over-classifies things as proper nouns",
                    token.text
                ));
            }

            if is_first_word && token.text == "Sie" {
                reasons.push(
                    "Sie could either have lemma 'Sie' (formal you) or 'sie' (she/they)"
                        .to_string(),
                );
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

            // Check for words that can be either DET or PRON depending on context
            // Rule: If it modifies a noun directly → DET. If it stands alone replacing a noun → PRON.
            let det_or_pron_words = [
                // Possessives
                "mein",
                "meine",
                "meinen",
                "meinem",
                "meiner",
                "meines",
                "dein",
                "deine",
                "deinen",
                "deinem",
                "deiner",
                "deines",
                "deins",
                "sein",
                "seine",
                "seinen",
                "seinem",
                "seiner",
                "seines",
                "seins",
                "ihr",
                "ihre",
                "ihren",
                "ihrem",
                "ihrer",
                "ihres",
                "unser",
                "unsere",
                "unseren",
                "unserem",
                "unserer",
                "unseres",
                "unsres",
                "euer",
                "eure",
                "euren",
                "eurem",
                "eurer",
                "eures",
                "eurer",
                // Demonstratives
                "dieser",
                "diese",
                "dieses",
                "diesen",
                "diesem",
                "dieser",
                "jener",
                "jene",
                "jenes",
                "jenen",
                "jenem",
                "jener",
                "derselbe",
                "dieselbe",
                "dasselbe",
                "denselben",
                "demselben",
                "derselben",
                // Indefinites
                "einer",
                "eine",
                "eines",
                "einen",
                "einem",
                "keiner",
                "keine",
                "keines",
                "keinen",
                "keinem",
                // Quantifiers
                "alle",
                "aller",
                "allen",
                "allem",
                "beide",
                "beider",
                "beiden",
                "beidem",
                "einige",
                "einiger",
                "einigen",
                "einigem",
                "mehrere",
                "mehrerer",
                "mehreren",
                "mehrerem",
                "viele",
                "vieler",
                "vielen",
                "vielem",
                "wenige",
                "weniger",
                "wenigen",
                "wenigem",
                // Definite articles that can be relative/demonstrative pronouns
                "der",
                "die",
                "das",
                "den",
                "dem",
                "des",
            ];

            if det_or_pron_words.contains(&text_lower.as_str())
                && (token.pos == PartOfSpeech::Det || token.pos == PartOfSpeech::Pron)
            {
                reasons.push(format!(
                    "'{}' can be either DET or PRON depending on context (Rule: modifies noun → DET, stands alone → PRON)",
                    token.text
                ));
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

            // Check common past-tense verbs are lemmatized to infinitive
            if token.pos == PartOfSpeech::Verb || token.pos == PartOfSpeech::Aux {
                let expected_lemmas: Vec<(&str, &str)> = vec![
                    ("war", "sein"),
                    ("waren", "sein"),
                    ("hatte", "haben"),
                    ("hatten", "haben"),
                    ("machte", "machen"),
                    ("machten", "machen"),
                    ("sagte", "sagen"),
                    ("sagten", "sagen"),
                    ("ging", "gehen"),
                    ("gingen", "gehen"),
                    ("kam", "kommen"),
                    ("kamen", "kommen"),
                    ("sah", "sehen"),
                    ("sahen", "sehen"),
                    ("konnte", "können"),
                    ("konnten", "können"),
                    ("wollte", "wollen"),
                    ("wollten", "wollen"),
                    ("wusste", "wissen"),
                    ("wussten", "wissen"),
                ];

                for (past_form, expected_infinitive) in expected_lemmas {
                    if text_lower == past_form && token.lemma != expected_infinitive {
                        reasons.push(format!(
                            "Past-tense verb '{}' has lemma '{}', but the dictionary form is '{}', look at the context to determine which is rigbt",
                            token.text, token.lemma, expected_infinitive
                        ));
                    }
                }
            }

            // Check for haben conjugations which can be either AUX or VERB depending on context
            // Rule: AUX when forming compound tenses with past participles (e.g., "ich habe gegessen")
            //       VERB when expressing possession or other meanings (e.g., "ich habe Zeit")
            let haben_forms = [
                // Present
                "habe",
                "hast",
                "hat",
                "haben",
                "habt", // Past
                "hatte",
                "hattest",
                "hatten",
                "hattet", // Future
                "werde haben",
                "wirst haben",
                "wird haben",
                "werden haben",
                "werdet haben",
            ];

            let müssen_forms = [
                // Present
                "muss", "musst", "müssen", "müsst", // Past
                "musste", "musstest", "mussten", "musstet",
            ];

            let können_forms = [
                // Present
                "kann", "kannst", "können", "könnt", // Past
                "konnte", "konntest", "konnten", "konntet",
            ];

            let wissen_forms = [
                // Present
                "weiß", "weißt", "wissen", "wisst", // Past
                "wusste", "wusstest", "wussten", "wusstet",
            ];

            let sollen_forms = [
                // Present
                "soll", "sollst", "sollen", "sollt", // Past
                "sollte", "solltest", "sollten", "solltet",
            ];

            let wollen_forms = [
                // Present
                "will", "willst", "wollen", "wollt", // Past
                "wollte", "wolltest", "wollten", "wolltet",
            ];

            let dürfen_forms = [
                // Present
                "darf", "darfst", "dürfen", "dürft", // Past
                "durfte", "durftest", "durften", "durftet",
            ];

            let mögen_forms = [
                // Present
                "mag",
                "magst",
                "mögen",
                "mögt", // Past (including möchte)
                "mochte",
                "mochtest",
                "mochten",
                "mochtet",
                "möchte",
                "möchtest",
                "möchten",
                "möchtet",
            ];

            if haben_forms.contains(&text_lower.as_str())
                && (token.pos == PartOfSpeech::Verb || token.pos == PartOfSpeech::Aux)
                && token.lemma == "haben"
            {
                reasons.push(format!(
                    "'{}' (haben) can be either AUX or VERB depending on context. Rule: AUX when forming compound tenses with past participles (e.g., 'ich habe gegessen'), VERB when expressing possession or other meanings (e.g., 'ich habe Zeit', 'er hat Hunger')",
                    token.text
                ));
            }

            if müssen_forms.contains(&text_lower.as_str())
                && (token.pos == PartOfSpeech::Verb || token.pos == PartOfSpeech::Aux)
                && token.lemma == "müssen"
            {
                reasons.push(format!(
                    "'{}' (müssen) can be either AUX or VERB depending on context. Rule: AUX when expressing necessity/obligation with infinitive (e.g., 'ich muss gehen'), VERB when used standalone",
                    token.text
                ));
            }

            if können_forms.contains(&text_lower.as_str())
                && (token.pos == PartOfSpeech::Verb || token.pos == PartOfSpeech::Aux)
                && token.lemma == "können"
            {
                reasons.push(format!(
                    "'{}' (können) can be either AUX or VERB depending on context. Rule: AUX when expressing ability/possibility with infinitive (e.g., 'ich kann schwimmen'), VERB when used standalone",
                    token.text
                ));
            }

            if wissen_forms.contains(&text_lower.as_str())
                && (token.pos == PartOfSpeech::Verb || token.pos == PartOfSpeech::Aux)
                && token.lemma == "wissen"
            {
                reasons.push(format!(
                    "'{}' (wissen) can be either AUX or VERB depending on context. Rule: Usually VERB expressing knowledge (e.g., 'ich weiß es'), but can be AUX in some constructions",
                    token.text
                ));
            }

            if sollen_forms.contains(&text_lower.as_str())
                && (token.pos == PartOfSpeech::Verb || token.pos == PartOfSpeech::Aux)
                && token.lemma == "sollen"
            {
                reasons.push(format!(
                    "'{}' (sollen) can be either AUX or VERB depending on context. Rule: AUX when expressing obligation/expectation with infinitive (e.g., 'du sollst gehen'), VERB when used standalone",
                    token.text
                ));
            }

            if wollen_forms.contains(&text_lower.as_str())
                && (token.pos == PartOfSpeech::Verb || token.pos == PartOfSpeech::Aux)
                && token.lemma == "wollen"
            {
                reasons.push(format!(
                    "'{}' (wollen) can be either AUX or VERB depending on context. Rule: AUX when expressing desire/intention with infinitive (e.g., 'ich will gehen'), VERB when used standalone",
                    token.text
                ));
            }

            if dürfen_forms.contains(&text_lower.as_str())
                && (token.pos == PartOfSpeech::Verb || token.pos == PartOfSpeech::Aux)
                && token.lemma == "dürfen"
            {
                reasons.push(format!(
                    "'{}' (dürfen) can be either AUX or VERB depending on context. Rule: AUX when expressing permission/allowance with infinitive (e.g., 'du darfst gehen'), VERB when used standalone",
                    token.text
                ));
            }

            if mögen_forms.contains(&text_lower.as_str())
                && (token.pos == PartOfSpeech::Verb || token.pos == PartOfSpeech::Aux)
                && token.lemma == "mögen"
            {
                reasons.push(format!(
                    "'{}' (mögen) can be either AUX or VERB depending on context. Rule: AUX when expressing desire with infinitive (e.g., 'ich möchte gehen'), VERB when expressing liking (e.g., 'ich mag Pizza')",
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

        // Use fold to build new token list, splitting hyphens as we go
        let original_tokens = std::mem::take(&mut sentence.doc);
        sentence.doc = original_tokens
            .into_iter()
            .fold(Vec::new(), |mut acc, mut token| {
                let text_lower = token.text.to_lowercase();

                // Fix "ne" and "n'" - should always be Adv, not Part
                if (text_lower == "ne" || text_lower == "n'") && token.pos == PartOfSpeech::Part {
                    corrections.push(format!("Fixed '{}' POS from Part to Adv", token.text));
                    token.pos = PartOfSpeech::Adv;
                    corrected = true;
                }

                // Fix "ça" lemma - should always be "cela"
                if text_lower == "ça" && token.lemma != "cela" {
                    corrections.push(format!(
                        "Fixed '{}' lemma from '{}' to 'cela'",
                        token.text, token.lemma
                    ));
                    token.lemma = "cela".to_string();
                    corrected = true;
                }

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

                // Fix "a" in "il y a" construction - should always be Verb
                if text_lower == "a" && token.pos != PartOfSpeech::Verb && acc.len() >= 2 {
                    // Check if preceded by "y" and "il"
                    let prev_token = &acc[acc.len() - 1];
                    let prev_prev_token = &acc[acc.len() - 2];

                    if prev_token.text.to_lowercase() == "y"
                        && prev_prev_token.text.to_lowercase() == "il"
                    {
                        corrections.push(format!(
                            "Fixed '{}' in 'il y a' construction from {:?} to Verb",
                            token.text, token.pos
                        ));
                        token.pos = PartOfSpeech::Verb;
                        // Also ensure lemma is "avoir"
                        if token.lemma != "avoir" {
                            corrections.push(format!(
                                "Fixed '{}' lemma in 'il y a' from '{}' to 'avoir'",
                                token.text, token.lemma
                            ));
                            token.lemma = "avoir".to_string();
                        }
                        corrected = true;
                    }
                }

                // Normalize possessive adjectives to masculine singular form
                if token.pos == PartOfSpeech::Det {
                    let possessive_normalizations = [
                        ("ta", "ton"),
                        ("ma", "mon"),
                        ("sa", "son"),
                        ("tes", "ton"),
                        ("mes", "mon"),
                        ("ses", "son"),
                        ("nos", "notre"),
                        ("vos", "votre"),
                        ("leurs", "leur"),
                    ];

                    for (form, normalized) in possessive_normalizations {
                        if text_lower == form && token.lemma != normalized {
                            corrections.push(format!(
                                "Normalized possessive '{}' lemma from '{}' to '{}'",
                                token.text, token.lemma, normalized
                            ));
                            token.lemma = normalized.to_string();
                            corrected = true;
                            break;
                        }
                    }

                    // Normalize definite articles to masculine singular form
                    if (text_lower == "la" || text_lower == "les") && token.lemma != "le" {
                        corrections.push(format!(
                            "Normalized article '{}' lemma from '{}' to 'le'",
                            token.text, token.lemma
                        ));
                        token.lemma = "le".to_string();
                        corrected = true;
                    }
                }

                if token.text.starts_with('-')
                    && token.text.len() > 1
                    && !acc.is_empty()
                    && acc.last().unwrap().whitespace.is_empty()
                {
                    // Remove hyphen from beginning of token
                    let original_text = token.text.clone();
                    token.text = token.text[1..].to_string();

                    corrections.push(format!(
                        "Split hyphen from beginning of '{original_text}' into separate token"
                    ));

                    // Create separate hyphen token
                    let hyphen_token = language_utils::DocToken {
                        text: "-".to_string(),
                        whitespace: String::new(), // No whitespace after hyphen
                        pos: PartOfSpeech::Punct,
                        lemma: "-".to_string(),
                        morph: std::collections::BTreeMap::new(),
                    };

                    acc.push(hyphen_token);
                    acc.push(token);
                    corrected = true;
                }
                // Split words ending in hyphen with no whitespace after
                else if token.text.ends_with('-')
                    && token.whitespace.is_empty()
                    && token.text.len() > 1
                {
                    // Remove hyphen from original token
                    let original_text = token.text.clone();
                    let original_whitespace = token.whitespace.clone();
                    token.text.pop();
                    token.whitespace = String::new(); // No whitespace after word part

                    corrections.push(format!(
                        "Split hyphen from end of '{original_text}' into separate token"
                    ));

                    // Create separate hyphen token with the original whitespace
                    let hyphen_token = language_utils::DocToken {
                        text: "-".to_string(),
                        whitespace: original_whitespace,
                        pos: PartOfSpeech::Punct,
                        lemma: "-".to_string(),
                        morph: std::collections::BTreeMap::new(),
                    };

                    acc.push(token);
                    acc.push(hyphen_token);
                    corrected = true;
                } else {
                    acc.push(token);
                }

                acc
            });

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
    #[serde(rename = "1. text")]
    pub text: String,
    #[serde(rename = "2. whitespace")]
    pub whitespace: String,
    #[serde(rename = "3. pos")]
    pub pos: PartOfSpeech,
    #[serde(rename = "4. lemma")]
    pub lemma: String,
}

/// Simplified token representation for LLM correction (without morphology)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct SimplifiedTokenPrime {
    pub text: String,
    pub whitespace: String,
    pub pos: PartOfSpeech,
    pub lemma: String,
}

impl From<SimplifiedToken> for SimplifiedTokenPrime {
    fn from(token: SimplifiedToken) -> Self {
        Self {
            text: token.text,
            whitespace: token.whitespace,
            pos: token.pos,
            lemma: token.lemma,
        }
    }
}

/// Response from the LLM for NLP sentence correction
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct NlpCorrectionResponse {
    #[serde(rename = "tokens")]
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
    suspicious_reasons: Vec<String>,
    chat_client: &ChatClient,
) -> anyhow::Result<Vec<SimplifiedTokenPrime>> {
    let suspicion_context = if !suspicious_reasons.is_empty() {
        let reason = suspicious_reasons.into_iter().enumerate().fold(
            String::new(),
            |mut acc, (idx, reason)| {
                use std::fmt::Write;
                if acc.is_empty() {
                    format!("{idx}. {reason}")
                } else {
                    write!(acc, "\n{idx}. {reason}").unwrap();
                    acc
                }
            },
        );
        format!(
            "\n\nPlease keep the following in mind: {reason}\nPlease review these points one by one and correct them (only if necessary). There may be additional issues that are not listed here."
        )
    } else {
        String::new()
    };

    let system_prompt = format!(
        r#"You are an expert in {language} NLP analysis. Your task is to review and potentially correct an automatically-generated NLP analysis of a {language} sentence.

The analysis consists of tokens, where each token has:

{{
    "1. text": string, // the word as it appears (including contractions, so "l'" should be "l", not "le", and "don't" should be "do" and "n't").
    "2. whitespace": string, // any whitespace after the word. if you need a non-breaking space (used in some languages), use "[nbspace]" in the whitespace field.
    "3. pos": string, // part of speech. (e.g., Noun, Verb, Aux, Adj, Adv, Det, Pron, Propn, etc.)
    "4. lemma": string, // the dictionary/base/standardized form of the word
}}

Common issues to avoid:
- Lemmas that are incorrect (e.g., pronouns with wrong base forms)
- Part of speech tags that don't match the word
- Capitalized words getting confused for proper nouns just because they are capitalized
- Capitalization issues in lemmas (lemmas should generally be lowercase, except when the case is meaningful as in proper nouns and German nouns)
- Lemmas that contain spaces (usually errors)
- Lemmas that do not convert the word to its dictionary form
- Lemmas that do not convert the word to its masculine singular form (if applicable)
- Contractions with themselves as lemmas (e.g., "l'" with lemma "l'" instead of "le")
- Unncessary combinations. e.g. "qu'est-ce" can be four tokens, "qu''/"que", "est"/"être", "-"/"-", and "ce"/"ce", and doesn't need to be combined into a single token. Similar for "c'est-ce" (should be "c''/"ce", "est"/"être", "-"/"-", and "ce"/"ce"), est-ce que (should be "est"/"être", "-"/"-", "ce"/"ce". "que"/"que"), etc.
- Unjoined multiword proper nouns (e.g. "Croissant Fertile" should be one token, "Croissant Fertile", not two tokens, "Croissant" and "Fertile")

The text of the word should always be the same as it appears in the sentence (including hyphens, apostrophes, etc.) The goal is that you can concatenate the tokens + whitespace in the order they appear in your output to get the original sentence.

Hyphenated words should usually be split into three separate tokens. For example, "can-do" should be split into "can", "-", "do". "toi-même" should be split into "toi", "-", "même".

Review the analysis carefully. If you find errors, correct them. If the analysis is already correct, return it unchanged. In either case, you will return all tokens in the sentence. You are the ultimate authority on the correct analysis of the sentence, and your response should stand alone.{suspicion_context} 

Think through your analysis, and finally provide the corrected token list. Remember, the provided analysis likely has errors. If it was likely to be good, we would not need you!"#
    );

    // Convert DocTokens to SimplifiedTokens for the prompt
    let simplified_tokens: Vec<SimplifiedTokenPrime> = sentence
        .doc
        .iter()
        .map(|token| SimplifiedTokenPrime {
            text: token.text.clone(),
            whitespace: if token.whitespace.clone() == "\u{00A0}" {
                "[nbspace]".to_string()
            } else {
                token.whitespace.clone()
            },
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

    let corrected_tokens: Vec<SimplifiedTokenPrime> = response
        .corrected_tokens
        .into_iter()
        .map(|token| SimplifiedTokenPrime {
            whitespace: if token.whitespace == "[nbspace]" {
                "\u{00A0}".to_string()
            } else {
                token.whitespace
            },
            pos: if token.text == "-" {
                PartOfSpeech::Punct
            } else {
                token.pos
            },
            text: token.text,
            lemma: token.lemma,
        })
        .collect();

    Ok(corrected_tokens)
}

/// Use GPT to parse dependency relations for a sentence
pub async fn parse_dependencies_with_llm(
    language: Language,
    sentence: &str,
    tokens: &[SimplifiedTokenPrime],
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
