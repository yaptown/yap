//! Phonemize every sentence with g2p and split its phonemes by gram, in the
//! format of `language_utils::sentence_phonemes`.
use language_utils::sentence_phonemes::{PhonemeToken, StressMark};
use language_utils::{Gram, Language, ReadingSentence, SentenceGram, readings::RubySegment};
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use std::collections::BTreeMap;

pub type RubySegmenter<'a> = dyn Fn(&str, &str) -> Option<Vec<RubySegment>> + Sync + 'a;

pub struct SentencePhonemes {
    /// Each phonemized sentence's sequence per gram.
    pub phonemes: FxHashMap<String, Vec<String>>,
    /// Sentences stored without phonemes, by reason.
    pub missing: BTreeMap<String, usize>,
}

pub fn g2p_language(language: Language) -> Option<g2p::Language> {
    match language {
        Language::SpanishLatinAmerican => Some(g2p::Language::SpanishLatinAmerica),
        Language::SpanishPeninsular => Some(g2p::Language::SpanishEuro),
        Language::PortugueseEuropean => Some(g2p::Language::PortugueseEuro),
        _ => g2p::Language::from_code(language.code()),
    }
}

pub fn phonemize_sentences(
    language: Language,
    sentences: &[(String, ReadingSentence<Gram<String>, String>)],
    segment_ruby: Option<&RubySegmenter<'_>>,
) -> SentencePhonemes {
    let mut out = SentencePhonemes {
        phonemes: FxHashMap::default(),
        missing: BTreeMap::new(),
    };
    let Some(g2p_language) = g2p_language(language) else {
        out.missing
            .insert("language unsupported by g2p".into(), sentences.len());
        return out;
    };
    let results: Vec<_> = sentences
        .par_iter()
        .map(|(text, sentence)| {
            sentence_phonemes(language, g2p_language, text, sentence, segment_ruby)
        })
        .collect();
    for ((text, _), result) in sentences.iter().zip(results) {
        match result {
            Ok(phonemes) => {
                out.phonemes.insert(text.clone(), phonemes);
            }
            Err(reason) => *out.missing.entry(reason).or_default() += 1,
        }
    }
    out
}

/// Write every phonemized sentence as a TTS training row (lexide/tts):
/// its text and, per gram, the gram's words, their parts of speech, the
/// gram's corpus count over all its senses, and its phoneme sequence. The
/// same rows `language-utils`' `dump_sentence_phonemes` reads out of a pack,
/// but for every phonemized sentence, before pack filters drop any. Returns
/// (rows written, rows with a supplied reading).
pub fn write_tts_rows(
    path: &std::path::Path,
    sentences: &[(String, ReadingSentence<Gram<String>, String>)],
    phonemes: &FxHashMap<String, Vec<String>>,
    frequencies: &[language_utils::ReadingGramFrequencyEntry<String>],
) -> std::io::Result<(usize, usize)> {
    use language_utils::{Atom, WordType};
    use std::io::Write;
    let mut counts = FxHashMap::<&Gram<String>, u32>::default();
    for entry in frequencies {
        *counts.entry(&entry.gram.sense.gram).or_default() += entry.count;
    }
    let mut out = std::io::BufWriter::new(std::fs::File::create(path)?);
    let (mut written, mut read) = (0, 0);
    for (text, sentence) in sentences {
        let Some(sequences) = phonemes.get(text) else {
            continue;
        };
        let mut has_reading = false;
        let grams: Vec<_> = sentence
            .grams
            .iter()
            .zip(sequences)
            .map(|(SentenceGram::Learnable(gram) | SentenceGram::Obvious(gram), sequence)| {
                has_reading |= gram.reading.is_some();
                let (words, pos): (Vec<&str>, Vec<serde_json::Value>) = gram
                    .sense
                    .gram
                    .iter()
                    .filter_map(|atom| match atom {
                        Atom::Tok(word) => Some((
                            word.text.as_str(),
                            match &word.word_type {
                                WordType::Heteronym(h) => serde_json::to_value(h.pos),
                                WordType::Other(o) => serde_json::to_value(o.other_tag),
                            }
                            .unwrap(),
                        )),
                        Atom::Control(_) => None,
                    })
                    .unzip();
                let count = counts.get(&gram.sense.gram).copied().unwrap_or(0);
                serde_json::json!({"words": words, "pos": pos, "count": count, "phonemes": sequence})
            })
            .collect();
        serde_json::to_writer(&mut out, &serde_json::json!({"text": text, "grams": grams}))?;
        out.write_all(b"\n")?;
        written += 1;
        read += usize::from(has_reading);
    }
    out.flush()?;
    Ok((written, read))
}

