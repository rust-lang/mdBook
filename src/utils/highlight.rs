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
    is_in_boring_chunk: bool,
}

impl<'a> HtmlGenerator<'a> {
    pub fn new(syntax: &'a SyntaxReference, syntaxes: &'a SyntaxSet, style: ClassStyle) -> Self {
        let parse_state = ParseState::new(syntax);
        let open_spans = 0;
        let html = String::new();
        let scope_stack = ScopeStack::new();
        let is_in_boring_chunk = false;
        Self {
            syntaxes,
            open_spans,
            parse_state,
            scope_stack,
            html,
            style,
            is_in_boring_chunk,
        }
    }

    pub fn parse_line(&mut self, line: &str, is_rust: bool) {
        let (line, did_boringify) = if is_rust {
            let (line, did_boringify) = boringify(line);
            (Cow::from(line), did_boringify)
        } else {
            (Cow::from(line), false)
        };
        let parsed_line = match (self.is_in_boring_chunk, did_boringify) {
            (false, true) => {
                let mut final_parsed_line = Vec::new();
                let inner_parsed_line = self.parse_state.parse_line(&line, self.syntaxes);
                // The empty scope is a valid prefix of every other scope.
                // If we tried to just use a scope called "boring", we'd need to modify
                // the Rust syntax definition.
                let boring = Scope::new("").expect("boring is a valid scope");
                // Close all open spans, insert `boring`, then re-open all of them.
                // `boring` must be at the very top, so that the parser doesn't touch it.
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
                final_parsed_line.extend(inner_parsed_line);
                final_parsed_line
            }
            (true, false) => {
                let mut final_parsed_line = Vec::new();
                let inner_parsed_line = self.parse_state.parse_line(&line, self.syntaxes);
                // Pop everything, including `boring`, which was passed to `line_tokens_to_classed_spans`,
                // and therefore wound up on the scope stack.
                final_parsed_line.push((0, ScopeStackOp::Pop(self.scope_stack.len())));
                // Push all the state back on at the end.
                for (i, item) in self.scope_stack.scopes.iter().enumerate() {
                    if i == 0 {
                        // The whole point of this elaborate work is to pop the boring scope off the stack,
                        // which requires popping everything else since it's on top,
                        // then to avoid pushing it back on again.
                        assert!(item.is_empty());
                    } else {
                        final_parsed_line.push((0, ScopeStackOp::Push(item.clone())));
                    }
                }
                // Since this line is not boringified, we need to first break out of the boring scope,
                // then we actually put all of this line's tokens.
                final_parsed_line.extend(inner_parsed_line);
                final_parsed_line
            }
            _ => self.parse_state.parse_line(&line, self.syntaxes),
        };
        let (mut formatted_line, delta) = html::line_tokens_to_classed_spans(
            &line,
            parsed_line.as_slice(),
            self.style,
            &mut self.scope_stack,
        );
        if did_boringify && !self.is_in_boring_chunk {
            // Since the boring scope is preceded only by a Pop operation,
            // it must be the first match on the line for <span class="">
            formatted_line =
                formatted_line.replace(r#"<span class="">"#, r#"<span class="boring">"#);
        }
        if did_boringify {
            // If we're in a boringify-ed line, then we must have an empty scope,
            // which I call "the boring scope", at the very top of the scope stack.
            assert!(self
                .scope_stack
                .scopes
                .first()
                .map(Scope::is_empty)
                .unwrap_or(false));
        } else if is_rust {
            // Otherwise, since the rust syntax definition doesn't use empty scopes,
            // then it shall not be there.
            assert!(!self
                .scope_stack
                .scopes
                .first()
                .map(Scope::is_empty)
                .unwrap_or(false));
        }
        self.is_in_boring_chunk = did_boringify;
        self.open_spans += delta;
        self.html.push_str(&formatted_line);
    }

    pub fn finalize(mut self) -> String {
        for _ in 0..self.open_spans {
            self.html.push_str("</span>");
        }
        // Since the boring scope is passed to `line_tokens_to_classed_spans`,
        // which is responsible for computing the delta, it was accounted for
        // in the `open_spans` count.
        self.is_in_boring_chunk = false;
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

    (result, false)
}
