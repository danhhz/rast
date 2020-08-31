// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::convert::TryInto;
use std::ops::Add;

use crate::common::*;
use crate::error::Error;
use crate::pointer::*;
use crate::segment::Segment;

#[derive(Clone)]
pub struct SegmentPointer<'a> {
  pub seg: Segment<'a>,
  pub off: NumWords,
}

impl<'a> SegmentPointer<'a> {
  pub fn empty() -> Self {
    SegmentPointer { seg: Segment::empty(), off: NumWords(0) }
  }

  pub fn from_root(seg: Segment<'a>) -> Self {
    SegmentPointer { seg: seg, off: NumWords(0) }
  }

  // TODO: This gets a little nicer with const generics.
  fn u8_raw(&self, offset: NumElements) -> Option<[u8; U8_WIDTH_BYTES]> {
    let begin = self.off.as_bytes() + offset.as_bytes(U8_WIDTH_BYTES);
    self
      .seg
      .buf()
      .get(begin..begin + U8_WIDTH_BYTES)
      .map(|raw| raw.try_into().expect("internal logic error"))
  }

  // TODO: This gets a little nicer with const generics.
  fn u16_raw(&self, offset: NumElements) -> Option<[u8; U16_WIDTH_BYTES]> {
    let begin = self.off.as_bytes() + offset.as_bytes(U16_WIDTH_BYTES);
    self
      .seg
      .buf()
      .get(begin..begin + U16_WIDTH_BYTES)
      .map(|raw| raw.try_into().expect("internal logic error"))
  }

  // TODO: This gets a little nicer with const generics.
  fn u64_raw(&self, offset: NumElements) -> Option<[u8; U64_WIDTH_BYTES]> {
    let begin = self.off.as_bytes() + offset.as_bytes(U64_WIDTH_BYTES);
    self
      .seg
      .buf()
      .get(begin..begin + U64_WIDTH_BYTES)
      .map(|raw| raw.try_into().expect("internal logic error"))
  }

  pub fn u8(&self, offset: NumElements) -> u8 {
    self.u8_raw(offset).map_or(0, |raw| u8::from_le_bytes(raw))
  }

  pub fn u16(&self, offset: NumElements) -> u16 {
    self.u16_raw(offset).map_or(0, |raw| u16::from_le_bytes(raw))
  }

  pub fn u64(&self, offset: NumElements) -> u64 {
    self.u64_raw(offset).map_or(0, |raw| u64::from_le_bytes(raw))
  }

  pub fn pointer(&self, offset: NumElements) -> Pointer {
    self.u64_raw(offset).map_or(Pointer::Null, |raw| decode_pointer(raw))
  }

  pub fn struct_pointer(
    &self,
    offset: NumElements,
  ) -> Result<(StructPointer, SegmentPointer<'a>), Error> {
    let p = self.pointer(offset);
    println!("struct_pointer={:?}", p);
    match p {
      Pointer::Null => Ok((StructPointer::empty(), SegmentPointer::empty())),
      Pointer::Struct(x) => {
        let sp = StructPointer { off: x.off, data_size: x.data_size, pointer_size: x.pointer_size };
        let sp_end = self.clone() + POINTER_WIDTH_WORDS * offset + POINTER_WIDTH_WORDS;
        Ok((sp, sp_end))
      }
      Pointer::Far(x) => {
        let seg = self.seg.other(x.seg).ok_or(Error("far pointer segment not found"))?;
        let far = SegmentPointer { seg: seg, off: x.off };
        match x.landing_pad_size {
          LandingPadSize::OneWord => {
            // If B == 0, then the “landing pad” of a far pointer is normally just
            // another pointer, which in turn points to the actual object.

            // TODO: Limit recursive call depth.
            // We already accounted for the offset in the BufPointer so pretend
            // we're reading a pointer at the very beginning of a struct's data.
            // TODO: This is kinda hacky.
            far.struct_pointer(NumElements(0))
          }
          LandingPadSize::TwoWords => {
            // If B == 1, then the “landing pad” is itself another far pointer
            // that is interpreted differently: This [far_far] pointer (which
            // always has B = 0) points to the start of the object’s content,
            // located in some other segment.

            // We already accounted for the offset in the BufPointer so pretend
            // we're reading a pointer at the very beginning of a struct's data.
            // TODO: This is kinda hacky.
            let far_far = match far.pointer(NumElements(0)) {
              Pointer::Far(x) => Ok(x),
              x => {
                dbg!(x);
                Err(Error("expected far pointer"))
              }
            }?;
            if let LandingPadSize::TwoWords = far_far.landing_pad_size {
              return Err(Error("encoding error: two word landing pad not a one word far pointer"));
            }

            // The [far_far pointer/landing pad] is itself immediately followed by
            // a tag word. The tag word looks exactly like an intra-segment
            // pointer to the target object would look, except that the offset is
            // always zero.
            let (sp_template, _) = far.struct_pointer(NumElements(1))?;
            let seg =
              far.seg.other(far_far.seg).ok_or(Error("far far pointer segment not found"))?;
            let sp_end = SegmentPointer { seg: seg, off: NumWords(0) };
            let sp = StructPointer {
              off: far_far.off,
              data_size: sp_template.data_size,
              pointer_size: sp_template.pointer_size,
            };
            Ok((sp, sp_end))
          }
        }
      }
      x => {
        dbg!(x);
        Err(Error("expected struct pointer"))
      }
    }
  }

