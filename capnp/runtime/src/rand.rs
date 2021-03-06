// Copyright 2020 Daniel Harrison. All Rights Reserved.

//! Random generation of Cap'n Proto types

use std::cmp;

use rand::distributions::Alphanumeric;
use rand::Rng;

use crate::common::Discriminant;
use crate::element::{
  DataElementShared, ElementShared, EnumElement, ListDecodedElementShared, StructElementShared,
  TextElementShared, UnionElementShared,
};
use crate::element_type::ElementType;
use crate::field_meta::FieldMeta;
use crate::list::ListMeta;
use crate::r#enum::EnumMeta;
use crate::r#struct::{StructMeta, TypedStructShared, UntypedStructOwned, UntypedStructShared};
use crate::union::UnionMeta;

/// A generator for random Cap'n Proto types.
///
/// Configurables:
/// - max_struct_recursion: An upper bound on total recursive calls into struct
///   generation used to construct a single struct (NB: not recursion depth).
///   Useful for self-referential structs.
pub struct Rand<'a, R: Rng> {
  rng: &'a mut R,

  /// An upper bound on recursive calls into struct generation.
  max_struct_recursion: usize,
}

impl<'a, R: Rng> Rand<'a, R> {
  /// Constructs a new [`Rand`].
  pub fn new(rng: &'a mut R, max_struct_recursion: usize) -> Self {
    Rand { rng: rng, max_struct_recursion: max_struct_recursion }
  }

  /// Returns a new Cap'n Proto struct with fields filled with randomly
  /// generated values.
  pub fn gen_typed_struct<T: TypedStructShared>(&mut self) -> T {
    T::from_untyped_struct(self.gen_untyped_struct(T::meta()))
  }

  fn gen_untyped_struct(&mut self, meta: &'static StructMeta) -> UntypedStructShared {
    let mut data = UntypedStructOwned::new_with_root_struct(meta.data_size, meta.pointer_size);
    for field_meta in meta.fields() {
      match field_meta {
        FieldMeta::Bool(x) => x.set(&mut data, self.rng.gen()),
        FieldMeta::I32(x) => x.set(&mut data, self.rng.gen()),
        FieldMeta::U8(x) => x.set(&mut data, self.rng.gen()),
        FieldMeta::U16(x) => x.set(&mut data, self.rng.gen()),
        FieldMeta::U32(x) => x.set(&mut data, self.rng.gen()),
        FieldMeta::U64(x) => x.set(&mut data, self.rng.gen()),
        FieldMeta::F32(x) => x.set(&mut data, self.rng.gen()),
        FieldMeta::F64(x) => x.set(&mut data, self.rng.gen()),
        FieldMeta::Data(x) => {
          x.set(&mut data, &self.gen_data_element().0);
        }
        FieldMeta::Text(x) => {
          x.set(&mut data, &self.gen_text_element().0);
        }
        FieldMeta::Enum(x) => x
          .set_enum_element(&mut data, &self.gen_enum_element(x.meta))
          .expect("internal logic error"),
        FieldMeta::Struct(x) => {
          if self.rng.gen_bool(0.5) || self.max_struct_recursion == 0 {
            continue;
          }
          self.max_struct_recursion -= 1;
          let untyped = self.gen_untyped_struct(x.meta);
          x.set_struct_element(&mut data, &StructElementShared(x.meta, untyped))
        }
        FieldMeta::List(x) => {
          let l = self.gen_list_element(x.meta);
          x.set_element(&mut data, &ElementShared::ListDecoded(l)).expect("internal logic error");
        }
        FieldMeta::Union(x) => {
          x.set_element(&mut data, &ElementShared::Union(self.gen_union_element(x.meta)))
            .expect("internal logic error");
        }
      }
    }
    data.into_shared()
  }

  fn gen_element(&mut self, element_type: &ElementType) -> ElementShared {
    match element_type {
      ElementType::Bool => ElementShared::Bool(self.rng.gen()),
      ElementType::I32 => ElementShared::I32(self.rng.gen()),
      ElementType::U8 => ElementShared::U8(self.rng.gen()),
      ElementType::U16 => ElementShared::U16(self.rng.gen()),
      ElementType::U32 => ElementShared::U32(self.rng.gen()),
      ElementType::U64 => ElementShared::U64(self.rng.gen()),
      ElementType::F32 => ElementShared::F32(self.rng.gen()),
      ElementType::F64 => ElementShared::F64(self.rng.gen()),
      ElementType::Data => ElementShared::Data(self.gen_data_element()),
      ElementType::Text => ElementShared::Text(self.gen_text_element()),
      ElementType::Enum(x) => ElementShared::Enum(self.gen_enum_element(x)),
      ElementType::Struct(x) => ElementShared::Struct(self.gen_struct_element(x)),
      ElementType::List(x) => ElementShared::ListDecoded(self.gen_list_element(x)),
      ElementType::Union(x) => ElementShared::Union(self.gen_union_element(x)),
    }
  }

  fn gen_data_element(&mut self) -> DataElementShared {
    DataElementShared((0..self.rng.gen_range(0, 5)).map(|_| self.rng.gen()).collect())
  }

  fn gen_text_element(&mut self) -> TextElementShared {
    TextElementShared(
      (0..self.rng.gen_range(0, 5)).map(|_| self.rng.sample(Alphanumeric)).collect(),
    )
  }

  fn gen_enum_element(&mut self, meta: &'static EnumMeta) -> EnumElement {
    let discriminant = Discriminant(self.rng.gen_range(0, meta.enumerants.len() as u16));
    EnumElement(meta, discriminant)
  }

  fn gen_struct_element(&mut self, meta: &'static StructMeta) -> StructElementShared {
    StructElementShared(meta, self.gen_untyped_struct(meta))
  }

