// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::convert::TryInto;

use crate::common::{
  Discriminant, ElementWidth, NumElements, NumWords, COMPOSITE_TAG_WIDTH_WORDS,
  POINTER_WIDTH_WORDS, U16_WIDTH_BYTES, U32_WIDTH_BYTES, U64_WIDTH_BYTES, U8_WIDTH_BYTES,
};
use crate::element_type::ElementType;
use crate::error::Error;
use crate::list::{ListElementDecoding, TypedListElement, UntypedList};
use crate::pointer::{
  LandingPadSize, ListCompositeTag, ListLayout, ListPointer, Pointer, StructPointer,
};
use crate::r#struct::UntypedStruct;
use crate::segment::SegmentID;
use crate::segment_pointer::SegmentPointer;
use crate::slice::Slice;
use crate::union::UntypedUnion;

pub(crate) trait SegmentPointerDecode<'a>: Sized {
  type Segment;

  fn empty() -> Self;
  fn from_root(seg: Self::Segment) -> Self;
  fn add(self, offset: NumWords) -> Self;
  fn buf(&self) -> &[u8];
  fn offset_w(&self) -> NumWords;
  fn other(self, id: SegmentID) -> Result<Self::Segment, (Self, Error)>;

  // TODO: This gets a little nicer with const generics.
  fn u8_raw(&self, offset_e: NumElements) -> Option<[u8; U8_WIDTH_BYTES]> {
    let begin = self.offset_w().as_bytes() + offset_e.as_bytes(U8_WIDTH_BYTES);
    self
      .buf()
      .get(begin..begin + U8_WIDTH_BYTES)
      .map(|raw| raw.try_into().expect("internal logic error"))
  }

  // TODO: This gets a little nicer with const generics.
  fn u16_raw(&self, offset_e: NumElements) -> Option<[u8; U16_WIDTH_BYTES]> {
    let begin = self.offset_w().as_bytes() + offset_e.as_bytes(U16_WIDTH_BYTES);
    self
      .buf()
      .get(begin..begin + U16_WIDTH_BYTES)
      .map(|raw| raw.try_into().expect("internal logic error"))
  }

  // TODO: This gets a little nicer with const generics.
  fn u32_raw(&self, offset_e: NumElements) -> Option<[u8; U32_WIDTH_BYTES]> {
    let begin = self.offset_w().as_bytes() + offset_e.as_bytes(U32_WIDTH_BYTES);
    self
      .buf()
      .get(begin..begin + U32_WIDTH_BYTES)
      .map(|raw| raw.try_into().expect("internal logic error"))
  }

  // TODO: This gets a little nicer with const generics.
  fn u64_raw(&self, offset_e: NumElements) -> Option<[u8; U64_WIDTH_BYTES]> {
    let begin = self.offset_w().as_bytes() + offset_e.as_bytes(U64_WIDTH_BYTES);
    self
      .buf()
      .get(begin..begin + U64_WIDTH_BYTES)
      .map(|raw| raw.try_into().expect("internal logic error"))
  }

  fn u8(&self, offset_e: NumElements) -> u8 {
    self.u8_raw(offset_e).map_or(0, |raw| u8::from_le_bytes(raw))
  }

  fn u16(&self, offset_e: NumElements) -> u16 {
    self.u16_raw(offset_e).map_or(0, |raw| u16::from_le_bytes(raw))
  }

  fn u32(&self, offset_e: NumElements) -> u32 {
    self.u32_raw(offset_e).map_or(0, |raw| u32::from_le_bytes(raw))
  }

  fn u64(&self, offset_e: NumElements) -> u64 {
    self.u64_raw(offset_e).map_or(0, |raw| u64::from_le_bytes(raw))
  }

  fn pointer(&self, offset_e: NumElements) -> Pointer {
    self.u64_raw(offset_e).map_or(Pointer::Null, |raw| Pointer::decode(raw))
  }

  fn struct_pointer(self, offset_e: NumElements) -> Result<(StructPointer, Self), Error> {
    match self.pointer(offset_e) {
      Pointer::Null => Ok((StructPointer::empty(), Self::empty())),
      Pointer::Struct(x) => {
        let sp = StructPointer { off: x.off, data_size: x.data_size, pointer_size: x.pointer_size };
        let sp_end = self.add(POINTER_WIDTH_WORDS * offset_e + POINTER_WIDTH_WORDS);
        Ok((sp, sp_end))
      }
      Pointer::Far(x) => {
        let seg = self
          .other(x.seg)
          .map_err(|(_, err)| Error::Encoding(format!("far struct pointer: {}", err)))?;
        let far = Self::from_root(seg).add(x.off);
        match x.landing_pad_size {
          LandingPadSize::OneWord => {
            // If B == 0, then the “landing pad” of a far pointer is normally just
            // another pointer, which in turn points to the actual object.

            // TODO: Limit recursive call depth.
            // We already accounted for the offset in the BufPointer so pretend
            // we're reading a pointer at the very beginning of a struct's data.
            far.struct_pointer(NumElements(0))
          }
          LandingPadSize::TwoWords => {
            // If B == 1, then the “landing pad” is itself another far pointer
            // that is interpreted differently: This [far_far] pointer (which
            // always has B = 0) points to the start of the object’s content,
            // located in some other segment.

            // We already accounted for the offset in the BufPointer so pretend
            // we're reading a pointer at the very beginning of a struct's data.
            let far_far = match far.pointer(NumElements(0)) {
              Pointer::Far(x) => Ok(x),
              x => Err(Error::Encoding(format!("expected far pointer got: {:?}", x))),
            }?;
            if let LandingPadSize::TwoWords = far_far.landing_pad_size {
              return Err(Error::Encoding(format!(
                "expected one word far pointer got: {:?}",
                far_far,
              )));
            }

            // The [far_far pointer/landing pad] is itself immediately followed by
            // a tag word. The tag word looks exactly like an intra-segment
            // pointer to the target object would look, except that the offset is
            // always zero.
            let sp_template = match far.pointer(NumElements(1)) {
              Pointer::Struct(x) => x,
              x => Err(Error::Encoding(format!("expected struct pointer got: {:?}", x)))?,
            };
            let seg = far
              .other(far_far.seg)
              .map_err(|(_, err)| Error::Encoding(format!("far far struct pointer: {}", err)))?;
            let sp_end = Self::from_root(seg);
            let sp = StructPointer {
              off: far_far.off,
              data_size: sp_template.data_size,
              pointer_size: sp_template.pointer_size,
            };
            Ok((sp, sp_end))
          }
        }
      }
      x => Err(Error::Encoding(format!("expected struct pointer got: {:?}", x))),
    }
  }

  // TODO: Dedup this with fn struct_pointer
  fn list_pointer(self, offset_e: NumElements) -> Result<(ListPointer, Self), Error> {
    match self.pointer(offset_e) {
      Pointer::Null => Ok((ListPointer::empty(), Self::empty())),
      Pointer::List(lp) => {
        let lp_end = self.add(POINTER_WIDTH_WORDS * offset_e + POINTER_WIDTH_WORDS);
        Ok((lp, lp_end))
      }
      Pointer::Far(x) => {
        let seg = self
          .other(x.seg)
          .map_err(|(_, err)| Error::Encoding(format!("far list pointer: {}", err)))?;

        let far = Self::from_root(seg).add(x.off);
        match x.landing_pad_size {
          LandingPadSize::OneWord => {
            // If B == 0, then the “landing pad” of a far pointer is normally just
            // another pointer, which in turn points to the actual object.

            // TODO: Limit recursive call depth.
            // We already accounted for the offset in the BufPointer so pretend
            // we're reading a pointer at the very beginning of a struct's data.
            far.list_pointer(NumElements(0))
          }
          LandingPadSize::TwoWords => {
            // If B == 1, then the “landing pad” is itself another far pointer
            // that is interpreted differently: This [far_far] pointer (which
            // always has B = 0) points to the start of the object’s content,
            // located in some other segment.

            // We already accounted for the offset in the BufPointer so pretend
            // we're reading a pointer at the very beginning of a struct's data.
            let far_far = match far.pointer(NumElements(0)) {
              Pointer::Far(x) => Ok(x),
              x => Err(Error::Encoding(format!("expected far pointer got: {:?}", x))),
            }?;
            if let LandingPadSize::TwoWords = far_far.landing_pad_size {
              return Err(Error::Encoding(format!(
                "expected one word far pointer got: {:?}",
                far_far,
              )));
            }

            // The [far_far pointer/landing pad] is itself immediately followed by
            // a tag word. The tag word looks exactly like an intra-segment
            // pointer to the target object would look, except that the offset is
            // always zero.
            let lp_template = match far.pointer(NumElements(1)) {
              Pointer::List(x) => x,
              x => Err(Error::Encoding(format!("expected list pointer got: {:?}", x)))?,
            };
            let seg = far
              .other(far_far.seg)
              .map_err(|(_, err)| Error::Encoding(format!("far far list pointer: {}", err)))?;
            let lp_end = Self::from_root(seg);
            let lp = ListPointer { off: far_far.off, layout: lp_template.layout };
            Ok((lp, lp_end))
          }
        }
      }
      x => Err(Error::Encoding(format!("expected list pointer got: {:?}", x))),
    }
  }

  fn list_composite_tag(self) -> Result<(ListCompositeTag, Self), Error> {
    let raw = self
      .u64_raw(NumElements(0))
      .ok_or_else(|| Error::Encoding(format!("expected composite tag")))?;
    let tag = ListCompositeTag::decode(raw)?;
    let tag_end = self.add(COMPOSITE_TAG_WIDTH_WORDS);
    Ok((tag, tag_end))
  }
}

