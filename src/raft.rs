// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::cmp;
use std::collections::{BTreeMap, HashMap};
use std::iter::Extend;
use std::time::{Duration, Instant};

use capnp_runtime::slice::Slice;

use super::compressed_log::CompressedLog;
pub use super::error::*;
pub use super::future::*;
pub use super::serde::*;

/// Raft tunables.
///
/// The configuration of all nodes in a group should match, but certain
/// variations between them are permissible when altering the configuration in a
/// rolling restart. TODO: Describe how this would work.
#[derive(Debug, Clone)]
pub struct Config {
  /// The interval after which a node will assume the current leader is dead and
  /// call an election. TODO: Notes on tuning this.
  pub election_timeout: Duration,
  /// The interval after which a leader will notify its peers that they don't
  /// need to call an elecation. This should be less than `election_timeout`.
  /// TODO: Should this be derived from `election_timeout`?
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

/// An input to [`step`](Raft::step).
///
/// Inputs include read requests, write requests, an rpc arrival, clock ticks,
/// and the completon of disk IO.
///
/// There are no particular constraints on the order that inputs are processed
/// except those described in [`Output`].
#[derive(Debug)]
pub enum Input<'a> {
  /// A user request to enter a write in the replicated state machine.
  ///
  /// The write payload is an opaque `Vec<u8>` handed as-is to the state machine
  /// for intrepretation. The provided future will be resolved when this write
  /// completes.
  Write(WriteReq, WriteFuture),
  /// A user request to read from the replicated state machine.
  ///
  /// The read payload is an opaque `Vec<u8>` handed as-is to the state machine
  /// for intrepretation. The provided future will be resolved when this read
  /// completes.
  Read(ReadReq, ReadFuture),
  /// A communication to the Raft logic of the current time.
  ///
  /// Correctness of this Raft implementation (including reads) is entirely
  /// independant from clocks. However, Raft very much relies on a periodic
  /// clock tick for availability. If ticks are delayed, unnecessary elections
  /// and leadership transfers will happen, which affects tail latencies.
  ///
  /// TODO: How often should a tick event happen for a given [`Config`]?
  Tick(Instant),
  /// An rpc resulting from a [`Output::Message`] on another node.
  Message(Message<'a>),
  /// A communication that a [`Output::PersistReq`] has completed.
  PersistRes(PersistRes),
  /// A communication that a [`Output::ReadStateMachineReq`] has completed.
  ReadStateMachineRes(ReadStateMachineRes),
}

/// An owned version of [`Input`].
#[derive(Clone, Debug)]
pub enum OwnedInput {
  /// An owned version of [`Input::Write`].
  Write(WriteReq, WriteFuture),
  /// An owned version of [`Input::Read`].
  Read(ReadReq, ReadFuture),
  /// An owned version of [`Input::Tick`].
  Tick(Instant),
  /// An owned version of [`Input::Message`].
  Message(MessageShared),
  /// An owned version of [`Input::PersistRes`].
  PersistRes(PersistRes),
  /// An owned version of [`Input::ReadStateMachineRes`].
  ReadStateMachineRes(ReadStateMachineRes),
}

impl OwnedInput {
  /// Returns a borrowed reference to this owned step input.
  pub fn as_ref<'a>(&'a self) -> Input<'a> {
    match self {
      // WIP no clones here
      OwnedInput::Write(req, res) => Input::Write(req.clone(), res.clone()),
      OwnedInput::Read(req, res) => Input::Read(req.clone(), res.clone()),
      OwnedInput::Tick(tick) => Input::Tick(*tick),
      OwnedInput::Message(msg) => Input::Message(msg.capnp_as_ref()),
      OwnedInput::PersistRes(res) => Input::PersistRes(res.clone()),
      OwnedInput::ReadStateMachineRes(res) => Input::ReadStateMachineRes(res.clone()),
    }
  }
}

