// Copyright 2020 Daniel Harrison. All Rights Reserved.

pub struct Request {
  // TODO: &[u8]
  buf: Vec<u8>
}

impl Request {
  pub fn with_payload(payload: &[u8]) -> Request {
    let len = 5*4 + payload.len();
    let mut buf = Vec::with_capacity(len);
    buf.copy_from_slice(payload);
    // WIP
    Request{ buf }
  }

  pub fn payload(&self) -> &[u8] {
    // WIP
    &self.buf
  }
}

pub struct Response {
  // TODO: &[u8]
  buf: Vec<u8>
}

impl Response {
  pub fn with_payload(payload: &[u8]) -> Response {
    let len = 5*4 + payload.len();
    let mut buf = Vec::with_capacity(len);
    buf.copy_from_slice(payload);
    // WIP
    Response{ buf }
  }

  pub fn payload(&self) -> &[u8] {
    // WIP
    &self.buf
  }
}
