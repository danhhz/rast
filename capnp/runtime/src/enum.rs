// Copyright 2020 Daniel Harrison. All Rights Reserved.

//! Cap'n Proto [enum]
//!
//! WIP
//!
//! [enum]: crate#enum

use crate::common::Discriminant;
use crate::error::UnknownDiscriminant;

/// A codegen'd Cap'n Proto enum
pub trait TypedEnum: Sized {
  /// The schema of this enum
  fn meta() -> &'static EnumMeta;
  /// Returns the enumerant of this enum with the given discriminant.
  fn from_discriminant(discriminant: Discriminant) -> Result<Self, UnknownDiscriminant>;
  /// Returns the discriminant of this enumerant.
  fn to_discriminant(&self) -> Discriminant;
}

/// Schema for one enumerant of the enum representing a Cap'n Proto enum
#[derive(Debug)]
pub struct EnumerantMeta {
  /// The enumerants's name
  pub name: &'static str,
  /// The enumerants's encoding discriminant
  pub discriminant: Discriminant,
}

/// Schema for a Cap'n Proto enum
#[derive(Debug)]
pub struct EnumMeta {
  /// The name of this enum
  pub name: &'static str,
  /// The enumerants of this enum
  pub enumerants: &'static [EnumerantMeta],
}

impl EnumMeta {
  /// Returns the
  pub fn get(&self, value: Discriminant) -> Option<&EnumerantMeta> {
    // TODO: This should be correct but feels sketchy
    self.enumerants.get(value.0 as usize)
  }
}
