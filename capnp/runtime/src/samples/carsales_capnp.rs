use capnp_runtime::prelude::*;

#[derive(Clone)]
pub struct Car<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> Car<'a> {
  const MAKE_META: &'static TextFieldMeta = &TextFieldMeta {
    name: "make",
    offset: NumElements(0),
  };
  const MODEL_META: &'static TextFieldMeta = &TextFieldMeta {
    name: "model",
    offset: NumElements(1),
  };
  const COLOR_META: &'static EnumFieldMeta = &EnumFieldMeta {
    name: "color",
    offset: NumElements(0),
    meta: &Color::META,
  };
  const SEATS_META: &'static U8FieldMeta = &U8FieldMeta {
    name: "seats",
    offset: NumElements(2),
  };
  const DOORS_META: &'static U8FieldMeta = &U8FieldMeta {
    name: "doors",
    offset: NumElements(3),
  };
  const WHEELS_META: &'static ListFieldMeta = &ListFieldMeta {
    name: "wheels",
    offset: NumElements(2),
    meta: &ListMeta {
      value_type: ElementType::Struct(&Wheel::META)
    },
  };
  const LENGTH_META: &'static U16FieldMeta = &U16FieldMeta {
    name: "length",
    offset: NumElements(2),
  };
  const WIDTH_META: &'static U16FieldMeta = &U16FieldMeta {
    name: "width",
    offset: NumElements(3),
  };
  const HEIGHT_META: &'static U16FieldMeta = &U16FieldMeta {
    name: "height",
    offset: NumElements(4),
  };
  const WEIGHT_META: &'static U32FieldMeta = &U32FieldMeta {
    name: "weight",
    offset: NumElements(3),
  };
  const ENGINE_META: &'static StructFieldMeta = &StructFieldMeta {
    name: "engine",
    offset: NumElements(3),
    meta: &Engine::META,
  };
  const FUEL_CAPACITY_META: &'static F32FieldMeta = &F32FieldMeta {
    name: "fuelCapacity",
    offset: NumElements(4),
  };
  const FUEL_LEVEL_META: &'static F32FieldMeta = &F32FieldMeta {
    name: "fuelLevel",
    offset: NumElements(5),
  };
  const HAS_POWER_WINDOWS_META: &'static BoolFieldMeta = &BoolFieldMeta {
    name: "hasPowerWindows",
    offset: NumElements(80),
  };
  const HAS_POWER_STEERING_META: &'static BoolFieldMeta = &BoolFieldMeta {
    name: "hasPowerSteering",
    offset: NumElements(81),
  };
  const HAS_CRUISE_CONTROL_META: &'static BoolFieldMeta = &BoolFieldMeta {
    name: "hasCruiseControl",
    offset: NumElements(82),
  };
  const CUP_HOLDERS_META: &'static U8FieldMeta = &U8FieldMeta {
    name: "cupHolders",
    offset: NumElements(11),
  };
  const HAS_NAV_SYSTEM_META: &'static BoolFieldMeta = &BoolFieldMeta {
    name: "hasNavSystem",
    offset: NumElements(83),
  };

  const META: &'static StructMeta = &StructMeta {
    name: "Car",
    data_size: NumWords(3),
    pointer_size: NumWords(4),
    fields: || &[
      FieldMeta::Text(Car::MAKE_META),
      FieldMeta::Text(Car::MODEL_META),
      FieldMeta::Enum(Car::COLOR_META),
      FieldMeta::U8(Car::SEATS_META),
      FieldMeta::U8(Car::DOORS_META),
      FieldMeta::List(Car::WHEELS_META),
      FieldMeta::U16(Car::LENGTH_META),
      FieldMeta::U16(Car::WIDTH_META),
      FieldMeta::U16(Car::HEIGHT_META),
      FieldMeta::U32(Car::WEIGHT_META),
      FieldMeta::Struct(Car::ENGINE_META),
      FieldMeta::F32(Car::FUEL_CAPACITY_META),
      FieldMeta::F32(Car::FUEL_LEVEL_META),
      FieldMeta::Bool(Car::HAS_POWER_WINDOWS_META),
      FieldMeta::Bool(Car::HAS_POWER_STEERING_META),
      FieldMeta::Bool(Car::HAS_CRUISE_CONTROL_META),
      FieldMeta::U8(Car::CUP_HOLDERS_META),
      FieldMeta::Bool(Car::HAS_NAV_SYSTEM_META),
    ],
  };

  pub fn make(&self) -> Result<&'a str, Error> { Car::MAKE_META.get(&self.data) }

  pub fn model(&self) -> Result<&'a str, Error> { Car::MODEL_META.get(&self.data) }

  pub fn color(&self) -> Result<Color, UnknownDiscriminant> { Car::COLOR_META.get(&self.data) }

  pub fn seats(&self) -> u8 { Car::SEATS_META.get(&self.data) }

  pub fn doors(&self) -> u8 { Car::DOORS_META.get(&self.data) }

  pub fn wheels(&self) -> Result<Slice<'a, Wheel<'a>>, Error> { Car::WHEELS_META.get(&self.data) }

  pub fn length(&self) -> u16 { Car::LENGTH_META.get(&self.data) }

  pub fn width(&self) -> u16 { Car::WIDTH_META.get(&self.data) }

  pub fn height(&self) -> u16 { Car::HEIGHT_META.get(&self.data) }

  pub fn weight(&self) -> u32 { Car::WEIGHT_META.get(&self.data) }

  pub fn engine(&self) -> Result<Engine<'a>, Error> { Car::ENGINE_META.get(&self.data) }

  pub fn fuel_capacity(&self) -> f32 { Car::FUEL_CAPACITY_META.get(&self.data) }

  pub fn fuel_level(&self) -> f32 { Car::FUEL_LEVEL_META.get(&self.data) }

  pub fn has_power_windows(&self) -> bool { Car::HAS_POWER_WINDOWS_META.get(&self.data) }

  pub fn has_power_steering(&self) -> bool { Car::HAS_POWER_STEERING_META.get(&self.data) }

  pub fn has_cruise_control(&self) -> bool { Car::HAS_CRUISE_CONTROL_META.get(&self.data) }

  pub fn cup_holders(&self) -> u8 { Car::CUP_HOLDERS_META.get(&self.data) }

  pub fn has_nav_system(&self) -> bool { Car::HAS_NAV_SYSTEM_META.get(&self.data) }

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
    make: &str,
    model: &str,
    color: Color,
    seats: u8,
    doors: u8,
    wheels: &'_ [WheelShared],
    length: u16,
    width: u16,
    height: u16,
    weight: u32,
    engine: Option<EngineShared>,
    fuel_capacity: f32,
    fuel_level: f32,
    has_power_windows: bool,
    has_power_steering: bool,
    has_cruise_control: bool,
    cup_holders: u8,
    has_nav_system: bool,
  ) -> CarShared {
    let mut data = UntypedStructOwned::new_with_root_struct(Car::META.data_size, Car::META.pointer_size);
    Car::MAKE_META.set(&mut data, make);
    Car::MODEL_META.set(&mut data, model);
    Car::COLOR_META.set(&mut data, color);
    Car::SEATS_META.set(&mut data, seats);
    Car::DOORS_META.set(&mut data, doors);
    Car::WHEELS_META.set(&mut data, wheels);
    Car::LENGTH_META.set(&mut data, length);
    Car::WIDTH_META.set(&mut data, width);
    Car::HEIGHT_META.set(&mut data, height);
    Car::WEIGHT_META.set(&mut data, weight);
    Car::ENGINE_META.set(&mut data, engine);
    Car::FUEL_CAPACITY_META.set(&mut data, fuel_capacity);
    Car::FUEL_LEVEL_META.set(&mut data, fuel_level);
    Car::HAS_POWER_WINDOWS_META.set(&mut data, has_power_windows);
    Car::HAS_POWER_STEERING_META.set(&mut data, has_power_steering);
    Car::HAS_CRUISE_CONTROL_META.set(&mut data, has_cruise_control);
    Car::CUP_HOLDERS_META.set(&mut data, cup_holders);
    Car::HAS_NAV_SYSTEM_META.set(&mut data, has_nav_system);
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

#[derive(Clone, Copy)]
pub enum Color {
  Black = 0,
  White = 1,
  Red = 2,
  Green = 3,
  Blue = 4,
  Cyan = 5,
  Magenta = 6,
  Yellow = 7,
  Silver = 8,
}

impl Color {
  const META: &'static EnumMeta = &EnumMeta {
    name: "Color",
    enumerants: &[
      EnumerantMeta{
        name: "black",
        discriminant: Discriminant(0),
      },
      EnumerantMeta{
        name: "white",
        discriminant: Discriminant(1),
      },
      EnumerantMeta{
        name: "red",
        discriminant: Discriminant(2),
      },
      EnumerantMeta{
        name: "green",
        discriminant: Discriminant(3),
      },
      EnumerantMeta{
        name: "blue",
        discriminant: Discriminant(4),
      },
      EnumerantMeta{
        name: "cyan",
        discriminant: Discriminant(5),
      },
      EnumerantMeta{
        name: "magenta",
        discriminant: Discriminant(6),
      },
      EnumerantMeta{
        name: "yellow",
        discriminant: Discriminant(7),
      },
      EnumerantMeta{
        name: "silver",
        discriminant: Discriminant(8),
      },
    ],
  };
}

