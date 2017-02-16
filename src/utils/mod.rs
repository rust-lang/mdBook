pub mod fs;

use pulldown_cmark::{Parser, html, Options, OPTION_ENABLE_TABLES, OPTION_ENABLE_FOOTNOTES};


///
///
/// Wrapper around the pulldown-cmark parser and renderer to render markdown

pub fn render_markdown(text: &str) -> String {
    let mut s = String::with_capacity(text.len() * 3 / 2);

    let mut opts = Options::empty();
    opts.insert(OPTION_ENABLE_TABLES);
    opts.insert(OPTION_ENABLE_FOOTNOTES);

    let p = Parser::new_ext(text, opts);
    html::push_html(&mut s, p);
    s
}
