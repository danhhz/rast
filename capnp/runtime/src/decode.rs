// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::convert::TryInto;

use crate::common::{
  Discriminant, ElementWidth, NumElements, NumWords, COMPOSITE_TAG_WIDTH_WORDS,
  POINTER_WIDTH_WORDS, U16_WIDTH_BYTES, U64_WIDTH_BYTES, U8_WIDTH_BYTES,
};
use crate::element_type::PrimitiveElementType;
use crate::error::Error;
use crate::list::{ListElementDecoding, TypedListElement, UntypedList};
use crate::pointer::{
  LandingPadSize, ListCompositeTag, ListLayout, ListPointer, Pointer, StructPointer,
};
use crate::r#struct::UntypedStruct;
use crate::segment::{Segment, SegmentID};
use crate::segment_pointer::SegmentPointer;
use crate::union::UntypedUnion;

pub trait SegmentPointerDecode<'a>: Sized {
  fn empty() -> Self;
  fn from_root(seg: Segment<'a>) -> Self;
  fn add(&self, offset: NumWords) -> Self;
  fn buf(&self) -> &[u8];
  fn offset_w(&self) -> NumWords;
  fn other(&self, id: SegmentID) -> Option<Segment<'a>>;
  fn all_other(&self) -> Vec<(SegmentID, Segment<'a>)>;

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

  fn u64(&self, offset_e: NumElements) -> u64 {
    self.u64_raw(offset_e).map_or(0, |raw| u64::from_le_bytes(raw))
  }

  fn pointer(&self, offset_e: NumElements) -> Pointer {
    self.u64_raw(offset_e).map_or(Pointer::Null, |raw| Pointer::decode(raw))
  }

  fn struct_pointer(&self, offset_e: NumElements) -> Result<(StructPointer, Self), Error> {
    match self.pointer(offset_e) {
      Pointer::Null => Ok((StructPointer::empty(), Self::empty())),
      Pointer::Struct(x) => {
        let sp = StructPointer { off: x.off, data_size: x.data_size, pointer_size: x.pointer_size };
        let sp_end = self.add(POINTER_WIDTH_WORDS * offset_e + POINTER_WIDTH_WORDS);
        Ok((sp, sp_end))
      }
      Pointer::Far(x) => {
        let seg = self.other(x.seg).ok_or_else(|| {
          Error::from(format!(
            "encoding: far struct pointer segment {:?} not found in {:?}",
            x.seg,
            self.all_other().iter().map(|x| x.0).collect::<Vec<_>>()
          ))
        })?;
        let far = Self::from_root(seg).add(x.off);
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
              x => Err(Error::from(format!("encoding: expected far pointer got: {:?}", x))),
            }?;
            if let LandingPadSize::TwoWords = far_far.landing_pad_size {
              return Err(Error::from(format!(
                "encoding: expected one word far pointer got: {:?}",
                far_far,
              )));
            }

            // The [far_far pointer/landing pad] is itself immediately followed by
            // a tag word. The tag word looks exactly like an intra-segment
            // pointer to the target object would look, except that the offset is
            // always zero.
            let (sp_template, _) = far.struct_pointer(NumElements(1))?;
            let seg = far.other(far_far.seg).ok_or_else(|| {
              Error::from(format!("encoding: far far pointer segment {:?} not found", far_far.seg))
            })?;
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
      x => Err(Error::from(format!("encoding: expected struct pointer got: {:?}", x))),
    }
  }

  // TODO: Dedup this with fn struct_pointer
  fn list_pointer(&self, offset_e: NumElements) -> Result<(ListPointer, Self), Error> {
    match self.pointer(offset_e) {
      Pointer::Null => Ok((ListPointer::empty(), Self::empty())),
      Pointer::List(lp) => {
        let lp_end = self.add(POINTER_WIDTH_WORDS * offset_e + POINTER_WIDTH_WORDS);
        Ok((lp, lp_end))
      }
      Pointer::Far(x) => {
        let seg = self.other(x.seg).ok_or_else(|| {
          Error::from(format!("encoding: far list pointer segment {:?} not found", x.seg))
        })?;
        let far = Self::from_root(seg).add(x.off);
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
              x => Err(Error::from(format!("encoding: expected far pointer got: {:?}", x))),
            }?;
            if let LandingPadSize::TwoWords = far_far.landing_pad_size {
              return Err(Error::from(format!(
                "encoding: expected one word far pointer got: {:?}",
                far_far,
              )));
            }

            // The [far_far pointer/landing pad] is itself immediately followed by
            // a tag word. The tag word looks exactly like an intra-segment
            // pointer to the target object would look, except that the offset is
            // always zero.
            let (lp_template, _) = far.list_pointer(NumElements(1))?;
            let seg = far.other(far_far.seg).ok_or_else(|| {
              Error::from(format!("encoding: far far pointer segment {:?} not found", far_far.seg))
            })?;
            let lp_end = Self::from_root(seg);
            let lp = ListPointer { off: far_far.off, layout: lp_template.layout };
            Ok((lp, lp_end))
          }
        }
      }
      x => Err(Error::from(format!("encoding: expected list pointer got: {:?}", x))),
    }
  }

  fn list_composite_tag(&self) -> Result<(ListCompositeTag, Self), Error> {
    let raw = self
      .u64_raw(NumElements(0))
      .ok_or_else(|| Error::from("encoding: expected composite tag"))?;
    let tag = ListCompositeTag::decode(raw)?;
    let tag_end = self.add(COMPOSITE_TAG_WIDTH_WORDS);
    Ok((tag, tag_end))
  }
}

