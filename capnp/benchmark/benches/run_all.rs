// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::time::Instant;

use criterion::{criterion_group, criterion_main, Criterion};

use benchmark;

fn run_one(c: &mut Criterion, case: &str, mode: &str, scratch: &str, compression: &str) {
  let name = format!("{} {} {} {}", case, mode, scratch, compression);
  c.bench_function(&name, |b| {
    b.iter_custom(|iters| {
      let start = Instant::now();
      benchmark::do_testcase3(case, mode, scratch, compression, iters).expect("benchmark failed");
      start.elapsed()
    })
  });
}

fn run_case(c: &mut Criterion, case: &str, scratch_options: &[&str]) {
  for scratch in scratch_options {
    run_one(c, case, "object", scratch, "none");
  }

  for mode in &["bytes", "pipe"] {
    for compression in &["none" /* TODO "packed" */] {
      for scratch in scratch_options {
        run_one(c, case, mode, scratch, compression);
      }
    }
  }
}

const CARSALES_SCRATCH: &'static [&'static str] = &["reuse", "no-reuse"];
const CATRANK_SCRATCH: &'static [&'static str] = &["no-reuse"];
const EVAL_SCRATCH: &'static [&'static str] = &["no-reuse"];

fn bench_carsales(c: &mut Criterion) {
  run_case(c, "carsales", CARSALES_SCRATCH)
}

fn bench_catrank(c: &mut Criterion) {
  run_case(c, "catrank", CATRANK_SCRATCH)
}

fn bench_eval(c: &mut Criterion) {
  run_case(c, "eval", EVAL_SCRATCH)
}
criterion_group!(benches, bench_carsales, bench_catrank, bench_eval);
criterion_main!(benches);
