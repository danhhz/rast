// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::prelude::*;
use crate::runtime::MemLog;
use crate::serde::{Payload, StartElectionReq};

pub struct DeterministicNode {
  pub raft: Raft,
  pub input: Vec<Input>,
  pub output: Vec<Output>,
  pub log: MemLog,
  pub state: Vec<u8>,
}

impl DeterministicNode {
  fn new(id: NodeID, nodes: Vec<NodeID>, cfg: Config, now: Instant) -> DeterministicNode {
    DeterministicNode {
      raft: Raft::new(id, nodes, cfg, now),
      input: vec![],
      output: vec![],
      log: MemLog::new(),
      state: vec![],
    }
  }

  pub fn start_election(&mut self) {
    let payload = Payload::StartElectionReq(StartElectionReq { term: self.raft.current_term() });
    self.step(Input::Message(Message {
      src: self.raft.id(),
      dest: self.raft.id(),
      payload: payload,
    }));
  }

  pub fn tick(&mut self, inc: Duration) {
    self.step(Input::Tick(self.raft.current_time() + inc));
  }

  pub fn write(&mut self, req: WriteReq) -> WriteFuture {
    let mut output = vec![];
    let res = WriteFuture::new();

    println!("write  {:?}: {:?}", self.raft.id().0, req);
    self.raft.step(&mut output, Input::Write(req, res.clone()));
    output.iter().for_each(|output| {
      println!("output {:?}: {:?}", self.raft.id().0, output);
    });
    println!();

    self.output.extend(output);
    res
  }

  pub fn read(&mut self, req: ReadReq) -> ReadFuture {
    let mut output = vec![];
    let res = ReadFuture::new();

    println!("read   {:?}: {:?}", self.raft.id().0, req);
    self.raft.step(&mut output, Input::Read(req, res.clone()));
    output.iter().for_each(|output| {
      println!("output {:?}: {:?}", self.raft.id().0, output);
    });
    println!();

    self.output.extend(output);
    res
  }

  pub fn step(&mut self, input: Input) {
    let mut output = vec![];

    println!("input  {:?}: {:?}", self.raft.id().0, input);
    self.raft.step(&mut output, input);
    output.iter().for_each(|output| {
      println!("output {:?}: {:?}", self.raft.id().0, output);
    });
    println!();

    self.output.extend(output);
  }

  fn drain_inputs(&mut self) -> bool {
    let mut did_work = false;
    let id = self.raft.id();
    for input in self.input.drain(..) {
      did_work = true;
      // WIP: dedup this with step
      let mut output = vec![];
      println!("input  {:?}: {:?}", id.0, input);
      self.raft.step(&mut output, input);
      output.iter().for_each(|output| {
        println!("output {:?}: {:?}", id.0, output);
      });
      println!();
      self.output.extend(output);
    }
    did_work
  }
}

pub struct DeterministicGroup1 {
  cfg: Config,
  pub n: DeterministicNode,
}

impl DeterministicGroup1 {
  pub fn new() -> DeterministicGroup1 {
    let now = Instant::now();
    let cfg: Config = Default::default();
    DeterministicGroup1 {
      n: DeterministicNode::new(NodeID(0), vec![NodeID(0)], cfg.clone(), now),
      cfg: cfg,
    }
  }
}

impl DeterministicGroup for DeterministicGroup1 {
  fn cfg(&self) -> &Config {
    &self.cfg
  }
  fn nodes(&self) -> Vec<&DeterministicNode> {
    vec![&self.n]
  }
  fn nodes_mut(&mut self) -> Vec<&mut DeterministicNode> {
    vec![&mut self.n]
  }
}

pub struct DeterministicGroup3 {
  cfg: Config,
  pub n0: DeterministicNode,
  pub n1: DeterministicNode,
  pub n2: DeterministicNode,
}

impl DeterministicGroup3 {
  pub fn new() -> DeterministicGroup3 {
    let now = Instant::now();
    let cfg: Config = Default::default();
    let nodes = vec![NodeID(0), NodeID(1), NodeID(2)];
    DeterministicGroup3 {
      n0: DeterministicNode::new(NodeID(0), nodes.clone(), cfg.clone(), now),
      n1: DeterministicNode::new(NodeID(1), nodes.clone(), cfg.clone(), now),
      n2: DeterministicNode::new(NodeID(2), nodes, cfg.clone(), now),
      cfg: cfg,
    }
  }
}

impl DeterministicGroup for DeterministicGroup3 {
  fn cfg(&self) -> &Config {
    &self.cfg
  }
  fn nodes(&self) -> Vec<&DeterministicNode> {
    vec![&self.n0, &self.n1, &self.n2]
  }
  fn nodes_mut(&mut self) -> Vec<&mut DeterministicNode> {
    vec![&mut self.n0, &mut self.n1, &mut self.n2]
  }
}

fn drain_inputs(nodes: &mut HashMap<NodeID, &mut DeterministicNode>) -> bool {
  let mut did_work = false;
  for (_, node) in nodes {
    did_work = did_work || node.drain_inputs();
  }
  did_work
}

fn drain_outputs(nodes: &mut HashMap<NodeID, &mut DeterministicNode>) {
  // TODO: do this without the intermediate vector
  let mut rpcs = vec![];
  for (_, node) in nodes.iter_mut() {
    for output in node.output.drain(..) {
      match output {
        Output::PersistReq(leader_id, entries, read_id) => {
          // TODO: test this being delayed
          for entry in entries {
            println!("APPEND {:?} {:?}", node.raft.id(), &entry);
            println!();
            node.log.add(entry);
          }
          node.input.push(Input::PersistRes(node.log.highest_index(), leader_id, read_id));
        }
        Output::ApplyReq(index) => {
          // TODO: test this being delayed
          node.log.mark_stable(index);
          let payload = node.log.get(index).unwrap();
          node.state.extend(payload);
          println!("APPLY  {:?} {:?}", node.raft.id(), &node.state);
          println!();
        }
        Output::ReadStateMachineReq(index, read_id, _) => {
          // TODO: test this being delayed
          println!("READ   {:?} {:?}", node.raft.id(), &node.state);
          println!();
          let payload = node.state.clone();
          node.input.push(Input::ReadStateMachineRes(index, read_id, payload));
        }
        Output::Message(msg) => rpcs.push(msg),
      }
    }
  }
  for msg in rpcs.drain(..) {
    let dest = msg.dest.clone();
    nodes
      .get_mut(&dest)
      .iter_mut()
      // TODO: get rid of this clone
      .for_each(|dest| dest.input.push(Input::Message(msg.clone())));
  }
}

pub trait DeterministicGroup {
  fn cfg(&self) -> &Config;
  fn nodes(&self) -> Vec<&DeterministicNode>;
  fn nodes_mut(&mut self) -> Vec<&mut DeterministicNode>;

  fn tick(&mut self, inc: Duration) {
    // WIP: ensure this ticks them all to the same time
    self.nodes_mut().iter_mut().for_each(|node| node.tick(inc));
  }

  fn drain(&mut self) {
    let mut nodes: HashMap<NodeID, &mut DeterministicNode> =
      self.nodes_mut().drain(..).map(|node| (node.raft.id(), node)).collect();
    drain_outputs(&mut nodes);
    while drain_inputs(&mut nodes) {
      drain_outputs(&mut nodes);
    }
  }
}
