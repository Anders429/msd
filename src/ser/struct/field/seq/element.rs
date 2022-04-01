use super::super::tuple;
use crate::ser::{r#struct, Error, Result, WriteExt};
use serde::{ser, ser::Impossible, Serialize};
use std::io::Write;

pub(super) struct Serializer<'a, W> {
    writer: &'a mut W,
}

impl<'a, W> Serializer<'a, W> {
    pub(super) fn new(writer: &'a mut W) -> Self {
        Self { writer }
    }
}

impl<'a, W> ser::Serializer for &'a mut Serializer<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = tuple::Serializer<'a, W>;
    type SerializeTupleStruct = tuple::Serializer<'a, W>;
    type SerializeTupleVariant = tuple::Serializer<'a, W>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = r#struct::Serializer<'a, W>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        if v {
            self.writer.write_parameter_unescaped(b"true")?;
        } else {
            self.writer.write_parameter_unescaped(b"false")?;
        }
        self.writer.close_tag()
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(v);
        self.writer.write_parameter_unescaped(s.as_bytes())?;

        self.writer.close_tag()
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(v);
        self.writer.write_parameter_unescaped(s.as_bytes())?;

        self.writer.close_tag()
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(v);
        self.writer.write_parameter_unescaped(s.as_bytes())?;

        self.writer.close_tag()
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(v);
        self.writer.write_parameter_unescaped(s.as_bytes())?;

        self.writer.close_tag()
    }

    #[cfg(has_i128)]
    fn serialize_i128(self, v: i128) -> Result<Self::Ok> {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(v);
        self.writer.write_parameter_unescaped(s.as_bytes())?;

        self.writer.close_tag()
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(v);
        self.writer.write_parameter_unescaped(s.as_bytes())?;

        self.writer.close_tag()
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(v);
        self.writer.write_parameter_unescaped(s.as_bytes())?;

        self.writer.close_tag()
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(v);
        self.writer.write_parameter_unescaped(s.as_bytes())?;

        self.writer.close_tag()
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(v);
        self.writer.write_parameter_unescaped(s.as_bytes())?;

        self.writer.close_tag()
    }

    #[cfg(has_u128)]
    fn serialize_u128(self, v: u128) -> Result<Self::Ok> {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(v);
        self.writer.write_parameter_unescaped(s.as_bytes())?;

        self.writer.close_tag()
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        let mut buffer = ryu::Buffer::new();
        let s = buffer.format(v);
        self.writer.write_parameter_unescaped(s.as_bytes())?;

        self.writer.close_tag()
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        let mut buffer = ryu::Buffer::new();
        let s = buffer.format(v);
        self.writer.write_parameter_unescaped(s.as_bytes())?;

        self.writer.close_tag()
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        let mut buffer = [0; 4];
        v.encode_utf8(&mut buffer);
        self.writer
            .write_parameter_escaped(&buffer[..v.len_utf8()])?;

        self.writer.close_tag()
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.writer.write_parameter_escaped(v.as_bytes())?;

        self.writer.close_tag()
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        self.writer.write_parameter_escaped(v)?;

        self.writer.close_tag()
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.writer.write_parameter_unescaped(b"")?;

        self.writer.close_tag()
    }

    fn serialize_some<T>(self, v: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        v.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        self.writer.write_parameter_unescaped(b"")?;

        self.writer.close_tag()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        self.writer.write_parameter_unescaped(b"")?;

        self.writer.close_tag()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.writer.write_parameter_escaped(variant.as_bytes())?;

        self.writer.close_tag()
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::UnsupportedType)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(tuple::Serializer::new(self.writer))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(tuple::Serializer::new(self.writer))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.writer.write_parameter_escaped(variant.as_bytes())?;
        Ok(tuple::Serializer::new(self.writer))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::UnsupportedType)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        self.writer.write_parameter_unescaped(b"")?;
        self.writer.close_tag()?;
        Ok(r#struct::Serializer::new(self.writer))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::UnsupportedType)
    }
}

#[cfg(test)]
mod tests {
    use super::Serializer;
    use claim::assert_ok;
    use serde::{ser::{SerializeTupleStruct, SerializeTupleVariant}, Serialize};
    use serde_bytes::Bytes;
    use serde_derive::Serialize;

    #[test]
    fn r#true() {
        let mut output = Vec::new();

