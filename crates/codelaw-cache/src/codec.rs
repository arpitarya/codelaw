//! The cache's own binary body format (SR-CACHE decision 4; L11: no crate).
//!
//! Not self-describing, like the format it replaced: fields are written in
//! declaration order and read back by the same type. Integers and floats are
//! fixed-width little-endian; a `usize` is a `u64`; a `char` is a `u32`;
//! a `bool` and an `Option` tag are one byte; a string, byte string or
//! sequence is a `u64` length then its items; a map is a `u64` count then
//! key, value pairs; an enum is a `u32` variant index then its fields.
//!
//! The output is deterministic (fixed field order, no hashing). Decoding is
//! total: any malformed, truncated or over-long input is an `Err`, never a
//! panic and never an allocation larger than the input.

use serde::de::{self, DeserializeOwned, DeserializeSeed, IntoDeserializer, Visitor};
use serde::{Serialize, ser};
use std::fmt;

/// A failed encode or decode. The cache treats either as a miss.
#[derive(Debug)]
pub struct Error(String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}
impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error(msg.to_string())
    }
}
impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error(msg.to_string())
    }
}

type Res<T> = Result<T, Error>;

fn err<T>(msg: &str) -> Res<T> {
    Err(Error(msg.to_string()))
}

/// Encode `value`.
pub fn to_bytes<T: Serialize + ?Sized>(value: &T) -> Res<Vec<u8>> {
    let mut out = Vec::new();
    value.serialize(&mut Enc(&mut out))?;
    Ok(out)
}

/// Decode a `T` that spans all of `bytes`; trailing bytes are an error.
pub fn from_bytes<T: DeserializeOwned>(bytes: &[u8]) -> Res<T> {
    let mut dec = Dec(bytes);
    let value = T::deserialize(&mut dec)?;
    if dec.0.is_empty() { Ok(value) } else { err("trailing bytes") }
}

struct Enc<'a>(&'a mut Vec<u8>);

impl Enc<'_> {
    fn len(&mut self, n: usize) {
        self.0.extend_from_slice(&(n as u64).to_le_bytes());
    }
}

macro_rules! enc_num {
    ($($name:ident $ty:ty),*) => {$(
        fn $name(self, v: $ty) -> Res<()> {
            self.0.extend_from_slice(&v.to_le_bytes());
            Ok(())
        }
    )*};
}

impl<'a, 'b> ser::Serializer for &'a mut Enc<'b> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    enc_num!(serialize_i8 i8, serialize_i16 i16, serialize_i32 i32, serialize_i64 i64,
        serialize_u8 u8, serialize_u16 u16, serialize_u32 u32, serialize_u64 u64,
        serialize_f32 f32, serialize_f64 f64);

    fn serialize_bool(self, v: bool) -> Res<()> {
        self.0.push(u8::from(v));
        Ok(())
    }
    fn serialize_char(self, v: char) -> Res<()> {
        self.serialize_u32(v as u32)
    }
    fn serialize_str(self, v: &str) -> Res<()> {
        self.serialize_bytes(v.as_bytes())
    }
    fn serialize_bytes(self, v: &[u8]) -> Res<()> {
        self.len(v.len());
        self.0.extend_from_slice(v);
        Ok(())
    }
    fn serialize_none(self) -> Res<()> {
        self.0.push(0);
        Ok(())
    }
    fn serialize_some<T: Serialize + ?Sized>(self, v: &T) -> Res<()> {
        self.0.push(1);
        v.serialize(self)
    }
    fn serialize_unit(self) -> Res<()> {
        Ok(())
    }
    fn serialize_unit_struct(self, _: &'static str) -> Res<()> {
        Ok(())
    }
    fn serialize_unit_variant(self, _: &'static str, idx: u32, _: &'static str) -> Res<()> {
        self.serialize_u32(idx)
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(self, _: &'static str, v: &T) -> Res<()> {
        v.serialize(self)
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(self, _: &'static str, idx: u32, _: &'static str, v: &T) -> Res<()> {
        self.serialize_u32(idx)?;
        v.serialize(self)
    }
    fn serialize_seq(self, len: Option<usize>) -> Res<Self> {
        let Some(n) = len else { return err("sequence of unknown length") };
        self.len(n);
        Ok(self)
    }
    fn serialize_tuple(self, _: usize) -> Res<Self> {
        Ok(self)
    }
    fn serialize_tuple_struct(self, _: &'static str, _: usize) -> Res<Self> {
        Ok(self)
    }
    fn serialize_tuple_variant(self, _: &'static str, idx: u32, _: &'static str, _: usize) -> Res<Self> {
        self.serialize_u32(idx)?;
        Ok(self)
    }
    fn serialize_map(self, len: Option<usize>) -> Res<Self> {
        let Some(n) = len else { return err("map of unknown length") };
        self.len(n);
        Ok(self)
    }
    fn serialize_struct(self, _: &'static str, _: usize) -> Res<Self> {
        Ok(self)
    }
    fn serialize_struct_variant(self, _: &'static str, idx: u32, _: &'static str, _: usize) -> Res<Self> {
        self.serialize_u32(idx)?;
        Ok(self)
    }
    fn is_human_readable(&self) -> bool {
        false
    }
}

macro_rules! enc_compound {
    ($($trait:ident $method:ident),*) => {$(
        impl ser::$trait for &mut Enc<'_> {
            type Ok = ();
            type Error = Error;
            fn $method<T: Serialize + ?Sized>(&mut self, v: &T) -> Res<()> {
                v.serialize(&mut **self)
            }
            fn end(self) -> Res<()> {
                Ok(())
            }
        }
    )*};
}
enc_compound!(SerializeSeq serialize_element, SerializeTuple serialize_element,
    SerializeTupleStruct serialize_field, SerializeTupleVariant serialize_field);

