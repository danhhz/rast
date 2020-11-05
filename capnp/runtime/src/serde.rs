// Copyright 2020 Daniel Harrison. All Rights Reserved.

//! [Serde] serializers for Cap'n Proto types
//!
//! [serde]: https://serde.rs

use serde::ser::{SerializeSeq, SerializeStruct};
use serde::{Serialize, Serializer};

use crate::element::{
  DataElement, Element, EnumElement, ListDecodedElement, ListElement, StructElement, TextElement,
  UnionElement,
};
use crate::field_meta::FieldMeta;

impl<'a> Serialize for Element<'a> {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    match self {
      Element::Bool(x) => serializer.serialize_bool(*x),
      Element::I32(x) => serializer.serialize_i32(*x),
      Element::U8(x) => serializer.serialize_u8(*x),
      Element::U16(x) => serializer.serialize_u16(*x),
      Element::U32(x) => serializer.serialize_u32(*x),
      Element::U64(x) => serializer.serialize_u64(*x),
      Element::F32(x) => serializer.serialize_f32(*x),
      Element::F64(x) => serializer.serialize_f64(*x),
      Element::Data(x) => x.serialize(serializer),
      Element::Text(x) => x.serialize(serializer),
      Element::Enum(x) => x.serialize(serializer),
      Element::Struct(x) => x.serialize(serializer),
      Element::List(x) => x.serialize(serializer),
      Element::ListDecoded(x) => x.serialize(serializer),
      Element::Union(x) => x.serialize(serializer),
    }
  }
}

impl<'a> Serialize for DataElement<'a> {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    let DataElement(value) = self;
    serializer.serialize_bytes(value)
  }
}

impl<'a> Serialize for TextElement<'a> {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    let TextElement(value) = self;
    serializer.serialize_str(value)
  }
}

impl Serialize for EnumElement {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    let EnumElement(meta, discriminant) = self;
    let enumerant_meta = meta.get(*discriminant).ok_or_else(|| todo!())?;
    serializer.serialize_str(enumerant_meta.name)
  }
}

impl<'a> Serialize for StructElement<'a> {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    let StructElement(meta, untyped) = self;
    let fields = meta.fields();
    let mut state = serializer.serialize_struct(meta.name, fields.len())?;
    for field in fields {
      if field.is_null(untyped) {
        continue;
      }
      match field {
        FieldMeta::U64(x) => state.serialize_field(field.name(), &x.get_element(untyped))?,
        _ => match field.get_element(untyped) {
          Err(_) => todo!(),
          Ok(x) => state.serialize_field(field.name(), &x)?,
        },
      }
    }
    state.end()
  }
}

impl<'a> Serialize for ListElement<'a> {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    match self.to_element_list() {
      Err(_) => todo!(),
      Ok(x) => {
        let mut seq = serializer.serialize_seq(Some(x.len()))?;
        for e in x {
          seq.serialize_element(&e)?;
        }
        seq.end()
      }
    }
  }
}

impl<'a> Serialize for ListDecodedElement<'a> {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    let ListDecodedElement(_, values) = self;
    let mut seq = serializer.serialize_seq(Some(values.len()))?;
    for e in values {
      seq.serialize_element(&e)?;
    }
    seq.end()
  }
}

impl<'a> Serialize for UnionElement<'a> {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    let UnionElement(meta, discriminant, value) = self;
    let variant_meta = meta.get(*discriminant).ok_or_else(|| todo!())?;
    serializer.serialize_newtype_variant(
      meta.name,
      u32::from(variant_meta.discriminant.0),
      variant_meta.field_meta.name(),
      value.as_ref(),
    )
  }
}

#[cfg(test)]
mod test {
  use serde_json;
  use std::error;

  use crate::samples::test_capnp::TestAllTypesRef;
  use capnp_runtime::prelude::*;
  use capnp_runtime::segment_framing_official;

  #[test]
  fn serialize_json() -> Result<(), Box<dyn error::Error>> {
    let buf = include_bytes!("../testdata/binary");
    let message: TestAllTypesRef = segment_framing_official::decode(buf)?;
    let expected_short = include_str!("../testdata/short.json");
    let actual_short = serde_json::ser::to_string(&message.as_element())?;
    assert_eq!(actual_short, expected_short);
    let expected_pretty = include_str!("../testdata/pretty.json");
    let actual_pretty = serde_json::ser::to_string_pretty(&message.as_element())?;
    assert_eq!(actual_pretty, expected_pretty);
    Ok(())
  }
}
