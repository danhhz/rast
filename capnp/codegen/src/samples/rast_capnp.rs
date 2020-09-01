use capnp_runtime::prelude::*;

/// An entry in the Raft log.
#[derive(Clone)]
pub struct Entry<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> Entry<'a> {
  const TERM_META: U64FieldMeta = U64FieldMeta {
    name: "term",
    offset: NumElements(0),
  };
  const INDEX_META: U64FieldMeta = U64FieldMeta {
    name: "index",
    offset: NumElements(1),
  };
  const PAYLOAD_META: ListFieldMeta = ListFieldMeta {
    name: "payload",
    offset: NumElements(0),
    get_element: |data, sink| sink.list(Entry{data: data.clone()}.payload().to_element_list()),
  };

  const META: StructMeta = StructMeta {
    name: "Entry",
    fields: &[
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(Entry::TERM_META)),
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(Entry::INDEX_META)),
      FieldMeta::Pointer(PointerFieldMeta::List(Entry::PAYLOAD_META)),
    ],
  };

  /// The term of the entry.
  pub fn term(&self) -> u64 { Entry::TERM_META.get(&self.data) }
  /// The index of the entry.
  pub fn index(&self) -> u64 { Entry::INDEX_META.get(&self.data) }
  /// The opaque user payload of the entry.
  pub fn payload(&self) -> Result<Vec<u8>, Error> { Entry::PAYLOAD_META.get(&self.data) }
}

impl<'a> TypedStruct<'a> for Entry<'a> {
  fn meta(&self) -> &'static StructMeta {
    &Entry::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    Entry { data: data }
  }
  fn to_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> std::fmt::Debug for Entry<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    PointerElement::Struct(&Entry::META, self.data.clone()).fmt(f)
  }
}

pub struct EntryShared {
  data: UntypedStructShared,
}

impl EntryShared {
  pub fn new(
    term: u64,
    index: u64,
    payload: &'_ [u8],
  ) -> EntryShared {
    let mut data = UntypedStructOwned::new_with_root_struct(NumWords(2), NumWords(1));
    Entry::TERM_META.set(&mut data, term);
    Entry::INDEX_META.set(&mut data, index);
    Entry::PAYLOAD_META.set(&mut data, payload);
    EntryShared { data: data.into_shared() }
  }

  pub fn as_ref<'a>(&'a self) -> Entry<'a> {
    Entry { data: self.data.as_ref() }
  }
}

impl TypedStructShared for EntryShared {
  fn to_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

/// An rpc message.
#[derive(Clone)]
pub struct Message<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> Message<'a> {
  const SRC_META: U64FieldMeta = U64FieldMeta {
    name: "src",
    offset: NumElements(0),
  };
  const DEST_META: U64FieldMeta = U64FieldMeta {
    name: "dest",
    offset: NumElements(1),
  };

  const META: StructMeta = StructMeta {
    name: "Message",
    fields: &[
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(Message::SRC_META)),
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(Message::DEST_META)),
    ],
  };

  /// The node sending this rpc.
  pub fn src(&self) -> u64 { Message::SRC_META.get(&self.data) }
  /// The node to receive this rpc.
  pub fn dest(&self) -> u64 { Message::DEST_META.get(&self.data) }
}

impl<'a> TypedStruct<'a> for Message<'a> {
  fn meta(&self) -> &'static StructMeta {
    &Message::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    Message { data: data }
  }
  fn to_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> std::fmt::Debug for Message<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    PointerElement::Struct(&Message::META, self.data.clone()).fmt(f)
  }
}

pub struct MessageShared {
  data: UntypedStructShared,
}

impl MessageShared {
  pub fn new(
    src: u64,
    dest: u64,
  ) -> MessageShared {
    let mut data = UntypedStructOwned::new_with_root_struct(NumWords(3), NumWords(1));
    Message::SRC_META.set(&mut data, src);
    Message::DEST_META.set(&mut data, dest);
    MessageShared { data: data.into_shared() }
  }

  pub fn as_ref<'a>(&'a self) -> Message<'a> {
    Message { data: self.data.as_ref() }
  }
}

