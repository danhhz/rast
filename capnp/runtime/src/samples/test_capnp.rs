use capnp_runtime::prelude::*;

#[derive(Clone, Copy)]
pub enum TestEnum {
  Foo = 0,
  Bar = 1,
  Baz = 2,
  Qux = 3,
  Quux = 4,
  Corge = 5,
  Grault = 6,
  Garply = 7,
}

impl TestEnum {
  const META: &'static EnumMeta = &EnumMeta {
    name: "TestEnum",
    enumerants: &[
      EnumerantMeta{
        name: "foo",
        discriminant: Discriminant(0),
      },
      EnumerantMeta{
        name: "bar",
        discriminant: Discriminant(1),
      },
      EnumerantMeta{
        name: "baz",
        discriminant: Discriminant(2),
      },
      EnumerantMeta{
        name: "qux",
        discriminant: Discriminant(3),
      },
      EnumerantMeta{
        name: "quux",
        discriminant: Discriminant(4),
      },
      EnumerantMeta{
        name: "corge",
        discriminant: Discriminant(5),
      },
      EnumerantMeta{
        name: "grault",
        discriminant: Discriminant(6),
      },
      EnumerantMeta{
        name: "garply",
        discriminant: Discriminant(7),
      },
    ],
  };
}

impl TypedEnum for TestEnum {
  fn meta() -> &'static EnumMeta {
    &TestEnum::META
  }
  fn from_discriminant(discriminant: Discriminant) -> Result<Self, UnknownDiscriminant> {
   match discriminant {
      Discriminant(0) => Ok(TestEnum::Foo),
      Discriminant(1) => Ok(TestEnum::Bar),
      Discriminant(2) => Ok(TestEnum::Baz),
      Discriminant(3) => Ok(TestEnum::Qux),
      Discriminant(4) => Ok(TestEnum::Quux),
      Discriminant(5) => Ok(TestEnum::Corge),
      Discriminant(6) => Ok(TestEnum::Grault),
      Discriminant(7) => Ok(TestEnum::Garply),
      d => Err(UnknownDiscriminant(d, TestEnum::META.name)),
    }
  }
  fn to_discriminant(&self) -> Discriminant {
    Discriminant(*self as u16)
  }
}

#[derive(Clone)]
pub struct TestAllTypes<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> TestAllTypes<'a> {
  const BOOL_FIELD_META: &'static BoolFieldMeta = &BoolFieldMeta {
    name: "boolField",
    offset: NumElements(0),
  };
  const INT32_FIELD_META: &'static I32FieldMeta = &I32FieldMeta {
    name: "int32Field",
    offset: NumElements(1),
  };
  const U_INT8_FIELD_META: &'static U8FieldMeta = &U8FieldMeta {
    name: "uInt8Field",
    offset: NumElements(16),
  };
  const U_INT16_FIELD_META: &'static U16FieldMeta = &U16FieldMeta {
    name: "uInt16Field",
    offset: NumElements(9),
  };
  const U_INT32_FIELD_META: &'static U32FieldMeta = &U32FieldMeta {
    name: "uInt32Field",
    offset: NumElements(5),
  };
  const U_INT64_FIELD_META: &'static U64FieldMeta = &U64FieldMeta {
    name: "uInt64Field",
    offset: NumElements(3),
  };
  const FLOAT32_FIELD_META: &'static F32FieldMeta = &F32FieldMeta {
    name: "float32Field",
    offset: NumElements(8),
  };
  const FLOAT64_FIELD_META: &'static F64FieldMeta = &F64FieldMeta {
    name: "float64Field",
    offset: NumElements(5),
  };
  const TEXT_FIELD_META: &'static TextFieldMeta = &TextFieldMeta {
    name: "textField",
    offset: NumElements(0),
  };
  const DATA_FIELD_META: &'static DataFieldMeta = &DataFieldMeta {
    name: "dataField",
    offset: NumElements(1),
  };
  const STRUCT_FIELD_META: &'static StructFieldMeta = &StructFieldMeta {
    name: "structField",
    offset: NumElements(2),
    meta: &TestAllTypes::META,
  };
  const ENUM_FIELD_META: &'static EnumFieldMeta = &EnumFieldMeta {
    name: "enumField",
    offset: NumElements(18),
    meta: &TestEnum::META,
  };
  const STRUCT_LIST_META: &'static ListFieldMeta = &ListFieldMeta {
    name: "structList",
    offset: NumElements(17),
    meta: &ListMeta {
      value_type: ElementType::Struct(&TestAllTypes::META)
    },
  };

  const META: &'static StructMeta = &StructMeta {
    name: "TestAllTypes",
    data_size: NumWords(6),
    pointer_size: NumWords(20),
    fields: || &[
      FieldMeta::Bool(TestAllTypes::BOOL_FIELD_META),
      FieldMeta::I32(TestAllTypes::INT32_FIELD_META),
      FieldMeta::U8(TestAllTypes::U_INT8_FIELD_META),
      FieldMeta::U16(TestAllTypes::U_INT16_FIELD_META),
      FieldMeta::U32(TestAllTypes::U_INT32_FIELD_META),
      FieldMeta::U64(TestAllTypes::U_INT64_FIELD_META),
      FieldMeta::F32(TestAllTypes::FLOAT32_FIELD_META),
      FieldMeta::F64(TestAllTypes::FLOAT64_FIELD_META),
      FieldMeta::Text(TestAllTypes::TEXT_FIELD_META),
      FieldMeta::Data(TestAllTypes::DATA_FIELD_META),
      FieldMeta::Struct(TestAllTypes::STRUCT_FIELD_META),
      FieldMeta::Enum(TestAllTypes::ENUM_FIELD_META),
      FieldMeta::List(TestAllTypes::STRUCT_LIST_META),
    ],
  };

  pub fn bool_field(&self) -> bool { TestAllTypes::BOOL_FIELD_META.get(&self.data) }

  pub fn int32_field(&self) -> i32 { TestAllTypes::INT32_FIELD_META.get(&self.data) }

  pub fn u_int8_field(&self) -> u8 { TestAllTypes::U_INT8_FIELD_META.get(&self.data) }

  pub fn u_int16_field(&self) -> u16 { TestAllTypes::U_INT16_FIELD_META.get(&self.data) }

  pub fn u_int32_field(&self) -> u32 { TestAllTypes::U_INT32_FIELD_META.get(&self.data) }

  pub fn u_int64_field(&self) -> u64 { TestAllTypes::U_INT64_FIELD_META.get(&self.data) }

  pub fn float32_field(&self) -> f32 { TestAllTypes::FLOAT32_FIELD_META.get(&self.data) }

  pub fn float64_field(&self) -> f64 { TestAllTypes::FLOAT64_FIELD_META.get(&self.data) }

  pub fn text_field(&self) -> Result<&'a str, Error> { TestAllTypes::TEXT_FIELD_META.get(&self.data) }

  pub fn data_field(&self) -> Result<&'a [u8], Error> { TestAllTypes::DATA_FIELD_META.get(&self.data) }

  pub fn struct_field(&self) -> Result<TestAllTypes<'a>, Error> { TestAllTypes::STRUCT_FIELD_META.get(&self.data) }

  pub fn enum_field(&self) -> Result<TestEnum, UnknownDiscriminant> { TestAllTypes::ENUM_FIELD_META.get(&self.data) }

  pub fn struct_list(&self) -> Result<Slice<'a, TestAllTypes<'a>>, Error> { TestAllTypes::STRUCT_LIST_META.get(&self.data) }

  pub fn capnp_to_owned(&self) -> TestAllTypesShared {
    TestAllTypesShared { data: self.data.capnp_to_owned() }
  }
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

