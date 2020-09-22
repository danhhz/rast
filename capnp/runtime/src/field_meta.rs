// Copyright 2020 Daniel Harrison. All Rights Reserved.

use crate::common::{CapnpAsRef, NumElements};
use crate::decode::StructDecode;
use crate::element::{
  DataElement, DataElementShared, Element, ElementShared, ListElement, StructElement,
  StructElementShared, UnionElement, UnionElementShared,
};
use crate::element_type::ElementType;
use crate::encode::StructEncode;
use crate::error::{Error, UnknownDiscriminant};
use crate::list::{ListMeta, TypedList, TypedListElementShared, UntypedList};
use crate::pointer::Pointer;
use crate::r#struct::{
  StructMeta, TypedStruct, TypedStructShared, UntypedStruct, UntypedStructOwned,
  UntypedStructShared,
};
use crate::union::{TypedUnion, TypedUnionShared, UnionMeta, UntypedUnion};

#[derive(Debug)]
pub enum FieldMeta {
  U64(U64FieldMeta),
  Struct(StructFieldMeta),
  List(ListFieldMeta),
  Data(DataFieldMeta),
  Union(UnionFieldMeta),
}

impl FieldMeta {
  pub fn name(&self) -> &'static str {
    match self {
      FieldMeta::U64(x) => x.name,
      FieldMeta::Data(x) => x.name,
      FieldMeta::Struct(x) => x.name,
      FieldMeta::List(x) => x.name,
      FieldMeta::Union(x) => x.name(),
    }
  }

  // WIP this shouldn't be exposed. instead move set_element from SegmentOwned
  // to UntypedStructOwned
  pub fn offset(&self) -> NumElements {
    match self {
      FieldMeta::U64(x) => x.offset,
      FieldMeta::Data(x) => x.offset(),
      FieldMeta::Struct(x) => x.offset(),
      FieldMeta::List(x) => x.offset(),
      FieldMeta::Union(x) => x.offset(),
    }
  }

  pub fn element_type(&self) -> ElementType {
    match self {
      FieldMeta::U64(_) => ElementType::U64,
      FieldMeta::Data(_) => ElementType::Data,
      FieldMeta::Struct(x) => ElementType::Struct(x.meta),
      FieldMeta::List(x) => ElementType::List(x.meta),
      FieldMeta::Union(x) => ElementType::Union(x.meta),
    }
  }

  // TODO: Make this an enum with Null/NotNull/NotNullable/Missing.
  pub fn is_null(&self, data: &UntypedStruct<'_>) -> bool {
    match self {
      FieldMeta::Data(x) => x.is_null(data),
      FieldMeta::Struct(x) => x.is_null(data),
      FieldMeta::List(x) => x.is_null(data),
      // Primitive and union fields cannot be null.
      _ => false,
    }
  }

  pub(crate) fn get_element<'a>(&self, data: &UntypedStruct<'a>) -> Result<Element<'a>, Error> {
    match self {
      FieldMeta::U64(x) => Ok(x.get_element(data)),
      FieldMeta::Data(x) => x.get_element(data).map(|x| Element::Data(x)),
      FieldMeta::Struct(x) => x.get_element(data).map(|x| Element::Struct(x)),
      FieldMeta::List(x) => x.get_element(data).map(|x| Element::List(x)),
      FieldMeta::Union(x) => x.get_element(data).map(|x| Element::Union(x)),
    }
  }

  pub(crate) fn set_element(
    &self,
    data: &mut UntypedStructOwned,
    value: &ElementShared,
  ) -> Result<(), Error> {
    match self {
      FieldMeta::U64(x) => x.set_element(data, value),
      FieldMeta::Data(x) => x.set_element(data, value),
      FieldMeta::Struct(x) => x.set_element(data, value),
      FieldMeta::List(x) => x.set_element(data, value),
      FieldMeta::Union(x) => x.set_element(data, value),
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

  pub fn set(&self, data: &mut UntypedStructOwned, value: u64) {
    data.set_u64(self.offset, value);
  }

  pub fn get_element<'a>(&self, data: &UntypedStruct<'a>) -> Element<'a> {
    Element::U64(self.get(data))
  }

  pub fn set_element(
    &self,
    data: &mut UntypedStructOwned,
    value: &ElementShared,
  ) -> Result<(), Error> {
    match value {
      ElementShared::U64(value) => {
        self.set(data, *value);
        Ok(())
      }
      value => Err(Error::Usage(format!(
        "U64FieldMeta::set_element unsupported_type: {:?}",
        value.capnp_as_ref().element_type()
      ))),
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

  pub fn set<T: TypedStructShared>(&self, data: &mut UntypedStructOwned, value: Option<T>) {
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
      ElementShared::Struct(value) => {
        self.set_struct_element(data, value);
        Ok(())
      }
      value => Err(Error::Usage(format!(
        "StructFieldMeta::set_element unsupported_type: {:?}",
        value.capnp_as_ref().element_type()
      ))),
    }
  }

  fn set_untyped(
    &self,
    data: &mut UntypedStructOwned,
    _value_meta: &'static StructMeta,
    value: Option<&UntypedStructShared>,
  ) {
    // TODO: Check that the metas match?
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

  pub fn set<T: TypedListElementShared>(&self, data: &mut UntypedStructOwned, value: &[T]) {
    data.set_list(self.offset, value)
  }

  pub fn set_element(
    &self,
    data: &mut UntypedStructOwned,
    value: &ElementShared,
  ) -> Result<(), Error> {
    match value {
      ElementShared::ListDecoded(x) => {
        // TODO: Check that the metas match?
        data.set_list_decoded_element(self.offset, x)
      }
      value => Err(Error::Usage(format!(
        "ListFieldMeta::set_element unsupported_type: {:?}",
        value.capnp_as_ref().element_type()
      ))),
    }
  }
}

#[derive(Debug)]
pub struct DataFieldMeta {
  pub name: &'static str,
  pub offset: NumElements,
}

impl DataFieldMeta {
  pub fn offset(&self) -> NumElements {
    self.offset
  }
  pub fn is_null(&self, data: &UntypedStruct<'_>) -> bool {
    match data.pointer_raw(self.offset) {
      Pointer::Null => true,
      _ => false,
    }
  }

  pub fn get_element<'a>(&self, data: &UntypedStruct<'a>) -> Result<DataElement<'a>, Error> {
    self.get(data).map(|value| DataElement(value))
  }

  pub fn get<'a>(&self, data: &UntypedStruct<'a>) -> Result<&'a [u8], Error> {
    data.bytes(self.offset)
  }

  pub fn set(&self, data: &mut UntypedStructOwned, value: &[u8]) {
    data.set_bytes(self.offset, value)
  }

  pub fn set_element(
    &self,
    data: &mut UntypedStructOwned,
    value: &ElementShared,
  ) -> Result<(), Error> {
    match value {
      ElementShared::Data(DataElementShared(value)) => {
        self.set(data, value);
        Ok(())
      }
      value => Err(Error::Usage(format!(
        "DataFieldMeta::set_element unsupported_type: {:?}",
        value.capnp_as_ref().element_type()
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
        value.capnp_as_ref().element_type()
      ))),
    }
  }
}
