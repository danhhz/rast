use capnp_runtime::prelude::*;

#[derive(Clone, Copy)]
pub enum Operation {
  Add = 0,
  Subtract = 1,
  Multiply = 2,
  Divide = 3,
  Modulus = 4,
}

impl Operation {
  const META: &'static EnumMeta = &EnumMeta {
    name: "Operation",
    enumerants: &[
      EnumerantMeta{
        name: "add",
        discriminant: Discriminant(0),
      },
      EnumerantMeta{
        name: "subtract",
        discriminant: Discriminant(1),
      },
      EnumerantMeta{
        name: "multiply",
        discriminant: Discriminant(2),
      },
      EnumerantMeta{
        name: "divide",
        discriminant: Discriminant(3),
      },
      EnumerantMeta{
        name: "modulus",
        discriminant: Discriminant(4),
      },
    ],
  };
}

impl TypedEnum for Operation {
  fn meta() -> &'static EnumMeta {
    &Operation::META
  }
  fn from_discriminant(discriminant: Discriminant) -> Result<Self, UnknownDiscriminant> {
   match discriminant {
      Discriminant(0) => Ok(Operation::Add),
      Discriminant(1) => Ok(Operation::Subtract),
      Discriminant(2) => Ok(Operation::Multiply),
      Discriminant(3) => Ok(Operation::Divide),
      Discriminant(4) => Ok(Operation::Modulus),
      d => Err(UnknownDiscriminant(d, Operation::META.name)),
    }
  }
  fn to_discriminant(&self) -> Discriminant {
    Discriminant(*self as u16)
  }
}

pub struct ExpressionMeta;

impl ExpressionMeta {
  const OP_META: &'static EnumFieldMeta = &EnumFieldMeta {
    name: "op",
    offset: NumElements(0),
    meta: &Operation::META,
  };
  const LEFT_META: &'static UnionFieldMeta = &UnionFieldMeta {
    name: "left",
    offset: NumElements(1),
    meta: &Left::META,
  };
  const RIGHT_META: &'static UnionFieldMeta = &UnionFieldMeta {
    name: "right",
    offset: NumElements(6),
    meta: &Right::META,
  };

  const META: &'static StructMeta = &StructMeta {
    name: "Expression",
    data_size: NumWords(2),
    pointer_size: NumWords(2),
    fields: || &[
      FieldMeta::Enum(ExpressionMeta::OP_META),
      FieldMeta::Union(ExpressionMeta::LEFT_META),
      FieldMeta::Union(ExpressionMeta::RIGHT_META),
    ],
  };
}

impl<'a> TypedStruct<'a> for ExpressionMeta {
  type Ref = ExpressionRef<'a>;
  type Shared = ExpressionShared;
  fn meta() -> &'static StructMeta {
    &ExpressionMeta::META
  }
}

pub trait Expression {

  fn op<'a>(&'a self) -> Result<Operation, UnknownDiscriminant>;

  fn left<'a>(&'a self) -> Result<Result<Left<'a>, UnknownDiscriminant>,Error>;

  fn right<'a>(&'a self) -> Result<Result<Right<'a>, UnknownDiscriminant>,Error>;
}

#[derive(Clone)]
pub struct ExpressionRef<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> ExpressionRef<'a> {

  pub fn op(&self) -> Result<Operation, UnknownDiscriminant> {ExpressionMeta::OP_META.get(&self.data) }

  pub fn left(&self) -> Result<Result<Left<'a>, UnknownDiscriminant>,Error> {ExpressionMeta::LEFT_META.get(&self.data) }

  pub fn right(&self) -> Result<Result<Right<'a>, UnknownDiscriminant>,Error> {ExpressionMeta::RIGHT_META.get(&self.data) }

  pub fn capnp_to_owned(&self) -> ExpressionShared {
    ExpressionShared { data: self.data.capnp_to_owned() }
  }
}

impl Expression for ExpressionRef<'_> {
  fn op<'a>(&'a self) -> Result<Operation, UnknownDiscriminant> {
    self.op()
 }
  fn left<'a>(&'a self) -> Result<Result<Left<'a>, UnknownDiscriminant>,Error> {
    self.left()
 }
  fn right<'a>(&'a self) -> Result<Result<Right<'a>, UnknownDiscriminant>,Error> {
    self.right()
 }
}

