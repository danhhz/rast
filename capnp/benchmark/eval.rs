// Copyright (c) 2013-2014 Sandstorm Development Group, Inc. and contributors
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

#![allow(unreachable_code)]

use capnp_runtime::prelude::{Discriminant, Error, TypedEnum};

use crate::common::{self, FastRand};
use crate::eval_capnp::{
  EvaluationResultMeta, EvaluationResultRef, EvaluationResultShared, ExpressionMeta, ExpressionRef,
  ExpressionShared, Left, LeftShared, Operation, Right, RightShared,
};

fn make_expression(rng: &mut FastRand, depth: u32) -> (ExpressionShared, i32) {
  let op: Operation = Operation::from_discriminant(Discriminant(
    rng.next_less_than(Operation::meta().enumerants.len() as u32) as u16,
  ))
  .expect("WIP");

  let left = if rng.next_less_than(8) < depth {
    // LeftShared::Value((rng.next_less_than(128) + 1) as i32)
    todo!()
  } else {
    let (expr, val) = make_expression(rng, depth + 1);
    (LeftShared::Expression(expr), val)
  };

  let right = if rng.next_less_than(8) < depth {
    // RightShared::Value((rng.next_less_than(128) + 1) as i32)
    todo!()
  } else {
    let (expr, val) = make_expression(rng, depth + 1);
    (RightShared::Expression(expr), val)
  };

  let expr = ExpressionShared::new(op, left.0, right.0);
  let val = match op {
    Operation::Add => left.1 + right.1,
    Operation::Subtract => left.1 - right.1,
    Operation::Multiply => left.1 * right.1,
    Operation::Divide => common::div(left.1, right.1),
    Operation::Modulus => common::modulus(left.1, right.1),
  };
  (expr, val)
}

fn evaluate_expression(exp: &ExpressionRef) -> Result<i32, Error> {
  let left = match exp.left()?.expect("WIP") {
    Left::Value(v) => v,
    Left::Expression(e) => evaluate_expression(&e)?,
  };
  let right = match exp.right()?.expect("WIP") {
    Right::Value(v) => v,
    Right::Expression(e) => evaluate_expression(&e)?,
  };

  match exp.op().expect("WIP") {
    Operation::Add => Ok(left + right),
    Operation::Subtract => Ok(left - right),
    Operation::Multiply => Ok(left * right),
    Operation::Divide => Ok(common::div(left, right)),
    Operation::Modulus => Ok(common::modulus(left, right)),
  }
}

pub struct Eval;

impl crate::TestCase for Eval {
  type Request = ExpressionMeta;
  type Response = EvaluationResultMeta;
  type Expectation = i32;

  fn setup_request(&self, rng: &mut FastRand) -> (ExpressionShared, i32) {
    make_expression(rng, 0)
  }

  fn handle_request(&self, req: &ExpressionRef<'_>) -> Result<EvaluationResultShared, Error> {
    let value = evaluate_expression(req)?;
    Ok(EvaluationResultShared::new(value))
  }

  fn check_response(&self, res: &EvaluationResultRef<'_>, expected: i32) -> Result<(), Error> {
    if res.value() == expected {
      Ok(())
    } else {
      Err(Error::Usage(format!("check_response() expected {} but got {}", expected, "WIP")))
    }
  }
}
