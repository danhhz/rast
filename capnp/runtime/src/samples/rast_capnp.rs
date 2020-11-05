use capnp_runtime::prelude::*;

pub struct EntryMeta;

impl EntryMeta {
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
      FieldMeta::U64(EntryMeta::TERM_META),
      FieldMeta::U64(EntryMeta::INDEX_META),
      FieldMeta::Data(EntryMeta::PAYLOAD_META),
    ],
  };
}

impl<'a> TypedStruct<'a> for EntryMeta {
  type Ref = EntryRef<'a>;
  type Shared = EntryShared;
  fn meta() -> &'static StructMeta {
    &EntryMeta::META
  }
}

pub trait Entry {

  /// The term of the entry.
  fn term<'a>(&'a self) -> Term;

  /// The index of the entry.
  fn index<'a>(&'a self) -> Index;

  /// The opaque user payload of the entry.
  fn payload<'a>(&'a self) -> Result<&'a [u8], Error>;
}

/// An entry in the Raft log.
#[derive(Clone)]
pub struct EntryRef<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> EntryRef<'a> {

  /// The term of the entry.
  pub fn term(&self) -> Term {Term(EntryMeta::TERM_META.get(&self.data)) }

  /// The index of the entry.
  pub fn index(&self) -> Index {Index(EntryMeta::INDEX_META.get(&self.data)) }

  /// The opaque user payload of the entry.
  pub fn payload(&self) -> Result<&'a [u8], Error> {EntryMeta::PAYLOAD_META.get(&self.data) }

  pub fn capnp_to_owned(&self) -> EntryShared {
    EntryShared { data: self.data.capnp_to_owned() }
  }
}

impl Entry for EntryRef<'_> {
  fn term<'a>(&'a self) -> Term {
    self.term()
 }
  fn index<'a>(&'a self) -> Index {
    self.index()
 }
  fn payload<'a>(&'a self) -> Result<&'a [u8], Error> {
    self.payload()
 }
}

impl<'a> TypedStructRef<'a> for EntryRef<'a> {
  fn meta() -> &'static StructMeta {
    &EntryMeta::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    EntryRef { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for EntryRef<'a> {
  type Owned = EntryShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    EntryRef::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for EntryRef<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for EntryRef<'a> {
  fn partial_cmp(&self, other: &EntryRef<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for EntryRef<'a> {
  fn eq(&self, other: &EntryRef<'a>) -> bool {
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
    let mut data = UntypedStructOwned::new_with_root_struct(EntryMeta::META.data_size, EntryMeta::META.pointer_size);
    EntryMeta::TERM_META.set(&mut data, term.0);
    EntryMeta::INDEX_META.set(&mut data, index.0);
    EntryMeta::PAYLOAD_META.set(&mut data, payload);
    EntryShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> EntryRef<'a> {
    EntryRef { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for EntryShared {
  fn meta() -> &'static StructMeta {
    &EntryMeta::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    EntryShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, EntryRef<'a>> for EntryShared {
  fn capnp_as_ref(&'a self) -> EntryRef<'a> {
    EntryShared::capnp_as_ref(self)
  }
}

pub struct MessageMeta;

impl MessageMeta {
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
      FieldMeta::U64(MessageMeta::SRC_META),
      FieldMeta::U64(MessageMeta::DEST_META),
      FieldMeta::Union(MessageMeta::PAYLOAD_META),
    ],
  };
}

impl<'a> TypedStruct<'a> for MessageMeta {
  type Ref = MessageRef<'a>;
  type Shared = MessageShared;
  fn meta() -> &'static StructMeta {
    &MessageMeta::META
  }
}

pub trait Message {

  /// The node sending this rpc.
  fn src<'a>(&'a self) -> NodeID;

  /// The node to receive this rpc.
  fn dest<'a>(&'a self) -> NodeID;

  fn payload<'a>(&'a self) -> Result<Result<Payload<'a>, UnknownDiscriminant>,Error>;
}

/// An rpc message.
#[derive(Clone)]
pub struct MessageRef<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> MessageRef<'a> {

  /// The node sending this rpc.
  pub fn src(&self) -> NodeID {NodeID(MessageMeta::SRC_META.get(&self.data)) }

  /// The node to receive this rpc.
  pub fn dest(&self) -> NodeID {NodeID(MessageMeta::DEST_META.get(&self.data)) }

  pub fn payload(&self) -> Result<Result<Payload<'a>, UnknownDiscriminant>,Error> {MessageMeta::PAYLOAD_META.get(&self.data) }

  pub fn capnp_to_owned(&self) -> MessageShared {
    MessageShared { data: self.data.capnp_to_owned() }
  }
}