/// How the finished pack stores phonemes, read back from its archive.
pub fn print_stats(pack: &language_utils::language_pack::LanguagePack) {
    let (mut without, mut occurrences, mut marked, mut max_variant) = (0, 0u64, 0u64, 0);
    for key in pack.sentence_keys() {
        let decoded = language_utils::sentence_codec::decode(
            pack.sentences.encoded_sentences.resolve(&key).copied(),
            &pack.written_ease_order.grams,
        );
        let Some(variants) = decoded.phoneme_variants else {
            without += 1;
            continue;
        };
        for (_, variant) in variants {
            occurrences += 1;
            marked += u64::from(variant > 0);
            max_variant = max_variant.max(variant);
        }
    }
    let table = &pack.sentences.gram_phonemes;
    let lists: Vec<_> = table.by_gram.iter().flatten().collect();
    let grams = lists.len();
    let varying = lists
        .iter()
        .filter(|list| table.variant_lists.resolve(list).len() > 1)
        .count();
    println!(
        "sentence phonemes[{}]: {} distinct sequences; {grams} grams, {varying} with several; \
         {marked} of {occurrences} gram occurrences ({:.2}%) need a variant marker (max variant {max_variant}); \
         {without} sentences without phonemes",
        pack.sentences.target_language.code(),
        table.sequences.len(),
        100.0 * marked as f64 / occurrences.max(1) as f64,
    );
}

/// Each gram's sequence, or why the sentence has none.
fn sentence_phonemes(
    language: Language,
    g2p_language: g2p::Language,
    text: &str,
    sentence: &ReadingSentence<Gram<String>, String>,
    segment_ruby: Option<&RubySegmenter<'_>>,
) -> Result<Vec<String>, String> {
    let grams: Vec<_> = sentence
        .grams
        .iter()
        .map(|(SentenceGram::Learnable(g) | SentenceGram::Obvious(g))| g)
        .collect();
    // Rebuild the text as the pack renders it, recording each gram's and
    // each token's byte range.
    let atoms = grams
        .iter()
        .enumerate()
        .flat_map(|(i, gram)| gram.sense.gram.iter().map(move |atom| (i, atom.clone())));
    let mut rendered = String::new();
    let mut gram_spans = vec![None::<(usize, usize)>; grams.len()];
    // Each gram's words, in order (English readings are per word).
    let mut word_spans = vec![Vec::new(); grams.len()];
    let mut token_spans = Vec::new();
    // Where a multi-word gram's words break (see the format docs): the byte
    // each later word starts at.
    let tokens_are_words = matches!(
        language,
        Language::ChineseSimplified | Language::ChineseTraditional | Language::Thai
    );
    let mut word_breaks = Vec::new();
    let mut previous_token = None;
    for (i, word, whitespace) in
        language_utils::render_words(atoms, sentence.capitalize_first, language)
    {
        let start = rendered.len();
        rendered.push_str(&word.text);
        let end = rendered.len();
        rendered.push_str(whitespace.as_str());
        word_spans[i].push((start, end));
        if start < end {
            token_spans.push((start, end));
            if let Some((gram, previous_end)) = previous_token
                && gram == i
                && (tokens_are_words || previous_end < start)
            {
                word_breaks.push(start);
            }
            previous_token = Some((i, end));
        }
        let span = gram_spans[i].get_or_insert((start, end));
        span.1 = end;
    }
    assert_eq!(rendered, text, "a sentence renders as its text");
    let gram_spans: Vec<_> = gram_spans
        .iter()
        .scan(0, |previous_end, span| {
            // A gram with no words is an empty range where it sits.
            let span = span.unwrap_or((*previous_end, *previous_end));
            *previous_end = span.1;
            Some(span)
        })
        .collect();

    let mut readings = Vec::new();
    for (i, (gram, &(start, end))) in grams.iter().zip(&gram_spans).enumerate() {
        let Some(reading) = &gram.reading else {
            continue;
        };
        match language {
            Language::ChineseSimplified => readings.push(g2p::Reading {
                start,
                end,
                reading: reading.clone(),
            }),
            Language::Japanese => {
                let surface = &text[start..end];
                let segments = segment_ruby
                    .and_then(|segment| segment(surface, reading))
                    .unwrap_or_else(|| {
                        language_utils::readings::ruby_segments(surface, reading, language)
                    });
                let mut cursor = start;
                for segment in segments {
                    let next = cursor + segment.base.len();
                    if let Some(ruby) = segment.ruby {
                        readings.push(g2p::Reading {
                            start: cursor,
                            end: next,
                            reading: ruby,
                        });
                    }
                    cursor = next;
                }
                assert_eq!(cursor, end, "ruby segments spell {surface:?}");
            }
            Language::English => {
                let words: Vec<_> = crate::readings::english::word_readings(reading).collect();
                assert_eq!(
                    words.len(),
                    word_spans[i].len(),
                    "a reading per word of {reading:?}"
                );
                for (reading, &(start, end)) in words.into_iter().zip(&word_spans[i]) {
                    if !reading.is_empty() {
                        readings.push(g2p::Reading {
                            start,
                            end,
                            reading: reading.to_owned(),
                        });
                    }
                }
            }
            _ => {}
        }
    }
    // Thai is unspaced; Yap's tokens tell g2p where its words are (every
    // gram boundary is a token boundary, and many grams are several words).
    let words = if language == Language::Thai {
        token_spans
    } else {
        Vec::new()
    };
    let hints = g2p::Hints {
        readings: &readings,
        words: &words,
    };
    let phonemized = g2p::phonemize_with(g2p_language, text, &hints).map_err(refusal)?;
    split_by_gram(&phonemized, &gram_spans, &word_breaks)
        .ok_or_else(|| "g2p gave no phoneme spans".into())
}

