// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::time::Duration;

use crate::prelude::*;
use crate::testutil;
use crate::testutil::{noopfuture, DeterministicGroup, DeterministicGroup1, DeterministicGroup3};

#[test]
fn election_one() {
  testutil::log_init();

  let mut g = DeterministicGroup1::new();
  assert_eq!(g.n.raft.debug(), "candidate");

  g.n.start_election();
  // TODO: this isn't accurate once we persist hard state
  assert_eq!(g.n.raft.debug(), "leader");
}

#[test]
fn election_multi() {
  testutil::log_init();

  let mut g = DeterministicGroup3::new();
  assert_eq!(g.n0.raft.debug(), "candidate");

  g.n0.start_election();
  assert_eq!(g.n0.raft.debug(), "candidate");
  assert_eq!(g.n1.raft.debug(), "candidate");
  assert_eq!(g.n2.raft.debug(), "candidate");

  g.drain();
  assert_eq!(g.n0.raft.debug(), "leader");
  assert_eq!(g.n1.raft.debug(), "follower");
  assert_eq!(g.n2.raft.debug(), "follower");
}

#[test]
fn tick() {
  testutil::log_init();

  let mut g = DeterministicGroup3::new();

  // all nodes call an election on startup
  g.n0.tick(Duration::from_nanos(0));
  g.n1.tick(Duration::from_nanos(0));
  g.n2.tick(Duration::from_nanos(0));
  assert_eq!(g.n0.raft.current_term(), Term(1));

  // Nothing happens for election_timeout, so n0 calls a fresh election with a
  // new term.
  g.n0.tick(g.cfg().election_timeout);
  assert_eq!(g.n0.raft.current_term(), Term(2));

  // This time it works.
  g.drain();
  assert_eq!(g.n0.raft.debug(), "leader");
  assert_eq!(g.n1.raft.debug(), "follower");
  assert_eq!(g.n2.raft.debug(), "follower");

  // Once the heartbeat interval has elapsed, the leader sends out a heartbeat.
  // The followers do nothing.
  g.n0.tick(g.cfg().heartbeat_interval);
  g.n1.tick(g.cfg().heartbeat_interval);
  g.n2.tick(g.cfg().heartbeat_interval);
  g.drain();
  assert_eq!(g.n0.raft.debug(), "leader");
  assert_eq!(g.n1.raft.debug(), "follower");
  assert_eq!(g.n2.raft.debug(), "follower");

  // If the leader doesn't heartbeat for the timeout interval, an election is
  // called.
  g.n1.tick(g.cfg().election_timeout);
  g.drain();
  assert_eq!(g.n0.raft.debug(), "follower");
  assert_eq!(g.n1.raft.debug(), "leader");
  assert_eq!(g.n2.raft.debug(), "follower");
}

#[test]
fn write_future() {
  testutil::log_init();

  let mut g = DeterministicGroup3::new();
  g.n0.start_election();
  g.drain();

  let payload = String::from("write_future").into_bytes();
  let mut res = g.n0.write(WriteReq { payload: payload.clone() });
  noopfuture::assert_pending(&mut res);

  g.drain();
  let res = noopfuture::assert_ready(&mut res).unwrap();
  // TODO: don't assume that the leader has it synced, it's possible for the
  // majority to be all followers
  assert_eq!(g.n0.log.get(res.index), Some(&payload));
}

#[test]
fn read_future() {
  testutil::log_init();

  // TODO:
  // - read kicks off append entries so it can be resolved immediately
  // - two reads while there are no outstanding append entries will batch
  // - read during each state transition pair, confirmed and non-confirmed

  let mut g = DeterministicGroup3::new();
  g.n0.start_election();
  g.drain();

  let payload = String::from("read_future").into_bytes();
  g.n0.write(WriteReq { payload: payload.clone() });
  let mut read = g.n0.read(ReadReq { payload: vec![] });
  noopfuture::assert_pending(&mut read);

  g.drain();
  let read = noopfuture::assert_ready(&mut read).unwrap();
  assert_eq!(read.payload, payload);
}

#[test]
fn leader_timeout() {
  testutil::log_init();

  let mut g = DeterministicGroup3::new();
  g.n0.start_election();
  g.drain();
  assert_eq!(g.n0.raft.debug(), "leader");

  // A write is sent to n0 while it's the leader.
  let payload = String::from("leader_timeout").into_bytes();
  let req = WriteReq { payload: payload };
  let mut res = g.n0.write(req);

  // n1 doesn't see a heartbeat from n0 for too long and calls an election.
  g.n1.tick(g.cfg().election_timeout * 2);
  g.drain();
  assert_eq!(g.n1.raft.debug(), "leader");

  // The n0 write should have errored.
  assert_eq!(
    noopfuture::assert_ready(&mut res),
    Err(ClientError::NotLeaderError(NotLeaderError::new(Some(g.n1.raft.id()))))
  );
}

