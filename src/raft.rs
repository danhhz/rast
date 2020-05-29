// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::cmp;
use std::collections::{BTreeMap, HashMap};
use std::iter::Extend;
use std::time::{Duration, Instant};

use super::compressed_log::CompressedLog;
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
  PersistRes(PersistRes),
  ReadStateMachineRes(ReadStateMachineRes),
}

#[derive(Debug)]
pub enum Output {
  Message(Message),
  PersistReq(PersistReq),
  ApplyReq(Index),
  ReadStateMachineReq(ReadStateMachineReq),
}

#[derive(Debug)]
pub struct PersistReq {
  pub leader_id: NodeID,
  pub read_id: ReadID,
  pub entries: Vec<Entry>,
}

#[derive(Debug)]
pub struct PersistRes {
  pub leader_id: NodeID,
  pub read_id: ReadID,
  pub log_index: Index, // TODO: remove this
}

#[derive(Debug)]
pub struct ReadStateMachineReq {
  pub index: Index,
  pub read_id: ReadID,
  pub payload: Vec<u8>,
}

#[derive(Debug)]
pub struct ReadStateMachineRes {
  pub index: Index, // TODO: remove this
  pub read_id: ReadID,
  pub payload: Vec<u8>,
}

/// An implementation of the [raft consensus protocol].
///
/// [raft consensus protocol]: https://raft.github.io/
pub struct Raft {
  state: Option<State>,
}

impl Raft {
  pub fn new(id: NodeID, nodes: Vec<NodeID>, cfg: Config) -> Raft {
    let state = State::Candidate(Candidate {
      shared: SharedState {
        id: id,
        cfg: cfg,
        current_term: Term(0),
        voted_for: None,
        log: CompressedLog::new(),
        commit_index: Index(0),
        last_applied: Index(0),
        nodes: nodes,
        current_time: None,
        last_communication: None,
      },
      received_votes: 0,
    });
    Raft { state: Some(state) }
  }

  pub fn id(&self) -> NodeID {
    return self.state.as_ref().expect("unreachable").id();
  }

  pub fn current_time(&self) -> Option<Instant> {
    return self.state.as_ref().expect("unreachable").shared().current_time;
  }

  pub fn current_term(&self) -> Term {
    return self.state.as_ref().expect("unreachable").shared().current_term;
  }

  pub fn debug(&self) -> &'static str {
    return self.state.as_ref().expect("unreachable").debug();
  }

  pub fn step(&mut self, output: &mut impl Extend<Output>, input: Input) {
    // TODO: this is not actually "unreachable" if step panics, handle this
    // somehow
    self.state = Some(self.state.take().expect("unreachable").step(output, input));
  }

  fn shutdown(&mut self) {
    self.state.take().expect("unreachable").shutdown()
  }
}

// TODO: split into persistent/volatile
struct SharedState {
  id: NodeID,
  cfg: Config,

  // Persistent state
  current_term: Term,
  voted_for: Option<NodeID>,
  log: CompressedLog,

  // Volatile state
  commit_index: Index,
  last_applied: Index,
  // TODO: double check this doesn't need to be persisted
  nodes: Vec<NodeID>,
  current_time: Option<Instant>,
  // TODO: this is overloaded fixme
  last_communication: Option<Instant>,
}

struct Candidate {
  shared: SharedState,

  received_votes: usize,
}

struct Leader {
  shared: SharedState,

  _next_index: HashMap<NodeID, Index>,
  match_index: HashMap<NodeID, (Index, ReadID)>,
  write_buffer: HashMap<(Term, Index), WriteFuture>,

  // invariant: every outgoing AppendEntries round gets a (Term, ReadID) that's
  // unique all time.
  next_read_id: ReadID,

  max_outstanding_read_id: Option<ReadID>,
  max_confirmed_read_id: Option<ReadID>,

  // invariant: all ReadIDs < next_read_id
  // invariant: shared.last_applied <= all Indexes <= shared.commit_index
  read_buffer: BTreeMap<(Index, ReadID), (Option<ReadReq>, ReadFuture)>,
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
  // TODO: this helper is awkward
  fn id(&self) -> NodeID {
    self.shared().id
  }

