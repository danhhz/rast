// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct MemRPC {
  conns: Arc<Mutex<HashMap<NodeID, Sender<Input>>>>,
}
impl MemRPC {
  pub fn new() -> MemRPC {
    MemRPC { conns: Default::default() }
  }

  pub fn register(&mut self, dest: NodeID, sender: Sender<Input>) {
    // TODO: handle error
    self.conns.lock().unwrap().insert(dest, sender);
  }

  pub fn dial(&self, node: NodeID) -> MemConn {
    // TODO: handle error
    let sender = self.conns.lock().unwrap().get(&node).unwrap().clone();
    MemConn { sender: sender }
  }
}

pub struct MemConn {
  sender: Sender<Input>,
}
impl MemConn {
  pub fn send(&self, m: Message) {
    // TODO: handle error
    self.sender.send(Input::Message(m)).unwrap();
  }
}
