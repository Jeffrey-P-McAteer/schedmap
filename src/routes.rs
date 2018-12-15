
use rocket::response::content::{Html,Css,JavaScript};

#[get("/")]
pub fn index() -> Html<&'static str> {
  Html(include_str!("client_app.html"))
}

#[get("/app.js")]
pub fn app_js() -> JavaScript<&'static str> {
  JavaScript(include_str!("client_app.js"))
}

#[get("/style.css")]
pub fn style() -> Css<&'static str> {
  Css(include_str!("client_style.css"))
}





