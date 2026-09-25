//! Learner problem reports, filed from a challenge's "⋯" menu on both hosts.
use crate::supabase::supabase_config;
use crate::{CardContent, TranscribeComprehensibleSentence, TranslateComprehensibleSentence};
use bridgerton::Error;
use language_utils::Language;
use weapon::supabase::SupabaseConfig;

/// The challenge a report is about; its JSON is stored with the report.
#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum IssueSubject {
    Flashcard(CardContent),
    Translation(TranslateComprehensibleSentence),
    Transcription(TranscribeComprehensibleSentence),
}

#[bridgerton::bridge(transparent)]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ReportIssueCopy {
    pub menu_label: String,
    pub title: String,
    pub description: String,
    pub field_label: String,
    pub placeholder: String,
    pub cancel_label: String,
    pub submit_label: String,
    pub submitting_label: String,
    pub failed_label: String,
}

#[bridgerton::bridge]
pub fn report_issue_copy() -> ReportIssueCopy {
    ReportIssueCopy {
        menu_label: "Report an Issue".into(),
        title: "Report an Issue".into(),
        description:
            "Describe the issue you're experiencing. We'll look into it as soon as possible.".into(),
        field_label: "Issue description".into(),
        placeholder: "Please describe the issue you're experiencing...".into(),
        cancel_label: "Cancel".into(),
        submit_label: "Submit Issue".into(),
        submitting_label: "Submitting...".into(),
        failed_label: "Couldn't send your report. Please try again.".into(),
    }
}

fn issue_text(language: Language, subject: &IssueSubject, issue: &str) -> String {
    let context = match subject {
        IssueSubject::Flashcard(content) => serde_json::to_string(content).unwrap(),
        IssueSubject::Translation(sentence) => format!(
            "Sentence challenge: {}",
            serde_json::to_string(sentence).unwrap()
        ),
        IssueSubject::Transcription(challenge) => format!(
            "Transcription challenge: {}",
            serde_json::to_string(challenge).unwrap()
        ),
    };
    let language = serde_json::to_value(language).unwrap();
    let language = language.as_str().unwrap();
    format!(
        "Language: {language}\n\nContext: {context}\n\nIssue: {}",
        issue.trim()
    )
}

/// Stores a report in Supabase's `issues` table as the signed-in learner.
#[bridgerton::bridge]
pub async fn report_issue(
    language: Language,
    subject: IssueSubject,
    issue: String,
    user_id: String,
    access_token: String,
) -> Result<(), Error> {
    let SupabaseConfig {
        supabase_url,
        supabase_anon_key,
    } = supabase_config();
    let response = fetch_happen::Client
        .post(format!("{supabase_url}/rest/v1/issues"))
        .header("apikey", &supabase_anon_key)
        .header("Authorization", format!("Bearer {access_token}"))
        .json(&serde_json::json!({
            "user_id": user_id,
            "issue_text": issue_text(language, &subject, &issue),
        }))
        .map_err(|e| Error::new(format!("{e:?}")))?
        .send()
        .await
        .map_err(|e| Error::new(format!("{e:?}")))?;
    if !response.ok() {
        return Err(Error::new(format!(
            "Couldn't send your report ({})",
            response.status()
        )));
    }
    Ok(())
}
