
use rocket::*;
use rocket::request;
use rocket::request::FromRequest;

use directories::{BaseDirs, UserDirs, ProjectDirs};

use std::sync::{Arc, Mutex};

use std::path::{PathBuf};
use std::fs;

use bus::Bus;
use std::fmt;

// Used because we cannot impl fmt::Debug for Bus<String>
pub struct BusWrapper {
  pub bus: Bus<String>,
}

impl fmt::Debug for BusWrapper {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Bus<String> {{ <bits and bytes> }}")
  }
}

#[derive(Debug)]
pub struct GCS { // Global Context Singleton
  // These fields will be available to all HTTP handlers in routes
  pub num_visitors: u8,
  data_dir: Option<String>,
  pub broadcast_to_browsers: BusWrapper,
  pub svg_map: Option<String>,
  
}

impl GCS {
  pub fn get_data_dir(&mut self) -> PathBuf {
    match &self.data_dir {
      Some(data_dir) => {
        return PathBuf::from(&data_dir);
      }
      None => {
        match ProjectDirs::from("com", "SchedMap",  "SchedMap") {
          Some(proj_dirs) => {
            let copied_path_str = format!("{}", proj_dirs.config_dir().to_str().unwrap() );
            
            // Make dirs
            fs::create_dir_all(copied_path_str.clone()).expect("Could not create data dir");
            
            self.data_dir = Some(copied_path_str);
          }
          None => {
            panic!("We have no idea where the user's home directory is, and cannot store any data.");
          }
        }
        //self.data_dir = Some();
        return self.get_data_dir();
      }
    }
  }
}

pub struct GCSBundle {
  pub ptr: Arc<Mutex< GCS >>,
}

impl GCSBundle {
  pub fn new() -> GCSBundle {
    return GCSBundle {
      ptr: Arc::new(Mutex::new(GCS {
        num_visitors: 0,
        data_dir: None,
        broadcast_to_browsers: BusWrapper {
          bus: Bus::new(12),
        },
        svg_map: None,
        
        
      })),
    };
  }
}

lazy_static! {
  // This variable stores all global server state data
  pub static ref global_context_singleton: GCSBundle = GCSBundle::new();
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


