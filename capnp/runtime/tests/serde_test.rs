// Copyright 2020 Daniel Harrison. All Rights Reserved.

mod samples;

#[cfg(feature = "serde")]
mod test {
  use std::convert::TryInto;
  use std::error;
  use std::fs::File;
  use std::io::Read;

  use serde_json;

  use super::samples::test::TestAllTypes;

  use capnp_runtime::prelude::*;

  #[test]
  fn serialize_json() -> Result<(), Box<dyn error::Error>> {
    let mut f = File::open("testdata/binary")?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    let seg = decode_segment(&buf)?;
    let message = TestAllTypes::from_untyped_struct(SegmentPointer::from_root(seg).try_into()?);

    let mut f = File::open("testdata/short.json")?;
    let mut expected = Vec::new();
    f.read_to_end(&mut expected)?;
    let expected = String::from_utf8(expected)?;

    let actual =
      serde_json::ser::to_string(&PointerElement::Struct(message.meta(), message.to_untyped()))?;
    assert_eq!(actual, expected);
    Ok(())
  }
}
