// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::collections::HashMap;
use std::convert::TryInto;
use std::iter::Iterator;
use std::rc::Rc;

use crate::common::{
  NumElements, NumWords, POINTER_WIDTH_BYTES, U16_WIDTH_BYTES, U64_WIDTH_BYTES, U8_WIDTH_BYTES,
  WORD_BYTES,
};
use crate::error::Error;
use crate::pointer::Pointer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SegmentID(pub u32);

#[derive(Debug)]
pub struct SegmentOwned {
  pub buf: Vec<u8>,
  pub other: HashMap<SegmentID, SegmentShared>,
}

impl SegmentOwned {
  pub fn new_from_buf(buf: Vec<u8>) -> SegmentOwned {
    SegmentOwned { buf: buf, other: HashMap::new() }
  }

  pub fn into_shared(self) -> SegmentShared {
    SegmentShared { buf: Rc::new(self.buf), other: Rc::new(self.other) }
  }

  pub fn len_words_rounded_up(&self) -> NumWords {
    // WIP: Verify soundness of this i32 conversion
    NumWords(((self.buf.len() + WORD_BYTES - 1) / WORD_BYTES) as i32)
  }

  pub fn set_u8(&mut self, off: NumWords, offset: NumElements, value: u8) {
    let begin = off.as_bytes() + offset.as_bytes(U8_WIDTH_BYTES);
    let end = begin + U8_WIDTH_BYTES;
    if self.buf.len() < end {
      self.buf.resize(end, 0);
    }
    // NB: This range is guaranteed to exist because we just resized it.
    self.buf[begin..end].copy_from_slice(&u8::to_le_bytes(value));
  }

  pub fn set_u16(&mut self, off: NumWords, offset: NumElements, value: u16) {
    let begin = off.as_bytes() + offset.as_bytes(U16_WIDTH_BYTES);
    let end = begin + U16_WIDTH_BYTES;
    if self.buf.len() < end {
      self.buf.resize(end, 0);
    }
    // NB: This range is guaranteed to exist because we just resized it.
    self.buf[begin..end].copy_from_slice(&u16::to_le_bytes(value));
  }

  pub fn set_u64(&mut self, off: NumWords, offset: NumElements, value: u64) {
    let begin = off.as_bytes() + offset.as_bytes(U64_WIDTH_BYTES);
    let end = begin + U64_WIDTH_BYTES;
    if self.buf.len() < end {
      self.buf.resize(end, 0);
    }
    // NB: This range is guaranteed to exist because we just resized it.
    self.buf[begin..end].copy_from_slice(&u64::to_le_bytes(value));
  }

  pub fn set_pointer(&mut self, off: NumWords, offset: NumElements, value: Pointer) {
    let begin = off.as_bytes() + offset.as_bytes(POINTER_WIDTH_BYTES);
    let end = begin + POINTER_WIDTH_BYTES;
    if self.buf.len() < end {
      self.buf.resize(end, 0);
    }
    // NB: This range is guaranteed to exist because we just resized it.
    self.buf[begin..end].copy_from_slice(&value.encode());
  }
}

#[derive(Debug, Clone)]
pub struct SegmentShared {
  buf: Rc<Vec<u8>>,
  other: Rc<HashMap<SegmentID, SegmentShared>>,
}

impl SegmentShared {
  pub fn as_ref<'a>(&'a self) -> SegmentBorrowed<'a> {
    let mut other: HashMap<SegmentID, &'a [u8]> = HashMap::with_capacity(self.other.len());
    for (k, v) in self.other.iter() {
      other.insert(*k, &v.buf);
    }
    SegmentBorrowed { buf: &self.buf, other: Some(Rc::new(other)) }
  }

  pub fn buf(&self) -> &[u8] {
    &self.buf
  }

  pub fn all_other<'a>(&'a self) -> Vec<(SegmentID, SegmentShared)> {
    self.other.iter().map(|(id, s)| (*id, s.clone())).collect()
  }
}

#[derive(Debug, Clone)]
pub struct SegmentBorrowed<'a> {
  buf: &'a [u8],
  other: Option<Rc<HashMap<SegmentID, &'a [u8]>>>,
}

impl<'a> SegmentBorrowed<'a> {
  const EMPTY_BUF: [u8; 0] = [0; 0];

  pub fn empty() -> SegmentBorrowed<'a> {
    SegmentBorrowed { buf: &SegmentBorrowed::EMPTY_BUF, other: None }
  }
}

