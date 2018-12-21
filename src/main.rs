#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
use rocket::config::{Config, Environment};

extern crate ws;

#[macro_use] extern crate serde_derive;
extern crate docopt;
use docopt::Docopt;

#[macro_use] extern crate lazy_static;

extern crate directories;

use std::thread;

// Holds HTTP route functions responsible for passing data into and out of client apps
mod routes;
// Holds shared state data; responsible for locking references
// and ensuring state is straightforward to change from `routes`
mod state;
// Handles all incoming data pings
mod websockets;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const USAGE: &'static str = r#"
The swiss army knife of all scheduling.

Usage:
  schedmap client <client-event>
  schedmap server [<config-file>] [--app-port=<port>] [--websocket-port=<port>]
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
    
    cmd_client: bool,
    arg_client_event: Option<String>,
    
    cmd_server: bool,
    flag_app_port: u16,
    flag_websocket_port: u16,
    
    flag_version: bool,
}

static mut MAIN_ARGS: Option<Args> = None;

fn main() {
  let args: Args = Docopt::new(USAGE)
                      .and_then(|d| d.deserialize())
                      .unwrap_or_else(|e| e.exit());
  
  unsafe {
    MAIN_ARGS = Some(args.clone());
  }
  
  if args.flag_version {
    println!("schedmap version {}", VERSION);
    return;
  }
  
  //println!("{:#?}", args);
  
  if args.cmd_client {
    do_client_websocket(args);
    return;
  }
  
  if args.cmd_server {
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

fn do_client_websocket(args: Args) {
  let event_str = args.arg_client_event.unwrap_or("ERR".to_string());
  ws::connect(format!("ws://127.0.0.1:{}", args.flag_websocket_port), |out| {
      out.send(event_str.as_str()).unwrap();
      move |msg| {
        println!("{}", msg);
        out.close(ws::CloseCode::Normal)
      }
  }).unwrap();
}

fn run_server(args: Args) {
  let ws_args = args.clone();
  let websocket_handle = thread::spawn(move || {
    ws::listen(format!("0.0.0.0:{}", ws_args.flag_websocket_port), |out| {
      // Spawn thread to listen for broadcasts
      match state::global_context_singleton.ptr.lock() {
        Ok(mut gcs) => {
          // Get a new receiving channel
          let mut our_broadcast_rx = gcs.broadcast_to_browsers.bus.add_rx();
          let bcast_out = out.clone();
          thread::spawn(move || {
            loop {
              match our_broadcast_rx.recv() { // blocks
                Ok(data) => {
                  bcast_out.send(data).expect("Could not transmit broadcast to websocket");
                }
                Err(e) => {
                  println!("{}", e);
                }
              }
            }
          });
          
        },
        Err(e) => {
          println!("{}", e);
        }
      }
      // Return on message handler
      move |msg| {
        websockets::handle_incoming(&out, msg)
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
      routes::instascan_min_js,
      
      routes::debug,
        routes::debug_toggle,
      
      routes::app_home,
        routes::app_home_map,
      
      routes::app_locations,
      
      routes::app_badge_input,
      
      routes::appvariables_js,
      
    ]).launch();
    
  });
  
  websocket_handle.join().expect("Error when websocket_handle exited");
  webserver_handle.join().expect("Error when webserver_handle exited");
}

