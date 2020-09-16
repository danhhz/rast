// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::error;
use std::fmt;

use crate::common::Discriminant;

#[derive(Debug, Clone)]
pub enum Error {
  Encoding(String),
  TODO(String),
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

#[derive(Debug, Clone)]
pub struct UnknownDiscriminant(pub Discriminant, pub &'static str);

impl error::Error for UnknownDiscriminant {}

impl fmt::Display for UnknownDiscriminant {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:?} from future schema for {}", self.0, self.1)
  }
}
