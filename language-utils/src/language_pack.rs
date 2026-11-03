use crate::indexmap::IndexMap;
use crate::minimal_pairs::{MinimalPairs, build_minimal_pairs_index};
use crate::{
    Atom, Audio, BookMetadata, ConsolidatedLanguageData, Course, DictionaryEntry, Frequency, Gram,
    GramDefinition, Heteronym, HomophonePractice, HomophoneWordPair, Lexeme, Literal, MorphemeInfo,
    MorphemeSegment, MovieMetadata, MultiwordTermMatch, PartOfSpeech, PatternPosition,
    PronunciationClip, PronunciationData, PronunciationGuide, ProperNounDefinition, SentenceGram,
    SentenceGrams, SentenceSource, SpurGram, TaggedGram, VoiceActor, WordType, grm,
};
use lasso::Spur;
use rustc_hash::FxHashMap;
use std::collections::BTreeMap;
use std::hash::Hash;
use std::sync::Arc;

/// Sentence identities are independent of shared vocabulary strings.
#[derive(Debug)]
pub enum SentenceText {}
pub type SentenceSpur = Spur<SentenceText>;

/// Native translation identities are independent of sentence and vocabulary keys.
/// This uses a reader because the pinned lasso does not archive resolvers.
#[derive(Debug)]
pub enum TranslationText {}
pub type TranslationSpur = Spur<TranslationText>;

/// Build-time index, easiest first (descending ease, then ascending key),
/// independent of the pack's frequency ordering.
#[derive(Clone, Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct EaseOrder<K: Eq + Hash> {
    entries: Vec<(K, f32)>,
    ranks: FxHashMap<K, u32>,
}

impl<K: Copy + Eq + Hash + Ord> EaseOrder<K> {
    fn new(entries: impl Iterator<Item = (K, f32)>) -> Self {
        let mut entries: Vec<_> = entries.collect();
        entries.sort_unstable_by(|(a, x), (b, y)| y.total_cmp(x).then_with(|| a.cmp(b)));
        let ranks = entries
            .iter()
            .enumerate()
            .map(|(i, (key, _))| (*key, i as u32))
            .collect();
        Self { entries, ranks }
    }

    pub fn rank(&self, key: &K) -> Option<u32> {
        self.ranks.get(key).copied()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn ease_at(&self, rank: u32) -> Option<f32> {
        self.entries.get(rank as usize).map(|(_, ease)| *ease)
    }

    pub fn iter_from(&self, rank: u32) -> impl Iterator<Item = K> + '_ {
        self.entries[rank as usize..].iter().map(|(key, _)| *key)
    }

    pub fn partition_point(&self, mut predicate: impl FnMut(f32) -> bool) -> u32 {
        self.entries.partition_point(|(_, ease)| predicate(*ease)) as u32
    }
}

fn ease_orders(
    frequencies: &FrequencyList,
) -> (EaseOrder<TaggedGram<SpurGram>>, EaseOrder<SpurGram>) {
    let written = EaseOrder::new(
        frequencies
            .entries
            .iter()
            .map(|(key, frequency)| (*key, frequency.ease)),
    );
    let mut listening: FxHashMap<SpurGram, f32> = FxHashMap::default();
    for (key, frequency) in frequencies.entries.iter() {
        listening
            .entry(key.gram)
            .and_modify(|ease| *ease = ease.max(frequency.ease))
            .or_insert(frequency.ease);
    }
    (written, EaseOrder::new(listening.into_iter()))
}

/// A frequency list with its total count (interned version for the language pack).
#[derive(Clone, Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct FrequencyList {
    pub entries: IndexMap<TaggedGram<SpurGram>, Frequency>,
    /// Total gram count from unfiltered data (for accurate percentage calculations)
    pub total_count: u64,
}

/// The assembled runtime pack. Never serialized itself — the on-disk/wire
/// format is the [`LanguagePackCore`] + [`LanguagePackSentences`] pair, put
/// back together by [`LanguagePack::from_parts`].
#[derive(Debug)]
pub struct LanguagePack {
    pub core: Arc<LanguagePackCore>,
    pub sentences: LanguagePackSentences,
}

impl std::ops::Deref for LanguagePack {
    type Target = LanguagePackCore;

    fn deref(&self) -> &Self::Target {
        &self.core
    }
}

impl LanguagePack {
    /// Listening cards represent the sound of a bare gram across all senses.
    /// Use the maximum sense ease, retain leading flags, and sum occurrence counts.
    pub fn gram_frequency_total(&self, gram: SpurGram) -> Option<Frequency> {
        let mut frequencies = self
            .senses_of(gram)
            .iter()
            .filter_map(|entry| self.gram_frequencies.entries.get(entry));
        let mut total = *frequencies.next()?;
        for frequency in frequencies {
            total.count += frequency.count;
            total.direct_count += frequency.direct_count;
            total.ease = total.ease.max(frequency.ease);
        }
        Some(total)
    }

    /// The requested vocabulary entry, falling back to the most frequent sense
    /// for untagged or stale identities. Foreign grams have no entry.
    pub fn resolve_entry(&self, entry: &TaggedGram<Gram<String>>) -> Option<TaggedGram<SpurGram>> {
        let gram = self.intern_gram(&entry.gram)?;
        let requested = TaggedGram {
            gram,
            sense: entry.sense,
        };
        self.gram_frequencies
            .entries
            .contains_key(&requested)
            .then_some(requested)
            .or_else(|| self.senses_of(gram).first().copied())
    }

    /// Canonical front prefix: the most frequent sense, independent of deck eligibility.
    pub fn word_prefix(
        &self,
        gram: SpurGram,
        language: crate::Language,
    ) -> Option<crate::features::WordPrefix> {
        let definition = self.gram_definitions.get(self.senses_of(gram).first()?)?;
        crate::compute_word_prefix(&self.resolve_gram(&gram), definition, language)
    }

    pub fn senses_of(&self, gram: SpurGram) -> &[TaggedGram<SpurGram>] {
        self.senses
            .get(&gram)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    /// The senses of `gram` worth showing in a dictionary: every sense that is not
    /// in a redundancy set, plus the most frequent member of each set. Frequency
    /// order, like `senses_of`.
    pub fn visible_senses(
        &self,
        gram: SpurGram,
        language: crate::Language,
    ) -> Vec<TaggedGram<SpurGram>> {
        self.senses_of(gram)
            .iter()
            .copied()
            .filter(|sense| self.is_visible(*sense, language))
            .collect()
    }

    /// Show the most frequent set member on each unprefixed dictionary page.
    /// A sense outside a redundancy set is its own representative.
    pub fn is_visible(&self, sense: TaggedGram<SpurGram>, language: crate::Language) -> bool {
        let set = self.redundant_with(sense);
        if set.len() == 1 {
            return true;
        }
        let display = self.resolve_gram(&sense.gram).to_display_string(language);
        // A redundancy set may span pages (est and il est); never hide a page.
        set.into_iter()
            .filter(|member| self.resolve_gram(&member.gram).to_display_string(language) == display)
            .min_by_key(|member| self.gram_frequencies.entries.get_index_of(member))
            == Some(sense)
    }

    /// All members of this sense's redundancy set, or just the sense itself.
    pub fn redundant_with(&self, sense: TaggedGram<SpurGram>) -> Vec<TaggedGram<SpurGram>> {
        if let Some(text) = self.sentences.prefixed_text.get(&sense.gram)
            && let Some(sets) = self.sentences.redundant_senses.get(text)
            && let Some(set) = sets.iter().find(|set| set.contains(&sense))
        {
            return set.clone();
        }
        vec![sense]
    }

    /// The guide teaching `pattern` at `position`, if this pack has one. A
    /// deck can hold a pronunciation card the pack no longer teaches (the
    /// guide lost every verified example, or the pack was rebuilt), so
    /// callers must treat `None` as "nothing to show".
    pub fn pronunciation_guide(
        &self,
        pattern: Spur,
        position: PatternPosition,
    ) -> Option<&PronunciationGuide> {
        let pattern = self.string_rodeo.resolve(&pattern);
        self.pronunciation_data
            .guides
            .iter()
            .find(|guide| guide.pattern == pattern && guide.position == position)
    }

    /// Intern a resolved gram back into this pack's rodeos. Returns None if
    /// any token, or the gram itself, was never interned.
    pub fn intern_gram(&self, gram: &Gram<String>) -> Option<SpurGram> {
        gram.get_interned(&self.string_rodeo)?
            .get_interned(&self.gram_rodeo)
    }

    /// Whether a gram is a real entry in this course. The rodeos intern more
    /// grams than the course actually teaches; the master frequency list is
    /// the canonical set.
    pub fn is_course_gram(&self, gram: &TaggedGram<SpurGram>) -> bool {
        self.gram_frequencies.entries.contains_key(gram)
    }

    /// Resolve an interned gram back to its owned string form.
    pub fn resolve_gram(&self, gram: &SpurGram) -> Gram<String> {
        self.gram_rodeo.resolve(gram).resolve(&self.string_rodeo)
    }

    /// Sentences that contain `required_gram` (when given) and are otherwise fully
    /// comprehensible: every learnable gram and multiword term must satisfy
    /// `is_comprehensible` (the required gram itself always counts), and the
    /// sentence must have at least one translation.
    pub fn comprehensible_sentences(
        &self,
        required_gram: Option<&TaggedGram<SpurGram>>,
        is_comprehensible: impl Fn(&TaggedGram<SpurGram>) -> bool,
    ) -> Vec<SentenceSpur> {
        // Search through all sentences - if we have a required gram, only look at sentences containing it
        let candidate_sentences: &[SentenceSpur] = if let Some(required) = required_gram {
            match self.sentences.sentences_containing_gram_index.get(required) {
                Some(sentences) => sentences,
                None => return Vec::new(),
            }
        } else {
            // If no required gram/phrase, consider all sentences
            &self
                .sentences
                .translations
                .keys()
                .copied()
                .collect::<Vec<_>>()
        };
        candidate_sentences
            .iter()
            .copied()
            .filter(|&sentence| {
                self.sentence_is_comprehensible(sentence, required_gram, &is_comprehensible)
            })
            .collect()
    }

    /// Whether one sentence is fully comprehensible: every learnable gram and
    /// multiword term satisfies `is_comprehensible` (the required gram itself
    /// always counts) and the sentence has a translation. This is the hot
    /// check behind [`Self::comprehensible_sentences`]; callers that already
    /// know their candidates use it directly.
    pub fn sentence_is_comprehensible(
        &self,
        sentence: SentenceSpur,
        required_gram: Option<&TaggedGram<SpurGram>>,
        is_comprehensible: &impl Fn(&TaggedGram<SpurGram>) -> bool,
    ) -> bool {
        let is_comprehensible = |gram: &TaggedGram<SpurGram>| {
            is_comprehensible(gram) || required_gram.is_some_and(|req| req == gram)
        };
        let Some(sentence_grams) = self.sentences.encoded_sentences.get(&sentence) else {
            return false;
        };
        let learnable = sentence_grams.grams.iter().filter_map(|g| match g {
            SentenceGram::Learnable(gram) => Some(&gram.sense),
            SentenceGram::Obvious(_) => None,
        });
        let multiword = sentence_grams
            .multiword_terms
            .iter()
            .chain(&sentence_grams.low_confidence_multiword_terms)
            .map(|term| &term.gram.sense);
        learnable.chain(multiword).all(is_comprehensible)
            && self
                .sentences
                .translations
                .get(&sentence)
                .is_some_and(|t| !t.is_empty())
    }

    /// Get all lexemes for words that share a pronunciation
    /// Returns an iterator over (word, lexeme) pairs
    pub fn pronunciation_to_lexemes(
        &self,
        pronunciation: &Spur,
    ) -> impl Iterator<Item = (Spur, Lexeme<Spur>)> + '_ {
        self.pronunciation_to_words
            .get(pronunciation)
            .into_iter()
            .flat_map(|words| words.iter())
            .flat_map(move |word| {
                self.words_to_heteronyms
                    .get(word)
                    .into_iter()
                    .flat_map(|heteronyms| heteronyms.iter())
                    .map(move |heteronym| {
                        (
                            *word,
                            Lexeme::Heteronym {
                                heteronym: *heteronym,
                            },
                        )
                    })
            })
    }

