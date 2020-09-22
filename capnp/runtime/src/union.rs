// Copyright 2020 Daniel Harrison. All Rights Reserved.

//! Cap'n Proto [union]
//!
//! A union is two or more fields of a struct which are stored in the same
//! location. Only one of these fields can be set at a time, and a separate tag
//! is maintained to track which one is currently set.
//!
//! [union]: crate#union

use crate::common::{CapnpAsRef, Discriminant, NumElements};
use crate::error::{Error, UnknownDiscriminant};
use crate::field_meta::FieldMeta;
use crate::r#struct::{UntypedStruct, UntypedStructOwned};

/// A borrowed codegen'd Cap'n Proto union
pub trait TypedUnion<'a>: Sized {
  /// The schema of this union
  fn meta() -> &'static UnionMeta;
  /// Returns an instance of this union using the given data.
  ///
  /// The caller is responsible for matching the given data to this type.
  /// Presumably it is an encoded instance of a past or future schema of this
  /// same union.
  ///
  /// NB: Double Result is intentional for better error handling. See
  /// https://sled.rs/errors.html
  fn from_untyped_union(
    data: &UntypedUnion<'a>,
  ) -> Result<Result<Self, UnknownDiscriminant>, Error>;
}

/// A reference-counted codegen'd Cap'n Proto union
pub trait TypedUnionShared<'a, T: TypedUnion<'a>>: CapnpAsRef<'a, T> {
  /// Sets the given discriminant value for this union in the given struct.
  fn set(&self, data: &mut UntypedStructOwned, discriminant_offset: NumElements);
}

/// Schema for one variant of the enum representing a Cap'n Proto union
#[derive(Debug)]
pub struct UnionVariantMeta {
  /// The variant's encoding discriminant
  pub discriminant: Discriminant,
  /// Schema for this variant's data
  ///
  /// NB: A Cap'n Proto union exists only as part of one particular struct. This
  /// means the struct data associated with this field is the exact same struct
  /// data as the one containing this union.
  pub field_meta: FieldMeta,
}

/// Schema for a Cap'n Proto union
#[derive(Debug)]
pub struct UnionMeta {
  /// The name of this union
  pub name: &'static str,
  /// The variants of this union
  pub variants: &'static [UnionVariantMeta],
}

impl UnionMeta {
  /// Returns the
  pub fn get(&self, value: Discriminant) -> Option<&UnionVariantMeta> {
    // WIP this should be correct but feels sketchy
    self.variants.get(value.0 as usize)
  }
}

/// A reference-counted Cap'n Proto union without schema
pub struct UntypedUnion<'a> {
  /// The set discriminant value
  pub discriminant: Discriminant,
  /// The dataspace in which this union's variant data exists
  pub variant_data: UntypedStruct<'a>,
}
