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
    get_element: |data, sink| sink.list(TestAllTypes{data: data.clone()}.data_field().to_element_list()),
  };
  const STRUCT_FIELD_META: StructFieldMeta = StructFieldMeta {
    name: "struct_field",
    offset: NumElements(2),
    meta: || &TestAllTypes::META,
  };
  const STRUCT_LIST_META: ListFieldMeta = ListFieldMeta {
    name: "struct_list",
    offset: NumElements(17),
    get_element: |data, sink| sink.list(TestAllTypes{data: data.clone()}.struct_list().to_element_list()),
  };

  const META: StructMeta = StructMeta {
    name: "TestAllTypes",
    fields: &[
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
  fn meta(&self) -> &'static StructMeta {
    &TestAllTypes::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    TestAllTypes { data: data }
  }
  fn to_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> std::fmt::Debug for TestAllTypes<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    PointerElement::Struct(&TestAllTypes::META, self.data.clone()).fmt(f)
  }
}

