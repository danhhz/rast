// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::convert::TryFrom;

use crate::common::*;
use crate::error::Error;
use crate::pointer::{ListPointer, Pointer, StructPointer};
use crate::segment::SegmentOwned;
use crate::segment_pointer::{SegmentPointer, SegmentPointerOwned, SegmentPointerShared};

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

#[derive(Clone)]
pub struct UntypedStructShared {
  pub pointer: StructPointer,
  pub pointer_end: SegmentPointerShared,
}

impl UntypedStructShared {
  pub fn as_ref<'a>(&'a self) -> UntypedStruct<'a> {
    UntypedStruct { pointer: self.pointer.clone(), pointer_end: self.pointer_end.as_ref() }
  }
}

pub struct UntypedStructOwned {
  pub pointer: StructPointer,
  pub pointer_end: SegmentPointerOwned,
}

impl UntypedStructOwned {
  pub fn new_with_root_struct(data_size: NumWords, pointer_size: NumWords) -> UntypedStructOwned {
    let buf_len = (POINTER_WIDTH_WORDS + data_size + pointer_size).as_bytes();
    let mut buf = Vec::with_capacity(buf_len);
    let pointer =
      StructPointer { off: NumWords(0), data_size: data_size, pointer_size: pointer_size };
    buf.extend(&pointer.encode());
    buf.resize(buf_len, 0);
    let off = POINTER_WIDTH_WORDS;
    let pointer_end = SegmentPointerOwned { seg: SegmentOwned::new_from_buf(buf), off: off };
    UntypedStructOwned { pointer: pointer, pointer_end: pointer_end }
  }

  pub fn into_shared(self) -> UntypedStructShared {
    UntypedStructShared { pointer: self.pointer, pointer_end: self.pointer_end.into_shared() }
  }

  pub fn set_u64(&mut self, offset: NumElements, value: u64) {
    // WIP: Check against self.pointer to see if we should even be setting anything.
    self.pointer_end.set_u64(offset, value)
  }

  pub fn set_pointer(&mut self, offset: NumElements, value: Pointer) {
    // WIP: Check against self.pointer to see if we should even be setting anything.
    // WIP: Hacks
    let offset = NumElements(offset.0 + self.pointer.data_size.0);
    self.pointer_end.set_pointer(offset, value)
  }
}
