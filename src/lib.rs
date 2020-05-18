// Copyright 2020 Daniel Harrison. All Rights Reserved.

#![warn(clippy::correctness, clippy::perf)]

use std::cmp;
use std::collections::HashMap;
use std::iter::Extend;
use std::time::{Duration, Instant};

mod error;
mod future;
mod log;
mod rpc;
mod runtime;

pub use error::*;
pub use future::*;
pub use log::*;
pub use rpc::*;
pub use runtime::*;

// TODO: figure out how to call output.extend without creating a vec
// TODO: more consistent method naming
// TODO: refactor out a statemachine lib
// TODO: compress log implementation
// TODO: randomized invariant testing
// TODO: failure testing
// TODO: ensure that heartbeat (empty append) is no disk write in common case
// TODO: write internals docs
// TODO: write externals docs
// TODO: example runtime
// TODO: restart node with non-empty log + hard state
// TODO: fix up crate features
// TODO: single node special cases
// - election concludes immediately
// - read/write req to candidate is successful
// - committed as soon as it's persisted
// TODO: benchmarks
// TODO: test behavior under shutdown
// TODO: graceful leader handoff
// TODO: test split vote
// TODO: retry append rpc, find where follower diverges
// TODO: test that nothing can be written at an index once that index is read

#[derive(Debug)]
pub enum Input {
  Write((WriteReq, WriteFuture)),
  Read((ReadReq, ReadFuture)),
  Tick(Instant),
  Message(Message),
  PersistRes(Index, NodeID),
  ReadStateMachine(Index, usize, Vec<u8>),
}

