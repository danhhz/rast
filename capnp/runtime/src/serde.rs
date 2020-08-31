// Copyright 2020 Daniel Harrison. All Rights Reserved.

use serde::ser::SerializeSeq;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

use crate::error::Error;
use crate::reflect::{
  Element, ElementSink, FieldMeta, PointerElement, PointerFieldMeta, PrimitiveElement,
  PrimitiveFieldMeta, StructMeta, ToElement,
};
use crate::untyped::UntypedStruct;

impl<'a> Serialize for Element<'a> {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    match self {
      Element::Primitive(x) => x.serialize(serializer),
      Element::Pointer(x) => x.serialize(serializer),
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
      PointerElement::Struct(meta, untyped) => {
        let mut state = serializer.serialize_struct(meta.name, meta.fields.len())?;
        for field in meta.fields {
          match field {
            FieldMeta::Primitive(PrimitiveFieldMeta::U64(x)) => {
              state.serialize_field(field.name(), &x.get(untyped))?
            }
            FieldMeta::Pointer(pointer) => {
              if pointer.is_null(untyped) {
                continue;
              }
              match pointer {
                PointerFieldMeta::Struct(x) => match x.get_untyped(untyped) {
                  Ok(untyped) => state
                    .serialize_field(field.name(), &PointerElement::Struct((x.meta)(), untyped))?,
                  Err(_) => todo!(),
                },
                PointerFieldMeta::List(x) => {
                  let mut sink: SerdeElementSink<S> = SerdeElementSink {
                    field_name: field.name(),
                    serializer: &mut state,
                    result: Ok(()),
                  };
                  (x.get_element)(untyped, &mut sink);
                  sink.result?;
                }
              }
            }
          }
        }
        state.end()
      }
      PointerElement::List(x) => {
        let data: Result<Vec<Element<'a>>, Error> = x.iter().map(|x| x.to_element()).collect();
        match data {
          Err(_) => todo!(),
          Ok(data) => {
            let mut seq = serializer.serialize_seq(Some(data.len()))?;
            for e in data {
              seq.serialize_element(&e)?;
            }
            seq.end()
          }
        }
      }
    }
  }
}

struct SerdeElementSink<'a, S: Serializer> {
  field_name: &'static str,
  serializer: &'a mut S::SerializeStruct,
  result: Result<(), S::Error>,
}

impl<'a, S: Serializer> ElementSink for SerdeElementSink<'a, S> {
  fn u8(&mut self, value: u8) {
    self.result = self.serializer.serialize_field(self.field_name, &value)
  }
  fn u64(&mut self, value: u64) {
    self.result = self.serializer.serialize_field(self.field_name, &value)
  }
  fn untyped_struct(&mut self, meta: &'static StructMeta, value: Result<UntypedStruct<'_>, Error>) {
    match value {
      Err(_) => todo!(),
      Ok(value) => {
        self.result =
          self.serializer.serialize_field(self.field_name, &PointerElement::Struct(meta, value))
      }
    }
  }
  fn list<'b>(&mut self, value: Result<Vec<&'b dyn ToElement<'b>>, Error>) {
    match value {
      Err(_) => todo!(),
      Ok(value) => {
        self.result = self.serializer.serialize_field(self.field_name, &PointerElement::List(value))
      }
    }
  }
}
