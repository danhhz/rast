// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::cmp;
use std::collections::HashMap;
use std::iter::Extend;
use std::time::{Duration, Instant};

pub use super::error::*;
pub use super::future::*;
pub use super::serde::*;

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

#[derive(Debug)]
pub enum Input {
  Write(WriteReq, WriteFuture),
  Read(ReadReq, ReadFuture),
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

/// An implementation of the [raft consensus protocol].
///
/// [raft consensus protocol]: https://raft.github.io/
pub struct Raft {
  state: Option<State>,
}

impl Raft {
  pub fn new(id: NodeID, nodes: Vec<NodeID>, cfg: Config, current_time: Instant) -> Raft {
    let state = State::Candidate(Candidate {
      shared: SharedState {
        id: id,
        cfg: cfg,
        current_term: Term(0),
        voted_for: None,
        log: vec![],
        commit_index: Index(0),
        last_applied: Index(0),
        nodes: nodes,
        current_time: current_time,
        last_communication: None,
      },
      received_votes: 0,
    });
    Raft { state: Some(state) }
  }

  pub fn id(&self) -> NodeID {
    return self.state.as_ref().expect("unreachable").id();
  }

  pub fn current_time(&self) -> Instant {
    return self.state.as_ref().expect("unreachable").shared().current_time;
  }

  pub fn current_term(&self) -> Term {
    return self.state.as_ref().expect("unreachable").shared().current_term;
  }

  pub fn debug(&self) -> &'static str {
    return self.state.as_ref().expect("unreachable").debug();
  }

  pub fn step(&mut self, output: &mut impl Extend<Output>, input: Input) {
    self.state = Some(self.state.take().expect("unreachable").step(output, input));
  }
}

// WIP: split into persistent/volatile
struct SharedState {
  pub id: NodeID,
  cfg: Config,

  // Persistent state
  current_term: Term,
  voted_for: Option<NodeID>,
  log: Vec<Entry>,

  // Volatile state
  commit_index: Index,
  last_applied: Index,
  nodes: Vec<NodeID>, // WIP: double check this doesn't need to be persisted
  pub current_time: Instant,
  // WIP this is overloaded fixme
  last_communication: Option<Instant>,
}

struct Candidate {
  shared: SharedState,

  received_votes: usize,
}

struct Leader {
  shared: SharedState,

  _next_index: HashMap<NodeID, Index>,
  match_index: HashMap<NodeID, Index>,
  write_buffer: HashMap<(Term, Index), WriteFuture>,
  min_read_index: Option<Index>,
  read_index: Option<Index>,
  read_buffer_pending: Vec<(ReadReq, ReadFuture)>,
  read_buffer_running: HashMap<usize, (ReadReq, ReadFuture)>,
}

struct Follower {
  shared: SharedState,

  leader_hint: Option<NodeID>,
}

#[allow(clippy::large_enum_variant)]
enum State {
  Candidate(Candidate),
  Follower(Follower),
  Leader(Leader),
}

impl State {
  // WIP: this helper is awkward
  fn id(&self) -> NodeID {
    self.shared().id
  }

  // WIP: this helper is awkward
  fn shared(&self) -> &SharedState {
    match self {
      State::Candidate(c) => &c.shared,
      State::Follower(f) => &f.shared,
      State::Leader(l) => &l.shared,
    }
  }

  fn shared_mut(&mut self) -> &mut SharedState {
    match self {
      State::Candidate(c) => &mut c.shared,
      State::Follower(f) => &mut f.shared,
      State::Leader(l) => &mut l.shared,
    }
  }

