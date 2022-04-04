mod error;
mod escaped;
mod map;
mod seq;
mod r#struct;
mod tuple;
mod write;

pub use error::{Error, Result};

use escaped::Escaped;
use serde::{ser, ser::Impossible, Serialize};
use std::io::Write;
use write::WriteExt;

pub struct Serializer<W> {
    writer: W,
}

impl<W> Serializer<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl<'a, W> ser::Serializer for &'a mut Serializer<W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = r#struct::Serializer<'a, W>;
    type SerializeStructVariant = r#struct::Serializer<'a, W>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        if v {
            self.writer.write_tag_name_unescaped(b"true")?;
        } else {
            self.writer.write_tag_name_unescaped(b"false")?;
        }
        self.writer.close_tag()
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(v);
        self.writer.write_tag_name_unescaped(s.as_bytes())?;
        self.writer.close_tag()
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(v);
        self.writer.write_tag_name_unescaped(s.as_bytes())?;
        self.writer.close_tag()
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(v);
        self.writer.write_tag_name_unescaped(s.as_bytes())?;
        self.writer.close_tag()
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(v);
        self.writer.write_tag_name_unescaped(s.as_bytes())?;
        self.writer.close_tag()
    }

    #[cfg(has_i128)]
    fn serialize_i128(self, v: i128) -> Result<Self::Ok> {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(v);
        self.writer.write_tag_name_unescaped(s.as_bytes())?;
        self.writer.close_tag()
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(v);
        self.writer.write_tag_name_unescaped(s.as_bytes())?;
        self.writer.close_tag()
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(v);
        self.writer.write_tag_name_unescaped(s.as_bytes())?;
        self.writer.close_tag()
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(v);
        self.writer.write_tag_name_unescaped(s.as_bytes())?;
        self.writer.close_tag()
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(v);
        self.writer.write_tag_name_unescaped(s.as_bytes())?;
        self.writer.close_tag()
    }

    #[cfg(has_u128)]
    fn serialize_u128(self, v: u128) -> Result<Self::Ok> {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(v);
        self.writer.write_tag_name_unescaped(s.as_bytes())?;
        self.writer.close_tag()
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        let mut buffer = ryu::Buffer::new();
        let s = buffer.format(v);
        self.writer.write_tag_name_unescaped(s.as_bytes())?;
        self.writer.close_tag()
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        let mut buffer = ryu::Buffer::new();
        let s = buffer.format(v);
        self.writer.write_tag_name_unescaped(s.as_bytes())?;
        self.writer.close_tag()
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        let mut buffer = [0; 4];
        v.encode_utf8(&mut buffer);
        self.writer
            .write_tag_name_escaped(&buffer[..v.len_utf8()])?;
        self.writer.close_tag()
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    fn serialize_some<T>(self, _v: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::UnsupportedType)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::UnsupportedType)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::UnsupportedType)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::UnsupportedType)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::UnsupportedType)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(r#struct::Serializer::new(&mut self.writer))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.writer.write_tag_name_escaped(variant.as_bytes())?;
        self.writer.close_tag()?;
        Ok(r#struct::Serializer::new(&mut self.writer))
    }
}

#[cfg(test)]
mod tests {
    use super::Serializer;
    use claim::assert_ok;
    use serde::Serialize;
    use serde_derive::Serialize;

    #[test]
    fn r#true() {
        let mut output = Vec::new();

