// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::cmp::Ordering;
use std::convert::TryInto;
use std::error;
use std::fs::File;
use std::io::Read;

mod samples;
use samples::test_capnp::TestAllTypes;

use capnp_runtime::prelude::*;

#[test]
fn cmp_equal() -> Result<(), Box<dyn error::Error>> {
  let mut f = File::open("testdata/binary")?;
  let mut buf = Vec::new();
  f.read_to_end(&mut buf)?;
  let seg = decode_stream_official(&buf)?;
  let binary = TestAllTypes::from_untyped_struct(SegmentPointer::from_root(seg).try_into()?);

  let mut f = File::open("testdata/segmented")?;
  let mut buf = Vec::new();
  f.read_to_end(&mut buf)?;
  let seg = decode_stream_official(&buf)?;
  let segmented = TestAllTypes::from_untyped_struct(SegmentPointer::from_root(seg).try_into()?);

  assert_eq!(binary.partial_cmp(&segmented), Some(Ordering::Equal));
  assert_eq!(binary == segmented, true);
  assert_eq!(binary, segmented);
  Ok(())
}
