extern crate handlebars;
extern crate rustc_serialize;
extern crate pulldown_cmark;

use renderer::html_handlebars::{helpers, theme};
use renderer::Renderer;
use book::{BookItems, BookConfig};
use utils;

use std::path::PathBuf;
use std::fs::{self, File};
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
    fn render(&self, book: BookItems, config: &BookConfig) -> Result<(), Box<Error>> {
        debug!("[fn]: render");
        let mut handlebars = Handlebars::new();

        // Load theme
        let theme = theme::Theme::new(&config.src());

        // Register template
        debug!("[*]: Register handlebars template");
        try!(handlebars.register_template_string("index", theme.index.to_owned()));

        // Register helpers
        debug!("[*]: Register handlebars helpers");
        handlebars.register_helper("toc", Box::new(helpers::toc::RenderToc));
        handlebars.register_helper("previous", Box::new(helpers::navigation::previous));
        handlebars.register_helper("next", Box::new(helpers::navigation::next));

        let mut data = try!(make_data(book.clone(), config));

        // Check if dest directory exists
        debug!("[*]: Check if destination directory exists");
        match utils::path::create_path(config.dest()) {
            Err(_) => return Err(Box::new(io::Error::new(io::ErrorKind::Other, "Unexcpected error when constructing destination path"))),
            _ => {},
        };

        // Render a file for every entry in the book
        let mut index = true;
        for (_, item) in book {

            if item.path != PathBuf::new() {

                let path = config.src().join(&item.path);

                debug!("[*]: Opening file: {:?}", path);
                let mut f = try!(File::open(&path));
                let mut content: String = String::new();

                debug!("[*]: Reading file");
                try!(f.read_to_string(&mut content));

                // Render markdown using the pulldown-cmark crate
                content = render_html(&content);

                // Remove content from previous file and render content for this one
                data.remove("path");
                data.insert("path".to_string(), item.path.to_str().unwrap().to_json());

                // Remove content from previous file and render content for this one
                data.remove("content");
                data.insert("content".to_string(), content.to_json());

                // Remove path to root from previous file and render content for this one
                data.remove("path_to_root");
                data.insert("path_to_root".to_string(), utils::path::path_to_root(&item.path).to_json());

                // Rendere the handlebars template with the data
                debug!("[*]: Render template");
                let rendered = try!(handlebars.render("index", &data));

                debug!("[*]: Create file {:?}", &config.dest().join(&item.path).with_extension("html"));
                // Write to file
                let mut file = try!(utils::path::create_file(&config.dest().join(&item.path).with_extension("html")));
                output!("[*] Creating {:?} ✓", &config.dest().join(&item.path).with_extension("html"));

                try!(file.write_all(&rendered.into_bytes()));

                // Create an index.html from the first element in SUMMARY.md
                if index {
                    debug!("[*]: index.html");
                    try!(fs::copy(
                        config.dest().join(&item.path.with_extension("html")),
                        config.dest().join("index.html")
                    ));

                    output!(
                        "[*] Creating index.html from {:?} ✓",
                        config.dest().join(&item.path.with_extension("html"))
                        );
                    index = false;
                }
            }
        }

        // Copy static files (js, css, images, ...)

        debug!("[*] Copy static files");
        // JavaScript
        let mut js_file = try!(File::create(config.dest().join("book.js")));
        try!(js_file.write_all(&theme.js));

        // Css
        let mut css_file = try!(File::create(config.dest().join("book.css")));
        try!(css_file.write_all(&theme.css));

        Ok(())
    }
}

fn make_data(book: BookItems, config: &BookConfig) -> Result<BTreeMap<String,Json>, Box<Error>> {
    debug!("[fn]: make_data");

    let mut data  = BTreeMap::new();
    data.insert("language".to_string(), "en".to_json());
    data.insert("title".to_string(), config.title.to_json());

    let mut chapters = vec![];

    for (section, item) in book {
        let mut chapter = BTreeMap::new();
        chapter.insert("section".to_string(), section.to_json());
        chapter.insert("name".to_string(), item.name.to_json());
        chapter.insert("path".to_string(), item.path.to_str().unwrap().to_json());

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
