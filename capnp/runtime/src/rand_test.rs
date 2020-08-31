// Copyright 2020 Daniel Harrison. All Rights Reserved.

use rand;
use std::error::Error;

use crate::runtime::{self, Decode, NumElements};
use crate::samples::test::{TestAllTypes, TestAllTypesOwned};

#[test]
fn rand_roundtrip() -> Result<(), Box<dyn Error>> {
  let before: TestAllTypesOwned = rand::random();
  let buf = before.as_ref().serialize();
  println!("{:?}", &buf);
  let seg = capnp::decode_segment(&buf)?;
  let after = TestAllTypes::decode(&seg.root(), NumElements(0))?;

  println!("before");
  println!("{:?}", before.as_ref());
  println!("after");
  println!("{:?}", after);
  assert_eq!(before.as_ref(), after);
  Ok(())
}
