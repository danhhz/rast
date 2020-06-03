// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::boxed::Box;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};
use std::task::{RawWaker, RawWakerVTable};

use crate::error::ClientError;

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

fn poll<T>(
  f: &mut (impl Unpin + Future<Output = Result<T, ClientError>>),
) -> Poll<Result<T, ClientError>> {
  let pinned: Box<Pin<_>> = Box::new(Pin::new(f));
  let waker = noop_waker();
  let mut context = Context::from_waker(&waker);
  pinned.poll(&mut context)
}

pub fn assert_pending<T: Debug>(f: &mut (impl Unpin + Future<Output = Result<T, ClientError>>)) {
  match poll(f) {
    Poll::Pending => {} // No-op
    Poll::Ready(res) => panic!("unexpectedly ready: {:?}", res),
  }
}

pub fn assert_ready<T: Debug>(
  f: &mut (impl Unpin + Future<Output = Result<T, ClientError>>),
) -> Result<T, ClientError> {
  match poll(f) {
    Poll::Pending => panic!("unexpectedly not ready: {:?}"),
    Poll::Ready(res) => res,
  }
}