impl<'a> CapnpToOwned<'a> for TestAllTypes<'a> {
  type Owned = TestAllTypesShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    TestAllTypes::capnp_to_owned(self)
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

#[derive(Clone)]
pub struct TestAllTypesShared {
  data: UntypedStructShared,
}

impl TestAllTypesShared {
  pub fn new(
    bool_field: bool,
    int32_field: i32,
    u_int8_field: u8,
    u_int16_field: u16,
    u_int32_field: u32,
    u_int64_field: u64,
    float32_field: f32,
    float64_field: f64,
    text_field: &str,
    data_field: &[u8],
    struct_field: Option<TestAllTypesShared>,
    enum_field: TestEnum,
    struct_list: &'_ [TestAllTypesShared],
  ) -> TestAllTypesShared {
    let mut data = UntypedStructOwned::new_with_root_struct(TestAllTypes::META.data_size, TestAllTypes::META.pointer_size);
    TestAllTypes::BOOL_FIELD_META.set(&mut data, bool_field);
    TestAllTypes::INT32_FIELD_META.set(&mut data, int32_field);
    TestAllTypes::U_INT8_FIELD_META.set(&mut data, u_int8_field);
    TestAllTypes::U_INT16_FIELD_META.set(&mut data, u_int16_field);
    TestAllTypes::U_INT32_FIELD_META.set(&mut data, u_int32_field);
    TestAllTypes::U_INT64_FIELD_META.set(&mut data, u_int64_field);
    TestAllTypes::FLOAT32_FIELD_META.set(&mut data, float32_field);
    TestAllTypes::FLOAT64_FIELD_META.set(&mut data, float64_field);
    TestAllTypes::TEXT_FIELD_META.set(&mut data, text_field);
    TestAllTypes::DATA_FIELD_META.set(&mut data, data_field);
    TestAllTypes::STRUCT_FIELD_META.set(&mut data, struct_field);
    TestAllTypes::ENUM_FIELD_META.set(&mut data, enum_field);
    TestAllTypes::STRUCT_LIST_META.set(&mut data, struct_list);
    TestAllTypesShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> TestAllTypes<'a> {
    TestAllTypes { data: self.data.capnp_as_ref() }
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

impl<'a> CapnpAsRef<'a, TestAllTypes<'a>> for TestAllTypesShared {
  fn capnp_as_ref(&'a self) -> TestAllTypes<'a> {
    TestAllTypesShared::capnp_as_ref(self)
  }
}