impl TypedStructShared for MessageShared {
  fn to_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

#[derive(Clone)]
pub struct AppendEntriesReq<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> AppendEntriesReq<'a> {
  const TERM_META: U64FieldMeta = U64FieldMeta {
    name: "term",
    offset: NumElements(0),
  };
  const LEADER_ID_META: U64FieldMeta = U64FieldMeta {
    name: "leader_id",
    offset: NumElements(1),
  };
  const PREV_LOG_INDEX_META: U64FieldMeta = U64FieldMeta {
    name: "prev_log_index",
    offset: NumElements(2),
  };
  const PREV_LOG_TERM_META: U64FieldMeta = U64FieldMeta {
    name: "prev_log_term",
    offset: NumElements(3),
  };
  const LEADER_COMMIT_META: U64FieldMeta = U64FieldMeta {
    name: "leader_commit",
    offset: NumElements(4),
  };
  const READ_ID_META: U64FieldMeta = U64FieldMeta {
    name: "read_id",
    offset: NumElements(5),
  };
  const ENTRIES_META: ListFieldMeta = ListFieldMeta {
    name: "entries",
    offset: NumElements(0),
    get_element: |data, sink| sink.list(AppendEntriesReq{data: data.clone()}.entries().to_element_list()),
  };

  const META: StructMeta = StructMeta {
    name: "AppendEntriesReq",
    fields: &[
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(AppendEntriesReq::TERM_META)),
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(AppendEntriesReq::LEADER_ID_META)),
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(AppendEntriesReq::PREV_LOG_INDEX_META)),
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(AppendEntriesReq::PREV_LOG_TERM_META)),
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(AppendEntriesReq::LEADER_COMMIT_META)),
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(AppendEntriesReq::READ_ID_META)),
      FieldMeta::Pointer(PointerFieldMeta::List(AppendEntriesReq::ENTRIES_META)),
    ],
  };

  pub fn term(&self) -> u64 { AppendEntriesReq::TERM_META.get(&self.data) }
  pub fn leader_id(&self) -> u64 { AppendEntriesReq::LEADER_ID_META.get(&self.data) }
  pub fn prev_log_index(&self) -> u64 { AppendEntriesReq::PREV_LOG_INDEX_META.get(&self.data) }
  pub fn prev_log_term(&self) -> u64 { AppendEntriesReq::PREV_LOG_TERM_META.get(&self.data) }
  pub fn leader_commit(&self) -> u64 { AppendEntriesReq::LEADER_COMMIT_META.get(&self.data) }
  pub fn read_id(&self) -> u64 { AppendEntriesReq::READ_ID_META.get(&self.data) }
  pub fn entries(&self) -> Result<Vec<Entry<'a>>, Error> { AppendEntriesReq::ENTRIES_META.get(&self.data) }
}

impl<'a> TypedStruct<'a> for AppendEntriesReq<'a> {
  fn meta(&self) -> &'static StructMeta {
    &AppendEntriesReq::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    AppendEntriesReq { data: data }
  }
  fn to_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> std::fmt::Debug for AppendEntriesReq<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    PointerElement::Struct(&AppendEntriesReq::META, self.data.clone()).fmt(f)
  }
}

pub struct AppendEntriesReqShared {
  data: UntypedStructShared,
}

impl AppendEntriesReqShared {
  pub fn new(
    term: u64,
    leader_id: u64,
    prev_log_index: u64,
    prev_log_term: u64,
    leader_commit: u64,
    read_id: u64,
    entries: &'_ [EntryShared],
  ) -> AppendEntriesReqShared {
    let mut data = UntypedStructOwned::new_with_root_struct(NumWords(6), NumWords(1));
    AppendEntriesReq::TERM_META.set(&mut data, term);
    AppendEntriesReq::LEADER_ID_META.set(&mut data, leader_id);
    AppendEntriesReq::PREV_LOG_INDEX_META.set(&mut data, prev_log_index);
    AppendEntriesReq::PREV_LOG_TERM_META.set(&mut data, prev_log_term);
    AppendEntriesReq::LEADER_COMMIT_META.set(&mut data, leader_commit);
    AppendEntriesReq::READ_ID_META.set(&mut data, read_id);
    AppendEntriesReq::ENTRIES_META.set(&mut data, entries);
    AppendEntriesReqShared { data: data.into_shared() }
  }

  pub fn as_ref<'a>(&'a self) -> AppendEntriesReq<'a> {
    AppendEntriesReq { data: self.data.as_ref() }
  }
}

