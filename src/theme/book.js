$( document ).ready(function() {

    // url
    var url = window.location.pathname;

    // Fix back button cache problem
    window.onunload = function(){};

    // Set theme
    var theme = localStorage.getItem('theme');
    if (theme == null) { theme = 'light'; }

    set_theme(theme);


    // Syntax highlighting Configuration
    hljs.configure({
        tabReplace: '    ', // 4 spaces
        languages: [],      // Languages used for auto-detection
    });

    $('code').each(function(i, block) {
        hljs.highlightBlock(block);
    });


    // Interesting DOM Elements
    var html = $("html");
    var sidebar = $("#sidebar");
    var page_wrapper = $("#page-wrapper");


    // Toggle sidebar
    $("#sidebar-toggle").click(function(event){
        if ( html.hasClass("sidebar-hidden") ) {
            html.removeClass("sidebar-hidden").addClass("sidebar-visible");
            localStorage.setItem('sidebar', 'visible');
        } else if ( html.hasClass("sidebar-visible") ) {
            html.removeClass("sidebar-visible").addClass("sidebar-hidden");
            localStorage.setItem('sidebar', 'hidden');
        } else {
            if(sidebar.position().left === 0){
                html.addClass("sidebar-hidden");
                localStorage.setItem('sidebar', 'hidden');
            } else {
                html.addClass("sidebar-visible");
                localStorage.setItem('sidebar', 'visible');
            }
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
                .append($('<div class="theme" id="rust">Rust</div>'))
                .append($('<div class="theme" id="coal">Coal</div>'))
                .append($('<div class="theme" id="navy">Navy</div>'));


            $(this).append(popup);

            $('.theme').click(function(){
                var theme = $(this).attr('id');

                set_theme(theme)
            });
        }

    });

    function set_theme(theme) {
        if (theme == 'coal' || theme == 'navy') {
            $("[href='tomorrow-night.css']").prop('disabled', false);
            $("[href='highlight.css']").prop('disabled', true);
        } else {
            $("[href='tomorrow-night.css']").prop('disabled', true);
            $("[href='highlight.css']").prop('disabled', false);
        }

        localStorage.setItem('theme', theme);

        $('body').removeClass().addClass(theme);
    }
});
