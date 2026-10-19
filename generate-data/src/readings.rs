//! Dictionary candidates -> persistent sense/reading hierarchies -> occurrence labels.
//! Corpus grams share one hierarchy; occurrence labels never change word identity.
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::num::NonZeroU32;

use anyhow::{Result, ensure};
use language_utils::readings::{is_ruby_base, supports_readings};
use language_utils::{
    Atom, Gram, GramInterners, Language, Literal, ReadingTaggedGram, SentenceGrams, SentenceInfo,
    TaggedGram, WordType,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tysm::chat_completions::{ChatClient, IndividualChatError};

use crate::usage_discovery::{UsageAnchor, UsageEntry, UsageInventory, UsageKind};
mod canonical;
mod corrections;
mod coverings;
mod hierarchy;
mod inflections;
mod inventory;
mod kanjidic;
mod occurrence;
mod phrases;
mod prompts;
mod repair_samples;
mod review;
mod sample;
pub mod sample_run;
mod validation;
use validation::{Rejections, prepare_proposal};

type Groups = BTreeMap<Gram<String>, Vec<Occurrence>>;
type Dictionary = BTreeMap<String, Vec<DictionaryReading>>;

/// Evidence belongs to this exact spelling and source sense, not to a reading
/// shared by unrelated entries. `primary` is that source's lexical pronunciation.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct DictionaryEvidence {
    source: String,
    spelling: String,
    meaning: String,
    primary: String,
    form: String,
    notes: Vec<String>,
    priority: Vec<String>,
}

/// One dictionary reading with its separate senses and spelling-bound provenance.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct DictionaryReading {
    reading: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    senses: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    evidence: Vec<DictionaryEvidence>,
}
impl DictionaryReading {
    fn bare(reading: String) -> Self {
        Self {
            reading,
            senses: Vec::new(),
            evidence: Vec::new(),
        }
    }
}

#[cfg(test)]
impl From<&str> for DictionaryReading {
    fn from(reading: &str) -> Self {
        Self::bare(reading.into())
    }
}

/// Schema failures and model refusals are per-answer outcomes. Provider and
/// transport failures must never become degraded menus or annotations.
fn model_answer_or_fatal<T>(
    answer: Result<T, IndividualChatError>,
) -> Result<Result<T, IndividualChatError>> {
    match answer {
        Ok(value) => Ok(Ok(value)),
        Err(
            error @ (IndividualChatError::ResponseNotConformantToSchema { .. }
            | IndividualChatError::Refusal(_)),
        ) => Ok(Err(error)),
        Err(error) => Err(error.into()),
    }
}

