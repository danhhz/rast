// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use crate::prelude::*;

/// An channel-based, in-process rpc implementation. Suitable for unit tests and
/// benchmarks.
#[derive(Debug, Clone)]
pub struct MemRPC {
  conns: Arc<Mutex<HashMap<NodeID, Sender<OwnedInput>>>>,
}
impl MemRPC {
  /// Constructs a new `MemRPC` with no connections.
  pub fn new() -> MemRPC {
    MemRPC { conns: Default::default() }
  }

  /// Registers a channel Sender and its destination.
  pub fn register(&mut self, dest: NodeID, sender: Sender<OwnedInput>) {
    // TODO: handle error
    self.conns.lock().unwrap().insert(dest, sender);
  }

  /// Returns a connection for sending to the specified node.
  pub fn dial(&self, node: NodeID) -> MemConn {
    // TODO: handle error
    let sender = self.conns.lock().unwrap().get(&node).unwrap().clone();
    MemConn { sender: sender }
  }
}

/// A channel-based, in-process rpc connection to a peer node. Suitable for unit
/// tests and benchmarks.
pub struct MemConn {
  sender: Sender<OwnedInput>,
}
impl MemConn {
  /// Sends the given message.
  pub fn send<'a>(&self, m: MessageRef<'a>) {
    // TODO: handle error
    self.sender.send(OwnedInput::Message(m.capnp_to_owned())).unwrap();
  }
}
