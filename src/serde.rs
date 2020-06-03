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

/// An entry in the Raft log.
#[derive(Clone)]
pub struct Entry {
  /// The term of the entry.
  pub term: Term,
  /// The index of the entry.
  pub index: Index,
  /// The opaque user payload of the entry.
  pub payload: Vec<u8>,
}

impl fmt::Debug for Entry {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match std::str::from_utf8(&self.payload) {
      Ok(payload) => write!(f, "({:}.{:} {:?})", self.term.0, self.index.0, payload),
      Err(_) => write!(f, "({:}.{:} {:?})", self.term.0, self.index.0, self.payload),
    }
  }
}

/// An rpc message.
#[derive(Clone)]
pub struct Message {
  /// The node sending this rpc.
  pub src: NodeID,
  /// The node to receive this rpc.
  pub dest: NodeID,
  /// TODO: Hide this from the external API.
  pub payload: Payload,
}

impl fmt::Debug for Message {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "[{:?}->{:?}:{:?}]", self.src.0, self.dest.0, self.payload)
  }
}

#[derive(Clone)]
pub enum Payload {
  AppendEntriesReq(AppendEntriesReq),
  AppendEntriesRes(AppendEntriesRes),
  RequestVoteReq(RequestVoteReq),
  RequestVoteRes(RequestVoteRes),
  StartElectionReq(StartElectionReq),
}

impl fmt::Debug for Payload {
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

#[derive(Clone)]
pub struct AppendEntriesReq {
  pub term: Term,
  pub leader_id: NodeID,
  pub prev_log_index: Index,
  pub prev_log_term: Term,
  pub leader_commit: Index,
  pub entries: Vec<Entry>,

  // NB: This is our own little extention.
  pub read_id: ReadID,
}

impl fmt::Debug for AppendEntriesReq {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "app({:}.{:} p{:}.{:} lc{:} r{:} {:?})",
      self.term.0,
      self.leader_id.0,
      self.prev_log_index.0,
      self.prev_log_term.0,
      self.leader_commit.0,
      self.read_id.0,
      self.entries
    )
  }
}

#[derive(Clone)]
pub struct AppendEntriesRes {
  pub term: Term,
  pub success: bool,

  // NB: These are our own little extentions.
  pub index: Index,
  pub read_id: ReadID,
}

impl fmt::Debug for AppendEntriesRes {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "appRes({:} r{:} success={:?})", self.term.0, self.read_id.0, self.success)
  }
}

#[derive(Clone)]
pub struct RequestVoteReq {
  pub term: Term,
  pub candidate_id: NodeID,
  pub last_log_index: Index,
  pub last_log_term: Term,
}

impl fmt::Debug for RequestVoteReq {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "vote({:} p{:}.{:} candidate={:})",
      self.term.0, self.last_log_index.0, self.last_log_term.0, self.candidate_id.0,
    )
  }
}

#[derive(Clone)]
pub struct RequestVoteRes {
  pub term: Term,
  pub vote_granted: bool,
}

impl fmt::Debug for RequestVoteRes {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "voteRes({:} granted={:?})", self.term.0, self.vote_granted)
  }
}

#[derive(Debug, Clone)]
pub struct StartElectionReq {
  pub term: Term,
}
