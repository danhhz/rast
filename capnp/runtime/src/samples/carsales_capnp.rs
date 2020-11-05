use capnp_runtime::prelude::*;

pub struct CarMeta;

impl CarMeta {
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
      value_type: ElementType::Struct(&WheelMeta::META)
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
    meta: &EngineMeta::META,
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
      FieldMeta::Text(CarMeta::MAKE_META),
      FieldMeta::Text(CarMeta::MODEL_META),
      FieldMeta::Enum(CarMeta::COLOR_META),
      FieldMeta::U8(CarMeta::SEATS_META),
      FieldMeta::U8(CarMeta::DOORS_META),
      FieldMeta::List(CarMeta::WHEELS_META),
      FieldMeta::U16(CarMeta::LENGTH_META),
      FieldMeta::U16(CarMeta::WIDTH_META),
      FieldMeta::U16(CarMeta::HEIGHT_META),
      FieldMeta::U32(CarMeta::WEIGHT_META),
      FieldMeta::Struct(CarMeta::ENGINE_META),
      FieldMeta::F32(CarMeta::FUEL_CAPACITY_META),
      FieldMeta::F32(CarMeta::FUEL_LEVEL_META),
      FieldMeta::Bool(CarMeta::HAS_POWER_WINDOWS_META),
      FieldMeta::Bool(CarMeta::HAS_POWER_STEERING_META),
      FieldMeta::Bool(CarMeta::HAS_CRUISE_CONTROL_META),
      FieldMeta::U8(CarMeta::CUP_HOLDERS_META),
      FieldMeta::Bool(CarMeta::HAS_NAV_SYSTEM_META),
    ],
  };
}

impl<'a> TypedStruct<'a> for CarMeta {
  type Ref = CarRef<'a>;
  type Shared = CarShared;
  fn meta() -> &'static StructMeta {
    &CarMeta::META
  }
}

pub trait Car {

  fn make<'a>(&'a self) -> Result<&'a str, Error>;

  fn model<'a>(&'a self) -> Result<&'a str, Error>;

  fn color<'a>(&'a self) -> Result<Color, UnknownDiscriminant>;

  fn seats<'a>(&'a self) -> u8;

  fn doors<'a>(&'a self) -> u8;

  fn wheels<'a>(&'a self) -> Result<Slice<'a, WheelRef<'a>>, Error>;

  fn length<'a>(&'a self) -> u16;

  fn width<'a>(&'a self) -> u16;

  fn height<'a>(&'a self) -> u16;

  fn weight<'a>(&'a self) -> u32;

  fn engine<'a>(&'a self) -> Result<EngineRef<'a>, Error>;

  fn fuel_capacity<'a>(&'a self) -> f32;

  fn fuel_level<'a>(&'a self) -> f32;

  fn has_power_windows<'a>(&'a self) -> bool;

  fn has_power_steering<'a>(&'a self) -> bool;

  fn has_cruise_control<'a>(&'a self) -> bool;

  fn cup_holders<'a>(&'a self) -> u8;

  fn has_nav_system<'a>(&'a self) -> bool;
}

#[derive(Clone)]
pub struct CarRef<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> CarRef<'a> {

  pub fn make(&self) -> Result<&'a str, Error> {CarMeta::MAKE_META.get(&self.data) }

  pub fn model(&self) -> Result<&'a str, Error> {CarMeta::MODEL_META.get(&self.data) }

  pub fn color(&self) -> Result<Color, UnknownDiscriminant> {CarMeta::COLOR_META.get(&self.data) }

  pub fn seats(&self) -> u8 {CarMeta::SEATS_META.get(&self.data) }

  pub fn doors(&self) -> u8 {CarMeta::DOORS_META.get(&self.data) }

  pub fn wheels(&self) -> Result<Slice<'a, WheelRef<'a>>, Error> {CarMeta::WHEELS_META.get(&self.data) }

  pub fn length(&self) -> u16 {CarMeta::LENGTH_META.get(&self.data) }

  pub fn width(&self) -> u16 {CarMeta::WIDTH_META.get(&self.data) }

  pub fn height(&self) -> u16 {CarMeta::HEIGHT_META.get(&self.data) }

  pub fn weight(&self) -> u32 {CarMeta::WEIGHT_META.get(&self.data) }

  pub fn engine(&self) -> Result<EngineRef<'a>, Error> {CarMeta::ENGINE_META.get(&self.data) }

  pub fn fuel_capacity(&self) -> f32 {CarMeta::FUEL_CAPACITY_META.get(&self.data) }

  pub fn fuel_level(&self) -> f32 {CarMeta::FUEL_LEVEL_META.get(&self.data) }

  pub fn has_power_windows(&self) -> bool {CarMeta::HAS_POWER_WINDOWS_META.get(&self.data) }

  pub fn has_power_steering(&self) -> bool {CarMeta::HAS_POWER_STEERING_META.get(&self.data) }

  pub fn has_cruise_control(&self) -> bool {CarMeta::HAS_CRUISE_CONTROL_META.get(&self.data) }

  pub fn cup_holders(&self) -> u8 {CarMeta::CUP_HOLDERS_META.get(&self.data) }

  pub fn has_nav_system(&self) -> bool {CarMeta::HAS_NAV_SYSTEM_META.get(&self.data) }

  pub fn capnp_to_owned(&self) -> CarShared {
    CarShared { data: self.data.capnp_to_owned() }
  }
}

