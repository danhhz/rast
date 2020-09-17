// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::convert::TryInto;
use std::hash::Hasher;
use std::iter::Iterator;
use std::sync::Arc;

use crate::common::{CapnpAsRef, CapnpToOwned, NumWords, WORD_BYTES};
use crate::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SegmentID(pub u32);

#[derive(Debug)]
pub struct SegmentOwned {
  buf: Vec<u8>,
  other: HashMap<SegmentID, Arc<Vec<u8>>>,
}

impl SegmentOwned {
  pub fn new_from_buf(buf: Vec<u8>) -> SegmentOwned {
    SegmentOwned { buf: buf, other: HashMap::new() }
  }

  pub fn into_shared(self) -> SegmentShared {
    SegmentShared { buf: Arc::new(self.buf), other: Arc::new(self.other) }
  }

  pub fn len_words_rounded_up(&mut self) -> NumWords {
    // WIP: Verify soundness of this i32 conversion
    NumWords(((self.buf.len() + WORD_BYTES - 1) / WORD_BYTES) as i32)
  }

  pub fn buf_mut(&mut self) -> &mut [u8] {
    &mut self.buf
  }

  pub fn ensure_len(&mut self, len_bytes: usize) {
    // TODO: Segments should always stay word-sized.
    if self.buf.len() < len_bytes {
      self.buf.resize(len_bytes, 0);
    }
  }

  pub fn other_reference(&mut self, other: SegmentShared) -> SegmentID {
    let mut h = DefaultHasher::new();
    // WIP: Box so this is stable
    h.write_usize(other.buf().as_ptr() as usize);
    let segment_id = SegmentID(h.finish() as u32);
    // WIP: Is this really needed? Makes things O(n^2).
    self.other.extend(other.all_other());
    self.other.insert(segment_id, other.buf);
    segment_id
  }
}

#[derive(Debug, Clone)]
pub struct SegmentShared {
  buf: Arc<Vec<u8>>,
  other: Arc<HashMap<SegmentID, Arc<Vec<u8>>>>,
}

impl SegmentShared {
  pub fn buf(&self) -> &[u8] {
    &self.buf
  }

  pub fn all_other<'a>(&'a self) -> Vec<(SegmentID, Arc<Vec<u8>>)> {
    self.other.iter().map(|(id, s)| (*id, s.clone())).collect()
  }
}

impl<'a> CapnpAsRef<'a, Segment<'a>> for SegmentShared {
  fn capnp_as_ref(&'a self) -> Segment<'a> {
    let mut other: HashMap<SegmentID, &'a [u8]> = HashMap::with_capacity(self.other.len());
    for (k, v) in self.other.iter() {
      other.insert(*k, &v);
    }
    Segment::Borrowed(SegmentBorrowed { buf: &self.buf, other: Some(Arc::new(other)) })
  }
}

#[derive(Debug, Clone)]
pub struct SegmentBorrowed<'a> {
  buf: &'a [u8],
  other: Option<Arc<HashMap<SegmentID, &'a [u8]>>>,
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
      Segment::Shared(o) => o
        .other
        .get(&id)
        .map(|s| Segment::Shared(SegmentShared { buf: s.clone(), other: o.other.clone() })),
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
      Segment::Shared(o) => o
        .other
        .iter()
        .map(|(id, s)| {
          (*id, Segment::Shared(SegmentShared { buf: s.clone(), other: o.other.clone() }))
        })
        .collect(),
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

impl<'a> CapnpToOwned<'a> for Segment<'a> {
  type Owned = SegmentShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    fn collect(seg: &Segment<'_>, by_id: &mut HashMap<SegmentID, Arc<Vec<u8>>>) {
      for (id, other_seg) in seg.all_other().into_iter() {
        if by_id.contains_key(&id) {
          continue;
        }
        by_id.insert(id, Arc::new(other_seg.buf().to_vec()));
        collect(&other_seg, by_id);
      }
    };
    let mut by_id = HashMap::new();
    collect(self, &mut by_id);
    SegmentShared { buf: Arc::new(self.buf().to_vec()), other: Arc::new(by_id) }
  }
}

pub fn decode_stream_official<'a>(buf: &'a [u8]) -> Result<Segment<'a>, Error> {
  let mut by_id = HashMap::new();

  let num_segments_bytes: [u8; 4] = buf
    .get(0..4)
    .ok_or_else(|| Error::Encoding(format!("incomplete segment count")))?
    .try_into()
    .unwrap();
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
        .ok_or_else(|| Error::Encoding(format!("invalid segment {:?} size", id)))?
        .try_into()
        .unwrap(),
    );
    let segment_size_bytes = segment_size_words as usize * 8;
    let segment_bytes = buf
      .get(buf_offset..buf_offset + segment_size_bytes)
      .ok_or_else(|| Error::Encoding(format!("insufficient segment {:?} bytes", id)))?;
    size_offset += 4;
    buf_offset += segment_size_bytes;
    by_id.insert(id, segment_bytes);
  }

  let first_segment_buf =
    by_id.get(&SegmentID(0)).ok_or_else(|| Error::Encoding(format!("no segments")))?;
  Ok(Segment::Borrowed(SegmentBorrowed { buf: first_segment_buf, other: Some(Arc::new(by_id)) }))
}

pub fn decode_stream_alternate<'a>(buf: &'a [u8]) -> Result<Segment<'a>, Error> {
  let mut by_id = HashMap::new();

  let mut buf_offset = 0;
  while buf_offset < buf.len() {
    let id = SegmentID(u32::from_le_bytes(
      buf
        .get(buf_offset..buf_offset + 4)
        .ok_or_else(|| Error::Encoding(format!("incomplete segment id")))?
        .try_into()
        .unwrap(),
    ));
    buf_offset += 4;
    let segment_size_words = u32::from_le_bytes(
      buf
        .get(buf_offset..buf_offset + 4)
        .ok_or_else(|| Error::Encoding(format!("invalid segment {:?} size", id)))?
        .try_into()
        .unwrap(),
    );
    buf_offset += 4;
    let segment_size_bytes = segment_size_words as usize * 8;
    let segment_bytes = buf
      .get(buf_offset..buf_offset + segment_size_bytes)
      .ok_or_else(|| Error::Encoding(format!("insufficient segment {:?} bytes", id)))?;
    buf_offset += segment_size_bytes;
    by_id.insert(id, segment_bytes);
  }

  let first_segment_buf =
    by_id.get(&SegmentID(0)).ok_or_else(|| Error::Encoding(format!("no segments")))?;
  Ok(Segment::Borrowed(SegmentBorrowed { buf: first_segment_buf, other: Some(Arc::new(by_id)) }))
}