impl ser::SerializeMap for &mut Enc<'_> {
    type Ok = ();
    type Error = Error;
    fn serialize_key<T: Serialize + ?Sized>(&mut self, k: &T) -> Res<()> {
        k.serialize(&mut **self)
    }
    fn serialize_value<T: Serialize + ?Sized>(&mut self, v: &T) -> Res<()> {
        v.serialize(&mut **self)
    }
    fn end(self) -> Res<()> {
        Ok(())
    }
}

impl ser::SerializeStruct for &mut Enc<'_> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, _: &'static str, v: &T) -> Res<()> {
        v.serialize(&mut **self)
    }
    fn end(self) -> Res<()> {
        Ok(())
    }
}

impl ser::SerializeStructVariant for &mut Enc<'_> {
    type Ok = ();
    type Error = Error;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, _: &'static str, v: &T) -> Res<()> {
        v.serialize(&mut **self)
    }
    fn end(self) -> Res<()> {
        Ok(())
    }
}

struct Dec<'de>(&'de [u8]);

impl<'de> Dec<'de> {
    fn take(&mut self, n: usize) -> Res<&'de [u8]> {
        if n > self.0.len() {
            return err("truncated");
        }
        let (head, tail) = self.0.split_at(n);
        self.0 = tail;
        Ok(head)
    }
    fn array<const N: usize>(&mut self) -> Res<[u8; N]> {
        let mut a = [0u8; N];
        a.copy_from_slice(self.take(N)?);
        Ok(a)
    }
    /// A length or count. Every item takes at least one byte, so a count
    /// beyond the bytes left is malformed — and a hostile one cannot make
    /// the decoder loop or allocate past the input.
    fn len(&mut self) -> Res<usize> {
        let n = u64::from_le_bytes(self.array()?);
        if n > self.0.len() as u64 { err("length beyond the input") } else { Ok(n as usize) }
    }
    fn str(&mut self) -> Res<&'de str> {
        let n = self.len()?;
        std::str::from_utf8(self.take(n)?).map_err(|_| Error("invalid utf-8".into()))
    }
}

macro_rules! dec_num {
    ($($name:ident $visit:ident $ty:ty),*) => {$(
        fn $name<V: Visitor<'de>>(self, v: V) -> Res<V::Value> {
            v.$visit(<$ty>::from_le_bytes(self.array()?))
        }
    )*};
}

