//! English heteronyms (read, lead, tear, record): each occurrence gets the
//! pronunciation it has in its sentence, for g2p to say. Part of speech,
//! lemma, capitalization and the occurrence's sense settle most of them; a
//! few grammar cues settle most of read's tenses; only what is left asks the
//! model. English readings are never shown: English text has no ruby base.
//!
//! A gram's reading is one segment per word, joined by ` | `, empty for a
//! word that is not a heteronym (`ɹ ˈɛ d | ` for "read it"), each segment in
//! g2p's phone notation.
use super::*;
use language_utils::Whitespace;

/// The heteronym table (`data/eng/heteronyms.json`): per lowercase spelling,
/// its pronunciations, each with the tags it needs.
pub struct Heteronyms(BTreeMap<String, Vec<Entry>>);

#[derive(Debug, Deserialize)]
pub struct Entry {
    /// lexide tags (`PROPN` included) this pronunciation can have.
    pos: Vec<String>,
    lemma: Option<String>,
    /// Must the word be (or not be) capitalized? Unknown for a sentence's
    /// first word, which then matches either way.
    capitalized: Option<bool>,
    pub reading: String,
    pub gloss: String,
}

/// What the tags say about one word.
#[derive(Debug)]
pub enum Resolution<'a> {
    /// Not a heteronym here (no entry matches its tags).
    None,
    Settled(&'a str),
    /// Several pronunciations remain; the model chooses.
    Ask(Vec<&'a Entry>),
}

const SEPARATOR: &str = " | ";

fn tag(word: &language_utils::Word<String>) -> (String, Option<&str>) {
    match &word.word_type {
        WordType::Heteronym(h) => (
            serde_json::to_value(h.pos)
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned(),
            Some(h.lemma.as_str()),
        ),
        WordType::Other(o) => (
            serde_json::to_value(o.other_tag)
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned(),
            None,
        ),
    }
}

impl Heteronyms {
    pub fn load() -> Result<Self> {
        Self::load_from("generate-data/data/eng/heteronyms.json")
    }

    fn load_from(path: &str) -> Result<Self> {
        Ok(Self(serde_json::from_reader(std::fs::File::open(path)?)?))
    }

    /// The pronunciation of `words[i]`, as far as its tags, its sense's
    /// readings and its neighbours settle it.
    pub fn resolve<'a>(
        &'a self,
        words: &[Literal<String>],
        i: usize,
        sense_readings: Option<&[String]>,
    ) -> Resolution<'a> {
        let word = &words[i].word;
        let Some(entries) = self.0.get(&word.text.to_lowercase()) else {
            return Resolution::None;
        };
        // espeak reads a word with its clitic as one word ("doesn't"), and
        // with it the heteronym is never in doubt.
        if words[i].whitespace == Whitespace::None
            && words.get(i + 1).is_some_and(|next| {
                next.word
                    .text
                    .starts_with(|c: char| c.is_alphabetic() || matches!(c, '\'' | '’'))
            })
        {
            return Resolution::None;
        }
        let (pos, lemma) = tag(word);
        let capitalized = word.text.starts_with(char::is_uppercase);
        let mut candidates: Vec<&Entry> = entries
            .iter()
            .filter(|e| e.pos.contains(&pos))
            .filter(|e| {
                e.lemma
                    .as_deref()
                    .is_none_or(|l| lemma.is_some_and(|lemma| lemma.eq_ignore_ascii_case(l)))
            })
            .filter(|e| i == 0 || e.capitalized.is_none_or(|c| c == capitalized))
            .collect();
        if let Some(readings) = sense_readings {
            candidates.retain(|e| readings.contains(&e.reading));
        }
        let mut distinct: Vec<_> = candidates.iter().map(|e| e.reading.as_str()).collect();
        distinct.sort();
        distinct.dedup();
        match distinct[..] {
            [] => Resolution::None,
            [reading] => Resolution::Settled(reading),
            _ => match grammar_cue(words, i) {
                Some(cue) => candidates
                    .iter()
                    .find(|e| e.gloss.starts_with(cue))
                    .map_or(Resolution::Ask(candidates.clone()), |e| {
                        Resolution::Settled(&e.reading)
                    }),
                None => Resolution::Ask(candidates),
            },
        }
    }
}

