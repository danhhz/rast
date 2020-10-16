// Copyright 2020 Daniel Harrison. All Rights Reserved.

//! A sequence of a capnp type

use std::fmt;
use std::iter;

use crate::common::{NumElements, NumWords};
use crate::pointer::{ListCompositeTag, StructPointer};
use crate::r#struct::UntypedStruct;
use crate::segment_pointer::SegmentPointer;

/// A sized sequence of a capnp type
#[derive(Clone)]
pub enum Slice<'a, T> {
  Empty,
  Packed {
    len: NumElements,
    list_data_begin: SegmentPointer<'a>,
    decode: fn(&SegmentPointer<'a>, NumElements) -> T,
  },
  Composite {
    tag: ListCompositeTag,
    list_data_begin: SegmentPointer<'a>,
    from_untyped_struct: fn(UntypedStruct<'a>) -> T,
  },
}

impl<'a, T> Slice<'a, T> {
  pub fn empty() -> Self {
    Slice::Empty
  }

  pub fn len(&self) -> usize {
    match self {
      Slice::Empty => 0,
      Slice::Packed { len, .. } => len.0 as usize,
      Slice::Composite { tag, .. } => tag.num_elements.0 as usize,
    }
  }

  pub fn iter(&self) -> Iter<'a, T> {
    match self {
      Slice::Empty => Iter::Empty,
      Slice::Packed { len, list_data_begin, decode } => Iter::Packed {
        start: NumElements(0),
        end: *len,
        list_data_begin: list_data_begin.clone(),
        decode: *decode,
      },
      Slice::Composite { tag, list_data_begin, from_untyped_struct } => Iter::Composite {
        start: NumElements(0),
        end: tag.num_elements,
        data_size: tag.data_size,
        pointer_size: tag.pointer_size,
        list_data_begin: list_data_begin.clone(),
        from_untyped_struct: *from_untyped_struct,
      },
    }
  }
}

impl<'a, T: fmt::Debug> fmt::Debug for Slice<'a, T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_list().entries(self.iter()).finish()
  }
}

impl<'a, T> iter::IntoIterator for Slice<'a, T> {
  type Item = T;
  type IntoIter = Iter<'a, T>;
  fn into_iter(self) -> Self::IntoIter {
    self.iter()
  }
}

/// An iterator over a sequence of some capnp type
pub enum Iter<'a, T> {
  Empty,
  Packed {
    start: NumElements,
    end: NumElements,
    list_data_begin: SegmentPointer<'a>,
    decode: fn(&SegmentPointer<'a>, NumElements) -> T,
  },
  Composite {
    start: NumElements,
    end: NumElements,
    data_size: NumWords,
    pointer_size: NumWords,
    list_data_begin: SegmentPointer<'a>,
    from_untyped_struct: fn(UntypedStruct<'a>) -> T,
  },
}

impl<T> iter::Iterator for Iter<'_, T> {
  type Item = T;

  fn next(&mut self) -> Option<Self::Item> {
    match self {
      Iter::Empty => None,
      Iter::Packed { start, end, list_data_begin, decode } => {
        if start < end {
          let ret = decode(list_data_begin, *start);
          *start = NumElements(start.0 + 1);
          Some(ret)
        } else {
          None
        }
      }
      Iter::Composite {
        start,
        end,
        data_size,
        pointer_size,
        list_data_begin,
        from_untyped_struct,
      } => {
        if start < end {
          let offset_w = (*data_size + *pointer_size) * *start;
          let pointer =
            StructPointer { off: offset_w, data_size: *data_size, pointer_size: *pointer_size };
          let ret = from_untyped_struct(UntypedStruct::new(pointer, list_data_begin.clone()));
          *start = NumElements(start.0 + 1);
          Some(ret)
        } else {
          None
        }
      }
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    match self {
      Iter::Empty => (0, Some(0)),
      Iter::Packed { start, end, .. } => {
        ((end.0 - start.0) as usize, Some((end.0 - start.0) as usize))
      }
      Iter::Composite { start, end, .. } => {
        ((end.0 - start.0) as usize, Some((end.0 - start.0) as usize))
      }
    }
  }
}

impl<T> iter::DoubleEndedIterator for Iter<'_, T> {
  fn next_back(&mut self) -> Option<Self::Item> {
    todo!()
  }
}

impl<T> iter::ExactSizeIterator for Iter<'_, T> {}
impl<T> iter::FusedIterator for Iter<'_, T> {}
