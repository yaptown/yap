//! File-level language and text-integrity gate shared by subtitle consumers.

use std::sync::LazyLock;

use language_utils::Language;
use movie_subtitles::SubtitleLine;
use serde::Deserialize;
use tysm::chat_completions::ChatClient;

static CLIENT: LazyLock<ChatClient> = LazyLock::new(|| {
    ChatClient::from_env("gpt-6-luna")
        .unwrap()
        .with_cache_directory("./.cache")
        .with_reasoning_effort("medium")
});

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SubtitleQualityResponse {
    /// Whether the subtitles match the course language/script and look intact.
    pub clean: bool,
    /// Brief explanation if not clean.
    pub reason: String,
}

fn course_language(language: Language) -> &'static str {
    match language {
        Language::Hindi => "Standard Hindi",
        Language::German => "Standard written German",
        Language::ChineseSimplified => "Mandarin Chinese (Simplified script)",
        Language::ChineseTraditional => "Mandarin Chinese (Traditional script)",
        // Standard varieties share sources; routing them to dialect courses is
        // separate from rejecting an unusable subtitle (YAP-113).
        _ => language.prompt_name(),
    }
}

fn prompt(lines: &[SubtitleLine], language: Language) -> String {
    // Eight cues from each of five evenly spaced blocks.
    let block_size = 8;
    let len = lines.len();
    let offsets = [
        5.min(len),
        len / 4,
        len / 2,
        3 * len / 4,
        len.saturating_sub(block_size),
    ];
    let mut sample = String::new();
    for (i, &start) in offsets.iter().enumerate() {
        sample.push_str(&format!("--- Block {i} ---\n"));
        for line in lines.iter().skip(start).take(block_size) {
            sample.push_str(&line.sentence);
            sample.push('\n');
        }
        sample.push('\n');
    }
    format!(
        "Is this movie subtitle file written in the course's language, {language}, and cleanly encoded? \
         Learners will study this text as examples of the target language, so a related language in the same script is not an acceptable substitute. \
         This is a file-level language, script and text-integrity check, not a judgment of literary style, translation quality, or suitability for beginners. \
         Judge only the text shown. Quote actual examples from it when rejecting; inferred characters or wording that are not present are not evidence. \
         Look for systemic problems affecting many lines:\n\
         - OCR errors: 'rhe'/'rhat' for 'the'/'that', 'II' for 'Il', 'l' (lowercase L) for 'I' \
           (e.g. 'lo' for 'Io' in Italian, 'ln' for 'In'), 'I' (capital I) for 'l' in French/Italian\n\
         - Missing diacritics: text in ASCII when the language requires accents \
           (e.g. Spanish without á/é/í/ó/ú/ñ, French without é/è/ê/à/ç)\n\
         - Wrong language or bilingual: most dialogue is in another language, or substantial untranslated dialogue in two languages is interleaved throughout the file\n\
         - A different regional language or dialect-language, even in the same script: \
           Bhojpuri, Awadhi or Maithili instead of Standard Hindi; Cantonese instead of Mandarin; \
           Swiss German dialect writing instead of Standard written German; Galician instead of Spanish. \
           Shared vocabulary and script are not enough: look for recurring distinctive grammar and morphology. \
           Hong Kong vocabulary in Mandarin-style written grammar is still Mandarin, not Cantonese\n\
         - Wrong script: for a Chinese course with a specified script, reject only if most of the text shown is written in the other script. \
           Isolated variant characters such as 麽 or 後 in an otherwise Simplified file are not a file-level problem. \
           Inspect the actual characters rather than inferring the script from regional words, particles or names\n\
         - Encoding corruption: mojibake, garbled accents, control characters, \
           Greek lookalike characters mixed into Latin text\n\
         - Formatting artifacts: {{\\an8}}, SSA/ASS tags, HTML tags\n\n\
         Standard regional varieties of the same language are acceptable here: Brazilian and European Portuguese, \
         Latin American and Peninsular Spanish, and Swiss Standard German all remain usable sources. \
         Routing those sources to particular dialect courses is a separate task. \
         Clitic placement, tu/você, vosotros, shared vocabulary or ss instead of ß alone are not rejection grounds.\n\n\
         Proper names, established loanwords, English interjections such as 'Okay', literary, poetic or archaic register, \
         slang, and ordinary dialogue fragments are not evidence against a file. Likewise, isolated credits, song lyrics, \
         foreign-language quotations, ordinary subtitle speaker markers, or individual spelling/grammar mistakes are not systemic problems. \
         Flag a file only for a sustained wrong-language or wrong-script mismatch or pervasive text-integrity damage. \
         If the language is wrong, name the apparent language and cite its recurring clues in your reason.\n\n{sample}",
        language = course_language(language)
    )
}

/// Check five sample blocks with a cached live request (not the Batch API).
/// Request errors are returned so callers can distinguish them from a pass.
pub async fn check(
    lines: &[SubtitleLine],
    language: Language,
) -> anyhow::Result<SubtitleQualityResponse> {
    if lines.len() < 10 {
        return Ok(SubtitleQualityResponse {
            clean: true,
            reason: "Too few cues for a meaningful quality check".into(),
        });
    }
    Ok(CLIENT
        .chat::<SubtitleQualityResponse>(prompt(lines, language))
        .await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_languages_and_scripts_without_gating_standard_varieties() {
        for (code, expected) in [
            ("hin", "Standard Hindi"),
            ("deu", "Standard written German"),
            ("zho-hans", "Mandarin Chinese (Simplified script)"),
            ("zho-hant", "Mandarin Chinese (Traditional script)"),
            ("spa", "Spanish"),
            ("spa-es", "Spanish"),
            ("por", "Portuguese"),
            ("por-pt", "Portuguese"),
        ] {
            let prompt = prompt(&[], Language::from_code(code).unwrap());
            assert!(prompt.starts_with(&format!(
                "Is this movie subtitle file written in the course's language, {expected}, and cleanly encoded?"
            )));
            assert!(
                prompt.contains("Standard regional varieties of the same language are acceptable")
            );
            assert!(prompt.contains("Judge only the text shown"));
            assert!(prompt.contains("Isolated variant characters"));
        }
    }

    #[test]
    fn samples_five_blocks_and_explains_same_script_mismatches() {
        let lines: Vec<_> = (0..100)
            .map(|i| SubtitleLine {
                sentence: format!("cue-{i}"),
                start_ms: i,
                end_ms: i + 1,
            })
            .collect();
        let prompt = prompt(&lines, Language::Hindi);
        assert_eq!(prompt.matches("--- Block").count(), 5);
        assert!(prompt.contains("cue-5\n") && prompt.contains("cue-99\n"));
        for language in [
            "Bhojpuri",
            "Awadhi",
            "Maithili",
            "Cantonese",
            "Swiss German",
            "Galician",
        ] {
            assert!(prompt.contains(language));
        }
        assert!(prompt.contains("Shared vocabulary and script are not enough"));
    }
}
