// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

use super::serde::NodeID;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientError {
  NotLeaderError(NotLeaderError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotLeaderError {
  hint: Option<NodeID>,
}

impl NotLeaderError {
  pub fn new(hint: Option<NodeID>) -> NotLeaderError {
    NotLeaderError { hint: hint }
  }
}

impl Display for NotLeaderError {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self.hint {
      Some(hint) => write!(f, "not leader, hint: {:?}", hint),
      None => write!(f, "not leader, hint: none"),
    }
  }
}

impl Error for NotLeaderError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    None
  }
}
