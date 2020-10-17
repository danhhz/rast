// Copyright 2020 Daniel Harrison. All Rights Reserved.

use std::convert::TryFrom;
use std::iter::{DoubleEndedIterator, FusedIterator};

use crate::serde::{Entry, Index, Term};

#[derive(Debug)]
pub struct CompressedLog {
  begin: Option<(Term, Index)>,
  end: Option<(Term, Index)>,
  term_changes: Vec<(Term, Index)>,
}

impl CompressedLog {
  pub fn new() -> CompressedLog {
    CompressedLog { begin: None, end: None, term_changes: Vec::new() }
  }

  pub fn first(&self) -> (Term, Index) {
    self.begin.unwrap_or((Term(0), Index(0)))
  }

  pub fn last(&self) -> (Term, Index) {
    self.end.unwrap_or((Term(0), Index(0)))
  }

  pub fn index_term(&self, index: Index) -> Option<Term> {
    if index == Index(0) {
      return Some(Term(0));
    }
    // TODO: binary search instead
    self.iter().find(|(_, i)| *i == index).map(|(t, _)| t)
  }

  // TODO: figure out how to accept either of Entry or EntryShared
  pub fn extend(&mut self, entries: &[Entry<'_>]) {
    if let Some(entry) = entries.first() {
      self.trim(Index(entry.index().0 - 1));
    }
    self.extend_trimmed(entries);
  }

  fn trim(&mut self, index: Index) {
    while let Some((_, tc_index)) = self.term_changes.last().copied() {
      if index >= tc_index {
        break;
      }
      self.term_changes.pop();
    }
    match self.term_changes.last().copied() {
      None => {
        self.begin = None;
        self.end = None;
      }
      Some((tc_term, _)) => {
        self.end = Some((tc_term, index));
      }
    }
  }

  fn extend_trimmed(&mut self, entries: &[Entry<'_>]) {
    for entry in entries {
      if self.begin == None {
        self.begin = Some((entry.term(), entry.index()));
      }
      // TODO: return an error instead
      debug_assert_eq!(entry.index(), Index(self.end.map_or(0, |(_, i)| i.0) + 1));
      self.end = Some((entry.term(), entry.index()));
      if self.term_changes.last().copied().map_or(true, |(tc_term, _)| entry.term() != tc_term) {
        self.term_changes.push((entry.term(), entry.index()));
      }
    }
  }

  pub fn iter<'a>(&'a self) -> CompressedLogIterator<'a> {
    let (begin_term, begin_index) = self.first();
    let (end_term, end_index) =
      self.end.map_or((begin_term, begin_index), |(end_term, end_index)| {
        (end_term, Index(end_index.0 + 1))
      });
    CompressedLogIterator {
      begin_term: begin_term,
      begin_index: begin_index,
      end_term: end_term,
      end_index: end_index,
      term_changes: &self.term_changes,
    }
  }
}

pub struct CompressedLogIterator<'a> {
  begin_term: Term,
  begin_index: Index,
  end_term: Term,
  end_index: Index,
  term_changes: &'a [(Term, Index)],
}

impl<'a> Iterator for CompressedLogIterator<'a> {
  type Item = (Term, Index);

  fn size_hint(&self) -> (usize, Option<usize>) {
    let len = usize::try_from(self.end_index.0 - self.begin_index.0).unwrap_or(usize::MAX);
    (len, Some(len))
  }

  fn next(&mut self) -> Option<Self::Item> {
    if self.begin_index == self.end_index {
      return None;
    }
    let term = self.begin_term;
    let index = self.begin_index;
    if let Some((_, tc_index)) = self.term_changes.first().copied() {
      if self.begin_index == tc_index {
        self.term_changes = &self.term_changes[1..];
      }
    }
    self.begin_index = Index(self.begin_index.0 + 1);
    if let Some((tc_term, tc_index)) = self.term_changes.first().copied() {
      if self.begin_index == tc_index {
        self.begin_term = tc_term;
      }
    }
    Some((term, index))
  }
}

impl<'a> DoubleEndedIterator for CompressedLogIterator<'a> {
  fn next_back(&mut self) -> Option<Self::Item> {
    if self.begin_index == self.end_index {
      return None;
    }
    self.end_index = Index(self.end_index.0 - 1);
    if let Some((_, tc_index)) = self.term_changes.last().copied() {
      if self.end_index < tc_index {
        self.term_changes = &self.term_changes[..self.term_changes.len() - 1];
        self.end_term = self.term_changes.last().map_or(self.begin_term, |(tc_term, _)| *tc_term);
      }
    }
    Some((self.end_term, self.end_index))
  }
}

