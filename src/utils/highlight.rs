//! Syntax highlighter with support for hiding lines.
//! This is essentially a version of [`syntect::html::ClassedHTMLGenerator`]
//! which allows you to mark a line as boring (hidden).

use std::borrow::Cow;

use regex::Regex;
use syntect::{
    html::{self, ClassStyle},
    parsing::{ParseState, ScopeStack, SyntaxReference, SyntaxSet},
};

pub struct HtmlGenerator<'a> {
    syntaxes: &'a SyntaxSet,
    open_spans: isize,
    parse_state: ParseState,
    scope_stack: ScopeStack,
    html: String,
    style: ClassStyle,
}

impl<'a> HtmlGenerator<'a> {
    pub fn new(syntax: &'a SyntaxReference, syntaxes: &'a SyntaxSet, style: ClassStyle) -> Self {
        let parse_state = ParseState::new(syntax);
        let open_spans = 0;
        let html = String::new();
        let scope_stack = ScopeStack::new();
        Self {
            syntaxes,
            open_spans,
            parse_state,
            scope_stack,
            html,
            style,
        }
    }

    pub fn parse_line(&mut self, line: &str, is_rust: bool) {
        let (line, did_boringify) = if is_rust {
            let (line, did_boringify) = boringify(line);
            (Cow::from(line), did_boringify)
        } else {
            (Cow::from(line), false)
        };
        let parsed_line = self.parse_state.parse_line(&line, self.syntaxes);
        let (formatted_line, delta) = html::line_tokens_to_classed_spans(
            &line,
            parsed_line.as_slice(),
            self.style,
            &mut self.scope_stack,
        );
        self.open_spans += delta;
        self.html.push_str(&if did_boringify {
            format!("<span class=\"boring\">{}</span>", formatted_line)
        } else {
            formatted_line
        });
    }

    pub fn finalize(mut self) -> String {
        for _ in 0..self.open_spans {
            self.html.push_str("</span>");
        }
        self.html
    }
}

lazy_static! {
    static ref BORING_LINE_REGEX: Regex = Regex::new(r"^(\s*)#(.?)(.*)\n$").unwrap();
}

fn boringify(line: &str) -> (String, bool) {
    let mut result = String::with_capacity(line.len());
    if let Some(caps) = BORING_LINE_REGEX.captures(line) {
        if &caps[2] == "#" {
            result += &caps[1];
            result += &caps[2];
            result += &caps[3];
            result += "\n";
            return (result, true);
        } else if &caps[2] != "!" && &caps[2] != "[" {
            result += &caps[1];
            if &caps[2] != " " {
                result += &caps[2];
            }
            result += &caps[3];
            result += "\n";
            return (result, true);
        }
    }
    result += line;
    result += "\n";

    (result, false)
}
