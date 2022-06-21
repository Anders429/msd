//! MSD deserialization.
//!
//! This module provides all of the tools necessary for deserializing MSD input into types
//! implementing [`Deserialize`].
//!
//! # Example
//! ```
//! use std::collections::HashMap;
//!
//! let input = b"#foo:1;\n#bar:2;\n";
//! let deserialized: HashMap<String, u64> = msd::from_bytes(input.as_slice()).unwrap();
//!
//! let mut expected = HashMap::new();
//! expected.insert("foo".to_owned(), 1);
//! expected.insert("bar".to_owned(), 2);
//!
//! assert_eq!(deserialized, expected);
//! ```
//!
//! [`Deserialize`]: serde::Deserialize

mod r#enum;
mod error;
mod map;
mod parse;
mod position;
mod seq;
mod r#struct;
mod tuple;

pub use error::{Error, Result};

use position::Position;
use serde::{
    de,
    de::{DeserializeOwned, Visitor},
    Deserialize,
};
use std::io::Read;

/// Deserializes data from MSD format.
///
/// `Deserializer` can be used to read from any value that implements the [`Read`] trait. The bytes
/// will be interpreted as MSD and deserialized into a given type.
///
/// # Example
/// ```
/// use serde::Deserialize;
/// use std::collections::HashMap;
///
/// let mut deserializer = msd::Deserializer::new(b"#foo:1;\n#bar:2;\n".as_slice());
/// let deserialized = HashMap::deserialize(&mut deserializer).unwrap();
///
/// let mut expected: HashMap<String, u64> = HashMap::new();
/// expected.insert("foo".to_owned(), 1);
/// expected.insert("bar".to_owned(), 2);
///
/// assert_eq!(deserialized, expected);
/// ```
#[derive(Debug)]
pub struct Deserializer<R> {
    tags: parse::Tags<R>,
}

impl<R> Deserializer<R> {
    pub fn new(reader: R) -> Self
    where
        R: Read,
    {
        Self {
            tags: parse::Tags::new(reader),
        }
    }
}

macro_rules! deserialize_impossible {
    ($deserialize_fn:ident) => {
        fn $deserialize_fn<V>(self, _visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            todo!()
        }
    }
}

macro_rules! deserialize_primitive {
    ($deserialize_fn:ident, $parse_fn:ident, $visit_fn:ident) => {
        fn $deserialize_fn<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            let mut tag = self.tags.next()?;
            let mut values = tag.next()?;
            let value = values.next()?;
            let parsed = value.$parse_fn()?;
            let value_position = value.position();
            values.assert_exhausted()?;
            tag.assert_exhausted()?;
            self.tags.assert_exhausted()?;
            visitor.$visit_fn(parsed).map_err(|mut error: Error| {
                error.set_position(value_position);
                error
            })
        }
    }
}

macro_rules! deserialize_borrowed_primitive {
    ($deserialize_fn:ident, $parse_fn:ident, $visit_fn:ident) => {
        fn $deserialize_fn<V>(self, visitor: V) -> Result<V::Value>
        where
            V: Visitor<'de>,
        {
            let mut tag = self.tags.next()?;
            let mut values = tag.next()?;
            let value = values.next()?;
            let parsed = value.$parse_fn()?;
            let value_position = value.position();
            values.assert_exhausted()?;
            tag.assert_exhausted()?;
            self.tags.assert_exhausted()?;
            visitor.$visit_fn(&parsed).map_err(|mut error: Error| {
                error.set_position(value_position);
                error
            })
        }
    }
}

