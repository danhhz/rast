// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::io;

use capnp_runtime::prelude::Error as CapnpError;

#[derive(Debug)]
pub enum Error {
  Capnp(CapnpError),
  Failed(String),
}

impl Error {
  pub fn failed(s: String) -> Self {
    Error::Failed(s)
  }
}

impl From<CapnpError> for Error {
  fn from(x: CapnpError) -> Error {
    Error::Capnp(x)
  }
}

impl From<io::Error> for Error {
  fn from(x: io::Error) -> Error {
    Error::Capnp(x.into())
  }
}
