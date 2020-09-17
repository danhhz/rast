Rast [Cap'n Proto]
====

[Cap'n Proto]: https://capnproto.org

- More "ergonomic" user-facing API
- Decoding can be both fast and user friendly, encoding less so
- Encoding is often not as much of a hot path, so make the common case easy but
  make it possible to be fast in the hot path case
- TODO: Implemented/not-implemented

- jargon: message, object, value, primitive, pointer, type, blob, struct, field,
  list, far pointer, landing pad, tag word, composite, list element, segment
  framing

# Samples

A number of samples of the actual generated code for a given `.capnp` file are
included. I've found in the past that this is often superior to attempting to
document how to use the generated code in the abstract (set a list field, get a
union, etc). To prevent rot, these are validated by a set of "golden" tests
which fail if they don't exactly match the generated output.

- [rast.capnp](runtime/src/samples/rast.capnp) and generated code
  [rast_capnp.rs](runtime/src/samples/rast_capnp.rs)
- [test.capnp](runtime/src/samples/test.capnp) and generated code
  [test_capnp.rs](runtime/src/samples/test_capnp.rs)

These are also quite useful for code review, as they provide the reviewer with
concrete examples of how things have changed.

If the "golden_test::*" tests ever fail, the generated files can be updated in
bulk (from the capnp subdirectory):

```
$ cargo test -p capnpc_rust golden -- -- --overwrite
```

# Roadmap: v0.1.0-alpha.0

- [ ] User-facing rustdoc
- [x] Implement random encode/decode test
- [x] Encoding via constructor
- [x] ListMeta
- [x] Merge StructMeta/ListMeta and StructElementType/ListElementType
- [x] Add context to error messages
- [x] Mark whether an error is user handleable
- [x] Bring back generated owned structs
- [x] Use in Rast superproject
- [ ] Return a reference to underlying bytes for capnp bytes fields
- [x] Bound size of rand value generation
- [ ] README
- [x] Document how to run the golden tests
- [ ] Set up CI
- [x] Clean up runtime prelude
- [ ] Audit pub usage
- [x] Audit TODO comments
- [ ] Resolve WIP comments
- [x] Remove dbg!
- [x] Remove println!
- [x] Remove "pub use"
- [ ] Test: Message evolution basics

# Roadmap: v0.1.0

- [ ] Name this project
- [ ] Developer-facing rustdoc
- [ ] Add version check for generated code vs the runtime crate it gets
- [ ] Use StringTree for generated code
- [ ] Support the remaining capnp field types
- [ ] Connect generated ref and owned structs with associated types
- [ ] Implement canonicalization
- [ ] Default values
- [ ] Copy struct without knowing type
- [ ] Packing
- [ ] Replace Vec in list in/out types with iterators
- [ ] Accept Struct or &Struct in constructor (same for struct lists)
- [ ] Wrapped fields
- [ ] Match up API naming with the official capnp
- [ ] Escape field names so metas don't conflict
- [ ] Figure out some obvious inlines
- [ ] Constructor with named args (likely via generated struct)
- [ ] Fully decode message into ^^, all accesses are error free
- [ ] Clean up codegen code structure
- [ ] Port remaining capnp testdata tests
- [ ] Run generated output through rustfmt
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


# Roadmap: After v0.1.0

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
