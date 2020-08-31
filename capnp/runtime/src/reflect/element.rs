// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::fmt;

use crate::error::Error;
use crate::fmt_debug::FmtDebugElementSink;
use crate::reflect::{StructMeta, TypedStruct};
use crate::untyped::UntypedStruct;

pub enum Element<'a> {
  Primitive(PrimitiveElement),
  Pointer(PointerElement<'a>),
}

impl<'a> Element<'a> {
  pub fn untyped_get(self, sink: &mut dyn ElementSink) {
    match self {
      Element::Primitive(x) => x.untyped_get(sink),
      Element::Pointer(x) => x.untyped_get(sink),
    }
  }
}

impl<'a> fmt::Debug for Element<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Element::Primitive(x) => x.fmt(f),
      Element::Pointer(x) => x.fmt(f),
    }
  }
}

pub enum PrimitiveElement {
  U8(u8),
  U64(u64),
}

impl PrimitiveElement {
  pub fn untyped_get(self, sink: &mut dyn ElementSink) {
    match self {
      PrimitiveElement::U8(x) => sink.u8(x),
      PrimitiveElement::U64(x) => sink.u64(x),
    }
  }
}

impl fmt::Debug for PrimitiveElement {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      PrimitiveElement::U8(x) => x.fmt(f),
      PrimitiveElement::U64(x) => x.fmt(f),
    }
  }
}

pub enum PointerElement<'a> {
  Struct(&'static StructMeta, UntypedStruct<'a>),
  List(Vec<&'a dyn ToElement<'a>>),
}

impl<'a> PointerElement<'a> {
  pub fn untyped_get(self, sink: &mut dyn ElementSink) {
    match self {
      PointerElement::Struct(meta, untyped) => sink.untyped_struct(meta, Ok(untyped.clone())),
      PointerElement::List(x) => sink.list(Ok(x)),
    }
  }
}

impl<'a> fmt::Debug for PointerElement<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      PointerElement::Struct(meta, untyped) => {
        let mut sink = FmtDebugElementSink { has_fields: false, fmt: f, result: Ok(()) };
        sink.untyped_struct(meta, Ok(untyped.clone()));
        sink.result
      }
      PointerElement::List(x) => {
        let elements: Result<Vec<Element<'a>>, Error> = x.iter().map(|x| x.to_element()).collect();
        match elements {
          Err(_) => elements.fmt(f),
          Ok(data) => data.fmt(f),
        }
      }
    }
  }
}

pub trait ElementSink {
  fn u8(&mut self, value: u8);
  fn u64(&mut self, value: u64);
  fn untyped_struct(&mut self, meta: &'static StructMeta, value: Result<UntypedStruct<'_>, Error>);
  fn list<'a>(&mut self, value: Result<Vec<&'a dyn ToElement<'a>>, Error>);
}

pub trait ToElement<'a> {
  // TODO: Make this take ownership of self?
  fn to_element(&'a self) -> Result<Element<'a>, Error>;
}

impl<'a> ToElement<'a> for u8 {
  // TODO: Make an infallable version of ToElement for primitives and use that
  // to implement ToElement?
  fn to_element(&'a self) -> Result<Element<'a>, Error> {
    Ok(Element::Primitive(PrimitiveElement::U8(*self)))
  }
}

impl<'a> ToElement<'a> for u64 {
  // TODO: Make an infallable version of ToElement for primitives and use that
  // to implement ToElement?
  fn to_element(&'a self) -> Result<Element<'a>, Error> {
    Ok(Element::Primitive(PrimitiveElement::U64(*self)))
  }
}

impl<'a, T: TypedStruct<'a>> ToElement<'a> for T {
  fn to_element(&'a self) -> Result<Element<'a>, Error> {
    Ok(Element::Pointer(PointerElement::Struct(self.meta(), self.to_untyped())))
  }
}

impl<'a, T: ToElement<'a>> ToElement<'a> for Vec<T> {
  fn to_element(&'a self) -> Result<Element<'a>, Error> {
    let list: Vec<&'a dyn ToElement<'a>> =
      self.iter().map(|x| x as &'a dyn ToElement<'a>).collect();
    Ok(Element::Pointer(PointerElement::List(list)))
  }
}

pub trait ToElementList<'a> {
  fn to_element_list(&'a self) -> Result<Vec<&'a dyn ToElement<'a>>, Error>;
}

impl<'a, T: 'a + ToElement<'a>> ToElementList<'a> for Result<Vec<T>, Error> {
  fn to_element_list(&'a self) -> Result<Vec<&'a dyn ToElement<'a>>, Error> {
    match self {
      Err(err) => Err(err.clone()),
      Ok(xs) => Ok(xs.iter().map(|x| x as &'a dyn ToElement<'a>).collect()),
    }
  }
}
