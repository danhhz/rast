// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::convert::TryInto;

use crate::common::{ElementWidth, NumElements, NumWords};
use crate::error::Error;
use crate::segment::SegmentID;

#[derive(Debug)]
pub enum Pointer {
  Null,
  Struct(StructPointer),
  List(ListPointer),
  Far(FarPointer),
  Other([u8; 8]),
}

impl Pointer {
  pub fn decode(buf: [u8; 8]) -> Pointer {
    if u64::from_le_bytes(buf) == 0 {
      return Pointer::Null;
    }
    let pointer_type =
      u32::from_le_bytes(buf[0..4].try_into().unwrap()) & 0b_00000000_00000000_00000000_00000011;
    match pointer_type {
      0 => Pointer::Struct(StructPointer::decode(buf)),
      1 => Pointer::List(ListPointer::decode(buf)),
      2 => Pointer::Far(FarPointer::decode(buf)),
      3 => Pointer::Other(buf),
      _ => unreachable!(),
    }
  }

  pub fn encode(&self) -> [u8; 8] {
    match self {
      Pointer::Null => [0; 8],
      Pointer::Struct(x) => x.encode(),
      Pointer::List(x) => x.encode(),
      Pointer::Far(x) => x.encode(),
      Pointer::Other(x) => x.clone(),
    }
  }
}

#[derive(Clone, Debug)]
pub struct StructPointer {
  pub off: NumWords,
  pub data_size: NumWords,
  pub pointer_size: NumWords,
}

impl StructPointer {
  pub fn empty() -> Self {
    StructPointer { off: NumWords(0), data_size: NumWords(0), pointer_size: NumWords(0) }
  }

  fn decode(buf: [u8; 8]) -> Self {
    let offset_words = i32::from_le_bytes(buf[0..4].try_into().unwrap()) >> 2;
    // debug_assert_eq!(offset_words, offset_words >> 2);
    let data_size_words = u16::from_le_bytes(buf[4..6].try_into().unwrap());
    let pointer_size_words = u16::from_le_bytes(buf[6..8].try_into().unwrap());
    StructPointer {
      off: NumWords(offset_words),
      data_size: NumWords(i32::from(data_size_words)),
      pointer_size: NumWords(i32::from(pointer_size_words)),
    }
  }

  pub fn encode(&self) -> [u8; 8] {
    let mut buf: [u8; 8] = [0; 8];
    buf[0..4].copy_from_slice(&u32::to_le_bytes((self.off.0 as u32) << 2));
    buf[4..6].copy_from_slice(&u16::to_le_bytes(self.data_size.0 as u16));
    buf[6..8].copy_from_slice(&u16::to_le_bytes(self.pointer_size.0 as u16));
    buf
  }
}

#[derive(Debug, Clone)]
pub enum ListLayout {
  Packed(NumElements, ElementWidth),
  Composite(NumWords),
}

#[derive(Debug, Clone)]
pub struct ListPointer {
  pub off: NumWords,
  pub layout: ListLayout,
}

impl ListPointer {
  pub fn empty() -> Self {
    ListPointer { off: NumWords(0), layout: ListLayout::Packed(NumElements(0), ElementWidth::Void) }
  }

  fn decode(buf: [u8; 8]) -> Self {
    let off = NumWords(i32::from_le_bytes(buf[0..4].try_into().unwrap()) >> 2);
    let element_type =
      u32::from_le_bytes(buf[4..8].try_into().unwrap()) & 0b_00000000_00000000_00000000_00000111;
    // We shift right 3 places, so it's guaranteed to fit in an i32.
    let len = i32::from_le_bytes(buf[4..8].try_into().unwrap()) >> 3;
    let layout = match element_type {
      0 => ListLayout::Packed(NumElements(len), ElementWidth::Void),
      1 => ListLayout::Packed(NumElements(len), ElementWidth::OneBit),
      2 => ListLayout::Packed(NumElements(len), ElementWidth::OneByte),
      3 => ListLayout::Packed(NumElements(len), ElementWidth::TwoBytes),
      4 => ListLayout::Packed(NumElements(len), ElementWidth::FourBytes),
      5 => ListLayout::Packed(NumElements(len), ElementWidth::EightBytesNonPointer),
      6 => ListLayout::Packed(NumElements(len), ElementWidth::EightBytesPointer),
      7 => ListLayout::Composite(NumWords(len)),
      _ => unreachable!(),
    };
    ListPointer { off: off, layout: layout }
  }