  // TODO: this helper is awkward
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

  fn step(self, output: &mut impl Extend<Output>, input: Input) -> State {
    debug!("  {:3}: step {:?}", self.id().0, self.debug());
    match input {
      Input::Write(req, res) => self.write(output, req.payload, Some(res)),
      Input::Read(req, res) => self.read(output, req, res),
      Input::Tick(now) => self.tick(output, now),
      Input::PersistRes(res) => self.persist_res(output, res),
      Input::ReadStateMachineRes(res) => self.read_state_machine_res(output, res),
      Input::Message(message) => self.message(output, message),
    }
  }

  fn maybe_wake_writes(mut leader: Leader) -> Leader {
    let current_term = leader.shared.current_term;
    let commit_index = leader.shared.commit_index;
    #[cfg(feature = "log")]
    let id = leader.shared.id;
    leader.write_buffer.retain(|(term, index), future| {
      debug_assert!(*term == current_term);
      if *index >= commit_index {
        let res = WriteRes { term: *term, index: *index };
        debug!("  {:3}: write success {:?}", id.0, res);
        future.fill(Ok(res));
        false
      } else {
        true
      }
    });
    leader
  }

  // NB: this needs to be called any time commit_index or min_outstanding_read
  // are updated.
  //
  // NB: don't call this directly, use leader_maybe_apply or
  // follower_maybe_apply.
  fn maybe_apply(
    shared: &mut SharedState,
    output: &mut impl Extend<Output>,
    upper_bound: Option<Index>,
  ) {
    debug!(
      "  {:3}: maybe_apply commit_index={:?} last_applied={:?} upper_bound={:?}",
      shared.id.0, shared.commit_index, shared.last_applied, upper_bound,
    );
    // We can't advance this past any outstanding reads, else we'd make it
    // impossible to serve them later.
    let new_applied = upper_bound
      .map_or(shared.commit_index, |upper_bound| cmp::min(upper_bound, shared.commit_index));
    if new_applied > shared.last_applied {
      shared.last_applied = new_applied;
      output.extend(vec![Output::ApplyReq(shared.last_applied)]);
    }
  }

  fn leader_maybe_advance_reads(mut leader: Leader, output: &mut impl Extend<Output>) -> Leader {
    debug!(
      "  {:3}: leader_maybe_advance_reads max_applied={:?} outstanding={:?} confirmed={:?} read_buffer={:?}",
      leader.shared.id.0,
      leader.shared.last_applied,
      leader.max_outstanding_read_id,
      leader.max_confirmed_read_id,
      leader.read_buffer.keys()
    );
    if let Some(((_, max_read_id), _)) = leader.read_buffer.iter().next_back() {
      let need_heartbeat =
        leader.max_confirmed_read_id.map_or(true, |confirmed| *max_read_id > confirmed)
          && leader.max_outstanding_read_id.map_or(true, |outstanding| *max_read_id > outstanding);
      debug!("  {:3}: need_heartbeat={:?}", leader.shared.id.0, need_heartbeat);
      if need_heartbeat {
        leader = State::leader_heartbeat(leader, output);
      }
    }

    let can_serve = (leader.shared.last_applied, leader.max_confirmed_read_id.unwrap_or(ReadID(0)));
    debug!("  {:3}: can_serve={:?}", leader.shared.id.0, can_serve);
    for ((index, read_id), (req, _)) in leader.read_buffer.range_mut(..can_serve) {
      // NB: Only do this once for each read.
      if let Some(req) = req.take() {
        let msg = ReadStateMachineReq { index: *index, read_id: *read_id, payload: req.payload };
        output.extend(vec![Output::ReadStateMachineReq(msg)]);
      }
    }
    leader
  }

