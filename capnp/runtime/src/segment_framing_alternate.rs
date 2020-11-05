// Copyright 2020 Daniel Harrison. All Rights Reserved.

//! An alternate segment framing used by this library
//!
//! The [official framing] uses sequential segment identifiers, which would
//! require rewriting many pointers during serialization. This format lifts that
//! restriction by encoding the segment ids.
//!
//! The encoding is any number of repetitions of:
//!
//! ```text
//! [4 byte segment id][4 byte segment len][segment data][padding to 64-bit word boundary]
//! ```

use std::collections::HashMap;
use std::collections::HashSet;
use std::convert::TryInto;
use std::io::{self, BufRead, Write};
use std::sync::Arc;

use crate::error::Error;
use crate::r#struct::{TypedStructRef, TypedStructShared, UntypedStruct, UntypedStructOwned};
use crate::segment::{SegmentBorrowed, SegmentID, SegmentOwned};

/// Decode the alternate Cap'n Proto segment framing used by this library.
pub fn decode<T: TypedStructShared, R: BufRead>(r: &mut R) -> Result<T, Error> {
  let seg = decode_segments(r)?;
  Ok(T::from_untyped_struct(UntypedStructOwned::from_root(seg)?.into_shared()))
}

fn decode_segments<R: BufRead>(r: &mut R) -> Result<SegmentOwned, Error> {
  let mut by_id = HashMap::new();

  let mut raw = [0u8; 4];
  let mut read_u32 = |r: &mut R| -> Result<u32, io::Error> {
    r.read_exact(&mut raw[..])?;
    Ok(u32::from_le_bytes(raw))
  };

  loop {
    let id =
      SegmentID(read_u32(r).or_else(|_| Err(Error::Encoding(format!("incomplete segment id"))))?);
    // The first segment is always id 0. A second segment id 0 is the sentinel
    // for the end of stream.
    if id.0 == 0 && by_id.len() > 0 {
      break;
    }

    let segment_size_words =
      read_u32(r).or_else(|_| Err(Error::Encoding(format!("incomplete segment id"))))?;
    let segment_size_bytes = segment_size_words as usize * 8;
    let mut segment_bytes = vec![0u8; segment_size_bytes];
    r.read_exact(&mut segment_bytes)
      .or_else(|_| Err(Error::Encoding(format!("insufficient segment {:?} bytes", id))))?;
    by_id.insert(id, Arc::new(segment_bytes));
  }

  let first_segment_buf =
    by_id.remove(&SegmentID(0)).ok_or_else(|| Error::Encoding(format!("no segments")))?;
  // TODO: This is pretty gross.
  let first_segment_buf = Arc::try_unwrap(first_segment_buf)
    .or_else(|_| Err(Error::Encoding(format!("internal error"))))?;
  Ok(SegmentOwned::new(first_segment_buf, by_id))
}

/// Decode the alternate Cap'n Proto segment framing used by this library.
pub fn decode_buf<'a, T: TypedStructRef<'a>>(buf: &'a [u8]) -> Result<T, Error> {
  let seg = decode_segments_buf(buf)?;
  Ok(T::from_untyped_struct(UntypedStruct::from_root(seg)?))
}

fn decode_segments_buf<'a>(buf: &'a [u8]) -> Result<SegmentBorrowed<'a>, Error> {
  let mut by_id = HashMap::new();

  let mut buf_offset = 0;
  loop {
    let id = SegmentID(u32::from_le_bytes(
      buf
        .get(buf_offset..buf_offset + 4)
        .ok_or_else(|| Error::Encoding(format!("incomplete segment id")))?
        .try_into()
        .unwrap(),
    ));
    buf_offset += 4;

    // The first segment is always id 0. A second segment id 0 is the sentinel
    // for the end of stream.
    if id.0 == 0 && by_id.len() > 0 {
      break;
    }

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

/// Encodes the alternate Cap'n Proto segment framing used by this library.
///
/// The given struct is used as the root of the first segment.
pub fn encode<'a, W: Write, T: TypedStructRef<'a>>(w: &mut W, root: &T) -> io::Result<()> {
  let (root_pointer, seg) = root.as_untyped().as_root();

  // Emit this struct's segment prefixed with a pointer to this struct.
  let root_pointer = root_pointer.encode();
  let seg_buf = seg.buf();
  let fake_len_bytes = seg_buf.len() + root_pointer.len();
  let fake_len_words = (fake_len_bytes + 7) / 8;
  let padding = vec![0; fake_len_words * 8 - fake_len_bytes];

  w.write_all(&u32::to_le_bytes(0))?;
  w.write_all(&u32::to_le_bytes(fake_len_words as u32))?;
  w.write_all(&root_pointer)?;
  w.write_all(seg_buf)?;
  w.write_all(&padding)?;

  let mut seen_segments = HashSet::new();
  for (id, segment) in seg.all_other() {
    if seen_segments.contains(&id) {
      continue;
    }
    seen_segments.insert(id);
    encode_segment(w, &mut seen_segments, id, segment)?;
  }

  // Emit a second 0 segment id to indicate the end of the stream.
  w.write_all(&u32::to_le_bytes(0))?;
  Ok(())
}

fn encode_segment<W: Write>(
  w: &mut W,
  seen_segments: &mut HashSet<SegmentID>,
  id: SegmentID,
  segment: SegmentBorrowed<'_>,
) -> io::Result<()> {
  let seg_buf = segment.buf();
  let len_bytes = seg_buf.len();
  let len_words = (len_bytes + 7) / 8;
  let padding = vec![0; len_words * 8 - len_bytes];

  {
    // A second 0 segment id is the marker for the end of the stream, so ensure
    // we didn't accidentally get one.
    debug_assert_ne!(0, id.0);
  }
  w.write_all(&u32::to_le_bytes(id.0))?;
  w.write_all(&u32::to_le_bytes(len_words as u32))?;
  w.write_all(seg_buf)?;
  w.write_all(&padding)?;
  for (id, segment) in segment.all_other() {
    if seen_segments.contains(&id) {
      continue;
    }
    seen_segments.insert(id);
    encode_segment(w, seen_segments, id, segment)?;
  }
  Ok(())
}