impl Message for MessageRef<'_> {
  fn src<'a>(&'a self) -> NodeID {
    self.src()
 }
  fn dest<'a>(&'a self) -> NodeID {
    self.dest()
 }
  fn payload<'a>(&'a self) -> Result<Result<Payload<'a>, UnknownDiscriminant>,Error> {
    self.payload()
 }
}

impl<'a> TypedStructRef<'a> for MessageRef<'a> {
  fn meta() -> &'static StructMeta {
    &MessageMeta::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    MessageRef { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for MessageRef<'a> {
  type Owned = MessageShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    MessageRef::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for MessageRef<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for MessageRef<'a> {
  fn partial_cmp(&self, other: &MessageRef<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for MessageRef<'a> {
  fn eq(&self, other: &MessageRef<'a>) -> bool {
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
    let mut data = UntypedStructOwned::new_with_root_struct(MessageMeta::META.data_size, MessageMeta::META.pointer_size);
    MessageMeta::SRC_META.set(&mut data, src.0);
    MessageMeta::DEST_META.set(&mut data, dest.0);
    MessageMeta::PAYLOAD_META.set(&mut data, payload);
    MessageShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> MessageRef<'a> {
    MessageRef { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for MessageShared {
  fn meta() -> &'static StructMeta {
    &MessageMeta::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    MessageShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, MessageRef<'a>> for MessageShared {
  fn capnp_as_ref(&'a self) -> MessageRef<'a> {
    MessageShared::capnp_as_ref(self)
  }
}

pub struct AppendEntriesReqMeta;

impl AppendEntriesReqMeta {
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
      value_type: ElementType::Struct(&EntryMeta::META)
    },
  };

  const META: &'static StructMeta = &StructMeta {
    name: "AppendEntriesReq",
    data_size: NumWords(6),
    pointer_size: NumWords(1),
    fields: || &[
      FieldMeta::U64(AppendEntriesReqMeta::TERM_META),
      FieldMeta::U64(AppendEntriesReqMeta::LEADER_ID_META),
      FieldMeta::U64(AppendEntriesReqMeta::PREV_LOG_INDEX_META),
      FieldMeta::U64(AppendEntriesReqMeta::PREV_LOG_TERM_META),
      FieldMeta::U64(AppendEntriesReqMeta::LEADER_COMMIT_META),
      FieldMeta::U64(AppendEntriesReqMeta::READ_ID_META),
      FieldMeta::List(AppendEntriesReqMeta::ENTRIES_META),
    ],
  };
}

impl<'a> TypedStruct<'a> for AppendEntriesReqMeta {
  type Ref = AppendEntriesReqRef<'a>;
  type Shared = AppendEntriesReqShared;
  fn meta() -> &'static StructMeta {
    &AppendEntriesReqMeta::META
  }
}

pub trait AppendEntriesReq {

  fn term<'a>(&'a self) -> Term;

  fn leader_id<'a>(&'a self) -> NodeID;

  fn prev_log_index<'a>(&'a self) -> Index;

  fn prev_log_term<'a>(&'a self) -> Term;

  fn leader_commit<'a>(&'a self) -> Index;

  fn read_id<'a>(&'a self) -> ReadID;

  fn entries<'a>(&'a self) -> Result<Slice<'a, EntryRef<'a>>, Error>;
}

#[derive(Clone)]
pub struct AppendEntriesReqRef<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> AppendEntriesReqRef<'a> {

  pub fn term(&self) -> Term {Term(AppendEntriesReqMeta::TERM_META.get(&self.data)) }

  pub fn leader_id(&self) -> NodeID {NodeID(AppendEntriesReqMeta::LEADER_ID_META.get(&self.data)) }

  pub fn prev_log_index(&self) -> Index {Index(AppendEntriesReqMeta::PREV_LOG_INDEX_META.get(&self.data)) }

  pub fn prev_log_term(&self) -> Term {Term(AppendEntriesReqMeta::PREV_LOG_TERM_META.get(&self.data)) }

  pub fn leader_commit(&self) -> Index {Index(AppendEntriesReqMeta::LEADER_COMMIT_META.get(&self.data)) }

  pub fn read_id(&self) -> ReadID {ReadID(AppendEntriesReqMeta::READ_ID_META.get(&self.data)) }

  pub fn entries(&self) -> Result<Slice<'a, EntryRef<'a>>, Error> {AppendEntriesReqMeta::ENTRIES_META.get(&self.data) }

  pub fn capnp_to_owned(&self) -> AppendEntriesReqShared {
    AppendEntriesReqShared { data: self.data.capnp_to_owned() }
  }
}

