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
//! let mut raft = Raft::new(NodeID(0), vec![NodeID(0)], Config::default(), Instant::now());
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
//! let raft = Raft::new(NodeID(0), vec![NodeID(0)], Config::default(), Instant::now());
//! let mut rpc = MemRPC::new();
//! let runtime = Runtime::new(raft, rpc.clone(), MemLog::new());
//! rpc.register(NodeID(0), runtime.sender());
//!
//! // This client is Clone+Send.
//! let client = runtime.client();
//! assert_eq!(extreme::run(do_work(client)), "1");
//! # }
//! ```

#![warn(clippy::correctness, clippy::perf, clippy::wildcard_imports)]

mod error;
mod future;
mod raft;
mod serde;

pub use crate::error::NotLeaderError;
pub use crate::future::{ReadFuture, WriteFuture};
pub use crate::raft::{Config, Input, Output, Raft};
pub use crate::serde::{Entry, Index, Message, NodeID, ReadReq, ReadRes, Term, WriteReq, WriteRes};

pub mod prelude {
  pub use crate::*;
}

#[cfg(any(feature = "runtime", test))]
pub mod runtime {
  mod runtime;
  pub use runtime::*;

  mod logimpl;
  pub use logimpl::*;
}

#[cfg(test)]
mod nemesis {
  mod nemesis;
  pub use nemesis::*;
}

#[cfg(test)]
mod testutil {
  mod deterministic;
  pub use deterministic::*;

  pub mod noopfuture;
}

// TODO: figure out how to call output.extend without creating a vec
// TODO: more consistent method naming
// TODO: compress log implementation
// TODO: randomized invariant testing
// TODO: failure testing
// TODO: ensure that heartbeat (empty append) is no disk write in common case
// TODO: write internals docs
// TODO: write externals docs
// TODO: restart node with non-empty log + hard state
// TODO: single node special cases
// - election concludes immediately
// - read/write req to candidate is successful
// - committed as soon as it's persisted
// TODO: benchmarks
// TODO: graceful leader handoff
// TODO: retry append rpc, find where follower diverges
// TODO: tests
// - election timeout, node isn't elected in a short enough time
// - stuck election/split vote, all nodes vote for themselves
// - election completes with majority but not all nodes
// - expand this list with examples from the raft paper
// - nothing can be written at an index once that index is read
// - runtime behavior under shutdown, gracefully returns errors
