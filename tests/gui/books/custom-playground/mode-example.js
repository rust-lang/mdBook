ace.define("ace/mode/example",
    ["require", "exports", "ace/lib/oop", "ace/mode/text", "ace/mode/text_highlight_rules"],
    function(require, exports) {
        const oop = require("../lib/oop");
        const TextMode = require("./text").Mode;
        const TextHighlightRules = require("./text_highlight_rules").TextHighlightRules;
        const Rules = function() {
            this.$rules = { start: [{ token: "keyword", regex: "\\bprint\\b" }] };
        };
        oop.inherits(Rules, TextHighlightRules);
        const Mode = function() { this.HighlightRules = Rules; };
        oop.inherits(Mode, TextMode);
        exports.Mode = Mode;
    });
