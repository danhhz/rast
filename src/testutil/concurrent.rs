// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::collections::HashMap;
use std::time::Instant;

use crate::prelude::*;
use crate::runtime::{MemLog, MemRPC, RastClient, Runtime};

pub struct ConcurrentNode {
  runtime: Runtime,
  rpc: MemRPC,
}

impl ConcurrentNode {
  fn new(id: NodeID, nodes: Vec<NodeID>) -> ConcurrentNode {
    let cfg = Config::default();
    let raft = Raft::new(id, nodes, cfg, Instant::now());
    let rpc = MemRPC::new();
    let runtime = Runtime::new(raft, rpc.clone(), MemLog::new());
    ConcurrentNode { runtime: runtime, rpc: rpc }
  }

  pub fn client(&self) -> RastClient {
    self.runtime.client()
  }
}

pub struct ConcurrentGroup {
  // WIP remove pub
  pub nodes: HashMap<NodeID, ConcurrentNode>,
}

impl ConcurrentGroup {
  pub fn new(nodes: u64) -> ConcurrentGroup {
    let node_ids: Vec<_> = (0..nodes).map(|node| NodeID(node)).collect();
    let mut nodes: HashMap<_, _> = node_ids
      .iter()
      .map(|node_id| (*node_id, ConcurrentNode::new(*node_id, node_ids.clone())))
      .collect();
    let senders: Vec<_> =
      nodes.iter().map(|(node_id, node)| (*node_id, node.runtime.sender())).collect();
    for (_, node) in nodes.iter_mut() {
      for (dest_id, dest_sender) in senders.iter() {
        node.rpc.register(*dest_id, dest_sender.clone());
      }
    }
    ConcurrentGroup { nodes: nodes }
  }
}
