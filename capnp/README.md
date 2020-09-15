Rast [Cap'n Proto]
====

- More "ergonomic" user-facing API
- Decoding can be both fast and user friendly, encoding less so
- Encoding is often not as much of a hot path, so make the common case easy but
  make it possible to be fast in the hot path case
- TODO: Implemented/not-implemented

- jargon: message, object, value, primitive, pointer, type, blob, struct, field,
  list, far pointer, landing pad, tag word, composite, list element, segment
  framing

[Cap'n Proto]: https://capnproto.org

# v0.1.0-alpha.1

- [ ] Name this project
- [ ] User-facing rustdoc
- [x] Implement random encode/decode test
- [x] Encoding via constructor
- [x] ListMeta
- [x] Merge StructMeta/ListMeta and StructElementType/ListElementType
- [x] Add context to error messages
- [ ] Mark whether an error is user handleable?
- [x] Bring back generated owned structs
- [ ] Use in Rast superproject
- [ ] Return a reference to underlying bytes for capnp bytes fields
- [ ] Replace Vec in list return types with IntoIter
- [ ] Bound size of rand value generation
- [ ] README
- [ ] Document how to run the golden tests
- [ ] Set up CI
- [x] Clean up runtime prelude
- [ ] Audit pub usage
- [ ] Audit TODO comments
- [ ] Resolve WIP comments
- [x] Remove dbg!
- [x] Remove println!
- [x] Remove "pub use"
- [ ] Test: Message evolution basics

# v0.1.0

- [ ] Developer-facing rustdoc
- [ ] Add version check for generated code vs the runtime crate it gets
- [ ] Use StringTree for generated code
- [ ] Support the remaining capnp field types
- [ ] Connect generated ref and owned structs with associated types
- [ ] Implement canonicalization
- [ ] Default values
- [ ] Copy struct without knowing type
- [ ] Packing
- [ ] Match up API naming with the official capnp
- [ ] Escape field names so metas don't conflict
- [ ] Clean up codegen code structure
- [ ] Port remaining capnp testdata tests
- [ ] Rename FieldTypeEnum
- [ ] Security: Pointer validation (tests)
- [ ] Security: Amplification attack
- [ ] Security: Stack overflow DoS attack
- [ ] Security: Fuzzing tests
- [ ] Benchmarking
  - [ ] Pointer validation in struct is lazy
- [ ] Test for zero-sized struct encoding
- [ ] Test for creating a cycle
- [ ] Test for Text type and interior null byte "The encoding allows bytes other
  than the last to be zero"
- [ ] Test union of union


# After v0.1.0

- [ ] Post-construction mutation
- [ ] Compatibility with json.capnp annotations
- [ ] Port tests for json.capnp annotations
- [ ] Namespaces
- [ ] Keep all underlying buffers word-aligned
- [ ] Test for pass-through behavior (proxy/cache)
  - [ ] Test: Read message with later schema, store in list, encode, read with
    later schema
  - [ ] Decision about canonicalizing data written before 0.5 struct list
    encoding change
