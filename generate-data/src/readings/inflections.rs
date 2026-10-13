//! JMdict conjugation classes indexed by the written stem, never a dictionary scan per gram.
use super::*;

struct Inflection {
    lemma: String,
    reading: String,
    pattern: std::sync::Arc<regex::Regex>,
}

#[derive(Default)]
pub(super) struct Index(BTreeMap<String, Vec<Inflection>>);

impl Index {
    pub fn new(rows: BTreeMap<String, Vec<(String, String, String)>>) -> Self {
        let mut patterns = BTreeMap::new();
        Self(
            rows.into_iter()
                .map(|(stem, entries)| {
                    let entries = entries
                        .into_iter()
                        .filter_map(|(lemma, reading, kind)| {
                            let tail = hiragana(&lemma[stem.len()..]);
                            pattern(&tail, &kind).map(|pattern| {
                                let pattern = patterns
                                    .entry(pattern.clone())
                                    .or_insert_with(|| {
                                        std::sync::Arc::new(
                                            regex::Regex::new(&format!(r"\A(?:{pattern})\z"))
                                                .unwrap(),
                                        )
                                    })
                                    .clone();
                                Inflection {
                                    lemma,
                                    reading,
                                    pattern,
                                }
                            })
                        })
                        .collect();
                    (stem, entries)
                })
                .collect(),
        )
    }

    pub fn knows(&self, lemma: &str, reading: &str) -> bool {
        lemma
            .char_indices()
            .skip(1)
            .map(|(end, _)| end)
            .chain(std::iter::once(lemma.len()))
            .filter_map(|end| self.0.get(&lemma[..end]))
            .flatten()
            .any(|entry| entry.lemma == lemma && entry.reading == reading)
    }

    pub fn readings(&self, text: &str, dictionary: &Dictionary) -> Vec<DictionaryReading> {
        self.readings_for(text, dictionary, None)
    }

    pub fn readings_for(
        &self,
        text: &str,
        dictionary: &Dictionary,
        preferred: Option<&str>,
    ) -> Vec<DictionaryReading> {
        let mut result = Vec::new();
        for (end, _) in text
            .char_indices()
            .skip(1)
            .chain(std::iter::once((text.len(), '\0')))
        {
            let stem = &text[..end];
            let Some(entries) = self.0.get(stem) else {
                continue;
            };
            let tail = hiragana(&text[end..]);
            for Inflection {
                lemma,
                reading,
                pattern,
            } in entries
            {
                if preferred.is_some_and(|p| p != lemma) {
                    continue;
                }
                let lemma_tail = hiragana(&lemma[end..]);
                let replaced = replaceable_lemma_tail(&lemma_tail, &tail);
                let restored = if preferred.is_some() {
                    format!(
                        "{}{}",
                        &lemma_tail[..lemma_tail.len() - replaced.len()],
                        tail
                    )
                } else {
                    tail.clone()
                };
                // Restore a bare omitted stem only when the caller supplied its
                // lemma; otherwise preserve the class's existing bare-stem rules.
                if tail.is_empty() {
                    if preferred.is_none() && !pattern.is_match(&tail) {
                        continue;
                    }
                } else if !pattern.is_match(&restored) {
                    continue;
                }
                let tail = restored.as_str();
                for entry in dictionary
                    .get(lemma)
                    .into_iter()
                    .flatten()
                    .filter(|r| &r.reading == reading)
                {
                    // いい is a dictionary-form alternative of よい, not the
                    // productive stem of よかった/よく. Keep its own fixed forms.
                    if entry.reading == "いい" && lemma_tail == "い" && tail != lemma_tail {
                        continue;
                    }
                    if let Some(root) = entry.reading.strip_suffix(&lemma_tail) {
                        let root = surface_stem(root, lemma, &lemma_tail, tail);
                        result.push(DictionaryReading {
                            reading: format!("{root}{}", tail.replace('方', "かた")),
                            ..entry.clone()
                        });
                    }
                }
            }
        }
        result
    }
}

