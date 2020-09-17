// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use extreme;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::prelude::*;
use crate::runtime::RastClient;
use crate::testutil;
use crate::testutil::ConcurrentGroup;

pub enum OpReq {
  Write(WriteReq),
  Read(ReadReq),
}

pub enum Op {
  Write(WriteOp),
  Read(ReadOp),
}

impl fmt::Debug for Op {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Op::Read(op) => write!(f, "r[{:?}]", op),
      Op::Write(op) => write!(f, "w[{:?}]", op),
    }
  }
}

#[derive(Debug, Clone)]
pub struct ReadOp {
  worker_idx: u64,
  start: Instant,
  req: ReadReq,
  res: Result<ReadRes, ClientError>,
  finish: Instant,
}

#[derive(Debug, Clone)]
pub struct WriteOp {
  worker_idx: u64,
  start: Instant,
  req: WriteReq,
  res: Result<WriteRes, ClientError>,
  finish: Instant,
}

#[derive(Debug, Clone)]
pub struct Config {
  pub nodes: u64,
  pub workers: u64,
  pub ops: u64,
  pub read: u64,
  pub write: u64,
}

pub struct Generator {
  cfg: Config,
  idx: AtomicU64,
}

impl Generator {
  pub fn new(cfg: Config) -> Generator {
    Generator { cfg: cfg, idx: AtomicU64::new(0) }
  }

  pub fn op(&self, r: &mut impl Rng) -> OpReq {
    let ops: Vec<(u64, Box<dyn Fn() -> OpReq>)> =
      vec![(self.cfg.read, Box::new(|| self.read())), (self.cfg.write, Box::new(|| self.write()))];
    let total: u64 = ops.iter().map(|op| op.0).sum();
    let mut selected = r.gen_range(0, total) as i64;
    for op in ops {
      selected -= op.0 as i64;
      if selected < 0 {
        return op.1();
      }
    }
    unreachable!();
  }

  fn read(&self) -> OpReq {
    OpReq::Read(ReadReq { payload: vec![] })
  }

  fn write(&self) -> OpReq {
    let payload = self.idx.fetch_add(1, Ordering::SeqCst);
    let payload = format!("[{}]", payload);
    OpReq::Write(WriteReq { payload: payload.into_bytes() })
  }
}

pub struct Applier<'a> {
  cfg: Config,
  ops: Arc<AtomicU64>,
  gen: &'a Generator,
  c: RastClient,
}

impl<'a> Applier<'a> {
  pub fn new(cfg: Config, ops: Arc<AtomicU64>, gen: &'a Generator, c: RastClient) -> Applier<'a> {
    Applier { cfg: cfg, ops: ops, gen: gen, c: c }
  }

  pub async fn worker(&self, worker_idx: u64, rng: &mut impl Rng) -> Vec<Op> {
    let mut results = vec![];
    loop {
      let op_idx = self.ops.fetch_add(1, Ordering::SeqCst);
      if op_idx >= self.cfg.ops {
        return results;
      }
      let op = self.gen.op(rng);
      match op {
        OpReq::Read(req) => {
          let res = self.read(worker_idx, req).await;
          results.push(res);
        }
        OpReq::Write(req) => {
          let res = self.write(worker_idx, req).await;
          results.push(res);
        }
      }
    }
  }

  async fn read(&self, worker_idx: u64, req: ReadReq) -> Op {
    let start = Instant::now();
    let res = self.c.read(req.clone()).await;
    let finish = Instant::now();
    Op::Read(ReadOp { worker_idx: worker_idx, start: start, req: req, res: res, finish: finish })
  }

  async fn write(&self, worker_idx: u64, req: WriteReq) -> Op {
    let start = Instant::now();
    let res = self.c.write(req.clone()).await;
    let finish = Instant::now();
    Op::Write(WriteOp { worker_idx: worker_idx, start: start, req: req, res: res, finish: finish })
  }
}

pub fn nemesis_test(cfg: Config) -> Result<(), ValidateError> {
  let group = ConcurrentGroup::new(cfg.nodes);
  let workers = cfg.workers;
  let ops = Arc::new(AtomicU64::new(0));
  let threads = (0..workers).map(|worker_idx| {
    let cfg = cfg.clone();
    let ops = ops.clone();
    let generator = Generator::new(cfg.clone());
    // TODO: round robin
    let client = group.nodes.iter().next().unwrap().1.client();
    let thread_name = format!("worker-{}", worker_idx);
    thread::Builder::new()
      .name(thread_name)
      .spawn(move || {
        let a = Applier::new(cfg, ops, &generator, client);
        let mut rng = SmallRng::seed_from_u64(worker_idx);
        extreme::run(a.worker(worker_idx, &mut rng))
      })
      .expect("WIP")
  });
  let results: Vec<_> = threads.flat_map(|thread| thread.join().unwrap()).collect();
  validate(results)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidateError {
  errs: Vec<String>,
}

impl Display for ValidateError {
  fn fmt(&self, f: &mut Formatter) -> FmtResult {
    write!(f, "{:?}", self.errs)
  }
}

impl Error for ValidateError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    None
  }
}