/// The nearest preceding word that is not an adverb, negation or subject
/// pronoun, within three words.
fn governor(words: &[Literal<String>], i: usize) -> Option<String> {
    const SKIP: &[&str] = &[
        "not", "n't", "never", "already", "just", "ever", "also", "really", "all", "often",
        "always", "even", "still", "only", "i", "you", "we", "they", "he", "she", "it",
    ];
    words[i.saturating_sub(3)..i]
        .iter()
        .rev()
        .map(|l| l.word.text.to_lowercase())
        .find(|w| !SKIP.contains(&w.as_str()))
}

/// Which gloss the grammar around a read-type verb, "use", "used" or
/// "supposed" selects: a perfect or passive auxiliary takes the past
/// participle, a modal, `to` or `do` the base form, and he or she without one
/// the past (present would be "reads"); "used to" and "supposed to" take
/// their `to` form, and so does "did use to".
fn grammar_cue(words: &[Literal<String>], i: usize) -> Option<&'static str> {
    const PAST: &[&str] = &[
        "have", "has", "had", "'ve", "having", "is", "are", "was", "were", "be", "been", "being",
        "'re", "'m", "get", "got", "gets",
    ];
    const BASE: &[&str] = &[
        "to", "will", "'ll", "can", "ca", "could", "should", "would", "must", "may", "might",
        "shall", "do", "does", "did", "let", "please", "wo",
    ];
    let word = words[i].word.text.to_lowercase();
    let previous = |n: usize| i.checked_sub(n).map(|j| words[j].word.text.to_lowercase());
    let to = words
        .get(i + 1)
        .is_some_and(|l| l.word.text.eq_ignore_ascii_case("to"));
    if word == "used" {
        // "is used to cut" is the past participle, "is used to it" the to form.
        let be = previous(1).is_some_and(|w| PAST.contains(&w.as_str()));
        return match (to, be) {
            (true, false) => Some("with to"),
            (true, true) => None,
            (false, _) => Some("past"),
        };
    }
    if word == "supposed" {
        return Some(if to { "with to" } else { "past" });
    }
    if word == "use" {
        let did = governor(words, i).is_some_and(|w| w == "did");
        return Some(if to && did { "with to" } else { "verb" });
    }
    if !word.ends_with("read") {
        return None;
    }
    if let Some(governor) = governor(words, i) {
        if PAST.contains(&governor.as_str()) {
            return Some("past");
        }
        if BASE.contains(&governor.as_str()) {
            return Some("present");
        }
    }
    previous(1)
        .is_some_and(|w| matches!(w.as_str(), "he" | "she"))
        .then_some("past")
}

#[derive(Serialize)]
pub struct Request {
    marked_sentence: String,
    choices: Vec<Choice>,
    #[serde(skip)]
    readings: Vec<String>,
}

#[derive(Serialize)]
struct Choice {
    index: usize,
    meaning: String,
    pronunciation: String,
}

#[derive(Deserialize, JsonSchema)]
struct Answer {
    /// Index of the pronunciation the marked word has in this sentence.
    choice: usize,
}

const SYSTEM: &str = "Choose how the marked English word is pronounced in its sentence. We insert « and » around the word to show which one to judge; they are not part of the original text. The choices give each pronunciation with the meaning or form it belongs to, in IPA. Judge this occurrence in context (tense, meaning, what the sentence is about), not the word's most common pronunciation. Return the index of the choice.";

