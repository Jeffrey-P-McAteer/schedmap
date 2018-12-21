
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
  Html("<script src=\"appvariables.js\"></script><script src=\"app.js\"></script><h1>Home Home Home</h1><object id=\"map\" type=\"image/svg+xml\" data=\"app_home/map.svg\"></object><button onclick='do_lobby();'>Do Lobby</button>")
}

#[get("/app_home/map.svg")]
pub fn app_home_map(_gcs_bundle: GCSBundle) -> Content<String> {
  Content(ContentType::new("image", "svg+xml"), include_str!("../test-assets/map.svg").to_string())
}

#[get("/app_locations.html")]
pub fn app_locations(_gcs_bundle: GCSBundle) -> Html<&'static str> {
  Html("<center>Locations</center>")
}

#[get("/app_badge_input.html")]
pub fn app_badge_input(_gcs_bundle: GCSBundle) -> Html<&'static str> {
  Html(r#"
<script type="text/javascript" src="instascan.min.js"></script>
<script src="appvariables.js">
</script><script src="app.js"></script>
<div style="position: relative; top:0; right:0; left:0; width:450px; margin:auto; padding:5px; background-color: #e0e0e0;">
<h3>Badge ID Input</h3>
<input id="badge_id_input" type="text">
<br><br>
<video id="preview" style="width:100%;"></video>
<script type="text/javascript">
  let scanner = new Instascan.Scanner({ video: document.getElementById('preview') });
  scanner.addListener('scan', function (content) {
    document.getElementById('badge_id_input').value = content;
    // Call the onchange event with a delay so the value can be flashed on the screen
    setTimeout(document.getElementById('badge_id_input').onchange, 750);
  });
  Instascan.Camera.getCameras().then(function (cameras) {
    if (cameras.length > 0) {
      scanner.start(cameras[0]);
    } else {
      console.error('No cameras found.');
    }
  }).catch(function (e) {
    console.error(e);
  });
</script>

</div>
"#)
}


