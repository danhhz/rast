// Copyright 2020 Daniel Harrison. All Rights Reserved.

use crate::common::{Discriminant, NumElements};
use crate::error::{Error, UnknownDiscriminant};
use crate::field_meta::FieldMeta;
use crate::r#struct::{UntypedStruct, UntypedStructOwned};

pub trait TypedUnion<'a>: Sized {
  fn meta() -> &'static UnionMeta;
  // NB: Double Result is intentional for better error handling. See
  // https://sled.rs/errors.html
  fn from_untyped_union(
    data: &UntypedUnion<'a>,
  ) -> Result<Result<Self, UnknownDiscriminant>, Error>;
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

pub struct UntypedUnion<'a> {
  pub discriminant: Discriminant,
  pub variant_data: UntypedStruct<'a>,
}
