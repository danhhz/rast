// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

use super::log::*;

#[derive(Debug)]
struct WriteFutureState {
  term_index: Option<(Term, Index)>,
  waker: Option<Waker>,
}

#[derive(Debug, Clone)]
pub struct WriteFuture {
  state: Arc<Mutex<WriteFutureState>>,
}

impl WriteFuture {
  pub fn new() -> WriteFuture {
    WriteFuture { state: Arc::new(Mutex::new(WriteFutureState { term_index: None, waker: None })) }
  }

  pub(crate) fn _fill(&mut self, term: Term, index: Index) {
    // WIP: what should we do if the lock is poisoned?
    if let Ok(mut state) = self.state.lock() {
      debug_assert!(state.term_index.is_none());
      state.term_index = Some((term, index));
      state.waker.iter_mut().for_each(|waker| waker.wake_by_ref());
    }
  }
}

impl Future for WriteFuture {
  type Output = WriteRes;

  fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
    let mut state = self.state.lock().unwrap();
    if let Some((term, index)) = state.term_index {
      Poll::Ready(WriteRes { term: term, index: index })
    } else {
      state.waker = Some(cx.waker().clone());
      Poll::Pending
    }
  }
}
