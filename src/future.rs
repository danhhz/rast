// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::{Context, Poll, Waker};

use super::error::{ClientError, NotLeaderError};
use super::serde::{NodeID, ReadRes, WriteRes};

struct RastFutureState<T> {
  finished: bool,
  result: Option<Result<T, ClientError>>,
  waker: Option<Waker>,
}

impl<T: fmt::Debug> fmt::Debug for RastFutureState<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:?}", self.result)
  }
}

#[derive(Debug, Clone)]
struct RastFuture<T> {
  state: Arc<Mutex<RastFutureState<T>>>,
}

impl<T> RastFuture<T> {
  fn new() -> RastFuture<T> {
    RastFuture {
      state: Arc::new(Mutex::new(RastFutureState { finished: false, result: None, waker: None })),
    }
  }

  fn fill(&mut self, result: Result<T, ClientError>) {
    // TODO: what should we do if the lock is poisoned?
    if let Ok(mut state) = self.state.lock() {
      debug_assert_eq!(state.finished, false);
      state.finished = true;
      state.result = Some(result);
      state.waker.iter_mut().for_each(|waker| waker.wake_by_ref());
    }
  }

  fn poll(&self, cx: &mut Context) -> Poll<Result<T, ClientError>> {
    let mut state: MutexGuard<RastFutureState<T>> = match self.state.lock() {
      Ok(guard) => guard,
      Err(_) => {
        // TODO: this isn't the right error but close enough for now
        return Poll::Ready(Err(ClientError::NotLeaderError(NotLeaderError::new(Some(NodeID(0))))));
      }
    };
    // TODO: this `take()` is technically correct since the Future api requires
    // that poll never be called once it's returned Ready, but it makes me
    // uncomfortable
    if let Some(result) = state.result.take() {
      Poll::Ready(result)
    } else {
      state.waker = Some(cx.waker().clone());
      Poll::Pending
    }
  }
}

#[derive(Debug, Clone)]
pub struct WriteFuture {
  f: RastFuture<WriteRes>,
}

impl WriteFuture {
  pub fn new() -> WriteFuture {
    WriteFuture { f: RastFuture::new() }
  }
  pub(crate) fn fill(&mut self, result: Result<WriteRes, ClientError>) {
    self.f.fill(result)
  }
}

impl Future for WriteFuture {
  type Output = Result<WriteRes, ClientError>;
  fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
    self.f.poll(cx)
  }
}

#[derive(Debug, Clone)]
pub struct ReadFuture {
  f: RastFuture<ReadRes>,
}

impl ReadFuture {
  pub fn new() -> ReadFuture {
    ReadFuture { f: RastFuture::new() }
  }
  pub(crate) fn fill(&mut self, result: Result<ReadRes, ClientError>) {
    self.f.fill(result)
  }
}

impl Future for ReadFuture {
  type Output = Result<ReadRes, ClientError>;
  fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
    self.f.poll(cx)
  }
}
