// Copyright 2017 Daniel Harrison. All Rights Reserved.

// TODO: Rustdoc for all of this.

use std::fmt::Display;
use std::io::{Result, Write};

pub trait StringTree<'a> {
  fn write(&'a self, _: &'a mut dyn Write) -> Result<()>;
}

pub trait AsStringTree<'a> {
  fn as_string_tree(&'a self) -> &'a dyn StringTree<'a>;
}

#[macro_export]
macro_rules! st {
  ($dst:expr, $($arg:expr),*) => ({
    $(
      {
        $arg.as_string_tree().write($dst)?;
      };
    )*
    Result::Ok(())
  });
}

#[macro_export]
macro_rules! stln {
  ($dst:expr) => ({
    $dst.write(b"\n")
  });
  ($dst:expr, $($arg:expr),*) => ({
    st!($dst, $($arg),*)?;
    $dst.write(b"\n")?;
    Result::Ok(())
  });
}

impl<'a, T: Display> StringTree<'a> for T {
  fn write(&'a self, w: &'a mut dyn Write) -> Result<()> {
    w.write_fmt(format_args!("{}", self))
  }
}

impl<'a, T: Display> AsStringTree<'a> for T {
  fn as_string_tree(&'a self) -> &'a dyn StringTree<'a> {
    self
  }
}

#[derive(Clone, Copy)]
pub struct StringTreeIndent {
  s: &'static str,
  level: usize,
}

impl StringTreeIndent {
  pub fn new(s: &'static str) -> StringTreeIndent {
    StringTreeIndent { s: s, level: 0 }
  }

  pub fn next(&self) -> StringTreeIndent {
    StringTreeIndent { s: self.s, level: self.level + 1 }
  }
}

impl<'a> StringTree<'a> for StringTreeIndent {
  fn write(&'a self, w: &'a mut dyn Write) -> Result<()> {
    for _ in 0..self.level {
      w.write(self.s.as_bytes())?;
    }
    Ok(())
  }
}

impl<'a> AsStringTree<'a> for StringTreeIndent {
  fn as_string_tree(&'a self) -> &'a dyn StringTree<'a> {
    self
  }
}

// pub struct StringTreeBuf {
//   sts: Vec<Box<dyn StringTree>>,
// }

// impl fmt::Display for StringTreeBuf {}

// impl StringTreeBuf {
//   pub fn new<I: Iterator<Item = Box<dyn StringTree>>>(sts: I) -> StringTreeBuf {
//     StringTreeBuf { sts: sts.collect() }
//   }
// }

// impl StringTree for StringTreeBuf {
//   fn read(&self, out: &mut String) {
//     for st in self.sts.iter() {
//       (**st).read(out);
//     }
//   }
// }

// impl fmt::Display for StringTreeIndent {}

// impl From<StringTreeIndent> for StringTree<'static> {
//   fn from(s: StringTreeIndent) -> Self {
//     StringTree(s)
//   }
// }

// impl StringTree for StringTreeIndent {
//   fn read(&self, out: &mut String) {
//     for _ in 0..self.level {
//       out.push_str(self.s);
//     }
//   }
// }

// pub struct StringTreeOption(Option<Box<dyn StringTree>>);

// impl fmt::Display for StringTreeOption {}

// impl StringTree for StringTreeOption {
//   fn read(&self, out: &mut String) {
//     if let Some(st) = self.0 {
//       st.read(out)
//     }
//   }
// }

// pub struct StringTreeFn(Box<dyn Fn() -> &'static str>);

// impl fmt::Display for StringTreeFn {}

// impl StringTree for StringTreeFn {
//   fn read(&self, out: &mut String) {
//     out.push_str(self.0());
//   }
// }

#[cfg(test)]
mod test {
  use super::*;
  use std::io::Result;

  #[test]
  fn test_foo() -> Result<()> {
    let i = StringTreeIndent::new("x").next();

    let mut out: Vec<u8> = Vec::new();
    stln!(&mut out, "foo", i, "bar")?;
    st!(&mut out, i.next(), "baz")?;
    assert_eq!(std::str::from_utf8(&out), Ok("fooxbar\nxxbaz"));
    Ok(())
  }

  // #[test]
  // fn test_stringtree_indent() {
  //   let i0 = StringTreeIndent::new(&"x");
  //   assert_eq!(i0.to_string(), "");
  //   assert_eq!(i0.next().to_string(), "x");
  //   assert_eq!(i0.next().next().to_string(), "xx");

  //   // None of the .next() calls should have mutated i0.
  //   assert_eq!(i0.to_string(), "");
  // }

  // fn strfoo() -> &'static str {
  //   "foo"
  // }

  // #[test]
  // fn test_stringtree_new() {
  //   assert_eq!(StringTree::new(&[]).to_string(), "");
  //   assert_eq!(StringTree::new(&[&"foo"]).to_string(), "foo");
  //   assert_eq!(StringTree::new(&[&"foo", &"bar"]).to_string(), "foobar");

  //   let i2 = StringTree::indent("x").next().next();
  //   assert_eq!(StringTree::new(&[&i2, &"bar"]).to_string(), "xxbar");

  //   let f: StringTreeFn = Box::new(strfoo);
  //   assert_eq!(StringTree::new(&[&f]).to_string(), "foo");
  //   assert_eq!(StringTree::new(&[&"bar", &f]).to_string(), "barfoo");

  //   let none: StringTreeOption = None;
  //   let c: &StringTree = &StringTree::new(&[&"opt"]);
  //   let some: StringTreeOption = Some(c);
  //   assert_eq!(StringTree::new(&[&none]).to_string(), "");
  //   assert_eq!(StringTree::new(&[&some]).to_string(), "opt");
  //   assert_eq!(StringTree::new(&[&"foo", &none, &some]).to_string(), "fooopt");
  // }
}
