// Copyright 2020 Daniel Harrison. All Rights Reserved.

use crate::common::{
  Discriminant, ElementWidth, NumElements, NumWords, COMPOSITE_TAG_WIDTH_BYTES,
  POINTER_WIDTH_BYTES, POINTER_WIDTH_WORDS, U16_WIDTH_BYTES, U64_WIDTH_BYTES, U8_WIDTH_BYTES,
};
use crate::decode::SegmentPointerDecode;
use crate::element::{
  ElementShared, ListDecodedElementShared, PointerElementShared, PrimitiveElement,
  StructElementShared, UnionElementShared,
};
use crate::element_type::{ElementType, PrimitiveElementType};
use crate::error::Error;
use crate::list::{ListElementEncoding, TypedListElementShared};
use crate::pointer::{
  FarPointer, LandingPadSize, ListCompositeTag, ListLayout, ListPointer, Pointer, StructPointer,
};
use crate::r#struct::UntypedStructShared;
use crate::segment_pointer::SegmentPointerBorrowMut;

pub trait SegmentPointerEncode {
  fn buf_mut(&mut self) -> &mut [u8];
  fn ensure_len(&mut self, len_bytes: usize);
  fn offset_w(&self) -> NumWords;

  fn set_u8_raw(&mut self, off: NumWords, offset: NumElements, value: u8) {
    let begin = off.as_bytes() + offset.as_bytes(U8_WIDTH_BYTES);
    let end = begin + U8_WIDTH_BYTES;
    self.ensure_len(end);
    // NB: This range is guaranteed to exist because we just resized it.
    self.buf_mut()[begin..end].copy_from_slice(&u8::to_le_bytes(value));
  }

  fn set_u16_raw(&mut self, off: NumWords, offset: NumElements, value: u16) {
    let begin = off.as_bytes() + offset.as_bytes(U16_WIDTH_BYTES);
    let end = begin + U16_WIDTH_BYTES;
    self.ensure_len(end);
    // NB: This range is guaranteed to exist because we just resized it.
    self.buf_mut()[begin..end].copy_from_slice(&u16::to_le_bytes(value));
  }

  fn set_u64_raw(&mut self, off: NumWords, offset: NumElements, value: u64) {
    let begin = off.as_bytes() + offset.as_bytes(U64_WIDTH_BYTES);
    let end = begin + U64_WIDTH_BYTES;
    self.ensure_len(end);
    // NB: This range is guaranteed to exist because we just resized it.
    self.buf_mut()[begin..end].copy_from_slice(&u64::to_le_bytes(value));
  }

  fn set_pointer_raw(&mut self, off: NumWords, offset: NumElements, value: Pointer) {
    let begin = off.as_bytes() + offset.as_bytes(POINTER_WIDTH_BYTES);
    let end = begin + POINTER_WIDTH_BYTES;
    self.ensure_len(end);
    // NB: This range is guaranteed to exist because we just resized it.
    self.buf_mut()[begin..end].copy_from_slice(&value.encode());
  }

  fn set_u8(&mut self, offset_e: NumElements, value: u8) {
    self.set_u8_raw(self.offset_w(), offset_e, value);
  }

  fn set_u16(&mut self, offset_e: NumElements, value: u16) {
    self.set_u16_raw(self.offset_w(), offset_e, value);
  }

  fn set_u64(&mut self, offset_e: NumElements, value: u64) {
    self.set_u64_raw(self.offset_w(), offset_e, value);
  }

  fn set_pointer(&mut self, offset_e: NumElements, value: Pointer) {
    self.set_pointer_raw(self.offset_w(), offset_e, value);
  }
}

