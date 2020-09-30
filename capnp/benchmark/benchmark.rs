// Copyright (c) 2013-2017 Sandstorm Development Group, Inc. and contributors
// Licensed under the MIT License:
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

// pub mod carsales;
// pub mod catrank;
pub mod common;
pub mod eval_capnp {
  include!("../runtime/src/samples/eval_capnp.rs");
}
pub mod eval;

use std::env;
use std::io::{BufRead, Write};

use capnp_runtime::prelude::{CapnpAsRef, Error, TypedStruct, TypedStructRef};
use capnp_runtime::segment_framing_alternate;

use crate::common::FastRand;

trait TestCase {
  type Request: for<'a> TypedStruct<'a>;
  type Response: for<'a> TypedStruct<'a>;
  type Expectation;

  fn setup_request(
    &self,
    rnd: &mut FastRand,
  ) -> (<Self::Request as TypedStruct>::Shared, Self::Expectation);
  fn handle_request(
    &self,
    r: &<Self::Request as TypedStruct>::Ref,
  ) -> Result<<Self::Response as TypedStruct>::Shared, Error>;
  fn check_response(
    &self,
    r: &<Self::Response as TypedStruct>::Ref,
    e: Self::Expectation,
  ) -> Result<(), Error>;
}

trait Serialize {
  fn read_message<'a, T: TypedStructRef<'a>>(&self, buf: &'a [u8]) -> Result<T, Error>;

  fn write_message<'a, W: Write, T: TypedStructRef<'a>>(
    &self,
    w: &mut W,
    message: T,
  ) -> Result<(), Error>;
}

struct NoCompression;

impl Serialize for NoCompression {
  fn read_message<'a, T: TypedStructRef<'a>>(&self, buf: &'a [u8]) -> Result<T, Error> {
    segment_framing_alternate::decode(buf)
  }

  fn write_message<'a, W: Write, T: TypedStructRef<'a>>(
    &self,
    w: &mut W,
    message: T,
  ) -> Result<(), Error> {
    segment_framing_alternate::encode(w, &message)?;
    Ok(())
  }
}

fn pass_by_object<T: TestCase>(testcase: T, iters: u64) -> Result<(), Error> {
  let mut rng = FastRand::new();
  for _ in 0..iters {
    let (req, expected) = testcase.setup_request(&mut rng);
    let res = testcase.handle_request(&req.capnp_as_ref())?;
    testcase.check_response(&res.capnp_as_ref(), expected)?;
  }
  Ok(())
}

fn pass_by_bytes<T: TestCase>(testcase: T, iters: u64) -> Result<(), Error> {
  let mut rng = FastRand::new();
  let (mut req_bytes, mut res_bytes) = (Vec::new(), Vec::new());
  for _ in 0..iters {
    {
      let (req, expected) = testcase.setup_request(&mut rng);
      NoCompression.write_message(&mut req_bytes, req.capnp_as_ref());
      let req: <T::Request as TypedStruct>::Ref =
        NoCompression.read_message(&mut req_bytes.as_slice())?;
      let res = testcase.handle_request(&req)?;
      {
        let res_ref = res.capnp_as_ref();
        NoCompression.write_message(&mut res_bytes, res_ref);
      }
      let res = NoCompression.read_message(&mut res_bytes.as_slice())?;
      testcase.check_response(&res, expected)?;
    }
    req_bytes.clear();
    res_bytes.clear();
  }
  Ok(())
}

pub enum Mode {
  Object,
  Bytes,
}

impl Mode {
  pub fn parse(s: &str) -> Result<Mode, Error> {
    match s {
      "object" => Ok(Mode::Object),
      "bytes" => Ok(Mode::Bytes),
      s => Err(Error::Usage(format!("unrecognized mode: {}", s))),
    }
  }
}

fn do_testcase<T: TestCase>(testcase: T, mode: Mode, iters: u64) -> Result<(), Error> {
  match mode {
    Mode::Object => pass_by_object(testcase, iters),
    Mode::Bytes => pass_by_bytes(testcase, iters),
  }
}

fn do_testcase_name(case: &str, mode: Mode, iters: u64) -> Result<(), Error> {
  match case {
    // "carsales" => do_testcase(carsales::CarSales, mode, scratch, compression, iters),
    // "catrank" => do_testcase(catrank::CatRank, mode, scratch, compression, iters),
    "eval" => do_testcase(eval::Eval, mode, iters),
    s => Err(Error::Usage(format!("unrecognized test case: {}", s))),
  }
}

fn try_main() -> Result<(), Error> {
  let args: Vec<String> = env::args().collect();

  assert!(args.len() == 6, "USAGE: {} CASE MODE ITERATION_COUNT", args[0]);

  let iters = match args[5].parse::<u64>() {
    Ok(n) => n,
    Err(_) => return Err(Error::Usage(format!("Could not parse a u64 from: {}", args[5]))),
  };

  let mode = Mode::parse(&*args[2])?;
  do_testcase_name(&*args[1], mode, iters)
}

pub fn main() {
  match try_main() {
    Ok(()) => (),
    Err(e) => {
      panic!("error: {:?}", e);
    }
  }
}
