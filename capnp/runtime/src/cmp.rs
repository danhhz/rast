// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::cmp::{self, Ordering};

use crate::element::{ListDecodedElement, ListElement, StructElement, UnionElement};
use crate::field_meta::FieldMeta;
use crate::r#struct::StructMeta;
use crate::union::UnionMeta;

impl<'a> cmp::PartialOrd for StructElement<'a> {
  // TODO: Reason about whether this meets the guarantees for cmp::Ord too.
  fn partial_cmp(&self, other: &StructElement<'a>) -> Option<Ordering> {
    let StructElement(self_meta, self_untyped) = self;
    let StructElement(_, other_untyped) = other;

    // NB: This intentionally uses self.meta.fields for both.
    let self_meta_fields = self_meta.fields();

    // TODO: Avoid the double Vec allocation.
    let mut self_field_elements = Vec::with_capacity(self_meta_fields.len());
    let mut other_field_elements = Vec::with_capacity(self_meta_fields.len());
    for field_meta in self_meta_fields {
      // TODO: This is getting around an infinite recursion, but it's incorrect.
      // A null field should be treated as the default value for comparisons.
      match field_meta {
        FieldMeta::Pointer(x) => {
          if x.is_null(self_untyped) {
            self_field_elements.push(None);
          } else {
            self_field_elements.push(Some(field_meta.get_element(self_untyped).expect("WIP")));
          }
          if x.is_null(other_untyped) {
            other_field_elements.push(None);
          } else {
            other_field_elements.push(Some(field_meta.get_element(other_untyped).expect("WIP")));
          }
        }
        FieldMeta::Primitive(_) => {
          self_field_elements.push(Some(field_meta.get_element(self_untyped).expect("WIP")));
          other_field_elements.push(Some(field_meta.get_element(other_untyped).expect("WIP")));
        }
        FieldMeta::Union(_) => {
          self_field_elements.push(Some(field_meta.get_element(self_untyped).expect("WIP")));
          other_field_elements.push(Some(field_meta.get_element(other_untyped).expect("WIP")));
        }
      }
    }
    self_field_elements.partial_cmp(&other_field_elements)
  }
}

impl<'a> cmp::PartialEq for StructElement<'a> {
  fn eq(&self, other: &StructElement<'a>) -> bool {
    self.partial_cmp(other) == Some(Ordering::Equal)
  }
}

impl<'a> cmp::PartialOrd for ListElement<'a> {
  fn partial_cmp(&self, other: &ListElement<'a>) -> Option<Ordering> {
    let self_values = self.to_element_list().expect("WIP");
    let other_values = other.to_element_list().expect("WIP");
    self_values.partial_cmp(&other_values)
  }
}

impl<'a> cmp::PartialEq for ListElement<'a> {
  fn eq(&self, other: &ListElement<'a>) -> bool {
    self.partial_cmp(other) == Some(Ordering::Equal)
  }
}

impl<'a> cmp::PartialOrd for ListDecodedElement<'a> {
  fn partial_cmp(&self, other: &ListDecodedElement<'a>) -> Option<Ordering> {
    let ListDecodedElement(_, self_values) = self;
    let ListDecodedElement(_, other_values) = other;
    self_values.partial_cmp(&other_values)
  }
}

impl<'a> cmp::PartialEq for ListDecodedElement<'a> {
  fn eq(&self, other: &ListDecodedElement<'a>) -> bool {
    self.partial_cmp(other) == Some(Ordering::Equal)
  }
}

impl<'a> cmp::PartialOrd for UnionElement<'a> {
  fn partial_cmp(&self, other: &UnionElement<'a>) -> Option<Ordering> {
    let UnionElement(_, _, self_value) = self;
    let UnionElement(_, _, other_value) = other;
    self_value.partial_cmp(&other_value)
  }
}

impl<'a> cmp::PartialEq for UnionElement<'a> {
  fn eq(&self, other: &UnionElement<'a>) -> bool {
    self.partial_cmp(other) == Some(Ordering::Equal)
  }
}

impl<'a> cmp::PartialOrd for StructMeta {
  fn partial_cmp(&self, other: &StructMeta) -> Option<Ordering> {
    if self as *const StructMeta == other as *const StructMeta {
      Some(Ordering::Equal)
    } else {
      None
    }
  }
}

impl<'a> cmp::PartialEq for StructMeta {
  fn eq(&self, other: &StructMeta) -> bool {
    self.partial_cmp(other) == Some(Ordering::Equal)
  }
}

impl<'a> cmp::PartialOrd for UnionMeta {
  fn partial_cmp(&self, other: &UnionMeta) -> Option<Ordering> {
    if self as *const UnionMeta == other as *const UnionMeta {
      Some(Ordering::Equal)
    } else {
      None
    }
  }
}

impl<'a> cmp::PartialEq for UnionMeta {
  fn eq(&self, other: &UnionMeta) -> bool {
    self.partial_cmp(other) == Some(Ordering::Equal)
  }
}
