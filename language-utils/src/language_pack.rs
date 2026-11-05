use crate::indexmap::IndexMap;
use crate::minimal_pairs::MinimalPairs;
use crate::sentence_codec::{self, GramKey, Symbol};
use crate::{
    Atom, Audio, BookMetadata, Course, Frequency, Gram, GramDefinition, Heteronym,
    HomophonePractice, HomophoneWordPair, Lexeme, Literal, MorphemeInfo, MorphemeSegment,
    MovieMetadata, PatternPosition, PronunciationClip, PronunciationData, PronunciationGuide,
    ProperNounDefinition, SentenceGram, SentenceGrams, SpurGram, SpurReadingGram, TaggedGram,
    VoiceActor, WordType,
};
use lasso::{Key, Spur};
use rustc_hash::FxHashMap;
use std::collections::BTreeMap;
use std::hash::Hash;
use std::sync::Arc;

/// Sentence identities are independent of shared vocabulary strings.
#[derive(Debug)]
pub enum SentenceText {}
pub type SentenceSpur = Spur<SentenceText>;

/// Native translation identities are independent of sentence and vocabulary keys.
#[derive(Debug)]
pub enum TranslationText {}
pub type TranslationSpur = Spur<TranslationText>;

/// Text stored in the core's definition table, independent of sentence data.
#[derive(Debug)]
pub enum DefinitionText {}
pub type DefinitionTextSpur = Spur<DefinitionText>;

#[derive(Debug)]
pub enum DefinitionList {}
#[derive(Debug)]
pub enum MorphologyList {}
#[derive(Debug)]
pub enum SegmentList {}
pub type DefinitionListSpur = Spur<DefinitionList>;
pub type MorphologyListSpur = Spur<MorphologyList>;
pub type SegmentListSpur = Spur<SegmentList>;
pub type DefinitionLists =
    lasso::RodeoResolver<Vec<crate::TargetToNativeWord<DefinitionTextSpur>>, DefinitionListSpur>;
pub type DefinitionMorphologies =
    lasso::RodeoResolver<Vec<crate::features::Morphology>, MorphologyListSpur>;
pub type DefinitionSegments =
    lasso::RodeoResolver<Vec<MorphemeSegment<DefinitionTextSpur>>, SegmentListSpur>;
pub type StoredGramDefinition =
    GramDefinition<DefinitionTextSpur, DefinitionListSpur, MorphologyListSpur, SegmentListSpur>;
pub type DefinitionMetadata<'a> = GramDefinition<
    DefinitionTextSpur,
    &'a [crate::TargetToNativeWord<DefinitionTextSpur>],
    &'a [crate::features::Morphology],
    &'a [MorphemeSegment<DefinitionTextSpur>],
>;

/// Variable-length lists indexed by dense sentence keys: N+1 offsets into
/// one flat array, with no per-sentence allocation or hash table.
#[derive(Debug, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct SentenceLists<T> {
    starts: Vec<u32>,
    keys: Vec<T>,
}

impl<T> Default for SentenceLists<T> {
    fn default() -> Self {
        Self {
            starts: vec![0],
            keys: Vec::new(),
        }
    }
}

impl<T> SentenceLists<T> {
    pub fn new(rows: impl IntoIterator<Item = impl IntoIterator<Item = T>>) -> Self {
        let mut starts = vec![0];
        let mut keys = Vec::new();
        for row in rows {
            keys.extend(row);
            starts.push(u32::try_from(keys.len()).unwrap());
        }
        Self { starts, keys }
    }

    pub fn get(&self, sentence: SentenceSpur) -> &[T] {
        let i = sentence.into_usize();
        &self.keys[self.starts[i] as usize..self.starts[i + 1] as usize]
    }
}

/// IDs are interned separately from metadata: some corpus sources have no
/// visible movie or reader-book entry, but their provenance must survive.
#[derive(Debug, Default, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct SourceIds {
    pub ids: Vec<String>,
    pub sentences: SentenceLists<u16>,
}

impl SourceIds {
    fn get(&self, sentence: SentenceSpur) -> impl Iterator<Item = &String> {
        self.sentences
            .get(sentence)
            .iter()
            .map(|&i| &self.ids[i as usize])
    }
}

#[derive(Debug, Default, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct SentenceSources {
    pub flags: Vec<u8>,
    pub movies: SourceIds,
    pub books: SourceIds,
}

