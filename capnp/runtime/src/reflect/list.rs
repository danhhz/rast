// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

use crate::common::{ElementWidth, NumElements, POINTER_WIDTH_BYTES, POINTER_WIDTH_WORDS};
use crate::common::{U64_WIDTH_BYTES, U8_WIDTH_BYTES};
use crate::error::Error;
use crate::pointer::{
  FarPointer, LandingPadSize, ListCompositeTag, ListLayout, ListPointer, Pointer,
};
use crate::reflect::{
  ElementShared, ElementType, ListDecodedElementShared, PointerElementShared, PointerElementType,
  PrimitiveElement, PrimitiveElementType, StructElementShared, StructElementType, TypedListShared,
  TypedStructShared,
};
use crate::segment::SegmentID;
use crate::segment::SegmentOwned;
use crate::untyped::UntypedStructOwned;

pub trait ListEncoder {
  // TODO: Make the infallable ones return error type Infallible.
  fn encode(&self, seg: &mut SegmentOwned) -> Result<Pointer, Error>;
}

impl TypedListShared for &[u8] {
  fn set(&self, data: &mut UntypedStructOwned, offset: NumElements) {
    let pointer = data.append_list(offset, self);
    data.set_pointer(offset, pointer);
  }
}

impl TypedListShared for &[u64] {
  fn set(&self, data: &mut UntypedStructOwned, offset: NumElements) {
    let pointer = data.append_list(offset, self);
    data.set_pointer(offset, pointer);
  }
}

impl<T: TypedStructShared> TypedListShared for &[&T] {
  fn set(&self, data: &mut UntypedStructOwned, offset: NumElements) {
    let pointer = data.append_list(offset, self);
    data.set_pointer(offset, pointer);
  }
}

impl ListEncoder for &[u8] {
  fn encode(&self, seg: &mut SegmentOwned) -> Result<Pointer, Error> {
    let list_begin = seg.len_words_rounded_up();
    let list_len = self.len() * U8_WIDTH_BYTES;
    // TODO: Segments should always stay word-sized.
    seg.buf.resize(list_begin.as_bytes() + list_len, 0);
    for (idx, el) in self.iter().enumerate() {
      seg.set_u8(list_begin, NumElements(idx as i32), *el);
    }
    Ok(Pointer::List(ListPointer {
      off: list_begin,
      layout: ListLayout::Packed(NumElements(self.len() as i32), ElementWidth::OneByte),
    }))
  }
}

impl ListEncoder for &[u64] {
  fn encode(&self, seg: &mut SegmentOwned) -> Result<Pointer, Error> {
    let list_begin = seg.len_words_rounded_up();
    let list_len = self.len() * U64_WIDTH_BYTES;
    seg.buf.resize(list_begin.as_bytes() + list_len, 0);
    for (idx, el) in self.iter().enumerate() {
      seg.set_u64(list_begin, NumElements(idx as i32), *el);
    }
    Ok(Pointer::List(ListPointer {
      off: list_begin,
      layout: ListLayout::Packed(
        NumElements(self.len() as i32),
        ElementWidth::EightBytesNonPointer,
      ),
    }))
  }
}

fn encode_packed_primitive(
  seg: &mut SegmentOwned,
  element_type: PrimitiveElementType,
  data: &[ElementShared],
) -> Result<Pointer, Error> {
  // NB: We accept PrimitiveElementType, all of which are supported by the
  // packed encoding.
  let element_type = ElementType::Primitive(element_type);
  let element_width = element_type.width();
  let list_begin = seg.len_words_rounded_up();
  let list_len = element_width.list_len_bytes(data.len());
  seg.buf.resize(list_begin.as_bytes() + list_len, 0);

  for (idx, el) in data.iter().enumerate() {
    if el.as_ref().element_type() != element_type {
      return Err(Error::from(format!(
        "cannot encode {:?} list containing {:?}",
        element_type,
        el.as_ref().element_type(),
      )));
    }
    match el {
      ElementShared::Primitive(x) => match x {
        PrimitiveElement::U8(x) => seg.set_u8(list_begin, NumElements(idx as i32), *x),
        PrimitiveElement::U64(x) => seg.set_u64(list_begin, NumElements(idx as i32), *x),
      },
      value => {
        return Err(Error::from(format!(
          "cannot encode packed list containing {:?}",
          value.as_ref().element_type(),
        )))
      }
    }
  }
  Ok(Pointer::List(ListPointer {
    off: list_begin,
    layout: ListLayout::Packed(NumElements(data.len() as i32), element_width),
  }))
}

