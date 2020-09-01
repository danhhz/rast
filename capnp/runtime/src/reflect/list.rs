// Copyright 2020 Daniel Harrison. All Rights Reserved.

use crate::common::{ElementWidth, NumElements, NumWords, U64_WIDTH_BYTES, U8_WIDTH_BYTES};
use crate::error::Error;
use crate::pointer::{ListLayout, ListPointer, Pointer};
use crate::segment::SegmentOwned;
use crate::untyped::UntypedList;

pub trait TypedListElement<'a>: Sized {
  fn from_untyped_list(untyped: UntypedList<'a>) -> Result<Vec<Self>, Error>;
}

pub trait NonCompositeListElement {
  fn width() -> ElementWidth;
  fn append(&self, seg: &mut SegmentOwned, offset: NumWords, idx: NumElements);
}

impl<'a> TypedListElement<'a> for u8 {
  fn from_untyped_list(untyped: UntypedList<'a>) -> Result<Vec<Self>, Error> {
    let num_elements = match untyped.pointer.layout {
      ListLayout::Packed(num_elements, ElementWidth::OneByte) => num_elements,
      x => return Err(Error::from(format!("unsupported list layout for u8: {:?}", x))),
    };
    let list_elements_begin = untyped.pointer_end + untyped.pointer.off;
    let mut ret = Vec::new();
    for idx in 0..num_elements.0 {
      ret.push(list_elements_begin.u8(NumElements(idx)));
    }
    Ok(ret)
  }
}

pub trait ListEncoder {
  fn append(&self, seg: &mut SegmentOwned) -> Pointer;
}

impl ListEncoder for &[u8] {
  fn append(&self, seg: &mut SegmentOwned) -> Pointer {
    let list_begin = seg.len_words_rounded_up();
    let list_len = self.len() * U8_WIDTH_BYTES;
    seg.buf.resize(list_begin.as_bytes() + list_len, 0);
    for (idx, el) in self.iter().enumerate() {
      seg.set_u8(list_begin, NumElements(idx as i32), *el);
    }
    Pointer::List(ListPointer {
      off: list_begin,
      layout: ListLayout::Packed(NumElements(self.len() as i32), ElementWidth::OneByte),
    })
  }
}

impl<'a> TypedListElement<'a> for u64 {
  fn from_untyped_list(untyped: UntypedList<'a>) -> Result<Vec<Self>, Error> {
    let num_elements = match untyped.pointer.layout {
      ListLayout::Packed(num_elements, ElementWidth::FourBytes) => num_elements,
      x => return Err(Error::from(format!("unsupported list layout for u64: {:?}", x))),
    };
    let list_elements_begin = untyped.pointer_end + untyped.pointer.off;
    let mut ret = Vec::new();
    for idx in 0..num_elements.0 {
      ret.push(list_elements_begin.u64(NumElements(idx)));
    }
    Ok(ret)
  }
}

impl ListEncoder for &[u64] {
  fn append(&self, seg: &mut SegmentOwned) -> Pointer {
    let list_begin = seg.len_words_rounded_up();
    let list_len = self.len() * U64_WIDTH_BYTES;
    seg.buf.resize(list_begin.as_bytes() + list_len, 0);
    for (idx, el) in self.iter().enumerate() {
      seg.set_u64(list_begin, NumElements(idx as i32), *el);
    }
    Pointer::List(ListPointer {
      off: list_begin,
      layout: ListLayout::Packed(
        NumElements(self.len() as i32),
        ElementWidth::EightBytesNonPointer,
      ),
    })
  }
}

impl<'a, T: TypedListElement<'a>> TypedListElement<'a> for Vec<T> {
  fn from_untyped_list(untyped: UntypedList<'a>) -> Result<Vec<Self>, Error> {
    Err(Error::from("unimplemented TypedListElement<'a> for Vec<T>"))
  }
}
