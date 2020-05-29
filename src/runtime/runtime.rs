// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::collections::{HashMap, VecDeque};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

use crate::prelude::*;
use crate::runtime::{MemConn, MemLog, MemRPC};

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
  pub id: NodeID,
  handle: Option<JoinHandle<Result<(), mpsc::RecvError>>>,
  client: RastClient,
}

impl Runtime {
  pub fn new(raft: Raft, rpc: MemRPC, log: MemLog) -> Runtime {
    let id = raft.id();
    let (sender, receiver) = mpsc::channel();
    let client = RastClient { sender: sender };
    let handle = thread::spawn(move || Runtime::run(raft, receiver, rpc, log));
    // TODO start up a ticker thread too
    Runtime { id: id, handle: Some(handle), client: client }
  }

  pub fn stop(&mut self) {
    // Send the shutdown sentinel.
    let msg = PersistRes { leader_id: NodeID(0), read_id: ReadID(0), log_index: Index(0) };
    match self.client.sender.send(Input::PersistRes(msg)).err() {
      Some(_) => {
        debug!("runtime crashed before stop");
      }
      None => {
        debug!("runtime stopping");
        self.handle.take().unwrap().join().unwrap().unwrap();
        debug!("runtime stopped");
      }
    }
  }

  pub fn client(&self) -> RastClient {
    self.client.clone()
  }

  // TODO: get rid of this
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
      match &cmd {
        Input::PersistRes(res) => {
          if res.log_index == Index(0) {
            return Ok(());
          }
        }
        _ => {}
      }
      raft.step(&mut output, cmd);
      #[cfg(feature = "log")]
      output.iter().for_each(|o| {
        debug!("  out: {:?}", o);
      });
      output.drain(..).for_each(|output| match output {
        Output::ApplyReq(_) => {
          // TODO: implement
        }
        Output::PersistReq(req) => {
          // TODO: implement
          req.entries.iter().for_each(|entry| state.extend(entry.payload.iter()));
          let msg = PersistRes {
            leader_id: req.leader_id,
            read_id: req.read_id,
            log_index: req.entries.last().unwrap().index,
          };
          cmds.push_back(Input::PersistRes(msg));
        }
        Output::ReadStateMachineReq(req) => {
          // TODO: implement
          let payload = state.clone();
          let msg =
            ReadStateMachineRes { index: req.index, read_id: req.read_id, payload: payload };
          cmds.push_back(Input::ReadStateMachineRes(msg));
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