impl TypedStructShared for AppendEntriesReqShared {
  fn to_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

#[derive(Clone)]
pub struct AppendEntriesRes<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> AppendEntriesRes<'a> {
  const TERM_META: U64FieldMeta = U64FieldMeta {
    name: "term",
    offset: NumElements(0),
  };
  const SUCCESS_META: U64FieldMeta = U64FieldMeta {
    name: "success",
    offset: NumElements(1),
  };
  const INDEX_META: U64FieldMeta = U64FieldMeta {
    name: "index",
    offset: NumElements(2),
  };
  const READ_ID_META: U64FieldMeta = U64FieldMeta {
    name: "read_id",
    offset: NumElements(3),
  };

  const META: StructMeta = StructMeta {
    name: "AppendEntriesRes",
    fields: &[
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(AppendEntriesRes::TERM_META)),
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(AppendEntriesRes::SUCCESS_META)),
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(AppendEntriesRes::INDEX_META)),
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(AppendEntriesRes::READ_ID_META)),
    ],
  };

  pub fn term(&self) -> u64 { AppendEntriesRes::TERM_META.get(&self.data) }
  pub fn success(&self) -> u64 { AppendEntriesRes::SUCCESS_META.get(&self.data) }
  pub fn index(&self) -> u64 { AppendEntriesRes::INDEX_META.get(&self.data) }
  pub fn read_id(&self) -> u64 { AppendEntriesRes::READ_ID_META.get(&self.data) }
}

impl<'a> TypedStruct<'a> for AppendEntriesRes<'a> {
  fn meta(&self) -> &'static StructMeta {
    &AppendEntriesRes::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    AppendEntriesRes { data: data }
  }
  fn to_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> std::fmt::Debug for AppendEntriesRes<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    PointerElement::Struct(&AppendEntriesRes::META, self.data.clone()).fmt(f)
  }
}

pub struct AppendEntriesResShared {
  data: UntypedStructShared,
}

impl AppendEntriesResShared {
  pub fn new(
    term: u64,
    success: u64,
    index: u64,
    read_id: u64,
  ) -> AppendEntriesResShared {
    let mut data = UntypedStructOwned::new_with_root_struct(NumWords(4), NumWords(0));
    AppendEntriesRes::TERM_META.set(&mut data, term);
    AppendEntriesRes::SUCCESS_META.set(&mut data, success);
    AppendEntriesRes::INDEX_META.set(&mut data, index);
    AppendEntriesRes::READ_ID_META.set(&mut data, read_id);
    AppendEntriesResShared { data: data.into_shared() }
  }

  pub fn as_ref<'a>(&'a self) -> AppendEntriesRes<'a> {
    AppendEntriesRes { data: self.data.as_ref() }
  }
}

impl TypedStructShared for AppendEntriesResShared {
  fn to_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

#[derive(Clone)]
pub struct RequestVoteReq<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> RequestVoteReq<'a> {
  const TERM_META: U64FieldMeta = U64FieldMeta {
    name: "term",
    offset: NumElements(0),
  };
  const CANDIDATE_ID_META: U64FieldMeta = U64FieldMeta {
    name: "candidate_id",
    offset: NumElements(1),
  };
  const LAST_LOG_INDEX_META: U64FieldMeta = U64FieldMeta {
    name: "last_log_index",
    offset: NumElements(2),
  };
  const LAST_LOG_TERM_META: U64FieldMeta = U64FieldMeta {
    name: "last_log_term",
    offset: NumElements(3),
  };

  const META: StructMeta = StructMeta {
    name: "RequestVoteReq",
    fields: &[
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(RequestVoteReq::TERM_META)),
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(RequestVoteReq::CANDIDATE_ID_META)),
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(RequestVoteReq::LAST_LOG_INDEX_META)),
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(RequestVoteReq::LAST_LOG_TERM_META)),
    ],
  };

  pub fn term(&self) -> u64 { RequestVoteReq::TERM_META.get(&self.data) }
  pub fn candidate_id(&self) -> u64 { RequestVoteReq::CANDIDATE_ID_META.get(&self.data) }
  pub fn last_log_index(&self) -> u64 { RequestVoteReq::LAST_LOG_INDEX_META.get(&self.data) }
  pub fn last_log_term(&self) -> u64 { RequestVoteReq::LAST_LOG_TERM_META.get(&self.data) }
}

impl<'a> TypedStruct<'a> for RequestVoteReq<'a> {
  fn meta(&self) -> &'static StructMeta {
    &RequestVoteReq::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    RequestVoteReq { data: data }
  }
  fn to_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> std::fmt::Debug for RequestVoteReq<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    PointerElement::Struct(&RequestVoteReq::META, self.data.clone()).fmt(f)
  }
}

