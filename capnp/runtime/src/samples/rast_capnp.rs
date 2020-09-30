use capnp_runtime::prelude::*;

/// An entry in the Raft log.
#[derive(Clone)]
pub struct Entry<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> Entry<'a> {
  const TERM_META: &'static U64FieldMeta = &U64FieldMeta {
    name: "term",
    offset: NumElements(0),
  };
  const INDEX_META: &'static U64FieldMeta = &U64FieldMeta {
    name: "index",
    offset: NumElements(1),
  };
  const PAYLOAD_META: &'static DataFieldMeta = &DataFieldMeta {
    name: "payload",
    offset: NumElements(0),
  };

  const META: &'static StructMeta = &StructMeta {
    name: "Entry",
    data_size: NumWords(2),
    pointer_size: NumWords(1),
    fields: || &[
      FieldMeta::U64(Entry::TERM_META),
      FieldMeta::U64(Entry::INDEX_META),
      FieldMeta::Data(Entry::PAYLOAD_META),
    ],
  };

  /// The term of the entry.
  pub fn term(&self) -> Term { Term(Entry::TERM_META.get(&self.data)) }

  /// The index of the entry.
  pub fn index(&self) -> Index { Index(Entry::INDEX_META.get(&self.data)) }

  /// The opaque user payload of the entry.
  pub fn payload(&self) -> Result<&'a [u8], Error> { Entry::PAYLOAD_META.get(&self.data) }

  pub fn capnp_to_owned(&self) -> EntryShared {
    EntryShared { data: self.data.capnp_to_owned() }
  }
}

impl<'a> TypedStruct<'a> for Entry<'a> {
  fn meta() -> &'static StructMeta {
    &Entry::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    Entry { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for Entry<'a> {
  type Owned = EntryShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    Entry::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for Entry<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for Entry<'a> {
  fn partial_cmp(&self, other: &Entry<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for Entry<'a> {
  fn eq(&self, other: &Entry<'a>) -> bool {
    self.partial_cmp(&other) == Some(std::cmp::Ordering::Equal)
  }
}

#[derive(Clone)]
pub struct EntryShared {
  data: UntypedStructShared,
}

impl EntryShared {
  pub fn new(
    term: Term,
    index: Index,
    payload: &[u8],
  ) -> EntryShared {
    let mut data = UntypedStructOwned::new_with_root_struct(Entry::META.data_size, Entry::META.pointer_size);
    Entry::TERM_META.set(&mut data, term.0);
    Entry::INDEX_META.set(&mut data, index.0);
    Entry::PAYLOAD_META.set(&mut data, payload);
    EntryShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> Entry<'a> {
    Entry { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for EntryShared {
  fn meta() -> &'static StructMeta {
    &Entry::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    EntryShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, Entry<'a>> for EntryShared {
  fn capnp_as_ref(&'a self) -> Entry<'a> {
    EntryShared::capnp_as_ref(self)
  }
}

/// An rpc message.
#[derive(Clone)]
pub struct Message<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> Message<'a> {
  const SRC_META: &'static U64FieldMeta = &U64FieldMeta {
    name: "src",
    offset: NumElements(0),
  };
  const DEST_META: &'static U64FieldMeta = &U64FieldMeta {
    name: "dest",
    offset: NumElements(1),
  };
  const PAYLOAD_META: &'static UnionFieldMeta = &UnionFieldMeta {
    name: "payload",
    offset: NumElements(8),
    meta: &Payload::META,
  };

  const META: &'static StructMeta = &StructMeta {
    name: "Message",
    data_size: NumWords(3),
    pointer_size: NumWords(1),
    fields: || &[
      FieldMeta::U64(Message::SRC_META),
      FieldMeta::U64(Message::DEST_META),
      FieldMeta::Union(Message::PAYLOAD_META),
    ],
  };

  /// The node sending this rpc.
  pub fn src(&self) -> NodeID { NodeID(Message::SRC_META.get(&self.data)) }

  /// The node to receive this rpc.
  pub fn dest(&self) -> NodeID { NodeID(Message::DEST_META.get(&self.data)) }

  pub fn payload(&self) -> Result<Result<Payload<'a>, UnknownDiscriminant>,Error> { Message::PAYLOAD_META.get(&self.data) }

  pub fn capnp_to_owned(&self) -> MessageShared {
    MessageShared { data: self.data.capnp_to_owned() }
  }
}

