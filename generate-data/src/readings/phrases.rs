//! Citation readings reuse actual token evidence before asking about unmatched headwords.
use super::*;
use language_utils::TaggedGram;

struct PhraseRequest {
    gram: Gram<String>,
    gloss: String,
    examples: Vec<String>,
}

impl PhraseRequest {
    fn prompt(&self, language: Language, dictionary: &Dictionary) -> String {
        serde_json::json!({
            "headword": self.gram.to_display_string(language),
            "sense_gloss": self.gloss,
            "dictionary_candidates": indexed_candidates(&self.gram, language, dictionary, &inflections::Index::default()),
            "matched_sentences": self.examples,
        }).to_string()
    }
}

#[derive(Deserialize, JsonSchema)]
struct PhraseAnswer {
    reading: String,
}

fn system(language: Language) -> String {
    let notation = match language {
        Language::Japanese => "hiragana, preserving the headword's kana and particle spellings",
        Language::ChineseSimplified => "space-separated pinyin with tone marks and lexical tones",
        _ => unreachable!(),
    };
    format!(
        "Give the pronunciation of the entire citation headword in {notation}. The sense gloss and sentences with matched words marked by «» provide evidence for its meaning. Matches may be inflected or separated by other words, so return the citation form's reading rather than the pronunciation of an example. Dictionary candidates are evidence, not an exhaustive list. Return one reading."
    )
}

fn most_common(counts: &BTreeMap<String, usize>) -> Option<String> {
    counts
        .iter()
        .max_by_key(|(reading, count)| (**count, std::cmp::Reverse(*reading)))
        .map(|(reading, _)| reading.clone())
}

