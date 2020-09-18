// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::error;
use std::fs::File;
use std::io::Read;

use crate::samples::test_capnp::TestAllTypes;

use capnp_runtime::decode_stream;

#[test]
fn decode_binary() -> Result<(), Box<dyn error::Error>> {
  let mut f = File::open("testdata/binary")?;
  let mut buf = Vec::new();
  f.read_to_end(&mut buf)?;
  let message: TestAllTypes = decode_stream::official(&buf)?;

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
  let message: TestAllTypes = decode_stream::official(&buf)?;

  let mut f = File::open("testdata/pretty.txt")?;
  let mut expected = Vec::new();
  f.read_to_end(&mut expected)?;
  let expected = String::from_utf8(expected)?;

  assert_eq!(format!("{:?}", message), expected);
  Ok(())
}
