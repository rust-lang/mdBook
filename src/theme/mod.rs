
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
