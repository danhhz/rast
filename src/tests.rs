// Copyright 2020 Daniel Harrison. All Rights Reserved.

use super::testgroup::*;
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

fn wait(f: &mut WriteFuture) -> WriteRes {
  loop {
    match poll(f) {
      Poll::Pending => {} // No-op
      Poll::Ready(res) => return res,
    }
  }
}

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
  let mut g = TestGroup1::new();
  g.n.tick(g.cfg().election_timeout);

  let req = WriteReq { payload: vec![] };
  let (mut res, _output) = g.n.sm.write_async(req);
  assert_eq!(poll(&mut res), Poll::Pending);

  g.drain();
  assert_eq!(wait(&mut res), WriteRes { term: Term(0), index: Index(0) });
}