  fn write(
    self,
    output: &mut impl Extend<Output>,
    payload: Vec<u8>,
    mut res: Option<WriteFuture>,
  ) -> State {
    #[cfg(feature = "log")]
    match std::str::from_utf8(&payload) {
      Ok(payload) => {
        debug!("  {:3}: write {:?}", self.id().0, payload);
      }
      Err(_) => {
        debug!("  {:3}: write {:?}", self.id().0, payload);
      }
    }
    match self {
      State::Leader(leader) => {
        State::Leader(State::leader_write(leader, output, vec![(payload, res)]))
      }
      State::Candidate(candidate) => match candidate.shared.voted_for {
        Some(voted_for) => {
          // TODO: if voted_for is this node, we may want to wait and see if we
          // win the election
          if let Some(mut res) = res.take() {
            res.fill(Err(ClientError::NotLeaderError(NotLeaderError::new(Some(voted_for)))));
          };
          return State::Candidate(candidate);
        }
        None => {
          // We haven't voted yet so start an election, then try the write
          // again, maybe we'll be able to serve it.

          // TODO: hard state means it's impossible to jump to leader in a
          // single step (even in a 1 node cluster) but maybe we can stash the
          // write somewhere on candidates and only time them out if it ends up
          // a follower instead of a leader
          let state = State::start_election(candidate, output);
          state.write(output, payload, res)
        }
      },
      State::Follower(_) => todo!(),
    }
  }

  fn leader_heartbeat(leader: Leader, output: &mut impl Extend<Output>) -> Leader {
    // Leaders: Upon election: send initial empty AppendEntries RPCs
    // (heartbeat) to each server; repeat during idle periods to prevent
    // election timeouts (§5.2)
    State::leader_write(leader, output, vec![])
  }

  fn leader_write(
    mut leader: Leader,
    output: &mut impl Extend<Output>,
    payloads: Vec<(Vec<u8>, Option<WriteFuture>)>,
  ) -> Leader {
    let (prev_log_term, prev_log_index) = leader.shared.log.last();
    let read_id = leader.next_read_id;
    leader.next_read_id = ReadID(leader.next_read_id.0 + 1);
    leader.max_outstanding_read_id = Some(read_id);
    let entries: Vec<_> = payloads
      .into_iter()
      .enumerate()
      .map(|(offset, (payload, res))| {
        let entry = Entry {
          term: leader.shared.current_term,
          index: prev_log_index + offset as u64 + 1,
          payload: payload,
        };
        debug_assert!(leader.write_buffer.get(&(entry.term, entry.index)).is_none());
        if let Some(res) = res {
          leader.write_buffer.insert((entry.term, entry.index), res);
        }
        entry
      })
      .collect();
    debug!("  {:3}: entries={:?}", leader.shared.id.0, entries);

    leader.shared.log.extend(&entries);
    // TODO: this is duplicated with the one in `follower_append_entries`
    debug!("  {:3}: persist {:?}", leader.shared.id.0, &entries);
    if entries.len() > 0 {
      let msg =
        PersistReq { leader_id: leader.shared.id, read_id: read_id, entries: entries.clone() };
      output.extend(vec![Output::PersistReq(msg)]);
    } else {
      let id = leader.shared.id;
      leader = State::ack_term_index(leader, output, id, prev_log_index, read_id);
    }
    let payload = Payload::AppendEntriesReq(AppendEntriesReq {
      term: leader.shared.current_term,
      leader_id: leader.shared.id,
      prev_log_index: prev_log_index,
      prev_log_term: prev_log_term,
      leader_commit: leader.shared.commit_index,
      entries: entries,
      read_id: read_id,
    });
    State::message_to_all_other_nodes(&leader.shared, output, &payload);
    leader
  }

  // start: save read_index
  // if no read_index, have to start everything when first heartbeat comes back
  // send heartbeat with read_id, save read with read_index and read_id
  // get heartbeat with read_id, mark every read with read_id <= that one as ready
  // when apply = read_index serve read. don't let apply get ahead of read_index
  // what if heartbeat doesn't come back? next heartbeat or write retries it
  fn read(self, output: &mut impl Extend<Output>, req: ReadReq, mut res: ReadFuture) -> State {
    debug!("  {:3}: read {:?}", self.id().0, req);
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
          // We haven't voted yet so start an election, then try the read
          // again, maybe we'll be able to serve it.
          let state = State::start_election(candidate, output);
          state.read(output, req, res)
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
    let read_id = leader.next_read_id;
    leader.next_read_id = ReadID(leader.next_read_id.0 + 1);
    let index = leader.shared.log.last().1;
    leader.read_buffer.insert((index, read_id), (Some(req), res));
    State::leader_maybe_advance_reads(leader, output)
  }

