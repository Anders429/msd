pub(super) mod element;
pub(super) mod key;
pub(super) mod nested;
pub(super) mod tag;

use crate::ser::{Error, Result, WriteExt};
use serde::{
    ser::{SerializeTuple, SerializeTupleStruct, SerializeTupleVariant},
    Serialize,
};
use std::io::Write;

pub struct Serializer<'a, W> {
    writer: &'a mut W,
}

impl<'a, W> Serializer<'a, W> {
    pub(super) fn new(writer: &'a mut W) -> Self {
        Self { writer }
    }
}

impl<'a, W> SerializeTuple for Serializer<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(element::Serializer::new(self.writer))
    }

    fn end(self) -> Result<Self::Ok> {
        self.writer.close_tag()
    }
}

impl<'a, W> SerializeTupleStruct for Serializer<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(element::Serializer::new(self.writer))
    }

    fn end(self) -> Result<Self::Ok> {
        self.writer.close_tag()
    }
}

impl<'a, W> SerializeTupleVariant for Serializer<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(element::Serializer::new(self.writer))
    }

    fn end(self) -> Result<Self::Ok> {
        self.writer.close_tag()
    }
}

#[cfg(test)]
mod tests {
    use super::Serializer;
    use claims::assert_ok;

    #[test]
    fn serialize_tuple_empty() {
        use serde::ser::SerializeTuple;

        let mut output = Vec::new();

        let serializer = Serializer::new(&mut output);

        assert_ok!(serializer.end());
        assert_eq!(output, b";\n");
    }

    #[test]
    fn serialize_tuple_single() {
        use serde::ser::SerializeTuple;

        let mut output = Vec::new();

        let mut serializer = Serializer::new(&mut output);

        assert_ok!(serializer.serialize_element(&42));
        assert_ok!(serializer.end());
        assert_eq!(output, b":42;\n");
    }

    #[test]
    fn serialize_tuple_multiple() {
        use serde::ser::SerializeTuple;

        let mut output = Vec::new();

        let mut serializer = Serializer::new(&mut output);

        assert_ok!(serializer.serialize_element(&42));
        assert_ok!(serializer.serialize_element(&"foo"));
        assert_ok!(serializer.serialize_element(&()));
        assert_ok!(serializer.serialize_element(&1.0));
        assert_ok!(serializer.end());
        assert_eq!(output, b":42:foo:1.0;\n");
    }

    #[test]
    fn serialize_tuple_struct_empty() {
        use serde::ser::SerializeTupleStruct;

        let mut output = Vec::new();

        let serializer = Serializer::new(&mut output);

        assert_ok!(serializer.end());
        assert_eq!(output, b";\n");
    }

    #[test]
    fn serialize_tuple_struct_single() {
        use serde::ser::SerializeTupleStruct;

        let mut output = Vec::new();

        let mut serializer = Serializer::new(&mut output);

        assert_ok!(serializer.serialize_field(&42));
        assert_ok!(serializer.end());
        assert_eq!(output, b":42;\n");
    }

    #[test]
    fn serialize_tuple_struct_multiple() {
        use serde::ser::SerializeTupleStruct;

        let mut output = Vec::new();

        let mut serializer = Serializer::new(&mut output);

        assert_ok!(serializer.serialize_field(&42));
        assert_ok!(serializer.serialize_field(&"foo"));
        assert_ok!(serializer.serialize_field(&()));
        assert_ok!(serializer.serialize_field(&1.0));
        assert_ok!(serializer.end());
        assert_eq!(output, b":42:foo:1.0;\n");
    }

    #[test]
    fn serialize_tuple_variant_empty() {
        use serde::ser::SerializeTupleVariant;

        let mut output = Vec::new();

        let serializer = Serializer::new(&mut output);

        assert_ok!(serializer.end());
        assert_eq!(output, b";\n");
    }

    #[test]
    fn serialize_tuple_variant_single() {
        use serde::ser::SerializeTupleVariant;

        let mut output = Vec::new();

        let mut serializer = Serializer::new(&mut output);

        assert_ok!(serializer.serialize_field(&42));
        assert_ok!(serializer.end());
        assert_eq!(output, b":42;\n");
    }

    #[test]
    fn serialize_tuple_variant_multiple() {
        use serde::ser::SerializeTupleVariant;

        let mut output = Vec::new();

        let mut serializer = Serializer::new(&mut output);

        assert_ok!(serializer.serialize_field(&42));
        assert_ok!(serializer.serialize_field(&"foo"));
        assert_ok!(serializer.serialize_field(&()));
        assert_ok!(serializer.serialize_field(&1.0));
        assert_ok!(serializer.end());
        assert_eq!(output, b":42:foo:1.0;\n");
    }
}
