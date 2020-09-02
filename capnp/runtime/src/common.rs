// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::mem;
use std::ops::{Add, Mul, Sub};

// NB: This only ever uses the bottom 29 bits.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct NumElements(pub i32);

#[cfg(target_pointer_width = "64")]
impl NumElements {
  pub const fn as_bytes(&self, width: usize) -> usize {
    self.0 as usize * width
  }
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct NumWords(pub i32);

#[cfg(target_pointer_width = "64")]
impl NumWords {
  pub const fn as_bytes(&self) -> usize {
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

pub const U8_WIDTH_BYTES: usize = mem::size_of::<u8>();
pub const U16_WIDTH_BYTES: usize = mem::size_of::<u16>();
pub const U64_WIDTH_BYTES: usize = mem::size_of::<u64>();
pub const DISCRIMINANT_WIDTH_BYTES: usize = U16_WIDTH_BYTES;

pub const WORD_BYTES: usize = NumWords(1).as_bytes();
pub const POINTER_WIDTH_WORDS: NumWords = NumWords(1);
pub const POINTER_WIDTH_BYTES: usize = POINTER_WIDTH_WORDS.as_bytes();
pub const COMPOSITE_TAG_WIDTH_BYTES: usize = U64_WIDTH_BYTES;

#[derive(Debug, Clone)]
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
