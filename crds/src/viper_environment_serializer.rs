use serde::{ser, Serialize};
use std::fmt::{Display, Formatter};
use std::iter;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Byte array is not supported")]
    ByteArrayNotSupported,

    #[error("Serializer is in invalid stack state. Expected {0:?}, got {1:?}")]
    InvalidStack(
        ViperEnvironmentSerializerStackElement,
        Vec<Option<ViperEnvironmentSerializerStackElement>>,
    ),

    #[error("Failed to serialize value: {0}")]
    SerializeError(String),
}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::SerializeError(msg.to_string())
    }
}

#[derive(Debug, Default)]
pub struct ViperEnvironmentSerializer {
    pub values: Vec<String>,
    prefix: Option<String>,
    field_stack: Vec<ViperEnvironmentSerializerStackElement>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ViperEnvironmentSerializerStackElement {
    String(String),
    Index(usize),
}

impl From<String> for ViperEnvironmentSerializerStackElement {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl<'a> From<&'a str> for ViperEnvironmentSerializerStackElement {
    fn from(s: &'a str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<usize> for ViperEnvironmentSerializerStackElement {
    fn from(s: usize) -> Self {
        Self::Index(s)
    }
}

impl Display for ViperEnvironmentSerializerStackElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ViperEnvironmentSerializerStackElement::String(s) => write!(f, "{}", s),
            ViperEnvironmentSerializerStackElement::Index(i) => write!(f, "{}", i),
        }
    }
}

fn build_field_stack_state_error(
    field_stack: &[ViperEnvironmentSerializerStackElement],
    tail: Option<ViperEnvironmentSerializerStackElement>,
    expected: ViperEnvironmentSerializerStackElement,
) -> Error {
    let ve = field_stack
        .iter()
        .map(Some)
        .chain(iter::once(tail.as_ref()))
        .map(|i| i.cloned())
        .collect();

    Error::InvalidStack(expected, ve)
}

impl ViperEnvironmentSerializer {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            prefix: None,
            field_stack: vec![],
        }
    }

    pub fn new_with_prefix(prefix: String) -> Self {
        Self {
            values: Vec::new(),
            prefix: Some(prefix),
            field_stack: vec![],
        }
    }

    fn get_key(&self) -> String {
        let mut key = String::new();

        for (i, element) in self.field_stack.iter().enumerate() {
            if i > 0 {
                key.push_str("__");
            }

            key.push_str(&element.to_string());
        }

        if let Some(prefix) = &self.prefix {
            format!("{}__{}", prefix, key)
        } else {
            key
        }
    }

    fn add_value(&mut self, v: impl Display) {
        let key = self.get_key();

        let pair = format!("{key}={v}");

        self.values.push(pair);
    }
}

impl<'a> ser::Serializer for &'a mut ViperEnvironmentSerializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.add_value(v);
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.add_value(v);
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.add_value(v);
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.add_value(v);
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.add_value(v);
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.add_value(v);
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.add_value(v);
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.add_value(v);
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.add_value(v);
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.add_value(v);
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.add_value(v);
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.add_value(v);
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.add_value(v);
        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Error::ByteArrayNotSupported)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.add_value(variant);
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.field_stack
            .push(ViperEnvironmentSerializerStackElement::Index(0));
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        todo!()
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        todo!()
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        todo!()
    }
}

impl<'a> ser::SerializeSeq for &'a mut ViperEnvironmentSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)?;
        let tail = self.field_stack.pop();
        if let Some(ViperEnvironmentSerializerStackElement::Index(idx)) = tail {
            self.field_stack
                .push(ViperEnvironmentSerializerStackElement::Index(idx + 1));
            Ok(())
        } else {
            Err(build_field_stack_state_error(
                &self.field_stack,
                tail,
                ViperEnvironmentSerializerStackElement::Index(0),
            ))
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let tail = self.field_stack.pop();
        if let Some(ViperEnvironmentSerializerStackElement::Index(_)) = tail {
            Ok(())
        } else {
            Err(build_field_stack_state_error(
                &self.field_stack,
                tail,
                ViperEnvironmentSerializerStackElement::Index(0),
            ))
        }
    }
}

