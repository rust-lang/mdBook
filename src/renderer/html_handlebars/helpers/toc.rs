use std::path::Path;
use std::collections::BTreeMap;

use serde_json;
use handlebars::{Handlebars, Helper, HelperDef, RenderContext, RenderError};
use pulldown_cmark::{html, Event, Parser, Tag};

// Handlebars helper to construct TOC
#[derive(Clone, Copy)]
pub struct RenderToc {
    pub no_section_label: bool,
}

impl HelperDef for RenderToc {
    fn call(&self, _h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
        // get value from context data
        // rc.get_path() is current json parent path, you should always use it like this
        // param is the key of value you want to display
        let chapters = rc.evaluate_absolute("chapters", true).and_then(|c| {
            serde_json::value::from_value::<Vec<BTreeMap<String, String>>>(c.clone())
                .map_err(|_| RenderError::new("Could not decode the JSON data"))
        })?;
        let current = rc.evaluate_absolute("path", true)?
            .as_str()
            .ok_or_else(|| RenderError::new("Type error for `path`, string expected"))?
            .replace("\"", "");

        rc.writer.write_all(b"<ol class=\"chapter\">")?;

        let mut current_level = 1;

        for item in chapters {
            // Spacer
            if item.get("spacer").is_some() {
                rc.writer.write_all(b"<li class=\"spacer\"></li>")?;
                continue;
            }

            let level = if let Some(s) = item.get("section") {
                s.matches('.').count()
            } else {
                1
            };

            if level > current_level {
                while level > current_level {
                    rc.writer.write_all(b"<li>")?;
                    rc.writer.write_all(b"<ol class=\"section\">")?;
                    current_level += 1;
                }
                rc.writer.write_all(b"<li>")?;
            } else if level < current_level {
                while level < current_level {
                    rc.writer.write_all(b"</ol>")?;
                    rc.writer.write_all(b"</li>")?;
                    current_level -= 1;
                }
                rc.writer.write_all(b"<li>")?;
            } else {
                rc.writer.write_all(b"<li")?;
                if item.get("section").is_none() {
                    rc.writer.write_all(b" class=\"affix\"")?;
                }
                rc.writer.write_all(b">")?;
            }

            // Link
            let path_exists = if let Some(path) = item.get("path") {
                if !path.is_empty() {
                    rc.writer.write_all(b"<a href=\"")?;

                    let tmp = Path::new(item.get("path").expect("Error: path should be Some(_)"))
                        .with_extension("html")
                        .to_str()
                        .unwrap()
                        // Hack for windows who tends to use `\` as separator instead of `/`
                        .replace("\\", "/");

                    // Add link
                    rc.writer.write_all(tmp.as_bytes())?;
                    rc.writer.write_all(b"\"")?;

                    if path == &current {
                        rc.writer.write_all(b" class=\"active\"")?;
                    }

                    rc.writer.write_all(b">")?;
                    true
                } else {
                    false
                }
            } else {
                false
            };

            if !self.no_section_label {
                // Section does not necessarily exist
                if let Some(section) = item.get("section") {
                    rc.writer.write_all(b"<strong aria-hidden=\"true\">")?;
                    rc.writer.write_all(section.as_bytes())?;
                    rc.writer.write_all(b"</strong> ")?;
                }
            }

            if let Some(name) = item.get("name") {
                // Render only inline code blocks

                // filter all events that are not inline code blocks
                let parser = Parser::new(name).filter(|event| match *event {
                    Event::Start(Tag::Code)
                    | Event::End(Tag::Code)
                    | Event::InlineHtml(_)
                    | Event::Text(_) => true,
                    _ => false,
                });

                // render markdown to html
                let mut markdown_parsed_name = String::with_capacity(name.len() * 3 / 2);
                html::push_html(&mut markdown_parsed_name, parser);

                // write to the handlebars template
                rc.writer.write_all(markdown_parsed_name.as_bytes())?;
            }

            if path_exists {
                rc.writer.write_all(b"</a>")?;
            }

            rc.writer.write_all(b"</li>")?;
        }
        while current_level > 1 {
            rc.writer.write_all(b"</ol>")?;
            rc.writer.write_all(b"</li>")?;
            current_level -= 1;
        }

        rc.writer.write_all(b"</ol>")?;
        Ok(())
    }
}
