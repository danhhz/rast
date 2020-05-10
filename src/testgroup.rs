// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::time::{Duration, Instant};

use super::*;

pub struct TestGroup {
  pub now: Instant,
  pub nodes: HashMap<NodeID, Rast>,
  pub node_queues: HashMap<NodeID, Vec<Input>>,
}

impl TestGroup {
  pub fn new(num_nodes: u64) -> TestGroup {
    let now = Instant::now();
    let cfg = Config {
      election_timeout: Duration::from_millis(100),
      heartbeat_interval: Duration::from_millis(10),
    };
    let node_ids: Vec<NodeID> = (0..num_nodes).map(|idx| NodeID(idx)).collect();
    let nodes: HashMap<NodeID, Rast> = node_ids
      .iter()
      .cloned()
      .map(|id| {
        let mut r = Rast::new(cfg.clone(), now);
        r.set_id(id);
        r.set_nodes(&node_ids);
        (id, r)
      })
      .collect();
    let node_queues: HashMap<NodeID, Vec<Input>> =
      node_ids.iter().cloned().map(|id| (id, vec![])).collect();
    TestGroup {
      now: now,
      nodes: nodes,
      node_queues: node_queues,
    }
  }

  pub fn node(&self, node: NodeID) -> &Rast {
    self.nodes.get(&node).unwrap()
  }

  pub fn node_mut(&mut self, node: NodeID) -> &mut Rast {
    self.nodes.get_mut(&node).unwrap()
  }

  pub fn _tick(&mut self, inc: Duration) {
    self.now += inc;
    let mut nodes: Vec<NodeID> = self.nodes.keys().cloned().collect();
    nodes
      .drain(..)
      .for_each(|node| self.tick_one(node, Duration::from_nanos(0)));
  }

  pub fn tick_one(&mut self, node: NodeID, inc: Duration) {
    self.now += inc;
    self.step(node, Input::Tick(self.now))
  }

  pub fn step(&mut self, node: NodeID, input: Input) {
    let mut output = self.nodes.get_mut(&node).map_or(vec![], |r| r.step(input));
    output.drain(..).for_each(|output| {
      match output {
        Output::Apply(_) => {} // No-op
        Output::Message(message) => {
          self
            .node_queues
            .get_mut(&message.dest)
            .map(|queue| queue.push(Input::Message(message)));
        }
      }
    });
  }

  pub fn drain(&mut self) {
    loop {
      // dbg!(&self.node_queues);
      let inputs_opt: Option<(NodeID, Vec<Input>)> = self
        .node_queues
        .iter_mut()
        .find(|queue| !queue.1.is_empty())
        .map(|queue| (*queue.0, queue.1.drain(..).collect()));
      // dbg!(&inputs_opt);
      if let Some((node, mut inputs)) = inputs_opt {
        inputs.drain(..).for_each(|input| self.step(node, input));
      } else {
        return;
      }
    }
  }
}
