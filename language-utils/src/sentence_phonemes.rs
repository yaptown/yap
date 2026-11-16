//! Every sentence's phonemes, broken down by gram, for on-device TTS.
//!
//! Each reading-tagged gram keeps the phoneme sequences it is spoken with
//! across the corpus, most common first. An encoded sentence marks each gram
//! spoken with a sequence other than its most common one
//! ([`crate::sentence_codec::SymbolKind::PhonemeVariant`]), so decoding a
//! sentence's grams also yields its phonemes.
//!
//! # Sequence format
//!
//! A sequence is tokens separated by single spaces (a silent gram's sequence
//! is empty). Each token is either:
//!
//! * a phone: an optional stress prefix (`ˈ` primary, `ˌ` secondary), the g2p
//!   phoneme, then its tone number when it bears one (Mandarin, Thai: `a3`),
//!   then `ꜜ` when it ends a Japanese accent nucleus mora (pitch drops after
//!   it); or
//! * `%`, which starts a Japanese accent phrase (before its first mora).
//!
//! Japanese H/L levels follow from the nucleus and phrase starts (g2p's
//! `tokyo_pitch_level`). No g2p phoneme contains a digit, `ꜜ`, `%`, or begins
//! with a stress mark, so [`tokens`] parses every sequence unambiguously.
//! Joining a sentence's gram sequences with spaces (skipping empty ones)
//! gives the whole sentence's tokens in order.
use crate::sentence_codec::GramKey;
use lasso::{Key, Spur};

#[derive(Debug)]
pub enum PhonemeSequenceText {}
pub type PhonemeSequenceSpur = Spur<PhonemeSequenceText>;

#[derive(Debug)]
pub enum PhonemeVariantList {}
pub type PhonemeVariantListSpur = Spur<PhonemeVariantList>;

#[derive(Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct GramPhonemes {
    /// Every distinct sequence, in a compressed rodeo of its own: its
    /// dictionary looks nothing like the other text tables'.
    pub sequences: lasso::CompressedRodeoReader<PhonemeSequenceSpur, lasso::RunTable, lasso::Text>,
    /// Each gram's sequences, most common first; interned, since many grams
    /// share a list and a vector per gram would cost more than the data.
    pub variant_lists: lasso::RodeoResolver<Vec<PhonemeSequenceSpur>, PhonemeVariantListSpur>,
    /// Indexed by [`GramKey`]. `None` for a gram no phonemized sentence uses
    /// as a word (phrases, or grams only in sentences without phonemes).
    pub by_gram: Vec<Option<PhonemeVariantListSpur>>,
}

impl Default for GramPhonemes {
    fn default() -> Self {
        Self {
            sequences: Default::default(),
            variant_lists: lasso::Rodeo::new().into_resolver(),
            by_gram: Vec::new(),
        }
    }
}

impl ArchivedGramPhonemes {
    /// The gram's `variant`-th most common sequence.
    pub fn sequence(&self, gram: GramKey, variant: u32) -> PhonemeSequenceSpur {
        let list = self.by_gram[gram.into_usize()]
            .as_ref()
            .expect("a phonemized sentence's gram has sequences");
        self.variant_lists.resolve(list)[variant as usize]
    }

    pub fn text(&self, sequence: &PhonemeSequenceSpur) -> String {
        self.sequences.resolve_string(sequence)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StressMark {
    Primary,
    Secondary,
}

impl StressMark {
    fn mark(self) -> char {
        match self {
            Self::Primary => 'ˈ',
            Self::Secondary => 'ˌ',
        }
    }
}

const ACCENT_PHRASE_START: &str = "%";
const ACCENT_NUCLEUS: char = 'ꜜ';

/// One token of a sequence; see the [module docs](self) for the text form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhonemeToken<'a> {
    AccentPhraseStart,
    Phone {
        phoneme: &'a str,
        stress: Option<StressMark>,
        tone: Option<u8>,
        accent_nucleus: bool,
    },
}

impl std::fmt::Display for PhonemeToken<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::AccentPhraseStart => f.write_str(ACCENT_PHRASE_START),
            Self::Phone {
                phoneme,
                stress,
                tone,
                accent_nucleus,
            } => {
                if let Some(stress) = stress {
                    write!(f, "{}", stress.mark())?;
                }
                f.write_str(phoneme)?;
                if let Some(tone) = tone {
                    write!(f, "{tone}")?;
                }
                if accent_nucleus {
                    write!(f, "{ACCENT_NUCLEUS}")?;
                }
                Ok(())
            }
        }
    }
}

impl<'a> PhonemeToken<'a> {
    pub fn parse(token: &'a str) -> Self {
        if token == ACCENT_PHRASE_START {
            return Self::AccentPhraseStart;
        }
        let (stress, rest) = [StressMark::Primary, StressMark::Secondary]
            .into_iter()
            .find_map(|stress| Some((Some(stress), token.strip_prefix(stress.mark())?)))
            .unwrap_or((None, token));
        let (rest, accent_nucleus) = match rest.strip_suffix(ACCENT_NUCLEUS) {
            Some(rest) => (rest, true),
            None => (rest, false),
        };
        let phoneme = rest.trim_end_matches(|c: char| c.is_ascii_digit());
        let tone = (phoneme.len() < rest.len()).then(|| rest[phoneme.len()..].parse().unwrap());
        Self::Phone {
            phoneme,
            stress,
            tone,
            accent_nucleus,
        }
    }
}

/// Parse a sequence (or a sentence's joined sequences) into its tokens.
pub fn tokens(sequence: &str) -> impl Iterator<Item = PhonemeToken<'_>> {
    sequence
        .split(' ')
        .filter(|t| !t.is_empty())
        .map(PhonemeToken::parse)
}