#[derive(Debug)]
pub enum Output {
  Message(Message),
  PersistReq(NodeID, Vec<Entry>), // WIP NodeID is the leader
  ApplyReq(Index),
  ReadStateMachine(Index, usize, Vec<u8>),
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

impl Default for Config {
  fn default() -> Config {
    Config {
      election_timeout: Duration::from_millis(100),
      heartbeat_interval: Duration::from_millis(10),
    }
  }
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
  write_buffer: HashMap<(Term, Index), WriteFuture>,
  min_read_index: Option<Index>,
  read_index: Option<Index>,
  read_buffer_pending: Vec<(ReadReq, ReadFuture)>,
  read_buffer_running: HashMap<usize, (ReadReq, ReadFuture)>,

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
      min_read_index: None,
      read_index: None,
      next_index: HashMap::new(),
      match_index: HashMap::new(),
      received_votes: 0,
      write_buffer: HashMap::new(),
      read_buffer_pending: Vec::new(),
      read_buffer_running: HashMap::new(),
    }
  }

  pub fn write_async(&mut self, output: &mut impl Extend<Output>, req: WriteReq) -> WriteFuture {
    let future = WriteFuture::new();
    self.step(output, Input::Write((req, future.clone())));
    future
  }

  pub fn read_async(&mut self, output: &mut impl Extend<Output>, req: ReadReq) -> ReadFuture {
    let future = ReadFuture::new();
    self.step(output, Input::Read((req, future.clone())));
    future
  }

  pub fn step(&mut self, output: &mut impl Extend<Output>, input: Input) {
    println!("  {:3}: step {:?}", self.id.0, self.role);
    // All Servers: If commitIndex > lastApplied: increment lastApplied, apply
    // log[lastApplied] to state machine (§5.3)
    self.maybe_apply(output);
    match input {
      Input::Write((req, state)) => self.write(output, req.payload, Some(state)),
      Input::Read((req, state)) => self.read(output, req, state),
      Input::Tick(now) => self.tick(output, now),
      Input::PersistRes(index, leader_id) => self.persist_res(output, index, leader_id),
      Input::ReadStateMachine(index, idx, payload) => {
        self.read_state_machine_res(index, idx, payload)
      }
      Input::Message(message) => self.message(output, message),
    };
  }

  fn maybe_wake_writes(&mut self) {
    let current_term = self.current_term;
    let commit_index = self.commit_index;
    let id = self.id;
    self.write_buffer.retain(|(term, index), future| {
      debug_assert!(*term == current_term);
      if *index >= commit_index {
        let res = WriteRes { term: *term, index: *index };
        println!("  {:3}: write success {:?}", id.0, res);
        future.fill(Ok(res));
        false
      } else {
        true
      }
    });
  }

  fn maybe_apply(&mut self, output: &mut impl Extend<Output>) {
    println!(
      "  {:3}: maybe apply read_index={:?} last_applied={:?} read_buffer_pending={:?}",
      self.id.0,
      self.read_index,
      self.last_applied,
      self.read_buffer_pending.len()
    );
    if let Some(read_index) = self.read_index {
      debug_assert!(read_index + 1 >= self.last_applied);
      if self.read_buffer_running.len() > 0 {
        return;
      }
      if read_index + 1 == self.last_applied && self.read_buffer_pending.len() > 0 {
        debug_assert_eq!(self.read_buffer_running.len(), 0);
        // WIP: better unique key generation
        self.read_buffer_running.extend(self.read_buffer_pending.drain(..).enumerate());
        output.extend(
          self
            .read_buffer_running
            .iter()
            .map(|(idx, (req, _))| Output::ReadStateMachine(read_index, *idx, req.payload.clone())),
        );
        // NB: We have to wait for all these to come back before we advance
        // last_applied and output an ApplyReq so we can guarantee that they
        // execute before any writes that were submitted after them (which would
        // be a serializability violation).
        //
        // WIP on that note, what happens if a read is submitted when there is
        // no min_read_index? this story needs some work
        return;
      }
    }
    if self.commit_index > self.last_applied {
      let Index(last_applied) = self.last_applied;
      self.last_applied = Index(last_applied + 1);
      output.extend(vec![Output::ApplyReq(self.last_applied)]);
    }
  }

  fn write(
    &mut self,
    output: &mut impl Extend<Output>,
    payload: Vec<u8>,
    state: Option<WriteFuture>,
  ) {
    match std::str::from_utf8(&payload) {
      Ok(payload) => println!("  {:3}: write {:?}", self.id.0, payload),
      Err(_) => println!("  {:3}: write {:?}", self.id.0, payload),
    }
    let (prev_log_term, prev_log_index) = match self.role {
      Role::Leader => {
        self.log.last().map_or((Term(0), Index(0)), |entry| (entry.term, entry.index))
      }
      Role::Candidate => {
        self.start_election(output, self.current_time);
        if let Some(mut state) = state {
          state.fill(Err(NotLeaderError::new(NodeID(0))));
        };
        return;
      }
      Role::Follower => todo!(),
    };
    let entry = Entry { term: self.current_term, index: prev_log_index + 1, payload: payload };
    // WIP debug assertion that this doesn't exist.
    if let Some(state) = state {
      self.write_buffer.insert((entry.term, entry.index), state);
    }
    // WIP: is this really the right place for this?
    self.log.extend(vec![entry.clone()]);
    println!("  {:3}: persist {:?}", self.id.0, &entry);
    // TODO: this is duplicated with the one in `process_append_entries`
    output.extend(vec![Output::PersistReq(self.id, vec![entry.clone()])]);
    self.ack_term_index(output, self.id, entry.term, entry.index);
    let payload = Payload::AppendEntriesReq(AppendEntriesReq {
      term: self.current_term,
      leader_id: self.id,
      prev_log_index: prev_log_index,
      prev_log_term: prev_log_term,
      leader_commit: self.commit_index,
      entries: vec![entry],
    });
    self.message_to_all_other_nodes(output, payload);
  }

  fn read(&mut self, output: &mut impl Extend<Output>, req: ReadReq, mut state: ReadFuture) {
    println!("  {:3}: read {:?}", self.id.0, req);
    match self.role {
      Role::Candidate => {
        self.start_election(output, self.current_time);
        // TODO: hint
        state.fill(Err(NotLeaderError::new(NodeID(0))));
        return;
      }
      Role::Follower => {
        // TODO: hint
        state.fill(Err(NotLeaderError::new(NodeID(0))));
        return;
      }
      Role::Leader => {} // Only a leader can serve a read, let's go.
    };
    self.read_buffer_pending.push((req, state));

    // TODO: Assert the read_index is only set if min_read_index is and it
    // should be >=.
    if self.read_index.is_none() {
      println!("  {:3}: self.read_index={:?}", self.id.0, self.min_read_index);
      // NB: min_read_index will be none if this leader is newly elected.
      self.read_index = self.min_read_index;
    }

    if self.read_buffer_pending.len() == 1 {
      // We can't serve this read request until the next heartbeat completes, so
      // proactively start one (but only if this is the first batched read
      // request).
      self.write(output, vec![], None);
    }
  }

  fn tick(&mut self, output: &mut impl Extend<Output>, now: Instant) {
    if now <= self.current_time {
      // Ignore a repeat tick (as well as one in the past, which shouldn't
      // happen).
      return;
    }
    self.current_time = now;
    match self.role {
      Role::Candidate => {
        // Candidates (§5.2): If election timeout elapses: start new election
        if now.duration_since(self.last_communication) > self.config.election_timeout {
          self.start_election(output, now);
        }
      }
      Role::Follower => {
        // Followers (§5.2): If election timeout elapses without receiving
        // AppendEntries RPC from current leader or granting vote to candidate:
        // convert to candidate
        if now.duration_since(self.last_communication) > self.config.election_timeout {
          self.convert_to_candidate(output, now);
        }
      }
      Role::Leader => {
        if now.duration_since(self.last_communication) > self.config.heartbeat_interval {
          // Leaders: Upon election: send initial empty AppendEntries RPCs
          // (heartbeat) to each server; repeat during idle periods to prevent
          // election timeouts (§5.2)
          self.send_append_entries(output, vec![]);
        }
      }
    }
  }

  fn persist_res(&mut self, output: &mut impl Extend<Output>, index: Index, leader_id: NodeID) {
    let payload = Payload::AppendEntriesRes(AppendEntriesRes {
      term: self.current_term,
      index: index,
      success: true,
    });
    let msg = Output::Message(Message { src: self.id, dest: leader_id, payload: payload });
    output.extend(vec![msg]);
  }

  fn read_state_machine_res(&mut self, index: Index, idx: usize, payload: Vec<u8>) {
    if self.read_index != Some(index) {
      // Read from some previous term as leader
      return;
    }
    if let Some((_, mut res)) = self.read_buffer_running.remove(&idx) {
      match std::str::from_utf8(&payload) {
        Ok(payload) => println!("  {:3}: read success {:?}", self.id.0, payload),
        Err(_) => println!("  {:3}: read success {:?}", self.id.0, payload),
      }
      res.fill(Ok(ReadRes { term: self.current_term, index: index, payload: payload }));
    }
    if self.read_buffer_running.len() == 0 {
      println!("  {:3}: self.read_index=None", self.id.0);
      self.read_index = None
    }
  }

  fn message(&mut self, output: &mut impl Extend<Output>, message: Message) {
    // TODO: avoid calling payload multiple times
    let term = match &message.payload {
      Payload::AppendEntriesReq(req) => req.term,
      Payload::RequestVoteReq(req) => req.term,
      Payload::AppendEntriesRes(res) => res.term,
      Payload::RequestVoteRes(res) => res.term,
    };
    if term > self.current_term {
      // All Servers: If RPC request or response contains term T > currentTerm:
      // set currentTerm = T, convert to follower (§5.1)
      self.current_term = term;
      self.convert_to_follower(output, message.src);
    }
    match &mut self.role {
      Role::Candidate => self.step_candidate(output, message),
      Role::Follower => self.step_follower(output, message),
      Role::Leader => self.step_leader(output, message),
    };
  }

  fn step_candidate(&mut self, output: &mut impl Extend<Output>, message: Message) {
    match &message.payload {
      Payload::RequestVoteRes(res) => self.process_request_vote_res(output, &res),
      Payload::AppendEntriesReq(req) => {
        if req.term > self.current_term {
          // Candidates (§5.2): If AppendEntries RPC received from new leader:
          // convert to follower
          self.convert_to_follower(output, message.src);
          // WIP: this is awkward
          self.step(output, Input::Message(message));
        }
      }
      Payload::RequestVoteReq(req) => self.process_request_vote(output, req),
      payload => todo!("{:?}", payload),
    }
  }

  fn step_follower(&mut self, output: &mut impl Extend<Output>, message: Message) {
    match message.payload {
      // Followers (§5.2): Respond to RPCs from candidates and leaders
      Payload::AppendEntriesReq(req) => self.process_append_entries(output, req),
      Payload::RequestVoteReq(req) => self.process_request_vote(output, &req),
      Payload::AppendEntriesRes(_) => {
        // TODO: double check this
        // No-op, stale response to a request sent out by this node when it was a leader.
      }
      payload => todo!("{:?}", payload),
    }
  }

  fn step_leader(&mut self, output: &mut impl Extend<Output>, message: Message) {
    match message.payload {
      Payload::AppendEntriesRes(res) => self.process_append_entries_res(output, message.src, res),
      Payload::RequestVoteRes(res) => self.process_request_vote_res(output, &res),
      payload => todo!("{:?}", payload),
    }
  }

  fn process_append_entries(&mut self, output: &mut impl Extend<Output>, req: AppendEntriesReq) {
    // Reply false if term < currentTerm (§5.1)
    if req.term < self.current_term {
      let payload = Payload::AppendEntriesRes(AppendEntriesRes {
        term: self.current_term,
        index: Index(0),
        success: false,
      });
      let msg = Output::Message(Message { src: self.id, dest: req.leader_id, payload: payload });
      output.extend(vec![msg]);
      self.convert_to_follower(output, req.leader_id);
    }

    // WIP: Reply false if log doesn’t contain an entry at prevLogIndex whose
    // term matches prevLogTerm (§5.3)

    // WIP: If an existing entry conflicts with a new one (same index but
    // different terms), delete the existing entry and all that follow it (§5.3)

    // WIP: Append any new entries not already in the log
    self.log.extend(req.entries.clone());
    let msg = Output::PersistReq(req.leader_id, req.entries);
    output.extend(vec![msg]);

    // If leaderCommit > commitIndex, set commitIndex = min(leaderCommit, index
    // of last new entry)
    if req.leader_commit > self.commit_index {
      let last_entry_index = self.log.last().map_or(Index(0), |entry| entry.index);
      self.commit_index = cmp::min(req.leader_commit, last_entry_index);
      self.maybe_apply(output);
    }
  }

  fn process_append_entries_res(
    &mut self,
    output: &mut impl Extend<Output>,
    src: NodeID,
    res: AppendEntriesRes,
  ) {
    // If successful: update nextIndex and matchIndex for follower (§5.3)
    if res.success {
      self.ack_term_index(output, src, res.term, res.index);
      return;
    }
    // If AppendEntries fails because of log inconsistency: decrement nextIndex and retry (§5.3)
    todo!()
  }

  fn ack_term_index(
    &mut self,
    output: &mut impl Extend<Output>,
    src: NodeID,
    _term: Term,
    index: Index,
  ) {
    self.match_index.insert(src, index);
    // If there exists an N such that N > commitIndex, a majority of
    // matchIndex[i] ≥ N, and log[N].term == currentTerm: set commitIndex = N
    // (§5.3, §5.4).
    let needed = self.majority();
    for entry in self.log.iter().rev() {
      if entry.index <= self.commit_index || entry.term < self.current_term {
        break;
      }
      // TODO: inefficient; instead, compute once the min index that has a
      // majority in match_index
      let count = self.match_index.iter().filter(|(_, index)| **index >= entry.index).count();
      if count >= needed {
        let new_commit_index = entry.index;
        self.commit_index = new_commit_index;
        println!("  {:3}: self.min_read_index={:?}", self.id.0, self.commit_index);
        self.min_read_index = Some(self.commit_index);
        self.maybe_wake_writes();
        self.maybe_apply(output);
        return;
      }
    }
  }

  fn process_request_vote(&mut self, output: &mut impl Extend<Output>, req: &RequestVoteReq) {
    // Reply false if term < currentTerm (§5.1)
    if req.term < self.current_term {
      let payload =
        Payload::RequestVoteRes(RequestVoteRes { term: self.current_term, vote_granted: false });
      let msg = Output::Message(Message { src: self.id, dest: req.candidate_id, payload: payload });
      output.extend(vec![msg]);
      return;
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
      let msg = Output::Message(Message { src: self.id, dest: req.candidate_id, payload: payload });
      output.extend(vec![msg]);
    }
  }

  fn process_request_vote_res(&mut self, output: &mut impl Extend<Output>, res: &RequestVoteRes) {
    // NB: The term was checked earlier so don't need to check it again.
    // WIP debug_assert!(res.term == self.current_term);
    if res.vote_granted {
      // WIP what happens if we get this message twice?
      self.received_votes += 1;
      let needed_votes = self.majority();
      if self.received_votes >= needed_votes {
        // Candidates (§5.2): If votes received from majority of servers:
        // become leader
        self.convert_to_leader(output);
        return;
      }
    }
  }

  fn send_append_entries(&mut self, output: &mut impl Extend<Output>, entries: Vec<Entry>) {
    let (prev_log_term, prev_log_index) =
      self.log.last().map_or((Term(0), Index(0)), |entry| (entry.term, entry.index));
    output.extend(vec![Output::PersistReq(self.id, entries.clone())]);
    self.ack_term_index(output, self.id, self.current_term, prev_log_index + entries.len() as u64);
    let payload = Payload::AppendEntriesReq(AppendEntriesReq {
      term: self.current_term,
      leader_id: self.id,
      prev_log_index: prev_log_index,
      prev_log_term: prev_log_term,
      leader_commit: self.commit_index,
      entries: entries,
    });
    self.message_to_all_other_nodes(output, payload);
  }

  fn start_election(&mut self, output: &mut impl Extend<Output>, now: Instant) {
    println!("  {:3}: start_election {:?}", self.id.0, now);
    // TODO: this is a little awkward, revisit
    if self.nodes.len() == 1 {
      self.convert_to_leader(output);
      return;
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
    println!("  {:3}: reqvote {:?}", self.id.0, payload);
    self.message_to_all_other_nodes(output, payload);
  }

  fn message_to_all_other_nodes(&self, output: &mut impl Extend<Output>, payload: Payload) {
    output.extend(
      self.nodes.iter().filter(|node| **node != self.id).map(|node| {
        Output::Message(Message { src: self.id, dest: *node, payload: payload.clone() })
      }),
    )
  }

  fn convert_to_candidate(&mut self, output: &mut impl Extend<Output>, now: Instant) {
    if self.role == Role::Leader {
      self.clear_outstanding_requests(None);
    }
    self.role = Role::Candidate;
    // Candidates (§5.2): On conversion to candidate, start election:
    self.start_election(output, now)
  }

  fn convert_to_follower(&mut self, _output: &mut impl Extend<Output>, leader_hint: NodeID) {
    if self.role == Role::Leader {
      self.clear_outstanding_requests(Some(leader_hint));
    }
    self.role = Role::Follower;
  }

  fn convert_to_leader(&mut self, output: &mut impl Extend<Output>) {
    println!("  {:3}: convert_to_leader", self.id.0);
    self.role = Role::Leader;
    println!("  {:3}: self.min_read_index=None", self.id.0);
    self.min_read_index = None;
    println!("  {:3}: self.read_index=None", self.id.0);
    self.read_index = None;
    self.next_index.clear();
    self.match_index.clear();
    // Leaders: Upon election: send initial empty AppendEntries RPCs
    // (heartbeat) to each server; repeat during idle periods to prevent
    // election timeouts (§5.2)
    self.write(output, vec![], None);
  }

  fn clear_outstanding_requests(&mut self, leader_hint: Option<NodeID>) {
    debug_assert_eq!(self.role, Role::Leader);
    self.write_buffer.drain().for_each(|(_, mut future)| {
      future.fill(Err(NotLeaderError::new(leader_hint.unwrap_or(NodeID(0)))));
    });
    self.read_buffer_pending.drain(..).for_each(|(_, mut future)| {
      future.fill(Err(NotLeaderError::new(leader_hint.unwrap_or(NodeID(0)))));
    });
    self.read_buffer_running.drain().for_each(|(_, (_, mut future))| {
      future.fill(Err(NotLeaderError::new(leader_hint.unwrap_or(NodeID(0)))));
    });
  }

  fn majority(&self) -> usize {
    (self.nodes.len() + 1) / 2
  }
}

#[cfg(test)]
mod testgroup;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub mod nemesis;
