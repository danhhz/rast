// Copyright 2020 Daniel Harrison. All Rights Reserved.

//! A sequence of a capnp type

use std::fmt;
use std::iter;

use crate::common::{NumElements, NumWords};
use crate::pointer::{ListCompositeTag, StructPointer};
use crate::r#struct::UntypedStruct;
use crate::segment_pointer::SegmentPointer;

enum SliceInner<'a, T> {
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

impl<'a, T> SliceInner<'a, T> {
  pub fn len(&self) -> usize {
    match self {
      SliceInner::Empty => 0,
      SliceInner::Packed { start, end, .. } => (end.0 - start.0) as usize,
      SliceInner::Composite { start, end, .. } => (end.0 - start.0) as usize,
    }
  }

  fn next(&mut self) -> Option<T> {
    match self {
      SliceInner::Empty => None,
      SliceInner::Packed { start, end, list_data_begin, decode } => {
        if start < end {
          let ret = decode(list_data_begin, *start);
          *start = NumElements(start.0 + 1);
          Some(ret)
        } else {
          None
        }
      }
      SliceInner::Composite {
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

  fn next_back(&mut self) -> Option<T> {
    match self {
      SliceInner::Empty => None,
      SliceInner::Packed { start, end, list_data_begin, decode } => {
        if start < end {
          let ret = decode(list_data_begin, *end);
          *end = NumElements(end.0 - 1);
          Some(ret)
        } else {
          None
        }
      }
      SliceInner::Composite {
        start,
        end,
        data_size,
        pointer_size,
        list_data_begin,
        from_untyped_struct,
      } => {
        if start < end {
          let offset_w = (*data_size + *pointer_size) * *end;
          let pointer =
            StructPointer { off: offset_w, data_size: *data_size, pointer_size: *pointer_size };
          let ret = from_untyped_struct(UntypedStruct::new(pointer, list_data_begin.clone()));
          *end = NumElements(end.0 - 1);
          Some(ret)
        } else {
          None
        }
      }
    }
  }
}

impl<'a, T> Clone for SliceInner<'a, T> {
  fn clone(&self) -> Self {
    match self {
      SliceInner::Empty => SliceInner::Empty,
      SliceInner::Packed { start, end, list_data_begin, decode } => SliceInner::Packed {
        start: *start,
        end: *end,
        list_data_begin: list_data_begin.clone(),
        decode: *decode,
      },
      SliceInner::Composite {
        start,
        end,
        data_size,
        pointer_size,
        list_data_begin,
        from_untyped_struct,
      } => SliceInner::Composite {
        start: *start,
        end: *end,
        data_size: *data_size,
        pointer_size: *pointer_size,
        list_data_begin: list_data_begin.clone(),
        from_untyped_struct: *from_untyped_struct,
      },
    }
  }
}

/// A sized sequence of a capnp type
#[derive(Clone)]
pub struct Slice<'a, T>(SliceInner<'a, T>);

impl<'a, T> Slice<'a, T> {
  /// Returns a new empty slice.
  pub fn empty() -> Self {
    Slice(SliceInner::Empty)
  }

  /// Returns a sized sequence of packed capnp values.
  pub(crate) fn packed(
    num_elements: NumElements,
    list_data_begin: SegmentPointer<'a>,
    decode: fn(&SegmentPointer<'a>, NumElements) -> T,
  ) -> Self {
    Slice(SliceInner::Packed { start: NumElements(0), end: num_elements, list_data_begin, decode })
  }

  /// Returns a sized sequence of composite capnp values.
  pub(crate) fn composite(
    tag: ListCompositeTag,
    list_data_begin: SegmentPointer<'a>,
    from_untyped_struct: fn(UntypedStruct<'a>) -> T,
  ) -> Self {
    Slice(SliceInner::Composite {
      start: NumElements(0),
      end: tag.num_elements,
      data_size: tag.data_size,
      pointer_size: tag.pointer_size,
      list_data_begin,
      from_untyped_struct,
    })
  }

  /// Returns the number of elements in the slice.
  pub fn len(&self) -> usize {
    self.0.len()
  }

  /// Returns an iterator over the slice.
  pub fn iter(&self) -> Iter<'a, T> {
    Iter(self.0.clone())
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
pub struct Iter<'a, T>(SliceInner<'a, T>);

impl<T> iter::Iterator for Iter<'_, T> {
  type Item = T;

  fn next(&mut self) -> Option<Self::Item> {
    self.0.next()
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    let len = self.0.len();
    (len, Some(len))
  }
}

impl<T> iter::DoubleEndedIterator for Iter<'_, T> {
  fn next_back(&mut self) -> Option<Self::Item> {
    self.0.next_back()
  }
}

impl<T> iter::ExactSizeIterator for Iter<'_, T> {}
impl<T> iter::FusedIterator for Iter<'_, T> {}
