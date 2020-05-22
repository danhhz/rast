// Copyright 2020 Daniel Harrison. All Rights Reserved.

use crate::prelude::*;
use crate::testutil::noopfuture;
use crate::testutil::{DeterministicGroup, DeterministicGroup1, DeterministicGroup3};

#[test]
fn election_one() {
  let mut g = DeterministicGroup1::new();
  assert_eq!(g.n.raft.debug(), "candidate");

  g.n.tick(g.cfg().election_timeout);
  // TODO: this isn't accurace once we persist hard state
  // assert_eq!(g.n.raft.debug(), "leader");
}

#[test]
fn election_multi() {
  let mut g = DeterministicGroup3::new();
  assert_eq!(g.n0.raft.debug(), "candidate");

  g.n0.tick(g.cfg().election_timeout);
  assert_eq!(g.n0.raft.debug(), "candidate");
  assert_eq!(g.n1.raft.debug(), "candidate");
  assert_eq!(g.n2.raft.debug(), "candidate");

  g.drain();
  assert_eq!(g.n0.raft.debug(), "leader");
  assert_eq!(g.n1.raft.debug(), "follower");
  assert_eq!(g.n2.raft.debug(), "follower");
}

#[test]
fn write_future() {
  let mut g = DeterministicGroup3::new();
  g.n0.tick(g.cfg().election_timeout);
  g.drain();

  let payload = String::from("write_future").into_bytes();
  let mut res = g.n0.write_async(WriteReq { payload: payload.clone() });
  noopfuture::assert_pending(&mut res);

  g.drain();
  let res = noopfuture::assert_ready(&mut res).unwrap();
  // TODO: don't assume that the leader has it synced, it's possible for the
  // majority to be all followers
  assert_eq!(g.n0.log.get(res.index), Some(&payload));
}

#[test]
fn read_future() {
  let mut g = DeterministicGroup3::new();
  g.n0.tick(g.cfg().election_timeout);
  g.drain();

  let payload = String::from("read_future").into_bytes();
  g.n0.write_async(WriteReq { payload: payload.clone() });
  let mut read = g.n0.read_async(ReadReq { payload: vec![] });
  noopfuture::assert_pending(&mut read);

  g.drain();
  let read = noopfuture::assert_ready(&mut read).unwrap();
  assert_eq!(read.payload, payload);
}

#[test]
fn leader_timeout() {
  let mut g = DeterministicGroup3::new();
  g.n0.tick(g.cfg().election_timeout);
  g.drain();
  assert_eq!(g.n0.raft.debug(), "leader");

  // A write is sent to n0 while it's the leader.
  let payload = String::from("leader_timeout").into_bytes();
  let req = WriteReq { payload: payload };
  let mut res = g.n0.write_async(req);

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
  // TODO: this test is designed to check that the a follower truncate its log
  // correctly, but it doesn't.

  let mut g = DeterministicGroup3::new();
  g.n0.tick(g.cfg().election_timeout);
  g.drain();
  assert_eq!(g.n0.raft.debug(), "leader");

  // A write is committed with n0 as leader.
  let mut res = g.n0.write_async(WriteReq { payload: String::from("1").into_bytes() });
  g.drain();
  let _ = noopfuture::assert_ready(&mut res);

  // Another write is started, but this one will not finish.
  g.n0.write_async(WriteReq { payload: String::from("2").into_bytes() });

  // n1 doesn't see a heartbeat from n0 for too long and calls an election.
  g.n1.tick(g.cfg().election_timeout * 2);
  g.drain();
  assert_eq!(g.n1.raft.debug(), "leader");

  // A write is committed with n1 as leader.
  let mut res = g.n1.write_async(WriteReq { payload: String::from("3").into_bytes() });
  g.drain();
  let _ = noopfuture::assert_ready(&mut res);

  // n0 doesn't see a heartbeat from n1 for too long and calls an election.
  g.n0.tick(g.cfg().election_timeout * 2);
  g.drain();
  assert_eq!(g.n0.raft.debug(), "leader");

  // A read on n1 shouldn't have the unfinished write.
  // TODO: make this work without the write
  let _ = g.n0.write_async(WriteReq { payload: String::from("4").into_bytes() });
  let mut res = g.n0.read_async(ReadReq { payload: vec![] });
  g.n0.tick(g.cfg().election_timeout);
  g.drain();
  let res = noopfuture::assert_ready(&mut res).unwrap();
  assert_eq!(res.payload, String::from("134").into_bytes());
}

#[test]
fn regression_request_starts_election() {
  // Regression test for a bug where a write request didn't start an election
  // (only a tick would).
  {
    let mut g = DeterministicGroup3::new();
    // Request fails with NotLeaderError, but kicks off an election.
    let mut res1 = g.n0.write_async(WriteReq { payload: String::from("1").into_bytes() });
    assert_eq!(
      noopfuture::assert_ready(&mut res1),
      Err(ClientError::NotLeaderError(NotLeaderError::new(Some(g.n0.raft.id()))))
    );
    g.drain();
    assert_eq!(g.n0.raft.debug(), "leader");
  }

  // Same thing but for a read request.
  {
    let mut g = DeterministicGroup3::new();
    // Request fails with NotLeaderError, but kicks off an election.
    let mut res1 = g.n0.read_async(ReadReq { payload: String::from("1").into_bytes() });
    assert_eq!(
      noopfuture::assert_ready(&mut res1),
      Err(ClientError::NotLeaderError(NotLeaderError::new(Some(g.n0.raft.id()))))
    );
    g.drain();
    assert_eq!(g.n0.raft.debug(), "leader");
  }
}