  // TODO: impl Debug instead
  fn debug(&self) -> &'static str {
    match self {
      State::Candidate(_) => "candidate",
      State::Follower(_) => "follower",
      State::Leader(_) => "leader",
    }
  }

  fn step(mut self, output: &mut impl Extend<Output>, input: Input) -> State {
    println!("  {:3}: step {:?}", self.id().0, self.debug());
    // All Servers: If commitIndex > lastApplied: increment lastApplied, apply
    // log[lastApplied] to state machine (§5.3)
    State::maybe_apply(self.shared_mut(), output);
    match input {
      Input::Write(req, state) => self.write(output, req.payload, Some(state)),
      Input::Read(req, state) => self.read(output, req, state),
      Input::Tick(now) => self.tick(output, now),
      Input::PersistRes(index, leader_id) => self.persist_res(output, index, leader_id),
      Input::ReadStateMachine(index, idx, payload) => match self {
        State::Leader(leader) => {
          State::Leader(State::read_state_machine_res(leader, index, idx, payload))
        }
        _ => todo!(),
      },
      Input::Message(message) => self.message(output, message),
    }
  }

  fn maybe_wake_writes(mut leader: Leader) -> Leader {
    let current_term = leader.shared.current_term;
    let commit_index = leader.shared.commit_index;
    let id = leader.shared.id;
    leader.write_buffer.retain(|(term, index), future| {
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
    leader
  }

  fn maybe_apply(shared: &mut SharedState, output: &mut impl Extend<Output>) {
    if shared.commit_index > shared.last_applied {
      println!(
        "  {:3}: maybe_apply commit_index={:?} last_applied={:?}",
        shared.id.0, shared.commit_index, shared.last_applied
      );
      let Index(last_applied) = shared.last_applied;
      shared.last_applied = Index(last_applied + 1);
      output.extend(vec![Output::ApplyReq(shared.last_applied)]);
    }
  }

  fn leader_maybe_fill_buffered_reads(
    mut leader: Leader,
    output: &mut impl Extend<Output>,
  ) -> Leader {
    println!(
      "  {:3}: leader_maybe_fill_buffered_reads read_index={:?} read_buffer_pending={:?}",
      leader.shared.id.0,
      leader.read_index,
      leader.read_buffer_pending.len()
    );
    if let Some(read_index) = leader.read_index {
      debug_assert!(read_index + 1 >= leader.shared.last_applied);
      if leader.read_buffer_running.len() > 0 {
        return leader;
      }
      if read_index + 1 == leader.shared.last_applied && leader.read_buffer_pending.len() > 0 {
        debug_assert_eq!(leader.read_buffer_running.len(), 0);
        // WIP: better unique key generation
        leader.read_buffer_running.extend(leader.read_buffer_pending.drain(..).enumerate());
        output.extend(
          leader
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
      }
    }
    leader
  }

  fn write(
    self,
    output: &mut impl Extend<Output>,
    payload: Vec<u8>,
    res: Option<WriteFuture>,
  ) -> State {
    match std::str::from_utf8(&payload) {
      Ok(payload) => println!("  {:3}: write {:?}", self.id().0, payload),
      Err(_) => println!("  {:3}: write {:?}", self.id().0, payload),
    }
    match self {
      State::Leader(leader) => State::Leader(State::leader_write(leader, output, payload, res)),
      State::Candidate(candidate) => match candidate.shared.voted_for {
        Some(voted_for) => {
          // TODO: if voted_for is this node, we may want to wait and see if we
          // win the election
          if let Some(mut res) = res {
            res.fill(Err(ClientError::NotLeaderError(NotLeaderError::new(Some(voted_for)))));
          };
          return State::Candidate(candidate);
        }
        None => {
          // We haven't voted yet so start an election, then try the write
          // again, maybe we'll be able to serve it.
          let now = candidate.shared.current_time;
          let candidate = State::start_election(candidate, output, now);
          // WIP: hard state means this is currently impossible but maybe we can
          // stash things somewhere on candidates and only time them out if it
          // becomes a follower instead of a leader
          return State::Candidate(candidate).write(output, payload, res);
        }
      },
      State::Follower(_) => todo!(),
    }
  }

  fn leader_heartbeat(leader: Leader, output: &mut impl Extend<Output>) -> Leader {
    // Leaders: Upon election: send initial empty AppendEntries RPCs
    // (heartbeat) to each server; repeat during idle periods to prevent
    // election timeouts (§5.2)
    State::leader_write(leader, output, vec![], None)
  }

  fn leader_write(
    mut leader: Leader,
    output: &mut impl Extend<Output>,
    payload: Vec<u8>,
    res: Option<WriteFuture>,
  ) -> Leader {
    let (prev_log_term, prev_log_index) =
      leader.shared.log.last().map_or((Term(0), Index(0)), |entry| (entry.term, entry.index));
    let entry =
      Entry { term: leader.shared.current_term, index: prev_log_index + 1, payload: payload };
    // WIP debug assertion that this doesn't exist.
    if let Some(res) = res {
      leader.write_buffer.insert((entry.term, entry.index), res);
    }
    // WIP: is this really the right place for this?
    leader.shared.log.extend(vec![entry.clone()]);
    println!("  {:3}: persist {:?}", leader.shared.id.0, &entry);
    // TODO: this is duplicated with the one in `process_append_entries`
    output.extend(vec![Output::PersistReq(leader.shared.id, vec![entry.clone()])]);
    let id = leader.shared.id;
    leader = State::ack_term_index(leader, output, id, entry.term, entry.index);
    let payload = Payload::AppendEntriesReq(AppendEntriesReq {
      term: leader.shared.current_term,
      leader_id: leader.shared.id,
      prev_log_index: prev_log_index,
      prev_log_term: prev_log_term,
      leader_commit: leader.shared.commit_index,
      entries: vec![entry],
    });
    State::message_to_all_other_nodes(&leader.shared, output, payload);
    leader
  }

  fn read(self, output: &mut impl Extend<Output>, req: ReadReq, mut res: ReadFuture) -> State {
    println!("  {:3}: read {:?}", self.id().0, req);
    match self {
      State::Leader(leader) => {
        // Only a leader can serve a read, let's go.
        State::Leader(State::leader_read(leader, output, req, res))
      }
      // TODO: dedeup these with the ones in write
      State::Candidate(candidate) => match candidate.shared.voted_for {
        Some(voted_for) => {
          // TODO: if voted_for is this node, we may want to wait and see if we
          // win the election
          res.fill(Err(ClientError::NotLeaderError(NotLeaderError::new(Some(voted_for)))));
          return State::Candidate(candidate);
        }
        None => {
          // We haven't voted yet so start an election, then try the write
          // again, maybe we'll be able to serve it.
          let now = candidate.shared.current_time;
          let candidate = State::start_election(candidate, output, now);
          return State::Candidate(candidate).read(output, req, res);
        }
      },
      State::Follower(follower) => {
        res.fill(Err(ClientError::NotLeaderError(NotLeaderError::new(follower.leader_hint))));
        State::Follower(follower)
      }
    }
  }

  fn leader_read(
    mut leader: Leader,
    output: &mut impl Extend<Output>,
    req: ReadReq,
    res: ReadFuture,
  ) -> Leader {
    leader.read_buffer_pending.push((req, res));

    // TODO: assert the read_index is only set if min_read_index is and it
    // should be >=
    if leader.read_index.is_none() {
      println!("  {:3}: self.read_index={:?}", leader.shared.id.0, leader.min_read_index);
      // NB: min_read_index will be none if this leader is newly elected.
      leader.read_index = leader.min_read_index;
    }

    if leader.read_buffer_pending.len() == 1 {
      // We can't serve this read request until the next heartbeat completes, so
      // proactively start one (but only if this is the first batched read
      // request).
      leader = State::leader_heartbeat(leader, output);
    }
    leader
  }

  fn tick(self, output: &mut impl Extend<Output>, now: Instant) -> State {
    println!(
      "  {:3}: self.tick={:?} current_time={:?}",
      self.shared().id.0,
      now,
      self.shared().current_time
    );
    if now <= self.shared().current_time {
      // Ignore a repeat tick (as well as one in the past, which shouldn't
      // happen).
      return self;
    }
    match self {
      State::Candidate(mut candidate) => {
        // Candidates (§5.2): If election timeout elapses: start new election
        let timed_out = candidate.shared.last_communication.map_or(true, |last_communication| {
          now.duration_since(last_communication) >= candidate.shared.cfg.election_timeout
        });
        candidate.shared.current_time = now;
        if timed_out {
          candidate = State::start_election(candidate, output, now);
        }
        State::Candidate(candidate)
      }
      State::Follower(mut follower) => {
        // Followers (§5.2): If election timeout elapses without receiving
        // AppendEntries RPC from current leader or granting vote to candidate:
        // convert to candidate
        let timed_out = follower.shared.last_communication.map_or(true, |last_communication| {
          now.duration_since(last_communication) >= follower.shared.cfg.election_timeout
        });
        follower.shared.current_time = now;
        if timed_out {
          return State::Candidate(State::follower_convert_to_candidate(follower, output, now));
        }
        State::Follower(follower)
      }
      State::Leader(mut leader) => {
        let need_heartbeat = leader.shared.last_communication.map_or(true, |last_communication| {
          now.duration_since(last_communication) >= leader.shared.cfg.heartbeat_interval
        });
        leader.shared.current_time = now;
        if need_heartbeat {
          // Leaders: Upon election: send initial empty AppendEntries RPCs
          // (heartbeat) to each server; repeat during idle periods to prevent
          // election timeouts (§5.2)
          leader = State::send_append_entries(leader, output, vec![]);
        }
        State::Leader(leader)
      }
    }
  }

  fn persist_res(self, output: &mut impl Extend<Output>, index: Index, leader_id: NodeID) -> State {
    let payload = Payload::AppendEntriesRes(AppendEntriesRes {
      term: self.shared().current_term,
      index: index,
      success: true,
    });
    let msg = Output::Message(Message { src: self.id(), dest: leader_id, payload: payload });
    // TODO: special case dest == self.id()
    output.extend(vec![msg]);
    self
  }

  fn read_state_machine_res(
    mut leader: Leader,
    index: Index,
    idx: usize,
    payload: Vec<u8>,
  ) -> Leader {
    // TODO: can we ever respond to reads after converting to a non-leader?
    // seems like yes but doing it is subtle and it's hard to avoid leaking the
    // res future
    if leader.read_index != Some(index) {
      // Read from some previous term as leader
      return leader;
    }
    if let Some((_, mut res)) = leader.read_buffer_running.remove(&idx) {
      match std::str::from_utf8(&payload) {
        Ok(payload) => println!("  {:3}: read success {:?}", leader.shared.id.0, payload),
        Err(_) => println!("  {:3}: read success {:?}", leader.shared.id.0, payload),
      }
      res.fill(Ok(ReadRes { term: leader.shared.current_term, index: index, payload: payload }));
    }
    if leader.read_buffer_running.len() == 0 {
      println!("  {:3}: self.read_index=None", leader.shared.id.0);
      leader.read_index = None
    }
    leader
  }

  fn message(mut self, output: &mut impl Extend<Output>, message: Message) -> State {
    {
      let mut shared = self.shared_mut();
      // TODO: avoid calling payload multiple times
      let term = match &message.payload {
        Payload::AppendEntriesReq(req) => req.term,
        Payload::AppendEntriesRes(res) => res.term,
        Payload::RequestVoteReq(req) => req.term,
        Payload::RequestVoteRes(res) => res.term,
        Payload::StartElectionReq(req) => req.term,
      };
      if term > shared.current_term {
        // All Servers: If RPC request or response contains term T > currentTerm:
        // set currentTerm = T, convert to follower (§5.1)
        shared.current_term = term;
        // WIP: probably want a helper for updating the term
        shared.voted_for = None;
        // WIP: do we really convert to follower on a RequestVoteReq with a higher
        // term?
        self = State::Follower(self.convert_to_follower(output, message.src));
      }
    }
    match self {
      State::Candidate(candidate) => State::candidate_step(candidate, output, message),
      State::Follower(follower) => State::follower_step(follower, output, message),
      State::Leader(leader) => State::leader_step(leader, output, message),
    }
  }

  fn candidate_step(
    candidate: Candidate,
    output: &mut impl Extend<Output>,
    message: Message,
  ) -> State {
    match &message.payload {
      Payload::RequestVoteRes(res) => {
        State::candidate_process_request_vote_res(candidate, output, &res)
      }
      Payload::AppendEntriesReq(req) => {
        if req.term > candidate.shared.current_term {
          // Candidates (§5.2): If AppendEntries RPC received from new leader:
          // convert to follower
          let follower = State::candidate_convert_to_follower(candidate, output, message.src);
          return State::Follower(follower).step(output, Input::Message(message));
        }
        State::Candidate(candidate)
      }
      Payload::RequestVoteReq(req) => State::Candidate(candidate).process_request_vote(output, req),
      Payload::StartElectionReq(req) => {
        if req.term < candidate.shared.current_term {
          // Stale request, ignore.
          return State::Candidate(candidate);
        }
        let now = candidate.shared.current_time;
        State::Candidate(State::start_election(candidate, output, now))
      }
      payload => todo!("{:?}", payload),
    }
  }

  fn follower_step(
    follower: Follower,
    output: &mut impl Extend<Output>,
    message: Message,
  ) -> State {
    match message.payload {
      // Followers (§5.2): Respond to RPCs from candidates and leaders
      Payload::AppendEntriesReq(req) => {
        State::Follower(State::process_append_entries(follower, output, req))
      }
      Payload::RequestVoteReq(req) => State::Follower(follower).process_request_vote(output, &req),
      Payload::AppendEntriesRes(_) => {
        // TODO: double check this
        // No-op, stale response to a request sent out by this node when it was a leader.
        State::Follower(follower)
      }
      Payload::StartElectionReq(req) => {
        if req.term < follower.shared.current_term {
          // Stale request, ignore.
          return State::Follower(follower);
        }
        let now = follower.shared.current_time;
        let candidate = State::follower_convert_to_candidate(follower, output, now);
        let now = candidate.shared.current_time;
        State::Candidate(State::start_election(candidate, output, now))
      }
      payload => todo!("{:?}", payload),
    }
  }

  fn leader_step(leader: Leader, output: &mut impl Extend<Output>, message: Message) -> State {
    match message.payload {
      Payload::AppendEntriesRes(res) => {
        State::Leader(State::leader_process_append_entries_res(leader, output, message.src, res))
      }
      Payload::RequestVoteRes(_) => {
        // Already the leader, nothing to do here.
        State::Leader(leader)
      }
      Payload::StartElectionReq(_) => {
        // Already the leader, nothing to do here.
        State::Leader(leader)
      }
      payload => todo!("{:?}", payload),
    }
  }

  fn process_append_entries(
    mut follower: Follower,
    output: &mut impl Extend<Output>,
    req: AppendEntriesReq,
  ) -> Follower {
    // Reply false if term < currentTerm (§5.1)
    if req.term < follower.shared.current_term {
      let payload = Payload::AppendEntriesRes(AppendEntriesRes {
        term: follower.shared.current_term,
        index: Index(0),
        success: false,
      });
      let msg =
        Output::Message(Message { src: follower.shared.id, dest: req.leader_id, payload: payload });
      output.extend(vec![msg]);
      return follower;
    }

    println!(
      "  {:3}: self.last_communication={:?}",
      follower.shared.id.0, follower.shared.current_time
    );
    follower.shared.last_communication = Some(follower.shared.current_time);

    // WIP: Reply false if log doesn’t contain an entry at prevLogIndex whose
    // term matches prevLogTerm (§5.3)

    // WIP: If an existing entry conflicts with a new one (same index but
    // different terms), delete the existing entry and all that follow it (§5.3)

    // WIP: Append any new entries not already in the log
    follower.shared.log.extend(req.entries.clone());
    let msg = Output::PersistReq(req.leader_id, req.entries);
    output.extend(vec![msg]);

    // If leaderCommit > commitIndex, set commitIndex = min(leaderCommit, index
    // of last new entry)
    if req.leader_commit > follower.shared.commit_index {
      let last_entry_index = follower.shared.log.last().map_or(Index(0), |entry| entry.index);
      follower.shared.commit_index = cmp::min(req.leader_commit, last_entry_index);
      State::maybe_apply(&mut follower.shared, output);
    }
    follower
  }

  fn leader_process_append_entries_res(
    leader: Leader,
    output: &mut impl Extend<Output>,
    src: NodeID,
    res: AppendEntriesRes,
  ) -> Leader {
    // If successful: update nextIndex and matchIndex for follower (§5.3)
    if res.success {
      return State::ack_term_index(leader, output, src, res.term, res.index);
    }
    // If AppendEntries fails because of log inconsistency: decrement nextIndex and retry (§5.3)
    todo!()
  }

  fn ack_term_index(
    mut leader: Leader,
    output: &mut impl Extend<Output>,
    src: NodeID,
    _term: Term,
    index: Index,
  ) -> Leader {
    leader.match_index.insert(src, index);
    // If there exists an N such that N > commitIndex, a majority of
    // matchIndex[i] ≥ N, and log[N].term == currentTerm: set commitIndex = N
    // (§5.3, §5.4).
    let needed = State::majority(&leader.shared);
    for entry in leader.shared.log.iter().rev() {
      if entry.index <= leader.shared.commit_index || entry.term < leader.shared.current_term {
        break;
      }
      // TODO: inefficient; instead, compute once the min index that has a
      // majority in match_index
      let count = leader.match_index.iter().filter(|(_, index)| **index >= entry.index).count();
      if count >= needed {
        let new_commit_index = entry.index;
        leader.shared.commit_index = new_commit_index;
        println!(
          "  {:3}: self.min_read_index={:?}",
          leader.shared.id.0, leader.shared.commit_index
        );
        leader.min_read_index = Some(leader.shared.commit_index);
        leader = State::maybe_wake_writes(leader);
        leader = State::leader_maybe_fill_buffered_reads(leader, output);
        State::maybe_apply(&mut leader.shared, output);
        break;
      }
    }
    return leader;
  }

  fn process_request_vote(
    mut self,
    output: &mut impl Extend<Output>,
    req: &RequestVoteReq,
  ) -> State {
    let mut shared = self.shared_mut();
    println!(
      "  {:3}: self.process_request_vote voted_for={:?} req={:?}",
      shared.id.0, shared.voted_for, req
    );
    // Reply false if term < currentTerm (§5.1)
    if req.term < shared.current_term {
      let payload =
        Payload::RequestVoteRes(RequestVoteRes { term: shared.current_term, vote_granted: false });
      let msg =
        Output::Message(Message { src: self.id(), dest: req.candidate_id, payload: payload });
      output.extend(vec![msg]);
      return self;
    }
    // If votedFor is null or candidateId, and candidate’s log is at least as
    // up-to-date as receiver’s log, grant vote (§5.2, §5.4)
    let should_grant = match shared.voted_for {
      None => true,
      Some(voted_for) => voted_for == req.candidate_id,
    };
    if should_grant {
      // WIP what was this? volatile.current_time = self.volatile.current_time;
      shared.voted_for = Some(req.candidate_id);
      let payload =
        Payload::RequestVoteRes(RequestVoteRes { term: shared.current_term, vote_granted: true });
      let msg =
        Output::Message(Message { src: shared.id, dest: req.candidate_id, payload: payload });
      output.extend(vec![msg]);
    }
    self
  }

  fn candidate_process_request_vote_res(
    mut candidate: Candidate,
    output: &mut impl Extend<Output>,
    res: &RequestVoteRes,
  ) -> State {
    // NB: The term was checked earlier so don't need to check it again.
    // WIP debug_assert!(res.term == self.current_term);
    if res.vote_granted {
      // WIP what happens if we get this message twice?
      candidate.received_votes += 1;
      let needed_votes = State::majority(&candidate.shared);
      if candidate.received_votes >= needed_votes {
        // Candidates (§5.2): If votes received from majority of servers:
        // become leader
        return State::Leader(State::candidate_convert_to_leader(candidate, output));
      }
    }
    return State::Candidate(candidate);
  }

  fn send_append_entries(
    mut leader: Leader,
    output: &mut impl Extend<Output>,
    entries: Vec<Entry>,
  ) -> Leader {
    let (prev_log_term, prev_log_index) =
      leader.shared.log.last().map_or((Term(0), Index(0)), |entry| (entry.term, entry.index));
    output.extend(vec![Output::PersistReq(leader.shared.id, entries.clone())]);
    let id = leader.shared.id;
    let current_term = leader.shared.current_term;
    leader = State::ack_term_index(
      leader,
      output,
      id,
      current_term,
      prev_log_index + entries.len() as u64,
    );
    let payload = Payload::AppendEntriesReq(AppendEntriesReq {
      term: leader.shared.current_term,
      leader_id: leader.shared.id,
      prev_log_index: prev_log_index,
      prev_log_term: prev_log_term,
      leader_commit: leader.shared.commit_index,
      entries: entries,
    });
    State::message_to_all_other_nodes(&leader.shared, output, payload);
    leader
  }

  fn start_election(
    mut candidate: Candidate,
    output: &mut impl Extend<Output>,
    now: Instant,
  ) -> Candidate {
    println!("  {:3}: start_election {:?}", candidate.shared.id.0, now);
    // WIP: this is a little awkward (and also probably wrong), revisit
    // if candidate.shared.nodes.len() == 1 {
    //         return State::Leader(self.candidate_convert_to_leader(candidate, output);
    // }
    // Increment currentTerm
    let Term(current_term) = candidate.shared.current_term;
    candidate.shared.current_term = Term(current_term + 1);
    // Vote for self
    candidate.received_votes = 1;
    candidate.shared.voted_for = Some(candidate.shared.id);
    // Reset election timer
    candidate.shared.last_communication = Some(now);
    // Send RequestVote RPCs to all other servers
    let (last_log_term, last_log_index) =
      candidate.shared.log.last().map_or((Term(0), Index(0)), |entry| (entry.term, entry.index));
    let payload = Payload::RequestVoteReq(RequestVoteReq {
      term: candidate.shared.current_term,
      candidate_id: candidate.shared.id,
      last_log_index: last_log_index,
      last_log_term: last_log_term,
    });
    println!("  {:3}: reqvote {:?}", candidate.shared.id.0, payload);
    State::message_to_all_other_nodes(&candidate.shared, output, payload);
    candidate
  }

  fn message_to_all_other_nodes(
    shared: &SharedState,
    output: &mut impl Extend<Output>,
    payload: Payload,
  ) {
    output.extend(shared.nodes.iter().filter(|node| **node != shared.id).map(|node| {
      Output::Message(Message { src: shared.id, dest: *node, payload: payload.clone() })
    }))
  }

  fn follower_convert_to_candidate(
    follower: Follower,
    output: &mut impl Extend<Output>,
    now: Instant,
  ) -> Candidate {
    println!("  {:3}: convert_to_candidate {:?}", follower.shared.id.0, now);
    let candidate = Candidate { shared: follower.shared, received_votes: 0 };
    // Candidates (§5.2): On conversion to candidate, start election:
    State::start_election(candidate, output, now)
  }

  fn convert_to_follower(
    self,
    output: &mut impl Extend<Output>,
    new_leader_hint: NodeID,
  ) -> Follower {
    match self {
      State::Candidate(candidate) => {
        State::candidate_convert_to_follower(candidate, output, new_leader_hint)
      }
      State::Leader(leader) => State::leader_convert_to_follower(leader, output, new_leader_hint),
      State::Follower(follower) => {
        // WIP: double check that this makes sense
        follower
      }
    }
  }

  fn candidate_convert_to_follower(
    candidate: Candidate,
    _output: &mut impl Extend<Output>,
    new_leader_hint: NodeID,
  ) -> Follower {
    println!("  {:3}: convert_to_follower {:?}", candidate.shared.id.0, new_leader_hint);
    Follower { shared: candidate.shared, leader_hint: Some(new_leader_hint) }
  }

  fn leader_convert_to_follower(
    mut leader: Leader,
    _output: &mut impl Extend<Output>,
    new_leader_hint: NodeID,
  ) -> Follower {
    println!("  {:3}: convert_to_follower {:?}", leader.shared.id.0, new_leader_hint);
    leader = State::clear_outstanding_requests(leader, Some(new_leader_hint));
    Follower { shared: leader.shared, leader_hint: Some(new_leader_hint) }
  }

  fn candidate_convert_to_leader(candidate: Candidate, output: &mut impl Extend<Output>) -> Leader {
    println!("  {:3}: convert_to_leader", candidate.shared.id.0);
    println!("  {:3}: self.min_read_index=None", candidate.shared.id.0);
    println!("  {:3}: self.read_index=None", candidate.shared.id.0);

    let leader = Leader {
      shared: candidate.shared,

      // TODO: roundtrip these through the other states and truncate them here
      // to save allocs
      _next_index: HashMap::new(),
      match_index: HashMap::new(),
      write_buffer: HashMap::new(),
      min_read_index: None,
      read_index: None,
      read_buffer_pending: Vec::new(),
      read_buffer_running: HashMap::new(),
    };
    // Leaders: Upon election: send initial empty AppendEntries RPCs
    // (heartbeat) to each server; repeat during idle periods to prevent
    // election timeouts (§5.2)
    State::leader_heartbeat(leader, output)
  }

  fn clear_outstanding_requests(mut leader: Leader, new_leader_hint: Option<NodeID>) -> Leader {
    leader.write_buffer.drain().for_each(|(_, mut future)| {
      future.fill(Err(ClientError::NotLeaderError(NotLeaderError::new(new_leader_hint))));
    });
    leader.read_buffer_pending.drain(..).for_each(|(_, mut future)| {
      future.fill(Err(ClientError::NotLeaderError(NotLeaderError::new(new_leader_hint))));
    });
    leader.read_buffer_running.drain().for_each(|(_, (_, mut future))| {
      future.fill(Err(ClientError::NotLeaderError(NotLeaderError::new(new_leader_hint))));
    });
    leader
  }

  fn majority(shared: &SharedState) -> usize {
    (shared.nodes.len() + 1) / 2
  }
}

#[cfg(test)]
#[path = "raft_tests.rs"]
mod tests;