impl<'a> ser::SerializeTuple for &'a mut ViperEnvironmentSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut ViperEnvironmentSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut ViperEnvironmentSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a> ser::SerializeMap for &'a mut ViperEnvironmentSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, _key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_value<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_entry<K: ?Sized, V: ?Sized>(
        &mut self,
        _key: &K,
        _value: &V,
    ) -> Result<(), Self::Error>
    where
        K: Serialize,
        V: Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a> ser::SerializeStruct for &'a mut ViperEnvironmentSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.field_stack
            .push(ViperEnvironmentSerializerStackElement::String(
                key.to_uppercase(),
            ));
        value.serialize(&mut **self)?;
        let popped = self.field_stack.pop();

        if popped
            != Some(ViperEnvironmentSerializerStackElement::String(
                key.to_uppercase(),
            ))
        {
            Err(build_field_stack_state_error(
                &self.field_stack,
                popped,
                ViperEnvironmentSerializerStackElement::String(key.to_uppercase()),
            ))
        } else {
            Ok(())
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut ViperEnvironmentSerializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        _value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::f32::consts::PI;

    #[test]
    fn simple_values() {
        #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct SomeObject {
            pub name: String,
            pub age: u32,
            pub is_active: bool,
            pub optional: Option<String>,
            pub optional2: Option<String>,
            pub an_enum: AnEnum,
        }

        #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub enum AnEnum {
            #[default]
            First,
            Second,
            Third,
        }

        let mut serializer = ViperEnvironmentSerializer::new();
        let value = SomeObject {
            name: "John".to_string(),
            age: 32,
            is_active: true,
            optional: Some("a value".to_string()),
            optional2: None,
            an_enum: AnEnum::Second,
        };
        value.serialize(&mut serializer).unwrap();
        assert_eq!(
            serializer.values,
            vec![
                "NAME=John".to_string(),
                "AGE=32".to_string(),
                "IS_ACTIVE=true".to_string(),
                "OPTIONAL=a value".to_string(),
                "AN_ENUM=Second".to_string(),
            ]
        );
    }

    #[test]
    fn string_list() {
        #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct SomeObject {
            pub string_list: Vec<String>,
        }

        let mut serializer = ViperEnvironmentSerializer::new();
        let value = SomeObject {
            string_list: vec!["a".to_string(), "b".to_string()],
        };
        value.serialize(&mut serializer).unwrap();
        assert_eq!(
            serializer.values,
            vec![
                "STRING_LIST__0=a".to_string(),
                "STRING_LIST__1=b".to_string(),
            ]
        );
    }

    #[test]
    fn nested_object() {
        #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct SomeObject {
            pub nested: NestedObject,
        }

        #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct NestedObject {
            pub something: i64,
        }

        let mut serializer = ViperEnvironmentSerializer::new();
        let value = SomeObject {
            nested: NestedObject { something: 42 },
        };
        value.serialize(&mut serializer).unwrap();
        assert_eq!(serializer.values, vec!["NESTED__SOMETHING=42".to_string(),]);
    }

    #[test]
    fn nested_list_objects() {
        #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct SomeObject {
            pub nested_list: Vec<NestedListObject>,
        }

        #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct NestedListObject {
            pub something: i64,
        }

        let mut serializer = ViperEnvironmentSerializer::new();
        let value = SomeObject {
            nested_list: vec![
                NestedListObject { something: 1 },
                NestedListObject { something: 2 },
            ],
        };
        value.serialize(&mut serializer).unwrap();
        assert_eq!(
            serializer.values,
            vec![
                "NESTED_LIST__0__SOMETHING=1".to_string(),
                "NESTED_LIST__1__SOMETHING=2".to_string(),
            ]
        );
    }

    #[test]
    fn all_together() {
        #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct SomeObject {
            pub name: String,
            pub age: u32,
            pub is_active: bool,
            pub optional: Option<String>,
            pub optional2: Option<String>,
            pub an_enum: AnEnum,
            pub string_list: Vec<String>,
            pub nested: NestedObject,
            pub nested_list: Vec<NestedListObject>,
            pub something_else: Option<f32>,
        }

        #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub enum AnEnum {
            #[default]
            First,
            Second,
            Third,
        }

        #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct NestedObject {
            pub something: i64,
        }

        #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct NestedListObject {
            pub something: i64,
        }

        let mut serializer = ViperEnvironmentSerializer::new();
        let value = SomeObject {
            name: "John".to_string(),
            age: 32,
            is_active: true,
            optional: Some("a value".to_string()),
            optional2: None,
            an_enum: AnEnum::Second,
            string_list: vec!["a".to_string(), "b".to_string()],
            nested: NestedObject { something: 42 },
            nested_list: vec![
                NestedListObject { something: 1 },
                NestedListObject { something: 2 },
            ],
            something_else: Some(PI),
        };
        value.serialize(&mut serializer).unwrap();
        assert_eq!(
            serializer.values,
            vec![
                "NAME=John".to_string(),
                "AGE=32".to_string(),
                "IS_ACTIVE=true".to_string(),
                "OPTIONAL=a value".to_string(),
                "AN_ENUM=Second".to_string(),
                "STRING_LIST__0=a".to_string(),
                "STRING_LIST__1=b".to_string(),
                "NESTED__SOMETHING=42".to_string(),
                "NESTED_LIST__0__SOMETHING=1".to_string(),
                "NESTED_LIST__1__SOMETHING=2".to_string(),
                "SOMETHING_ELSE=3.1415927".to_string(),
            ]
        );
    }
}
