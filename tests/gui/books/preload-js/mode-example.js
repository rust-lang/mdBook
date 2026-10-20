// mode-example.js
ace.define("ace/mode/example",
    ["require", "exports", "ace/lib/oop", "ace/mode/text", "ace/mode/text_highlight_rules"],
    function(require, exports) {
        var oop = require("../lib/oop");
        var TextMode = require("./text").Mode;
        var TextHighlightRules = require("./text_highlight_rules").TextHighlightRules;
        var Rules = function() {
            this.$rules = { start: [{ token: "keyword", regex: "\\bprint\\b" }] };
        };
        oop.inherits(Rules, TextHighlightRules);
        var Mode = function() { this.HighlightRules = Rules; };
        oop.inherits(Mode, TextMode);
        exports.Mode = Mode;
    });
