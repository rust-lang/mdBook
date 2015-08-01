$( document ).ready(function() {

    // Interesting DOM Elements
    var sidebar = $("#sidebar");
    var page_wrapper = $("#page-wrapper");

    $("#sidebar-toggle").click(function(event){
        if(sidebar.position().left === 0){
            sidebar.css({left: "-300px"});
            page_wrapper.css({left: "15px"});
        } else {
            sidebar.css({left: "0"});
            page_wrapper.css({left: "315px"});
        }
    });


    // Hide navigation icons when there is no next or previous link
    // JavaScript Solution until I find a way to do this in the template
    $(".nav-chapters").each(function(){
        if(!$(this).attr('href')){
            this.remove();
        }
    });

});
