// Copyright 2020 Daniel Harrison. All Rights Reserved.

//! An implementation of the [raft consensus protocol].
//!
//! [raft consensus protocol]: https://raft.github.io/
//!
//! # Examples
//!
//! The core logic is deterministic and single threaded.
//!
//! ```
//! use std::time::Instant;
//! use rast::prelude::*;
//!
//! # fn main() {
//! let mut raft = Raft::new(NodeID(0), vec![NodeID(0)], Config::default());
//! let mut output = vec![];
//! raft.step(&mut output, Input::Tick(Instant::now()));
//! # }
//! ```
//!
//! A "batteries included" runtime is also available that hooks this up to a
//! ticker and persistent log. This is enabled by opting in to the "runtime"
//! crate feature.
//!
//! ```
//! # #[cfg(feature = "runtime")] {
//! use std::time::Instant;
//! use rast::prelude::*;
//! use rast::runtime::{Runtime,MemRPC,MemLog,RastClient};
//! use extreme;
//!
//! async fn do_work(client: RastClient) -> String {
//!   # // TODO: the following line is working around a bug where the first
//!   # // write gets eaten
//!   # let _ = client.write(WriteReq{payload: vec![]});
//!   let _ = client.write(WriteReq{payload: "1".as_bytes().to_vec()});
//!   let read = client.read(ReadReq{payload: vec![]});
//!   let result_bytes = read.await.unwrap();
//!   String::from_utf8(result_bytes.payload).unwrap()
//! }
//!
//! # fn main() {
//! let raft = Raft::new(NodeID(0), vec![NodeID(0)], Config::default());
//! let mut rpc = MemRPC::new();
//! let runtime = Runtime::new("rast".to_string(), raft, rpc.clone(), MemLog::new());
//! rpc.register(NodeID(0), runtime.sender());
//!
//! // This client is Clone+Send.
//! let client = runtime.client();
//! assert_eq!(extreme::run(do_work(client)), "1");
//! # }
//! # }
//! ```

#![warn(missing_docs, unsafe_code)]
#![warn(clippy::correctness, clippy::perf, clippy::wildcard_imports)]

// Log macros copied from the rand crate without changes:
// https://crates.io/crates/rand.
#[allow(unused)]
macro_rules! trace { ($($x:tt)*) => (
    #[cfg(feature = "log")] {
        log::trace!($($x)*)
    }
) }
#[allow(unused)]
macro_rules! debug { ($($x:tt)*) => (
    #[cfg(feature = "log")] {
        log::debug!($($x)*)
    }
) }
#[allow(unused)]
macro_rules! info { ($($x:tt)*) => (
    #[cfg(feature = "log")] {
        log::info!($($x)*)
    }
) }
#[allow(unused)]
macro_rules! warn { ($($x:tt)*) => (
    #[cfg(feature = "log")] {
        log::warn!($($x)*)
    }
) }
#[allow(unused)]
macro_rules! error { ($($x:tt)*) => (
    #[cfg(feature = "log")] {
        log::error!($($x)*)
    }
) }

mod compressed_log;
mod error;
mod future;
mod raft;
mod serde;

pub use crate::error::{ClientError, NotLeaderError};
pub use crate::future::{ReadFuture, WriteFuture};
pub use crate::raft::{Config, Input, Output, OwnedInput, PersistRes, Raft, ReadStateMachineRes};
pub use crate::serde::{
  EntryRef, EntryShared, Index, MessageRef, MessageShared, NodeID, ReadID, ReadReq, ReadRes, Term,
  WriteReq, WriteRes,
};

/// The Raft prelude.
///
/// This module re-exports all the types necessary to implement a runtime around
/// the core deterministic Raft logic. It exists because modules named "prelude"
/// are special cased by clippy::wildcard_imports.
pub mod prelude {
  pub use crate::*;
}

/// A "batteries included" runtime.
///
/// This hooks the determinic Raft implementation up to a ticker, persistent
/// log, and rpc system. This is enabled by opting in to the "runtime" crate
/// feature.
#[cfg(any(feature = "runtime", test))]
pub mod runtime {
  mod memlog;
  pub use memlog::*;

  mod memrpc;
  pub use memrpc::*;

  mod runtime;
  pub use runtime::*;
}

#[cfg(test)]
mod nemesis {
  mod nemesis;
  pub use nemesis::*;
}

#[cfg(test)]
mod testutil {
  use env_logger;
  pub fn log_init() {
    let _ = env_logger::builder()
      .is_test(true)
      .format_module_path(false)
      .format_timestamp(None)
      .try_init();
  }

  mod deterministic;
  pub use deterministic::*;

  mod concurrent;
  pub use concurrent::*;

  #[allow(unsafe_code)]
  pub mod noopfuture;
}

// TODO: figure out how to call output.extend without creating a vec
// TODO: more consistent method naming
// TODO: restart node with non-empty log + hard state
// TODO: retry append rpc, find where follower diverges
// TODO: nemesis test shouldn't hang when something panics
// TODO: tests
// - election timeout, node isn't elected in a short enough time
// - stuck election/split vote, all nodes vote for themselves
// - election completes with majority but not all nodes
// - expand this list with examples from the raft paper
// - nothing can be written at an index once that index is read
// - runtime behavior under shutdown, gracefully returns errors
// - regression test for committing an entry from a previous term
// - single node special cases
//   - election concludes immediately
//   - read/write req to candidate is successful
//   - committed as soon as it's persisted
