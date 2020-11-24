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

pub mod carsales_capnp {
  include!("../../runtime/src/samples/carsales_capnp.rs");
}
pub mod carsales;
pub mod catrank_capnp {
  include!("../../runtime/src/samples/catrank_capnp.rs");
}
pub mod catrank;
pub mod common;
pub mod eval_capnp {
  include!("../../runtime/src/samples/eval_capnp.rs");
}
pub mod error;
pub mod eval;

use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::thread;

use capnp_runtime::prelude::{CapnpAsRef, TypedStruct, TypedStructRef, TypedStructShared};
use capnp_runtime::segment_framing_alternate;
use os_pipe;

use crate::carsales::CarSales;
use crate::catrank::CatRank;
use crate::common::FastRand;
use crate::error::Error;
use crate::eval::Eval;

pub trait TestCase: Clone + Send + 'static {
  type Request: for<'a> TypedStruct<'a>;
  type Response: for<'a> TypedStruct<'a>;
  type Expectation;

  fn setup_request(
    &self,
    rnd: &mut FastRand,
  ) -> (<Self::Request as TypedStruct>::Shared, Self::Expectation);
  fn handle_request(
    &self,
    r: <Self::Request as TypedStruct>::Ref,
  ) -> Result<<Self::Response as TypedStruct>::Shared, Error>;
  fn check_response(
    &self,
    r: <Self::Response as TypedStruct>::Ref,
    e: Self::Expectation,
  ) -> Result<(), Error>;
}

pub trait Serialize: Clone + Send + 'static {
  fn read_message<T: TypedStructShared, R: BufRead>(&self, r: &mut R) -> Result<T, Error>;

  fn read_message_buf<'a, T: TypedStructRef<'a>>(&self, buf: &'a [u8]) -> Result<T, Error>;

  fn write_message<'a, W: Write, T: TypedStructRef<'a>>(
    &self,
    w: &mut W,
    message: T,
  ) -> Result<(), Error>;
}

#[derive(Clone)]
pub struct NoCompression;

impl Serialize for NoCompression {
  fn read_message<T: TypedStructShared, R: BufRead>(&self, r: &mut R) -> Result<T, Error> {
    let x = segment_framing_alternate::decode(r)?;
    Ok(x)
  }

