//! Display-only morphology: never changes the first-reading prefix used by cards.
use crate::features::{Morphology, Number, Person, Polite, Tense};

pub fn morphology_label(readings: &[Morphology]) -> String {
    let mut rows: Vec<[String; 3]> = readings
        .iter()
        .map(|m| {
            let mut agreement = Vec::new();
            if let Some(person) = m.person {
                agreement.push(
                    match person {
                        Person::Zeroth => "0th-person",
                        Person::First => "1st-person",
                        Person::Second => "2nd-person",
                        Person::Third => "3rd-person",
                        Person::Fourth => "4th-person",
                    }
                    .to_owned(),
                );
            }
            if let Some(number) = m.number {
                agreement.push(
                    match number {
                        Number::Singular => "singular",
                        Number::Plural => "plural",
                        Number::Dual => "dual",
                        Number::Trial => "trial",
                        Number::Paucal => "paucal",
                        Number::GreaterPaucal => "greater paucal",
                        Number::GreaterPlural => "greater plural",
                        Number::Inverse => "inverse number",
                        Number::Count => "count plural",
                        Number::PluraleTantum => "plural-only",
                        Number::Collective => "collective",
                    }
                    .to_owned(),
                );
            }
            if let Some(gender) = m.gender {
                agreement.push(format!("{gender:?}").to_lowercase());
            }
            if let Some(polite) = m.politeness {
                agreement.push(
                    match polite {
                        Polite::Intimate => "intimate",
                        Polite::Informal => "informal",
                        Polite::Formal => "formal",
                        Polite::Elev => "elevated",
                        Polite::Humb => "humble",
                    }
                    .to_owned(),
                );
            }
            let tense = m
                .tense
                .map(|tense| match tense {
                    Tense::Past => "past tense",
                    Tense::Present => "present tense",
                    Tense::Future => "future tense",
                    Tense::Imperfect => "imperfect tense",
                    Tense::Pluperfect => "pluperfect tense",
                })
                .unwrap_or_default()
                .to_owned();
            let case = m
                .case
                .map(|case| format!("{case:?}").to_lowercase())
                .unwrap_or_default();
            [agreement.join(" "), tense, case]
        })
        .filter(|row| row.iter().any(|s| !s.is_empty()))
        .collect();
    // Keep data order. There is no frequency signal with which to rank readings.
    let mut unique = Vec::new();
    rows.retain(|row| {
        if unique.contains(row) {
            false
        } else {
            unique.push(row.clone());
            true
        }
    });
    let Some(first) = rows.first() else {
        return String::new();
    };
    let shared: Vec<_> = (0..3)
        .filter(|&column| rows.iter().all(|row| row[column] == first[column]))
        .collect();
    let mut parts = Vec::new();
    let alternatives = rows
        .iter()
        .map(|row| {
            row.iter()
                .enumerate()
                .filter(|(column, text)| !shared.contains(column) && !text.is_empty())
                .map(|(_, text)| text.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join(" / ");
    if !alternatives.is_empty() {
        parts.push(alternatives);
    }
    parts.extend(
        shared
            .into_iter()
            .filter(|&column| !first[column].is_empty())
            .map(|column| first[column].clone()),
    );
    parts.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hindi_alternatives_keep_agreement_together() {
        let readings = [
            Morphology {
                person: Some(Person::Second),
                number: Some(Number::Singular),
                politeness: Some(Polite::Intimate),
                tense: Some(Tense::Present),
                ..Default::default()
            },
            Morphology {
                person: Some(Person::Third),
                number: Some(Number::Singular),
                tense: Some(Tense::Present),
                ..Default::default()
            },
        ];
        assert_eq!(
            morphology_label(&readings),
            "2nd-person singular intimate / 3rd-person singular, present tense"
        );
        let plural = [
            Morphology {
                person: Some(Person::First),
                number: Some(Number::Plural),
                tense: Some(Tense::Present),
                ..Default::default()
            },
            Morphology {
                person: Some(Person::Third),
                number: Some(Number::Plural),
                tense: Some(Tense::Present),
                ..Default::default()
            },
            Morphology {
                person: Some(Person::Second),
                politeness: Some(Polite::Formal),
                tense: Some(Tense::Present),
                ..Default::default()
            },
        ];
        assert_eq!(
            morphology_label(&plural),
            "1st-person plural / 3rd-person plural / 2nd-person formal, present tense"
        );
        assert_eq!(morphology_label(&[]), "");
    }
}
