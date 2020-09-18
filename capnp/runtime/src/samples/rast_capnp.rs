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
  const PAYLOAD_META: DataFieldMeta = DataFieldMeta {
    name: "payload",
    offset: NumElements(0),
  };

  const META: StructMeta = StructMeta {
    name: "Entry",
    data_size: NumWords(2),
    pointer_size: NumWords(1),
    fields: || &[
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(Entry::TERM_META)),
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(Entry::INDEX_META)),
      FieldMeta::Pointer(PointerFieldMeta::Data(Entry::PAYLOAD_META)),
    ],
  };

  /// The term of the entry.
  pub fn term(&self) -> u64 { Entry::TERM_META.get(&self.data) }

  /// The index of the entry.
  pub fn index(&self) -> u64 { Entry::INDEX_META.get(&self.data) }

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
    term: u64,
    index: u64,
    payload: &[u8],
  ) -> EntryShared {
    let mut data = UntypedStructOwned::new_with_root_struct(Entry::META.data_size, Entry::META.pointer_size);
    Entry::TERM_META.set(&mut data, term);
    Entry::INDEX_META.set(&mut data, index);
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
  const SRC_META: U64FieldMeta = U64FieldMeta {
    name: "src",
    offset: NumElements(0),
  };
  const DEST_META: U64FieldMeta = U64FieldMeta {
    name: "dest",
    offset: NumElements(1),
  };
  const PAYLOAD_META: UnionFieldMeta = UnionFieldMeta {
    name: "payload",
    offset: NumElements(8),
    meta: &Payload::META,
  };

  const META: StructMeta = StructMeta {
    name: "Message",
    data_size: NumWords(3),
    pointer_size: NumWords(1),
    fields: || &[
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(Message::SRC_META)),
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(Message::DEST_META)),
      FieldMeta::Union(Message::PAYLOAD_META),
    ],
  };

  /// The node sending this rpc.
  pub fn src(&self) -> u64 { Message::SRC_META.get(&self.data) }

  /// The node to receive this rpc.
  pub fn dest(&self) -> u64 { Message::DEST_META.get(&self.data) }

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
    src: u64,
    dest: u64,
    payload: PayloadShared,
  ) -> MessageShared {
    let mut data = UntypedStructOwned::new_with_root_struct(Message::META.data_size, Message::META.pointer_size);
    Message::SRC_META.set(&mut data, src);
    Message::DEST_META.set(&mut data, dest);
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
    meta: &ListMeta {
      value_type: ElementType::Pointer(PointerElementType::Struct(&Entry::META))
    },
  };

  const META: StructMeta = StructMeta {
    name: "AppendEntriesReq",
    data_size: NumWords(6),
    pointer_size: NumWords(1),
    fields: || &[
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
    term: u64,
    leader_id: u64,
    prev_log_index: u64,
    prev_log_term: u64,
    leader_commit: u64,
    read_id: u64,
    entries: &'_ [EntryShared],
  ) -> AppendEntriesReqShared {
    let mut data = UntypedStructOwned::new_with_root_struct(AppendEntriesReq::META.data_size, AppendEntriesReq::META.pointer_size);
    AppendEntriesReq::TERM_META.set(&mut data, term);
    AppendEntriesReq::LEADER_ID_META.set(&mut data, leader_id);
    AppendEntriesReq::PREV_LOG_INDEX_META.set(&mut data, prev_log_index);
    AppendEntriesReq::PREV_LOG_TERM_META.set(&mut data, prev_log_term);
    AppendEntriesReq::LEADER_COMMIT_META.set(&mut data, leader_commit);
    AppendEntriesReq::READ_ID_META.set(&mut data, read_id);
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
    data_size: NumWords(4),
    pointer_size: NumWords(0),
    fields: || &[
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
    term: u64,
    success: u64,
    index: u64,
    read_id: u64,
  ) -> AppendEntriesResShared {
    let mut data = UntypedStructOwned::new_with_root_struct(AppendEntriesRes::META.data_size, AppendEntriesRes::META.pointer_size);
    AppendEntriesRes::TERM_META.set(&mut data, term);
    AppendEntriesRes::SUCCESS_META.set(&mut data, success);
    AppendEntriesRes::INDEX_META.set(&mut data, index);
    AppendEntriesRes::READ_ID_META.set(&mut data, read_id);
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
    data_size: NumWords(4),
    pointer_size: NumWords(0),
    fields: || &[
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
    term: u64,
    candidate_id: u64,
    last_log_index: u64,
    last_log_term: u64,
  ) -> RequestVoteReqShared {
    let mut data = UntypedStructOwned::new_with_root_struct(RequestVoteReq::META.data_size, RequestVoteReq::META.pointer_size);
    RequestVoteReq::TERM_META.set(&mut data, term);
    RequestVoteReq::CANDIDATE_ID_META.set(&mut data, candidate_id);
    RequestVoteReq::LAST_LOG_INDEX_META.set(&mut data, last_log_index);
    RequestVoteReq::LAST_LOG_TERM_META.set(&mut data, last_log_term);
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
    data_size: NumWords(2),
    pointer_size: NumWords(0),
    fields: || &[
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(RequestVoteRes::TERM_META)),
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(RequestVoteRes::VOTE_GRANTED_META)),
    ],
  };

  pub fn term(&self) -> u64 { RequestVoteRes::TERM_META.get(&self.data) }

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
    term: u64,
    vote_granted: u64,
  ) -> RequestVoteResShared {
    let mut data = UntypedStructOwned::new_with_root_struct(RequestVoteRes::META.data_size, RequestVoteRes::META.pointer_size);
    RequestVoteRes::TERM_META.set(&mut data, term);
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
  const TERM_META: U64FieldMeta = U64FieldMeta {
    name: "term",
    offset: NumElements(0),
  };

  const META: StructMeta = StructMeta {
    name: "StartElectionReq",
    data_size: NumWords(1),
    pointer_size: NumWords(0),
    fields: || &[
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(StartElectionReq::TERM_META)),
    ],
  };

  pub fn term(&self) -> u64 { StartElectionReq::TERM_META.get(&self.data) }

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
    term: u64,
  ) -> StartElectionReqShared {
    let mut data = UntypedStructOwned::new_with_root_struct(StartElectionReq::META.data_size, StartElectionReq::META.pointer_size);
    StartElectionReq::TERM_META.set(&mut data, term);
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
  const APPEND_ENTRIES_REQ_META: StructFieldMeta = StructFieldMeta {
    name: "append_entries_req",
    offset: NumElements(0),
    meta: &AppendEntriesReq::META,
  };
  const APPEND_ENTRIES_RES_META: StructFieldMeta = StructFieldMeta {
    name: "append_entries_res",
    offset: NumElements(0),
    meta: &AppendEntriesRes::META,
  };
  const REQUEST_VOTE_REQ_META: StructFieldMeta = StructFieldMeta {
    name: "request_vote_req",
    offset: NumElements(0),
    meta: &RequestVoteReq::META,
  };
  const REQUEST_VOTE_RES_META: StructFieldMeta = StructFieldMeta {
    name: "request_vote_res",
    offset: NumElements(0),
    meta: &RequestVoteRes::META,
  };
  const START_ELECTION_REQ_META: StructFieldMeta = StructFieldMeta {
    name: "start_election_req",
    offset: NumElements(0),
    meta: &StartElectionReq::META,
  };
  const META: UnionMeta = UnionMeta {
    name: "Payload",
    variants: &[
      UnionVariantMeta{
        discriminant: Discriminant(0),
        field_meta: FieldMeta::Pointer(PointerFieldMeta::Struct(Payload::APPEND_ENTRIES_REQ_META)),
      },
      UnionVariantMeta{
        discriminant: Discriminant(1),
        field_meta: FieldMeta::Pointer(PointerFieldMeta::Struct(Payload::APPEND_ENTRIES_RES_META)),
      },
      UnionVariantMeta{
        discriminant: Discriminant(2),
        field_meta: FieldMeta::Pointer(PointerFieldMeta::Struct(Payload::REQUEST_VOTE_REQ_META)),
      },
      UnionVariantMeta{
        discriminant: Discriminant(3),
        field_meta: FieldMeta::Pointer(PointerFieldMeta::Struct(Payload::REQUEST_VOTE_RES_META)),
      },
      UnionVariantMeta{
        discriminant: Discriminant(4),
        field_meta: FieldMeta::Pointer(PointerFieldMeta::Struct(Payload::START_ELECTION_REQ_META)),
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
        Payload::APPEND_ENTRIES_REQ_META.set(data, Some(x.clone()));
      }
      PayloadShared::AppendEntriesRes(x) => {
        data.set_discriminant(discriminant_offset, Discriminant(1));
        Payload::APPEND_ENTRIES_RES_META.set(data, Some(x.clone()));
      }
      PayloadShared::RequestVoteReq(x) => {
        data.set_discriminant(discriminant_offset, Discriminant(2));
        Payload::REQUEST_VOTE_REQ_META.set(data, Some(x.clone()));
      }
      PayloadShared::RequestVoteRes(x) => {
        data.set_discriminant(discriminant_offset, Discriminant(3));
        Payload::REQUEST_VOTE_RES_META.set(data, Some(x.clone()));
      }
      PayloadShared::StartElectionReq(x) => {
        data.set_discriminant(discriminant_offset, Discriminant(4));
        Payload::START_ELECTION_REQ_META.set(data, Some(x.clone()));
      }
    }
  }
}

impl<'a> CapnpAsRef<'a, Payload<'a>> for PayloadShared {
  fn capnp_as_ref(&'a self) -> Payload<'a> {
    PayloadShared::capnp_as_ref(self)
  }
}
