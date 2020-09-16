// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::cmp;

use rand::Rng;

use crate::element::{
  DataElementShared, ElementShared, ListDecodedElementShared, PointerElementShared,
  PrimitiveElement, StructElementShared, UnionElementShared,
};
use crate::element_type::{
  DataElementType, ElementType, ListElementType, PointerElementType, PrimitiveElementType,
  StructElementType, UnionElementType,
};
use crate::field_meta::{FieldMeta, PointerFieldMeta, PrimitiveFieldMeta};
use crate::r#struct::{StructMeta, TypedStructShared, UntypedStructOwned, UntypedStructShared};

pub struct Rand<'a, R: Rng> {
  rng: &'a mut R,

  /// An upper bound on recursive calls into struct generation.
  max_struct_recursion: usize,
}

impl<'a, R: Rng> Rand<'a, R> {
  pub fn new(rng: &'a mut R, max_struct_recursion: usize) -> Self {
    Rand { rng: rng, max_struct_recursion: max_struct_recursion }
  }

  pub fn gen_typed_struct<T: TypedStructShared>(&mut self) -> T {
    T::from_untyped_struct(self.gen_untyped_struct(T::meta()))
  }

  fn gen_untyped_struct(&mut self, meta: &'static StructMeta) -> UntypedStructShared {
    let mut data = UntypedStructOwned::new_with_root_struct(meta.data_size, meta.pointer_size);
    for field_meta in meta.fields() {
      match field_meta {
        FieldMeta::Primitive(x) => match x {
          PrimitiveFieldMeta::U64(x) => x.set(&mut data, self.rng.gen()),
        },
        FieldMeta::Pointer(x) => match x {
          PointerFieldMeta::Data(x) => {
            x.set(&mut data, &self.gen_data_element(&x.element_type()).0);
          }
          PointerFieldMeta::Struct(x) => {
            if self.rng.gen_bool(0.5) || self.max_struct_recursion == 0 {
              continue;
            }
            self.max_struct_recursion -= 1;
            let untyped = self.gen_untyped_struct(x.meta);
            x.set_struct_element(&mut data, &StructElementShared(x.meta, untyped))
          }
          PointerFieldMeta::List(x) => {
            let l = self.gen_list_element(&ListElementType { meta: x.meta });
            x.set_element(&mut data, &ElementShared::Pointer(PointerElementShared::ListDecoded(l)))
              .expect("WIP");
          }
        },
        FieldMeta::Union(x) => {
          x.set_element(
            &mut data,
            &ElementShared::Union(self.gen_union_element(&x.element_type())),
          )
          .expect("WIP");
        }
      }
    }
    data.into_shared()
  }

  fn gen_element(&mut self, element_type: &ElementType) -> ElementShared {
    match element_type {
      ElementType::Primitive(x) => ElementShared::Primitive(self.gen_primitive_element(x)),
      ElementType::Pointer(x) => ElementShared::Pointer(self.gen_pointer_element(x)),
      ElementType::Union(x) => ElementShared::Union(self.gen_union_element(x)),
    }
  }

  fn gen_primitive_element(&mut self, element_type: &PrimitiveElementType) -> PrimitiveElement {
    match element_type {
      PrimitiveElementType::U8 => PrimitiveElement::U8(self.rng.gen()),
      PrimitiveElementType::U64 => PrimitiveElement::U64(self.rng.gen()),
    }
  }

  fn gen_pointer_element(&mut self, element_type: &PointerElementType) -> PointerElementShared {
    match element_type {
      PointerElementType::Data(x) => PointerElementShared::Data(self.gen_data_element(x)),
      PointerElementType::Struct(x) => PointerElementShared::Struct(self.gen_struct_element(x)),
      PointerElementType::List(x) => PointerElementShared::ListDecoded(self.gen_list_element(x)),
    }
  }

  fn gen_data_element(&mut self, _: &DataElementType) -> DataElementShared {
    DataElementShared((0..self.rng.gen_range(0, 5)).map(|_| self.rng.gen()).collect())
  }

  fn gen_struct_element(&mut self, element_type: &StructElementType) -> StructElementShared {
    StructElementShared(element_type.meta, self.gen_untyped_struct(element_type.meta))
  }

  fn gen_list_element(&mut self, element_type: &ListElementType) -> ListDecodedElementShared {
    ListDecodedElementShared(
      element_type.meta,
      self.gen_element_list(&element_type.meta.value_type),
    )
  }

  fn gen_element_list(&mut self, value_type: &ElementType) -> Vec<ElementShared> {
    // TODO: Use a Poisson (or user-selectable) distribution for this.
    let mut len = self.rng.gen_range(0, 3);
    if let ElementType::Pointer(PointerElementType::Struct(_)) = value_type {
      len = cmp::min(len, self.max_struct_recursion);
      self.max_struct_recursion -= len;
    }
    (0..len).map(|_| self.gen_element(value_type)).collect()
  }

  fn gen_union_element(&mut self, element_type: &UnionElementType) -> UnionElementShared {
    let variant_meta =
      &element_type.meta.variants[self.rng.gen_range(0, element_type.meta.variants.len())];
    UnionElementShared(
      element_type.meta,
      variant_meta.discriminant,
      Box::new(self.gen_element(&variant_meta.field_meta.element_type())),
    )
  }
}

#[cfg(test)]
mod test {
  use rand;
  use std::convert::TryInto;
  use std::error::Error;

  use crate::samples::rast_capnp::{Message, MessageShared};
  use crate::samples::test_capnp::{TestAllTypes, TestAllTypesShared};
  use capnp_runtime::prelude::*;

  #[test]
  fn rand_roundtrip_testalltypes() -> Result<(), Box<dyn Error>> {
    let before: TestAllTypesShared =
      capnp_runtime::rand::Rand::new(&mut rand::thread_rng(), 20).gen_typed_struct();
    let mut buf = Vec::new();
    before.capnp_as_ref().as_untyped().encode_as_root_alternate(&mut buf)?;
    let seg = decode_stream_alternate(&buf)?;
    let after = TestAllTypes::from_untyped_struct(SegmentPointer::from_root(seg).try_into()?);
    assert_eq!(before.capnp_as_ref(), after);
    Ok(())
  }

  #[test]
  fn rand_roundtrip_rast() -> Result<(), Box<dyn Error>> {
    let before: MessageShared =
      capnp_runtime::rand::Rand::new(&mut rand::thread_rng(), 20).gen_typed_struct();
    let mut buf = Vec::new();
    before.capnp_as_ref().as_untyped().encode_as_root_alternate(&mut buf)?;
    let seg = decode_stream_alternate(&buf)?;
    let after = Message::from_untyped_struct(SegmentPointer::from_root(seg).try_into()?);
    assert_eq!(before.capnp_as_ref(), after);
    Ok(())
  }
}
