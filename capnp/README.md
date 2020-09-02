Rast [Cap'n Proto]
====

[Cap'n Proto]: https://capnproto.org

# v0.1.0-alpha.1

- [ ] Name this project
- [ ] User-facing rustdoc
- [ ] Implement random encode/decode test (via serde Deserialize?)
- [ ] Support the remaining non-bool field types
- [x] Encoding via constructor
- [ ] Clean up codegen code structure
- [ ] Port remaining capnp testdata tests
- [ ] Rename FieldTypeEnum
- [x] ListMeta
- [x] Merge StructMeta/ListMeta and StructElementType/ListElementType
- [x] Add context to error messages
- [ ] Mark whether an error is user handleable?
- [x] Bring back generated owned structs
- [ ] Return a reference to underlying bytes for capnp bytes fields
- [ ] Replace Vec in list return types with IntoIter
- [ ] Escape field names so metas don't conflict
- [ ] README
- [ ] Document how to run the golden tests
- [ ] Set up CI
- [ ] Clean up runtime prelude
- [ ] Audit pub usage
- [ ] Audit TODO comments
- [ ] Resolve WIP comments
- [x] Remove dbg!
- [ ] Remove println!
- [ ] Remove "pub use"
- [ ] Test for zero-sized struct encoding

# v0.1.0

- [ ] Developer-facing rustdoc
- [ ] Add version check for generated code vs the runtime crate it gets
- [ ] Use StringTree for generated code
- [ ] Support for bool fields
- [ ] Connect generated ref and owned structs with associated types
- [ ] Security: Pointer validation (tests)
- [ ] Security: Amplification attack
- [ ] Security: Stack overflow DoS attack
- [ ] Security: Fuzzing tests

# After v0.1.0

- [ ] Post-construction mutation
- [ ] Compatibility with json.capnp annotations
- [ ] Port tests for json.capnp annotations
