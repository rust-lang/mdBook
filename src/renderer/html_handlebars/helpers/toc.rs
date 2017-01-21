use std::path::Path;
use std::collections::{VecDeque, BTreeMap};

use serde_json;
use handlebars::{Handlebars, HelperDef, RenderError, RenderContext, Helper, Context};
use pulldown_cmark::{Parser, html, Event, Tag};

// Handlebars helper to construct TOC
#[derive(Clone, Copy)]
pub struct RenderToc;

impl HelperDef for RenderToc {
    fn call(&self, c: &Context, _h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {

        // get value from context data
        // rc.get_path() is current json parent path, you should always use it like this
        // param is the key of value you want to display
        let chapters = c.navigate(rc.get_path(), &VecDeque::new(), "chapters");
        let current = c.navigate(rc.get_path(), &VecDeque::new(), "path").to_string().replace("\"", "");
        try!(rc.writer.write("<ul class=\"chapter\">".as_bytes()));

        // Decode json format
        let decoded: Vec<BTreeMap<String, String>> = serde_json::from_str(&chapters.to_string()).unwrap();

        let mut current_level = 1;

        for item in decoded {

            // Spacer
            if let Some(_) = item.get("spacer") {
                try!(rc.writer.write("<li class=\"spacer\"></li>".as_bytes()));
                continue;
            }

            let level = if let Some(s) = item.get("section") {
                s.len() / 2
            } else {
                1
            };

            if level > current_level {
                while level > current_level {
                    try!(rc.writer.write("<li>".as_bytes()));
                    try!(rc.writer.write("<ul class=\"section\">".as_bytes()));
                    current_level += 1;
                }
                try!(rc.writer.write("<li>".as_bytes()));
            } else if level < current_level {
                while level < current_level {
                    try!(rc.writer.write("</ul>".as_bytes()));
                    try!(rc.writer.write("</li>".as_bytes()));
                    current_level -= 1;
                }
                try!(rc.writer.write("<li>".as_bytes()));
            } else {
                try!(rc.writer.write("<li".as_bytes()));
                if let None = item.get("section") {
                    try!(rc.writer.write(" class=\"affix\"".as_bytes()));
                }
                try!(rc.writer.write(">".as_bytes()));
            }

            // Link
            let path_exists = if let Some(path) = item.get("path") {
                if !path.is_empty() {
                    try!(rc.writer.write("<a href=\"".as_bytes()));

                    // Add link
                    try!(rc.writer.write(Path::new(item.get("path")
                                                       .expect("Error: path should be Some(_)"))
                                             .with_extension("html")
                                             .to_str()
                                             .unwrap()
                                             // Hack for windows who tends to use `\` as separator instead of `/`
                                             .replace("\\", "/")
                                             .as_bytes()));

                    try!(rc.writer.write("\"".as_bytes()));

                    if path == &current {
                        try!(rc.writer.write(" class=\"active\"".as_bytes()));
                    }

                    try!(rc.writer.write(">".as_bytes()));
                    true
                } else {
                    false
                }
            } else {
                false
            };

            // Section does not necessarily exist
            if let Some(section) = item.get("section") {
                try!(rc.writer.write("<strong>".as_bytes()));
                try!(rc.writer.write(section.as_bytes()));
                try!(rc.writer.write("</strong> ".as_bytes()));
            }

            if let Some(title) = item.get("title") {
                // Render only inline code blocks

                // filter all events that are not inline code blocks
                let parser = Parser::new(&title).filter(|event| {
                    match event {
                        &Event::Start(Tag::Code) |
                        &Event::End(Tag::Code) => true,
                        &Event::InlineHtml(_) => true,
                        &Event::Text(_) => true,
                        _ => false,
                    }
                });

                // render markdown to html
                let mut markdown_parsed_name = String::with_capacity(title.len() * 3 / 2);
                html::push_html(&mut markdown_parsed_name, parser);

                // write to the handlebars template
                try!(rc.writer.write(markdown_parsed_name.as_bytes()));
            }

            if path_exists {
                try!(rc.writer.write("</a>".as_bytes()));
            }

            try!(rc.writer.write("</li>".as_bytes()));

        }
        while current_level > 1 {
            try!(rc.writer.write("</ul>".as_bytes()));
            try!(rc.writer.write("</li>".as_bytes()));
            current_level -= 1;
        }

        try!(rc.writer.write("</ul>".as_bytes()));
        Ok(())
    }
}
