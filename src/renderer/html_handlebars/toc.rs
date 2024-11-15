use std::cmp::Ordering;
use std::path::Path;

use super::hbs_renderer::Chapter;
use crate::config::HtmlConfig;
use crate::utils::special_escape;

pub fn render(
    html_config: &HtmlConfig,
    chapters: &[Chapter],
    // If true, then this is the iframe and we need target="_parent"
    is_toc_html: bool,
) -> String {
    let mut out = String::new();

    let fold_enable = html_config.fold.enable;
    let fold_level = html_config.fold.level;

    out.push_str("<ol class=\"chapter\">");

    let mut current_level = 1;

    for item in chapters {
        let (has_section, level) = if let Chapter::Chapter {
            section: Some(s), ..
        } = item
        {
            (true, s.matches('.').count())
        } else {
            (false, 1)
        };

        // Expand if folding is disabled, or if levels that are larger than this would not
        // be folded.
        let is_expanded = !fold_enable || level - 1 < (fold_level as usize);

        match level.cmp(&current_level) {
            Ordering::Greater => {
                while level > current_level {
                    out.push_str("<li>");
                    out.push_str("<ol class=\"section\">");
                    current_level += 1;
                }
                write_li_open_tag(&mut out, is_expanded, false);
            }
            Ordering::Less => {
                while level < current_level {
                    out.push_str("</ol>");
                    out.push_str("</li>");
                    current_level -= 1;
                }
                write_li_open_tag(&mut out, is_expanded, false);
            }
            Ordering::Equal => {
                write_li_open_tag(&mut out, is_expanded, !has_section);
            }
        }

        // Spacer
        if matches!(item, Chapter::Separator) {
            out.push_str("<li class=\"spacer\"></li>");
            continue;
        }

        // Part title
        if let Chapter::PartTitle(title) = item {
            out.push_str("<li class=\"part-title\">");
            out.push_str(&special_escape(title));
            out.push_str("</li>");
            continue;
        }

        // Link
        let path_exists: bool;
        match item {
            Chapter::Chapter {
                path: Some(path), ..
            } if !path.is_empty() => {
                out.push_str("<a href=\"");
                let tmp = Path::new(path)
                    .with_extension("html")
                    .to_str()
                    .unwrap()
                    // Hack for windows who tends to use `\` as separator instead of `/`
                    .replace('\\', "/");

                // Add link
                out.push_str(&tmp);
                out.push_str(if is_toc_html {
                    "\" target=\"_parent\">"
                } else {
                    "\">"
                });
                path_exists = true;
            }
            _ => {
                out.push_str("<div>");
                path_exists = false;
            }
        }

        if !html_config.no_section_label {
            // Section does not necessarily exist
            if let Chapter::Chapter {
                section: Some(section),
                ..
            } = item
            {
                out.push_str("<strong aria-hidden=\"true\">");
                out.push_str(section);
                out.push_str("</strong> ");
            }
        }

        if let Chapter::Chapter { name, .. } = item {
            out.push_str(&special_escape(name));
        }

        out.push_str(if path_exists { "</a>" } else { "</div>" });

        // Render expand/collapse toggle
        if fold_enable {
            if let Chapter::Chapter {
                has_sub_items: true,
                ..
            } = item
            {
                out.push_str("<a class=\"toggle\"><div>❱</div></a>");
            }
        }
        out.push_str("</li>");
    }
    while current_level > 1 {
        out.push_str("</ol>");
        out.push_str("</li>");
        current_level -= 1;
    }

    out.push_str("</ol>");
    out
}

fn write_li_open_tag(out: &mut String, is_expanded: bool, is_affix: bool) {
    out.push_str("<li class=\"chapter-item ");
    if is_expanded {
        out.push_str("expanded ");
    }
    if is_affix {
        out.push_str("affix ");
    }
    out.push_str("\">");
}
