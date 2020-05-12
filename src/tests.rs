// Copyright 2020 Daniel Harrison. All Rights Reserved.

use super::testgroup::*;
use super::*;

mod future {
  use super::*;
  use std::boxed::Box;
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

  fn poll(f: &mut WriteFuture) -> Poll<WriteRes> {
    let pinned: Box<Pin<_>> = Box::new(Pin::new(f));
    let waker = noop_waker();
    let mut context = Context::from_waker(&waker);
    pinned.poll(&mut context)
  }

  pub fn assert_pending(f: &mut WriteFuture) {
    match poll(f) {
      Poll::Pending => {} // No-op
      Poll::Ready(res) => panic!("unexpectedly ready: {:?}", res),
    }
  }

  pub fn assert_ready(f: &mut WriteFuture) -> WriteRes {
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
  let req = WriteReq { payload: payload.clone() };
  let mut res = g.n0.write_async(req);
  assert_pending(&mut res);

  g.drain();
  let res = assert_ready(&mut res);
  // TODO: don't assume that the leader has it synced, it's possible for the
  // majority to be all followers
  assert_eq!(g.n0.log.get(res.index), Some(&payload));
}

#[test]
fn leader_timeout() {
  let mut g = TestGroup3::new();
  g.n0.tick(g.cfg().election_timeout);
  g.drain();
  assert_eq!(g.n0.sm.role, Role::Leader);

  // n1 doesn't see a heartbeat from n0 for too long and calls an election
  g.n1.tick(g.cfg().election_timeout * 2);
  g.drain();
  assert_eq!(g.n1.sm.role, Role::Leader);
}