  fn tick(self, output: &mut impl Extend<Output>, now: Instant) -> State {
    debug!(
      "  {:3}: self.tick={:?} current_time={:?}",
      self.shared().id.0,
      now,
      self.shared().current_time
    );
    if self.shared().current_time.map_or(false, |current_time| now <= current_time) {
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
        candidate.shared.current_time = Some(now);
        if timed_out {
          return State::start_election(candidate, output);
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
        follower.shared.current_time = Some(now);
        if timed_out {
          return State::follower_convert_to_candidate(follower, output);
        }
        State::Follower(follower)
      }
      State::Leader(mut leader) => {
        let need_heartbeat = leader.shared.last_communication.map_or(true, |last_communication| {
          now.duration_since(last_communication) >= leader.shared.cfg.heartbeat_interval
        });
        leader.shared.current_time = Some(now);
        if need_heartbeat {
          // Leaders: Upon election: send initial empty AppendEntries RPCs
          // (heartbeat) to each server; repeat during idle periods to prevent
          // election timeouts (§5.2)
          leader = State::leader_heartbeat(leader, output)
        }
        State::Leader(leader)
      }
    }
  }

  fn persist_res(self, output: &mut impl Extend<Output>, res: PersistRes) -> State {
    let payload = Payload::AppendEntriesRes(AppendEntriesRes {
      term: self.shared().current_term,
      success: true,
      index: res.log_index,
      read_id: res.read_id,
    });
    let msg = Message { src: self.id(), dest: res.leader_id, payload: payload };
    if msg.src == msg.dest {
      return State::step(self, output, Input::Message(msg));
    }
    output.extend(vec![Output::Message(msg)]);
    self
  }

  fn read_state_machine_res(
    self,
    output: &mut impl Extend<Output>,
    res: ReadStateMachineRes,
  ) -> State {
    let mut leader = match self {
      State::Leader(leader) => leader,
      // NB: It should be possible to serve reads even after losing leadership
      // but it's subtle and also hard to avoid leaking the result future. Seems
      // not worth it.
      State::Candidate(candidate) => return State::Candidate(candidate),
      State::Follower(follower) => return State::Follower(follower),
    };

    let index = res.index;
    let payload = res.payload;
    // Remove the entry so ReadStateMachineRes is idempotent.
    if let Some((_, mut res)) = leader.read_buffer.remove(&(res.index, res.read_id)) {
      #[cfg(feature = "log")]
      match std::str::from_utf8(&payload) {
        Ok(payload) => {
          debug!("  {:3}: read success {:?}", leader.shared.id.0, payload);
        }
        Err(_) => {
          debug!("  {:3}: read success {:?}", leader.shared.id.0, payload);
        }
      }
      res.fill(Ok(ReadRes { term: leader.shared.current_term, index: index, payload: payload }));
    }
    // If that was the last outstanding read, it may have unblocked applying new
    // entries.
    leader = State::leader_maybe_apply(leader, output);
    State::Leader(leader)
  }

  fn leader_maybe_apply(mut leader: Leader, output: &mut impl Extend<Output>) -> Leader {
    let min_outstanding_read: Option<Index> =
      leader.read_buffer.iter().next().map(|((index, _), _)| *index);
    State::maybe_apply(&mut leader.shared, output, min_outstanding_read);
    leader
  }

  fn follower_maybe_apply(mut follower: Follower, output: &mut impl Extend<Output>) -> Follower {
    State::maybe_apply(&mut follower.shared, output, None);
    follower
  }

