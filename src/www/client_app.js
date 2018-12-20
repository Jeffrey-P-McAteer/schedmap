
/* Mostly copy-paste from a blog post on vanilla JS tabs. There's never a "nice" or "elegant" html tab implementation. */
function tabs(x)
{
  var lis=document.getElementById("tabList").childNodes; //gets all the LI from the UL

  for(i=0;i<lis.length;i++)
  {
    var lis_links=lis[i].childNodes;
    for(j=0;j<lis_links.length;j++) {
      lis_links[j].className=""; //removes the classname from all the LI
    }
  }
  
  x.className="selected"; //the clicked tab gets the classname selected
  
}

function do_lobby() {
  if (Math.random() < 0.5) {
    change_map_svg_elm_color("Lobby", "red");
  }
  else {
    change_map_svg_elm_color("Lobby", "blue");
  }
}

function change_map_svg_elm_color(elm_id, color) {
  var svg_elm = document.getElementById("map").getSVGDocument().getElementById(elm_id);
  svg_elm.setAttribute("fill", color);
  svg_elm.style.fill = color;
}

function setup_websocket() {
  var web_socket = new WebSocket("ws://" + location.hostname + ":"+window.websocket_port + "/");
  
  web_socket.onopen = function (evt) {
    web_socket.send("Hello from a browser!");
  };
  
  web_socket.onmessage = function (evt) {
    console.log("web_socket got: "+evt.data);
    change_map_svg_elm_color("Lobby", evt.data);
  };
  
  web_socket.onclose = function (evt) {
    console.log("Websocket closed, reconnecting in 5s...");
    setTimeout(setup_websocket, 5 * 1000);
  };
}

window.addEventListener("load", function() {
  if (window.location.pathname.includes("/app_home.html")) {
    // Spawn websocket handler
    setup_websocket();
  }
  
});
