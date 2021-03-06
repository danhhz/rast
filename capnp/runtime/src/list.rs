// Copyright 2020 Daniel Harrison. All Rights Reserved.

//! Cap'n Proto [list]
//!
//! A list is an ordered sequence of elements of the same type.
//!
//! [list]: crate#list
//! [elements]: crate#element

use crate::common::{CapnpAsRef, NumElements};
use crate::decode::{ListDecode, SegmentPointerDecode};
use crate::element_type::ElementType;
use crate::encode::{SegmentPointerEncode, StructEncode};
use crate::error::Error;
use crate::pointer::ListPointer;
use crate::r#struct::{
  TypedStructRef, TypedStructShared, UntypedStruct, UntypedStructOwned, UntypedStructShared,
};
use crate::segment_pointer::{SegmentPointer, SegmentPointerBorrowMut, SegmentPointerShared};
use crate::slice::Slice;

/// Metadata for intrepreting an encoded Cap'n Proto list
///
/// This contains all the information necessary to fully intrepret an encoded
/// Cap'n Proto list as its final codegen type.
#[derive(Debug, PartialOrd, PartialEq)]
pub struct ListMeta {
  /// The type of item this list repeats.
  pub value_type: ElementType,
}

pub trait TypedList<'a>: Sized {
  fn from_untyped_list(untyped: &UntypedList<'a>) -> Result<Self, Error>;
}

impl<'a, T: TypedListElement<'a>> TypedList<'a> for Slice<'a, T> {
  fn from_untyped_list(untyped: &UntypedList<'a>) -> Result<Self, Error> {
    untyped.list()
  }
}

// TODO: Relate TypedListShared to TypedList.
pub(crate) trait TypedListShared {
  fn set(&self, data: &mut UntypedStructOwned, offset: NumElements);
}

impl<T: TypedListElementShared> TypedListShared for &[T] {
  fn set(&self, data: &mut UntypedStructOwned, offset: NumElements) {
    data.set_list(offset, self);
  }
}

pub enum ListElementDecoding<'a, T> {
  Packed(ElementType, fn(&SegmentPointer<'a>, NumElements) -> T),
  Composite(fn(UntypedStruct<'a>) -> T),
}

pub enum ListElementEncoding<T> {
  Packed(ElementType, fn(&mut SegmentPointerBorrowMut<'_>, NumElements, &T)),
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
    ListElementDecoding::Packed(ElementType::U8, decode_u8_list_element)
  }
}

impl TypedListElementShared for u8 {
  fn encoding() -> ListElementEncoding<Self> {
    ListElementEncoding::Packed(ElementType::U8, encode_u8_list_element)
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
    ListElementDecoding::Packed(ElementType::U64, decode_u64_list_element)
  }
}

impl TypedListElementShared for u64 {
  fn encoding() -> ListElementEncoding<Self> {
    ListElementEncoding::Packed(ElementType::U64, encode_u64_list_element)
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

impl<'a, T: TypedStructRef<'a>> TypedListElement<'a> for T {
  fn decoding() -> ListElementDecoding<'a, Self> {
    ListElementDecoding::Composite(|untyped| T::from_untyped_struct(untyped))
  }
}

impl<T: TypedStructShared> TypedListElementShared for T {
  fn encoding() -> ListElementEncoding<T> {
    ListElementEncoding::Composite(|x| x.as_untyped())
  }
}

/// A borrowed Cap'n Proto list without schema
pub struct UntypedList<'a> {
  pointer: ListPointer,
  pointer_end: SegmentPointer<'a>,
}

impl<'a> UntypedList<'a> {
  /// Returns a new [`UntypedList`]
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

/// A reference-counted Cap'n Proto list without schema
pub struct UntypedListShared {
  pub(crate) pointer: ListPointer,
  pub(crate) pointer_end: SegmentPointerShared,
}

impl<'a> CapnpAsRef<'a, UntypedList<'a>> for UntypedListShared {
  fn capnp_as_ref(&'a self) -> UntypedList<'a> {
    UntypedList { pointer: self.pointer.clone(), pointer_end: self.pointer_end.capnp_as_ref() }
  }
}
