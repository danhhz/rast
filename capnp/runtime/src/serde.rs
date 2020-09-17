// Copyright 2020 Daniel Harrison. All Rights Reserved.

use serde::ser::{SerializeSeq, SerializeStruct};
use serde::{Serialize, Serializer};

use crate::element::{
  Element, ListDecodedElement, ListElement, PointerElement, PrimitiveElement, StructElement,
  UnionElement,
};
use crate::field_meta::FieldMeta;

impl<'a> Serialize for Element<'a> {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    match self {
      Element::Primitive(x) => x.serialize(serializer),
      Element::Pointer(x) => x.serialize(serializer),
      Element::Union(x) => x.serialize(serializer),
    }
  }
}

impl Serialize for PrimitiveElement {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    match self {
      PrimitiveElement::U8(x) => serializer.serialize_u8(*x),
      PrimitiveElement::U64(x) => serializer.serialize_u64(*x),
    }
  }
}

impl<'a> Serialize for PointerElement<'a> {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    match self {
      PointerElement::Struct(x) => x.serialize(serializer),
      PointerElement::List(x) => x.serialize(serializer),
      PointerElement::ListDecoded(x) => x.serialize(serializer),
    }
  }
}

impl<'a> Serialize for StructElement<'a> {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    let StructElement(meta, untyped) = self;
    let fields = meta.fields();
    let mut state = serializer.serialize_struct(meta.name, fields.len())?;
    for field in fields {
      match field {
        FieldMeta::Primitive(x) => state.serialize_field(field.name(), &x.get_element(untyped))?,
        FieldMeta::Pointer(x) => {
          if x.is_null(untyped) {
            continue;
          }
          match x.get_element(untyped) {
            Err(_) => todo!(),
            Ok(x) => state.serialize_field(field.name(), &x)?,
          }
        }
        FieldMeta::Union(x) => match x.get_element(untyped) {
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
    let variant_meta = meta.get(*discriminant).expect("WIP");
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
  use std::convert::TryInto;
  use std::error;
  use std::fs::File;
  use std::io::Read;

  use crate::samples::test_capnp::TestAllTypes;
  use capnp_runtime::prelude::*;

  #[test]
  fn serialize_json() -> Result<(), Box<dyn error::Error>> {
    let mut f = File::open("testdata/binary")?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    let seg = decode_stream_official(&buf)?;
    let message = TestAllTypes::from_untyped_struct(SegmentPointer::from_root(seg).try_into()?);

    let mut f = File::open("testdata/short.json")?;
    let mut expected = Vec::new();
    f.read_to_end(&mut expected)?;
    let expected = String::from_utf8(expected)?;

    let actual = serde_json::ser::to_string(&message.as_element())?;
    assert_eq!(actual, expected);
    Ok(())
  }
}
