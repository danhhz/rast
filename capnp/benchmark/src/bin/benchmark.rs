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

use benchmark;
use benchmark::error::Error;

fn try_main() -> Result<(), Error> {
  let args: Vec<String> = ::std::env::args().collect();

  assert!(args.len() == 6, "USAGE: {} CASE MODE REUSE COMPRESSION ITERATION_COUNT", args[0]);

  let iters = match args[5].parse::<u64>() {
    Ok(n) => n,
    Err(_) => return Err(Error::failed(format!("Could not parse a u64 from: {}", args[5]))),
  };

  benchmark::do_testcase3(&*args[1], &*args[2], &*args[3], &*args[4], iters)
}

pub fn main() {
  match try_main() {
    Ok(()) => (),
    Err(e) => {
      panic!("error: {:?}", e);
    }
  }
}
