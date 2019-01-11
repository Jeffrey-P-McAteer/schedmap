/*
 * Imports for the entire crate,
 * versions are specified in Cargo.toml
 */

#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
use rocket::config::{Config, Environment};

extern crate multipart;

extern crate ws;

#[macro_use] extern crate serde_derive;
extern crate docopt;
use docopt::Docopt;

#[macro_use] extern crate lazy_static;

extern crate directories;

extern crate quick_xml;

extern crate csv;

use std::thread;

// Holds HTTP route functions responsible for passing data into and out of client apps
mod routes;
mod routes_types; // Aux. logic directly for use in routes

// Holds shared state data; responsible for locking references
// and ensuring state is straightforward to change from `routes`
mod state;
// Handles all incoming data pings
mod websockets;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

// These two vars are only set when deploying (unless you'd like to set them yourself)
const COMPILE_GIT_HASH: Option<&'static str> = option_env!("GIT_HASH");
const COMPILE_DATE: Option<&'static str> = option_env!("COMPILE_DATE");

const COMPILE_LINUX_USER: Option<&'static str> = option_env!("USER");
const COMPILE_WIN_USER: Option<&'static str> = option_env!("USERNAME");

const COMPILE_LINUX_HOST: Option<&'static str> = option_env!("HOST"); // I think this is actually a zsh-ism
const COMPILE_WIN_HOST: Option<&'static str> = option_env!("USERDOMAIN");

const USAGE: &'static str = r#"
The swiss army knife of all scheduling.

Usage:
  schedmap client <client-event>
  schedmap server [--config-dir=<config-dir>] [--app-port=<port>] [--websocket-port=<port>]
  schedmap (-h | --help)
  schedmap --version [--config-dir=<config-dir>] [--app-port=<port>] [--websocket-port=<port>]

Options:
  -h --help                  Show this screen.
  --config-dir=<config-dir>  Persistent config dir for app data.
  --app-port=<port>          Port for web UI [default: 8000].
  --websocket-port=<port>    Port for websocket streams [default: 8001].
  --version                  Show version.
"#;

/*
 * The above string is parsed by Docopt and values are put into this Args structure
 */
#[derive(Debug, Deserialize, Clone)]
struct Args {
    flag_config_dir: Option<String>,
    
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
  
  // This is unsafe because we are mutating a global variable,
  // but we know it will be a safe operation because there are no other
  // threads running and we are not reading any uninitialized data
  unsafe {
    MAIN_ARGS = Some(args.clone());
  }
  
  if args.flag_version {
    println!("schedmap version {}", VERSION);
    let mut gcs = state::global_context_singleton.ptr.lock().unwrap();
    println!("schedmap config_dir = {:?}", gcs.get_data_dir() );
    println!("schedmap get_map_room_ids = {:?}", gcs.get_map_room_ids() );
    println!("schedmap known_employees = {:?}", gcs.known_employees );
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
  
}

/*
 * Transmit a command via a new websocket similar to a browser sending us an event.
 * Primarially used to test capabilities.
 */
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

/*
 * We spawn 2 threads: one to handle incoming websockets and one to handle incoming HTTP traffic.
 */
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
      
      routes::app_home,
        routes::app_home_map,
      
      routes::app_locations,
        routes::app_upload_map,
        routes::app_upload_employees,
      
      routes::app_badge_input,
      
      routes::appvariables_js,
      
    ]).launch();
    
  });
  
  websocket_handle.join().expect("Error when websocket_handle exited");
  webserver_handle.join().expect("Error when webserver_handle exited");
}

pub fn get_version_string() -> String {
  let user = match COMPILE_LINUX_USER {
    Some(user) => user,
    None => {
      match COMPILE_WIN_USER {
        Some(user) => user,
        None => "USERNAME",
      }
    }
  };
  let host = match COMPILE_LINUX_HOST {
    Some(host) => host,
    None => {
      match COMPILE_WIN_HOST {
        Some(host) => host,
        None => "HOSTNAME",
      }
    }
  };
  let git_hash = COMPILE_GIT_HASH.unwrap_or("NONE");
  let compile_date = COMPILE_DATE.unwrap_or("UNK");
  // String should not contain single quotes, it is embedded into JS land
  return format!("Version {} hash {}, compiled by {} on {} at {}", VERSION, git_hash, user, host, compile_date);
}

