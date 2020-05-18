// Copyright 2020 Daniel Harrison. All Rights Reserved.

use super::nemesis;
use super::testgroup::*;
use super::*;

mod future {
  use super::*;
  use std::boxed::Box;
  use std::fmt::Debug;
  use std::future::Future;
  use std::pin::Pin;
  use std::task::{Context, Poll, Waker};
  use std::task::{RawWaker, RawWakerVTable};

  fn noop_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
      noop_raw_waker()
    }
    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(0 as *const (), vtable)
  }

  fn noop_waker() -> Waker {
    unsafe { Waker::from_raw(noop_raw_waker()) }
  }

  fn poll<T: Clone>(f: &mut RastFuture<T>) -> Poll<Result<T, NotLeaderError>> {
    let pinned: Box<Pin<_>> = Box::new(Pin::new(f));
    let waker = noop_waker();
    let mut context = Context::from_waker(&waker);
    pinned.poll(&mut context)
  }

  pub fn assert_pending<T: Clone + Debug>(f: &mut RastFuture<T>) {
    match poll(f) {
      Poll::Pending => {} // No-op
      Poll::Ready(res) => panic!("unexpectedly ready: {:?}", res),
    }
  }

  pub fn assert_ready<T: Clone>(f: &mut RastFuture<T>) -> Result<T, NotLeaderError> {
    match poll(f) {
      Poll::Pending => panic!("unexpectedly not ready: {:?}"),
      Poll::Ready(res) => res,
    }
  }
}
use future::*;

// TODO: Tests
// - election timeout, node isn't elected in a short enough time
// - stuck election, all nodes vote for themselves
// - election completes with majority but not all nodes
// - expand this list with examples from the raft paper

#[test]
fn election_one() {
  let mut g = TestGroup1::new();
  assert_eq!(g.n.sm.role, Role::Candidate);

  g.n.tick(g.cfg().election_timeout);
  assert_eq!(g.n.sm.role, Role::Leader);
}

#[test]
fn election_multi() {
  let mut g = TestGroup3::new();
  assert_eq!(g.n0.sm.role, Role::Candidate);

  g.n0.tick(g.cfg().election_timeout);
  assert_eq!(g.n0.sm.role, Role::Candidate);
  assert_eq!(g.n1.sm.role, Role::Candidate);
  assert_eq!(g.n2.sm.role, Role::Candidate);

  g.drain();
  assert_eq!(g.n0.sm.role, Role::Leader);
  assert_eq!(g.n1.sm.role, Role::Follower);
  assert_eq!(g.n2.sm.role, Role::Follower);
}

#[test]
fn write_future() {
  let mut g = TestGroup3::new();
  g.n0.tick(g.cfg().election_timeout);
  g.drain();

  let payload = String::from("write_future").into_bytes();
  let mut res = g.n0.write_async(WriteReq { payload: payload.clone() });
  assert_pending(&mut res);

  g.drain();
  let res = assert_ready(&mut res).unwrap();
  // TODO: don't assume that the leader has it synced, it's possible for the
  // majority to be all followers
  assert_eq!(g.n0.log.get(res.index), Some(&payload));
}

#[test]
fn read_future() {
  let mut g = TestGroup3::new();
  g.n0.tick(g.cfg().election_timeout);
  g.drain();

  let payload = String::from("read_future").into_bytes();
  g.n0.write_async(WriteReq { payload: payload.clone() });
  let mut read = g.n0.read_async(ReadReq { payload: vec![] });
  assert_pending(&mut read);

  g.drain();
  let read = assert_ready(&mut read).unwrap();
  assert_eq!(read.payload, payload);
}

#[test]
fn leader_timeout() {
  let mut g = TestGroup3::new();
  g.n0.tick(g.cfg().election_timeout);
  g.drain();
  assert_eq!(g.n0.sm.role, Role::Leader);

  // A write is sent to n0 while it's the leader.
  let payload = String::from("leader_timeout").into_bytes();
  let req = WriteReq { payload: payload };
  let mut res = g.n0.write_async(req);

  // n1 doesn't see a heartbeat from n0 for too long and calls an election.
  g.n1.tick(g.cfg().election_timeout * 2);
  g.drain();
  assert_eq!(g.n1.sm.role, Role::Leader);

  // The n0 write should have errored.
  assert_eq!(assert_ready(&mut res), Err(NotLeaderError::new(g.n1.sm.id)));
}

#[test]
fn overwrite_entries() {
  let mut g = TestGroup3::new();
  g.n0.tick(g.cfg().election_timeout);
  g.drain();
  assert_eq!(g.n0.sm.role, Role::Leader);

  // A write is committed with n0 as leader.
  let mut res = g.n0.write_async(WriteReq { payload: String::from("1").into_bytes() });
  g.drain();
  let _ = assert_ready(&mut res);

  // Another write is started, but this one will not finish.
  g.n0.write_async(WriteReq { payload: String::from("2").into_bytes() });

  // n1 doesn't see a heartbeat from n0 for too long and calls an election.
  g.n1.tick(g.cfg().election_timeout * 2);
  g.drain();
  assert_eq!(g.n1.sm.role, Role::Leader);

  // A write is committed with n1 as leader.
  let mut res = g.n1.write_async(WriteReq { payload: String::from("3").into_bytes() });
  g.drain();
  let _ = assert_ready(&mut res);

  // n0 doesn't see a heartbeat from n1 for too long and calls an election.
  g.n0.tick(g.cfg().election_timeout * 2);
  g.drain();
  assert_eq!(g.n0.sm.role, Role::Leader);

  // A read on n1 shouldn't have the unfinished write.
  // TODO: make this work without the write
  let _ = g.n0.write_async(WriteReq { payload: String::from("4").into_bytes() });
  let mut res = g.n0.read_async(ReadReq { payload: vec![] });
  g.n0.tick(g.cfg().election_timeout);
  g.drain();
  let res = assert_ready(&mut res).unwrap();
  assert_eq!(res.payload, String::from("134").into_bytes());
}

#[test]
fn regression_request_starts_election() {
  // Regression test for a bug where a write request didn't start an election
  // (only a tick would).
  {
    let mut g = TestGroup3::new();
    // Request fails with NotLeaderError, but kicks off an election.
    let mut res1 = g.n0.write_async(WriteReq { payload: String::from("1").into_bytes() });
    assert_eq!(assert_ready(&mut res1), Err(NotLeaderError::new(NodeID(0))));
    g.drain();
    assert_eq!(g.n0.sm.role, Role::Leader);
  }

  // Same thing but for a read request.
  {
    let mut g = TestGroup3::new();
    // Request fails with NotLeaderError, but kicks off an election.
    let mut res1 = g.n0.read_async(ReadReq { payload: String::from("1").into_bytes() });
    assert_eq!(assert_ready(&mut res1), Err(NotLeaderError::new(NodeID(0))));
    g.drain();
    assert_eq!(g.n0.sm.role, Role::Leader);
  }
}

#[test]
fn nemesis_single() {
  let cfg = nemesis::Config { nodes: 1, workers: 1, ops: 100, read: 50, write: 50 };
  let failures = nemesis::nemesis_test(cfg);
  failures.expect("consistency violation");
}
