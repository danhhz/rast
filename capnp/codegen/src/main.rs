// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::io;

#[allow(dead_code)]
#[rustfmt::skip]
mod schema_capnp;

#[cfg(test)]
mod golden_test;

#[allow(dead_code)]
mod generate;

fn main() {
  generate::Generator::generate(&mut io::stdin(), &mut io::stdout()).unwrap();
}
