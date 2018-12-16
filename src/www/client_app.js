
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