/// One item per request. All provider-sized jobs are submitted together.
async fn batch<'a, I, T: serde::de::DeserializeOwned + JsonSchema>(
    client: &ChatClient,
    system: &str,
    items: &'a [I],
    prompt: impl Fn(&I) -> String,
) -> Result<Vec<(&'a I, Result<T, IndividualChatError>)>> {
    use movie_subtitles::llm_segment::batch_jobs;
    let results = batch_jobs::run(|| {
        client.batch_chat_with_system_prompt_fn::<_, _, T>(system, items, &prompt, |_| {})
    })
    .await?;
    let mut answers = Vec::with_capacity(items.len());
    for (request, answer) in results {
        answers.push((request, model_answer_or_fatal(answer)?));
    }
    Ok(answers)
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
struct Occurrence {
    text: String,
    gram_index: usize,
    bytes: (usize, usize),
    spans: Vec<(u32, u32)>,
}
impl Occurrence {
    fn marked(&self) -> String {
        format!(
            "{}«{}»{}",
            &self.text[..self.bytes.0],
            &self.text[self.bytes.0..self.bytes.1],
            &self.text[self.bytes.1..]
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SenseReadings {
    pub index: u32,
    pub kind: String,
    pub gloss: String,
    pub readings: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub synthetic_readings: BTreeSet<String>,
}
impl SenseReadings {
    fn attachment_only(&self, reading: &str) -> bool {
        self.synthetic_readings.contains(reading)
            || self
                .readings
                .iter()
                .any(|plain| canonical::sound_changed(reading, plain))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
struct Choice {
    sense: u32,
    reading: String,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct IndexedChoice {
    sense: u32,
    reading: usize,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct OccurrenceAnswer {
    answer: OccurrenceDecision,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "decision", rename_all = "snake_case", deny_unknown_fields)]
enum OccurrenceDecision {
    Listed {
        choices: Vec<IndexedChoice>,
    },
    NoneOfThese {
        whole_gram_reading: String,
        /// A meaning the menu needs to add, not an explanation of a listed choice.
        missing_meaning: Option<String>,
    },
}

#[derive(Default)]
pub struct CorpusReadings {
    pub(crate) inventories: Option<Vec<UsageInventory>>,
    dictionary: Dictionary,
    pub hierarchies: BTreeMap<Gram<String>, Vec<SenseReadings>>,
    candidates: BTreeMap<Gram<String>, Candidates>,
    /// Deliberate correction removals are not failed inference or fallback inputs.
    corrections_dropped: BTreeSet<Gram<String>>,
    /// Keep every ordered alternative, not just the choice that goes in the pack.
    labels: BTreeMap<(String, usize), occurrence::Label>,
    /// Any pruned first-choice sense excludes its entire sentence everywhere.
    rejected_sentences: BTreeSet<String>,
    sample: Option<sample_run::Sample>,
}

#[derive(Clone, Serialize)]
struct AtomHint {
    text: String,
    lemma: Option<String>,
    pos: String,
    /// The atom's own dictionary entry, shown for phrases so B can see what
    /// each part means; a single-atom gram's entry is its candidate list.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    readings: Vec<DictionaryReading>,
    /// Only for an atom the dictionary cannot read, as a hint for an open gram.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    lemma_readings: Vec<String>,
}
#[derive(Clone, Serialize)]
struct Candidates {
    /// Dictionary senses per whole-gram reading. Readings composed from atoms
    /// have none; their atoms carry the meanings instead.
    senses: BTreeMap<String, Vec<String>>,
    evidence: BTreeMap<String, Vec<DictionaryEvidence>>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    coverings: BTreeMap<String, Vec<coverings::Covering>>,
    readings: BTreeSet<String>,
    #[serde(skip)]
    whole_word_readings: BTreeSet<String>,
    open: bool,
    atoms: Vec<AtomHint>,
    #[serde(skip)]
    prior_single: bool,
}

impl Candidates {
    /// Display order is also index order, including after dictionary priority sorting.
    fn ordered_readings(&self) -> Vec<&String> {
        let mut readings: Vec<_> = self.readings.iter().collect();
        readings.sort_by_key(|reading| {
            prompts::evidence_priority(self.evidence.get(*reading).map_or(&[], Vec::as_slice))
        });
        readings
    }

    fn dictionary_evidence(&self) -> impl Iterator<Item = &DictionaryEvidence> {
        self.evidence
            .values()
            .flatten()
            .chain(
                self.atoms
                    .iter()
                    .flat_map(|a| &a.readings)
                    .flat_map(|r| &r.evidence),
            )
            .chain(
                self.coverings
                    .values()
                    .flatten()
                    .flatten()
                    .flat_map(|part| &part.entry.evidence),
            )
    }
}

/// Apply the learner convention only to an actual 言う lemma atom. Reopened
/// phrases use the same exact atom alignment as phrase projection; uncertain
/// boundaries are a retry, never permission for a global kana replacement.
fn learner_lemma_form(atoms: &[AtomHint], reading: &str) -> Option<String> {
    let iu = |atom: &AtomHint| {
        atom.lemma.as_deref() == Some("言う")
            && atom
                .text
                .chars()
                .any(|c| is_ruby_base(c, Language::Japanese))
    };
    if !reading.contains('ゆ') || !atoms.iter().any(iu) {
        return Some(reading.into());
    }
    if let [atom] = atoms
        && iu(atom)
    {
        return Some(
            reading
                .strip_prefix('ゆ')
                .map_or_else(|| reading.into(), |tail| format!("い{tail}")),
        );
    }
    let options: Vec<_> = atoms
        .iter()
        .map(|atom| {
            let mut options: Vec<_> = atom
                .readings
                .iter()
                .map(|r| (r.reading.clone(), r.reading.clone()))
                .collect();
            if atom.pos == "Punct" {
                options.push((String::new(), String::new()));
            }
            if iu(atom) {
                for r in &atom.readings {
                    if let Some(tail) = r.reading.strip_prefix('い') {
                        options.push((format!("ゆ{tail}"), r.reading.clone()));
                    }
                }
            }
            options
        })
        .collect();
    phrases::align_atoms(reading, &options)
}

fn hiragana(text: &str) -> String {
    text.chars()
        .map(language_utils::readings::canonical_kana)
        .collect()
}
fn is_kana(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|c| ('ぁ'..='ゖ').contains(&c) || c == 'ー')
}

/// An inflected Japanese surface shares a written stem with its dictionary lemma.
/// Replace the inflection ending, retaining fixed stem kana omitted from the
/// spelling (預って still needs あずかって). Kana already written in the surface
/// tail supply their own pronunciation, and incompatible kanji contribute nothing.
fn inflected_readings(
    text: &str,
    lemma: &str,
    dictionary: &Dictionary,
    inflections: &inflections::Index,
) -> Vec<DictionaryReading> {
    let stem: String = text
        .chars()
        .zip(lemma.chars())
        .take_while(|(a, b)| a == b)
        .map(|(a, _)| a)
        .collect();
    let (tail, lemma_tail) = (
        hiragana(&text[stem.len()..]),
        hiragana(&lemma[stem.len()..]),
    );
    if !stem.chars().any(|c| is_ruby_base(c, Language::Japanese))
        || ![&tail, &lemma_tail]
            .iter()
            .all(|t| t.is_empty() || is_kana(t))
    {
        return Vec::new();
    }
    dictionary
        .get(lemma)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let replaced_tail = inflections::replaceable_lemma_tail(&lemma_tail, &tail);
            // Restoring unwritten stem kana needs a compatible typed conjugation.
            // The finite index is not a validator for every colloquial continuation
            // of an already-written stem, so those retain the existing fallback.
            if replaced_tail != lemma_tail && inflections.knows(lemma, &entry.reading) {
                return None;
            }
            if entry.reading == "いい" && lemma_tail == "い" && text != lemma {
                return None;
            }
            let root = entry.reading.strip_suffix(replaced_tail)?;
            let root = inflections::surface_stem(root, lemma, &lemma_tail, &tail);
            (!root.is_empty()).then(|| DictionaryReading {
                reading: format!("{root}{tail}"),
                ..entry.clone()
            })
        })
        .collect()
}

fn normalized_entries(
    text: &str,
    language: Language,
    entries: &[DictionaryReading],
) -> Vec<DictionaryReading> {
    let mut normalized = BTreeMap::<String, DictionaryReading>::new();
    for entry in entries {
        if let Some(reading) = canonical::checked_surface_reading(language, text, &entry.reading) {
            let merged = normalized
                .entry(reading.clone())
                .or_insert_with(|| DictionaryReading::bare(reading));
            for sense in &entry.senses {
                if !merged.senses.contains(sense) {
                    merged.senses.push(sense.clone());
                }
            }
            for evidence in &entry.evidence {
                if !merged.evidence.contains(evidence) {
                    merged.evidence.push(evidence.clone());
                }
            }
        } else {
            log::warn!(
                "readings[{}]: invalid dictionary reading {:?} for {:?}",
                language.code(),
                entry.reading,
                text
            );
        }
    }
    normalized.into_values().collect()
}

fn atom_readings(
    word: &language_utils::Word<String>,
    language: Language,
    dictionary: &Dictionary,
    inflections: &inflections::Index,
) -> Vec<DictionaryReading> {
    let entries = raw_atom_readings(word, language, dictionary, inflections);
    if matches!(&word.word_type, WordType::Other(o) if o.other_tag == language_utils::OtherWordType::Punct)
    {
        return entries;
    }
    normalized_entries(&word.text, language, &entries)
}

fn raw_atom_readings(
    word: &language_utils::Word<String>,
    language: Language,
    dictionary: &Dictionary,
    inflections: &inflections::Index,
) -> Vec<DictionaryReading> {
    if word.text.chars().any(|c| is_ruby_base(c, language)) {
        let mut readings =
            dictionary
                .get(&word.text)
                .cloned()
                .unwrap_or_else(|| match &word.word_type {
                    WordType::Heteronym(h) if language == Language::Japanese => {
                        let typed =
                            inflections.readings_for(&word.text, dictionary, Some(&h.lemma));
                        if typed.is_empty() {
                            // Keep unknown and extended surface forms, but do not
                            // bypass the typed check when restoring omitted stem kana.
                            inflected_readings(&word.text, &h.lemma, dictionary, inflections)
                        } else {
                            typed
                        }
                    }
                    _ => Vec::new(),
                });
        if language == Language::Japanese {
            for entry in inflections.readings(&word.text, dictionary) {
                if let Some(existing) = readings.iter_mut().find(|r| r.reading == entry.reading) {
                    existing.evidence.extend(entry.evidence);
                    for sense in entry.senses {
                        if !existing.senses.contains(&sense) {
                            existing.senses.push(sense);
                        }
                    }
                } else {
                    readings.push(entry);
                }
            }
        }
        return readings;
    }
    if let Some(entry) = dictionary.get(&word.text) {
        return entry.clone();
    }
    if matches!(&word.word_type, WordType::Other(o) if o.other_tag == language_utils::OtherWordType::Punct)
    {
        return vec![DictionaryReading::bare(String::new())];
    }
    let kana = hiragana(&word.text);
    if language == Language::Japanese && is_kana(&kana) {
        vec![DictionaryReading::bare(kana)]
    } else {
        Vec::new()
    }
}

#[cfg(test)]
fn candidates(gram: &Gram<String>, language: Language, dictionary: &Dictionary) -> Candidates {
    indexed_candidates(gram, language, dictionary, &inflections::Index::default())
}

fn indexed_candidates(
    gram: &Gram<String>,
    language: Language,
    dictionary: &Dictionary,
    inflections: &inflections::Index,
) -> Candidates {
    let surface = gram.to_display_string(language);
    let words: Vec<_> = gram
        .iter()
        .filter_map(|atom| match atom {
            Atom::Tok(word) => Some(word),
            _ => None,
        })
        .collect();
    let atom_readings: Vec<_> = words
        .iter()
        .map(|word| atom_readings(word, language, dictionary, inflections))
        .collect();
    let atoms: Vec<_> = words
        .iter()
        .zip(&atom_readings)
        .map(|(word, own)| {
            let (lemma, pos) = match &word.word_type {
                WordType::Heteronym(h) => (Some(h.lemma.clone()), h.pos.to_string()),
                WordType::Other(other) => (None, format!("{:?}", other.other_tag)),
            };
            let lemma_readings = if own.is_empty() {
                lemma
                    .iter()
                    .flat_map(|l| {
                        dictionary
                            .get(l)
                            .into_iter()
                            .flat_map(|entries| normalized_entries(l, language, entries))
                    })
                    .map(|r| r.reading)
                    .collect()
            } else {
                Vec::new()
            };
            let readings = if words.len() > 1 {
                own.iter()
                    .filter(|r| !r.reading.is_empty())
                    .cloned()
                    .collect()
            } else {
                Vec::new()
            };
            AtomHint {
                text: word.text.clone(),
                lemma,
                pos,
                readings,
                lemma_readings,
            }
        })
        .collect();
    let full = dictionary
        .get(&surface)
        .map(|entries| normalized_entries(&surface, language, entries));
    let full = full.as_ref();
    // A single atom's own entry (looked up or inflected) is the whole gram's.
    let whole: &[DictionaryReading] = match full {
        _ if words.len() == 1 => &atom_readings[0],
        Some(full) => full,
        None => &[],
    };
    let mut senses: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut evidence: BTreeMap<String, Vec<DictionaryEvidence>> = BTreeMap::new();
    for entry in whole {
        evidence
            .entry(entry.reading.clone())
            .or_default()
            .extend(entry.evidence.clone());
        senses
            .entry(entry.reading.clone())
            .or_default()
            .extend(entry.senses.iter().cloned());
    }
    // An unknown atom makes the whole product empty. Check before expanding a
    // potentially large prefix, then union with full-headword readings.
    let mut product = BTreeSet::new();
    if language == Language::Japanese && atom_readings.iter().all(|r| !r.is_empty()) {
        product.insert(String::new());
        for options in &atom_readings {
            product = product
                .iter()
                .flat_map(|prefix| {
                    options.iter().map(move |option| {
                        let reading = &option.reading;
                        if reading.is_empty() {
                            prefix.clone()
                        } else if prefix.is_empty() || language == Language::Japanese {
                            format!("{prefix}{reading}")
                        } else {
                            format!("{prefix} {reading}")
                        }
                    })
                })
                .collect();
        }
    }
    let mut coverings = BTreeMap::new();
    if language == Language::ChineseSimplified {
        // Complete coverings are independent of tokenizer boundaries. Whole-word
        // evidence stays in its existing field; composed readings keep real parts.
        product.clear();
        let mut whole_entries = Vec::new();
        for (reading, paths) in coverings::readings(&surface, dictionary) {
            product.insert(reading.clone());
            let mut composed = Vec::new();
            for mut parts in paths {
                if parts.len() == 1 && parts[0].text == surface {
                    whole_entries.push(parts.pop().unwrap().entry);
                } else {
                    composed.push(parts);
                }
            }
            if !composed.is_empty() {
                coverings.insert(reading, composed);
            }
        }
        for entry in normalized_entries(&surface, language, &whole_entries) {
            evidence
                .entry(entry.reading)
                .or_default()
                .extend(entry.evidence);
        }
    }
    let headword: BTreeSet<_> = full
        .into_iter()
        .flatten()
        .map(|r| r.reading.clone())
        .collect();
    // Keep compositional and headword pronunciations separate until B has
    // assigned meanings. Tone equivalence alone cannot identify that meaning.
    let product: BTreeSet<_> = product
        .into_iter()
        .filter_map(|r| canonical::checked_surface_reading(language, &surface, &r))
        .collect();
    let prior_single = full.map_or(product.len() == 1, |_| headword.len() == 1);
    let mut readings = product;
    readings.extend(headword.iter().cloned());
    senses.retain(|reading, _| readings.contains(reading));
    let open = readings.is_empty();
    let whole_word_readings = headword;
    Candidates {
        senses,
        evidence,
        coverings,
        readings,
        whole_word_readings,
        open,
        atoms,
        prior_single,
    }
}

/// Only a word with one reading and one dictionary sense has nothing for B to
/// organize. Chinese "/" items mix synonyms and senses, so several items send
/// a word to B, which merges the synonyms back into one sense. A word with no
/// dictionary sense (a reading composed from parts) can still be polysemous,
/// so B builds its senses once it occurs often enough for two of them to be
/// attested; a rarer one can never have sense cards and skips B and C.
fn needs_hierarchy(gram: &Gram<String>, candidates: &Candidates, occurrences: usize) -> bool {
    let senses = candidates.senses.values().map(Vec::len).sum::<usize>();
    gram.len() > 1
        || candidates.open
        || candidates.readings.len() != 1
        || senses > 1
        || (senses == 0 && occurrences >= 2 * inventory::MIN_ATTESTATIONS)
}

/// B/C depend only on encoded tokens and their text, never matching overlays.
fn corpus_groups(
    language: Language,
    app: &BTreeMap<String, SentenceInfo>,
    restricted: &BTreeMap<String, SentenceInfo>,
    interners: &GramInterners,
) -> Groups {
    let mut groups = Groups::new();
    for (text, info) in app.iter().chain(
        restricted
            .iter()
            .filter(|(text, _)| !app.contains_key(*text)),
    ) {
        let grams = info
            .sentence
            .tokens
            .iter()
            .map(|g| interners.grams.resolve(&g.gram).resolve(&interners.strings))
            .collect();
        add_occurrences(
            &mut groups,
            text,
            grams,
            &info.decode_words(interners, language),
            language,
        );
    }
    groups
}

fn add_occurrences(
    groups: &mut Groups,
    text: &str,
    grams: Vec<Gram<String>>,
    words: &[Literal<String>],
    language: Language,
) {
    let mut positions = Vec::new();
    let (mut byte, mut character) = (0, 0u32);
    for literal in words {
        let end = byte + literal.word.text.len();
        assert_eq!(
            &text[byte..end],
            literal.word.text,
            "reading spans preserve source text"
        );
        let char_end = character + literal.word.text.chars().count() as u32;
        positions.push(((byte, end), (character, char_end)));
        byte = end + literal.whitespace.as_str().len();
        character = char_end + literal.whitespace.as_str().chars().count() as u32;
    }
    let mut word_index = 0;
    for (gram_index, gram) in grams.into_iter().enumerate() {
        let count = gram.iter().filter(|a| matches!(a, Atom::Tok(_))).count();
        let range = word_index..word_index + count;
        word_index += count;
        if count == 0
            || !gram.iter().any(
                |a| matches!(a, Atom::Tok(w) if w.text.chars().any(|c| is_ruby_base(c, language))),
            )
        {
            continue;
        }
        let bytes = (positions[range.start].0.0, positions[range.end - 1].0.1);
        let spans = positions[range.clone()]
            .iter()
            .zip(&words[range])
            .filter(|(_, literal)| matches!(&literal.word.word_type, WordType::Heteronym(_)))
            .map(|((_, chars), _)| *chars)
            .collect();
        groups.entry(gram).or_default().push(Occurrence {
            text: text.to_owned(),
            gram_index,
            bytes,
            spans,
        });
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
struct ProposedReading {
    /// The zero-based candidate index, or null for a reading outside the list.
    index: Option<usize>,
    /// One complete pronunciation of the entire marked gram, not an atom or syllable.
    #[serde(rename = "whole_gram_reading")]
    reading: String,
    /// For a reading outside the candidate list, the zero-based example line
    /// that needs it; null for a listed candidate.
    example_line: Option<usize>,
}
/// A validated reading. Only readings outside the dictionary keep the example
/// that justified them, which is also the context for focused verification.
#[derive(Clone, Debug, Serialize)]
struct GroundedReading {
    reading: String,
    example: Option<usize>,
}
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
struct Proposal<R = ProposedReading> {
    reopen: bool,
    senses: Vec<ProposedSense<R>>,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
struct SenseGrounding {
    /// Source sense IDs whose meaning applies, not merely whose reading matches.
    #[serde(default)]
    sources: Vec<String>,
    /// During additive repair, the existing sense to extend; null creates a sense.
    existing_sense: Option<u32>,
    /// Example lines demonstrating this meaning, including forced missing meanings.
    #[serde(default)]
    examples: Vec<usize>,
}
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
struct ProposedSense<R = ProposedReading> {
    grounding: SenseGrounding,
    kind: UsageKind,
    /// A concise gloss that tells this sense apart from the gram's others.
    gloss: String,
    readings: Vec<R>,
}
#[derive(Clone)]
struct HierarchyRequest {
    gram: Gram<String>,
    candidates: Candidates,
    prompt: serde_json::Value,
    examples: Vec<Occurrence>,
}
impl HierarchyRequest {
    async fn sampled(
        gram: &Gram<String>,
        candidates: Candidates,
        occurrences: &[Occurrence],
        language: Language,
        store: &osmo::Store,
        cache: Option<&sample::Cache>,
    ) -> Self {
        let examples = sample::persistent_select(cache, gram, occurrences, language, store).await;
        let mut prompt = prompts::hierarchy_prompt(gram, &candidates, language);
        prompt["examples"] = prompts::numbered_examples(&examples);
        Self {
            gram: gram.clone(),
            candidates,
            prompt,
            examples,
        }
    }
}
fn resolve_choices(choices: &[IndexedChoice], hierarchy: &[SenseReadings]) -> Option<Vec<Choice>> {
    if choices.is_empty() {
        return None;
    }
    choices
        .iter()
        .map(|c| {
            let sense = hierarchy.iter().find(|s| s.index == c.sense)?;
            Some(Choice {
                sense: c.sense,
                reading: sense.readings.get(c.reading)?.clone(),
            })
        })
        .collect()
}
/// The gram's single unsplit sense, which is not a vocabulary card of its own.
fn implicit_hierarchy(readings: Vec<String>) -> Vec<SenseReadings> {
    vec![SenseReadings {
        index: 0,
        kind: "meaning".into(),
        synthetic_readings: BTreeSet::new(),
        gloss: String::new(),
        readings,
    }]
}

impl CorpusReadings {
    /// One reading and at most one dictionary sense: nothing for B to organize.
    /// With no sense at all, the word is too rare for sense cards (see
    /// `needs_hierarchy`), so C has nothing to choose either; it could only
    /// escape to ask for a meaning no learner-facing output uses. Label it here.
    fn install_implicit(
        &mut self,
        gram: &Gram<String>,
        candidates: Candidates,
        occurrences: &[Occurrence],
    ) {
        let gloss = candidates
            .senses
            .values()
            .flatten()
            .cloned()
            .collect::<Vec<_>>()
            .join("; ");
        let readings: Vec<_> = candidates.readings.into_iter().collect();
        if gloss.is_empty() {
            for o in occurrences {
                self.labels.insert(
                    (o.text.clone(), o.gram_index),
                    occurrence::Label::Ranked {
                        choices: vec![Choice {
                            sense: 0,
                            reading: readings[0].clone(),
                        }],
                    },
                );
            }
        }
        let mut senses = implicit_hierarchy(readings);
        senses[0].gloss = gloss;
        self.hierarchies.insert(gram.clone(), senses);
        if let Some(sample) = &mut self.sample {
            sample.provenance.insert(gram.clone(), "implicit");
        }
    }

    fn install_candidate_fallbacks(&mut self, failed: &BTreeSet<Gram<String>>) {
        for gram in failed {
            let candidates = &self.candidates[gram];
            if !self.corrections_dropped.contains(gram)
                && !candidates.readings.is_empty()
                && !self.hierarchies.contains_key(gram)
            {
                self.hierarchies.insert(
                    gram.clone(),
                    implicit_hierarchy(
                        candidates.ordered_readings().into_iter().cloned().collect(),
                    ),
                );
                if let Some(sample) = &mut self.sample {
                    sample.provenance.insert(gram.clone(), "b_fallback");
                    sample.b_fallback.insert(gram.clone());
                }
            }
        }
    }

    pub async fn generate(
        language: Language,
        app: &BTreeMap<String, SentenceInfo>,
        restricted: &BTreeMap<String, SentenceInfo>,
        interners: &GramInterners,
        store: &osmo::Store,
        sample: Option<sample_run::Sample>,
    ) -> Result<Self> {
        if !supports_readings(language) {
            return Ok(Self::default());
        }
        let output = std::process::Command::new("python3")
            .args([
                "generate-data/scripts/reading_candidates.py",
                language.code(),
            ])
            .output()?;
        ensure!(
            output.status.success(),
            "dictionary extraction failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        let path = format!(".cache/readings/{}-dictionary.json", language.code());
        let mut data = Self {
            dictionary: serde_json::from_reader(std::fs::File::open(path)?)?,
            sample,
            ..Default::default()
        };
        let groups = corpus_groups(language, app, restricted, interners);
        data.annotate(language, &groups, store).await?;
        Ok(data)
    }

    async fn annotate(
        &mut self,
        language: Language,
        groups: &Groups,
        store: &osmo::Store,
    ) -> Result<()> {
        let mut examples = if self.sample.is_none() {
            Some(sample::Cache::load(
                format!("out/{}/reading_examples.jsonl", language.corpus_code()).into(),
            )?)
        } else {
            None
        };
        let mut repair_examples = if self.sample.is_none() {
            Some(repair_samples::Samples::load(language)?)
        } else {
            None
        };
        let mut corrections = corrections::Corrections::load(language)?;
        let path = format!(
            "generate-data/data/{}/usage_inventories.jsonl",
            language.corpus_code()
        );
        let contents = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(e.into()),
        };
        // The inventory is only the persistent slot ledger here: dictionary
        // senses replace meaning discovery for reading languages, so its old
        // usages are never shown to B (see inventory::materialize).
        let mut rows: Vec<UsageInventory> = contents
            .lines()
            .map(serde_json::from_str)
            .collect::<Result<_, _>>()?;
        assert!(rows.iter().all(|r| r.usages.len() >= 2));
        let kanji = if language == Language::Japanese {
            let characters =
                serde_json::from_reader(std::fs::File::open(".cache/readings/jpn-kanjidic.json")?)?;
            let stems = serde_json::from_reader(std::fs::File::open(
                ".cache/readings/jpn-reading-stems.json",
            )?)?;
            kanjidic::Verifier::new(characters, &self.dictionary, stems)
        } else {
            kanjidic::Verifier::default()
        };
        let inflections = if language == Language::Japanese {
            inflections::Index::new(serde_json::from_reader(std::fs::File::open(
                ".cache/readings/jpn-inflections.json",
            )?)?)
        } else {
            inflections::Index::default()
        };
        // C is sentence-sampled; B always selects its examples from the full groups.
        let sampled_groups = self.sample.as_ref().map(|sample| sample.groups(groups));
        let occurrence_groups = sampled_groups.as_ref().unwrap_or(groups);
        let mut requests = Vec::new();
        let (mut single, mut ambiguous, mut open) = (0, 0, 0);
        let mut population = 0;
        let (mut multi_atom, mut added_multi_atom) = (0, 0);
        for (gram, occurrences) in occurrence_groups {
            let candidates = indexed_candidates(gram, language, &self.dictionary, &inflections);
            self.candidates.insert(gram.clone(), candidates.clone());
            if gram.len() > 1 {
                multi_atom += 1;
                added_multi_atom += usize::from(candidates.prior_single);
            }
            let needs_hierarchy = needs_hierarchy(gram, &candidates, groups[gram].len());
            if needs_hierarchy {
                population += 1;
            }
            if candidates.open {
                open += occurrences.len();
            } else if needs_hierarchy {
                ambiguous += occurrences.len();
            } else {
                single += occurrences.len();
            }
            if !needs_hierarchy {
                self.install_implicit(gram, candidates, occurrences);
                continue;
            }
            requests.push(HierarchyRequest {
                prompt: prompts::hierarchy_prompt(gram, &candidates, language),
                gram: gram.clone(),
                candidates,
                examples: Vec::new(),
            });
        }
        println!(
            "readings[{}]: multi-atom B population={multi_atom}, added over headword-priority rule={added_multi_atom}",
            language.code()
        );
        println!(
            "readings[{}]: selecting diverse examples for {} hierarchies",
            language.code(),
            requests.len()
        );
        use futures::StreamExt;
        let cache = examples.as_ref();
        let requests: Vec<_> =
            futures::stream::iter(requests.into_iter().map(|request| async move {
                HierarchyRequest::sampled(
                    &request.gram,
                    request.candidates,
                    &groups[&request.gram],
                    language,
                    store,
                    cache,
                )
                .await
            }))
            .buffered(8)
            .collect()
            .await;
        if let Some(cache) = &mut examples {
            for request in &requests {
                sample::remember(
                    cache,
                    &request.gram,
                    &groups[&request.gram],
                    &request.examples,
                );
            }
            cache.save()?;
        }
        println!(
            "readings[{}]: dictionary occurrence coverage single_atom_unique={single}, closed_hierarchy={ambiguous}, open={open}; hierarchy requests={}",
            language.code(),
            requests.len()
        );
        let mut accepted =
            hierarchy::infer(language, &requests, &kanji, population, "initial").await?;
        let failed = {
            let accepted_grams: BTreeSet<_> = accepted.iter().map(|(r, _)| &r.gram).collect();
            requests
                .iter()
                .filter(|r| !accepted_grams.contains(&r.gram))
                .map(|r| r.gram.clone())
                .collect()
        };
        self.corrections_dropped
            .extend(corrections.apply(&mut accepted, "initial"));
        if let Some(sample) = &mut self.sample {
            sample.b_initial = requests.iter().map(|r| r.gram.clone()).collect();
            sample.record_accepted(&accepted, "b_initial");
        }
        hierarchy::install(self, accepted, language, "initial", occurrence_groups)?;
        self.install_candidate_fallbacks(&failed);
        if language == Language::Japanese {
            hierarchy::expand_variants(self, &kanji);
        }
        let selected = occurrence_groups.keys().cloned().collect();
        println!(
            "readings[{}]: all-ruby C population={} grams, {} occurrences (including {single} formerly dictionary-unique occurrences)",
            language.code(),
            occurrence_groups.len(),
            occurrence_groups.values().map(Vec::len).sum::<usize>()
        );
        let escapes = occurrence::annotate(
            self,
            language,
            occurrence_groups,
            &selected,
            &BTreeSet::new(),
            &kanji,
            "initial",
        )
        .await?;
        if let Some(initial) = self
            .sample
            .as_ref()
            .map(|_| sample_run::labels(self, occurrence_groups))
            && let Some(sample) = &mut self.sample
        {
            sample.initial = initial;
        }
        let remaining = if escapes.is_empty() {
            occurrence::Escapes::new()
        } else {
            let original: BTreeMap<_, _> = requests.iter().map(|r| (&r.gram, r)).collect();
            let mut repair = Vec::new();
            for (gram, escaped) in &escapes {
                let mut request = if let Some(request) = original.get(gram) {
                    (*request).clone()
                } else {
                    let candidates =
                        indexed_candidates(gram, language, &self.dictionary, &inflections);
                    let request = HierarchyRequest::sampled(
                        gram,
                        candidates,
                        &groups[gram],
                        language,
                        store,
                        examples.as_ref(),
                    )
                    .await;
                    if let Some(cache) = &mut examples {
                        sample::remember(cache, gram, &groups[gram], &request.examples);
                    }
                    request
                };
                for sense in self.hierarchies.get(gram).into_iter().flatten() {
                    request
                        .candidates
                        .readings
                        .extend(sense.readings.iter().cloned());
                }
                request.prompt = prompts::hierarchy_prompt(gram, &request.candidates, language);
                let forced = repair_samples::forced(repair_examples.as_mut(), gram, escaped);
                occurrence::force_into(&mut request, &forced);
                request.prompt["existing_hierarchy"] =
                    serde_json::json!(self.hierarchies.get(gram).map_or(&[][..], Vec::as_slice));
                let attested_pool: Vec<_> = occurrence_groups[gram]
                    .iter()
                    .filter_map(|o| {
                        let choices = self
                            .labels
                            .get(&(o.text.clone(), o.gram_index))?
                            .choices()?;
                        Some(repair_samples::Attested::new(o, choices))
                    })
                    .collect();
                let attested =
                    repair_samples::attested(repair_examples.as_mut(), gram, &attested_pool);
                request.prompt["attested_examples"] = serde_json::json!(attested);
                repair.push(request);
            }
            if let Some(cache) = &examples {
                cache.save()?;
            }
            if let Some(cache) = &repair_examples {
                cache.save()?;
            }
            let repair_population = population
                + repair
                    .iter()
                    .filter(|r| !original.contains_key(&r.gram))
                    .count();
            let mut accepted = hierarchy::infer(
                language,
                &repair,
                &kanji,
                repair_population,
                "escape-repair",
            )
            .await?;
            for gram in corrections.apply(&mut accepted, "escape-repair") {
                // Dropping every proposed addition does not remove protected
                // existing senses. Only a genuinely empty corrected menu is skipped.
                if self.hierarchies.get(&gram).is_none_or(|h| h.is_empty()) {
                    self.corrections_dropped.insert(gram);
                }
            }
            if let Some(sample) = &mut self.sample {
                sample.b_repair = repair.iter().map(|r| r.gram.clone()).collect();
                sample.record_accepted(&accepted, "b_repair");
            }
            let previous_menus: BTreeMap<_, _> = escapes
                .keys()
                .map(|g| (g.clone(), self.hierarchies.get(g).cloned()))
                .collect();
            hierarchy::install(self, accepted, language, "escape-repair", occurrence_groups)?;
            if language == Language::Japanese {
                hierarchy::expand_variants(self, &kanji);
            }
            let repair_grams = escapes.keys().cloned().collect();
            let reconsider = previous_menus
                .into_iter()
                .filter_map(|(gram, before)| {
                    (self.hierarchies.get(&gram) != before.as_ref()).then_some(gram)
                })
                .collect();
            occurrence::annotate(
                self,
                language,
                occurrence_groups,
                &repair_grams,
                &reconsider,
                &kanji,
                "repair",
            )
            .await?
        };
        corrections.report_unmatched();
        if let Some(sample) = &self.sample {
            return sample.write(self, occurrence_groups, language);
        }
        inventory::finish(self, &mut rows, groups, language)?;
        self.inventories = Some(rows);
        if !tysm::measurement::enabled() {
            occurrence::write_remaining(language, &remaining)?;
        }
        Ok(())
    }

    pub(crate) fn has_observed_sense(&self, gram: &Gram<String>, sense: u32) -> bool {
        self.hierarchies
            .get(gram)
            .is_some_and(|senses| senses.iter().any(|s| s.index == sense))
    }

    pub fn sense(&self, text: &str, gram_index: usize) -> Option<NonZeroU32> {
        NonZeroU32::new(
            self.labels
                .get(&(text.to_owned(), gram_index))?
                .first()?
                .sense,
        )
    }
    pub fn includes_sentence(&self, text: &str) -> bool {
        !self.rejected_sentences.contains(text)
    }

    /// Apply the same rejection boundary before assignment, translation, and
    /// frequency/source construction. Later generated inputs use the predicate.
    pub fn retain_corpus(
        &self,
        corpus: &mut crate::target_sentences::TargetSentences,
        app: &mut BTreeMap<String, SentenceInfo>,
        restricted: &mut BTreeMap<String, SentenceInfo>,
    ) {
        corpus
            .app_sentences
            .retain(|(s, _, _)| self.includes_sentence(s));
        corpus
            .book_sentences
            .retain(|(s, _, _)| self.includes_sentence(s));
        corpus
            .restricted_sentences
            .retain(|(s, _)| self.includes_sentence(s));
        app.retain(|s, _| self.includes_sentence(s));
        restricted.retain(|s, _| self.includes_sentence(s));
    }

    fn reading(&self, text: &str, gram_index: usize) -> Option<&str> {
        // Use only this occurrence's checked reading, including a guarded escape;
        // a missing reading never borrows another occurrence's pronunciation.
        self.labels.get(&(text.to_owned(), gram_index))?.reading()
    }

    pub fn pack(
        &self,
        sentences: Vec<(String, SentenceGrams<TaggedGram<Gram<String>>>)>,
        language: Language,
    ) -> Vec<(
        String,
        language_utils::ReadingSentence<Gram<String>, String>,
    )> {
        let mut sentences: Vec<_> = sentences
            .into_iter()
            .filter(|(text, _)| self.includes_sentence(text))
            .map(|(text, sentence)| {
                let grams = sentence
                    .grams
                    .into_iter()
                    .enumerate()
                    .map(|(index, g)| {
                        g.map(|sense| {
                            let reading = self.reading(&text, index).map(str::to_owned);
                            ReadingTaggedGram { sense, reading }
                        })
                    })
                    .collect();
                (
                    text,
                    SentenceGrams {
                        grams,
                        capitalize_first: sentence.capitalize_first,
                        multiword_terms: sentence
                            .multiword_terms
                            .into_iter()
                            .map(|m| m.map(|g| ReadingTaggedGram::without_reading(g.clone())))
                            .collect(),
                        low_confidence_multiword_terms: sentence
                            .low_confidence_multiword_terms
                            .into_iter()
                            .map(|m| m.map(|g| ReadingTaggedGram::without_reading(g.clone())))
                            .collect(),
                    },
                )
            })
            .collect();
        let stats = phrases::annotate(&mut sentences, language, &self.dictionary);
        println!(
            "readings[{}]: phrase grams={}, without reading={}",
            language.code(),
            stats.grams,
            stats.without_reading
        );
        sentences
    }
}

// A repair round uses the full current B population too, not just its small
// subset of escaped grams.
fn hierarchy_failure_limit_exceeded(failed: usize, population: usize) -> bool {
    failed * 100 > population
}

fn write_inventories<T: Serialize>(path: &str, rows: &[T]) -> Result<()> {
    let temporary = format!("{path}.tmp");
    let mut file = std::io::BufWriter::new(std::fs::File::create(&temporary)?);
    for row in rows {
        writeln!(file, "{}", serde_json::to_string(row)?)?;
    }
    file.flush()?;
    std::fs::rename(temporary, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use language_utils::{Heteronym, PartOfSpeech, Word};

    #[test]
    fn b_and_c_groups_ignore_unknown_slot_overlays() {
        use language_utils::{EncodedSentence, MultiwordTermMatch, MultiwordTerms};
        let mut strings = lasso::Rodeo::default();
        let mut grams = lasso::Rodeo::default();
        let tokens = [gram(&["帮助"]), gram(&["人"])]
            .into_iter()
            .map(|gram| {
                let key = grams.get_or_intern(gram.get_or_intern(&mut strings));
                TaggedGram {
                    gram: key,
                    sense: None,
                }
            })
            .collect();
        let interners = GramInterners {
            strings: strings.into_reader(),
            grams: grams.into_reader(),
        };
        let mut app = BTreeMap::from([(
            "帮助人".into(),
            SentenceInfo {
                sentence: EncodedSentence {
                    tokens,
                    capitalize_first: false,
                },
                multiword_terms: MultiwordTerms {
                    high_confidence: vec![],
                    low_confidence: vec![],
                },
            },
        )]);
        let before = corpus_groups(
            Language::ChineseSimplified,
            &app,
            &BTreeMap::new(),
            &interners,
        );
        app.get_mut("帮助人")
            .unwrap()
            .multiword_terms
            .high_confidence
            .push(MultiwordTermMatch {
                gram: TaggedGram {
                    gram: gram(&["帮助", "某人"]),
                    sense: NonZeroU32::new(7),
                },
                matched_word_indices: vec![0, 1],
            });
        let after = corpus_groups(
            Language::ChineseSimplified,
            &app,
            &BTreeMap::new(),
            &interners,
        );
        assert_eq!(before, after);
        assert_eq!(before.len(), 2);
    }

    #[tokio::test]
    async fn missing_initial_retry_or_repair_b_answer_never_reaches_c() {
        let client = ChatClient::new("unused", "gpt-6-luna").with_cached_only();
        for phase in ["initial B", "B retry", "B verification", "B repair"] {
            let mut reached_c = false;
            let outcome: Result<()> = async {
                batch::<_, Proposal>(&client, phase, &["missing"], |text| text.to_string()).await?;
                reached_c = true;
                Ok(())
            }
            .await;
            assert!(outcome.is_err(), "{phase}");
            assert!(!reached_c, "{phase}");
        }
        // Actual model-quality rejections remain distinct from an absent answer.
        assert!(
            model_answer_or_fatal::<()>(Err(IndividualChatError::Refusal("real refusal".into())))
                .is_ok()
        );
    }
    fn kanji() -> kanjidic::Verifier {
        kanjidic::Verifier::new(
            BTreeMap::from([('硝', vec!["しょう".into()]), ('子', vec!["こ".into()])]),
            &BTreeMap::new(),
            BTreeMap::new(),
        )
    }
    fn example() -> Occurrence {
        Occurrence {
            text: "何".into(),
            gram_index: 0,
            bytes: (0, 3),
            spans: vec![(0, 1)],
        }
    }
    pub(super) fn gram(texts: &[&str]) -> Gram<String> {
        Gram(
            texts
                .iter()
                .map(|text| {
                    Atom::Tok(Word {
                        text: (*text).into(),
                        word_type: WordType::Heteronym(Heteronym {
                            word: (*text).into(),
                            lemma: (*text).into(),
                            pos: PartOfSpeech::Noun,
                        }),
                    })
                })
                .collect(),
        )
    }
    pub(super) fn word(text: &str, lemma: &str, pos: PartOfSpeech) -> Gram<String> {
        Gram(vec![Atom::Tok(Word {
            text: text.into(),
            word_type: WordType::Heteronym(Heteronym {
                word: text.into(),
                lemma: lemma.into(),
                pos,
            }),
        })])
    }
    fn entry(reading: &str, senses: &[&str]) -> DictionaryReading {
        DictionaryReading {
            reading: reading.into(),
            senses: senses.iter().map(|s| (*s).to_owned()).collect(),
            evidence: Vec::new(),
        }
    }

    #[test]
    fn reopened_phrases_apply_iu_policy_at_proven_atom_boundaries() {
        let dictionary = BTreeMap::from([
            ("夕方".into(), vec!["ゆうがた".into()]),
            ("夕方言った".into(), vec!["ゆうがたいった".into()]),
            ("言う".into(), vec!["いう".into()]),
        ]);
        let mut phrase = gram(&["夕方"]);
        phrase
            .0
            .extend(word("言った", "言う", PartOfSpeech::Verb).0);
        let c = candidates(&phrase, Language::Japanese, &dictionary);
        assert!(c.readings.contains("ゆうがたいった"));
        assert!(!c.readings.contains("ゆうがたゆった"));
        assert_eq!(
            c.whole_word_readings,
            BTreeSet::from(["ゆうがたいった".into()])
        );
        let written = candidates(
            &word("ゆう", "言う", PartOfSpeech::Verb),
            Language::Japanese,
            &dictionary,
        );
        assert_eq!(written.readings, BTreeSet::from(["ゆう".into()]));
        assert_eq!(
            learner_lemma_form(&written.atoms, "ゆう").as_deref(),
            Some("ゆう")
        );
        assert_eq!(
            learner_lemma_form(&c.atoms, "ゆうがたゆった").as_deref(),
            Some("ゆうがたいった")
        );
        assert_eq!(
            learner_lemma_form(&c.atoms, "ゆうがたいった").as_deref(),
            Some("ゆうがたいった")
        );
        assert!(learner_lemma_form(&c.atoms, "ゆうかたゆった").is_none());
        let c = candidates(
            &word("言った", "言う", PartOfSpeech::Verb),
            Language::Japanese,
            &dictionary,
        );
        assert_eq!(
            learner_lemma_form(&c.atoms, "ゆった").as_deref(),
            Some("いった")
        );
        let c = candidates(
            &word("結った", "結う", PartOfSpeech::Verb),
            Language::Japanese,
            &dictionary,
        );
        assert_eq!(
            learner_lemma_form(&c.atoms, "ゆった").as_deref(),
            Some("ゆった")
        );
        assert!(
            phrases::align_atoms(
                "いう",
                &[
                    vec![("い".into(), "い".into()), ("いう".into(), "ゆう".into())],
                    vec![("う".into(), "う".into()), ("".into(), "".into())]
                ]
            )
            .is_none()
        );
    }

    #[test]
    fn provider_errors_abort_b_c_and_verification_before_answer_fallback() {
        assert_eq!(model_answer_or_fatal::<u32>(Ok(7)).unwrap().unwrap(), 7);
        let schema_error = IndividualChatError::ResponseNotConformantToSchema {
            error: serde_json::from_str::<u32>("\"not a number\"").unwrap_err(),
            response: "\"not a number\"".into(),
            schema: "integer".into(),
        };
        assert!(matches!(
            model_answer_or_fatal::<u32>(Err(schema_error)).unwrap(),
            Err(IndividualChatError::ResponseNotConformantToSchema { .. })
        ));
        assert!(matches!(
            model_answer_or_fatal::<u32>(Err(IndividualChatError::Refusal("model refusal".into())))
                .unwrap(),
            Err(IndividualChatError::Refusal(_))
        ));
        for message in [
            "billing_hard_limit_reached",
            "401 Unauthorized",
            "connection reset after retries",
        ] {
            let error =
                model_answer_or_fatal::<u32>(Err(IndividualChatError::Other(message.into())))
                    .unwrap_err();
            assert!(error.to_string().contains(message));
        }
        assert!(model_answer_or_fatal::<u32>(Err(IndividualChatError::CacheMiss)).is_err());
    }

    #[test]
    fn rare_composed_readings_without_a_meaning_skip_b_and_c() {
        let dictionary = Dictionary::from([
            ("拿".into(), vec!["ná".into()]),
            ("回".into(), vec!["huí".into()]),
            (
                "马达".into(),
                vec![DictionaryReading {
                    senses: vec!["motor".into()],
                    ..DictionaryReading::bare("mǎ dá".into())
                }],
            ),
        ]);
        let mut data = CorpusReadings::default();
        for (text, labelled) in [("拿回", true), ("马达", false)] {
            let gram = gram(&[text]);
            let c = candidates(&gram, Language::ChineseSimplified, &dictionary);
            assert!(!needs_hierarchy(&gram, &c, 5));
            // Frequent enough for two attested senses: only B can find them.
            assert_eq!(needs_hierarchy(&gram, &c, 6), labelled);
            let occurrence = Occurrence {
                text: text.into(),
                gram_index: 0,
                bytes: (0, text.len()),
                spans: vec![(0, 2)],
            };
            data.install_implicit(&gram, c, &[occurrence]);
            assert_eq!(
                data.labels
                    .get(&(text.to_string(), 0))
                    .and_then(|l| l.reading()),
                labelled.then_some(data.hierarchies[&gram][0].readings[0].as_str())
            );
        }
        assert_eq!(
            data.labels[&("拿回".to_string(), 0)].reading(),
            Some("ná huí")
        );
    }

    #[test]
    fn failed_hierarchies_reuse_unnamed_candidates_without_assigning_labels() {
        let gram = gram(&["私"]);
        let dictionary =
            Dictionary::from([("私".into(), vec!["わたし".into(), "わたくし".into()])]);
        let mut data = CorpusReadings {
            sample: Some(sample_run::Sample::with_pins(10.0, BTreeSet::new()).unwrap()),
            ..Default::default()
        };
        data.candidates.insert(
            gram.clone(),
            candidates(&gram, Language::Japanese, &dictionary),
        );
        data.install_candidate_fallbacks(&BTreeSet::from([gram.clone()]));
        let menu = &data.hierarchies[&gram];
        assert_eq!(menu.len(), 1);
        assert_eq!(menu[0].index, 0);
        assert!(menu[0].gloss.is_empty());
        assert_eq!(menu[0].readings.len(), 2);
        assert_eq!(
            data.sample.as_ref().unwrap().provenance[&gram],
            "b_fallback"
        );
        assert!(data.labels.is_empty());
        data.hierarchies.get_mut(&gram).unwrap()[0].gloss = "existing meaning".into();
        data.install_candidate_fallbacks(&BTreeSet::from([gram.clone()]));
        assert_eq!(data.hierarchies[&gram][0].gloss, "existing meaning");
    }

    #[test]
    fn phrase_source_validation_accepts_all_prompt_visible_atom_evidence() {
        let mut reading = DictionaryReading::bare("ねこ".into());
        reading.evidence.push(DictionaryEvidence {
            source: "jmdict:cat:0".into(),
            spelling: "猫".into(),
            meaning: "cat".into(),
            primary: "ねこ".into(),
            form: "primary".into(),
            notes: vec![],
            priority: vec![],
        });
        let dictionary = Dictionary::from([("猫".into(), vec![reading])]);
        let gram = gram(&["猫", "の"]);
        let c = candidates(&gram, Language::Japanese, &dictionary);
        assert!(c.evidence.is_empty());
        assert!(c.dictionary_evidence().any(|e| e.source == "jmdict:cat:0"));
        let request = HierarchyRequest {
            prompt: prompts::hierarchy_prompt(&gram, &c, Language::Japanese),
            gram,
            candidates: c,
            examples: vec![],
        };
        let proposal = Proposal {
            reopen: false,
            senses: vec![ProposedSense {
                grounding: SenseGrounding {
                    sources: vec!["jmdict:cat:0".into()],
                    ..Default::default()
                },
                kind: UsageKind::Meaning,
                gloss: "猫の所有・所属".into(),
                readings: vec![ProposedReading {
                    index: Some(0),
                    reading: "ねこの".into(),
                    example_line: None,
                }],
            }],
        };
        assert!(
            prepare_proposal(
                proposal,
                &request,
                Language::Japanese,
                &kanjidic::Verifier::default(),
                &mut Rejections::default()
            )
            .is_some()
        );
    }

    #[test]
    fn b_sees_each_readings_separate_senses_once() {
        let dictionary = BTreeMap::from([(
            "东西".into(),
            vec![
                entry("dōng xi", &["thing", "stuff", "person"]),
                entry("dōng xī", &["east and west"]),
            ],
        )]);
        let gram = gram(&["东西"]);
        let c = candidates(&gram, Language::ChineseSimplified, &dictionary);
        let prompt = prompts::hierarchy_prompt(&gram, &c, Language::ChineseSimplified);
        let readings = &prompt["candidates"]["readings"];
        assert_eq!(
            readings[0]["senses"],
            serde_json::json!(["thing", "stuff", "person"])
        );
        assert_eq!(readings[1]["senses"], serde_json::json!(["east and west"]));
        // A single-atom gram's entry is already its candidate list.
        assert!(prompt["candidates"]["atoms"][0].get("readings").is_none());
    }

    #[test]
    fn phrases_show_their_atoms_meanings_and_composed_readings_carry_none() {
        let dictionary = BTreeMap::from([
            ("告诉".into(), vec![entry("gào su", &["to tell"])]),
            ("我".into(), vec![entry("wǒ", &["I", "me"])]),
        ]);
        let gram = gram(&["告诉", "我"]);
        let c = candidates(&gram, Language::ChineseSimplified, &dictionary);
        assert_eq!(c.readings, BTreeSet::from(["gào su wǒ".into()]));
        assert!(c.senses.is_empty());
        let prompt = prompts::hierarchy_prompt(&gram, &c, Language::ChineseSimplified);
        assert!(prompt["candidates"]["readings"][0].get("senses").is_none());
        assert_eq!(
            prompt["candidates"]["atoms"][1]["readings"][0]["senses"],
            serde_json::json!(["I", "me"])
        );
    }

    #[test]
    fn inflected_japanese_atoms_take_closed_candidates_from_their_lemma() {
        let dictionary = BTreeMap::from([(
            "入る".into(),
            vec![
                entry("はいる", &["to enter"]),
                entry("いる", &["to get in", "to go in"]),
            ],
        )]);
        for (surface, expected) in [
            ("入った", ["いった", "はいった"]),
            ("入っ", ["いっ", "はいっ"]),
        ] {
            let gram = word(surface, "入る", PartOfSpeech::Verb);
            let c = candidates(&gram, Language::Japanese, &dictionary);
            assert!(!c.open, "{surface}");
            assert_eq!(c.readings, BTreeSet::from(expected.map(String::from)));
            assert_eq!(c.senses[expected[1]], ["to enter"]);
            assert!(needs_hierarchy(&gram, &c, 1));
        }
        // A lemma spelled with other kanji shares no written stem.
        let c = candidates(
            &word("這入った", "入る", PartOfSpeech::Verb),
            Language::Japanese,
            &dictionary,
        );
        assert!(c.open);
    }

    #[test]
    fn only_one_reading_with_one_sense_skips_b() {
        let dictionary = BTreeMap::from([
            ("猫".into(), vec![entry("ねこ", &["cat"])]),
            ("気".into(), vec![entry("き", &["spirit", "mood", "air"])]),
            ("朋友".into(), vec![entry("péng you", &["friend"])]),
            ("看".into(), vec![entry("kàn", &["to see", "to read"])]),
        ]);
        for (language, text, needs) in [
            (Language::Japanese, "猫", false),
            (Language::Japanese, "気", true),
            (Language::ChineseSimplified, "朋友", false),
            (Language::ChineseSimplified, "看", true),
        ] {
            let gram = gram(&[text]);
            assert_eq!(
                needs_hierarchy(&gram, &candidates(&gram, language, &dictionary), 100),
                needs,
                "{text}"
            );
        }
    }

    /// Renders B requests from the real cached dictionaries and measures how
    /// much they grew. Run by hand after reading_candidates.py:
    /// `cargo test -p generate-data --release real_dictionary_b_prompts -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn real_dictionary_b_prompts() {
        let load = |language: Language| -> Dictionary {
            let path = format!("../.cache/readings/{}-dictionary.json", language.code());
            serde_json::from_reader(std::io::BufReader::new(std::fs::File::open(path).unwrap()))
                .unwrap()
        };
        let render = |language, gram: &Gram<String>, dictionary: &Dictionary| {
            let c = candidates(gram, language, dictionary);
            prompts::hierarchy_prompt(gram, &c, language)
        };
        let samples = [
            (Language::ChineseSimplified, gram(&["东西"])),
            (Language::Japanese, gram(&["方"])),
            (Language::Japanese, gram(&["猫", "の"])),
            (
                Language::Japanese,
                word("入った", "入る", PartOfSpeech::Verb),
            ),
        ];
        for language in [Language::ChineseSimplified, Language::Japanese] {
            let dictionary = load(language);
            for (_, gram) in samples.iter().filter(|(l, _)| *l == language) {
                println!(
                    "=== {} ===\n{}",
                    gram.to_display_string(language),
                    serde_json::to_string_pretty(&render(language, gram, &dictionary)).unwrap()
                );
            }
            // Growth over a sample of real B requests from the last paid run.
            // The old request had candidate readings without senses, every
            // atom's lemma readings, and the meaning-discovery usages.
            let file = std::io::BufReader::new(
                std::fs::File::open(format!(
                    "../.cache/readings/{}-hierarchies.jsonl",
                    language.code()
                ))
                .unwrap(),
            );
            let tokens = |text: &str| {
                let cjk = text.chars().filter(|&c| c > '\u{2e80}').count();
                cjk + (text.chars().count() - cjk) / 4
            };
            let (mut old_total, mut new_total, mut sampled) = (0usize, 0usize, 0usize);
            let mut seen = BTreeSet::new();
            for line in std::io::BufRead::lines(file).step_by(97).take(4000) {
                let row: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
                let gram: Gram<String> = serde_json::from_value(row["gram"].clone()).unwrap();
                if row["round"] != "initial" || !seen.insert(gram.clone()) {
                    continue;
                }
                let examples: Vec<Occurrence> = row["examples"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|o| Occurrence {
                        text: o["text"].as_str().unwrap().into(),
                        gram_index: 0,
                        bytes: (
                            o["bytes"][0].as_u64().unwrap() as usize,
                            o["bytes"][1].as_u64().unwrap() as usize,
                        ),
                        spans: vec![],
                    })
                    .collect();
                let mut new = render(language, &gram, &dictionary);
                new["examples"] = prompts::numbered_examples(&examples);
                let mut old = new.clone();
                for reading in old["candidates"]["readings"].as_array_mut().unwrap() {
                    reading.as_object_mut().unwrap().remove("senses");
                }
                for (atom, word) in old["candidates"]["atoms"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .zip(gram.iter().filter_map(|a| match a {
                        Atom::Tok(w) => Some(w),
                        _ => None,
                    }))
                {
                    let atom = atom.as_object_mut().unwrap();
                    atom.remove("readings");
                    let lemma = match &word.word_type {
                        WordType::Heteronym(h) => Some(&h.lemma),
                        _ => None,
                    };
                    let readings: Vec<_> = lemma
                        .and_then(|l| dictionary.get(l))
                        .into_iter()
                        .flatten()
                        .map(|r| r.reading.clone())
                        .collect();
                    atom.insert("lemma_readings".into(), serde_json::json!(readings));
                }
                old["usages"] = serde_json::json!([]);
                old_total += tokens(&old.to_string());
                new_total += tokens(&new.to_string());
                sampled += 1;
            }
            println!(
                "{}: sampled {sampled} grams; mean user prompt ≈{} → ≈{} tokens; system prompt ≈{} tokens",
                language.code(),
                old_total / sampled.max(1),
                new_total / sampled.max(1),
                tokens(&prompts::hierarchy_system_prompt(language)),
            );
        }
    }

    #[test]
    fn failure_rate_uses_the_full_population_for_repair() {
        assert!(!hierarchy_failure_limit_exceeded(91, 48_236));
        assert!(!hierarchy_failure_limit_exceeded(1, 100));
        assert!(hierarchy_failure_limit_exceeded(2, 100));
        assert!(!hierarchy_failure_limit_exceeded(0, 0));
    }

    #[test]
    fn headword_and_atom_candidates_are_unioned_and_inflections_stay_open() {
        let dictionary = BTreeMap::from([
            ("入院".into(), vec!["にゅういん".into()]),
            ("中".into(), vec!["なか".into(), "ちゅう".into()]),
            ("入院中".into(), vec!["にゅういんちゅう".into()]),
            ("食べる".into(), vec!["たべる".into()]),
        ]);
        let c = candidates(&gram(&["入院", "中"]), Language::Japanese, &dictionary);
        assert!(!c.open);
        assert_eq!(
            c.readings,
            BTreeSet::from(["にゅういんちゅう".into(), "にゅういんなか".into()])
        );
        assert!(c.prior_single);
        assert!(needs_hierarchy(&gram(&["入院", "中"]), &c, 1));
        assert!(candidates(&gram(&["食べ"]), Language::Japanese, &dictionary).open);
    }
    #[test]
    fn multi_atom_headword_matches_do_not_hide_compositional_readings() {
        let dictionary = BTreeMap::from([
            (
                "今日は".into(),
                vec!["こんにちは".into(), "こんにちわ".into()],
            ),
            ("今日".into(), vec!["きょう".into()]),
            ("彼".into(), vec!["かれ".into()]),
        ]);
        let g = gram(&["今日", "は"]);
        let c = candidates(&g, Language::Japanese, &dictionary);
        assert_eq!(
            c.readings,
            BTreeSet::from(["きょうは".into(), "こんにちは".into()])
        );
        assert!(c.prior_single);
        assert!(needs_hierarchy(&g, &c, 1));
        let g = gram(&["彼", "の"]);
        let c = candidates(&g, Language::Japanese, &dictionary);
        assert_eq!(c.readings.len(), 1);
        assert!(needs_hierarchy(&g, &c, 1));
        let g = gram(&["彼"]);
        assert!(!needs_hierarchy(
            &g,
            &candidates(&g, Language::Japanese, &dictionary),
            1
        ));
        assert!(candidates(&gram(&["ヶ"]), Language::Japanese, &dictionary).open);
    }

    #[test]
    fn invalid_candidates_are_filtered_before_b_and_singleton_bypass() {
        let dictionary = BTreeMap::from([
            (
                "吧".into(),
                vec!["ba".into(), "biā".into(), "bā".into(), "hello".into()],
            ),
            ("边".into(), vec!["biān".into()]),
            ("表".into(), vec!["biǎo".into()]),
            ("你好".into(), vec!["nǐ".into()]),
        ]);
        let c = candidates(&gram(&["吧"]), Language::ChineseSimplified, &dictionary);
        assert_eq!(
            c.readings,
            BTreeSet::from(["ba".into(), "biā".into(), "bā".into()])
        );
        let proposal = Proposal {
            reopen: false,
            senses: vec![ProposedSense {
                grounding: Default::default(),
                kind: UsageKind::Meaning,
                gloss: "dictionary evidence".into(),
                readings: c
                    .readings
                    .iter()
                    .enumerate()
                    .map(|(index, r)| ProposedReading {
                        index: Some(index),
                        reading: r.clone(),
                        example_line: None,
                    })
                    .collect(),
            }],
        };
        let request = HierarchyRequest {
            gram: gram(&["吧"]),
            candidates: c,
            examples: vec![],
            prompt: serde_json::Value::Null,
        };
        assert!(
            prepare_proposal(
                proposal,
                &request,
                Language::ChineseSimplified,
                &kanjidic::Verifier::default(),
                &mut Rejections::default()
            )
            .is_some()
        );
        for text in ["边", "表"] {
            assert!(!candidates(&gram(&[text]), Language::ChineseSimplified, &dictionary).open);
        }
        let c = candidates(&gram(&["你好"]), Language::ChineseSimplified, &dictionary);
        assert!(c.open);
        assert!(needs_hierarchy(&gram(&["你好"]), &c, 1));
    }
    #[test]
    fn candidates_keep_tones_distinct_until_meaning_assignment() {
        let dictionary = BTreeMap::from([
            ("街".into(), vec!["jiē".into()]),
            ("上".into(), vec!["shàng".into()]),
            ("街上".into(), vec!["jiē shang".into()]),
            ("陌".into(), vec!["mò".into()]),
            ("东".into(), vec!["dōng".into()]),
            ("西".into(), vec!["xī".into()]),
            ("东西".into(), vec!["dōng xi".into(), "dōng xī".into()]),
            ("告".into(), vec!["gào".into()]),
            ("诉".into(), vec!["sù".into()]),
            ("告诉".into(), vec!["gào su".into(), "gào sù".into()]),
        ]);
        for (words, expected) in [
            (vec!["街", "上"], vec!["jiē shang", "jiē shàng"]),
            (vec!["陌", "上"], vec!["mò shàng"]),
            (vec!["东", "西"], vec!["dōng xi", "dōng xī"]),
            (vec!["告", "诉"], vec!["gào su", "gào sù"]),
        ] {
            assert_eq!(
                candidates(&gram(&words), Language::ChineseSimplified, &dictionary).readings,
                expected.into_iter().map(str::to_owned).collect()
            );
        }
    }
    #[test]
    fn dictionary_candidates_cannot_copy_the_lemma_reading() {
        let dictionary =
            BTreeMap::from([("入った".into(), vec!["はいる".into(), "はいった".into()])]);
        let c = candidates(&gram(&["入った"]), Language::Japanese, &dictionary);
        assert_eq!(c.readings, BTreeSet::from(["はいった".into()]));
        assert!(!c.open);
        let dictionary = BTreeMap::from([(
            "香具師".into(),
            vec!["やし".into(), "やつ".into(), "こうぐし".into()],
        )]);
        let c = candidates(&gram(&["香具師"]), Language::Japanese, &dictionary);
        assert_eq!(
            c.readings,
            BTreeSet::from(["やし".into(), "やつ".into(), "こうぐし".into()])
        );
    }

    #[test]
    fn whole_gram_schema_and_guards_reject_syllable_lists() {
        let parsed: ProposedReading = serde_json::from_str(
            r#"{"index":null,"whole_gram_reading":"nǐ hǎo ma","example_line":0}"#,
        )
        .unwrap();
        assert_eq!(parsed.reading, "nǐ hǎo ma");
        assert!(
            serde_json::from_str::<ProposedReading>(r#"{"index":null,"reading":"nǐ"}"#).is_err()
        );
        let mut request = HierarchyRequest {
            gram: gram(&["你", "好", "吗"]),
            candidates: Candidates {
                senses: BTreeMap::new(),
                evidence: BTreeMap::new(),
                coverings: BTreeMap::new(),
                whole_word_readings: BTreeSet::new(),
                readings: BTreeSet::new(),
                open: true,
                atoms: vec![],
                prior_single: false,
            },
            prompt: serde_json::Value::Null,
            examples: vec![example()],
        };
        let mut proposal = Proposal {
            reopen: false,
            senses: vec![ProposedSense {
                grounding: Default::default(),
                kind: UsageKind::Meaning,
                gloss: "greeting".into(),
                readings: ["nǐ", "hǎo", "ma"]
                    .into_iter()
                    .map(|r| ProposedReading {
                        index: None,
                        reading: r.into(),
                        example_line: Some(0),
                    })
                    .collect(),
            }],
        };
        let mut checks = Rejections::default();
        assert!(
            prepare_proposal(
                proposal.clone(),
                &request,
                Language::ChineseSimplified,
                &kanjidic::Verifier::default(),
                &mut checks
            )
            .is_none()
        );
        assert_eq!(checks.canonical_readings, 3);
        proposal.senses[0].readings = vec![ProposedReading {
            index: None,
            reading: "nǐ hǎo ma".into(),
            example_line: Some(0),
        }];
        assert!(
            prepare_proposal(
                proposal.clone(),
                &request,
                Language::ChineseSimplified,
                &kanjidic::Verifier::default(),
                &mut Rejections::default()
            )
            .is_some()
        );
        request.gram = gram(&["香具師"]);
        proposal.senses[0].readings[0].reading = "やし".into();
        let mut checks = Rejections::default();
        assert!(
            prepare_proposal(
                proposal,
                &request,
                Language::Japanese,
                &kanji(),
                &mut checks
            )
            .is_some()
        );
    }

    #[test]
    fn index_agreement_and_okurigana_have_separate_counts() {
        let mut request = HierarchyRequest {
            gram: gram(&["娘"]),
            candidates: Candidates {
                senses: BTreeMap::new(),
                evidence: BTreeMap::new(),
                coverings: BTreeMap::new(),
                whole_word_readings: BTreeSet::new(),
                readings: BTreeSet::from(["こ".into(), "むすめ".into()]),
                open: false,
                atoms: vec![],
                prior_single: false,
            },
            prompt: serde_json::Value::Null,
            examples: vec![example()],
        };
        let mut proposal = Proposal {
            reopen: false,
            senses: vec![ProposedSense {
                grounding: Default::default(),
                kind: UsageKind::Meaning,
                gloss: "娘".into(),
                readings: vec![ProposedReading {
                    index: Some(0),
                    reading: "むすめ".into(),
                    example_line: Some(0),
                }],
            }],
        };
        let mut checks = Rejections::default();
        assert!(
            prepare_proposal(
                proposal.clone(),
                &request,
                Language::Japanese,
                &kanji(),
                &mut checks
            )
            .is_none()
        );
        assert_eq!(checks.index_disagreements, 1);
        assert_eq!(checks.okurigana_readings, 0);
        request.gram = gram(&["入った"]);
        request.candidates.open = true;
        proposal.senses[0].readings[0].index = None;
        proposal.senses[0].readings[0].reading = "はいる".into();
        let mut checks = Rejections::default();
        assert!(
            prepare_proposal(
                proposal,
                &request,
                Language::Japanese,
                &kanji(),
                &mut checks
            )
            .is_none()
        );
        assert_eq!(checks.okurigana_readings, 1);
        assert_eq!(checks.index_disagreements, 0);
    }

    #[test]
    fn candidate_cartesian_product() {
        let dictionary = BTreeMap::from([
            ("方".into(), vec!["ほう".into(), "かた".into()]),
            ("中".into(), vec!["なか".into(), "ちゅう".into()]),
        ]);
        let c = candidates(&gram(&["方", "中"]), Language::Japanese, &dictionary);
        assert_eq!(c.readings.len(), 4);
        assert!(c.readings.contains("かたちゅう"));
        assert!(!c.open);
    }
    #[test]
    fn each_senses_candidate_indices_must_agree_with_their_text() {
        let request = HierarchyRequest {
            gram: gram(&["何"]),
            candidates: Candidates {
                senses: BTreeMap::new(),
                evidence: BTreeMap::new(),
                coverings: BTreeMap::new(),
                whole_word_readings: BTreeSet::new(),
                readings: BTreeSet::from(["なに".into(), "なん".into()]),
                open: false,
                atoms: vec![],
                prior_single: false,
            },
            prompt: serde_json::Value::Null,
            examples: vec![example()],
        };
        let mut proposal = Proposal {
            reopen: false,
            senses: vec![
                ProposedSense {
                    grounding: Default::default(),
                    kind: UsageKind::Meaning,
                    gloss: "what".into(),
                    readings: vec![ProposedReading {
                        index: Some(0),
                        reading: "なに".into(),
                        example_line: None,
                    }],
                },
                ProposedSense {
                    grounding: Default::default(),
                    kind: UsageKind::Construction,
                    gloss: "which".into(),
                    readings: vec![ProposedReading {
                        index: Some(1),
                        reading: "なん".into(),
                        example_line: None,
                    }],
                },
            ],
        };
        let prepared = prepare_proposal(
            proposal.clone(),
            &request,
            Language::Japanese,
            &kanji(),
            &mut Rejections::default(),
        )
        .unwrap();
        assert_eq!(prepared.senses[0].readings[0].reading, "なに");
        assert_eq!(prepared.senses[1].readings[0].reading, "なん");
        proposal.senses[0].readings[0].index = Some(1);
        assert!(
            prepare_proposal(
                proposal.clone(),
                &request,
                Language::Japanese,
                &kanji(),
                &mut Rejections::default()
            )
            .is_none()
        );
        proposal.senses[0].readings[0].index = Some(0);
        proposal.senses[1].readings[0].index = Some(2);
        assert!(
            prepare_proposal(
                proposal,
                &request,
                Language::Japanese,
                &kanji(),
                &mut Rejections::default()
            )
            .is_none()
        );
    }

    #[test]
    fn whole_word_dictionary_readings_bypass_kanjidic_even_when_reopened() {
        let g = gram(&["一日"]);
        let request = HierarchyRequest {
            candidates: candidates(
                &g,
                Language::Japanese,
                &BTreeMap::from([("一日".into(), vec!["ついたち".into()])]),
            ),
            gram: g,
            prompt: serde_json::Value::Null,
            examples: vec![example()],
        };
        let mut proposal = Proposal {
            reopen: true,
            senses: vec![ProposedSense {
                grounding: Default::default(),
                kind: UsageKind::Meaning,
                gloss: "月の第一日".into(),
                readings: vec![ProposedReading {
                    index: None,
                    reading: "ついたち".into(),
                    example_line: Some(0),
                }],
            }],
        };
        assert!(
            prepare_proposal(
                proposal.clone(),
                &request,
                Language::Japanese,
                &kanjidic::Verifier::default(),
                &mut Rejections::default()
            )
            .is_some()
        );
        proposal.senses[0].readings[0].reading = "ひいちにち".into();
        let mut checks = Rejections::default();
        assert!(
            prepare_proposal(
                proposal,
                &request,
                Language::Japanese,
                &kanjidic::Verifier::default(),
                &mut checks
            )
            .is_some()
        );
        assert_eq!(checks.kanjidic_readings, 1);
    }

    #[test]
    fn closed_gram_can_reopen_for_a_missing_name_reading() {
        let request = HierarchyRequest {
            gram: gram(&["硝子"]),
            candidates: Candidates {
                senses: BTreeMap::new(),
                evidence: BTreeMap::new(),
                coverings: BTreeMap::new(),
                whole_word_readings: BTreeSet::new(),
                readings: BTreeSet::from(["がらす".into()]),
                open: false,
                atoms: vec![],
                prior_single: false,
            },
            prompt: serde_json::Value::Null,
            examples: vec![example()],
        };
        let proposal = Proposal {
            reopen: true,
            senses: vec![ProposedSense {
                grounding: Default::default(),
                kind: UsageKind::Meaning,
                gloss: "人名".into(),
                readings: vec![ProposedReading {
                    index: None,
                    reading: "ショウコ".into(),
                    example_line: Some(0),
                }],
            }],
        };
        assert_eq!(
            prepare_proposal(
                proposal,
                &request,
                Language::Japanese,
                &kanji(),
                &mut Rejections::default()
            )
            .unwrap()
            .senses[0]
                .readings[0]
                .reading,
            "しょうこ"
        );
    }

    #[test]
    fn occurrence_indices_resolve_without_copying_reading_text() {
        let hierarchy = vec![SenseReadings {
            index: 2,
            kind: "meaning".into(),
            synthetic_readings: BTreeSet::new(),
            gloss: "what".into(),
            readings: vec!["なに".into(), "なん".into()],
        }];
        let choices = vec![
            IndexedChoice {
                sense: 2,
                reading: 1,
            },
            IndexedChoice {
                sense: 2,
                reading: 0,
            },
        ];
        assert_eq!(
            resolve_choices(&choices, &hierarchy)
                .unwrap()
                .iter()
                .map(|c| c.reading.as_str())
                .collect::<Vec<_>>(),
            ["なん", "なに"]
        );
        assert!(
            resolve_choices(
                &[IndexedChoice {
                    sense: 1,
                    reading: 0
                }],
                &hierarchy
            )
            .is_none()
        );
        assert!(
            resolve_choices(
                &[IndexedChoice {
                    sense: 2,
                    reading: 2
                }],
                &hierarchy
            )
            .is_none()
        );
    }

    #[test]
    fn rejected_text_is_removed_from_all_corpus_views_and_generated_inputs() {
        let mut data = CorpusReadings::default();
        data.rejected_sentences.insert("drop".into());
        let rows = || {
            ["keep", "drop"]
                .into_iter()
                .map(|s| (s.into(), None, language_utils::SentenceSource::none()))
                .collect()
        };
        let mut corpus = crate::target_sentences::TargetSentences {
            books: Default::default(),
            movie_varieties: Default::default(),
            app_sentences: rows(),
            book_sentences: rows(),
            restricted_sentences: vec![("keep".into(), vec![]), ("drop".into(), vec![])],
        };
        let nlp = || {
            ["keep", "drop"]
                .into_iter()
                .map(|s| {
                    (
                        s.into(),
                        SentenceInfo {
                            sentence: language_utils::EncodedSentence {
                                tokens: vec![],
                                capitalize_first: false,
                            },
                            multiword_terms: language_utils::MultiwordTerms {
                                high_confidence: vec![],
                                low_confidence: vec![],
                            },
                        },
                    )
                })
                .collect()
        };
        let (mut app, mut restricted) = (nlp(), nlp());
        data.retain_corpus(&mut corpus, &mut app, &mut restricted);
        assert_eq!(
            (
                corpus.app_sentences.len(),
                corpus.book_sentences.len(),
                corpus.restricted_sentences.len()
            ),
            (1, 1, 1)
        );
        assert_eq!(app.keys().map(String::as_str).collect::<Vec<_>>(), ["keep"]);
        assert_eq!(
            restricted.keys().map(String::as_str).collect::<Vec<_>>(),
            ["keep"]
        );
        let generated: Vec<_> = ["drop", "new"]
            .into_iter()
            .filter(|s| data.includes_sentence(s))
            .collect();
        assert_eq!(generated, ["new"]);
    }

    #[test]
    fn missing_c_labels_never_borrow_a_reading_from_sense_frequency() {
        let mut data = CorpusReadings::default();
        let gram = gram(&["何"]);
        assert_eq!(data.reading("何", 0), None);
        data.hierarchies.insert(
            gram.clone(),
            vec![SenseReadings {
                index: 1,
                kind: "meaning".into(),
                synthetic_readings: BTreeSet::new(),
                gloss: "what".into(),
                readings: vec!["なに".into(), "なん".into()],
            }],
        );
        assert_eq!(data.sense("何", 0), None);
        assert_eq!(data.reading("何", 0), None);
    }
}
