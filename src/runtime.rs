// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::thread;
use std::sync::mpsc;

pub struct RastClient {
  sender: Sender<Command>,
}

impl RastClient {
  pub fn run(&self, c: Command) -> Response {
    Response::with_payload(c.payload())
  }
}

pub struct Runtime {
  handle: JoinHandle<Result<(), mpsc::RecvError>>,
  sender: Sender<Command>,
}

impl Runtime {
  pub fn new(r: Rast, rpc: RPC) -> Runtime {
    let (sender, receiver) = mpsc::channel();
    let sender2 = sender.clone(); // WIP ugh
    let handle = thread::spawn(move || { Runtime::run(r, sender2, receiver, rpc) })
    // TODO start up a ticker thread too
    Runtime{handle: handle, sender: sender}
  }

  // WIP implement drop?
  pub fn stop(self) {
    // WIP send stop message
    self.handle.join().unwrap()
  }

  pub fn client(&self) -> RastClient {
    RastClient{sender: self.sender.clone()}
  }

  fn run(sm: Rast, res: Sender<Command>, reqs: Receiver<Command>, rpc: RPC) -> Result<(), mpsc::RecvError> {
    let conns: HashMap<Node, Conn> = HashMap::new();
    loop {
      // TODO: tick
      let cmd = reqs.recv()?;
      let req = Message::append_entries_req(cmd.payload());
      let input = Input::Message(req);
      let output = sm.step(input);
      output.map(|o| {
        match o {
          Apply(_) => todo!()
          Message(message) => {
            let dest = message.destination();
            let conn = conns.entry().or_insert(rpc.dial(dest, res.clone()));
            conn.send(message);
          }
        }
      })
    }
  }
}
