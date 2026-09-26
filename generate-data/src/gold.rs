//! The hand-corrected gold token set (`out/cleaned_<lang>.jsonl`) and the one
//! place that reads it.
//!
//! Gold is the LLM-cleaned, human-reviewable analysis for a subset of the app
//! sentences. It has always overridden the teacher's silver in the training
//! export; [`overlay`] is what makes the *pipeline* honour it too, so a sentence
//! we have taken the trouble to get right is analyzed the same way in the app as
//! it is in lexide's training data.
//!
//! Everything here runs through `token_corrections::fix_tokens`, exactly as the
//! silver loader does. Gold is not exempt from the correction rules: it was
//! produced by a model too, and demonstrably reproduces some of the teacher's
//! mistakes (the French `les` article tagged `PRON` under a `det` relation
//! appears in both). Canonicalizing on load is also what lets a rule written
//! today repair gold that was cleaned under an older rule set, with no
//! re-cleaning run.

use anyhow::{Context, Result};
use language_utils::{Language, PartOfSpeechTag};
use lexide::{Tokenization, Whitespace};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use token_corrections::{lexide_pos_to_tag, tag_to_lexide_pos};

/// The gold `cleaned_*.jsonl` token schema: flat text/lemma, the dependency as
/// its UD label string. Every field is required — since English joined the
/// dependency pass every gold file carries dep/head, and a missing one should
/// fail loudly rather than be papered over.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct CleanedToken {
    pub text: String,
    pub whitespace: Whitespace,
    pub pos: PartOfSpeechTag,
    pub lemma: String,
    pub dep: String,
    pub head: i32,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CleanedSentence {
    pub sentence: String,
    pub tokens: Vec<CleanedToken>,
}

/// Flatten lexide tokens into the gold schema.
pub fn to_flat(tokens: Vec<lexide::Token>) -> Vec<CleanedToken> {
    tokens
        .into_iter()
        .map(|t| CleanedToken {
            text: t.text.text,
            whitespace: t.whitespace,
            pos: lexide_pos_to_tag(t.pos),
            lemma: t.lemma.lemma,
            dep: serde_json::to_value(t.dep)
                .expect("dep serializes")
                .as_str()
                .expect("dep is a string")
                .to_string(),
            head: t.head,
        })
        .collect()
}

