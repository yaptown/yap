//! Type-directed WASM conversion. Rust resolves aliases before selecting an ABI.
use std::marker::PhantomData;
use wasm_bindgen::{
    JsValue,
    convert::{FromWasmAbi, IntoWasmAbi, OptionFromWasmAbi},
    describe::{NAMED_EXTERNREF, WasmDescribe, inform},
};

pub trait WasmType {
    const INPUT_LEN: u32;
    const OUTPUT_LEN: u32 = Self::INPUT_LEN;
    const ELEMENT_LEN: u32 = Self::OUTPUT_LEN;
    fn describe_name<const INPUT: bool>();
    fn describe_element() {
        Self::describe_name::<false>();
    }
    const ARRAY_INPUT_LEN: u32 = 7 + Self::INPUT_LEN;
    const ARRAY_OUTPUT_LEN: u32 = 7 + Self::ELEMENT_LEN;
    fn describe_array<const INPUT: bool>() {
        crate::__describe!("Array<");
        if INPUT {
            Self::describe_name::<true>();
        } else {
            Self::describe_element();
        }
        crate::__describe!(">");
    }
}

/// A real JS reference with a type-directed TypeScript descriptor.
pub struct TypedJs<T, const INPUT: bool>(pub JsValue, PhantomData<T>);
impl<T, const INPUT: bool> From<JsValue> for TypedJs<T, INPUT> {
    fn from(value: JsValue) -> Self {
        Self(value, PhantomData)
    }
}
impl<T, const INPUT: bool> From<TypedJs<T, INPUT>> for JsValue {
    fn from(value: TypedJs<T, INPUT>) -> Self {
        value.0
    }
}
impl<T: WasmType, const INPUT: bool> WasmDescribe for TypedJs<T, INPUT> {
    fn describe() {
        inform(NAMED_EXTERNREF);
        inform(if INPUT { T::INPUT_LEN } else { T::OUTPUT_LEN });
        T::describe_name::<INPUT>();
    }
}
impl<T: WasmType, const INPUT: bool> IntoWasmAbi for TypedJs<T, INPUT> {
    type Abi = <JsValue as IntoWasmAbi>::Abi;
    fn into_abi(self) -> Self::Abi {
        self.0.into_abi()
    }
}
impl<T: WasmType, const INPUT: bool> FromWasmAbi for TypedJs<T, INPUT> {
    type Abi = <JsValue as FromWasmAbi>::Abi;
    unsafe fn from_abi(value: Self::Abi) -> Self {
        Self::from(unsafe { JsValue::from_abi(value) })
    }
}

impl<T: WasmType, const INPUT: bool> OptionFromWasmAbi for TypedJs<T, INPUT> {
    fn is_none(value: &Self::Abi) -> bool {
        <JsValue as OptionFromWasmAbi>::is_none(value)
    }
}

