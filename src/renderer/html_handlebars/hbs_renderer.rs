extern crate handlebars;
extern crate rustc_serialize;
extern crate pulldown_cmark;

use renderer::html_handlebars::helpers;
use renderer::Renderer;
use book::MDBook;
use book::bookitem::BookItem;
use {utils, theme};

use std::path::{Path, PathBuf};
use std::fs::File;
use std::error::Error;
use std::io::{self, Read, Write};
use std::collections::BTreeMap;

use self::handlebars::{Handlebars, JsonRender};
use self::rustc_serialize::json::{Json, ToJson};
use self::pulldown_cmark::{Parser, html};

pub struct HtmlHandlebars;

impl HtmlHandlebars {
    pub fn new() -> Self {
        HtmlHandlebars
    }
}

impl Renderer for HtmlHandlebars {
    fn render(&self, book: &MDBook) -> Result<(), Box<Error>> {
        debug!("[fn]: render");
        let mut handlebars = Handlebars::new();

        // Load theme
        let theme = theme::Theme::new(book.get_src());

        // Register template
        debug!("[*]: Register handlebars template");
        try!(handlebars.register_template_string("index", try!(String::from_utf8(theme.index))));

        // Register helpers
        debug!("[*]: Register handlebars helpers");
        handlebars.register_helper("toc", Box::new(helpers::toc::RenderToc));
        handlebars.register_helper("previous", Box::new(helpers::navigation::previous));
        handlebars.register_helper("next", Box::new(helpers::navigation::next));

        let mut data = try!(make_data(book));

        // Print version
        let mut print_content: String = String::new();

        // Check if dest directory exists
        debug!("[*]: Check if destination directory exists");
        match utils::create_path(book.get_dest()) {
            Err(_) => return Err(Box::new(io::Error::new(io::ErrorKind::Other, "Unexcpected error when constructing destination path"))),
            _ => {},
        };

        // Render a file for every entry in the book
        let mut index = true;
        for item in book.iter() {

            match item {
                &BookItem::Chapter(_, ref ch) | &BookItem::Affix(ref ch) => {
                    if ch.path != PathBuf::new() {

                        let path = book.get_src().join(&ch.path);

                        debug!("[*]: Opening file: {:?}", path);
                        let mut f = try!(File::open(&path));
                        let mut content: String = String::new();

                        debug!("[*]: Reading file");
                        try!(f.read_to_string(&mut content));

                        // Render markdown using the pulldown-cmark crate
                        content = render_html(&content);
                        print_content.push_str(&content);

                        // Remove content from previous file and render content for this one
                        data.remove("path");
                        match ch.path.to_str() {
                            Some(p) => { data.insert("path".to_string(), p.to_json()); },
                            None => return Err(Box::new(io::Error::new(io::ErrorKind::Other, "Could not convert path to str"))),
                        }


                        // Remove content from previous file and render content for this one
                        data.remove("content");
                        data.insert("content".to_string(), content.to_json());

                        // Remove path to root from previous file and render content for this one
                        data.remove("path_to_root");
                        data.insert("path_to_root".to_string(), utils::path_to_root(&ch.path).to_json());

                        // Rendere the handlebars template with the data
                        debug!("[*]: Render template");
                        let rendered = try!(handlebars.render("index", &data));

                        debug!("[*]: Create file {:?}", &book.get_dest().join(&ch.path).with_extension("html"));
                        // Write to file
                        let mut file = try!(utils::create_file(&book.get_dest().join(&ch.path).with_extension("html")));
                        output!("[*] Creating {:?} ✓", &book.get_dest().join(&ch.path).with_extension("html"));

                        try!(file.write_all(&rendered.into_bytes()));

                        // Create an index.html from the first element in SUMMARY.md
                        if index {
                            debug!("[*]: index.html");

                            let mut index_file = try!(File::create(book.get_dest().join("index.html")));
                            let mut content = String::new();
                            let _source = try!(File::open(book.get_dest().join(&ch.path.with_extension("html"))))
                                                        .read_to_string(&mut content);

                            // This could cause a problem when someone displays code containing <base href=...>
                            // on the front page, however this case should be very very rare...
                            content = content.lines().filter(|line| !line.contains("<base href=")).collect();

                            try!(index_file.write_all(content.as_bytes()));

                            output!(
                                "[*] Creating index.html from {:?} ✓",
                                book.get_dest().join(&ch.path.with_extension("html"))
                                );
                            index = false;
                        }
                    }
                }
                _ => {}
            }
        }

        // Print version

        // Remove content from previous file and render content for this one
        data.remove("path");
        data.insert("path".to_string(), "print.md".to_json());

        // Remove content from previous file and render content for this one
        data.remove("content");
        data.insert("content".to_string(), print_content.to_json());

        // Remove path to root from previous file and render content for this one
        data.remove("path_to_root");
        data.insert("path_to_root".to_string(), utils::path_to_root(Path::new("print.md")).to_json());

        // Rendere the handlebars template with the data
        debug!("[*]: Render template");
        let rendered = try!(handlebars.render("index", &data));
        let mut file = try!(utils::create_file(&book.get_dest().join("print").with_extension("html")));
        try!(file.write_all(&rendered.into_bytes()));
        output!("[*] Creating print.html ✓");

        // Copy static files (js, css, images, ...)

        debug!("[*] Copy static files");
        // JavaScript
        let mut js_file = try!(File::create(book.get_dest().join("book.js")));
        try!(js_file.write_all(&theme.js));

        // Css
        let mut css_file = try!(File::create(book.get_dest().join("book.css")));
        try!(css_file.write_all(&theme.css));

        // JQuery local fallback
        let mut jquery = try!(File::create(book.get_dest().join("jquery.js")));
        try!(jquery.write_all(&theme.jquery));

        // Font Awesome local fallback
        let mut font_awesome = try!(utils::create_file(&book.get_dest().join("_FontAwesome/css/font-awesome").with_extension("css")));
        try!(font_awesome.write_all(theme::FONT_AWESOME));
        let mut font_awesome = try!(utils::create_file(&book.get_dest().join("_FontAwesome/fonts/fontawesome-webfont.eot")));
        try!(font_awesome.write_all(theme::FONT_AWESOME_EOT));
        let mut font_awesome = try!(utils::create_file(&book.get_dest().join("_FontAwesome/fonts/fontawesome-webfont.svg")));
        try!(font_awesome.write_all(theme::FONT_AWESOME_SVG));
        let mut font_awesome = try!(utils::create_file(&book.get_dest().join("_FontAwesome/fonts/fontawesome-webfont.ttf")));
        try!(font_awesome.write_all(theme::FONT_AWESOME_TTF));
        let mut font_awesome = try!(utils::create_file(&book.get_dest().join("_FontAwesome/fonts/fontawesome-webfont.woff")));
        try!(font_awesome.write_all(theme::FONT_AWESOME_WOFF));
        let mut font_awesome = try!(utils::create_file(&book.get_dest().join("_FontAwesome/fonts/fontawesome-webfont.woff2")));
        try!(font_awesome.write_all(theme::FONT_AWESOME_WOFF2));
        let mut font_awesome = try!(utils::create_file(&book.get_dest().join("_FontAwesome/fonts/FontAwesome.ttf")));
        try!(font_awesome.write_all(theme::FONT_AWESOME_TTF));

        // syntax highlighting
        let mut highlight_css = try!(File::create(book.get_dest().join("highlight.css")));
        try!(highlight_css.write_all(&theme.highlight_css));
        let mut tomorrow_night_css = try!(File::create(book.get_dest().join("tomorrow-night.css")));
        try!(tomorrow_night_css.write_all(&theme.tomorrow_night_css));
        let mut highlight_js = try!(File::create(book.get_dest().join("highlight.js")));
        try!(highlight_js.write_all(&theme.highlight_js));

        Ok(())
    }
}