impl<'a> ExactSizeIterator for CompressedLogIterator<'a> {}

impl<'a> FusedIterator for CompressedLogIterator<'a> {}

#[cfg(test)]
mod tests {
  #![allow(clippy::wildcard_imports)]
  use super::*;

  use crate::serde::EntryShared;

  #[test]
  fn empty() {
    let log = CompressedLog::new();
    assert_eq!((Term(0), Index(0)), log.first());
    assert_eq!((Term(0), Index(0)), log.last());
    assert_eq!(Some(Term(0)), log.index_term(Index(0)));
    assert_eq!(None, log.index_term(Index(1)));

    assert_eq!(0, log.iter().len());
    assert_eq!(0, log.iter().rev().len());
    assert_eq!(Vec::<(Term, Index)>::new(), log.iter().collect::<Vec<_>>());
    assert_eq!(Vec::<(Term, Index)>::new(), log.iter().rev().collect::<Vec<_>>());
  }

  #[test]
  fn compressed_log() {
    let mut log = CompressedLog::new();
    let history =
      vec![(Term(1), Index(1)), (Term(2), Index(2)), (Term(2), Index(3)), (Term(3), Index(4))];
    let history_rev = history.iter().rev().copied().collect::<Vec<_>>();
    let entries = history
      .iter()
      .copied()
      .map(|(term, index)| EntryShared::new(term, index, &[]))
      .collect::<Vec<_>>();

    log.extend(&entries.iter().map(|x| x.capnp_as_ref()).collect::<Vec<_>>());
    assert_eq!((Term(1), Index(1)), log.first());
    assert_eq!((Term(3), Index(4)), log.last());

    assert_eq!(Some(Term(0)), log.index_term(Index(0)));
    assert_eq!(Some(Term(1)), log.index_term(Index(1)));
    assert_eq!(Some(Term(3)), log.index_term(Index(4)));
    assert_eq!(None, log.index_term(Index(5)));

    assert_eq!(4, log.iter().len());
    assert_eq!(4, log.iter().rev().len());
    assert_eq!(history, log.iter().collect::<Vec<_>>());
    assert_eq!(history_rev, log.iter().rev().collect::<Vec<_>>());
    // Check iterators again to make sure the iterator resets.
    assert_eq!(history, log.iter().collect::<Vec<_>>());
    assert_eq!(history_rev, log.iter().rev().collect::<Vec<_>>());
  }

  #[test]
  fn extend() {
    let mut log = CompressedLog::new();
    let history =
      vec![(Term(1), Index(1)), (Term(2), Index(2)), (Term(2), Index(3)), (Term(3), Index(4))];
    let entries = history
      .iter()
      .copied()
      .map(|(term, index)| EntryShared::new(term, index, &[]))
      .collect::<Vec<_>>();

    log.extend(&entries.iter().map(|x| x.capnp_as_ref()).collect::<Vec<_>>());
    assert_eq!(history, log.iter().collect::<Vec<_>>());

    log.extend(&entries[1..3].iter().map(|x| x.capnp_as_ref()).collect::<Vec<_>>());
    assert_eq!(history[..3], log.iter().collect::<Vec<_>>()[..]);

    log.extend(&entries[2..].iter().map(|x| x.capnp_as_ref()).collect::<Vec<_>>());
    assert_eq!(history, log.iter().collect::<Vec<_>>());

    let alt_history = vec![(Term(5), Index(1)), (Term(6), Index(2)), (Term(7), Index(3))];
    let alt_entries = alt_history
      .iter()
      .copied()
      .map(|(term, index)| EntryShared::new(term, index, &[]))
      .collect::<Vec<_>>();

    log.extend(&alt_entries[1..].iter().map(|x| x.capnp_as_ref()).collect::<Vec<_>>());
    let mut expected = vec![history[0]];
    expected.extend(alt_history[1..].iter());
    assert_eq!(expected, log.iter().collect::<Vec<_>>());

    log.extend(&alt_entries.iter().map(|x| x.capnp_as_ref()).collect::<Vec<_>>());
    assert_eq!(alt_history, log.iter().collect::<Vec<_>>());

    log.trim(Index(0));
    assert_eq!((Term(0), Index(0)), log.first());
    assert_eq!((Term(0), Index(0)), log.last());
  }
}