  pub fn encode(&self) -> [u8; 8] {
    let (len, element_type) = match &self.layout {
      ListLayout::Packed(NumElements(len), width) => (
        *len,
        match width {
          ElementWidth::Void => 0,
          ElementWidth::OneBit => 1,
          ElementWidth::OneByte => 2,
          ElementWidth::TwoBytes => 3,
          ElementWidth::FourBytes => 4,
          ElementWidth::EightBytesNonPointer => 5,
          ElementWidth::EightBytesPointer => 6,
        },
      ),
      ListLayout::Composite(NumWords(len)) => (*len, 7),
    };

    let mut buf: [u8; 8] = [0; 8];
    buf[0..4].copy_from_slice(&u32::to_le_bytes(
      (self.off.0 as u32) << 2 | 0b_00000000_00000000_00000000_00000001,
    ));
    buf[4..8].copy_from_slice(&u32::to_le_bytes((len as u32) << 3 | element_type));
    buf
  }
}

#[derive(Debug)]
pub enum LandingPadSize {
  OneWord,
  TwoWords,
}

#[derive(Debug)]
pub struct FarPointer {
  pub landing_pad_size: LandingPadSize,
  pub off: NumWords,
  pub seg: SegmentID,
}

impl FarPointer {
  fn decode(buf: [u8; 8]) -> Self {
    let landing_pad_size = u8::from_le_bytes(buf[0..1].try_into().unwrap()) & 0b00000100;
    let landing_pad_size =
      if landing_pad_size == 0 { LandingPadSize::OneWord } else { LandingPadSize::TwoWords };
    // We shift right 3 places, so it's guaranteed to fit in an i32.
    let offset = i32::from_le_bytes(buf[0..4].try_into().unwrap()) >> 3;
    let segment_id = u32::from_le_bytes(buf[4..8].try_into().unwrap());
    FarPointer {
      landing_pad_size: landing_pad_size,
      off: NumWords(offset),
      seg: SegmentID(segment_id),
    }
  }

  pub fn encode(&self) -> [u8; 8] {
    let mut buf: [u8; 8] = [0; 8];
    let landing_pad = match self.landing_pad_size {
      LandingPadSize::OneWord => 0,
      LandingPadSize::TwoWords => 1,
    };
    buf[0..4].copy_from_slice(&u32::to_le_bytes(
      (self.off.0 as u32) << 3 | landing_pad << 2 | 0b_00000000_00000000_00000000_00000010,
    ));
    buf[4..8].copy_from_slice(&u32::to_le_bytes(self.seg.0));
    buf
  }
}

#[derive(Debug, Clone)]
pub struct ListCompositeTag {
  pub num_elements: NumElements,
  pub data_size: NumWords,
  pub pointer_size: NumWords,
}

impl ListCompositeTag {
  pub fn decode(buf: [u8; 8]) -> Result<Self, Error> {
    // The tag has the same layout as a struct pointer, except that the pointer
    // offset (B) instead indicates the number of elements in the list.
    let sp = match Pointer::decode(buf) {
      Pointer::Struct(sp) => sp,
      x => return Err(Error::Encoding(format!("expected composite tag got: {:?}", x))),
    };
    Ok(ListCompositeTag {
      num_elements: NumElements(sp.off.0),
      data_size: sp.data_size,
      pointer_size: sp.pointer_size,
    })
  }

  pub fn encode(&self) -> [u8; 8] {
    StructPointer {
      off: NumWords(self.num_elements.0),
      data_size: self.data_size,
      pointer_size: self.pointer_size,
    }
    .encode()
  }
}
