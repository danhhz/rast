// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::error;

use crate::samples::test_capnp::TestAllTypes;

use capnp_runtime::packed;
use capnp_runtime::segment_framing_official;

#[test]
fn decode_binary() -> Result<(), Box<dyn error::Error>> {
  let buf = include_bytes!("../testdata/binary");
  let message: TestAllTypes = segment_framing_official::decode(buf)?;
  let expected = include_str!("../testdata/short.txt");
  assert_eq!(format!("{:?}", message), expected);
  Ok(())
}

#[test]
fn decode_segmented() -> Result<(), Box<dyn error::Error>> {
  let buf = include_bytes!("../testdata/segmented");
  let message: TestAllTypes = segment_framing_official::decode(buf)?;
  let expected = include_str!("../testdata/short.txt");
  assert_eq!(format!("{:?}", message), expected);
  Ok(())
}

#[test]
fn decode_packed() -> Result<(), Box<dyn error::Error>> {
  let buf = include_bytes!("../testdata/packed");
  let buf = packed::decode(&mut buf.as_ref())?;
  let message: TestAllTypes = segment_framing_official::decode(&buf)?;
  let expected = include_str!("../testdata/short.txt");
  assert_eq!(format!("{:?}", message), expected);
  Ok(())
}