impl SentenceSources {
    fn get(&self, sentence: SentenceSpur) -> SentenceSourceRef<'_> {
        let flags = self.flags[sentence.into_usize()];
        SentenceSourceRef {
            from_anki: flags & 1 != 0,
            from_tatoeba: flags & 2 != 0,
            from_manual: flags & 4 != 0,
            from_song: flags & 8 != 0,
            sources: self,
            sentence,
        }
    }
}

pub struct SentenceSourceRef<'a> {
    pub from_anki: bool,
    pub from_tatoeba: bool,
    pub from_manual: bool,
    pub from_song: bool,
    sources: &'a SentenceSources,
    sentence: SentenceSpur,
}

impl<'a> SentenceSourceRef<'a> {
    pub fn movie_ids(&self) -> impl Iterator<Item = &'a String> {
        self.sources.movies.get(self.sentence)
    }

    pub fn book_ids(&self) -> impl Iterator<Item = &'a String> {
        self.sources.books.get(self.sentence)
    }
}

/// Build-time index, easiest first (descending ease, then ascending key),
/// independent of the pack's frequency ordering.
#[derive(Clone, Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct EaseOrder<K: Eq + Hash> {
    pub entries: Vec<(K, f32)>,
    pub ranks: FxHashMap<K, u32>,
}

impl<K: Copy + Eq + Hash> EaseOrder<K> {
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

/// A frequency list with its total count (interned version for the language pack).
#[derive(Clone, Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct FrequencyList<K: Clone + Eq + Hash = GramKey> {
    pub entries: IndexMap<K, Frequency>,
    /// Total gram count from unfiltered data (for accurate percentage calculations)
    pub total_count: u64,
}

impl FrequencyList<SpurReadingGram> {
    pub fn sense_entries<'a>(
        entries: impl Iterator<Item = (&'a SpurReadingGram, &'a Frequency)>,
    ) -> IndexMap<TaggedGram<SpurGram>, Frequency> {
        let mut senses: IndexMap<TaggedGram<SpurGram>, Frequency> = IndexMap::new();
        for (key, frequency) in entries {
            senses
                .entry(key.sense)
                .and_modify(|total| {
                    total.count += frequency.count;
                    total.direct_count += frequency.direct_count;
                    total.ease = total.ease.max(frequency.ease);
                })
                .or_insert(*frequency);
        }
        let mut senses: Vec<_> = senses
            .iter()
            .map(|(key, frequency)| (*key, *frequency))
            .collect();
        // Keep the canonical list's order for ties, including dictionary indices.
        senses.sort_by_key(|(_, frequency)| std::cmp::Reverse(frequency.count));
        let mut sense_entries = IndexMap::new();
        for (key, frequency) in senses {
            sense_entries.insert(key, frequency);
        }
        sense_entries
    }
}

/// Count-ordered source frequencies. Separate arrays avoid padding each u8
/// count to the alignment of a GramKey; all other frequency data is global.
#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct SourceFrequencyList {
    pub keys: Vec<GramKey>,
    pub counts: Vec<u8>,
    pub total_count: u64,
}

impl SourceFrequencyList {
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn resolved<'a>(
        &'a self,
        pack: &'a LanguagePack,
    ) -> impl DoubleEndedIterator<Item = (&'a SpurReadingGram, Frequency)> {
        self.keys.iter().zip(&self.counts).map(|(key, &count)| {
            (
                pack.reading_gram(*key),
                Frequency {
                    count: u32::from(count),
                    ..*pack.gram_frequencies.entries.get(key).unwrap()
                },
            )
        })
    }
}

/// The written order owns each reading-tagged gram once. Learnable keys form
/// the prefix whose indices are ease ranks; sentence-only keys follow it.
#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct WrittenEaseOrder {
    pub grams: lasso::RodeoReader<SpurReadingGram, GramKey>,
    pub ease: Vec<f32>,
}

impl WrittenEaseOrder {
    pub fn rank(&self, gram: &SpurReadingGram) -> Option<u32> {
        let key = self.grams.get(gram)?;
        (key.into_usize() < self.ease.len()).then_some(key.index())
    }
    pub fn len(&self) -> usize {
        self.ease.len()
    }
    pub fn is_empty(&self) -> bool {
        self.ease.is_empty()
    }
    pub fn ease_at(&self, rank: u32) -> Option<f32> {
        self.ease.get(rank as usize).copied()
    }
    pub fn iter_from(&self, rank: u32) -> impl Iterator<Item = SpurReadingGram> + '_ {
        (rank as usize..self.ease.len())
            .map(|i| *self.grams.resolve(&GramKey::try_from_usize(i).unwrap()))
    }
    pub fn partition_point(&self, mut predicate: impl FnMut(f32) -> bool) -> u32 {
        self.ease.partition_point(|ease| predicate(*ease)) as u32
    }
}

