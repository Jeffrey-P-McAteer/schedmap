
/*
 * This module handles all our HTTP routes (the endpoints at "/index.html" etc.)
 * Many of our static assets in src/www/ are simply included as strings at build time,
 * using the  include_str!() macro, which takes the contents of a file
 * and turns it into a "String" expression of type &'static str.
 */

use rocket::response::content::{Html,Css,JavaScript,Content};
use rocket::http::{ContentType,RawStr};
//use rocket::Data;
use rocket::*;

// "crate::" means us
use crate::state::*;

use crate::routes_types::*;

use std::io::Read;

#[get("/")]
pub fn index() -> Html<&'static str> {
  global_context_singleton.with_gcs_mut(|gcs| {
    // Increment, overwrapping via modulo operator
    gcs.num_visitors = ((gcs.num_visitors as u64 + 1) % 255) as u8;
  }).expect("Could not lock GCS");
  Html(include_str!("www/client_app.html"))
}

#[get("/app.js")]
pub fn app_js() -> JavaScript<&'static str> {
  JavaScript(include_str!("www/client_app.js"))
}

/* This is used to tell the client about global application configuration variables,
 * such as the websocket port.
 */
#[get("/appvariables.js")]
pub fn appvariables_js() -> JavaScript<String> {
  let maybe_js = global_context_singleton.with_gcs_mut(|gcs| {
      let _x = gcs.get_data_dir();
      let websocket_port = unsafe { crate::MAIN_ARGS.clone() }.unwrap().flag_websocket_port;
      
      JavaScript(format!(r#"
window.websocket_port = {};
window.map_room_ids = {:?};
"#,
  websocket_port, gcs.get_map_room_ids()
  ) )

  });
  return maybe_js.unwrap_or(JavaScript(format!("console.log('some error happened!');")));
}

// 3rd-party library to capture QR codes via a webcam.
#[get("/instascan.min.js")]
pub fn instascan_min_js() -> JavaScript<&'static str> {
  JavaScript(include_str!("www/instascan.min.js"))
}

#[get("/style.css")]
pub fn style() -> Css<&'static str> {
  Css(include_str!("www/client_style.css"))
}

// Dumps pretty-formatted global state data for the server
#[get("/debug")]
pub fn debug() -> String {
  match global_context_singleton.ptr.lock() {
    Ok(mut gcs) => {
      let _x = gcs.get_data_dir();
      return format!("{:#?}", *gcs);
    },
    Err(e) => {
      return format!("{}", e);
    }
  }
}

#[get("/app_home.html")]
pub fn app_home() -> Html<&'static str> {
  Html(include_str!("www/app_home.html"))
}

#[get("/app_home/map.svg")]
pub fn app_home_map() -> Content<String> {
  match global_context_singleton.ptr.lock() {
    Ok(gcs) => {
      match &gcs.svg_map {
        Some(map_str) => {
          return Content(ContentType::new("image", "svg+xml"), map_str.to_string().clone());
        }
        None => {
          // Will fall through to last return
        }
      }
    },
    Err(e) => {
      println!("{}", e);
    }
  }
  // Default in case of error or empty SVG
  return Content(ContentType::new("image", "svg+xml"), "<!-- Error no map! -->".to_string())
}

#[get("/app_badge_input.html")]
pub fn app_badge_input() -> Html<String> {
  Html(include_str!("www/app_badge_input.html").to_string())
}

#[get("/app_locations.html")]
pub fn app_locations() -> Html<String> {
  Html(include_str!("www/app_locations.html").to_string())
}

#[post("/upload_map", data = "<data>")]
pub fn app_upload_map(data: Data, multip_boundry: MultiPartBoundry) -> Html<&'static str> {
  use std::fs;
  
  match global_context_singleton.ptr.lock() {
    Ok(mut gcs) => {
      //let mut data_buffer: Vec<u8> = vec![];
      //data.stream_to(&mut data_buffer).expect("Failed to write SVG to memory buffer");
      
      let mut mp = multipart::server::Multipart::with_body(data.open(), multip_boundry.to_string() );
      let entries = mp.save().temp().into_entries().expect("Could not unwrap into_entries");
      //println!("{:#?}", entries);
      
      let svg_map_contents = &entries.fields.get(&"data".to_string()).unwrap()[0].data;
      let mut svg_reader = svg_map_contents.readable().unwrap();
      
      let mut new_svg_map_str: String = String::new();
      svg_reader.read_to_string(&mut new_svg_map_str).unwrap();
      
      gcs.svg_map = Some(new_svg_map_str.clone());
      
      // We also need to write to gcs.get_data_dir() / "svg_map.svg"
      
      let mut svg_map_file = gcs.get_data_dir();
      svg_map_file.push("svg_map.svg");
      let svg_map_file = svg_map_file.as_path();
      fs::write(svg_map_file, new_svg_map_str.clone() ).expect("Unable to write file");
      
      println!("Saved new SVG map to {:?}", svg_map_file);
      
    },
    Err(e) => {
      println!("{}", e);
    }
  }
  Html(r#"
<p>Map Uploaded!</p>
"#)
}

#[post("/upload_employees", data = "<data>")]
pub fn app_upload_employees(data: Data, multip_boundry: MultiPartBoundry) -> Html<&'static str> {
  use std::fs;
  
  match global_context_singleton.ptr.lock() {
    Ok(mut gcs) => {
      //let mut data_buffer: Vec<u8> = vec![];
      //data.stream_to(&mut data_buffer).expect("Failed to write SVG to memory buffer");
      
      let mut mp = multipart::server::Multipart::with_body(data.open(), multip_boundry.to_string() );
      let entries = mp.save().temp().into_entries().expect("Could not unwrap into_entries");
      //println!("{:#?}", entries);
      
      let employee_contents = &entries.fields.get(&"data".to_string()).unwrap()[0].data;
      let mut employee_reader = employee_contents.readable().unwrap();
      
      let mut new_employee_str: String = String::new();
      employee_reader.read_to_string(&mut new_employee_str).unwrap();
      
      let mut employees_file = gcs.get_data_dir();
      employees_file.push("known_employees.csv");
      let employees_file = employees_file.as_path();
      fs::write(employees_file, new_employee_str.clone() ).expect("Unable to write file");
      
      println!("Saved new Employees CSV to {:?}", employees_file);
      
      let config_dir_s = gcs.get_data_dir().as_path().to_string_lossy().to_string();
      gcs.known_employees = GCSBundle::read_employee_records( &config_dir_s );
      
    },
    Err(e) => {
      println!("{}", e);
    }
  }
  Html(r#"
<p>Employees Changed!</p>
"#)
}


