// Copyright 2020 Daniel Harrison. All Rights Reserved.

//! An implemention of the [Cap'n Proto encoding]
//!
//! [cap'n proto encoding]: https://capnproto.org/encoding.html
//!
//! [Cap'n Proto] is a zero-allocation, zero-copy, lazily-decoded serialization
//! format. Messages are defined in a Cap'n Proto specific [schema language]. A
//! code generator sibling project uses these to creates thin wrappers around
//! this runtime library.
//!
//! [cap'n proto]: https://capnproto.org [schema language]:
//! https://capnproto.org/language.html
//!
//! A struct schema `Foo` outputs the following:
//!  - A `Foo` struct with borrow semantics: This may be constructed from an
//!    encoded buffer or borrowed from a `FooShared` or `FooOwned`.
//!  - A `FooShared` struct with reference-counted semantics: This may be
//!    constructed directly.
//!  - A `FooOwned` struct with move semantics: At the moment, this is only used
//!    internally in the construction of a `FooShared` but it will be used for
//!    post-construction mutation once support for that is added to this
//!    library.
//!
//! The user-facing (and developer-facing) components of this library generally
//! match up 1:1 with Cap'n Proto jargon:
//!
//! - <a name="element"></a> *Element*: An object encodeable/decodeable by Cap'n
//!   Proto. Currently `u8`, `u64`, a Cap'n Proto struct, and lists of these.
//!
//!   This library contains an Element "newtype" that will become part of the
//!   user-facing [Dynamic Reflection] API once that is added.
//!
//!   [dynamic reflection]: https://capnproto.org/cxx.html#dynamic-reflection
//!
//! - <a name="struct"></a> *[Struct]*: A named set of named, typed fields.
//!
//!   [struct]: https://capnproto.org/language.html#structs
//!
//! - <a name="list"></a> *[List]*: An ordered sequence of elements.
//!
//!   [list]: https://capnproto.org/language.html#built-in-types
//!
//! - <a name="union"></a> *[Union]*: A mutually-exclusive set of fields within
//!   a struct.
//!
//!   [union]: https://capnproto.org/language.html#unions
//!
//! - <a name="segment"></a> *[Segment]*: A flat blob of bytes (`[u8]`).
//!
//!   [segment]: https://capnproto.org/encoding.html#messages
//!
//! # fmt
//!
//! Each generated struct implements Debug.
//!
//! ```rust
//! # mod samples;
//! # use samples::rast_capnp::EntryShared;
//! # fn main() {
//! assert_eq!(
//!   "(term = 1, index = 2, payload = [03, 04])",
//!   format!("{:?}", EntryShared::new(1, 2, vec![3, 4].as_slice()).capnp_as_ref()),
//! );
//! # }
//! ```
//!
//! The alternate flag can be used to pretty print.
//!
//! ```rust
//! # mod samples;
//! # use samples::rast_capnp::EntryShared;
//! # fn main() {
//! assert_eq!(
//!   "(\n  term = 1,\n  index = 2,\n  payload = [03, 04],\n)",
//!   format!("{:#?}", EntryShared::new(1, 2, vec![3, 4].as_slice()).capnp_as_ref()),
//! );
//! # }
//! ```

// TODO: message, object, value, primitive, pointer, type, blob, field, far
//   pointer, landing pad, tag word, composite, list element, framing

#![warn(missing_docs, unsafe_code)]
#![warn(
  clippy::correctness,
  clippy::perf,
  clippy::wildcard_imports,
  clippy::trivially_copy_pass_by_ref
)]

mod cmp;
mod common;
mod decode;
mod element;
mod element_type;
mod encode;
mod r#enum;
mod error;
mod field_meta;
mod fmt_debug;
mod list;
mod pointer;
mod segment;
pub mod segment_framing_alternate;
pub mod segment_framing_official;
mod segment_pointer;
mod r#struct;
mod union;

/// This module re-exports all the types used by the [generated code].
///
/// [generated code]: crate#generated-code
pub mod prelude {
  pub use super::common::{CapnpAsRef, CapnpToOwned, Discriminant, NumElements, NumWords};
  pub use crate::element_type::ElementType;
  pub use crate::error::{Error, UnknownDiscriminant};
  pub use crate::field_meta::{
    DataFieldMeta, EnumFieldMeta, FieldMeta, I32FieldMeta, ListFieldMeta, StructFieldMeta,
    U64FieldMeta, UnionFieldMeta,
  };
  pub use crate::list::ListMeta;
  pub use crate::r#enum::{EnumMeta, EnumerantMeta, TypedEnum};
  pub use crate::r#struct::{
    StructMeta, TypedStruct, TypedStructShared, UntypedStruct, UntypedStructOwned,
    UntypedStructShared,
  };
  pub use crate::union::{TypedUnion, TypedUnionShared, UnionMeta, UnionVariantMeta, UntypedUnion};
}

#[cfg(feature = "serde")]
pub mod serde;

#[cfg(feature = "rand")]
pub mod rand;

#[cfg(test)]
#[rustfmt::skip]
pub mod samples;

#[cfg(test)]
mod init_test;

#[cfg(test)]
mod decode_test;
