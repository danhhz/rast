// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::error;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Error {
  Str(String),
  Wrapped(Rc<dyn error::Error>),
}

impl error::Error for Error {}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Error::Str(x) => std::fmt::Debug::fmt(x, f),
      Error::Wrapped(x) => std::fmt::Debug::fmt(x, f),
    }
  }
}

impl From<&'_ str> for Error {
  fn from(x: &'_ str) -> Self {
    // TODO: Is it possible to do with without the copy?
    Error::Str(x.to_string())
  }
}

impl From<String> for Error {
  fn from(x: String) -> Self {
    Error::Str(x)
  }
}

impl From<std::array::TryFromSliceError> for Error {
  fn from(x: std::array::TryFromSliceError) -> Self {
    Error::Wrapped(Rc::new(x))
  }
}
