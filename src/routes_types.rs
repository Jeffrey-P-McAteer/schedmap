
/*
 * This module is used to write implementations for our own data structures
 * to make them integrate well with 3rd-party libraries.
 */

use rocket::response::content::{Html,Css,JavaScript,Content};
use rocket::http::{ContentType,RawStr};
//use rocket::Data;
use rocket::request::FromRequest;
use rocket::*;

use crate::routes::*;

#[derive(Clone, Debug, PartialEq)]
pub enum MultiPartBoundry {
  Value(String),
  Unk
}

impl MultiPartBoundry {
  pub fn to_string(self) -> String {
    match self {
      MultiPartBoundry::Value(value) => {
        format!("{}", value)
      }
      MultiPartBoundry::Unk => {
        format!("")
      }
    }
  }
}

impl<'r, 'a> FromRequest<'r, 'a> for MultiPartBoundry {
  type Error = ();

  fn from_request(request: &'r Request<'a>) -> request::Outcome<MultiPartBoundry, ()> {
    let keys = request.headers().get("Content-Type").collect::<Vec<_>>();
    if keys.len() < 1 {
      return Outcome::Success(MultiPartBoundry::Unk);
    }

    let vec = keys[0].split("boundary=").collect::<Vec<&str>>();
    if vec.len() > 1 {
      return Outcome::Success(
        MultiPartBoundry::Value(vec[1].to_string())
      );
    }

    return Outcome::Success(MultiPartBoundry::Unk);
  }
}

