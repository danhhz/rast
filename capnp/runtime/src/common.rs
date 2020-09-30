// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::mem;
use std::ops::{Add, Mul, Sub};

/// A count of elements of some type
///
/// This represents N i32s or N lists, etc.
///
/// NB: This only ever uses the bottom 29 bits.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct NumElements(pub i32);

#[cfg(target_pointer_width = "64")]
impl NumElements {
  /// The number of bytes represented, given an element width in bytes
  pub const fn as_bytes(self, width: usize) -> usize {
    // TODO: Consolidate this with ElementWidth::list_len_bytes.
    self.0 as usize * width
  }
}

/// A count of 64-bit words
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct NumWords(pub i32);

#[cfg(target_pointer_width = "64")]
impl NumWords {
  /// The number of bytes represented
  pub const fn as_bytes(self) -> usize {
    self.0 as usize * 8
  }
}

impl Add<NumWords> for NumWords {
  type Output = NumWords;
  fn add(self, other: NumWords) -> NumWords {
    NumWords(self.0 + other.0)
  }
}

impl Sub<NumWords> for NumWords {
  type Output = NumWords;
  fn sub(self, other: NumWords) -> NumWords {
    NumWords(self.0 - other.0)
  }
}

impl Mul<NumElements> for NumWords {
  type Output = NumWords;
  fn mul(self, other: NumElements) -> NumWords {
    NumWords(self.0 * other.0)
  }
}

/// An enum or union discriminant
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Discriminant(pub u16);

pub const U8_WIDTH_BYTES: usize = mem::size_of::<u8>();
pub const U16_WIDTH_BYTES: usize = mem::size_of::<u16>();
pub const U32_WIDTH_BYTES: usize = mem::size_of::<u32>();
pub const U64_WIDTH_BYTES: usize = mem::size_of::<u64>();

pub const WORD_BYTES: usize = NumWords(1).as_bytes();
pub const POINTER_WIDTH_WORDS: NumWords = NumWords(1);
pub const POINTER_WIDTH_BYTES: usize = POINTER_WIDTH_WORDS.as_bytes();
pub const COMPOSITE_TAG_WIDTH_WORDS: NumWords = POINTER_WIDTH_WORDS;
pub const COMPOSITE_TAG_WIDTH_BYTES: usize = COMPOSITE_TAG_WIDTH_WORDS.as_bytes();

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum ElementWidth {
  Void,
  OneBit,
  OneByte,
  TwoBytes,
  FourBytes,
  EightBytesNonPointer,
  EightBytesPointer,
}

impl ElementWidth {
  pub fn list_len_bytes(&self, list_len: usize) -> usize {
    match self {
      ElementWidth::OneByte => 1 * list_len,
      ElementWidth::EightBytesNonPointer => 8 * list_len,
      ElementWidth::EightBytesPointer => 8 * list_len,
      _ => todo!(),
    }
  }
}

/// Used to return the reference companion of an owned or shared object
///
/// This is similar to [`std::convert::AsRef`], but with a lifetime parameter.
pub trait CapnpAsRef<'a, T> {
  /// Performs the conversion.
  fn capnp_as_ref(&'a self) -> T;
}

/// Used to return the owned companion of a shared or reference object
///
/// This is similar to [`std::borrow::ToOwned`], but with a lifetime parameter.
pub trait CapnpToOwned<'a>: Sized {
  // TODO: Rename CapnpAsRef to CapnpBorrow

  /// The resulting type after obtaining ownership.
  type Owned: CapnpAsRef<'a, Self>;
  /// Creates owned data from borrowed data by cloning.
  fn capnp_to_owned(&self) -> Self::Owned;
}

#[cfg(test)]
pub mod test {
  use std::fmt;

  pub struct PrettyBytes<'a>(pub &'a [u8]);

  impl<'a> fmt::Debug for PrettyBytes<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      for (idx, chunk) in self.0.chunks(8).enumerate() {
        write!(f, "{}: {:?}\n", idx * 8, chunk)?;
      }
      Ok(())
    }
  }
}
