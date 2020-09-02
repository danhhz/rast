// Copyright 2020 Daniel Harrison. All Rights Reserved.

use crate::error::Error;
use crate::reflect::{
  ElementType, ListElementType, ListMeta, PointerElementType, PrimitiveElementType,
  StructElementType, StructMeta, TypedList,
};
use crate::untyped::{UntypedList, UntypedListShared, UntypedStruct, UntypedStructShared};

#[derive(PartialEq, PartialOrd)]
pub enum Element<'a> {
  Primitive(PrimitiveElement),
  Pointer(PointerElement<'a>),
}

impl<'a> Element<'a> {
  pub fn element_type(&self) -> ElementType {
    match self {
      Element::Primitive(x) => ElementType::Primitive(x.element_type()),
      Element::Pointer(x) => ElementType::Pointer(x.element_type()),
    }
  }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum PrimitiveElement {
  // TODO: Break these out into U64Element, etc?
  // TODO: Derive Copy?
  U8(u8),
  U64(u64),
}

impl PrimitiveElement {
  pub fn element_type(&self) -> PrimitiveElementType {
    match self {
      PrimitiveElement::U8(x) => PrimitiveElementType::U8,
      PrimitiveElement::U64(x) => PrimitiveElementType::U8,
    }
  }
}

#[derive(PartialEq, PartialOrd)]
pub enum PointerElement<'a> {
  Struct(StructElement<'a>),
  List(ListElement<'a>),
  ListDecoded(ListDecodedElement<'a>),
}

impl<'a> PointerElement<'a> {
  pub fn element_type(&self) -> PointerElementType {
    match self {
      PointerElement::Struct(x) => PointerElementType::Struct(x.element_type()),
      PointerElement::List(x) => PointerElementType::List(x.element_type()),
      PointerElement::ListDecoded(x) => PointerElementType::List(x.element_type()),
    }
  }
}

pub struct StructElement<'a>(pub &'static StructMeta, pub UntypedStruct<'a>);

impl<'a> StructElement<'a> {
  pub fn from_untyped_list(
    meta: &'static StructMeta,
    untyped: &UntypedList<'a>,
  ) -> Result<Vec<Self>, Error> {
    Vec::<UntypedStruct<'a>>::from_untyped_list(untyped)
      .map(|xs| xs.into_iter().map(|x| StructElement(meta, x)).collect())
  }

  pub fn element_type(&self) -> StructElementType {
    let StructElement(meta, _) = self;
    StructElementType { meta: meta }
  }
}

pub struct ListElement<'a>(pub &'static ListMeta, pub UntypedList<'a>);

impl<'a> ListElement<'a> {
  pub fn to_element_list(&self) -> Result<Vec<Element<'a>>, Error> {
    self.0.value_type.to_element_list(&self.1)
  }

  pub fn from_untyped_list(
    values: &ElementType,
    untyped: &UntypedList<'a>,
  ) -> Result<Vec<Self>, Error> {
    todo!()
  }

  pub fn element_type(&self) -> ListElementType {
    let ListElement(meta, _) = self;
    ListElementType { meta: meta }
  }
}

// TODO: It'd be nice to make this Vec a slice instead.
pub struct ListDecodedElement<'a>(pub &'static ListMeta, pub Vec<Element<'a>>);

impl<'a> ListDecodedElement<'a> {
  pub fn element_type(&self) -> ListElementType {
    let ListDecodedElement(meta, _) = self;
    ListElementType { meta: meta }
  }
}

pub enum ElementShared {
  Primitive(PrimitiveElement),
  Pointer(PointerElementShared),
}

impl ElementShared {
  pub fn as_ref<'a>(&'a self) -> Element<'a> {
    match self {
      ElementShared::Primitive(x) => Element::Primitive(x.clone()),
      ElementShared::Pointer(x) => Element::Pointer(x.as_ref()),
    }
  }
}

pub enum PointerElementShared {
  Struct(StructElementShared),
  List(ListElementShared),
  ListDecoded(ListDecodedElementShared),
}

impl PointerElementShared {
  pub fn as_ref<'a>(&'a self) -> PointerElement<'a> {
    match self {
      PointerElementShared::Struct(x) => PointerElement::Struct(x.as_ref()),
      PointerElementShared::List(x) => PointerElement::List(x.as_ref()),
      PointerElementShared::ListDecoded(x) => PointerElement::ListDecoded(x.as_ref()),
    }
  }
}

pub struct StructElementShared(pub &'static StructMeta, pub UntypedStructShared);

impl StructElementShared {
  pub fn as_ref<'a>(&'a self) -> StructElement<'a> {
    let StructElementShared(meta, untyped) = self;
    StructElement(meta, untyped.as_ref())
  }
}

pub struct ListElementShared(pub &'static ListMeta, pub UntypedListShared);

impl ListElementShared {
  pub fn as_ref<'a>(&'a self) -> ListElement<'a> {
    let ListElementShared(meta, untyped) = self;
    ListElement(meta, untyped.as_ref())
  }
}

pub struct ListDecodedElementShared(pub &'static ListMeta, pub Vec<ElementShared>);

impl ListDecodedElementShared {
  pub fn as_ref<'a>(&'a self) -> ListDecodedElement<'a> {
    let ListDecodedElementShared(meta, values) = self;
    ListDecodedElement(meta, values.iter().map(|v| v.as_ref()).collect())
  }
}

// pub trait ToElement<'a> {
//   // TODO: Make this take ownership of self?
//   fn to_element(&'a self) -> Result<Element<'a>, Error>;
// }

// impl<'a> ToElement<'a> for u8 {
//   // TODO: Make an infallable version of ToElement for primitives and use that
//   // to implement ToElement?
//   fn to_element(&'a self) -> Result<Element<'a>, Error> {
//     Ok(Element::Primitive(PrimitiveElement::U8(*self)))
//   }
// }

// impl<'a> ToElement<'a> for u64 {
//   // TODO: Make an infallable version of ToElement for primitives and use that
//   // to implement ToElement?
//   fn to_element(&'a self) -> Result<Element<'a>, Error> {
//     Ok(Element::Primitive(PrimitiveElement::U64(*self)))
//   }
// }

// impl<'a, T: TypedStruct<'a>> ToElement<'a> for T {
//   fn to_element(&'a self) -> Result<Element<'a>, Error> {
//     Ok(Element::Pointer(PointerElement::Struct(StructElement(T::meta(), self.as_untyped()))))
//   }
// }

// impl<'a, T: ToElement<'a>> ToElement<'a> for Vec<T> {
//   fn to_element(&'a self) -> Result<Element<'a>, Error> {
//     let list: Vec<&'a dyn ToElement<'a>> =
//       self.iter().map(|x| x as &'a dyn ToElement<'a>).collect();
//     Ok(Element::Pointer(PointerElement::List(ListElement(list))))
//   }
// }

// pub trait ToElementList<'a> {
//   fn to_element_list(&'a self) -> Result<Vec<&'a dyn ToElement<'a>>, Error>;
// }

// impl<'a, T: 'a + ToElement<'a>> ToElementList<'a> for Result<Vec<T>, Error> {
//   fn to_element_list(&'a self) -> Result<Vec<&'a dyn ToElement<'a>>, Error> {
//     match self {
//       Err(err) => Err(err.clone()),
//       Ok(xs) => Ok(xs.iter().map(|x| x as &'a dyn ToElement<'a>).collect()),
//     }
//   }
// }