  fn message(mut self, output: &mut impl Extend<Output>, message: Message) -> State {
    {
      let mut shared = self.shared_mut();
      let term = match &message.payload {
        Payload::AppendEntriesReq(req) => req.term,
        Payload::AppendEntriesRes(res) => res.term,
        Payload::RequestVoteReq(req) => req.term,
        Payload::RequestVoteRes(res) => res.term,
        Payload::StartElectionReq(req) => req.term,
      };
      if term > shared.current_term {
        // All Servers: If RPC request or response contains term T >
        // currentTerm: set currentTerm = T, convert to follower (§5.1)
        shared.current_term = term;
        // TODO: probably want a helper for updating the term
        shared.voted_for = None;
        // TODO: do we really convert to follower on a RequestVoteReq with a
        // higher term?
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
        State::start_election(candidate, output)
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
        State::Follower(State::follower_append_entries(follower, output, req))
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
        State::follower_convert_to_candidate(follower, output)
      }
      Payload::RequestVoteRes(_) => {
        // Already a follower, no-op.
        State::Follower(follower)
      }
    }
  }

  fn leader_step(leader: Leader, output: &mut impl Extend<Output>, message: Message) -> State {
    match message.payload {
      Payload::AppendEntriesRes(res) => {
        State::Leader(State::leader_append_entries_res(leader, output, message.src, res))
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

  fn follower_append_entries(
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
        read_id: req.read_id,
      });
      let msg =
        Output::Message(Message { src: follower.shared.id, dest: req.leader_id, payload: payload });
      output.extend(vec![msg]);
      return follower;
    }

    debug!(
      "  {:3}: self.last_communication={:?}",
      follower.shared.id.0, follower.shared.current_time
    );
    follower.shared.last_communication = follower.shared.current_time;

    // Reply false if log doesn’t contain an entry at prevLogIndex whose term
    // matches prevLogTerm (§5.3)
    let log_match = follower
      .shared
      .log
      .index_term(req.prev_log_index)
      .map_or(false, |term| term == req.prev_log_term);
    if !log_match {
      // TODO: Send back a hint of what we do have.
      let payload = Payload::AppendEntriesRes(AppendEntriesRes {
        term: follower.shared.current_term,
        index: Index(0),
        success: false,
        read_id: req.read_id,
      });
      let msg =
        Output::Message(Message { src: follower.shared.id, dest: req.leader_id, payload: payload });
      output.extend(vec![msg]);
      return follower;
    }

    // If an existing entry conflicts with a new one (same index but different
    // terms), delete the existing entry and all that follow it (§5.3). Append
    // any new entries not already in the log
    if req.entries.len() > 0 {
      follower.shared.log.extend(&req.entries);
      let msg = PersistReq { leader_id: req.leader_id, read_id: req.read_id, entries: req.entries };
      output.extend(vec![Output::PersistReq(msg)]);
    } else {
      // TODO: duplicated with persist_res
      let payload = Payload::AppendEntriesRes(AppendEntriesRes {
        term: follower.shared.current_term,
        success: true,
        index: req.prev_log_index, // TODO: is this right?
        read_id: req.read_id,
      });
      let msg = Message { src: follower.shared.id, dest: req.leader_id, payload: payload };
      output.extend(vec![Output::Message(msg)]);
    }

    // If leaderCommit > commitIndex, set commitIndex = min(leaderCommit, index
    // of last new entry)
    if req.leader_commit > follower.shared.commit_index {
      let last_entry_index = follower.shared.log.last().1;
      follower.shared.commit_index = cmp::min(req.leader_commit, last_entry_index);
      follower = State::follower_maybe_apply(follower, output);
    }
    follower
  }

  fn leader_append_entries_res(
    leader: Leader,
    output: &mut impl Extend<Output>,
    src: NodeID,
    res: AppendEntriesRes,
  ) -> Leader {
    // If successful: update nextIndex and matchIndex for follower (§5.3)
    if res.success {
      return State::ack_term_index(leader, output, src, res.index, res.read_id);
    }
    // If AppendEntries fails because of log inconsistency: decrement nextIndex and retry (§5.3)
    todo!()
  }

