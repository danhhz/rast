// Copyright 2020 Daniel Harrison. All Rights Reserved.

use crate::common::Discriminant;
use crate::element_type::{
  ElementType, ListElementType, PointerElementType, PrimitiveElementType, StructElementType,
  UnionElementType,
};
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
      Element::Union(x) => ElementType::Union(x.element_type()),
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
  Struct(StructElement<'a>),
  List(ListElement<'a>),
  ListDecoded(ListDecodedElement<'a>),
}

impl<'a> PointerElement<'a> {
  pub fn element_type(&self) -> PointerElementType {
    match self {
      PointerElement::Struct(x) => PointerElementType::Struct(x.element_type()),
      PointerElement::List(x) => PointerElementType::List(x.element_type()),
      PointerElement::ListDecoded(x) => PointerElementType::List(x.element_type()),
    }
  }
}

pub struct StructElement<'a>(pub &'static StructMeta, pub UntypedStruct<'a>);

impl<'a> StructElement<'a> {
  pub fn from_untyped_list(
    meta: &'static StructMeta,
    untyped: &UntypedList<'a>,
  ) -> Result<Vec<Self>, Error> {
    Vec::<UntypedStruct<'a>>::from_untyped_list(untyped)
      .map(|xs| xs.into_iter().map(|x| StructElement(meta, x)).collect())
  }

  pub fn element_type(&self) -> StructElementType {
    let StructElement(meta, _) = self;
    StructElementType { meta: meta }
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

  pub fn element_type(&self) -> ListElementType {
    let ListElement(meta, _) = self;
    ListElementType { meta: meta }
  }
}

pub struct ListDecodedElement<'a>(pub &'static ListMeta, pub Vec<Element<'a>>);

impl<'a> ListDecodedElement<'a> {
  pub fn element_type(&self) -> ListElementType {
    let ListDecodedElement(meta, _) = self;
    ListElementType { meta: meta }
  }
}

pub struct UnionElement<'a>(pub &'static UnionMeta, pub Discriminant, pub Box<Element<'a>>);

impl<'a> UnionElement<'a> {
  pub fn element_type(&self) -> UnionElementType {
    let UnionElement(meta, _, _) = self;
    UnionElementType { meta: meta }
  }
}

pub enum ElementShared {
  Primitive(PrimitiveElement),
  Pointer(PointerElementShared),
  Union(UnionElementShared),
}

impl ElementShared {
  pub fn as_ref<'a>(&'a self) -> Element<'a> {
    match self {
      ElementShared::Primitive(x) => Element::Primitive(x.clone()),
      ElementShared::Pointer(x) => Element::Pointer(x.as_ref()),
      ElementShared::Union(x) => Element::Union(x.as_ref()),
    }
  }
}

pub enum PointerElementShared {
  Struct(StructElementShared),
  List(ListElementShared),
  ListDecoded(ListDecodedElementShared),
}

impl PointerElementShared {
  pub fn as_ref<'a>(&'a self) -> PointerElement<'a> {
    match self {
      PointerElementShared::Struct(x) => PointerElement::Struct(x.as_ref()),
      PointerElementShared::List(x) => PointerElement::List(x.as_ref()),
      PointerElementShared::ListDecoded(x) => PointerElement::ListDecoded(x.as_ref()),
    }
  }
}

pub struct StructElementShared(pub &'static StructMeta, pub UntypedStructShared);

impl StructElementShared {
  pub fn as_ref<'a>(&'a self) -> StructElement<'a> {
    let StructElementShared(meta, untyped) = self;
    StructElement(meta, untyped.as_ref())
  }
}

pub struct ListElementShared(pub &'static ListMeta, pub UntypedListShared);

impl ListElementShared {
  pub fn as_ref<'a>(&'a self) -> ListElement<'a> {
    let ListElementShared(meta, untyped) = self;
    ListElement(meta, untyped.as_ref())
  }
}

pub struct ListDecodedElementShared(pub &'static ListMeta, pub Vec<ElementShared>);

impl ListDecodedElementShared {
  pub fn as_ref<'a>(&'a self) -> ListDecodedElement<'a> {
    let ListDecodedElementShared(meta, values) = self;
    ListDecodedElement(meta, values.iter().map(|v| v.as_ref()).collect())
  }
}

pub struct UnionElementShared(pub &'static UnionMeta, pub Discriminant, pub Box<ElementShared>);

impl UnionElementShared {
  pub fn as_ref<'a>(&'a self) -> UnionElement<'a> {
    let UnionElementShared(meta, discriminant, value) = self;
    UnionElement(meta, *discriminant, Box::new(value.as_ref().as_ref()))
  }
}