/// The inverse of [`to_flat`], for feeding gold back into a pipeline that speaks
/// lexide tokens. The dependency round-trips through serde because that is how
/// [`to_flat`] wrote it, so the two stay in step by construction.
pub fn to_lexide(sentence: String, tokens: Vec<CleanedToken>) -> Result<Tokenization> {
    let tokens = tokens
        .into_iter()
        .map(|t| {
            let dep = serde_json::from_value(serde_json::Value::String(t.dep.clone()))
                .with_context(|| format!("unknown dependency label {:?} in gold", t.dep))?;
            Ok(lexide::Token {
                text: lexide::Text { text: t.text },
                whitespace: t.whitespace,
                pos: tag_to_lexide_pos(t.pos),
                lemma: lexide::Lemma { lemma: t.lemma },
                dep,
                head: t.head,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Tokenization::new(sentence, tokens)?)
}

/// Path of a language's gold set, alongside the per-language `out/` dirs.
/// Gold tokenizations are of the shared corpus, so dialects share them.
pub fn gold_path(out_dir: &Path, language: Language) -> PathBuf {
    out_dir.join(format!("cleaned_{}.jsonl", language.corpus_code()))
}

/// Load gold through the same validated, canonicalized boundary as silver.
pub fn load(path: &Path, language: Language) -> Result<BTreeMap<String, Tokenization>> {
    Ok(load_with_report(path, language)?.rows)
}

pub fn load_with_report(
    path: &Path,
    language: Language,
) -> Result<crate::nlp::LoadedTokenizations> {
    crate::nlp::load_tokenizations(path, language, |line| {
        let record: CleanedSentence = serde_json::from_str(line)?;
        to_lexide(record.sentence, record.tokens)
    })
}

type GoldData = BTreeMap<String, Tokenization>;
type GoldCache = BTreeMap<Language, std::sync::Arc<GoldData>>;

fn load_as_lexide(language: Language) -> Result<std::sync::Arc<GoldData>> {
    static CACHE: OnceLock<Mutex<GoldCache>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(BTreeMap::new()));
    if let Some(hit) = cache.lock().unwrap().get(&language) {
        return Ok(hit.clone());
    }
    let loaded = std::sync::Arc::new(load(&gold_path(Path::new("./out"), language), language)?);
    cache.lock().unwrap().insert(language, loaded.clone());
    Ok(loaded)
}

/// Serve every requested sentence we have a gold analysis for from gold.
///
/// Scoped to `wanted` rather than to all of gold, because the pipeline analyzes
/// the corpus it was asked for — the training export injects gold-only
/// sentences since more data helps there, but a sentence nobody requested has
/// no business appearing in a store the app reads.
///
/// Both effects matter. A requested sentence already in the silver store has
/// its analysis replaced; a requested sentence the silver store never saw is
/// *added*, which also means it is no longer a cache miss and costs no
/// tokenizer call.
pub fn overlay<'a>(
    language: Language,
    wanted: impl IntoIterator<Item = &'a str>,
    store: &mut BTreeMap<String, Tokenization>,
) -> Result<()> {
    let loaded = load_as_lexide(language)?;
    let gold = loaded.as_ref();
    let (replaced, added) = apply_overlay(gold, wanted, store);
    if replaced > 0 || added > 0 {
        println!(
            "gold[{}]: {replaced} sentences taken from gold instead of silver, \
             {added} supplied by gold alone",
            language.code(),
        );
    }
    Ok(())
}

fn apply_overlay<'a>(
    gold: &BTreeMap<String, Tokenization>,
    wanted: impl IntoIterator<Item = &'a str>,
    store: &mut BTreeMap<String, Tokenization>,
) -> (usize, usize) {
    let (mut replaced, mut added) = (0usize, 0usize);
    for sentence in wanted {
        let Some(tokens) = gold.get(sentence) else {
            continue;
        };
        match store.insert(sentence.to_string(), tokens.clone()) {
            Some(_) => replaced += 1,
            None => added += 1,
        }
    }
    (replaced, added)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token(text: &str) -> CleanedToken {
        CleanedToken {
            text: text.into(),
            whitespace: Whitespace::None,
            pos: PartOfSpeechTag::Intj,
            lemma: text.into(),
            dep: "root".into(),
            head: 0,
        }
    }

    #[test]
    fn invalid_gold_leaves_valid_silver_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("gold.jsonl");
        let record = CleanedSentence {
            sentence: "hello".into(),
            tokens: vec![token("wrong")],
        };
        std::fs::write(&path, serde_json::to_string(&record).unwrap()).unwrap();
        let loaded = load_with_report(&path, Language::English).unwrap();
        assert_eq!(loaded.invalid_rows, 1);
        assert!(loaded.invalid_sentences.contains("hello"));
        let mut silver = BTreeMap::from([(
            "hello".into(),
            to_lexide("hello".into(), vec![token("hello")]).unwrap(),
        )]);
        assert_eq!(apply_overlay(&loaded.rows, ["hello"], &mut silver), (0, 0));
        assert_eq!(silver["hello"].tokens()[0].text.text, "hello");
    }

    #[test]
    fn valid_gold_overrides_silver_and_supplies_gold_only_sentences() {
        let mut silver_token = token("hello");
        silver_token.lemma = "silver".into();
        let mut silver = BTreeMap::from([(
            "hello".into(),
            to_lexide("hello".into(), vec![silver_token]).unwrap(),
        )]);
        let gold = ["hello", "rescued"]
            .into_iter()
            .map(|s| (s.to_owned(), to_lexide(s.into(), vec![token(s)]).unwrap()))
            .collect();
        assert_eq!(
            apply_overlay(&gold, ["hello", "rescued"], &mut silver),
            (1, 1)
        );
        assert_eq!(silver["hello"].tokens()[0].lemma.lemma, "hello");
        assert_eq!(silver["rescued"].sentence(), "rescued");
    }

    #[test]
    fn flat_and_lexide_round_trip() {
        let row = to_lexide("hello".into(), vec![token("hello")]).unwrap();
        let flat = to_flat(row.clone().into_tokens());
        assert_eq!(flat[0].dep, "root");
        let back = to_lexide(row.sentence().into(), flat).unwrap();
        assert_eq!(back.tokens(), row.tokens());
    }

    #[test]
    fn unknown_dependency_label_is_rejected() {
        let mut bogus = token("hello");
        bogus.dep = "not-a-relation".into();
        assert!(
            to_lexide("hello".into(), vec![bogus])
                .unwrap_err()
                .to_string()
                .contains("not-a-relation")
        );
    }

    #[test]
    fn gold_articles_are_corrected_on_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("gold.jsonl");
        let mut article = token("les");
        article.pos = PartOfSpeechTag::Pron;
        article.lemma = "le".into();
        article.dep = "det".into();
        let record = CleanedSentence {
            sentence: "les".into(),
            tokens: vec![article],
        };
        std::fs::write(&path, serde_json::to_string(&record).unwrap()).unwrap();
        let loaded = load(&path, Language::French).unwrap();
        assert_eq!(loaded["les"].tokens()[0].pos, lexide::PartOfSpeech::Det);
    }
}
