// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::convert::TryInto;

#[derive(Debug,Copy,Clone,PartialEq,Eq,PartialOrd,Ord)]
pub struct Term(pub u64);

#[derive(Debug,Copy,Clone,PartialEq,Eq,PartialOrd,Ord)]
pub struct Index(pub u64);

#[derive(Debug)]
pub struct Group(pub u64);

#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub struct Node(pub u64);

#[derive(Debug)]
pub struct Entry {
  // TODO: &[u8]
  buf: Vec<u8>
}

impl Entry {
  pub fn term(&self) -> Term {
    Term(u64::from_le_bytes(self.buf[0..8].try_into().unwrap()))
  }
  pub fn index(&self) -> Index {
    Index(u64::from_le_bytes(self.buf[8..16].try_into().unwrap()))
  }
  pub fn payload(&self) -> &[u8] {
    &self.buf[16..]
  }
}

pub struct Message {
  // TODO: &[u8]
  buf: Vec<u8>
}

const APPEND_ENTRIES_REQ: u8 = 0;
const APPEND_ENTRIES_RES: u8 = 1;
const REQUEST_VOTE_REQ: u8 = 2;
const REQUEST_VOTE_RES: u8 = 3;

impl Message {
  pub fn append_entries_res(_term: Term, _success: bool) -> Message {
    Message{buf: vec![]}  // WIP
  }

  pub fn request_vote_req(_term: Term, _candidate_id: Node, _last_log_index: Index, _last_log_term: Term) -> Message {
    Message{buf: vec![]}  // WIP
  }

  pub fn request_vote_res(_term: Term, _vote_granted: bool) -> Message {
    Message{buf: vec![]}  // WIP
  }

  pub fn payload(&self) -> Payload {
    let payload_buf = &self.buf[1..];
    match self.buf[0] {
      APPEND_ENTRIES_REQ => Payload::AppendEntriesReq(AppendEntriesReq::decode(payload_buf)),
      APPEND_ENTRIES_RES => Payload::AppendEntriesRes(AppendEntriesRes::decode(payload_buf)),
      REQUEST_VOTE_REQ => Payload::RequestVoteReq(RequestVoteReq::decode(payload_buf)),
      REQUEST_VOTE_RES => Payload::RequestVoteRes(RequestVoteRes::decode(payload_buf)),
      _ => panic!(),
    }
  }
}

pub enum Payload {
  AppendEntriesReq(AppendEntriesReq),
  AppendEntriesRes(AppendEntriesRes),
  RequestVoteReq(RequestVoteReq),
  RequestVoteRes(RequestVoteRes),
}

#[derive(Debug)]
pub struct AppendEntriesReq {
  pub term: Term,
  pub leader_id: Node,
  pub prev_log_index: Index,
  pub prev_log_term: Term,
  pub leader_commit: Index,
  pub entries: Vec<Entry>,
}

impl AppendEntriesReq {
  fn decode(buf: &[u8]) -> AppendEntriesReq {
    let term_buf: [u8; 8] = buf[0..8].try_into().unwrap();
    let leader_id_buf: [u8; 8] = buf[8..16].try_into().unwrap();
    let prev_log_index_buf: [u8; 8] = buf[16..24].try_into().unwrap();
    let prev_log_term_buf: [u8; 8] = buf[24..32].try_into().unwrap();
    let leader_commit_buf: [u8; 8] = buf[32..40].try_into().unwrap();

    AppendEntriesReq{
      term: Term(u64::from_le_bytes(term_buf)),
      leader_id: Node(u64::from_le_bytes(leader_id_buf)),
      prev_log_index: Index(u64::from_le_bytes(prev_log_index_buf)),
      prev_log_term: Term(u64::from_le_bytes(prev_log_term_buf)),
      leader_commit: Index(u64::from_le_bytes(leader_commit_buf)),
      entries: vec![],
    }
  }
}

#[derive(Debug)]
pub struct AppendEntriesRes {
  pub term: Term,
  pub success: bool,
}

impl AppendEntriesRes {
  fn decode(buf: &[u8]) -> AppendEntriesRes {
    let term_buf: [u8; 8] = buf[0..8].try_into().unwrap();
    let success_buf: [u8; 8] = buf[8..16].try_into().unwrap();

    AppendEntriesRes{
      term: Term(u64::from_le_bytes(term_buf)),
      success: u64::from_le_bytes(success_buf) > 0,
    }
  }
}

#[derive(Debug)]
pub struct RequestVoteReq {
  pub term: Term,
  pub candidate_id: Node,
  pub last_log_index: Index,
  pub last_log_term: Term,
}

impl RequestVoteReq {
  fn decode(buf: &[u8]) -> RequestVoteReq {
    let term_buf: [u8; 8] = buf[0..8].try_into().unwrap();
    let candidate_id_buf: [u8; 8] = buf[8..16].try_into().unwrap();
    let last_log_index_buf: [u8; 8] = buf[16..32].try_into().unwrap();
    let last_log_term_buf: [u8; 8] = buf[32..40].try_into().unwrap();
    RequestVoteReq{
      term: Term(u64::from_le_bytes(term_buf)),
      candidate_id: Node(u64::from_le_bytes(candidate_id_buf)),
      last_log_index: Index(u64::from_le_bytes(last_log_index_buf)),
      last_log_term: Term(u64::from_le_bytes(last_log_term_buf)),
    }
  }
}

#[derive(Debug)]
pub struct RequestVoteRes {
  pub term: Term,
  pub vote_granted: bool,
}

impl RequestVoteRes {
  fn decode(buf: &[u8]) -> RequestVoteRes {
    let term_buf: [u8; 8] = buf[0..8].try_into().unwrap();
    let vote_granted_buf: [u8; 8] = buf[8..16].try_into().unwrap();

    RequestVoteRes{
      term: Term(u64::from_le_bytes(term_buf)),
      vote_granted: u64::from_le_bytes(vote_granted_buf) > 0,
    }
  }
}
