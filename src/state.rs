
/*
 * This module is responsible for storing and making available a global
 * singleton full of ephemeral system state data.
 * If any of this data needs to be persisted it should be a new config option
 * that is written to global_context_singleton on startup.
 * 
 */

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
  pub num_connected_machines: u16,
  pub badged_in_employee_ids: Vec<String>,
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
    let data_dir = unsafe{crate::MAIN_ARGS.clone()}.unwrap().flag_config_dir.unwrap_or("/tmp/".to_string());
    let svg_map = match fs::read_to_string(format!("{}/svg_map.svg", data_dir)) {
      Ok(svg_contents) => svg_contents,
      Err(e) => {
        println!("{}", e);
        format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<svg
   xmlns:dc="http://purl.org/dc/elements/1.1/"
   xmlns:cc="http://creativecommons.org/ns#"
   xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
   xmlns:svg="http://www.w3.org/2000/svg"
   xmlns="http://www.w3.org/2000/svg"
   xmlns:sodipodi="http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd"
   xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape"
   width="200mm"
   height="100mm"
   viewBox="0 0 200 100"
   version="1.1"
   sodipodi:docname="map.svg">
  <text x="5" y="20" fill="black" font-size="12">No map, see the 'Locations' tab to upload a map.</text>
</svg>
"#)
      }
    };
    
    return GCSBundle {
      ptr: Arc::new(Mutex::new(GCS {
        num_visitors: 0,
        data_dir: Some(data_dir),
        broadcast_to_browsers: BusWrapper {
          bus: Bus::new(12),
        },
        svg_map: Some(svg_map),
        num_connected_machines: 0,
        badged_in_employee_ids: vec![],
      })),
    };
  }
  
  pub fn change_connected_machines(&self, delta: isize) {
    match self.ptr.lock() {
      Ok(mut gcs) => {
        gcs.num_connected_machines = ((gcs.num_connected_machines as isize) + delta) as u16;
      }
      Err(e) => {
        println!("{}", e);
      }
    }
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