#[derive(Debug, Clone)]
pub enum Segment<'a> {
  Shared(SegmentShared),
  Borrowed(SegmentBorrowed<'a>),
}

impl<'a> Segment<'a> {
  pub fn empty() -> Segment<'a> {
    Segment::Borrowed(SegmentBorrowed::empty())
  }

  pub fn buf(&'a self) -> &'a [u8] {
    match self {
      Segment::Shared(o) => o.buf.as_slice(),
      Segment::Borrowed(b) => b.buf,
    }
  }

  pub fn other(&self, id: SegmentID) -> Option<Segment<'a>> {
    match self {
      Segment::Shared(o) => o.other.get(&id).map(|s| Segment::Shared(s.clone())),
      Segment::Borrowed(b) => match &b.other {
        None => None,
        Some(other) => other
          .get(&id)
          .map(|buf| Segment::Borrowed(SegmentBorrowed { buf: buf, other: b.other.clone() })),
      },
    }
  }

  // TODO: Figure out how to return an iterator here.
  pub fn all_other(&self) -> Vec<(SegmentID, Segment<'a>)> {
    match self {
      Segment::Shared(o) => {
        o.other.iter().map(|(id, s)| (*id, Segment::Shared(s.clone()))).collect()
      }
      Segment::Borrowed(b) => match &b.other {
        None => vec![],
        Some(other) => other
          .iter()
          .map(|(id, buf)| {
            (*id, Segment::Borrowed(SegmentBorrowed { buf: buf, other: b.other.clone() }))
          })
          .collect(),
      },
    }
  }
}

pub fn decode_stream_official<'a>(buf: &'a [u8]) -> Result<Segment<'a>, Error> {
  let mut by_id = HashMap::new();

  let num_segments_bytes: [u8; 4] =
    buf.get(0..4).ok_or(Error::from("encoding: incomplete segment count"))?.try_into().unwrap();
  let num_segments_minus_one = u32::from_le_bytes(num_segments_bytes);
  let num_segments = num_segments_minus_one + 1;
  let mut size_offset = 4;
  let padding = if num_segments % 2 == 1 { 0 } else { 4 };
  let mut buf_offset = 4 * (1 + num_segments as usize) + padding;

  for idx in 0..num_segments {
    let id = SegmentID(idx);
    let segment_size_words = u32::from_le_bytes(
      buf
        .get(size_offset..size_offset + 4)
        .ok_or(Error::from(format!("encoding: invalid segment {:?} size", id)))?
        .try_into()
        .unwrap(),
    );
    let segment_size_bytes = segment_size_words as usize * 8;
    let segment_bytes = buf
      .get(buf_offset..buf_offset + segment_size_bytes)
      .ok_or(Error::from(format!("encoding: insufficient segment {:?} bytes", id)))?;
    size_offset += 4;
    buf_offset += segment_size_bytes;
    by_id.insert(id, segment_bytes);
  }

  let first_segment_buf = by_id.get(&SegmentID(0)).ok_or(Error::from("encoding: no segments"))?;
  Ok(Segment::Borrowed(SegmentBorrowed { buf: first_segment_buf, other: Some(Rc::new(by_id)) }))
}

pub fn decode_stream_alternate<'a>(buf: &'a [u8]) -> Result<Segment<'a>, Error> {
  let mut by_id = HashMap::new();

  let mut buf_offset = 0;
  while buf_offset < buf.len() {
    let id = SegmentID(u32::from_le_bytes(
      buf
        .get(buf_offset..buf_offset + 4)
        .ok_or(Error::from(format!("encoding: incomplete segment id")))?
        .try_into()
        .unwrap(),
    ));
    buf_offset += 4;
    let segment_size_words = u32::from_le_bytes(
      buf
        .get(buf_offset..buf_offset + 4)
        .ok_or(Error::from(format!("encoding: invalid segment {:?} size", id)))?
        .try_into()
        .unwrap(),
    );
    buf_offset += 4;
    let segment_size_bytes = segment_size_words as usize * 8;
    let segment_bytes = buf
      .get(buf_offset..buf_offset + segment_size_bytes)
      .ok_or(Error::from(format!("encoding: insufficient segment {:?} bytes", id)))?;
    buf_offset += segment_size_bytes;
    by_id.insert(id, segment_bytes);
  }

  let first_segment_buf = by_id.get(&SegmentID(0)).ok_or(Error::from("encoding: no segments"))?;
  Ok(Segment::Borrowed(SegmentBorrowed { buf: first_segment_buf, other: Some(Rc::new(by_id)) }))
}