impl<'a> TypedStruct<'a> for Message<'a> {
  fn meta() -> &'static StructMeta {
    &Message::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    Message { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for Message<'a> {
  type Owned = MessageShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    Message::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for Message<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for Message<'a> {
  fn partial_cmp(&self, other: &Message<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for Message<'a> {
  fn eq(&self, other: &Message<'a>) -> bool {
    self.partial_cmp(&other) == Some(std::cmp::Ordering::Equal)
  }
}

#[derive(Clone)]
pub struct MessageShared {
  data: UntypedStructShared,
}

impl MessageShared {
  pub fn new(
    src: NodeID,
    dest: NodeID,
    payload: PayloadShared,
  ) -> MessageShared {
    let mut data = UntypedStructOwned::new_with_root_struct(Message::META.data_size, Message::META.pointer_size);
    Message::SRC_META.set(&mut data, src.0);
    Message::DEST_META.set(&mut data, dest.0);
    Message::PAYLOAD_META.set(&mut data, payload);
    MessageShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> Message<'a> {
    Message { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for MessageShared {
  fn meta() -> &'static StructMeta {
    &Message::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    MessageShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, Message<'a>> for MessageShared {
  fn capnp_as_ref(&'a self) -> Message<'a> {
    MessageShared::capnp_as_ref(self)
  }
}

#[derive(Clone)]
pub struct AppendEntriesReq<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> AppendEntriesReq<'a> {
  const TERM_META: &'static U64FieldMeta = &U64FieldMeta {
    name: "term",
    offset: NumElements(0),
  };
  const LEADER_ID_META: &'static U64FieldMeta = &U64FieldMeta {
    name: "leaderId",
    offset: NumElements(1),
  };
  const PREV_LOG_INDEX_META: &'static U64FieldMeta = &U64FieldMeta {
    name: "prevLogIndex",
    offset: NumElements(2),
  };
  const PREV_LOG_TERM_META: &'static U64FieldMeta = &U64FieldMeta {
    name: "prevLogTerm",
    offset: NumElements(3),
  };
  const LEADER_COMMIT_META: &'static U64FieldMeta = &U64FieldMeta {
    name: "leaderCommit",
    offset: NumElements(4),
  };
  const READ_ID_META: &'static U64FieldMeta = &U64FieldMeta {
    name: "readId",
    offset: NumElements(5),
  };
  const ENTRIES_META: &'static ListFieldMeta = &ListFieldMeta {
    name: "entries",
    offset: NumElements(0),
    meta: &ListMeta {
      value_type: ElementType::Struct(&Entry::META)
    },
  };

  const META: &'static StructMeta = &StructMeta {
    name: "AppendEntriesReq",
    data_size: NumWords(6),
    pointer_size: NumWords(1),
    fields: || &[
      FieldMeta::U64(AppendEntriesReq::TERM_META),
      FieldMeta::U64(AppendEntriesReq::LEADER_ID_META),
      FieldMeta::U64(AppendEntriesReq::PREV_LOG_INDEX_META),
      FieldMeta::U64(AppendEntriesReq::PREV_LOG_TERM_META),
      FieldMeta::U64(AppendEntriesReq::LEADER_COMMIT_META),
      FieldMeta::U64(AppendEntriesReq::READ_ID_META),
      FieldMeta::List(AppendEntriesReq::ENTRIES_META),
    ],
  };

  pub fn term(&self) -> Term { Term(AppendEntriesReq::TERM_META.get(&self.data)) }

  pub fn leader_id(&self) -> NodeID { NodeID(AppendEntriesReq::LEADER_ID_META.get(&self.data)) }

  pub fn prev_log_index(&self) -> Index { Index(AppendEntriesReq::PREV_LOG_INDEX_META.get(&self.data)) }

  pub fn prev_log_term(&self) -> Term { Term(AppendEntriesReq::PREV_LOG_TERM_META.get(&self.data)) }

  pub fn leader_commit(&self) -> Index { Index(AppendEntriesReq::LEADER_COMMIT_META.get(&self.data)) }

  pub fn read_id(&self) -> ReadID { ReadID(AppendEntriesReq::READ_ID_META.get(&self.data)) }

  pub fn entries(&self) -> Result<Slice<'a, Entry<'a>>, Error> { AppendEntriesReq::ENTRIES_META.get(&self.data) }

  pub fn capnp_to_owned(&self) -> AppendEntriesReqShared {
    AppendEntriesReqShared { data: self.data.capnp_to_owned() }
  }
}

