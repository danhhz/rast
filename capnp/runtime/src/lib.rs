// Copyright 2020 Daniel Harrison. All Rights Reserved.

mod cmp;
mod common;
mod decode;
mod element;
mod element_type;
mod encode;
mod error;
mod field_meta;
mod fmt_debug;
mod list;
mod pointer;
mod segment;
mod segment_pointer;
mod r#struct;
mod union;

pub mod prelude {
  pub use super::common::{Discriminant, NumElements, NumWords};
  pub use crate::element_type::{
    ElementType, ListElementType, PointerElementType, PrimitiveElementType, StructElementType,
    UnionElementType,
  };
  pub use crate::error::Error;
  pub use crate::field_meta::{
    FieldMeta, ListFieldMeta, PointerFieldMeta, PrimitiveFieldMeta, StructFieldMeta, U64FieldMeta,
    UnionFieldMeta,
  };
  pub use crate::list::ListMeta;
  pub use crate::r#struct::{
    StructMeta, TypedStruct, TypedStructShared, UntypedStruct, UntypedStructOwned,
    UntypedStructShared,
  };
  pub use crate::union::{TypedUnion, TypedUnionShared, UnionMeta, UnionVariantMeta, UntypedUnion};

  // TODO: Remove these from prelude
  pub use crate::segment::{decode_stream_alternate, decode_stream_official};
  pub use crate::segment_pointer::SegmentPointer;
}

#[cfg(feature = "serde")]
pub mod serde;

#[cfg(feature = "rand")]
pub mod rand;
