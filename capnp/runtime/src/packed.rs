// Copyright 2020 Daniel Harrison. All Rights Reserved.

//! Cap'n Proto "packed" compression scheme

use std::cmp;
use std::io::{self, Read};

use crate::common::WORD_BYTES;

/// Decode bytes compressed with Cap'n Proto "packed".
pub fn decode<R: Read>(r: &mut R) -> io::Result<Vec<u8>> {
  let r = PackedRead::new(r);
  r.bytes().collect()
}

struct PackedRead<R> {
  r: R,
  buf_len: usize,
  buf: [u8; WORD_BYTES],
  zero_words: u8,
  exact_words: u8,
}

impl<R: Read> PackedRead<R> {
  fn new(r: R) -> Self {
    PackedRead { r: r, buf_len: 0, buf: [0; 8], exact_words: 0, zero_words: 0 }
  }

  fn read_one(&mut self) -> io::Result<Option<u8>> {
    let mut buf = [0u8; 1];
    match self.r.read(&mut buf)? {
      0 => Ok(None),
      1 => Ok(Some(buf[0])),
      _ => unreachable!("read more than 1 byte into a 1 byte buffer"),
    }
  }

  fn read_word(&mut self, tag: u8) -> io::Result<()> {
    debug_assert_eq!(0, self.buf_len);
    self.buf.iter_mut().for_each(|x| *x = 0);
    self.buf_len = 8;
    if tag & 1 << 0 > 0 {
      self.r.read_exact(&mut self.buf[0..1])?;
    }
    if tag & 1 << 1 > 0 {
      self.r.read_exact(&mut self.buf[1..2])?;
    }
    if tag & 1 << 2 > 0 {
      self.r.read_exact(&mut self.buf[2..3])?;
    }
    if tag & 1 << 3 > 0 {
      self.r.read_exact(&mut self.buf[3..4])?;
    }
    if tag & 1 << 4 > 0 {
      self.r.read_exact(&mut self.buf[4..5])?;
    }
    if tag & 1 << 5 > 0 {
      self.r.read_exact(&mut self.buf[5..6])?;
    }
    if tag & 1 << 6 > 0 {
      self.r.read_exact(&mut self.buf[6..7])?;
    }
    if tag & 1 << 7 > 0 {
      self.r.read_exact(&mut self.buf[7..8])?;
    }
    Ok(())
  }
}

impl<R: Read> Read for PackedRead<R> {
  fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
    let mut buf = buf;
    let mut output = 0;
    loop {
      debug_assert!(self.zero_words == 0 || self.exact_words == 0);

      if buf.len() == 0 {
        return Ok(output);
      }
      if self.buf_len > 0 {
        let copy_len = cmp::min(self.buf_len, buf.len());
        debug_assert!(copy_len > 0);
        buf[..copy_len].copy_from_slice(&self.buf[..copy_len]);
        buf = &mut buf[copy_len..];
        self.buf.copy_within(copy_len.., 0);
        self.buf_len -= copy_len;
        output += copy_len;
        continue;
      }

      if self.zero_words > 0 {
        debug_assert_eq!(0, self.buf_len);
        self.zero_words -= 1;
        self.buf.iter_mut().for_each(|x| *x = 0);
        self.buf_len = 8;
        continue;
      }

      if self.exact_words > 0 {
        debug_assert_eq!(0, self.buf_len);
        self.exact_words -= 1;
        self.r.read_exact(&mut self.buf)?;
        self.buf_len = 8;
        continue;
      }

      debug_assert_eq!(0, self.buf_len);
      debug_assert_eq!(0, self.zero_words);

      let tag = match self.read_one()? {
        None => return Ok(output),
        Some(tag) => tag,
      };
      self.read_word(tag)?;

      match tag {
        0x00 => {
          self.zero_words = self
            .read_one()?
            .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "missing zeros count"))?;
          continue;
        }
        0xff => {
          self.exact_words = self.read_one()?.ok_or_else(|| {
            io::Error::new(io::ErrorKind::UnexpectedEof, "missing exact bytes count")
          })?;
        }
        _ => {} // No-op.
      }
    }
  }
}

#[cfg(test)]
mod test {
  use std::error::Error;
  use std::io::{self, Read};

  use super::PackedRead;

  fn unpack(x: &[u8]) -> Result<Vec<u8>, io::Error> {
    PackedRead::new(x).bytes().collect::<Result<Vec<_>, _>>()
  }

  #[test]
  fn packed_read() -> Result<(), Box<dyn Error>> {
    assert_eq!(unpack(&[])?, vec![0; 0]);
    assert_eq!(unpack(&[0, 0])?, vec![0; 8]);
    assert_eq!(unpack(&[0, 1])?, vec![0; 16]);
    assert_eq!(unpack(&[0, 3])?, vec![0; 32]);
    assert_eq!(unpack(&[0x24, 12, 34])?, vec![0, 0, 12, 0, 0, 34, 0, 0]);
    assert_eq!(unpack(&[0xff, 1, 3, 2, 4, 5, 7, 6, 8, 0])?, vec![1, 3, 2, 4, 5, 7, 6, 8]);
    assert_eq!(
      unpack(&[0, 0, 0xff, 1, 3, 2, 4, 5, 7, 6, 8, 0])?,
      vec![0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 2, 4, 5, 7, 6, 8]
    );
    assert_eq!(
      unpack(&[0x24, 12, 34, 0xff, 1, 3, 2, 4, 5, 7, 6, 8, 0])?,
      vec![0, 0, 12, 0, 0, 34, 0, 0, 1, 3, 2, 4, 5, 7, 6, 8]
    );
    assert_eq!(
      unpack(&[0xff, 1, 3, 2, 4, 5, 7, 6, 8, 1, 8, 6, 7, 4, 5, 2, 3, 1])?,
      vec![1, 3, 2, 4, 5, 7, 6, 8, 8, 6, 7, 4, 5, 2, 3, 1]
    );
    assert_eq!(
      unpack(&[
        0xff, 1, 2, 3, 4, 5, 6, 7, 8, 3, 1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3,
        4, 5, 6, 7, 8, 0xd6, 2, 4, 9, 5, 1
      ])?,
      vec![
        1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6,
        7, 8, 0, 2, 4, 0, 9, 0, 5, 1
      ]
    );
    assert_eq!(
      unpack(&[
        0xff, 1, 2, 3, 4, 5, 6, 7, 8, 3, 1, 2, 3, 4, 5, 6, 7, 8, 6, 2, 4, 3, 9, 0, 5, 1, 1, 2, 3,
        4, 5, 6, 7, 8, 0xd6, 2, 4, 9, 5, 1
      ])?,
      vec![
        1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8, 6, 2, 4, 3, 9, 0, 5, 1, 1, 2, 3, 4, 5, 6,
        7, 8, 0, 2, 4, 0, 9, 0, 5, 1
      ]
    );
    assert_eq!(
      unpack(&[0xed, 8, 100, 6, 1, 1, 2, 0, 2, 0xd4, 1, 2, 3, 1])?,
      vec![
        8, 0, 100, 6, 0, 1, 1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 1, 0, 2, 0, 3, 1
      ]
    );
    Ok(())
  }
}
