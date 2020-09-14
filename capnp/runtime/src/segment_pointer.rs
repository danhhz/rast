// Copyright 2020 Daniel Harrison. All Rights Reserved.

use crate::common::*;
use crate::decode::SegmentPointerDecode;
use crate::encode::SegmentPointerEncode;
use crate::segment::{Segment, SegmentID, SegmentOwned, SegmentShared};

#[derive(Clone)]
pub struct SegmentPointer<'a> {
  pub seg: Segment<'a>,
  pub off: NumWords,
}

impl<'a> SegmentPointer<'a> {
  pub fn from_root(seg: Segment<'a>) -> Self {
    SegmentPointer { seg: seg, off: NumWords(0) }
  }
}

impl<'a> SegmentPointerDecode<'a> for SegmentPointer<'a> {
  fn empty() -> Self {
    SegmentPointer { seg: Segment::empty(), off: NumWords(0) }
  }
  fn from_root(seg: Segment<'a>) -> Self {
    SegmentPointer::from_root(seg)
  }
  fn add(&self, offset: NumWords) -> Self {
    SegmentPointer { seg: self.seg.clone(), off: self.off + offset }
  }
  fn buf(&self) -> &[u8] {
    self.seg.buf()
  }
  fn offset_w(&self) -> NumWords {
    self.off
  }
  fn other(&self, id: SegmentID) -> Option<Segment<'a>> {
    self.seg.other(id)
  }
  fn all_other(&self) -> Vec<(SegmentID, Segment<'a>)> {
    self.seg.all_other()
  }
}

#[derive(Clone)]
pub struct SegmentPointerShared {
  pub seg: SegmentShared,
  pub off: NumWords,
}

impl SegmentPointerShared {
  pub fn as_ref<'a>(&'a self) -> SegmentPointer<'a> {
    SegmentPointer { seg: Segment::Borrowed(self.seg.as_ref()), off: self.off }
  }
}

pub struct SegmentPointerOwned {
  pub seg: SegmentOwned,
  pub off: NumWords,
}

impl SegmentPointerOwned {
  pub fn borrow_mut<'a>(&'a mut self) -> SegmentPointerBorrowMut<'a> {
    SegmentPointerBorrowMut { seg: &mut self.seg, off: self.off }
  }

  pub fn into_shared(self) -> SegmentPointerShared {
    SegmentPointerShared { seg: self.seg.into_shared(), off: self.off }
  }
}

impl SegmentPointerEncode for SegmentPointerOwned {
  fn buf_mut(&mut self) -> &mut [u8] {
    self.seg.buf_mut()
  }
  fn ensure_len(&mut self, len_bytes: usize) {
    self.seg.ensure_len(len_bytes)
  }
  fn offset_w(&self) -> NumWords {
    self.off
  }
}

pub struct SegmentPointerBorrowMut<'a> {
  pub seg: &'a mut SegmentOwned,
  pub off: NumWords,
}

impl<'a> SegmentPointerBorrowMut<'a> {
  pub fn add(self, offset_w: NumWords) -> SegmentPointerBorrowMut<'a> {
    SegmentPointerBorrowMut { seg: self.seg, off: self.off + offset_w }
  }
}

impl<'a> SegmentPointerEncode for SegmentPointerBorrowMut<'a> {
  fn buf_mut(&mut self) -> &mut [u8] {
    self.seg.buf_mut()
  }
  fn ensure_len(&mut self, len_bytes: usize) {
    self.seg.ensure_len(len_bytes)
  }
  fn offset_w(&self) -> NumWords {
    self.off
  }
}
