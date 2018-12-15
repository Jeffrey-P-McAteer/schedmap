
/* Mostly copy-paste from a blog post on vanilla JS tabs. There's never a "nice" or "elegant" html tab implementation. */
function tabs(x)
{
  var lis=document.getElementById("sidebarTabs").childNodes; //gets all the LI from the UL

  for(i=0;i<lis.length;i++)
  {
    lis[i].className=""; //removes the classname from all the LI
  }
  
  x.className="selected"; //the clicked tab gets the classname selected
  var res = document.getElementById("tabContent");  //the resource for the main tabContent
  var tab = x.id;
  var content_elm = document.getElementById(tab+"Content");
  if (!content_elm) {
    console.log("Someone coded a '"+tab+"' without a corresponding '"+tab+"Content' div!");
  }
  res.innerHTML = content_elm.innerHTML;
}