impl AppendEntriesReq for AppendEntriesReqRef<'_> {
  fn term<'a>(&'a self) -> Term {
    self.term()
 }
  fn leader_id<'a>(&'a self) -> NodeID {
    self.leader_id()
 }
  fn prev_log_index<'a>(&'a self) -> Index {
    self.prev_log_index()
 }
  fn prev_log_term<'a>(&'a self) -> Term {
    self.prev_log_term()
 }
  fn leader_commit<'a>(&'a self) -> Index {
    self.leader_commit()
 }
  fn read_id<'a>(&'a self) -> ReadID {
    self.read_id()
 }
  fn entries<'a>(&'a self) -> Result<Slice<'a, EntryRef<'a>>, Error> {
    self.entries()
 }
}

impl<'a> TypedStructRef<'a> for AppendEntriesReqRef<'a> {
  fn meta() -> &'static StructMeta {
    &AppendEntriesReqMeta::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    AppendEntriesReqRef { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for AppendEntriesReqRef<'a> {
  type Owned = AppendEntriesReqShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    AppendEntriesReqRef::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for AppendEntriesReqRef<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for AppendEntriesReqRef<'a> {
  fn partial_cmp(&self, other: &AppendEntriesReqRef<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for AppendEntriesReqRef<'a> {
  fn eq(&self, other: &AppendEntriesReqRef<'a>) -> bool {
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
    let mut data = UntypedStructOwned::new_with_root_struct(AppendEntriesReqMeta::META.data_size, AppendEntriesReqMeta::META.pointer_size);
    AppendEntriesReqMeta::TERM_META.set(&mut data, term.0);
    AppendEntriesReqMeta::LEADER_ID_META.set(&mut data, leader_id.0);
    AppendEntriesReqMeta::PREV_LOG_INDEX_META.set(&mut data, prev_log_index.0);
    AppendEntriesReqMeta::PREV_LOG_TERM_META.set(&mut data, prev_log_term.0);
    AppendEntriesReqMeta::LEADER_COMMIT_META.set(&mut data, leader_commit.0);
    AppendEntriesReqMeta::READ_ID_META.set(&mut data, read_id.0);
    AppendEntriesReqMeta::ENTRIES_META.set(&mut data, entries);
    AppendEntriesReqShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> AppendEntriesReqRef<'a> {
    AppendEntriesReqRef { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for AppendEntriesReqShared {
  fn meta() -> &'static StructMeta {
    &AppendEntriesReqMeta::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    AppendEntriesReqShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, AppendEntriesReqRef<'a>> for AppendEntriesReqShared {
  fn capnp_as_ref(&'a self) -> AppendEntriesReqRef<'a> {
    AppendEntriesReqShared::capnp_as_ref(self)
  }
}

pub struct AppendEntriesResMeta;

impl AppendEntriesResMeta {
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
      FieldMeta::U64(AppendEntriesResMeta::TERM_META),
      FieldMeta::U64(AppendEntriesResMeta::SUCCESS_META),
      FieldMeta::U64(AppendEntriesResMeta::INDEX_META),
      FieldMeta::U64(AppendEntriesResMeta::READ_ID_META),
    ],
  };
}

impl<'a> TypedStruct<'a> for AppendEntriesResMeta {
  type Ref = AppendEntriesResRef<'a>;
  type Shared = AppendEntriesResShared;
  fn meta() -> &'static StructMeta {
    &AppendEntriesResMeta::META
  }
}

pub trait AppendEntriesRes {

  fn term<'a>(&'a self) -> Term;

  fn success<'a>(&'a self) -> u64;

  fn index<'a>(&'a self) -> Index;

  fn read_id<'a>(&'a self) -> ReadID;
}