/// A per-sentence refusal becomes a count; a broken backend (Python not on
/// PATH, espeak failing to start) must fail the build instead of shipping a
/// pack without phonemes.
fn refusal(error: g2p::Error) -> String {
    match error {
        g2p::Error::Unlabelable(code) => {
            format!("g2p refused: {}", code.split(':').next().unwrap())
        }
        g2p::Error::UnknownPhoneme(_) => "g2p refused: unknown phoneme".into(),
        g2p::Error::NulByte => "g2p refused: NUL byte".into(),
        g2p::Error::Synth(_) => "g2p refused: synthesis failed".into(),
        error @ (g2p::Error::Init(_)
        | g2p::Error::Data(_)
        | g2p::Error::UnknownVoice(_)
        | g2p::Error::UnsupportedLanguage(_)
        | g2p::Error::Backend(_)) => panic!("g2p backend failed: {error}"),
    }
}

/// Split a sentence's tokens into one sequence per gram (`grams` are the
/// grams' byte ranges, in order). Each phoneme goes to the first gram it
/// overlaps: the first gram ending after the phoneme's text starts (the last
/// gram if none does). That way a phoneme spoken from two grams' text (a
/// geminate across a boundary, a contraction) goes to the first. A phoneme
/// never goes to an earlier gram than the phoneme before it, so the
/// sequences always join back into the sentence's tokens in order.
///
/// `word_breaks` are the bytes where a gram's later words start; a `#` goes
/// between a gram's phonemes whose text lies on either side of one.
fn split_by_gram(
    phonemized: &g2p::Phonemized,
    grams: &[(usize, usize)],
    word_breaks: &[usize],
) -> Option<Vec<String>> {
    if phonemized.phoneme_text_spans.len() != phonemized.phonemes.len() {
        return None;
    }
    let mut sequences = vec![Vec::new(); grams.len()];
    let mut owner = 0;
    let mut previous_start = None;
    for (phoneme, token) in tokens(phonemized) {
        let (start, _) = phonemized.phoneme_text_spans[phoneme];
        let first = grams
            .iter()
            .position(|&(_, end)| start < end)
            .unwrap_or(grams.len() - 1);
        if first > owner {
            previous_start = None;
        }
        owner = owner.max(first);
        let (gram_start, gram_end) = grams[owner];
        if let Some(previous) = previous_start
            && word_breaks
                .iter()
                .any(|&at| previous < at && at <= start && gram_start < at && at < gram_end)
        {
            sequences[owner].push(PhonemeToken::WordBreak.to_string());
        }
        previous_start = Some(start);
        sequences[owner].push(token.to_string());
    }
    Some(
        sequences
            .into_iter()
            .map(|tokens| tokens.join(" "))
            .collect(),
    )
}