/// Replace the conjugating ending plus fixed kana already visible in the surface
/// tail. Fixed kana omitted from the spelling remain in the spoken stem.
pub(super) fn replaceable_lemma_tail<'a>(lemma_tail: &'a str, surface_tail: &str) -> &'a str {
    let ending = lemma_tail
        .strip_suffix("する")
        .or_else(|| lemma_tail.strip_suffix("ずる"))
        .map_or_else(
            || lemma_tail.char_indices().last().map_or(0, |(i, _)| i),
            str::len,
        );
    let fixed = &lemma_tail[..ending];
    let start = fixed
        .char_indices()
        .find_map(|(i, _)| surface_tail.starts_with(&fixed[i..]).then_some(i))
        .unwrap_or(ending);
    &lemma_tail[start..]
}

/// The written kanji hides the irregular stem. Both the typed index and the
/// lemma fallback must use the same alternation, including polite imperatives.
pub(super) fn surface_stem(root: &str, lemma: &str, lemma_tail: &str, tail: &str) -> String {
    if (lemma.ends_with("来る") || lemma.ends_with("來る"))
        && lemma_tail == "る"
        && let Some(prefix) = root.strip_suffix('く')
    {
        let sound = if tail.starts_with('る') || tail.starts_with("れば") {
            "く"
        } else if tail.starts_with("なさい") || tail.starts_with("ながら") {
            "き"
        } else if tail.starts_with(['な', 'ら', 'さ', 'ず', 'ぬ', 'ん', 'よ']) || tail == "い"
        {
            "こ"
        } else {
            "き"
        };
        format!("{prefix}{sound}")
    } else if lemma.ends_with("為る")
        && lemma_tail == "る"
        && let Some(prefix) = root.strip_suffix('す')
    {
        let sound = if tail.starts_with('る') || tail.starts_with("れば") {
            "す"
        } else if tail.starts_with(['れ', 'せ']) {
            "さ"
        } else if tail.starts_with(['ず', 'ぬ', 'ん']) || tail == "よ" {
            "せ"
        } else {
            "し"
        };
        format!("{prefix}{sound}")
    } else {
        root.to_owned()
    }
}

