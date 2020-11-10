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
      true,
      -12345678,
      234,
      45678,
      3456789012,
      12345678901234567890,
      1234.5,
      -1.23e47,
      "foo",
      "bar".as_bytes(),
      Some(TestAllTypesShared::new(
        true,
        -78901234,
        90,
        1234,
        56789012,
        345678901234567890,
        -1.25e-10,
        345.0,
        "baz",
        "qux".as_bytes(),
        None,
        TestEnum::Baz,
        &[],
      )),
      TestEnum::Bar,
      vec![TestAllTypesShared::new(
        false,
        0,
        0,
        0,
        0,
        0,
        0.0,
        0.0,
        "structlist 1",
        &[],
        None,
        TestEnum::Foo,
        &[],
      )]
      .as_slice(),
    );
    let expected = "(boolField = false, int32Field = -12345678, uInt8Field = 234, uInt16Field = 45678, uInt32Field = 3456789012, uInt64Field = 12345678901234567890, float32Field = 1234.5, float64Field = -123000000000000000000000000000000000000000000000.0, textField = \"foo\", dataField = \"bar\", structField = (boolField = false, int32Field = -78901234, uInt8Field = 90, uInt16Field = 1234, uInt32Field = 56789012, uInt64Field = 345678901234567890, float32Field = -0.000000000125, float64Field = 345.0, textField = \"baz\", dataField = \"qux\", enumField = baz), enumField = bar, structList = [(boolField = false, int32Field = 0, uInt8Field = 0, uInt16Field = 0, uInt32Field = 0, uInt64Field = 0, float32Field = 0.0, float64Field = 0.0, textField = \"structlist 1\", enumField = foo)])";
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
