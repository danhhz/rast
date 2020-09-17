// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::fmt;
use std::ops::Add;

/// A Raft term.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Term(pub u64);

/// A Raft index.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Index(pub u64);

/// A unique identifier for a node in a raft group.
///
/// Nodes must restart with the same ID, unless they lose data, in which case
/// they need to be started from scratch with a new ID.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct NodeID(pub u64);

/// An internal identifier for tracking the allowability of a read request.
///
/// TODO: Make this more general.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReadID(pub u64);

impl Add<u64> for Index {
  type Output = Index;

  fn add(self, other: u64) -> Index {
    Index(self.0 + other)
  }
}

/// See [`Input::Write`](crate::Input::Write).
#[derive(Clone)]
pub struct WriteReq {
  /// An opaque payload handed to the state machine.
  pub payload: Vec<u8>,
}

impl fmt::Debug for WriteReq {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match std::str::from_utf8(&self.payload) {
      Ok(payload) => write!(f, "w{:?}", payload),
      Err(_) => write!(f, "w{:?}", self.payload),
    }
  }
}

impl From<String> for WriteReq {
  fn from(payload: String) -> Self {
    WriteReq { payload: payload.into_bytes() }
  }
}

/// See [`Input::Write`](crate::Input::Write).
#[derive(Debug, Clone, PartialEq)]
pub struct WriteRes {
  /// The term at which the write happened.
  pub term: Term,
  /// The index at which the write happened.
  pub index: Index,
}

/// See [`Input::Read`](crate::Input::Read).
#[derive(Clone)]
pub struct ReadReq {
  /// The read payload to be handed to the replicated state machine.
  ///
  /// For example: This could be a key when the replicated state machine is a
  /// key-value store.
  pub payload: Vec<u8>,
}

impl From<String> for ReadReq {
  fn from(payload: String) -> Self {
    ReadReq { payload: payload.into_bytes() }
  }
}

impl fmt::Debug for ReadReq {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match std::str::from_utf8(&self.payload) {
      Ok(payload) => write!(f, "r{:?}", payload),
      Err(_) => write!(f, "r{:?}", self.payload),
    }
  }
}

/// See [`Input::Read`](crate::Input::Read).
#[derive(Debug, Clone, PartialEq)]
pub struct ReadRes {
  /// The term at which the read happened.
  pub term: Term,
  /// The index at which the read happened.
  pub index: Index,
  /// The result of reading the state machine with the request's payload.
  ///
  /// For example: This could be a value when the replicated state machine is a
  /// key-value store.
  pub payload: Vec<u8>,
}

#[allow(missing_docs)]
mod generated {
  use std::fmt;

  include!("../capnp/runtime/src/samples/rast_capnp.rs");

  impl<'a> fmt::Display for Entry<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      match std::str::from_utf8(&self.payload().expect("WIP")) {
        Ok(payload) => write!(f, "({:}.{:} {:?})", self.term(), self.index(), payload),
        Err(_) => write!(f, "({:}.{:} {:?})", self.term(), self.index(), self.payload()),
      }
    }
  }

  impl fmt::Debug for EntryShared {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      self.capnp_as_ref().fmt(f)
    }
  }

  impl fmt::Display for Message<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      write!(f, "[{:?}->{:?}:{:?}]", self.src(), self.dest(), self.payload().expect("WIP"))
    }
  }

  impl fmt::Debug for MessageShared {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      self.capnp_as_ref().fmt(f)
    }
  }

  impl fmt::Debug for Payload<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      match self {
        Payload::AppendEntriesReq(r) => r.fmt(f),
        Payload::AppendEntriesRes(r) => r.fmt(f),
        Payload::RequestVoteReq(r) => r.fmt(f),
        Payload::RequestVoteRes(r) => r.fmt(f),
        Payload::StartElectionReq(r) => r.fmt(f),
      }
    }
  }

  impl fmt::Debug for PayloadShared {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      self.capnp_as_ref().fmt(f)
    }
  }

  impl fmt::Display for AppendEntriesReq<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      write!(
        f,
        "app({:}.{:} p{:}.{:} lc{:} r{:} {:?})",
        self.term(),
        self.leader_id(),
        self.prev_log_index(),
        self.prev_log_term(),
        self.leader_commit(),
        self.read_id(),
        self.entries(),
      )
    }
  }

  impl fmt::Debug for AppendEntriesReqShared {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      self.capnp_as_ref().fmt(f)
    }
  }

  impl fmt::Display for AppendEntriesRes<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      write!(f, "appRes({:} r{:} success={:?})", self.term(), self.read_id(), self.success())
    }
  }

  impl fmt::Debug for AppendEntriesResShared {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      self.capnp_as_ref().fmt(f)
    }
  }

  impl fmt::Display for RequestVoteReq<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      write!(
        f,
        "vote({:} p{:}.{:} candidate={:})",
        self.term(),
        self.last_log_index(),
        self.last_log_term(),
        self.candidate_id(),
      )
    }
  }

  impl fmt::Debug for RequestVoteReqShared {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      self.capnp_as_ref().fmt(f)
    }
  }

  impl fmt::Display for RequestVoteRes<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      write!(f, "voteRes({:} granted={:?})", self.term(), self.vote_granted())
    }
  }

  impl fmt::Debug for RequestVoteResShared {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      self.capnp_as_ref().fmt(f)
    }
  }

  impl fmt::Display for StartElectionReq<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      write!(f, "election({:})", self.term())
    }
  }

  impl fmt::Debug for StartElectionReqShared {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      self.capnp_as_ref().fmt(f)
    }
  }
}
pub use generated::*;
