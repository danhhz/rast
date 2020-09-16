// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::collections::BTreeMap;

use crate::prelude::*;

/// An unpersisted Raft log implementation suitable for unit tests and
/// benchmarks.
pub struct MemLog {
  /// The Raft log entries.
  pub entries: BTreeMap<Index, (Term, Vec<u8>)>,
  /// A guarantee that any entry with a lesser term will never change.
  pub stable: Option<Index>,
}

impl MemLog {
  /// Constructs a new, empty `MemLog`.
  pub fn new() -> MemLog {
    MemLog { entries: BTreeMap::new(), stable: None }
  }

  /// Returns the largest index added to this log.
  ///
  /// This index is not monotonic, but it will never regress lower than
  /// `stable`.
  pub fn highest_index(&self) -> Index {
    self.entries.keys().next_back().map_or(Index(0), |index| *index)
  }

  /// Appends a new entry to the log, truncating existing entries that conflict,
  /// if necessary.
  pub fn add<'a>(&mut self, entry: Entry<'a>) {
    // Invariant: All entries <= the stable one will not change.
    debug_assert!(self.stable.map_or(true, |stable| Index(entry.index()) > stable));
    // Invariant: Indexes are consecutive.
    debug_assert!({
      let mut preceding = self.entries.range(..Index(entry.index()));
      preceding.next_back().map_or(true, |prev| *prev.0 + 1 == Index(entry.index()))
    });
    // Remove all entries >= the index of the new one. This is an awkward way to
    // do it but we're limited by the BTreeMap interface.
    let _ = self.entries.split_off(&Index(entry.index()));
    self
      .entries
      .insert(Index(entry.index()), (Term(entry.term()), entry.payload().expect("WIP").to_vec()));
  }

  /// Returns the payload of the entry at the given index or None if that index
  /// doesn't exist.
  pub fn get(&self, index: Index) -> Option<&Vec<u8>> {
    self.entries.get(&index).map(|value| &value.1)
  }

  /// Marks the given index as stable, promising that it will never be truncated
  /// by a later addition.
  pub fn mark_stable(&mut self, index: Index) {
    // TODO: only forward stable
    self.stable = Some(index);
  }
}
