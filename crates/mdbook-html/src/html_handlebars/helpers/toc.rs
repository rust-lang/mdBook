use std::path::Path;
use std::{cmp::Ordering, collections::BTreeMap};

use handlebars::{
    Context, Handlebars, Helper, HelperDef, Output, RenderContext, RenderError, RenderErrorReason,
};
use mdbook_markdown::special_escape;

// Handlebars helper to construct TOC
#[derive(Clone, Copy)]
pub(crate) struct RenderToc {
    pub no_section_label: bool,
}

impl HelperDef for RenderToc {
    fn call<'reg: 'rc, 'rc>(
        &self,
        _h: &Helper<'rc>,
        _r: &'reg Handlebars<'_>,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> Result<(), RenderError> {
        // get value from context data
        // rc.get_path() is current json parent path, you should always use it like this
        // param is the key of value you want to display
        let chapters = rc.evaluate(ctx, "@root/chapters").and_then(|c| {
            serde_json::value::from_value::<Vec<BTreeMap<String, String>>>(c.as_json().clone())
                .map_err(|_| {
                    RenderErrorReason::Other("Could not decode the JSON data".to_owned()).into()
                })
        })?;

        let fold_enable = rc
            .evaluate(ctx, "@root/fold_enable")?
            .as_json()
            .as_bool()
            .ok_or_else(|| {
                RenderErrorReason::Other("Type error for `fold_enable`, bool expected".to_owned())
            })?;

        let fold_level = rc
            .evaluate(ctx, "@root/fold_level")?
            .as_json()
            .as_u64()
            .ok_or_else(|| {
                RenderErrorReason::Other("Type error for `fold_level`, u64 expected".to_owned())
            })?;

        // If true, then this is the iframe and we need target="_parent"
        let is_toc_html = rc
            .evaluate(ctx, "@root/is_toc_html")?
            .as_json()
            .as_bool()
            .unwrap_or(false);

        out.write("<ol class=\"chapter\">")?;

        let mut current_level = 1;

        for item in chapters {
            let (_section, level) = if let Some(s) = item.get("section") {
                (s.as_str(), s.matches('.').count())
            } else {
                ("", 1)
            };

            // Expand if folding is disabled, or if levels that are larger than this would not
            // be folded.
            let is_expanded = !fold_enable || level - 1 < (fold_level as usize);

            match level.cmp(&current_level) {
                Ordering::Greater => {
                    while level > current_level {
                        out.write("<li>")?;
                        out.write("<ol class=\"section\">")?;
                        current_level += 1;
                    }
                    write_li_open_tag(out, is_expanded, false)?;
                }
                Ordering::Less => {
                    while level < current_level {
                        out.write("</ol>")?;
                        out.write("</li>")?;
                        current_level -= 1;
                    }
                    write_li_open_tag(out, is_expanded, false)?;
                }
                Ordering::Equal => {
                    write_li_open_tag(out, is_expanded, !item.contains_key("section"))?;
                }
            }

            // Spacer
            if item.contains_key("spacer") {
                out.write("<li class=\"spacer\"></li>")?;
                continue;
            }

            // Part title
            if let Some(title) = item.get("part") {
                out.write("<li class=\"part-title\">")?;
                out.write(&special_escape(title))?;
                out.write("</li>")?;
                continue;
            }

            // Link
            let path_exists = match item.get("path") {
                Some(path) if !path.is_empty() => {
                    out.write("<a href=\"")?;
                    let tmp = Path::new(path)
                        .with_extension("html")
                        .to_str()
                        .unwrap()
                        // Hack for windows who tends to use `\` as separator instead of `/`
                        .replace('\\', "/");

                    // Add link
                    out.write(&tmp)?;
                    out.write(if is_toc_html {
                        "\" target=\"_parent\">"
                    } else {
                        "\">"
                    })?;
                    true
                }
                _ => {
                    out.write("<div>")?;
                    false
                }
            };

            if !self.no_section_label {
                // Section does not necessarily exist
                if let Some(section) = item.get("section") {
                    out.write("<strong aria-hidden=\"true\">")?;
                    out.write(section)?;
                    out.write("</strong> ")?;
                }
            }

            if let Some(name) = item.get("name") {
                out.write(&special_escape(name))?
            }

            if path_exists {
                out.write("</a>")?;
            } else {
                out.write("</div>")?;
            }

            // Render expand/collapse toggle
            if let Some(flag) = item.get("has_sub_items") {
                let has_sub_items = flag.parse::<bool>().unwrap_or_default();
                if fold_enable && has_sub_items {
                    out.write("<a class=\"toggle\"><div>❱</div></a>")?;
                }
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

fn write_li_open_tag(
    out: &mut dyn Output,
    is_expanded: bool,
    is_affix: bool,
) -> Result<(), std::io::Error> {
    let mut li = String::from("<li class=\"chapter-item ");
    if is_expanded {
        li.push_str("expanded ");
    }
    if is_affix {
        li.push_str("affix ");
    }
    li.push_str("\">");
    out.write(&li)
}