fn make_data(book: &MDBook) -> Result<BTreeMap<String,Json>, Box<Error>> {
    debug!("[fn]: make_data");

    let mut data  = BTreeMap::new();
    data.insert("language".to_string(), "en".to_json());
    data.insert("title".to_string(), book.get_title().to_json());

    let mut chapters = vec![];

    for item in book.iter() {
        // Create the data to inject in the template
        let mut chapter = BTreeMap::new();

        match item {
            &BookItem::Affix(ref ch) => {
                chapter.insert("name".to_string(), ch.name.to_json());
                match ch.path.to_str() {
                    Some(p) => { chapter.insert("path".to_string(), p.to_json()); },
                    None => return Err(Box::new(io::Error::new(io::ErrorKind::Other, "Could not convert path to str"))),
                }
            },
            &BookItem::Chapter(ref s, ref ch) => {
                chapter.insert("section".to_string(), s.to_json());
                chapter.insert("name".to_string(), ch.name.to_json());
                match ch.path.to_str() {
                    Some(p) => { chapter.insert("path".to_string(), p.to_json()); },
                    None => return Err(Box::new(io::Error::new(io::ErrorKind::Other, "Could not convert path to str"))),
                }
            },
            &BookItem::Spacer => {
                chapter.insert("spacer".to_string(), "_spacer_".to_json());
            }

        }

        chapters.push(chapter);
    }

    data.insert("chapters".to_string(), chapters.to_json());

    debug!("[*]: JSON constructed");
    Ok(data)
}

fn render_html(text: &str) -> String {
    let mut s = String::with_capacity(text.len() * 3 / 2);
    let p = Parser::new(&text);
    html::push_html(&mut s, p);
    s
}
