// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::sync::mpsc::Sender;

pub trait RPC {
  // WIP this is awkward and abstraction breaking
  fn dial(node: ::crate::log::Node, responses: Sender<Message>) -> Conn;
}

pub trait Conn {
  fn send(m: Message);
}
