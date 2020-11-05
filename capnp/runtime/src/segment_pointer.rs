// Copyright 2020 Daniel Harrison. All Rights Reserved.

use crate::common::{CapnpAsRef, CapnpToOwned, NumWords};
use crate::decode::SegmentPointerDecode;
use crate::encode::SegmentPointerEncode;
use crate::error::Error;
use crate::segment::{SegmentBorrowed, SegmentID, SegmentOwned, SegmentShared};

#[derive(Clone)]
pub struct SegmentPointer<'a> {
  pub seg: SegmentBorrowed<'a>,
  pub off: NumWords,
}

impl<'a> SegmentPointer<'a> {
  pub fn from_root(seg: SegmentBorrowed<'a>) -> Self {
    SegmentPointer { seg: seg, off: NumWords(0) }
  }

  pub fn buf_ref(&self) -> &'a [u8] {
    self.seg.buf()
  }
}

impl<'a> CapnpToOwned<'a> for SegmentPointer<'a> {
  type Owned = SegmentPointerShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    SegmentPointerShared { seg: self.seg.capnp_to_owned(), off: self.off }
  }
}

impl<'a> SegmentPointerDecode<'a> for SegmentPointer<'a> {
  type Segment = SegmentBorrowed<'a>;

  fn empty() -> Self {
    SegmentPointer { seg: SegmentBorrowed::empty(), off: NumWords(0) }
  }
  fn from_root(seg: Self::Segment) -> Self {
    SegmentPointer::from_root(seg)
  }
  fn add(self, offset: NumWords) -> Self {
    SegmentPointer { seg: self.seg, off: self.off + offset }
  }
  fn buf(&self) -> &[u8] {
    self.seg.buf()
  }
  fn offset_w(&self) -> NumWords {
    self.off
  }
  fn other(self, id: SegmentID) -> Result<Self::Segment, (Self, Error)> {
    self.seg.other(id).ok_or_else(|| {
      let err = Error::Encoding(format!(
        "segment {:?} not found in {:?}",
        id,
        self.seg.all_other().iter().map(|x| x.0).collect::<Vec<_>>()
      ));
      (self, err)
    })
  }
}

#[derive(Clone)]
pub struct SegmentPointerShared {
  pub seg: SegmentShared,
  pub off: NumWords,
}

impl<'a> CapnpAsRef<'a, SegmentPointer<'a>> for SegmentPointerShared {
  fn capnp_as_ref(&'a self) -> SegmentPointer<'a> {
    SegmentPointer { seg: self.seg.capnp_as_ref(), off: self.off }
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

  pub fn from_root(seg: SegmentOwned) -> Self {
    SegmentPointerOwned { seg: seg, off: NumWords(0) }
  }

  pub fn into_shared(self) -> SegmentPointerShared {
    SegmentPointerShared { seg: self.seg.into_shared(), off: self.off }
  }
}

impl<'a> SegmentPointerDecode<'a> for SegmentPointerOwned {
  type Segment = SegmentOwned;

  fn empty() -> Self {
    SegmentPointerOwned { seg: SegmentOwned::new_from_buf(Vec::new()), off: NumWords(0) }
  }
  fn from_root(seg: Self::Segment) -> Self {
    SegmentPointerOwned::from_root(seg)
  }
  fn add(self, offset: NumWords) -> Self {
    SegmentPointerOwned { seg: self.seg, off: self.off + offset }
  }
  fn buf(&self) -> &[u8] {
    self.seg.buf()
  }
  fn offset_w(&self) -> NumWords {
    self.off
  }
  fn other(self, _id: SegmentID) -> Result<Self::Segment, (Self, Error)> {
    todo!()
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

impl<'a> CapnpAsRef<'a, SegmentPointer<'a>> for SegmentPointerBorrowMut<'a> {
  fn capnp_as_ref(&'a self) -> SegmentPointer<'a> {
    SegmentPointer { seg: self.seg.capnp_as_ref(), off: self.off }
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
