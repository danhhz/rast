// Copyright 2020 Daniel Harrison. All Rights Reserved.

use crate::common::NumElements;
use crate::decode::StructDecode;
use crate::element::{
  Element, ElementShared, ListElement, PointerElement, PointerElementShared, PrimitiveElement,
  StructElement, StructElementShared, UnionElement, UnionElementShared,
};
use crate::element_type::{
  ElementType, ListElementType, PointerElementType, PrimitiveElementType, StructElementType,
  UnionElementType,
};
use crate::encode::StructEncode;
use crate::error::{Error, UnknownDiscriminant};
use crate::list::{ListMeta, TypedList, TypedListShared, UntypedList};
use crate::pointer::Pointer;
use crate::r#struct::{
  StructMeta, TypedStruct, TypedStructShared, UntypedStruct, UntypedStructOwned,
  UntypedStructShared,
};
use crate::union::{TypedUnion, TypedUnionShared, UnionMeta, UntypedUnion};

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
      PrimitiveFieldMeta::U64(_) => PrimitiveElementType::U64,
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
    data.u64(self.offset)
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
      value => Err(Error::Usage(format!(
        "U64FieldMeta::set_element unsupported_type: {:?}",
        value.as_ref().element_type()
      ))),
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
    match data.pointer_raw(self.offset) {
      Pointer::Null => true,
      _ => false,
    }
  }

  fn get_untyped<'a>(&self, data: &UntypedStruct<'a>) -> Result<UntypedStruct<'a>, Error> {
    data.untyped_struct(self.offset)
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

  pub fn set_struct_element(&self, data: &mut UntypedStructOwned, value: &StructElementShared) {
    let StructElementShared(meta, untyped) = value;
    self.set_untyped(data, meta, Some(untyped));
  }

  pub fn set_element(
    &self,
    data: &mut UntypedStructOwned,
    value: &ElementShared,
  ) -> Result<(), Error> {
    match value {
      ElementShared::Pointer(PointerElementShared::Struct(value)) => {
        self.set_struct_element(data, value);
        Ok(())
      }
      value => Err(Error::Usage(format!(
        "StructFieldMeta::set_element unsupported_type: {:?}",
        value.as_ref().element_type()
      ))),
    }
  }

  fn set_untyped(
    &self,
    data: &mut UntypedStructOwned,
    _value_meta: &'static StructMeta,
    value: Option<&UntypedStructShared>,
  ) {
    // TODO: Check that _value_meta matches the expected one?
    data.set_struct(self.offset, value)
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
    match data.pointer_raw(self.offset) {
      Pointer::Null => true,
      _ => false,
    }
  }

  fn get_untyped<'a>(&self, data: &UntypedStruct<'a>) -> Result<UntypedList<'a>, Error> {
    data.untyped_list(self.offset)
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
        data.set_list_decoded_element(self.offset, x)
      }
      value => Err(Error::Usage(format!(
        "ListFieldMeta::set_element unsupported_type: {:?}",
        value.as_ref().element_type()
      ))),
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

  fn get_untyped<'a>(&self, data: &UntypedStruct<'a>) -> UntypedUnion<'a> {
    data.untyped_union(self.offset)
  }

  // NB: Double Result is intentional for better error handling. See
  // https://sled.rs/errors.html
  pub fn get<'a, T: TypedUnion<'a>>(
    &self,
    data: &UntypedStruct<'a>,
  ) -> Result<Result<T, UnknownDiscriminant>, Error> {
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
      value => Err(Error::Usage(format!(
        "UnionFieldMeta::set_element unsupported_type: {:?}",
        value.as_ref().element_type()
      ))),
    }
  }
}
