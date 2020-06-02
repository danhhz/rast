// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

use super::serde::NodeID;

/// An error to be handed by the user of this Raft library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientError {
  /// See [`NotLeaderError`].
  NotLeaderError(NotLeaderError),
}

/// An error returned when a read or write was sent to a node that was not the
/// Raft leader.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotLeaderError {
  /// A hint for which node might be able to serve this request.
  pub hint: Option<NodeID>,
}

impl NotLeaderError {
  pub(crate) fn new(hint: Option<NodeID>) -> NotLeaderError {
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
