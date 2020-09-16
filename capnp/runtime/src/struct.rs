// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::collections::HashSet;
use std::convert::TryFrom;
use std::io::{self, Write};

use crate::common::{
  CapnpAsRef, CapnpToOwned, Discriminant, NumElements, NumWords, POINTER_WIDTH_WORDS,
};
use crate::decode::{SegmentPointerDecode, StructDecode};
use crate::element::StructElement;
use crate::encode::StructEncode;
use crate::error::Error;
use crate::field_meta::FieldMeta;
use crate::pointer::StructPointer;
use crate::segment::{SegmentBorrowed, SegmentID, SegmentOwned};
use crate::segment_pointer::{
  SegmentPointer, SegmentPointerBorrowMut, SegmentPointerOwned, SegmentPointerShared,
};

pub struct StructMeta {
  pub name: &'static str,
  pub data_size: NumWords,
  pub pointer_size: NumWords,
  pub fields: fn() -> &'static [FieldMeta],
}

impl StructMeta {
  pub fn fields(&self) -> &'static [FieldMeta] {
    (self.fields)()
  }
}

pub trait TypedStruct<'a> {
  fn meta() -> &'static StructMeta;
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self;
  fn as_untyped(&self) -> UntypedStruct<'a>;
  // TODO: Move this
  fn as_element(&self) -> StructElement<'a> {
    StructElement(Self::meta(), self.as_untyped())
  }
}

pub trait TypedStructShared {
  fn meta() -> &'static StructMeta;
  fn from_untyped_struct(data: UntypedStructShared) -> Self;
  fn as_untyped(&self) -> UntypedStructShared;
}

#[derive(Clone)]
pub struct UntypedStruct<'a> {
  pointer: StructPointer,
  pointer_end: SegmentPointer<'a>,
}

impl<'a> TryFrom<SegmentPointer<'a>> for UntypedStruct<'a> {
  type Error = Error;

  fn try_from(value: SegmentPointer<'a>) -> Result<Self, Self::Error> {
    let (pointer, pointer_end) = value.struct_pointer(NumElements(0))?;
    Ok(UntypedStruct { pointer: pointer, pointer_end: pointer_end })
  }
}

impl<'a> UntypedStruct<'a> {
  pub fn new(pointer: StructPointer, pointer_end: SegmentPointer<'a>) -> Self {
    UntypedStruct { pointer: pointer, pointer_end: pointer_end }
  }

  pub fn encode_as_root_alternate<W: Write>(&self, w: &mut W) -> io::Result<()> {
    // Emit this struct's segment prefixed with a pointer to this struct.
    let root_pointer = StructPointer {
      off: self.pointer_end.off + self.pointer.off,
      data_size: self.pointer.data_size,
      pointer_size: self.pointer.pointer_size,
    }
    .encode();
    let seg_buf = self.pointer_end.seg.buf();
    let fake_len_bytes = seg_buf.len() + root_pointer.len();
    let fake_len_words = (fake_len_bytes + 7) / 8;
    let padding = vec![0; fake_len_words * 8 - fake_len_bytes];

    // WIP: What do we do about the segment id?
    w.write_all(&u32::to_le_bytes(0))?;
    w.write_all(&u32::to_le_bytes(fake_len_words as u32))?;
    w.write_all(&root_pointer)?;
    w.write_all(seg_buf)?;
    w.write_all(&padding)?;

    let mut seen_segments = HashSet::new();
    for (id, segment) in self.pointer_end.seg.all_other() {
      if seen_segments.contains(&id) {
        continue;
      }
      seen_segments.insert(id);
      UntypedStruct::encode_segment_alternate(w, &mut seen_segments, id, segment)?;
    }
    Ok(())
  }

  fn encode_segment_alternate<W: Write>(
    w: &mut W,
    seen_segments: &mut HashSet<SegmentID>,
    id: SegmentID,
    segment: SegmentBorrowed<'_>,
  ) -> io::Result<()> {
    let seg_buf = segment.buf();
    let len_bytes = seg_buf.len();
    let len_words = (len_bytes + 7) / 8;
    let padding = vec![0; len_words * 8 - len_bytes];

    w.write_all(&u32::to_le_bytes(id.0))?;
    w.write_all(&u32::to_le_bytes(len_words as u32))?;
    w.write_all(seg_buf)?;
    w.write_all(&padding)?;
    for (id, segment) in segment.all_other() {
      if seen_segments.contains(&id) {
        continue;
      }
      seen_segments.insert(id);
      UntypedStruct::encode_segment_alternate(w, seen_segments, id, segment)?;
    }
    Ok(())
  }
}

impl<'a> CapnpToOwned<'a> for UntypedStruct<'a> {
  type Owned = UntypedStructShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    UntypedStructShared {
      pointer: self.pointer.clone(),
      pointer_end: self.pointer_end.capnp_to_owned(),
    }
  }
}

impl<'a> StructDecode<'a> for UntypedStruct<'a> {
  fn pointer(&self) -> &StructPointer {
    &self.pointer
  }
  fn pointer_end(&self) -> &SegmentPointer<'a> {
    &self.pointer_end
  }
}

#[derive(Clone)]
pub struct UntypedStructShared {
  pub pointer: StructPointer,
  pub pointer_end: SegmentPointerShared,
}

impl<'a> CapnpAsRef<'a, UntypedStruct<'a>> for UntypedStructShared {
  fn capnp_as_ref(&'a self) -> UntypedStruct<'a> {
    UntypedStruct { pointer: self.pointer.clone(), pointer_end: self.pointer_end.capnp_as_ref() }
  }
}

pub struct UntypedStructOwned {
  pointer: StructPointer,
  pointer_end: SegmentPointerOwned,
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

  pub fn set_discriminant(&mut self, offset: NumElements, value: Discriminant) {
    StructEncode::set_discriminant(self, offset, value)
  }
}

impl StructEncode for UntypedStructOwned {
  fn pointer(&self) -> &StructPointer {
    &self.pointer
  }
  fn pointer_end<'a>(&'a mut self) -> SegmentPointerBorrowMut<'a> {
    self.pointer_end.borrow_mut()
  }
}