impl<'a> TypedStruct<'a> for AppendEntriesReq<'a> {
  fn meta() -> &'static StructMeta {
    &AppendEntriesReq::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    AppendEntriesReq { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for AppendEntriesReq<'a> {
  type Owned = AppendEntriesReqShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    AppendEntriesReq::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for AppendEntriesReq<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for AppendEntriesReq<'a> {
  fn partial_cmp(&self, other: &AppendEntriesReq<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for AppendEntriesReq<'a> {
  fn eq(&self, other: &AppendEntriesReq<'a>) -> bool {
    self.partial_cmp(&other) == Some(std::cmp::Ordering::Equal)
  }
}

#[derive(Clone)]
pub struct AppendEntriesReqShared {
  data: UntypedStructShared,
}

impl AppendEntriesReqShared {
  pub fn new(
    term: Term,
    leader_id: NodeID,
    prev_log_index: Index,
    prev_log_term: Term,
    leader_commit: Index,
    read_id: ReadID,
    entries: &'_ [EntryShared],
  ) -> AppendEntriesReqShared {
    let mut data = UntypedStructOwned::new_with_root_struct(AppendEntriesReq::META.data_size, AppendEntriesReq::META.pointer_size);
    AppendEntriesReq::TERM_META.set(&mut data, term.0);
    AppendEntriesReq::LEADER_ID_META.set(&mut data, leader_id.0);
    AppendEntriesReq::PREV_LOG_INDEX_META.set(&mut data, prev_log_index.0);
    AppendEntriesReq::PREV_LOG_TERM_META.set(&mut data, prev_log_term.0);
    AppendEntriesReq::LEADER_COMMIT_META.set(&mut data, leader_commit.0);
    AppendEntriesReq::READ_ID_META.set(&mut data, read_id.0);
    AppendEntriesReq::ENTRIES_META.set(&mut data, entries);
    AppendEntriesReqShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> AppendEntriesReq<'a> {
    AppendEntriesReq { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for AppendEntriesReqShared {
  fn meta() -> &'static StructMeta {
    &AppendEntriesReq::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    AppendEntriesReqShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, AppendEntriesReq<'a>> for AppendEntriesReqShared {
  fn capnp_as_ref(&'a self) -> AppendEntriesReq<'a> {
    AppendEntriesReqShared::capnp_as_ref(self)
  }
}

#[derive(Clone)]
pub struct AppendEntriesRes<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> AppendEntriesRes<'a> {
  const TERM_META: &'static U64FieldMeta = &U64FieldMeta {
    name: "term",
    offset: NumElements(0),
  };
  const SUCCESS_META: &'static U64FieldMeta = &U64FieldMeta {
    name: "success",
    offset: NumElements(1),
  };
  const INDEX_META: &'static U64FieldMeta = &U64FieldMeta {
    name: "index",
    offset: NumElements(2),
  };
  const READ_ID_META: &'static U64FieldMeta = &U64FieldMeta {
    name: "readId",
    offset: NumElements(3),
  };

  const META: &'static StructMeta = &StructMeta {
    name: "AppendEntriesRes",
    data_size: NumWords(4),
    pointer_size: NumWords(0),
    fields: || &[
      FieldMeta::U64(AppendEntriesRes::TERM_META),
      FieldMeta::U64(AppendEntriesRes::SUCCESS_META),
      FieldMeta::U64(AppendEntriesRes::INDEX_META),
      FieldMeta::U64(AppendEntriesRes::READ_ID_META),
    ],
  };

  pub fn term(&self) -> Term { Term(AppendEntriesRes::TERM_META.get(&self.data)) }

  pub fn success(&self) -> u64 { AppendEntriesRes::SUCCESS_META.get(&self.data) }

  pub fn index(&self) -> Index { Index(AppendEntriesRes::INDEX_META.get(&self.data)) }

  pub fn read_id(&self) -> ReadID { ReadID(AppendEntriesRes::READ_ID_META.get(&self.data)) }

  pub fn capnp_to_owned(&self) -> AppendEntriesResShared {
    AppendEntriesResShared { data: self.data.capnp_to_owned() }
  }
}