#[test]
fn overwrite_entries() {
  testutil::log_init();

  let mut g = DeterministicGroup3::new();
  g.n0.start_election();
  g.drain();
  assert_eq!(g.n0.raft.debug(), "leader");

  // A write is committed with n0 as leader.
  let mut res = g.n0.write(WriteReq { payload: String::from("1").into_bytes() });
  g.drain();
  let _ = noopfuture::assert_ready(&mut res);

  // Another write is started, but this one will not finish.
  g.n0.write(WriteReq { payload: String::from("2").into_bytes() });

  // n1 is elected as the new leader.
  g.n1.start_election();
  g.drain();
  assert_eq!(g.n1.raft.debug(), "leader");

  // A write is committed with n1 as leader.
  let mut res = g.n1.write(WriteReq { payload: String::from("3").into_bytes() });
  g.drain();
  let _ = noopfuture::assert_ready(&mut res);

  // Leadership is transferred back to n0.
  g.n0.start_election();
  g.drain();
  assert_eq!(g.n0.raft.debug(), "leader");

  println!("\n\nWIP\n\n");

  // A read on n1 shouldn't have the unfinished write.
  let mut res = g.n0.read(ReadReq { payload: vec![] });
  g.drain();
  let res = noopfuture::assert_ready(&mut res).unwrap();
  assert_eq!(res.payload, String::from("13").into_bytes());
}

#[test]
fn regression_request_starts_election() {
  testutil::log_init();

  // Regression test for a bug where a write request didn't start an election
  // (only a tick would).
  {
    let mut g = DeterministicGroup3::new();
    // Request fails with NotLeaderError, but kicks off an election.
    let mut res1 = g.n0.write(WriteReq { payload: String::from("1").into_bytes() });
    assert_eq!(
      noopfuture::assert_ready(&mut res1),
      Err(ClientError::NotLeaderError(NotLeaderError::new(Some(g.n0.raft.id()))))
    );
    g.drain();
    assert_eq!(g.n0.raft.debug(), "leader");
  }

  // Same thing but for a 1 node cluster.
  {
    let mut g = DeterministicGroup1::new();
    // Request fails with NotLeaderError, but kicks off an election.
    let mut res1 = g.n.write(WriteReq { payload: String::from("1").into_bytes() });
    noopfuture::assert_pending(&mut res1);
    g.drain();
    assert_eq!(g.n.raft.debug(), "leader");
    assert_eq!(
      noopfuture::assert_ready(&mut res1),
      Ok(WriteRes { term: Term(1), index: Index(1) }),
    );
  }

  // Same thing but for a read request.
  {
    let mut g = DeterministicGroup3::new();
    // Request fails with NotLeaderError, but kicks off an election.
    let mut res1 = g.n0.read(ReadReq { payload: String::from("1").into_bytes() });
    assert_eq!(
      noopfuture::assert_ready(&mut res1),
      Err(ClientError::NotLeaderError(NotLeaderError::new(Some(g.n0.raft.id()))))
    );
    g.drain();
    assert_eq!(g.n0.raft.debug(), "leader");
  }

  // Same thing but for a 1 node cluster.
  {
    let mut g = DeterministicGroup1::new();
    // Request fails with NotLeaderError, but kicks off an election.
    let mut res1 = g.n.read(ReadReq { payload: String::from("1").into_bytes() });
    noopfuture::assert_pending(&mut res1);
    g.drain();
    assert_eq!(g.n.raft.debug(), "leader");
    assert_eq!(
      noopfuture::assert_ready(&mut res1),
      Ok(ReadRes { term: Term(1), index: Index(0), payload: vec![] })
    );
  }
}

/// Regression test for a bug where a write sent to a follower would panic.
#[test]
fn regression_follower_write() {
  testutil::log_init();

  let mut g = DeterministicGroup3::new();
  g.n0.start_election();
  g.drain();

  // A write is sent to a follower. This used to panic.
  assert_eq!(g.n1.raft.debug(), "follower");
  g.n1.write(WriteReq { payload: String::from("1").into_bytes() });
}
