// Copyright 2020 Daniel Harrison. All Rights Reserved.

use super::log::NodeID;
use std::error::Error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotLeaderError {
  hint: NodeID,
}

impl NotLeaderError {
  pub fn new(hint: NodeID) -> NotLeaderError {
    NotLeaderError { hint: hint }
  }
}

impl Display for NotLeaderError {
  fn fmt(&self, f: &mut Formatter) -> Result {
    write!(f, "not leader, hint: {:?}", self.hint)
  }
}

impl Error for NotLeaderError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    None
  }
}
