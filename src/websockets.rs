
use ws;

// If return is Some("data") we transmit it back to the browser in main.rs
pub fn handle_incoming(out: &ws::Sender, data: ws::Message) -> Result<(), ws::Error> {
  println!("From browser: {}", data);
  
  out.send( format!("Server got your ({}) ", data) ).expect("Could not send to browser");
  
  
  Ok(())
}

