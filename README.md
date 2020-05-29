Rast
====

[![Build Status](https://travis-ci.org/danhhz/rsstringtree.svg?branch=master)](https://travis-ci.org/danhhz/rsstringtree)


~~F~~ast. R~~u~~st. Ra~~f~~t.

Rast is a toy implementation of the [raft consistency protocol] with a focus on
steady-state speed.

[raft consistency protocol]: https://raft.github.io/

# v0.1.0-alpha.0

- [ ] Replace printlns with log crate
- [ ] Resolve all WIP comments
- [ ] Audit all TODO comments
- [ ] Resolve all ignored tests
- [ ] External documentation
- [x] Small cleanups
  - [x] Make current_time an Option
  - [x] Structs for {Persist,ReadStateMachine}{Req,Res}
  - [x] Clear max_outstanding_read_id when applicable

# v0.1.0-alpha.1

- [ ] Handle node restarts
- [ ] Zero-copy message serialization
- [ ] More extensive nemesis testing
- [ ] Initial benchmarking
- [ ] Message idempotency
- [ ] Read-write requests (cput)
- [ ] Handle panics in state machine
- [ ] Audit public interface
- [ ] Make logging an optional dependency
- [ ] Clean up log messages
- [ ] Internal documentation

# v0.1.0

- [ ] Test with [maelstrom]
- [ ] Failure testing
- [ ] Harden public interface
- [ ] Membership changes

# After v0.1.0

- [ ] Snapshots

[maelstrom]: https://github.com/jepsen-io/maelstrom