impl<'a> From<Input<'a>> for OwnedInput {
  fn from(input: Input<'a>) -> Self {
    match input {
      Input::Write(req, res) => OwnedInput::Write(req, res),
      Input::Read(req, res) => OwnedInput::Read(req, res),
      Input::Tick(tick) => OwnedInput::Tick(tick),
      Input::Message(msg) => OwnedInput::Message(msg.capnp_to_owned()),
      Input::PersistRes(res) => OwnedInput::PersistRes(res),
      Input::ReadStateMachineRes(res) => OwnedInput::ReadStateMachineRes(res),
    }
  }
}

/// An output from [`step`](Raft::step).
///
/// Outputs include rpcs to send and data to be persisted.
///
/// `Message` outputs are assumed to be lossy (any necessary messages will be
/// retried if they are lost). It is also not required for correctness that
/// messages are delivered in the order that they are output. However, it's best
/// for availability if messages between any two nodes are a delivered in order.
///
/// All disk outputs must be processed and in the order they are emitted. This
/// applies to the `PersistReq`, `ApplyReq`, and `ReadStateMachineReq` outputs.
#[derive(Debug)]
pub enum Output {
  /// An rpc to be sent to another node by the runtime.
  Message(MessageShared),
  /// A request that the given entries be durably written to the Raft log.
  ///
  /// Completion is communciated to Raft by an [`Input::PersistRes`]. Processing
  /// this request is subject to the ordering requirements described on
  /// [`Output`].
  PersistReq(PersistReq),
  /// A request that the given entries be applied to the state machine.
  ///
  /// No communication of completion is necessary but processing this request is
  /// subject to the ordering requirements described on [`Output`].
  ApplyReq(Index),
  /// A request that the state machine's current state be read.
  ///
  /// Completion is communciated to Raft by an [`Input::ReadStateMachineRes`].
  /// Processing this request is subject to the ordering requirements described
  /// on [`Output`].
  ReadStateMachineReq(ReadStateMachineReq),
}

/// See [`Output::PersistReq`].
#[derive(Debug)]
pub struct PersistReq {
  /// The id of the leader that initiated this write. This must be copied to the
  /// resulting `PersistRes`.
  pub leader_id: NodeID,
  /// This must be copied to the resulting `PersistRes`.
  pub read_id: ReadID,
  /// The Raft log entries to be durably persisted to disk.
  pub entries: Vec<EntryShared>,
}

/// See [`Input::PersistRes`].
#[derive(Clone, Debug)]
pub struct PersistRes {
  /// The id of the leader that initiated this write. This must be copied from
  /// the corresponding `PersistReq`.
  pub leader_id: NodeID,
  /// This must be copied from the corresponding `PersistReq`.
  pub read_id: ReadID,
  /// TODO: Remove this.
  pub log_index: Index,
}

/// See [`Output::ReadStateMachineReq`].
#[derive(Debug)]
pub struct ReadStateMachineReq {
  /// This must be copied to the resulting `ReadStateMachineRes`. TODO: Remove
  /// this.
  pub index: Index,
  /// This must be copied to the resulting `ReadStateMachineRes`.
  pub read_id: ReadID,
  /// The read payload to be handed to the replicated state machine.
  ///
  /// For example: This could be a key when the replicated state machine is a
  /// key-value store.
  pub payload: Vec<u8>,
}

/// See [`Input::ReadStateMachineRes`].
#[derive(Clone, Debug)]
pub struct ReadStateMachineRes {
  /// This must be copied from the corresponding `PersistReq`. TODO: Remove
  /// this.
  pub index: Index,
  /// This must be copied from the corresponding `PersistReq`.
  pub read_id: ReadID,
  /// The result of reading the state machine with the request's payload.
  ///
  /// For example: This could be a value when the replicated state machine is a
  /// key-value store.
  pub payload: Vec<u8>,
}