fn encode_composite_struct(
  seg: &mut SegmentOwned,
  element_type: StructElementType,
  data: &[ElementShared],
) -> Result<Pointer, Error> {
  // TODO: This will truncate data in a struct that came from a later version of
  // this schema. Needs a test.
  let composite_tag = ListCompositeTag {
    num_elements: NumElements(data.len() as i32),
    data_size: element_type.meta.data_size,
    pointer_size: element_type.meta.pointer_size,
  };

  let list_begin = seg.len_words_rounded_up();
  let composite_begin = list_begin.as_bytes();
  let composite_end = composite_begin + POINTER_WIDTH_BYTES;
  seg.buf.resize(composite_end, 0);
  // NB: This range is guaranteed to exist because we just resized it.
  seg.buf[composite_begin..composite_end].copy_from_slice(&composite_tag.encode());

  let one_struct_len = composite_tag.data_size + composite_tag.pointer_size;
  let len_before = seg.buf.len();
  for x in data.iter() {
    let x = match x {
      // TODO: Verify meta matches?
      ElementShared::Pointer(PointerElementShared::Struct(StructElementShared(_, x))) => x,
      _ => {
        return Err(Error::from(format!(
          "set struct list unsupported_type: {:?}",
          x.as_ref().element_type()
        )))
      }
    };
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
    seg.buf.extend(&src_buf[src_begin..src_begin + x.pointer.data_size.as_bytes()]);

    // Fill in the pointer bits with far pointers to the original pointers
    // (expect for null pointers, which are filled directly).
    let segment_id = {
      let mut h = DefaultHasher::new();
      h.write_usize(x.pointer_end.seg.buf().as_ptr() as usize);
      SegmentID(h.finish() as u32)
    };
    let pointers_begin = src_begin + x.pointer.data_size.as_bytes();
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
      seg.buf.extend(&far_pointer.encode());
    }

    // Create segment references so these far pointers still work.
    seg.other.insert(segment_id, x.pointer_end.seg.clone());
    seg.other.extend(x.pointer_end.seg.all_other());
  }
  let len_after = seg.buf.len();

  let composite_len = one_struct_len * NumElements(data.len() as i32);
  debug_assert_eq!(len_after - len_before, composite_len.as_bytes());
  let pointer =
    Pointer::List(ListPointer { off: list_begin, layout: ListLayout::Composite(composite_len) });

  Ok(pointer)
}

impl ListEncoder for &[ElementShared] {
  fn encode(&self, seg: &mut SegmentOwned) -> Result<Pointer, Error> {
    let element_type = match self.first() {
      Some(x) => x.as_ref().element_type(),
      None => {
        // TODO: Support encoding distinction of unset vs empty lists?
        return Ok(Pointer::Null);
      }
    };
    match element_type {
      ElementType::Primitive(x) => encode_packed_primitive(seg, x, self),
      ElementType::Pointer(x) => match x {
        PointerElementType::List(_) => todo!(),
        PointerElementType::Struct(x) => encode_composite_struct(seg, x, self),
      },
      ElementType::Union(_) => todo!(),
    }
  }
}

impl<'a> ListEncoder for ListDecodedElementShared {
  fn encode(&self, seg: &mut SegmentOwned) -> Result<Pointer, Error> {
    // TODO: Check that the resulting pointer matches meta?
    let ListDecodedElementShared(_, values) = self;
    values.as_slice().encode(seg)
  }
}

// WIP: Dedup with encode_composite_struct
impl<T: TypedStructShared> ListEncoder for &[&T] {
  fn encode(&self, seg: &mut SegmentOwned) -> Result<Pointer, Error> {
    let composite_tag = if let Some(first) = self.first() {
      let untyped = first.as_untyped();
      ListCompositeTag {
        num_elements: NumElements(self.len() as i32),
        data_size: untyped.pointer.data_size,
        pointer_size: untyped.pointer.pointer_size,
      }
    } else {
      return Ok(Pointer::Null);
    };

    let list_begin = seg.len_words_rounded_up();
    let composite_begin = list_begin.as_bytes();
    let composite_end = composite_begin + POINTER_WIDTH_BYTES;
    seg.buf.resize(composite_end, 0);
    // NB: This range is guaranteed to exist because we just resized it.
    seg.buf[composite_begin..composite_end].copy_from_slice(&composite_tag.encode());

    let one_struct_len = composite_tag.data_size + composite_tag.pointer_size;
    let len_before = seg.buf.len();
    for x in self.iter() {
      let x = x.as_untyped();
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
      seg.buf.extend(&src_buf[src_begin..src_begin + x.pointer.data_size.as_bytes()]);

      // Fill in the pointer bits with far pointers to the original pointers
      // (expect for null pointers, which are filled directly).
      let segment_id = {
        let mut h = DefaultHasher::new();
        h.write_usize(x.pointer_end.seg.buf().as_ptr() as usize);
        SegmentID(h.finish() as u32)
      };
      let pointers_begin = src_begin + x.pointer.data_size.as_bytes();
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
        seg.buf.extend(&far_pointer.encode());
      }

      // Create segment references so these far pointers still work.
      seg.other.extend(x.pointer_end.seg.all_other());
      seg.other.insert(segment_id, x.pointer_end.seg);
    }
    let len_after = seg.buf.len();

    let composite_len = one_struct_len * NumElements(self.len() as i32);
    debug_assert_eq!(len_after - len_before, composite_len.as_bytes());
    let pointer =
      Pointer::List(ListPointer { off: list_begin, layout: ListLayout::Composite(composite_len) });

    Ok(pointer)
  }
}