impl CorpusReadings {
    /// Run after sense assignment: overlays use the same persistent default sense
    /// IDs as tokens. Prefer that sense's evidence, falling back to the gram's
    /// evidence only when its default sense has no checked reading.
    pub async fn read_phrases(
        &mut self,
        language: Language,
        app: &BTreeMap<String, SentenceInfo>,
        restricted: &BTreeMap<String, SentenceInfo>,
        interners: &GramInterners,
        inventories: &BTreeMap<Gram<String>, UsageInventory>,
    ) -> Result<()> {
        if !supports_readings(language) {
            return Ok(());
        }
        let phrases = self.canonical_phrases(app);
        let mut observed = BTreeSet::new();
        let mut by_gram: BTreeMap<Gram<String>, BTreeMap<String, usize>> = BTreeMap::new();
        let mut by_sense: BTreeMap<TaggedGram<Gram<String>>, BTreeMap<String, usize>> =
            BTreeMap::new();
        for (text, info) in app.iter().chain(
            restricted
                .iter()
                .filter(|(text, _)| !app.contains_key(*text)),
        ) {
            for (index, token) in info.sentence.tokens.iter().enumerate() {
                let gram = interners
                    .grams
                    .resolve(&token.gram)
                    .resolve(&interners.strings);
                if !phrases.contains_key(&gram) {
                    continue;
                }
                observed.insert(gram.clone());
                if let Some(reading) = self.reading(text, index) {
                    *by_gram
                        .entry(gram.clone())
                        .or_default()
                        .entry(reading.into())
                        .or_default() += 1;
                    *by_sense
                        .entry(TaggedGram {
                            gram,
                            sense: token.sense,
                        })
                        .or_default()
                        .entry(reading.into())
                        .or_default() += 1;
                }
            }
        }
        for (gram, phrase) in &phrases {
            if let Some(reading) = by_sense
                .get(&phrase.sense)
                .or_else(|| by_gram.get(gram))
                .and_then(most_common)
            {
                self.headwords.insert(gram.clone(), reading);
            }
        }
        let discovered = crate::usage_discovery::load_discovered_terms(language)?;
        let mut requests = BTreeMap::new();
        for info in app.values() {
            for term in info
                .multiword_terms
                .high_confidence
                .iter()
                .chain(&info.multiword_terms.low_confidence)
            {
                let gram = &term.gram.gram;
                if observed.contains(gram) {
                    continue;
                }
                let request = requests.entry(gram.clone()).or_insert_with(|| {
                    let gloss = inventories
                        .get(gram)
                        .and_then(|i| {
                            term.gram
                                .sense
                                .and_then(|s| i.usages.get(s.get() as usize - 1))
                        })
                        .map(|u| u.gloss.clone())
                        .or_else(|| {
                            discovered
                                .iter()
                                .find(|t| t.citation.as_ref().unwrap_or(&t.gram) == gram)
                                .map(|t| t.gloss.clone())
                        })
                        .unwrap_or_default();
                    PhraseRequest {
                        gram: gram.clone(),
                        gloss,
                        examples: Vec::new(),
                    }
                });
                if request.examples.len() < 5 {
                    let marked: String = info
                        .decode_words(interners, language)
                        .iter()
                        .enumerate()
                        .map(|(i, word)| {
                            let surface = if term.matched_word_indices.contains(&(i as u16)) {
                                format!("«{}»", word.word.text)
                            } else {
                                word.word.text.clone()
                            };
                            format!("{surface}{}", word.whitespace.as_str())
                        })
                        .collect();
                    if !request.examples.contains(&marked) {
                        request.examples.push(marked);
                    }
                }
            }
        }
        if requests.is_empty() {
            return Ok(());
        }
        let requests: Vec<_> = requests.into_values().collect();
        let (model, effort) = hierarchy::settings(language);
        let client =
            crate::apply_cache_only(crate::base_chat_client(model).with_reasoning_effort(effort));
        tysm::measurement::stage(&format!("{}:phrase-readings", language.code()));
        for (request, answer) in
            batch::<_, PhraseAnswer>(&client, &system(language), &requests, |r| {
                r.prompt(language, &self.dictionary)
            })
            .await?
        {
            let answer = answer?;
            let headword = request.gram.to_display_string(language);
            // A reading that contradicts the headword's own kana leaves the
            // phrase unannotated, like a token phrase with no checked reading.
            match canonical::checked_surface_reading(language, &headword, &answer.reading) {
                Some(reading) => {
                    self.headwords.insert(request.gram.clone(), reading);
                }
                None => log::warn!(
                    "rejected citation reading for {headword}: {}",
                    answer.reading
                ),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn token_evidence_prefers_the_phrase_sense_without_inference() {
        use language_utils::{EncodedSentence, MultiwordTermMatch, MultiwordTerms};
        let gram = super::super::tests::gram(&["生", "物"]);
        let mut strings = lasso::Rodeo::default();
        let mut grams = lasso::Rodeo::default();
        let key = grams.get_or_intern(gram.get_or_intern(&mut strings));
        let interners = GramInterners {
            strings: strings.into_reader(),
            grams: grams.into_reader(),
        };
        let phrase = TaggedGram {
            gram: gram.clone(),
            sense: NonZeroU32::new(1),
        };
        let mut app = BTreeMap::new();
        let mut data = CorpusReadings::default();
        for (text, sense, reading) in [
            ("a", 1, "せいぶつ"),
            ("b", 2, "なまもの"),
            ("c", 2, "なまもの"),
        ] {
            app.insert(
                text.into(),
                SentenceInfo {
                    sentence: EncodedSentence {
                        tokens: vec![TaggedGram {
                            gram: key,
                            sense: NonZeroU32::new(sense),
                        }],
                        capitalize_first: false,
                    },
                    multiword_terms: MultiwordTerms {
                        high_confidence: vec![MultiwordTermMatch {
                            gram: phrase.clone(),
                            matched_word_indices: vec![0, 1],
                        }],
                        low_confidence: vec![],
                    },
                },
            );
            data.labels.insert(
                (text.into(), 0),
                occurrence::Label::Ranked {
                    choices: vec![Choice {
                        sense,
                        reading: reading.into(),
                    }],
                },
            );
        }
        data.read_phrases(
            Language::Japanese,
            &app,
            &BTreeMap::new(),
            &interners,
            &BTreeMap::new(),
        )
        .await
        .unwrap();
        assert_eq!(
            data.canonical_phrases(&app)[&gram].reading.as_deref(),
            Some("せいぶつ")
        );
        // A default sense without token evidence uses the gram-wide mode.
        for info in app.values_mut() {
            info.multiword_terms.high_confidence[0].gram.sense = NonZeroU32::new(3);
        }
        data.read_phrases(
            Language::Japanese,
            &app,
            &BTreeMap::new(),
            &interners,
            &BTreeMap::new(),
        )
        .await
        .unwrap();
        assert_eq!(
            data.canonical_phrases(&app)[&gram].reading.as_deref(),
            Some("なまもの")
        );
    }
}
