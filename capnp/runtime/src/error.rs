// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Error(pub &'static str);

impl error::Error for Error {}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    std::fmt::Debug::fmt(&self, f)
  }
}

impl From<std::array::TryFromSliceError> for Error {
  fn from(x: std::array::TryFromSliceError) -> Self {
    Error("could not convert slice to array")
  }
}