impl Car for CarRef<'_> {
  fn make<'a>(&'a self) -> Result<&'a str, Error> {
    self.make()
 }
  fn model<'a>(&'a self) -> Result<&'a str, Error> {
    self.model()
 }
  fn color<'a>(&'a self) -> Result<Color, UnknownDiscriminant> {
    self.color()
 }
  fn seats<'a>(&'a self) -> u8 {
    self.seats()
 }
  fn doors<'a>(&'a self) -> u8 {
    self.doors()
 }
  fn wheels<'a>(&'a self) -> Result<Slice<'a, WheelRef<'a>>, Error> {
    self.wheels()
 }
  fn length<'a>(&'a self) -> u16 {
    self.length()
 }
  fn width<'a>(&'a self) -> u16 {
    self.width()
 }
  fn height<'a>(&'a self) -> u16 {
    self.height()
 }
  fn weight<'a>(&'a self) -> u32 {
    self.weight()
 }
  fn engine<'a>(&'a self) -> Result<EngineRef<'a>, Error> {
    self.engine()
 }
  fn fuel_capacity<'a>(&'a self) -> f32 {
    self.fuel_capacity()
 }
  fn fuel_level<'a>(&'a self) -> f32 {
    self.fuel_level()
 }
  fn has_power_windows<'a>(&'a self) -> bool {
    self.has_power_windows()
 }
  fn has_power_steering<'a>(&'a self) -> bool {
    self.has_power_steering()
 }
  fn has_cruise_control<'a>(&'a self) -> bool {
    self.has_cruise_control()
 }
  fn cup_holders<'a>(&'a self) -> u8 {
    self.cup_holders()
 }
  fn has_nav_system<'a>(&'a self) -> bool {
    self.has_nav_system()
 }
}

