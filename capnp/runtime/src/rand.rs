// Copyright 2020 Daniel Harrison. All Rights Reserved.

use rand::Rng;

use crate::reflect::{
  ElementShared, ElementType, FieldMeta, ListDecodedElementShared, ListElementType,
  PointerElementShared, PointerElementType, PointerFieldMeta, PrimitiveElement,
  PrimitiveElementType, PrimitiveFieldMeta, StructElementShared, StructElementType, StructMeta,
  TypedStructShared, UnionElementShared, UnionElementType,
};
use crate::untyped::{UntypedStructOwned, UntypedStructShared};

trait RandElement<T> {
  fn gen<R: Rng>(&self, rng: &mut R) -> T;
}

impl RandElement<ElementShared> for ElementType {
  fn gen<R: Rng>(&self, rng: &mut R) -> ElementShared {
    match self {
      ElementType::Primitive(x) => ElementShared::Primitive(x.gen(rng)),
      ElementType::Pointer(x) => ElementShared::Pointer(x.gen(rng)),
      ElementType::Union(x) => ElementShared::Union(x.gen(rng)),
    }
  }
}

impl RandElement<PrimitiveElement> for PrimitiveElementType {
  fn gen<R: Rng>(&self, rng: &mut R) -> PrimitiveElement {
    match self {
      PrimitiveElementType::U8 => PrimitiveElement::U8(rng.gen()),
      PrimitiveElementType::U64 => PrimitiveElement::U64(rng.gen()),
    }
  }
}

impl RandElement<PointerElementShared> for PointerElementType {
  fn gen<R: Rng>(&self, rng: &mut R) -> PointerElementShared {
    match self {
      PointerElementType::Struct(x) => PointerElementShared::Struct(x.gen(rng)),
      PointerElementType::List(x) => PointerElementShared::ListDecoded(x.gen(rng)),
    }
  }
}

impl RandElement<StructElementShared> for StructElementType {
  fn gen<R: Rng>(&self, rng: &mut R) -> StructElementShared {
    StructElementShared(self.meta, gen_untyped_struct(rng, self.meta))
  }
}

impl RandElement<ListDecodedElementShared> for ListElementType {
  fn gen<R: Rng>(&self, rng: &mut R) -> ListDecodedElementShared {
    ListDecodedElementShared(self.meta, gen_element_list(rng, &self.meta.value_type))
  }
}

impl RandElement<UnionElementShared> for UnionElementType {
  fn gen<R: Rng>(&self, rng: &mut R) -> UnionElementShared {
    let variant_meta = &self.meta.variants[rng.gen_range(0, self.meta.variants.len())];
    UnionElementShared(
      self.meta,
      variant_meta.discriminant,
      Box::new(variant_meta.field_meta.element_type().gen(rng)),
    )
  }
}

fn gen_element_list<R: Rng>(rng: &mut R, value_type: &ElementType) -> Vec<ElementShared> {
  (0..rng.gen_range(0, 3)).map(|_| value_type.gen(rng)).collect()
}

pub fn gen_untyped_struct<R: Rng>(rng: &mut R, meta: &'static StructMeta) -> UntypedStructShared {
  let mut data = UntypedStructOwned::new_with_root_struct(meta.data_size, meta.pointer_size);
  for field_meta in meta.fields() {
    match field_meta {
      FieldMeta::Primitive(x) => match x {
        PrimitiveFieldMeta::U64(x) => x.set(&mut data, rng.gen()),
      },
      FieldMeta::Pointer(x) => match x {
        PointerFieldMeta::Struct(x) => {
          if rng.gen_bool(0.5) {
            let untyped = gen_untyped_struct(rng, x.meta);
            x.set_untyped(&mut data, x.meta, Some(&untyped))
          }
        }
        PointerFieldMeta::List(x) => {
          // Keep a limit on the recursion
          if let ElementType::Pointer(PointerElementType::Struct(_)) = x.meta.value_type {
            if rng.gen_bool(0.5) {
              continue;
            }
          }
          let l = ListElementType { meta: x.meta }.gen(rng);
          x.set_element(&mut data, &ElementShared::Pointer(PointerElementShared::ListDecoded(l)))
            .expect("WIP");
        }
      },
      FieldMeta::Union(x) => {
        x.set_element(&mut data, &ElementShared::Union(x.element_type().gen(rng))).expect("WIP");
      }
    }
  }
  data.into_shared()
}

pub fn gen_typed<T: TypedStructShared, R: Rng>(rng: &mut R) -> T {
  T::from_untyped_struct(gen_untyped_struct(rng, T::meta()))
}