impl<'a> TypedStructRef<'a> for ExpressionRef<'a> {
  fn meta() -> &'static StructMeta {
    &ExpressionMeta::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    ExpressionRef { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for ExpressionRef<'a> {
  type Owned = ExpressionShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    ExpressionRef::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for ExpressionRef<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for ExpressionRef<'a> {
  fn partial_cmp(&self, other: &ExpressionRef<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for ExpressionRef<'a> {
  fn eq(&self, other: &ExpressionRef<'a>) -> bool {
    self.partial_cmp(&other) == Some(std::cmp::Ordering::Equal)
  }
}

#[derive(Clone)]
pub struct ExpressionShared {
  data: UntypedStructShared,
}

impl ExpressionShared {
  pub fn new(
    op: Operation,
    left: LeftShared,
    right: RightShared,
  ) -> ExpressionShared {
    let mut data = UntypedStructOwned::new_with_root_struct(ExpressionMeta::META.data_size, ExpressionMeta::META.pointer_size);
    ExpressionMeta::OP_META.set(&mut data, op);
    ExpressionMeta::LEFT_META.set(&mut data, left);
    ExpressionMeta::RIGHT_META.set(&mut data, right);
    ExpressionShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> ExpressionRef<'a> {
    ExpressionRef { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for ExpressionShared {
  fn meta() -> &'static StructMeta {
    &ExpressionMeta::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    ExpressionShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, ExpressionRef<'a>> for ExpressionShared {
  fn capnp_as_ref(&'a self) -> ExpressionRef<'a> {
    ExpressionShared::capnp_as_ref(self)
  }
}

#[derive(Clone)]
pub enum Left<'a> {
  Value(i32),
  Expression(ExpressionRef<'a>),
}

impl Left<'_> {
  const VALUE_META: &'static I32FieldMeta = &I32FieldMeta {
    name: "value",
    offset: NumElements(1),
  };
  const EXPRESSION_META: &'static StructFieldMeta = &StructFieldMeta {
    name: "expression",
    offset: NumElements(0),
    meta: &ExpressionMeta::META,
  };
  const META: &'static UnionMeta = &UnionMeta {
    name: "Left",
    variants: &[
      UnionVariantMeta{
        discriminant: Discriminant(0),
        field_meta: FieldMeta::I32(Left::VALUE_META),
      },
      UnionVariantMeta{
        discriminant: Discriminant(1),
        field_meta: FieldMeta::Struct(Left::EXPRESSION_META),
      },
    ],
  };

  pub fn capnp_to_owned(&self) -> LeftShared {
    match self {
      Left::Value(x) => LeftShared::Value(*x),
      Left::Expression(x) => LeftShared::Expression(x.capnp_to_owned()),
    }
  }
}

impl<'a> TypedUnion<'a> for Left<'a> {
  fn meta() -> &'static UnionMeta {
    &Left::META
  }
  fn from_untyped_union(untyped: &UntypedUnion<'a>) -> Result<Result<Self, UnknownDiscriminant>, Error> {
    match untyped.discriminant {
      Discriminant(0) => Ok(Ok(Left::Value(Left::VALUE_META.get(&untyped.variant_data)))),
      Discriminant(1) => Left::EXPRESSION_META.get(&untyped.variant_data).map(|x| Ok(Left::Expression(x))),
      x => Ok(Err(UnknownDiscriminant(x, Left::META.name))),
    }
  }
}

impl<'a> CapnpToOwned<'a> for Left<'a> {
  type Owned = LeftShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    Left::capnp_to_owned(self)
  }
}

#[derive(Clone)]
pub enum LeftShared {
  Value(i32),
  Expression(ExpressionShared),
}

impl LeftShared {
  pub fn capnp_as_ref<'a>(&'a self) -> Left<'a> {
    match self {
      LeftShared::Value(x) => Left::Value(*x),
      LeftShared::Expression(x) => Left::Expression(x.capnp_as_ref()),
    }
  }
}

impl<'a> TypedUnionShared<'a, Left<'a>> for LeftShared {
  fn set(&self, data: &mut UntypedStructOwned, discriminant_offset: NumElements) {
    match self {
      LeftShared::Value(x) => {
        data.set_discriminant(discriminant_offset, Discriminant(0));
        Left::VALUE_META.set(data, x.clone().into());
      }
      LeftShared::Expression(x) => {
        data.set_discriminant(discriminant_offset, Discriminant(1));
        Left::EXPRESSION_META.set(data, x.clone().into());
      }
    }
  }
}

impl<'a> CapnpAsRef<'a, Left<'a>> for LeftShared {
  fn capnp_as_ref(&'a self) -> Left<'a> {
    LeftShared::capnp_as_ref(self)
  }
}

#[derive(Clone)]
pub enum Right<'a> {
  Value(i32),
  Expression(ExpressionRef<'a>),
}

impl Right<'_> {
  const VALUE_META: &'static I32FieldMeta = &I32FieldMeta {
    name: "value",
    offset: NumElements(2),
  };
  const EXPRESSION_META: &'static StructFieldMeta = &StructFieldMeta {
    name: "expression",
    offset: NumElements(1),
    meta: &ExpressionMeta::META,
  };
  const META: &'static UnionMeta = &UnionMeta {
    name: "Right",
    variants: &[
      UnionVariantMeta{
        discriminant: Discriminant(0),
        field_meta: FieldMeta::I32(Right::VALUE_META),
      },
      UnionVariantMeta{
        discriminant: Discriminant(1),
        field_meta: FieldMeta::Struct(Right::EXPRESSION_META),
      },
    ],
  };

  pub fn capnp_to_owned(&self) -> RightShared {
    match self {
      Right::Value(x) => RightShared::Value(*x),
      Right::Expression(x) => RightShared::Expression(x.capnp_to_owned()),
    }
  }
}

