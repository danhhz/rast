// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::hash::Hasher;
use std::io::{self, Write};

use crate::common::*;
use crate::error::Error;
use crate::pointer::{FarPointer, LandingPadSize, ListPointer, Pointer, StructPointer};
use crate::reflect::list::ListEncoder;
use crate::reflect::{
  ElementShared, ListDecodedElementShared, PointerElementShared, PrimitiveElement,
  StructElementShared, UnionElementShared,
};
use crate::segment::{Segment, SegmentID, SegmentOwned};
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

impl<'a> UntypedStruct<'a> {
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
    segment: Segment<'_>,
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

  pub fn set_u8(&mut self, offset: NumElements, value: u8) {
    // WIP: Check against self.pointer to see if we should even be setting anything.
    self.pointer_end.set_u8(offset, value)
  }

  pub fn set_u16(&mut self, offset: NumElements, value: u16) {
    // WIP: Check against self.pointer to see if we should even be setting anything.
    self.pointer_end.set_u16(offset, value)
  }

  pub fn set_discriminant(&mut self, offset: NumElements, value: Discriminant) {
    // WIP: Check against self.pointer to see if we should even be setting anything.
    self.pointer_end.set_u16(offset, value.0)
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

  pub fn set_element(&mut self, offset: NumElements, value: &ElementShared) {
    match value {
      ElementShared::Primitive(x) => self.set_primitive_element(offset, x),
      ElementShared::Pointer(x) => self.set_pointer_element(offset, x),
      ElementShared::Union(x) => self.set_union_element(offset, x),
    }
  }

  pub fn set_primitive_element(&mut self, offset: NumElements, value: &PrimitiveElement) {
    match value {
      PrimitiveElement::U8(x) => self.set_u8(offset, *x),
      PrimitiveElement::U64(x) => self.set_u64(offset, *x),
    }
  }

  pub fn set_pointer_element(&mut self, offset: NumElements, value: &PointerElementShared) {
    match value {
      PointerElementShared::Struct(x) => self.set_struct_element(offset, x),
      PointerElementShared::ListDecoded(x) => self.set_list_decoded_element(offset, x),
      PointerElementShared::List(_) => todo!(),
    }
  }

  pub fn set_struct_element(&mut self, offset: NumElements, value: &StructElementShared) {
    let StructElementShared(_, untyped) = value;

    // Create a reference to the segment so the far pointer works.
    let segment_id = {
      let mut h = DefaultHasher::new();
      // WIP: Box so this is stable
      h.write_usize(untyped.pointer_end.seg.buf().as_ptr() as usize);
      SegmentID(h.finish() as u32)
    };
    self.pointer_end.seg.other.insert(segment_id, untyped.pointer_end.seg.clone());
    // WIP: Is this really needed? Makes things O(n^2).
    self.pointer_end.seg.other.extend(untyped.pointer_end.seg.all_other());

    let far_pointer = Pointer::Far(FarPointer {
      landing_pad_size: LandingPadSize::OneWord,
      // NB: POINTER_WIDTH_WORDS is subtracted because a far pointer points to the
      // _beginning_ of a pointer but pointer_end points to the end of the
      // pointer.
      off: untyped.pointer.off + untyped.pointer_end.off - POINTER_WIDTH_WORDS,
      seg: segment_id,
    });

    self.set_pointer(offset, far_pointer);
  }

  pub fn append_list<T: ListEncoder>(&mut self, offset: NumElements, value: &T) -> Pointer {
    let pointer = value.encode(&mut self.pointer_end.seg).expect("WIP");
    match pointer {
      Pointer::Null => Pointer::Null,
      Pointer::List(x) => {
        let lp_end_off = self.pointer_end.off
          + self.pointer.off
          + self.pointer.data_size
          + NumWords(offset.0)
          + POINTER_WIDTH_WORDS;
        let lp = ListPointer { off: x.off - lp_end_off, layout: x.layout };
        Pointer::List(lp)
      }
      _ => unreachable!(),
    }
  }

  // WIP This doesn't work for lists of lists.
  pub fn set_list_decoded_element(
    &mut self,
    offset: NumElements,
    value: &ListDecodedElementShared,
  ) {
    let pointer = self.append_list(offset, value);
    self.set_pointer(offset, pointer);
  }

  pub fn set_union_element(&mut self, offset: NumElements, value: &UnionElementShared) {
    let UnionElementShared(meta, discriminant, value) = value;
    let variant_meta = meta.get(*discriminant).expect("WIP");
    self.set_u16(offset, variant_meta.discriminant.0);
    variant_meta.field_meta.set_element(self, value.as_ref()).expect("WIP");
  }
}

pub struct UntypedList<'a> {
  pub pointer: ListPointer,
  pub pointer_end: SegmentPointer<'a>,
}

pub struct UntypedListShared {
  pub pointer: ListPointer,
  pub pointer_end: SegmentPointerShared,
}

impl UntypedListShared {
  pub fn as_ref<'a>(&'a self) -> UntypedList<'a> {
    UntypedList { pointer: self.pointer.clone(), pointer_end: self.pointer_end.as_ref() }
  }
}

pub struct UntypedUnion<'a> {
  pub discriminant: Discriminant,
  pub variant_data: UntypedStruct<'a>,
}
