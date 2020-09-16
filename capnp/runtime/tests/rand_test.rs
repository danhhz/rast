// Copyright 2020 Daniel Harrison. All Rights Reserved.

mod samples;

#[cfg(feature = "rand")]
mod test {
  use std::convert::TryInto;
  use std::error::Error;

  use super::samples::rast_capnp::{Message, MessageShared};
  use super::samples::test_capnp::{TestAllTypes, TestAllTypesShared};
  use rand;

  use capnp_runtime::prelude::*;

  #[test]
  fn rand_roundtrip_testalltypes() -> Result<(), Box<dyn Error>> {
    let before: TestAllTypesShared =
      capnp_runtime::rand::Rand::new(&mut rand::thread_rng(), 20).gen_typed_struct();
    let mut buf = Vec::new();
    before.capnp_as_ref().as_untyped().encode_as_root_alternate(&mut buf)?;
    let seg = decode_stream_alternate(&buf)?;
    let after = TestAllTypes::from_untyped_struct(SegmentPointer::from_root(seg).try_into()?);
    assert_eq!(before.capnp_as_ref(), after);
    Ok(())
  }

  #[test]
  fn rand_roundtrip_rast() -> Result<(), Box<dyn Error>> {
    let before: MessageShared =
      capnp_runtime::rand::Rand::new(&mut rand::thread_rng(), 20).gen_typed_struct();
    let mut buf = Vec::new();
    before.capnp_as_ref().as_untyped().encode_as_root_alternate(&mut buf)?;
    let seg = decode_stream_alternate(&buf)?;
    let after = Message::from_untyped_struct(SegmentPointer::from_root(seg).try_into()?);
    assert_eq!(before.capnp_as_ref(), after);
    Ok(())
  }
}
