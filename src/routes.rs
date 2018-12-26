
use rocket::response::content::{Html,Css,JavaScript,Content};
use rocket::http::{ContentType,RawStr};
//use rocket::Data;
use rocket::request::FromRequest;
use rocket::*;

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

#[get("/instascan.min.js")]
pub fn instascan_min_js() -> JavaScript<&'static str> {
  JavaScript(include_str!("www/instascan.min.js"))
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
  Html(r#"
<script src="appvariables.js"></script>
<script src="app.js"></script>
<h1>Home Home Home</h1>
<object id="map" type="image/svg+xml" data="app_home/map.svg"></object>
<button onclick='do_lobby();'>Do Lobby</button>
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
  return Content(ContentType::new("image", "svg+xml"), "<!-- We got nuthin sorry -->".to_string())
}

#[get("/app_badge_input.html")]
pub fn app_badge_input(_gcs_bundle: GCSBundle) -> Html<String> {
  Html(include_str!("www/app_badge_input.html").to_string())
}

#[get("/app_locations.html")]
pub fn app_locations(_gcs_bundle: GCSBundle) -> Html<&'static str> {
  Html(r#"
<script type="text/javascript" src="instascan.min.js"></script>
<script src="appvariables.js">
</script><script src="app.js"></script>
<form action="upload_map" method="POST" enctype="multipart/form-data">
<input type="file" id="data" name="data" accept="image/svg+xml">
<input type="submit" value="Change Map">
</form>
"#)
}

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

#[post("/upload_map", data = "<data>")]
pub fn app_upload_map(gcs_bundle: GCSBundle, data: Data, multip_boundry: MultiPartBoundry) -> Html<&'static str> {
  match gcs_bundle.ptr.lock() {
    Ok(mut gcs) => {
      //let mut data_buffer: Vec<u8> = vec![];
      //data.stream_to(&mut data_buffer).expect("Failed to write SVG to memory buffer");
      
      let mut mp = multipart::server::Multipart::with_body(data.open(), multip_boundry.to_string() );
      let mut entries = mp.save().temp().into_entries().expect("Could not unwrap into_entries");
      println!("{:?}", entries);
      println!("entries.fields_count() = {}", entries.fields_count());
      println!("entries.recount_fields() = {}", entries.recount_fields());
      
      //let svg_as_string = String::from_utf8(data_buffer).unwrap_or("Some Error, IDK. We didn't crash I guess.".to_string());
      //gcs.svg_map = Some(svg_as_string);
      
    },
    Err(e) => {
      println!("{}", e);
    }
  }
  Html(r#"
<p>Thanks!</p>
"#)
}


