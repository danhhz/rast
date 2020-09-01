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
      Some(TestAllTypesShared::new(789, &vec![], None, &vec![])),
      &vec![TestAllTypesShared::new(10, &vec![], None, &vec![])],
    );
    // WIP: Why are the null data fields printing? Am I encoding them that way?
    let expected = "(u_int64_field = 123, data_field = [4, 5, 6], struct_field = (u_int64_field = 789, data_field = []), struct_list = [(u_int64_field = 10, data_field = [])])";
    assert_eq!(format!("{:?}", message.as_ref()), expected);
    Ok(())
  }

  #[test]
  fn init_rast() -> Result<(), Box<dyn error::Error>> {
    let entry = EntryShared::new(7, 8, &vec![9, 10]);
    assert_eq!(format!("{:?}", entry.as_ref()), "(term = 7, index = 8, payload = [9, 10])");
    let message = AppendEntriesReqShared::new(
      1,
      2,
      3,
      4,
      5,
      6,
      &vec![entry, EntryShared::new(11, 12, &vec![13])],
    );
    let expected = "(term = 1, leader_id = 2, prev_log_index = 3, prev_log_term = 4, leader_commit = 5, read_id = 6, entries = [(term = 7, index = 8, payload = [9, 10]), (term = 11, index = 12, payload = [13])])";
    assert_eq!(format!("{:?}", message.as_ref()), expected);
    Ok(())
  }
}
