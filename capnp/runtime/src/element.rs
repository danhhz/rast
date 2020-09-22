// Copyright 2020 Daniel Harrison. All Rights Reserved.

use crate::common::{CapnpAsRef, Discriminant};
use crate::element_type::ElementType;
use crate::error::Error;
use crate::list::{ListMeta, UntypedList, UntypedListShared};
use crate::r#struct::{StructMeta, UntypedStruct, UntypedStructShared};
use crate::union::UnionMeta;

#[derive(PartialEq, PartialOrd)]
pub enum Element<'a> {
  U8(u8),
  U64(u64),
  Data(DataElement<'a>),
  Struct(StructElement<'a>),
  List(ListElement<'a>),
  ListDecoded(ListDecodedElement<'a>),
  Union(UnionElement<'a>),
}

impl<'a> Element<'a> {
  pub fn element_type(&self) -> ElementType {
    match self {
      Element::U8(_) => ElementType::U8,
      Element::U64(_) => ElementType::U64,
      Element::Data(_) => ElementType::Data,
      Element::Struct(StructElement(meta, _)) => ElementType::Struct(meta),
      Element::List(ListElement(meta, _)) => ElementType::List(meta),
      Element::ListDecoded(ListDecodedElement(meta, _)) => ElementType::List(meta),
      Element::Union(UnionElement(meta, _, _)) => ElementType::Union(meta),
    }
  }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct DataElement<'a>(pub &'a [u8]);

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
  U8(u8),
  U64(u64),
  Data(DataElementShared),
  Struct(StructElementShared),
  ListDecoded(ListDecodedElementShared),
  Union(UnionElementShared),
}

impl<'a> CapnpAsRef<'a, Element<'a>> for ElementShared {
  fn capnp_as_ref(&'a self) -> Element<'a> {
    match self {
      ElementShared::U8(x) => Element::U8(*x),
      ElementShared::U64(x) => Element::U64(*x),
      ElementShared::Data(x) => Element::Data(x.capnp_as_ref()),
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
