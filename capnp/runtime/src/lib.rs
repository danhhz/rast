// Copyright 2020 Daniel Harrison. All Rights Reserved.

#![allow(dead_code, unused_variables)]
// WIP
#![allow(unreachable_code)]

mod common;
mod decode;
mod encode;
mod error;
mod list;
mod pointer;
mod reflect;
mod segment;
mod segment_pointer;
mod untyped;

pub mod prelude {
  pub use crate::common::*;
  pub use crate::error::*;
  pub use crate::list::ListMeta;
  pub use crate::reflect::*;
  pub use crate::segment::*;
  pub use crate::segment_pointer::*;
  pub use crate::untyped::*;
}

#[cfg(feature = "serde")]
pub mod serde;

#[cfg(feature = "rand")]
pub mod rand;
