use capnp_runtime::prelude::*;

pub struct SearchResultMeta;

impl SearchResultMeta {
  const URL_META: &'static TextFieldMeta = &TextFieldMeta {
    name: "url",
    offset: NumElements(0),
  };
  const SCORE_META: &'static F64FieldMeta = &F64FieldMeta {
    name: "score",
    offset: NumElements(0),
  };
  const SNIPPET_META: &'static TextFieldMeta = &TextFieldMeta {
    name: "snippet",
    offset: NumElements(1),
  };

  const META: &'static StructMeta = &StructMeta {
    name: "SearchResult",
    data_size: NumWords(1),
    pointer_size: NumWords(2),
    fields: || &[
      FieldMeta::Text(SearchResultMeta::URL_META),
      FieldMeta::F64(SearchResultMeta::SCORE_META),
      FieldMeta::Text(SearchResultMeta::SNIPPET_META),
    ],
  };
}

impl<'a> TypedStruct<'a> for SearchResultMeta {
  type Ref = SearchResultRef<'a>;
  type Shared = SearchResultShared;
  fn meta() -> &'static StructMeta {
    &SearchResultMeta::META
  }
}

pub trait SearchResult {

  fn url<'a>(&'a self) -> Result<&'a str, Error>;

  fn score<'a>(&'a self) -> f64;

  fn snippet<'a>(&'a self) -> Result<&'a str, Error>;
}

#[derive(Clone)]
pub struct SearchResultRef<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> SearchResultRef<'a> {

  pub fn url(&self) -> Result<&'a str, Error> {SearchResultMeta::URL_META.get(&self.data) }

  pub fn score(&self) -> f64 {SearchResultMeta::SCORE_META.get(&self.data) }

  pub fn snippet(&self) -> Result<&'a str, Error> {SearchResultMeta::SNIPPET_META.get(&self.data) }

  pub fn capnp_to_owned(&self) -> SearchResultShared {
    SearchResultShared { data: self.data.capnp_to_owned() }
  }
}

impl SearchResult for SearchResultRef<'_> {
  fn url<'a>(&'a self) -> Result<&'a str, Error> {
    self.url()
 }
  fn score<'a>(&'a self) -> f64 {
    self.score()
 }
  fn snippet<'a>(&'a self) -> Result<&'a str, Error> {
    self.snippet()
 }
}

impl<'a> TypedStructRef<'a> for SearchResultRef<'a> {
  fn meta() -> &'static StructMeta {
    &SearchResultMeta::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    SearchResultRef { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for SearchResultRef<'a> {
  type Owned = SearchResultShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    SearchResultRef::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for SearchResultRef<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for SearchResultRef<'a> {
  fn partial_cmp(&self, other: &SearchResultRef<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for SearchResultRef<'a> {
  fn eq(&self, other: &SearchResultRef<'a>) -> bool {
    self.partial_cmp(&other) == Some(std::cmp::Ordering::Equal)
  }
}

#[derive(Clone)]
pub struct SearchResultShared {
  data: UntypedStructShared,
}

impl SearchResultShared {
  pub fn new(
    url: &str,
    score: f64,
    snippet: &str,
  ) -> SearchResultShared {
    let mut data = UntypedStructOwned::new_with_root_struct(SearchResultMeta::META.data_size, SearchResultMeta::META.pointer_size);
    SearchResultMeta::URL_META.set(&mut data, url);
    SearchResultMeta::SCORE_META.set(&mut data, score);
    SearchResultMeta::SNIPPET_META.set(&mut data, snippet);
    SearchResultShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> SearchResultRef<'a> {
    SearchResultRef { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for SearchResultShared {
  fn meta() -> &'static StructMeta {
    &SearchResultMeta::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    SearchResultShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, SearchResultRef<'a>> for SearchResultShared {
  fn capnp_as_ref(&'a self) -> SearchResultRef<'a> {
    SearchResultShared::capnp_as_ref(self)
  }
}

pub struct SearchResultListMeta;

impl SearchResultListMeta {
  const RESULTS_META: &'static ListFieldMeta = &ListFieldMeta {
    name: "results",
    offset: NumElements(0),
    meta: &ListMeta {
      value_type: ElementType::Struct(&SearchResultMeta::META)
    },
  };

  const META: &'static StructMeta = &StructMeta {
    name: "SearchResultList",
    data_size: NumWords(0),
    pointer_size: NumWords(1),
    fields: || &[
      FieldMeta::List(SearchResultListMeta::RESULTS_META),
    ],
  };
}

impl<'a> TypedStruct<'a> for SearchResultListMeta {
  type Ref = SearchResultListRef<'a>;
  type Shared = SearchResultListShared;
  fn meta() -> &'static StructMeta {
    &SearchResultListMeta::META
  }
}

pub trait SearchResultList {

  fn results<'a>(&'a self) -> Result<Slice<'a, SearchResultRef<'a>>, Error>;
}

#[derive(Clone)]
pub struct SearchResultListRef<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> SearchResultListRef<'a> {

  pub fn results(&self) -> Result<Slice<'a, SearchResultRef<'a>>, Error> {SearchResultListMeta::RESULTS_META.get(&self.data) }

  pub fn capnp_to_owned(&self) -> SearchResultListShared {
    SearchResultListShared { data: self.data.capnp_to_owned() }
  }
}

impl SearchResultList for SearchResultListRef<'_> {
  fn results<'a>(&'a self) -> Result<Slice<'a, SearchResultRef<'a>>, Error> {
    self.results()
 }
}

impl<'a> TypedStructRef<'a> for SearchResultListRef<'a> {
  fn meta() -> &'static StructMeta {
    &SearchResultListMeta::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    SearchResultListRef { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for SearchResultListRef<'a> {
  type Owned = SearchResultListShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    SearchResultListRef::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for SearchResultListRef<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for SearchResultListRef<'a> {
  fn partial_cmp(&self, other: &SearchResultListRef<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for SearchResultListRef<'a> {
  fn eq(&self, other: &SearchResultListRef<'a>) -> bool {
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
    let mut data = UntypedStructOwned::new_with_root_struct(SearchResultListMeta::META.data_size, SearchResultListMeta::META.pointer_size);
    SearchResultListMeta::RESULTS_META.set(&mut data, results);
    SearchResultListShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> SearchResultListRef<'a> {
    SearchResultListRef { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for SearchResultListShared {
  fn meta() -> &'static StructMeta {
    &SearchResultListMeta::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    SearchResultListShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, SearchResultListRef<'a>> for SearchResultListShared {
  fn capnp_as_ref(&'a self) -> SearchResultListRef<'a> {
    SearchResultListShared::capnp_as_ref(self)
  }
}
