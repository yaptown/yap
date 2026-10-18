//! Bound an open reading to the characters inside the marked gram.
use std::collections::{BTreeMap, BTreeSet};

use language_utils::{
    Language,
    readings::{canonical_kana, is_ruby_base},
};
use unicode_normalization::UnicodeNormalization;

pub(super) type Table = BTreeMap<char, Vec<String>>;
type Lexicon = BTreeMap<String, Vec<String>>;

#[derive(Default)]
pub(super) struct Verifier {
    characters: BTreeMap<char, BTreeSet<String>>,
    segments: BTreeMap<String, BTreeSet<String>>,
}
impl Verifier {
    pub fn new(characters: Table, words: &super::Dictionary, stems: Lexicon) -> Self {
        let characters = characters
            .into_iter()
            .map(|(c, r)| (c, phonological_variants(&r, true, true, |_| true)))
            .collect();
        let mut segments: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (word, readings) in stems.into_iter().chain(
            words
                .iter()
                .filter(|(word, _)| word.chars().count() > 1)
                .map(|(word, readings)| {
                    (
                        word.clone(),
                        readings.iter().map(|r| r.reading.clone()).collect(),
                    )
                }),
        ) {
            segments
                .entry(word.nfkc().collect())
                .or_default()
                .extend(readings);
        }
        Self {
            characters,
            segments,
        }
    }
}

pub(super) fn variants(readings: &[String], initial: bool, final_edge: bool) -> BTreeSet<String> {
    // Lyman's law constrains offered rendaku. Verification remains permissive:
    // unusual attested forms and names are evidence to check, not synthetic defaults.
    phonological_variants(readings, initial, final_edge, |base| {
        !base
            .chars()
            .skip(1)
            .any(|c| c.to_string().nfd().any(|mark| mark == '\u{3099}'))
    })
}

fn phonological_variants(
    readings: &[String],
    initial: bool,
    final_edge: bool,
    allow_rendaku: impl Fn(&str) -> bool,
) -> BTreeSet<String> {
    let mut variants = BTreeSet::new();
    for reading in readings {
        let reading: String = reading
            .nfkc()
            .map(canonical_kana)
            .filter(|&c| c != '-')
            .collect();
        // The stem lets the surface's already-aligned kana supply the okurigana.
        let stem = reading.split('.').next().unwrap();
        for base in [stem.to_owned(), reading.replace('.', "")] {
            if base.is_empty() {
                continue;
            }
            let mut starts = vec![base.clone()];
            let first = base.chars().next().unwrap();
            let voiced: &[char] = match first {
                'か' => &['が'],
                'き' => &['ぎ'],
                'く' => &['ぐ'],
                'け' => &['げ'],
                'こ' => &['ご'],
                'さ' => &['ざ'],
                'し' => &['じ'],
                'す' => &['ず'],
                'せ' => &['ぜ'],
                'そ' => &['ぞ'],
                'た' => &['だ'],
                'ち' => &['ぢ'],
                'つ' => &['づ'],
                'て' => &['で'],
                'と' => &['ど'],
                'は' => &['ば', 'ぱ'],
                'ひ' => &['び', 'ぴ'],
                'ふ' => &['ぶ', 'ぷ'],
                'へ' => &['べ', 'ぺ'],
                'ほ' => &['ぼ', 'ぽ'],
                _ => &[],
            };
            starts.extend(
                voiced
                    .iter()
                    .filter(|_| initial && allow_rendaku(&base))
                    .map(|c| format!("{c}{}", &base[first.len_utf8()..])),
            );
            for reading in starts {
                if final_edge && matches!(reading.chars().last(), Some('つ' | 'ち' | 'く' | 'き'))
                {
                    let mut geminated = reading.clone();
                    geminated.pop();
                    geminated.push('っ');
                    variants.insert(geminated);
                }
                variants.insert(reading);
            }
        }
    }
    variants
}