        assert_ok!(true.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b"#true;\n");
    }

    #[test]
    fn r#false() {
        let mut output = Vec::new();

        assert_ok!(false.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b"#false;\n");
    }

    #[test]
    fn i8() {
        let mut output = Vec::new();

        assert_ok!(42i8.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b"#42;\n");
    }

    #[test]
    fn i16() {
        let mut output = Vec::new();

        assert_ok!(42i16.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b"#42;\n");
    }

    #[test]
    fn i32() {
        let mut output = Vec::new();

        assert_ok!(42i32.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b"#42;\n");
    }

    #[test]
    fn i64() {
        let mut output = Vec::new();

        assert_ok!(42i64.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b"#42;\n");
    }

    #[test]
    #[cfg_attr(not(has_i128), ignore)]
    fn i128() {
        let mut output = Vec::new();

        assert_ok!(42i128.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b"#42;\n");
    }

    #[test]
    fn i8_neg() {
        let mut output = Vec::new();

        assert_ok!((-42i8).serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b"#-42;\n");
    }

    #[test]
    fn i16_neg() {
        let mut output = Vec::new();

        assert_ok!((-42i16).serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b"#-42;\n");
    }

    #[test]
    fn i32_neg() {
        let mut output = Vec::new();

        assert_ok!((-42i32).serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b"#-42;\n");
    }

    #[test]
    fn i64_neg() {
        let mut output = Vec::new();

        assert_ok!((-42i64).serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b"#-42;\n");
    }

    #[test]
    #[cfg_attr(not(has_i128), ignore)]
    fn i128_neg() {
        let mut output = Vec::new();

        assert_ok!((-42i128).serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b"#-42;\n");
    }

    #[test]
    fn u8() {
        let mut output = Vec::new();

        assert_ok!(42u8.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b"#42;\n");
    }

    #[test]
    fn u16() {
        let mut output = Vec::new();

        assert_ok!(42u16.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b"#42;\n");
    }

    #[test]
    fn u32() {
        let mut output = Vec::new();

        assert_ok!(42u32.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b"#42;\n");
    }

    #[test]
    fn u64() {
        let mut output = Vec::new();

        assert_ok!(42u64.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b"#42;\n");
    }

    #[test]
    #[cfg_attr(not(has_i128), ignore)]
    fn u128() {
        let mut output = Vec::new();

        assert_ok!(42u128.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b"#42;\n");
    }

    #[test]
    fn f32() {
        let mut output = Vec::new();

        assert_ok!(42f32.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b"#42.0;\n");
    }

    #[test]
    fn f64() {
        let mut output = Vec::new();

        assert_ok!(42f64.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b"#42.0;\n");
    }

    #[test]
    fn char() {
        let mut output = Vec::new();

        assert_ok!('a'.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b"#a;\n");
    }

    #[test]
    fn char_escape_number_sign() {
        let mut output = Vec::new();

        assert_ok!('#'.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b"#\\#;\n");
    }

    #[test]
    fn char_escape_colon() {
        let mut output = Vec::new();

        assert_ok!(':'.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b"#\\:;\n");
    }

    #[test]
    fn char_escape_semicolon() {
        let mut output = Vec::new();

        assert_ok!(';'.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b"#\\;;\n");
    }

    #[test]
    fn char_escape_backslash() {
        let mut output = Vec::new();

        assert_ok!('\\'.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b"#\\\\;\n");
    }

    #[test]
    fn char_does_not_escape_forward_slash() {
        let mut output = Vec::new();

        assert_ok!('/'.serialize(&mut Serializer::new(&mut output)));

        assert_eq!(output, b"#/;\n");
    }

    #[test]
    fn newtype_struct() {
        #[derive(Serialize)]
        struct Foo {
            bar: usize,
            baz: Option<()>,
            qux: Option<&'static str>,
        }
        #[derive(Serialize)]
        struct Newtype(Foo);
        let mut output = Vec::new();

        assert_ok!(Newtype(Foo {
            bar: 42,
            baz: None,
            qux: Some("test:test"),
        })
        .serialize(&mut Serializer::new(&mut output)));
        assert_eq!(output, b"#bar:42;\n#qux:test\\:test;\n");
    }

    #[test]
    fn struct_empty() {
        #[derive(Serialize)]
        struct Foo {}
        let mut output = Vec::new();

        assert_ok!(Foo {}.serialize(&mut Serializer::new(&mut output)));
        assert_eq!(output, b"");
    }

    #[test]
    fn struct_single_field() {
        #[derive(Serialize)]
        struct Foo {
            bar: usize,
        }
        let mut output = Vec::new();

        assert_ok!(Foo { bar: 42 }.serialize(&mut Serializer::new(&mut output)));
        assert_eq!(output, b"#bar:42;\n");
    }

    #[test]
    fn struct_multiple_fields() {
        #[derive(Serialize)]
        struct Foo {
            bar: usize,
            baz: Option<()>,
            qux: Option<&'static str>,
        }
        let mut output = Vec::new();

        assert_ok!(Foo {
            bar: 42,
            baz: None,
            qux: Some("test:test"),
        }
        .serialize(&mut Serializer::new(&mut output)));
        assert_eq!(output, b"#bar:42;\n#qux:test\\:test;\n");
    }

    #[test]
    fn struct_variant_empty() {
        #[derive(Serialize)]
        enum Foo {
            Variant {},
        }
        let mut output = Vec::new();

        assert_ok!(Foo::Variant {}.serialize(&mut Serializer::new(&mut output)));
        assert_eq!(output, b"#Variant;\n");
    }

    #[test]
    fn struct_variant_single_field() {
        #[derive(Serialize)]
        enum Foo {
            Variant { bar: usize },
        }
        let mut output = Vec::new();

        assert_ok!(Foo::Variant { bar: 42 }.serialize(&mut Serializer::new(&mut output)));
        assert_eq!(output, b"#Variant;\n#bar:42;\n");
    }

    #[test]
    fn struct_variant_multiple_fields() {
        #[derive(Serialize)]
        enum Foo {
            Variant {
                bar: usize,
                baz: Option<()>,
                qux: Option<&'static str>,
            },
        }
        let mut output = Vec::new();

        assert_ok!(Foo::Variant {
            bar: 42,
            baz: None,
            qux: Some("test:test"),
        }
        .serialize(&mut Serializer::new(&mut output)));
        assert_eq!(output, b"#Variant;\n#bar:42;\n#qux:test\\:test;\n");
    }
}