impl<'a> TypedStructRef<'a> for CarRef<'a> {
  fn meta() -> &'static StructMeta {
    &CarMeta::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    CarRef { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for CarRef<'a> {
  type Owned = CarShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    CarRef::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for CarRef<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for CarRef<'a> {
  fn partial_cmp(&self, other: &CarRef<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for CarRef<'a> {
  fn eq(&self, other: &CarRef<'a>) -> bool {
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
    let mut data = UntypedStructOwned::new_with_root_struct(CarMeta::META.data_size, CarMeta::META.pointer_size);
    CarMeta::MAKE_META.set(&mut data, make);
    CarMeta::MODEL_META.set(&mut data, model);
    CarMeta::COLOR_META.set(&mut data, color);
    CarMeta::SEATS_META.set(&mut data, seats);
    CarMeta::DOORS_META.set(&mut data, doors);
    CarMeta::WHEELS_META.set(&mut data, wheels);
    CarMeta::LENGTH_META.set(&mut data, length);
    CarMeta::WIDTH_META.set(&mut data, width);
    CarMeta::HEIGHT_META.set(&mut data, height);
    CarMeta::WEIGHT_META.set(&mut data, weight);
    CarMeta::ENGINE_META.set(&mut data, engine);
    CarMeta::FUEL_CAPACITY_META.set(&mut data, fuel_capacity);
    CarMeta::FUEL_LEVEL_META.set(&mut data, fuel_level);
    CarMeta::HAS_POWER_WINDOWS_META.set(&mut data, has_power_windows);
    CarMeta::HAS_POWER_STEERING_META.set(&mut data, has_power_steering);
    CarMeta::HAS_CRUISE_CONTROL_META.set(&mut data, has_cruise_control);
    CarMeta::CUP_HOLDERS_META.set(&mut data, cup_holders);
    CarMeta::HAS_NAV_SYSTEM_META.set(&mut data, has_nav_system);
    CarShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> CarRef<'a> {
    CarRef { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for CarShared {
  fn meta() -> &'static StructMeta {
    &CarMeta::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    CarShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, CarRef<'a>> for CarShared {
  fn capnp_as_ref(&'a self) -> CarRef<'a> {
    CarShared::capnp_as_ref(self)
  }
}

pub struct ParkingLotMeta;

impl ParkingLotMeta {
  const CARS_META: &'static ListFieldMeta = &ListFieldMeta {
    name: "cars",
    offset: NumElements(0),
    meta: &ListMeta {
      value_type: ElementType::Struct(&CarMeta::META)
    },
  };

  const META: &'static StructMeta = &StructMeta {
    name: "ParkingLot",
    data_size: NumWords(0),
    pointer_size: NumWords(1),
    fields: || &[
      FieldMeta::List(ParkingLotMeta::CARS_META),
    ],
  };
}

impl<'a> TypedStruct<'a> for ParkingLotMeta {
  type Ref = ParkingLotRef<'a>;
  type Shared = ParkingLotShared;
  fn meta() -> &'static StructMeta {
    &ParkingLotMeta::META
  }
}

pub trait ParkingLot {

  fn cars<'a>(&'a self) -> Result<Slice<'a, CarRef<'a>>, Error>;
}

#[derive(Clone)]
pub struct ParkingLotRef<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> ParkingLotRef<'a> {

  pub fn cars(&self) -> Result<Slice<'a, CarRef<'a>>, Error> {ParkingLotMeta::CARS_META.get(&self.data) }

  pub fn capnp_to_owned(&self) -> ParkingLotShared {
    ParkingLotShared { data: self.data.capnp_to_owned() }
  }
}

impl ParkingLot for ParkingLotRef<'_> {
  fn cars<'a>(&'a self) -> Result<Slice<'a, CarRef<'a>>, Error> {
    self.cars()
 }
}

impl<'a> TypedStructRef<'a> for ParkingLotRef<'a> {
  fn meta() -> &'static StructMeta {
    &ParkingLotMeta::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    ParkingLotRef { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for ParkingLotRef<'a> {
  type Owned = ParkingLotShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    ParkingLotRef::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for ParkingLotRef<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for ParkingLotRef<'a> {
  fn partial_cmp(&self, other: &ParkingLotRef<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for ParkingLotRef<'a> {
  fn eq(&self, other: &ParkingLotRef<'a>) -> bool {
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
    let mut data = UntypedStructOwned::new_with_root_struct(ParkingLotMeta::META.data_size, ParkingLotMeta::META.pointer_size);
    ParkingLotMeta::CARS_META.set(&mut data, cars);
    ParkingLotShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> ParkingLotRef<'a> {
    ParkingLotRef { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for ParkingLotShared {
  fn meta() -> &'static StructMeta {
    &ParkingLotMeta::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    ParkingLotShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, ParkingLotRef<'a>> for ParkingLotShared {
  fn capnp_as_ref(&'a self) -> ParkingLotRef<'a> {
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

pub struct WheelMeta;

impl WheelMeta {
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
      FieldMeta::U16(WheelMeta::DIAMETER_META),
      FieldMeta::F32(WheelMeta::AIR_PRESSURE_META),
      FieldMeta::Bool(WheelMeta::SNOW_TIRES_META),
    ],
  };
}

impl<'a> TypedStruct<'a> for WheelMeta {
  type Ref = WheelRef<'a>;
  type Shared = WheelShared;
  fn meta() -> &'static StructMeta {
    &WheelMeta::META
  }
}

pub trait Wheel {

  fn diameter<'a>(&'a self) -> u16;

  fn air_pressure<'a>(&'a self) -> f32;

  fn snow_tires<'a>(&'a self) -> bool;
}

#[derive(Clone)]
pub struct WheelRef<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> WheelRef<'a> {

  pub fn diameter(&self) -> u16 {WheelMeta::DIAMETER_META.get(&self.data) }

  pub fn air_pressure(&self) -> f32 {WheelMeta::AIR_PRESSURE_META.get(&self.data) }

  pub fn snow_tires(&self) -> bool {WheelMeta::SNOW_TIRES_META.get(&self.data) }

  pub fn capnp_to_owned(&self) -> WheelShared {
    WheelShared { data: self.data.capnp_to_owned() }
  }
}

