Rast Cap'n Proto
====

[![Build Status](https://travis-ci.org/danhhz/rast.svg?branch=dev)](https://travis-ci.org/danhhz/rast)
![crates.io](https://img.shields.io/crates/v/rast.svg)

This is a toy implemention of the [Cap'n Proto] encoding with a focus on
ergonomics.

[cap'n proto]: https://capnproto.org

### This is a proof of concept and _not ready for production_ use.

# Features

Copied from the [Cap'n Proto] homepage:

- Incremental reads: It is easy to start processing a Cap’n Proto message before
  you have received all of it since outer objects appear entirely before inner
  objects (as opposed to most encodings, where outer objects encompass inner
  objects).
- Random access/lazy decoding: You can read just one field of a message without
  parsing the whole thing.
- mmap: Read a large Cap’n Proto file by memory-mapping it. The OS won’t even
  read in the parts that you don’t access.
- Tiny generated code: Protobuf generates dedicated parsing and serialization
  code for every message type, and this code tends to be enormous. Cap’n Proto
  generated code is smaller by an order of magnitude or more. In fact, usually
  it’s no more than some inline accessor methods!
- Tiny runtime library: Due to the simplicity of the Cap’n Proto format, the
  runtime library can be much smaller.

Specific to this implementation:

- More "ergonomic" user-facing API: Decoding in "zero-alloc" serialization
  libraries can be made both fast and easy to use (especially given Rust's
  iterators).

  Encoding, however, tends to force arena allocation, which requires plumbing
  the arena through your code. It also strongly assumes that a given message is
  being constructed to be serialized and sent somewhere, whereas many
  applications that adopt [Protocol Buffers] eventually start using them as
  general purpose data containers and build library code around them.

  Thankfully, encoding is often not as performance critical as decoding. This
  means we can sacrifice some small amount of speed to make the common case easy, especially if it's possible to switch to the arena allocation implemention for hot paths.

  This implementation attempts to do exactly that by making what Cap'n Proto
  calls an "[orphan]" (and describes as an "advanced feature [that] typical
  applications probably won’t use") into the primary interface.

[protocol buffers]: https://developers.google.com/protocol-buffers
[orphan]: https://capnproto.org/cxx.html#orphans

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
- [ ] Replace Vec in list in types with iterators
- [x] Replace Vec in list out types with iterators
- [ ] Accept Struct or &Struct in constructor (same for struct lists)
- [x] Wrapped primitive fields
- [ ] Match up API naming with the official capnp
- [ ] Escape field names so metas don't conflict
- [ ] Figure out some obvious inlines
- [ ] Constructor with named args (likely via generated struct)
- [ ] Fully decode message into ^^, all accesses are error free
- [ ] Set list field in one message to list field from another message
- [x] Audit const usage (make them &'static)
- [x] Alternate "pretty" debug format
- [ ] Cleanups
  - [ ] Rename the meta things to schema
- [ ] Clean up codegen code structure
- [ ] Port remaining capnp testdata tests
- [ ] Security: Pointer validation (tests)
- [ ] Security: Amplification attack
- [ ] Security: Stack overflow DoS attack
- [ ] Security: Fuzzing tests
- [ ] Benchmarking
  - [ ] Pointer validation in struct is lazy
- [ ] Test: Zero-sized struct encoding
- [ ] Test: Creating a cycle
- [ ] Test: Text type and interior null byte "The encoding allows bytes other
  than the last to be zero"
- [ ] Test: Union of union
- [ ] Test: Message evolution basics


# Roadmap: After v0.1.0

- [ ] Post-construction mutation
- [ ] Compatibility with json.capnp annotations
- [ ] Port tests for json.capnp annotations
- [ ] Namespaces
- [ ] Keep all underlying buffers word-aligned
- [ ] Support no_std
- [ ] Wrapped non-primitive fields
- [ ] Run generated output through rustfmt
- [ ] Test for pass-through behavior (proxy/cache)
  - [ ] Test: Read message with later schema, store in list, encode, read with
    later schema
  - [ ] Decision about canonicalizing data written before 0.5 struct list
    encoding change
