$( document ).ready(function() {

    // Search functionality
    //
    // Usage: call init() on startup. You can use hasFocus() to disable prevent keyhandling
    // while the user is typing his search.
    var search = {
        searchbar : $('#searchbar'),
        searchbar_outer : $('#searchbar-outer'),
        searchresults : $('#searchresults'),
        searchresults_outer : $("#searchresults-outer"),
        searchresults_header : $("#searchresults-header"),
        searchicon : $("#search-icon"),
        content : $('#content'),

        searchindex : null,
        searchoptions : {
            bool: "AND",
            expand: true,
            fields: {
                title: {boost: 1},
                body: {boost: 1},
                breadcrumbs: {boost: 0}
            }
        },
        mark_exclude : [], // ['.hljs']
        current_searchterm : "",
        teaser_size_half : 80,
        resultcount_limit : 30,
        SEARCH_PARAM : 'search',
        MARK_PARAM : 'highlight',

        SEARCH_HOTKEY_KEYCODE: 83,
        ESCAPE_KEYCODE: 27,

        formatSearchMetric : function(count, searchterm) {
            if (count == 1) {
                return count + " search result for '" + searchterm + "':";
            } else if (count == 0) {
                return "No search results for '" + searchterm + "'.";
            } else {
                return count + " search results for '" + searchterm + "':";
            }
        }
        ,
        create_test_searchindex : function () {
            var searchindex = elasticlunr(function () {
                this.addField('body');
                this.addField('title');
                this.addField('breadcrumbs')
                this.setRef('id');
            });
            var base_breadcrumbs = "";
            var active_chapter = $('.sidebar ul a.active');
            base_breadcrumbs = active_chapter.text().split('. ', 2)[1]; // demo
            while (true) {
                var parent_ul = active_chapter.parents('ul');
                if (parent_ul.length == 0) break;
                var parent_li = parent_ul.parents('li');
                if (parent_li.length == 0) break;
                var pre_li = parent_li.prev('li');
                if (pre_li.length == 0) break;
                base_breadcrumbs = pre_li.text().split('. ', 2)[1] + ' » ' + base_breadcrumbs;
                active_chapter = pre_li;
            }
            var paragraphs = this.content.children();
            var curr_title = "";
            var curr_body = "";
            var curr_ref = "";
            var push = function(ref) {
                if ((curr_title.length > 0 || curr_body.length > 0) && curr_ref.length > 0) {
                    var doc = {
                        "id": curr_ref,
                        "body": curr_body,
                        "title": curr_title,
                        "breadcrumbs": base_breadcrumbs //"Header1 » Header2"
                    }
                    searchindex.addDoc(doc);
                }
                curr_body = "";
                curr_title = "";
                curr_ref = "";
            };
            paragraphs.each(function(index, element) {
                // todo uppercase
                var el = $(element);
                if (el.prop('nodeName').toUpperCase() == "A") {
                    // new header, push old paragraph to index
                    push(index);
                    curr_title = el.text();
                    curr_ref = el.attr('href');
                } else {
                    curr_body += " \n " + el.text();
                }
                // last paragraph
                if (index == paragraphs.length - 1) {
                    push(index);
                }
            });
            this.searchindex = searchindex;
        }
        ,
        parseURL : function (url) {
            var a =  document.createElement('a');
            a.href = url;
            return {
                source: url,
                protocol: a.protocol.replace(':',''),
                host: a.hostname,
                port: a.port,
                params: (function(){
                    var ret = {};
                    var seg = a.search.replace(/^\?/,'').split('&');
                    var len = seg.length, i = 0, s;
                    for (;i<len;i++) {
                        if (!seg[i]) { continue; }
                        s = seg[i].split('=');
                        ret[s[0]] = s[1];
                    }
                    return ret;
                })(),
                file: (a.pathname.match(/\/([^/?#]+)$/i) || [,''])[1],
                hash: a.hash.replace('#',''),
                path: a.pathname.replace(/^([^/])/,'/$1')
            };
        }
        ,
        renderURL : function (urlobject) {
            var url = urlobject.protocol + "://" + urlobject.host;
            if (urlobject.port != "") {
                url += ":" + urlobject.port;
            }
            url += urlobject.path;
            var joiner = "?";
            for(var prop in urlobject.params) {
                if(urlobject.params.hasOwnProperty(prop)) {
                    url += joiner + prop + "=" + urlobject.params[prop];
                    joiner = "&";
                }
            }
            if (urlobject.hash != "") {
                url += "#" + urlobject.hash;
            }
            return url;
        }
        ,
        escapeHTML: (function() {
            var MAP = {
                '&': '&amp;',
                '<': '&lt;',
                '>': '&gt;',
                '"': '&#34;',
                "'": '&#39;'
            };
            var repl = function(c) { return MAP[c]; };
            return function(s) {
                return s.replace(/[&<>'"]/g, repl);
            };
        })()
        ,
        formatSearchResult : function (result, searchterms) {
            // Show text around first occurrence of first search term.
            var firstoccurence = result.doc.body.search(searchterms[0]);
            var teaser = "";
            if (firstoccurence != -1) {
                var teaserstartindex = firstoccurence - this.teaser_size_half;
                var nextwordindex = result.doc.body.indexOf(" ", teaserstartindex);
                if (nextwordindex != -1) {
                    teaserstartindex = nextwordindex;
                }
                var teaserendindex = firstoccurence + this.teaser_size_half;
                nextwordindex = result.doc.body.indexOf(" ", teaserendindex);
                if (nextwordindex != -1) {
                    teaserendindex = nextwordindex;
                }
                teaser = (teaserstartindex > 0) ? "... " : "";
                teaser += result.doc.body.substring(teaserstartindex, teaserendindex) + " ...";
            } else {
                teaser = result.doc.body.substr(0, this.teaser_size_half * 2) + " ...";
            }

            // The ?MARK_PARAM= parameter belongs inbetween the page and the #heading-anchor
            var url = result.ref.split("#");
            if (url.length == 1) {
                url.push("");
            }

            return $('<li><a href="'
                    + url[0] + '?' + this.MARK_PARAM + '=' + searchterms + '#' + url[1]
                    + '">' + result.doc.breadcrumbs + '</a>' // doc.title
                    + '<span class="breadcrumbs">' + '</span>'
                    + '<span class="teaser">' + this.escapeHTML(teaser) + '</span>'
                    + '</li>');
        }
        ,
        doSearch : function (searchterm) {

            // Don't search the same twice
            if (this.current_searchterm == searchterm) { return; }
            else { this.current_searchterm = searchterm; }

            if (this.searchindex == null) { return; }

            // Do the actual search
            var results = this.searchindex.search(searchterm, this.searchoptions);
            var resultcount = (results.length > this.resultcount_limit)
                ? this.resultcount_limit : results.length;

            // Display search metrics
            this.searchresults_header.text(this.formatSearchMetric(resultcount, searchterm));

            // Clear and insert results
            var searchterms  = searchterm.split(' ');
            this.searchresults.empty();
            for(var i = 0; i < resultcount ; i++){
                this.searchresults.append(this.formatSearchResult(results[i], searchterms));
            }

            // Display and scroll to results
            this.searchresults_outer.slideDown();
            // this.searchicon.scrollTop(0);
        }
        ,
        doSearchOrMarkFromUrl : function () {
            // Check current URL for search request
            var url = this.parseURL(window.location.href);
            if (url.params.hasOwnProperty(this.SEARCH_PARAM)
                && url.params[this.SEARCH_PARAM] != "") {
                this.searchbar_outer.slideDown();
                this.searchbar[0].value = decodeURIComponent(
                    (url.params[this.SEARCH_PARAM]+'').replace(/\+/g, '%20'));
                this.searchbarKeyUpHandler();
            } else {
                this.searchbar_outer.slideUp();
            }

            if (url.params.hasOwnProperty(this.MARK_PARAM)) {
                var words = url.params[this.MARK_PARAM].split(' ');
                var header = $('#' + url.hash);
                this.content.mark(words, {
                    exclude : this.mark_exclude
                });
            }
        }
        ,
        init : function () {
            var this_ = this;
            window.md = this;

            // For testing purposes: Index current page
            //this.create_test_searchindex();

            $.getJSON("searchindex.json", function(json) {
                //this_.searchindex = elasticlunr.Index.load(json);

                // TODO: Workaround: reindex everything
                var searchindex = elasticlunr(function () {
                    this.addField('body');
                    this.addField('title');
                    this.addField('breadcrumbs')
                    this.setRef('id');
                });
                window.mjs = json;
                var docs = json.documentStore.docs;
                for (var key in docs) {
                    searchindex.addDoc(docs[key]);
                }
                this_.searchindex = searchindex;


                // Set up events
                this_.searchicon.click( function(e) { this_.searchIconClickHandler(); } );
                this_.searchbar.on('keyup', function(e) { this_.searchbarKeyUpHandler(); } );
                $(document).on('keydown', function (e) { this_.globalKeyHandler(e); });
                // If the user uses the browser buttons, do the same as if a reload happened
                window.onpopstate = function(e) { this_.doSearchOrMarkFromUrl(); };

                // If reloaded, do the search or mark again, depending on the current url parameters
                this_.doSearchOrMarkFromUrl();

            });

        }
        ,
        hasFocus : function () {
            return this.searchbar.is(':focus');
        }
        ,
        globalKeyHandler : function (e) {
            if (e.altKey || e.ctrlKey || e.metaKey || e.shiftKey) { return; }
            if (e.keyCode == this.ESCAPE_KEYCODE) {
                e.preventDefault();
                this.searchbar.removeClass("active");
                // this.searchbar[0].value = "";
                this.setSearchUrlParameters("",
                    (this.searchbar[0].value.trim() != 0) ? "push" : "replace");
                this.unfocusSearchbar();
                this.searchbar_outer.slideUp();
                return;
            }
            if (!this.hasFocus() && e.keyCode == this.SEARCH_HOTKEY_KEYCODE) {
                e.preventDefault();
                this.searchbar_outer.slideDown()
                this.searchbar.focus();
            }
        }
        ,
        unfocusSearchbar : function () {
            // hacky, but just focusing a div only works once
            var tmp = $('<input style="position: absolute; opacity: 0;">');
            tmp.insertAfter(this.searchicon);
            tmp.focus();
            tmp.remove();
        }
        ,
        searchIconClickHandler : function () {
            this.searchbar_outer.slideToggle();
            this.searchbar.focus();
            // TODO:
            // If invisible, clear URL search parameter
        }
        ,
        searchbarKeyUpHandler : function () {
            var searchterm = this.searchbar[0].value.trim();
            if (searchterm != "") {
                this.searchbar.addClass("active");
                this.doSearch(searchterm);
            } else {
                this.searchbar.removeClass("active");
                this.searchresults_outer.slideUp();
                this.searchresults.empty();
            }

            this.setSearchUrlParameters(searchterm, "if_begin_search");

            // Remove marks
            this.content.unmark();
        }
        ,
        setSearchUrlParameters : function(searchterm, action) {
            // Update url with ?SEARCH_PARAM= parameter, remove ?MARK_PARAM and #heading-anchor
            var url = this.parseURL(window.location.href);
            var first_search = ! url.params.hasOwnProperty(this.SEARCH_PARAM);
            if (searchterm != "" || action == "if_begin_search") {
                url.params[this.SEARCH_PARAM] = searchterm;
                delete url.params[this.MARK_PARAM];
                url.hash = "";
            } else {
                delete url.params[this.SEARCH_PARAM];
            }
            // A new search will also add a new history item, so the user can go back
            // to the page prior to searching. A updated search term will only replace
            // the url.
            if (action == "push" || (action == "if_begin_search" && first_search) ) {
                history.pushState({}, document.title, this.renderURL(url));
            } else if (action == "replace" || (action == "if_begin_search" && !first_search) ) {
                history.replaceState({}, document.title, this.renderURL(url));
            }

        }
    };

    // Interesting DOM Elements
    var sidebar = $("#sidebar");

    // url
    var url = window.location.pathname;

    // Fix back button cache problem
    window.onunload = function(){};

    // Set theme
    var theme = store.get('mdbook-theme');
    if (theme === null || theme === undefined) { theme = 'light'; }

    set_theme(theme);

    // Syntax highlighting Configuration
    hljs.configure({
        tabReplace: '    ', // 4 spaces
        languages: [],      // Languages used for auto-detection
    });

    if (window.ace) {
        // language-rust class needs to be removed for editable
        // blocks or highlightjs will capture events
        $('code.editable').removeClass('language-rust');

        $('code').not('.editable').each(function(i, block) {
            hljs.highlightBlock(block);
        });
    } else {
        $('code').each(function(i, block) {
            hljs.highlightBlock(block);
        });
    }

    // Adding the hljs class gives code blocks the color css
    // even if highlighting doesn't apply
    $('code').addClass('hljs');

    var KEY_CODES = {
        PREVIOUS_KEY: 37,
        NEXT_KEY: 39
    };

    $(document).on('keydown', function (e) {
        if (e.altKey || e.ctrlKey || e.metaKey || e.shiftKey) { return; }
        if (search.hasFocus()) { return; }
        switch (e.keyCode) {
            case KEY_CODES.NEXT_KEY:
                e.preventDefault();
                if($('.nav-chapters.next').length) {
                    window.location.href = $('.nav-chapters.next').attr('href');
                }
                break;
            case KEY_CODES.PREVIOUS_KEY:
                e.preventDefault();
                if($('.nav-chapters.previous').length) {
                    window.location.href = $('.nav-chapters.previous').attr('href');
                }
                break;
        }
    });

    // Help keyboard navigation by always focusing on page content
    $(".page").focus();

    // Toggle sidebar
    $("#sidebar-toggle").click(sidebarToggle);

    // Hide sidebar on section link click if it occupies large space
    // in relation to the whole screen (phone in portrait)
    $("#sidebar a").click(function(event){
        if (sidebar.width() > window.screen.width * 0.4) {
            sidebarToggle();
        }
    });

    // Scroll sidebar to current active section
    var activeSection = sidebar.find(".active");
    if(activeSection.length) {
        sidebar.scrollTop(activeSection.offset().top);
    }

    // Search
    search.init();

    // Theme button
    $("#theme-toggle").click(function(){
        if($('.theme-popup').length) {
            $('.theme-popup').remove();
        } else {
            var popup = $('<div class="theme-popup"></div>')
                .append($('<div class="theme" id="light">Light <span class="default">(default)</span><div>'))
                .append($('<div class="theme" id="rust">Rust</div>'))
                .append($('<div class="theme" id="coal">Coal</div>'))
                .append($('<div class="theme" id="navy">Navy</div>'))
                .append($('<div class="theme" id="ayu">Ayu</div>'));


            popup.insertAfter(this);

            $('.theme').click(function(){
                var theme = $(this).attr('id');
                set_theme(theme);
            });
        }
    });

    // Hide theme selector popup when clicking outside of it
    $(document).click(function(event){
        var popup = $('.theme-popup');
        if(popup.length) {
            var target = $(event.target);
            if(!target.closest('.theme').length && !target.closest('#theme-toggle').length) {
                popup.remove();
            }
        }
    });

    function set_theme(theme) {
        let ace_theme;

        if (theme == 'coal' || theme == 'navy') {
            $("[href='ayu-highlight.css']").prop('disabled', true);
            $("[href='tomorrow-night.css']").prop('disabled', false);
            $("[href='highlight.css']").prop('disabled', true);

            ace_theme = "ace/theme/tomorrow_night";
        } else if (theme == 'ayu') {
            $("[href='ayu-highlight.css']").prop('disabled', false);
            $("[href='tomorrow-night.css']").prop('disabled', true);
            $("[href='highlight.css']").prop('disabled', true);

            ace_theme = "ace/theme/tomorrow_night";
        } else {
            $("[href='ayu-highlight.css']").prop('disabled', true);
            $("[href='tomorrow-night.css']").prop('disabled', true);
            $("[href='highlight.css']").prop('disabled', false);

            ace_theme = "ace/theme/dawn";
        }

        if (window.ace && window.editors) {
            window.editors.forEach(function(editor) {
                editor.setTheme(ace_theme);
            });
        }

        store.set('mdbook-theme', theme);

        $('body').removeClass().addClass(theme);
    }


    // Hide Rust code lines prepended with a specific character
    var hiding_character = "#";

    $("code.language-rust").each(function(i, block){

        var code_block = $(this);
        var pre_block = $(this).parent();
        // hide lines
        var lines = code_block.html().split("\n");
        var first_non_hidden_line = false;
        var lines_hidden = false;

        for(var n = 0; n < lines.length; n++){
            if($.trim(lines[n])[0] == hiding_character){
                if(first_non_hidden_line){
                    lines[n] = "<span class=\"hidden\">" + "\n" + lines[n].replace(/(\s*)# ?/, "$1") + "</span>";
                }
                else {
                    lines[n] = "<span class=\"hidden\">" + lines[n].replace(/(\s*)# ?/, "$1") + "\n"  +  "</span>";
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
        code_block.html(lines.join(""));

        // If no lines were hidden, return
        if(!lines_hidden) { return; }

        // add expand button
        pre_block.prepend("<div class=\"buttons\"><i class=\"fa fa-expand\" title=\"Show hidden lines\"></i></div>");

        pre_block.find("i").click(function(e){
            if( $(this).hasClass("fa-expand") ) {
                $(this).removeClass("fa-expand").addClass("fa-compress");
                $(this).attr("title", "Hide lines");
                pre_block.find("span.hidden").removeClass("hidden").addClass("unhidden");
            }
            else {
                $(this).removeClass("fa-compress").addClass("fa-expand");
                $(this).attr("title", "Show hidden lines");
                pre_block.find("span.unhidden").removeClass("unhidden").addClass("hidden");
            }
        });
    });

    // Process playpen code blocks
    $(".playpen").each(function(block){
        var pre_block = $(this);
        // Add play button
        var buttons = pre_block.find(".buttons");
        if( buttons.length === 0 ) {
            pre_block.prepend("<div class=\"buttons\"></div>");
            buttons = pre_block.find(".buttons");
        }
        buttons.prepend("<i class=\"fa fa-play play-button hidden\" title=\"Run this code\"></i>");
        buttons.prepend("<i class=\"fa fa-copy clip-button\" title=\"Copy to clipboard\"><i class=\"tooltiptext\"></i></i>");

        let code_block = pre_block.find("code").first();
        if (window.ace && code_block.hasClass("editable")) {
            buttons.prepend("<i class=\"fa fa-history reset-button\" title=\"Undo changes\"></i>");
        }

        buttons.find(".play-button").click(function(e){
            run_rust_code(pre_block);
        });
        buttons.find(".clip-button").mouseout(function(e){
            hideTooltip(e.currentTarget);
        });
        buttons.find(".reset-button").click(function() {
            if (!window.ace) { return; }
            let editor = window.ace.edit(code_block.get(0));
            editor.setValue(editor.originalCode);
            editor.clearSelection();
        });
    });

    var clipboardSnippets = new Clipboard('.clip-button', {
        text: function(trigger) {
            hideTooltip(trigger);
            let playpen = $(trigger).parents(".playpen");
            return playpen_text(playpen);
        }
    });
    clipboardSnippets.on('success', function(e) {
            e.clearSelection();
            showTooltip(e.trigger, "Copied!");
    });
    clipboardSnippets.on('error', function(e) {
            showTooltip(e.trigger, "Clipboard error!");
    });

    $.ajax({
        url: "https://play.rust-lang.org/meta/crates",
        method: "POST",
        crossDomain: true,
        dataType: "json",
        contentType: "application/json",
        success: function(response){
            // get list of crates available in the rust playground
            let playground_crates = response.crates.map(function(item) {return item["id"];} );
            $(".playpen").each(function(block) {
                handle_crate_list_update($(this), playground_crates);
            });
        },
    });

});

function playpen_text(playpen) {
    let code_block = playpen.find("code").first();

    if (window.ace && code_block.hasClass("editable")) {
        let editor = window.ace.edit(code_block.get(0));
        return editor.getValue();
    } else {
        return code_block.get(0).textContent;
    }
}

function handle_crate_list_update(playpen_block, playground_crates) {
    // update the play buttons after receiving the response
    update_play_button(playpen_block, playground_crates);

    // and install on change listener to dynamically update ACE editors
    if (window.ace) {
        let code_block = playpen_block.find("code").first();
        if (code_block.hasClass("editable")) {
            let editor = window.ace.edit(code_block.get(0));
            editor.on("change", function(e){
                update_play_button(playpen_block, playground_crates);
            });
        }
    }
}

// updates the visibility of play button based on `no_run` class and
// used crates vs ones available on http://play.rust-lang.org
function update_play_button(pre_block, playground_crates) {
    var play_button = pre_block.find(".play-button");

    var classes = pre_block.find("code").attr("class").split(" ");
    // skip if code is `no_run`
    if (classes.indexOf("no_run") > -1) {
        play_button.addClass("hidden");
        return;
    }

    // get list of `extern crate`'s from snippet
    var txt = playpen_text(pre_block);
    var re = /extern\s+crate\s+([a-zA-Z_0-9]+)\s*;/g;
    var snippet_crates = [];
    while (item = re.exec(txt)) {
        snippet_crates.push(item[1]);
    }

    // check if all used crates are available on play.rust-lang.org
    var all_available = snippet_crates.every(function(elem) {
        return playground_crates.indexOf(elem) > -1;
    });

    if (all_available) {
        play_button.removeClass("hidden");
    } else {
        play_button.addClass("hidden");
    }
}

function hideTooltip(elem) {
    elem.firstChild.innerText="";
    elem.setAttribute('class', 'fa fa-copy clip-button');
}

function showTooltip(elem, msg) {
    elem.firstChild.innerText=msg;
    elem.setAttribute('class', 'fa fa-copy tooltipped');
}

function sidebarToggle() {
    var html = $("html");
    if ( html.hasClass("sidebar-hidden") ) {
        html.removeClass("sidebar-hidden").addClass("sidebar-visible");
        store.set('mdbook-sidebar', 'visible');
    } else if ( html.hasClass("sidebar-visible") ) {
        html.removeClass("sidebar-visible").addClass("sidebar-hidden");
        store.set('mdbook-sidebar', 'hidden');
    } else {
        if($("#sidebar").position().left === 0){
            html.addClass("sidebar-hidden");
            store.set('mdbook-sidebar', 'hidden');
        } else {
            html.addClass("sidebar-visible");
            store.set('mdbook-sidebar', 'visible');
        }
    }
}

function run_rust_code(code_block) {
    var result_block = code_block.find(".result");
    if(result_block.length === 0) {
        code_block.append("<code class=\"result hljs language-bash\"></code>");
        result_block = code_block.find(".result");
    }

    let text = playpen_text(code_block);

    var params = {
	channel: "stable",
	mode: "debug",
	crateType: "bin",
	tests: false,
	code: text,
    }

    if(text.indexOf("#![feature") !== -1) {
        params.channel = "nightly";
    }

    result_block.text("Running...");

    $.ajax({
        url: "https://play.rust-lang.org/execute",
        method: "POST",
        crossDomain: true,
        dataType: "json",
        contentType: "application/json",
        data: JSON.stringify(params),
        timeout: 15000,
        success: function(response){
           result_block.text(response.success ? response.stdout : response.stderr);
        },
        error: function(qXHR, textStatus, errorThrown){
            result_block.text("Playground communication " + textStatus);
        },
    });
}

