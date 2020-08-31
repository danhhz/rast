// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::convert::TryFrom;

use crate::common::*;
use crate::error::Error;
use crate::pointer::{ListPointer, StructPointer};
use crate::segment_pointer::SegmentPointer;

#[derive(Clone)]
pub struct UntypedStruct<'a> {
  pub pointer: StructPointer,
  pub pointer_end: SegmentPointer<'a>,
}

impl<'a> TryFrom<SegmentPointer<'a>> for UntypedStruct<'a> {
  type Error = Error;

  fn try_from(value: SegmentPointer<'a>) -> Result<Self, Self::Error> {
    let (pointer, pointer_end) = value.struct_pointer(NumElements(0))?;
    Ok(UntypedStruct { pointer: pointer, pointer_end: pointer_end })
  }
}

pub struct UntypedList<'a> {
  pub pointer: ListPointer,
  pub pointer_end: SegmentPointer<'a>,
}

// pub struct UntypedStructOwned {
//   pointer: StructPointer,
//   pointer_end: SegmentOwned,
// }

// impl UntypedStructOwned {
//   fn as_ref<'a>(&'a self) -> UntypedStruct<'a> {
//     UntypedStruct { pointer: self.pointer, pointer_end: self.pointer_end.as_ref() }
//   }
// }