pub(crate) trait StructDecode<'a> {
  fn pointer(&self) -> &StructPointer;
  fn pointer_end(&self) -> &SegmentPointer<'a>;

  fn data_fields_begin(&self) -> SegmentPointer<'a> {
    self.pointer_end().clone().add(self.pointer().off)
  }

  fn pointer_fields_begin(&self) -> SegmentPointer<'a> {
    self.data_fields_begin().add(self.pointer().data_size)
  }

  fn bool(&self, offset_e: NumElements) -> bool {
    let byte = self.u8(NumElements(offset_e.0 >> 3));
    let bit_pos = (offset_e.0 % 8) as usize;
    byte & (1 << bit_pos) == 1
  }

  fn i32(&self, offset_e: NumElements) -> i32 {
    self.data_fields_begin().u32(offset_e) as i32
  }

  fn u8(&self, offset_e: NumElements) -> u8 {
    self.data_fields_begin().u8(offset_e)
  }

  fn u16(&self, offset_e: NumElements) -> u16 {
    self.data_fields_begin().u16(offset_e)
  }

  fn u32(&self, offset_e: NumElements) -> u32 {
    self.data_fields_begin().u32(offset_e)
  }

  fn u64(&self, offset_e: NumElements) -> u64 {
    self.data_fields_begin().u64(offset_e)
  }

  fn f32(&self, offset_e: NumElements) -> f32 {
    f32::from_le_bytes(u32::to_le_bytes(self.u32(offset_e)))
  }

  fn f64(&self, offset_e: NumElements) -> f64 {
    f64::from_le_bytes(u64::to_le_bytes(self.u64(offset_e)))
  }

  fn discriminant(&self, offset_e: NumElements) -> Discriminant {
    Discriminant(self.data_fields_begin().u16(offset_e))
  }

  fn pointer_raw(&self, offset_e: NumElements) -> Pointer {
    self.pointer_fields_begin().pointer(offset_e)
  }

  fn bytes(&self, offset_e: NumElements) -> Result<&'a [u8], Error> {
    let (pointer, pointer_end) = self.pointer_fields_begin().list_pointer(offset_e)?;
    match pointer.layout {
      ListLayout::Packed(NumElements(0), ElementWidth::Void) => Ok(&[]),
      ListLayout::Packed(num_elements, ElementWidth::OneByte) => {
        let sp = pointer_end.add(pointer.off);
        let data_begin = sp.offset_w().as_bytes();
        let data_end = data_begin + num_elements.as_bytes(U8_WIDTH_BYTES);
        sp.buf_ref().get(data_begin..data_end).ok_or_else(|| {
          Error::Encoding(format!(
            "truncated byte field had {} of {}",
            sp.buf()[data_begin..].len(),
            data_end - data_begin
          ))
        })
      }
      _ => Err(Error::Encoding(format!("unsupposed list layout for data: {:?}", pointer.layout))),
    }
  }

  fn text(&self, offset_e: NumElements) -> Result<&'a str, Error> {
    // TODO: Maybe we want a version of field access that allows the caller to
    // trust that the encoded bytes are valid UTF-8.

    // TODO: Verify that value is null-terminated.
    self
      .bytes(offset_e)
      .and_then(|bytes| std::str::from_utf8(bytes).map_err(|e| Error::Encoding(e.to_string())))
  }

  fn untyped_struct(&self, offset_e: NumElements) -> Result<UntypedStruct<'a>, Error> {
    let (pointer, pointer_end) = self.pointer_fields_begin().struct_pointer(offset_e)?;
    Ok(UntypedStruct::new(pointer, pointer_end))
  }

  fn untyped_list(&self, offset_e: NumElements) -> Result<UntypedList<'a>, Error> {
    let (pointer, pointer_end) = self.pointer_fields_begin().list_pointer(offset_e)?;
    Ok(UntypedList::new(pointer, pointer_end))
  }

  fn untyped_union(&self, offset_e: NumElements) -> UntypedUnion<'a> {
    UntypedUnion {
      discriminant: self.discriminant(offset_e),
      variant_data: UntypedStruct::new(self.pointer().clone(), self.pointer_end().clone()),
    }
  }
}

