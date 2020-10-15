// Copyright 2020 Daniel Harrison. All Rights Reserved.

mod test {
  use std::error;

  use crate::samples::rast_capnp::{
    AppendEntriesReqShared, EntryShared, MessageShared, PayloadShared,
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
    let expected = "(int32_field = -123, u_int64_field = 123, data_field = [04, 05, 06], struct_field = (int32_field = 0, u_int64_field = 789, enum_field = foo), enum_field = bar, struct_list = [(int32_field = 0, u_int64_field = 10, enum_field = foo)])";
    assert_eq!(format!("{:?}", message.capnp_as_ref()), expected);
    Ok(())
  }

  #[test]
  fn init_rast() -> Result<(), Box<dyn error::Error>> {
    let entry = EntryShared::new(9, 10, &[11, 12]);
    assert_eq!(format!("{:?}", entry.capnp_as_ref()), "(term = 9, index = 10, payload = [0b, 0c])");
    let entries = vec![entry, EntryShared::new(13, 14, &[15])];
    let req = AppendEntriesReqShared::new(3, 4, 5, 6, 7, 8, entries.as_slice());
    let message = MessageShared::new(1, 2, PayloadShared::AppendEntriesReq(req));
    let expected = "(src = 1, dest = 2, payload = (append_entries_req = (term = 3, leader_id = 4, prev_log_index = 5, prev_log_term = 6, leader_commit = 7, read_id = 8, entries = [(term = 9, index = 10, payload = [0b, 0c]), (term = 13, index = 14, payload = [0f])])))";
    assert_eq!(format!("{:?}", message.capnp_as_ref()), expected);
    Ok(())
  }
}