    /// Derive flat literals for a sentence from its encoded gram representation.
    /// This computes the literals on-the-fly from `encoded_sentences` rather than
    /// storing a separate precomputed map.
    pub fn sentence_to_literals(
        &self,
        sentence: &SentenceSpur,
        language: crate::Language,
    ) -> Option<Vec<Literal<String>>> {
        Some(
            self.sentences
                .encoded_sentences
                .get(sentence)?
                .clone()
                .without_readings()
                .to_literals(&self.string_rodeo, &self.gram_rodeo, language)
                .into_iter()
                .flat_map(|gram| match gram {
                    SentenceGram::Learnable((_, literals))
                    | SentenceGram::Obvious((_, literals)) => literals,
                })
                .collect(),
        )
    }

    /// Get the maximum frequency for any word with this pronunciation
    pub fn pronunciation_max_frequency(&self, pronunciation: &Spur) -> Option<Frequency> {
        self.pronunciation_max_freq_cache
            .get(pronunciation)
            .copied()
    }

    /// Native-language gloss for a single heteronym: finds its most-frequent
    /// gram, looks up the dictionary entry, and returns the first definition.
    pub fn heteronym_gloss(&self, heteronym: &Heteronym<Spur>) -> Option<String> {
        let gram = self.heteronym_to_grams.get(heteronym)?.first()?;
        let gram = self.senses_of(*gram).first()?;
        match self.gram_definitions.get(gram)? {
            GramDefinition::Dictionary(d) => d.definitions.first().map(|td| td.native.clone()),
            GramDefinition::Phrasebook(_) => None,
        }
    }

    /// Get a learner-facing gloss for a morpheme.
    ///
    /// - `Root` morphemes are glossed by looking up the heteronym's first
    ///   dictionary entry and joining its native-language definitions.
    /// - `Bound`, `Derivation`, and `Inflection` return their stored `meaning`.
    ///
    /// Returns `None` if no gloss is available (e.g. a `Root` whose heteronym
    /// has no dictionary entry, or an affix with `meaning: None`).
    pub fn morpheme_gloss(&self, info: &MorphemeInfo<Spur>) -> Option<String> {
        match info {
            MorphemeInfo::Root { heteronym } => self.heteronym_gloss(heteronym),
            MorphemeInfo::Bound { meaning }
            | MorphemeInfo::Derivation { meaning }
            | MorphemeInfo::Inflection { meaning } => {
                meaning.map(|spur| self.string_rodeo.resolve(&spur).to_string())
            }
        }
    }

    /// Break a heteronym's word into its constituent morphemes. Each entry is
    /// `(surface, canonical, gloss)`. The gloss is required for a morpheme-level
    /// breakdown (all-or-nothing); `canonical` is `Some` only when it differs
    /// from the surface (and from the full word).
    fn compute_morpheme_breakdown(
        &self,
        heteronym: &Heteronym<Spur>,
    ) -> Option<Vec<(String, Option<String>, String)>> {
        let rodeo = &self.string_rodeo;

        // The segmentation lives on the DictionaryEntry, reached via the
        // heteronym's first (most frequent) gram.
        let gram = self.heteronym_to_grams.get(heteronym)?.first()?;
        let gram = self.senses_of(*gram).first()?;
        let dict_entry = match self.gram_definitions.get(gram)? {
            GramDefinition::Dictionary(d) => d,
            GramDefinition::Phrasebook(_) => return None,
        };

        if dict_entry.segments.len() < 2 {
            return None;
        }

        let full_word = rodeo.resolve(&heteronym.word);

        dict_entry
            .segments
            .iter()
            .map(|segment| {
                let key = MorphemeSegment {
                    surface: rodeo.get(&segment.surface)?,
                    canonical: rodeo.get(&segment.canonical)?,
                    tag: match &segment.tag {
                        Some(t) => Some(rodeo.get(t)?),
                        None => None,
                    },
                };
                let info = self.morphemes.get(&key)?;
                let gloss = self.morpheme_gloss(info)?;
                let canonical =
                    if segment.canonical == segment.surface || segment.canonical == full_word {
                        None
                    } else {
                        Some(segment.canonical.clone())
                    };
                Some((segment.surface.clone(), canonical, gloss))
            })
            .collect()
    }

    /// Learner-facing breakdown of a gram. Each entry is
    /// `(surface, canonical, gloss)`; `canonical` is `Some` only when it differs
    /// from the surface, and `gloss` is optional so punctuation atoms in
    /// multi-word grams can render with an empty native-language cell.
    ///
    /// - Single-heteronym grams → **morpheme-level** decomposition (all glosses
    ///   required, all-or-nothing).
    /// - Multi-atom grams → **word-level** decomposition, but only when the
    ///   phrase has a `Phrasebook` entry flagged `compositional` (for idiomatic
    ///   phrases, per-word glosses are misleading).
    #[allow(clippy::type_complexity)]
    pub fn compute_breakdown(
        &self,
        gram_spur: SpurGram,
    ) -> Option<Vec<(String, Option<String>, Option<String>)>> {
        let gram = self.gram_rodeo.resolve(&gram_spur);
        match gram.atoms() {
            [Atom::Tok(word)] => match &word.word_type {
                WordType::Heteronym(het) => {
                    let inner = self.compute_morpheme_breakdown(het)?;
                    Some(inner.into_iter().map(|(s, c, g)| (s, c, Some(g))).collect())
                }
                _ => None,
            },
            atoms => {
                let is_compositional = matches!(
                    self.gram_definitions.get(self.senses_of(gram_spur).first()?),
                    Some(GramDefinition::Phrasebook(entry)) if entry.compositional
                );
                if !is_compositional {
                    return None;
                }
                let rodeo = &self.string_rodeo;
                let out: Vec<(String, Option<String>, Option<String>)> = atoms
                    .iter()
                    .filter_map(|atom| match atom {
                        Atom::Tok(word) => {
                            let surface = rodeo.resolve(&word.text).to_string();
                            let (canonical, gloss) = match &word.word_type {
                                WordType::Heteronym(het) => {
                                    let lemma = rodeo.resolve(&het.lemma);
                                    let canonical = if lemma == surface {
                                        None
                                    } else {
                                        Some(lemma.to_string())
                                    };
                                    (canonical, self.heteronym_gloss(het))
                                }
                                _ => (None, None),
                            };
                            Some((surface, canonical, gloss))
                        }
                        _ => None,
                    })
                    .collect();
                if out.len() < 2 { None } else { Some(out) }
            }
        }
    }