impl Request {
    pub fn new(text: &str, bytes: (usize, usize), candidates: &[&Entry]) -> Self {
        let mut choices: Vec<Choice> = Vec::new();
        let mut readings = Vec::new();
        for entry in candidates {
            if let Some(i) = readings.iter().position(|r| r == &entry.reading) {
                let meaning: &mut String = &mut choices[i].meaning;
                meaning.push_str("; ");
                meaning.push_str(&entry.gloss);
                continue;
            }
            choices.push(Choice {
                index: choices.len(),
                meaning: entry.gloss.clone(),
                pronunciation: entry.reading.replace(' ', ""),
            });
            readings.push(entry.reading.clone());
        }
        Self {
            marked_sentence: format!(
                "{}«{}»{}",
                &text[..bytes.0],
                &text[bytes.0..bytes.1],
                &text[bytes.1..]
            ),
            choices,
            readings,
        }
    }
}

/// The model's choice for each request (None when it gave no valid answer).
/// Batched unless `live` (small samples that should not wait for a batch).
pub async fn ask(requests: &[Request], live: bool) -> Result<Vec<Option<String>>> {
    if requests.is_empty() {
        return Ok(Vec::new());
    }
    let mut client = crate::base_chat_client("gpt-6-luna").with_reasoning_effort("low");
    if live {
        client = client.with_no_batch();
    }
    let answers = batch::<_, Answer>(&client, SYSTEM, requests, |r| {
        serde_json::to_string(r).unwrap()
    })
    .await?;
    Ok(answers
        .into_iter()
        .map(|(request, answer)| request.readings.get(answer.ok()?.choice).cloned())
        .collect())
}

/// Byte ranges of each decoded word in `text`.
fn word_bytes(text: &str, words: &[Literal<String>]) -> Vec<(usize, usize)> {
    let mut byte = 0;
    words
        .iter()
        .map(|literal| {
            let start = byte;
            byte += literal.word.text.len();
            assert_eq!(
                &text[start..byte],
                literal.word.text,
                "words spell the text"
            );
            byte += literal.whitespace.as_str().len();
            (start, byte - literal.whitespace.as_str().len())
        })
        .collect()
}

impl CorpusReadings {
    /// Run after sense assignment, so an occurrence's sense can settle its
    /// pronunciation (tear "drop from the eye" vs "rip").
    pub async fn read_english_heteronyms(
        &mut self,
        language: Language,
        app: &BTreeMap<String, SentenceInfo>,
        restricted: &BTreeMap<String, SentenceInfo>,
        interners: &GramInterners,
        inventories: &BTreeMap<Gram<String>, UsageInventory>,
    ) -> Result<()> {
        if language != Language::English {
            return Ok(());
        }
        let table = Heteronyms::load()?;
        // Per occurrence (its key and sense), each word's pronunciation; a
        // question leaves a placeholder that its answer fills.
        type Occurrence = ((String, usize), u32, Vec<Option<String>>);
        let mut occurrences: Vec<Occurrence> = Vec::new();
        let mut requests = Vec::new();
        let mut asked = Vec::new();
        let (mut settled, mut asking) = (0usize, 0usize);
        for (text, info) in app.iter().chain(
            restricted
                .iter()
                .filter(|(text, _)| !app.contains_key(*text)),
        ) {
            let words = info.decode_words(interners, language);
            let bytes = word_bytes(text, &words);
            let mut word = 0;
            for (index, token) in info.sentence.tokens.iter().enumerate() {
                let gram = interners
                    .grams
                    .resolve(&token.gram)
                    .resolve(&interners.strings);
                let count = gram.iter().filter(|a| matches!(a, Atom::Tok(_))).count();
                let sense_readings = (count == 1)
                    .then(|| {
                        let usage = inventories
                            .get(&gram)?
                            .usages
                            .get(token.sense?.get() as usize - 1)?;
                        usage.readings.as_deref()
                    })
                    .flatten();
                let mut readings = Vec::with_capacity(count);
                let mut asks = false;
                for (i, &span) in bytes.iter().enumerate().skip(word).take(count) {
                    readings.push(match table.resolve(&words, i, sense_readings) {
                        Resolution::None => None,
                        Resolution::Settled(reading) => {
                            settled += 1;
                            Some(reading.to_owned())
                        }
                        Resolution::Ask(candidates) => {
                            asking += 1;
                            asks = true;
                            asked.push((occurrences.len(), i - word));
                            requests.push(Request::new(text, span, &candidates));
                            None
                        }
                    });
                }
                word += count;
                if asks || readings.iter().any(Option::is_some) {
                    occurrences.push((
                        (text.clone(), index),
                        token.sense.map_or(0, NonZeroU32::get),
                        readings,
                    ));
                }
            }
        }
        println!("readings[eng]: heteronym words settled by tags={settled}, asked={asking}");
        for ((occurrence, word), answer) in asked.into_iter().zip(ask(&requests, false).await?) {
            occurrences[occurrence].2[word] = answer;
        }
        for (key, sense, readings) in occurrences {
            if readings.iter().all(Option::is_none) {
                continue;
            }
            let reading = readings
                .iter()
                .map(|r| r.as_deref().unwrap_or_default())
                .collect::<Vec<_>>()
                .join(SEPARATOR);
            self.labels.insert(
                key,
                occurrence::Label::Ranked {
                    choices: vec![super::Choice { sense, reading }],
                },
            );
        }
        Ok(())
    }
}