impl FrequencyList {
    pub fn resolved<'a>(
        &'a self,
        pack: &'a LanguagePack,
    ) -> impl DoubleEndedIterator<Item = (&'a SpurReadingGram, &'a Frequency)> {
        self.entries
            .iter()
            .map(|(key, frequency)| (pack.reading_gram(*key), frequency))
    }
}

/// The assembled runtime pack. Never serialized itself — the on-disk/wire
/// format is the [`LanguagePackCore`] + [`LanguagePackSentences`] pair, put
/// back together by [`LanguagePack::from_parts`].
#[derive(Debug)]
pub struct LanguagePack {
    pub sense_frequencies: IndexMap<TaggedGram<SpurGram>, Frequency>,
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
            .filter_map(|entry| self.sense_frequencies.get(entry));
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
        self.sense_frequencies
            .contains_key(&requested)
            .then_some(requested)
            .or_else(|| self.senses_of(gram).first().copied())
    }

    /// Resolve definition text only when a caller needs a display/UI value.
    pub fn gram_definition(&self, gram: &TaggedGram<SpurGram>) -> Option<GramDefinition<String>> {
        self.definition_metadata(gram)
            .map(|definition| definition.map_text(|text, _| self.definition_text(text)))
    }

    /// Borrow inline flags and interned lists; this never decodes or allocates text.
    pub fn definition_metadata(
        &self,
        gram: &TaggedGram<SpurGram>,
    ) -> Option<DefinitionMetadata<'_>> {
        self.gram_definitions.get(gram).map(|definition| {
            definition.clone().map_lists(
                |key| self.definition_lists.resolve(&key),
                |key| self.definition_morphology.resolve(&key),
                |key| self.definition_segments.resolve(&key),
            )
        })
    }

    pub fn definition_text(&self, text: &DefinitionTextSpur) -> String {
        self.definition_texts.resolve_string(text)
    }

    /// Canonical front prefix: the most frequent sense, independent of deck eligibility.
    pub fn word_prefix(
        &self,
        gram: SpurGram,
        language: crate::Language,
    ) -> Option<crate::features::WordPrefix> {
        let definition = self.definition_metadata(self.senses_of(gram).first()?)?;
        crate::compute_word_prefix(&self.resolve_gram(&gram), &definition, language)
    }

    pub fn reading_gram(&self, key: GramKey) -> &SpurReadingGram {
        self.written_ease_order.grams.resolve(&key)
    }

    pub fn reading_keys(&self, sense: &TaggedGram<SpurGram>) -> &[GramKey] {
        self.reading_keys
            .get(sense)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub fn reading_ranks(
        &self,
        sense: &TaggedGram<SpurGram>,
    ) -> impl ExactSizeIterator<Item = (SpurReadingGram, u32)> + '_ {
        self.reading_keys(sense)
            .iter()
            .map(|key| (*self.reading_gram(*key), key.index()))
    }

    /// Exact lookup of already-cleaned text, verified against the decoded row.
    pub fn sentence_key(&self, text: &str) -> Option<SentenceSpur> {
        self.sentences
            .encoded_sentences
            .get(&self.render_context(), text)
    }

    fn render_context(&self) -> crate::sentence_text::RenderContext<'_> {
        crate::sentence_text::RenderContext {
            grams: &self.written_ease_order.grams,
            gram_rodeo: &self.gram_rodeo,
            strings: &self.string_rodeo,
            language: self.sentences.target_language,
        }
    }

    pub fn sentence_text(&self, key: SentenceSpur) -> String {
        let mut text = String::new();
        let _ = crate::sentence_text::render(
            &self.render_context(),
            self.sentences.encoded_sentences.resolve(&key).copied(),
            |piece| {
                text.push_str(piece);
                std::ops::ControlFlow::Continue(())
            },
        );
        text
    }

    pub fn sentence_keys(&self) -> impl Iterator<Item = SentenceSpur> + '_ {
        (0..self.sentences.encoded_sentences.len())
            .map(|i| SentenceSpur::try_from_usize(i).unwrap())
    }

    pub fn translations_of(&self, sentence: SentenceSpur) -> &[TranslationSpur] {
        self.sentences.translations.get(sentence)
    }

    pub fn source_of(&self, sentence: SentenceSpur) -> SentenceSourceRef<'_> {
        self.sentences.sentence_sources.get(sentence)
    }

    pub fn decoded_sentences(
        &self,
    ) -> impl Iterator<Item = (SentenceSpur, SentenceGrams<SpurReadingGram>)> + '_ {
        self.sentence_keys()
            .map(|key| (key, self.sentence_grams(key).unwrap()))
    }

    pub fn sentence_grams(&self, sentence: SentenceSpur) -> Option<SentenceGrams<SpurReadingGram>> {
        Some(sentence_codec::decode(
            self.sentences
                .encoded_sentences
                .try_resolve(&sentence)?
                .copied(),
            &self.written_ease_order.grams,
        ))
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
            .min_by_key(|member| self.sense_frequencies.get_index_of(member))
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
        self.sense_frequencies.contains_key(gram)
    }

    /// Resolve an interned gram back to its owned string form.
    pub fn resolve_gram(&self, gram: &SpurGram) -> Gram<String> {
        self.gram_rodeo.resolve(gram).resolve(&self.string_rodeo)
    }

    /// Sentences containing the required sense (if any) whose other learnable
    /// symbols pass the caller's rank-bitset test. Translations are a build invariant.
    pub fn comprehensible_sentences(
        &self,
        required_gram: Option<&TaggedGram<SpurGram>>,
        known_rank: impl Fn(u32) -> bool,
    ) -> Vec<SentenceSpur> {
        let required_keys = required_gram.map_or(&[][..], |gram| self.reading_keys(gram));
        // Preserve the gram index's candidate order; full-corpus searches use slot order.
        let candidate_sentences: &[SentenceSpur] = if let Some(required) = required_gram {
            match self.sentences.sentences_containing_gram_index.get(required) {
                Some(sentences) => sentences,
                None => return Vec::new(),
            }
        } else {
            &self.sentence_keys().collect::<Vec<_>>()
        };
        candidate_sentences
            .iter()
            .copied()
            .filter(|&sentence| {
                self.sentence_is_comprehensible(sentence, required_keys, &known_rank)
            })
            .collect()
    }

    /// Decode lazily, stopping at the first unknown learnable symbol. The symbol
    /// itself carries its rank: no gram resolution, hashing, or grammar allocation.
    pub fn sentence_is_comprehensible(
        &self,
        sentence: SentenceSpur,
        required_keys: &[GramKey],
        known_rank: &impl Fn(u32) -> bool,
    ) -> bool {
        let Some(mut symbols) = self.sentences.encoded_sentences.try_resolve(&sentence) else {
            return false;
        };
        symbols.all(|symbol| {
            !symbol.is_learnable()
                || known_rank(symbol.key().index())
                || required_keys.contains(&symbol.key())
        })
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
            self.sentence_grams(*sentence)?
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
        match self.definition_metadata(gram)? {
            GramDefinition::Dictionary(d) => d
                .definitions
                .first()
                .map(|td| self.definition_text(&td.native)),
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
        let dict_entry = match self.gram_definition(gram)? {
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
}

/// The word-level half of a language pack: dictionary, frequency,
/// pronunciation and morphology data, plus all shared strings and grams.
/// Downloaded first so the placement test can run before sentence data arrives.
///
/// The core owns every shared string and gram referenced by either half.
/// Sentence texts and native translations have independent typed interners in
/// [`LanguagePackSentences`]. The halves must come from the same
/// pack-builder invocation and be versioned and shipped as one unit.
#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct LanguagePackCore {
    /// Fingerprint of this core's string, gram and reading-key spaces.
    /// [`LanguagePack::from_parts`] refuses a sentences half whose
    /// fingerprint differs, so halves from different builds can't be
    /// silently combined.
    pub spur_space_fingerprint: u64,
    pub string_rodeo: lasso::RodeoReader,
    pub gram_rodeo: lasso::RodeoReader<Gram<Spur>>,
    pub written_ease_order: WrittenEaseOrder,
    pub listening_ease_order: EaseOrder<SpurGram>,
    pub reading_keys: FxHashMap<TaggedGram<SpurGram>, Vec<GramKey>>,
    pub senses: FxHashMap<SpurGram, Vec<TaggedGram<SpurGram>>>,
    pub words_to_heteronyms: FxHashMap<Spur, Vec<Heteronym<Spur>>>,
    pub word_to_pronunciation: FxHashMap<Spur, Spur>,
    pub pronunciation_to_words: FxHashMap<Spur, Vec<Spur>>,
    pub minimal_pairs: MinimalPairs,
    pub pronunciation_data: PronunciationData,
    pub pattern_frequency_map: FxHashMap<(Spur, PatternPosition), u32>,
    pub pronunciation_max_freq_cache: FxHashMap<Spur, Frequency>,
    pub proper_noun_definitions: BTreeMap<Spur, ProperNounDefinition>,
    pub gram_frequencies: FrequencyList,
    pub definition_texts:
        lasso::CompressedRodeoReader<DefinitionTextSpur, lasso::RunTable, lasso::Text>,
    pub gram_definitions: FxHashMap<TaggedGram<SpurGram>, StoredGramDefinition>,
    pub definition_lists: DefinitionLists,
    pub definition_morphology: DefinitionMorphologies,
    pub definition_segments: DefinitionSegments,
    pub heteronym_to_grams: FxHashMap<Heteronym<Spur>, Vec<SpurGram>>,
    pub string_to_grams: FxHashMap<String, Vec<TaggedGram<SpurGram>>>,
    pub morphemes: FxHashMap<MorphemeSegment<Spur>, MorphemeInfo<Spur>>,
}

/// The sentence half of a language pack: sentence texts, translations,
/// per-sentence gram encodings and indexes, source metadata and human audio.
/// See [`LanguagePackCore`] for the spur-space contract between the halves.
#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct LanguagePackSentences {
    /// Canonical phrase reading first, then unique token readings ordered by
    /// observed frequency (lexical ties); computed at build time.
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
    pub target_language: crate::Language,
    pub translation_rodeo:
        lasso::CompressedRodeoReader<TranslationSpur, lasso::RunTable, lasso::Text>,
    pub translations: SentenceLists<TranslationSpur>,
    pub source_gram_frequencies: FxHashMap<crate::FrequencySourceId, SourceFrequencyList>,
    pub homophone_practice: FxHashMap<HomophoneWordPair<Spur>, HomophonePractice<SentenceSpur>>,
    pub movies: FxHashMap<String, MovieMetadata>,
    pub books: FxHashMap<String, BookMetadata>,
    pub sentence_sources: SentenceSources,
    pub encoded_sentences: lasso::CompressedRodeoReader<SentenceSpur, Vec<Symbol>, SentenceText>,
    pub sentences_containing_gram_index: FxHashMap<TaggedGram<SpurGram>, Vec<SentenceSpur>>,
    pub human_audio: FxHashMap<VoiceActor, FxHashMap<String, Audio>>,
    pub pronunciation_audio: FxHashMap<String, PronunciationClip>,
}