/// The sentence's tokens, each with the phoneme it is (or, for an accent
/// phrase start, the phoneme it precedes).
fn tokens(phonemized: &g2p::Phonemized) -> Vec<(usize, PhonemeToken<'_>)> {
    let pitch_of = |i: usize| phonemized.pitch.get(i).and_then(Option::as_ref);
    let mut tokens = Vec::new();
    let mut phrase = None;
    // g2p gives pitch to a mora's vowel (or moraic phone) only; the
    // pitchless phones before it are the mora's onset. This is where they
    // start in `tokens`.
    let mut onset = None;
    for (i, phoneme) in phonemized.phonemes.iter().enumerate() {
        let mut accent_nucleus = false;
        if let Some(pitch) = pitch_of(i) {
            if phrase != Some(pitch.phrase) {
                let at = onset.unwrap_or(tokens.len());
                let first = tokens.get(at).map_or(i, |&(first, _)| first);
                tokens.insert(at, (first, PhonemeToken::AccentPhraseStart));
                phrase = Some(pitch.phrase);
            }
            onset = None;
            // The last phone of the nucleus mora: the pitch drops after it.
            let same_mora =
                |next: &g2p::Pitch| (next.phrase, next.mora) == (pitch.phrase, pitch.mora);
            accent_nucleus = pitch.nucleus != 0
                && pitch.mora == pitch.nucleus
                && !pitch_of(i + 1).is_some_and(same_mora);
        } else if onset.is_none() {
            onset = Some(tokens.len());
        }
        tokens.push((
            i,
            PhonemeToken::Phone {
                phoneme: phoneme.as_str(),
                stress: match phonemized.stress.get(i) {
                    Some(g2p::Stress::Primary) => Some(StressMark::Primary),
                    Some(g2p::Stress::Secondary) => Some(StressMark::Secondary),
                    Some(g2p::Stress::None) | None => None,
                },
                tone: phonemized.tone.get(i).copied().flatten(),
                accent_nucleus,
            },
        ));
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    use language_utils::sentence_phonemes;

    /// Joining the gram sequences gives the sentence's tokens, and parsing
    /// them back gives g2p's phonemes.
    fn assert_joins(phonemized: &g2p::Phonemized, sequences: &[String]) {
        let joined: Vec<_> = sequences
            .iter()
            .filter(|s| !s.is_empty())
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" ")
            .split(' ')
            .filter(|t| !t.is_empty() && *t != "#")
            .map(str::to_owned)
            .collect();
        let expected: Vec<_> = tokens(phonemized)
            .iter()
            .map(|(_, t)| t.to_string())
            .collect();
        assert_eq!(joined, expected);
        let phones: Vec<_> = sequences
            .iter()
            .flat_map(|s| sentence_phonemes::tokens(s))
            .filter_map(|token| match token {
                PhonemeToken::Phone { phoneme, .. } => Some(phoneme.to_owned()),
                PhonemeToken::AccentPhraseStart | PhonemeToken::WordBreak => None,
            })
            .collect();
        let original: Vec<_> = phonemized
            .phonemes
            .iter()
            .map(|p| p.as_str().to_owned())
            .collect();
        assert_eq!(phones, original);
    }

    /// Splits `text` into `grams`; a gram's words break after its spaces.
    fn split(language: g2p::Language, text: &str, grams: &[&str]) -> Vec<String> {
        let mut spans = Vec::new();
        let mut breaks = Vec::new();
        let mut cursor = 0;
        for gram in grams {
            let start = cursor + text[cursor..].find(gram).unwrap();
            cursor = start + gram.len();
            spans.push((start, cursor));
            breaks.extend(gram.match_indices(' ').map(|(i, _)| start + i + 1));
        }
        let phonemized = g2p::phonemize(language, text).unwrap();
        let sequences = split_by_gram(&phonemized, &spans, &breaks).unwrap();
        assert_joins(&phonemized, &sequences);
        sequences
    }

    #[test]
    fn sequences_split_by_gram_and_join_back() {
        let french = split(
            g2p::Language::French,
            "C'est l'homme, non ?",
            &["C'", "est", "l'", "homme", ",", "non", "?"],
        );
        assert_eq!(french[0], "s");
        assert_eq!(french[4], "", "punctuation is silent");
        assert!(french.iter().any(|s| s.contains('ˈ')), "{french:?}");
        // A geminate across the boundary goes to the first gram.
        let english = split(
            g2p::Language::English,
            "I'm with them.",
            &["I'm", "with", "them", "."],
        );
        assert!(
            !english[1].is_empty() && !english[2].is_empty(),
            "{english:?}"
        );
        let mandarin = split(
            g2p::Language::ChineseSimplified,
            "我很好。",
            &["我", "很", "好", "。"],
        );
        assert!(
            mandarin[0].chars().any(|c| c.is_ascii_digit()),
            "{mandarin:?}"
        );
    }

    #[test]
    fn multi_word_grams_break_between_their_words() {
        let english = split(
            g2p::Language::English,
            "And then we left.",
            &["And then", "we", "left", "."],
        );
        assert_eq!(english[0].matches(" # ").count(), 1, "{english:?}");
        assert!(english.iter().skip(1).all(|s| !s.contains('#')));
        let french = split(
            g2p::Language::French,
            "C'est fini.",
            &["C'est", "fini", "."],
        );
        assert!(french.iter().all(|s| !s.contains('#')), "{french:?}");
    }

    #[test]
    fn japanese_marks_phrase_starts_and_accent_nuclei() {
        let sequences = split(
            g2p::Language::Japanese,
            "雨が降っています。",
            &["雨", "が", "降って", "い", "ます", "。"],
        );
        // あめ is atamadaka: the pitch drops after its first mora.
        assert_eq!(sequences[0], "% aꜜ m e", "{sequences:?}");
        // A phrase starts before its first mora's onset consonant.
        let sequences = split(
            g2p::Language::Japanese,
            "私はトラクターの乗り方を習いたいです。",
            &[
                "私",
                "は",
                "トラクター",
                "の",
                "乗り方",
                "を",
                "習いたい",
                "です",
                "。",
            ],
        );
        assert_eq!(sequences[0], "% w a t a ɕ i", "{sequences:?}");
        assert_eq!(sequences[2], "% t o ɾ aꜜ k ɯ̥ᵝ t a a", "{sequences:?}");
    }

    /// English readings reach g2p per word of their gram.
    #[test]
    fn english_readings_pronounce_their_words() {
        use language_utils::{
            Atom, Heteronym, PartOfSpeech, ReadingTaggedGram, SentenceGrams, TaggedGram, Word,
            WordType,
        };
        let word = |text: &str, pos: PartOfSpeech| {
            Atom::Tok(Word {
                text: text.to_owned(),
                word_type: WordType::Heteronym(Heteronym {
                    word: text.to_owned(),
                    lemma: text.to_owned(),
                    pos,
                }),
            })
        };
        let gram = |atoms: Vec<Atom<String>>, reading: Option<&str>| {
            SentenceGram::Learnable(ReadingTaggedGram {
                sense: TaggedGram {
                    gram: Gram::from(atoms),
                    sense: None,
                },
                reading: reading.map(str::to_owned),
            })
        };
        let sentence = |reading: Option<&str>| SentenceGrams {
            grams: vec![
                gram(vec![word("i", PartOfSpeech::Pron)], None),
                gram(
                    vec![
                        word("read", PartOfSpeech::Verb),
                        word("it", PartOfSpeech::Pron),
                    ],
                    reading,
                ),
            ],
            capitalize_first: true,
            multiword_terms: vec![],
            low_confidence_multiword_terms: vec![],
        };
        let phonemes = |reading| {
            sentence_phonemes(
                Language::English,
                g2p::Language::English,
                "I read it",
                &sentence(reading),
                None,
            )
            .unwrap()
        };
        assert!(phonemes(Some("ɹ ˈɛ d | "))[1].contains('ɛ'));
        assert!(phonemes(Some("ɹ ˈiː d | "))[1].contains("iː"));
        assert_eq!(phonemes(Some(" | "))[1], phonemes(None)[1]);
    }

    #[test]
    fn a_phoneme_never_moves_back_to_an_earlier_gram() {
        let mut phonemized = g2p::phonemize(g2p::Language::French, "le chat").unwrap();
        // Pretend the last phoneme was spoken from the first word.
        let last = phonemized.phoneme_text_spans.len() - 1;
        phonemized.phoneme_text_spans[last] = (0, 2);
        let sequences = split_by_gram(&phonemized, &[(0, 2), (3, 7)], &[]).unwrap();
        assert_joins(&phonemized, &sequences);
        assert!(!sequences[1].is_empty());
    }

    #[test]
    fn every_phoneme_survives_the_token_format() {
        for phoneme in g2p::Phoneme::ALL {
            let phoneme = phoneme.as_str();
            for stress in [None, Some(StressMark::Primary), Some(StressMark::Secondary)] {
                for tone in [None, Some(3)] {
                    for accent_nucleus in [false, true] {
                        let token = PhonemeToken::Phone {
                            phoneme,
                            stress,
                            tone,
                            accent_nucleus,
                        };
                        assert_eq!(PhonemeToken::parse(&token.to_string()), token);
                    }
                }
            }
        }
    }
}
