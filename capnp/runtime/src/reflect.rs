// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

use crate::common::{NumElements, NumWords, POINTER_WIDTH_BYTES, POINTER_WIDTH_WORDS};
use crate::error::Error;
use crate::pointer::{
  FarPointer, LandingPadSize, ListCompositeTag, ListLayout, ListPointer, Pointer, StructPointer,
};
use crate::reflect::list::ListEncoder;
use crate::segment::{SegmentID, SegmentOwned};
use crate::untyped::{UntypedList, UntypedStruct, UntypedStructOwned, UntypedStructShared};

mod element;
pub use element::*;

mod list;
pub use list::*;

pub struct StructMeta {
  pub name: &'static str,
  pub fields: &'static [FieldMeta],
}

pub trait TypedStruct<'a> {
  fn meta(&self) -> &'static StructMeta;
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self;
  fn to_untyped(&self) -> UntypedStruct<'a>;
}

impl<'a, T: TypedStruct<'a>> TypedListElement<'a> for T {
  fn from_untyped_list(untyped: UntypedList<'a>) -> Result<Vec<Self>, Error> {
    let pointer_declared_len = match untyped.pointer.layout {
      ListLayout::Composite(num_words) => num_words,
      x => return Err(Error::from(format!("unsupported list layout for TypedStruct: {:?}", x))),
    };
    let list_elements_begin = untyped.pointer_end + untyped.pointer.off;
    let (tag, tag_end) = list_elements_begin.list_composite_tag()?;
    let composite_len = (tag.data_size + tag.pointer_size) * tag.num_elements;
    if composite_len != pointer_declared_len {
      return Err(Error::from(format!(
        "composite tag length ({:?}) doesn't agree with pointer ({:?})",
        composite_len, pointer_declared_len
      )));
    }

    let mut ret = Vec::with_capacity(tag.num_elements.0 as usize);
    for idx in 0..tag.num_elements.0 {
      let off = (tag.data_size + tag.pointer_size) * NumElements(idx);
      let pointer =
        StructPointer { off: off, data_size: tag.data_size, pointer_size: tag.pointer_size };
      let untyped = UntypedStruct { pointer: pointer, pointer_end: tag_end.clone() };
      ret.push(T::from_untyped_struct(untyped));
    }
    Ok(ret)
  }
}

pub trait TypedStructShared {
  fn to_untyped(&self) -> UntypedStructShared;
}

impl<T: TypedStructShared> ListEncoder for &[T] {
  fn append(&self, seg: &mut SegmentOwned) -> Pointer {
    let composite_tag = if let Some(first) = self.first() {
      let untyped = first.to_untyped();
      ListCompositeTag {
        num_elements: NumElements(self.len() as i32),
        data_size: untyped.pointer.data_size,
        pointer_size: untyped.pointer.pointer_size,
      }
    } else {
      return Pointer::Null;
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
      let x = x.to_untyped();
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
      seg.other.insert(segment_id, x.pointer_end.seg);
    }
    let len_after = seg.buf.len();

    let composite_len = one_struct_len * NumElements(self.len() as i32);
    debug_assert_eq!(len_after - len_before, composite_len.as_bytes());
    let pointer =
      Pointer::List(ListPointer { off: list_begin, layout: ListLayout::Composite(composite_len) });

    println!("created struct list pointer {:?}\n  {:?}", &pointer, seg.buf);
    pointer
  }
}

pub enum FieldMeta {
  Primitive(PrimitiveFieldMeta),
  Pointer(PointerFieldMeta),
}

impl FieldMeta {
  pub fn name(&self) -> &'static str {
    match self {
      FieldMeta::Primitive(x) => x.name(),
      FieldMeta::Pointer(x) => x.name(),
    }
  }
  pub fn untyped_get(&self, data: &UntypedStruct<'_>, sink: &mut dyn ElementSink) {
    match self {
      FieldMeta::Primitive(x) => x.untyped_get(data, sink),
      FieldMeta::Pointer(x) => x.untyped_get(data, sink),
    }
  }
}

pub enum PrimitiveFieldMeta {
  U64(U64FieldMeta),
}

impl PrimitiveFieldMeta {
  pub fn name(&self) -> &'static str {
    match self {
      PrimitiveFieldMeta::U64(x) => x.name,
    }
  }

  pub fn untyped_get(&self, data: &UntypedStruct<'_>, sink: &mut dyn ElementSink) {
    match self {
      PrimitiveFieldMeta::U64(x) => sink.u64(x.get(data)),
    }
  }
}

pub enum PointerFieldMeta {
  Struct(StructFieldMeta),
  List(ListFieldMeta),
}

impl PointerFieldMeta {
  pub fn name(&self) -> &'static str {
    match self {
      PointerFieldMeta::Struct(x) => x.name,
      PointerFieldMeta::List(x) => x.name,
    }
  }

  pub fn untyped_get(&self, data: &UntypedStruct<'_>, sink: &mut dyn ElementSink) {
    match self {
      PointerFieldMeta::Struct(x) => sink.untyped_struct((x.meta)(), x.get_untyped(data)),
      PointerFieldMeta::List(x) => (x.get_element)(data, sink),
    }
  }

  pub fn is_null(&self, data: &UntypedStruct<'_>) -> bool {
    match self {
      PointerFieldMeta::Struct(x) => x.is_null(data),
      PointerFieldMeta::List(x) => x.is_null(data),
    }
  }
}

