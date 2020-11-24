use capnp_runtime::prelude::*;

#[derive(Clone)]
pub struct Car<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> Car<'a> {
  const WHEELS_META: &'static ListFieldMeta = &ListFieldMeta {
    name: "wheels",
    offset: NumElements(2),
    meta: &ListMeta {
      value_type: ElementType::Struct(&Wheel::META)
    },
  };
  const ENGINE_META: &'static StructFieldMeta = &StructFieldMeta {
    name: "engine",
    offset: NumElements(3),
    meta: &Engine::META,
  };

  const META: &'static StructMeta = &StructMeta {
    name: "Car",
    data_size: NumWords(3),
    pointer_size: NumWords(4),
    fields: || &[
      FieldMeta::List(Car::WHEELS_META),
      FieldMeta::Struct(Car::ENGINE_META),
    ],
  };

  pub fn wheels(&self) -> Result<Slice<'a, Wheel<'a>>, Error> { Car::WHEELS_META.get(&self.data) }

  pub fn engine(&self) -> Result<Engine<'a>, Error> { Car::ENGINE_META.get(&self.data) }

  pub fn capnp_to_owned(&self) -> CarShared {
    CarShared { data: self.data.capnp_to_owned() }
  }
}

impl<'a> TypedStruct<'a> for Car<'a> {
  fn meta() -> &'static StructMeta {
    &Car::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    Car { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for Car<'a> {
  type Owned = CarShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    Car::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for Car<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for Car<'a> {
  fn partial_cmp(&self, other: &Car<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for Car<'a> {
  fn eq(&self, other: &Car<'a>) -> bool {
    self.partial_cmp(&other) == Some(std::cmp::Ordering::Equal)
  }
}

#[derive(Clone)]
pub struct CarShared {
  data: UntypedStructShared,
}

impl CarShared {
  pub fn new(
    wheels: &'_ [WheelShared],
    engine: Option<EngineShared>,
  ) -> CarShared {
    let mut data = UntypedStructOwned::new_with_root_struct(Car::META.data_size, Car::META.pointer_size);
    Car::WHEELS_META.set(&mut data, wheels);
    Car::ENGINE_META.set(&mut data, engine);
    CarShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> Car<'a> {
    Car { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for CarShared {
  fn meta() -> &'static StructMeta {
    &Car::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    CarShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, Car<'a>> for CarShared {
  fn capnp_as_ref(&'a self) -> Car<'a> {
    CarShared::capnp_as_ref(self)
  }
}

#[derive(Clone)]
pub struct ParkingLot<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> ParkingLot<'a> {
  const CARS_META: &'static ListFieldMeta = &ListFieldMeta {
    name: "cars",
    offset: NumElements(0),
    meta: &ListMeta {
      value_type: ElementType::Struct(&Car::META)
    },
  };

  const META: &'static StructMeta = &StructMeta {
    name: "ParkingLot",
    data_size: NumWords(0),
    pointer_size: NumWords(1),
    fields: || &[
      FieldMeta::List(ParkingLot::CARS_META),
    ],
  };

  pub fn cars(&self) -> Result<Slice<'a, Car<'a>>, Error> { ParkingLot::CARS_META.get(&self.data) }

  pub fn capnp_to_owned(&self) -> ParkingLotShared {
    ParkingLotShared { data: self.data.capnp_to_owned() }
  }
}

impl<'a> TypedStruct<'a> for ParkingLot<'a> {
  fn meta() -> &'static StructMeta {
    &ParkingLot::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    ParkingLot { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for ParkingLot<'a> {
  type Owned = ParkingLotShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    ParkingLot::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for ParkingLot<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for ParkingLot<'a> {
  fn partial_cmp(&self, other: &ParkingLot<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for ParkingLot<'a> {
  fn eq(&self, other: &ParkingLot<'a>) -> bool {
    self.partial_cmp(&other) == Some(std::cmp::Ordering::Equal)
  }
}

#[derive(Clone)]
pub struct ParkingLotShared {
  data: UntypedStructShared,
}

impl ParkingLotShared {
  pub fn new(
    cars: &'_ [CarShared],
  ) -> ParkingLotShared {
    let mut data = UntypedStructOwned::new_with_root_struct(ParkingLot::META.data_size, ParkingLot::META.pointer_size);
    ParkingLot::CARS_META.set(&mut data, cars);
    ParkingLotShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> ParkingLot<'a> {
    ParkingLot { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for ParkingLotShared {
  fn meta() -> &'static StructMeta {
    &ParkingLot::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    ParkingLotShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, ParkingLot<'a>> for ParkingLotShared {
  fn capnp_as_ref(&'a self) -> ParkingLot<'a> {
    ParkingLotShared::capnp_as_ref(self)
  }
}

#[derive(Clone)]
pub struct Wheel<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> Wheel<'a> {

  const META: &'static StructMeta = &StructMeta {
    name: "Wheel",
    data_size: NumWords(1),
    pointer_size: NumWords(0),
    fields: || &[
    ],
  };

  pub fn capnp_to_owned(&self) -> WheelShared {
    WheelShared { data: self.data.capnp_to_owned() }
  }
}

impl<'a> TypedStruct<'a> for Wheel<'a> {
  fn meta() -> &'static StructMeta {
    &Wheel::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    Wheel { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for Wheel<'a> {
  type Owned = WheelShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    Wheel::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for Wheel<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for Wheel<'a> {
  fn partial_cmp(&self, other: &Wheel<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for Wheel<'a> {
  fn eq(&self, other: &Wheel<'a>) -> bool {
    self.partial_cmp(&other) == Some(std::cmp::Ordering::Equal)
  }
}

#[derive(Clone)]
pub struct WheelShared {
  data: UntypedStructShared,
}

impl WheelShared {
  pub fn new(
  ) -> WheelShared {
    let mut data = UntypedStructOwned::new_with_root_struct(Wheel::META.data_size, Wheel::META.pointer_size);
    WheelShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> Wheel<'a> {
    Wheel { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for WheelShared {
  fn meta() -> &'static StructMeta {
    &Wheel::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    WheelShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, Wheel<'a>> for WheelShared {
  fn capnp_as_ref(&'a self) -> Wheel<'a> {
    WheelShared::capnp_as_ref(self)
  }
}

#[derive(Clone)]
pub struct Engine<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> Engine<'a> {

  const META: &'static StructMeta = &StructMeta {
    name: "Engine",
    data_size: NumWords(1),
    pointer_size: NumWords(0),
    fields: || &[
    ],
  };

  pub fn capnp_to_owned(&self) -> EngineShared {
    EngineShared { data: self.data.capnp_to_owned() }
  }
}

impl<'a> TypedStruct<'a> for Engine<'a> {
  fn meta() -> &'static StructMeta {
    &Engine::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    Engine { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for Engine<'a> {
  type Owned = EngineShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    Engine::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for Engine<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for Engine<'a> {
  fn partial_cmp(&self, other: &Engine<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for Engine<'a> {
  fn eq(&self, other: &Engine<'a>) -> bool {
    self.partial_cmp(&other) == Some(std::cmp::Ordering::Equal)
  }
}

#[derive(Clone)]
pub struct EngineShared {
  data: UntypedStructShared,
}

impl EngineShared {
  pub fn new(
  ) -> EngineShared {
    let mut data = UntypedStructOwned::new_with_root_struct(Engine::META.data_size, Engine::META.pointer_size);
    EngineShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> Engine<'a> {
    Engine { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for EngineShared {
  fn meta() -> &'static StructMeta {
    &Engine::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    EngineShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, Engine<'a>> for EngineShared {
  fn capnp_as_ref(&'a self) -> Engine<'a> {
    EngineShared::capnp_as_ref(self)
  }
}

#[derive(Clone)]
pub struct TotalValue<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> TotalValue<'a> {
  const AMOUNT_META: &'static U64FieldMeta = &U64FieldMeta {
    name: "amount",
    offset: NumElements(0),
  };

  const META: &'static StructMeta = &StructMeta {
    name: "TotalValue",
    data_size: NumWords(1),
    pointer_size: NumWords(0),
    fields: || &[
      FieldMeta::U64(TotalValue::AMOUNT_META),
    ],
  };

  pub fn amount(&self) -> u64 { TotalValue::AMOUNT_META.get(&self.data) }

  pub fn capnp_to_owned(&self) -> TotalValueShared {
    TotalValueShared { data: self.data.capnp_to_owned() }
  }
}

impl<'a> TypedStruct<'a> for TotalValue<'a> {
  fn meta() -> &'static StructMeta {
    &TotalValue::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    TotalValue { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for TotalValue<'a> {
  type Owned = TotalValueShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    TotalValue::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for TotalValue<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for TotalValue<'a> {
  fn partial_cmp(&self, other: &TotalValue<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for TotalValue<'a> {
  fn eq(&self, other: &TotalValue<'a>) -> bool {
    self.partial_cmp(&other) == Some(std::cmp::Ordering::Equal)
  }
}

#[derive(Clone)]
pub struct TotalValueShared {
  data: UntypedStructShared,
}

impl TotalValueShared {
  pub fn new(
    amount: u64,
  ) -> TotalValueShared {
    let mut data = UntypedStructOwned::new_with_root_struct(TotalValue::META.data_size, TotalValue::META.pointer_size);
    TotalValue::AMOUNT_META.set(&mut data, amount);
    TotalValueShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> TotalValue<'a> {
    TotalValue { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for TotalValueShared {
  fn meta() -> &'static StructMeta {
    &TotalValue::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    TotalValueShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, TotalValue<'a>> for TotalValueShared {
  fn capnp_as_ref(&'a self) -> TotalValue<'a> {
    TotalValueShared::capnp_as_ref(self)
  }
}
