// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::fmt::{self, Debug};

use crate::error::Error;
use crate::reflect::{Element, ElementSink, FieldMeta, StructMeta, ToElement};
use crate::untyped::UntypedStruct;

pub struct FmtDebugElementSink<'a, 'b> {
  pub has_fields: bool,
  pub fmt: &'a mut fmt::Formatter<'b>,
  pub result: fmt::Result,
}

impl<'a, 'b> ElementSink for FmtDebugElementSink<'a, 'b> {
  fn u8(&mut self, value: u8) {
    self.result = self.result.and_then(|_| value.fmt(self.fmt));
    self.has_fields = true;
  }

  fn u64(&mut self, value: u64) {
    self.result = self.result.and_then(|_| value.fmt(self.fmt));
    self.has_fields = true;
  }

  fn untyped_struct(&mut self, meta: &StructMeta, value: Result<UntypedStruct<'_>, Error>) {
    self.result = self.result.and_then(|_| {
      let data = match value {
        Ok(value) => value,
        Err(err) => return err.fmt(self.fmt),
      };

      self.fmt.write_str("(")?;
      self.has_fields = false;
      for field_meta in meta.fields {
        match field_meta {
          FieldMeta::Pointer(x) => {
            if x.is_null(&data) {
              continue;
            }
          }
          _ => {} // No-op
        }
        if self.has_fields {
          self.fmt.write_str(", ")?;
        }
        self.fmt.write_str(field_meta.name())?;
        self.fmt.write_str(" = ")?;
        field_meta.untyped_get(&data, self);
      }
      self.fmt.write_str(")")
    });
    self.has_fields = true;
  }

  fn list<'c>(&mut self, value: Result<Vec<&'c dyn ToElement<'c>>, Error>) {
    self.result = self.result.and_then(|_| {
      let data = match value {
        Ok(value) => value,
        Err(err) => return err.fmt(self.fmt),
      };

      let data: Result<Vec<Element<'c>>, Error> = data.iter().map(|x| x.to_element()).collect();
      match data {
        Ok(value) => value.fmt(self.fmt),
        Err(err) => err.fmt(self.fmt),
      }
    });
    self.has_fields = true;
  }
}