fn pattern(tail: &str, kind: &str) -> Option<String> {
    let (last, root) = (
        tail.chars().last()?,
        &tail[..tail.len() - tail.chars().last()?.len_utf8()],
    );
    // These continuations are grammatical auxiliaries, not arbitrary kana tails.
    let negative = "な(?:い|かった|く(?:て|なった)?|ければ)|ず|ぬ|ん";
    let polite = "ま(?:す|した|せん(?:でした)?|しょう)";
    let te = "(?:て(?:いる|いた|いない|います|る|た|ない|ます|しまう|しまった|おく|おいた|ある|あった|ください)?|た(?:ら|り)?|ちゃ(?:う|った|って|います|わない)|ちま(?:う|った|って))";
    let continuative =
        format!("{polite}|たい|たかった|たく(?:ない|なかった|て|なる|なった)|ながら|なさい|方|");
    let ichidan_base = format!("る|れ(?:ば)?|ろ|よ|よう|{negative}|{continuative}|{te}");
    let ichidan = format!("{ichidan_base}|(?:られ|させ|させられ)(?:{ichidan_base})");
    let passive = format!("(?:れ|せ|せられ|され)(?:{ichidan_base})");
    Some(match kind {
        "kuru" if tail == "る" => format!(
            "る|れば|い|よう|{negative}|{continuative}|{te}|(?:られ|させ|させられ)(?:{ichidan_base})"
        ),
        "suru" if tail == "る" => {
            format!("る|れば|ろ|よ|よう|{negative}|{continuative}|{te}|{passive}")
        }
        "zuru" if tail.ends_with("ずる") => {
            let root = tail.strip_suffix("ずる").unwrap();
            format!("{root}(?:ずる|ずれば|ぜよ|じ(?:{ichidan}))")
        }
        "suru" if tail.ends_with("する") => {
            let root = tail.strip_suffix("する").unwrap();
            format!(
                "{root}(?:する|すれば|しろ|せよ|しよう|し(?:{negative}|{continuative}|{te})|(?:され|させ|させられ)(?:{ichidan_base}))"
            )
        }
        "ichidan" if last == 'る' => format!("{root}(?:{ichidan})"),
        "godan" | "iku" | "u-special" => {
            let (a, i, e, o, onbin) = match last {
                'う' => (
                    "わ",
                    "い",
                    "え",
                    "お",
                    if kind == "u-special" { "う" } else { "っ" },
                ),
                'く' => (
                    "か",
                    "き",
                    "け",
                    "こ",
                    if kind == "iku" { "っ" } else { "い" },
                ),
                'ぐ' => ("が", "ぎ", "げ", "ご", "い"),
                'す' => ("さ", "し", "せ", "そ", "し"),
                'つ' => ("た", "ち", "て", "と", "っ"),
                'ぬ' => ("な", "に", "ね", "の", "ん"),
                'ぶ' => ("ば", "び", "べ", "ぼ", "ん"),
                'む' => ("ま", "み", "め", "も", "ん"),
                'る' => ("ら", "り", "れ", "ろ", "っ"),
                _ => return None,
            };
            let te = if "ぐぬぶむ".contains(last) {
                te.replace("て(?:", "で(?:")
                    .replace("|た(?:", "|だ(?:")
                    .replace("ちゃ", "じゃ")
                    .replace("ちま", "じま")
            } else {
                te.into()
            };
            format!(
                "{root}(?:{last}|{a}(?:{negative}|{passive})|{i}(?:{continuative})|{e}(?:ば|{ichidan})|{o}う|{onbin}(?:{te}|))"
            )
        }
        "adjective" if last == 'い' => {
            format!("{root}(?:い|かった(?:ら|り)?|く(?:て|ない|なかった)?|ければ|かろう|さ|そう)")
        }
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn known_classes_restore_omitted_kana_without_accepting_wrong_conjugations() {
        let mut dictionary = Dictionary::new();
        let mut classes = BTreeMap::<String, Vec<(String, String, String)>>::new();
        for (stem, lemma, reading, kind) in [
            ("預", "預かる", "あずかる", "godan"),
            ("終", "終わる", "おわる", "godan"),
            ("行", "行なう", "おこなう", "godan"),
            ("信", "信ずる", "しんずる", "zuru"),
            ("重", "重んずる", "おもんずる", "zuru"),
            ("乗", "乗る", "のる", "godan"),
            ("癒", "癒える", "いえる", "ichidan"),
            ("食", "食べる", "たべる", "ichidan"),
            ("見", "見る", "みる", "ichidan"),
            ("来", "来る", "くる", "kuru"),
            ("為", "為る", "する", "suru"),
        ] {
            dictionary.insert(lemma.into(), vec![reading.into()]);
            classes.entry(stem.into()).or_default().push((
                lemma.into(),
                reading.into(),
                kind.into(),
            ));
        }
        let index = Index::new(classes);
        for (surface, lemma, expected) in [
            ("預って", "預かる", "あずかって"),
            ("預らない", "預かる", "あずからない"),
            ("終った", "終わる", "おわった"),
            ("行いました", "行なう", "おこないました"),
            ("信じた", "信ずる", "しんじた"),
            ("重じた", "重んずる", "おもんじた"),
            ("癒えました", "癒える", "いえました"),
            ("癒ました", "癒える", "いえました"),
            ("来ちゃった", "来る", "きちゃった"),
            ("来ながら", "来る", "きながら"),
            ("食べたい", "食べる", "たべたい"),
            ("見たい", "見る", "みたい"),
            ("見ながら", "見る", "みながら"),
            ("見なさい", "見る", "みなさい"),
            ("食べたくない", "食べる", "たべたくない"),
            ("食べられなかった", "食べる", "たべられなかった"),
            ("為れた", "為る", "された"),
            ("為せない", "為る", "させない"),
        ] {
            let readings = index.readings_for(surface, &dictionary, Some(lemma));
            assert_eq!(
                readings
                    .iter()
                    .map(|r| r.reading.as_str())
                    .collect::<Vec<_>>(),
                [expected],
                "{surface}"
            );
        }
        assert!(index.readings("食", &dictionary).is_empty());
        // A finite list of indexed forms must not veto ordinary extended forms
        // that do not restore any unwritten stem kana.
        let extended =
            super::super::tests::word("乗ってません", "乗る", language_utils::PartOfSpeech::Verb);
        assert!(
            indexed_candidates(&extended, Language::Japanese, &dictionary, &index)
                .readings
                .contains("のってません")
        );
        assert!(
            index
                .readings_for("癒りました", &dictionary, Some("癒える"))
                .is_empty()
        );
        assert!(inflected_readings("癒りました", "癒える", &dictionary, &index).is_empty());
        let gram =
            super::super::tests::word("癒りました", "癒える", language_utils::PartOfSpeech::Verb);
        assert!(
            indexed_candidates(&gram, Language::Japanese, &dictionary, &index)
                .readings
                .is_empty()
        );
    }

    #[test]
    fn omitted_okurigana_stays_in_the_reading_stem() {
        let dictionary = Dictionary::from([
            ("預かる".into(), vec!["あずかる".into()]),
            ("終わる".into(), vec!["おわる".into()]),
            ("行なう".into(), vec!["おこなう".into()]),
            ("勉強する".into(), vec!["べんきょうする".into()]),
            ("信ずる".into(), vec!["しんずる".into()]),
            ("重んずる".into(), vec!["おもんずる".into()]),
        ]);
        for (surface, lemma, expected) in [
            ("預って", "預かる", "あずかって"),
            ("預かって", "預かる", "あずかって"),
            ("預らない", "預かる", "あずからない"),
            ("終った", "終わる", "おわった"),
            ("行いました", "行なう", "おこないました"),
            ("勉強した", "勉強する", "べんきょうした"),
            ("勉強すれば", "勉強する", "べんきょうすれば"),
            ("信じた", "信ずる", "しんじた"),
            ("重んじた", "重んずる", "おもんじた"),
            ("重じた", "重んずる", "おもんじた"),
        ] {
            let readings = inflected_readings(surface, lemma, &dictionary, &Index::default());
            assert_eq!(
                readings
                    .iter()
                    .map(|r| r.reading.as_str())
                    .collect::<Vec<_>>(),
                [expected],
                "{surface}"
            );
        }
        assert_eq!(replaceable_lemma_tail("かなう", "なった"), "なう");
    }

    #[test]
    fn canonical_dictionary_iu_reading_survives_inflection_and_composition() {
        let dictionary = BTreeMap::from([
            ("言う".into(), vec!["いう".into()]),
            ("結う".into(), vec!["ゆう".into()]),
        ]);
        let index = Index::new(BTreeMap::from([
            (
                "言".into(),
                vec![("言う".into(), "いう".into(), "godan".into())],
            ),
            (
                "結".into(),
                vec![("結う".into(), "ゆう".into(), "godan".into())],
            ),
        ]));
        for (surface, expected) in [
            ("言う", "いう"),
            ("言った", "いった"),
            ("言わない", "いわない"),
        ] {
            for readings in [
                index.readings(surface, &dictionary),
                inflected_readings(surface, "言う", &dictionary, &Index::default()),
            ] {
                assert!(!readings.is_empty());
                assert!(readings.iter().all(|r| r.reading == expected), "{surface}");
            }
        }
        // A supplied lemma identifies the bare written stem without admitting
        // every homographic verb when no lemma was supplied.
        let gram = super::super::tests::word("言", "言う", language_utils::PartOfSpeech::Verb);
        let Atom::Tok(word) = &gram.0[0] else {
            unreachable!()
        };
        assert!(
            atom_readings(word, Language::Japanese, &dictionary, &index)
                .iter()
                .all(|r| r.reading == "い")
        );
        assert_eq!(
            inflected_readings("言", "言う", &dictionary, &Index::default())[0].reading,
            "い"
        );
        assert_eq!(index.readings("結った", &dictionary)[0].reading, "ゆった");
    }

    #[test]
    fn irregular_matching_keeps_other_readings_of_the_same_written_suffix() {
        let dictionary = BTreeMap::from([
            ("出来る".into(), vec!["できる".into()]),
            ("来る".into(), vec!["くる".into(), "きたる".into()]),
            ("來る".into(), vec!["くる".into()]),
        ]);
        let index = Index::new(BTreeMap::from([
            (
                "出来".into(),
                vec![("出来る".into(), "できる".into(), "ichidan".into())],
            ),
            (
                "来".into(),
                vec![
                    ("来る".into(), "くる".into(), "kuru".into()),
                    ("来る".into(), "きたる".into(), "godan".into()),
                ],
            ),
            (
                "來".into(),
                vec![("來る".into(), "くる".into(), "kuru".into())],
            ),
        ]));
        for (text, lemma, expected) in [
            ("出来た", "出来る", vec!["できた"]),
            ("出来ない", "出来る", vec!["できない"]),
            ("来れば", "来る", vec!["くれば", "きたれば"]),
            ("來た", "來る", vec!["きた"]),
        ] {
            for readings in [
                index.readings(text, &dictionary),
                inflected_readings(text, lemma, &dictionary, &Index::default()),
            ] {
                assert_eq!(
                    readings
                        .iter()
                        .map(|r| r.reading.as_str())
                        .collect::<Vec<_>>(),
                    expected,
                    "{text}"
                );
            }
        }
    }

    #[test]
    fn conjugation_classes_keep_their_readings_and_kuru_changes_its_hidden_stem() {
        let dictionary = BTreeMap::from([
            ("居る".into(), vec!["いる".into(), "おる".into()]),
            (
                "来る".into(),
                vec![
                    DictionaryReading {
                        reading: "くる".into(),
                        senses: vec!["come".into()],
                        evidence: vec![],
                    },
                    "きたる".into(),
                ],
            ),
            (
                "来た".into(),
                vec![DictionaryReading {
                    reading: "きた".into(),
                    senses: vec!["it's here!".into()],
                    evidence: vec![],
                }],
            ),
        ]);
        let index = Index::new(BTreeMap::from([
            (
                "居".into(),
                vec![
                    ("居る".into(), "いる".into(), "ichidan".into()),
                    ("居る".into(), "おる".into(), "godan".into()),
                ],
            ),
            (
                "来".into(),
                vec![
                    ("来る".into(), "くる".into(), "kuru".into()),
                    ("来る".into(), "きたる".into(), "godan".into()),
                ],
            ),
        ]));
        for (text, expected) in [
            ("居た", "いた"),
            ("居った", "おった"),
            ("来よう", "こよう"),
            ("来ず", "こず"),
            ("来ぬ", "こぬ"),
            ("来ん", "こん"),
            ("来させる", "こさせる"),
        ] {
            let got = index.readings(text, &dictionary);
            assert_eq!(
                got.iter().map(|r| r.reading.as_str()).collect::<Vec<_>>(),
                [expected],
                "{text}"
            );
        }
        let g = super::super::tests::word("来よう", "来る", language_utils::PartOfSpeech::Verb);
        let c = indexed_candidates(&g, Language::Japanese, &dictionary, &index);
        assert_eq!(c.readings, BTreeSet::from(["こよう".into()]));
        let g = super::super::tests::word("来た", "来る", language_utils::PartOfSpeech::Verb);
        let c = indexed_candidates(&g, Language::Japanese, &dictionary, &index);
        assert_eq!(c.senses["きた"], ["it's here!", "come"]);
    }

    #[test]
    fn fallback_and_index_share_irregular_surface_stems() {
        let dictionary = BTreeMap::from([
            ("来る".into(), vec!["くる".into()]),
            ("連れて来る".into(), vec!["つれてくる".into()]),
            ("為る".into(), vec!["する".into()]),
            ("勉強する".into(), vec!["べんきょうする".into()]),
        ]);
        for (surface, lemma, expected) in [
            ("来た", "来る", "きた"),
            ("来て", "来る", "きて"),
            ("来ない", "来る", "こない"),
            ("来なさい", "来る", "きなさい"),
            ("来よう", "来る", "こよう"),
            ("来い", "来る", "こい"),
            ("来れば", "来る", "くれば"),
            ("来られる", "来る", "こられる"),
            ("来ちゃった", "来る", "きちゃった"),
            ("連れて来なさい", "連れて来る", "つれてきなさい"),
            ("為た", "為る", "した"),
            ("為ない", "為る", "しない"),
            ("為れば", "為る", "すれば"),
            ("為よう", "為る", "しよう"),
            ("為ず", "為る", "せず"),
            ("為れる", "為る", "される"),
            ("為せる", "為る", "させる"),
            ("為れた", "為る", "された"),
            ("為せない", "為る", "させない"),
            ("勉強した", "勉強する", "べんきょうした"),
            ("勉強しない", "勉強する", "べんきょうしない"),
            ("勉強すれば", "勉強する", "べんきょうすれば"),
        ] {
            let got = inflected_readings(surface, lemma, &dictionary, &Index::default());
            assert_eq!(
                got.iter().map(|r| r.reading.as_str()).collect::<Vec<_>>(),
                [expected],
                "{surface}"
            );
        }
        let index = Index::new(BTreeMap::from([
            (
                "来".into(),
                vec![("来る".into(), "くる".into(), "kuru".into())],
            ),
            (
                "勉強".into(),
                vec![("勉強する".into(), "べんきょうする".into(), "suru".into())],
            ),
        ]));
        for (surface, expected) in [
            ("来なさい", "きなさい"),
            ("来ない", "こない"),
            ("勉強した", "べんきょうした"),
            ("勉強しよう", "べんきょうしよう"),
        ] {
            assert_eq!(index.readings(surface, &dictionary)[0].reading, expected);
        }
    }

    #[test]
    fn ii_is_not_the_stem_of_yokatta() {
        let dictionary = BTreeMap::from([("良い".into(), vec!["よい".into(), "いい".into()])]);
        let index = Index::new(BTreeMap::from([(
            "良".into(),
            vec![("良い".into(), "よい".into(), "adjective".into())],
        )]));
        let g = super::super::tests::word("良かった", "良い", language_utils::PartOfSpeech::Adj);
        let c = indexed_candidates(&g, Language::Japanese, &dictionary, &index);
        assert_eq!(c.readings, BTreeSet::from(["よかった".into()]));
        assert_eq!(
            candidates(&g, Language::Japanese, &dictionary).readings,
            c.readings
        );
    }

    #[test]
    fn homograph_verbs_need_compatible_conjugations() {
        let dictionary = BTreeMap::from([
            ("通る".into(), vec!["とおる".into()]),
            ("通う".into(), vec!["かよう".into()]),
            ("通す".into(), vec!["とおす".into()]),
            ("降る".into(), vec!["ふる".into()]),
            ("降りる".into(), vec!["おりる".into()]),
        ]);
        let index = Index::new(BTreeMap::from([
            (
                "通".into(),
                vec![
                    ("通る".into(), "とおる".into(), "godan".into()),
                    ("通う".into(), "かよう".into(), "godan".into()),
                    ("通す".into(), "とおす".into(), "godan".into()),
                ],
            ),
            (
                "降".into(),
                vec![
                    ("降る".into(), "ふる".into(), "godan".into()),
                    ("降りる".into(), "おりる".into(), "ichidan".into()),
                ],
            ),
        ]));
        for (text, expected) in [
            ("通って", vec!["とおって", "かよって"]),
            ("通ってる", vec!["とおってる", "かよってる"]),
            ("降り方", vec!["ふりかた", "おりかた"]),
        ] {
            assert_eq!(
                index
                    .readings(text, &dictionary)
                    .iter()
                    .map(|r| r.reading.as_str())
                    .collect::<Vec<_>>(),
                expected
            );
        }
        assert!(index.readings("通べた", &dictionary).is_empty());
    }
}
