// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::fmt;

use crate::common::{Discriminant, ElementWidth, NumElements, NumWords};
use crate::error::Error;
use crate::pointer::{ListLayout, Pointer, StructPointer};
use crate::untyped::{
  UntypedList, UntypedStruct, UntypedStructOwned, UntypedStructShared, UntypedUnion,
};

mod element;
pub use element::*;

mod element_type;
pub use element_type::*;

pub mod cmp;
pub use cmp::*;

pub mod fmt_debug;
pub use fmt_debug::*;

pub mod list;
pub use list::*;

pub struct StructMeta {
  pub name: &'static str,
  pub data_size: NumWords,
  pub pointer_size: NumWords,
  pub fields: fn() -> &'static [FieldMeta],
}

impl StructMeta {
  pub fn fields(&self) -> &'static [FieldMeta] {
    (self.fields)()
  }
}

impl fmt::Debug for StructMeta {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("StructMeta")
      .field("name", &self.name)
      .field("data_size", &self.data_size)
      .field("pointer_size", &self.pointer_size)
      .field("fields", &"WIP")
      .finish()
  }
}

pub trait TypedStruct<'a> {
  fn meta() -> &'static StructMeta;
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self;
  fn as_untyped(&self) -> UntypedStruct<'a>;
  // TODO: Move this
  fn as_element(&self) -> StructElement<'a> {
    StructElement(Self::meta(), self.as_untyped())
  }
}

#[derive(Debug, PartialOrd, PartialEq)]
pub struct ListMeta {
  pub value_type: ElementType,
}

pub trait TypedList<'a>: Sized {
  fn from_untyped_list(untyped: &UntypedList<'a>) -> Result<Self, Error>;
}

// TODO: Relate TypedListShared to TypedList.
pub trait TypedListShared {
  fn set(&self, data: &mut UntypedStructOwned, offset: NumElements);
}

impl<'a> TypedList<'a> for Vec<u8> {
  fn from_untyped_list(untyped: &UntypedList<'a>) -> Result<Self, Error> {
    let num_elements = match &untyped.pointer.layout {
      ListLayout::Packed(NumElements(0), ElementWidth::Void) => {
        // NB: NumElements(0), ElementWidth::Void is a null pointer.
        return Ok(vec![]);
      }
      ListLayout::Packed(num_elements, ElementWidth::OneByte) => num_elements,
      x => return Err(Error::from(format!("unsupported list layout for u8: {:?}", x))),
    };
    let list_elements_begin = untyped.pointer_end.clone() + untyped.pointer.off;
    let mut ret = Vec::new();
    for idx in 0..num_elements.0 {
      let el = list_elements_begin.u8(NumElements(idx));
      ret.push(el);
    }
    Ok(ret)
  }
}

impl<'a> TypedList<'a> for Vec<u64> {
  fn from_untyped_list(untyped: &UntypedList<'a>) -> Result<Self, Error> {
    todo!()
  }
}

// impl<'a> TypedList<'a> for Vec<StructElement<'a>> {
//   fn from_untyped_list(untyped: &UntypedList<'a>) -> Result<Self, Error> {
//     todo!()
//   }
// }

impl<'a> TypedList<'a> for Vec<UntypedStruct<'a>> {
  fn from_untyped_list(untyped: &UntypedList<'a>) -> Result<Self, Error> {
    let pointer_declared_len = match &untyped.pointer.layout {
      ListLayout::Composite(num_words) => *num_words,
      x => return Err(Error::from(format!("unsupported list layout for TypedStruct: {:?}", x))),
    };
    let list_elements_begin = untyped.pointer_end.clone() + untyped.pointer.off;
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
      ret.push(untyped);
    }
    Ok(ret)
  }
}

impl<'a, T: TypedStruct<'a>> TypedList<'a> for Vec<T> {
  fn from_untyped_list(untyped: &UntypedList<'a>) -> Result<Self, Error> {
    Vec::<UntypedStruct<'a>>::from_untyped_list(untyped)
      .map(|xs| xs.into_iter().map(|x| T::from_untyped_struct(x)).collect())
  }
}