  fn read_message_buf<'a, T: TypedStructRef<'a>>(&self, buf: &'a [u8]) -> Result<T, Error> {
    let x = segment_framing_alternate::decode_buf(buf)?;
    Ok(x)
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

#[derive(Clone)]
pub struct Packed;

impl Serialize for Packed {
  fn read_message<T: TypedStructShared, R: BufRead>(&self, _r: &mut R) -> Result<T, Error> {
    todo!()
  }

  fn read_message_buf<'a, T: TypedStructRef<'a>>(&self, _buf: &'a [u8]) -> Result<T, Error> {
    todo!()
  }

  fn write_message<'a, W: Write, T: TypedStructRef<'a>>(
    &self,
    _w: &mut W,
    _message: T,
  ) -> Result<(), Error> {
    todo!()
  }
}

pub trait Scratch<'a>: Clone + Send + 'static {
  // type Allocator: message::Allocator;

  // fn get_allocators(&'a mut self) -> (Self::Allocator, Self::Allocator);
}

const SCRATCH_WORDS: usize = 128 * 1024;
const SCRATCH_BYTES: usize = SCRATCH_WORDS * 8;

#[derive(Clone, Copy)]
pub struct NoScratch;

impl<'a> Scratch<'a> for NoScratch {
  // type Allocator = message::HeapAllocator;

  // fn get_allocators(&'a mut self) -> (Self::Allocator, Self::Allocator) {
  //   (message::HeapAllocator::new(), message::HeapAllocator::new())
  // }
}

#[derive(Clone)]
pub struct UseScratch {
  _buffer1: Vec<u8>,
  _buffer2: Vec<u8>,
}

impl UseScratch {
  pub fn new() -> UseScratch {
    UseScratch {
      _buffer1: Vec::with_capacity(SCRATCH_BYTES),
      _buffer2: Vec::with_capacity(SCRATCH_BYTES),
    }
  }
}

impl<'a> Scratch<'a> for UseScratch {
  // type Allocator = message::ScratchSpaceHeapAllocator<'a>;

  // fn get_allocators(&'a mut self) -> (Self::Allocator, Self::Allocator) {
  //   let UseScratch { ref mut buffer1, ref mut buffer2 } = self;
  //   (
  //     message::ScratchSpaceHeapAllocator::new(capnp::Word::words_to_bytes_mut(buffer1)),
  //     message::ScratchSpaceHeapAllocator::new(capnp::Word::words_to_bytes_mut(buffer2)),
  //   )
  // }
}

fn pass_by_object<S, T>(testcase: T, _reuse: S, iters: u64) -> Result<(), Error>
where
  S: for<'a> Scratch<'a>,
  T: TestCase,
{
  let mut rng = common::FastRand::new();
  for _ in 0..iters {
    let (req, expected) = testcase.setup_request(&mut rng);
    // TODO: It should be possible to call `req.capnp_as_ref()` instead of
    // roundtripping through UntypedStruct, but I couldn't get it to work even
    // after days of fiddling.
    let res = testcase.handle_request(
      <<T as TestCase>::Request as TypedStruct>::Ref::from_untyped_struct(
        req.as_untyped().capnp_as_ref(),
      ),
    )?;
    // TODO: It should be possible to call `res.capnp_as_ref()` instead of
    // roundtripping through UntypedStruct, but I couldn't get it to work even
    // after days of fiddling.
    testcase.check_response(
      <<T as TestCase>::Response as TypedStruct>::Ref::from_untyped_struct(
        res.as_untyped().capnp_as_ref(),
      ),
      expected,
    )?;
  }
  Ok(())
}

fn pass_by_bytes<C, S, T>(testcase: T, _reuse: S, compression: C, iters: u64) -> Result<(), Error>
where
  C: Serialize,
  S: for<'a> Scratch<'a>,
  T: TestCase,
{
  let mut request_bytes = vec![0u8; SCRATCH_BYTES];
  let mut response_bytes = vec![0u8; SCRATCH_BYTES];
  let mut rng = common::FastRand::new();
  for _ in 0..iters {
    let (req, expected) = testcase.setup_request(&mut rng);

    let res = {
      {
        request_bytes.clear();
        // WIP let mut writer: &mut [u8] = &mut request_bytes;
        // TODO: It should be possible to call `req.capnp_as_ref()` instead of
        // roundtripping through UntypedStruct, but I couldn't get it to work even
        // after days of fiddling.
        compression.write_message(
          &mut request_bytes,
          <<T as TestCase>::Request as TypedStruct>::Ref::from_untyped_struct(
            req.as_untyped().capnp_as_ref(),
          ),
        )?;
      }

      let mut request_bytes1: &[u8] = &request_bytes;
      let req: <T::Request as TypedStruct>::Ref =
        compression.read_message_buf(&mut request_bytes1)?;
      // eprintln!("req {:?}", &req);
      testcase.handle_request(req)?
    };

    {
      let mut writer: &mut [u8] = &mut response_bytes;
      // TODO: It should be possible to call `res.capnp_as_ref()` instead of
      // roundtripping through UntypedStruct, but I couldn't get it to work even
      // after days of fiddling.
      compression.write_message(
        &mut writer,
        <<T as TestCase>::Response as TypedStruct>::Ref::from_untyped_struct(
          res.as_untyped().capnp_as_ref(),
        ),
      )?;
    }

    let mut response_bytes1: &[u8] = &response_bytes;
    let res2 = compression.read_message_buf(&mut response_bytes1)?;

    testcase.check_response(res2, expected).map_err(|err| {
      // let mut request_bytes1: &[u8] = &request_bytes;
      // let req: <T::Request as TypedStruct>::Ref =
      //   compression.read_message_buf(&mut request_bytes1).unwrap();
      // eprintln!("req: {:?}", req);
      err
    })?;
  }
  Ok(())
}

fn server<C, S, T, R, W>(
  testcase: T,
  mut _reuse: S,
  compression: C,
  iters: u64,
  mut input: R,
  mut output: W,
) -> Result<(), Error>
where
  C: Serialize,
  S: for<'a> Scratch<'a>,
  T: TestCase,
  R: Read,
  W: Write,
{
  let mut out_buffered = BufWriter::new(&mut output);
  let mut in_buffered = BufReader::new(&mut input);
  for _ in 0..iters {
    let res = {
      let req: <<T as TestCase>::Request as TypedStruct>::Shared =
        compression.read_message(&mut in_buffered)?;
      // TODO: It should be possible to call `req.capnp_as_ref()` instead of
      // roundtripping through UntypedStruct, but I couldn't get it to work even
      // after days of fiddling.
      testcase.handle_request(
        <<T as TestCase>::Request as TypedStruct>::Ref::from_untyped_struct(
          req.as_untyped().capnp_as_ref(),
        ),
      )?
    };

    // TODO: It should be possible to call `res.capnp_as_ref()` instead of
    // roundtripping through UntypedStruct, but I couldn't get it to work even
    // after days of fiddling.
    compression.write_message(
      &mut out_buffered,
      <<T as TestCase>::Response as TypedStruct>::Ref::from_untyped_struct(
        res.as_untyped().capnp_as_ref(),
      ),
    )?;
    out_buffered.flush()?;
  }
  Ok(())
}

fn sync_client<C, S, T, R, W>(
  testcase: T,
  mut _reuse: S,
  compression: C,
  iters: u64,
  input: R,
  output: W,
) -> Result<(), Error>
where
  C: Serialize,
  S: for<'a> Scratch<'a>,
  T: TestCase,
  R: Read,
  W: Write,
{
  let mut in_buffered = BufReader::new(input);
  let mut out_buffered = BufWriter::new(output);
  let mut rng = common::FastRand::new();
  for _ in 0..iters {
    let (req, expected) = testcase.setup_request(&mut rng);
    // TODO: It should be possible to call `req.capnp_as_ref()` instead of
    // roundtripping through UntypedStruct, but I couldn't get it to work even
    // after days of fiddling.
    compression.write_message(
      &mut out_buffered,
      <<T as TestCase>::Request as TypedStruct>::Ref::from_untyped_struct(
        req.as_untyped().capnp_as_ref(),
      ),
    )?;
    out_buffered.flush()?;

    let res: <<T as TestCase>::Response as TypedStruct>::Shared =
      compression.read_message(&mut in_buffered)?;
    // TODO: It should be possible to call `res.capnp_as_ref()` instead of
    // roundtripping through UntypedStruct, but I couldn't get it to work even
    // after days of fiddling.
    testcase.check_response(
      <<T as TestCase>::Response as TypedStruct>::Ref::from_untyped_struct(
        res.as_untyped().capnp_as_ref(),
      ),
      expected,
    )?;
  }
  Ok(())
}

fn pass_by_pipe<C, S, T>(testcase: T, reuse: S, compression: C, iters: u64) -> Result<(), Error>
where
  C: Serialize,
  S: for<'a> Scratch<'a>,
  T: TestCase,
{
  let (server_in, client_out) = os_pipe::pipe()?;
  let (client_in, server_out) = os_pipe::pipe()?;

  let (testcase1, reuse1, compression1) = (testcase.clone(), reuse.clone(), compression.clone());

  let server =
    thread::spawn(move || server(testcase1, reuse1, compression1, iters, server_in, server_out));

  let client =
    thread::spawn(move || sync_client(testcase, reuse, compression, iters, client_in, client_out));

  client.join().or_else(|_| Err(Error::failed("client paniced".to_string())))??;
  server.join().or_else(|_| Err(Error::failed("server paniced".to_string())))??;
  Ok(())
}

pub enum Mode {
  Object,
  Bytes,
  Client,
  Server,
  Pipe,
}

impl Mode {
  pub fn parse(s: &str) -> Result<Mode, Error> {
    match s {
      "object" => Ok(Mode::Object),
      "bytes" => Ok(Mode::Bytes),
      "client" => Ok(Mode::Client),
      "server" => Ok(Mode::Server),
      "pipe" => Ok(Mode::Pipe),
      s => Err(Error::failed(format!("unrecognized mode: {}", s))),
    }
  }
}

fn do_testcase<C, S, T>(
  testcase: T,
  mode: Mode,
  reuse: S,
  compression: C,
  iters: u64,
) -> Result<(), Error>
where
  C: Serialize,
  S: for<'a> Scratch<'a>,
  T: TestCase,
{
  match mode {
    Mode::Object => pass_by_object(testcase, reuse, iters),
    Mode::Bytes => pass_by_bytes(testcase, reuse, compression, iters),
    Mode::Client => sync_client(testcase, reuse, compression, iters, io::stdin(), io::stdout()),
    Mode::Server => server(testcase, reuse, compression, iters, io::stdin(), io::stdout()),
    Mode::Pipe => pass_by_pipe(testcase, reuse, compression, iters),
  }
}

fn do_testcase1<C, S>(
  case: &str,
  mode: Mode,
  scratch: S,
  compression: C,
  iters: u64,
) -> Result<(), Error>
where
  C: Serialize,
  S: for<'a> Scratch<'a>,
{
  match case {
    "carsales" => do_testcase(CarSales, mode, scratch, compression, iters),
    "catrank" => do_testcase(CatRank, mode, scratch, compression, iters),
    "eval" => do_testcase(Eval, mode, scratch, compression, iters),
    s => Err(Error::failed(format!("unrecognized test case: {}", s))),
  }
}

fn do_testcase2<C>(
  case: &str,
  mode: Mode,
  scratch: &str,
  compression: C,
  iters: u64,
) -> Result<(), Error>
where
  C: Serialize,
{
  match scratch {
    "no-reuse" => do_testcase1(case, mode, NoScratch, compression, iters),
    "reuse" => do_testcase1(case, mode, UseScratch::new(), compression, iters),
    s => Err(Error::failed(format!("unrecognized reuse option: {}", s))),
  }
}

pub fn do_testcase3(
  case: &str,
  mode: &str,
  scratch: &str,
  compression: &str,
  iters: u64,
) -> Result<(), Error> {
  let mode = Mode::parse(mode)?;

  match compression {
    "none" => do_testcase2(case, mode, scratch, NoCompression, iters),
    "packed" => do_testcase2(case, mode, scratch, Packed, iters),
    s => Err(Error::failed(format!("unrecognized compression: {}", s))),
  }
}