pub trait StructEncode {
  fn pointer(&self) -> &StructPointer;
  fn pointer_end<'a>(&'a mut self) -> SegmentPointerBorrowMut<'a>;

  fn data_fields_begin<'a>(&'a mut self) -> SegmentPointerBorrowMut<'a> {
    let pointer_off = self.pointer().off;
    self.pointer_end().add(pointer_off)
  }

  fn pointer_fields_begin<'a>(&'a mut self) -> SegmentPointerBorrowMut<'a> {
    let data_size = self.pointer().data_size;
    self.data_fields_begin().add(data_size)
  }

  fn set_u8(&mut self, offset_e: NumElements, value: u8) {
    // WIP: Check against self.pointer to see if we should even be setting anything.
    self.data_fields_begin().set_u8(offset_e, value)
  }

  fn set_u16(&mut self, offset_e: NumElements, value: u16) {
    // WIP: Check against self.pointer to see if we should even be setting anything.
    self.data_fields_begin().set_u16(offset_e, value)
  }

  fn set_discriminant(&mut self, offset_e: NumElements, value: Discriminant) {
    // WIP: Check against self.pointer to see if we should even be setting anything.
    self.data_fields_begin().set_u16(offset_e, value.0)
  }

  fn set_u64(&mut self, offset_e: NumElements, value: u64) {
    // WIP: Check against self.pointer to see if we should even be setting anything.
    self.data_fields_begin().set_u64(offset_e, value)
  }

  fn set_pointer(&mut self, offset_e: NumElements, value: Pointer) {
    // WIP: Check against self.pointer to see if we should even be setting anything.
    self.pointer_fields_begin().set_pointer(offset_e, value)
  }

  fn set_struct(&mut self, offset_e: NumElements, untyped: Option<&UntypedStructShared>) {
    let untyped = match untyped {
      None => return self.set_pointer(offset_e, Pointer::Null),
      Some(untyped) => untyped,
    };

    // Create a reference to the segment so the far pointer works.
    let segment_id = self.pointer_end().seg.other_reference(untyped.pointer_end.seg.clone());

    let far_pointer = Pointer::Far(FarPointer {
      landing_pad_size: LandingPadSize::OneWord,
      // NB: POINTER_WIDTH_WORDS is subtracted because a far pointer points to the
      // _beginning_ of a pointer but pointer_end points to the end of the
      // pointer.
      off: untyped.pointer.off + untyped.pointer_end.off - POINTER_WIDTH_WORDS,
      seg: segment_id,
    });

    self.set_pointer(offset_e, far_pointer);
  }

  fn set_list<T: TypedListElementShared>(&mut self, offset_e: NumElements, value: &[T]) {
    let pointer = match T::encoding() {
      ListElementEncoding::Primitive(element_type, encode) => {
        self.set_primitive_list(offset_e, element_type.width(), encode, value)
      }
      ListElementEncoding::Composite(as_untyped) => {
        self.set_composite_list(offset_e, as_untyped, value)
      }
    };
    self.set_pointer(offset_e, pointer);
  }

  fn set_primitive_list<T>(
    &mut self,
    offset_e: NumElements,
    width: ElementWidth,
    encode: fn(&mut SegmentPointerBorrowMut<'_>, NumElements, &T),
    value: &[T],
  ) -> Pointer {
    // TODO: Support encoding distinction of unset vs empty lists?
    if value.len() == 0 {
      return Pointer::Null;
    }
    let list_begin = self.pointer_end().seg.len_words_rounded_up();
    let list_len = width.list_len_bytes(value.len());
    self.pointer_end().seg.ensure_len(list_begin.as_bytes() + list_len);
    let mut list_data_begin =
      SegmentPointerBorrowMut { seg: self.pointer_end().seg, off: list_begin };
    for (idx, el) in value.iter().enumerate() {
      encode(&mut list_data_begin, NumElements(idx as i32), el);
    }
    let lp_end_off =
      self.pointer_fields_begin().add(POINTER_WIDTH_WORDS * offset_e + NumWords(1)).off;
    Pointer::List(ListPointer {
      off: list_begin - lp_end_off,
      layout: ListLayout::Packed(NumElements(value.len() as i32), width),
    })
  }

  fn set_composite_list<T: TypedListElementShared>(
    &mut self,
    offset_e: NumElements,
    as_untyped: fn(&T) -> UntypedStructShared,
    value: &[T],
  ) -> Pointer {
    let composite_tag = match value.first() {
      // TODO: Support encoding distinction of unset vs empty lists?
      None => return Pointer::Null,
      Some(first) => {
        let untyped = as_untyped(first);
        ListCompositeTag {
          num_elements: NumElements(value.len() as i32),
          data_size: untyped.pointer.data_size,
          pointer_size: untyped.pointer.pointer_size,
        }
      }
    };

    let list_begin = self.pointer_end().seg.len_words_rounded_up();
    let composite_begin = list_begin.as_bytes();
    let composite_end = composite_begin + COMPOSITE_TAG_WIDTH_BYTES;
    self.pointer_end().seg.ensure_len(composite_end);
    // NB: This range is guaranteed to exist because we just resized it.
    self.pointer_end().seg.buf_mut()[composite_begin..composite_end]
      .copy_from_slice(&composite_tag.encode());

    let one_struct_len = composite_tag.data_size + composite_tag.pointer_size;
    let len_before = self.pointer_end().seg.buf_mut().len();
    for x in value.iter() {
      let x: UntypedStructShared = as_untyped(x.clone());
      if x.pointer.data_size != composite_tag.data_size {
        // TODO: I think we can handle this by padding them out with 0s to match
        // the largest data_size in the list. Definitely needs unit tests.
        todo!(
          "struct list with mismatched data_size: {:?} vs {:?}",
          x.pointer.data_size,
          composite_tag.data_size
        );
      }
      if x.pointer.pointer_size != composite_tag.pointer_size {
        // TODO: I think we can handle this by padding them out with 0s to match
        // the largest pointer_size in the list. Definitely needs unit tests.
        todo!(
          "struct list with mismatched pointer_size: {:?} vs {:?}",
          x.pointer.data_size,
          composite_tag.data_size
        );
      }

      // Copy in the data bits unchanged.
      let src_buf = x.pointer_end.seg.buf();
      let src_begin = (x.pointer_end.off + x.pointer.off).as_bytes();
      let dst_begin = self.pointer_end().seg.buf_mut().len();
      // TODO: Do all these ensure_lens once at the top.
      self.pointer_end().seg.ensure_len(dst_begin + x.pointer.data_size.as_bytes());
      self.pointer_end().seg.buf_mut()[dst_begin..dst_begin + x.pointer.data_size.as_bytes()]
        .copy_from_slice(&src_buf[src_begin..src_begin + x.pointer.data_size.as_bytes()]);

      // Fill in the pointer bits with far pointers to the original pointers
      // (expect for null pointers, which are filled directly).
      let segment_id = self.pointer_end().seg.other_reference(x.pointer_end.seg.clone());
      for idx in 0..x.pointer.pointer_size.0 {
        // WIP: Hacks
        let pointer = x.pointer_end.as_ref().pointer(NumElements(x.pointer.data_size.0 + idx));
        let far_pointer = match pointer {
          Pointer::Null => Pointer::Null,
          _ => Pointer::Far(FarPointer {
            landing_pad_size: LandingPadSize::OneWord,
            off: x.pointer_end.off
                  + x.pointer.off
                  + x.pointer.data_size
                  // NB: Point to the beginning of it, not the end as usual.
                  + POINTER_WIDTH_WORDS * NumElements(idx),
            seg: segment_id,
          }),
        };
        let dst_begin = self.pointer_end().seg.buf_mut().len();
        // TODO: Do all these ensure_lens once at the top.
        self.pointer_end().seg.ensure_len(dst_begin + POINTER_WIDTH_BYTES);
        self.pointer_end().seg.buf_mut()[dst_begin..dst_begin + POINTER_WIDTH_BYTES]
          .copy_from_slice(&far_pointer.encode());
      }
    }
    let len_after = self.pointer_end().seg.buf_mut().len();

    let composite_len = one_struct_len * NumElements(value.len() as i32);
    debug_assert_eq!(len_after - len_before, composite_len.as_bytes());

    let lp_end_off =
      self.pointer_fields_begin().add(POINTER_WIDTH_WORDS * offset_e + NumWords(1)).off;
    Pointer::List(ListPointer {
      off: list_begin - lp_end_off,
      layout: ListLayout::Composite(composite_len),
    })
  }

  fn set_element(&mut self, offset_e: NumElements, value: &ElementShared) -> Result<(), Error> {
    match value {
      ElementShared::Primitive(x) => Ok(self.set_primitive_element(offset_e, x)),
      ElementShared::Pointer(x) => self.set_pointer_element(offset_e, x),
      ElementShared::Union(x) => self.set_union_element(offset_e, x),
    }
  }

  fn set_primitive_element(&mut self, offset_e: NumElements, value: &PrimitiveElement) {
    match value {
      PrimitiveElement::U8(x) => self.set_u8(offset_e, *x),
      PrimitiveElement::U64(x) => self.set_u64(offset_e, *x),
    }
  }

  fn set_pointer_element(
    &mut self,
    offset_e: NumElements,
    value: &PointerElementShared,
  ) -> Result<(), Error> {
    match value {
      PointerElementShared::Struct(x) => Ok(self.set_struct_element(offset_e, x)),
      PointerElementShared::ListDecoded(x) => self.set_list_decoded_element(offset_e, x),
      PointerElementShared::List(_) => todo!(),
    }
  }

  fn set_struct_element(&mut self, offset_e: NumElements, value: &StructElementShared) {
    let StructElementShared(_, untyped) = value;
    self.set_struct(offset_e, Some(untyped));
  }

  fn set_list_decoded_element(
    &mut self,
    offset_e: NumElements,
    value: &ListDecodedElementShared,
  ) -> Result<(), Error> {
    let ListDecodedElementShared(_, value) = value;
    let element_type = match value.first() {
      // TODO: Support encoding distinction of unset vs empty lists?
      None => {
        self.set_pointer(offset_e, Pointer::Null);
        return Ok(());
      }
      Some(first) => first.as_ref().element_type(),
    };

    match element_type {
      ElementType::Primitive(PrimitiveElementType::U8) => {
        let mut typed_value = Vec::with_capacity(value.len());
        for x in value.iter() {
          match x {
            ElementShared::Primitive(PrimitiveElement::U8(x)) => typed_value.push(*x),
            x => {
              return Err(Error::from(format!(
                "cannot encode {:?} list containing {:?}",
                element_type,
                x.as_ref().element_type(),
              )))
            }
          }
        }
        self.set_list(offset_e, &typed_value);
        Ok(())
      }
      element_type => {
        Err(Error::from(format!("TODO: set_list_decoded_element for {:?}", element_type)))
      }
    }
  }

  fn set_union_element(
    &mut self,
    offset_e: NumElements,
    value: &UnionElementShared,
  ) -> Result<(), Error> {
    let UnionElementShared(meta, discriminant, _value) = value;
    let variant_meta = meta.get(*discriminant).expect("WIP");
    self.set_u16(offset_e, variant_meta.discriminant.0);
    // WIP this should be breaking stuff
    // variant_meta.field_meta.set_element(self, value.as_ref()).expect("WIP");
    Ok(())
  }
}
