// Copyright (c) 2013-2014 Sandstorm Development Group, Inc. and contributors
// Licensed under the MIT License:
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use capnp_runtime::prelude::TypedEnum;

use crate::carsales_capnp::*;
use crate::common::*;
use crate::error::Error;

trait CarValue {
  fn car_value(&self) -> Result<u64, Error>;
}

impl<'a> CarValue for CarRef<'a> {
  fn car_value(&self) -> Result<u64, Error> {
    let mut result: u64 = 0;
    result += self.seats() as u64 * 200;
    result += self.doors() as u64 * 350;

    {
      for wheel in self.wheels()? {
        result += wheel.diameter() as u64 * wheel.diameter() as u64;
        result += if wheel.snow_tires() { 100 } else { 0 };
      }
    }

    result += self.length() as u64 * self.width() as u64 * self.height() as u64 / 50;

    {
      let engine = self.engine()?;
      result += engine.horsepower() as u64 * 40;
      if engine.uses_electric() {
        if engine.uses_gas() {
          //# hybrid
          result += 5000;
        } else {
          result += 3000;
        }
      }
    }

    result += if self.has_power_windows() { 100 } else { 0 };
    result += if self.has_power_steering() { 200 } else { 0 };
    result += if self.has_cruise_control() { 400 } else { 0 };
    result += if self.has_nav_system() { 2000 } else { 0 };

    result += self.cup_holders() as u64 * 25;

    // eprintln!("value {} {:?}", result, self);

    Ok(result)
  }
}

const MAKES: [&'static str; 5] = ["Toyota", "GM", "Ford", "Honda", "Tesla"];
const MODELS: [&'static str; 6] = ["Camry", "Prius", "Volt", "Accord", "Leaf", "Model S"];

pub fn random_car(rng: &mut FastRand) -> CarShared {
  let make = MAKES[rng.next_less_than(MAKES.len() as u32) as usize];
  let model = MODELS[rng.next_less_than(MODELS.len() as u32) as usize];

  let color_enumerants = Color::meta().enumerants;
  let color = Color::from_discriminant(
    color_enumerants[rng.next_less_than(color_enumerants.len() as u32) as usize].discriminant,
  )
  .expect("internal logic error");
  let seats = 2 + rng.next_less_than(6) as u8;
  let doors = 2 + rng.next_less_than(3) as u8;

  let length = 170 + rng.next_less_than(150) as u16;
  let width = 48 + rng.next_less_than(36) as u16;
  let height = 54 + rng.next_less_than(48) as u16;
  let weight = length as u32 * width as u32 * height as u32 / 200;

  let engine = {
    let horsepower = 100 * rng.next_less_than(400) as u16;
    let cylinders = 4 + 2 * rng.next_less_than(3) as u8;
    let cc = 800 + rng.next_less_than(10000);
    let uses_gas = true;
    let uses_electric = rng.next_bool();
    EngineShared::new(horsepower, cylinders, cc, uses_gas, uses_electric)
  };

  let fuel_capacity = (10.0 + rng.next_double(30.0)) as f32;
  let fuel_level = rng.next_double(fuel_capacity as f64) as f32;
  let has_power_windows = rng.next_bool();
  let has_power_steering = rng.next_bool();
  let has_cruise_control = rng.next_bool();
  let cup_holders = rng.next_less_than(12) as u8;
  let has_nav_system = rng.next_bool();

  let wheels = (0..4).map(|_| {
    let diameter = 25 + rng.next_less_than(15) as u16;
    let air_pressure = (30.0 + rng.next_double(20.0)) as f32;
    let snow_tires = rng.next_less_than(16) == 0;
    WheelShared::new(diameter, air_pressure, snow_tires)
  });

  CarShared::new(
    make,
    model,
    color,
    seats,
    doors,
    wheels.collect::<Vec<_>>().as_slice(),
    length,
    width,
    height,
    weight,
    Some(engine),
    fuel_capacity,
    fuel_level,
    has_power_windows,
    has_power_steering,
    has_cruise_control,
    cup_holders,
    has_nav_system,
  )
}

#[derive(Clone)]
pub struct CarSales;

impl crate::TestCase for CarSales {
  type Request = ParkingLotMeta;
  type Response = TotalValueMeta;
  type Expectation = u64;

  fn setup_request(&self, rng: &mut FastRand) -> (ParkingLotShared, u64) {
    let mut total_value = 0;
    let cars = (0..rng.next_less_than(200)).map(|_| {
      let car = random_car(rng);
      total_value += car.capnp_as_ref().car_value().unwrap();
      car
    });

    let lot = ParkingLotShared::new(cars.collect::<Vec<_>>().as_slice());
    (lot, total_value)
  }

  fn handle_request(&self, request: ParkingLotRef<'_>) -> Result<TotalValueShared, Error> {
    let mut result = 0;
    for car in request.cars()?.iter() {
      result += car.car_value()?;
    }
    Ok(TotalValueShared::new(result))
  }

  fn check_response(&self, response: TotalValueRef<'_>, expected: u64) -> Result<(), Error> {
    if response.amount() == expected {
      Ok(())
    } else {
      Err(Error::failed(format!(
        "check_response() expected {} but got {}",
        expected,
        response.amount()
      )))
    }
  }
}
