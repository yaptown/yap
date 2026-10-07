//! Shared UI copy. UI language is independent of the language being learned.
use std::sync::atomic::{AtomicU8, Ordering};

pub use fluent_bundle::{FluentArgs, FluentValue};
use fluent_bundle::{FluentBundle, FluentResource};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum UiLanguage {
    #[default]
    English,
    French,
    Spanish,
    Chinese,
}

impl UiLanguage {
    pub const ALL: [Self; 4] = [Self::English, Self::French, Self::Spanish, Self::Chinese];

    pub const fn tag(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::French => "fr",
            Self::Spanish => "es",
            Self::Chinese => "zh-Hans",
        }
    }

    pub const fn source(self) -> &'static str {
        match self {
            Self::English => include_str!("../locales/en/yap.ftl"),
            Self::French => include_str!("../locales/fr/yap.ftl"),
            Self::Spanish => include_str!("../locales/es/yap.ftl"),
            Self::Chinese => include_str!("../locales/zh-Hans/yap.ftl"),
        }
    }
}

static LANGUAGE: AtomicU8 = AtomicU8::new(UiLanguage::English as u8);

pub fn negotiate(preferences: &[String]) -> UiLanguage {
    preferences
        .iter()
        .find_map(
            |tag| match tag.split(['-', '_']).next()?.to_ascii_lowercase().as_str() {
                "en" => Some(UiLanguage::English),
                "fr" => Some(UiLanguage::French),
                "es" => Some(UiLanguage::Spanish),
                "zh" => Some(UiLanguage::Chinese),
                _ => None,
            },
        )
        .unwrap_or_default()
}

pub fn set_ui_language(tag: &str) {
    LANGUAGE.store(negotiate(&[tag.to_owned()]) as u8, Ordering::Relaxed);
}

pub fn ui_language() -> UiLanguage {
    UiLanguage::ALL[LANGUAGE.load(Ordering::Relaxed) as usize]
}

fn bundle(language: UiLanguage) -> FluentBundle<FluentResource> {
    let mut bundle = FluentBundle::new(vec![language.tag().parse().unwrap()]);
    bundle.set_use_isolating(false);
    let resource =
        FluentResource::try_new(language.source().to_owned()).expect("invalid UI catalog");
    bundle
        .add_resource(resource)
        .expect("duplicate UI messages");
    bundle
}

thread_local! {
    // Fluent's memoizer is thread-local; the chosen language is process-wide.
    static BUNDLES: [FluentBundle<FluentResource>; 4] = UiLanguage::ALL.map(bundle);
}

pub fn translate(id: &str, args: Option<&FluentArgs<'_>>) -> String {
    translate_in(ui_language(), id, args)
}

pub fn translate_in(language: UiLanguage, id: &str, args: Option<&FluentArgs<'_>>) -> String {
    BUNDLES.with(|bundles| format_in(bundles, language, id, args))
}

fn format_in(
    bundles: &[FluentBundle<FluentResource>; 4],
    language: UiLanguage,
    id: &str,
    args: Option<&FluentArgs<'_>>,
) -> String {
    let (message, attribute) = id.split_once('.').map_or((id, None), |(m, a)| (m, Some(a)));
    let pattern = |index: usize| {
        let message = bundles[index].get_message(message)?;
        match attribute {
            Some(name) => message.get_attribute(name).map(|a| a.value()),
            None => message.value(),
        }
    };
    let english = pattern(0).unwrap_or_else(|| panic!("missing English UI message: {id}"));
    let (bundle, pattern) = match pattern(language as usize) {
        Some(pattern) => (&bundles[language as usize], pattern),
        None => (&bundles[0], english),
    };
    let mut errors = Vec::new();
    let result = bundle
        .format_pattern(pattern, args, &mut errors)
        .into_owned();
    assert!(errors.is_empty(), "invalid UI message {id}: {errors:?}");
    result
}

#[macro_export]
macro_rules! tr {
    ($id:literal $(,)?) => { $crate::translate($id, None) };
    ($id:literal, $($name:ident = $value:expr),+ $(,)?) => {{
        let mut args = $crate::FluentArgs::new();
        $(args.set(stringify!($name), $value);)+
        $crate::translate($id, Some(&args))
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_translation_uses_english_for_messages_and_attributes() {
        let mut bundles = UiLanguage::ALL.map(bundle);
        bundles[UiLanguage::French as usize] = FluentBundle::new(vec!["fr".parse().unwrap()]);
        assert_eq!(
            format_in(&bundles, UiLanguage::French, "common-continue", None),
            "Continue"
        );
        assert_eq!(
            format_in(
                &bundles,
                UiLanguage::French,
                "next-review-word.before",
                None
            ),
            "You'll review "
        );
    }

    #[test]
    fn selected_language_is_visible_on_other_threads() {
        set_ui_language("fr-CA");
        assert_eq!(
            std::thread::spawn(ui_language).join().unwrap(),
            UiLanguage::French
        );
        set_ui_language("en");
    }
}
