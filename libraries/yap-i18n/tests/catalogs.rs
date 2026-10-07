use fluent_syntax::ast::{self, Entry, Expression, InlineExpression, PatternElement};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};
use yap_i18n::{FluentArgs, UiLanguage, negotiate, translate_in};

type Catalog = BTreeMap<String, BTreeSet<String>>;

fn inline_variables(expression: &InlineExpression<&str>, variables: &mut BTreeSet<String>) {
    match expression {
        InlineExpression::VariableReference { id } => {
            variables.insert(id.name.to_owned());
        }
        InlineExpression::Placeable { expression } => expression_variables(expression, variables),
        InlineExpression::FunctionReference { arguments, .. } => {
            argument_variables(arguments, variables)
        }
        InlineExpression::TermReference {
            arguments: Some(arguments),
            ..
        } => argument_variables(arguments, variables),
        _ => {}
    }
}

fn argument_variables(arguments: &ast::CallArguments<&str>, variables: &mut BTreeSet<String>) {
    for argument in &arguments.positional {
        inline_variables(argument, variables);
    }
    for argument in &arguments.named {
        inline_variables(&argument.value, variables);
    }
}

fn expression_variables(expression: &Expression<&str>, variables: &mut BTreeSet<String>) {
    match expression {
        Expression::Inline(inline) => inline_variables(inline, variables),
        Expression::Select { selector, variants } => {
            inline_variables(selector, variables);
            for variant in variants {
                pattern_variables(&variant.value, variables);
            }
        }
    }
}

fn pattern_variables(pattern: &ast::Pattern<&str>, variables: &mut BTreeSet<String>) {
    for element in &pattern.elements {
        if let PatternElement::Placeable { expression } = element {
            expression_variables(expression, variables);
        }
    }
}

fn catalog(language: UiLanguage) -> Catalog {
    let resource = fluent_syntax::parser::parse(language.source()).expect("catalog must parse");
    let mut entries = Catalog::new();
    for entry in resource.body {
        let Entry::Message(message) = entry else {
            continue;
        };
        for (key, value) in message
            .value
            .into_iter()
            .map(|v| (message.id.name.to_owned(), v))
            .chain(
                message
                    .attributes
                    .into_iter()
                    .map(|a| (format!("{}.{}", message.id.name, a.id.name), a.value)),
            )
        {
            let mut variables = BTreeSet::new();
            pattern_variables(&value, &mut variables);
            assert!(
                entries.insert(key.clone(), variables).is_none(),
                "duplicate {key}"
            );
        }
    }
    entries
}

#[test]
fn catalogs_have_exact_ids_attributes_and_variables() {
    let english = catalog(UiLanguage::English);
    let variables_by_message = |catalog: &Catalog| {
        let mut messages: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (id, variables) in catalog {
            let message = id.split('.').next().unwrap().to_owned();
            messages
                .entry(message)
                .or_default()
                .extend(variables.iter().cloned());
        }
        messages
    };
    for language in UiLanguage::ALL {
        let translated = catalog(language);
        assert_eq!(
            translated.keys().collect::<Vec<_>>(),
            english.keys().collect::<Vec<_>>(),
            "{} IDs and attributes",
            language.tag()
        );
        // Variables may move between rich-text attributes, but never disappear.
        assert_eq!(
            variables_by_message(&translated),
            variables_by_message(&english),
            "{} variables",
            language.tag()
        );
    }
}

#[test]
fn negotiates_the_first_supported_primary_subtag() {
    for (preferences, expected) in [
        (vec!["de", "fr-CA", "es"], UiLanguage::French),
        (vec!["en-GB", "fr"], UiLanguage::English),
        (vec!["zh-Hant-TW", "en"], UiLanguage::Chinese),
        (vec!["ES_mx"], UiLanguage::Spanish),
        (vec!["ja"], UiLanguage::English),
        (vec![], UiLanguage::English),
    ] {
        assert_eq!(
            negotiate(
                &preferences
                    .into_iter()
                    .map(String::from)
                    .collect::<Vec<_>>()
            ),
            expected
        );
    }
}

#[test]
fn plurals_format_in_every_locale() {
    for language in UiLanguage::ALL {
        for count in [0, 1, 2, 1000] {
            for (id, variables) in catalog(language) {
                if !variables.contains("count") {
                    continue;
                }
                let mut args = FluentArgs::new();
                for variable in variables {
                    args.set(variable, "example");
                }
                args.set("count", count);
                assert!(!translate_in(language, &id, Some(&args)).is_empty(), "{id}");
            }
        }
    }
    let mut args = FluentArgs::new();
    args.set("count", 1);
    assert_eq!(
        translate_in(UiLanguage::English, "review-cards", Some(&args)),
        "Review 1 card"
    );
    args.set("count", 1000);
    assert_eq!(
        translate_in(UiLanguage::English, "review-cards", Some(&args)),
        "Review 1000 cards"
    );
}

#[test]
#[should_panic(expected = "missing English UI message")]
fn missing_english_is_a_programming_error() {
    translate_in(UiLanguage::French, "intentionally-missing", None);
}

#[test]
fn source_references_exist_and_every_message_is_used() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let english = catalog(UiLanguage::English);
    let reference =
        regex::Regex::new(r#"\b(?:tr!|t|L|uiText|rich)\s*\(\s*["']([a-z][a-z0-9.-]+)["']"#)
            .unwrap();
    let mut used = BTreeSet::new();
    for directory in [
        "yap-frontend-reducers/src",
        "yap-frontend-rs/src",
        "yap-frontend/src",
        "yap-frontend/vite.config.ts",
        "yap-ios/src",
        "yap-ios/Sources",
    ] {
        for entry in walkdir::WalkDir::new(root.join(directory)) {
            let entry = entry.unwrap();
            if !entry.file_type().is_file() {
                continue;
            }
            if !matches!(
                entry.path().extension().and_then(|x| x.to_str()),
                Some("rs" | "ts" | "tsx" | "swift")
            ) {
                continue;
            }
            let source = fs::read_to_string(entry.path()).unwrap();
            for capture in reference.captures_iter(&source) {
                let id = capture[1].to_owned();
                assert!(
                    english.contains_key(&id),
                    "{} references missing {id}",
                    entry.path().display()
                );
                used.insert(id);
            }
        }
    }
    // relative_time selects one of this closed set by the TIERS unit name.
    let screens = fs::read_to_string(root.join("yap-frontend-rs/src/screens.rs")).unwrap();
    assert!(screens.contains("format!(\"time-{name}\")"));
    for unit in ["second", "minute", "hour", "day", "week", "month", "year"] {
        let id = format!("time-{unit}");
        assert!(english.contains_key(&id));
        used.insert(id);
    }
    let unused: Vec<_> = english.keys().filter(|key| !used.contains(*key)).collect();
    assert!(unused.is_empty(), "unused English UI messages: {unused:?}");
}