impl Wheel for WheelRef<'_> {
  fn diameter<'a>(&'a self) -> u16 {
    self.diameter()
 }
  fn air_pressure<'a>(&'a self) -> f32 {
    self.air_pressure()
 }
  fn snow_tires<'a>(&'a self) -> bool {
    self.snow_tires()
 }
}

impl<'a> TypedStructRef<'a> for WheelRef<'a> {
  fn meta() -> &'static StructMeta {
    &WheelMeta::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    WheelRef { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for WheelRef<'a> {
  type Owned = WheelShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    WheelRef::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for WheelRef<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for WheelRef<'a> {
  fn partial_cmp(&self, other: &WheelRef<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for WheelRef<'a> {
  fn eq(&self, other: &WheelRef<'a>) -> bool {
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
    let mut data = UntypedStructOwned::new_with_root_struct(WheelMeta::META.data_size, WheelMeta::META.pointer_size);
    WheelMeta::DIAMETER_META.set(&mut data, diameter);
    WheelMeta::AIR_PRESSURE_META.set(&mut data, air_pressure);
    WheelMeta::SNOW_TIRES_META.set(&mut data, snow_tires);
    WheelShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> WheelRef<'a> {
    WheelRef { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for WheelShared {
  fn meta() -> &'static StructMeta {
    &WheelMeta::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    WheelShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, WheelRef<'a>> for WheelShared {
  fn capnp_as_ref(&'a self) -> WheelRef<'a> {
    WheelShared::capnp_as_ref(self)
  }
}

pub struct EngineMeta;

impl EngineMeta {
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
      FieldMeta::U16(EngineMeta::HORSEPOWER_META),
      FieldMeta::U8(EngineMeta::CYLINDERS_META),
      FieldMeta::U32(EngineMeta::CC_META),
      FieldMeta::Bool(EngineMeta::USES_GAS_META),
      FieldMeta::Bool(EngineMeta::USES_ELECTRIC_META),
    ],
  };
}

impl<'a> TypedStruct<'a> for EngineMeta {
  type Ref = EngineRef<'a>;
  type Shared = EngineShared;
  fn meta() -> &'static StructMeta {
    &EngineMeta::META
  }
}

pub trait Engine {

  fn horsepower<'a>(&'a self) -> u16;

  fn cylinders<'a>(&'a self) -> u8;

  fn cc<'a>(&'a self) -> u32;

  fn uses_gas<'a>(&'a self) -> bool;

  fn uses_electric<'a>(&'a self) -> bool;
}

#[derive(Clone)]
pub struct EngineRef<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> EngineRef<'a> {

  pub fn horsepower(&self) -> u16 {EngineMeta::HORSEPOWER_META.get(&self.data) }

  pub fn cylinders(&self) -> u8 {EngineMeta::CYLINDERS_META.get(&self.data) }

  pub fn cc(&self) -> u32 {EngineMeta::CC_META.get(&self.data) }

  pub fn uses_gas(&self) -> bool {EngineMeta::USES_GAS_META.get(&self.data) }

  pub fn uses_electric(&self) -> bool {EngineMeta::USES_ELECTRIC_META.get(&self.data) }

  pub fn capnp_to_owned(&self) -> EngineShared {
    EngineShared { data: self.data.capnp_to_owned() }
  }
}

impl Engine for EngineRef<'_> {
  fn horsepower<'a>(&'a self) -> u16 {
    self.horsepower()
 }
  fn cylinders<'a>(&'a self) -> u8 {
    self.cylinders()
 }
  fn cc<'a>(&'a self) -> u32 {
    self.cc()
 }
  fn uses_gas<'a>(&'a self) -> bool {
    self.uses_gas()
 }
  fn uses_electric<'a>(&'a self) -> bool {
    self.uses_electric()
 }
}