impl<'de> de::Deserializer<'de> for &mut Dec<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, _: V) -> Res<V::Value> {
        err("the cache format is not self-describing")
    }
    fn deserialize_ignored_any<V: Visitor<'de>>(self, _: V) -> Res<V::Value> {
        err("the cache format is not self-describing")
    }
    dec_num!(deserialize_i8 visit_i8 i8, deserialize_i16 visit_i16 i16, deserialize_i32 visit_i32 i32,
        deserialize_i64 visit_i64 i64, deserialize_u8 visit_u8 u8, deserialize_u16 visit_u16 u16,
        deserialize_u32 visit_u32 u32, deserialize_u64 visit_u64 u64, deserialize_f32 visit_f32 f32,
        deserialize_f64 visit_f64 f64);

    fn deserialize_bool<V: Visitor<'de>>(self, v: V) -> Res<V::Value> {
        match self.take(1)?[0] {
            0 => v.visit_bool(false),
            1 => v.visit_bool(true),
            _ => err("bad bool"),
        }
    }
    fn deserialize_char<V: Visitor<'de>>(self, v: V) -> Res<V::Value> {
        let c = u32::from_le_bytes(self.array()?);
        v.visit_char(char::from_u32(c).ok_or_else(|| Error("bad char".into()))?)
    }
    fn deserialize_str<V: Visitor<'de>>(self, v: V) -> Res<V::Value> {
        v.visit_borrowed_str(self.str()?)
    }
    fn deserialize_string<V: Visitor<'de>>(self, v: V) -> Res<V::Value> {
        self.deserialize_str(v)
    }
    fn deserialize_bytes<V: Visitor<'de>>(self, v: V) -> Res<V::Value> {
        let n = self.len()?;
        v.visit_borrowed_bytes(self.take(n)?)
    }
    fn deserialize_byte_buf<V: Visitor<'de>>(self, v: V) -> Res<V::Value> {
        self.deserialize_bytes(v)
    }
    fn deserialize_option<V: Visitor<'de>>(self, v: V) -> Res<V::Value> {
        match self.take(1)?[0] {
            0 => v.visit_none(),
            1 => v.visit_some(self),
            _ => err("bad option tag"),
        }
    }
    fn deserialize_unit<V: Visitor<'de>>(self, v: V) -> Res<V::Value> {
        v.visit_unit()
    }
    fn deserialize_unit_struct<V: Visitor<'de>>(self, _: &'static str, v: V) -> Res<V::Value> {
        v.visit_unit()
    }
    fn deserialize_newtype_struct<V: Visitor<'de>>(self, _: &'static str, v: V) -> Res<V::Value> {
        v.visit_newtype_struct(self)
    }
    fn deserialize_seq<V: Visitor<'de>>(self, v: V) -> Res<V::Value> {
        let n = self.len()?;
        v.visit_seq(Counted { dec: self, left: n })
    }
    fn deserialize_tuple<V: Visitor<'de>>(self, n: usize, v: V) -> Res<V::Value> {
        v.visit_seq(Counted { dec: self, left: n })
    }
    fn deserialize_tuple_struct<V: Visitor<'de>>(self, _: &'static str, n: usize, v: V) -> Res<V::Value> {
        self.deserialize_tuple(n, v)
    }
    fn deserialize_map<V: Visitor<'de>>(self, v: V) -> Res<V::Value> {
        let n = self.len()?;
        v.visit_map(Counted { dec: self, left: n })
    }
    fn deserialize_struct<V: Visitor<'de>>(self, _: &'static str, fields: &'static [&'static str], v: V) -> Res<V::Value> {
        self.deserialize_tuple(fields.len(), v)
    }
    fn deserialize_enum<V: Visitor<'de>>(self, _: &'static str, _: &'static [&'static str], v: V) -> Res<V::Value> {
        v.visit_enum(self)
    }
    fn deserialize_identifier<V: Visitor<'de>>(self, _: V) -> Res<V::Value> {
        err("the cache format is not self-describing")
    }
    fn is_human_readable(&self) -> bool {
        false
    }
}

struct Counted<'a, 'de> {
    dec: &'a mut Dec<'de>,
    left: usize,
}

