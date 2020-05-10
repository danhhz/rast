// Copyright 2020 Daniel Harrison. All Rights Reserved.

#![warn(clippy::correctness, clippy::perf)]

use std::cmp;
use std::collections::HashMap;
use std::time::{Duration, Instant};

mod future;
mod log;

pub use future::*;
pub use log::*;

#[derive(Debug)]
pub enum Input {
  Write((WriteReq, WriteFuture)),
  Tick(Instant),
  Message(Message),
  PersistRes(Index, NodeID),
}

#[derive(Debug)]
pub enum Output {
  Message(Message),
  PersistReq(Index, NodeID),
  ApplyReq(Index),
}

// WIP: keep the role-specific state in this enum
#[derive(Debug, PartialEq, Eq)]
enum Role {
  Candidate,
  Follower,
  Leader,
}

#[derive(Clone)]
pub struct Config {
  pub election_timeout: Duration,
  pub heartbeat_interval: Duration,
}

// TODO: rename (to state machine?)
pub struct Rast {
  config: Config,
  id: NodeID,
  role: Role,

  // Persistent state
  current_term: Term,
  voted_for: Option<NodeID>,
  log: Vec<Entry>,

  // Volatile state
  commit_index: Index,
  last_applied: Index,
  nodes: Vec<NodeID>, // WIP: double check this doesn't need to be persisted
  current_time: Instant,
  // WIP this is overloaded fixme
  last_communication: Instant,

  // Leader volatile state
  next_index: HashMap<NodeID, Index>,
  match_index: HashMap<NodeID, Index>,
  command_buffer: HashMap<(Term, Index), WriteFuture>,

  // Candidate volatile state
  received_votes: usize,
}

impl Rast {
  pub fn new(id: NodeID, nodes: Vec<NodeID>, config: Config, current_time: Instant) -> Rast {
    // WIP: hack to initialize last_communication such that an election is
    // immediately called
    let last_communication = current_time - config.election_timeout;
    Rast {
      id: id,
      config: config,
      role: Role::Candidate,
      current_term: Term(0),
      voted_for: None,
      log: vec![],
      commit_index: Index(0),
      last_applied: Index(0),
      nodes: nodes,
      current_time: current_time,
      last_communication: last_communication,
      next_index: HashMap::new(),
      match_index: HashMap::new(),
      received_votes: 0,
      command_buffer: HashMap::new(),
    }
  }

  pub fn write_async(&mut self, req: WriteReq) -> (WriteFuture, Vec<Output>) {
    let future = WriteFuture::new();
    let output = self.step(Input::Write((req, future.clone())));
    (future, output)
  }

  pub fn step(&mut self, input: Input) -> Vec<Output> {
    let mut output = vec![];
    // All Servers: If commitIndex > lastApplied: increment lastApplied, apply
    // log[lastApplied] to state machine (§5.3)
    output.extend(self.maybe_apply());
    output.extend(match input {
      Input::Write((req, state)) => self.write(req.payload, Some(state)),
      Input::Tick(now) => self.tick(now),
      Input::PersistRes(index, leader_id) => self.persist_res(index, leader_id),
      Input::Message(message) => self.message(message),
    });
    output
  }

  fn maybe_wake(&mut self) {
    let current_term = self.current_term;
    let commit_index = self.commit_index;
    self.command_buffer.retain(|(term, index), future| {
      debug_assert!(*term == current_term);
      if *index >= commit_index {
        future.fill(*term, *index);
        false
      } else {
        true
      }
    });
  }

  fn maybe_apply(&mut self) -> Vec<Output> {
    if self.commit_index > self.last_applied {
      let Index(last_applied) = self.last_applied;
      self.last_applied = Index(last_applied + 1);
      vec![Output::ApplyReq(self.last_applied)]
    } else {
      vec![]
    }
  }

  fn write(&mut self, payload: Vec<u8>, state: Option<WriteFuture>) -> Vec<Output> {
    let (prev_log_term, prev_log_index) = match self.role {
      Role::Leader => {
        self.log.last().map_or((Term(0), Index(0)), |entry| (entry.term, entry.index))
      }
      Role::Candidate => todo!(),
      Role::Follower => todo!(),
    };
    let entry = Entry { term: self.current_term, index: prev_log_index + 1, payload: payload };
    // WIP debug assertion that this doesn't exist.
    if let Some(state) = state {
      self.command_buffer.insert((entry.term, entry.index), state);
    }
    // WIP: is this really the right place for this?
    self.log.extend(vec![entry.clone()]);
    let mut output = self.ack_term_index(self.id, entry.term, entry.index);
    let payload = Payload::AppendEntriesReq(AppendEntriesReq {
      term: self.current_term,
      leader_id: self.id,
      prev_log_index: prev_log_index,
      prev_log_term: prev_log_term,
      leader_commit: self.commit_index,
      entries: vec![entry],
    });
    output.extend(self.message_to_all_other_nodes(payload));
    output
  }

