// Copyright 2020 Daniel Harrison. All Rights Reserved.

use crate::common::ElementWidth;
use crate::element::{Element, StructElement};
use crate::error::Error;
use crate::list::{ListMeta, TypedList, UntypedList};
use crate::r#enum::EnumMeta;
use crate::r#struct::{StructMeta, UntypedStruct};
use crate::slice::Slice;
use crate::union::UnionMeta;

/// Schema for a Cap'n Proto [element]
///
/// [element]: crate#element
// TODO: Rename these all to *Meta?
#[derive(Debug, PartialOrd, PartialEq)]
pub enum ElementType {
  /// An [`i32`]
  I32,
  /// A [`u8`]
  U8,
  /// A [`u64`]
  U64,
  /// A slice of [`u8`]s
  Data,
  /// Schema for a Cap'n Proto [enum](crate#enum)
  Enum(&'static EnumMeta),
  /// Schema for a Cap'n Proto [struct](crate#struct)
  Struct(&'static StructMeta),
  /// Schema for a Cap'n Proto [list](crate#list)
  List(&'static ListMeta),
  /// Schema for a Cap'n Proto [union](crate#union)
  Union(&'static UnionMeta),
}

impl ElementType {
  /// Width of an element with this schema
  pub fn width(&self) -> ElementWidth {
    match self {
      ElementType::I32 => ElementWidth::FourBytes,
      ElementType::U8 => ElementWidth::OneByte,
      ElementType::U64 => ElementWidth::EightBytesNonPointer,
      ElementType::Data => ElementWidth::EightBytesPointer,
      ElementType::Enum(_) => ElementWidth::TwoBytes,
      ElementType::Struct(_) => ElementWidth::EightBytesPointer,
      ElementType::List(_) => ElementWidth::EightBytesPointer,
      ElementType::Union(_) => todo!(),
    }
  }

  /// Intreprets the given encoded list as elements of this type.
  pub fn to_element_list<'a>(&self, untyped: &UntypedList<'a>) -> Result<Vec<Element<'a>>, Error> {
    match self {
      ElementType::I32 => todo!(),
      ElementType::U8 => Slice::<u8>::from_untyped_list(untyped)
        .map(|xs| xs.into_iter().map(|x| Element::U8(x)).collect()),
      ElementType::U64 => Slice::<u64>::from_untyped_list(untyped)
        .map(|xs| xs.into_iter().map(|x| Element::U64(x)).collect()),
      ElementType::Data => todo!(),
      ElementType::Enum(_) => todo!(),
      ElementType::Struct(meta) => Slice::<UntypedStruct<'a>>::from_untyped_list(untyped)
        .map(|xs| xs.into_iter().map(|x| Element::Struct(StructElement(meta, x))).collect()),
      ElementType::List(_) => todo!(),
      ElementType::Union(_) => todo!(),
    }
  }
}