  fn gen_list_element(&mut self, meta: &'static ListMeta) -> ListDecodedElementShared {
    ListDecodedElementShared(meta, self.gen_element_list(&meta.value_type))
  }

  fn gen_element_list(&mut self, value_type: &ElementType) -> Vec<ElementShared> {
    // TODO: Use a Poisson (or user-selectable) distribution for this.
    let mut len = self.rng.gen_range(0, 3);
    fn should_trim(value_type: &ElementType) -> bool {
      match value_type {
        ElementType::Struct(_) => true,
        ElementType::Union(x) => {
          x.variants.iter().any(|v| should_trim(&v.field_meta.element_type()))
        }
        _ => false,
      }
    }
    if should_trim(value_type) {
      len = cmp::min(len, self.max_struct_recursion);
      self.max_struct_recursion -= len;
    }
    (0..len).map(|_| self.gen_element(value_type)).collect()
  }

  fn gen_union_element(&mut self, meta: &'static UnionMeta) -> UnionElementShared {
    let variant_meta = &meta.variants[self.rng.gen_range(0, meta.variants.len())];
    UnionElementShared(
      meta,
      variant_meta.discriminant,
      Box::new(self.gen_element(&variant_meta.field_meta.element_type())),
    )
  }
}

#[cfg(test)]
mod test {
  use rand;
  use std::error::Error;

  use crate::samples::carsales_capnp::{
    ParkingLotRef, ParkingLotShared, TotalValueRef, TotalValueShared,
  };
  use crate::samples::catrank_capnp::{SearchResultListRef, SearchResultListShared};
  use crate::samples::eval_capnp::{
    EvaluationResultRef, EvaluationResultShared, ExpressionRef, ExpressionShared,
  };
  use crate::samples::rast_capnp::{MessageRef, MessageShared};
  use crate::samples::test_capnp::{TestAllTypesRef, TestAllTypesShared};
  use capnp_runtime::segment_framing_alternate;

  // TODO: Pull out all the common bits here.

  #[test]
  fn rand_roundtrip_carsales() -> Result<(), Box<dyn Error>> {
    {
      let before: ParkingLotShared =
        capnp_runtime::rand::Rand::new(&mut rand::thread_rng(), 20).gen_typed_struct();
      let mut buf = Vec::new();
      segment_framing_alternate::encode(&mut buf, &before.capnp_as_ref())?;
      let after: ParkingLotRef = segment_framing_alternate::decode_buf(&mut &*buf)?;
      assert_eq!(before.capnp_as_ref(), after);
    }
    {
      let before: TotalValueShared =
        capnp_runtime::rand::Rand::new(&mut rand::thread_rng(), 20).gen_typed_struct();
      let mut buf = Vec::new();
      segment_framing_alternate::encode(&mut buf, &before.capnp_as_ref())?;
      let after: TotalValueRef = segment_framing_alternate::decode_buf(&mut &*buf)?;
      assert_eq!(before.capnp_as_ref(), after);
    }
    Ok(())
  }

  #[test]
  fn rand_roundtrip_catrank() -> Result<(), Box<dyn Error>> {
    let before: SearchResultListShared =
      capnp_runtime::rand::Rand::new(&mut rand::thread_rng(), 20).gen_typed_struct();
    let mut buf = Vec::new();
    segment_framing_alternate::encode(&mut buf, &before.capnp_as_ref())?;
    let after: SearchResultListRef = segment_framing_alternate::decode_buf(&mut &*buf)?;
    assert_eq!(before.capnp_as_ref(), after);
    Ok(())
  }

  #[test]
  fn rand_roundtrip_eval() -> Result<(), Box<dyn Error>> {
    {
      let before: ExpressionShared =
        capnp_runtime::rand::Rand::new(&mut rand::thread_rng(), 10).gen_typed_struct();
      let mut buf = Vec::new();
      segment_framing_alternate::encode(&mut buf, &before.capnp_as_ref())?;
      let after: ExpressionRef = segment_framing_alternate::decode_buf(&mut &*buf)?;
      assert_eq!(before.capnp_as_ref(), after);
    }
    {
      let before: EvaluationResultShared =
        capnp_runtime::rand::Rand::new(&mut rand::thread_rng(), 20).gen_typed_struct();
      let mut buf = Vec::new();
      segment_framing_alternate::encode(&mut buf, &before.capnp_as_ref())?;
      let after: EvaluationResultRef = segment_framing_alternate::decode_buf(&mut &*buf)?;
      assert_eq!(before.capnp_as_ref(), after);
    }
    Ok(())
  }

  #[test]
  fn rand_roundtrip_rast() -> Result<(), Box<dyn Error>> {
    let before: MessageShared =
      capnp_runtime::rand::Rand::new(&mut rand::thread_rng(), 20).gen_typed_struct();
    let mut buf = Vec::new();
    segment_framing_alternate::encode(&mut buf, &before.capnp_as_ref())?;
    let after: MessageRef = segment_framing_alternate::decode_buf(&mut &*buf)?;
    assert_eq!(before.capnp_as_ref(), after);
    Ok(())
  }

  #[test]
  fn rand_roundtrip_test() -> Result<(), Box<dyn Error>> {
    let before: TestAllTypesShared =
      capnp_runtime::rand::Rand::new(&mut rand::thread_rng(), 20).gen_typed_struct();
    let mut buf = Vec::new();
    segment_framing_alternate::encode(&mut buf, &before.capnp_as_ref())?;
    let after: TestAllTypesRef = segment_framing_alternate::decode_buf(&mut &*buf)?;
    assert_eq!(before.capnp_as_ref(), after);
    Ok(())
  }
}