#[derive(Clone)]
pub struct AppendEntriesResRef<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> AppendEntriesResRef<'a> {

  pub fn term(&self) -> Term {Term(AppendEntriesResMeta::TERM_META.get(&self.data)) }

  pub fn success(&self) -> u64 {AppendEntriesResMeta::SUCCESS_META.get(&self.data) }

  pub fn index(&self) -> Index {Index(AppendEntriesResMeta::INDEX_META.get(&self.data)) }

  pub fn read_id(&self) -> ReadID {ReadID(AppendEntriesResMeta::READ_ID_META.get(&self.data)) }

  pub fn capnp_to_owned(&self) -> AppendEntriesResShared {
    AppendEntriesResShared { data: self.data.capnp_to_owned() }
  }
}

impl AppendEntriesRes for AppendEntriesResRef<'_> {
  fn term<'a>(&'a self) -> Term {
    self.term()
 }
  fn success<'a>(&'a self) -> u64 {
    self.success()
 }
  fn index<'a>(&'a self) -> Index {
    self.index()
 }
  fn read_id<'a>(&'a self) -> ReadID {
    self.read_id()
 }
}

impl<'a> TypedStructRef<'a> for AppendEntriesResRef<'a> {
  fn meta() -> &'static StructMeta {
    &AppendEntriesResMeta::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    AppendEntriesResRef { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for AppendEntriesResRef<'a> {
  type Owned = AppendEntriesResShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    AppendEntriesResRef::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for AppendEntriesResRef<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for AppendEntriesResRef<'a> {
  fn partial_cmp(&self, other: &AppendEntriesResRef<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for AppendEntriesResRef<'a> {
  fn eq(&self, other: &AppendEntriesResRef<'a>) -> bool {
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
    let mut data = UntypedStructOwned::new_with_root_struct(AppendEntriesResMeta::META.data_size, AppendEntriesResMeta::META.pointer_size);
    AppendEntriesResMeta::TERM_META.set(&mut data, term.0);
    AppendEntriesResMeta::SUCCESS_META.set(&mut data, success);
    AppendEntriesResMeta::INDEX_META.set(&mut data, index.0);
    AppendEntriesResMeta::READ_ID_META.set(&mut data, read_id.0);
    AppendEntriesResShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> AppendEntriesResRef<'a> {
    AppendEntriesResRef { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for AppendEntriesResShared {
  fn meta() -> &'static StructMeta {
    &AppendEntriesResMeta::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    AppendEntriesResShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, AppendEntriesResRef<'a>> for AppendEntriesResShared {
  fn capnp_as_ref(&'a self) -> AppendEntriesResRef<'a> {
    AppendEntriesResShared::capnp_as_ref(self)
  }
}

pub struct RequestVoteReqMeta;

impl RequestVoteReqMeta {
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
      FieldMeta::U64(RequestVoteReqMeta::TERM_META),
      FieldMeta::U64(RequestVoteReqMeta::CANDIDATE_ID_META),
      FieldMeta::U64(RequestVoteReqMeta::LAST_LOG_INDEX_META),
      FieldMeta::U64(RequestVoteReqMeta::LAST_LOG_TERM_META),
    ],
  };
}

impl<'a> TypedStruct<'a> for RequestVoteReqMeta {
  type Ref = RequestVoteReqRef<'a>;
  type Shared = RequestVoteReqShared;
  fn meta() -> &'static StructMeta {
    &RequestVoteReqMeta::META
  }
}

pub trait RequestVoteReq {

  fn term<'a>(&'a self) -> Term;

  fn candidate_id<'a>(&'a self) -> NodeID;

  fn last_log_index<'a>(&'a self) -> Index;

  fn last_log_term<'a>(&'a self) -> Term;
}

#[derive(Clone)]
pub struct RequestVoteReqRef<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> RequestVoteReqRef<'a> {

  pub fn term(&self) -> Term {Term(RequestVoteReqMeta::TERM_META.get(&self.data)) }

  pub fn candidate_id(&self) -> NodeID {NodeID(RequestVoteReqMeta::CANDIDATE_ID_META.get(&self.data)) }

  pub fn last_log_index(&self) -> Index {Index(RequestVoteReqMeta::LAST_LOG_INDEX_META.get(&self.data)) }

  pub fn last_log_term(&self) -> Term {Term(RequestVoteReqMeta::LAST_LOG_TERM_META.get(&self.data)) }

  pub fn capnp_to_owned(&self) -> RequestVoteReqShared {
    RequestVoteReqShared { data: self.data.capnp_to_owned() }
  }
}

