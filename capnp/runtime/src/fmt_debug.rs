// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::fmt;

use crate::common::CapnpAsRef;
use crate::element::{
  DataElement, Element, ElementShared, ListDecodedElement, ListElement, PointerElement,
  PrimitiveElement, StructElement, UnionElement,
};
use crate::field_meta::FieldMeta;
use crate::r#struct::StructMeta;

impl fmt::Debug for StructMeta {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("StructMeta")
      .field("name", &self.name)
      .field("data_size", &self.data_size)
      .field("pointer_size", &self.pointer_size)
      // NB: Can't just print out the fields here or we'll get infinite
      // recursion in self-referencing struct types.
      .field("fields", &"WIP")
      .finish()
  }
}

impl fmt::Debug for ElementShared {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.capnp_as_ref().fmt(f)
  }
}

impl fmt::Debug for PrimitiveElement {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      PrimitiveElement::U8(x) => x.fmt(f),
      PrimitiveElement::U64(x) => x.fmt(f),
    }
  }
}

impl<'a> fmt::Debug for Element<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Element::Primitive(x) => x.fmt(f),
      Element::Pointer(x) => x.fmt(f),
      Element::Union(x) => x.fmt(f),
    }
  }
}

impl<'a> fmt::Debug for PointerElement<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      PointerElement::Data(x) => x.fmt(f),
      PointerElement::Struct(x) => x.fmt(f),
      PointerElement::List(x) => x.fmt(f),
      PointerElement::ListDecoded(x) => x.fmt(f),
    }
  }
}

impl<'a> fmt::Debug for DataElement<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

impl<'a> fmt::Debug for StructElement<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let StructElement(meta, untyped) = self;
    f.write_str("(")?;
    let mut has_fields = false;
    for field_meta in meta.fields() {
      match field_meta {
        FieldMeta::Pointer(x) => {
          if x.is_null(untyped) {
            continue;
          }
        }
        _ => {} // No-op
      }
      if has_fields {
        f.write_str(", ")?;
      }
      has_fields = true;
      f.write_str(field_meta.name())?;
      f.write_str(" = ")?;
      match field_meta.get_element(untyped) {
        Ok(x) => x.fmt(f)?,
        x => x.fmt(f)?,
      };
    }
    f.write_str(")")
  }
}

impl<'a> fmt::Debug for ListElement<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let values = match self.to_element_list() {
      Ok(x) => x,
      Err(err) => {
        f.write_str("Err(")?;
        err.fmt(f)?;
        return f.write_str(")");
      }
    };

    f.write_str("[")?;
    let mut has_fields = false;
    for value in values {
      if has_fields {
        f.write_str(", ")?;
      }
      has_fields = true;
      value.fmt(f)?;
    }
    f.write_str("]")
  }
}

impl<'a> fmt::Debug for ListDecodedElement<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let ListDecodedElement(_, values) = self;
    values.as_slice().fmt(f)
  }
}

impl<'a> fmt::Debug for UnionElement<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let UnionElement(meta, discriminant, value) = self;
    let discriminant = meta.get(*discriminant).expect("WIP");
    f.write_str("(")?;
    f.write_str(discriminant.field_meta.name())?;
    f.write_str(" = ")?;
    value.fmt(f)?;
    f.write_str(")")
  }
}
