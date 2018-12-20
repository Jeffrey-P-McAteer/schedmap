
use ws;
use crate::state::global_context_singleton;

// If return is Some("data") we transmit it back to the browser in main.rs
pub fn handle_incoming(out: &ws::Sender, data: ws::Message) -> Result<(), ws::Error> {
  let data_str: String = format!("{}", data);
  
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
  
  out.send( format!("Server got your ({}) ", data) ).expect("Could not send to browser");
  
  Ok(())
}