// Invariants:
// - read that finishes entirely after write sees it
// - read that finishes entirely before write doesn't see it
// - write either succeeds or not
// - reads and writes from a single client are ordered
// - raft invariant: election safety
// - raft invariant: log matching

fn debug_print(payload: &Vec<u8>) -> &str {
  match std::str::from_utf8(payload) {
    Ok(payload) => payload,
    Err(_) => "",
  }
}

fn validate(mut ops: Vec<Op>) -> Result<(), ValidateError> {
  // Sort everything by index, all of the logic below depends on it. Since a
  // read of a given index _includes_ any writes at that index, sort writes
  // before reads.
  ops.sort_by_key(|op| match op {
    Op::Write(write) => write.res.clone().map(|res| (res.index, 0)).ok(),
    Op::Read(read) => read.res.clone().map(|res| (res.index, 1)).ok(),
  });

  let mut replicated_state: BTreeMap<Index, Vec<u8>> = BTreeMap::new();
  let mut errors: Vec<String> = Vec::new();
  for op in ops.iter() {
    match op {
      Op::Write(write) => write.res.iter().for_each(|res| {
        let mut new_state = replicated_state.iter().next_back().map_or(vec![], |(index, state)| {
          if *index >= res.index {
            todo!()
          }
          state.clone()
        });
        new_state.extend(write.req.payload.iter());
        replicated_state.insert(res.index, new_state);
      }),
      Op::Read(read) => read.res.iter().for_each(|res| {
        let expected =
          replicated_state.iter().next_back().map_or(vec![], |(_, state)| state.clone());
        if res.payload != expected {
          errors.push(format!(
            "read at {:?} expected {:?} got {:?}",
            res.index.0,
            debug_print(&expected),
            debug_print(&res.payload)
          ));
        }
      }),
    }
  }

  if errors.len() == 0 {
    Ok(())
  } else {
    Err(ValidateError { errs: errors })
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::wildcard_imports)]
  use super::*;

  fn read(index: u64, payload: &'static str) -> Op {
    Op::Read(ReadOp {
      worker_idx: 0,
      start: Instant::now(),
      finish: Instant::now(),
      req: ReadReq { payload: vec![] },
      res: Ok(ReadRes { term: Term(1), index: Index(index), payload: payload.as_bytes().to_vec() }),
    })
  }

  fn write(index: u64, payload: &'static str) -> Op {
    Op::Write(WriteOp {
      worker_idx: 0,
      start: Instant::now(),
      finish: Instant::now(),
      req: WriteReq { payload: payload.as_bytes().to_vec() },
      res: Ok(WriteRes { term: Term(1), index: Index(index) }),
    })
  }

  fn err(msg: &'static str) -> Result<(), ValidateError> {
    Err(ValidateError { errs: vec![msg.to_string()] })
  }

  #[test]
  fn validate_ops() {
    testutil::log_init();

    // Empty
    assert_eq!(validate(vec![]), Ok(()));

    // Single read of nothing
    assert_eq!(validate(vec![read(1, "")]), Ok(()));

    // Single write
    assert_eq!(validate(vec![write(1, "1")]), Ok(()));

    // RWRWR
    {
      #[rustfmt::skip]
      let ops = vec![
        read(1, ""),
        write(2, "2"),
        read(2, "2"),
        write(4, "4"),
        write(6, "24"),
      ];
      assert_eq!(validate(ops), Ok(()));
    }

    // Read value that wasn't written yet
    {
      #[rustfmt::skip]
      let ops = vec![
        read(1, "1"),
        write(2, "1"),
      ];
      assert_eq!(validate(ops), err(r#"read at 1 expected "" got "1""#));
    }

    // Writes in wrong order
    {
      #[rustfmt::skip]
      let ops = vec![
        write(1, "1"),
        write(2, "2"),
        read(2, "21"),
      ];
      assert_eq!(validate(ops), err(r#"read at 2 expected "12" got "21""#));
    }
  }

  #[test]
  fn nemesis_single() {
    testutil::log_init();
    let cfg = Config { nodes: 1, workers: 4, ops: 100, read: 50, write: 50 };
    let failures = nemesis_test(cfg);
    failures.expect("consistency violation");
  }

  #[test]
  fn nemesis_multi() {
    testutil::log_init();
    let cfg = Config { nodes: 3, workers: 4, ops: 100, read: 50, write: 50 };
    let failures = nemesis_test(cfg);
    failures.expect("consistency violation");
  }
}
