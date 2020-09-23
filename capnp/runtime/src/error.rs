// Copyright 2020 Daniel Harrison. All Rights Reserved.

//! Errors returned from Cap'n Proto encoding and decoding.

use std::cmp::{self, Ordering};
use std::error;
use std::fmt;

use crate::common::Discriminant;

/// An error returned from Cap'n Proto encoding or decoding.
#[derive(Debug, Clone)]
pub enum Error {
  /// An error in the encoded bytes being intrepreted
  Encoding(String),
  /// An unimplemented feature in this library
  TODO(String),
  /// An incorrect usage of this library's APIs
  Usage(String),
}

impl error::Error for Error {}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Error::Encoding(x) => {
        f.write_str("encoding: ")?;
        std::fmt::Display::fmt(x, f)
      }
      Error::TODO(x) => {
        f.write_str("unimplemented: ")?;
        std::fmt::Display::fmt(x, f)
      }
      Error::Usage(x) => {
        f.write_str("usage: ")?;
        std::fmt::Display::fmt(x, f)
      }
    }
  }
}

impl<'a> cmp::PartialEq for Error {
  fn eq(&self, other: &Error) -> bool {
    self.partial_cmp(other) == Some(Ordering::Equal)
  }
}

impl<'a> cmp::PartialOrd for Error {
  fn partial_cmp(&self, _other: &Error) -> Option<Ordering> {
    None
  }
}

/// A placeholder for an unknown discriminant
///
/// This process received a union or enum encoded by process with a future
/// version of the schema.
#[derive(Debug, Clone)]
pub struct UnknownDiscriminant(pub Discriminant, pub &'static str);

impl error::Error for UnknownDiscriminant {}

impl fmt::Display for UnknownDiscriminant {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:?} from future schema for {}", self.0, self.1)
  }
}
