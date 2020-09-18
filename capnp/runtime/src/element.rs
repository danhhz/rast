// Copyright 2020 Daniel Harrison. All Rights Reserved.

use crate::common::{CapnpAsRef, Discriminant};
use crate::element_type::{ElementType, PointerElementType, PrimitiveElementType};
use crate::error::Error;
use crate::list::{ListMeta, TypedList, UntypedList, UntypedListShared};
use crate::r#struct::{StructMeta, UntypedStruct, UntypedStructShared};
use crate::union::UnionMeta;

#[derive(PartialEq, PartialOrd)]
pub enum Element<'a> {
  Primitive(PrimitiveElement),
  Pointer(PointerElement<'a>),
  Union(UnionElement<'a>),
}

impl<'a> Element<'a> {
  pub fn element_type(&self) -> ElementType {
    match self {
      Element::Primitive(x) => ElementType::Primitive(x.element_type()),
      Element::Pointer(x) => ElementType::Pointer(x.element_type()),
      Element::Union(UnionElement(meta, _, _)) => ElementType::Union(meta),
    }
  }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum PrimitiveElement {
  // TODO: Break these out into U64Element, etc?
  U8(u8),
  U64(u64),
}

impl PrimitiveElement {
  pub fn element_type(&self) -> PrimitiveElementType {
    match self {
      PrimitiveElement::U8(_) => PrimitiveElementType::U8,
      PrimitiveElement::U64(_) => PrimitiveElementType::U8,
    }
  }
}

#[derive(PartialEq, PartialOrd)]
pub enum PointerElement<'a> {
  Data(DataElement<'a>),
  Struct(StructElement<'a>),
  List(ListElement<'a>),
  ListDecoded(ListDecodedElement<'a>),
}

impl<'a> PointerElement<'a> {
  pub fn element_type(&self) -> PointerElementType {
    match self {
      PointerElement::Data(_) => PointerElementType::Data,
      PointerElement::Struct(StructElement(meta, _)) => PointerElementType::Struct(meta),
      PointerElement::List(ListElement(meta, _)) => PointerElementType::List(meta),
      PointerElement::ListDecoded(ListDecodedElement(meta, _)) => PointerElementType::List(meta),
    }
  }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct DataElement<'a>(pub &'a [u8]);

pub struct StructElement<'a>(pub &'static StructMeta, pub UntypedStruct<'a>);

impl<'a> StructElement<'a> {
  pub fn from_untyped_list(
    meta: &'static StructMeta,
    untyped: &UntypedList<'a>,
  ) -> Result<Vec<Self>, Error> {
    Vec::<UntypedStruct<'a>>::from_untyped_list(untyped)
      .map(|xs| xs.into_iter().map(|x| StructElement(meta, x)).collect())
  }
}

pub struct ListElement<'a>(pub &'static ListMeta, pub UntypedList<'a>);

impl<'a> ListElement<'a> {
  pub fn to_element_list(&self) -> Result<Vec<Element<'a>>, Error> {
    self.0.value_type.to_element_list(&self.1)
  }

  pub fn from_untyped_list(
    _values: &ElementType,
    _untyped: &UntypedList<'a>,
  ) -> Result<Vec<Self>, Error> {
    todo!()
  }
}

pub struct ListDecodedElement<'a>(pub &'static ListMeta, pub Vec<Element<'a>>);

pub struct UnionElement<'a>(pub &'static UnionMeta, pub Discriminant, pub Box<Element<'a>>);

pub enum ElementShared {
  Primitive(PrimitiveElement),
  Pointer(PointerElementShared),
  Union(UnionElementShared),
}

impl<'a> CapnpAsRef<'a, Element<'a>> for ElementShared {
  fn capnp_as_ref(&'a self) -> Element<'a> {
    match self {
      ElementShared::Primitive(x) => Element::Primitive(x.clone()),
      ElementShared::Pointer(x) => Element::Pointer(x.capnp_as_ref()),
      ElementShared::Union(x) => Element::Union(x.capnp_as_ref()),
    }
  }
}

pub enum PointerElementShared {
  Data(DataElementShared),
  Struct(StructElementShared),
  List(ListElementShared),
  ListDecoded(ListDecodedElementShared),
}

impl<'a> CapnpAsRef<'a, PointerElement<'a>> for PointerElementShared {
  fn capnp_as_ref(&'a self) -> PointerElement<'a> {
    match self {
      PointerElementShared::Data(x) => PointerElement::Data(x.capnp_as_ref()),
      PointerElementShared::Struct(x) => PointerElement::Struct(x.capnp_as_ref()),
      PointerElementShared::List(x) => PointerElement::List(x.capnp_as_ref()),
      PointerElementShared::ListDecoded(x) => PointerElement::ListDecoded(x.capnp_as_ref()),
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
