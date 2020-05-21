// Copyright 2020 Daniel Harrison. All Rights Reserved.

use super::log::{Message, NodeID};

pub struct RPC {}
impl RPC {
  pub fn dial(&self, _node: NodeID) -> Conn {
    todo!()
  }
}

pub struct Conn {}
impl Conn {
  pub fn send(&self, _m: Message) {
    todo!()
  }
}
