// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::cmp;
use std::time::{Duration,Instant};

mod command;
pub use command::{Command,Response};

pub mod log;
use log::*;

pub struct RastClient {
  _r: Rast,
}

impl RastClient {
  pub fn run(&self, c: Command) -> Response {
    Response::with_payload(c.payload())
  }
}

// WIP: Keep the role-specific state in this enum.
enum Role {
  Candidate,
  Follower,
  Leader,
}

pub struct Config {
  pub election_timeout: Duration,
  pub heartbeat_interval: Duration,
}

pub struct Rast {
  config: Config,
  id: Node,
  role: Role,

  // Persistent state
  current_term: Term,
  voted_for: Option<Node>,
  log: Vec<Entry>,

  // Volatile state
  commit_index: Index,
  last_applied: Index,
  nodes: Vec<Node>, // WIP: Double check this doesn't need to be persisted
  current_time: Instant,
  // WIP this is overloaded fixme
  last_communication: Instant,

  // Leader volatile state
  next_index: Vec<Index>,
  match_index: Vec<Index>,

  // Candidate volatile state
  received_votes: usize,
}

impl Rast {
  pub fn new(config: Config, current_time: Instant) -> Rast {
    // WIP: hack to initialize last_communication such that an election is
    // immediately called
    let last_communication = current_time - config.election_timeout;
    // TODO: Immediately call an election and return the output?
    Rast{
      id: Node(0), // WIP
      config: config,
      role: Role::Candidate,
      current_term: Term(0),
      voted_for: None,
      log: vec![],
      commit_index: Index(0),
      last_applied: Index(0),
      nodes: vec![],
      current_time: current_time,
      last_communication: last_communication,
      next_index: vec![],
      match_index: vec![],
      received_votes: 0,
    }
  }

  pub fn client(&self) -> RastClient {
    unimplemented!()
  }

  pub fn step(&mut self, input: Input) -> Vec<Output> {
    let mut output = vec![];
    if self.commit_index > self.last_applied {
      // All Servers: If commitIndex > lastApplied: increment lastApplied, apply
      // log[lastApplied] to state machine (§5.3)
      let Index(last_applied) = self.last_applied;
      self.last_applied = Index(last_applied + 1);
      output.push(Output::Apply(self.last_applied));
    }
    output.extend(match input {
      Input::Tick(now) => self.tick(now),
      Input::Message(message) => self.message(message),
    });
    output
  }

  fn tick(&mut self, now: Instant) -> Vec<Output> {
    if now <= self.current_time {
      // Ignore a repeat tick (as well as one in the past, which shouldn't
      // happen).
      return vec![]
    }
    self.current_time = now;
    match self.role {
      Role::Candidate => {
        // Candidates (§5.2): If election timeout elapses: start new election
        if now.duration_since(self.last_communication) > self.config.election_timeout {
          return self.start_election(now)
        }
        return vec![]
      },
      Role::Follower => {
        // Followers (§5.2): If election timeout elapses without receiving
        // AppendEntries RPC from current leader or granting vote to candidate:
        // convert to candidate
        if now.duration_since(self.last_communication) > self.config.election_timeout {
          return self.convert_to_candidate(now)
        }
        return vec![]
      },
      Role::Leader => {
        if now.duration_since(self.last_communication) > self.config.heartbeat_interval {
          // Leaders: Upon election: send initial empty AppendEntries RPCs
          // (heartbeat) to each server; repeat during idle periods to prevent
          // election timeouts (§5.2)
          return self.send_append_entries(vec![])
        }
        return vec![]
      },
    }
  }

  fn message(&mut self, message: Message) -> Vec<Output> {
    // TODO: avoid calling payload multiple times
    let term = match message.payload() {
      Payload::AppendEntriesReq(req) => req.term,
      Payload::RequestVoteReq(req) => req.term,
      Payload::AppendEntriesRes(res) => res.term,
      Payload::RequestVoteRes(res) => res.term,
    };
    if term > self.current_term {
      // All Servers: If RPC request or response contains term T > currentTerm:
      // set currentTerm = T, convert to follower (§5.1)
      self.current_term = term;
      return self.convert_to_follower()
    }
    match &mut self.role {
      Role::Candidate => self.step_candidate(message),
      Role::Follower => self.step_follower(message),
      Role::Leader => self.step_leader(message),
    }
  }