    /// Build a pack from generated corpus data. Runtime loaders use `from_parts`.
    pub fn new(language_data: ConsolidatedLanguageData, course: Course) -> Self {
        let target_language = course.target_language;
        let texts = crate::word_prefix::prefixed_texts(
            &language_data.gram_frequencies.entries,
            &language_data.gram_dictionary,
            &language_data.phrasebook,
            target_language,
        );
        let rodeo = {
            let mut rodeo = lasso::Rodeo::new();
            language_data.intern(&mut rodeo);
            for text in texts.values() {
                rodeo.get_or_intern(text);
            }
            rodeo.into_reader()
        };

        let mut sentence_rodeo: lasso::Rodeo<String, SentenceSpur> = lasso::Rodeo::new();
        let mut sentence_texts: std::collections::BTreeSet<&str> = language_data
            .target_language_sentences
            .iter()
            .map(String::as_str)
            .collect();
        sentence_texts.extend(language_data.translations.iter().map(|(s, _)| s.as_str()));
        sentence_texts.extend(
            language_data
                .encoded_sentences
                .iter()
                .map(|(s, _)| s.as_str()),
        );
        sentence_texts.extend(
            language_data
                .sentence_sources
                .iter()
                .map(|(s, _)| s.as_str()),
        );
        for practice in language_data.homophone_practice.values() {
            for pair in &practice.sentence_pairs {
                sentence_texts.extend([pair.sentence1.as_str(), pair.sentence2.as_str()]);
            }
        }
        for text in sentence_texts {
            sentence_rodeo.get_or_intern(text);
        }
        let sentence_rodeo = sentence_rodeo.into_reader();
        let mut translation_rodeo: lasso::Rodeo<String, TranslationSpur> = lasso::Rodeo::new();
        let translation_texts: std::collections::BTreeSet<&str> = language_data
            .translations
            .iter()
            .flat_map(|(_, ts)| ts.iter().map(String::as_str))
            .collect();
        for text in translation_texts {
            translation_rodeo.get_or_intern(text);
        }
        let translations = language_data
            .translations
            .iter()
            .map(|(s, ts)| {
                (
                    sentence_rodeo.get(s).unwrap(),
                    ts.iter()
                        .map(|t| translation_rodeo.get(t).unwrap())
                        .collect(),
                )
            })
            .collect();
        let translation_rodeo = translation_rodeo.into_reader();

        let words_to_heteronyms: FxHashMap<Spur, Vec<Heteronym<Spur>>> = {
            let mut map: FxHashMap<Spur, Vec<(Heteronym<Spur>, u32)>> = FxHashMap::default();

            for entry in &language_data.gram_frequencies.entries {
                if let Some(heteronym) = entry.gram.gram.heteronym() {
                    let word_spur = rodeo.get(&heteronym.word).unwrap();
                    let interned_het = Heteronym {
                        word: rodeo.get(&heteronym.word).unwrap(),
                        lemma: rodeo.get(&heteronym.lemma).unwrap(),
                        pos: heteronym.pos,
                    };
                    let vec = map.entry(word_spur).or_default();
                    if let Some(existing) = vec.iter_mut().find(|(h, _)| *h == interned_het) {
                        existing.1 = existing.1.max(entry.count);
                    } else {
                        vec.push((interned_het, entry.count));
                    }
                }
            }

            map.into_iter()
                .map(|(word, mut hets)| {
                    hets.sort_by_key(|b| std::cmp::Reverse(b.1));
                    (word, hets.into_iter().map(|(h, _)| h).collect::<Vec<_>>())
                })
                .collect()
        };

        let word_to_pronunciation = {
            language_data
                .word_to_pronunciation
                .iter()
                .map(|(word, pronunciations)| {
                    (
                        rodeo
                            .get(word)
                            .unwrap_or_else(|| panic!("word not in rodeo: {word:?}")),
                        // Language pack only stores the main pronunciation;
                        // verifier alternates aren't surfaced to the frontend yet.
                        rodeo.get(&pronunciations.main).unwrap_or_else(|| {
                            panic!("pronunciation not in rodeo: {:?}", pronunciations.main)
                        }),
                    )
                })
                .collect()
        };

        let pronunciation_to_words: FxHashMap<Spur, Vec<Spur>> = {
            language_data
                .pronunciation_to_words
                .iter()
                .map(|(pronunciation, words)| {
                    (
                        rodeo.get(pronunciation).unwrap(),
                        words.iter().map(|word| rodeo.get(word).unwrap()).collect(),
                    )
                })
                .collect()
        };

        let pronunciation_data = language_data.pronunciation_data.clone();

        let pattern_frequency_map = {
            pronunciation_data
                .pattern_frequencies
                .iter()
                .map(|((pattern, position), freq)| {
                    ((rodeo.get(pattern).unwrap(), *position), *freq)
                })
                .collect()
        };

        let homophone_practice = language_data
            .homophone_practice
            .iter()
            .map(|(word_pair, practice)| {
                (
                    word_pair.get_interned(&rodeo).unwrap(),
                    practice.get_interned(&sentence_rodeo).unwrap(),
                )
            })
            .collect();

        let movies = visible_movies(language_data.movies);
        let books = language_data.books;

        let human_audio = language_data.human_audio.clone();
        let pronunciation_audio = language_data.pronunciation_audio.clone();

        let source_gram_frequencies_data = language_data.source_gram_frequencies.clone();

        let sentence_sources = {
            language_data
                .sentence_sources
                .iter()
                .map(|(sentence, source)| (sentence_rodeo.get(sentence).unwrap(), source.clone()))
                .collect()
        };

        let proper_noun_definitions = {
            language_data
                .proper_noun_definitions
                .iter()
                .map(|(proper_noun, definition)| {
                    (rodeo.get(proper_noun).unwrap(), definition.clone())
                })
                .collect()
        };

        let gram_vocabulary: Vec<_> = language_data
            .gram_vocabulary
            .iter()
            .map(|entry| {
                entry
                    .get_interned(&rodeo)
                    .expect("all gram vocab atoms should be interned")
            })
            .collect();

        let gram_rodeo = {
            let mut gram_rodeo: lasso::Rodeo<Gram<Spur>> = lasso::Rodeo::new();
            for entry in &gram_vocabulary {
                gram_rodeo.get_or_intern(entry.atoms.clone());
            }
            gram_rodeo.into_reader()
        };

        let encoded_sentences: FxHashMap<
            SentenceSpur,
            SentenceGrams<crate::ReadingTaggedGram<SpurGram, Spur>>,
        > = language_data
            .encoded_sentences
            .iter()
            .filter_map(|(sentence, encoded)| {
                let interned_grams: Option<
                    Vec<SentenceGram<crate::ReadingTaggedGram<SpurGram, Spur>>>,
                > = encoded
                    .grams
                    .iter()
                    .map(|g| {
                        g.clone().try_map(|entry| {
                            entry
                                .get_interned(&rodeo)
                                .expect("encoded sentence strings must be interned")
                                .get_interned(&gram_rodeo)
                        })
                    })
                    .collect();
                let interned_multiword_terms: Option<
                    Vec<MultiwordTermMatch<crate::ReadingTaggedGram<SpurGram, Spur>>>,
                > = encoded
                    .multiword_terms
                    .iter()
                    .map(|term| {
                        let interned = term
                            .gram
                            .get_interned(&rodeo)
                            .expect("encoded phrase strings must be interned");
                        Some(MultiwordTermMatch {
                            gram: interned.get_interned(&gram_rodeo)?,
                            matched_word_indices: term.matched_word_indices.clone(),
                        })
                    })
                    .collect();
                let interned_low_confidence_multiword_terms: Option<
                    Vec<MultiwordTermMatch<crate::ReadingTaggedGram<SpurGram, Spur>>>,
                > = encoded
                    .low_confidence_multiword_terms
                    .iter()
                    .map(|term| {
                        let interned = term
                            .gram
                            .get_interned(&rodeo)
                            .expect("encoded phrase strings must be interned");
                        Some(MultiwordTermMatch {
                            gram: interned.get_interned(&gram_rodeo)?,
                            matched_word_indices: term.matched_word_indices.clone(),
                        })
                    })
                    .collect();
                Some((
                    sentence_rodeo.get(sentence)?,
                    SentenceGrams::<crate::ReadingTaggedGram<SpurGram, Spur>> {
                        grams: interned_grams?,
                        capitalize_first: encoded.capitalize_first,
                        multiword_terms: interned_multiword_terms?,
                        low_confidence_multiword_terms: interned_low_confidence_multiword_terms?,
                    },
                ))
            })
            .collect();

        // Build as counts first; we'll compute the `easy` flag after gram_definitions is available.
        let gram_counts: IndexMap<TaggedGram<SpurGram>, (u32, u32)> = {
            let mut map = IndexMap::new();
            for entry in &language_data.gram_frequencies.entries {
                let interned_gram = entry.gram.get_interned(&rodeo);
                if let Some(interned_gram) = interned_gram
                    && let Some(gram_spur) = interned_gram.get_interned(&gram_rodeo)
                {
                    map.insert(gram_spur, (entry.count, entry.direct_count));
                }
            }
            map
        };

        let gram_definitions: FxHashMap<TaggedGram<SpurGram>, GramDefinition> = {
            let mut map = FxHashMap::default();

            for (gram, definition) in &language_data.gram_dictionary {
                let interned_gram: Option<Gram<Spur>> = gram
                    .gram
                    .iter()
                    .map(|atom| atom.get_interned(&rodeo))
                    .collect();
                if let Some(interned_gram) = interned_gram
                    && let Some(gram_spur) = interned_gram.get_interned(&gram_rodeo)
                {
                    map.insert(
                        TaggedGram {
                            gram: gram_spur,
                            sense: gram.sense,
                        },
                        GramDefinition::Dictionary(definition.clone()),
                    );
                }
            }

            for (gram, entry) in &language_data.phrasebook {
                let interned_gram: Option<Gram<Spur>> = gram
                    .gram
                    .iter()
                    .map(|atom| atom.get_interned(&rodeo))
                    .collect();
                if let Some(interned_gram) = interned_gram
                    && let Some(gram_spur) = interned_gram.get_interned(&gram_rodeo)
                {
                    map.insert(
                        TaggedGram {
                            gram: gram_spur,
                            sense: gram.sense,
                        },
                        GramDefinition::Phrasebook(entry.clone()),
                    );
                }
            }

            map
        };

        let gram_frequencies_map: IndexMap<TaggedGram<SpurGram>, Frequency> = {
            const COGNATE_BONUS: f32 = 2.0;

            // First pass: compute base ease for single-atom grams so we can reference them
            // when computing multi-atom gram ease.
            let mut single_atom_ease: FxHashMap<Atom<Spur>, f32> = FxHashMap::default();
            for (gram_spur, (count, _direct_count)) in gram_counts.iter() {
                let gram = gram_rodeo.resolve(&gram_spur.gram);
                if gram.len() == 1
                    && let Some(atom) = gram.iter().next()
                {
                    let base_ease = (*count as f32).ln();
                    let has_cognate = !course.teaches_new_writing_system()
                        && has_cognate_definition(gram_definitions.get(gram_spur));
                    let ease = if has_cognate {
                        base_ease + COGNATE_BONUS
                    } else {
                        base_ease
                    };
                    single_atom_ease
                        .entry(*atom)
                        .and_modify(|old| *old = old.max(ease))
                        .or_insert(ease);
                }
            }

            let mut map = IndexMap::new();
            for (gram_spur, (count, direct_count)) in gram_counts.iter() {
                let gram = gram_rodeo.resolve(&gram_spur.gram);
                let easy = !course.teaches_new_writing_system()
                    && is_gram_easy(gram, gram_definitions.get(gram_spur));

                let ln_count = (*count as f32).ln();

                let definition = gram_definitions.get(gram_spur);
                let (is_compositional, is_literally_translatable, is_cognate) = match definition {
                    Some(GramDefinition::Phrasebook(entry)) => (
                        entry.compositional,
                        entry.can_be_translated_literally,
                        entry.cognate && !entry.false_cognate,
                    ),
                    _ => (false, false, false),
                };
                let compositional = is_compositional || is_literally_translatable;

                let ease = if gram.len() <= 1 {
                    ln_count
                        + if !course.teaches_new_writing_system()
                            && has_cognate_definition(definition)
                        {
                            COGNATE_BONUS
                        } else {
                            0.0
                        }
                } else {
                    // Multi-atom gram: ease depends on compositionality and component word ease.
                    // A learnable component with no standalone gram never occurs outside this
                    // gram, so it can't be easier than the gram itself — fall back to ln_count
                    // instead of skipping it. Skipping would let the easy components set the
                    // ease (e.g. "est infecté" inheriting the ease of "est" when "infecté" has
                    // no entry of its own).
                    let min_component_ease = gram
                        .iter()
                        .filter_map(|atom| {
                            let ease = single_atom_ease.get(atom).copied();
                            match atom {
                                Atom::Tok(word)
                                    if matches!(word.word_type, WordType::Heteronym(_)) =>
                                {
                                    Some(ease.unwrap_or(ln_count))
                                }
                                _ => ease,
                            }
                        })
                        .reduce(f32::min);

                    let base_ease = match (
                        is_literally_translatable,
                        is_compositional,
                        min_component_ease,
                    ) {
                        // Can be translated word-for-word: as easy as the hardest component.
                        (true, _, Some(min_ease)) => min_ease.max(ln_count),
                        // Compositional but not literal: blend component ease with own frequency.
                        (false, true, Some(min_ease)) => {
                            (0.7 * min_ease + 0.3 * ln_count).max(ln_count)
                        }
                        // Non-compositional (idiom) or no component data: own frequency only.
                        _ => ln_count,
                    };

                    let cognate_bonus = if is_cognate && !course.teaches_new_writing_system() {
                        COGNATE_BONUS
                    } else {
                        0.0
                    };

                    base_ease + cognate_bonus
                };

                map.insert(
                    *gram_spur,
                    Frequency {
                        count: *count,
                        direct_count: *direct_count,
                        easy,
                        compositional,
                        ease,
                    },
                );
            }
            map
        };

        let gram_frequencies = FrequencyList {
            total_count: language_data.gram_frequencies.total_count,
            entries: gram_frequencies_map,
        };

        let senses = sense_index(&gram_frequencies);
        let (written_ease_order, listening_ease_order) = ease_orders(&gram_frequencies);
        let heteronym_to_grams: FxHashMap<Heteronym<Spur>, Vec<SpurGram>> = {
            let mut map: FxHashMap<Heteronym<Spur>, Vec<(SpurGram, Frequency)>> =
                FxHashMap::default();
            for (gram_spur, freq) in gram_frequencies.entries.iter() {
                let gram = gram_rodeo.resolve(&gram_spur.gram);
                if gram.len() == 1
                    && let Some(Atom::Tok(word)) = gram.iter().next()
                    && let WordType::Heteronym(heteronym) = &word.word_type
                {
                    map.entry(*heteronym)
                        .or_default()
                        .push((gram_spur.gram, *freq));
                }
            }
            map.into_iter()
                .map(|(k, mut v)| {
                    v.sort_by_key(|b| std::cmp::Reverse(b.1.count));
                    (k, {
                        let mut seen = std::collections::HashSet::new();
                        v.into_iter()
                            .map(|(spur, _)| spur)
                            .filter(|g| seen.insert(*g))
                            .collect()
                    })
                })
                .collect()
        };

        let sentences_containing_gram_index: FxHashMap<TaggedGram<SpurGram>, Vec<SentenceSpur>> = {
            let mut map: FxHashMap<TaggedGram<SpurGram>, Vec<SentenceSpur>> = FxHashMap::default();
            for (sentence_spur, sentence_grams) in encoded_sentences.iter() {
                for gram in sentence_grams.all_grams() {
                    map.entry(gram.sense).or_default().push(*sentence_spur);
                }
            }
            map
        };

        let source_gram_frequencies: FxHashMap<crate::FrequencySourceId, FrequencyList> =
            source_gram_frequencies_data
                .iter()
                .map(|(source_id, freq_list)| {
                    let mut map: IndexMap<TaggedGram<SpurGram>, Frequency> = IndexMap::new();
                    for entry in &freq_list.entries {
                        let interned_gram = entry.gram.get_interned(&rodeo);
                        if let Some(interned_gram) = interned_gram
                            && let Some(gram_spur) = interned_gram.get_interned(&gram_rodeo)
                        {
                            // Use ease from global gram_frequencies; fall back to ln(count)
                            let global_freq = gram_frequencies.entries.get(&gram_spur);
                            map.insert(
                                gram_spur,
                                Frequency {
                                    count: entry.count,
                                    direct_count: entry.direct_count,
                                    easy: global_freq.is_some_and(|f| f.easy),
                                    compositional: global_freq.is_some_and(|f| f.compositional),
                                    ease: global_freq
                                        .map(|f| f.ease)
                                        .unwrap_or_else(|| (entry.count as f32).ln()),
                                },
                            );
                        }
                    }
                    (
                        source_id.clone(),
                        FrequencyList {
                            entries: map,
                            total_count: freq_list.total_count,
                        },
                    )
                })
                .filter(|(_, list)| !list.entries.is_empty())
                .collect();

        let pronunciation_max_freq_cache: FxHashMap<Spur, Frequency> = pronunciation_to_words
            .iter()
            .filter_map(|(pronunciation, words)| {
                let max_freq = words
                    .iter()
                    .flat_map(|word| {
                        words_to_heteronyms
                            .get(word)
                            .map(|heteronyms| heteronyms.iter())
                            .into_iter()
                            .flatten()
                    })
                    .filter_map(|heteronym| {
                        heteronym_to_grams
                            .get(heteronym)?
                            .iter()
                            .filter_map(|gram_spur| {
                                gram_frequencies
                                    .entries
                                    .get(senses.get(gram_spur)?.first()?)
                                    .copied()
                            })
                            .max_by_key(|f| f.count)
                    })
                    .max_by_key(|f| f.count)?;
                Some((*pronunciation, max_freq))
            })
            .collect();

        // Rank minimal pairs by standalone usage; each heteronym
        // lists its single-atom grams in descending frequency order.
        let word_max_single_gram_freq: FxHashMap<Spur, u32> = words_to_heteronyms
            .iter()
            .filter_map(|(word, hets)| {
                let max = hets
                    .iter()
                    .filter_map(|het| {
                        let top_gram = heteronym_to_grams.get(het)?.first()?;
                        gram_frequencies
                            .entries
                            .get(senses.get(top_gram)?.first()?)
                            .map(|f| f.count)
                    })
                    .max()?;
                Some((*word, max))
            })
            .collect();

        let minimal_pairs = build_minimal_pairs_index(
            &language_data.minimal_pairs,
            &rodeo,
            &word_max_single_gram_freq,
        );

        let string_to_grams: FxHashMap<String, Vec<TaggedGram<SpurGram>>> = {
            let mut map: FxHashMap<String, Vec<TaggedGram<SpurGram>>> = FxHashMap::default();
            for (&gram_spur, _) in gram_frequencies.entries.iter() {
                let resolved = gram_rodeo.resolve(&gram_spur.gram).resolve(&rodeo);
                let display = resolved.to_display_string(target_language);
                map.entry(display).or_default().push(gram_spur);
            }
            map
        };

        let morphemes: FxHashMap<MorphemeSegment<Spur>, MorphemeInfo<Spur>> = language_data
            .morphemes
            .iter()
            .map(|(segment, info)| {
                let key = MorphemeSegment {
                    surface: rodeo.get(&segment.surface).unwrap(),
                    canonical: rodeo.get(&segment.canonical).unwrap(),
                    tag: segment.tag.as_ref().map(|t| rodeo.get(t).unwrap()),
                };
                let interned = match info {
                    MorphemeInfo::Root { heteronym } => MorphemeInfo::Root {
                        heteronym: Heteronym {
                            word: rodeo.get(&heteronym.word).unwrap(),
                            lemma: rodeo.get(&heteronym.lemma).unwrap(),
                            pos: heteronym.pos,
                        },
                    },
                    MorphemeInfo::Bound { meaning } => MorphemeInfo::Bound {
                        meaning: meaning.as_ref().map(|m| rodeo.get(m).unwrap()),
                    },
                    MorphemeInfo::Derivation { meaning } => MorphemeInfo::Derivation {
                        meaning: meaning.as_ref().map(|m| rodeo.get(m).unwrap()),
                    },
                    MorphemeInfo::Inflection { meaning } => MorphemeInfo::Inflection {
                        meaning: meaning.as_ref().map(|m| rodeo.get(m).unwrap()),
                    },
                };
                (key, interned)
            })
            .collect();

        let prefixed_text: FxHashMap<_, _> = texts
            .iter()
            .filter_map(|(gram, text)| {
                Some((
                    gram.get_interned(&rodeo)?.get_interned(&gram_rodeo)?,
                    rodeo.get(text)?,
                ))
            })
            .collect();
        let mut grams_with_prefixed_text: FxHashMap<_, Vec<_>> = FxHashMap::default();
        for (&gram, _) in gram_frequencies.entries.iter() {
            if let Some(&text) = prefixed_text.get(&gram.gram) {
                grams_with_prefixed_text.entry(text).or_default().push(gram);
            }
        }
        let redundant_senses = language_data
            .redundant_senses
            .iter()
            .filter_map(|(text, sets)| {
                let text = rodeo.get(text)?;
                let sets = sets
                    .iter()
                    .filter_map(|set| {
                        let members: Vec<_> = set
                            .iter()
                            .filter_map(|gram| gram.get_interned(&rodeo)?.get_interned(&gram_rodeo))
                            .collect();
                        (members.len() >= 2).then_some(members)
                    })
                    .collect();
                Some((text, sets))
            })
            .collect();

        let gram_readings = derive_readings(&encoded_sentences, &rodeo, &gram_rodeo);
        let fingerprint = spur_space_fingerprint(&rodeo, &gram_rodeo);
        Self {
            core: Arc::new(LanguagePackCore {
                spur_space_fingerprint: fingerprint,
                string_rodeo: rodeo,
                gram_rodeo,
                senses,
                written_ease_order,
                listening_ease_order,
                words_to_heteronyms,
                word_to_pronunciation,
                pronunciation_to_words,
                minimal_pairs,
                pronunciation_data,
                pattern_frequency_map,
                pronunciation_max_freq_cache,
                proper_noun_definitions,
                gram_frequencies,
                gram_definitions,
                heteronym_to_grams,
                string_to_grams,
                morphemes,
                pronunciation_audio,
            }),
            sentences: LanguagePackSentences {
                spur_space_fingerprint: fingerprint,
                ruby_segmentations: BTreeMap::new(),
                gram_readings,
                prefixed_text,
                grams_with_prefixed_text,
                redundant_senses,
                sentence_rodeo,
                translation_rodeo,
                translations,
                source_gram_frequencies,
                homophone_practice,
                movies,
                books,
                sentence_sources,
                encoded_sentences,
                sentences_containing_gram_index,
                human_audio,
                strokes: language_data.strokes,
            },
        }
    }
}

