// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::Arc;

use crate::error::Error;
use crate::r#struct::{TypedStruct, UntypedStruct};
use crate::segment::{SegmentBorrowed, SegmentID};

pub fn official<'a, T: TypedStruct<'a>>(buf: &'a [u8]) -> Result<T, Error> {
  let seg = segments_official(buf)?;
  Ok(T::from_untyped_struct(UntypedStruct::from_root(seg)?))
}

pub fn alternate<'a, T: TypedStruct<'a>>(buf: &'a [u8]) -> Result<T, Error> {
  let seg = segments_alternate(buf)?;
  Ok(T::from_untyped_struct(UntypedStruct::from_root(seg)?))
}

fn segments_official<'a>(buf: &'a [u8]) -> Result<SegmentBorrowed<'a>, Error> {
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
  Ok(SegmentBorrowed::new(first_segment_buf, Some(Arc::new(by_id))))
}

pub fn segments_alternate<'a>(buf: &'a [u8]) -> Result<SegmentBorrowed<'a>, Error> {
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
  Ok(SegmentBorrowed::new(first_segment_buf, Some(Arc::new(by_id))))
}
