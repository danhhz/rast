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

use crate::catrank_capnp::*;
use crate::common::*;
use crate::error::Error;

#[derive(Clone, Debug)]
pub struct ScoredResult<'a> {
  score: f64,
  result: SearchResultRef<'a>,
}

const URL_PREFIX: &'static str = "http://example.com";

#[derive(Clone)]
pub struct CatRank;

impl crate::TestCase for CatRank {
  type Request = SearchResultListMeta;
  type Response = SearchResultListMeta;
  type Expectation = i32;

  fn setup_request(&self, rng: &mut FastRand) -> (SearchResultListShared, i32) {
    let count = rng.next_less_than(1000);
    let mut good_count: i32 = 0;

    let list = (0..count).map(|i| {
      let score = 1000.0 - i as f64;
      let url_size = rng.next_less_than(100) as usize;

      let url_prefix_length = URL_PREFIX.as_bytes().len();
      let url = {
        let mut url = String::with_capacity(url_size + url_prefix_length);

        url.push_str(URL_PREFIX);
        for _ in 0..url_size {
          url.push((97 + rng.next_less_than(26)) as u8 as char);
        }
        url
      };

      let is_cat = rng.next_less_than(8) == 0;
      let is_dog = rng.next_less_than(8) == 0;
      if is_cat && !is_dog {
        good_count += 1;
      }

      let mut snippet = " ".to_string();

      let prefix = rng.next_less_than(20) as usize;
      for _ in 0..prefix {
        snippet.push_str(WORDS[rng.next_less_than(WORDS.len() as u32) as usize]);
      }
      if is_cat {
        snippet.push_str("cat ")
      }
      if is_dog {
        snippet.push_str("dog ")
      }

      let suffix = rng.next_less_than(20) as usize;
      for _ in 0..suffix {
        snippet.push_str(WORDS[rng.next_less_than(WORDS.len() as u32) as usize]);
      }

      SearchResultShared::new(&url, score, &snippet)
    });

    let list = SearchResultListShared::new(list.collect::<Vec<_>>().as_slice());
    (list, good_count)
  }

  fn handle_request(
    &self,
    request: SearchResultListRef<'_>,
  ) -> Result<SearchResultListShared, Error> {
    let mut scored_results: Vec<ScoredResult> = Vec::new();

    for result in request.results()? {
      let mut score = result.score();
      let snippet = result.snippet()?;
      if snippet.contains(" cat ") {
        score *= 10000.0;
      }
      if snippet.contains(" dog ") {
        score /= 10000.0;
      }
      // eprintln!("score {} {:?}", score, result);
      scored_results.push(ScoredResult { score: score, result: result });
    }

    // sort in decreasing order
    scored_results.sort_by(|v1, v2| {
      if v1.score < v2.score {
        ::std::cmp::Ordering::Greater
      } else {
        ::std::cmp::Ordering::Less
      }
    });
    // eprintln!("top scores {:?}", &scored_results[..3]);

    let scored_results = scored_results.iter().map(|ScoredResult { result, score }| {
      Ok(SearchResultShared::new(result.url()?, *score, result.snippet()?))
    });

    Ok(SearchResultListShared::new(scored_results.collect::<Result<Vec<_>, Error>>()?.as_slice()))
  }

  fn check_response(
    &self,
    response: SearchResultListRef<'_>,
    expected_good_count: i32,
  ) -> Result<(), Error> {
    let mut good_count: i32 = 0;
    for result in response.results()? {
      if result.score() > 1001.0 {
        good_count += 1;
      } else {
        break;
      }
    }

    if good_count == expected_good_count {
      Ok(())
    } else {
      for result in response.results()? {
        if result.score() > 1001.0 {
          eprintln!("{:?}", result);
        } else {
          break;
        }
      }
      Err(Error::failed(format!(
        "check_response() expected {} but got {}: {:?}",
        expected_good_count,
        good_count,
        response.results()?.iter().take(3).collect::<Vec<_>>(),
      )))
    }
  }
}