impl<'de, 'a, R> de::Deserializer<'de> for &'a mut Deserializer<R>
where
    R: Read,
{
    type Error = Error;

    deserialize_impossible!(deserialize_any);
    deserialize_impossible!(deserialize_ignored_any);

    deserialize_primitive!(deserialize_bool, parse_bool, visit_bool);
    deserialize_primitive!(deserialize_i8, parse_i8, visit_i8);
    deserialize_primitive!(deserialize_i16, parse_i16, visit_i16);
    deserialize_primitive!(deserialize_i32, parse_i32, visit_i32);
    deserialize_primitive!(deserialize_i64, parse_i64, visit_i64);
    #[cfg(has_i128)]
    deserialize_primitive!(deserialize_i128, parse_i128, visit_i128);
    deserialize_primitive!(deserialize_u8, parse_u8, visit_u8);
    deserialize_primitive!(deserialize_u16, parse_u16, visit_u16);
    deserialize_primitive!(deserialize_u32, parse_u32, visit_u32);
    deserialize_primitive!(deserialize_u64, parse_u64, visit_u64);
    #[cfg(has_i128)]
    deserialize_primitive!(deserialize_u128, parse_u128, visit_u128);
    deserialize_primitive!(deserialize_f32, parse_f32, visit_f32);
    deserialize_primitive!(deserialize_f64, parse_f64, visit_f64);
    deserialize_primitive!(deserialize_char, parse_char, visit_char);
    deserialize_borrowed_primitive!(deserialize_str, parse_string, visit_str);
    deserialize_primitive!(deserialize_string, parse_string, visit_string);
    deserialize_borrowed_primitive!(deserialize_identifier, parse_identifier, visit_str);

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut tag = self.tags.next()?;
        let mut values = tag.next()?;
        let value = values.next()?;
        // Parsed bytes must be owned, since it removes escaping and comments.
        let parsed = value.parse_byte_buf();
        let value_position = value.position();
        values.assert_exhausted()?;
        tag.assert_exhausted()?;
        self.tags.assert_exhausted()?;
        visitor.visit_bytes(&parsed).map_err(|mut error: Error| {
            error.set_position(value_position);
            error
        })
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut tag = self.tags.next()?;
        let mut values = tag.next()?;
        let value = values.next()?;
        let parsed = value.parse_byte_buf();
        let value_position = value.position();
        values.assert_exhausted()?;
        tag.assert_exhausted()?;
        self.tags.assert_exhausted()?;
        visitor.visit_byte_buf(parsed).map_err(|mut error: Error| {
            error.set_position(value_position);
            error
        })
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.tags.has_next()? {
            visitor.visit_some(self)
        } else {
            visitor.visit_none()
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut tag = self.tags.next()?;
        let mut values = tag.next()?;
        let value = values.next()?;
        value.parse_unit()?;
        let value_position = value.position();
        values.assert_exhausted()?;
        tag.assert_exhausted()?;
        self.tags.assert_exhausted()?;
        visitor.visit_unit().map_err(|mut error: Error| {
            error.set_position(value_position);
            error
        })
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut tag = self.tags.next()?;
        let mut values = tag.next()?;
        let value = values.next()?;
        value.parse_unit()?;
        let value_position = value.position();
        values.assert_exhausted()?;
        tag.assert_exhausted()?;
        self.tags.assert_exhausted()?;
        visitor.visit_unit().map_err(|mut error: Error| {
            error.set_position(value_position);
            error
        })
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let result = visitor.visit_seq(seq::root::Access::new(&mut self.tags))?;
        self.tags.assert_exhausted()?;
        Ok(result)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut tag = self.tags.next()?;
        let mut values = tag.next()?;
        let result = visitor.visit_seq(tuple::Access::new(&mut values, len))?;
        values.assert_exhausted()?;
        tag.assert_exhausted()?;
        self.tags.assert_exhausted()?;
        Ok(result)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut tag = self.tags.next()?;
        let mut values = tag.next()?;
        let result = visitor.visit_seq(tuple::Access::new(&mut values, len))?;
        values.assert_exhausted()?;
        tag.assert_exhausted()?;
        self.tags.assert_exhausted()?;
        Ok(result)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let result = visitor.visit_map(map::root::Access::new(&mut self.tags))?;
        self.tags.assert_exhausted()?;
        Ok(result)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let result = visitor.visit_map(r#struct::Access::new(&mut self.tags, fields))?;
        self.tags.assert_exhausted()?;
        Ok(result)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let mut tag = self.tags.next()?;
        let mut values = tag.next()?;
        let result = visitor.visit_enum(r#enum::Access::new(&mut values))?;
        values.assert_exhausted()?;
        tag.assert_exhausted()?;
        self.tags.assert_exhausted()?;
        Ok(result)
    }
}

unsafe impl<R> Send for Deserializer<R> {}

unsafe impl<R> Sync for Deserializer<R> {}

/// Deserialize a value of type `T` from the given `reader`.
pub fn from_reader<R, T>(reader: R) -> Result<T>
where
    R: Read,
    T: DeserializeOwned,
{
    let mut deserializer = Deserializer::new(reader);
    T::deserialize(&mut deserializer)
}

/// Deserialize a value of type `T` from a slice of bytes.
pub fn from_bytes<'a, T>(bytes: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::new(bytes);
    T::deserialize(&mut deserializer)
}

#[cfg(test)]
mod tests {
    use super::{error, Deserializer, Error, Position};
    use claim::{assert_err_eq, assert_ok_eq};
    use serde::{de, de::Visitor, Deserialize};
    use serde_bytes::ByteBuf;
    use serde_derive::Deserialize;
    use std::{collections::HashMap, fmt};

    #[test]
    fn bool_true() {
        let mut deserializer = Deserializer::new(b"#true;".as_slice());

        assert_ok_eq!(bool::deserialize(&mut deserializer), true);
    }

    #[test]
    fn bool_false() {
        let mut deserializer = Deserializer::new(b"#false;".as_slice());

        assert_ok_eq!(bool::deserialize(&mut deserializer), false);
    }

    #[test]
    fn bool_invalid() {
        let mut deserializer = Deserializer::new(b"#invalid;".as_slice());

        assert_err_eq!(
            bool::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedBool, Position::new(0, 1))
        );
    }

    #[test]
    fn bool_custom_error() {
        #[derive(Debug)]
        struct CustomBool;

        impl<'de> Deserialize<'de> for CustomBool {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct CustomBoolVisitor;

                impl<'de> Visitor<'de> for CustomBoolVisitor {
                    type Value = CustomBool;

                    fn expecting(&self, _f: &mut fmt::Formatter) -> fmt::Result {
                        unimplemented!()
                    }

                    fn visit_bool<E>(self, _value: bool) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Err(de::Error::custom("foo"))
                    }
                }

                deserializer.deserialize_bool(CustomBoolVisitor)
            }
        }

        let mut deserializer = Deserializer::new(b"#true;".as_slice());

        assert_err_eq!(
            CustomBool::deserialize(&mut deserializer),
            Error::new(error::Kind::Custom("foo".to_string()), Position::new(0, 1))
        );
    }

    #[test]
    fn i8_positive() {
        let mut deserializer = Deserializer::new(b"#42;".as_slice());

        assert_ok_eq!(i8::deserialize(&mut deserializer), 42);
    }

    #[test]
    fn i8_negative() {
        let mut deserializer = Deserializer::new(b"#-42;".as_slice());

        assert_ok_eq!(i8::deserialize(&mut deserializer), -42);
    }

    #[test]
    fn i8_zero() {
        let mut deserializer = Deserializer::new(b"#0;".as_slice());

        assert_ok_eq!(i8::deserialize(&mut deserializer), 0);
    }

    #[test]
    fn i8_positive_overflow() {
        let mut deserializer = Deserializer::new(b"#128;".as_slice());

        assert_err_eq!(
            i8::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedI8, Position::new(0, 1))
        );
    }

    #[test]
    fn i8_negative_overflow() {
        let mut deserializer = Deserializer::new(b"#-129;".as_slice());

        assert_err_eq!(
            i8::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedI8, Position::new(0, 1))
        );
    }

    #[test]
    fn i8_invalid() {
        let mut deserializer = Deserializer::new(b"#invalid;".as_slice());

        assert_err_eq!(
            i8::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedI8, Position::new(0, 1))
        );
    }

    #[test]
    fn i8_custom_error() {
        #[derive(Debug)]
        struct CustomI8;

        impl<'de> Deserialize<'de> for CustomI8 {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct CustomI8Visitor;

                impl<'de> Visitor<'de> for CustomI8Visitor {
                    type Value = CustomI8;

                    fn expecting(&self, _f: &mut fmt::Formatter) -> fmt::Result {
                        unimplemented!()
                    }

                    fn visit_i8<E>(self, _value: i8) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Err(de::Error::custom("foo"))
                    }
                }

                deserializer.deserialize_i8(CustomI8Visitor)
            }
        }

        let mut deserializer = Deserializer::new(b"#42;".as_slice());

        assert_err_eq!(
            CustomI8::deserialize(&mut deserializer),
            Error::new(error::Kind::Custom("foo".to_string()), Position::new(0, 1))
        );
    }

    #[test]
    fn i16_positive() {
        let mut deserializer = Deserializer::new(b"#42;".as_slice());

        assert_ok_eq!(i16::deserialize(&mut deserializer), 42);
    }

    #[test]
    fn i16_negative() {
        let mut deserializer = Deserializer::new(b"#-42;".as_slice());

        assert_ok_eq!(i16::deserialize(&mut deserializer), -42);
    }

    #[test]
    fn i16_zero() {
        let mut deserializer = Deserializer::new(b"#0;".as_slice());

        assert_ok_eq!(i16::deserialize(&mut deserializer), 0);
    }

    #[test]
    fn i16_positive_overflow() {
        let mut deserializer = Deserializer::new(b"#32768;".as_slice());

        assert_err_eq!(
            i16::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedI16, Position::new(0, 1))
        );
    }

    #[test]
    fn i16_negative_overflow() {
        let mut deserializer = Deserializer::new(b"#-32769;".as_slice());

        assert_err_eq!(
            i16::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedI16, Position::new(0, 1))
        );
    }

    #[test]
    fn i16_invalid() {
        let mut deserializer = Deserializer::new(b"#invalid;".as_slice());

        assert_err_eq!(
            i16::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedI16, Position::new(0, 1))
        );
    }

    #[test]
    fn i16_custom_error() {
        #[derive(Debug)]
        struct CustomI16;

        impl<'de> Deserialize<'de> for CustomI16 {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct CustomI16Visitor;

                impl<'de> Visitor<'de> for CustomI16Visitor {
                    type Value = CustomI16;

                    fn expecting(&self, _f: &mut fmt::Formatter) -> fmt::Result {
                        unimplemented!()
                    }

                    fn visit_i16<E>(self, _value: i16) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Err(de::Error::custom("foo"))
                    }
                }

                deserializer.deserialize_i16(CustomI16Visitor)
            }
        }

        let mut deserializer = Deserializer::new(b"#42;".as_slice());

        assert_err_eq!(
            CustomI16::deserialize(&mut deserializer),
            Error::new(error::Kind::Custom("foo".to_string()), Position::new(0, 1))
        );
    }

    #[test]
    fn i32_positive() {
        let mut deserializer = Deserializer::new(b"#42;".as_slice());

        assert_ok_eq!(i32::deserialize(&mut deserializer), 42);
    }

    #[test]
    fn i32_negative() {
        let mut deserializer = Deserializer::new(b"#-42;".as_slice());

        assert_ok_eq!(i32::deserialize(&mut deserializer), -42);
    }

    #[test]
    fn i32_zero() {
        let mut deserializer = Deserializer::new(b"#0;".as_slice());

        assert_ok_eq!(i32::deserialize(&mut deserializer), 0);
    }

    #[test]
    fn i32_positive_overflow() {
        let mut deserializer = Deserializer::new(b"#2147483648;".as_slice());

        assert_err_eq!(
            i32::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedI32, Position::new(0, 1))
        );
    }

    #[test]
    fn i32_negative_overflow() {
        let mut deserializer = Deserializer::new(b"#-2147483649;".as_slice());

        assert_err_eq!(
            i32::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedI32, Position::new(0, 1))
        );
    }

    #[test]
    fn i32_invalid() {
        let mut deserializer = Deserializer::new(b"#invalid;".as_slice());

        assert_err_eq!(
            i32::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedI32, Position::new(0, 1))
        );
    }

    #[test]
    fn i32_custom_error() {
        #[derive(Debug)]
        struct CustomI32;

        impl<'de> Deserialize<'de> for CustomI32 {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct CustomI32Visitor;

                impl<'de> Visitor<'de> for CustomI32Visitor {
                    type Value = CustomI32;

                    fn expecting(&self, _f: &mut fmt::Formatter) -> fmt::Result {
                        unimplemented!()
                    }

                    fn visit_i32<E>(self, _value: i32) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Err(de::Error::custom("foo"))
                    }
                }

                deserializer.deserialize_i32(CustomI32Visitor)
            }
        }

        let mut deserializer = Deserializer::new(b"#42;".as_slice());

        assert_err_eq!(
            CustomI32::deserialize(&mut deserializer),
            Error::new(error::Kind::Custom("foo".to_string()), Position::new(0, 1))
        );
    }

    #[test]
    fn i64_positive() {
        let mut deserializer = Deserializer::new(b"#42;".as_slice());

        assert_ok_eq!(i64::deserialize(&mut deserializer), 42);
    }

    #[test]
    fn i64_negative() {
        let mut deserializer = Deserializer::new(b"#-42;".as_slice());

        assert_ok_eq!(i64::deserialize(&mut deserializer), -42);
    }

    #[test]
    fn i64_zero() {
        let mut deserializer = Deserializer::new(b"#0;".as_slice());

        assert_ok_eq!(i64::deserialize(&mut deserializer), 0);
    }

    #[test]
    fn i64_positive_overflow() {
        let mut deserializer = Deserializer::new(b"#9223372036854775808;".as_slice());

        assert_err_eq!(
            i64::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedI64, Position::new(0, 1))
        );
    }

    #[test]
    fn i64_negative_overflow() {
        let mut deserializer = Deserializer::new(b"#-9223372036854775809;".as_slice());

        assert_err_eq!(
            i64::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedI64, Position::new(0, 1))
        );
    }

    #[test]
    fn i64_invalid() {
        let mut deserializer = Deserializer::new(b"#invalid;".as_slice());

        assert_err_eq!(
            i64::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedI64, Position::new(0, 1))
        );
    }

    #[test]
    fn i64_custom_error() {
        #[derive(Debug)]
        struct CustomI64;

        impl<'de> Deserialize<'de> for CustomI64 {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct CustomI64Visitor;

                impl<'de> Visitor<'de> for CustomI64Visitor {
                    type Value = CustomI64;

                    fn expecting(&self, _f: &mut fmt::Formatter) -> fmt::Result {
                        unimplemented!()
                    }

                    fn visit_i64<E>(self, _value: i64) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Err(de::Error::custom("foo"))
                    }
                }

                deserializer.deserialize_i64(CustomI64Visitor)
            }
        }

        let mut deserializer = Deserializer::new(b"#42;".as_slice());

        assert_err_eq!(
            CustomI64::deserialize(&mut deserializer),
            Error::new(error::Kind::Custom("foo".to_string()), Position::new(0, 1))
        );
    }

    #[test]
    fn i128_positive() {
        let mut deserializer = Deserializer::new(b"#42;".as_slice());

        assert_ok_eq!(i128::deserialize(&mut deserializer), 42);
    }

    #[test]
    fn i128_negative() {
        let mut deserializer = Deserializer::new(b"#-42;".as_slice());

        assert_ok_eq!(i128::deserialize(&mut deserializer), -42);
    }

    #[test]
    fn i128_zero() {
        let mut deserializer = Deserializer::new(b"#0;".as_slice());

        assert_ok_eq!(i128::deserialize(&mut deserializer), 0);
    }

    #[test]
    fn i128_positive_overflow() {
        let mut deserializer =
            Deserializer::new(b"#170141183460469231731687303715884105728;".as_slice());

        assert_err_eq!(
            i128::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedI128, Position::new(0, 1))
        );
    }

    #[test]
    fn i128_negative_overflow() {
        let mut deserializer =
            Deserializer::new(b"#-170141183460469231731687303715884105729;".as_slice());

        assert_err_eq!(
            i128::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedI128, Position::new(0, 1))
        );
    }

    #[test]
    fn i128_invalid() {
        let mut deserializer = Deserializer::new(b"#invalid;".as_slice());

        assert_err_eq!(
            i128::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedI128, Position::new(0, 1))
        );
    }

    #[test]
    fn i128_custom_error() {
        #[derive(Debug)]
        struct CustomI128;

        impl<'de> Deserialize<'de> for CustomI128 {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct CustomI128Visitor;

                impl<'de> Visitor<'de> for CustomI128Visitor {
                    type Value = CustomI128;

                    fn expecting(&self, _f: &mut fmt::Formatter) -> fmt::Result {
                        unimplemented!()
                    }

                    fn visit_i128<E>(self, _value: i128) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Err(de::Error::custom("foo"))
                    }
                }

                deserializer.deserialize_i128(CustomI128Visitor)
            }
        }

        let mut deserializer = Deserializer::new(b"#42;".as_slice());

        assert_err_eq!(
            CustomI128::deserialize(&mut deserializer),
            Error::new(error::Kind::Custom("foo".to_string()), Position::new(0, 1))
        );
    }

    #[test]
    fn u8() {
        let mut deserializer = Deserializer::new(b"#42;".as_slice());

        assert_ok_eq!(u8::deserialize(&mut deserializer), 42);
    }

    #[test]
    fn u8_zero() {
        let mut deserializer = Deserializer::new(b"#0;".as_slice());

        assert_ok_eq!(u8::deserialize(&mut deserializer), 0);
    }

    #[test]
    fn u8_overflow() {
        let mut deserializer = Deserializer::new(b"#256;".as_slice());

        assert_err_eq!(
            u8::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedU8, Position::new(0, 1))
        );
    }

    #[test]
    fn u8_invalid() {
        let mut deserializer = Deserializer::new(b"#invalid;".as_slice());

        assert_err_eq!(
            u8::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedU8, Position::new(0, 1))
        );
    }

    #[test]
    fn u8_custom_error() {
        #[derive(Debug)]
        struct CustomU8;

        impl<'de> Deserialize<'de> for CustomU8 {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct CustomU8Visitor;

                impl<'de> Visitor<'de> for CustomU8Visitor {
                    type Value = CustomU8;

                    fn expecting(&self, _f: &mut fmt::Formatter) -> fmt::Result {
                        unimplemented!()
                    }

                    fn visit_u8<E>(self, _value: u8) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Err(de::Error::custom("foo"))
                    }
                }

                deserializer.deserialize_u8(CustomU8Visitor)
            }
        }

        let mut deserializer = Deserializer::new(b"#42;".as_slice());

        assert_err_eq!(
            CustomU8::deserialize(&mut deserializer),
            Error::new(error::Kind::Custom("foo".to_string()), Position::new(0, 1))
        );
    }

    #[test]
    fn u16() {
        let mut deserializer = Deserializer::new(b"#42;".as_slice());

        assert_ok_eq!(u16::deserialize(&mut deserializer), 42);
    }

    #[test]
    fn u16_zero() {
        let mut deserializer = Deserializer::new(b"#0;".as_slice());

        assert_ok_eq!(u16::deserialize(&mut deserializer), 0);
    }

    #[test]
    fn u16_overflow() {
        let mut deserializer = Deserializer::new(b"#65536;".as_slice());

        assert_err_eq!(
            u16::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedU16, Position::new(0, 1))
        );
    }

    #[test]
    fn u16_invalid() {
        let mut deserializer = Deserializer::new(b"#invalid;".as_slice());

        assert_err_eq!(
            u16::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedU16, Position::new(0, 1))
        );
    }

    #[test]
    fn u16_custom_error() {
        #[derive(Debug)]
        struct CustomU16;

        impl<'de> Deserialize<'de> for CustomU16 {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct CustomU16Visitor;

                impl<'de> Visitor<'de> for CustomU16Visitor {
                    type Value = CustomU16;

                    fn expecting(&self, _f: &mut fmt::Formatter) -> fmt::Result {
                        unimplemented!()
                    }

                    fn visit_u16<E>(self, _value: u16) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Err(de::Error::custom("foo"))
                    }
                }

                deserializer.deserialize_u16(CustomU16Visitor)
            }
        }

        let mut deserializer = Deserializer::new(b"#42;".as_slice());

        assert_err_eq!(
            CustomU16::deserialize(&mut deserializer),
            Error::new(error::Kind::Custom("foo".to_string()), Position::new(0, 1))
        );
    }

    #[test]
    fn u32() {
        let mut deserializer = Deserializer::new(b"#42;".as_slice());

        assert_ok_eq!(u32::deserialize(&mut deserializer), 42);
    }

    #[test]
    fn u32_zero() {
        let mut deserializer = Deserializer::new(b"#0;".as_slice());

        assert_ok_eq!(u32::deserialize(&mut deserializer), 0);
    }

    #[test]
    fn u32_overflow() {
        let mut deserializer = Deserializer::new(b"#4294967296;".as_slice());

        assert_err_eq!(
            u32::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedU32, Position::new(0, 1))
        );
    }

    #[test]
    fn u32_invalid() {
        let mut deserializer = Deserializer::new(b"#invalid;".as_slice());

        assert_err_eq!(
            u32::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedU32, Position::new(0, 1))
        );
    }

    #[test]
    fn u32_custom_error() {
        #[derive(Debug)]
        struct CustomU32;

        impl<'de> Deserialize<'de> for CustomU32 {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct CustomU32Visitor;

                impl<'de> Visitor<'de> for CustomU32Visitor {
                    type Value = CustomU32;

                    fn expecting(&self, _f: &mut fmt::Formatter) -> fmt::Result {
                        unimplemented!()
                    }

                    fn visit_u32<E>(self, _value: u32) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Err(de::Error::custom("foo"))
                    }
                }

                deserializer.deserialize_u32(CustomU32Visitor)
            }
        }

        let mut deserializer = Deserializer::new(b"#42;".as_slice());

        assert_err_eq!(
            CustomU32::deserialize(&mut deserializer),
            Error::new(error::Kind::Custom("foo".to_string()), Position::new(0, 1))
        );
    }

    #[test]
    fn u64() {
        let mut deserializer = Deserializer::new(b"#42;".as_slice());

        assert_ok_eq!(u64::deserialize(&mut deserializer), 42);
    }

    #[test]
    fn u64_zero() {
        let mut deserializer = Deserializer::new(b"#0;".as_slice());

        assert_ok_eq!(u64::deserialize(&mut deserializer), 0);
    }

    #[test]
    fn u64_overflow() {
        let mut deserializer = Deserializer::new(b"#18446744073709551616;".as_slice());

        assert_err_eq!(
            u64::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedU64, Position::new(0, 1))
        );
    }

    #[test]
    fn u64_invalid() {
        let mut deserializer = Deserializer::new(b"#invalid;".as_slice());

        assert_err_eq!(
            u64::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedU64, Position::new(0, 1))
        );
    }

    #[test]
    fn u64_custom_error() {
        #[derive(Debug)]
        struct CustomU64;

        impl<'de> Deserialize<'de> for CustomU64 {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct CustomU64Visitor;

                impl<'de> Visitor<'de> for CustomU64Visitor {
                    type Value = CustomU64;

                    fn expecting(&self, _f: &mut fmt::Formatter) -> fmt::Result {
                        unimplemented!()
                    }

                    fn visit_u64<E>(self, _value: u64) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Err(de::Error::custom("foo"))
                    }
                }

                deserializer.deserialize_u64(CustomU64Visitor)
            }
        }

        let mut deserializer = Deserializer::new(b"#42;".as_slice());

        assert_err_eq!(
            CustomU64::deserialize(&mut deserializer),
            Error::new(error::Kind::Custom("foo".to_string()), Position::new(0, 1))
        );
    }

    #[test]
    fn u128() {
        let mut deserializer = Deserializer::new(b"#42;".as_slice());

        assert_ok_eq!(u128::deserialize(&mut deserializer), 42);
    }

    #[test]
    fn u128_zero() {
        let mut deserializer = Deserializer::new(b"#0;".as_slice());

        assert_ok_eq!(u128::deserialize(&mut deserializer), 0);
    }

    #[test]
    fn u128_overflow() {
        let mut deserializer =
            Deserializer::new(b"#340282366920938463463374607431768211456;".as_slice());

        assert_err_eq!(
            u128::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedU128, Position::new(0, 1))
        );
    }

    #[test]
    fn u128_invalid() {
        let mut deserializer = Deserializer::new(b"#invalid;".as_slice());

        assert_err_eq!(
            u128::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedU128, Position::new(0, 1))
        );
    }

    #[test]
    fn u128_custom_error() {
        #[derive(Debug)]
        struct CustomU128;

        impl<'de> Deserialize<'de> for CustomU128 {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct CustomU128Visitor;

                impl<'de> Visitor<'de> for CustomU128Visitor {
                    type Value = CustomU128;

                    fn expecting(&self, _f: &mut fmt::Formatter) -> fmt::Result {
                        unimplemented!()
                    }

                    fn visit_u128<E>(self, _value: u128) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Err(de::Error::custom("foo"))
                    }
                }

                deserializer.deserialize_u128(CustomU128Visitor)
            }
        }

        let mut deserializer = Deserializer::new(b"#42;".as_slice());

        assert_err_eq!(
            CustomU128::deserialize(&mut deserializer),
            Error::new(error::Kind::Custom("foo".to_string()), Position::new(0, 1))
        );
    }

    #[test]
    fn f32_positive() {
        let mut deserializer = Deserializer::new(b"#42.9;".as_slice());

        assert_ok_eq!(f32::deserialize(&mut deserializer), 42.9);
    }

    #[test]
    fn f32_negative() {
        let mut deserializer = Deserializer::new(b"#-42.9;".as_slice());

        assert_ok_eq!(f32::deserialize(&mut deserializer), -42.9);
    }

    #[test]
    fn f32_zero() {
        let mut deserializer = Deserializer::new(b"#0;".as_slice());

        assert_ok_eq!(f32::deserialize(&mut deserializer), 0.0);
    }

    #[test]
    fn f32_positive_overflow() {
        let mut deserializer = Deserializer::new(b"#3.40282347E+39;".as_slice());

        assert_ok_eq!(f32::deserialize(&mut deserializer), f32::INFINITY,);
    }

    #[test]
    fn f32_negative_overflow() {
        let mut deserializer = Deserializer::new(b"#-3.40282347E+39;".as_slice());

        assert_ok_eq!(f32::deserialize(&mut deserializer), f32::NEG_INFINITY);
    }

    #[test]
    fn f32_invalid() {
        let mut deserializer = Deserializer::new(b"#invalid;".as_slice());

        assert_err_eq!(
            f32::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedF32, Position::new(0, 1))
        );
    }

    #[test]
    fn f32_custom_error() {
        #[derive(Debug)]
        struct CustomF32;

        impl<'de> Deserialize<'de> for CustomF32 {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct CustomF32Visitor;

                impl<'de> Visitor<'de> for CustomF32Visitor {
                    type Value = CustomF32;

                    fn expecting(&self, _f: &mut fmt::Formatter) -> fmt::Result {
                        unimplemented!()
                    }

                    fn visit_f32<E>(self, _value: f32) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Err(de::Error::custom("foo"))
                    }
                }

                deserializer.deserialize_f32(CustomF32Visitor)
            }
        }

        let mut deserializer = Deserializer::new(b"#42;".as_slice());

        assert_err_eq!(
            CustomF32::deserialize(&mut deserializer),
            Error::new(error::Kind::Custom("foo".to_string()), Position::new(0, 1))
        );
    }

    #[test]
    fn f64_positive() {
        let mut deserializer = Deserializer::new(b"#42.9;".as_slice());

        assert_ok_eq!(f64::deserialize(&mut deserializer), 42.9);
    }

    #[test]
    fn f64_negative() {
        let mut deserializer = Deserializer::new(b"#-42.9;".as_slice());

        assert_ok_eq!(f64::deserialize(&mut deserializer), -42.9);
    }

    #[test]
    fn f64_zero() {
        let mut deserializer = Deserializer::new(b"#0;".as_slice());

        assert_ok_eq!(f64::deserialize(&mut deserializer), 0.0);
    }

    #[test]
    fn f64_positive_overflow() {
        let mut deserializer = Deserializer::new(b"#1.7976931348623157E+309;".as_slice());

        assert_ok_eq!(f64::deserialize(&mut deserializer), f64::INFINITY,);
    }

    #[test]
    fn f64_negative_overflow() {
        let mut deserializer = Deserializer::new(b"#-1.7976931348623157E+309;".as_slice());

        assert_ok_eq!(f64::deserialize(&mut deserializer), f64::NEG_INFINITY);
    }

    #[test]
    fn f64_invalid() {
        let mut deserializer = Deserializer::new(b"#invalid;".as_slice());

        assert_err_eq!(
            f64::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedF64, Position::new(0, 1))
        );
    }

    #[test]
    fn f64_custom_error() {
        #[derive(Debug)]
        struct CustomF64;

        impl<'de> Deserialize<'de> for CustomF64 {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct CustomF64Visitor;

                impl<'de> Visitor<'de> for CustomF64Visitor {
                    type Value = CustomF64;

                    fn expecting(&self, _f: &mut fmt::Formatter) -> fmt::Result {
                        unimplemented!()
                    }

                    fn visit_f64<E>(self, _value: f64) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Err(de::Error::custom("foo"))
                    }
                }

                deserializer.deserialize_f64(CustomF64Visitor)
            }
        }

        let mut deserializer = Deserializer::new(b"#42;".as_slice());

        assert_err_eq!(
            CustomF64::deserialize(&mut deserializer),
            Error::new(error::Kind::Custom("foo".to_string()), Position::new(0, 1))
        );
    }

    #[test]
    fn char() {
        let mut deserializer = Deserializer::new(b"#a;".as_slice());

        assert_ok_eq!(char::deserialize(&mut deserializer), 'a');
    }

    #[test]
    fn char_empty() {
        let mut deserializer = Deserializer::new(b"".as_slice());

        assert_err_eq!(
            char::deserialize(&mut deserializer),
            Error::new(error::Kind::EndOfFile, Position::new(0, 0))
        );
    }

    #[test]
    fn char_unexpected_tag() {
        let mut deserializer = Deserializer::new(b"#a;#b;".as_slice());

        assert_err_eq!(
            char::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedTag, Position::new(0, 3))
        );
    }

    #[test]
    fn char_unexpected_values() {
        let mut deserializer = Deserializer::new(b"#a;b;".as_slice());

        assert_err_eq!(
            char::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedValues, Position::new(0, 3))
        );
    }

    #[test]
    fn char_unexpected_value() {
        let mut deserializer = Deserializer::new(b"#a:b;\n".as_slice());

        assert_err_eq!(
            char::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedValue, Position::new(0, 3))
        );
    }

    #[test]
    fn char_invalid() {
        let mut deserializer = Deserializer::new(b"#abc;\n".as_slice());

        assert_err_eq!(
            char::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedChar, Position::new(0, 1)),
        );
    }

    #[test]
    fn char_custom_error() {
        #[derive(Debug)]
        struct CustomChar;

        impl<'de> Deserialize<'de> for CustomChar {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct CustomCharVisitor;

                impl<'de> Visitor<'de> for CustomCharVisitor {
                    type Value = CustomChar;

                    fn expecting(&self, _f: &mut fmt::Formatter) -> fmt::Result {
                        unimplemented!()
                    }

                    fn visit_char<E>(self, _value: char) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Err(de::Error::custom("foo"))
                    }
                }

                deserializer.deserialize_char(CustomCharVisitor)
            }
        }

        let mut deserializer = Deserializer::new(b"#a;".as_slice());

        assert_err_eq!(
            CustomChar::deserialize(&mut deserializer),
            Error::new(error::Kind::Custom("foo".to_string()), Position::new(0, 1))
        );
    }

    #[test]
    fn string() {
        let mut deserializer = Deserializer::new(b"#foo;".as_slice());

        assert_ok_eq!(String::deserialize(&mut deserializer), "foo".to_string(),);
    }

    #[test]
    fn string_unexpected_tag() {
        let mut deserializer = Deserializer::new(b"#foo;#bar;".as_slice());

        assert_err_eq!(
            String::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedTag, Position::new(0, 5))
        );
    }

    #[test]
    fn string_unexpected_values() {
        let mut deserializer = Deserializer::new(b"#foo;bar;".as_slice());

        assert_err_eq!(
            String::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedValues, Position::new(0, 5))
        );
    }

    #[test]
    fn string_unexpected_value() {
        let mut deserializer = Deserializer::new(b"#foo:bar;\n".as_slice());

        assert_err_eq!(
            String::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedValue, Position::new(0, 5))
        );
    }

    #[test]
    fn string_invalid() {
        let mut deserializer = Deserializer::new(b"#\xF0\x9Ffoo;\n".as_slice());

        assert_err_eq!(
            String::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedString, Position::new(0, 1)),
        );
    }

    #[test]
    fn string_custom_error() {
        #[derive(Debug)]
        struct CustomString;

        impl<'de> Deserialize<'de> for CustomString {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct CustomStringVisitor;

                impl<'de> Visitor<'de> for CustomStringVisitor {
                    type Value = CustomString;

                    fn expecting(&self, _f: &mut fmt::Formatter) -> fmt::Result {
                        unimplemented!()
                    }

                    fn visit_str<E>(self, _value: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Err(de::Error::custom("foo"))
                    }
                }

                deserializer.deserialize_string(CustomStringVisitor)
            }
        }

        let mut deserializer = Deserializer::new(b"#a;".as_slice());

        assert_err_eq!(
            CustomString::deserialize(&mut deserializer),
            Error::new(error::Kind::Custom("foo".to_string()), Position::new(0, 1))
        );
    }

    #[test]
    fn byte_buf() {
        let mut deserializer = Deserializer::new(b"#foo;".as_slice());

        assert_ok_eq!(ByteBuf::deserialize(&mut deserializer), b"foo",);
    }

    #[test]
    fn byte_buf_unexpected_tag() {
        let mut deserializer = Deserializer::new(b"#foo;#bar;".as_slice());

        assert_err_eq!(
            ByteBuf::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedTag, Position::new(0, 5))
        );
    }

    #[test]
    fn byte_buf_unexpected_values() {
        let mut deserializer = Deserializer::new(b"#foo;bar;".as_slice());

        assert_err_eq!(
            ByteBuf::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedValues, Position::new(0, 5))
        );
    }

    #[test]
    fn byte_buf_unexpected_value() {
        let mut deserializer = Deserializer::new(b"#foo:bar;\n".as_slice());

        assert_err_eq!(
            ByteBuf::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedValue, Position::new(0, 5))
        );
    }

    #[test]
    fn byte_buf_non_ascii() {
        let mut deserializer = Deserializer::new(b"#\xF0\x9Ffoo;\n".as_slice());

        assert_ok_eq!(ByteBuf::deserialize(&mut deserializer), b"\xF0\x9Ffoo",);
    }

    #[test]
    fn byte_buf_custom_error() {
        #[derive(Debug)]
        struct CustomByteBuf;

        impl<'de> Deserialize<'de> for CustomByteBuf {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct CustomByteBufVisitor;

                impl<'de> Visitor<'de> for CustomByteBufVisitor {
                    type Value = CustomByteBuf;

                    fn expecting(&self, _f: &mut fmt::Formatter) -> fmt::Result {
                        unimplemented!()
                    }

                    fn visit_byte_buf<E>(self, _value: Vec<u8>) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Err(de::Error::custom("foo"))
                    }
                }

                deserializer.deserialize_byte_buf(CustomByteBufVisitor)
            }
        }

        let mut deserializer = Deserializer::new(b"#a;".as_slice());

        assert_err_eq!(
            CustomByteBuf::deserialize(&mut deserializer),
            Error::new(error::Kind::Custom("foo".to_string()), Position::new(0, 1))
        );
    }

    #[test]
    fn none() {
        let mut deserializer = Deserializer::new(b"\n\n".as_slice());

        assert_ok_eq!(Option::<()>::deserialize(&mut deserializer), None);
    }

    #[test]
    fn some_integer() {
        let mut deserializer = Deserializer::new(b"#42;\n".as_slice());

        assert_ok_eq!(Option::<u64>::deserialize(&mut deserializer), Some(42));
    }

    #[test]
    fn unit() {
        let mut deserializer = Deserializer::new(b"#;".as_slice());

        assert_ok_eq!(<()>::deserialize(&mut deserializer), ());
    }

    #[test]
    fn unit_invalid() {
        let mut deserializer = Deserializer::new(b"#invalid;".as_slice());

        assert_err_eq!(
            <()>::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedUnit, Position::new(0, 1))
        );
    }

    #[test]
    fn unit_unexpected_tag() {
        let mut deserializer = Deserializer::new(b"#;\n#;".as_slice());

        assert_err_eq!(
            <()>::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedTag, Position::new(1, 0))
        );
    }

    #[test]
    fn unit_unexpected_values() {
        let mut deserializer = Deserializer::new(b"#;\n;".as_slice());

        assert_err_eq!(
            <()>::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedValues, Position::new(1, 0))
        );
    }

    #[test]
    fn unit_unexpected_value() {
        let mut deserializer = Deserializer::new(b"#:;\n".as_slice());

        assert_err_eq!(
            <()>::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedValue, Position::new(0, 2))
        );
    }

    #[test]
    fn unit_custom_error() {
        #[derive(Debug)]
        struct CustomUnit;

        impl<'de> Deserialize<'de> for CustomUnit {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct CustomUnitVisitor;

                impl<'de> Visitor<'de> for CustomUnitVisitor {
                    type Value = CustomUnit;

                    fn expecting(&self, _f: &mut fmt::Formatter) -> fmt::Result {
                        unimplemented!()
                    }

                    fn visit_unit<E>(self) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Err(de::Error::custom("foo"))
                    }
                }

                deserializer.deserialize_unit(CustomUnitVisitor)
            }
        }

        let mut deserializer = Deserializer::new(b"#;".as_slice());

        assert_err_eq!(
            CustomUnit::deserialize(&mut deserializer),
            Error::new(error::Kind::Custom("foo".to_string()), Position::new(0, 1))
        );
    }

    #[test]
    fn unit_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Unit;
        let mut deserializer = Deserializer::new(b"#;".as_slice());

        assert_ok_eq!(Unit::deserialize(&mut deserializer), Unit);
    }

    #[test]
    fn unit_struct_invalid() {
        #[derive(Debug, Deserialize)]
        struct Unit;
        let mut deserializer = Deserializer::new(b"#invalid;".as_slice());

        assert_err_eq!(
            Unit::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedUnit, Position::new(0, 1))
        );
    }

    #[test]
    fn unit_struct_unexpected_tag() {
        #[derive(Debug, Deserialize)]
        struct Unit;
        let mut deserializer = Deserializer::new(b"#;\n#;".as_slice());

        assert_err_eq!(
            Unit::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedTag, Position::new(1, 0))
        );
    }

    #[test]
    fn unit_struct_unexpected_values() {
        #[derive(Debug, Deserialize)]
        struct Unit;
        let mut deserializer = Deserializer::new(b"#;\n;".as_slice());

        assert_err_eq!(
            Unit::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedValues, Position::new(1, 0))
        );
    }

    #[test]
    fn unit_struct_unexpected_value() {
        #[derive(Debug, Deserialize)]
        struct Unit;
        let mut deserializer = Deserializer::new(b"#:;\n".as_slice());

        assert_err_eq!(
            Unit::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedValue, Position::new(0, 2))
        );
    }

    #[test]
    fn unit_struct_custom_error() {
        #[derive(Debug)]
        struct CustomUnitStruct;

        impl<'de> Deserialize<'de> for CustomUnitStruct {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct CustomUnitStructVisitor;

                impl<'de> Visitor<'de> for CustomUnitStructVisitor {
                    type Value = CustomUnitStruct;

                    fn expecting(&self, _f: &mut fmt::Formatter) -> fmt::Result {
                        unimplemented!()
                    }

                    fn visit_unit<E>(self) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Err(de::Error::custom("foo"))
                    }
                }

                deserializer.deserialize_unit_struct("CustomUnitStruct", CustomUnitStructVisitor)
            }
        }

        let mut deserializer = Deserializer::new(b"#;".as_slice());

        assert_err_eq!(
            CustomUnitStruct::deserialize(&mut deserializer),
            Error::new(error::Kind::Custom("foo".to_string()), Position::new(0, 1))
        );
    }

    #[test]
    fn newtype_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Newtype(u64);
        let mut deserializer = Deserializer::new(b"#42;\n".as_slice());

        assert_ok_eq!(Newtype::deserialize(&mut deserializer), Newtype(42));
    }

    #[test]
    fn seq() {
        let mut deserializer = Deserializer::new(b"#foo;#bar;#baz;".as_slice());

        assert_ok_eq!(
            Vec::<String>::deserialize(&mut deserializer),
            vec!["foo".to_owned(), "bar".to_owned(), "baz".to_owned()]
        );
    }

    #[test]
    fn tuple_single_value() {
        let mut deserializer = Deserializer::new(b"#foo;\n".as_slice());

        assert_ok_eq!(
            <(String,)>::deserialize(&mut deserializer),
            ("foo".to_owned(),)
        );
    }

    #[test]
    fn tuple_multiple_values() {
        let mut deserializer = Deserializer::new(b"#foo:42::1.2;\n".as_slice());

        assert_ok_eq!(
            <(String, u64, (), f64)>::deserialize(&mut deserializer),
            ("foo".to_owned(), 42, (), 1.2)
        );
    }

    #[test]
    fn tuple_nested_values() {
        let mut deserializer = Deserializer::new(b"#foo:42::1.2;\n".as_slice());

        assert_ok_eq!(
            <(String, (u64, ()), f64)>::deserialize(&mut deserializer),
            ("foo".to_owned(), (42, ()), 1.2)
        );
    }

    #[test]
    fn tuple_too_many_values() {
        let mut deserializer = Deserializer::new(b"#foo:42::1.2;\n".as_slice());

        assert_err_eq!(
            <(String, u64, ())>::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedValue, Position::new(0, 9))
        );
    }

    #[test]
    fn tuple_unexpected_values() {
        let mut deserializer = Deserializer::new(b"#foo:42::1.2;\nbar:100;\n".as_slice());

        assert_err_eq!(
            <(String, u64, (), f64)>::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedValues, Position::new(1, 0))
        );
    }

    #[test]
    fn tuple_unexpected_tag() {
        let mut deserializer = Deserializer::new(b"#foo:42::1.2;\n#bar:100;\n".as_slice());

        assert_err_eq!(
            <(String, u64, (), f64)>::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedTag, Position::new(1, 0))
        );
    }

    #[test]
    fn tuple_struct_single_value() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct TupleStruct(String);
        let mut deserializer = Deserializer::new(b"#foo;\n".as_slice());

        assert_ok_eq!(
            TupleStruct::deserialize(&mut deserializer),
            TupleStruct("foo".to_owned(),)
        );
    }

    #[test]
    fn tuple_struct_multiple_values() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct TupleStruct(String, u64, (), f64);
        let mut deserializer = Deserializer::new(b"#foo:42::1.2;\n".as_slice());

        assert_ok_eq!(
            TupleStruct::deserialize(&mut deserializer),
            TupleStruct("foo".to_owned(), 42, (), 1.2)
        );
    }

    #[test]
    fn tuple_struct_nested_values() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct NestedTupleStruct(u64, ());
        #[derive(Debug, Deserialize, PartialEq)]
        struct TupleStruct(String, NestedTupleStruct, f64);
        let mut deserializer = Deserializer::new(b"#foo:42::1.2;\n".as_slice());

        assert_ok_eq!(
            TupleStruct::deserialize(&mut deserializer),
            TupleStruct("foo".to_owned(), NestedTupleStruct(42, ()), 1.2)
        );
    }

    #[test]
    fn tuple_struct_too_many_values() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct TupleStruct(String, u64, ());
        let mut deserializer = Deserializer::new(b"#foo:42::1.2;\n".as_slice());

        assert_err_eq!(
            TupleStruct::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedValue, Position::new(0, 9))
        );
    }

    #[test]
    fn tuple_struct_unexpected_values() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct TupleStruct(String, u64, (), f64);
        let mut deserializer = Deserializer::new(b"#foo:42::1.2;\nbar:100;\n".as_slice());

        assert_err_eq!(
            TupleStruct::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedValues, Position::new(1, 0))
        );
    }

    #[test]
    fn tuple_struct_unexpected_tag() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct TupleStruct(String, u64, (), f64);
        let mut deserializer = Deserializer::new(b"#foo:42::1.2;\n#bar:100;\n".as_slice());

        assert_err_eq!(
            TupleStruct::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedTag, Position::new(1, 0))
        );
    }

    #[test]
    fn map() {
        let mut deserializer =
            Deserializer::new(b"#foo:1;\n#bar:2;\n#baz:3;\n#qux:4;\n".as_slice());

        let mut expected = HashMap::new();
        expected.insert("foo".to_owned(), 1);
        expected.insert("bar".to_owned(), 2);
        expected.insert("baz".to_owned(), 3);
        expected.insert("qux".to_owned(), 4);
        assert_ok_eq!(
            HashMap::<String, u64>::deserialize(&mut deserializer),
            expected
        );
    }

    #[test]
    fn r#struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Struct {
            foo: String,
            bar: u64,
            baz: (),
            qux: f64,
        }
        let mut deserializer =
            Deserializer::new(b"#foo:text;\n#bar:42;\n#baz:;\n#qux:1.2;\n".as_slice());

        assert_ok_eq!(
            Struct::deserialize(&mut deserializer),
            Struct {
                foo: "text".to_owned(),
                bar: 42,
                baz: (),
                qux: 1.2,
            }
        );
    }

    #[test]
    fn struct_optional_field_present() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Struct {
            foo: String,
            bar: Option<u64>,
            baz: (),
            qux: f64,
        }
        let mut deserializer =
            Deserializer::new(b"#foo:text;\n#bar:42;\n#baz:;\n#qux:1.2;\n".as_slice());

        assert_ok_eq!(
            Struct::deserialize(&mut deserializer),
            Struct {
                foo: "text".to_owned(),
                bar: Some(42),
                baz: (),
                qux: 1.2,
            }
        );
    }

    #[test]
    fn struct_optional_field_missing() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Struct {
            foo: String,
            bar: Option<u64>,
            baz: (),
            qux: f64,
        }
        let mut deserializer = Deserializer::new(b"#foo:text;\n#baz:;\n#qux:1.2;\n".as_slice());

        assert_ok_eq!(
            Struct::deserialize(&mut deserializer),
            Struct {
                foo: "text".to_owned(),
                bar: None,
                baz: (),
                qux: 1.2,
            }
        );
    }

    #[test]
    fn struct_containing_map() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Struct {
            foo: String,
            bar: u64,
            baz: (),
            qux: HashMap<String, u64>,
        }
        let mut deserializer =
            Deserializer::new(b"#foo:text;\n#bar:42;\n#baz:;\n#qux:a:1;b:2;c:3;d:4;\n".as_slice());

        let mut expected = HashMap::new();
        expected.insert("a".to_owned(), 1);
        expected.insert("b".to_owned(), 2);
        expected.insert("c".to_owned(), 3);
        expected.insert("d".to_owned(), 4);
        assert_ok_eq!(
            Struct::deserialize(&mut deserializer),
            Struct {
                foo: "text".to_owned(),
                bar: 42,
                baz: (),
                qux: expected,
            }
        );
    }

    #[test]
    fn struct_order_does_not_matter() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Struct {
            foo: String,
            bar: u64,
            baz: (),
            qux: f64,
        }
        let mut deserializer =
            Deserializer::new(b"#bar:42;\n#foo:text;\n#qux:1.2;\n#baz:;\n".as_slice());

        assert_ok_eq!(
            Struct::deserialize(&mut deserializer),
            Struct {
                foo: "text".to_owned(),
                bar: 42,
                baz: (),
                qux: 1.2,
            }
        );
    }

    #[test]
    fn enum_unit_variant() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Unit {
            Variant,
        }
        let mut deserializer = Deserializer::new(b"#Variant;\n".as_slice());

        assert_ok_eq!(Unit::deserialize(&mut deserializer), Unit::Variant);
    }

    #[test]
    fn enum_unit_variant_too_many_values() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Unit {
            Variant,
        }
        let mut deserializer = Deserializer::new(b"#Variant:42;\n".as_slice());

        assert_err_eq!(
            Unit::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedValue, Position::new(0, 9))
        );
    }

    #[test]
    fn enum_unit_variant_unexpected_values() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Unit {
            Variant,
        }
        let mut deserializer = Deserializer::new(b"#Variant;42;\n".as_slice());

        assert_err_eq!(
            Unit::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedValues, Position::new(0, 9))
        );
    }

    #[test]
    fn enum_unit_variant_unexpected_tag() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Unit {
            Variant,
        }
        let mut deserializer = Deserializer::new(b"#Variant;\n#42;\n".as_slice());

        assert_err_eq!(
            Unit::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedTag, Position::new(1, 0))
        );
    }

    #[test]
    fn enum_newtype_variant() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Newtype {
            Variant(u64),
        }
        let mut deserializer = Deserializer::new(b"#Variant:42;\n".as_slice());

        assert_ok_eq!(
            Newtype::deserialize(&mut deserializer),
            Newtype::Variant(42),
        );
    }

    #[test]
    fn enum_newtype_variant_too_many_values() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Newtype {
            Variant(u64),
        }
        let mut deserializer = Deserializer::new(b"#Variant:42:foo;\n".as_slice());

        assert_err_eq!(
            Newtype::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedValue, Position::new(0, 12))
        );
    }

    #[test]
    fn enum_newtype_variant_unexpected_values() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Newtype {
            Variant(u64),
        }
        let mut deserializer = Deserializer::new(b"#Variant:42;foo;\n".as_slice());

        assert_err_eq!(
            Newtype::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedValues, Position::new(0, 12))
        );
    }

    #[test]
    fn enum_newtype_variant_unexpected_tag() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Newtype {
            Variant(u64),
        }
        let mut deserializer = Deserializer::new(b"#Variant:42;\n#foo;\n".as_slice());

        assert_err_eq!(
            Newtype::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedTag, Position::new(1, 0))
        );
    }

    #[test]
    fn enum_tuple_variant() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Tuple {
            Variant(u64, String, (), f64),
        }
        let mut deserializer = Deserializer::new(b"#Variant:42:foo::1.2;\n".as_slice());

        assert_ok_eq!(
            Tuple::deserialize(&mut deserializer),
            Tuple::Variant(42, "foo".to_owned(), (), 1.2),
        );
    }

    #[test]
    fn enum_tuple_variant_too_many_values() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Tuple {
            Variant(u64, String, (), f64),
        }
        let mut deserializer = Deserializer::new(b"#Variant:42:foo::1.2:bar;\n".as_slice());

        assert_err_eq!(
            Tuple::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedValue, Position::new(0, 21))
        );
    }

    #[test]
    fn enum_tuple_variant_unexpected_values() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Tuple {
            Variant(u64, String, (), f64),
        }
        let mut deserializer = Deserializer::new(b"#Variant:42:foo::1.2;bar;\n".as_slice());

        assert_err_eq!(
            Tuple::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedValues, Position::new(0, 21))
        );
    }

    #[test]
    fn enum_tuple_variant_unexpected_tag() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Tuple {
            Variant(u64, String, (), f64),
        }
        let mut deserializer = Deserializer::new(b"#Variant:42:foo::1.2;\n#bar;\n".as_slice());

        assert_err_eq!(
            Tuple::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedTag, Position::new(1, 0))
        );
    }

    #[derive(Debug, PartialEq)]
    struct Identifier(String);

    impl<'de> Deserialize<'de> for Identifier {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            struct IdentifierVisitor;

            impl<'de> Visitor<'de> for IdentifierVisitor {
                type Value = Identifier;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("identifier")
                }

                fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(Identifier(value.to_owned()))
                }
            }

            deserializer.deserialize_identifier(IdentifierVisitor)
        }
    }

    #[test]
    fn identifier() {
        let mut deserializer = Deserializer::new(b"#foo;".as_slice());

        assert_ok_eq!(
            Identifier::deserialize(&mut deserializer),
            Identifier("foo".to_string())
        );
    }

    #[test]
    fn identifier_whitespace() {
        let mut deserializer = Deserializer::new(b"#   foo  ;".as_slice());

        assert_ok_eq!(
            Identifier::deserialize(&mut deserializer),
            Identifier("foo".to_string())
        );
    }

    #[test]
    fn identifier_unexpected_tag() {
        let mut deserializer = Deserializer::new(b"#foo;#bar;".as_slice());

        assert_err_eq!(
            String::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedTag, Position::new(0, 5))
        );
    }

    #[test]
    fn identifier_unexpected_values() {
        let mut deserializer = Deserializer::new(b"#foo;bar;".as_slice());

        assert_err_eq!(
            Identifier::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedValues, Position::new(0, 5))
        );
    }

    #[test]
    fn identifier_unexpected_value() {
        let mut deserializer = Deserializer::new(b"#foo:bar;\n".as_slice());

        assert_err_eq!(
            Identifier::deserialize(&mut deserializer),
            Error::new(error::Kind::UnexpectedValue, Position::new(0, 5))
        );
    }

    #[test]
    fn identifier_invalid() {
        let mut deserializer = Deserializer::new(b"#\xF0\x9Ffoo;\n".as_slice());

        assert_err_eq!(
            Identifier::deserialize(&mut deserializer),
            Error::new(error::Kind::ExpectedIdentifier, Position::new(0, 1)),
        );
    }

    #[test]
    fn identifier_custom_error() {
        #[derive(Debug)]
        struct CustomIdentifier;

        impl<'de> Deserialize<'de> for CustomIdentifier {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct CustomIdentifierVisitor;

                impl<'de> Visitor<'de> for CustomIdentifierVisitor {
                    type Value = CustomIdentifier;

                    fn expecting(&self, _f: &mut fmt::Formatter) -> fmt::Result {
                        unimplemented!()
                    }

                    fn visit_str<E>(self, _value: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Err(de::Error::custom("foo"))
                    }
                }

                deserializer.deserialize_identifier(CustomIdentifierVisitor)
            }
        }

        let mut deserializer = Deserializer::new(b"#a;".as_slice());

        assert_err_eq!(
            CustomIdentifier::deserialize(&mut deserializer),
            Error::new(error::Kind::Custom("foo".to_string()), Position::new(0, 1))
        );
    }
}
