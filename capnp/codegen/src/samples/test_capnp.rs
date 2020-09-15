use capnp_runtime::prelude::*;

#[derive(Clone)]
pub struct TestAllTypes<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> TestAllTypes<'a> {
  const U_INT64_FIELD_META: U64FieldMeta = U64FieldMeta {
    name: "u_int64_field",
    offset: NumElements(3),
  };
  const DATA_FIELD_META: ListFieldMeta = ListFieldMeta {
    name: "data_field",
    offset: NumElements(1),
    meta: &ListMeta {
      value_type: ElementType::Primitive(PrimitiveElementType::U8)
    },
  };
  const STRUCT_FIELD_META: StructFieldMeta = StructFieldMeta {
    name: "struct_field",
    offset: NumElements(2),
    meta: &TestAllTypes::META,
  };
  const STRUCT_LIST_META: ListFieldMeta = ListFieldMeta {
    name: "struct_list",
    offset: NumElements(17),
    meta: &ListMeta {
      value_type: ElementType::Pointer(PointerElementType::Struct(StructElementType {meta: &TestAllTypes::META}))
    },
  };

  const META: StructMeta = StructMeta {
    name: "TestAllTypes",
    data_size: NumWords(6),
    pointer_size: NumWords(20),
    fields: || &[
      FieldMeta::Primitive(PrimitiveFieldMeta::U64(TestAllTypes::U_INT64_FIELD_META)),
      FieldMeta::Pointer(PointerFieldMeta::List(TestAllTypes::DATA_FIELD_META)),
      FieldMeta::Pointer(PointerFieldMeta::Struct(TestAllTypes::STRUCT_FIELD_META)),
      FieldMeta::Pointer(PointerFieldMeta::List(TestAllTypes::STRUCT_LIST_META)),
    ],
  };

  pub fn u_int64_field(&self) -> u64 { TestAllTypes::U_INT64_FIELD_META.get(&self.data) }
  pub fn data_field(&self) -> Result<Vec<u8>, Error> { TestAllTypes::DATA_FIELD_META.get(&self.data) }
  pub fn struct_field(&self) -> Result<TestAllTypes<'a>, Error> { TestAllTypes::STRUCT_FIELD_META.get(&self.data) }
  pub fn struct_list(&self) -> Result<Vec<TestAllTypes<'a>>, Error> { TestAllTypes::STRUCT_LIST_META.get(&self.data) }
}

impl<'a> TypedStruct<'a> for TestAllTypes<'a> {
  fn meta() -> &'static StructMeta {
    &TestAllTypes::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    TestAllTypes { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> std::fmt::Debug for TestAllTypes<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for TestAllTypes<'a> {
  fn partial_cmp(&self, other: &TestAllTypes<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for TestAllTypes<'a> {
  fn eq(&self, other: &TestAllTypes<'a>) -> bool {
    self.partial_cmp(&other) == Some(std::cmp::Ordering::Equal)
  }
}

pub struct TestAllTypesShared {
  data: UntypedStructShared,
}

impl TestAllTypesShared {
  pub fn new(
    u_int64_field: u64,
    data_field: &'_ [u8],
    struct_field: Option<&TestAllTypesShared>,
    struct_list: &'_ [TestAllTypesShared],
  ) -> TestAllTypesShared {
    let mut data = UntypedStructOwned::new_with_root_struct(TestAllTypes::META.data_size, TestAllTypes::META.pointer_size);
    TestAllTypes::U_INT64_FIELD_META.set(&mut data, u_int64_field);
    TestAllTypes::DATA_FIELD_META.set(&mut data, data_field);
    TestAllTypes::STRUCT_FIELD_META.set(&mut data, struct_field);
    TestAllTypes::STRUCT_LIST_META.set(&mut data, struct_list);
    TestAllTypesShared { data: data.into_shared() }
  }

  pub fn as_ref<'a>(&'a self) -> TestAllTypes<'a> {
    TestAllTypes { data: self.data.as_ref() }
  }
}

impl TypedStructShared for TestAllTypesShared {
  fn meta() -> &'static StructMeta {
    &TestAllTypes::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    TestAllTypesShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

