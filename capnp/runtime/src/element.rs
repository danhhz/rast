// Copyright 2020 Daniel Harrison. All Rights Reserved.

use crate::common::{CapnpAsRef, Discriminant};
use crate::element_type::ElementType;
use crate::error::Error;
use crate::list::{ListMeta, UntypedList, UntypedListShared};
use crate::r#enum::EnumMeta;
use crate::r#struct::{StructMeta, UntypedStruct, UntypedStructShared};
use crate::union::UnionMeta;

#[derive(PartialEq, PartialOrd)]
pub enum Element<'a> {
  Bool(bool),
  I32(i32),
  U8(u8),
  U16(u16),
  U32(u32),
  U64(u64),
  F32(f32),
  F64(f64),
  Data(DataElement<'a>),
  Text(TextElement<'a>),
  Enum(EnumElement),
  Struct(StructElement<'a>),
  List(ListElement<'a>),
  ListDecoded(ListDecodedElement<'a>),
  Union(UnionElement<'a>),
}

impl<'a> Element<'a> {
  pub fn element_type(&self) -> ElementType {
    match self {
      Element::Bool(_) => ElementType::Bool,
      Element::I32(_) => ElementType::I32,
      Element::U8(_) => ElementType::U8,
      Element::U16(_) => ElementType::U16,
      Element::U32(_) => ElementType::U32,
      Element::U64(_) => ElementType::U64,
      Element::F32(_) => ElementType::F32,
      Element::F64(_) => ElementType::F64,
      Element::Data(_) => ElementType::Data,
      Element::Text(_) => ElementType::Text,
      Element::Enum(EnumElement(meta, _)) => ElementType::Enum(meta),
      Element::Struct(StructElement(meta, _)) => ElementType::Struct(meta),
      Element::List(ListElement(meta, _)) => ElementType::List(meta),
      Element::ListDecoded(ListDecodedElement(meta, _)) => ElementType::List(meta),
      Element::Union(UnionElement(meta, _, _)) => ElementType::Union(meta),
    }
  }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct DataElement<'a>(pub &'a [u8]);

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct TextElement<'a>(pub &'a str);

#[derive(PartialEq, PartialOrd)]
pub struct EnumElement(pub &'static EnumMeta, pub Discriminant);

pub struct StructElement<'a>(pub &'static StructMeta, pub UntypedStruct<'a>);

pub struct ListElement<'a>(pub &'static ListMeta, pub UntypedList<'a>);

impl<'a> ListElement<'a> {
  pub fn to_element_list(&self) -> Result<Vec<Element<'a>>, Error> {
    self.0.value_type.to_element_list(&self.1)
  }
}

pub struct ListDecodedElement<'a>(pub &'static ListMeta, pub Vec<Element<'a>>);

pub struct UnionElement<'a>(pub &'static UnionMeta, pub Discriminant, pub Box<Element<'a>>);

// TODO: Polish, document, and expose this.
#[allow(dead_code)]
pub enum ElementShared {
  Bool(bool),
  I32(i32),
  U8(u8),
  U16(u16),
  U32(u32),
  U64(u64),
  F32(f32),
  F64(f64),
  Data(DataElementShared),
  Text(TextElementShared),
  Enum(EnumElement),
  Struct(StructElementShared),
  ListDecoded(ListDecodedElementShared),
  Union(UnionElementShared),
}

impl<'a> CapnpAsRef<'a, Element<'a>> for ElementShared {
  fn capnp_as_ref(&'a self) -> Element<'a> {
    match self {
      ElementShared::Bool(x) => Element::Bool(*x),
      ElementShared::I32(x) => Element::I32(*x),
      ElementShared::U8(x) => Element::U8(*x),
      ElementShared::U16(x) => Element::U16(*x),
      ElementShared::U32(x) => Element::U32(*x),
      ElementShared::U64(x) => Element::U64(*x),
      ElementShared::F32(x) => Element::F32(*x),
      ElementShared::F64(x) => Element::F64(*x),
      ElementShared::Data(x) => Element::Data(x.capnp_as_ref()),
      ElementShared::Text(x) => Element::Text(x.capnp_as_ref()),
      ElementShared::Enum(EnumElement(meta, x)) => Element::Enum(EnumElement(meta, *x)),
      ElementShared::Struct(x) => Element::Struct(x.capnp_as_ref()),
      ElementShared::ListDecoded(x) => Element::ListDecoded(x.capnp_as_ref()),
      ElementShared::Union(x) => Element::Union(x.capnp_as_ref()),
    }
  }
}

pub struct DataElementShared(pub Vec<u8>);

impl<'a> CapnpAsRef<'a, DataElement<'a>> for DataElementShared {
  fn capnp_as_ref(&'a self) -> DataElement<'a> {
    let DataElementShared(value) = self;
    DataElement(&value)
  }
}

pub struct TextElementShared(pub String);

impl<'a> CapnpAsRef<'a, TextElement<'a>> for TextElementShared {
  fn capnp_as_ref(&'a self) -> TextElement<'a> {
    let TextElementShared(value) = self;
    TextElement(&value)
  }
}

pub struct StructElementShared(pub &'static StructMeta, pub UntypedStructShared);

impl<'a> CapnpAsRef<'a, StructElement<'a>> for StructElementShared {
  fn capnp_as_ref(&'a self) -> StructElement<'a> {
    let StructElementShared(meta, untyped) = self;
    StructElement(meta, untyped.capnp_as_ref())
  }
}

pub struct ListElementShared(pub &'static ListMeta, pub UntypedListShared);

impl<'a> CapnpAsRef<'a, ListElement<'a>> for ListElementShared {
  fn capnp_as_ref(&'a self) -> ListElement<'a> {
    let ListElementShared(meta, untyped) = self;
    ListElement(meta, untyped.capnp_as_ref())
  }
}

pub struct ListDecodedElementShared(pub &'static ListMeta, pub Vec<ElementShared>);

impl<'a> CapnpAsRef<'a, ListDecodedElement<'a>> for ListDecodedElementShared {
  fn capnp_as_ref(&'a self) -> ListDecodedElement<'a> {
    let ListDecodedElementShared(meta, values) = self;
    ListDecodedElement(meta, values.iter().map(|v| v.capnp_as_ref()).collect())
  }
}

pub struct UnionElementShared(pub &'static UnionMeta, pub Discriminant, pub Box<ElementShared>);

impl<'a> CapnpAsRef<'a, UnionElement<'a>> for UnionElementShared {
  fn capnp_as_ref(&'a self) -> UnionElement<'a> {
    let UnionElementShared(meta, discriminant, value) = self;
    UnionElement(meta, *discriminant, Box::new(value.as_ref().capnp_as_ref()))
  }
}
