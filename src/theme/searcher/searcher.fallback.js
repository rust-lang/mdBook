"use strict";
window.search = window.search || {};
(function search(search) {
    // Search functionality
    //
    // You can use !hasFocus() to prevent keyhandling in your key
    // event handlers while the user is typing their search.

    if (!Mark || !elasticlunr) {
        return;
    }

    //IE 11 Compatibility from https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/startsWith
    if (!String.prototype.startsWith) {
        String.prototype.startsWith = function(search, pos) {
            return this.substr(!pos || pos < 0 ? 0 : +pos, search.length) === search;
        };
    }

    var search_wrap = document.getElementById('search-wrapper'),
        searchbar = document.getElementById('searchbar'),
        searchbar_outer = document.getElementById('searchbar-outer'),
        searchresults = document.getElementById('searchresults'),
        searchresults_outer = document.getElementById('searchresults-outer'),
        searchresults_header = document.getElementById('searchresults-header'),
        searchicon = document.getElementById('search-toggle'),
        content = document.getElementById('content'),

        searchindex = null,
        doc_urls = [],
        results_options = {
            teaser_word_count: 30,
            limit_results: 30,
        },
        search_options = {
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

    const REGEX_WHITE_SPACE = /\p{White_Space}+/gu,
        REGEX_SEARCH_SPLITTER = /(?:([\p{Unified_Ideograph}\uAC00-\uD7AF]|[^\p{White_Space}\p{P}\p{Sm}\p{Sc}\p{So}\p{Unified_Ideograph}\uAC00-\uD7AF\p{Z}\p{C}]+|\p{So}\p{Sk}?(?:\u200D\p{So}\p{Sk}?)*)|([\p{P}\p{Sm}\p{Sc}\p{Z}\p{C}]+))\p{White_Space}*/gu,
        REGEX_STEM = /([a-zA-Z0-9]+)|[^a-zA-Z0-9]+/gu,
        REGEX_ESCAPE = /[.*+?^${}()|[\]\\]/gu,
        REGEX_DEFAULT_BEGIN = /^[^\p{White_Space}\p{P}\p{Sm}\p{Sc}\p{So}\p{Unified_Ideograph}\uAC00-\uD7AF\p{Z}\p{C}]/u,
        REGEX_DEFAULT_END = /[^\p{White_Space}\p{P}\p{Sm}\p{Sc}\p{So}\p{Unified_Ideograph}\uAC00-\uD7AF\p{Z}\p{C}]$/u,
        REGEX_SENTENCE = /.+?(?:[。？！．](?:(?![\r\n])[\p{White_Space}\p{Po}])*[\r\n]*|(?:[.?!](?:(?![\r\n])[\p{White_Space}\p{Po}])*?(?:(?![\r\n])\p{White_Space})+)+(?=[^\p{L}]*(?!\p{Ll})\p{L})|[\r\n]+)|.+?$/gu,
        REGEX_CLAUSE = /.*?(?:(?:[，；]|……)[\p{White_Space}\p{Po}]*|[,;](?:\p{Po}*?\p{White_Space}+)+)|.+?$/gus,
        REGEX_SEGMENT = /([\p{Unified_Ideograph}\uAC00-\uD7AF]+)|([^\p{White_Space}\p{P}\p{Sm}\p{Sc}\p{So}\p{Unified_Ideograph}\uAC00-\uD7AF\p{Z}\p{C}]+)|(\p{So}\p{Sk}?(?:\u200D\p{So}\p{Sk}?)*)|([\p{White_Space}\p{P}\p{Sm}\p{Sc}\p{Z}\p{C}]+)/gu;

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
        var repl = function (c, inMap) { return inMap ? MAP[c] : "<br/>"; };
        return function (s) {
            return s.replace(/([&<>'"])|[\r\n]+/g, repl);
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

    function formatSearchResult(result, searchTerms) {
        var teaser = makeTeaser(result.doc, searchTerms);
        if (!teaser) return;

        teaser_count++;

        // The ?URL_MARK_PARAM= parameter belongs inbetween the page and the #heading-anchor
        var url = doc_urls[result.ref].split("#");
        if (url.length == 1) { // no anchor found
            url.push("");
        }

        return '<a href="' + path_to_root + url[0] + '?' + URL_MARK_PARAM + '=' + searchTerms.url + '#' + url[1]
            + '" aria-details="teaser_' + teaser_count + '">' + teaser.breadcrumbs + '</a>'
            + '<span class="teaser" id="teaser_' + teaser_count + '" aria-label="Search Result Teaser">'
            + teaser.body + '</span>';
    }

    // `targets` is an array of {begin: number, end: number} that has been sorted by begin
    // in ascending order, and shouldn't overlap.
    // `range` is {begin: number, end: number}
    function highlightAndEscape(text, targets, range) {
        const limit = range ? range.end : text.length;
        var lastEnd = range ? range.begin : 0;
        if (!targets.length) return escapeHTML(text.slice(lastEnd, limit));

        for (var i = 0; targets[i].end <= lastEnd; i++) ; // skip targets before range
        const parts = [], begin = targets[i].begin;
        if (lastEnd > begin) lastEnd = begin;

        for (; i < targets.length; i++) {
            const target = targets[i], begin = target.begin;
            if (begin >= limit) break; // omit targets after range
            const end = target.end;
            parts.push(escapeHTML(text.slice(lastEnd, begin)), '<em>', escapeHTML(text.slice(begin, end)), '</em>');
            lastEnd = end;
        }
        parts.push(escapeHTML(text.slice(lastEnd, limit).trimEnd()));

        return "".concat(...parts);
    }

    // Merge overlapping or contiguous ranges
    function mergeRanges(ranges) {
        if (!ranges.length) return [];

        var last = {begin: ranges[0].begin, end: ranges[0].end};
        const result = [last];
        for (const range of ranges.slice(1)) {
            if (last.end < range.begin) {
                last = {begin: range.begin, end: range.end};
                result.push(last);
            } else if (last.end < range.end) {
                last.end = range.end;
            }
        }
        return result;
    }

    class StructuredText {
        constructor(text) {
            this.original = text;
            this.segments = new Map();
            this.pos = 0;
            this.stemmedPos = 0; // `this` is passed to the constructors, and the `pos` fields will be updated there.
            const matches = text.match(REGEX_SENTENCE);
            this.sentences = matches ? matches.map(match => new Sentence(match, this)) : [];
            this.stemmed = "".concat(...Array.from(this.segments.values(), segment => segment.stemmed.text));
            delete this.pos;
            delete this.stemmedPos;
        }

        originalPos(stemmedPos) {
            if (stemmedPos <= 0) return stemmedPos;
            const offset = stemmedPos - this.stemmed.length;
            if (offset >= 0) return this.original.length + offset;
            const segment = this.segments.get(stemmedPos);
            if (segment) return segment.lower.begin;
            for (var pos = stemmedPos - 1; ; pos--) {
                const segment = this.segments.get(pos);
                if (segment) {
                    return segment.lower.begin + (segment instanceof DefaultSegment ? segment.lower.text.length : stemmedPos - pos);
                }
            }
        }

        segmentAtStemmed(stemmedPos) {
            if (stemmedPos < 0) return;
            if (stemmedPos >= this.stemmed.length) return;
            const segment = this.segments.get(stemmedPos);
            if (segment) return segment;
            for (var pos = stemmedPos - 1; ; pos--) {
                const segment = this.segments.get(pos);
                if (segment) return segment;
            }
        }

        // `begin` and `end`are indexed on stemmed text.
        wordCount(begin, end) {
            if (begin >= end) return 0;
            const segment = this.segmentAtStemmed(begin), segmentEnd = segment.stemmed.end;
            if (segment instanceof IdeographSegment) {
                return [...this.stemmed.slice(begin, Math.min(end, segmentEnd))].length / 2 + this.wordCount(segmentEnd, end);
            } else {
                return segment.wordCount + this.wordCount(segmentEnd, end);
            }
        }

        // `targetsInStemmed` is an array of {begin: number, end: number} that has been sorted by begin in ascending order.
        // `ranges` is an array of {begin: number, end: number}
        highlightAndEscapeByStemmed(targetsInStemmed, ranges) {
            targetsInStemmed = mergeRanges(targetsInStemmed);
            if (!Array.isArray(ranges)) return this.highlightAndEscapeByStemmedInRange(targetsInStemmed, ranges);
            ranges = mergeRanges(ranges);
            if (!ranges.length) return "";
            const parts = ranges.map(range => this.highlightAndEscapeByStemmedInRange(targetsInStemmed, range));
            if (ranges[0].begin > 0) parts.unshift("");
            if (ranges[ranges.length - 1].end < this.stemmed.length) parts.push("");
            return parts.join("……");
        }

        highlightAndEscapeByStemmedInRange(targetsInStemmed, range) {
            return highlightAndEscape(this.original, targetsInStemmed.map(target => {
                return {begin: this.originalPos(target.begin), end: this.originalPos(target.end)};
            }), range ? {begin: this.originalPos(range.begin), end: this.originalPos(range.end)} : undefined);
        }

        // Expands `range`'s end by `wordCount` words.
        // `range` is like {begin: number, end: number} and is indexed on stemmed text.
        // The range is modified in-place.
        // `limit` is where the range would stop expanding even the required `wordCount` isn't satisfied.
        // In this case the remaining `wordCount` to be expanded is returned (otherwise undefined is returned)
        // If `limit` is undefined, expanding would stop at the end of the text.
        expandEnd(range, wordCount, limit) {
            if (typeof limit !== "number" || limit > this.stemmed.length) limit = this.stemmed.length;
            if (range.end < range.begin) range.end = range.begin;
            if (range.end >= limit) {
                if (wordCount < 1) return;
                return wordCount;
            }
            const pos = range.end, segment = this.segmentAtStemmed(pos);
            if (segment instanceof IdeographSegment) {
                if (wordCount * 2 < 1) return;
                const end = Math.min(segment.stemmed.end, limit);
                const slice = [...this.stemmed.slice(pos, end)];
                const remainingWordCount = wordCount - slice.length / 2;
                if (remainingWordCount < 0) {
                    range.end += "".concat(...slice.slice(0, wordCount * 2)).length;
                    return;
                }
                range.end = end;
                return this.expandEnd(range, remainingWordCount, limit);
            } else {
                wordCount -= segment.wordCount;
                if (wordCount < 0) return;
                range.end = Math.min(segment.stemmed.end, limit);
                return this.expandEnd(range, wordCount, limit);
            }
        }

        // Counterpart to expandEnd
        expandBegin(range, wordCount, limit) {
            if (wordCount < 1) return;
            if (typeof limit !== "number" || limit < 0) limit = 0;
            if (range.begin > range.end) range.begin = range.end;
            if (range.begin <= limit) {
                if (wordCount < 1) return;
                return wordCount;
            }
            const pos = range.begin, segment = this.segmentAtStemmed(pos - 1);
            if (segment instanceof IdeographSegment) {
                if (wordCount * 2 < 1) return;
                const begin = Math.max(segment.stemmed.begin, limit);
                const slice = [...this.stemmed.slice(begin, pos)];
                const remainingWordCount = wordCount - slice.length / 2;
                if (remainingWordCount < 0) {
                    range.begin -= "".concat(...slice.slice(-wordCount * 2)).length;
                    return;
                }
                range.begin = begin;
                return this.expandBegin(range, remainingWordCount, limit);
            } else {
                wordCount -= segment.wordCount;
                range.begin = Math.max(segment.stemmed.begin, limit);
                return this.expandBegin(range, wordCount, limit);
            }
        }

        // Expands `range`'s end to `type`'s boundary.
        // `range` is like {begin: number, end: number} and is indexed on stemmed text.
        // The range is modified in-place.
        // `limit` is where the range would stop expanding even the required `type`'s boundary isn't reached.
        // In this case true is returned (otherwise false is returned)
        // If `limit` is undefined, expanding would stop at the end of the text.
        expandEndToBoundary(range, type, limit) {
            if (typeof limit !== "number") limit = this.stemmed.length;
            if (range.end < range.begin) range.end = range.begin;
            if (range.end >= limit) return true;
            var part = this.segmentAtStemmed(range.end);
            while (!(part instanceof type)) part = part.parent;
            const partEnd = part.stemmed.end;
            if (partEnd <= limit) {
                range.end = partEnd;
                return false;
            }
            range.end = limit;
            return true;
        }

        // Counterpart to expandEndToBoundary
        expandBeginToBoundary(range, type, limit) {
            if (typeof limit !== "number") limit = 0;
            if (range.begin > range.end) range.begin = range.end;
            if (range.begin <= limit) return true;
            var part = this.segmentAtStemmed(range.begin - 1);
            while (!(part instanceof type)) part = part.parent;
            const partBegin = part.stemmed.begin;
            if (partBegin >= limit) {
                range.begin = partBegin;
                return false;
            }
            range.begin = limit;
            return true;
        }

        // Counterpart to expandEndToBoundary
        shrinkEndToBoundary(range, type, limit) {
            if (typeof limit !== "number") limit = range.begin;
            if (range.end > this.stemmed.length) range.end = this.stemmed.length;
            if (range.end <= limit) return true;
            var part = this.segmentAtStemmed(range.end - 1);
            while (!(part instanceof type)) part = part.parent;
            const partBegin = part.stemmed.begin;
            if (partBegin >= limit) {
                range.end = partBegin;
                return false;
            }
            range.end = limit;
            return true;
        }

        // Counterpart to expandBeginToBoundary
        shrinkBeginToBoundary(range, type, limit) {
            if (typeof limit !== "number") limit = range.end;
            if (range.begin < 0) range.begin = 0;
            if (range.begin >= limit) return true;
            var part = this.segmentAtStemmed(range.begin);
            while (!(part instanceof type)) part = part.parent;
            const partEnd = part.stemmed.end;
            if (partEnd <= limit) {
                range.begin = partEnd;
                return false;
            }
            range.begin = limit;
            return true;
        }
    }

    class Sentence {
        constructor(original, base) {
            this.original = {text: original, begin: base.pos}
            const begin = base.stemmedPos;
            this.clauses = original.toLowerCase().match(REGEX_CLAUSE).map(match => new Clause(match, this, base));
            this.stemmed = {begin, end: base.stemmedPos}
        }
    }

    class Clause {
        constructor(lower, parent, base) {
            this.lower = {text: lower, begin: base.pos}
            const begin = base.stemmedPos, segments = [];
            for (const match of lower.matchAll(REGEX_SEGMENT)) {
                if (match[1]) {
                    segments.push(new IdeographSegment(match[0], this, base));
                } else if (match[2]) {
                    segments.push(new DefaultSegment(match[0], this, base));
                } else if (match[3]) {
                    segments.push(new EmojiSegment(match[0], this, base));
                } else if (match[4]) {
                    segments.push(new NonWordSegment(match[0], this, base));
                }
            }
            this.segments = segments;
            this.stemmed = {begin, end: base.stemmedPos};
            this.parent = parent;
        }
    }

    class Segment {
        constructor(lower, stemmed, parent, base) {
            this.lower = {text: lower, begin: base.pos}
            const begin = base.stemmedPos;
            base.pos += lower.length;
            base.stemmedPos += stemmed.length;
            base.segments.set(begin, this);
            this.stemmed = {text: stemmed, begin, end: base.stemmedPos}
            this.parent = parent;
        }
    }

    class IdeographSegment extends Segment {
        constructor(lower, parent, base) {
            super(lower, lower, parent, base);
            this.wordCount = [...lower].length / 2; // 2 characters count as 1 word
        }
    }

    class EmojiSegment extends Segment {
        constructor(lower, parent, base) {
            super(lower, lower, parent, base);
        }

        get wordCount() {
            return 1;
        }
    }

    class NonWordSegment extends Segment {
        constructor(lower, parent, base) {
            super(lower, lower, parent, base);
        }

        get wordCount() {
            return 0;
        }
    }

    class DefaultSegment extends Segment {
        constructor(lower, parent, base) {
            super(lower, elasticlunr.stemmer(lower), parent, base);
        }

        get wordCount() {
            return 1;
        }
    }

    function makeTeaser(doc, searchTerms) {
        const body = new StructuredText(doc.body), breadcrumbs = new StructuredText(doc.breadcrumbs),
            requireMatchAll = search_options.bool === 'AND', matchesInBody = [], matchesInBreadcrumbs = [];
        var termCountInBody = 0;
        for (const [index, regex] of searchTerms.regex.entries()) {
            const currentTermInBody = [];
            for (const match of body.stemmed.matchAll(regex)) {
                currentTermInBody.push({
                    begin: match.index, end: match.index + match[0].length, index
                });
            }
            const currentTermInBreadcrumbs = [];
            for (const match of breadcrumbs.stemmed.matchAll(regex)) {
                currentTermInBreadcrumbs.push({
                    begin: match.index, end: match.index + match[0].length
                });
            }
            if (currentTermInBody.length) {
                termCountInBody++;
            } else if (requireMatchAll && !currentTermInBreadcrumbs.length) {
                return;
            }
            matchesInBody.push(...currentTermInBody);
            matchesInBreadcrumbs.push(...currentTermInBreadcrumbs);
        }
        if (!termCountInBody && !matchesInBreadcrumbs.length) return;
        matchesInBreadcrumbs.sort((a, b) => a.begin - b.begin);

        if (!matchesInBody.length) {
            const range = {begin: 0, end: 0};
            body.expandEnd(range, results_options.teaser_word_count);
            var highlightedBody = body.highlightAndEscapeByStemmed(matchesInBody, [range]);
            return {
                body: highlightedBody,
                breadcrumbs: breadcrumbs.highlightAndEscapeByStemmed(matchesInBreadcrumbs)
            };
        }
        matchesInBody.sort((a, b) => a.begin - b.begin);

        // Find the minimum window that contains at least one occurrence of each search term.
        // `matches` is an array of { begin: number, end: number, index: number } where index is the index of search term.
        // `termCount` is the number of unique search terms occurred.
        function minWindow(matches, termCount) {
            var begin = 0, end = 0, termCountInRange = 0, result = {begin: 0, end: body.stemmed.length};
            const termCountTableInRange = [];

            // Contract window's begin until it no longer contains all keywords
            function contractWindow() {
                while (true) {
                    const index = matches[begin].index;
                    begin++;
                    termCountTableInRange[index]--;
                    if (!termCountTableInRange[index]) {
                        const currentWindow = {
                            begin: matches[begin - 1].begin, end: matches[end - 1].end
                        };
                        if (currentWindow.end - currentWindow.begin < result.end - result.begin) result = currentWindow;
                        break;
                    }
                }
            }

            // Expand window's end until it contains all keywords
            while (end < matches.length) {
                const index = matches[end].index;
                end++;
                if (termCountTableInRange[index]) {
                    termCountTableInRange[index]++;
                } else {
                    termCountTableInRange[index] = 1;
                    termCountInRange++;
                    if (termCountInRange >= termCount) {
                        contractWindow();
                        break;
                    }
                }
            }

            // Expand window's end until it contains all keywords again
            while (end < matches.length) {
                const index = matches[end].index;
                end++;
                termCountTableInRange[index]++;
                if (termCountTableInRange[index] === 1) contractWindow();
            }
            return result;
        }

        const range = minWindow(matchesInBody, termCountInBody);
        const rawBegin = range.begin, rawEnd = range.end;
        body.expandBeginToBoundary(range, Sentence);
        body.expandEndToBoundary(range, Sentence);

        const wordCountLimit = results_options.teaser_word_count;
        var ranges = [], wordCount = body.wordCount(range.begin, range.end);
        if (wordCount < wordCountLimit) {
            var oldBegin, oldWordCount;
            do {
                oldBegin = range.begin;
                oldWordCount = wordCount;
                const reachedLimit = body.expandBeginToBoundary(range, Sentence);
                wordCount = body.wordCount(range.begin, range.end);
                if (reachedLimit) break;
            } while (wordCount < wordCountLimit);
            if (wordCount > wordCountLimit) {
                range.begin = oldBegin;
                wordCount = oldWordCount;
            }
            if (wordCount < wordCountLimit) {
                const remainingWordCount = body.expandEnd(range, wordCountLimit - wordCount);
                if (remainingWordCount) body.expandBegin(range, remainingWordCount);
            }
            ranges.push(range);
        } else if (wordCount === wordCountLimit) {
            ranges.push(range);
        } else {
            // When `range` can't be shrunk to `wordCountLimit`, the actual wordCount is returned.
            function tryShrink(range, wordCount, wordCountLimit, rawBegin, rawEnd) {
                var oldEnd;
                do {
                    oldEnd = range.end;
                    if (body.shrinkEndToBoundary(range, Clause, rawEnd)) {
                        range.end = oldEnd;
                        break;
                    }
                    wordCount = body.wordCount(range.begin, range.end);
                } while (wordCount > wordCountLimit);
                if (wordCount <= wordCountLimit) {
                    if (wordCount < wordCountLimit) body.expandEnd(range, wordCountLimit - wordCount);
                    ranges.push(range);
                } else {
                    var oldBegin;
                    do {
                        oldBegin = range.begin;
                        if (body.shrinkBeginToBoundary(range, Clause, rawBegin)) {
                            range.begin = oldBegin;
                            break;
                        }
                        wordCount = body.wordCount(range.begin, range.end);
                    } while (wordCount > wordCountLimit);
                    if (wordCount > wordCountLimit) return wordCount;
                    if (wordCount < wordCountLimit) body.expandBegin(range, wordCountLimit - wordCount);
                }
            }
            wordCount = tryShrink(range, wordCount, wordCountLimit, rawBegin, rawEnd);
            if (!wordCount) {
                ranges.push(range);
            } else {
                // split the result into pieces and shrink them individually, then join them with ……
                var freshMatchesInBody = matchesInBody.filter(match => match.begin >= range.begin && match.end <= range.end);
                for (const sentence of body.sentences) {
                    const sentenceEnd = sentence.stemmed.end;
                    if (sentenceEnd > freshMatchesInBody[0].begin) {
                        ranges.push({begin: sentence.stemmed.begin, end: sentenceEnd});
                        const currentIndex = freshMatchesInBody[0].index;
                        freshMatchesInBody = freshMatchesInBody.filter(match => match.index !== currentIndex);
                        while (freshMatchesInBody.length && sentenceEnd > freshMatchesInBody[0].begin) {
                            const currentIndex = freshMatchesInBody[0].index;
                            freshMatchesInBody = freshMatchesInBody.filter(match => match.index !== currentIndex);
                        }
                        if (!freshMatchesInBody.length) break;
                    }
                }
                const wordCountList = ranges.map(range => body.wordCount(range.begin, range.end));
                wordCount = wordCountList.reduce((sum, wordCount) => sum + wordCount);
                var exceedingWordCount = wordCount - wordCountLimit;
                if (exceedingWordCount < 0) {
                    var remainingWordCount = wordCountLimit - wordCount;
                    for (var i = 0; i < ranges.length - 1; i++) {
                        remainingWordCount = body.expandEnd(ranges[i], remainingWordCount, ranges[i + 1].begin);
                        if (!remainingWordCount) break;
                    }
                    if (remainingWordCount) body.expandEnd(ranges[i], remainingWordCount);
                } else if (exceedingWordCount > 0) {
                    const reversedMatchesInBody = [...matchesInBody];
                    reversedMatchesInBody.sort((a, b) => b.end - a.end);
                    for (i = ranges.length - 1; i >= 0; i--) {
                        const range = ranges[i];
                        const actualWordCount = tryShrink(range, wordCountList[i], wordCountList[i] - exceedingWordCount,
                            matchesInBody.find(match => match.begin >= range.begin).begin,
                            reversedMatchesInBody.find(match => match.end <= range.end).end);
                        if (!actualWordCount) break;
                        exceedingWordCount -= wordCountList[i] - actualWordCount;
                    }
                }
            }
        }
        return {
            body: body.highlightAndEscapeByStemmed(matchesInBody, ranges),
            breadcrumbs: breadcrumbs.highlightAndEscapeByStemmed(matchesInBreadcrumbs)
        };
    }

    function init(config) {
        results_options = config.results_options;
        search_options = config.search_options;
        searchbar_outer = config.searchbar_outer;
        doc_urls = config.doc_urls;
        searchindex = elasticlunr.Index.load(config.index);

        // Set up events
        searchicon.addEventListener('click', function(e) { searchIconClickHandler(); }, false);
        searchbar.addEventListener('keyup', function(e) { searchbarKeyUpHandler(); }, false);
        document.addEventListener('keydown', function(e) { globalKeyHandler(e); }, false);
        // If the user uses the browser buttons, do the same as if a reload happened
        window.onpopstate = function(e) { doSearchOrMarkFromUrl(); };
        // Suppress "submit" events so the page doesn't reload when the user presses Enter
        document.addEventListener('submit', function(e) { e.preventDefault(); }, false);

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
                (url.params[URL_SEARCH_PARAM] + '').replace(/\+/g, '%20'));
            searchbarKeyUpHandler(); // -> doSearch()
        } else {
            showSearch(false);
        }

        if (url.params.hasOwnProperty(URL_MARK_PARAM)) {
            var words = decodeURIComponent(url.params[URL_MARK_PARAM]).split(' ');
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
        if (e.altKey || e.ctrlKey || e.metaKey || e.shiftKey || e.target.type === 'textarea' || e.target.type === 'text' || !hasFocus() && /^(?:input|select|textarea)$/i.test(e.target.nodeName)) { return; }

        if (e.keyCode === ESCAPE_KEYCODE) {
            e.preventDefault();
            searchbar.classList.remove("active");
            setSearchUrlParameters("",
                (searchbar.value.trim() !== "") ? "push" : "replace");
            if (hasFocus()) {
                unfocusSearchbar();
            }
            showSearch(false);
            marker.unmark();
        } else if (!hasFocus() && e.keyCode === SEARCH_HOTKEY_KEYCODE) {
            e.preventDefault();
            showSearch(true);
            window.scrollTo(0, 0);
            searchbar.select();
        } else if (hasFocus() && e.keyCode === DOWN_KEYCODE) {
            e.preventDefault();
            unfocusSearchbar();
            searchresults.firstElementChild.classList.add("focus");
        } else if (!hasFocus() && (e.keyCode === DOWN_KEYCODE
            || e.keyCode === UP_KEYCODE
            || e.keyCode === SELECT_KEYCODE)) {
            // not `:focus` because browser does annoying scrolling
            var focused = searchresults.querySelector("li.focus");
            if (!focused) return;
            e.preventDefault();
            if (e.keyCode === DOWN_KEYCODE) {
                var next = focused.nextElementSibling;
                if (next) {
                    focused.classList.remove("focus");
                    next.classList.add("focus");
                }
            } else if (e.keyCode === UP_KEYCODE) {
                focused.classList.remove("focus");
                var prev = focused.previousElementSibling;
                if (prev) {
                    prev.classList.add("focus");
                } else {
                    searchbar.select();
                }
            } else { // SELECT_KEYCODE
                window.location.assign(focused.querySelector('a'));
            }
        }
    }

    function showSearch(yes) {
        if (yes) {
            search_wrap.classList.remove('hidden');
            searchicon.setAttribute('aria-expanded', 'true');
        } else {
            search_wrap.classList.add('hidden');
            searchicon.setAttribute('aria-expanded', 'false');
            var results = searchresults.children;
            for (var i = 0; i < results.length; i++) {
                results[i].classList.remove("focus");
            }
        }
    }

    function showResults(yes) {
        if (yes) {
            searchresults_outer.classList.remove('hidden');
        } else {
            searchresults_outer.classList.add('hidden');
        }
    }

    // Eventhandler for search icon
    function searchIconClickHandler() {
        if (search_wrap.classList.contains('hidden')) {
            showSearch(true);
            window.scrollTo(0, 0);
            searchbar.select();
        } else {
            showSearch(false);
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
        var first_search = !url.params.hasOwnProperty(URL_SEARCH_PARAM);
        if (searchterm != "" || action == "push_if_new_search_else_replace") {
            url.params[URL_SEARCH_PARAM] = searchterm;
            delete url.params[URL_MARK_PARAM];
            url.hash = "";
        } else {
            delete url.params[URL_MARK_PARAM];
            delete url.params[URL_SEARCH_PARAM];
        }
        // A new search will also add a new history item, so the user can go back
        // to the page prior to searching. A updated search term will only replace
        // the url.
        if (action == "push" || (action == "push_if_new_search_else_replace" && first_search)) {
            history.pushState({}, document.title, renderURL(url));
        } else if (action == "replace" || (action == "push_if_new_search_else_replace" && !first_search)) {
            history.replaceState({}, document.title, renderURL(url));
        }
    }

    function preprocessSearchTerms(searchTerms) {
        const original = searchTerms.split(REGEX_WHITE_SPACE);
        const stemmed = original.map(term => term.toLowerCase().replace(REGEX_STEM, (match, english) => english ? elasticlunr.stemmer(match) : match));
        return {
            original,
            stemmed,
            lunr: searchTerms.replace(REGEX_SEARCH_SPLITTER, (_, word) => word ? `${word} ` : ""),
            regex: stemmed.map(term => {
                var escaped = term.replace(REGEX_ESCAPE, '\\$&');
                if (REGEX_DEFAULT_BEGIN.test(term)) {
                    escaped = "(?<![^\\p{White_Space}\\p{P}\\p{Sm}\\p{Sc}\\p{So}\\p{Unified_Ideograph}\\uAC00-\\uD7AF\\p{Z}\\p{C}])" + escaped;
                }
                if (REGEX_DEFAULT_END.test(term)) {
                    escaped += search_options.expand ? "[^\\p{White_Space}\\p{P}\\p{Sm}\\p{Sc}\\p{So}\\p{Unified_Ideograph}\\uAC00-\\uD7AF\\p{Z}\\p{C}]*" : "(?![^\\p{White_Space}\\p{P}\\p{Sm}\\p{Sc}\\p{So}\\p{Unified_Ideograph}\\uAC00-\\uD7AF\\p{Z}\\p{C}])";
                }
                return new RegExp(escaped, 'gu');
            }),
            // encodeURIComponent escapes all chars that could allow an XSS except
            // for '. Due to that we also manually replace ' with its url-encoded
            // representation (%27).
            url: encodeURIComponent(searchTerms.replace(/\'/g, "%27"))
        };
    }

    function doSearch(searchterm) {

        // Don't search the same twice
        if (current_searchterm == searchterm) { return; }
        else { current_searchterm = searchterm; }

        if (searchindex == null) { return; }

        const searchTerms = preprocessSearchTerms(searchterm);

        // Do the actual search
        var results = searchindex.search(searchTerms.lunr, search_options);

        // Clear and insert results
        removeChildren(searchresults);
        var resultCount = 0;
        for (const result of results) {
            const resultHtml = formatSearchResult(result, searchTerms);
            if (!resultHtml) continue;
            var resultElem = document.createElement('li');
            resultElem.innerHTML = resultHtml;
            searchresults.appendChild(resultElem);
            resultCount++;
            if (resultCount >= results_options.limit_results) break;
        }

        // Display search metrics
        searchresults_header.innerText = formatSearchMetric(resultCount, searchterm);

        // Display results
        showResults(true);
    }

    fetch(path_to_root + 'searchindex.json')
        .then(response => response.json())
        .then(json => init(json))
        .catch(error => { // Try to load searchindex.js if fetch failed
            var script = document.createElement('script');
            script.src = path_to_root + 'searchindex.js';
            script.onload = () => init(window.search);
            document.head.appendChild(script);
        });

    // Exported functions
    search.hasFocus = hasFocus;
})(window.search);