pub trait TypedStructShared {
  fn meta() -> &'static StructMeta;
  fn from_untyped_struct(data: UntypedStructShared) -> Self;
  fn as_untyped(&self) -> UntypedStructShared;
}

pub trait TypedUnion<'a>: Sized {
  fn meta() -> &'static UnionMeta;
  fn from_untyped_union(data: &UntypedUnion<'a>) -> Result<Self, Error>;
}

pub trait TypedUnionShared<'a, T: TypedUnion<'a>> {
  fn as_ref(&'a self) -> T;
  fn set(&self, data: &mut UntypedStructOwned, discriminant_offset: NumElements);
}

#[derive(Debug)]
pub struct UnionVariantMeta {
  pub discriminant: Discriminant,
  pub field_meta: FieldMeta,
}

#[derive(Debug)]
pub struct UnionMeta {
  pub name: &'static str,
  pub variants: &'static [UnionVariantMeta],
}

impl UnionMeta {
  pub fn get(&self, value: Discriminant) -> Option<&UnionVariantMeta> {
    // WIP this should be correct but feels sketchy
    self.variants.get(value.0 as usize)
  }
}

#[derive(Debug)]
pub enum FieldMeta {
  Primitive(PrimitiveFieldMeta),
  Pointer(PointerFieldMeta),
  Union(UnionFieldMeta),
}

impl FieldMeta {
  pub fn name(&self) -> &'static str {
    match self {
      FieldMeta::Primitive(x) => x.name(),
      FieldMeta::Pointer(x) => x.name(),
      FieldMeta::Union(x) => x.name(),
    }
  }

  // WIP this shouldn't be exposed. instead move set_element from SegmentOwned
  // to UntypedStructOwned
  pub fn offset(&self) -> NumElements {
    match self {
      FieldMeta::Primitive(x) => x.offset(),
      FieldMeta::Pointer(x) => x.offset(),
      FieldMeta::Union(x) => x.offset(),
    }
  }

  pub fn element_type(&self) -> ElementType {
    match self {
      FieldMeta::Primitive(x) => ElementType::Primitive(x.element_type()),
      FieldMeta::Pointer(x) => ElementType::Pointer(x.element_type()),
      FieldMeta::Union(x) => ElementType::Union(x.element_type()),
    }
  }

  pub fn set_element(
    &self,
    data: &mut UntypedStructOwned,
    value: &ElementShared,
  ) -> Result<(), Error> {
    match self {
      FieldMeta::Primitive(x) => x.set_element(data, value),
      FieldMeta::Pointer(x) => x.set_element(data, value),
      FieldMeta::Union(x) => x.set_element(data, value),
    }
  }

  pub fn get_element<'a>(&self, data: &UntypedStruct<'a>) -> Result<Element<'a>, Error> {
    match self {
      FieldMeta::Primitive(x) => Ok(Element::Primitive(x.get_element(data))),
      FieldMeta::Pointer(x) => x.get_element(data).map(|x| Element::Pointer(x)),
      FieldMeta::Union(x) => x.get_element(data).map(|x| Element::Union(x)),
    }
  }
}

#[derive(Debug)]
pub enum PrimitiveFieldMeta {
  U64(U64FieldMeta),
}

impl PrimitiveFieldMeta {
  pub fn name(&self) -> &'static str {
    match self {
      PrimitiveFieldMeta::U64(x) => x.name,
    }
  }
  pub fn offset(&self) -> NumElements {
    match self {
      PrimitiveFieldMeta::U64(x) => x.offset,
    }
  }
  pub fn element_type(&self) -> PrimitiveElementType {
    match self {
      PrimitiveFieldMeta::U64(x) => PrimitiveElementType::U64,
    }
  }
  pub fn get_element(&self, data: &UntypedStruct<'_>) -> PrimitiveElement {
    match self {
      PrimitiveFieldMeta::U64(x) => x.get_element(data),
    }
  }
  pub fn set_element(
    &self,
    data: &mut UntypedStructOwned,
    value: &ElementShared,
  ) -> Result<(), Error> {
    match self {
      PrimitiveFieldMeta::U64(x) => x.set_element(data, value),
    }
  }
}

