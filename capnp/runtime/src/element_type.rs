// Copyright 2020 Daniel Harrison. All Rights Reserved.

use crate::common::ElementWidth;
use crate::element::{Element, StructElement};
use crate::error::Error;
use crate::list::{ListMeta, TypedList, UntypedList};
use crate::r#struct::{StructMeta, UntypedStruct};
use crate::union::UnionMeta;

// TODO: Rename these all to *Meta?
#[derive(Debug, PartialOrd, PartialEq)]
pub enum ElementType {
  U8,
  U64,
  Data,
  Struct(&'static StructMeta),
  List(&'static ListMeta),
  Union(&'static UnionMeta),
}

impl ElementType {
  pub fn width(&self) -> ElementWidth {
    match self {
      ElementType::U8 => ElementWidth::OneByte,
      ElementType::U64 => ElementWidth::EightBytesNonPointer,
      ElementType::Data => ElementWidth::EightBytesPointer,
      ElementType::Struct(_) => ElementWidth::EightBytesPointer,
      ElementType::List(_) => ElementWidth::EightBytesPointer,
      ElementType::Union(_) => todo!(),
    }
  }

  pub fn to_element_list<'a>(&self, untyped: &UntypedList<'a>) -> Result<Vec<Element<'a>>, Error> {
    match self {
      ElementType::U8 => Vec::<u8>::from_untyped_list(untyped)
        .map(|xs| xs.into_iter().map(|x| Element::U8(x)).collect()),
      ElementType::U64 => Vec::<u64>::from_untyped_list(untyped)
        .map(|xs| xs.into_iter().map(|x| Element::U64(x)).collect()),
      ElementType::Data => todo!(),
      ElementType::Struct(meta) => Vec::<UntypedStruct<'a>>::from_untyped_list(untyped)
        .map(|xs| xs.into_iter().map(|x| Element::Struct(StructElement(meta, x))).collect()),
      ElementType::List(_) => todo!(),
      ElementType::Union(_) => todo!(),
    }
  }
}
