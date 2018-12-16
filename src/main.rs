#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
use rocket::config::{Config, Environment};

extern crate ws;

#[macro_use] extern crate serde_derive;
extern crate docopt;
use docopt::Docopt;

#[macro_use] extern crate lazy_static;

use std::thread;

// Holds HTTP route functions responsible for passing data into and out of client apps
mod routes;
// Holds shared state data; responsible for locking references
// and ensuring state is straightforward to change from `routes`
mod state;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const USAGE: &'static str = r#"
The swiss army knife of all scheduling.

Usage:
  schedmap <config_file>
  schedmap --server [<config_file>] [--app-port=<port>] [--websocket-port=<port>]
  schedmap (-h | --help)
  schedmap --version

Options:
  -h --help                Show this screen.
  --app-port=<port>        Port for web UI [default: 8000].
  --websocket-port=<port>  Port for websocket streams [default: 8001].
  --version                Show version.
"#;

#[derive(Debug, Deserialize, Clone)]
struct Args {
    arg_config_file: Option<String>,
    
    flag_server: bool,
    flag_app_port: u16,
    flag_websocket_port: u16,
    
    flag_version: bool,
}

fn main() {
  let args: Args = Docopt::new(USAGE)
                      .and_then(|d| d.deserialize())
                      .unwrap_or_else(|e| e.exit());
  if args.flag_version {
    println!("schedmap version {}", VERSION);
    return;
  }
  
  if args.flag_server {
    run_server(args);
    return;
  }
  
  match args.arg_config_file {
    Some(arg_config_file) => {
      println!("Reading business config from file {}...", arg_config_file);
      // Meh todo what will this become?
    }
    None => {
      println!("{}", USAGE);
      return;
    }
  }
  
}

fn run_server(args: Args) {
  let ws_args = args.clone();
  let websocket_handle = thread::spawn(move || {
    ws::listen(format!("0.0.0.0:{}", ws_args.flag_websocket_port), |out| {
      move |msg| {
        out.send(msg)
      }
    }).expect("Error on websocket server");
  });
  let web_args = args.clone();
  let webserver_handle = thread::spawn(move || {
    let config = Config::build(Environment::Staging)
                .address("0.0.0.0")
                .port(web_args.flag_app_port)
                .finalize().expect("Could not configure rocket");

    let app = rocket::custom(config);
    
    app.mount("/", routes![
      routes::index,
      routes::style,
      routes::app_js,
      routes::debug,
      
    ]).launch();
    
  });
  
  websocket_handle.join().expect("Error when websocket_handle exited");
  webserver_handle.join().expect("Error when webserver_handle exited");
}