pub(crate) trait ListDecode<'a> {
  fn pointer(&self) -> &ListPointer;
  fn pointer_end(&self) -> &SegmentPointer<'a>;

  fn list_data_begin(&self) -> SegmentPointer<'a> {
    self.pointer_end().clone().add(self.pointer().off)
  }

  fn list<T: TypedListElement<'a>>(&self) -> Result<Slice<'a, T>, Error> {
    match T::decoding() {
      ListElementDecoding::Packed(element_type, decode) => self.packed_list(element_type, decode),
      ListElementDecoding::Composite(from_untyped_struct) => {
        self.composite_list(from_untyped_struct)
      }
    }
  }

  fn packed_list<T>(
    &self,
    element_type: ElementType,
    decode: fn(&SegmentPointer<'a>, NumElements) -> T,
  ) -> Result<Slice<'a, T>, Error> {
    let num_elements = match &self.pointer().layout {
      ListLayout::Packed(NumElements(0), ElementWidth::Void) => {
        // NB: NumElements(0), ElementWidth::Void is a null pointer.
        return Ok(Slice::empty());
      }
      ListLayout::Packed(num_elements, width) => {
        if width != &element_type.width() {
          return Err(Error::Encoding(format!(
            "unsupported list layout for {:?}: {:?}",
            element_type,
            self.pointer().layout
          )));
        }
        num_elements
      }
      x => {
        return Err(Error::Encoding(format!(
          "unsupported list layout for {:?}: {:?}",
          element_type, x
        )))
      }
    };
    let list_data_begin = self.list_data_begin();
    Ok(Slice::packed(*num_elements, list_data_begin, decode))
  }

  fn composite_list<T>(
    &self,
    from_untyped_struct: fn(UntypedStruct<'a>) -> T,
  ) -> Result<Slice<'a, T>, Error> {
    let pointer_declared_len = match &self.pointer().layout {
      ListLayout::Packed(NumElements(0), ElementWidth::Void) => {
        // NB: NumElements(0), ElementWidth::Void is a null pointer.
        return Ok(Slice::empty());
      }
      ListLayout::Composite(num_words) => *num_words,
      x => {
        return Err(Error::Encoding(format!("unsupported list layout for TypedStruct: {:?}", x)))
      }
    };
    let (tag, tag_end) = self.list_data_begin().list_composite_tag()?;
    let composite_len = (tag.data_size + tag.pointer_size) * tag.num_elements;
    if composite_len != pointer_declared_len {
      return Err(Error::Encoding(format!(
        "composite tag length ({:?}) doesn't agree with pointer ({:?})",
        composite_len, pointer_declared_len
      )));
    }

    Ok(Slice::composite(tag, tag_end, from_untyped_struct))
  }
}
