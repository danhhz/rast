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

pub struct TestAllTypesMeta;

impl TestAllTypesMeta {
  const INT32_FIELD_META: &'static I32FieldMeta = &I32FieldMeta {
    name: "int32_field",
    offset: NumElements(1),
  };
  const U_INT64_FIELD_META: &'static U64FieldMeta = &U64FieldMeta {
    name: "u_int64_field",
    offset: NumElements(3),
  };
  const DATA_FIELD_META: &'static DataFieldMeta = &DataFieldMeta {
    name: "data_field",
    offset: NumElements(1),
  };
  const STRUCT_FIELD_META: &'static StructFieldMeta = &StructFieldMeta {
    name: "struct_field",
    offset: NumElements(2),
    meta: &TestAllTypesMeta::META,
  };
  const ENUM_FIELD_META: &'static EnumFieldMeta = &EnumFieldMeta {
    name: "enum_field",
    offset: NumElements(18),
    meta: &TestEnum::META,
  };
  const STRUCT_LIST_META: &'static ListFieldMeta = &ListFieldMeta {
    name: "struct_list",
    offset: NumElements(17),
    meta: &ListMeta {
      value_type: ElementType::Struct(&TestAllTypesMeta::META)
    },
  };

  const META: &'static StructMeta = &StructMeta {
    name: "TestAllTypes",
    data_size: NumWords(6),
    pointer_size: NumWords(20),
    fields: || &[
      FieldMeta::I32(TestAllTypesMeta::INT32_FIELD_META),
      FieldMeta::U64(TestAllTypesMeta::U_INT64_FIELD_META),
      FieldMeta::Data(TestAllTypesMeta::DATA_FIELD_META),
      FieldMeta::Struct(TestAllTypesMeta::STRUCT_FIELD_META),
      FieldMeta::Enum(TestAllTypesMeta::ENUM_FIELD_META),
      FieldMeta::List(TestAllTypesMeta::STRUCT_LIST_META),
    ],
  };
}

impl<'a> TypedStruct<'a> for TestAllTypesMeta {
  type Ref = TestAllTypesRef<'a>;
  type Shared = TestAllTypesShared;
  fn meta() -> &'static StructMeta {
    &TestAllTypesMeta::META
  }
}

pub trait TestAllTypes {

  fn int32_field<'a>(&'a self) -> i32;

  fn u_int64_field<'a>(&'a self) -> u64;

  fn data_field<'a>(&'a self) -> Result<&'a [u8], Error>;

  fn struct_field<'a>(&'a self) -> Result<TestAllTypesRef<'a>, Error>;

  fn enum_field<'a>(&'a self) -> Result<TestEnum, UnknownDiscriminant>;

  fn struct_list<'a>(&'a self) -> Result<Slice<'a, TestAllTypesRef<'a>>, Error>;
}

#[derive(Clone)]
pub struct TestAllTypesRef<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> TestAllTypesRef<'a> {

  pub fn int32_field(&self) -> i32 {TestAllTypesMeta::INT32_FIELD_META.get(&self.data) }

  pub fn u_int64_field(&self) -> u64 {TestAllTypesMeta::U_INT64_FIELD_META.get(&self.data) }

  pub fn data_field(&self) -> Result<&'a [u8], Error> {TestAllTypesMeta::DATA_FIELD_META.get(&self.data) }

  pub fn struct_field(&self) -> Result<TestAllTypesRef<'a>, Error> {TestAllTypesMeta::STRUCT_FIELD_META.get(&self.data) }

  pub fn enum_field(&self) -> Result<TestEnum, UnknownDiscriminant> {TestAllTypesMeta::ENUM_FIELD_META.get(&self.data) }

  pub fn struct_list(&self) -> Result<Slice<'a, TestAllTypesRef<'a>>, Error> {TestAllTypesMeta::STRUCT_LIST_META.get(&self.data) }

  pub fn capnp_to_owned(&self) -> TestAllTypesShared {
    TestAllTypesShared { data: self.data.capnp_to_owned() }
  }
}

impl TestAllTypes for TestAllTypesRef<'_> {
  fn int32_field<'a>(&'a self) -> i32 {
    self.int32_field()
 }
  fn u_int64_field<'a>(&'a self) -> u64 {
    self.u_int64_field()
 }
  fn data_field<'a>(&'a self) -> Result<&'a [u8], Error> {
    self.data_field()
 }
  fn struct_field<'a>(&'a self) -> Result<TestAllTypesRef<'a>, Error> {
    self.struct_field()
 }
  fn enum_field<'a>(&'a self) -> Result<TestEnum, UnknownDiscriminant> {
    self.enum_field()
 }
  fn struct_list<'a>(&'a self) -> Result<Slice<'a, TestAllTypesRef<'a>>, Error> {
    self.struct_list()
 }
}

impl<'a> TypedStructRef<'a> for TestAllTypesRef<'a> {
  fn meta() -> &'static StructMeta {
    &TestAllTypesMeta::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    TestAllTypesRef { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for TestAllTypesRef<'a> {
  type Owned = TestAllTypesShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    TestAllTypesRef::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for TestAllTypesRef<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for TestAllTypesRef<'a> {
  fn partial_cmp(&self, other: &TestAllTypesRef<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for TestAllTypesRef<'a> {
  fn eq(&self, other: &TestAllTypesRef<'a>) -> bool {
    self.partial_cmp(&other) == Some(std::cmp::Ordering::Equal)
  }
}

#[derive(Clone)]
pub struct TestAllTypesShared {
  data: UntypedStructShared,
}

impl TestAllTypesShared {
  pub fn new(
    int32_field: i32,
    u_int64_field: u64,
    data_field: &[u8],
    struct_field: Option<TestAllTypesShared>,
    enum_field: TestEnum,
    struct_list: &'_ [TestAllTypesShared],
  ) -> TestAllTypesShared {
    let mut data = UntypedStructOwned::new_with_root_struct(TestAllTypesMeta::META.data_size, TestAllTypesMeta::META.pointer_size);
    TestAllTypesMeta::INT32_FIELD_META.set(&mut data, int32_field);
    TestAllTypesMeta::U_INT64_FIELD_META.set(&mut data, u_int64_field);
    TestAllTypesMeta::DATA_FIELD_META.set(&mut data, data_field);
    TestAllTypesMeta::STRUCT_FIELD_META.set(&mut data, struct_field);
    TestAllTypesMeta::ENUM_FIELD_META.set(&mut data, enum_field);
    TestAllTypesMeta::STRUCT_LIST_META.set(&mut data, struct_list);
    TestAllTypesShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> TestAllTypesRef<'a> {
    TestAllTypesRef { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for TestAllTypesShared {
  fn meta() -> &'static StructMeta {
    &TestAllTypesMeta::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    TestAllTypesShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, TestAllTypesRef<'a>> for TestAllTypesShared {
  fn capnp_as_ref(&'a self) -> TestAllTypesRef<'a> {
    TestAllTypesShared::capnp_as_ref(self)
  }
}
