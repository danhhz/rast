// Copyright 2020 Daniel Harrison. All Rights Reserved.

use crate::common::{CapnpAsRef, NumElements};
use crate::decode::{ListDecode, SegmentPointerDecode};
use crate::element_type::{ElementType, PrimitiveElementType};
use crate::encode::{SegmentPointerEncode, StructEncode};
use crate::error::Error;
use crate::pointer::ListPointer;
use crate::r#struct::{
  TypedStruct, TypedStructShared, UntypedStruct, UntypedStructOwned, UntypedStructShared,
};
use crate::segment_pointer::{SegmentPointer, SegmentPointerBorrowMut, SegmentPointerShared};

#[derive(Debug, PartialOrd, PartialEq)]
pub struct ListMeta {
  pub value_type: ElementType,
}

pub trait TypedList<'a>: Sized {
  fn from_untyped_list(untyped: &UntypedList<'a>) -> Result<Self, Error>;
}

impl<'a, T: TypedListElement<'a>> TypedList<'a> for Vec<T> {
  fn from_untyped_list(untyped: &UntypedList<'a>) -> Result<Self, Error> {
    untyped.list()
  }
}

// TODO: Relate TypedListShared to TypedList.
pub trait TypedListShared {
  fn set(&self, data: &mut UntypedStructOwned, offset: NumElements);
}

impl<T: TypedListElementShared> TypedListShared for &[T] {
  fn set(&self, data: &mut UntypedStructOwned, offset: NumElements) {
    data.set_list(offset, self);
  }
}

pub enum ListElementDecoding<'a, T> {
  Primitive(PrimitiveElementType, fn(&SegmentPointer<'a>, NumElements) -> T),
  Composite(fn(UntypedStruct<'a>) -> T),
}

pub enum ListElementEncoding<T> {
  Primitive(PrimitiveElementType, fn(&mut SegmentPointerBorrowMut<'_>, NumElements, &T)),
  Composite(fn(&T) -> UntypedStructShared),
}

pub trait TypedListElement<'a>: Sized {
  fn decoding() -> ListElementDecoding<'a, Self>;
}

pub trait TypedListElementShared: Sized {
  fn encoding() -> ListElementEncoding<Self>;
}

fn decode_u8_list_element(list_data_begin: &SegmentPointer<'_>, offset_e: NumElements) -> u8 {
  list_data_begin.u8(offset_e)
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn encode_u8_list_element(
  list_data_begin: &mut SegmentPointerBorrowMut<'_>,
  offset_e: NumElements,
  value: &u8,
) {
  list_data_begin.set_u8(offset_e, *value)
}

impl<'a> TypedListElement<'a> for u8 {
  fn decoding() -> ListElementDecoding<'a, Self> {
    ListElementDecoding::Primitive(PrimitiveElementType::U8, decode_u8_list_element)
  }
}

impl TypedListElementShared for u8 {
  fn encoding() -> ListElementEncoding<Self> {
    ListElementEncoding::Primitive(PrimitiveElementType::U8, encode_u8_list_element)
  }
}

fn decode_u64_list_element(list_data_begin: &SegmentPointer<'_>, offset_e: NumElements) -> u64 {
  list_data_begin.u64(offset_e)
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn encode_u64_list_element(
  list_data_begin: &mut SegmentPointerBorrowMut<'_>,
  offset_e: NumElements,
  value: &u64,
) {
  list_data_begin.set_u64(offset_e, *value)
}

impl<'a> TypedListElement<'a> for u64 {
  fn decoding() -> ListElementDecoding<'a, Self> {
    ListElementDecoding::Primitive(PrimitiveElementType::U64, decode_u64_list_element)
  }
}

impl TypedListElementShared for u64 {
  fn encoding() -> ListElementEncoding<Self> {
    ListElementEncoding::Primitive(PrimitiveElementType::U64, encode_u64_list_element)
  }
}

impl<'a> TypedListElement<'a> for UntypedStruct<'a> {
  fn decoding() -> ListElementDecoding<'a, Self> {
    ListElementDecoding::Composite(std::convert::identity)
  }
}

impl TypedListElementShared for &UntypedStructShared {
  fn encoding() -> ListElementEncoding<Self> {
    ListElementEncoding::Composite(|x| (*x).clone())
  }
}

impl<'a, T: TypedStruct<'a>> TypedListElement<'a> for T {
  fn decoding() -> ListElementDecoding<'a, Self> {
    ListElementDecoding::Composite(|untyped| T::from_untyped_struct(untyped))
  }
}

impl<T: TypedStructShared> TypedListElementShared for T {
  fn encoding() -> ListElementEncoding<T> {
    ListElementEncoding::Composite(|x| x.as_untyped())
  }
}

pub struct UntypedList<'a> {
  pointer: ListPointer,
  pointer_end: SegmentPointer<'a>,
}

impl<'a> UntypedList<'a> {
  pub fn new(pointer: ListPointer, pointer_end: SegmentPointer<'a>) -> Self {
    UntypedList { pointer: pointer, pointer_end: pointer_end }
  }
}

impl<'a> ListDecode<'a> for UntypedList<'a> {
  fn pointer(&self) -> &ListPointer {
    &self.pointer
  }
  fn pointer_end(&self) -> &SegmentPointer<'a> {
    &self.pointer_end
  }
}

pub struct UntypedListShared {
  pub pointer: ListPointer,
  pub pointer_end: SegmentPointerShared,
}

impl<'a> CapnpAsRef<'a, UntypedList<'a>> for UntypedListShared {
  fn capnp_as_ref(&'a self) -> UntypedList<'a> {
    UntypedList { pointer: self.pointer.clone(), pointer_end: self.pointer_end.capnp_as_ref() }
  }
}
