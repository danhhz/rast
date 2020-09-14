// Copyright 2020 Daniel Harrison. All Rights Reserved.

mod samples;

mod test {
  use std::error;

  use super::samples::rast_capnp::*;
  use super::samples::test_capnp::TestAllTypesShared;

  #[test]
  fn init_testalltypes() -> Result<(), Box<dyn error::Error>> {
    let message = TestAllTypesShared::new(
      123,
      &vec![4, 5, 6],
      Some(&TestAllTypesShared::new(789, &vec![], None, &vec![])),
      &vec![TestAllTypesShared::new(10, &vec![], None, &vec![])],
    );
    let expected = "(u_int64_field = 123, data_field = [4, 5, 6], struct_field = (u_int64_field = 789), struct_list = [(u_int64_field = 10)])";
    assert_eq!(format!("{:?}", message.as_ref()), expected);
    Ok(())
  }

  #[test]
  fn init_rast() -> Result<(), Box<dyn error::Error>> {
    let entry = EntryShared::new(9, 10, &vec![11, 12]);
    assert_eq!(format!("{:?}", entry.as_ref()), "(term = 9, index = 10, payload = [11, 12])");
    let req = AppendEntriesReqShared::new(
      3,
      4,
      5,
      6,
      7,
      8,
      &vec![entry, EntryShared::new(13, 14, &vec![15])],
    );
    let message = MessageShared::new(1, 2, PayloadShared::AppendEntriesReq(req));
    let expected = "(src = 1, dest = 2, payload = (append_entries_req = (term = 3, leader_id = 4, prev_log_index = 5, prev_log_term = 6, leader_commit = 7, read_id = 8, entries = [(term = 9, index = 10, payload = [11, 12]), (term = 13, index = 14, payload = [15])])))";
    assert_eq!(format!("{:?}", message.as_ref()), expected);
    Ok(())
  }
}
