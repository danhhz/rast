// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::error::Error;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

use crate::Generator;

fn read_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, io::Error> {
  let mut f = File::open(path)?;
  let mut buf = Vec::new();
  f.read_to_end(&mut buf)?;
  Ok(buf)
}

fn write_file<P: AsRef<Path>>(path: P, mut contents: &[u8]) -> Result<(), io::Error> {
  let mut f = File::create(path)?;
  f.write_all(&mut contents)?;
  Ok(())
}

fn golden_capnp(file: &'static str) -> Result<(), Box<dyn Error>> {
  let input_path = format!("../runtime/src/samples/{}.capnp.bin", file);
  let expected_path = format!("../runtime/src/samples/{}_capnp.rs", file);

  let overwrite = std::env::args().any(|x| x == "--overwrite");

  let mut input = File::open(&input_path).unwrap();
  let mut output = Vec::new();
  Generator::generate(&mut input, &mut output)?;

  let expected = read_file(&expected_path);
  if overwrite && expected.as_ref().map_or(true, |e| !output.eq(e)) {
    if let Some(err) = write_file(&expected_path, &mut output).err() {
      eprintln!("failed to update golden {}: {}", &expected_path, err);
    }
  }
  assert_eq!(output, expected?);
  Ok(())
}

#[test]
fn golden_test_capnp() -> Result<(), Box<dyn Error>> {
  golden_capnp("test")
}

#[test]
fn golden_rast_capnp() -> Result<(), Box<dyn Error>> {
  golden_capnp("rast")
}