  fn tick(&mut self, now: Instant) -> Vec<Output> {
    if now <= self.current_time {
      // Ignore a repeat tick (as well as one in the past, which shouldn't
      // happen).
      return vec![];
    }
    self.current_time = now;
    match self.role {
      Role::Candidate => {
        // Candidates (§5.2): If election timeout elapses: start new election
        if now.duration_since(self.last_communication) > self.config.election_timeout {
          return self.start_election(now);
        }
        return vec![];
      }
      Role::Follower => {
        // Followers (§5.2): If election timeout elapses without receiving
        // AppendEntries RPC from current leader or granting vote to candidate:
        // convert to candidate
        if now.duration_since(self.last_communication) > self.config.election_timeout {
          return self.convert_to_candidate(now);
        }
        return vec![];
      }
      Role::Leader => {
        if now.duration_since(self.last_communication) > self.config.heartbeat_interval {
          // Leaders: Upon election: send initial empty AppendEntries RPCs
          // (heartbeat) to each server; repeat during idle periods to prevent
          // election timeouts (§5.2)
          return self.send_append_entries(vec![]);
        }
        return vec![];
      }
    }
  }

  fn persist_res(&mut self, index: Index, leader_id: NodeID) -> Vec<Output> {
    let payload = Payload::AppendEntriesRes(AppendEntriesRes {
      term: self.current_term,
      index: index,
      success: true,
    });
    vec![Output::Message(Message { src: self.id, dest: leader_id, payload: payload })]
  }

  fn message(&mut self, message: Message) -> Vec<Output> {
    // TODO: avoid calling payload multiple times
    let term = match &message.payload {
      Payload::AppendEntriesReq(req) => req.term,
      Payload::RequestVoteReq(req) => req.term,
      Payload::AppendEntriesRes(res) => res.term,
      Payload::RequestVoteRes(res) => res.term,
    };
    let mut output = vec![];
    if term > self.current_term {
      // All Servers: If RPC request or response contains term T > currentTerm:
      // set currentTerm = T, convert to follower (§5.1)
      self.current_term = term;
      output.extend(self.convert_to_follower());
    }
    output.extend(match &mut self.role {
      Role::Candidate => self.step_candidate(message),
      Role::Follower => self.step_follower(message),
      Role::Leader => self.step_leader(message),
    });
    output
  }

  fn step_candidate(&mut self, message: Message) -> Vec<Output> {
    match &message.payload {
      Payload::RequestVoteRes(res) => self.process_request_vote_res(&res),
      Payload::AppendEntriesReq(req) => {
        if req.term > self.current_term {
          // Candidates (§5.2): If AppendEntries RPC received from new leader:
          // convert to follower
          let mut output = self.convert_to_follower();
          // WIP: this is awkward
          output.extend(self.step(Input::Message(message)));
          return output;
        }
        return vec![];
      }
      Payload::RequestVoteReq(req) => self.process_request_vote(req),
      _ => vec![], // WIP no-op
    }
  }

  fn step_follower(&mut self, message: Message) -> Vec<Output> {
    match message.payload {
      // Followers (§5.2): Respond to RPCs from candidates and leaders
      Payload::AppendEntriesReq(req) => self.process_append_entries(req),
      Payload::RequestVoteReq(req) => self.process_request_vote(&req),
      _ => vec![], // WIP no-op
    }
  }

  fn step_leader(&mut self, message: Message) -> Vec<Output> {
    match message.payload {
      Payload::AppendEntriesRes(res) => self.process_append_entries_res(message.src, res),
      Payload::RequestVoteRes(res) => self.process_request_vote_res(&res),
      _ => todo!("{:?}", message.payload),
    }
  }

  fn process_append_entries(&mut self, req: AppendEntriesReq) -> Vec<Output> {
    // Reply false if term < currentTerm (§5.1)
    if req.term < self.current_term {
      let payload = Payload::AppendEntriesRes(AppendEntriesRes {
        term: self.current_term,
        index: Index(0),
        success: false,
      });
      let mut output =
        vec![Output::Message(Message { src: self.id, dest: req.leader_id, payload: payload })];
      output.extend(self.convert_to_follower());
      return output;
    }

    // WIP: Reply false if log doesn’t contain an entry at prevLogIndex whose
    // term matches prevLogTerm (§5.3)

    // WIP: If an existing entry conflicts with a new one (same index but
    // different terms), delete the existing entry and all that follow it (§5.3)

    // WIP: Append any new entries not already in the log
    self.log.extend(req.entries);
    let new_index = self.log.last().map_or(Index(0), |entry| entry.index);
    let mut output = vec![Output::PersistReq(new_index, req.leader_id)];

    // If leaderCommit > commitIndex, set commitIndex = min(leaderCommit, index
    // of last new entry)
    if req.leader_commit > self.commit_index {
      let last_entry_index = self.log.last().map_or(Index(0), |entry| entry.index);
      self.commit_index = cmp::min(req.leader_commit, last_entry_index);
      output.extend(self.maybe_apply());
    }

    output
  }