pub trait FromWasm: Sized {
    type Input: FromWasmAbi + WasmDescribe;
    fn from_wasm(value: Self::Input) -> Result<Self, JsValue>;
    fn from_js(value: JsValue) -> Result<Self, JsValue>;
}
pub trait IntoWasm: Sized {
    type Output: IntoWasmAbi + WasmDescribe + Into<JsValue>;
    fn into_wasm(self) -> Result<Self::Output, JsValue>;
    fn into_js(self) -> Result<JsValue, JsValue> {
        self.into_wasm().map(Into::into)
    }
    fn into_element(self) -> Result<JsValue, JsValue> {
        self.into_js()
    }
    fn array_to_js(values: Vec<Self>) -> Result<JsValue, JsValue> {
        let array = js_sys::Array::new();
        for value in values {
            array.push(&value.into_element()?);
        }
        Ok(array.into())
    }
}
macro_rules! scalar {
    ($ty:ty, $name:literal $(, $array:ident)?) => {
        impl WasmType for $ty {
            const INPUT_LEN: u32 = $name.len() as u32;
            fn describe_name<const INPUT: bool>() { crate::__describe!($name); }
            $(const ARRAY_INPUT_LEN: u32 = stringify!($array).len() as u32;
              const ARRAY_OUTPUT_LEN: u32 = Self::ARRAY_INPUT_LEN;
              fn describe_array<const INPUT: bool>() { crate::__describe!($array); })?
        }
        impl FromWasm for $ty {
            type Input = Self;
            fn from_wasm(value: Self) -> Result<Self, JsValue> { Ok(value) }
            fn from_js(value: JsValue) -> Result<Self, JsValue> { crate::from_js(value) }
        }
        impl IntoWasm for $ty {
            type Output = Self;
            fn into_wasm(self) -> Result<Self, JsValue> { Ok(self) }
            $(fn array_to_js(values: Vec<Self>) -> Result<JsValue, JsValue> {
                let array = js_sys::$array::new_with_length(values.len() as u32);
                for (index, value) in values.into_iter().enumerate() { array.set_index(index as u32, value as _); }
                Ok(array.into())
            })?
        }
    }
}
scalar!(u8, "number", Uint8Array);
scalar!(i8, "number", Int8Array);
scalar!(u16, "number", Uint16Array);
scalar!(i16, "number", Int16Array);
scalar!(u32, "number", Uint32Array);
scalar!(i32, "number", Int32Array);
scalar!(usize, "number", Uint32Array);
scalar!(isize, "number", Int32Array);
scalar!(u64, "bigint", BigUint64Array);
scalar!(i64, "bigint", BigInt64Array);
scalar!(f32, "number", Float32Array);
scalar!(f64, "number", Float64Array);
scalar!(bool, "boolean");
scalar!(String, "string");
impl WasmType for () {
    const INPUT_LEN: u32 = 4;
    fn describe_name<const INPUT: bool>() {
        crate::__describe!("void");
    }
}
impl IntoWasm for () {
    type Output = TypedJs<Self, false>;
    fn into_wasm(self) -> Result<Self::Output, JsValue> {
        Ok(JsValue::UNDEFINED.into())
    }
}

impl<A: WasmType, B: WasmType> WasmType for (A, B) {
    const INPUT_LEN: u32 = A::INPUT_LEN + B::INPUT_LEN + 4;
    const OUTPUT_LEN: u32 = A::ELEMENT_LEN + B::ELEMENT_LEN + 4;
    fn describe_name<const INPUT: bool>() {
        crate::__describe!("[");
        if INPUT {
            A::describe_name::<true>();
        } else {
            A::describe_element();
        }
        crate::__describe!(", ");
        if INPUT {
            B::describe_name::<true>();
        } else {
            B::describe_element();
        }
        crate::__describe!("]");
    }
}
impl<A: FromWasm + WasmType, B: FromWasm + WasmType> FromWasm for (A, B) {
    type Input = TypedJs<Self, true>;
    fn from_wasm(value: Self::Input) -> Result<Self, JsValue> {
        Self::from_js(value.0)
    }
    fn from_js(value: JsValue) -> Result<Self, JsValue> {
        if !js_sys::Array::is_array(&value) {
            return Err(JsValue::from_str("expected a pair"));
        }
        let pair = js_sys::Array::from(&value);
        if pair.length() != 2 {
            return Err(JsValue::from_str("expected a pair"));
        }
        Ok((A::from_js(pair.get(0))?, B::from_js(pair.get(1))?))
    }
}
impl<A: IntoWasm + WasmType, B: IntoWasm + WasmType> IntoWasm for (A, B) {
    type Output = TypedJs<Self, false>;
    fn into_wasm(self) -> Result<Self::Output, JsValue> {
        let pair = js_sys::Array::new();
        pair.push(&self.0.into_element()?);
        pair.push(&self.1.into_element()?);
        Ok(JsValue::from(pair).into())
    }
}

