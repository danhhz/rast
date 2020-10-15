// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::fmt::{self, Write};
use std::str;

use crate::common::CapnpAsRef;
use crate::element::{
  DataElement, Element, ElementShared, ListDecodedElement, ListElement, StructElement, UnionElement,
};
use crate::error::UnknownDiscriminant;
use crate::r#struct::StructMeta;

impl fmt::Debug for StructMeta {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("StructMeta")
      .field("name", &self.name)
      .field("data_size", &self.data_size)
      .field("pointer_size", &self.pointer_size)
      // NB: Can't just print out the fields here or we'll get infinite
      // recursion in self-referencing struct types.
      .field("fields", &"TODO")
      .finish()
  }
}

impl fmt::Debug for ElementShared {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.capnp_as_ref().fmt(f)
  }
}

impl<'a> fmt::Debug for Element<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Element::U8(x) => x.fmt(f),
      Element::U64(x) => x.fmt(f),
      Element::Data(x) => x.fmt(f),
      Element::Struct(x) => x.fmt(f),
      Element::List(x) => x.fmt(f),
      Element::ListDecoded(x) => x.fmt(f),
      Element::Union(x) => x.fmt(f),
    }
  }
}

impl<'a> fmt::Debug for DataElement<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match str::from_utf8(self.0) {
      // Attempt to print the bytes as a string if that seems "reasonable".
      Ok(x) if x.chars().all(|x| x.is_alphanumeric()) => x.fmt(f),
      // Otherwise print it as hex. This disagrees with the official capnp
      // format, which prints all bytes types as strings.
      _ => {
        f.write_str("[")?;
        let mut first_value = true;
        for x in self.0 {
          if !first_value {
            f.write_str(", ")?;
          }
          first_value = false;
          write!(f, "{:02x}", x)?;
        }
        f.write_str("]")
      }
    }
  }
}

impl<'a> fmt::Debug for StructElement<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let StructElement(meta, untyped) = self;
    f.write_str("(")?;
    let mut has_fields = false;
    for field_meta in meta.fields() {
      if field_meta.is_null(untyped) {
        continue;
      }
      let first_field = !has_fields;
      has_fields = true;
      if f.alternate() {
        f.write_str("\n")?;
        let mut writer = PaddedWriter::new(f);
        writer.write_str(field_meta.name())?;
        writer.write_str(" = ")?;
        match field_meta.get_element(untyped) {
          Ok(x) => write!(&mut writer, "{:#?}", x)?,
          x => write!(&mut writer, "{:#?}", x)?,
        };
        writer.write_str(",")?;
      } else {
        if !first_field {
          f.write_str(", ")?;
        }
        f.write_str(field_meta.name())?;
        f.write_str(" = ")?;
        match field_meta.get_element(untyped) {
          Ok(x) => x.fmt(f)?,
          x => x.fmt(f)?,
        };
      }
    }
    if f.alternate() {
      f.write_str("\n)")
    } else {
      f.write_str(")")
    }
  }
}

impl<'a> fmt::Debug for ListElement<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let values = match self.to_element_list() {
      Ok(x) => x,
      Err(err) => {
        f.write_str("Err(")?;
        err.fmt(f)?;
        return f.write_str(")");
      }
    };

    f.write_str("[")?;
    let mut has_fields = false;
    for value in values {
      if has_fields {
        f.write_str(", ")?;
      }
      has_fields = true;
      value.fmt(f)?;
    }
    f.write_str("]")
  }
}

impl<'a> fmt::Debug for ListDecodedElement<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let ListDecodedElement(_, values) = self;
    values.as_slice().fmt(f)
  }
}

impl<'a> fmt::Debug for UnionElement<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let UnionElement(meta, discriminant, value) = self;
    match meta.get(*discriminant) {
      Some(variant) => {
        f.write_str("(")?;
        f.write_str(variant.field_meta.name())?;
        f.write_str(" = ")?;
        value.fmt(f)?;
        f.write_str(")")
      }
      None => UnknownDiscriminant(*discriminant, meta.name).fmt(f),
    }
  }
}

// The following is forked from the rust stdlib PadAdapter.

struct PaddedWriter<'a, 'b> {
  fmt: &'b mut fmt::Formatter<'a>,
  on_newline: bool,
}

impl<'a, 'b> PaddedWriter<'a, 'b> {
  fn new(fmt: &'b mut fmt::Formatter<'a>) -> Self {
    PaddedWriter { fmt: fmt, on_newline: true }
  }
}

impl fmt::Write for PaddedWriter<'_, '_> {
  fn write_str(&mut self, mut s: &str) -> fmt::Result {
    while !s.is_empty() {
      if self.on_newline {
        self.fmt.write_str("  ")?;
      }

      let split = match s.find('\n') {
        Some(pos) => {
          self.on_newline = true;
          pos + 1
        }
        None => {
          self.on_newline = false;
          s.len()
        }
      };
      self.fmt.write_str(&s[..split])?;
      s = &s[split..];
    }

    Ok(())
  }
}

#[cfg(test)]
mod test {
  use std::error;

  use crate::samples::test_capnp::TestAllTypes;

  use capnp_runtime::segment_framing_official;

  #[test]
  fn fmt_debug_short() -> Result<(), Box<dyn error::Error>> {
    let buf = include_bytes!("../testdata/binary");
    let message: TestAllTypes = segment_framing_official::decode(buf)?;
    let expected = include_str!("../testdata/short.txt");
    assert_eq!(format!("{:?}", message), expected);
    Ok(())
  }

  #[test]
  fn fmt_debug_pretty() -> Result<(), Box<dyn error::Error>> {
    let buf = include_bytes!("../testdata/binary");
    let message: TestAllTypes = segment_framing_official::decode(buf)?;
    // TODO: This doesn't exactly match the official capnp pretty format, which
    // does some work to smush things on one line if they fit.
    let expected = include_str!("../testdata/pretty.txt");
    assert_eq!(format!("{:#?}", message), expected);
    Ok(())
  }
}