/// Check if a resolved gram is "easy" based on its definition.
/// Easy grams are single-word cognates with a single-word definition.
fn is_gram_easy(gram: &grm<Spur>, definition: Option<&GramDefinition>) -> bool {
    if gram.len() != 1 {
        return false;
    }
    let Some(Atom::Tok(word)) = gram.iter().next() else {
        return false;
    };
    let WordType::Heteronym(h) = &word.word_type else {
        return false;
    };
    if h.pos == PartOfSpeech::Intj {
        return false;
    }

    let Some(GramDefinition::Dictionary(entry)) = definition else {
        return false;
    };
    is_dictionary_entry_easy(entry)
}

/// Check if a gram has any cognate definition (more permissive than is_gram_easy).
pub fn has_cognate_definition(definition: Option<&GramDefinition>) -> bool {
    match definition {
        Some(GramDefinition::Dictionary(entry)) => entry
            .definitions
            .iter()
            .any(|d| d.cognate && !d.false_cognate),
        Some(GramDefinition::Phrasebook(entry)) => entry.cognate && !entry.false_cognate,
        None => false,
    }
}

fn is_dictionary_entry_easy(entry: &DictionaryEntry) -> bool {
    if entry.definitions.len() != 1 {
        return false;
    }
    let Some(definition) = entry.definitions.first() else {
        return false;
    };
    if definition.native.contains(' ') {
        return false;
    }
    if definition.native == entry.target_language_word {
        return false;
    }
    definition.cognate && !definition.false_cognate
}

