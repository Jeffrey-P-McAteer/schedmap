
/*
 * This module is responsible for handling individual websocket connections,
 * and has a similar role to the 'routes' module but for websockets.
 */

use ws;
use crate::state::global_context_singleton;

pub fn handle_incoming(out: &ws::Sender, data: ws::Message) -> Result<(), ws::Error> {
  let mut data_str: String = format!("{}", data);
  
  if data_str == "browser-has-connected" {
    global_context_singleton.change_connected_machines(1);
    return Ok(());
  }
  
  if data_str.contains("read-id:") {
    let split_idx = data_str.find(":").expect("No ':' in data_str when expected") + 1;
    // data_str is modified and contains first half
    let id = data_str.split_off(split_idx);
    
    // TODO lookup ID, determine and transmit location change.
    println!("Someone with ID {} just badged in!", id);
    
    // tell client to clear their input field
    
    out.send(r#"
document.getElementById('badge_id_input').value = '';
document.body.style.background = 'green';
setTimeout(function() { document.body.style.background = ''; }, 2 * 1000);
"#).expect("Could not send to browser");
    
    
    return Ok(());
  }
  
  println!("From browser: {}", data_str);
  
  // Send to all other nodes
  match global_context_singleton.ptr.lock() {
    Ok(mut gcs) => {
      // Send a message to everyone listening
      gcs.broadcast_to_browsers.bus.broadcast(data_str);
    },
    Err(e) => {
      println!("{}", e);
    }
  }
  
  //out.send( format!("Server got your ({}) ", data) ).expect("Could not send to browser");
  
  Ok(())
}