impl<'a> TypedStructRef<'a> for EngineRef<'a> {
  fn meta() -> &'static StructMeta {
    &EngineMeta::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    EngineRef { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for EngineRef<'a> {
  type Owned = EngineShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    EngineRef::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for EngineRef<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for EngineRef<'a> {
  fn partial_cmp(&self, other: &EngineRef<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for EngineRef<'a> {
  fn eq(&self, other: &EngineRef<'a>) -> bool {
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
    let mut data = UntypedStructOwned::new_with_root_struct(EngineMeta::META.data_size, EngineMeta::META.pointer_size);
    EngineMeta::HORSEPOWER_META.set(&mut data, horsepower);
    EngineMeta::CYLINDERS_META.set(&mut data, cylinders);
    EngineMeta::CC_META.set(&mut data, cc);
    EngineMeta::USES_GAS_META.set(&mut data, uses_gas);
    EngineMeta::USES_ELECTRIC_META.set(&mut data, uses_electric);
    EngineShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> EngineRef<'a> {
    EngineRef { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for EngineShared {
  fn meta() -> &'static StructMeta {
    &EngineMeta::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    EngineShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, EngineRef<'a>> for EngineShared {
  fn capnp_as_ref(&'a self) -> EngineRef<'a> {
    EngineShared::capnp_as_ref(self)
  }
}

pub struct TotalValueMeta;

impl TotalValueMeta {
  const AMOUNT_META: &'static U64FieldMeta = &U64FieldMeta {
    name: "amount",
    offset: NumElements(0),
  };

  const META: &'static StructMeta = &StructMeta {
    name: "TotalValue",
    data_size: NumWords(1),
    pointer_size: NumWords(0),
    fields: || &[
      FieldMeta::U64(TotalValueMeta::AMOUNT_META),
    ],
  };
}

impl<'a> TypedStruct<'a> for TotalValueMeta {
  type Ref = TotalValueRef<'a>;
  type Shared = TotalValueShared;
  fn meta() -> &'static StructMeta {
    &TotalValueMeta::META
  }
}

pub trait TotalValue {

  fn amount<'a>(&'a self) -> u64;
}

#[derive(Clone)]
pub struct TotalValueRef<'a> {
  data: UntypedStruct<'a>,
}

impl<'a> TotalValueRef<'a> {

  pub fn amount(&self) -> u64 {TotalValueMeta::AMOUNT_META.get(&self.data) }

  pub fn capnp_to_owned(&self) -> TotalValueShared {
    TotalValueShared { data: self.data.capnp_to_owned() }
  }
}

impl TotalValue for TotalValueRef<'_> {
  fn amount<'a>(&'a self) -> u64 {
    self.amount()
 }
}

impl<'a> TypedStructRef<'a> for TotalValueRef<'a> {
  fn meta() -> &'static StructMeta {
    &TotalValueMeta::META
  }
  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {
    TotalValueRef { data: data }
  }
  fn as_untyped(&self) -> UntypedStruct<'a> {
    self.data.clone()
  }
}

impl<'a> CapnpToOwned<'a> for TotalValueRef<'a> {
  type Owned = TotalValueShared;
  fn capnp_to_owned(&self) -> Self::Owned {
    TotalValueRef::capnp_to_owned(self)
  }
}

impl<'a> std::fmt::Debug for TotalValueRef<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.as_element().fmt(f)
  }
}

impl<'a> std::cmp::PartialOrd for TotalValueRef<'a> {
  fn partial_cmp(&self, other: &TotalValueRef<'a>) -> Option<std::cmp::Ordering> {
    self.as_element().partial_cmp(&other.as_element())
  }
}

impl<'a> std::cmp::PartialEq for TotalValueRef<'a> {
  fn eq(&self, other: &TotalValueRef<'a>) -> bool {
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
    let mut data = UntypedStructOwned::new_with_root_struct(TotalValueMeta::META.data_size, TotalValueMeta::META.pointer_size);
    TotalValueMeta::AMOUNT_META.set(&mut data, amount);
    TotalValueShared { data: data.into_shared() }
  }

  pub fn capnp_as_ref<'a>(&'a self) -> TotalValueRef<'a> {
    TotalValueRef { data: self.data.capnp_as_ref() }
  }
}

impl TypedStructShared for TotalValueShared {
  fn meta() -> &'static StructMeta {
    &TotalValueMeta::META
  }
  fn from_untyped_struct(data: UntypedStructShared) -> Self {
    TotalValueShared { data: data }
  }
  fn as_untyped(&self) -> UntypedStructShared {
    self.data.clone()
  }
}

impl<'a> CapnpAsRef<'a, TotalValueRef<'a>> for TotalValueShared {
  fn capnp_as_ref(&'a self) -> TotalValueRef<'a> {
    TotalValueShared::capnp_as_ref(self)
  }
}
