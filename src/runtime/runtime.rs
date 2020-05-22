// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::collections::{HashMap, VecDeque};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

use crate::prelude::*;
use crate::runtime::logimpl::MemLog;

#[derive(Clone)]
pub struct RastClient {
  sender: Sender<Input>,
}

impl RastClient {
  pub fn read(&self, req: ReadReq) -> ReadFuture {
    let mut res = ReadFuture::new();
    self.sender.send(Input::Read(req, res.clone())).err().iter().for_each(|_| {
      // An error here means the channel is closed, which means the raft loop
      // has exited. Dunno who the leader is but it's not us.
      res.fill(Err(ClientError::NotLeaderError(NotLeaderError::new(None))));
    });
    res
  }

  pub fn write(&self, req: WriteReq) -> WriteFuture {
    let mut res = WriteFuture::new();
    self.sender.send(Input::Write(req, res.clone())).err().iter().for_each(|_| {
      // An error here means the channel is closed, which means the raft loop
      // has exited. Dunno who the leader is but it's not us.
      res.fill(Err(ClientError::NotLeaderError(NotLeaderError::new(None))));
    });
    res
  }
}

pub struct Runtime {
  handle: Option<JoinHandle<Result<(), mpsc::RecvError>>>,
  client: RastClient,
}

impl Runtime {
  pub fn new(raft: Raft, rpc: MemRPC, log: MemLog) -> Runtime {
    let (sender, receiver) = mpsc::channel();
    let client = RastClient { sender: sender };
    let handle = thread::spawn(move || Runtime::run(raft, receiver, rpc, log));
    // TODO start up a ticker thread too
    Runtime { handle: Some(handle), client: client }
  }

  pub fn stop(&mut self) {
    // Send the shutdown sentinel.
    match self.client.sender.send(Input::PersistRes(Index(0), NodeID(0))).err() {
      Some(_) => println!("runtime crashed before stop"),
      None => {
        println!("runtime stopping");
        self.handle.take().unwrap().join().unwrap().unwrap();
        println!("runtime stopped");
      }
    }
  }

  pub fn client(&self) -> RastClient {
    self.client.clone()
  }

  // WIP get rid of this
  pub fn sender(&self) -> Sender<Input> {
    self.client.sender.clone()
  }

  fn run(
    mut raft: Raft,
    reqs: Receiver<Input>,
    rpc: MemRPC,
    _log: MemLog,
  ) -> Result<(), mpsc::RecvError> {
    let mut conns: HashMap<NodeID, MemConn> = HashMap::new();
    let mut cmds = VecDeque::new();
    let mut output = vec![];
    let mut state = vec![];
    loop {
      let cmd = match cmds.pop_front() {
        Some(cmd) => cmd,
        None => reqs.recv()?,
      };
      // If we got the shutdown sentinel, exit.
      match cmd {
        // WIP make this a first class message type
        Input::PersistRes(index, _) => {
          if index == Index(0) {
            return Ok(());
          }
        }
        _ => {}
      }
      println!("{:?}: {:?}", raft.id().0, cmd);
      raft.step(&mut output, cmd);
      output.iter().for_each(|o| println!("  out: {:?}", o));
      output.drain(..).for_each(|output| match output {
        Output::ApplyReq(_) => {
          // WIP implement
        }
        Output::PersistReq(node, entries) => {
          // WIP implement
          entries.iter().for_each(|entry| state.extend(entry.payload.iter()));
          cmds.push_back(Input::PersistRes(entries.last().unwrap().index, node));
        }
        Output::ReadStateMachine(index, idx, _) => {
          // WIP implement
          let payload = state.clone();
          cmds.push_back(Input::ReadStateMachine(index, idx, payload));
        }
        Output::Message(message) => {
          let dest = message.dest;
          let conn = conns.entry(dest).or_insert_with(|| rpc.dial(dest));
          conn.send(message);
        }
      });
    }
  }
}

impl Drop for Runtime {
  fn drop(&mut self) {
    self.stop();
  }
}

#[derive(Clone)]
pub struct MemRPC {
  conns: Arc<Mutex<HashMap<NodeID, Sender<Input>>>>,
}
impl MemRPC {
  pub fn new() -> MemRPC {
    MemRPC { conns: Default::default() }
  }

  pub fn register(&mut self, dest: NodeID, sender: Sender<Input>) {
    // WIP handle error
    self.conns.lock().unwrap().insert(dest, sender);
  }

  pub fn dial(&self, node: NodeID) -> MemConn {
    // WIP handle error
    let sender = self.conns.lock().unwrap().get(&node).unwrap().clone();
    MemConn { sender: sender }
  }
}

pub struct MemConn {
  sender: Sender<Input>,
}
impl MemConn {
  pub fn send(&self, m: Message) {
    // WIP handle error
    self.sender.send(Input::Message(m)).unwrap();
  }
}
