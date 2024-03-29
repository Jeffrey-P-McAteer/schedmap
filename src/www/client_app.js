
/* Mostly copy-paste from a blog post on vanilla JS tabs. There's never a "nice" or "elegant" html tab implementation. */
function tabs(x) {
  if (x == null) {
    return;
  }
  var lis=document.getElementById("tabList").childNodes; //gets all the LI from the UL
  for(var i=0; i<lis.length; i++) {
    var lis_links = lis[i].childNodes;
    for(var j=0; j<lis_links.length ;j++) {
      lis_links[j].className = ""; //removes the classname from all the LI
    }
  }
  x.className="selected"; //the clicked tab gets the classname selected
}

function constantly_re_focus_badge_id_input() {
  // Listen on change
  window.last_id_in_change_ms = Date.now();
  
  var location_in_elm = document.getElementById('location_input');
  var id_in_elm = document.getElementById('badge_id_input');
  
  var last_location = window.localStorage.getItem("location") || "";
  
  // Populate location using window.map_room_ids
  for (var i=0; i<window.map_room_ids.length; i++) {
    var room_id = window.map_room_ids[i];
    var opt = document.createElement("option");
    opt.value= room_id;
    opt.innerHTML = room_id;
    if (room_id.trim() == last_location.trim()) {
      opt.selected = 'selected';
    }
    
    location_in_elm.appendChild(opt);
  }
  
  location_in_elm.onchange = function() {
    window.localStorage.setItem("location", location_in_elm.options[location_in_elm.selectedIndex].value );
  };
  
  id_in_elm.onchange = function() {
    var now_ms = Date.now();
    var delta_ms = now_ms - window.last_id_in_change_ms;
    
    if (delta_ms < 2000) {
      // Change happened very soon, wait until input complete, check every quarter second
      window.id_in_timer = setInterval(id_in_elm.onchange, 250);
      
    }
    else if (id_in_elm.value.length > 0) {
      // Change was 2 seconds ago, assume input is finished being typed in
      window.last_id_in_change_ms = now_ms;
      clearInterval(window.id_in_timer);
      
      var location = location_in_elm.value;
      var value = id_in_elm.value;
      window.app_web_socket.send("read-id:"+value+":"+location);
      
      // Save location for next time 
      window.localStorage.setItem("location", location);
      
    }
  };
  
  
  // re-focus every 2s
  setInterval(function() {
    if (document.activeElement === location_in_elm) {
      return;
    }
    id_in_elm.focus();
  }, 2000);
}

function change_map_svg_elm_color(elm_id, color) {
  var svg_elm = document.getElementById("map").getSVGDocument().getElementById(elm_id);
  svg_elm.setAttribute("fill", color);
  svg_elm.style.fill = color;
}

function set_map_elm_person_name(elm_id, employee_name) {
  if (window.person_name_elements == null) {
    window.person_name_elements = {};
  }
  
  var map_elm = document.getElementById("map");
  var svg_elm = document.getElementById("map").getSVGDocument().getElementById(elm_id);
  
  var newText = document.createElementNS("http://www.w3.org/2000/svg", "text");
  newText.setAttributeNS(null, "x", Number(svg_elm.getAttribute("x")) + 2 );
  newText.setAttributeNS(null, "y", Number(svg_elm.getAttribute("y")) + Number(svg_elm.getAttribute("width")) - 20);
  newText.setAttributeNS(null, "font-size", "6");
  
  var textNode = document.createTextNode( employee_name /* displayed text*/ );
  newText.appendChild(textNode);
  
  if (window.person_name_elements[elm_id] != null) {
    svg_elm.parentNode.removeChild(window.person_name_elements[elm_id]);
  }
  
  svg_elm.parentNode.appendChild(newText);
  window.person_name_elements[elm_id] = newText;
  
}

function setup_websocket() {
  window.app_web_socket = new WebSocket("ws://" + location.hostname + ":"+window.websocket_port + "/");
  
  window.app_web_socket.onopen = function (evt) {
    window.app_web_socket.send("browser-has-connected");
  };
  
  window.app_web_socket.onmessage = function (evt) {
    console.log("window.app_web_socket got: "+evt.data);
    // Try executing the payload as JS code
    eval( evt.data );
  };
  
  window.app_web_socket.onclose = function (evt) {
    console.log("Websocket closed, reconnecting in 5s...");
    setTimeout(setup_websocket, 5 * 1000);
  };
}

window.addEventListener("load", function() {
  setup_websocket();
  
  if (window.location.pathname.includes("/app_badge_input.html")) {
    constantly_re_focus_badge_id_input();
    
  }
  
});
