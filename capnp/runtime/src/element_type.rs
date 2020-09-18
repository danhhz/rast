// Copyright 2020 Daniel Harrison. All Rights Reserved.

use crate::common::ElementWidth;
use crate::element::{Element, ListElement, PointerElement, PrimitiveElement, StructElement};
use crate::error::Error;
use crate::list::{ListMeta, TypedList, UntypedList};
use crate::r#struct::StructMeta;
use crate::union::UnionMeta;

// TODO: Rename these all to *Meta?
#[derive(Debug, PartialOrd, PartialEq)]
pub enum ElementType {
  Primitive(PrimitiveElementType),
  Pointer(PointerElementType),
  Union(&'static UnionMeta),
}

impl ElementType {
  pub fn width(&self) -> ElementWidth {
    match self {
      ElementType::Primitive(x) => x.width(),
      ElementType::Pointer(x) => x.width(),
      ElementType::Union(_) => todo!(),
    }
  }

  pub fn to_element_list<'a>(&self, untyped: &UntypedList<'a>) -> Result<Vec<Element<'a>>, Error> {
    match self {
      ElementType::Primitive(x) => {
        x.to_element_list(untyped).map(|xs| xs.into_iter().map(|x| Element::Primitive(x)).collect())
      }
      ElementType::Pointer(x) => {
        x.to_element_list(untyped).map(|xs| xs.into_iter().map(|x| Element::Pointer(x)).collect())
      }
      ElementType::Union(_) => todo!(),
    }
  }
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub enum PrimitiveElementType {
  U8,
  U64,
}

impl PrimitiveElementType {
  pub fn width(&self) -> ElementWidth {
    match self {
      PrimitiveElementType::U8 => ElementWidth::OneByte,
      PrimitiveElementType::U64 => ElementWidth::EightBytesNonPointer,
    }
  }

  pub fn to_element_list(&self, untyped: &UntypedList<'_>) -> Result<Vec<PrimitiveElement>, Error> {
    match self {
      PrimitiveElementType::U8 => Vec::<u8>::from_untyped_list(untyped)
        .map(|xs| xs.into_iter().map(|x| PrimitiveElement::U8(x)).collect()),
      PrimitiveElementType::U64 => Vec::<u64>::from_untyped_list(untyped)
        .map(|xs| xs.into_iter().map(|x| PrimitiveElement::U64(x)).collect()),
    }
  }
}

#[derive(Debug, PartialOrd, PartialEq)]
pub enum PointerElementType {
  Data,
  Struct(&'static StructMeta),
  List(&'static ListMeta),
}

impl PointerElementType {
  pub fn width(&self) -> ElementWidth {
    ElementWidth::EightBytesPointer
  }

  pub fn to_element_list<'a>(
    &self,
    untyped: &UntypedList<'a>,
  ) -> Result<Vec<PointerElement<'a>>, Error> {
    match self {
      PointerElementType::Data => todo!(),
      PointerElementType::Struct(m) => StructElement::from_untyped_list(m, untyped)
        .map(|xs| xs.into_iter().map(|x| PointerElement::Struct(x)).collect()),
      PointerElementType::List(m) => ListElement::from_untyped_list(&m.value_type, untyped)
        .map(|xs| xs.into_iter().map(|x| PointerElement::List(x)).collect()),
    }
  }
}
