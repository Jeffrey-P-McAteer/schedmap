
use rocket::response::content::{Html,Css,JavaScript,Content};
use rocket::http::{ContentType,RawStr};

// "crate::" means us
use crate::state::*;

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

#[get("/style.css")]
pub fn style() -> Css<&'static str> {
  Css(include_str!("www/client_style.css"))
}

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

#[get("/debug/toggle/<svg_id>")]
pub fn debug_toggle(gcs_bundle: GCSBundle, svg_id: &RawStr) -> Html<&'static str> {
  match gcs_bundle.ptr.lock() {
    Ok(mut gcs) => {
      // Send a message to everyone listening
      gcs.broadcast_to_browsers.bus.broadcast(svg_id.to_string());
    },
    Err(e) => {
      println!("{}", e);
    }
  }
  Html("Done")
}



#[get("/app_home.html")]
pub fn app_home(_gcs_bundle: GCSBundle) -> Html<&'static str> {
  Html("<script src=\"app.js\"></script><h1>Home Home Home</h1><object id=\"map\" type=\"image/svg+xml\" data=\"app_home/map.svg\"></object><button onclick='do_lobby();'>Do Lobby</button>")
}

#[get("/app_home/map.svg")]
pub fn app_home_map(_gcs_bundle: GCSBundle) -> Content<String> {
  Content(ContentType::new("image", "svg+xml"), include_str!("../test-assets/map.svg").to_string())
}

#[get("/app_locations.html")]
pub fn app_locations(_gcs_bundle: GCSBundle) -> Html<&'static str> {
  Html("<center>Locations</center>")
}


