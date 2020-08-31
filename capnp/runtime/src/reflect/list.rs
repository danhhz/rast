// Copyright 2020 Daniel Harrison. All Rights Reserved.

use crate::common::{ElementWidth, NumElements};
use crate::error::Error;
use crate::pointer::ListLayout;
use crate::untyped::UntypedList;

pub trait TypedListElement<'a>: Sized {
  fn from_untyped_list(untyped: UntypedList<'a>) -> Result<Vec<Self>, Error>;
}

impl<'a> TypedListElement<'a> for u8 {
  fn from_untyped_list(untyped: UntypedList<'a>) -> Result<Vec<Self>, Error> {
    let num_elements = match untyped.pointer.layout {
      ListLayout::Packed(num_elements, ElementWidth::OneByte) => num_elements,
      _ => {
        dbg!(untyped.pointer.layout);
        return Err(Error("unsupported list layout for u8"));
      }
    };
    let list_elements_begin = untyped.pointer_end + untyped.pointer.off;
    let mut ret = Vec::new();
    for idx in 0..num_elements.0 {
      ret.push(list_elements_begin.u8(NumElements(idx)));
    }
    Ok(ret)
  }
}

impl<'a> TypedListElement<'a> for u64 {
  fn from_untyped_list(untyped: UntypedList<'a>) -> Result<Vec<Self>, Error> {
    let num_elements = match untyped.pointer.layout {
      ListLayout::Packed(num_elements, ElementWidth::FourBytes) => num_elements,
      _ => return Err(Error("unsupported list layout for u64")),
    };
    let list_elements_begin = untyped.pointer_end + untyped.pointer.off;
    let mut ret = Vec::new();
    for idx in 0..num_elements.0 {
      ret.push(list_elements_begin.u64(NumElements(idx)));
    }
    Ok(ret)
  }
}

impl<'a, T: TypedListElement<'a>> TypedListElement<'a> for Vec<T> {
  fn from_untyped_list(untyped: UntypedList<'a>) -> Result<Vec<Self>, Error> {
    Err(Error("unimplemented TypedListElement<'a> for Vec<T>"))
  }
}