pub struct U64FieldMeta {
  pub name: &'static str,
  pub offset: NumElements,
}

impl U64FieldMeta {
  pub fn get(&self, data: &UntypedStruct<'_>) -> u64 {
    let data_fields_begin = data.pointer_end.clone() + data.pointer.off;
    data_fields_begin.u64(self.offset)
  }

  pub fn set(&self, data: &mut UntypedStructOwned, value: u64) {
    data.set_u64(self.offset, value);
  }
}

pub struct StructFieldMeta {
  pub name: &'static str,
  pub offset: NumElements,
  pub meta: fn() -> &'static StructMeta,
}

impl StructFieldMeta {
  pub fn is_null(&self, data: &UntypedStruct<'_>) -> bool {
    let pointer_fields_begin = data.pointer_end.clone() + data.pointer.off + data.pointer.data_size;
    match pointer_fields_begin.pointer(self.offset) {
      Pointer::Null => true,
      _ => false,
    }
  }

  pub fn get_untyped<'a>(&self, data: &UntypedStruct<'a>) -> Result<UntypedStruct<'a>, Error> {
    let pointer_fields_begin = data.pointer_end.clone() + data.pointer.off + data.pointer.data_size;
    let (pointer, pointer_end) = pointer_fields_begin.struct_pointer(self.offset)?;
    Ok(UntypedStruct { pointer: pointer, pointer_end: pointer_end })
  }

  pub fn get<'a, T: TypedStruct<'a>>(&self, data: &UntypedStruct<'a>) -> Result<T, Error> {
    self.get_untyped(data).map(|x| T::from_untyped_struct(x))
  }

  pub fn set<T: TypedStructShared>(&self, data: &mut UntypedStructOwned, value: Option<T>) {
    let pointer = match value {
      None => Pointer::Null,
      Some(value) => {
        let untyped = value.to_untyped();

        // Create a reference to the segment so the far pointer works.
        let segment_id = {
          let mut h = DefaultHasher::new();
          // WIP: Box so this is stable
          h.write_usize(data.pointer_end.seg.buf.as_slice().as_ptr() as usize);
          SegmentID(h.finish() as u32)
        };
        data.pointer_end.seg.other.insert(segment_id, untyped.pointer_end.seg);

        let far_pointer = Pointer::Far(FarPointer {
          landing_pad_size: LandingPadSize::OneWord,
          off: data.pointer_end.off - POINTER_WIDTH_WORDS,
          seg: segment_id,
        });
        println!(
          "created far pointer to {:?} {:?}\n  {:?}",
          &data.pointer, &far_pointer, data.pointer_end.seg.buf,
        );

        far_pointer
      }
    };

    data.set_pointer(self.offset, pointer);
  }
}

pub struct ListFieldMeta {
  pub name: &'static str,
  pub offset: NumElements,
  // WIP: Make this a meta to mirror StructMeta
  // WIP: This should be able to distinguish between a capnp data field and
  // repeated u8
  pub get_element: fn(&UntypedStruct<'_>, &mut dyn ElementSink),
}

impl ListFieldMeta {
  pub fn is_null(&self, data: &UntypedStruct<'_>) -> bool {
    let pointer_fields_begin = data.pointer_end.clone() + data.pointer.off + data.pointer.data_size;
    match pointer_fields_begin.pointer(self.offset) {
      Pointer::Null => true,
      _ => false,
    }
  }

  pub fn get_untyped<'a>(&self, data: &UntypedStruct<'a>) -> Result<UntypedList<'a>, Error> {
    let pointer_fields_begin = data.pointer_end.clone() + data.pointer.off + data.pointer.data_size;
    let (pointer, pointer_end) = pointer_fields_begin.list_pointer(self.offset)?;
    Ok(UntypedList { pointer: pointer, pointer_end: pointer_end })
  }

  pub fn get<'a, T: TypedListElement<'a>>(
    &self,
    data: &UntypedStruct<'a>,
  ) -> Result<Vec<T>, Error> {
    match self.get_untyped(data) {
      Ok(untyped) => T::from_untyped_list(untyped),
      Err(err) => Err(err),
    }
  }

  pub fn set<T: ListEncoder>(&self, data: &mut UntypedStructOwned, value: T) {
    // TODO: This is pretty inelegant.
    let pointer_end_off = data.pointer_end.off
      + data.pointer.off
      + data.pointer.data_size
      + NumWords(self.offset.0)
      + POINTER_WIDTH_WORDS;
    let pointer = value.append(&mut data.pointer_end.seg);
    let pointer = match pointer {
      Pointer::Null => Pointer::Null,
      Pointer::List(x) => {
        Pointer::List(ListPointer { off: x.off - pointer_end_off, layout: x.layout })
      }
      _ => unreachable!(),
    };

    data.set_pointer(self.offset, pointer);
  }
}