impl<'a> TypedStruct<'a> for AppendEntriesRes<'a> {
  fn meta() -> &'static StructMeta {
    &AppendEntriesRes::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    AppendEntriesRes { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for AppendEntriesRes<'a> {
  type Owned = AppendEntriesResShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    AppendEntriesRes::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for AppendEntriesRes<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for AppendEntriesRes<'a> {
  fn partial_cmp(&self, other: &AppendEntriesRes<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for AppendEntriesRes<'a> {
  fn eq(&self, other: &AppendEntriesRes<'a>) -> bool {
    self.partial_cmp(&other) == Some(std::cmp::Ordering::Equal)
  }
}

#[derive(Clone)]
pub struct AppendEntriesResShared {
  data: UntypedStructShared,
}

impl AppendEntriesResShared {
  pub fn new(
    term: Term,
    success: u64,
    index: Index,
    read_id: ReadID,
  ) -> AppendEntriesResShared {
    let mut data = UntypedStructOwned::new_with_root_struct(AppendEntriesRes::META.data_size, AppendEntriesRes::META.pointer_size);
    AppendEntriesRes::TERM_META.set(&mut data, term.0);
    AppendEntriesRes::SUCCESS_META.set(&mut data, success);
    AppendEntriesRes::INDEX_META.set(&mut data, index.0);
    AppendEntriesRes::READ_ID_META.set(&mut data, read_id.0);
    AppendEntriesResShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> AppendEntriesRes<'a> {
    AppendEntriesRes { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for AppendEntriesResShared {
  fn meta() -> &'static StructMeta {
    &AppendEntriesRes::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    AppendEntriesResShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, AppendEntriesRes<'a>> for AppendEntriesResShared {
  fn capnp_as_ref(&'a self) -> AppendEntriesRes<'a> {
    AppendEntriesResShared::capnp_as_ref(self)
  }
}

#[derive(Clone)]
pub struct RequestVoteReq<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> RequestVoteReq<'a> {
  const TERM_META: &'static U64FieldMeta = &U64FieldMeta {
    name: "term",
    offset: NumElements(0),
  };
  const CANDIDATE_ID_META: &'static U64FieldMeta = &U64FieldMeta {
    name: "candidateId",
    offset: NumElements(1),
  };
  const LAST_LOG_INDEX_META: &'static U64FieldMeta = &U64FieldMeta {
    name: "lastLogIndex",
    offset: NumElements(2),
  };
  const LAST_LOG_TERM_META: &'static U64FieldMeta = &U64FieldMeta {
    name: "lastLogTerm",
    offset: NumElements(3),
  };

  const META: &'static StructMeta = &StructMeta {
    name: "RequestVoteReq",
    data_size: NumWords(4),
    pointer_size: NumWords(0),
    fields: || &[
      FieldMeta::U64(RequestVoteReq::TERM_META),
      FieldMeta::U64(RequestVoteReq::CANDIDATE_ID_META),
      FieldMeta::U64(RequestVoteReq::LAST_LOG_INDEX_META),
      FieldMeta::U64(RequestVoteReq::LAST_LOG_TERM_META),
    ],
  };

  pub fn term(&self) -> Term { Term(RequestVoteReq::TERM_META.get(&self.data)) }

  pub fn candidate_id(&self) -> NodeID { NodeID(RequestVoteReq::CANDIDATE_ID_META.get(&self.data)) }

  pub fn last_log_index(&self) -> Index { Index(RequestVoteReq::LAST_LOG_INDEX_META.get(&self.data)) }

  pub fn last_log_term(&self) -> Term { Term(RequestVoteReq::LAST_LOG_TERM_META.get(&self.data)) }

  pub fn capnp_to_owned(&self) -> RequestVoteReqShared {
    RequestVoteReqShared { data: self.data.capnp_to_owned() }
  }
}