impl<T: WasmType> WasmType for Vec<T> {
    const INPUT_LEN: u32 = T::ARRAY_INPUT_LEN;
    const OUTPUT_LEN: u32 = T::ARRAY_OUTPUT_LEN;
    fn describe_name<const INPUT: bool>() {
        T::describe_array::<INPUT>();
    }
}
impl<T: FromWasm + WasmType> FromWasm for Vec<T> {
    type Input = TypedJs<Self, true>;
    fn from_wasm(value: Self::Input) -> Result<Self, JsValue> {
        Self::from_js(value.0)
    }
    fn from_js(value: JsValue) -> Result<Self, JsValue> {
        if !js_sys::Array::is_array(&value) && !js_sys::ArrayBuffer::is_view(&value) {
            return Err(JsValue::from_str("expected an array or typed array"));
        }
        js_sys::Array::from(&value).iter().map(T::from_js).collect()
    }
}
impl<T: IntoWasm + WasmType> IntoWasm for Vec<T> {
    type Output = TypedJs<Self, false>;
    fn into_wasm(self) -> Result<Self::Output, JsValue> {
        T::array_to_js(self).map(Into::into)
    }
}
impl<T: WasmType> WasmType for Option<T> {
    const INPUT_LEN: u32 = 1 + T::INPUT_LEN + " | null | undefined)".len() as u32;
    const OUTPUT_LEN: u32 = 1 + T::OUTPUT_LEN + " | undefined)".len() as u32;
    const ELEMENT_LEN: u32 = 1 + T::ELEMENT_LEN + " | null)".len() as u32;
    fn describe_name<const INPUT: bool>() {
        crate::__describe!("(");
        T::describe_name::<INPUT>();
        if INPUT {
            crate::__describe!(" | null | undefined)");
        } else {
            crate::__describe!(" | undefined)");
        }
    }
    fn describe_element() {
        crate::__describe!("(");
        T::describe_element();
        crate::__describe!(" | null)");
    }
}
impl<T: FromWasm + WasmType> FromWasm for Option<T> {
    type Input = Option<TypedJs<T, true>>;
    fn from_wasm(value: Self::Input) -> Result<Self, JsValue> {
        Self::from_js(value.map_or(JsValue::UNDEFINED, |value| value.0))
    }
    fn from_js(value: JsValue) -> Result<Self, JsValue> {
        if value.is_null() || value.is_undefined() {
            Ok(None)
        } else {
            T::from_js(value).map(Some)
        }
    }
}
impl<T: IntoWasm + WasmType> IntoWasm for Option<T> {
    type Output = TypedJs<Self, false>;
    fn into_wasm(self) -> Result<Self::Output, JsValue> {
        self.map_or(Ok(JsValue::UNDEFINED), T::into_js)
            .map(Into::into)
    }
    fn into_element(self) -> Result<JsValue, JsValue> {
        self.map_or(Ok(JsValue::NULL), T::into_element)
    }
}
/// Errors cross to JavaScript as `Error` objects. Every error carries its
/// message; typed errors also carry a `detail` property shaped like the value
/// Swift receives.
pub trait WasmError {
    fn into_js_error(self) -> JsValue;
}

#[doc(hidden)]
pub fn js_error(message: &str, detail: Option<JsValue>) -> JsValue {
    let error = js_sys::Error::new(message);
    if let Some(detail) = detail {
        let _ = js_sys::Reflect::set(&error, &JsValue::from_str("detail"), &detail);
    }
    error.into()
}

impl WasmError for crate::Error {
    fn into_js_error(self) -> JsValue {
        js_error(&self.to_string(), None)
    }
}
impl WasmError for String {
    fn into_js_error(self) -> JsValue {
        js_error(&self, None)
    }
}
impl WasmError for &str {
    fn into_js_error(self) -> JsValue {
        js_error(self, None)
    }
}
impl WasmError for abort_signal::Aborted {
    fn into_js_error(self) -> JsValue {
        js_error(&self.to_string(), None)
    }
}
/// A raw JavaScript error, for example from a browser API, is rethrown as is.
impl WasmError for JsValue {
    fn into_js_error(self) -> JsValue {
        self
    }
}
impl WasmError for std::io::Error {
    fn into_js_error(self) -> JsValue {
        let message = self.to_string();
        let detail = crate::io_error::IoError::from(self).into_js().ok();
        js_error(&message, detail)
    }
}
impl<T: IntoWasm, E: WasmError> IntoWasm for Result<T, E> {
    type Output = T::Output;
    fn into_wasm(self) -> Result<Self::Output, JsValue> {
        self.map_err(WasmError::into_js_error)?.into_wasm()
    }
}

