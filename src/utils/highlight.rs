//! Syntax highlighter with support for hiding lines.
//! This is essentially a version of [`syntect::html::ClassedHTMLGenerator`]
//! which allows you to mark a line as boring (hidden).

use std::borrow::Cow;

use regex::Regex;
use syntect::{
    html::{self, ClassStyle},
    parsing::{ParseState, Scope, ScopeStack, ScopeStackOp, SyntaxReference, SyntaxSet},
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
        let parsed_line = if did_boringify {
            // The empty scope is a valid prefix of every other scope.
            // If we tried to just use a scope called "boring", we'd need to modify
            // the Rust syntax definition.
            let boring = Scope::new("").expect("boring is a valid scope");
            // Close all open spans, insert `boring`, then re-open all of them.
            // `boring` must be at the very top, so that the parser doesn't touch it.
            let mut final_parsed_line = Vec::new();
            if self.scope_stack.len() != 0 {
                final_parsed_line.push((0, ScopeStackOp::Pop(self.scope_stack.len())));
            }
            final_parsed_line.push((0, ScopeStackOp::Push(boring.clone())));
            for item in &self.scope_stack.scopes {
                final_parsed_line.push((0, ScopeStackOp::Push(item.clone())));
            }
            // Now run the parser.
            // It should see basically the stack it expects, except the `boring` at the very top,
            // which it shouldn't touch because it doesn't know it's there.
            let inner_parsed_line = self.parse_state.parse_line(&line, self.syntaxes);
            final_parsed_line.extend_from_slice(&inner_parsed_line);
            // Figure out what the final stack is.
            let mut stack_at_end = self.scope_stack.clone();
            for (_, item) in inner_parsed_line {
                stack_at_end.apply(&item);
            }
            // Pop everything, including `boring`.
            final_parsed_line.push((line.len(), ScopeStackOp::Pop(stack_at_end.len() + 1)));
            // Push all the state back on at the end.
            for item in stack_at_end.scopes.into_iter() {
                final_parsed_line.push((line.len(), ScopeStackOp::Push(item)));
            }
            final_parsed_line
        } else {
            self.parse_state.parse_line(&line, self.syntaxes)
        };
        let (mut formatted_line, delta) = html::line_tokens_to_classed_spans(
            &line,
            parsed_line.as_slice(),
            self.style,
            &mut self.scope_stack,
        );
        if did_boringify {
            // Since the boring scope is preceded only by a Pop operation,
            // it must be the first match on the line for <span class="">
            formatted_line =
                formatted_line.replace(r#"<span class="">"#, r#"<span class="boring">"#);
        }
        self.open_spans += delta;
        self.html.push_str(&formatted_line);
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