impl<'a> TypedStruct<'a> for RequestVoteReq<'a> {
  fn meta() -> &'static StructMeta {
    &RequestVoteReq::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    RequestVoteReq { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for RequestVoteReq<'a> {
  type Owned = RequestVoteReqShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    RequestVoteReq::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for RequestVoteReq<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for RequestVoteReq<'a> {
  fn partial_cmp(&self, other: &RequestVoteReq<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for RequestVoteReq<'a> {
  fn eq(&self, other: &RequestVoteReq<'a>) -> bool {
    self.partial_cmp(&other) == Some(std::cmp::Ordering::Equal)
  }
}

#[derive(Clone)]
pub struct RequestVoteReqShared {
  data: UntypedStructShared,
}

impl RequestVoteReqShared {
  pub fn new(
    term: Term,
    candidate_id: NodeID,
    last_log_index: Index,
    last_log_term: Term,
  ) -> RequestVoteReqShared {
    let mut data = UntypedStructOwned::new_with_root_struct(RequestVoteReq::META.data_size, RequestVoteReq::META.pointer_size);
    RequestVoteReq::TERM_META.set(&mut data, term.0);
    RequestVoteReq::CANDIDATE_ID_META.set(&mut data, candidate_id.0);
    RequestVoteReq::LAST_LOG_INDEX_META.set(&mut data, last_log_index.0);
    RequestVoteReq::LAST_LOG_TERM_META.set(&mut data, last_log_term.0);
    RequestVoteReqShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> RequestVoteReq<'a> {
    RequestVoteReq { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for RequestVoteReqShared {
  fn meta() -> &'static StructMeta {
    &RequestVoteReq::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    RequestVoteReqShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, RequestVoteReq<'a>> for RequestVoteReqShared {
  fn capnp_as_ref(&'a self) -> RequestVoteReq<'a> {
    RequestVoteReqShared::capnp_as_ref(self)
  }
}

#[derive(Clone)]
pub struct RequestVoteRes<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> RequestVoteRes<'a> {
  const TERM_META: &'static U64FieldMeta = &U64FieldMeta {
    name: "term",
    offset: NumElements(0),
  };
  const VOTE_GRANTED_META: &'static U64FieldMeta = &U64FieldMeta {
    name: "voteGranted",
    offset: NumElements(1),
  };

  const META: &'static StructMeta = &StructMeta {
    name: "RequestVoteRes",
    data_size: NumWords(2),
    pointer_size: NumWords(0),
    fields: || &[
      FieldMeta::U64(RequestVoteRes::TERM_META),
      FieldMeta::U64(RequestVoteRes::VOTE_GRANTED_META),
    ],
  };

  pub fn term(&self) -> Term { Term(RequestVoteRes::TERM_META.get(&self.data)) }

  pub fn vote_granted(&self) -> u64 { RequestVoteRes::VOTE_GRANTED_META.get(&self.data) }

  pub fn capnp_to_owned(&self) -> RequestVoteResShared {
    RequestVoteResShared { data: self.data.capnp_to_owned() }
  }
}

impl<'a> TypedStruct<'a> for RequestVoteRes<'a> {
  fn meta() -> &'static StructMeta {
    &RequestVoteRes::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    RequestVoteRes { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for RequestVoteRes<'a> {
  type Owned = RequestVoteResShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    RequestVoteRes::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for RequestVoteRes<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for RequestVoteRes<'a> {
  fn partial_cmp(&self, other: &RequestVoteRes<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for RequestVoteRes<'a> {
  fn eq(&self, other: &RequestVoteRes<'a>) -> bool {
    self.partial_cmp(&other) == Some(std::cmp::Ordering::Equal)
  }
}

#[derive(Clone)]
pub struct RequestVoteResShared {
  data: UntypedStructShared,
}

impl RequestVoteResShared {
  pub fn new(
    term: Term,
    vote_granted: u64,
  ) -> RequestVoteResShared {
    let mut data = UntypedStructOwned::new_with_root_struct(RequestVoteRes::META.data_size, RequestVoteRes::META.pointer_size);
    RequestVoteRes::TERM_META.set(&mut data, term.0);
    RequestVoteRes::VOTE_GRANTED_META.set(&mut data, vote_granted);
    RequestVoteResShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> RequestVoteRes<'a> {
    RequestVoteRes { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for RequestVoteResShared {
  fn meta() -> &'static StructMeta {
    &RequestVoteRes::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    RequestVoteResShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, RequestVoteRes<'a>> for RequestVoteResShared {
  fn capnp_as_ref(&'a self) -> RequestVoteRes<'a> {
    RequestVoteResShared::capnp_as_ref(self)
  }
}

#[derive(Clone)]
pub struct StartElectionReq<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> StartElectionReq<'a> {
  const TERM_META: &'static U64FieldMeta = &U64FieldMeta {
    name: "term",
    offset: NumElements(0),
  };

  const META: &'static StructMeta = &StructMeta {
    name: "StartElectionReq",
    data_size: NumWords(1),
    pointer_size: NumWords(0),
    fields: || &[
      FieldMeta::U64(StartElectionReq::TERM_META),
    ],
  };

  pub fn term(&self) -> Term { Term(StartElectionReq::TERM_META.get(&self.data)) }

  pub fn capnp_to_owned(&self) -> StartElectionReqShared {
    StartElectionReqShared { data: self.data.capnp_to_owned() }
  }
}

impl<'a> TypedStruct<'a> for StartElectionReq<'a> {
  fn meta() -> &'static StructMeta {
    &StartElectionReq::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    StartElectionReq { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for StartElectionReq<'a> {
  type Owned = StartElectionReqShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    StartElectionReq::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for StartElectionReq<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for StartElectionReq<'a> {
  fn partial_cmp(&self, other: &StartElectionReq<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for StartElectionReq<'a> {
  fn eq(&self, other: &StartElectionReq<'a>) -> bool {
    self.partial_cmp(&other) == Some(std::cmp::Ordering::Equal)
  }
}

#[derive(Clone)]
pub struct StartElectionReqShared {
  data: UntypedStructShared,
}

impl StartElectionReqShared {
  pub fn new(
    term: Term,
  ) -> StartElectionReqShared {
    let mut data = UntypedStructOwned::new_with_root_struct(StartElectionReq::META.data_size, StartElectionReq::META.pointer_size);
    StartElectionReq::TERM_META.set(&mut data, term.0);
    StartElectionReqShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> StartElectionReq<'a> {
    StartElectionReq { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for StartElectionReqShared {
  fn meta() -> &'static StructMeta {
    &StartElectionReq::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    StartElectionReqShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, StartElectionReq<'a>> for StartElectionReqShared {
  fn capnp_as_ref(&'a self) -> StartElectionReq<'a> {
    StartElectionReqShared::capnp_as_ref(self)
  }
}

#[derive(Clone)]
pub enum Payload<'a> {
  AppendEntriesReq(AppendEntriesReq<'a>),
  AppendEntriesRes(AppendEntriesRes<'a>),
  RequestVoteReq(RequestVoteReq<'a>),
  RequestVoteRes(RequestVoteRes<'a>),
  StartElectionReq(StartElectionReq<'a>),
}

impl Payload<'_> {
  const APPEND_ENTRIES_REQ_META: &'static StructFieldMeta = &StructFieldMeta {
    name: "appendEntriesReq",
    offset: NumElements(0),
    meta: &AppendEntriesReq::META,
  };
  const APPEND_ENTRIES_RES_META: &'static StructFieldMeta = &StructFieldMeta {
    name: "appendEntriesRes",
    offset: NumElements(0),
    meta: &AppendEntriesRes::META,
  };
  const REQUEST_VOTE_REQ_META: &'static StructFieldMeta = &StructFieldMeta {
    name: "requestVoteReq",
    offset: NumElements(0),
    meta: &RequestVoteReq::META,
  };
  const REQUEST_VOTE_RES_META: &'static StructFieldMeta = &StructFieldMeta {
    name: "requestVoteRes",
    offset: NumElements(0),
    meta: &RequestVoteRes::META,
  };
  const START_ELECTION_REQ_META: &'static StructFieldMeta = &StructFieldMeta {
    name: "startElectionReq",
    offset: NumElements(0),
    meta: &StartElectionReq::META,
  };
  const META: &'static UnionMeta = &UnionMeta {
    name: "Payload",
    variants: &[
      UnionVariantMeta{
        discriminant: Discriminant(0),
        field_meta: FieldMeta::Struct(Payload::APPEND_ENTRIES_REQ_META),
      },
      UnionVariantMeta{
        discriminant: Discriminant(1),
        field_meta: FieldMeta::Struct(Payload::APPEND_ENTRIES_RES_META),
      },
      UnionVariantMeta{
        discriminant: Discriminant(2),
        field_meta: FieldMeta::Struct(Payload::REQUEST_VOTE_REQ_META),
      },
      UnionVariantMeta{
        discriminant: Discriminant(3),
        field_meta: FieldMeta::Struct(Payload::REQUEST_VOTE_RES_META),
      },
      UnionVariantMeta{
        discriminant: Discriminant(4),
        field_meta: FieldMeta::Struct(Payload::START_ELECTION_REQ_META),
      },
    ],
  };

  pub fn capnp_to_owned(&self) -> PayloadShared {
    match self {
      Payload::AppendEntriesReq(x) => PayloadShared::AppendEntriesReq(x.capnp_to_owned()),
      Payload::AppendEntriesRes(x) => PayloadShared::AppendEntriesRes(x.capnp_to_owned()),
      Payload::RequestVoteReq(x) => PayloadShared::RequestVoteReq(x.capnp_to_owned()),
      Payload::RequestVoteRes(x) => PayloadShared::RequestVoteRes(x.capnp_to_owned()),
      Payload::StartElectionReq(x) => PayloadShared::StartElectionReq(x.capnp_to_owned()),
    }
  }
}

impl<'a> TypedUnion<'a> for Payload<'a> {
  fn meta() -> &'static UnionMeta {
    &Payload::META
  }
  fn from_untyped_union(untyped: &UntypedUnion<'a>) -> Result<Result<Self, UnknownDiscriminant>, Error> {
    match untyped.discriminant {
      Discriminant(0) => Payload::APPEND_ENTRIES_REQ_META.get(&untyped.variant_data).map(|x| Ok(Payload::AppendEntriesReq(x))),
      Discriminant(1) => Payload::APPEND_ENTRIES_RES_META.get(&untyped.variant_data).map(|x| Ok(Payload::AppendEntriesRes(x))),
      Discriminant(2) => Payload::REQUEST_VOTE_REQ_META.get(&untyped.variant_data).map(|x| Ok(Payload::RequestVoteReq(x))),
      Discriminant(3) => Payload::REQUEST_VOTE_RES_META.get(&untyped.variant_data).map(|x| Ok(Payload::RequestVoteRes(x))),
      Discriminant(4) => Payload::START_ELECTION_REQ_META.get(&untyped.variant_data).map(|x| Ok(Payload::StartElectionReq(x))),
      x => Ok(Err(UnknownDiscriminant(x, Payload::META.name))),
    }
  }
}

impl<'a> CapnpToOwned<'a> for Payload<'a> {
  type Owned = PayloadShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    Payload::capnp_to_owned(self)
  }
}

#[derive(Clone)]
pub enum PayloadShared {
  AppendEntriesReq(AppendEntriesReqShared),
  AppendEntriesRes(AppendEntriesResShared),
  RequestVoteReq(RequestVoteReqShared),
  RequestVoteRes(RequestVoteResShared),
  StartElectionReq(StartElectionReqShared),
}

impl PayloadShared {
  pub fn capnp_as_ref<'a>(&'a self) -> Payload<'a> {
    match self {
      PayloadShared::AppendEntriesReq(x) => Payload::AppendEntriesReq(x.capnp_as_ref()),
      PayloadShared::AppendEntriesRes(x) => Payload::AppendEntriesRes(x.capnp_as_ref()),
      PayloadShared::RequestVoteReq(x) => Payload::RequestVoteReq(x.capnp_as_ref()),
      PayloadShared::RequestVoteRes(x) => Payload::RequestVoteRes(x.capnp_as_ref()),
      PayloadShared::StartElectionReq(x) => Payload::StartElectionReq(x.capnp_as_ref()),
    }
  }
}

