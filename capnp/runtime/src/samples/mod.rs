// Copyright 2020 Daniel Harrison. All Rights Reserved.

#![allow(dead_code)]

pub mod rast_capnp {
  #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
  pub struct Term(pub u64);

  #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
  pub struct Index(pub u64);

  #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
  pub struct NodeID(pub u64);

  #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
  pub struct ReadID(pub u64);

  include!("rast_capnp.rs");
}
pub mod carsales_capnp;
pub mod catrank_capnp;
pub mod eval_capnp;
pub mod test_capnp;
