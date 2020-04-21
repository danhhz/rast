// Copyright 2020 Daniel Harrison. All Rights Reserved.

mod reqres;
pub use reqres::{Request,Response};

pub struct Rast {

}

impl Rast {
  pub fn new() -> Rast {
    Rast{}
  }

  pub fn run(&self, req: Request) -> Response {
    Response::with_payload(req.payload())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works() {
    let r = Rast::new();
    let payload = vec![];
    let res = r.run(Request::with_payload(&payload));
    assert_eq!(res.payload(), payload.as_slice());
  }
}
