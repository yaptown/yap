use bridgerton::bridge;

#[bridge(transparent)]
#[derive(bridgerton::serde::Serialize, bridgerton::serde::Deserialize, Debug, PartialEq)]
#[serde(crate = "bridgerton::serde")]
pub struct ConditionalRecord {
    #[cfg(any())]
    pub absent: MissingType,
    #[cfg_attr(all(), cfg(all()))]
    pub present: u32,
}

#[bridge(transparent)]
#[derive(bridgerton::serde::Serialize, bridgerton::serde::Deserialize, Debug, PartialEq)]
#[serde(crate = "bridgerton::serde")]
pub enum ConditionalEnum {
    #[cfg(any())]
    Missing(MissingType),
    Present {
        #[cfg(any())]
        absent: MissingType,
        value: u32,
    },
    Tuple(#[cfg(any())] MissingType, u32),
}

pub type Text = str;

#[bridge]
pub fn echo_text(text: &Text) -> String {
    text.to_owned()
}

#[bridge]
pub async fn text_later(text: &str) -> String {
    super::sleep(1).await;
    text.to_owned()
}

#[bridge]
pub async fn sum_later(values: &[u32]) -> u32 {
    super::sleep(1).await;
    values.iter().sum()
}

#[bridge]
pub fn conditional_record(value: ConditionalRecord) -> ConditionalRecord {
    value
}

#[bridge]
pub fn conditional_enum(value: ConditionalEnum) -> ConditionalEnum {
    value
}

#[bridge(opaque)]
#[derive(Default)]
pub struct Selection;

#[bridge(only(new, selected))]
impl Selection {
    #[bridge(constructor)]
    pub fn new() -> Self {
        Self
    }
    pub fn selected(&self) -> u32 {
        7
    }
    pub fn hidden(&self) -> u32 {
        9
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selected_methods_are_still_available_to_rust() {
        assert_eq!(Selection::new().hidden(), 9);
    }
}

pub type Terms = Vec<super::Term>;
pub type MaybeTerms = Option<Terms>;
pub type TermsResult = Result<MaybeTerms, bridgerton::Error>;

#[bridge]
pub async fn echo_terms(terms: MaybeTerms) -> TermsResult {
    super::sleep(1).await;
    Ok(terms)
}

#[bridge]
pub fn echo_pairs(pairs: Vec<(String, Option<super::Term>)>) -> Vec<(String, Option<super::Term>)> {
    pairs
}

pub type Numbers = Vec<u32>;
#[bridge]
pub fn echo_numbers(numbers: Numbers) -> Numbers {
    numbers
}

#[bridge(transparent)]
#[derive(bridgerton::serde::Serialize, bridgerton::serde::Deserialize, Debug, PartialEq)]
#[serde(crate = "bridgerton::serde")]
pub struct TargetConfiguration {
    pub common: u32,
    #[cfg(feature = "extra")]
    pub extra: bool,
    #[cfg(target_os = "ios")]
    pub ios: bool,
}
#[bridge]
pub fn target_configuration() -> TargetConfiguration {
    TargetConfiguration {
        common: 42,
        #[cfg(feature = "extra")]
        extra: true,
        #[cfg(target_os = "ios")]
        ios: true,
    }
}
#[bridge]
pub fn keyword_arguments(r#type: String, class: u32) -> String {
    format!("{type}:{class}")
}

#[bridge]
pub fn many_objects(count: u32) -> Vec<super::Counter> {
    (0..count).map(|_| super::Counter::new()).collect()
}
#[bridge]
pub fn emit_many(callback: bridgerton::Callback<u32>, count: u32) -> Result<(), bridgerton::Error> {
    for i in 0..count {
        callback.call(i)?;
    }
    Ok(())
}
#[bridge]
pub async fn ready_value() -> u32 {
    42
}

#[bridge(transparent)]
#[derive(bridgerton::serde::Serialize, bridgerton::serde::Deserialize, Debug, PartialEq)]
#[serde(crate = "bridgerton::serde")]
#[allow(non_camel_case_types)]
pub struct actor {
    pub r#type: String,
}
#[bridge]
pub fn keyword_value(value: actor) -> actor {
    value
}

#[bridge]
impl Selection {
    pub fn handle(&self) -> u32 {
        23
    }
}

#[bridge]
pub fn generic_values(
    value: super::Envelope<Vec<Option<u64>>>,
) -> super::Envelope<Vec<Option<u64>>> {
    value
}

#[bridge(transparent, large_number_types_as_bigints, missing_as_null)]
#[derive(bridgerton::serde::Serialize, bridgerton::serde::Deserialize)]
#[serde(crate = "bridgerton::serde")]
pub struct ConfiguredEnvelope<T> {
    pub value: T,
}
#[bridge]
pub fn configured_values(
    value: ConfiguredEnvelope<Vec<Option<u64>>>,
) -> ConfiguredEnvelope<Vec<Option<u64>>> {
    value
}

#[bridge(transparent)]
#[derive(bridgerton::serde::Serialize, bridgerton::serde::Deserialize, Debug, PartialEq)]
#[serde(crate = "bridgerton::serde")]
pub struct Link {
    pub value: u32,
    pub next: Option<Box<Link>>,
}
#[bridge]
pub fn echo_link(value: Link) -> Link {
    value
}

thread_local! { static STABLE_CALLS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) }; }
#[bridge]
pub fn stable_calls() -> u32 {
    STABLE_CALLS.with(std::cell::Cell::get)
}

// Both free-function attribute orders work, without changing their signatures.
#[bridge]
#[bridgerton::stable]
pub fn stable_term(text: String) -> super::Term {
    STABLE_CALLS.with(|calls| calls.set(calls.get() + 1));
    super::Term { text, gloss: None }
}

#[bridgerton::stable(strong)]
#[bridge]
pub fn stable_bytes(value: u32) -> Option<Vec<u8>> {
    (value != 0).then(|| vec![value as u8, 2, 3])
}

#[bridge]
#[bridgerton::stable]
pub fn stable_number(value: u32) -> u32 {
    value
}

#[bridge]
#[bridgerton::stable]
pub fn stable_text(value: String) -> String {
    value
}

#[bridge]
#[bridgerton::stable]
pub fn stable_result(value: String) -> Result<String, bridgerton::Error> {
    if value.is_empty() {
        Err(bridgerton::Error::new("empty stable result"))
    } else {
        Ok(value)
    }
}

#[bridge]
#[bridgerton::stable]
pub fn stable_unit() {}

#[bridge]
pub fn debug_build() -> bool {
    cfg!(debug_assertions)
}

#[bridge]
#[bridgerton::stable]
pub fn stable_typed_result(
    fail: bool,
) -> Result<super::Term, super::native_interfaces::ReviewError> {
    if fail {
        Err(super::native_interfaces::ReviewError::Offline)
    } else {
        Ok(super::Term {
            text: "typed".into(),
            gloss: None,
        })
    }
}

#[bridge]
#[bridgerton::stable]
pub fn stable_float(value: f64) -> Vec<f64> {
    vec![value]
}
