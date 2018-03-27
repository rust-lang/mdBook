window.search = window.search || {};
(function search(search) {
    // Search functionality
    //
    // You can use !hasFocus() to prevent keyhandling in your key
    // event handlers while the user is typing his search.

    if (!Mark || !elasticlunr) {
        return;
    }
    
    var searchbar = document.getElementById('searchbar'),
        searchbar_outer = document.getElementById('searchbar-outer'),
        searchresults = document.getElementById('searchresults'),
        searchresults_outer = document.getElementById('searchresults-outer'),
        searchresults_header = document.getElementById('searchresults-header'),
        searchicon = document.getElementById('search-toggle'),
        content = document.getElementById('content'),

        searchindex = null,
        resultsoptions = {
            teaser_word_count: 30,
            limit_results: 30,
        },
        searchoptions = {
            bool: "AND",
            expand: true,
            fields: {
                title: {boost: 1},
                body: {boost: 1},
                breadcrumbs: {boost: 0}
            }
        },
        mark_exclude = [],
        marker = new Mark(content),
        current_searchterm = "",
        URL_SEARCH_PARAM = 'search',
        URL_MARK_PARAM = 'highlight',
        teaser_count = 0,

        SEARCH_HOTKEY_KEYCODE = 83,
        ESCAPE_KEYCODE = 27,
        DOWN_KEYCODE = 40,
        UP_KEYCODE = 38,
        SELECT_KEYCODE = 13;

    function hasFocus() {
        return searchbar === document.activeElement;
    }

    function removeChildren(elem) {
        while (elem.firstChild) {
            elem.removeChild(elem.firstChild);
        }
    }

    // Helper to parse a url into its building blocks.
    function parseURL(url) {
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
    
    // Helper to recreate a url string from its building blocks.
    function renderURL(urlobject) {
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
    
    // Helper to escape html special chars for displaying the teasers
    var escapeHTML = (function() {
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
    })();
    
    function formatSearchMetric(count, searchterm) {
        if (count == 1) {
            return count + " search result for '" + searchterm + "':";
        } else if (count == 0) {
            return "No search results for '" + searchterm + "'.";
        } else {
            return count + " search results for '" + searchterm + "':";
        }
    }
    
    function formatSearchResult(result, searchterms) {
        var teaser = makeTeaser(escapeHTML(result.doc.body), searchterms);
        teaser_count++;

        // The ?URL_MARK_PARAM= parameter belongs inbetween the page and the #heading-anchor
        var url = result.ref.split("#");
        if (url.length == 1) { // no anchor found
            url.push("");
        }

        return '<a href="' + url[0] + '?' + URL_MARK_PARAM + '=' + searchterms + '#' + url[1]
            + '" aria-details="teaser_' + teaser_count + '">' + result.doc.breadcrumbs + '</a>'
            + '<span class="teaser" id="teaser_' + teaser_count + '" aria-label="Search Result Teaser">' 
            + teaser + '</span>';
    }
    
    function makeTeaser(body, searchterms) {
        // The strategy is as follows:
        // First, assign a value to each word in the document:
        //  Words that correspond to search terms (stemmer aware): 40
        //  Normal words: 2
        //  First word in a sentence: 8
        // Then use a sliding window with a constant number of words and count the
        // sum of the values of the words within the window. Then use the window that got the
        // maximum sum. If there are multiple maximas, then get the last one.
        // Enclose the terms in <em>.
        var stemmed_searchterms = searchterms.map(function(w) {
            return elasticlunr.stemmer(w.toLowerCase());
        });
        var searchterm_weight = 40;
        var weighted = []; // contains elements of ["word", weight, index_in_document]
        // split in sentences, then words
        var sentences = body.toLowerCase().split('. ');
        var index = 0;
        var value = 0;
        var searchterm_found = false;
        for (var sentenceindex in sentences) {
            var words = sentences[sentenceindex].split(' ');
            value = 8;
            for (var wordindex in words) {
                var word = words[wordindex];
                if (word.length > 0) {
                    for (var searchtermindex in stemmed_searchterms) {
                        if (elasticlunr.stemmer(word).startsWith(stemmed_searchterms[searchtermindex])) {
                            value = searchterm_weight;
                            searchterm_found = true;
                        }
                    };
                    weighted.push([word, value, index]);
                    value = 2;
                }
                index += word.length;
                index += 1; // ' ' or '.' if last word in sentence
            };
            index += 1; // because we split at a two-char boundary '. '
        };

        if (weighted.length == 0) {
            return body;
        }

        var window_weight = [];
        var window_size = Math.min(weighted.length, resultsoptions.teaser_word_count);

        var cur_sum = 0;
        for (var wordindex = 0; wordindex < window_size; wordindex++) {
            cur_sum += weighted[wordindex][1];
        };
        window_weight.push(cur_sum);
        for (var wordindex = 0; wordindex < weighted.length - window_size; wordindex++) {
            cur_sum -= weighted[wordindex][1];
            cur_sum += weighted[wordindex + window_size][1];
            window_weight.push(cur_sum);
        };

        if (searchterm_found) {
            var max_sum = 0;
            var max_sum_window_index = 0;
            // backwards
            for (var i = window_weight.length - 1; i >= 0; i--) {
                if (window_weight[i] > max_sum) {
                    max_sum = window_weight[i];
                    max_sum_window_index = i;
                }
            };
        } else {
            max_sum_window_index = 0;
        }

        // add <em/> around searchterms
        var teaser_split = [];
        var index = weighted[max_sum_window_index][2];
        for (var i = max_sum_window_index; i < max_sum_window_index+window_size; i++) {
            var word = weighted[i];
            if (index < word[2]) {
                // missing text from index to start of `word`
                teaser_split.push(body.substring(index, word[2]));
                index = word[2];
            }
            if (word[1] == searchterm_weight) {
                teaser_split.push("<em>")
            }
            index = word[2] + word[0].length;
            teaser_split.push(body.substring(word[2], index));
            if (word[1] == searchterm_weight) {
                teaser_split.push("</em>")
            }
        };

        return teaser_split.join('');
    }

    function init() {
        resultsoptions = window.search.resultsoptions;
        searchoptions = window.search.searchoptions;
        searchindex = elasticlunr.Index.load(window.search.index);

        // Set up events
        searchicon.addEventListener('click', function(e) { searchIconClickHandler(); }, false);
        searchbar.addEventListener('keyup', function(e) { searchbarKeyUpHandler(); }, false);
        document.addEventListener('keydown', function (e) { globalKeyHandler(e); }, false);
        // If the user uses the browser buttons, do the same as if a reload happened
        window.onpopstate = function(e) { doSearchOrMarkFromUrl(); };

        // If reloaded, do the search or mark again, depending on the current url parameters
        doSearchOrMarkFromUrl();
    }
    
    function unfocusSearchbar() {
        // hacky, but just focusing a div only works once
        var tmp = document.createElement('input');
        tmp.setAttribute('style', 'position: absolute; opacity: 0;');
        searchicon.appendChild(tmp);
        tmp.focus();
        tmp.remove();
    }
    
    // On reload or browser history backwards/forwards events, parse the url and do search or mark
    function doSearchOrMarkFromUrl() {
        // Check current URL for search request
        var url = parseURL(window.location.href);
        if (url.params.hasOwnProperty(URL_SEARCH_PARAM)
            && url.params[URL_SEARCH_PARAM] != "") {
            showSearch(true);
            searchbar.value = decodeURIComponent(
                (url.params[URL_SEARCH_PARAM]+'').replace(/\+/g, '%20'));
            searchbarKeyUpHandler(); // -> doSearch()
        } else {
            showSearch(false);
        }

        if (url.params.hasOwnProperty(URL_MARK_PARAM)) {
            var words = url.params[URL_MARK_PARAM].split(' ');
            marker.mark(words, {
                exclude: mark_exclude
            });

            var markers = document.querySelectorAll("mark");
            function hide() {
                for (var i = 0; i < markers.length; i++) {
                    markers[i].classList.add("fade-out");
                    window.setTimeout(function(e) { marker.unmark(); }, 300);
                }
            }
            for (var i = 0; i < markers.length; i++) {
                markers[i].addEventListener('click', hide);
            }
        }
    }
    
    // Eventhandler for keyevents on `document`
    function globalKeyHandler(e) {
        if (e.altKey || e.ctrlKey || e.metaKey || e.shiftKey) { return; }

        if (e.keyCode == ESCAPE_KEYCODE) {
            e.preventDefault();
            searchbar.classList.remove("active");
            setSearchUrlParameters("",
                (searchbar.value.trim() != "") ? "push" : "replace");
            if (hasFocus()) {
                unfocusSearchbar();
            }
            showSearch(false);
            marker.unmark();
            return;
        }
        if (!hasFocus() && e.keyCode == SEARCH_HOTKEY_KEYCODE) {
            e.preventDefault();
            showSearch(true);
            window.scrollTo(0, 0);
            searchbar.focus();
            return;
        }
        if (hasFocus() && e.keyCode == DOWN_KEYCODE) {
            e.preventDefault();
            unfocusSearchbar();
            searchresults.children('li').first().classList.add("focus");
            return;
        }
        if (!hasFocus() && (e.keyCode == DOWN_KEYCODE
                            || e.keyCode == UP_KEYCODE
                            || e.keyCode == SELECT_KEYCODE)) {
            // not `:focus` because browser does annoying scrolling
            var current_focus = search.searchresults.find("li.focus");
            if (current_focus.length == 0) return;
            e.preventDefault();
            if (e.keyCode == DOWN_KEYCODE) {
                var next = current_focus.next()
                if (next.length > 0) {
                    current_focus.classList.remove("focus");
                    next.classList.add("focus");
                }
            } else if (e.keyCode == UP_KEYCODE) {
                current_focus.classList.remove("focus");
                var prev = current_focus.prev();
                if (prev.length == 0) {
                    searchbar.focus();
                } else {
                    prev.classList.add("focus");
                }
            } else {
                window.location = current_focus.children('a').attr('href');
            }
        }
    }
    
    function showSearch(yes) {
        if (yes) {
            searchbar_outer.style.display = 'block';
            content.style.display = 'none';
            searchicon.setAttribute('aria-expanded', 'true');
        } else {
            content.style.display = 'block';
            searchbar_outer.style.display = 'none';
            searchresults_outer.style.display = 'none';
            searchbar.value = '';
            removeChildren(searchresults);
            searchicon.setAttribute('aria-expanded', 'false');
        }
    }

    function showResults(yes) {
        if (yes) {
            searchbar_outer.style.display = 'block';
            content.style.display = 'none';
            searchresults_outer.style.display = 'block';
        } else {
            content.style.display = 'block';
            searchresults_outer.style.display = 'none';
        }
    }

    // Eventhandler for search icon
    function searchIconClickHandler() {
        if (searchbar_outer.style.display === 'block') {
            showSearch(false);
        } else {
            showSearch(true);
            window.scrollTo(0, 0);
            searchbar.focus();
        }
    }
    
    // Eventhandler for keyevents while the searchbar is focused
    function searchbarKeyUpHandler() {
        var searchterm = searchbar.value.trim();
        if (searchterm != "") {
            searchbar.classList.add("active");
            doSearch(searchterm);
        } else {
            searchbar.classList.remove("active");
            showResults(false);
            removeChildren(searchresults);
        }

        setSearchUrlParameters(searchterm, "push_if_new_search_else_replace");

        // Remove marks
        marker.unmark();
    }
    
    // Update current url with ?URL_SEARCH_PARAM= parameter, remove ?URL_MARK_PARAM and #heading-anchor .
    // `action` can be one of "push", "replace", "push_if_new_search_else_replace"
    // and replaces or pushes a new browser history item.
    // "push_if_new_search_else_replace" pushes if there is no `?URL_SEARCH_PARAM=abc` yet.
    function setSearchUrlParameters(searchterm, action) {
        var url = parseURL(window.location.href);
        var first_search = ! url.params.hasOwnProperty(URL_SEARCH_PARAM);
        if (searchterm != "" || action == "push_if_new_search_else_replace") {
            url.params[URL_SEARCH_PARAM] = searchterm;
            delete url.params[URL_MARK_PARAM];
            url.hash = "";
        } else {
            delete url.params[URL_SEARCH_PARAM];
        }
        // A new search will also add a new history item, so the user can go back
        // to the page prior to searching. A updated search term will only replace
        // the url.
        if (action == "push" || (action == "push_if_new_search_else_replace" && first_search) ) {
            history.pushState({}, document.title, renderURL(url));
        } else if (action == "replace" || (action == "push_if_new_search_else_replace" && !first_search) ) {
            history.replaceState({}, document.title, renderURL(url));
        }
    }
    
    function doSearch(searchterm) {

        // Don't search the same twice
        if (current_searchterm == searchterm) { return; }
        else { current_searchterm = searchterm; }

        if (searchindex == null) { return; }

        // Do the actual search
        var results = searchindex.search(searchterm, searchoptions);
        var resultcount = Math.min(results.length, resultsoptions.limit_results);

        // Display search metrics
        searchresults_header.innerText = formatSearchMetric(resultcount, searchterm);

        // Clear and insert results
        var searchterms  = searchterm.split(' ');
        removeChildren(searchresults);
        for(var i = 0; i < resultcount ; i++){
            var resultElem = document.createElement('li');
            resultElem.innerHTML = formatSearchResult(results[i], searchterms);
            searchresults.appendChild(resultElem);
        }

        // Display results
        showResults(true);
    }

    init();
    // Exported functions
    search.hasFocus = hasFocus;
})(window.search);