/// An implementation of the [raft consensus protocol].
///
/// [raft consensus protocol]: https://raft.github.io/
///
/// Paraphrased from the [Raft paper]: Raft is a consensus algorithm for
/// managing a replicated state machine via a replicated log. Each _node_
/// (called a server by the Raft literature) stores a log containing a series of
/// commands, which its state machine executes in order. Each log contains the
/// same commands in the same order, so each state machine processes the same
/// sequence of commands. Since the state machines are deterministic, the
/// maintained states all match.
///
/// [raft paper]: https://raft.github.io/raft.pdf
///
/// This is a deterministic implementation of the Raft logic. Network and disk
/// IO are modeled as inputs and outputs to the [`step`](Raft::step) method.
/// This method takes one input (an rpc arrival, a disk write has finished) and
/// produces zero or more outputs (send this rpc, write this to disk). Raft uses
/// a clock for availability and this is also modeled as an input. `step` is
/// called in a loop, invoked whenever there is a new input. This produces
/// outputs, which represent network/disk IO to be performed. The completion of
/// that IO is then communicated as an input to `step` (possibly on another
/// node).
///
/// These input and output events are written to be fully pipelineable. For
/// example, when an "AppendEntries" rpc arrives, Raft requires that the new log
/// entries are persisted to disk before the rpc response is sent (similarly for
/// "RequestVote" and persisting the Raft "hard state"). An output requests the
/// disk write and when it finishes an input communicating this is given to
/// `step`. This will, in turn, cause an output with the "AppendEntries" rpc
/// response. While the entries is being persisted to disk, `step` can continue
/// to be called with other rpc messages, click ticks, etc. See [`Input`] and
/// [`Output`] for details.
///
/// [`Read`](Input::Read)s and [`Write`](Input::Write)s are similarly modeled as
/// inputs. With each, a [`Future`](std::future::Future) is passed in that is
/// resolved with the result of the read or write when it completes.
///
/// This is an implementation of only the core Raft logic and needs an rpc
/// system, service discovery, a disk-backed log, and a disk-backed state
/// machine. Implementations of these, as well as a "batteries included" runtime
/// loop is included in the [`runtime`](crate::runtime) module.
///
/// TODO: Document consistency guarantees.
pub struct Raft {
  state: Option<State>,
}

impl Raft {
  /// Returns a new, empty Raft node.
  ///
  /// This should not be used when a node restarts. The `id` must be unique
  /// all-time. It must be reused if the node restarts and cannot ever be reused
  /// (whether by another node or this one if it loses data). The `peers` must
  /// contain all nodes in the group, including this one.
  pub fn new(id: NodeID, peers: Vec<NodeID>, cfg: Config) -> Raft {
    let state = State::Candidate(Candidate {
      shared: SharedState {
        id: id,
        cfg: cfg,
        current_term: Term(0),
        voted_for: None,
        log: CompressedLog::new(),
        commit_index: Index(0),
        last_applied: Index(0),
        peers: peers,
        current_time: None,
        last_communication: None,
      },
      received_votes: 0,
    });
    Raft { state: Some(state) }
  }

  /// The unique id of this node.
  pub fn id(&self) -> NodeID {
    return self.state_ref().id();
  }

  #[cfg(test)]
  pub fn current_time(&self) -> Option<Instant> {
    return self.state_ref().shared().current_time;
  }

  #[cfg(test)]
  pub fn current_term(&self) -> Term {
    return self.state_ref().shared().current_term;
  }

  #[cfg(test)]
  fn debug(&self) -> &'static str {
    return self.state_ref().debug();
  }

  /// Advance the raft logic in response to a single input.
  ///
  /// This is guaranteed to be non-blocking. Any blocking work (network/disk IO)
  /// is emitted as an [`Output`] entry.
  pub fn step(&mut self, output: &mut impl Extend<Output>, input: Input) {
    // TODO: this is not actually "unreachable" if step panics, handle this
    self.state = Some(self.state.take().expect("unreachable").step(output, input));
  }

  fn state_ref(&self) -> &State {
    // TODO: this is not actually "unreachable" if step panics, handle this
    self.state.as_ref().expect("unreachable")
  }

  /// Instruct the (possibly remote) node to gracefully become the new leader.
  pub fn start_election(&mut self, output: &mut impl Extend<Output>, new_leader: NodeID) {
    let current_term = self.state_ref().shared().current_term;
    let msg = PayloadShared::StartElectionReq(StartElectionReqShared::new(current_term));
    let msg = MessageShared::new(self.id(), new_leader, msg);
    if msg.capnp_as_ref().dest() == self.id() {
      self.step(output, Input::Message(msg.capnp_as_ref()));
    } else {
      output.extend(vec![Output::Message(msg)]);
    }
  }

  fn shutdown(&mut self) {
    self.state.take().expect("unreachable").shutdown()
  }
}