impl RequestVoteReq for RequestVoteReqRef<'_> {
  fn term<'a>(&'a self) -> Term {
    self.term()
 }
  fn candidate_id<'a>(&'a self) -> NodeID {
    self.candidate_id()
 }
  fn last_log_index<'a>(&'a self) -> Index {
    self.last_log_index()
 }
  fn last_log_term<'a>(&'a self) -> Term {
    self.last_log_term()
 }
}

impl<'a> TypedStructRef<'a> for RequestVoteReqRef<'a> {
  fn meta() -> &'static StructMeta {
    &RequestVoteReqMeta::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    RequestVoteReqRef { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for RequestVoteReqRef<'a> {
  type Owned = RequestVoteReqShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    RequestVoteReqRef::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for RequestVoteReqRef<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for RequestVoteReqRef<'a> {
  fn partial_cmp(&self, other: &RequestVoteReqRef<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for RequestVoteReqRef<'a> {
  fn eq(&self, other: &RequestVoteReqRef<'a>) -> bool {
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
    let mut data = UntypedStructOwned::new_with_root_struct(RequestVoteReqMeta::META.data_size, RequestVoteReqMeta::META.pointer_size);
    RequestVoteReqMeta::TERM_META.set(&mut data, term.0);
    RequestVoteReqMeta::CANDIDATE_ID_META.set(&mut data, candidate_id.0);
    RequestVoteReqMeta::LAST_LOG_INDEX_META.set(&mut data, last_log_index.0);
    RequestVoteReqMeta::LAST_LOG_TERM_META.set(&mut data, last_log_term.0);
    RequestVoteReqShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> RequestVoteReqRef<'a> {
    RequestVoteReqRef { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for RequestVoteReqShared {
  fn meta() -> &'static StructMeta {
    &RequestVoteReqMeta::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    RequestVoteReqShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, RequestVoteReqRef<'a>> for RequestVoteReqShared {
  fn capnp_as_ref(&'a self) -> RequestVoteReqRef<'a> {
    RequestVoteReqShared::capnp_as_ref(self)
  }
}

pub struct RequestVoteResMeta;

impl RequestVoteResMeta {
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
      FieldMeta::U64(RequestVoteResMeta::TERM_META),
      FieldMeta::U64(RequestVoteResMeta::VOTE_GRANTED_META),
    ],
  };
}

impl<'a> TypedStruct<'a> for RequestVoteResMeta {
  type Ref = RequestVoteResRef<'a>;
  type Shared = RequestVoteResShared;
  fn meta() -> &'static StructMeta {
    &RequestVoteResMeta::META
  }
}

pub trait RequestVoteRes {

  fn term<'a>(&'a self) -> Term;

  fn vote_granted<'a>(&'a self) -> u64;
}

#[derive(Clone)]
pub struct RequestVoteResRef<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> RequestVoteResRef<'a> {

  pub fn term(&self) -> Term {Term(RequestVoteResMeta::TERM_META.get(&self.data)) }

  pub fn vote_granted(&self) -> u64 {RequestVoteResMeta::VOTE_GRANTED_META.get(&self.data) }

  pub fn capnp_to_owned(&self) -> RequestVoteResShared {
    RequestVoteResShared { data: self.data.capnp_to_owned() }
  }
}

impl RequestVoteRes for RequestVoteResRef<'_> {
  fn term<'a>(&'a self) -> Term {
    self.term()
 }
  fn vote_granted<'a>(&'a self) -> u64 {
    self.vote_granted()
 }
}

