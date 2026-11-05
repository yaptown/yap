use bridgerton::schema::{NativeType, Registry, Type};

#[bridgerton::bridge(transparent)]
#[derive(serde::Serialize, serde::Deserialize)]
struct Defaulted<T = String, U = T> {
    first: T,
    second: U,
}

#[test]
fn native_names_omit_only_matching_trailing_defaults() {
    let mut registry = Registry::default();
    assert_eq!(
        Defaulted::<String>::native_type(&mut registry),
        Type::Named("Defaulted".into())
    );
    assert_eq!(
        Defaulted::<u32>::native_type(&mut registry),
        Type::Named("Defaulted_UInt32".into())
    );
    assert_eq!(
        Defaulted::<String, u32>::native_type(&mut registry),
        Type::Named("Defaulted_String_UInt32".into())
    );
    // Box<String> shares String's wire shape, but is not the declared default.
    assert_eq!(
        Defaulted::<Box<String>>::native_type(&mut registry),
        Type::Named("Defaulted_String".into())
    );
    registry.validate().unwrap();
}
