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

  pub fn encode(&self) -> [u8; 8] {
    let mut ret: [u8; 8] = [0; 8];
    // WIP: self.off
    ret[4..6].copy_from_slice(&u16::to_le_bytes(self.data_size.0 as u16));
    ret[6..8].copy_from_slice(&u16::to_le_bytes(self.pointer_size.0 as u16));
    ret
  }
}

#[derive(Debug)]
pub enum ListLayout {
  Packed(NumElements, ElementWidth),
  Composite(NumWords),
}

#[derive(Debug)]
pub struct ListPointer {
  pub off: NumWords,
  pub layout: ListLayout,
}

impl ListPointer {
  pub fn empty() -> Self {
    ListPointer { off: NumWords(0), layout: ListLayout::Packed(NumElements(0), ElementWidth::Void) }
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

pub fn decode_pointer(buf: [u8; 8]) -> Pointer {
  if u64::from_le_bytes(buf) == 0 {
    return Pointer::Null;
  }
  let pointer_type =
    u32::from_le_bytes(buf[0..4].try_into().unwrap()) & 0b_00000000_00000000_00000000_00000011;
  match pointer_type {
    0 => Pointer::Struct(decode_struct_pointer(buf)),
    1 => Pointer::List(decode_list_pointer(buf)),
    2 => Pointer::Far(decode_far_pointer(buf)),
    3 => Pointer::Other(buf),
    _ => unreachable!(),
  }
}

fn decode_struct_pointer(buf: [u8; 8]) -> StructPointer {
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

fn decode_list_pointer(buf: [u8; 8]) -> ListPointer {
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

fn decode_far_pointer(buf: [u8; 8]) -> FarPointer {
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

#[derive(Debug)]
pub struct ListCompositeTag {
  pub num_elements: NumElements,
  pub data_size: NumWords,
  pub pointer_size: NumWords,
}

pub fn decode_composite_tag(buf: [u8; 8]) -> Result<ListCompositeTag, Error> {
  // The tag has the same layout as a struct pointer, except that the pointer
  // offset (B) instead indicates the number of elements in the list.
  let sp = match decode_pointer(buf) {
    Pointer::Struct(sp) => sp,
    x => return Err(Error::from(format!("expected composite tag got: {:?}", x))),
  };
  Ok(ListCompositeTag {
    num_elements: NumElements(sp.off.0),
    data_size: sp.data_size,
    pointer_size: sp.pointer_size,
  })
}
