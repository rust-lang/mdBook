use super::helpers;
use crate::config::Playpen;
use crate::errors::Result;
use crate::utils;
use handlebars::Handlebars;
use regex::{Captures, Regex};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug)]
pub struct HbsConfig {
    pub index_template: String,
    pub header_template: String,
    pub no_section_label: bool,
}

#[derive(Debug)]
pub struct HbsWrapper {
    handlebars: Handlebars,
}

impl HbsWrapper {
    /// Factory function for create configured handlebars
    pub fn with_config(cfg: HbsConfig) -> Result<Self> {
        let mut handlebars = Handlebars::new();
        debug!("Register the index handlebars template");
        handlebars.register_template_string("index", cfg.index_template)?;

        debug!("Register the header handlebars template");
        handlebars.register_partial("header", cfg.header_template)?;

        debug!("Register handlebars helpers");

        handlebars.register_helper(
            "toc",
            Box::new(helpers::toc::RenderToc {
                no_section_label: cfg.no_section_label,
            }),
        );
        handlebars.register_helper("previous", Box::new(helpers::navigation::previous));
        handlebars.register_helper("next", Box::new(helpers::navigation::next));
        handlebars.register_helper("theme_option", Box::new(helpers::theme::theme_option));

        Ok(HbsWrapper { handlebars })
    }

    pub fn render<T>(&self, name: &str, data: &T, playpen: &Playpen) -> Result<String>
    where
        T: Serialize,
    {
        let rendered = self.handlebars.render(name, data)?;
        let rendered = post_process(rendered, &playpen);

        Ok(rendered)
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy::let_and_return))]
fn post_process(rendered: String, playpen_config: &Playpen) -> String {
    let rendered = build_header_links(&rendered);
    let rendered = fix_code_blocks(&rendered);
    let rendered = add_playpen_pre(&rendered, playpen_config);

    rendered
}

/// Goes through the rendered HTML, making sure all header tags have
/// an anchor respectively so people can link to sections directly.
fn build_header_links(html: &str) -> String {
    let regex = Regex::new(r"<h(\d)>(.*?)</h\d>").unwrap();
    let mut id_counter = HashMap::new();

    regex
        .replace_all(html, |caps: &Captures<'_>| {
            let level = caps[1]
                .parse()
                .expect("Regex should ensure we only ever get numbers here");

            insert_link_into_header(level, &caps[2], &mut id_counter)
        })
        .into_owned()
}

/// Insert a sinle link into a header, making sure each link gets its own
/// unique ID by appending an auto-incremented number (if necessary).
fn insert_link_into_header(
    level: usize,
    content: &str,
    id_counter: &mut HashMap<String, usize>,
) -> String {
    let raw_id = utils::id_from_content(content);

    let id_count = id_counter.entry(raw_id.clone()).or_insert(0);

    let id = match *id_count {
        0 => raw_id,
        other => format!("{}-{}", raw_id, other),
    };

    *id_count += 1;

    format!(
        r##"<h{level}><a class="header" href="#{id}" id="{id}">{text}</a></h{level}>"##,
        level = level,
        id = id,
        text = content
    )
}

// The rust book uses annotations for rustdoc to test code snippets,
// like the following:
// ```rust,should_panic
// fn main() {
//     // Code here
// }
// ```
// This function replaces all commas by spaces in the code block classes
fn fix_code_blocks(html: &str) -> String {
    let regex = Regex::new(r##"<code([^>]+)class="([^"]+)"([^>]*)>"##).unwrap();
    regex
        .replace_all(html, |caps: &Captures<'_>| {
            let before = &caps[1];
            let classes = &caps[2].replace(",", " ");
            let after = &caps[3];

            format!(
                r#"<code{before}class="{classes}"{after}>"#,
                before = before,
                classes = classes,
                after = after
            )
        })
        .into_owned()
}

fn add_playpen_pre(html: &str, playpen_config: &Playpen) -> String {
    let regex = Regex::new(r##"((?s)<code[^>]?class="([^"]+)".*?>(.*?)</code>)"##).unwrap();
    regex
        .replace_all(html, |caps: &Captures<'_>| {
            let text = &caps[1];
            let classes = &caps[2];
            let code = &caps[3];

            if (classes.contains("language-rust")
                && !classes.contains("ignore")
                && !classes.contains("noplaypen"))
                || classes.contains("mdbook-runnable")
            {
                // wrap the contents in an external pre block
                if playpen_config.editable && classes.contains("editable")
                    || text.contains("fn main")
                    || text.contains("quick_main!")
                {
                    format!("<pre class=\"playpen\">{}</pre>", text)
                } else {
                    // we need to inject our own main
                    let (attrs, code) = partition_source(code);

                    format!(
                        "<pre class=\"playpen\"><code class=\"{}\">\n# \
                         #![allow(unused_variables)]\n{}#fn main() {{\n{}#}}</code></pre>",
                        classes, attrs, code
                    )
                }
            } else {
                // not language-rust, so no-op
                text.to_owned()
            }
        })
        .into_owned()
}

fn partition_source(s: &str) -> (String, String) {
    let mut after_header = false;
    let mut before = String::new();
    let mut after = String::new();

    for line in s.lines() {
        let trimline = line.trim();
        let header = trimline.chars().all(char::is_whitespace) || trimline.starts_with("#![");
        if !header || after_header {
            after_header = true;
            after.push_str(line);
            after.push_str("\n");
        } else {
            before.push_str(line);
            before.push_str("\n");
        }
    }

    (before, after)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_build_header_links() {
        let inputs = vec![
            (
                "blah blah <h1>Foo</h1>",
                r##"blah blah <h1><a class="header" href="#foo" id="foo">Foo</a></h1>"##,
            ),
            (
                "<h1>Foo</h1>",
                r##"<h1><a class="header" href="#foo" id="foo">Foo</a></h1>"##,
            ),
            (
                "<h3>Foo^bar</h3>",
                r##"<h3><a class="header" href="#foobar" id="foobar">Foo^bar</a></h3>"##,
            ),
            (
                "<h4></h4>",
                r##"<h4><a class="header" href="#" id=""></a></h4>"##,
            ),
            (
                "<h4><em>Hï</em></h4>",
                r##"<h4><a class="header" href="#hï" id="hï"><em>Hï</em></a></h4>"##,
            ),
            (
                "<h1>Foo</h1><h3>Foo</h3>",
                r##"<h1><a class="header" href="#foo" id="foo">Foo</a></h1><h3><a class="header" href="#foo-1" id="foo-1">Foo</a></h3>"##,
            ),
        ];

        for (src, should_be) in inputs {
            let got = build_header_links(&src);
            assert_eq!(got, should_be);
        }
    }
}