impl<'a> TypedUnionShared<'a, Payload<'a>> for PayloadShared {
  fn set(&self, data: &mut UntypedStructOwned, discriminant_offset: NumElements) {
    match self {
      PayloadShared::AppendEntriesReq(x) => {
        data.set_discriminant(discriminant_offset, Discriminant(0));
        Payload::APPEND_ENTRIES_REQ_META.set(data, x.clone().into());
      }
      PayloadShared::AppendEntriesRes(x) => {
        data.set_discriminant(discriminant_offset, Discriminant(1));
        Payload::APPEND_ENTRIES_RES_META.set(data, x.clone().into());
      }
      PayloadShared::RequestVoteReq(x) => {
        data.set_discriminant(discriminant_offset, Discriminant(2));
        Payload::REQUEST_VOTE_REQ_META.set(data, x.clone().into());
      }
      PayloadShared::RequestVoteRes(x) => {
        data.set_discriminant(discriminant_offset, Discriminant(3));
        Payload::REQUEST_VOTE_RES_META.set(data, x.clone().into());
      }
      PayloadShared::StartElectionReq(x) => {
        data.set_discriminant(discriminant_offset, Discriminant(4));
        Payload::START_ELECTION_REQ_META.set(data, x.clone().into());
      }
    }
  }
}

impl<'a> CapnpAsRef<'a, Payload<'a>> for PayloadShared {
  fn capnp_as_ref(&'a self) -> Payload<'a> {
    PayloadShared::capnp_as_ref(self)
  }
}