  fn step_candidate(&mut self, message: Message) -> Vec<Output> {
    match message.payload() {
      Payload::RequestVoteRes(res) => {
        // NB: The term was checked earlier so don't need to check it again.
        debug_assert!(res.term == self.current_term);
        if res.vote_granted {
          self.received_votes += 1;
          let needed_votes = (self.nodes.len() +1) / 2;
          if self.received_votes >= needed_votes {
            // Candidates (§5.2): If votes received from majority of servers:
            // become leader
            return self.convert_to_leader()
          }
        }
        return vec![]
      }
      Payload::AppendEntriesReq(req) => {
        if req.term > self.current_term {
          // Candidates (§5.2): If AppendEntries RPC received from new leader:
          // convert to follower
          let mut output = self.convert_to_follower();
          // WIP: This is awkward
          output.extend(self.step(Input::Message(message)));
          return output
        }
        return vec![]
      }
      _ => vec![] // WIP no-op
    }
  }

  fn step_follower(&mut self, message: Message) -> Vec<Output> {
    match message.payload() {
      // Followers (§5.2): Respond to RPCs from candidates and leaders
      Payload::AppendEntriesReq(req) => self.process_append_entries(req),
      Payload::RequestVoteReq(req) => self.process_request_vote(req),
      _ => vec![] // WIP no-op
    }
  }

  fn step_leader(&mut self, _message: Message) -> Vec<Output> {
    unimplemented!()
  }

  fn process_append_entries(&mut self, req: AppendEntriesReq) -> Vec<Output> {
    // Reply false if term < currentTerm (§5.1)
    if req.term < self.current_term {
      let msg = Message::append_entries_res(self.current_term, false);
      let mut output = vec![Output::Message(msg)];
      output.extend(self.convert_to_follower());
      return output
    }

    // WIP: Implement the real logic
    self.log.extend(req.entries);

    // WIP: Reply false if log doesn’t contain an entry at prevLogIndex whose
    // term matches prevLogTerm (§5.3)

    // WIP: If an existing entry conflicts with a new one (same index but
    // different terms), delete the existing entry and all that follow it (§5.3)

    // WIP: Append any new entries not already in the log

    // If leaderCommit > commitIndex, set commitIndex = min(leaderCommit, index
    // of last new entry)
    if req.leader_commit > self.commit_index {
      let last_entry_index = self.log.last().map_or(Index(0), |entry| entry.index());
      self.commit_index = cmp::min(req.leader_commit, last_entry_index)
    }

    vec![]
  }

  fn process_request_vote(&mut self, req: RequestVoteReq) -> Vec<Output> {
    // Reply false if term < currentTerm (§5.1)
    if req.term < self.current_term {
      let msg = Message::request_vote_res(self.current_term, false);
      return vec![Output::Message(msg)]
    }
    // If votedFor is null or candidateId, and candidate’s log is at least as
    // up-to-date as receiver’s log, grant vote (§5.2, §5.4)
    let should_grant = match self.voted_for {
      None => false,
      Some(voted_for) => voted_for == req.candidate_id,
    };
    if should_grant {
      // volatile.current_time = self.volatile.current_time;
      self.voted_for = Some(req.candidate_id);
      let msg = Message::request_vote_res(req.term, true);
      return vec![Output::Message(msg)]
    }
    return vec![]
  }

  fn send_append_entries(&mut self, _entries: Vec<Entry>) -> Vec<Output> {
    unimplemented!()
  }

  fn start_election(&mut self, now: Instant) -> Vec<Output> {
    // Increment currentTerm
    let Term(current_term) = self.current_term;
    self.current_term = Term(current_term + 1);
    // Vote for self
    self.received_votes = 1;
    // Reset election timer
    self.last_communication = now;
    // Send RequestVote RPCs to all other servers
    let id = self.id.clone(); // WIP: hacks
    let (last_log_index, last_log_term) = self.log.last().map_or((Index(0), Term(0)), |entry| {
      (entry.index(), entry.term())
    });
    let output = self.nodes.iter().flat_map(|node| {
      if node == &id {
        return None
      }
      let msg = Message::request_vote_req(self.current_term, self.id, last_log_index, last_log_term);
      Some(Output::Message(msg))
    }).collect();
    output
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
    self.next_index.truncate(0);
    self.match_index.truncate(0);
    // Leaders: Upon election: send initial empty AppendEntries RPCs
    // (heartbeat) to each server; repeat during idle periods to prevent
    // election timeouts (§5.2)
    vec![]
  }
}

pub enum Input {
  Message(Message),
  Tick(Instant),
}

pub enum Output {
  Message(Message),
  Apply(Index),
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works() {
    let cfg = Config{
      election_timeout: 100 * Duration::MILLISECOND,
      heartbeat_interval: 10 * Duration::MILLISECOND,
    };
    let r = RastClient::new(cfg, Time::now());
    let payload = vec![];
    let res = r.run(Command::with_payload(&payload));
    assert_eq!(res.payload(), payload.as_slice());
  }
}
