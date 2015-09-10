$( document ).ready(function() {

    // url
    var url = window.location.pathname;

    // Syntax highlighting Configuration
    hljs.configure({
        tabReplace: '    ', // 4 spaces
        languages: [],      // Languages used for auto-detection
    });

    $('code').each(function(i, block) {
        hljs.highlightBlock(block);
    });


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

    // Print button
    $("#print-button").click(function(){
        var printWindow = window.open("print.html");
    });

    if( url.substring(url.lastIndexOf('/')+1) == "print.html" ) {
        window.print();
    }

    // Theme button
    $("#theme-toggle").click(function(){
        if($('.theme-popup').length) {
            $('.theme-popup').remove();
        } else {
            var popup = $('<div class="theme-popup"></div>')
                .append($('<div class="theme" id="light">Light (default)<div>'))
                .append($('<div class="theme" id="dark">Dark</div>'));


            $(this).append(popup);

            $('.theme').click(function(){
                var theme = $(this).attr('id');
                $('body').removeClass().addClass(theme);
            });
        }

    });

});
