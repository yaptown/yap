use anyhow::{Context, Result};
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use language_utils::{DocToken, Language, MultiwordTerms, NlpAnalyzedSentence, PartOfSpeech};
use lexide::matching::{DependencyMatcher, TreeNode};
use lexide::{Lexide, LexideConfig, Token};
use std::collections::{BTreeMap, HashSet};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// Convert language_utils::Language to lexide::Language
/// Returns None if the language is not supported by lexide
fn to_lexide_language(lang: Language) -> Option<lexide::Language> {
    match lang {
        Language::French => Some(lexide::Language::French),
        Language::English => Some(lexide::Language::English),
        Language::Spanish => Some(lexide::Language::Spanish),
        Language::Korean => Some(lexide::Language::Korean),
        Language::German => Some(lexide::Language::German),
        // Languages not yet supported by lexide
        Language::Chinese
        | Language::Japanese
        | Language::Russian
        | Language::Portuguese
        | Language::Italian => None,
    }
}

/// Convert lexide::PartOfSpeech to language_utils::PartOfSpeech
fn to_part_of_speech(pos: &lexide::pos::PartOfSpeech) -> PartOfSpeech {
    use lexide::pos::PartOfSpeech as LPos;
    match pos {
        LPos::Adj => PartOfSpeech::Adj,
        LPos::Adp => PartOfSpeech::Adp,
        LPos::Adv => PartOfSpeech::Adv,
        LPos::Aux => PartOfSpeech::Aux,
        LPos::Cconj => PartOfSpeech::Cconj,
        LPos::Det => PartOfSpeech::Det,
        LPos::Intj => PartOfSpeech::Intj,
        LPos::Noun => PartOfSpeech::Noun,
        LPos::Num => PartOfSpeech::Num,
        LPos::Part => PartOfSpeech::Part,
        LPos::Pron => PartOfSpeech::Pron,
        LPos::Propn => PartOfSpeech::Propn,
        LPos::Punct => PartOfSpeech::Punct,
        LPos::Sconj => PartOfSpeech::Sconj,
        LPos::Sym => PartOfSpeech::Sym,
        LPos::Verb => PartOfSpeech::Verb,
        LPos::X => PartOfSpeech::X,
        LPos::Space => PartOfSpeech::Space,
    }
}

/// Convert lexide Token to DocToken
fn to_doc_token(token: &Token) -> DocToken {
    let pos = to_part_of_speech(&token.pos);

    // Note: lexide doesn't currently provide morphological features,
    // so we use an empty map
    let morph = BTreeMap::new();

    DocToken {
        text: token.text.text.clone(),
        whitespace: token.whitespace.clone(),
        pos,
        lemma: token.lemma.lemma.clone(),
        morph,
    }
}

pub struct MultiwordTermDetector {
    lexide: Lexide,
    language: Language,
    // Pre-computed lemma mappings
    lemma_to_terms: BTreeMap<Vec<String>, Vec<String>>,
    // Pre-computed dependency patterns
    dependency_patterns: Vec<(String, TreeNode)>,
}

impl MultiwordTermDetector {
    pub async fn new(terms_file: &Path, language: Language) -> Result<Self> {
        // Check if language is supported
        let _lexide_lang = to_lexide_language(language).ok_or_else(|| {
            anyhow::anyhow!("Language {} is not yet supported by lexide", language)
        })?;

        // Initialize lexide
        println!("Initializing lexide NLP model...");
        let lexide = Lexide::from_pretrained(LexideConfig::default())
            .await
            .context("Failed to initialize lexide")?;

        // Load multiword terms
        println!("Loading multiword terms from {}...", terms_file.display());
        let multiword_terms = Self::load_terms(terms_file)?;
        println!("Loaded {} multiword terms", multiword_terms.len());

        // Pre-compute patterns and mappings
        println!("Creating patterns and lemma mappings...");
        let (lemma_to_terms, dependency_patterns) =
            Self::create_patterns_and_mappings(&lexide, &multiword_terms, language).await?;

        println!("Created {} dependency patterns", dependency_patterns.len());

        Ok(Self {
            lexide,
            language,
            lemma_to_terms,
            dependency_patterns,
        })
    }