/// A gram's reading split into its words' readings (empty: not given).
pub fn word_readings(reading: &str) -> impl Iterator<Item = &str> {
    reading.split(SEPARATOR)
}

#[cfg(test)]
mod tests {
    use super::*;
    use language_utils::{Heteronym, OtherWord, OtherWordType, Whitespace, Word};

    /// "text/POS/lemma" words; a lemma of `-` is the text itself.
    fn words(spec: &str) -> Vec<Literal<String>> {
        spec.split(' ')
            .map(|w| {
                let mut parts = w.split('/');
                let text = parts.next().unwrap().to_owned();
                let pos = parts.next().unwrap();
                let lemma = parts.next().map_or(text.to_lowercase(), str::to_owned);
                let word_type = if pos == "PROPN" {
                    WordType::Other(OtherWord {
                        other_tag: OtherWordType::Propn,
                    })
                } else {
                    WordType::Heteronym(Heteronym {
                        word: text.clone(),
                        lemma,
                        pos: serde_json::from_value(serde_json::json!(pos)).unwrap(),
                    })
                };
                Literal {
                    word: Word { text, word_type },
                    whitespace: Whitespace::Space,
                }
            })
            .collect()
    }

    fn resolve(spec: &str, word: &str) -> Option<String> {
        let table = Heteronyms::load_from("data/eng/heteronyms.json").unwrap();
        let words = words(spec);
        let i = words.iter().position(|l| l.word.text == word).unwrap();
        match table.resolve(&words, i, None) {
            Resolution::Settled(reading) => Some(reading.to_owned()),
            Resolution::Ask(_) => Some("?".into()),
            Resolution::None => None,
        }
    }

    #[test]
    fn tags_settle_most_heteronyms() {
        assert_eq!(
            resolve("I/PRON live/VERB here/ADV", "live").unwrap(),
            "l ˈɪ v"
        );
        assert_eq!(
            resolve("a/DET live/ADJ show/NOUN", "live").unwrap(),
            "l ˈaɪ v"
        );
        assert_eq!(
            resolve("we/PRON use/VERB it/PRON", "use").unwrap(),
            "j ˈuː z"
        );
        assert_eq!(resolve("no/DET use/NOUN", "use").unwrap(), "j ˈuː s");
        assert_eq!(
            resolve("they/PRON record/VERB songs/NOUN", "record").unwrap(),
            "ɹ ə k ˈɔːɹ d"
        );
        // Lemma: wound as past of wind vs injure.
        assert_eq!(
            resolve("he/PRON wound/VERB/wind it/PRON up/ADP", "wound").unwrap(),
            "w ˈaʊ n d"
        );
        assert_eq!(
            resolve("it/PRON wound/VERB/wound him/PRON", "wound").unwrap(),
            "w ˈuː n d"
        );
        // Capitalization, except on the first word.
        assert_eq!(
            resolve("we/PRON love/VERB Nice/PROPN", "Nice").unwrap(),
            "n ˈiː s"
        );
        assert_eq!(resolve("Nice/ADJ work/NOUN", "Nice").unwrap(), "n ˈaɪ s");
        assert_eq!(
            resolve("a/DET Polish/ADJ town/NOUN", "Polish").unwrap(),
            "p ˈoʊ l ɪ ʃ"
        );
        assert_eq!(
            resolve("Polish/VERB it/PRON", "Polish").unwrap(),
            "p ˈɑː l ɪ ʃ"
        );
        // A tag the table does not list is left to espeak.
        assert_eq!(resolve("the/DET live/INTJ", "live"), None);
        assert_eq!(resolve("the/DET cat/NOUN", "cat"), None);
    }

