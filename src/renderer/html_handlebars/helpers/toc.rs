use std::path::Path;
use std::collections::BTreeMap;

use serde_json;
use handlebars::{Handlebars, HelperDef, RenderError, RenderContext, Helper};
use pulldown_cmark::{Parser, html, Event, Tag};

// Handlebars helper to construct TOC
#[derive(Clone, Copy)]
pub struct RenderToc;

impl HelperDef for RenderToc {
    fn call(&self, _h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {

        // get value from context data
        // rc.get_path() is current json parent path, you should always use it like this
        // param is the key of value you want to display
        let chapters = rc.evaluate_absolute("chapters")
            .and_then(|c| {
                          serde_json::value::from_value::<Vec<BTreeMap<String, String>>>(c.clone())
                              .map_err(|_| RenderError::new("Could not decode the JSON data"))
                      })?;
        let current = rc.evaluate_absolute("path")?
            .as_str()
            .ok_or(RenderError::new("Type error for `path`, string expected"))?
            .replace("\"", "");

        rc.writer.write_all("<ul class=\"chapter\">".as_bytes())?;

        let mut current_level = 1;

        for item in chapters {

            // Spacer
            if item.get("spacer").is_some() {
                rc.writer
                    .write_all("<li class=\"spacer\"></li>".as_bytes())?;
                continue;
            }

            let level = if let Some(s) = item.get("section") {
                s.matches(".").count()
            } else {
                1
            };

            if level > current_level {
                while level > current_level {
                    rc.writer.write_all("<li>".as_bytes())?;
                    rc.writer.write_all("<ul class=\"section\">".as_bytes())?;
                    current_level += 1;
                }
                rc.writer.write_all("<li>".as_bytes())?;
            } else if level < current_level {
                while level < current_level {
                    rc.writer.write_all("</ul>".as_bytes())?;
                    rc.writer.write_all("</li>".as_bytes())?;
                    current_level -= 1;
                }
                rc.writer.write_all("<li>".as_bytes())?;
            } else {
                rc.writer.write_all("<li".as_bytes())?;
                if item.get("section").is_none() {
                    rc.writer.write_all(" class=\"affix\"".as_bytes())?;
                }
                rc.writer.write_all(">".as_bytes())?;
            }

            // Link
            let path_exists = if let Some(path) = item.get("path") {
                if !path.is_empty() {
                    rc.writer.write_all("<a href=\"".as_bytes())?;

                    let tmp = Path::new(item.get("path").expect("Error: path should be Some(_)"))
                        .with_extension("html")
                        .to_str()
                        .unwrap()
                        // Hack for windows who tends to use `\` as separator instead of `/`
                        .replace("\\", "/");

                    // Add link
                    rc.writer.write_all(tmp.as_bytes())?;
                    rc.writer.write_all("\"".as_bytes())?;

                    if path == &current {
                        rc.writer.write_all(" class=\"active\"".as_bytes())?;
                    }

                    rc.writer.write_all(">".as_bytes())?;
                    true
                } else {
                    false
                }
            } else {
                false
            };

            // Section does not necessarily exist
            if let Some(section) = item.get("section") {
                rc.writer.write_all("<strong>".as_bytes())?;
                rc.writer.write_all(section.as_bytes())?;
                rc.writer.write_all("</strong> ".as_bytes())?;
            }

            if let Some(name) = item.get("name") {
                // Render only inline code blocks

                // filter all events that are not inline code blocks
                let parser = Parser::new(name).filter(|event| match *event {
                                                          Event::Start(Tag::Code) |
                                                          Event::End(Tag::Code) |
                                                          Event::InlineHtml(_) |
                                                          Event::Text(_) => true,
                                                          _ => false,
                                                      });

                // render markdown to html
                let mut markdown_parsed_name = String::with_capacity(name.len() * 3 / 2);
                html::push_html(&mut markdown_parsed_name, parser);

                // write to the handlebars template
                rc.writer.write_all(markdown_parsed_name.as_bytes())?;
            }

            if path_exists {
                rc.writer.write_all("</a>".as_bytes())?;
            }

            rc.writer.write_all("</li>".as_bytes())?;

        }
        while current_level > 1 {
            rc.writer.write_all("</ul>".as_bytes())?;
            rc.writer.write_all("</li>".as_bytes())?;
            current_level -= 1;
        }

        rc.writer.write_all("</ul>".as_bytes())?;
        Ok(())
    }
}
