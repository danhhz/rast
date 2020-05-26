// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::fmt;
use std::ops::Add;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Term(pub u64);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Index(pub u64);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct NodeID(pub u64);

impl Add<u64> for Index {
  type Output = Index;

  fn add(self, other: u64) -> Index {
    Index(self.0 + other)
  }
}

#[derive(Debug, Clone)]
pub struct WriteReq {
  pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WriteRes {
  pub term: Term,
  pub index: Index,
}

#[derive(Clone)]
pub struct ReadReq {
  pub payload: Vec<u8>,
}

impl fmt::Debug for ReadReq {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match std::str::from_utf8(&self.payload) {
      Ok(payload) => write!(f, "read:{:?}", payload),
      Err(_) => write!(f, "read:{:?}", self.payload),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReadRes {
  pub term: Term,
  pub index: Index,
  pub payload: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Entry {
  pub term: Term,
  pub index: Index,
  pub payload: Vec<u8>,
}

#[derive(Clone)]
pub struct Message {
  pub src: NodeID,
  pub dest: NodeID,
  pub payload: Payload,
}

impl fmt::Debug for Message {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "[{:?}->{:?}:{:?}]", self.src.0, self.dest.0, self.payload)
  }
}

#[derive(Debug, Clone)]
pub enum Payload {
  AppendEntriesReq(AppendEntriesReq),
  AppendEntriesRes(AppendEntriesRes),
  RequestVoteReq(RequestVoteReq),
  RequestVoteRes(RequestVoteRes),
  StartElectionReq(StartElectionReq),
}

#[derive(Debug, Clone)]
pub struct AppendEntriesReq {
  pub term: Term,
  pub leader_id: NodeID,
  pub prev_log_index: Index,
  pub prev_log_term: Term,
  pub leader_commit: Index,
  pub entries: Vec<Entry>,
}

#[derive(Debug, Clone)]
pub struct AppendEntriesRes {
  pub term: Term,
  pub success: bool,

  // NB: This is our own little extention.
  pub index: Index,
}

#[derive(Debug, Clone)]
pub struct RequestVoteReq {
  pub term: Term,
  pub candidate_id: NodeID,
  pub last_log_index: Index,
  pub last_log_term: Term,
}

#[derive(Debug, Clone)]
pub struct RequestVoteRes {
  pub term: Term,
  pub vote_granted: bool,
}

#[derive(Debug, Clone)]
pub struct StartElectionReq {
  pub term: Term,
}