// TODO: split into persistent/volatile
#[derive(Debug)]
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
  peers: Vec<NodeID>,
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

  leader_hint: NodeID,
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
  #[cfg(test)]
  fn debug(&self) -> &'static str {
    match self {
      State::Candidate(_) => "candidate",
      State::Follower(_) => "follower",
      State::Leader(_) => "leader",
    }
  }

  // NB: In some cases, step recursively calls itself. Example: When a candidate
  // receives a read or write, it campaigns for leadership and tries the read or
  // write again.
  fn step(self, output: &mut impl Extend<Output>, input: Input) -> State {
    #[cfg(test)]
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
      State::Follower(follower) => {
        if let Some(mut res) = res.take() {
          res.fill(Err(ClientError::NotLeaderError(NotLeaderError::new(Some(
            follower.leader_hint,
          )))));
        };
        State::Follower(follower)
      }
    }
  }

  fn leader_heartbeat(leader: Leader, output: &mut impl Extend<Output>) -> Leader {
    // Leaders: Upon election: send initial empty AppendEntries rpcs
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
        let entry = EntryShared::new(
          leader.shared.current_term,
          prev_log_index + offset as u64 + 1,
          &payload,
        );
        let entry_ref: Entry = entry.capnp_as_ref();
        debug_assert!(leader.write_buffer.get(&(entry_ref.term(), entry_ref.index())).is_none());
        if let Some(res) = res {
          leader.write_buffer.insert((entry_ref.term(), entry_ref.index()), res);
        }
        entry
      })
      .collect();
    debug!("  {:3}: entries={:?}", leader.shared.id.0, entries);

    leader.shared.log.extend(&entries.iter().map(|x| x.capnp_as_ref()).collect::<Vec<_>>());
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
    let payload = PayloadShared::AppendEntriesReq(AppendEntriesReqShared::new(
      leader.shared.current_term,
      leader.shared.id,
      prev_log_index,
      prev_log_term,
      leader.shared.commit_index,
      read_id,
      &entries,
    ));
    State::message_to_all_other_nodes(&leader.shared, output, payload);
    leader
  }

  /// Queues a user read request to be processed.
  ///
  /// Reads are implemented as the write-less variant described in the Raft
  /// paper. (NB: Not the leasing variant that's clock dependant). Currently,
  /// reads are only serveable by leaders, but it's possible to extend this
  /// scheme to followers. The paper describes two requirements for serving
  /// these reads.
  ///
  /// 1) The leader must have committed some entry, thus committing everything
  ///    that was in its log when it was elected. This is only interesting for
  ///    new leaders.
  /// 2) The leader snapshots its highest log index (the per-read "read index")
  ///    at the time the read is queued and must wait for a
  ///    heartbeat/AppendEntries to succeed. This confirms the leader was still
  ///    active at the time the read was queued and allows the read to be served
  ///    at the read index.
  ///
  /// Mechanically, this is implemented as follows:
  ///
  /// - An id space, [`ReadID`], is introduced for AppendEntries and reads. This
  ///   resets to 0 for each term and increments for each read and for each
  ///   "round" of AppendEntries. This means that (Term, ReadID) gives a total
  ///   ordering to reads and AppendEntries.
  /// - When a read is queued, the highest log index is snapshotted and the next
  ///   ReadID is taken. These are buffered with the read request and response
  ///   future.
  /// - To ensure responsiveness, if there were no reads queued, the next
  ///   heartbeat is immediately started. (NB: This heartbeat will have a higher
  ///   ReadID). To prevent excessive heartbeats, if any reads were already
  ///   queued, we don't immediately kick off a second heartbeat. However,
  ///   whenever a heartbeat finishes, if there are buffered reads that don't
  ///   have an outstanding AppendEntries (imagine a write in the meantime),
  ///   another one is immediately kicked off. This naturally batches reads
  ///   under high read loads and keeps latencies low, while bounding the number
  ///   of outstanding heartbeats.
  /// - Whenever an AppendEntries receives its majority of successful responses,
  ///   any buffered reads with a lower ReadID are now eligible to be served at
  ///   the read index they were queued with.
  /// - To avoid complexity in the replicated state machine implementation, we
  ///   hold the `ReadStateMachineReq` until the read index has been applied.
  ///   Further, we hold up applying any later indexes until the
  ///   `ReadStateMachineReq` finishes, similar to a "barrier". This means the
  ///   replicated state machine can blindly respond to any
  ///   `ReadStateMachineReq`s that it receives without worrying about the
  ///   indexes of what it's applied. (This assumes it has handled all
  ///   `PersistReq`s and `ApplyReq`s as is contractually required by `step`.)
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
        res.fill(Err(ClientError::NotLeaderError(NotLeaderError::new(Some(follower.leader_hint)))));
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
        // AppendEntries rpc from current leader or granting vote to candidate:
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
          // Leaders: Upon election: send initial empty AppendEntries rpcs
          // (heartbeat) to each server; repeat during idle periods to prevent
          // election timeouts (§5.2)
          leader = State::leader_heartbeat(leader, output)
        }
        State::Leader(leader)
      }
    }
  }

  fn persist_res(self, output: &mut impl Extend<Output>, res: PersistRes) -> State {
    let payload = PayloadShared::AppendEntriesRes(AppendEntriesResShared::new(
      self.shared().current_term,
      1, // WIP true
      res.log_index,
      res.read_id,
    ));
    let msg = MessageShared::new(self.id(), res.leader_id, payload);
    if msg.capnp_as_ref().src() == msg.capnp_as_ref().dest() {
      return State::step(self, output, Input::Message(msg.capnp_as_ref()));
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
      let term = match &message.payload().expect("WIP").expect("WIP") {
        Payload::AppendEntriesReq(req) => req.term(),
        Payload::AppendEntriesRes(res) => res.term(),
        Payload::RequestVoteReq(req) => req.term(),
        Payload::RequestVoteRes(res) => res.term(),
        Payload::StartElectionReq(req) => req.term(),
      };
      if term > shared.current_term {
        // All Servers: If rpc request or response contains term T >
        // currentTerm: set currentTerm = T, convert to follower (§5.1)
        shared.current_term = term;
        // TODO: probably want a helper for updating the term
        shared.voted_for = None;
        // TODO: do we really convert to follower on a RequestVoteReq with a
        // higher term?
        self = State::Follower(self.convert_to_follower(output, message.src()));
      }
    }
    match self {
      State::Candidate(candidate) => State::candidate_step(candidate, output, message),
      State::Follower(follower) => State::follower_step(follower, output, message),
      State::Leader(leader) => State::leader_step(leader, output, message),
    }
  }

  fn candidate_step<'i>(
    candidate: Candidate,
    output: &mut impl Extend<Output>,
    message: Message<'i>,
  ) -> State {
    match message.payload().expect("WIP").expect("WIP") {
      Payload::RequestVoteRes(res) => {
        State::candidate_process_request_vote_res(candidate, output, res)
      }
      Payload::AppendEntriesReq(req) => {
        if req.term() > candidate.shared.current_term {
          // Candidates (§5.2): If AppendEntries rpc received from new leader:
          // convert to follower
          let follower = State::candidate_convert_to_follower(candidate, output, message.src());
          return State::Follower(follower).step(output, Input::Message(message));
        }
        State::Candidate(candidate)
      }
      Payload::RequestVoteReq(req) => State::Candidate(candidate).process_request_vote(output, req),
      Payload::StartElectionReq(req) => {
        if req.term() < candidate.shared.current_term {
          // Stale request, ignore.
          return State::Candidate(candidate);
        }
        State::start_election(candidate, output)
      }
      payload => todo!("{:?}", payload),
    }
  }

  fn follower_step<'i>(
    follower: Follower,
    output: &mut impl Extend<Output>,
    message: Message<'i>,
  ) -> State {
    match message.payload().expect("WIP").expect("WIP") {
      // Followers (§5.2): Respond to rpcs from candidates and leaders
      Payload::AppendEntriesReq(req) => {
        State::Follower(State::follower_append_entries(follower, output, req))
      }
      Payload::RequestVoteReq(req) => State::Follower(follower).process_request_vote(output, req),
      Payload::AppendEntriesRes(_) => {
        // No-op, stale response to a request sent out by this node when it was
        // a leader. TODO: double check this
        State::Follower(follower)
      }
      Payload::StartElectionReq(req) => {
        if req.term() < follower.shared.current_term {
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

  fn leader_step<'i>(
    leader: Leader,
    output: &mut impl Extend<Output>,
    message: Message<'i>,
  ) -> State {
    match message.payload().expect("WIP").expect("WIP") {
      Payload::AppendEntriesRes(res) => {
        State::Leader(State::leader_append_entries_res(leader, output, message.src(), res))
      }
      Payload::RequestVoteRes(_) => {
        // Already the leader, nothing to do here.
        State::Leader(leader)
      }
      Payload::StartElectionReq(_) => {
        // Already the leader, nothing to do here.
        State::Leader(leader)
      }
      payload => todo!("{:?} {:?}", payload, leader.shared),
    }
  }

  fn follower_append_entries<'a>(
    mut follower: Follower,
    output: &'a mut impl Extend<Output>,
    req: AppendEntriesReq<'a>,
  ) -> Follower {
    // Reply false if term < currentTerm (§5.1)
    if req.term() < follower.shared.current_term {
      let payload = PayloadShared::AppendEntriesRes(AppendEntriesResShared::new(
        follower.shared.current_term,
        0, // WIP false
        Index(0),
        req.read_id(),
      ));
      let msg = MessageShared::new(follower.shared.id, req.leader_id(), payload);
      output.extend(vec![Output::Message(msg)]);
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
      .index_term(req.prev_log_index())
      .map_or(false, |term| term == req.prev_log_term());
    if !log_match {
      // TODO: send back a hint of what we do have
      let payload = PayloadShared::AppendEntriesRes(AppendEntriesResShared::new(
        follower.shared.current_term,
        0, // WIP false
        Index(0),
        req.read_id(),
      ));
      let msg = MessageShared::new(follower.shared.id, req.leader_id(), payload);
      output.extend(vec![Output::Message(msg)]);
      return follower;
    }

    // If an existing entry conflicts with a new one (same index but different
    // terms), delete the existing entry and all that follow it (§5.3). Append
    // any new entries not already in the log
    let entries: Slice<_> = req.entries().expect("WIP");
    if entries.len() > 0 {
      let entries = entries.into_iter().collect::<Vec<_>>();
      follower.shared.log.extend(&entries);
      let msg = PersistReq {
        leader_id: req.leader_id(),
        read_id: req.read_id(),
        entries: entries.iter().map(|e| e.capnp_to_owned()).collect(),
      };
      output.extend(vec![Output::PersistReq(msg)]);
    } else {
      // TODO: duplicated with persist_res
      let payload = PayloadShared::AppendEntriesRes(AppendEntriesResShared::new(
        follower.shared.current_term,
        1,                    // WIP true
        req.prev_log_index(), // TODO: is this right?
        req.read_id(),
      ));
      let msg = MessageShared::new(follower.shared.id, req.leader_id(), payload);
      output.extend(vec![Output::Message(msg)]);
    }

    // If leaderCommit > commitIndex, set commitIndex = min(leaderCommit, index
    // of last new entry)
    if req.leader_commit() > follower.shared.commit_index {
      let last_entry_index = follower.shared.log.last().1;
      follower.shared.commit_index = cmp::min(req.leader_commit(), last_entry_index);
      follower = State::follower_maybe_apply(follower, output);
    }
    follower
  }

  fn leader_append_entries_res<'a>(
    leader: Leader,
    output: &'a mut impl Extend<Output>,
    src: NodeID,
    res: AppendEntriesRes<'a>,
  ) -> Leader {
    // If successful: update nextIndex and matchIndex for follower (§5.3)
    if res.success() > 0 {
      return State::ack_term_index(leader, output, src, res.index(), res.read_id());
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
      // TODO: debug_assert that new_max_confirmed_read_id has a majority and
      // that it's the highest read_id with a majority
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
        "  {:3}: is committed? index={:} current_term={:} commit_index={:}",
        leader.shared.id.0,
        entry_index.0,
        leader.shared.current_term.0,
        leader.shared.commit_index.0,
      );
      if entry_index <= leader.shared.commit_index {
        break;
      }
      // TODO: inefficient; instead, compute once the min index that has a
      // majority in match_index
      let count = leader.match_index.iter().filter(|(_, (index, _))| *index >= entry_index).count();
      if count >= needed {
        let new_commit_index = entry_index;
        debug!("  {:3}: new_commit_index={:?}", leader.shared.id.0, new_commit_index);
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

  fn process_request_vote<'a>(
    mut self,
    output: &'a mut impl Extend<Output>,
    req: RequestVoteReq<'a>,
  ) -> State {
    // TODO: To prevent [disruption from removed nodes], servers disregard
    // RequestVote rpcs when they believe a current leader exists. Specifically,
    // if a server receives a RequestVote rpc within the minimum election
    // timeout of hearing from a current leader, it does not update its term or
    // grant its vote. (§6)

    let mut shared = self.shared_mut();
    debug!(
      "  {:3}: self.process_request_vote voted_for={:?} req={:?}",
      shared.id.0, shared.voted_for, req
    );
    // Reply false if term < currentTerm (§5.1)
    if req.term() < shared.current_term {
      let payload =
        PayloadShared::RequestVoteRes(RequestVoteResShared::new(shared.current_term, 0));
      let msg = MessageShared::new(self.id(), req.candidate_id(), payload);
      output.extend(vec![Output::Message(msg)]);
      return self;
    }
    // If votedFor is null or candidateId, and candidate’s log is at least as
    // up-to-date as receiver’s log, grant vote (§5.2, §5.4)
    let should_grant = match shared.voted_for {
      None => true,
      Some(voted_for) => voted_for == req.candidate_id(),
    };
    if should_grant {
      shared.voted_for = Some(req.candidate_id());
      let payload =
        PayloadShared::RequestVoteRes(RequestVoteResShared::new(shared.current_term, 1));
      let msg = MessageShared::new(shared.id, req.candidate_id(), payload);
      output.extend(vec![Output::Message(msg)]);
    }
    self
  }

  fn candidate_process_request_vote_res<'a>(
    mut candidate: Candidate,
    output: &'a mut impl Extend<Output>,
    res: RequestVoteRes<'a>,
  ) -> State {
    // NB: The term was checked earlier so don't need to check it again.
    if res.vote_granted() > 0 {
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
    // Send RequestVote rpcs to all other servers
    let (last_log_term, last_log_index) = candidate.shared.log.last();
    let payload = PayloadShared::RequestVoteReq(RequestVoteReqShared::new(
      candidate.shared.current_term,
      candidate.shared.id,
      last_log_index,
      last_log_term,
    ));
    debug!("  {:3}: reqvote {:?}", candidate.shared.id.0, payload);
    State::message_to_all_other_nodes(&candidate.shared, output, payload);
    // Vote for self
    let res = RequestVoteResShared::new(candidate.shared.current_term, 1);
    return State::candidate_process_request_vote_res(candidate, output, res.capnp_as_ref());
  }

  fn message_to_all_other_nodes<'a>(
    shared: &'a SharedState,
    output: &'a mut impl Extend<Output>,
    payload: PayloadShared,
  ) {
    output.extend(
      shared
        .peers
        .iter()
        .filter(|peer| **peer != shared.id)
        .map(|node| Output::Message(MessageShared::new(shared.id, *node, payload.clone()))),
    )
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
    Follower { shared: candidate.shared, leader_hint: new_leader_hint }
  }

  fn leader_convert_to_follower(
    mut leader: Leader,
    _output: &mut impl Extend<Output>,
    new_leader_hint: NodeID,
  ) -> Follower {
    debug!("  {:3}: convert_to_follower leader={:?}", leader.shared.id.0, new_leader_hint.0);
    leader = State::clear_outstanding_requests(leader, Some(new_leader_hint));
    Follower { shared: leader.shared, leader_hint: new_leader_hint }
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
    // Leaders: Upon election: send initial empty AppendEntries rpcs
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
    (shared.peers.len() + 1) / 2
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