pub struct RequestVoteReqShared {
  data: UntypedStructShared,
}

impl RequestVoteReqShared {
  pub fn new(
    term: u64,
    candidate_id: u64,
    last_log_index: u64,
    last_log_term: u64,
  ) -> RequestVoteReqShared {
    let mut data = UntypedStructOwned::new_with_root_struct(NumWords(4), NumWords(0));
    RequestVoteReq::TERM_META.set(&mut data, term);
    RequestVoteReq::CANDIDATE_ID_META.set(&mut data, candidate_id);
    RequestVoteReq::LAST_LOG_INDEX_META.set(&mut data, last_log_index);
    RequestVoteReq::LAST_LOG_TERM_META.set(&mut data, last_log_term);
    RequestVoteReqShared { data: data.into_shared() }
  }

  pub fn as_ref<'a>(&'a self) -> RequestVoteReq<'a> {
    RequestVoteReq { data: self.data.as_ref() }
  }
}

impl TypedStructShared for RequestVoteReqShared {
  fn to_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

#[derive(Clone)]
pub struct RequestVoteRes<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> RequestVoteRes<'a> {
  const TERM_META: U64FieldMeta = U64FieldMeta {
    name: "term",
    offset: NumElements(0),
  };
  const VOTE_GRANTED_META: U64FieldMeta = U64FieldMeta {
    name: "vote_granted",
    offset: NumElements(1),
  };

  const META: StructMeta = StructMeta {
    name: "RequestVoteRes",
    fields: &[
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(RequestVoteRes::TERM_META)),
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(RequestVoteRes::VOTE_GRANTED_META)),
    ],
  };

  pub fn term(&self) -> u64 { RequestVoteRes::TERM_META.get(&self.data) }
  pub fn vote_granted(&self) -> u64 { RequestVoteRes::VOTE_GRANTED_META.get(&self.data) }
}

impl<'a> TypedStruct<'a> for RequestVoteRes<'a> {
  fn meta(&self) -> &'static StructMeta {
    &RequestVoteRes::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    RequestVoteRes { data: data }
  }
  fn to_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> std::fmt::Debug for RequestVoteRes<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    PointerElement::Struct(&RequestVoteRes::META, self.data.clone()).fmt(f)
  }
}

pub struct RequestVoteResShared {
  data: UntypedStructShared,
}

impl RequestVoteResShared {
  pub fn new(
    term: u64,
    vote_granted: u64,
  ) -> RequestVoteResShared {
    let mut data = UntypedStructOwned::new_with_root_struct(NumWords(2), NumWords(0));
    RequestVoteRes::TERM_META.set(&mut data, term);
    RequestVoteRes::VOTE_GRANTED_META.set(&mut data, vote_granted);
    RequestVoteResShared { data: data.into_shared() }
  }

  pub fn as_ref<'a>(&'a self) -> RequestVoteRes<'a> {
    RequestVoteRes { data: self.data.as_ref() }
  }
}

impl TypedStructShared for RequestVoteResShared {
  fn to_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

#[derive(Clone)]
pub struct StartElectionReq<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> StartElectionReq<'a> {
  const TERM_META: U64FieldMeta = U64FieldMeta {
    name: "term",
    offset: NumElements(0),
  };

  const META: StructMeta = StructMeta {
    name: "StartElectionReq",
    fields: &[
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(StartElectionReq::TERM_META)),
    ],
  };

  pub fn term(&self) -> u64 { StartElectionReq::TERM_META.get(&self.data) }
}

impl<'a> TypedStruct<'a> for StartElectionReq<'a> {
  fn meta(&self) -> &'static StructMeta {
    &StartElectionReq::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    StartElectionReq { data: data }
  }
  fn to_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> std::fmt::Debug for StartElectionReq<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    PointerElement::Struct(&StartElectionReq::META, self.data.clone()).fmt(f)
  }
}

pub struct StartElectionReqShared {
  data: UntypedStructShared,
}

impl StartElectionReqShared {
  pub fn new(
    term: u64,
  ) -> StartElectionReqShared {
    let mut data = UntypedStructOwned::new_with_root_struct(NumWords(1), NumWords(0));
    StartElectionReq::TERM_META.set(&mut data, term);
    StartElectionReqShared { data: data.into_shared() }
  }

  pub fn as_ref<'a>(&'a self) -> StartElectionReq<'a> {
    StartElectionReq { data: self.data.as_ref() }
  }
}

impl TypedStructShared for StartElectionReqShared {
  fn to_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