  fn ack_term_index(
    mut leader: Leader,
    output: &mut impl Extend<Output>,
    src: NodeID,
    index: Index,
    read_id: ReadID,
  ) -> Leader {
    debug!("  {:3}: self.ack_term_index src={:?} index={:}", leader.shared.id.0, src, index.0);
    leader
      .match_index
      .entry(src)
      .and_modify(|index_read_id| *index_read_id = cmp::max(*index_read_id, (index, read_id)))
      .or_insert((index, read_id));

    // See if max_confirmed_read_id has advanced.
    let mut read_ids: Vec<ReadID> =
      leader.match_index.iter().map(|(_, (_, read_id))| *read_id).collect();
    read_ids.sort_unstable();
    debug!("  {:3}: read_ids={:?}", leader.shared.id.0, &read_ids);
    if read_ids.len() >= State::majority(&leader.shared) {
      let new_max_confirmed_read_id =
        read_ids.get(read_ids.len() - State::majority(&leader.shared)).copied();
      debug!(
        "  {:3}: read_ids={:?} new_confirmed={:?}",
        leader.shared.id.0, &read_ids, new_max_confirmed_read_id
      );
      debug_assert!(
        new_max_confirmed_read_id >= leader.max_confirmed_read_id,
        "{:?} vs {:?}",
        new_max_confirmed_read_id,
        leader.max_confirmed_read_id
      );
      // TODO: debug_assert that new_max_confirmed_read_id has a majority and that
      // it's the highest read_id with a majority.
      leader.max_confirmed_read_id = new_max_confirmed_read_id;
      debug!(
        "  {:3}: outstanding={:?} confirmed={:?}",
        leader.shared.id.0, leader.max_outstanding_read_id, leader.max_confirmed_read_id
      );
      if new_max_confirmed_read_id >= leader.max_outstanding_read_id {
        debug!("  {:3}: no outstanding reads", leader.shared.id.0);
        leader.max_outstanding_read_id = None;
      }
      leader = State::leader_maybe_advance_reads(leader, output);
    }

    debug!(
      "  {:3}: match_indexes={:?}",
      leader.shared.id.0,
      leader.match_index.iter().map(|(_, (index, _))| *index).collect::<Vec<_>>(),
    );
    // If there exists an N such that N > commitIndex, a majority of
    // matchIndex[i] ≥ N, and log[N].term == currentTerm: set commitIndex = N
    // (§5.3, §5.4).
    let needed = State::majority(&leader.shared);
    for (_, entry_index) in leader.shared.log.iter().rev() {
      debug!(
        "  {:3}: is committed? index={:} commit_index={:}",
        leader.shared.id.0, entry_index.0, leader.shared.commit_index.0
      );
      if entry_index <= leader.shared.commit_index {
        break;
      }
      // TODO: inefficient; instead, compute once the min index that has a
      // majority in match_index
      let count = leader.match_index.iter().filter(|(_, (index, _))| *index >= entry_index).count();
      if count >= needed {
        let new_commit_index = entry_index;
        debug!("  {:3}: commit_index={:?}", leader.shared.id.0, new_commit_index);
        leader.shared.commit_index = new_commit_index;
        leader = State::leader_maybe_apply(leader, output);
        // TODO: think about the order of these
        leader = State::maybe_wake_writes(leader);
        leader = State::leader_maybe_advance_reads(leader, output);
        break;
      }
    }
    leader
  }