        assert_ok!(true.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":true;\n");
    }

    #[test]
    fn r#false() {
        let mut output = Vec::new();

        assert_ok!(false.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":false;\n");
    }

    #[test]
    fn i8() {
        let mut output = Vec::new();

        assert_ok!(42i8.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":42;\n");
    }

    #[test]
    fn i16() {
        let mut output = Vec::new();

        assert_ok!(42i16.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":42;\n");
    }

    #[test]
    fn i32() {
        let mut output = Vec::new();

        assert_ok!(42i32.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":42;\n");
    }

    #[test]
    fn i64() {
        let mut output = Vec::new();

        assert_ok!(42i64.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":42;\n");
    }

    #[test]
    #[cfg_attr(not(has_i128), ignore)]
    fn i128() {
        let mut output = Vec::new();

        assert_ok!(42i128.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":42;\n");
    }

    #[test]
    fn i8_neg() {
        let mut output = Vec::new();

        assert_ok!((-42i8).serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":-42;\n");
    }

    #[test]
    fn i16_neg() {
        let mut output = Vec::new();

        assert_ok!((-42i16).serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":-42;\n");
    }

    #[test]
    fn i32_neg() {
        let mut output = Vec::new();

        assert_ok!((-42i32).serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":-42;\n");
    }

    #[test]
    fn i64_neg() {
        let mut output = Vec::new();

        assert_ok!((-42i64).serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":-42;\n");
    }

    #[test]
    #[cfg_attr(not(has_i128), ignore)]
    fn i128_neg() {
        let mut output = Vec::new();

        assert_ok!((-42i128).serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":-42;\n");
    }

    #[test]
    fn u8() {
        let mut output = Vec::new();

        assert_ok!(42u8.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":42;\n");
    }

    #[test]
    fn u16() {
        let mut output = Vec::new();

        assert_ok!(42u16.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":42;\n");
    }

    #[test]
    fn u32() {
        let mut output = Vec::new();

        assert_ok!(42u32.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":42;\n");
    }

    #[test]
    fn u64() {
        let mut output = Vec::new();

        assert_ok!(42u64.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":42;\n");
    }

    #[test]
    #[cfg_attr(not(has_i128), ignore)]
    fn u128() {
        let mut output = Vec::new();

        assert_ok!(42u128.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":42;\n");
    }

    #[test]
    fn char() {
        let mut output = Vec::new();

        assert_ok!('a'.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":a;\n");
    }

    #[test]
    fn char_escape_number_sign() {
        let mut output = Vec::new();

        assert_ok!('#'.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":\\#;\n");
    }

    #[test]
    fn char_escape_colon() {
        let mut output = Vec::new();

        assert_ok!(':'.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":\\:;\n");
    }

    #[test]
    fn char_escape_semicolon() {
        let mut output = Vec::new();

        assert_ok!(';'.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":\\;;\n");
    }

    #[test]
    fn char_escape_backslash() {
        let mut output = Vec::new();

        assert_ok!('\\'.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":\\\\;\n");
    }

    #[test]
    fn char_does_not_escape_forward_slash() {
        let mut output = Vec::new();

        assert_ok!('/'.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":/;\n");
    }

    #[test]
    fn str() {
        let mut output = Vec::new();

        assert_ok!("bar".serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":bar;\n");
    }

    #[test]
    fn str_escape_number_sign() {
        let mut output = Vec::new();

        assert_ok!("ba#r".serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":ba\\#r;\n");
    }

    #[test]
    fn str_escape_colon() {
        let mut output = Vec::new();

        assert_ok!("ba:r".serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":ba\\:r;\n");
    }

    #[test]
    fn str_escape_semicolon() {
        let mut output = Vec::new();

        assert_ok!("ba;r".serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":ba\\;r;\n");
    }

    #[test]
    fn str_escape_backslash() {
        let mut output = Vec::new();

        assert_ok!("ba\\r".serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":ba\\\\r;\n");
    }

    #[test]
    fn str_escape_double_forwardslash() {
        let mut output = Vec::new();

        assert_ok!("ba//r".serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":ba\\/\\/r;\n");
    }

    #[test]
    fn str_do_not_escape_single_forwardslash() {
        let mut output = Vec::new();

        assert_ok!("ba/r".serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":ba/r;\n");
    }

    #[test]
    fn bytes() {
        let mut output = Vec::new();

        assert_ok!(Bytes::new(b"bar").serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":bar;\n");
    }

    #[test]
    fn bytes_escape_number_sign() {
        let mut output = Vec::new();

        assert_ok!(Bytes::new(b"ba#r").serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":ba\\#r;\n");
    }

    #[test]
    fn bytes_escape_colon() {
        let mut output = Vec::new();

        assert_ok!(Bytes::new(b"ba:r").serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":ba\\:r;\n");
    }

    #[test]
    fn bytes_escape_semicolon() {
        let mut output = Vec::new();

        assert_ok!(Bytes::new(b"ba;r").serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":ba\\;r;\n");
    }

    #[test]
    fn bytes_escape_backslash() {
        let mut output = Vec::new();

        assert_ok!(Bytes::new(b"ba\\r").serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":ba\\\\r;\n");
    }

    #[test]
    fn bytes_escape_double_forwardslash() {
        let mut output = Vec::new();

        assert_ok!(Bytes::new(b"ba//r").serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":ba\\/\\/r;\n");
    }

    #[test]
    fn bytes_do_not_escape_single_forwardslash() {
        let mut output = Vec::new();

        assert_ok!(Bytes::new(b"ba/r").serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":ba/r;\n");
    }

    #[test]
    fn none() {
        let mut output = Vec::new();

        assert_ok!(Option::<()>::None.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":;\n");
    }

    #[test]
    fn some() {
        let mut output = Vec::new();

        assert_ok!(Some(42).serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":42;\n");
    }

    #[test]
    fn unit() {
        let mut output = Vec::new();

        assert_ok!(().serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":;\n");
    }

    #[test]
    fn unit_struct() {
        #[derive(Serialize)]
        struct Bar;

        let mut output = Vec::new();

        assert_ok!(Bar.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":;\n");
    }

    #[test]
    fn unit_variant() {
        #[derive(Serialize)]
        enum Enum {
            A,
        }

        let mut output = Vec::new();

        assert_ok!(Enum::A.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":A;\n");
    }

    #[test]
    fn newtype_struct() {
        #[derive(Serialize)]
        struct NewtypeStruct(u32);

        let mut output = Vec::new();

        assert_ok!(NewtypeStruct(42).serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":42;\n");
    }

    #[test]
    fn empty_tuple() {
        let mut output = Vec::new();

        assert_ok!(<[(); 0]>::serialize(&[], &mut Serializer::new(&mut output)));

        assert_eq!(output, b";\n");
    }

    #[test]
    fn single_element_tuple() {
        let mut output = Vec::new();

        assert_ok!((42).serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":42;\n");
    }

    #[test]
    fn multiple_element_tuple() {
        let mut output = Vec::new();

        assert_ok!((42, "bar", (), 1.0).serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":42:bar::1.0;\n");
    }

    #[test]
    fn empty_tuple_struct() {
        #[derive(Serialize)]
        struct TupleStruct();

        let mut output = Vec::new();

        assert_ok!(TupleStruct().serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b";\n");
    }

    #[test]
    fn single_element_tuple_struct() {
        struct TupleStruct(usize);
        impl Serialize for TupleStruct {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let mut ts = serializer.serialize_tuple_struct("TupleStruct", 1)?;
                ts.serialize_field(&self.0)?;
                ts.end()
            }
        }

        let mut output = Vec::new();

        assert_ok!(TupleStruct(42).serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":42;\n");
    }

    #[test]
    fn multiple_element_tuple_struct() {
        #[derive(Serialize)]
        struct TupleStruct(usize, &'static str, (), f32);

        let mut output = Vec::new();

        assert_ok!(TupleStruct(42, "bar", (), 1.0).serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":42:bar::1.0;\n");
    }

    #[test]
    fn empty_tuple_variant() {
        enum TupleEnum {
            Variant()
        }
        impl Serialize for TupleEnum {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_tuple_variant("TupleEnum", 0, "Variant", 0)?.end()
            }
        }

        let mut output = Vec::new();

        assert_ok!(TupleEnum::Variant().serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":Variant;\n");
    }

    #[test]
    fn single_element_tuple_variant() {
        enum TupleEnum {
            Variant(usize),
        }
        impl Serialize for TupleEnum {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                if let Self::Variant(inner) = self {
                    let mut tv = serializer.serialize_tuple_variant("TupleEnum", 0, "Variant", 1)?;
                    tv.serialize_field(&inner)?;
                    tv.end()
                } else {
                    unreachable!("there is only one variant")
                }
            }
        }

        let mut output = Vec::new();

        assert_ok!(TupleEnum::Variant(42).serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":Variant:42;\n");
    }

    #[test]
    fn multiple_element_tuple_variant() {
        #[derive(Serialize)]
        enum TupleEnum {
            Variant(usize, &'static str, (), f32),
        }

        let mut output = Vec::new();

        assert_ok!(
            TupleEnum::Variant(42, "bar", (), 1.0).serialize(&mut Serializer::new(&mut output))
        );

        assert_eq!(output, b":Variant:42:bar::1.0;\n");
    }

    #[test]
    fn nested_tuple_variant() {
        #[derive(Serialize)]
        enum TupleEnum  {
            Variant(usize, (usize, usize), ((usize, usize), usize), usize)
        }

        let mut output = Vec::new();

        assert_ok!(TupleEnum::Variant(1, (2, 3), ((4, 5), 6), 7).serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":Variant:1:2:3:4:5:6:7;\n");
    }

    #[test]
    fn r#struct() {
        #[derive(Serialize)]
        struct Struct {
            foo: usize,
            bar: &'static str,
            baz: (),
            qux: Option<f32>,
        }

        let mut output = Vec::new();

        assert_ok!(Struct {
            foo: 42,
            bar: "test",
            baz: (),
            qux: None,
        }
        .serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b":;\n#foo:42;\n#bar:test;\n#baz:;\n");
    }
}