/// The word-level half of a language pack: dictionary, frequency,
/// pronunciation and morphology data, plus all shared strings and grams.
/// Downloaded first so the placement test can run before sentence data arrives.
///
/// The core owns every shared string and gram referenced by either half.
/// Sentence texts and native translations have independent typed interners in
/// [`LanguagePackSentences`]. The halves must come from the same
/// [`LanguagePack::new`] call and be versioned and shipped as one unit.
#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct LanguagePackCore {
    /// Fingerprint of this core's spur space (see [`spur_space_fingerprint`]).
    /// [`LanguagePack::from_parts`] refuses a sentences half whose
    /// fingerprint differs, so halves from different builds can't be
    /// silently combined.
    pub spur_space_fingerprint: u64,
    pub string_rodeo: lasso::RodeoReader,
    pub gram_rodeo: lasso::RodeoReader<Gram<Spur>>,
    pub written_ease_order: EaseOrder<TaggedGram<SpurGram>>,
    pub listening_ease_order: EaseOrder<SpurGram>,
    senses: FxHashMap<SpurGram, Vec<TaggedGram<SpurGram>>>,
    pub words_to_heteronyms: FxHashMap<Spur, Vec<Heteronym<Spur>>>,
    pub word_to_pronunciation: FxHashMap<Spur, Spur>,
    pub pronunciation_to_words: FxHashMap<Spur, Vec<Spur>>,
    pub minimal_pairs: MinimalPairs,
    pub pronunciation_data: PronunciationData,
    pub pronunciation_audio: FxHashMap<String, PronunciationClip>,
    pub pattern_frequency_map: FxHashMap<(Spur, PatternPosition), u32>,
    pub pronunciation_max_freq_cache: FxHashMap<Spur, Frequency>,
    pub proper_noun_definitions: BTreeMap<Spur, ProperNounDefinition>,
    pub gram_frequencies: FrequencyList,
    pub gram_definitions: FxHashMap<TaggedGram<SpurGram>, GramDefinition>,
    pub heteronym_to_grams: FxHashMap<Heteronym<Spur>, Vec<SpurGram>>,
    pub string_to_grams: FxHashMap<String, Vec<TaggedGram<SpurGram>>>,
    pub morphemes: FxHashMap<MorphemeSegment<Spur>, MorphemeInfo<Spur>>,
}

/// The sentence half of a language pack: sentence texts, translations,
/// per-sentence gram encodings and indexes, source metadata and human audio.
/// See [`LanguagePackCore`] for the spur-space contract between the halves.
#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct LanguagePackSentences {
    /// Observed readings, frequency-first with lexical ties; computed at build time.
    pub gram_readings: FxHashMap<TaggedGram<SpurGram>, Vec<Spur>>,
    /// Exact displayed text + reading; only segmentations differing from the fallback.
    pub ruby_segmentations: BTreeMap<(String, String), Vec<crate::readings::RubySegment>>,
    /// Cross-gram redundancy sets keyed by exact prefixed text, judged per course.
    pub redundant_senses: FxHashMap<Spur, Vec<Vec<TaggedGram<SpurGram>>>>,
    pub prefixed_text: FxHashMap<SpurGram, Spur>,
    pub grams_with_prefixed_text: FxHashMap<Spur, Vec<TaggedGram<SpurGram>>>,
    /// Downloaded stroke forms; not needed by the placement-test core.
    pub strokes: crate::StrokeTable,
    /// Fingerprint of the core spur space this half was built against.
    pub spur_space_fingerprint: u64,
    pub sentence_rodeo: lasso::RodeoReader<String, SentenceSpur>,
    pub translation_rodeo: lasso::RodeoReader<String, TranslationSpur>,
    pub translations: FxHashMap<SentenceSpur, Vec<TranslationSpur>>,
    pub source_gram_frequencies: FxHashMap<crate::FrequencySourceId, FrequencyList>,
    pub homophone_practice: FxHashMap<HomophoneWordPair<Spur>, HomophonePractice<SentenceSpur>>,
    pub movies: FxHashMap<String, MovieMetadata>,
    pub books: FxHashMap<String, BookMetadata>,
    pub sentence_sources: FxHashMap<SentenceSpur, SentenceSource>,
    pub encoded_sentences:
        FxHashMap<SentenceSpur, SentenceGrams<crate::ReadingTaggedGram<SpurGram, Spur>>>,
    pub sentences_containing_gram_index: FxHashMap<TaggedGram<SpurGram>, Vec<SentenceSpur>>,
    pub human_audio: FxHashMap<VoiceActor, FxHashMap<String, Audio>>,
}

impl LanguagePackSentences {
    fn empty(spur_space_fingerprint: u64) -> Self {
        Self {
            spur_space_fingerprint,
            sentence_rodeo: lasso::Rodeo::new().into_reader(),
            translation_rodeo: lasso::Rodeo::new().into_reader(),
            gram_readings: FxHashMap::default(),
            ruby_segmentations: BTreeMap::new(),
            redundant_senses: FxHashMap::default(),
            prefixed_text: FxHashMap::default(),
            grams_with_prefixed_text: FxHashMap::default(),
            strokes: crate::StrokeTable::default(),
            translations: FxHashMap::default(),
            source_gram_frequencies: FxHashMap::default(),
            homophone_practice: FxHashMap::default(),
            movies: FxHashMap::default(),
            books: FxHashMap::default(),
            sentence_sources: FxHashMap::default(),
            encoded_sentences: FxHashMap::default(),
            sentences_containing_gram_index: FxHashMap::default(),
            human_audio: FxHashMap::default(),
        }
    }
}

/// Fingerprint of a core spur space: every core string and every core gram,
/// in spur order. A sentences half fits a core exactly when the core's spur
/// space (both interners' contents *and* key assignment) matches the one it
/// was split against, which is precisely what this hashes — identical strings
/// alone are not enough, since e.g. a reordered frequency list reassigns gram
/// spurs without changing any string. Identical content keeps producing
/// identical fingerprints across rebuilds (the archives stay reproducible).
fn spur_space_fingerprint(
    core_strings: &lasso::RodeoReader,
    core_grams: &lasso::RodeoReader<Gram<Spur>>,
) -> u64 {
    let mut hasher = xxhash_rust::xxh3::Xxh3::new();
    hasher.update(&(core_strings.len() as u64).to_le_bytes());
    hasher.update(&(core_grams.len() as u64).to_le_bytes());
    for (_, s) in core_strings.iter() {
        hasher.update(&(s.len() as u64).to_le_bytes());
        hasher.update(s.as_bytes());
    }
    for (_, g) in core_grams.iter() {
        // A gram is atoms over the (already-hashed) string spur space; its
        // rkyv encoding is a stable byte form of exactly that.
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&Gram(g.atoms().to_vec()))
            .expect("gram serialization for fingerprint cannot fail");
        hasher.update(&(bytes.len() as u64).to_le_bytes());
        hasher.update(&bytes);
    }
    hasher.digest()
}

fn derive_readings(
    sentences: &FxHashMap<SentenceSpur, SentenceGrams<crate::ReadingTaggedGram<SpurGram, Spur>>>,
    strings: &lasso::RodeoReader,
    grams: &lasso::RodeoReader<Gram<Spur>>,
) -> FxHashMap<TaggedGram<SpurGram>, Vec<Spur>> {
    let mut counts: FxHashMap<_, FxHashMap<Spur, usize>> = FxHashMap::default();
    for sentence in sentences.values() {
        let words: Vec<_> = sentence
            .grams
            .iter()
            .flat_map(|g| {
                let (SentenceGram::Learnable(g) | SentenceGram::Obvious(g)) = g;
                grams.resolve(&g.sense.gram).iter().filter_map(|a| match a {
                    crate::Atom::Tok(w) => Some(w.text),
                    _ => None,
                })
            })
            .collect();
        let phrases = sentence
            .multiword_terms
            .iter()
            .chain(&sentence.low_confidence_multiword_terms)
            .filter(|term| {
                // Phrase keys are lemma sequences. Their occurrence reading is
                // a headword reading only when the selected surface is identical.
                let canonical =
                    grams
                        .resolve(&term.gram.sense.gram)
                        .iter()
                        .filter_map(|a| match a {
                            crate::Atom::Tok(w) => Some(w.text),
                            _ => None,
                        });
                canonical.map(Some).eq(term
                    .matched_word_indices
                    .iter()
                    .map(|&i| words.get(usize::from(i)).copied()))
            })
            .map(|term| &term.gram);
        for token in sentence
            .grams
            .iter()
            .map(|g| match g {
                SentenceGram::Learnable(g) | SentenceGram::Obvious(g) => g,
            })
            .chain(phrases)
        {
            if let Some(reading) = token.reading {
                *counts
                    .entry(token.sense)
                    .or_default()
                    .entry(reading)
                    .or_default() += 1;
            }
        }
    }
    counts
        .into_iter()
        .map(|(sense, counts)| {
            let mut readings: Vec<_> = counts.into_iter().collect();
            readings.sort_by(|a, b| {
                b.1.cmp(&a.1)
                    .then_with(|| strings.resolve(&a.0).cmp(strings.resolve(&b.0)))
            });
            (
                sense,
                readings.into_iter().map(|(reading, _)| reading).collect(),
            )
        })
        .collect()
}

impl LanguagePack {
    pub fn readings_for_sense(&self, gram: TaggedGram<SpurGram>) -> impl Iterator<Item = &str> {
        self.sentences
            .gram_readings
            .get(&gram)
            .into_iter()
            .flatten()
            .map(|s| self.string_rodeo.resolve(s))
    }

    /// Assemble a runtime pack from its halves. With `sentences: None`, the
    /// result has empty sentence-side maps — enough for the placement test
    /// and dictionary lookups, but no comprehensible-sentence search.
    pub fn from_parts(
        core: impl Into<Arc<LanguagePackCore>>,
        sentences: Option<LanguagePackSentences>,
    ) -> Self {
        let core = core.into();
        let sentences =
            sentences.unwrap_or_else(|| LanguagePackSentences::empty(core.spur_space_fingerprint));
        assert_eq!(
            core.spur_space_fingerprint, sentences.spur_space_fingerprint,
            "language pack halves are from different builds (core spur-space fingerprint \
             doesn't match the one the sentences half was built against)"
        );
        Self { core, sentences }
    }

    /// Add the sentence half without deserializing the core again.
    pub fn with_sentences(&self, sentences: LanguagePackSentences) -> Self {
        Self::from_parts(self.core.clone(), Some(sentences))
    }
}

/// Filenames of the two halves of a split language pack, relative to a
/// course's data directory. Shared by the generator, the converters, the
/// backend and every native loader so they can't drift apart.
pub const CORE_FILENAME: &str = "language_data_core.rkyv";
pub const SENTENCES_FILENAME: &str = "language_data_sentences.rkyv";

/// Load and assemble a full language pack from a course directory holding
/// the split `language_data_core.rkyv` + `language_data_sentences.rkyv` pair.
#[cfg(not(target_arch = "wasm32"))]
pub fn load_split_dir(dir: &std::path::Path) -> Result<LanguagePack, std::io::Error> {
    let read = |filename: &str| {
        std::fs::read(dir.join(filename)).map_err(|e| {
            std::io::Error::new(
                e.kind(),
                format!("failed to read {filename} in {}: {e}", dir.display()),
            )
        })
    };
    let rkyv_err = |e: rkyv::rancor::Error| std::io::Error::other(e.to_string());
    let core_bytes = read(CORE_FILENAME)?;
    let sentences_bytes = read(SENTENCES_FILENAME)?;
    let core = rkyv::access::<ArchivedLanguagePackCore, rkyv::rancor::Error>(&core_bytes)
        .map_err(rkyv_err)?;
    let core =
        rkyv::deserialize::<LanguagePackCore, rkyv::rancor::Error>(core).map_err(rkyv_err)?;
    let sentences =
        rkyv::access::<ArchivedLanguagePackSentences, rkyv::rancor::Error>(&sentences_bytes)
            .map_err(rkyv_err)?;
    let sentences = rkyv::deserialize::<LanguagePackSentences, rkyv::rancor::Error>(sentences)
        .map_err(rkyv_err)?;
    Ok(LanguagePack::from_parts(core, Some(sentences)))
}

