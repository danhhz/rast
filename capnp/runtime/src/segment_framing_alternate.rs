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
use std::io::{self, Write};
use std::sync::Arc;

use crate::error::Error;
use crate::r#struct::{TypedStruct, UntypedStruct};
use crate::segment::{SegmentBorrowed, SegmentID};

/// Decode the alternate Cap'n Proto segment framing used by this library.
pub fn decode<'a, T: TypedStruct<'a>>(buf: &'a [u8]) -> Result<T, Error> {
  let seg = decode_segments(buf)?;
  Ok(T::from_untyped_struct(UntypedStruct::from_root(seg)?))
}

fn decode_segments<'a>(buf: &'a [u8]) -> Result<SegmentBorrowed<'a>, Error> {
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

/// Encodes the alternate Cap'n Proto segment framing used by this library.
///
/// The given struct is used as the root of the first segment.
pub fn encode<'a, W: Write, T: TypedStruct<'a>>(w: &mut W, root: &T) -> io::Result<()> {
  // Emit this struct's segment prefixed with a pointer to this struct.
  let (root_pointer, seg) = root.as_untyped().as_root();
  let root_pointer = root_pointer.encode();
  let seg_buf = seg.buf();
  let fake_len_bytes = seg_buf.len() + root_pointer.len();
  let fake_len_words = (fake_len_bytes + 7) / 8;
  let padding = vec![0; fake_len_words * 8 - fake_len_bytes];

  // WIP: What do we do about the segment id?
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
