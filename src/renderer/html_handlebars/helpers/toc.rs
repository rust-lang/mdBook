use std::collections::BTreeMap;
use std::path::Path;

use crate::utils;

use handlebars::{Context, Handlebars, Helper, HelperDef, Output, RenderContext, RenderError};
use pulldown_cmark::{html, Event, Parser};

// Handlebars helper to construct TOC
#[derive(Clone, Copy)]
pub struct RenderToc {
    pub no_section_label: bool,
}

impl HelperDef for RenderToc {
    fn call<'reg: 'rc, 'rc>(
        &self,
        _h: &Helper<'reg, 'rc>,
        _r: &'reg Handlebars,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg>,
        out: &mut dyn Output,
    ) -> Result<(), RenderError> {
        // get value from context data
        // rc.get_path() is current json parent path, you should always use it like this
        // param is the key of value you want to display
        let chapters = rc.evaluate(ctx, "@root/chapters").and_then(|c| {
            serde_json::value::from_value::<Vec<BTreeMap<String, String>>>(c.as_json().clone())
                .map_err(|_| RenderError::new("Could not decode the JSON data"))
        })?;
        let current = rc
            .evaluate(ctx, "@root/path")?
            .as_json()
            .as_str()
            .ok_or_else(|| RenderError::new("Type error for `path`, string expected"))?
            .replace("\"", "");

        out.write("<ol class=\"chapter\">")?;

        let mut current_level = 1;

        for item in chapters {
            // Spacer
            if item.get("spacer").is_some() {
                out.write("<li class=\"spacer\"></li>")?;
                continue;
            }

            let level = if let Some(s) = item.get("section") {
                s.matches('.').count()
            } else {
                1
            };

            if level > current_level {
                while level > current_level {
                    out.write("<li>")?;
                    out.write("<ol class=\"section\">")?;
                    current_level += 1;
                }
                out.write("<li>")?;
            } else if level < current_level {
                while level < current_level {
                    out.write("</ol>")?;
                    out.write("</li>")?;
                    current_level -= 1;
                }
                out.write("<li>")?;
            } else {
                out.write("<li")?;
                if item.get("section").is_none() {
                    out.write(" class=\"affix\"")?;
                }
                out.write(">")?;
            }

            // Link
            let path_exists = if let Some(path) = item.get("path") {
                if !path.is_empty() {
                    out.write("<a href=\"")?;

                    let tmp = Path::new(item.get("path").expect("Error: path should be Some(_)"))
                        .with_extension("html")
                        .to_str()
                        .unwrap()
                        // Hack for windows who tends to use `\` as separator instead of `/`
                        .replace("\\", "/");

                    // Add link
                    out.write(&utils::fs::path_to_root(&current))?;
                    out.write(&tmp)?;
                    out.write("\"")?;

                    if path == &current {
                        out.write(" class=\"active\"")?;
                    }

                    out.write(">")?;
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
                    out.write("<strong aria-hidden=\"true\">")?;
                    out.write(&section)?;
                    out.write("</strong> ")?;
                }
            }

            if let Some(name) = item.get("name") {
                // Render only inline code blocks

                // filter all events that are not inline code blocks
                let parser = Parser::new(name).filter(|event| match *event {
                    Event::Code(_) | Event::InlineHtml(_) | Event::Text(_) => true,
                    _ => false,
                });

                // render markdown to html
                let mut markdown_parsed_name = String::with_capacity(name.len() * 3 / 2);
                html::push_html(&mut markdown_parsed_name, parser);

                // write to the handlebars template
                out.write(&markdown_parsed_name)?;
            }

            if path_exists {
                out.write("</a>")?;
            }

            out.write("</li>")?;
        }
        while current_level > 1 {
            out.write("</ol>")?;
            out.write("</li>")?;
            current_level -= 1;
        }

        out.write("</ol>")?;
        Ok(())
    }
}
