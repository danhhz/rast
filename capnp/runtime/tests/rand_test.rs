// Copyright 2020 Daniel Harrison. All Rights Reserved.

mod samples;

#[cfg(feature = "rand")]
mod test {
  use std::convert::TryInto;
  use std::error::Error;

  use super::samples::rast_capnp::{AppendEntriesReq, AppendEntriesReqShared};
  use super::samples::test_capnp::{TestAllTypes, TestAllTypesShared};
  use rand;

  use capnp_runtime::prelude::*;

  #[test]
  fn rand_roundtrip_testalltypes() -> Result<(), Box<dyn Error>> {
    let before: TestAllTypesShared = capnp_runtime::rand::gen_typed(&mut rand::thread_rng());
    let mut buf = Vec::new();
    before.as_ref().as_untyped().encode_as_root_alternate(&mut buf)?;
    // println!("{:?}", &buf);
    let seg = decode_stream_alternate(&buf)?;
    let after = TestAllTypes::from_untyped_struct(SegmentPointer::from_root(seg).try_into()?);

    println!("before");
    println!("{:?}", before.as_ref());
    println!("after");
    println!("{:?}", after);
    assert_eq!(before.as_ref(), after);
    Ok(())
  }

  #[test]
  fn rand_roundtrip_appendentriesreq() -> Result<(), Box<dyn Error>> {
    let before: AppendEntriesReqShared = capnp_runtime::rand::gen_typed(&mut rand::thread_rng());
    let mut buf = Vec::new();
    before.as_ref().as_untyped().encode_as_root_alternate(&mut buf)?;
    println!("{:?}", &buf);
    let seg = decode_stream_alternate(&buf)?;
    let after = AppendEntriesReq::from_untyped_struct(SegmentPointer::from_root(seg).try_into()?);

    println!("before");
    println!("{:?}", before.as_ref());
    println!("after");
    println!("{:?}", after);
    assert_eq!(before.as_ref(), after);
    Ok(())
  }
}