impl TypedEnum for Color {
  fn meta() -> &'static EnumMeta {
    &Color::META
  }
  fn from_discriminant(discriminant: Discriminant) -> Result<Self, UnknownDiscriminant> {
   match discriminant {
      Discriminant(0) => Ok(Color::Black),
      Discriminant(1) => Ok(Color::White),
      Discriminant(2) => Ok(Color::Red),
      Discriminant(3) => Ok(Color::Green),
      Discriminant(4) => Ok(Color::Blue),
      Discriminant(5) => Ok(Color::Cyan),
      Discriminant(6) => Ok(Color::Magenta),
      Discriminant(7) => Ok(Color::Yellow),
      Discriminant(8) => Ok(Color::Silver),
      d => Err(UnknownDiscriminant(d, Color::META.name)),
    }
  }
  fn to_discriminant(&self) -> Discriminant {
    Discriminant(*self as u16)
  }
}

#[derive(Clone)]
pub struct Wheel<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> Wheel<'a> {
  const DIAMETER_META: &'static U16FieldMeta = &U16FieldMeta {
    name: "diameter",
    offset: NumElements(0),
  };
  const AIR_PRESSURE_META: &'static F32FieldMeta = &F32FieldMeta {
    name: "airPressure",
    offset: NumElements(1),
  };
  const SNOW_TIRES_META: &'static BoolFieldMeta = &BoolFieldMeta {
    name: "snowTires",
    offset: NumElements(16),
  };

  const META: &'static StructMeta = &StructMeta {
    name: "Wheel",
    data_size: NumWords(1),
    pointer_size: NumWords(0),
    fields: || &[
      FieldMeta::U16(Wheel::DIAMETER_META),
      FieldMeta::F32(Wheel::AIR_PRESSURE_META),
      FieldMeta::Bool(Wheel::SNOW_TIRES_META),
    ],
  };

  pub fn diameter(&self) -> u16 { Wheel::DIAMETER_META.get(&self.data) }

  pub fn air_pressure(&self) -> f32 { Wheel::AIR_PRESSURE_META.get(&self.data) }

  pub fn snow_tires(&self) -> bool { Wheel::SNOW_TIRES_META.get(&self.data) }

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
    diameter: u16,
    air_pressure: f32,
    snow_tires: bool,
  ) -> WheelShared {
    let mut data = UntypedStructOwned::new_with_root_struct(Wheel::META.data_size, Wheel::META.pointer_size);
    Wheel::DIAMETER_META.set(&mut data, diameter);
    Wheel::AIR_PRESSURE_META.set(&mut data, air_pressure);
    Wheel::SNOW_TIRES_META.set(&mut data, snow_tires);
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
  const HORSEPOWER_META: &'static U16FieldMeta = &U16FieldMeta {
    name: "horsepower",
    offset: NumElements(0),
  };
  const CYLINDERS_META: &'static U8FieldMeta = &U8FieldMeta {
    name: "cylinders",
    offset: NumElements(2),
  };
  const CC_META: &'static U32FieldMeta = &U32FieldMeta {
    name: "cc",
    offset: NumElements(1),
  };
  const USES_GAS_META: &'static BoolFieldMeta = &BoolFieldMeta {
    name: "usesGas",
    offset: NumElements(24),
  };
  const USES_ELECTRIC_META: &'static BoolFieldMeta = &BoolFieldMeta {
    name: "usesElectric",
    offset: NumElements(25),
  };

  const META: &'static StructMeta = &StructMeta {
    name: "Engine",
    data_size: NumWords(1),
    pointer_size: NumWords(0),
    fields: || &[
      FieldMeta::U16(Engine::HORSEPOWER_META),
      FieldMeta::U8(Engine::CYLINDERS_META),
      FieldMeta::U32(Engine::CC_META),
      FieldMeta::Bool(Engine::USES_GAS_META),
      FieldMeta::Bool(Engine::USES_ELECTRIC_META),
    ],
  };

  pub fn horsepower(&self) -> u16 { Engine::HORSEPOWER_META.get(&self.data) }

  pub fn cylinders(&self) -> u8 { Engine::CYLINDERS_META.get(&self.data) }

  pub fn cc(&self) -> u32 { Engine::CC_META.get(&self.data) }

  pub fn uses_gas(&self) -> bool { Engine::USES_GAS_META.get(&self.data) }

  pub fn uses_electric(&self) -> bool { Engine::USES_ELECTRIC_META.get(&self.data) }

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
    horsepower: u16,
    cylinders: u8,
    cc: u32,
    uses_gas: bool,
    uses_electric: bool,
  ) -> EngineShared {
    let mut data = UntypedStructOwned::new_with_root_struct(Engine::META.data_size, Engine::META.pointer_size);
    Engine::HORSEPOWER_META.set(&mut data, horsepower);
    Engine::CYLINDERS_META.set(&mut data, cylinders);
    Engine::CC_META.set(&mut data, cc);
    Engine::USES_GAS_META.set(&mut data, uses_gas);
    Engine::USES_ELECTRIC_META.set(&mut data, uses_electric);
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
