
/*
 * This module is responsible for handling individual websocket connections,
 * and has a similar role to the 'routes' module but for websockets.
 */

use ws;
use crate::state::global_context_singleton;
use crate::state::EmployeeBadgeIn;

pub fn handle_incoming(out: &ws::Sender, data: ws::Message) -> Result<(), ws::Error> {
  let mut data_str: String = format!("{}", data);
  
  if data_str == "browser-has-connected" {
    global_context_singleton.with_gcs_mut(|gcs| {
      gcs.change_connected_machines(1);
    }).expect("Could not lock GCS");
    
    global_context_singleton.with_gcs(|gcs| {
      for punched_in_employee_status in &gcs.badged_in_employee_ids {
        if punched_in_employee_status.employee_location.is_none() {
          continue; // ???
        }
        out.send(
          format!("change_map_svg_elm_color('{}', 'green');", punched_in_employee_status.employee_location.clone().expect("Could not get employee location!"))
        ).expect("Could not send change to browser");
      }
    }).expect("Could not lock GCS");
    
    // Tell browser about version
    out.send(
      format!("var m = document.getElementById('versionmessage'); if (m) {{ m.innerHTML = '{}'; }} ", crate::get_version_string() )
    ).expect("Could not send change to browser");
    
    return Ok(());
  }
  
  if data_str.contains("read-id:") {
    let split_idx = data_str.find(":").expect("No ':' in data_str when expected") + 1;
    // data_str is modified and contains first half
    let mut id = data_str.split_off(split_idx); // will contain "id:location" at first
    
    let location_split_idx = id.find(":").expect("No ':' in data_str when expected") + 1;
    let location = id.split_off(location_split_idx);
    
    id.truncate(id.len()-1); // remove a trailing ':' character from id
    
    // tell client to clear their input field
    
    out.send(r#"
document.getElementById('badge_id_input').value = '';
"#).expect("Could not send to browser");
    
    // Tell all browsers to set map location to full
    
    global_context_singleton.with_gcs_mut(|gcs| {
      println!("Someone with ID {}, name {:?} just badged in at {}", id.clone(), gcs.get_employee_name(id.clone()), location );
      
      let emp_id_obj = EmployeeBadgeIn::new(id.clone());
      
      if gcs.badged_in_employee_ids.contains(&emp_id_obj) {
        // Employee is LEAVING work
        let index = gcs.badged_in_employee_ids.iter().position(|r| r == &emp_id_obj).unwrap();
        gcs.badged_in_employee_ids.remove(index);
        gcs.broadcast_to_browsers.bus.broadcast(format!("change_map_svg_elm_color('{}', 'yellow');", location));
        out.send(r#"
document.body.style.background = 'yellow';
setTimeout(function() { document.body.style.background = ''; }, 2 * 1000);
"#).expect("Could not send to browser");
      }
      else {
        // The employee is coming IN to work
        gcs.badged_in_employee_ids.push(EmployeeBadgeIn {
          employee_badge_id: id.clone(),
          employee_location: Some(location.clone()),
        });
        gcs.broadcast_to_browsers.bus.broadcast(format!("change_map_svg_elm_color('{}', 'green');", location));
        out.send(r#"
document.body.style.background = 'green';
setTimeout(function() { document.body.style.background = ''; }, 2 * 1000);
"#).expect("Could not send to browser");
      }
    }).expect("Could not lock GCS");
    
    return Ok(());
  }
  
  println!("From browser: {}", data_str);
  
  // Send to all other nodes
  global_context_singleton.with_gcs_mut(|gcs| {
    // Send a message to everyone listening
    gcs.broadcast_to_browsers.bus.broadcast(data_str.clone());
  }).expect("Could not lock GCS");
  
  //out.send( format!("Server got your ({}) ", data) ).expect("Could not send to browser");
  
  Ok(())
}