/// Expand one lexical atom, retaining its written kana. A proven Han/kana
/// boundary also licenses gemination before written ち, as in a name suffix.
pub(super) fn atom_variants(
    surface: &str,
    reading: &str,
    allow_onset: bool,
    allow_ending: bool,
) -> BTreeSet<String> {
    let initial = allow_onset
        && surface
            .chars()
            .next()
            .is_some_and(|c| is_ruby_base(c, Language::Japanese));
    let final_edge = allow_ending
        && surface
            .chars()
            .last()
            .is_some_and(|c| is_ruby_base(c, Language::Japanese));
    let mut result = variants(&[reading.into()], initial, final_edge);
    for base in result.clone() {
        let segments = language_utils::readings::ruby_segments(surface, &base, Language::Japanese);
        let mut parts = Vec::new();
        for (i, segment) in segments.iter().enumerate() {
            let mut options = BTreeSet::new();
            if let Some(ruby) = &segment.ruby {
                let before_chi = segments.get(i + 1).is_some_and(|next| {
                    next.ruby.is_none()
                        && next.base.chars().next().map(canonical_kana) == Some('ち')
                });
                if before_chi
                    && segment
                        .base
                        .chars()
                        .all(|c| is_ruby_base(c, Language::Japanese))
                {
                    options = variants(std::slice::from_ref(ruby), false, true);
                }
                options.insert(ruby.clone());
            } else {
                options.insert(
                    segment
                        .base
                        .chars()
                        .map(canonical_kana)
                        .filter(|c| ('ぁ'..='ゖ').contains(c) || *c == 'ー')
                        .collect(),
                );
            }
            parts.push(options);
        }
        // A whole-ruby fallback cannot invent an internal boundary; the checks
        // above require an actual Han island followed by literal written kana.
        let mut product = BTreeSet::from([String::new()]);
        for options in parts {
            product = product
                .iter()
                .flat_map(|prefix| {
                    options
                        .iter()
                        .map(move |suffix| format!("{prefix}{suffix}"))
                })
                .collect();
        }
        result.extend(product.into_iter().filter(|r| {
            super::canonical::checked_surface_reading(Language::Japanese, surface, r).is_some()
        }));
    }
    result
}

