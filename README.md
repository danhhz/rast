Rast
====

[![Build Status](https://travis-ci.org/danhhz/rast.svg?branch=dev)](https://travis-ci.org/danhhz/rast)
![crates.io](https://img.shields.io/crates/v/rast.svg)

~~F~~ast. R~~u~~st. Ra~~f~~t.

Rast is a toy implementation of the [raft consistency protocol] with a focus on
steady-state speed.

[raft consistency protocol]: https://raft.github.io/

### This is a proof of concept and _not ready for production_ use.

# Features

- Fully pipelined. The raft logic is non-blocking and can continue to respond to
  incoming rpcs and clock ticks during disk IO.
- [Not yet implemented] Zero-copy serde. One thing that [Kafka] got right was
  making the network format and the disk format the same. This allowed for use
  of the Linux [zero-copy] optimization that copies the incoming network traffic
  straight to disk for persistance. There's no reason a Raft log implemenation
  couldn't work the same way and io_uring makes this even easier.
- [Not yet implemented] Zero-alloc, lazy serde. The Raft logic in the steady
  state hot-path only looks at a couple fields of each incoming message, but
  popular serde implementations (like the [Protocol Buffers] of GRPC) require
  that an entire message be deserialized and allocate while doing so.
  Alternative wire formats like [Cap’n Proto] and [FlatBuffers] avoid this.

[kafka]: https://kafka.apache.org/
[zero-copy]: https://lwn.net/Articles/726917/
[protocol buffers]: https://developers.google.com/protocol-buffers
[grpc]: https://grpc.io/
[cap’n proto]: https://capnproto.org/
[flatbuffers]: https://google.github.io/flatbuffers/

# v0.1.0-alpha.0

- [x] Replace printlns with log crate
- [x] Resolve all WIP comments
- [x] Audit all TODO comments
- [x] Resolve all ignored tests
- [x] External documentation
- [x] Small cleanups
  - [x] Make current_time an Option
  - [x] Structs for {Persist,ReadStateMachine}{Req,Res}
  - [x] Clear max_outstanding_read_id when applicable

# v0.1.0-alpha.1

- [ ] Handle node restarts
  - [ ] Persist hard state
- [ ] Zero-copy message serialization
- [ ] More extensive nemesis testing
- [ ] Initial benchmarking
- [ ] Message idempotency
- [ ] Read-write requests (cput)
- [ ] Handle panics in state machine
- [ ] Audit public interface
- [x] Make logging an optional dependency
- [ ] Clean up log messages
- [ ] Internal documentation
- [ ] Idiomatic rustdoc

# v0.1.0

- [ ] Test with [maelstrom]
- [ ] Failure testing
- [ ] Harden public interface
- [ ] Membership changes

# After v0.1.0

- [ ] Snapshots

[maelstrom]: https://github.com/jepsen-io/maelstrom
