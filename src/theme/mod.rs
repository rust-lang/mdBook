
pub fn get_index_hbs() -> &'static str {
    let index = include_str!("index.hbs");
    index
}

pub fn get_css() -> &'static str {
    let css = include_str!("book.css");
    css
}

pub fn get_js() -> &'static str {
    let js = include_str!("book.js");
    js
}