    #[test]
    fn grammar_settles_most_tenses_of_read() {
        assert_eq!(
            resolve("I/PRON have/AUX read/VERB it/PRON", "read").unwrap(),
            "ɹ ˈɛ d"
        );
        assert_eq!(
            resolve("have/AUX you/PRON read/VERB it/PRON", "read").unwrap(),
            "ɹ ˈɛ d"
        );
        assert_eq!(
            resolve("it/PRON was/AUX read/VERB aloud/ADV", "read").unwrap(),
            "ɹ ˈɛ d"
        );
        assert_eq!(
            resolve("I/PRON want/VERB to/PART read/VERB", "read").unwrap(),
            "ɹ ˈiː d"
        );
        assert_eq!(
            resolve("did/AUX n't/PART you/PRON read/VERB", "read").unwrap(),
            "ɹ ˈiː d"
        );
        assert_eq!(
            resolve("she/PRON read/VERB it/PRON", "read").unwrap(),
            "ɹ ˈɛ d"
        );
        assert_eq!(resolve("I/PRON read/VERB it/PRON", "read").unwrap(), "?");
        assert_eq!(
            resolve("a/DET good/ADJ read/NOUN", "read").unwrap(),
            "ɹ ˈiː d"
        );
        assert_eq!(
            resolve("I/PRON used/VERB to/PART go/VERB", "used").unwrap(),
            "j ˈuː s t"
        );
        assert_eq!(
            resolve("I/PRON used/VERB it/PRON", "used").unwrap(),
            "j ˈuː z d"
        );
        assert_eq!(
            resolve("did/AUX she/PRON use/VERB to/PART go/VERB", "use").unwrap(),
            "j ˈuː s"
        );
        assert_eq!(
            resolve("we/PRON use/VERB it/PRON", "use").unwrap(),
            "j ˈuː z"
        );
        assert_eq!(
            resolve("it/PRON is/AUX used/VERB to/PART cut/VERB", "used").unwrap(),
            "?"
        );
    }

    #[test]
    fn a_sense_settles_a_meaning_heteronym() {
        let table = Heteronyms::load_from("data/eng/heteronyms.json").unwrap();
        let words = words("a/DET tear/NOUN fell/VERB");
        assert!(matches!(table.resolve(&words, 1, None), Resolution::Ask(c) if c.len() == 2));
        let teardrop = ["t ˈɪɹ".to_owned()];
        assert!(matches!(
            table.resolve(&words, 1, Some(&teardrop)),
            Resolution::Settled("t ˈɪɹ")
        ));
    }

    #[test]
    fn every_reading_is_a_word_g2p_can_say() {
        let table = Heteronyms::load_from("data/eng/heteronyms.json").unwrap();
        for (form, entries) in &table.0 {
            for entry in entries {
                let readings = [g2p::Reading {
                    start: 0,
                    end: form.len(),
                    reading: entry.reading.clone(),
                }];
                let hints = g2p::Hints {
                    readings: &readings,
                    ..Default::default()
                };
                g2p::phonemize_with(g2p::Language::English, form, &hints)
                    .unwrap_or_else(|e| panic!("{form} {:?}: {e}", entry.reading));
            }
        }
    }
}