impl WasmType for JsValue {
    const INPUT_LEN: u32 = 3;
    fn describe_name<const INPUT: bool>() {
        crate::__describe!("any");
    }
}
impl FromWasm for JsValue {
    type Input = Self;
    fn from_wasm(value: Self) -> Result<Self, JsValue> {
        Ok(value)
    }
    fn from_js(value: Self) -> Result<Self, JsValue> {
        Ok(value)
    }
}
impl IntoWasm for JsValue {
    type Output = Self;
    fn into_wasm(self) -> Result<Self, JsValue> {
        Ok(self)
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __wasm_object {
    ($ty:ident, $name:literal, $name_len:literal) => {
        impl $crate::WasmType for $ty {
            const INPUT_LEN: u32 = $name_len;
            fn describe_name<const INPUT: bool>() {
                $crate::__describe!($name);
            }
        }
        impl $crate::FromWasm for $ty {
            type Input = Self;
            fn from_wasm(value: Self) -> Result<Self, $crate::__wasm_bindgen::JsValue> {
                Ok(value)
            }
            fn from_js(
                value: $crate::__wasm_bindgen::JsValue,
            ) -> Result<Self, $crate::__wasm_bindgen::JsValue> {
                <Self as $crate::__wasm_bindgen::convert::TryFromJsValue>::try_from_js_value(value)
            }
        }
        impl $crate::IntoWasm for $ty {
            type Output = Self;
            fn into_wasm(self) -> Result<Self, $crate::__wasm_bindgen::JsValue> {
                Ok(self)
            }
        }
    };
}

impl<T: crate::JsArguments + 'static> FromWasm for crate::Callback<T> {
    type Input = Self;
    fn from_wasm(value: Self) -> Result<Self, JsValue> {
        Ok(value)
    }
    fn from_js(value: JsValue) -> Result<Self, JsValue> {
        use wasm_bindgen::JsCast;
        value.dyn_into::<js_sys::Function>().map(Self::from_js)
    }
}
impl<T> WasmType for crate::Callback<T> {
    const INPUT_LEN: u32 = 8;
    fn describe_name<const INPUT: bool>() {
        crate::__describe!("Function");
    }
}
impl FromWasm for crate::AbortSignal {
    type Input = Self;
    fn from_wasm(value: Self) -> Result<Self, JsValue> {
        Ok(value)
    }
    fn from_js(value: JsValue) -> Result<Self, JsValue> {
        use wasm_bindgen::JsCast;
        value.dyn_into::<web_sys::AbortSignal>().map(Into::into)
    }
}
impl WasmType for crate::AbortSignal {
    const INPUT_LEN: u32 = 11;
    fn describe_name<const INPUT: bool>() {
        crate::__describe!("AbortSignal");
    }
}
impl IntoWasm for js_sys::Uint8Array {
    type Output = Self;
    fn into_wasm(self) -> Result<Self, JsValue> {
        Ok(self)
    }
}

/// The representation inside a Serde value differs from the direct function ABI:
/// e.g. Vec<u32> is an ordinary JS array, and u64 is a number unless configured.
#[doc(hidden)]
pub trait SerdeType<const BIGINT: bool, const NULL: bool, const OBJECTS: bool> {
    const LEN: u32;
    fn describe();
}
macro_rules! serde_scalar {
    ($($ty:ty),*) => {$(
        impl<const B: bool, const N: bool, const O: bool> SerdeType<B, N, O> for $ty {
            const LEN: u32 = <$ty as WasmType>::INPUT_LEN;
            fn describe() { <$ty as WasmType>::describe_name::<true>(); }
        }
    )*};
}
serde_scalar!(
    u8, i8, u16, i16, u32, i32, usize, isize, f32, f64, bool, String
);
macro_rules! serde_integer {
    ($($ty:ty),*) => {$(
        impl<const B: bool, const N: bool, const O: bool> SerdeType<B, N, O> for $ty {
            const LEN: u32 = 6;
            fn describe() { if B { crate::__describe!("bigint"); } else { crate::__describe!("number"); } }
        }
    )*};
}
serde_integer!(u64, i64, u128, i128);
impl<T: SerdeType<B, N, O>, const B: bool, const N: bool, const O: bool> SerdeType<B, N, O>
    for Vec<T>
{
    const LEN: u32 = 7 + T::LEN;
    fn describe() {
        crate::__describe!("Array<");
        T::describe();
        crate::__describe!(">");
    }
}
impl<T: SerdeType<B, N, O>, const B: bool, const N: bool, const O: bool> SerdeType<B, N, O>
    for Option<T>
{
    const LEN: u32 = T::LEN + if N { 9 } else { 14 };
    fn describe() {
        crate::__describe!("(");
        T::describe();
        if N {
            crate::__describe!(" | null)");
        } else {
            crate::__describe!(" | undefined)");
        }
    }
}
impl<T: SerdeType<B, N, O>, const B: bool, const N: bool, const O: bool> SerdeType<B, N, O>
    for Box<T>
{
    const LEN: u32 = T::LEN;
    fn describe() {
        T::describe();
    }
}
impl<A: SerdeType<B, N, O>, Z: SerdeType<B, N, O>, const B: bool, const N: bool, const O: bool>
    SerdeType<B, N, O> for (A, Z)
{
    const LEN: u32 = A::LEN + Z::LEN + 4;
    fn describe() {
        crate::__describe!("[");
        A::describe();
        crate::__describe!(", ");
        Z::describe();
        crate::__describe!("]");
    }
}

impl<const B: bool, const N: bool, const O: bool> SerdeType<B, N, O> for std::num::NonZeroU32 {
    const LEN: u32 = <u32 as SerdeType<B, N, O>>::LEN;
    fn describe() {
        <u32 as SerdeType<B, N, O>>::describe();
    }
}

/// Only transparent data can participate in content-addressed return interning.
#[diagnostic::on_unimplemented(
    message = "stable returns must be transparent bridge values, not opaque handles"
)]
pub trait StableValue: serde::Serialize + IntoWasm {}

