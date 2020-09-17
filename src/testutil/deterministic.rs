// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::prelude::*;
use crate::runtime::MemLog;

pub struct DeterministicNode {
  pub raft: Raft,
  pub now: Instant,
  pub input: Vec<OwnedInput>,
  pub output: Vec<Output>,
  pub log: MemLog,
  pub state: Vec<u8>,
}

impl DeterministicNode {
  fn new(id: NodeID, nodes: Vec<NodeID>, cfg: Config, now: Instant) -> DeterministicNode {
    DeterministicNode {
      raft: Raft::new(id, nodes, cfg),
      now: now,
      input: vec![],
      output: vec![],
      log: MemLog::new(),
      state: vec![],
    }
  }

  pub fn start_election(&mut self) {
    let mut output = vec![];

    #[cfg(feature = "log")]
    debug!("e   {:?}", self.raft.id().0);
    self.raft.start_election(&mut output, self.raft.id());
    #[cfg(feature = "log")]
    {
      output.iter().for_each(|output| {
        debug!("out {:?}: {:?}", self.raft.id().0, output);
      });
      debug!("");
    }

    self.output.extend(output);
  }

  pub fn tick(&mut self, inc: Duration) {
    self.now += inc;
    self.step(Input::Tick(self.now));
  }

  pub fn write(&mut self, req: WriteReq) -> WriteFuture {
    let mut output = vec![];
    let res = WriteFuture::new();

    #[cfg(feature = "log")]
    debug!("w   {:?}: {:?}", self.raft.id().0, req);
    self.raft.step(&mut output, Input::Write(req, res.clone()));
    #[cfg(feature = "log")]
    {
      output.iter().for_each(|output| {
        debug!("out {:?}: {:?}", self.raft.id().0, output);
      });
      debug!("");
    }

    self.output.extend(output);
    res
  }

  pub fn read(&mut self, req: ReadReq) -> ReadFuture {
    let mut output = vec![];
    let res = ReadFuture::new();

    #[cfg(feature = "log")]
    debug!("r   {:?}: {:?}", self.raft.id().0, req);
    self.raft.step(&mut output, Input::Read(req, res.clone()));
    #[cfg(feature = "log")]
    {
      output.iter().for_each(|output| {
        debug!("out {:?}: {:?}", self.raft.id().0, output);
      });
      debug!("");
    }

    self.output.extend(output);
    res
  }

  pub fn step(&mut self, input: Input) {
    let mut output = vec![];

    #[cfg(feature = "log")]
    debug!("in  {:?}: {:?}", self.raft.id().0, input);
    self.raft.step(&mut output, input);
    #[cfg(feature = "log")]
    {
      output.iter().for_each(|output| {
        debug!("out {:?}: {:?}", self.raft.id().0, output);
      });
      debug!("");
    }

    self.output.extend(output);
  }

  fn drain_inputs(&mut self) -> bool {
    let mut did_work = false;
    #[cfg(feature = "log")]
    let id = self.raft.id();
    for input in self.input.drain(..) {
      did_work = true;
      // TODO: dedup this with step
      let mut output = vec![];
      #[cfg(feature = "log")]
      debug!("in  {:?}: {:?}", id.0, input);
      self.raft.step(&mut output, input.as_ref());
      #[cfg(feature = "log")]
      {
        output.iter().for_each(|output| {
          debug!("out {:?}: {:?}", id.0, output);
        });
        debug!("");
      }
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
        Output::PersistReq(req) => {
          // TODO: test this being delayed
          for entry in req.entries {
            debug!("APPEND {:?} {:?}", node.raft.id(), &entry.capnp_as_ref());
            node.log.add(entry.capnp_as_ref());
          }
          debug!(
            "STATE {:?} last={:?} stable={:?} {:?}",
            node.raft.id(),
            node.log.highest_index(),
            node.log.stable,
            node.log.entries
          );
          debug!("");
          let msg = PersistRes {
            leader_id: req.leader_id,
            read_id: req.read_id,
            log_index: node.log.highest_index(),
          };
          node.input.push(Input::PersistRes(msg).into());
        }
        Output::ApplyReq(index) => {
          // TODO: test this being delayed
          node.log.mark_stable(index);
          let mut state: Vec<u8> = vec![];
          for (_, (_, payload)) in node.log.entries.range(..=index) {
            state.extend(payload.iter());
          }
          debug!("APPLY  {:?} {:?}", node.raft.id(), state);
          debug!("");
        }
        Output::ReadStateMachineReq(req) => {
          // TODO: test this being delayed
          debug!("READ   {:?} {:?}", node.raft.id(), &node.state);
          debug!("");
          let mut state = vec![];
          if let Some(stable_index) = node.log.stable {
            for (_, (_, payload)) in node.log.entries.range(..=stable_index) {
              state.extend(payload.iter());
            }
          }
          let msg = ReadStateMachineRes { index: req.index, read_id: req.read_id, payload: state };
          node.input.push(Input::ReadStateMachineRes(msg).into());
        }
        Output::Message(msg) => rpcs.push(msg),
      }
    }
  }
  for msg in rpcs.drain(..) {
    let dest = NodeID(msg.capnp_as_ref().dest());
    nodes
      .get_mut(&dest)
      .iter_mut()
      // TODO: get rid of this clone
      .for_each(|dest| dest.input.push(OwnedInput::Message(msg.clone())));
  }
}

pub trait DeterministicGroup {
  fn cfg(&self) -> &Config;
  fn nodes(&self) -> Vec<&DeterministicNode>;
  fn nodes_mut(&mut self) -> Vec<&mut DeterministicNode>;

  fn tick(&mut self, inc: Duration) {
    // TODO: ensure this ticks them all to the same time
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
