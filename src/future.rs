// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::{Context, Poll, Waker};

use super::error::*;
use super::log::*;

#[derive(Debug)]
struct WriteFutureState {
  result: Option<Result<WriteRes, NotLeaderError>>,
  waker: Option<Waker>,
}

#[derive(Debug, Clone)]
pub struct WriteFuture {
  state: Arc<Mutex<WriteFutureState>>,
}

impl WriteFuture {
  pub fn new() -> WriteFuture {
    WriteFuture { state: Arc::new(Mutex::new(WriteFutureState { result: None, waker: None })) }
  }

  pub(crate) fn fill(&mut self, result: Result<WriteRes, NotLeaderError>) {
    // WIP: what should we do if the lock is poisoned?
    if let Ok(mut state) = self.state.lock() {
      debug_assert!(state.result.is_none());
      state.result = Some(result);
      state.waker.iter_mut().for_each(|waker| waker.wake_by_ref());
    }
  }
}

impl Future for WriteFuture {
  type Output = Result<WriteRes, NotLeaderError>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
    let mut state: MutexGuard<WriteFutureState> = match self.state.lock() {
      Ok(guard) => guard,
      Err(_) => {
        // TODO: this isn't the right error but close enough for now
        return Poll::Ready(Err(NotLeaderError::new(NodeID(0))));
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