impl<'de> de::SeqAccess<'de> for Counted<'_, 'de> {
    type Error = Error;
    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Res<Option<T::Value>> {
        if self.left == 0 {
            return Ok(None);
        }
        self.left -= 1;
        seed.deserialize(&mut *self.dec).map(Some)
    }
    fn size_hint(&self) -> Option<usize> {
        Some(self.left)
    }
}

impl<'de> de::MapAccess<'de> for Counted<'_, 'de> {
    type Error = Error;
    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Res<Option<K::Value>> {
        if self.left == 0 {
            return Ok(None);
        }
        self.left -= 1;
        seed.deserialize(&mut *self.dec).map(Some)
    }
    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Res<V::Value> {
        seed.deserialize(&mut *self.dec)
    }
    fn size_hint(&self) -> Option<usize> {
        Some(self.left)
    }
}

impl<'de> de::EnumAccess<'de> for &mut Dec<'de> {
    type Error = Error;
    type Variant = Self;
    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Res<(V::Value, Self)> {
        let idx = u32::from_le_bytes(self.array()?);
        let v = seed.deserialize(idx.into_deserializer())?;
        Ok((v, self))
    }
}

impl<'de> de::VariantAccess<'de> for &mut Dec<'de> {
    type Error = Error;
    fn unit_variant(self) -> Res<()> {
        Ok(())
    }
    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Res<T::Value> {
        seed.deserialize(self)
    }
    fn tuple_variant<V: Visitor<'de>>(self, n: usize, v: V) -> Res<V::Value> {
        v.visit_seq(Counted { dec: self, left: n })
    }
    fn struct_variant<V: Visitor<'de>>(self, fields: &'static [&'static str], v: V) -> Res<V::Value> {
        v.visit_seq(Counted { dec: self, left: fields.len() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::collections::BTreeMap;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    enum Shape {
        Unit,
        New(String),
        Tup(u8, i64),
        Rec { a: Option<u32>, b: Vec<char> },
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Doc {
        name: String,
        flag: bool,
        n: usize,
        x: f64,
        pair: (u16, i8),
        shapes: Vec<Shape>,
        map: BTreeMap<String, Vec<u8>>,
    }

    fn doc() -> Doc {
        Doc {
            name: "héllo".into(),
            flag: true,
            n: 7,
            x: -1.5,
            pair: (65535, -3),
            shapes: vec![Shape::Unit, Shape::New("s".into()), Shape::Tup(9, i64::MIN), Shape::Rec { a: Some(4), b: vec!['a', '✓'] }, Shape::Rec { a: None, b: vec![] }],
            map: BTreeMap::from([("k".into(), vec![1, 2]), ("z".into(), vec![])]),
        }
    }

    #[test]
    fn round_trips() {
        let bytes = to_bytes(&doc()).unwrap();
        assert_eq!(from_bytes::<Doc>(&bytes).unwrap(), doc());
        assert_eq!(to_bytes(&doc()).unwrap(), bytes, "deterministic");
    }

    #[test]
    fn fixed_layout() {
        assert_eq!(to_bytes(&("ab", 1u16, Some(true))).unwrap(), [2, 0, 0, 0, 0, 0, 0, 0, b'a', b'b', 1, 0, 1, 1]);
    }

    #[test]
    fn every_truncation_and_bit_flip_is_an_error_or_a_value_never_a_panic() {
        let bytes = to_bytes(&doc()).unwrap();
        for cut in 0..bytes.len() {
            assert!(from_bytes::<Doc>(&bytes[..cut]).is_err(), "cut at {cut}");
        }
        let mut longer = bytes.clone();
        longer.push(0);
        assert!(from_bytes::<Doc>(&longer).is_err());
        for i in 0..bytes.len() {
            let mut b = bytes.clone();
            b[i] ^= 0xff;
            let _ = from_bytes::<Doc>(&b);
        }
    }

    #[test]
    fn a_huge_length_is_rejected_without_allocating() {
        let mut b = u64::MAX.to_le_bytes().to_vec();
        b.extend_from_slice(b"xx");
        assert!(from_bytes::<Vec<u8>>(&b).is_err());
        assert!(from_bytes::<String>(&b).is_err());
    }
}
