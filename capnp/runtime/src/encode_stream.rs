// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::collections::HashSet;
use std::io::{self, Write};

use crate::r#struct::TypedStruct;
use crate::segment::{SegmentBorrowed, SegmentID};

pub fn alternate<'a, W: Write, T: TypedStruct<'a>>(w: &mut W, root: &T) -> io::Result<()> {
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
    segment_alternate(w, &mut seen_segments, id, segment)?;
  }
  Ok(())
}

fn segment_alternate<W: Write>(
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
    segment_alternate(w, seen_segments, id, segment)?;
  }
  Ok(())
}