pub trait StructDecode<'a> {
  fn pointer(&self) -> &StructPointer;
  fn pointer_end(&self) -> &SegmentPointer<'a>;

  fn data_fields_begin(&self) -> SegmentPointer<'a> {
    self.pointer_end().add(self.pointer().off)
  }

  fn pointer_fields_begin(&self) -> SegmentPointer<'a> {
    self.data_fields_begin().add(self.pointer().data_size)
  }

  fn u64(&self, offset_e: NumElements) -> u64 {
    self.data_fields_begin().u64(offset_e)
  }

  fn pointer_raw(&self, offset_e: NumElements) -> Pointer {
    self.pointer_fields_begin().pointer(offset_e)
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
    let discriminant = Discriminant(self.data_fields_begin().u16(offset_e));
    UntypedUnion {
      discriminant: discriminant,
      variant_data: UntypedStruct::new(self.pointer().clone(), self.pointer_end().clone()),
    }
  }
}

pub trait ListDecode<'a> {
  fn pointer(&self) -> &ListPointer;
  fn pointer_end(&self) -> &SegmentPointer<'a>;

  fn list_data_begin(&self) -> SegmentPointer<'a> {
    self.pointer_end().add(self.pointer().off)
  }

  fn list<T: TypedListElement<'a>>(&self) -> Result<Vec<T>, Error> {
    match T::decoding() {
      ListElementDecoding::Primitive(element_type, decode) => {
        self.primitive_list(element_type, decode)
      }
      ListElementDecoding::Composite(from_untyped_struct) => {
        self.composite_list(from_untyped_struct)
      }
    }
  }

  fn primitive_list<T>(
    &self,
    element_type: PrimitiveElementType,
    decode: fn(&SegmentPointer<'a>, NumElements) -> T,
  ) -> Result<Vec<T>, Error> {
    let num_elements = match &self.pointer().layout {
      ListLayout::Packed(NumElements(0), ElementWidth::Void) => {
        // NB: NumElements(0), ElementWidth::Void is a null pointer.
        return Ok(vec![]);
      }
      ListLayout::Packed(num_elements, width) => {
        if width != &element_type.width() {
          return Err(Error::from(format!(
            "unsupported list layout for {:?}: {:?}",
            element_type,
            self.pointer().layout
          )));
        }
        num_elements
      }
      x => {
        return Err(Error::from(format!("unsupported list layout for {:?}: {:?}", element_type, x)))
      }
    };
    let list_data_begin = self.list_data_begin();
    let mut ret = Vec::new();
    for idx in 0..num_elements.0 {
      ret.push(decode(&list_data_begin, NumElements(idx)));
    }
    Ok(ret)
  }

  fn composite_list<T>(
    &self,
    from_untyped_struct: fn(UntypedStruct<'a>) -> T,
  ) -> Result<Vec<T>, Error> {
    let pointer_declared_len = match &self.pointer().layout {
      ListLayout::Composite(num_words) => *num_words,
      x => return Err(Error::from(format!("unsupported list layout for TypedStruct: {:?}", x))),
    };
    let (tag, tag_end) = self.list_data_begin().list_composite_tag()?;
    let composite_len = (tag.data_size + tag.pointer_size) * tag.num_elements;
    if composite_len != pointer_declared_len {
      return Err(Error::from(format!(
        "composite tag length ({:?}) doesn't agree with pointer ({:?})",
        composite_len, pointer_declared_len
      )));
    }

    let mut ret = Vec::with_capacity(tag.num_elements.0 as usize);
    for idx in 0..tag.num_elements.0 {
      let offset_w = (tag.data_size + tag.pointer_size) * NumElements(idx);
      let pointer =
        StructPointer { off: offset_w, data_size: tag.data_size, pointer_size: tag.pointer_size };
      ret.push(from_untyped_struct(UntypedStruct::new(pointer, tag_end.clone())));
    }
    Ok(ret)
  }
}