/// Write `pack` as the pair plus the `language_data.hash` metadata
/// file (line 1: core `hash;size`, line 2: sentences `hash;size`) into a
/// course directory. The one writer shared by the generator and the
/// converter, so the on-disk layout has a single source of truth.
#[cfg(not(target_arch = "wasm32"))]
pub fn write_split_dir(dir: &std::path::Path, pack: LanguagePack) -> Result<(), std::io::Error> {
    use xxhash_rust::const_xxh3::xxh3_64;

    let core_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&*pack.core)
        .expect("failed to serialize language pack core");
    let sentences_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&pack.sentences)
        .expect("failed to serialize language pack sentences");

    std::fs::write(dir.join(CORE_FILENAME), &core_bytes)?;
    std::fs::write(dir.join(SENTENCES_FILENAME), &sentences_bytes)?;
    std::fs::write(
        dir.join("language_data.hash"),
        format!(
            "{};{}\n{};{}",
            xxh3_64(&core_bytes),
            core_bytes.len(),
            xxh3_64(&sentences_bytes),
            sentences_bytes.len()
        ),
    )?;
    Ok(())
}

/// Public, immutable language-pack objects (no learner data).
pub const PACKS_ORIGIN: &str = "https://packs.yap.town";

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackPart {
    Core,
    Sentences,
}

impl PackPart {
    pub const ALL: [Self; 2] = [Self::Core, Self::Sentences];

