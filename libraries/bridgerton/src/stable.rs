//! A framed serde fingerprint, preserving values that JSON would erase or reject.
use serde::{Serialize, Serializer, ser};
use std::fmt::{self, Write};
use xxhash_rust::xxh3::Xxh3;

pub(crate) fn hash(value: &(impl Serialize + ?Sized)) -> Result<u128, Error> {
    let mut serializer = HashSerializer(Xxh3::new());
    value.serialize(&mut serializer)?;
    Ok(serializer.0.digest128())
}

#[derive(Debug)]
pub(crate) struct Error(String);
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}
impl ser::Error for Error {
    fn custom<T: fmt::Display>(message: T) -> Self {
        Self(message.to_string())
    }
}

#[repr(u8)]
enum Tag {
    End,
    Bool,
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Char,
    Str,
    Bytes,
    None,
    Some,
    Unit,
    UnitStruct,
    UnitVariant,
    NewtypeStruct,
    NewtypeVariant,
    Seq,
    Tuple,
    TupleStruct,
    TupleVariant,
    Map,
    Struct,
    StructVariant,
}
struct HashSerializer(Xxh3);
impl HashSerializer {
    fn tag(&mut self, tag: Tag) {
        self.0.update(&[tag as u8]);
    }
    fn begin(&mut self, tag: Tag, len: Option<usize>) -> &mut Self {
        self.tag(tag);
        self.0.update(&[u8::from(len.is_some())]);
        if let Some(len) = len {
            self.0.update(&(len as u64).to_le_bytes());
        }
        self
    }
    // Untagged enums serialize their inner variant directly, so the index alone
    // cannot tell two inner enums apart; the name can.
    fn variant(&mut self, tag: Tag, index: u32, name: &str) {
        self.tag(tag);
        self.0.update(&index.to_le_bytes());
        self.0.update(&(name.len() as u64).to_le_bytes());
        self.0.update(name.as_bytes());
    }
    fn blob(&mut self, tag: Tag, bytes: &[u8]) {
        self.tag(tag);
        self.0.update(&(bytes.len() as u64).to_le_bytes());
        self.0.update(bytes);
    }
}

macro_rules! number {
    ($method:ident, $ty:ty, $tag:ident) => {
        fn $method(self, value: $ty) -> Result<(), Error> {
            self.tag(Tag::$tag);
            self.0.update(&value.to_le_bytes());
            Ok(())
        }
    };
}
impl Serializer for &mut HashSerializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    number!(serialize_i8, i8, I8);
    number!(serialize_i16, i16, I16);
    number!(serialize_i32, i32, I32);
    number!(serialize_i64, i64, I64);
    number!(serialize_i128, i128, I128);
    number!(serialize_u8, u8, U8);
    number!(serialize_u16, u16, U16);
    number!(serialize_u32, u32, U32);
    number!(serialize_u64, u64, U64);
    number!(serialize_u128, u128, U128);

    fn serialize_bool(self, value: bool) -> Result<(), Error> {
        self.tag(Tag::Bool);
        self.0.update(&[u8::from(value)]);
        Ok(())
    }
    fn serialize_f32(self, value: f32) -> Result<(), Error> {
        self.tag(Tag::F32);
        self.0.update(&value.to_bits().to_le_bytes());
        Ok(())
    }
    fn serialize_f64(self, value: f64) -> Result<(), Error> {
        self.tag(Tag::F64);
        self.0.update(&value.to_bits().to_le_bytes());
        Ok(())
    }
    fn serialize_char(self, value: char) -> Result<(), Error> {
        self.tag(Tag::Char);
        self.0.update(&(value as u32).to_le_bytes());
        Ok(())
    }
    fn serialize_str(self, value: &str) -> Result<(), Error> {
        self.blob(Tag::Str, value.as_bytes());
        Ok(())
    }
    fn serialize_bytes(self, value: &[u8]) -> Result<(), Error> {
        self.blob(Tag::Bytes, value);
        Ok(())
    }
    fn serialize_none(self) -> Result<(), Error> {
        self.tag(Tag::None);
        Ok(())
    }
    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<(), Error> {
        self.tag(Tag::Some);
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), Error> {
        self.tag(Tag::Unit);
        Ok(())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<(), Error> {
        self.tag(Tag::UnitStruct);
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        index: u32,
        name: &'static str,
    ) -> Result<(), Error> {
        self.variant(Tag::UnitVariant, index, name);
        Ok(())
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        self.tag(Tag::NewtypeStruct);
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        index: u32,
        name: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        self.variant(Tag::NewtypeVariant, index, name);
        value.serialize(self)
    }
    fn serialize_seq(self, len: Option<usize>) -> Result<Self, Error> {
        Ok(self.begin(Tag::Seq, len))
    }
    fn serialize_tuple(self, len: usize) -> Result<Self, Error> {
        Ok(self.begin(Tag::Tuple, Some(len)))
    }
    fn serialize_tuple_struct(self, _: &'static str, len: usize) -> Result<Self, Error> {
        Ok(self.begin(Tag::TupleStruct, Some(len)))
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        index: u32,
        name: &'static str,
        len: usize,
    ) -> Result<Self, Error> {
        self.variant(Tag::TupleVariant, index, name);
        Ok(self.begin(Tag::Tuple, Some(len)))
    }
    fn serialize_map(self, len: Option<usize>) -> Result<Self, Error> {
        Ok(self.begin(Tag::Map, len))
    }
    fn serialize_struct(self, _: &'static str, len: usize) -> Result<Self, Error> {
        Ok(self.begin(Tag::Struct, Some(len)))
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        index: u32,
        name: &'static str,
        len: usize,
    ) -> Result<Self, Error> {
        self.variant(Tag::StructVariant, index, name);
        Ok(self.begin(Tag::Struct, Some(len)))
    }
    fn collect_str<T: fmt::Display + ?Sized>(self, value: &T) -> Result<(), Error> {
        // Count before hashing so Display's chunking neither allocates nor changes the hash.
        struct Output<'a> {
            len: usize,
            hash: Option<&'a mut Xxh3>,
        }
        impl fmt::Write for Output<'_> {
            fn write_str(&mut self, text: &str) -> fmt::Result {
                self.len += text.len();
                if let Some(hash) = &mut self.hash {
                    hash.update(text.as_bytes());
                }
                Ok(())
            }
        }
        let mut output = Output { len: 0, hash: None };
        write!(output, "{value}").map_err(ser::Error::custom)?;
        self.tag(Tag::Str);
        self.0.update(&(output.len as u64).to_le_bytes());
        output.hash = Some(&mut self.0);
        write!(output, "{value}").map_err(ser::Error::custom)
    }
}

