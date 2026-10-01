//! Reviewed exceptions for films whose audio verdict names no variety.
use language_utils::Language::{self, *};

pub(super) const OVERRIDES: &[(&str, Language)] = &[
    // Spain productions; all five IDs verified against the corpus plan.
    ("tt8291806", SpanishPeninsular), // Pain and Glory
    ("tt8228288", SpanishPeninsular), // The Platform
    ("tt0256009", SpanishPeninsular), // The Devil's Backbone
    ("tt0464141", SpanishPeninsular), // The Orphanage
    ("tt0070040", SpanishPeninsular), // The Spirit of the Beehive
];

pub(super) fn heard(fallback: Language, name: &str) -> Option<Language> {
    let name = name.to_lowercase();
    match fallback.corpus_code() {
        "spa" if name.contains("spanish") => {
            if ["european", "peninsular", "castilian", "spain"]
                .iter()
                .any(|s| name.contains(s))
            {
                Some(SpanishPeninsular)
            } else if [
                "latin american",
                "argentin",
                "rioplatense",
                "mexic",
                "chilean",
            ]
            .iter()
            .any(|s| name.contains(s))
            {
                Some(SpanishLatinAmerican)
            } else {
                None
            }
        }
        "por" if name.contains("portuguese") => {
            if name.contains("brazil") {
                Some(PortugueseBrazilian)
            } else if name.contains("european") || name.contains("portugal") {
                Some(PortugueseEuropean)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::{Movie, Source};
    use serde_json::json;

    #[test]
    fn override_then_same_track_hearing_then_corpus_default() {
        let root = tempfile::tempdir().unwrap();
        let mut movie = Movie {
            imdb_id: "tt0".into(),
            title: "test".into(),
            year: None,
            path: "film.mkv".into(),
            original_language: "Portuguese".into(),
            source: Source::Missing,
        };
        let dir = root.path().join("tt0");
        std::fs::create_dir(&dir).unwrap();
        assert_eq!(movie.course(root.path()), Some(PortugueseBrazilian));
        let stream = json!({"stream_index":0,"codec":"opus","channels":1});
        std::fs::write(
            dir.join("audio.json"),
            json!({"filename":"film.mkv","duration_ms":1000,"stream":stream}).to_string(),
        )
        .unwrap();
        let mut check = json!({"model":"old-model","expected":"Brazilian Portuguese", "filename":"film.mkv", "stream":stream,
            "verdict":{"spoken_language":"European Portuguese", "expected_language_spoken":false,
            "enough_dialogue":true, "commentary":false,"confidence":"high","notes":"old narrow criterion"}});
        let write = |check: &serde_json::Value| {
            std::fs::write(dir.join("audio-check.json"), check.to_string()).unwrap()
        };
        write(&check);
        assert_eq!(movie.course(root.path()), Some(PortugueseEuropean));
        assert_eq!(movie.course(root.path()).unwrap().corpus_code(), "por");
        check["verdict"]["spoken_language"] = "European Spanish".into();
        write(&check);
        assert_eq!(movie.course(root.path()), Some(PortugueseBrazilian));
        movie.original_language = "Spanish".into();
        assert_eq!(movie.course(root.path()), Some(SpanishPeninsular));
        check["stream"]["stream_index"] = 1.into();
        write(&check);
        assert_eq!(movie.course(root.path()), Some(SpanishLatinAmerican));
        check["stream"]["stream_index"] = 0.into();
        check["verdict"]["enough_dialogue"] = false.into();
        write(&check);
        assert_eq!(movie.course(root.path()), Some(SpanishLatinAmerican));
        check["verdict"]["enough_dialogue"] = true.into();
        check["verdict"]["commentary"] = true.into();
        write(&check);
        assert_eq!(movie.course(root.path()), Some(SpanishLatinAmerican));
        for &(id, expected) in OVERRIDES {
            movie.imdb_id = id.into();
            assert_eq!(movie.course(root.path()), Some(expected));
            assert_eq!(expected.corpus_code(), "spa");
        }
        movie.imdb_id = "tt16277242".into(); // Society of the Snow: not Spain Spanish.
        assert_eq!(movie.course(root.path()), Some(SpanishLatinAmerican));
        assert_eq!(
            crate::audio_check::expected_language("Portuguese"),
            "Portuguese"
        );
    }
}
