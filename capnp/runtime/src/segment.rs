// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::collections::HashMap;
use std::convert::TryInto;
use std::rc::Rc;

use crate::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SegmentID(pub u32);

#[derive(Debug, Clone)]
pub struct SegmentOwned {
  // TODO: The whole structure of this probably has to change to implement
  // mutability on the generated capnp structs. This Rc, for example, will have
  // to go.
  buf: Rc<Vec<u8>>,
  other: Rc<HashMap<SegmentID, SegmentOwned>>,
}

impl SegmentOwned {
  pub fn as_ref<'a>(&'a self) -> SegmentBorrowed<'a> {
    let mut other: HashMap<SegmentID, &'a [u8]> = HashMap::with_capacity(self.other.len());
    for (k, v) in self.other.iter() {
      other.insert(*k, &v.buf);
    }
    SegmentBorrowed { buf: &self.buf, other: Some(Rc::new(other)) }
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
  Owned(SegmentOwned),
  Borrowed(SegmentBorrowed<'a>),
}

impl<'a> Segment<'a> {
  pub fn empty() -> Segment<'a> {
    Segment::Borrowed(SegmentBorrowed::empty())
  }

  pub fn buf(&'a self) -> &'a [u8] {
    match self {
      Segment::Owned(o) => o.buf.as_slice(),
      Segment::Borrowed(b) => b.buf,
    }
  }

  pub fn other(&self, id: SegmentID) -> Option<Segment<'a>> {
    match self {
      Segment::Owned(o) => o.other.get(&id).map(|s| Segment::Owned(s.clone())),
      Segment::Borrowed(b) => match &b.other {
        None => None,
        Some(other) => other
          .get(&id)
          .map(|buf| Segment::Borrowed(SegmentBorrowed { buf: buf, other: b.other.clone() })),
      },
    }
  }
}

pub fn decode_segment<'a>(buf: &'a [u8]) -> Result<Segment<'a>, Error> {
  let mut by_id = HashMap::new();

  let num_segments_bytes: [u8; 4] =
    buf.get(0..4).ok_or(Error("invalid segment count"))?.try_into().unwrap();
  let num_segments_minus_one = u32::from_le_bytes(num_segments_bytes);
  let num_segments = num_segments_minus_one + 1;
  let mut size_offset = 4;
  let padding = if num_segments % 2 == 1 { 0 } else { 4 };
  let mut buf_offset = 4 * (1 + num_segments as usize) + padding;

  for idx in 0..num_segments {
    let segment_size_words = u32::from_le_bytes(
      buf
        .get(size_offset..size_offset + 4)
        .ok_or(Error("invalid segment size"))?
        .try_into()
        .unwrap(),
    );
    let segment_size_bytes = segment_size_words as usize * 8;
    let segment_bytes = buf
      .get(buf_offset..buf_offset + segment_size_bytes)
      .ok_or(Error("insufficient segment bytes"))?;
    size_offset += 4;
    buf_offset += segment_size_bytes;
    by_id.insert(SegmentID(idx), segment_bytes);
  }

  let first_segment_buf = by_id.get(&SegmentID(0)).ok_or(Error("missing first segment"))?;
  Ok(Segment::Borrowed(SegmentBorrowed { buf: first_segment_buf, other: Some(Rc::new(by_id)) }))
}