#[diagnostic::on_unimplemented(
    message = "stable returns must be transparent bridge values, not opaque handles"
)]
pub trait StableReturn: IntoWasm {
    fn into_stable_wasm(self, cache: &StableCache) -> Result<Self::Output, JsValue>;
}
impl<T: StableValue> StableReturn for T
where
    T::Output: StableOutput,
{
    fn into_stable_wasm(self, cache: &StableCache) -> Result<Self::Output, JsValue> {
        let hash =
            crate::stable::hash(&self).map_err(|error| JsValue::from_str(&error.to_string()))?;
        T::Output::stable_output(cache, hash, || self.into_wasm())
    }
}
impl<T: StableReturn, E: WasmError> StableReturn for Result<T, E> {
    fn into_stable_wasm(self, cache: &StableCache) -> Result<Self::Output, JsValue> {
        self.map_err(WasmError::into_js_error)?
            .into_stable_wasm(cache)
    }
}

pub trait StableOutput: Sized {
    fn stable_output(
        cache: &StableCache,
        hash: u128,
        build: impl FnOnce() -> Result<Self, JsValue>,
    ) -> Result<Self, JsValue>;
}
impl<T> StableOutput for TypedJs<T, false> {
    fn stable_output(
        cache: &StableCache,
        hash: u128,
        build: impl FnOnce() -> Result<Self, JsValue>,
    ) -> Result<Self, JsValue> {
        cache
            .get_or_insert(hash, || build().map(Into::into))
            .map(Into::into)
    }
}
macro_rules! stable_scalar {
    ($($ty:ty),*) => {$ (
        impl StableValue for $ty {}
        impl StableOutput for $ty {
            fn stable_output(_: &StableCache, _: u128, build: impl FnOnce() -> Result<Self, JsValue>) -> Result<Self, JsValue> { build() }
        }
    )*};
}
stable_scalar!(
    u8, i8, u16, i16, u32, i32, u64, i64, usize, isize, f32, f64, bool, String
);
impl StableValue for () {}
impl<T: StableValue + WasmType> StableValue for Vec<T> {}
impl<T: StableValue + WasmType> StableValue for Option<T> {}
impl<A: StableValue + WasmType, B: StableValue + WasmType> StableValue for (A, B) {}

