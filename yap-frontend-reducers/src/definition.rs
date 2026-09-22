//! Shared definition labels, matching the web's morphology display.

use language_utils::features::{Gender, Morphology, Person, Polite, Tense};

/// Only person, gender, tense, politeness, and case are displayed, in that order.
#[bridgerton::bridge]
pub fn morphology_label(morphology: Morphology) -> String {
    let mut parts = Vec::new();

    if let Some(person) = morphology.person {
        parts.push(match person {
            Person::Zeroth => "0th-person",
            Person::First => "1st-person",
            Person::Second => "2nd-person",
            Person::Third => "3rd-person",
            Person::Fourth => "4th-person",
        });
    }

    if let Some(gender) = morphology.gender {
        parts.push(match gender {
            Gender::Masculine => "masculine",
            Gender::Feminine => "feminine",
            Gender::Neuter => "neuter",
            Gender::Common => "common",
        });
    }

    if let Some(tense) = morphology.tense {
        parts.push(match tense {
            Tense::Past => "past tense",
            Tense::Present => "present tense",
            Tense::Future => "future tense",
            Tense::Imperfect => "imperfect tense",
            Tense::Pluperfect => "pluperfect tense",
        });
    }

    if let Some(politeness) = morphology.politeness {
        parts.push(match politeness {
            Polite::Intimate => "intimate",
            Polite::Informal => "informal",
            Polite::Formal => "formal",
            Polite::Elev => "elevated",
            Polite::Humb => "humble",
        });
    }

    // All 37 web caseMap labels are the lowercased variant name; this also
    // supplies the same fallback for any additional Rust case variants.
    let case = morphology
        .case
        .map(|case| format!("{case:?}").to_lowercase());
    if let Some(case) = case.as_deref() {
        parts.push(case);
    }

    parts.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use language_utils::features::{Aspect, Case, Mood, Number};

    #[test]
    fn empty_morphology_has_no_label() {
        assert_eq!(morphology_label(Morphology::default()), "");
    }

    #[test]
    fn person_precedes_tense() {
        assert_eq!(
            morphology_label(Morphology {
                person: Some(Person::First),
                tense: Some(Tense::Past),
                ..Default::default()
            }),
            "1st-person, past tense"
        );
    }

    #[test]
    fn displays_only_web_fields_in_order() {
        assert_eq!(
            morphology_label(Morphology {
                person: Some(Person::Third),
                gender: Some(Gender::Feminine),
                tense: Some(Tense::Pluperfect),
                politeness: Some(Polite::Humb),
                case: Some(Case::Superessive),
                number: Some(Number::Plural),
                mood: Some(Mood::Indicative),
                aspect: Some(Aspect::Perfect),
            }),
            "3rd-person, feminine, pluperfect tense, humble, superessive"
        );
    }
}