impl<'a> TypedUnion<'a> for Right<'a> {
  fn meta() -> &'static UnionMeta {
    &Right::META
  }
  fn from_untyped_union(untyped: &UntypedUnion<'a>) -> Result<Result<Self, UnknownDiscriminant>, Error> {
    match untyped.discriminant {
      Discriminant(0) => Ok(Ok(Right::Value(Right::VALUE_META.get(&untyped.variant_data)))),
      Discriminant(1) => Right::EXPRESSION_META.get(&untyped.variant_data).map(|x| Ok(Right::Expression(x))),
      x => Ok(Err(UnknownDiscriminant(x, Right::META.name))),
    }
  }
}

impl<'a> CapnpToOwned<'a> for Right<'a> {
  type Owned = RightShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    Right::capnp_to_owned(self)
  }
}

#[derive(Clone)]
pub enum RightShared {
  Value(i32),
  Expression(ExpressionShared),
}

impl RightShared {
  pub fn capnp_as_ref<'a>(&'a self) -> Right<'a> {
    match self {
      RightShared::Value(x) => Right::Value(*x),
      RightShared::Expression(x) => Right::Expression(x.capnp_as_ref()),
    }
  }
}

impl<'a> TypedUnionShared<'a, Right<'a>> for RightShared {
  fn set(&self, data: &mut UntypedStructOwned, discriminant_offset: NumElements) {
    match self {
      RightShared::Value(x) => {
        data.set_discriminant(discriminant_offset, Discriminant(0));
        Right::VALUE_META.set(data, x.clone().into());
      }
      RightShared::Expression(x) => {
        data.set_discriminant(discriminant_offset, Discriminant(1));
        Right::EXPRESSION_META.set(data, x.clone().into());
      }
    }
  }
}

impl<'a> CapnpAsRef<'a, Right<'a>> for RightShared {
  fn capnp_as_ref(&'a self) -> Right<'a> {
    RightShared::capnp_as_ref(self)
  }
}

pub struct EvaluationResultMeta;

impl EvaluationResultMeta {
  const VALUE_META: &'static I32FieldMeta = &I32FieldMeta {
    name: "value",
    offset: NumElements(0),
  };

  const META: &'static StructMeta = &StructMeta {
    name: "EvaluationResult",
    data_size: NumWords(1),
    pointer_size: NumWords(0),
    fields: || &[
      FieldMeta::I32(EvaluationResultMeta::VALUE_META),
    ],
  };
}

impl<'a> TypedStruct<'a> for EvaluationResultMeta {
  type Ref = EvaluationResultRef<'a>;
  type Shared = EvaluationResultShared;
  fn meta() -> &'static StructMeta {
    &EvaluationResultMeta::META
  }
}

pub trait EvaluationResult {

  fn value<'a>(&'a self) -> i32;
}

#[derive(Clone)]
pub struct EvaluationResultRef<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> EvaluationResultRef<'a> {

  pub fn value(&self) -> i32 {EvaluationResultMeta::VALUE_META.get(&self.data) }

  pub fn capnp_to_owned(&self) -> EvaluationResultShared {
    EvaluationResultShared { data: self.data.capnp_to_owned() }
  }
}

impl EvaluationResult for EvaluationResultRef<'_> {
  fn value<'a>(&'a self) -> i32 {
    self.value()
 }
}

impl<'a> TypedStructRef<'a> for EvaluationResultRef<'a> {
  fn meta() -> &'static StructMeta {
    &EvaluationResultMeta::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    EvaluationResultRef { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for EvaluationResultRef<'a> {
  type Owned = EvaluationResultShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    EvaluationResultRef::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for EvaluationResultRef<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for EvaluationResultRef<'a> {
  fn partial_cmp(&self, other: &EvaluationResultRef<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for EvaluationResultRef<'a> {
  fn eq(&self, other: &EvaluationResultRef<'a>) -> bool {
    self.partial_cmp(&other) == Some(std::cmp::Ordering::Equal)
  }
}

#[derive(Clone)]
pub struct EvaluationResultShared {
  data: UntypedStructShared,
}

impl EvaluationResultShared {
  pub fn new(
    value: i32,
  ) -> EvaluationResultShared {
    let mut data = UntypedStructOwned::new_with_root_struct(EvaluationResultMeta::META.data_size, EvaluationResultMeta::META.pointer_size);
    EvaluationResultMeta::VALUE_META.set(&mut data, value);
    EvaluationResultShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> EvaluationResultRef<'a> {
    EvaluationResultRef { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for EvaluationResultShared {
  fn meta() -> &'static StructMeta {
    &EvaluationResultMeta::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    EvaluationResultShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, EvaluationResultRef<'a>> for EvaluationResultShared {
  fn capnp_as_ref(&'a self) -> EvaluationResultRef<'a> {
    EvaluationResultShared::capnp_as_ref(self)
  }
}
