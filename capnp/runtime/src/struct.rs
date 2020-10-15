// Copyright 2020 Daniel Harrison. All Rights Reserved.

//! Cap'n Proto [struct]
//!
//! A struct has a set of named, typed fields, numbered consecutively starting
//! from zero. Fields can have default values.
//!
//! [struct]: crate#struct

use crate::common::{
  CapnpAsRef, CapnpToOwned, Discriminant, NumElements, NumWords, POINTER_WIDTH_WORDS,
};
use crate::decode::{SegmentPointerDecode, StructDecode};
use crate::element::StructElement;
use crate::encode::StructEncode;
use crate::error::Error;
use crate::field_meta::FieldMeta;
use crate::pointer::StructPointer;
use crate::segment::{SegmentBorrowed, SegmentOwned};
use crate::segment_pointer::{
  SegmentPointer, SegmentPointerBorrowMut, SegmentPointerOwned, SegmentPointerShared,
};

/// Schema for a Cap'n Proto [struct](crate#struct)
pub struct StructMeta {
  /// The name of this struct
  pub name: &'static str,
  /// Space necessary to encode all primitive fields this version of the struct
  /// schema knows of
  pub data_size: NumWords,
  /// Space necessary to encode all pointer fields this version of the struct
  /// schema knows of
  pub pointer_size: NumWords,
  /// This struct's fields
  ///
  /// NB: function pointer to avoid a cycle in self-referencing structs.
  pub fields: fn() -> &'static [FieldMeta],
}

impl StructMeta {
  /// This struct's fields.
  ///
  /// This is a helper to avoid the awkward usage of calling the function
  /// pointer stored on the struct.
  #[inline(always)]
  pub fn fields(&self) -> &'static [FieldMeta] {
    (self.fields)()
  }
}

/// A borrowed codegen'd Cap'n Proto struct
pub trait TypedStruct<'a>: Sized {
  /// The schema of this struct
  fn meta() -> &'static StructMeta;
  /// Returns an instance of this struct using the given data.
  ///
  /// The caller is responsible for matching the given data to this type.
  /// Presumably it is an encoded instance of a past or future schema of this
  /// same struct.
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self;
  /// Returns the underlying encoded data.
  fn as_untyped(&self) -> UntypedStruct<'a>;
  /// Returns this struct as dynamic reflection [element](crate#element).
  fn as_element(&self) -> StructElement<'a> {
    StructElement(Self::meta(), self.as_untyped())
  }
}

/// A reference-counted codegen'd Cap'n Proto struct
pub trait TypedStructShared: Sized {
  /// The schema of this struct
  fn meta() -> &'static StructMeta;
  /// Returns an instance of this struct using the given data.
  ///
  /// The caller is responsible for matching the given data to this type.
  /// Presumably it is an encoded instance of a past or future schema of this
  /// same struct.
  fn from_untyped_struct(data: UntypedStructShared) -> Self;
  /// Returns the underlying encoded data.
  fn as_untyped(&self) -> UntypedStructShared;
}

/// A borrowed Cap'n Proto struct without schema
#[derive(Clone)]
pub struct UntypedStruct<'a> {
  pointer: StructPointer,
  pointer_end: SegmentPointer<'a>,
}

impl<'a> UntypedStruct<'a> {
  /// Returns a new [`UntypedStruct`].
  pub fn new(pointer: StructPointer, pointer_end: SegmentPointer<'a>) -> Self {
    UntypedStruct { pointer: pointer, pointer_end: pointer_end }
  }

  pub(crate) fn from_root(seg: SegmentBorrowed<'a>) -> Result<Self, Error> {
    let (pointer, pointer_end) = SegmentPointer::from_root(seg).struct_pointer(NumElements(0))?;
    Ok(UntypedStruct { pointer: pointer, pointer_end: pointer_end })
  }

  pub(crate) fn as_root(&self) -> (StructPointer, SegmentBorrowed<'a>) {
    let root_pointer = StructPointer {
      off: self.pointer_end.off + self.pointer.off,
      data_size: self.pointer.data_size,
      pointer_size: self.pointer.pointer_size,
    };
    (root_pointer, self.pointer_end.seg.clone())
  }
}

impl<'a> CapnpToOwned<'a> for UntypedStruct<'a> {
  type Owned = UntypedStructShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    UntypedStructShared {
      pointer: self.pointer.clone(),
      pointer_end: self.pointer_end.capnp_to_owned(),
    }
  }
}

impl<'a> StructDecode<'a> for UntypedStruct<'a> {
  fn pointer(&self) -> &StructPointer {
    &self.pointer
  }
  fn pointer_end(&self) -> &SegmentPointer<'a> {
    &self.pointer_end
  }
}

/// A reference-counted Cap'n Proto struct without schema
#[derive(Clone)]
pub struct UntypedStructShared {
  // TODO: Remove these `pub(crate)`s.
  pub(crate) pointer: StructPointer,
  pub(crate) pointer_end: SegmentPointerShared,
}

impl<'a> CapnpAsRef<'a, UntypedStruct<'a>> for UntypedStructShared {
  fn capnp_as_ref(&'a self) -> UntypedStruct<'a> {
    UntypedStruct { pointer: self.pointer.clone(), pointer_end: self.pointer_end.capnp_as_ref() }
  }
}

/// An owned Cap'n Proto struct without schema
pub struct UntypedStructOwned {
  pointer: StructPointer,
  pointer_end: SegmentPointerOwned,
}

impl UntypedStructOwned {
  /// Returns a new [`UntypedStructOwned`] with a root pointer to a struct with space for the given
  /// data and pointer sizes.
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

  /// Returns this struct data as a reference-counted version of the same.
  pub fn into_shared(self) -> UntypedStructShared {
    UntypedStructShared { pointer: self.pointer, pointer_end: self.pointer_end.into_shared() }
  }

  /// Sets the given discriminant value in this struct.
  pub fn set_discriminant(&mut self, offset: NumElements, value: Discriminant) {
    StructEncode::set_discriminant(self, offset, value)
  }
}

impl StructEncode for UntypedStructOwned {
  fn pointer(&self) -> &StructPointer {
    &self.pointer
  }
  fn pointer_end<'a>(&'a mut self) -> SegmentPointerBorrowMut<'a> {
    self.pointer_end.borrow_mut()
  }
}