  pub fn list_pointer(
    &self,
    offset: NumElements,
  ) -> Result<(ListPointer, SegmentPointer<'a>), Error> {
    match self.pointer(offset) {
      Pointer::Null => Ok((ListPointer::empty(), SegmentPointer::empty())),
      Pointer::List(lp) => {
        let lp_end = self.clone() + POINTER_WIDTH_WORDS * offset + POINTER_WIDTH_WORDS;
        Ok((lp, lp_end))
      }
      Pointer::Far(x) => {
        dbg!(&x);
        let seg = self.seg.other(x.seg).ok_or(Error("far pointer segment not found"))?;
        let far = SegmentPointer { seg: seg, off: x.off };
        match x.landing_pad_size {
          LandingPadSize::OneWord => {
            // If B == 0, then the “landing pad” of a far pointer is normally just
            // another pointer, which in turn points to the actual object.

            // TODO: Limit recursive call depth.
            // We already accounted for the offset in the BufPointer so pretend
            // we're reading a pointer at the very beginning of a struct's data.
            // TODO: This is kinda hacky.
            far.list_pointer(NumElements(0))
          }
          LandingPadSize::TwoWords => {
            // If B == 1, then the “landing pad” is itself another far pointer
            // that is interpreted differently: This [far_far] pointer (which
            // always has B = 0) points to the start of the object’s content,
            // located in some other segment.

            // We already accounted for the offset in the BufPointer so pretend
            // we're reading a pointer at the very beginning of a struct's data.
            // TODO: This is kinda hacky.
            let far_far = match far.pointer(NumElements(0)) {
              Pointer::Far(x) => Ok(x),
              x => {
                dbg!(x);
                Err(Error("expected far pointer"))
              }
            }?;
            if let LandingPadSize::TwoWords = far_far.landing_pad_size {
              return Err(Error("encoding error: two word landing pad not a one word far pointer"));
            }

            // The [far_far pointer/landing pad] is itself immediately followed by
            // a tag word. The tag word looks exactly like an intra-segment
            // pointer to the target object would look, except that the offset is
            // always zero.
            let (lp_template, _) = far.list_pointer(NumElements(1))?;
            let seg =
              far.seg.other(far_far.seg).ok_or(Error("far far pointer segment not found"))?;
            let lp_end = SegmentPointer { seg: seg, off: NumWords(0) };
            let lp = ListPointer { off: far_far.off, layout: lp_template.layout };
            Ok((lp, lp_end))
          }
        }
      }
      _ => Err(Error("expected list pointer")),
    }
  }

  pub fn list_composite_tag(&self) -> Result<(ListCompositeTag, SegmentPointer<'a>), Error> {
    let raw = self.u64_raw(NumElements(0)).ok_or(Error("expected composite tag"))?;
    let tag = decode_composite_tag(raw)?;
    let tag_end = self.clone() + POINTER_WIDTH_WORDS;
    Ok((tag, tag_end))
  }
}

impl<'a> Add<NumWords> for SegmentPointer<'a> {
  type Output = SegmentPointer<'a>;
  fn add(self, other: NumWords) -> SegmentPointer<'a> {
    SegmentPointer { seg: self.seg, off: self.off + other }
  }
}