  fn process_append_entries_res(&mut self, src: NodeID, res: AppendEntriesRes) -> Vec<Output> {
    // If successful: update nextIndex and matchIndex for follower (§5.3)
    if res.success {
      return self.ack_term_index(src, res.term, res.index);
    }
    // If AppendEntries fails because of log inconsistency: decrement nextIndex and retry (§5.3)
    todo!()
  }

  fn ack_term_index(&mut self, src: NodeID, _term: Term, index: Index) -> Vec<Output> {
    println!("ack_term_index src {:?} index {:?}", src, index);
    self.match_index.insert(src, index);
    // If there exists an N such that N > commitIndex, a majority of
    // matchIndex[i] ≥ N, and log[N].term == currentTerm: set commitIndex = N
    // (§5.3, §5.4).
    let needed = self.majority();
    for entry in self.log.iter().rev() {
      println!("ack_term_index needed {:?} entry {:?}", needed, entry);
      if entry.index <= self.commit_index || entry.term < self.current_term {
        break;
      }
      // TODO: inefficient; instead, compute once the min index that has a
      // majority in match_index
      let count = self.match_index.iter().filter(|(_, index)| **index >= entry.index).count();
      if count >= needed {
        self.commit_index = entry.index;
        self.maybe_wake();
        return self.maybe_apply();
      }
    }
    return vec![];
  }

  fn process_request_vote(&mut self, req: &RequestVoteReq) -> Vec<Output> {
    // Reply false if term < currentTerm (§5.1)
    if req.term < self.current_term {
      let payload =
        Payload::RequestVoteRes(RequestVoteRes { term: self.current_term, vote_granted: false });
      return vec![Output::Message(Message {
        src: self.id,
        dest: req.candidate_id,
        payload: payload,
      })];
    }
    // If votedFor is null or candidateId, and candidate’s log is at least as
    // up-to-date as receiver’s log, grant vote (§5.2, §5.4)
    let should_grant = match self.voted_for {
      None => true,
      Some(voted_for) => voted_for == req.candidate_id,
    };
    if should_grant {
      // WIP what was this? volatile.current_time = self.volatile.current_time;
      self.voted_for = Some(req.candidate_id);
      let payload =
        Payload::RequestVoteRes(RequestVoteRes { term: self.current_term, vote_granted: true });
      return vec![Output::Message(Message {
        src: self.id,
        dest: req.candidate_id,
        payload: payload,
      })];
    }
    return vec![];
  }

  fn process_request_vote_res(&mut self, res: &RequestVoteRes) -> Vec<Output> {
    // NB: The term was checked earlier so don't need to check it again.
    // WIP debug_assert!(res.term == self.current_term);
    if res.vote_granted {
      // WIP what happens if we get this message twice?
      self.received_votes += 1;
      let needed_votes = self.majority();
      if self.received_votes >= needed_votes {
        // Candidates (§5.2): If votes received from majority of servers:
        // become leader
        return self.convert_to_leader();
      }
    }
    return vec![];
  }

  fn send_append_entries(&mut self, _entries: Vec<Entry>) -> Vec<Output> {
    todo!()
  }

  fn start_election(&mut self, now: Instant) -> Vec<Output> {
    // TODO: this is a little awkward, revisit
    if self.nodes.len() == 1 {
      return self.convert_to_leader();
    }
    // Increment currentTerm
    let Term(current_term) = self.current_term;
    self.current_term = Term(current_term + 1);
    // Vote for self
    self.received_votes = 1;
    // Reset election timer
    self.last_communication = now;
    // Send RequestVote RPCs to all other servers
    let (last_log_term, last_log_index) =
      self.log.last().map_or((Term(0), Index(0)), |entry| (entry.term, entry.index));
    let payload = Payload::RequestVoteReq(RequestVoteReq {
      term: self.current_term,
      candidate_id: self.id,
      last_log_index: last_log_index,
      last_log_term: last_log_term,
    });
    self.message_to_all_other_nodes(payload)
  }

  fn message_to_all_other_nodes(&self, payload: Payload) -> Vec<Output> {
    self
      .nodes
      .iter()
      .filter(|node| **node != self.id)
      .map(|node| Output::Message(Message { src: self.id, dest: *node, payload: payload.clone() }))
      .collect()
  }

  fn convert_to_candidate(&mut self, now: Instant) -> Vec<Output> {
    self.role = Role::Candidate;
    // Candidates (§5.2): On conversion to candidate, start election:
    self.start_election(now)
  }

  fn convert_to_follower(&mut self) -> Vec<Output> {
    self.role = Role::Follower;
    // WIP set last_communication?
    vec![]
  }

  fn convert_to_leader(&mut self) -> Vec<Output> {
    self.role = Role::Leader;
    self.next_index.clear();
    self.match_index.clear();
    // Leaders: Upon election: send initial empty AppendEntries RPCs
    // (heartbeat) to each server; repeat during idle periods to prevent
    // election timeouts (§5.2)
    self.write(vec![], None)
  }

  fn majority(&self) -> usize {
    (self.nodes.len() + 1) / 2
  }
}

#[cfg(test)]
mod testgroup;

#[cfg(test)]
mod tests;
