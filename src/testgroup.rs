// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::time::{Duration, Instant};

use super::*;

pub struct TestNode {
  pub sm: Rast,
  pub input: Vec<Input>,
  pub output: Vec<Output>,
}

impl TestNode {
  fn new(id: NodeID, nodes: Vec<NodeID>, cfg: Config, now: Instant) -> TestNode {
    TestNode { sm: Rast::new(id, nodes, cfg, now), input: vec![], output: vec![] }
  }

  pub fn tick(&mut self, inc: Duration) {
    self.step(Input::Tick(self.sm.current_time + inc));
  }

  pub fn step(&mut self, input: Input) {
    self.output.extend(self.sm.step(input));
  }

  fn drain_inputs(&mut self) -> bool {
    let mut did_work = false;
    for input in self.input.drain(..) {
      did_work = true;
      self.output.extend(self.sm.step(input));
    }
    did_work
  }
}

pub struct TestGroup1 {
  cfg: Config,
  pub n: TestNode,
}

impl TestGroup1 {
  pub fn new() -> TestGroup1 {
    let now = Instant::now();
    let cfg = default_cfg();
    TestGroup1 { n: TestNode::new(NodeID(0), vec![NodeID(0)], cfg.clone(), now), cfg: cfg }
  }
}

impl TestGroup for TestGroup1 {
  fn cfg(&self) -> &Config {
    &self.cfg
  }
  fn nodes(&self) -> Vec<&TestNode> {
    vec![&self.n]
  }
  fn nodes_mut(&mut self) -> Vec<&mut TestNode> {
    vec![&mut self.n]
  }
}

pub struct TestGroup3 {
  cfg: Config,
  pub n0: TestNode,
  pub n1: TestNode,
  pub n2: TestNode,
}

impl TestGroup3 {
  pub fn new() -> TestGroup3 {
    let now = Instant::now();
    let cfg = default_cfg();
    let nodes = vec![NodeID(0), NodeID(1), NodeID(2)];
    TestGroup3 {
      n0: TestNode::new(NodeID(0), nodes.clone(), cfg.clone(), now),
      n1: TestNode::new(NodeID(1), nodes.clone(), cfg.clone(), now),
      n2: TestNode::new(NodeID(2), nodes, cfg.clone(), now),
      cfg: cfg,
    }
  }
}

impl TestGroup for TestGroup3 {
  fn cfg(&self) -> &Config {
    &self.cfg
  }
  fn nodes(&self) -> Vec<&TestNode> {
    vec![&self.n0, &self.n1, &self.n2]
  }
  fn nodes_mut(&mut self) -> Vec<&mut TestNode> {
    vec![&mut self.n0, &mut self.n1, &mut self.n2]
  }
}

fn default_cfg() -> Config {
  Config {
    election_timeout: Duration::from_millis(100),
    heartbeat_interval: Duration::from_millis(10),
  }
}

fn drain_inputs(nodes: &mut HashMap<NodeID, &mut TestNode>) -> bool {
  let mut did_work = false;
  for (_, node) in nodes {
    did_work = did_work || node.drain_inputs();
  }
  did_work
}

fn drain_outputs(nodes: &mut HashMap<NodeID, &mut TestNode>) {
  // TODO: Do this without the intermediate vector.
  let mut output = vec![];
  for (_, node) in nodes.iter_mut() {
    output.append(&mut node.output);
  }
  for output in output.drain(..) {
    match output {
      Output::Apply(_) => todo!(),
      Output::Message(msg) => {
        nodes
          .get_mut(&msg.dest)
          .iter_mut()
          // TODO: Get rid of this clone.
          .for_each(|dest| dest.input.push(Input::Message(msg.clone())));
      }
    }
  }
}

pub trait TestGroup {
  fn cfg(&self) -> &Config;
  fn nodes(&self) -> Vec<&TestNode>;
  fn nodes_mut(&mut self) -> Vec<&mut TestNode>;

  fn tick(&mut self, inc: Duration) {
    // WIP: ensure this ticks them all to the same time
    self.nodes_mut().iter_mut().for_each(|node| node.tick(inc));
  }

  fn drain(&mut self) {
    let mut nodes: HashMap<NodeID, &mut TestNode> =
      self.nodes_mut().drain(..).map(|node| (node.sm.id, node)).collect();
    drain_outputs(&mut nodes);
    while drain_inputs(&mut nodes) {
      drain_outputs(&mut nodes);
    }
  }
}