#[derive(Debug)]
pub struct U64FieldMeta {
  pub name: &'static str,
  pub offset: NumElements,
}

impl U64FieldMeta {
  pub fn get(&self, data: &UntypedStruct<'_>) -> u64 {
    let data_fields_begin = data.pointer_end.clone() + data.pointer.off;
    data_fields_begin.u64(self.offset)
  }

  pub fn get_element(&self, data: &UntypedStruct<'_>) -> PrimitiveElement {
    PrimitiveElement::U64(self.get(data))
  }

  pub fn set(&self, data: &mut UntypedStructOwned, value: u64) {
    data.set_u64(self.offset, value);
  }

  pub fn set_element(
    &self,
    data: &mut UntypedStructOwned,
    value: &ElementShared,
  ) -> Result<(), Error> {
    match value {
      ElementShared::Primitive(PrimitiveElement::U64(value)) => {
        self.set(data, *value);
        Ok(())
      }
      value => {
        Err(Error::from(format!("set u64 unsupported_type: {:?}", value.as_ref().element_type())))
      }
    }
  }
}

#[derive(Debug)]
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
  pub fn offset(&self) -> NumElements {
    match self {
      PointerFieldMeta::Struct(x) => x.offset(),
      PointerFieldMeta::List(x) => x.offset(),
    }
  }
  pub fn element_type(&self) -> PointerElementType {
    match self {
      PointerFieldMeta::Struct(x) => PointerElementType::Struct(x.element_type()),
      PointerFieldMeta::List(x) => PointerElementType::List(x.element_type()),
    }
  }
  pub fn is_null(&self, data: &UntypedStruct<'_>) -> bool {
    match self {
      PointerFieldMeta::Struct(x) => x.is_null(data),
      PointerFieldMeta::List(x) => x.is_null(data),
    }
  }
  pub fn get_element<'a>(&self, data: &UntypedStruct<'a>) -> Result<PointerElement<'a>, Error> {
    match self {
      PointerFieldMeta::Struct(x) => x.get_element(data).map(|x| PointerElement::Struct(x)),
      PointerFieldMeta::List(x) => x.get_element(data).map(|x| PointerElement::List(x)),
    }
  }
  pub fn set_element(
    &self,
    data: &mut UntypedStructOwned,
    value: &ElementShared,
  ) -> Result<(), Error> {
    match self {
      PointerFieldMeta::Struct(x) => x.set_element(data, value),
      PointerFieldMeta::List(x) => x.set_element(data, value),
    }
  }
}

#[derive(Debug)]
pub struct StructFieldMeta {
  pub name: &'static str,
  pub offset: NumElements,
  pub meta: &'static StructMeta,
}

impl StructFieldMeta {
  pub fn element_type(&self) -> StructElementType {
    StructElementType { meta: self.meta }
  }
  pub fn offset(&self) -> NumElements {
    self.offset
  }
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

  pub fn get_element<'a>(&self, data: &UntypedStruct<'a>) -> Result<StructElement<'a>, Error> {
    self.get_untyped(data).map(|untyped| StructElement(self.meta, untyped))
  }

  // TODO: Spec allows returning default value in the case of an out-of-bounds
  // pointer.
  pub fn get<'a, T: TypedStruct<'a>>(&self, data: &UntypedStruct<'a>) -> Result<T, Error> {
    self.get_untyped(data).map(|x| T::from_untyped_struct(x))
  }

  pub fn set<T: TypedStructShared>(&self, data: &mut UntypedStructOwned, value: Option<&T>) {
    if let Some(value) = value {
      self.set_untyped(data, T::meta(), Some(&value.as_untyped()));
    }
  }

  pub fn set_element(
    &self,
    data: &mut UntypedStructOwned,
    value: &ElementShared,
  ) -> Result<(), Error> {
    match value {
      ElementShared::Pointer(PointerElementShared::Struct(StructElementShared(meta, untyped))) => {
        self.set_untyped(data, meta, Some(untyped));
        Ok(())
      }
      value => Err(Error::from(format!(
        "set struct unsupported_type: {:?}",
        value.as_ref().element_type()
      ))),
    }
  }

  pub fn set_untyped(
    &self,
    data: &mut UntypedStructOwned,
    value_meta: &'static StructMeta,
    value: Option<&UntypedStructShared>,
  ) {
    // TODO: Check that value_meta matches the expected one?
    match value {
      None => data.set_pointer(self.offset, Pointer::Null),
      Some(value) => {
        data.set_struct_element(self.offset, &StructElementShared(value_meta, value.clone()));
      }
    }
  }
}

