// Copyright 2020 Daniel Harrison. All Rights Reserved.

#![allow(dead_code, unused_variables)]

mod common;
mod error;
pub mod fmt_debug;
mod pointer;
mod reflect;
mod segment;
mod segment_pointer;
mod untyped;

pub mod prelude {
  pub use crate::common::*;
  pub use crate::error::*;
  pub use crate::reflect::*;
  pub use crate::segment::*;
  pub use crate::segment_pointer::*;
  pub use crate::untyped::*;
}

#[cfg(feature = "serde")]
pub mod serde;
