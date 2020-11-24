use capnp_runtime::prelude::*;

#[derive(Clone)]
pub struct SearchResult<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> SearchResult<'a> {

  const META: &'static StructMeta = &StructMeta {
    name: "SearchResult",
    data_size: NumWords(1),
    pointer_size: NumWords(2),
    fields: || &[
    ],
  };

  pub fn capnp_to_owned(&self) -> SearchResultShared {
    SearchResultShared { data: self.data.capnp_to_owned() }
  }
}

impl<'a> TypedStruct<'a> for SearchResult<'a> {
  fn meta() -> &'static StructMeta {
    &SearchResult::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    SearchResult { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for SearchResult<'a> {
  type Owned = SearchResultShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    SearchResult::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for SearchResult<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for SearchResult<'a> {
  fn partial_cmp(&self, other: &SearchResult<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for SearchResult<'a> {
  fn eq(&self, other: &SearchResult<'a>) -> bool {
    self.partial_cmp(&other) == Some(std::cmp::Ordering::Equal)
  }
}

#[derive(Clone)]
pub struct SearchResultShared {
  data: UntypedStructShared,
}

impl SearchResultShared {
  pub fn new(
  ) -> SearchResultShared {
    let mut data = UntypedStructOwned::new_with_root_struct(SearchResult::META.data_size, SearchResult::META.pointer_size);
    SearchResultShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> SearchResult<'a> {
    SearchResult { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for SearchResultShared {
  fn meta() -> &'static StructMeta {
    &SearchResult::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    SearchResultShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, SearchResult<'a>> for SearchResultShared {
  fn capnp_as_ref(&'a self) -> SearchResult<'a> {
    SearchResultShared::capnp_as_ref(self)
  }
}

#[derive(Clone)]
pub struct SearchResultList<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> SearchResultList<'a> {
  const RESULTS_META: &'static ListFieldMeta = &ListFieldMeta {
    name: "results",
    offset: NumElements(0),
    meta: &ListMeta {
      value_type: ElementType::Struct(&SearchResult::META)
    },
  };

  const META: &'static StructMeta = &StructMeta {
    name: "SearchResultList",
    data_size: NumWords(0),
    pointer_size: NumWords(1),
    fields: || &[
      FieldMeta::List(SearchResultList::RESULTS_META),
    ],
  };

  pub fn results(&self) -> Result<Slice<'a, SearchResult<'a>>, Error> { SearchResultList::RESULTS_META.get(&self.data) }

  pub fn capnp_to_owned(&self) -> SearchResultListShared {
    SearchResultListShared { data: self.data.capnp_to_owned() }
  }
}

impl<'a> TypedStruct<'a> for SearchResultList<'a> {
  fn meta() -> &'static StructMeta {
    &SearchResultList::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    SearchResultList { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for SearchResultList<'a> {
  type Owned = SearchResultListShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    SearchResultList::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for SearchResultList<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for SearchResultList<'a> {
  fn partial_cmp(&self, other: &SearchResultList<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for SearchResultList<'a> {
  fn eq(&self, other: &SearchResultList<'a>) -> bool {
    self.partial_cmp(&other) == Some(std::cmp::Ordering::Equal)
  }
}

#[derive(Clone)]
pub struct SearchResultListShared {
  data: UntypedStructShared,
}

impl SearchResultListShared {
  pub fn new(
    results: &'_ [SearchResultShared],
  ) -> SearchResultListShared {
    let mut data = UntypedStructOwned::new_with_root_struct(SearchResultList::META.data_size, SearchResultList::META.pointer_size);
    SearchResultList::RESULTS_META.set(&mut data, results);
    SearchResultListShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> SearchResultList<'a> {
    SearchResultList { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for SearchResultListShared {
  fn meta() -> &'static StructMeta {
    &SearchResultList::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    SearchResultListShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, SearchResultList<'a>> for SearchResultListShared {
  fn capnp_as_ref(&'a self) -> SearchResultList<'a> {
    SearchResultListShared::capnp_as_ref(self)
  }
}