#[derive(Debug)]
pub struct ListFieldMeta {
  pub name: &'static str,
  pub offset: NumElements,
  pub meta: &'static ListMeta,
}

impl ListFieldMeta {
  pub fn element_type(&self) -> ListElementType {
    ListElementType { meta: self.meta }
  }
  pub fn offset(&self) -> NumElements {
    self.offset
  }
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

  pub fn get_element<'a>(&self, data: &UntypedStruct<'a>) -> Result<ListElement<'a>, Error> {
    self.get_untyped(data).map(|untyped| ListElement(self.meta, untyped))
  }

  pub fn get<'a, T: TypedList<'a>>(&self, data: &UntypedStruct<'a>) -> Result<T, Error> {
    self.get_untyped(data).and_then(|untyped| T::from_untyped_list(&untyped))
  }

  pub fn set<T: TypedListShared>(&self, data: &mut UntypedStructOwned, value: T) {
    value.set(data, self.offset)
  }

  pub fn set_element(
    &self,
    data: &mut UntypedStructOwned,
    value: &ElementShared,
  ) -> Result<(), Error> {
    match value {
      ElementShared::Pointer(PointerElementShared::ListDecoded(x)) => {
        // TODO: Check that the metas match?
        data.set_list_decoded_element(self.offset, x);
        Ok(())
      }
      value => {
        Err(Error::from(format!("set list unsupported_type: {:?}", value.as_ref().element_type())))
      }
    }
  }
}

#[derive(Debug)]
pub struct UnionFieldMeta {
  pub name: &'static str,
  pub offset: NumElements,
  pub meta: &'static UnionMeta,
}

impl UnionFieldMeta {
  pub fn element_type(&self) -> UnionElementType {
    UnionElementType { meta: self.meta }
  }
  pub fn offset(&self) -> NumElements {
    self.offset
  }
  pub fn name(&self) -> &'static str {
    self.name
  }

  pub fn get_untyped<'a>(&self, data: &UntypedStruct<'a>) -> UntypedUnion<'a> {
    let data_fields_begin = data.pointer_end.clone() + data.pointer.off;
    let discriminant = Discriminant(data_fields_begin.u16(self.offset));
    UntypedUnion { discriminant: discriminant, variant_data: data.clone() }
  }
  pub fn get<'a, T: TypedUnion<'a>>(&self, data: &UntypedStruct<'a>) -> Result<T, Error> {
    T::from_untyped_union(&self.get_untyped(data))
  }
  pub fn get_element<'a>(&self, data: &UntypedStruct<'a>) -> Result<UnionElement<'a>, Error> {
    let untyped = self.get_untyped(data);
    let variant_meta = self.meta.get(untyped.discriminant).expect("WIP");
    let value = variant_meta.field_meta.get_element(data)?;
    Ok(UnionElement(self.meta, variant_meta.discriminant, Box::new(value)))
  }

  pub fn set<'a, U: TypedUnion<'a>, S: TypedUnionShared<'a, U>>(
    &self,
    data: &mut UntypedStructOwned,
    value: S,
  ) {
    value.set(data, self.offset);
  }

  pub fn set_element(
    &self,
    data: &mut UntypedStructOwned,
    value: &ElementShared,
  ) -> Result<(), Error> {
    match value {
      ElementShared::Union(x) => {
        // TODO: Check that the metas match?
        let UnionElementShared(_, discriminant, value) = x;
        let variant_meta = self.meta.get(*discriminant).expect("WIP");
        data.set_discriminant(self.offset, variant_meta.discriminant);
        variant_meta.field_meta.set_element(data, value.as_ref())
      }
      value => {
        Err(Error::from(format!("set union unsupported_type: {:?}", value.as_ref().element_type())))
      }
    }
  }
}
