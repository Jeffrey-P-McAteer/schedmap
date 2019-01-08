
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
pub fn index(gcs_bundle: GCSBundle) -> Html<&'static str> {
  match gcs_bundle.ptr.lock() {
    Ok(mut gcs) => {
      // Increment, overwrapping via modulo operator
      gcs.num_visitors = ((gcs.num_visitors as u64 + 1) % 255) as u8;
    },
    Err(e) => {
      println!("{}", e);
    }
  }
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
pub fn appvariables_js(gcs_bundle: GCSBundle) -> JavaScript<String> {
  match gcs_bundle.ptr.lock() {
    Ok(mut gcs) => {
      let _x = gcs.get_data_dir();
      let websocket_port = unsafe { crate::MAIN_ARGS.clone() }.unwrap().flag_websocket_port;
      return JavaScript(format!(r#"
window.websocket_port = {};
"#, websocket_port) );
    },
    Err(e) => {
      return JavaScript(format!("console.log('{:?}');", e));
    }
  }
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
pub fn debug(gcs_bundle: GCSBundle) -> String {
  match gcs_bundle.ptr.lock() {
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
pub fn app_home(_gcs_bundle: GCSBundle) -> Html<&'static str> {
  Html(r#"
<script src="appvariables.js"></script>
<script src="app.js"></script>
<object id="map" type="image/svg+xml" data="app_home/map.svg"></object>
"#)
}

#[get("/app_home/map.svg")]
pub fn app_home_map(gcs_bundle: GCSBundle) -> Content<String> {
  match gcs_bundle.ptr.lock() {
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
pub fn app_badge_input(_gcs_bundle: GCSBundle) -> Html<String> {
  Html(include_str!("www/app_badge_input.html").to_string())
}

#[get("/app_locations.html")]
pub fn app_locations(_gcs_bundle: GCSBundle) -> Html<String> {
  Html(include_str!("www/app_locations.html").to_string())
}

#[post("/upload_map", data = "<data>")]
pub fn app_upload_map(gcs_bundle: GCSBundle, data: Data, multip_boundry: MultiPartBoundry) -> Html<&'static str> {
  match gcs_bundle.ptr.lock() {
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
      
      gcs.svg_map = Some(new_svg_map_str);
      
    },
    Err(e) => {
      println!("{}", e);
    }
  }
  Html(r#"
<p>Thanks!</p>
"#)
}


