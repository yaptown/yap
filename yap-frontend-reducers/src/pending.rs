//! Shared identity for disposable review drafts. Hosts only handle storage/codecs.
use crate::TranslateComprehensibleSentence;
use language_utils::transcription_challenge::Part;
use sha2::{Digest, Sha256};

#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, Clone, Debug)]
pub struct PendingReviewSlot {
    pub key: String,
    pub identity: String,
}

fn pending_slot(
    kind: &str,
    challenge: &impl serde::Serialize,
    scope: String,
    app_version: String,
    review_count: u64,
) -> PendingReviewSlot {
    let encoded = serde_json::to_vec(challenge).expect("review challenges serialize to JSON");
    let digest = hex::encode(Sha256::digest(encoded));
    PendingReviewSlot {
        key: format!("yap-pending-{kind}-{scope}"),
        identity: format!("v1-{app_version}-{review_count}-{digest}"),
    }
}

#[bridgerton::bridge]
pub fn translation_pending_slot(
    sentence: TranslateComprehensibleSentence,
    scope: String,
    app_version: String,
    review_count: u64,
) -> PendingReviewSlot {
    pending_slot("translation", &sentence, scope, app_version, review_count)
}

#[bridgerton::bridge]
pub fn transcription_pending_slot(
    parts: Vec<Part>,
    scope: String,
    app_version: String,
    review_count: u64,
) -> PendingReviewSlot {
    pending_slot("transcription", &parts, scope, app_version, review_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_scope_storage_and_invalidate_changed_drafts() {
        let slot = transcription_pending_slot(vec![], "account:fra:eng".into(), "build".into(), 42);
        assert_eq!(slot.key, "yap-pending-transcription-account:fra:eng");
        assert_eq!(
            slot.identity,
            "v1-build-42-4f53cda18c2baa0c0354bb5f9a3ecbe5ed12ab4d8e11ba873c2f11161202b945"
        );
        let again =
            transcription_pending_slot(vec![], "account:fra:eng".into(), "build".into(), 42);
        assert_eq!(slot.identity, again.identity);
        for scope in ["other:fra:eng", "account:spa:eng"] {
            assert_ne!(
                slot.key,
                transcription_pending_slot(vec![], scope.into(), "build".into(), 42).key
            );
        }
        for (version, reviews) in [("next-build", 42), ("build", 43)] {
            let changed = transcription_pending_slot(
                vec![],
                "account:fra:eng".into(),
                version.into(),
                reviews,
            );
            assert_eq!(slot.key, changed.key);
            assert_ne!(slot.identity, changed.identity);
        }
        let changed = transcription_pending_slot(
            vec![Part::AskedToTranscribe { parts: vec![] }],
            "account:fra:eng".into(),
            "build".into(),
            42,
        );
        assert_ne!(slot.identity, changed.identity);
        assert_ne!(
            slot.key,
            pending_slot(
                "translation",
                &Vec::<Part>::new(),
                "account:fra:eng".into(),
                "build".into(),
                42
            )
            .key
        );
    }
}

#[cfg(test)]
mod sense_tests {
    use super::*;
    use language_utils::{Gram, TaggedGram};
    #[test]
    fn tagging_and_changing_sense_invalidate_drafts() {
        let bare = Gram::<String>(vec![]);
        let mut entry = TaggedGram {
            gram: bare.clone(),
            sense: std::num::NonZeroU32::new(1),
        };
        let old = pending_slot("translation", &bare, "scope".into(), "build".into(), 0);
        let first = pending_slot("translation", &entry, "scope".into(), "build".into(), 0);
        assert_eq!(
            first.identity,
            "v1-build-0-e277234761763fcdb0967bbf5ce60c66c548aea3fe1905327075e2e6b7a43dd3"
        );
        entry.sense = std::num::NonZeroU32::new(2);
        let second = pending_slot("translation", &entry, "scope".into(), "build".into(), 0);
        assert_eq!(old.key, first.key);
        assert_eq!(first.key, second.key);
        assert_ne!(old.identity, first.identity);
        assert_ne!(first.identity, second.identity);
    }
}
