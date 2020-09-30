// Copyright 2020 Daniel Harrison. All Rights Reserved.

mod test {
  use std::error;

  use crate::samples::rast_capnp::{
    AppendEntriesReqShared, EntryShared, Index, MessageShared, NodeID, PayloadShared, ReadID, Term,
  };
  use crate::samples::test_capnp::{TestAllTypesShared, TestEnum};

  #[test]
  fn init_testalltypes() -> Result<(), Box<dyn error::Error>> {
    let message = TestAllTypesShared::new(
      -123,
      123,
      &[4, 5, 6],
      Some(TestAllTypesShared::new(0, 789, &[], None, TestEnum::Foo, &[])),
      TestEnum::Bar,
      vec![TestAllTypesShared::new(0, 10, &[], None, TestEnum::Foo, &[])].as_slice(),
    );
    let expected = "(int32Field = -123, uInt64Field = 123, dataField = [04, 05, 06], structField = (int32Field = 0, uInt64Field = 789, enumField = foo), enumField = bar, structList = [(int32Field = 0, uInt64Field = 10, enumField = foo)])";
    assert_eq!(format!("{:?}", message.capnp_as_ref()), expected);
    Ok(())
  }

  #[test]
  fn init_rast() -> Result<(), Box<dyn error::Error>> {
    let entry = EntryShared::new(Term(9), Index(10), &[11, 12]);
    assert_eq!(format!("{:?}", entry.capnp_as_ref()), "(term = 9, index = 10, payload = [0b, 0c])");
    let entries = vec![entry, EntryShared::new(Term(13), Index(14), &[15])];
    let req = AppendEntriesReqShared::new(
      Term(3),
      NodeID(4),
      Index(5),
      Term(6),
      Index(7),
      ReadID(8),
      entries.as_slice(),
    );
    let message = MessageShared::new(NodeID(1), NodeID(2), PayloadShared::AppendEntriesReq(req));
    let expected = "(src = 1, dest = 2, payload = (appendEntriesReq = (term = 3, leaderId = 4, prevLogIndex = 5, prevLogTerm = 6, leaderCommit = 7, readId = 8, entries = [(term = 9, index = 10, payload = [0b, 0c]), (term = 13, index = 14, payload = [0f])])))";
    assert_eq!(format!("{:?}", message.capnp_as_ref()), expected);
    Ok(())
  }
}