impl<'a> TypedStructRef<'a> for RequestVoteResRef<'a> {
  fn meta() -> &'static StructMeta {
    &RequestVoteResMeta::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    RequestVoteResRef { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for RequestVoteResRef<'a> {
  type Owned = RequestVoteResShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    RequestVoteResRef::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for RequestVoteResRef<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for RequestVoteResRef<'a> {
  fn partial_cmp(&self, other: &RequestVoteResRef<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for RequestVoteResRef<'a> {
  fn eq(&self, other: &RequestVoteResRef<'a>) -> bool {
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
    let mut data = UntypedStructOwned::new_with_root_struct(RequestVoteResMeta::META.data_size, RequestVoteResMeta::META.pointer_size);
    RequestVoteResMeta::TERM_META.set(&mut data, term.0);
    RequestVoteResMeta::VOTE_GRANTED_META.set(&mut data, vote_granted);
    RequestVoteResShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> RequestVoteResRef<'a> {
    RequestVoteResRef { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for RequestVoteResShared {
  fn meta() -> &'static StructMeta {
    &RequestVoteResMeta::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    RequestVoteResShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, RequestVoteResRef<'a>> for RequestVoteResShared {
  fn capnp_as_ref(&'a self) -> RequestVoteResRef<'a> {
    RequestVoteResShared::capnp_as_ref(self)
  }
}

pub struct StartElectionReqMeta;

impl StartElectionReqMeta {
  const TERM_META: &'static U64FieldMeta = &U64FieldMeta {
    name: "term",
    offset: NumElements(0),
  };

  const META: &'static StructMeta = &StructMeta {
    name: "StartElectionReq",
    data_size: NumWords(1),
    pointer_size: NumWords(0),
    fields: || &[
      FieldMeta::U64(StartElectionReqMeta::TERM_META),
    ],
  };
}

impl<'a> TypedStruct<'a> for StartElectionReqMeta {
  type Ref = StartElectionReqRef<'a>;
  type Shared = StartElectionReqShared;
  fn meta() -> &'static StructMeta {
    &StartElectionReqMeta::META
  }
}

pub trait StartElectionReq {

  fn term<'a>(&'a self) -> Term;
}

#[derive(Clone)]
pub struct StartElectionReqRef<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> StartElectionReqRef<'a> {

  pub fn term(&self) -> Term {Term(StartElectionReqMeta::TERM_META.get(&self.data)) }

  pub fn capnp_to_owned(&self) -> StartElectionReqShared {
    StartElectionReqShared { data: self.data.capnp_to_owned() }
  }
}

impl StartElectionReq for StartElectionReqRef<'_> {
  fn term<'a>(&'a self) -> Term {
    self.term()
 }
}

impl<'a> TypedStructRef<'a> for StartElectionReqRef<'a> {
  fn meta() -> &'static StructMeta {
    &StartElectionReqMeta::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    StartElectionReqRef { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for StartElectionReqRef<'a> {
  type Owned = StartElectionReqShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    StartElectionReqRef::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for StartElectionReqRef<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for StartElectionReqRef<'a> {
  fn partial_cmp(&self, other: &StartElectionReqRef<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for StartElectionReqRef<'a> {
  fn eq(&self, other: &StartElectionReqRef<'a>) -> bool {
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
    let mut data = UntypedStructOwned::new_with_root_struct(StartElectionReqMeta::META.data_size, StartElectionReqMeta::META.pointer_size);
    StartElectionReqMeta::TERM_META.set(&mut data, term.0);
    StartElectionReqShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> StartElectionReqRef<'a> {
    StartElectionReqRef { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for StartElectionReqShared {
  fn meta() -> &'static StructMeta {
    &StartElectionReqMeta::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    StartElectionReqShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, StartElectionReqRef<'a>> for StartElectionReqShared {
  fn capnp_as_ref(&'a self) -> StartElectionReqRef<'a> {
    StartElectionReqShared::capnp_as_ref(self)
  }
}

#[derive(Clone)]
pub enum Payload<'a> {
  AppendEntriesReq(AppendEntriesReqRef<'a>),
  AppendEntriesRes(AppendEntriesResRef<'a>),
  RequestVoteReq(RequestVoteReqRef<'a>),
  RequestVoteRes(RequestVoteResRef<'a>),
  StartElectionReq(StartElectionReqRef<'a>),
}

impl Payload<'_> {
  const APPEND_ENTRIES_REQ_META: &'static StructFieldMeta = &StructFieldMeta {
    name: "appendEntriesReq",
    offset: NumElements(0),
    meta: &AppendEntriesReqMeta::META,
  };
  const APPEND_ENTRIES_RES_META: &'static StructFieldMeta = &StructFieldMeta {
    name: "appendEntriesRes",
    offset: NumElements(0),
    meta: &AppendEntriesResMeta::META,
  };
  const REQUEST_VOTE_REQ_META: &'static StructFieldMeta = &StructFieldMeta {
    name: "requestVoteReq",
    offset: NumElements(0),
    meta: &RequestVoteReqMeta::META,
  };
  const REQUEST_VOTE_RES_META: &'static StructFieldMeta = &StructFieldMeta {
    name: "requestVoteRes",
    offset: NumElements(0),
    meta: &RequestVoteResMeta::META,
  };
  const START_ELECTION_REQ_META: &'static StructFieldMeta = &StructFieldMeta {
    name: "startElectionReq",
    offset: NumElements(0),
    meta: &StartElectionReqMeta::META,
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