    fn load_terms(terms_file: &Path) -> Result<Vec<String>> {
        let file =
            std::fs::File::open(terms_file).context("Failed to open multiword terms file")?;
        let reader = BufReader::new(file);

        let terms = reader
            .lines()
            .filter_map(|line| line.ok())
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        Ok(terms)
    }

    async fn create_patterns_and_mappings(
        lexide: &Lexide,
        multiword_terms: &[String],
        language: Language,
    ) -> Result<(BTreeMap<Vec<String>, Vec<String>>, Vec<(String, TreeNode)>)> {
        let mut lemma_to_terms = BTreeMap::new();
        let mut dependency_patterns = Vec::new();

        let lexide_language =
            to_lexide_language(language).ok_or_else(|| anyhow::anyhow!("Unsupported language"))?;

        let pb = ProgressBar::new(multiword_terms.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} terms ({per_sec}, {eta})")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        // Process all terms with controlled concurrency
        let analyses = futures::stream::iter(multiword_terms.iter())
            .map(|term| {
                let pb = pb.clone();
                async move {
                    let result = match lexide.analyze(term, lexide_language).await {
                        Ok(tokenization) => Some((term.clone(), tokenization)),
                        Err(e) => {
                            eprintln!("Warning: Failed to analyze term '{}': {}", term, e);
                            None
                        }
                    };
                    pb.inc(1);
                    result
                }
            })
            .buffer_unordered(10)
            .collect::<Vec<_>>()
            .await;

        for result in analyses {
            if let Some((term, tokenization)) = result {
                // Create lemma mapping
                let lemmas: Vec<String> = tokenization
                    .tokens
                    .iter()
                    .map(|t| t.lemma.lemma.clone())
                    .collect();

                lemma_to_terms
                    .entry(lemmas.clone())
                    .or_insert_with(Vec::new)
                    .push(term.clone());

                // Create dependency pattern if needed
                if Self::should_create_dependency_pattern(&tokenization, language) {
                    match TreeNode::try_from(tokenization) {
                        Ok(tree) => {
                            dependency_patterns.push((term, tree));
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: Failed to create dependency tree for term '{}': {}",
                                term, e
                            );
                        }
                    }
                }
            }
        }

        pb.finish_with_message("Patterns created");

        Ok((lemma_to_terms, dependency_patterns))
    }

    fn should_create_dependency_pattern(
        tokenization: &lexide::Tokenization,
        language: Language,
    ) -> bool {
        // For single-token terms, we don't need dependency patterns
        if tokenization.tokens.len() <= 1 {
            return false;
        }

        // Special handling for French negations like "ne...pas"
        if language == Language::French && tokenization.tokens.len() == 2 {
            let first = &tokenization.tokens[0].lemma.lemma;
            if first == "ne" {
                return true;
            }
        }

        // For longer phrases, create dependency patterns
        true
    }

    pub async fn find_multiword_terms_batch(
        &self,
        sentences: &[String],
    ) -> Result<Vec<Option<(lexide::Tokenization, MultiwordTerms)>>> {
        let lexide_language = to_lexide_language(self.language)
            .ok_or_else(|| anyhow::anyhow!("Unsupported language"))?;

        // Analyze all sentences
        let tokenizations: Vec<Option<lexide::Tokenization>> = {
            futures::stream::iter(sentences.iter())
                .map(|sentence| async move {
                    match self.lexide.analyze(sentence, lexide_language).await {
                        Ok(tokenization) => Some(tokenization),
                        Err(e) => {
                            eprintln!("Warning: Failed to analyze sentence '{}': {}", sentence, e);
                            None
                        }
                    }
                })
                .buffer_unordered(8)
                .collect()
                .await
        };

        // Process each tokenization
        let results = tokenizations
            .into_iter()
            .map(|tokenization_opt| {
                tokenization_opt.map(|tokenization| {
                    let multiword_terms = self.find_terms_in_tokenization(&tokenization);
                    (tokenization, multiword_terms)
                })
            })
            .collect();

        Ok(results)
    }

    fn find_terms_in_tokenization(&self, tokenization: &lexide::Tokenization) -> MultiwordTerms {
        let mut high_confidence = HashSet::new();
        let mut low_confidence = HashSet::new();

        // High confidence: Sequential lemma matching
        let sentence_lemmas: Vec<String> = tokenization
            .tokens
            .iter()
            .map(|t| t.lemma.lemma.clone())
            .collect();

        // Check for matching lemma sequences
        for (pattern_lemmas, terms) in &self.lemma_to_terms {
            if pattern_lemmas.len() == 1 {
                // Single word - check if it appears
                if sentence_lemmas.contains(&pattern_lemmas[0]) {
                    for term in terms {
                        high_confidence.insert(term.clone());
                    }
                }
            } else {
                // Multi-word - check for sequential matches
                for window in sentence_lemmas.windows(pattern_lemmas.len()) {
                    if window == pattern_lemmas.as_slice() {
                        for term in terms {
                            high_confidence.insert(term.clone());
                        }
                    }
                }
            }
        }

        // Low confidence: Dependency tree matching
        if !self.dependency_patterns.is_empty() {
            match TreeNode::try_from(tokenization.clone()) {
                Ok(sentence_tree) => {
                    for (term, pattern) in &self.dependency_patterns {
                        let matcher = DependencyMatcher::new(&[pattern.clone()]);
                        let matches = matcher.find_all(&sentence_tree);

                        if !matches.is_empty() && !high_confidence.contains(term) {
                            low_confidence.insert(term.clone());
                        }
                    }
                }
                Err(_e) => {
                    // Skip dependency matching for this sentence if tree creation fails
                    // (error not logged to avoid spam - these are usually legitimate parsing edge cases)
                }
            }
        }

        MultiwordTerms {
            high_confidence: high_confidence.into_iter().collect(),
            low_confidence: low_confidence.into_iter().collect(),
        }
    }
}

