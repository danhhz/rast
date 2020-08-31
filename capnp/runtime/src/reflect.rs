// Copyright 2020 Daniel Harrison. All Rights Reserved.

use crate::common::NumElements;
use crate::error::Error;
use crate::pointer::{ListLayout, Pointer, StructPointer};
use crate::untyped::{UntypedList, UntypedStruct};

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
}