impl LanguagePackSentences {
    fn empty(spur_space_fingerprint: u64) -> Self {
        Self {
            spur_space_fingerprint,
            target_language: crate::Language::English,
            translation_rodeo: lasso::CompressedRodeoReader::default(),
            gram_readings: FxHashMap::default(),
            ruby_segmentations: BTreeMap::new(),
            redundant_senses: FxHashMap::default(),
            prefixed_text: FxHashMap::default(),
            grams_with_prefixed_text: FxHashMap::default(),
            strokes: crate::StrokeTable::default(),
            translations: SentenceLists::new(std::iter::empty::<Vec<TranslationSpur>>()),
            source_gram_frequencies: FxHashMap::default(),
            homophone_practice: FxHashMap::default(),
            movies: FxHashMap::default(),
            books: FxHashMap::default(),
            sentence_sources: SentenceSources::default(),
            encoded_sentences: lasso::CompressedRodeoReader::default(),
            sentences_containing_gram_index: FxHashMap::default(),
            human_audio: FxHashMap::default(),
            pronunciation_audio: FxHashMap::default(),
        }
    }
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
        let sense_frequencies = FrequencyList::sense_entries(
            core.gram_frequencies
                .entries
                .iter()
                .map(|(key, frequency)| (core.written_ease_order.grams.resolve(key), frequency)),
        );
        Self {
            core,
            sentences,
            sense_frequencies,
        }
    }

    /// Add the sentence half without deserializing the core again.
    pub fn with_sentences(&self, sentences: LanguagePackSentences) -> Self {
        Self::from_parts(self.core.clone(), Some(sentences))
    }
}

/// Filenames of the two halves of a split language pack, relative to a
/// course's data directory. Shared by the generator, the
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
/// course directory, keeping the on-disk layout in one place.
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
