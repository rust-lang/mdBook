
pub fn get_index_hbs() -> &'static str {
    let index = include_str!("index.hbs");
    index
}

pub fn get_css() -> &'static [u8] {
    let css = include_bytes!("book.css");
    css
}

pub fn get_js() -> &'static [u8] {
    let js = include_bytes!("book.js");
    js
}

pub fn get_highlight_js() -> &'static [u8] {
    let highlight_js = include_bytes!("highlight.js");
    highlight_js
}

pub fn get_highlight_css() -> &'static [u8] {
    let highlight_css = include_bytes!("highlight.css");
    highlight_css
}
