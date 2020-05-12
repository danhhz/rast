// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::collections::BTreeMap;

use super::log::{Entry, Index, Term};

pub struct TestLog {
  pub entries: BTreeMap<Index, (Term, Vec<u8>)>,
  stable: Option<Index>,
}

impl TestLog {
  pub fn new() -> TestLog {
    TestLog { entries: BTreeMap::new(), stable: None }
  }

  pub fn highest_index(&self) -> Index {
    self.entries.keys().next_back().map_or(Index(0), |index| *index)
  }

  pub fn add(&mut self, entry: Entry) {
    // Invariant: All entries <= the stable one will not change.
    debug_assert!(self.stable.map_or(true, |stable| entry.index > stable));
    // Invariant: Indexes are consecutive.
    debug_assert!({
      let mut preceding = self.entries.range(..entry.index);
      preceding.next_back().map_or(true, |prev| *prev.0 + 1 == entry.index)
    });
    // Remove all entries >= the index of the new one. This is an awkward way to
    // do it but we're limited by the BTreeMap interface.
    let _ = self.entries.split_off(&entry.index);
    self.entries.insert(entry.index, (entry.term, entry.payload));
  }

  pub fn get(&self, index: Index) -> Option<&Vec<u8>> {
    self.entries.get(&index).map(|value| &value.1)
  }

  pub fn mark_stable(&mut self, index: Index) {
    // WIP: only forward stable
    self.stable = Some(index);
  }
}