/// Process sentences from a JSONL file and add NLP analysis with multiword terms
pub async fn process_sentences(
    sentences_file: &Path,
    terms_file: &Path,
    output_file: &Path,
    language: Language,
) -> Result<()> {
    println!("\nInitializing multiword term detector for language: {language}...");
    let detector = MultiwordTermDetector::new(terms_file, language).await?;

    // Count total lines for progress
    println!("\nCounting sentences...");
    let file = std::fs::File::open(sentences_file)?;
    let reader = BufReader::new(file);
    let total_lines = reader
        .lines()
        .filter_map(|l| l.ok())
        .filter(|l| !l.trim().is_empty())
        .count();
    println!("Found {total_lines} sentences to process");

    // Process sentences in batches
    println!("\nProcessing sentences...");
    let batch_size = 500;

    let input_file = std::fs::File::open(sentences_file)?;
    let reader = BufReader::new(input_file);

    let output_file_handle = std::fs::File::create(output_file)?;
    let mut writer = std::io::BufWriter::new(output_file_handle);

    let pb = ProgressBar::new(total_lines as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} sentences ({per_sec}, {eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let mut batch_sentences = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let sentence: String = serde_json::from_str(&line)?;
        batch_sentences.push(sentence);

        // Process batch when full
        if batch_sentences.len() >= batch_size {
            process_batch(&detector, &batch_sentences, &mut writer).await?;
            pb.inc(batch_sentences.len() as u64);
            batch_sentences.clear();
        }
    }

    // Process remaining sentences
    if !batch_sentences.is_empty() {
        process_batch(&detector, &batch_sentences, &mut writer).await?;
        pb.inc(batch_sentences.len() as u64);
    }

    pb.finish_with_message("Processing complete");

    writer.flush()?;
    println!(
        "\nProcessing complete! Output written to {}",
        output_file.display()
    );

    Ok(())
}

async fn process_batch(
    detector: &MultiwordTermDetector,
    sentences: &[String],
    writer: &mut dyn Write,
) -> Result<()> {
    let results = detector.find_multiword_terms_batch(sentences).await?;

    for (sentence, result) in sentences.iter().zip(results.into_iter()) {
        if let Some((tokenization, multiword_terms)) = result {
            // Convert tokens to DocTokens
            let doc: Vec<DocToken> = tokenization.tokens.iter().map(to_doc_token).collect();

            let analyzed = NlpAnalyzedSentence {
                sentence: sentence.clone(),
                multiword_terms,
                doc,
            };

            // Write the enhanced data
            let json = serde_json::to_string(&analyzed)?;
            writeln!(writer, "{json}")?;
        }
        // If result is None, we skip this sentence (error was already logged)
    }

    Ok(())
}
