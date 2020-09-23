// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::Hasher;
use std::iter::Iterator;
use std::sync::Arc;

use crate::common::{CapnpAsRef, CapnpToOwned, NumWords, WORD_BYTES};

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
    // TODO: Verify soundness of this i32 conversion
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
    // TODO: Box so this is stable
    h.write_usize(other.buf().as_ptr() as usize);
    let segment_id = SegmentID(h.finish() as u32);
    // TODO: Is this really needed? Makes things O(n^2).
    self.other.extend(other.all_other());
    self.other.insert(segment_id, other.buf);
    segment_id
  }
}

impl<'a> CapnpAsRef<'a, SegmentBorrowed<'a>> for SegmentOwned {
  fn capnp_as_ref(&'a self) -> SegmentBorrowed<'a> {
    // TODO: Make this less expensive.
    let mut other: HashMap<SegmentID, &'a [u8]> = HashMap::with_capacity(self.other.len());
    for (k, v) in self.other.iter() {
      other.insert(*k, &v);
    }
    SegmentBorrowed { buf: &self.buf, other: Some(Arc::new(other)) }
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

impl<'a> CapnpAsRef<'a, SegmentBorrowed<'a>> for SegmentShared {
  fn capnp_as_ref(&'a self) -> SegmentBorrowed<'a> {
    let mut other: HashMap<SegmentID, &'a [u8]> = HashMap::with_capacity(self.other.len());
    for (k, v) in self.other.iter() {
      other.insert(*k, &v);
    }
    SegmentBorrowed { buf: &self.buf, other: Some(Arc::new(other)) }
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

  pub fn new(buf: &'a [u8], other: Option<Arc<HashMap<SegmentID, &'a [u8]>>>) -> Self {
    SegmentBorrowed { buf: buf, other: other }
  }

  pub fn buf(&self) -> &'a [u8] {
    self.buf
  }

  pub fn other(&self, id: SegmentID) -> Option<SegmentBorrowed<'a>> {
    match &self.other {
      None => None,
      Some(other) => {
        other.get(&id).map(|buf| SegmentBorrowed { buf: buf, other: self.other.clone() })
      }
    }
  }

  pub fn all_other(&self) -> Vec<(SegmentID, SegmentBorrowed<'a>)> {
    match &self.other {
      None => vec![],
      Some(other) => other
        .iter()
        .map(|(id, buf)| (*id, SegmentBorrowed { buf: buf, other: self.other.clone() }))
        .collect(),
    }
  }
}

impl<'a> CapnpToOwned<'a> for SegmentBorrowed<'a> {
  type Owned = SegmentShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    fn collect(seg: &SegmentBorrowed<'_>, by_id: &mut HashMap<SegmentID, Arc<Vec<u8>>>) {
      for (id, other_seg) in seg.all_other().into_iter() {
        if by_id.contains_key(&id) {
          continue;
        }
        by_id.insert(id, Arc::new(other_seg.buf.to_vec()));
        collect(&other_seg, by_id);
      }
    };
    let mut by_id = HashMap::new();
    collect(self, &mut by_id);
    SegmentShared { buf: Arc::new(self.buf.to_vec()), other: Arc::new(by_id) }
  }
}