use std::{cell::RefCell, collections::HashMap, rc::Rc};
use wasm_bindgen::{JsCast, closure::Closure};
type WeakValues = Rc<RefCell<HashMap<u128, js_sys::WeakRef>>>;

pub enum StableCache {
    Weak {
        values: WeakValues,
        registry: js_sys::FinalizationRegistry,
    },
    Strong(RefCell<HashMap<u128, JsValue>>),
}
impl StableCache {
    pub fn new(strong: bool) -> Self {
        if strong {
            return Self::Strong(RefCell::new(HashMap::new()));
        }
        let values: WeakValues = Rc::default();
        let weak = Rc::downgrade(&values);
        let cleanup = Closure::<dyn FnMut(JsValue)>::new(move |key: JsValue| {
            let hash = u128::from_str_radix(&key.as_string().unwrap(), 16).unwrap();
            if let Some(values) = weak.upgrade() {
                let mut values = values.borrow_mut();
                // An old finalizer must not remove a live replacement for this hash.
                if values
                    .get(&hash)
                    .is_some_and(|value| value.deref().is_none())
                {
                    values.remove(&hash);
                }
            }
        });
        let registry = js_sys::FinalizationRegistry::new(cleanup.as_ref().unchecked_ref());
        // Each generated function owns one process-lifetime thread-local cache.
        cleanup.forget();
        Self::Weak { values, registry }
    }
    fn get_or_insert(
        &self,
        hash: u128,
        build: impl FnOnce() -> Result<JsValue, JsValue>,
    ) -> Result<JsValue, JsValue> {
        let cached = match self {
            Self::Strong(values) => values.borrow().get(&hash).cloned(),
            Self::Weak { values, .. } => values
                .borrow()
                .get(&hash)
                .and_then(js_sys::WeakRef::deref)
                .map(Into::into),
        };
        if let Some(value) = cached {
            return Ok(value);
        }
        let value = build()?;
        if value.is_object() {
            #[cfg(debug_assertions)]
            freeze_data(&value);
            match self {
                Self::Strong(values) => {
                    values.borrow_mut().insert(hash, value.clone());
                }
                Self::Weak {
                    values, registry, ..
                } => {
                    values.borrow_mut().insert(
                        hash,
                        js_sys::WeakRef::new(value.unchecked_ref::<js_sys::Object>()),
                    );
                    registry.register(&value, &JsValue::from_str(&format!("{hash:x}")));
                }
            }
        }
        Ok(value)
    }
}

// Freeze only ordinary records and arrays: typed arrays cannot be frozen, and
// Map/Set internal slots remain mutable even after Object.freeze.
#[cfg(debug_assertions)]
fn freeze_data(value: &JsValue) {
    if !value.is_object() {
        return;
    }
    let object = value.unchecked_ref::<js_sys::Object>();
    let prototype = js_sys::Object::get_prototype_of(object);
    let plain_prototype = js_sys::Object::get_prototype_of(&js_sys::Object::new());
    if !js_sys::Array::is_array(value) && !prototype.is_null() && prototype != plain_prototype {
        return;
    }
    for child in js_sys::Object::values(object).iter() {
        freeze_data(&child);
    }
    js_sys::Object::freeze(object);
}
