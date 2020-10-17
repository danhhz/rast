# Copyright 2020 Daniel Harrison. All Rights Reserved.

@0xea2e2ab925cc327b;

annotation newType @0xed2033c8233f2a96 (field) :Text;

struct Entry {
  # An entry in the Raft log.

  term @0 :UInt64 $newType("Term");
  # The term of the entry.

  index @1 :UInt64 $newType("Index");
  # The index of the entry.

  payload @2 :Data;
  # The opaque user payload of the entry.
}

const foo :Entry = (term = 1, index = 2, payload = "payload");

const bar :Message = (
  src = 1,
  dest = 2,
  payload = (appendEntriesReq = (
    term = 3,
    leaderId = 4,
    prevLogIndex = 5,
    prevLogTerm = 6,
    leaderCommit = 7,
    readId = 8,
    entries = [
      (term = 9, index = 10, payload = "e1"),
      (term = 11, index = 12, payload = "e2")
    ]
  ))
);

struct Message {
  # An rpc message.

  src @0 :UInt64 $newType("NodeID");
  # The node sending this rpc.

  dest @1 :UInt64 $newType("NodeID");
  # The node to receive this rpc.

  payload :group {
    union {
      appendEntriesReq @2 :AppendEntriesReq;
      appendEntriesRes @3 :AppendEntriesRes;
      requestVoteReq @4 :RequestVoteReq;
      requestVoteRes @5 :RequestVoteRes;
      startElectionReq @6 :StartElectionReq;
    }
  }
}

struct AppendEntriesReq {
  term @0 :UInt64 $newType("Term");
  leaderId @1 :UInt64 $newType("NodeID");
  prevLogIndex @2 :UInt64 $newType("Index");
  prevLogTerm @3 :UInt64 $newType("Term");
  leaderCommit @4 :UInt64 $newType("Index");
  readId @5 :UInt64 $newType("ReadID");
  entries @6 :List(Entry);
}

struct AppendEntriesRes {
  term @0 :UInt64 $newType("Term");
  success @1 :UInt64;
  index @2 :UInt64 $newType("Index");
  readId @3 :UInt64 $newType("ReadID");
}

struct RequestVoteReq {
  term @0 :UInt64 $newType("Term");
  candidateId @1 :UInt64 $newType("NodeID");
  lastLogIndex @2 :UInt64 $newType("Index");
  lastLogTerm @3 :UInt64 $newType("Term");
}

struct RequestVoteRes {
  term @0 :UInt64 $newType("Term");
  voteGranted @1 :UInt64;
}

struct StartElectionReq {
  term @0 :UInt64 $newType("Term");
}