macro_rules! compound {
    ($trait:ident, $($method:ident $(($key:ident))?),+) => {
        impl ser::$trait for &mut HashSerializer {
            type Ok = ();
            type Error = Error;
            $(fn $method<T: Serialize + ?Sized>(&mut self, $($key: &'static str,)? value: &T) -> Result<(), Error> {
                // Field names distinguish conditionally skipped fields too.
                $(self.serialize_str($key)?;)?
                value.serialize(&mut **self)
            })+
            fn end(self) -> Result<(), Error> {
                self.tag(Tag::End);
                Ok(())
            }
        }
    };
}
compound!(SerializeSeq, serialize_element);
compound!(SerializeTuple, serialize_element);
compound!(SerializeTupleStruct, serialize_field);
compound!(SerializeTupleVariant, serialize_field);
compound!(SerializeMap, serialize_key, serialize_value);
compound!(SerializeStruct, serialize_field(key));
compound!(SerializeStructVariant, serialize_field(key));

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn floats_preserve_every_bit() {
        let values = [
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
            f64::from_bits(0x7ff8_0000_0000_0001),
            0.0,
            -0.0,
        ];
        for (i, value) in values.iter().enumerate() {
            assert_eq!(hash(&[*value]).unwrap(), hash(&[*value]).unwrap());
            for other in &values[i + 1..] {
                assert_ne!(hash(&[*value]).unwrap(), hash(&[*other]).unwrap());
            }
        }
        assert_ne!(hash(&0.0f32).unwrap(), hash(&-0.0f32).unwrap());
    }

    #[test]
    fn structured_map_keys_are_not_json_keys() {
        #[derive(Serialize, Eq, PartialEq, Ord, PartialOrd)]
        struct Key {
            id: u8,
            active: bool,
        }
        let original = BTreeMap::from([(
            Key {
                id: 1,
                active: true,
            },
            String::from("value"),
        )]);
        let equal = BTreeMap::from([(
            Key {
                id: 1,
                active: true,
            },
            String::from("value"),
        )]);
        let changed = BTreeMap::from([(
            Key {
                id: 2,
                active: true,
            },
            String::from("value"),
        )]);
        assert_eq!(hash(&original).unwrap(), hash(&equal).unwrap());
        assert_ne!(hash(&original).unwrap(), hash(&changed).unwrap());
    }

    #[test]
    fn framing_preserves_types_and_nesting() {
        assert_ne!(
            hash(&Some(None::<u8>)).unwrap(),
            hash(&None::<Option<u8>>).unwrap()
        );
        assert_ne!(
            hash(&vec![vec![1u8], vec![2]]).unwrap(),
            hash(&vec![vec![1u8, 2]]).unwrap()
        );
        struct Bytes;
        impl Serialize for Bytes {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_bytes(b"ab")
            }
        }
        assert_ne!(hash("ab").unwrap(), hash(&Bytes).unwrap());
        assert_ne!(hash(&1u8).unwrap(), hash(&1u64).unwrap());
        assert_ne!(hash(&i128::MAX).unwrap(), hash(&u128::MAX).unwrap());
    }

    #[test]
    fn untagged_enums_distinguish_inner_variants_by_name() {
        #[derive(Serialize)]
        enum First {
            Alpha,
        }
        #[derive(Serialize)]
        enum Second {
            Beta,
        }
        #[derive(Serialize)]
        #[serde(untagged)]
        enum Either {
            First(First),
            Second(Second),
        }
        assert_ne!(
            hash(&Either::First(First::Alpha)).unwrap(),
            hash(&Either::Second(Second::Beta)).unwrap()
        );
    }

    #[test]
    fn display_streams_like_a_string() {
        struct Text;
        impl fmt::Display for Text {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("語")?;
                f.write_str("\0ab")
            }
        }
        impl Serialize for Text {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.collect_str(self)
            }
        }
        assert_eq!(hash(&Text).unwrap(), hash("語\0ab").unwrap());
    }
}
