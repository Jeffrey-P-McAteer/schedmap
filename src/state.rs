
use rocket::*;
use rocket::request;
use rocket::request::FromRequest;

use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, PartialEq)]
pub struct GCS { // Global Context Singleton
  // These fields will be available to all HTTP handlers in routes
  pub num_visitors: u8,
}

pub struct GCSBundle {
  pub ptr: Arc<Mutex<GCS>>,
}

lazy_static! {
  // This variable stores all global server state data
  static ref global_context_singleton: GCSBundle = GCSBundle {
    ptr: Arc::new(Mutex::new(GCS {
      num_visitors: 0,
    }))
  };
}

impl<'r, 'a> FromRequest<'r, 'a> for GCSBundle {
  type Error = ();

  fn from_request(_request: &'r Request<'a>) -> request::Outcome<GCSBundle, ()> {
    // Always return the same mutex value by cloning the Arc pointer to it
    return Outcome::Success(
      GCSBundle { ptr: global_context_singleton.ptr.clone() }
    );
  }
}