pub(super) fn matches(surface: &str, reading: &str, verifier: &Verifier) -> bool {
    let surface: String = surface.nfkc().collect();
    let chars: Vec<_> = surface.char_indices().collect();
    let boundaries: Vec<_> = chars
        .iter()
        .map(|(byte, _)| *byte)
        .chain(std::iter::once(surface.len()))
        .collect();
    let mut positions = vec![BTreeSet::new(); chars.len() + 1];
    positions[0].insert(0);
    let mut previous = None;
    for (i, &(byte, c)) in chars.iter().enumerate() {
        let options = if is_ruby_base(c, Language::Japanese) {
            let kanji = match c {
                '々' => previous,
                'ヶ' | 'ヵ' => Some('箇'),
                _ => Some(c),
            };
            previous = kanji;
            kanji
                .and_then(|c| verifier.characters.get(&c))
                .cloned()
                .unwrap_or_default()
        } else {
            let c = canonical_kana(c);
            let literal = if ('ぁ'..='ゖ').contains(&c) || c == 'ー' {
                c.to_string()
            } else {
                String::new()
            };
            BTreeSet::from([literal])
        };
        let starts = std::mem::take(&mut positions[i]);
        if starts.is_empty() {
            continue;
        }
        let segments: Vec<_> = (i + 1..=chars.len())
            .filter_map(|end| {
                verifier
                    .segments
                    .get(&surface[byte..boundaries[end]])
                    .map(|readings| (end, readings))
            })
            .collect();
        for start in starts {
            for part in &options {
                if reading[start..].starts_with(part) {
                    positions[i + 1].insert(start + part.len());
                }
            }
            // Dictionary spans handle jukujikun inside compounds, while stripped
            // verb/adjective stems leave the surface kana to the literal edges.
            for (end, readings) in &segments {
                for part in *readings {
                    if reading[start..].starts_with(part) {
                        positions[*end].insert(start + part.len());
                    }
                }
            }
        }
    }
    positions[chars.len()].contains(&reading.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn table() -> Verifier {
        let characters = [
            ('一', vec!["イチ", "ひと.つ"]),
            ('日', vec!["ニチ", "ジツ", "ひ"]),
            ('入', vec!["い.る", "はい.る"]),
            ('学', vec!["ガク"]),
            ('校', vec!["コウ"]),
            ('手', vec!["て"]),
            ('紙', vec!["かみ"]),
            ('人', vec!["ひと"]),
            ('本', vec!["ホン"]),
            ('三', vec!["サン"]),
            ('箇', vec!["カ"]),
            ('月', vec!["ゲツ"]),
            ('今', vec!["コン", "いま"]),
        ]
        .into_iter()
        .map(|(c, readings)| (c, readings.into_iter().map(str::to_owned).collect()))
        .collect();
        Verifier::new(characters, &BTreeMap::new(), BTreeMap::new())
    }
    #[test]
    fn offered_rendaku_obeys_lymans_law_without_restricting_verification() {
        let readings = vec!["つづき".into()];
        assert!(!variants(&readings, true, false).contains("づづき"));
        assert!(phonological_variants(&readings, true, false, |_| true).contains("づづき"));
        assert!(variants(&["かみ".into()], true, false).contains("がみ"));
        assert!(!variants(&["かざり".into()], true, false).contains("がざり"));
        assert!(variants(&["いち".into()], false, true).contains("いっ"));
        assert!(
            variants(&["つづき".into(), "づづき".into()], true, false).contains("づづき"),
            "an explicitly supplied reading is not a synthetic invention"
        );
    }

    #[test]
    fn cannot_absorb_a_neighbouring_character() {
        assert!(matches("一日", "いちにち", &table()));
        assert!(!matches("一日", "ひいちにち", &table()));
    }
    #[test]
    fn aligns_okurigana_and_standard_sound_changes() {
        for (surface, reading) in [
            ("入った", "はいった"),
            ("入った", "いった"),
            ("学校", "がっこう"),
            ("手紙", "てがみ"),
            ("日本", "にっぽん"),
            ("人々", "ひとびと"),
            ("三ヶ月", "さんかげつ"),
        ] {
            assert!(matches(surface, reading, &table()), "{surface} {reading}");
        }
        assert!(!matches("入った", "はいる", &table()));
    }
    #[test]
    fn dictionary_spans_and_inflection_stems_cover_open_compounds() {
        let words = BTreeMap::from([
            ("今日".into(), vec!["きょう".into()]),
            ("土産".into(), vec!["みやげ".into()]),
            ("母さん".into(), vec!["かあさん".into()]),
            ("兄さま".into(), vec!["にいさま".into()]),
        ]);
        let stems = BTreeMap::from([
            ("可愛".into(), vec!["かわい".into()]),
            ("上手".into(), vec!["うま".into()]),
            ("分".into(), vec!["わか".into()]),
            ("似".into(), vec!["に".into()]),
        ]);
        let verifier = Verifier::new(BTreeMap::new(), &words, stems);
        for (surface, reading) in [
            ("可愛がって", "かわいがって"),
            ("上手く", "うまく"),
            ("分った", "わかった"),
            ("今日も", "きょうも"),
            ("お土産", "おみやげ"),
            ("お兄さま", "おにいさま"),
            ("お母さんに似ている", "おかあさんににている"),
        ] {
            assert!(matches(surface, reading, &verifier), "{surface} {reading}");
        }
        assert!(!matches("一日", "ひいちにち", &verifier));
    }
    #[test]
    fn jukujikun_need_the_whole_word_dictionary_exemption() {
        assert!(!matches("今日", "きょう", &table()));
        assert!(!matches("一日", "ついたち", &table()));
    }
}
