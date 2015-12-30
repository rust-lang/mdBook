$( document ).ready(function() {

    // url
    var url = window.location.pathname;

    // Fix back button cache problem
    window.onunload = function(){};

    // Set theme
    var theme = localStorage.getItem('theme');
    if (theme === null) { theme = 'light'; }

    set_theme(theme);


    // Syntax highlighting Configuration
    hljs.configure({
        tabReplace: '    ', // 4 spaces
        languages: [],      // Languages used for auto-detection
    });

    $('code').each(function(i, block) {
        hljs.highlightBlock(block);
    });

    var KEY_CODES = {
        PREVIOUS_KEY: 37,
        NEXT_KEY: 39
    };

    $(document).on('keydown', function (e) {
        switch (e.keyCode) {
            case KEY_CODES.NEXT_KEY:
                e.preventDefault();
                window.location.href = $('.nav-chapters.next').attr('href');
                break;
            case KEY_CODES.PREVIOUS_KEY:
                e.preventDefault();
                window.location.href = $('.nav-chapters.previous').attr('href');
                break;
        }
    });

    // Interesting DOM Elements
    var html = $("html");
    var sidebar = $("#sidebar");
    var page_wrapper = $("#page-wrapper");
    var content = $("#content");


    // Add anchors for all content headers
    content.find("h1, h2, h3, h4, h5").wrap(function(){
        var wrapper = $("<a class=\"header\">");
        wrapper.attr("name", $(this).text());
        return wrapper;
    });


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


    // Scroll sidebar to current active section
    var activeSection = sidebar.find(".active");
    if(activeSection.length) {
        sidebar.scrollTop(activeSection.offset().top);
    }


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

                set_theme(theme);
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


    // Hide Rust code lines prepended with a specific character
    var hiding_character = "#";

    $("code.language-rust").each(function(i, block){

        // hide lines
        var lines = $(this).html().split("\n");
        var first_non_hidden_line = false;
        var lines_hidden = false;

        for(var n = 0; n < lines.length; n++){
            if($.trim(lines[n])[0] == hiding_character){
                if(first_non_hidden_line){
                    lines[n] = "<span class=\"hidden\">" + "\n" + lines[n].substr(1) + "</span>";
                }
                else {
                    lines[n] = "<span class=\"hidden\">" + lines[n].substr(1) + "\n"  +  "</span>";
                }
                lines_hidden = true;
            }
            else if(first_non_hidden_line) {
                lines[n] = "\n" + lines[n];
            }
            else {
                first_non_hidden_line = true;
            }
        }
        $(this).html(lines.join(""));

        // If no lines were hidden, return
        if(!lines_hidden) { return; }

        // add expand button
        $(this).parent().prepend("<i class=\"fa fa-expand\"></i>");

        $(this).parent().find("i").click(function(e){
            if( $(this).hasClass("fa-expand") ) {
                $(this).removeClass("fa-expand").addClass("fa-compress");
                $(this).parent().find("span.hidden").removeClass("hidden").addClass("unhidden");
            }
            else {
                $(this).removeClass("fa-compress").addClass("fa-expand");
                $(this).parent().find("span.unhidden").removeClass("unhidden").addClass("hidden");
            }
        });
    });


});
