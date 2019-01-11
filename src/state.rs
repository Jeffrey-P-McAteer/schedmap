
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

use csv;

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

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
pub struct EmployeeBadgeIn {
  pub employee_badge_id: String,
  pub employee_location: Option<String>, // None indicates not badged in anywhere
}

impl EmployeeBadgeIn {
  pub fn new(employee_badge_id: String) -> EmployeeBadgeIn {
    EmployeeBadgeIn {
      employee_badge_id: employee_badge_id,
      employee_location: None
    }
  }
}

impl PartialEq for EmployeeBadgeIn {
    fn eq(&self, other: &EmployeeBadgeIn) -> bool {
        self.employee_badge_id == other.employee_badge_id
    }
}

#[derive(Debug)]
pub struct EmployeeRecord {
  pub employee_badge_id: String,
  pub employee_name: String,
}

impl EmployeeRecord {
  pub fn new(employee_badge_id: String, employee_name: String) -> EmployeeRecord {
    EmployeeRecord {
      employee_badge_id: employee_badge_id,
      employee_name: employee_name
    }
  }
}

impl PartialEq for EmployeeRecord {
    fn eq(&self, other: &EmployeeRecord) -> bool {
        self.employee_badge_id == other.employee_badge_id
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
  pub badged_in_employee_ids: Vec<EmployeeBadgeIn>,
  pub known_employees: Vec<EmployeeRecord>,
}

impl GCS {
  pub fn get_data_dir(&mut self) -> PathBuf {
    return GCS::get_data_dir_static(&mut self.data_dir);
  }
  pub fn get_data_dir_static(mut data_dir: &mut Option<String>) -> PathBuf {
    match data_dir {
      Some(data_dir) => {
        return PathBuf::from(&data_dir);
      }
      None => {
        match ProjectDirs::from("com", "SchedMap",  "SchedMap") {
          Some(proj_dirs) => {
            let copied_path_str = format!("{}", proj_dirs.config_dir().to_str().unwrap() );
            
            // Make dirs
            fs::create_dir_all(copied_path_str.clone()).expect("Could not create data dir");
            
            (*data_dir) = Some(copied_path_str);
          }
          None => {
            panic!("We have no idea where the user's home directory is, and cannot store any data.");
          }
        }
        return GCS::get_data_dir_static(&mut data_dir);
      }
    }
  }
  pub fn get_map_room_ids(&self) -> Vec<String> {
    use quick_xml::{Reader, events::Event};
    use std::str;
    
    let mut room_ids: Vec<String> = vec![];
    
    match &self.svg_map {
      Some(svg_map) => {
        let our_svg_map = svg_map.clone();
        let our_svg_map = our_svg_map.as_str();
        let mut reader = Reader::from_str(our_svg_map);
        reader.trim_text(true);
        
        loop {
          let mut buf = Vec::new();
          match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name() {
                    b"rect" => {
                      let mut attrib_iter = e.attributes();
                      let attrib_iter = attrib_iter.with_checks(false);
                      
                      for attrib in attrib_iter {
                        match attrib {
                          Ok(attrib) => {
                            let key = str::from_utf8(attrib.key).expect("Error decoding an xml attribute");
                            let value = str::from_utf8(&attrib.value).expect("Error decoding an xml attribute");
                            if key == "id" {
                              room_ids.push(value.to_string());
                            }
                            // Debugging
                            //println!("{}={}", key, value);
                          }
                          Err(e) => {
                            println!("{}", e);
                          }
                        }
                      }
                      
                    },
                    //b"tag2" => count += 1,
                    _ => (),
                }
            },
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (),
          }
        }
      }
      None => { }
    }
    return room_ids;
  }
  
  pub fn change_connected_machines(&mut self, delta: isize) {
    self.num_connected_machines = ((self.num_connected_machines as isize) + delta) as u16;
  }
  
}

pub struct GCSBundle {
  pub ptr: Arc<Mutex< GCS >>,
}

impl GCSBundle {
  pub fn new() -> GCSBundle {
    let mut data_dir = unsafe{crate::MAIN_ARGS.clone()}.unwrap().flag_config_dir;
    // data_dir may be null
    let data_dir = GCS::get_data_dir_static(&mut data_dir).into_os_string().into_string().unwrap();
    // Now it will either be specified or generated
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
    
    let known_employees = {
      let mut built_vec: Vec<EmployeeRecord> = vec![];
      let file = fs::File::open(format!("{}/known_employees.csv", data_dir));
      match file {
        Ok(file) => {
          let mut rdr = csv::Reader::from_reader(file);
          for result in rdr.records() {
            match result {
              Ok(record) => {
                built_vec.push(
                  EmployeeRecord::new(
                    record.get(0).unwrap_or("").to_string(),
                    record.get(1).unwrap_or("").to_string(),
                  )
                );
              }
              Err(e) => {
                println!("{}", e);
              }
            }
          }

          
        }
        Err(e) => {
          println!("{}", e);
        }
      }
      
      built_vec
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
        known_employees: known_employees,
      })),
    };
  }
  
  pub fn with_gcs_mut<F, T>(&self, f: F) -> Result<T, PoisonError<MutexGuard<GCS>>> where F: Fn(&mut MutexGuard<GCS>) -> T {
    use std::borrow::BorrowMut;
    match self.ptr.lock() {
      Ok(mut gcs) => {
        let stuff = f(gcs.borrow_mut());
        return Ok(stuff);
      }
      Err(e) => {
        println!("{}", e);
        return Err(e);
      }
    }
  }
  
  pub fn with_gcs<F, T>(&self, f: F) -> Result<T, PoisonError<MutexGuard<GCS>>> where F: Fn(&MutexGuard<GCS>) -> T {
    use std::borrow::Borrow;
    match self.ptr.lock() {
      Ok(gcs) => {
        let stuff = f(gcs.borrow());
        return Ok(stuff);
      }
      Err(e) => {
        println!("{}", e);
        return Err(e);
      }
    }
  }
  
  
}

lazy_static! {
  // This variable stores all global server state data
  pub static ref global_context_singleton: GCSBundle = GCSBundle::new();
}
