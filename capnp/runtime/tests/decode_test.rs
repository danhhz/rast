// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::convert::TryInto;
use std::error;
use std::fs::File;
use std::io::Read;

mod samples;
use samples::test_capnp::TestAllTypes;

use capnp_runtime::prelude::*;

#[test]
fn decode_binary() -> Result<(), Box<dyn error::Error>> {
  let mut f = File::open("testdata/binary")?;
  let mut buf = Vec::new();
  f.read_to_end(&mut buf)?;
  let seg = decode_stream_official(&buf)?;
  let message = TestAllTypes::from_untyped_struct(SegmentPointer::from_root(seg).try_into()?);

  let mut f = File::open("testdata/pretty.txt")?;
  let mut expected = Vec::new();
  f.read_to_end(&mut expected)?;
  let expected = String::from_utf8(expected)?;

  assert_eq!(format!("{:?}", message), expected);
  Ok(())
}

#[test]
fn decode_segmented() -> Result<(), Box<dyn error::Error>> {
  let mut f = File::open("testdata/segmented")?;
  let mut buf = Vec::new();
  f.read_to_end(&mut buf)?;
  let seg = decode_stream_official(&buf)?;
  let message = TestAllTypes::from_untyped_struct(SegmentPointer::from_root(seg).try_into()?);

  let mut f = File::open("testdata/pretty.txt")?;
  let mut expected = Vec::new();
  f.read_to_end(&mut expected)?;
  let expected = String::from_utf8(expected)?;

  assert_eq!(format!("{:?}", message), expected);
  Ok(())
}