    pub fn slug(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Sentences => "sentences",
        }
    }

    pub fn filename(self) -> &'static str {
        match self {
            Self::Core => CORE_FILENAME,
            Self::Sentences => SENTENCES_FILENAME,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PartMeta {
    pub hash: u64,
    pub size: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackMetadata {
    pub core: PartMeta,
    pub sentences: PartMeta,
}

impl PackMetadata {
    pub fn part(self, part: PackPart) -> PartMeta {
        match part {
            PackPart::Core => self.core,
            PackPart::Sentences => self.sentences,
        }
    }
}

/// Exactly two `hash;size` lines, core then sentences. Reject malformed or
/// empty archives rather than manufacturing a URL from incomplete metadata.
pub fn parse_hash_metadata(metadata: &str) -> Result<PackMetadata, String> {
    let parse = |line: &str| -> Result<PartMeta, String> {
        let (hash, size) = line.trim().split_once(';').ok_or("expected hash;size")?;
        let hash = hash.parse().map_err(|_| "invalid pack hash")?;
        let size = size.parse().map_err(|_| "invalid pack size")?;
        if size == 0 {
            return Err("empty pack".into());
        }
        Ok(PartMeta { hash, size })
    };
    let mut lines = metadata.trim().lines();
    let core = parse(lines.next().ok_or("missing core metadata")?)?;
    let sentences = parse(lines.next().ok_or("missing sentences metadata")?)?;
    if lines.next().is_some() {
        return Err("extra pack metadata".into());
    }
    Ok(PackMetadata { core, sentences })
}

pub fn course_directory_slug(course: Course) -> String {
    format!(
        "{}_for_{}",
        course.target_language.code(),
        course.native_language.code()
    )
}

pub fn course_from_directory_slug(slug: &str) -> Option<Course> {
    let (target, native) = slug.split_once("_for_")?;
    let course = Course {
        target_language: crate::Language::from_code(target)?,
        native_language: crate::Language::from_code(native)?,
    };
    (course_directory_slug(course) == slug).then_some(course)
}

pub fn pack_key(course: Course, part: PackPart, hash: u64) -> String {
    format!(
        "{}/language_data_{}_{hash}.rkyv",
        course_directory_slug(course),
        part.slug()
    )
}

pub fn pack_url(origin: &str, course: Course, part: PackPart, hash: u64) -> String {
    format!(
        "{}/{}",
        origin.trim_end_matches('/'),
        pack_key(course, part, hash)
    )
}

#[cfg(test)]
mod pack_metadata_tests {
    use super::*;
    #[test]
    fn metadata_is_strict() {
        assert_eq!(
            parse_hash_metadata("12;34\n56;78\n").unwrap(),
            PackMetadata {
                core: PartMeta { hash: 12, size: 34 },
                sentences: PartMeta { hash: 56, size: 78 }
            }
        );
        for bad in [
            "",
            "1;2",
            "1;0\n2;3",
            "x;2\n3;4",
            "1;2\n3;4\n5;6",
            "1;2;3\n4;5",
        ] {
            assert!(parse_hash_metadata(bad).is_err(), "{bad}");
        }
    }
    #[test]
    fn keys_are_canonical() {
        let course = course_from_directory_slug("fra_for_eng").unwrap();
        assert_eq!(
            pack_url("https://packs.yap.town/", course, PackPart::Core, 123),
            "https://packs.yap.town/fra_for_eng/language_data_core_123.rkyv"
        );
        for bad in ["../fra_for_eng", "fra_for_eng/", "fra_for_unknown"] {
            assert!(course_from_directory_slug(bad).is_none());
        }
    }
}

/// Only films with a poster (and so with metadata) are shown anywhere. Applied
/// when a pack is built. Sentence
/// provenance is untouched: hiding a film must not remove its sentences.
fn visible_movies(
    mut movies: FxHashMap<String, MovieMetadata>,
) -> FxHashMap<String, MovieMetadata> {
    movies.retain(|_, movie| movie.poster_bytes.is_some());
    movies
}

fn sense_index(frequencies: &FrequencyList) -> FxHashMap<SpurGram, Vec<TaggedGram<SpurGram>>> {
    let mut index: FxHashMap<SpurGram, Vec<TaggedGram<SpurGram>>> = FxHashMap::default();
    for (entry, _) in frequencies.entries.iter() {
        index.entry(entry.gram).or_default().push(*entry);
    }
    index
}

#[cfg(test)]
mod sense_tests {
    #[test]
    fn visible_senses_use_frequency_order_and_survive_serialization() {
        let mut pack = pack_with_senses(&[(2, 20), (1, 10), (3, 5)]);
        let bare = pack.intern_gram(&gram("bank")).unwrap();
        let senses = pack.senses_of(bare).to_vec();
        assert_eq!(senses[0].sense.unwrap().get(), 2);
        assert_eq!(pack.visible_senses(bare, Language::English), senses);
        assert_eq!(pack.redundant_with(senses[0]), vec![senses[0]]);
        pack.sentences.redundant_senses.insert(
            pack.sentences.prefixed_text[&bare],
            vec![vec![senses[1], senses[0]]],
        );
        let standalone = TaggedGram {
            gram: bare,
            sense: NonZeroU32::new(3),
        };
        assert_eq!(
            pack.visible_senses(bare, Language::English),
            vec![senses[0], standalone]
        );
        assert_eq!(pack.redundant_with(senses[0]), vec![senses[1], senses[0]]);
        assert_eq!(pack.redundant_with(standalone), vec![standalone]);
        let untagged = TaggedGram {
            gram: bare,
            sense: None,
        };
        assert_eq!(pack.redundant_with(untagged), vec![untagged]);
        let LanguagePack { core, sentences } = pack;
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&sentences).unwrap();
        let sentences =
            rkyv::from_bytes::<LanguagePackSentences, rkyv::rancor::Error>(&bytes).unwrap();
        let core = LanguagePack::from_parts(core, None);
        let bare = core.intern_gram(&gram("bank")).unwrap();
        assert!(core.sentences.redundant_senses.is_empty());
        assert_eq!(
            core.visible_senses(bare, Language::English),
            core.senses_of(bare)
        );
        let full = core.with_sentences(sentences);
        assert_eq!(
            full.visible_senses(bare, Language::English),
            vec![full.senses_of(bare)[0], standalone]
        );
        assert_eq!(full.redundant_with(full.senses_of(bare)[0]).len(), 2);
    }

    #[test]
    fn ease_order_is_deterministic_and_independent_of_frequency_order() {
        let entries = [(3, 4.0), (2, 1.0), (1, 4.0)];
        let a = EaseOrder::new(entries.into_iter());
        let b = EaseOrder::new(entries.into_iter().rev());
        assert_eq!(a.iter_from(0).collect::<Vec<_>>(), vec![1, 3, 2]);
        assert_eq!(
            a.iter_from(0).collect::<Vec<_>>(),
            b.iter_from(0).collect::<Vec<_>>()
        );
        assert_eq!(a.rank(&1), Some(0));
        assert_eq!(a.rank(&4), None);
        assert_eq!(a.ease_at(1), Some(4.0));
        assert_eq!(a.ease_at(3), None);
        assert_eq!(a.partition_point(|ease| ease >= 4.0), 2);
        assert_eq!(a.partition_point(|ease| ease >= 5.0), 0);
        assert_eq!(a.partition_point(|ease| ease >= 0.0), 3);
        assert_eq!(a.iter_from(3).count(), 0);
        let empty = EaseOrder::<u32>::new(std::iter::empty());
        assert_eq!(empty.partition_point(|_| true), 0);
        assert_eq!(empty.iter_from(0).count(), 0);
    }

    #[test]
    fn stored_ease_orders_and_senses_agree_with_frequencies() {
        let pack = pack();
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&*pack.core).unwrap();
        let core = rkyv::from_bytes::<LanguagePackCore, rkyv::rancor::Error>(&bytes).unwrap();
        let loaded = LanguagePack::from_parts(core, None);
        for pack in [&pack, &loaded] {
            let bare = pack.intern_gram(&gram("bank")).unwrap();
            let (written, listening) = ease_orders(&pack.gram_frequencies);
            assert_eq!(pack.senses, sense_index(&pack.gram_frequencies));
            assert_eq!(pack.written_ease_order.entries, written.entries);
            assert_eq!(pack.written_ease_order.ranks, written.ranks);
            assert_eq!(pack.listening_ease_order.entries, listening.entries);
            assert_eq!(pack.listening_ease_order.ranks, listening.ranks);
            assert_eq!(pack.gram_frequency_total(bare).unwrap().count, 30);
            assert_eq!(pack.listening_ease_order.ease_at(0), Some(20_f32.ln()));
            assert_eq!(pack.written_ease_order.ease_at(0), Some(20_f32.ln()));
        }
    }

    #[test]
    fn listening_frequency_sums_all_sense_counts() {
        let mut pack = pack();
        let bare = pack.intern_gram(&gram("bank")).unwrap();
        let entry = pack.senses_of(bare)[1];
        Arc::get_mut(&mut pack.core)
            .unwrap()
            .gram_frequencies
            .entries
            .get_mut(&entry)
            .unwrap()
            .direct_count = 7;
        let total = pack.gram_frequency_total(bare).unwrap();
        assert_eq!(total.count, 30);
        assert_eq!(total.direct_count, 27);
        assert_eq!(
            total.ease,
            pack.gram_frequencies
                .entries
                .get(&pack.senses_of(bare)[0])
                .unwrap()
                .ease
        );
    }

    use super::*;
    use crate::{
        GramFrequencyEntry, GramFrequencyList, GramVocabEntry, Language, PronunciationData, Word,
    };
    use std::num::NonZeroU32;

    fn movie(id: &str, poster_bytes: Option<Vec<u8>>) -> MovieMetadata {
        MovieMetadata {
            id: id.into(),
            title: id.into(),
            year: None,
            original_language: None,
            rotten_tomatoes_score: None,
            variety: None,
            poster_bytes,
        }
    }

    #[test]
    fn packs_hide_movies_without_a_poster() {
        let movies: FxHashMap<_, _> = [
            ("visible".into(), movie("visible", Some(vec![1, 2, 3]))),
            ("hidden".into(), movie("hidden", None)),
        ]
        .into_iter()
        .collect();
        let loaded = LanguagePack::new(
            ConsolidatedLanguageData {
                movies,
                ..Default::default()
            },
            Course {
                target_language: Language::English,
                native_language: Language::French,
            },
        );
        assert_eq!(
            loaded.sentences.movies.keys().collect::<Vec<_>>(),
            vec!["visible"]
        );
    }

    fn gram(word: &str) -> Gram<String> {
        Gram(vec![Atom::Tok(Word {
            text: word.into(),
            word_type: WordType::Heteronym(Heteronym {
                word: word.into(),
                lemma: word.into(),
                pos: PartOfSpeech::Noun,
            }),
        })])
    }

    #[test]
    fn sentence_upgrade_shares_core_and_keeps_placement_pack_usable() {
        let LanguagePack { core, sentences } = pack();
        let core = LanguagePack::from_parts(core, None);
        let full = core.with_sentences(sentences);
        assert!(Arc::ptr_eq(&core.core, &full.core));
        assert_eq!(core.spur_space_fingerprint, full.spur_space_fingerprint);
        assert!(core.sentences.translations.is_empty());
        assert!(!full.sentences.translations.is_empty());
        let gram = core.intern_gram(&gram("bank")).unwrap();
        assert_eq!(full.senses_of(gram), core.senses_of(gram));
        for (spur, text) in core.string_rodeo.iter() {
            assert_eq!(full.string_rodeo.resolve(&spur), text);
        }
    }

    #[test]
    fn sentence_half_only_references_core_keys() {
        let tagged = |text: &str| TaggedGram {
            gram: gram(text),
            sense: None,
        };
        let frequency = |text: &str| GramFrequencyEntry {
            gram: tagged(text),
            count: 10,
            direct_count: 10,
            disambiguation_key: 0,
        };
        let encoded = |word: &str, obvious: bool| {
            let entry = crate::ReadingTaggedGram {
                sense: tagged(word),
                reading: Some("reading-only".into()),
            };
            SentenceGrams {
                grams: vec![if obvious {
                    SentenceGram::Obvious(entry)
                } else {
                    SentenceGram::Learnable(entry)
                }],
                capitalize_first: false,
                multiword_terms: vec![],
                low_confidence_multiword_terms: vec![],
            }
        };
        let data = ConsolidatedLanguageData {
            target_language_sentences: vec!["bank".into(), "corpus sentence".into()],
            translations: vec![
                (
                    "bank".into(),
                    vec!["bank".into(), "translation-only".into()],
                ),
                ("corpus sentence".into(), vec!["translation-only".into()]),
            ],
            gram_vocabulary: ["bank", "corpus-only", "source-only", "unused"]
                .into_iter()
                .map(|word| GramVocabEntry {
                    atoms: gram(word),
                    frequency: 10,
                })
                .collect(),
            gram_frequencies: GramFrequencyList {
                entries: vec![frequency("bank")],
                total_count: 10,
            },
            encoded_sentences: vec![
                ("bank".into(), encoded("bank", false)),
                ("corpus sentence".into(), encoded("corpus-only", true)),
            ],
            sentence_sources: vec![("source sentence".into(), SentenceSource::none())],
            source_gram_frequencies: [(
                crate::FrequencySourceId::Movie("film".into()),
                GramFrequencyList {
                    entries: vec![frequency("source-only")],
                    total_count: 10,
                },
            )]
            .into_iter()
            .collect(),
            homophone_practice: [(
                HomophoneWordPair {
                    word1: "bank".into(),
                    word2: "banc".into(),
                },
                HomophonePractice {
                    sentence_pairs: vec![crate::HomophoneSentencePair {
                        sentence1: "bank".into(),
                        sentence2: "homophone sentence".into(),
                    }],
                },
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        };
        let course = Course {
            target_language: Language::English,
            native_language: Language::French,
        };
        let repeated = LanguagePack::new(data.clone(), course);
        let generated = LanguagePack::new(data, course);
        assert_eq!(
            rkyv::to_bytes::<rkyv::rancor::Error>(&*generated.core)
                .unwrap()
                .as_slice(),
            rkyv::to_bytes::<rkyv::rancor::Error>(&*repeated.core)
                .unwrap()
                .as_slice(),
        );
        assert_eq!(
            rkyv::to_bytes::<rkyv::rancor::Error>(&generated.sentences)
                .unwrap()
                .as_slice(),
            rkyv::to_bytes::<rkyv::rancor::Error>(&repeated.sentences)
                .unwrap()
                .as_slice(),
        );
        let sentence = generated
            .sentences
            .sentence_rodeo
            .get("corpus sentence")
            .unwrap();
        assert_eq!(
            generated.sentences.sentence_rodeo.resolve(&sentence),
            "corpus sentence"
        );
        let original_literals = generated
            .sentence_to_literals(&sentence, Language::English)
            .unwrap();
        let LanguagePack { core, sentences } = generated;
        let fingerprint = core.spur_space_fingerprint;
        for shared in ["bank", "banc", "corpus-only", "source-only", "reading-only"] {
            assert!(
                core.string_rodeo.get(shared).is_some(),
                "missing shared string: {shared}"
            );
        }
        // Sentence texts and translations live only in the sentence half.
        for sentence_only in [
            "corpus sentence",
            "source sentence",
            "homophone sentence",
            "translation-only",
        ] {
            assert!(
                core.string_rodeo.get(sentence_only).is_none(),
                "sentence-half text in the core: {sentence_only}"
            );
        }
        for text in [
            "bank",
            "corpus sentence",
            "source sentence",
            "homophone sentence",
        ] {
            assert!(sentences.sentence_rodeo.get(text).is_some());
        }
        assert_eq!(sentences.translation_rodeo.len(), 2);
        let core_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&*core).unwrap();
        let sentence_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&sentences).unwrap();
        let core = rkyv::from_bytes::<LanguagePackCore, rkyv::rancor::Error>(&core_bytes).unwrap();
        let sentences =
            rkyv::from_bytes::<LanguagePackSentences, rkyv::rancor::Error>(&sentence_bytes)
                .unwrap();
        let core = LanguagePack::from_parts(core, None);
        let full = core.with_sentences(sentences);
        assert_eq!(full.spur_space_fingerprint, fingerprint);
        assert!(core.sentences.sentence_rodeo.is_empty());
        assert!(core.sentences.translation_rodeo.is_empty());
        let SentenceGram::Obvious(entry) = &full.sentences.encoded_sentences[&sentence].grams[0]
        else {
            panic!("sentence-only gram must remain obvious after assembly")
        };
        assert_eq!(full.resolve_gram(&entry.sense.gram), gram("corpus-only"));
        assert_eq!(
            full.string_rodeo.resolve(&entry.reading.unwrap()),
            "reading-only"
        );
        assert_eq!(
            full.sentence_to_literals(&sentence, Language::English)
                .unwrap(),
            original_literals
        );
        for (sentence, translations) in &full.sentences.translations {
            assert!(
                full.sentences
                    .sentence_rodeo
                    .try_resolve(sentence)
                    .is_some()
            );
            for translation in translations {
                assert!(
                    full.sentences
                        .translation_rodeo
                        .try_resolve(translation)
                        .is_some()
                );
            }
        }
        for (words, practice) in &full.sentences.homophone_practice {
            assert_eq!(words.resolve(&full.string_rodeo).word2, "banc");
            assert_eq!(
                practice
                    .resolve(&full.sentences.sentence_rodeo)
                    .sentence_pairs[0]
                    .sentence2,
                "homophone sentence"
            );
        }
        assert_eq!(
            full.sentences.encoded_sentences,
            repeated.sentences.encoded_sentences
        );
        assert_eq!(full.sentences.translations, repeated.sentences.translations);
        assert_eq!(
            full.sentences.sentence_sources,
            repeated.sentences.sentence_sources
        );
        assert!(Arc::ptr_eq(&core.core, &full.core));
    }

    #[test]
    #[should_panic(expected = "language pack halves are from different builds")]
    fn sentence_upgrade_rejects_mismatched_core() {
        let LanguagePack {
            core,
            mut sentences,
        } = pack();
        sentences.spur_space_fingerprint ^= 1;
        LanguagePack::from_parts(core, None).with_sentences(sentences);
    }

    #[test]
    fn ruby_segmentations_live_only_in_the_sentence_half() {
        let mut pack = pack();
        let expected = BTreeMap::from([(
            ("東 京".into(), "とうきょう".into()),
            vec![crate::readings::RubySegment {
                base: "東".into(),
                ruby: Some("とう".into()),
            }],
        )]);
        pack.sentences.ruby_segmentations = expected.clone();
        let LanguagePack { core, sentences } = pack;
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&sentences).unwrap();
        let sentences =
            rkyv::from_bytes::<LanguagePackSentences, rkyv::rancor::Error>(&bytes).unwrap();
        let core_only = LanguagePack::from_parts(core, None);
        assert!(core_only.sentences.ruby_segmentations.is_empty());
        assert_eq!(
            core_only
                .with_sentences(sentences)
                .sentences
                .ruby_segmentations,
            expected
        );
    }

    fn pack() -> LanguagePack {
        pack_with_senses(&[(2, 20), (1, 10)])
    }

    /// The "bank" fixture with these (sense, count) frequency entries;
    /// sense 0 is the untagged entry.
    fn pack_with_senses(senses: &[(u32, u32)]) -> LanguagePack {
        pack_with_surface(senses, "bank")
    }
    fn pack_with_surface(senses: &[(u32, u32)], surface: &str) -> LanguagePack {
        let bare = gram("bank");
        let tagged = |sense| TaggedGram {
            gram: bare.clone(),
            sense: NonZeroU32::new(sense),
        };
        LanguagePack::new(
            ConsolidatedLanguageData {
                redundant_senses: BTreeMap::new(),
                strokes: crate::StrokeTable::from_iter([(
                    "一".into(),
                    vec![crate::StrokeGlyph {
                        standard: crate::StrokeStandard::Japan,
                        strokes: vec![crate::Stroke {
                            points: vec![(0.1, 0.5), (0.9, 0.5)],
                        }],
                    }],
                )]),
                target_language_sentences: vec![surface.into()],
                translations: vec![(surface.into(), vec!["banque".into()])],
                nlp_sentences: vec![],
                phrasebook: BTreeMap::new(),
                proper_noun_definitions: BTreeMap::new(),
                source_gram_frequencies: FxHashMap::default(),
                word_to_pronunciation: vec![],
                pronunciation_to_words: vec![],
                minimal_pairs: vec![],
                pronunciation_data: PronunciationData {
                    sounds: vec![],
                    guides: vec![],
                    pattern_frequencies: vec![],
                },
                homophone_practice: BTreeMap::new(),
                movies: FxHashMap::default(),
                books: FxHashMap::default(),
                sentence_sources: vec![],
                gram_vocabulary: [bare.clone(), gram(surface)]
                    .into_iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .into_iter()
                    .map(|atoms| GramVocabEntry {
                        atoms,
                        frequency: 30,
                    })
                    .collect(),
                gram_frequencies: GramFrequencyList {
                    entries: senses
                        .iter()
                        .map(|&(sense, count)| GramFrequencyEntry {
                            count,
                            direct_count: count,
                            disambiguation_key: 0,
                            gram: tagged(sense),
                        })
                        .collect(),
                    total_count: senses.iter().map(|&(_, count)| u64::from(count)).sum(),
                },
                encoded_sentences: vec![(
                    surface.into(),
                    SentenceGrams {
                        grams: vec![SentenceGram::Learnable(crate::ReadingTaggedGram {
                            sense: TaggedGram {
                                gram: gram(surface),
                                sense: NonZeroU32::new(1),
                            },
                            reading: Some("ばんく".into()),
                        })],
                        capitalize_first: false,
                        multiword_terms: vec![crate::MultiwordTermMatch {
                            gram: crate::ReadingTaggedGram {
                                sense: tagged(0),
                                reading: Some("ぎんこう".into()),
                            },
                            matched_word_indices: vec![0],
                        }],
                        low_confidence_multiword_terms: vec![crate::MultiwordTermMatch {
                            gram: crate::ReadingTaggedGram {
                                sense: tagged(2),
                                reading: Some("ていぼう".into()),
                            },
                            matched_word_indices: vec![0],
                        }],
                    },
                )],
                gram_dictionary: BTreeMap::new(),
                morphemes: BTreeMap::new(),
                human_audio: FxHashMap::default(),
                pronunciation_audio: FxHashMap::default(),
            },
            Course {
                target_language: Language::English,
                native_language: Language::French,
            },
        )
    }

    fn assert_resolution(pack: &LanguagePack) {
        let request = |word, sense| TaggedGram {
            gram: gram(word),
            sense: NonZeroU32::new(sense),
        };
        let exact = pack.resolve_entry(&request("bank", 1)).unwrap();
        assert_eq!(exact.sense, NonZeroU32::new(1));
        for sense in [0, 99] {
            assert_eq!(
                pack.resolve_entry(&request("bank", sense)).unwrap().sense,
                NonZeroU32::new(2)
            );
        }
        assert_eq!(
            pack.senses_of(exact.gram)
                .iter()
                .map(|g| g.sense.unwrap().get())
                .collect::<Vec<_>>(),
            [2, 1]
        );
        assert!(pack.resolve_entry(&request("foreign", 1)).is_none());
    }

    #[test]
    fn entry_resolution_fallbacks() {
        assert_resolution(&pack());
    }

    /// Regenerating a pack retires sense ids: a card written with an id that
    /// no longer exists must reach the gram's most frequent current sense.
    #[test]
    fn retired_sense_ids_fall_back_to_the_current_most_frequent_sense() {
        let card = |sense| TaggedGram {
            gram: gram("bank"),
            sense: NonZeroU32::new(sense),
        };
        // The gram became monosemous: only the untagged entry is left.
        let monosemous = pack_with_senses(&[(0, 30)]);
        for sense in [1, 2] {
            assert_eq!(monosemous.resolve_entry(&card(sense)).unwrap().sense, None);
        }
        // Every new sense id lies above the retired ones. (Packs list
        // frequencies most frequent first.)
        let renumbered = pack_with_senses(&[(5, 20), (4, 10)]);
        for sense in [0, 1, 2] {
            assert_eq!(
                renumbered.resolve_entry(&card(sense)).unwrap().sense,
                NonZeroU32::new(5)
            );
        }
        assert_eq!(
            renumbered.resolve_entry(&card(4)).unwrap().sense,
            NonZeroU32::new(4)
        );
    }

    #[test]
    fn strokes_live_in_sentences_and_round_trip() {
        let pack = pack();
        let strokes = pack.sentences.strokes.clone();
        assert!(!strokes.is_empty());
        let LanguagePack { core, sentences } = pack;
        assert_eq!(sentences.strokes, strokes);
        let core_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&*core).unwrap();
        let sentence_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&sentences).unwrap();
        let core = rkyv::from_bytes::<LanguagePackCore, rkyv::rancor::Error>(&core_bytes).unwrap();
        let sentences =
            rkyv::from_bytes::<LanguagePackSentences, rkyv::rancor::Error>(&sentence_bytes)
                .unwrap();
        let pack = LanguagePack::from_parts(core, Some(sentences));
        assert_eq!(pack.sentences.strokes, strokes);
        let core = pack.core;
        assert!(
            LanguagePack::from_parts(core, None)
                .sentences
                .strokes
                .is_empty()
        );
    }

    #[test]
    fn phrase_readings_are_interned_and_the_index_survives_serialization() {
        let pack = pack();
        assert_eq!(pack.sentences.encoded_sentences.len(), 1);
        let resolved = |pack: &LanguagePack| {
            pack.sentences
                .gram_readings
                .iter()
                .map(|(gram, readings)| {
                    (
                        TaggedGram {
                            gram: pack.resolve_gram(&gram.gram),
                            sense: gram.sense,
                        },
                        readings
                            .iter()
                            .map(|r| pack.string_rodeo.resolve(r).to_owned())
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<BTreeMap<_, _>>()
        };
        let expected = resolved(&pack);
        assert_eq!(expected.len(), 3);
        let LanguagePack { core, sentences } = pack;
        for reading in ["ばんく", "ぎんこう", "ていぼう"] {
            assert!(core.string_rodeo.get(reading).is_some());
        }
        let core_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&*core).unwrap();
        let sentence_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&sentences).unwrap();
        let core = rkyv::from_bytes::<LanguagePackCore, rkyv::rancor::Error>(&core_bytes).unwrap();
        let sentences =
            rkyv::from_bytes::<LanguagePackSentences, rkyv::rancor::Error>(&sentence_bytes)
                .unwrap();
        let pack = LanguagePack::from_parts(core, Some(sentences));
        assert_eq!(resolved(&pack), expected);
        let sentence =
            &pack.sentences.encoded_sentences[&pack.sentences.sentence_rodeo.get("bank").unwrap()];
        for (terms, reading) in [
            (&sentence.multiword_terms, "ぎんこう"),
            (&sentence.low_confidence_multiword_terms, "ていぼう"),
        ] {
            assert_eq!(terms.len(), 1);
            assert_eq!(
                pack.readings_for_sense(terms[0].gram.sense)
                    .collect::<Vec<_>>(),
                [reading]
            );
        }
    }

    #[test]
    fn loading_uses_the_stored_reading_index_without_scanning_sentences() {
        let LanguagePack {
            core,
            mut sentences,
        } = pack();
        let expected = sentences.gram_readings.clone();
        assert!(!expected.is_empty());
        sentences.encoded_sentences.clear();
        let pack = LanguagePack::from_parts(core, Some(sentences));
        assert_eq!(pack.sentences.gram_readings, expected);
    }

    #[test]
    fn inflected_phrase_readings_are_not_canonical_headword_readings() {
        let pack = pack_with_surface(&[(0, 30), (1, 20), (2, 10)], "banked");
        assert_eq!(pack.sentences.gram_readings.len(), 1);
        let sentence = pack.sentences.encoded_sentences.values().next().unwrap();
        assert!(sentence.multiword_terms[0].gram.reading.is_some());
        assert!(
            sentence.low_confidence_multiword_terms[0]
                .gram
                .reading
                .is_some()
        );
        assert!(
            !pack
                .sentences
                .gram_readings
                .contains_key(&sentence.multiword_terms[0].gram.sense)
        );
        assert!(
            !pack
                .sentences
                .gram_readings
                .contains_key(&sentence.low_confidence_multiword_terms[0].gram.sense)
        );
    }
    #[test]
    fn reading_frequency_includes_phrases_and_breaks_ties_lexically() {
        let mut pack = pack();
        let sentence = pack
            .sentences
            .encoded_sentences
            .values_mut()
            .next()
            .unwrap();
        let SentenceGram::Learnable(entry) = sentence.grams[0] else {
            panic!()
        };
        sentence.multiword_terms[0].gram.sense = entry.sense;
        sentence.low_confidence_multiword_terms[0].gram.sense = entry.sense;
        sentence
            .multiword_terms
            .push(sentence.multiword_terms[0].clone());
        let readings = derive_readings(
            &pack.sentences.encoded_sentences,
            &pack.string_rodeo,
            &pack.gram_rodeo,
        );
        assert_eq!(
            readings[&entry.sense]
                .iter()
                .map(|r| pack.string_rodeo.resolve(r))
                .collect::<Vec<_>>(),
            ["ぎんこう", "ていぼう", "ばんく"]
        );
    }

    #[test]
    fn tagged_sentence_split_round_trip() {
        let LanguagePack { core, sentences } = pack();
        let core_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&*core).unwrap();
        let sentence_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&sentences).unwrap();
        let core = rkyv::from_bytes::<LanguagePackCore, rkyv::rancor::Error>(&core_bytes).unwrap();
        let sentences =
            rkyv::from_bytes::<LanguagePackSentences, rkyv::rancor::Error>(&sentence_bytes)
                .unwrap();
        let pack = LanguagePack::from_parts(core, Some(sentences));
        assert_resolution(&pack);
        let sentence = pack.sentences.sentence_rodeo.get("bank").unwrap();
        let SentenceGram::Learnable(entry) = pack.sentences.encoded_sentences[&sentence].grams[0]
        else {
            panic!("expected a learnable gram")
        };
        assert_eq!(entry.sense.sense, NonZeroU32::new(1));
        assert_eq!(pack.resolve_gram(&entry.sense.gram), gram("bank"));
        assert_eq!(pack.string_rodeo.resolve(&entry.reading.unwrap()), "ばんく");
        assert_eq!(
            pack.readings_for_sense(entry.sense).collect::<Vec<_>>(),
            ["ばんく"]
        );
        let core = pack.core;
        assert_resolution(&LanguagePack::from_parts(core, None));
    }
}