  fn process_request_vote(
    mut self,
    output: &mut impl Extend<Output>,
    req: &RequestVoteReq,
  ) -> State {
    // TODO: To prevent [disruption from removed nodes] servers disregard
    // RequestVote RPCs when they believe a current leader exists. Specifically,
    // if a server receives a RequestVote RPC within the minimum election
    // timeout of hearing from a current leader, it does not update its term or
    // grant its vote. (§6)

    let mut shared = self.shared_mut();
    debug!(
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
    if res.vote_granted {
      // TODO: this is not idempotent
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

  fn start_election(mut candidate: Candidate, output: &mut impl Extend<Output>) -> State {
    debug!("  {:3}: start_election {:?}", candidate.shared.id.0, candidate.shared.current_time);
    candidate.received_votes = 0;
    // TODO: this is awkward
    candidate.shared.voted_for = Some(candidate.shared.id);
    // Increment currentTerm
    candidate.shared.current_term = Term(candidate.shared.current_term.0 + 1);
    // Reset election timer
    candidate.shared.last_communication = candidate.shared.current_time;
    // Send RequestVote RPCs to all other servers
    let (last_log_term, last_log_index) = candidate.shared.log.last();
    let payload = Payload::RequestVoteReq(RequestVoteReq {
      term: candidate.shared.current_term,
      candidate_id: candidate.shared.id,
      last_log_index: last_log_index,
      last_log_term: last_log_term,
    });
    debug!("  {:3}: reqvote {:?}", candidate.shared.id.0, payload);
    State::message_to_all_other_nodes(&candidate.shared, output, &payload);
    // Vote for self
    let res = RequestVoteRes { term: candidate.shared.current_term, vote_granted: true };
    return State::candidate_process_request_vote_res(candidate, output, &res);
  }

  fn message_to_all_other_nodes(
    shared: &SharedState,
    output: &mut impl Extend<Output>,
    payload: &Payload,
  ) {
    output.extend(shared.nodes.iter().filter(|node| **node != shared.id).map(|node| {
      Output::Message(Message { src: shared.id, dest: *node, payload: payload.clone() })
    }))
  }

  fn follower_convert_to_candidate(follower: Follower, output: &mut impl Extend<Output>) -> State {
    debug!("  {:3}: convert_to_candidate", follower.shared.id.0);
    let candidate = Candidate { shared: follower.shared, received_votes: 0 };
    // Candidates (§5.2): On conversion to candidate, start election:
    State::start_election(candidate, output)
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
        // NB: This can happen if a follower gets an AppendEntries from an
        // already elected leader in a new term.
        follower
      }
    }
  }

  fn candidate_convert_to_follower(
    candidate: Candidate,
    _output: &mut impl Extend<Output>,
    new_leader_hint: NodeID,
  ) -> Follower {
    debug!("  {:3}: convert_to_follower leader={:?}", candidate.shared.id.0, new_leader_hint.0);
    Follower { shared: candidate.shared, leader_hint: Some(new_leader_hint) }
  }

  fn leader_convert_to_follower(
    mut leader: Leader,
    _output: &mut impl Extend<Output>,
    new_leader_hint: NodeID,
  ) -> Follower {
    debug!("  {:3}: convert_to_follower leader={:?}", leader.shared.id.0, new_leader_hint.0);
    leader = State::clear_outstanding_requests(leader, Some(new_leader_hint));
    Follower { shared: leader.shared, leader_hint: Some(new_leader_hint) }
  }

  fn candidate_convert_to_leader(candidate: Candidate, output: &mut impl Extend<Output>) -> Leader {
    debug!("  {:3}: convert_to_leader", candidate.shared.id.0);

    let leader = Leader {
      shared: candidate.shared,

      // next_read_id resets to 0 for each new term
      next_read_id: ReadID(0),

      // TODO: roundtrip these through the other states and truncate them here
      // to save allocs
      _next_index: HashMap::new(),
      match_index: HashMap::new(),
      write_buffer: HashMap::new(),

      max_outstanding_read_id: None,
      max_confirmed_read_id: None,
      read_buffer: BTreeMap::new(),
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
    leader.read_buffer.iter_mut().for_each(|(_, (_, future))| {
      future.fill(Err(ClientError::NotLeaderError(NotLeaderError::new(new_leader_hint))));
    });
    leader.read_buffer.clear();
    leader
  }

  fn shutdown(self) {
    match self {
      State::Follower(_) | State::Candidate(_) => {} // No-op.
      State::Leader(leader) => {
        let _ = State::clear_outstanding_requests(leader, None);
      }
    }
  }

  fn majority(shared: &SharedState) -> usize {
    (shared.nodes.len() + 1) / 2
  }
}

impl Drop for Raft {
  fn drop(&mut self) {
    self.shutdown()
  }
}

#[cfg(test)]
#[path = "raft_tests.rs"]
mod tests;
