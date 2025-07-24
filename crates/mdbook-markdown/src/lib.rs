//! Markdown processing used in mdBook.
//!
//! This crate provides functions for processing Markdown in the same way as
//! [mdBook](https://rust-lang.github.io/mdBook/). The [`pulldown_cmark`]
//! crate is used as the underlying parser. This crate re-exports
//! [`pulldown_cmark`] so that you can access its types.
//!
//! The parser in this library adds several modifications to the
//! [`pulldown_cmark`] event stream. For example, it adjusts some links,
//! modifies the behavior of footnotes, and adds various HTML wrappers.

use pulldown_cmark::{CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd, html};
use regex::Regex;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write;
use std::path::Path;
use std::sync::LazyLock;

#[doc(inline)]
pub use pulldown_cmark;

#[cfg(test)]
mod tests;

/// Wrapper around the pulldown-cmark parser for rendering markdown to HTML.
pub fn render_markdown(text: &str, smart_punctuation: bool) -> String {
    render_markdown_with_path(text, smart_punctuation, None)
}

/// Creates a new pulldown-cmark parser of the given text.
pub fn new_cmark_parser(text: &str, smart_punctuation: bool) -> Parser<'_> {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    if smart_punctuation {
        opts.insert(Options::ENABLE_SMART_PUNCTUATION);
    }
    Parser::new_ext(text, opts)
}

/// Renders markdown to HTML.
///
/// `path` should only be set if this is being generated for the consolidated
/// print page. It should point to the page being rendered relative to the
/// root of the book.
pub fn render_markdown_with_path(
    text: &str,
    smart_punctuation: bool,
    path: Option<&Path>,
) -> String {
    let mut body = String::with_capacity(text.len() * 3 / 2);

    // Based on
    // https://github.com/pulldown-cmark/pulldown-cmark/blob/master/pulldown-cmark/examples/footnote-rewrite.rs

    // This handling of footnotes is a two-pass process. This is done to
    // support linkbacks, little arrows that allow you to jump back to the
    // footnote reference. The first pass collects the footnote definitions.
    // The second pass modifies those definitions to include the linkbacks,
    // and inserts the definitions back into the `events` list.

    // This is a map of name -> (number, count)
    // `name` is the name of the footnote.
    // `number` is the footnote number displayed in the output.
    // `count` is the number of references to this footnote (used for multiple
    // linkbacks, and checking for unused footnotes).
    let mut footnote_numbers = HashMap::new();
    // This is a map of name -> Vec<Event>
    // `name` is the name of the footnote.
    // The events list is the list of events needed to build the footnote definition.
    let mut footnote_defs = HashMap::new();

    // The following are used when currently processing a footnote definition.
    //
    // This is the name of the footnote (escaped).
    let mut in_footnote_name = String::new();
    // This is the list of events to build the footnote definition.
    let mut in_footnote = Vec::new();

    let events = new_cmark_parser(text, smart_punctuation)
        .map(clean_codeblock_headers)
        .map(|event| adjust_links(event, path))
        .flat_map(|event| {
            let (a, b) = wrap_tables(event);
            a.into_iter().chain(b)
        })
        // Footnote rewriting must go last to ensure inner definition contents
        // are processed (since they get pulled out of the initial stream).
        .filter_map(|event| {
            match event {
                Event::Start(Tag::FootnoteDefinition(name)) => {
                    if !in_footnote.is_empty() {
                        log::warn!("internal bug: nested footnote not expected in {path:?}");
                    }
                    in_footnote_name = special_escape(&name);
                    None
                }
                Event::End(TagEnd::FootnoteDefinition) => {
                    let def_events = std::mem::take(&mut in_footnote);
                    let name = std::mem::take(&mut in_footnote_name);

                    if footnote_defs.contains_key(&name) {
                        log::warn!(
                            "footnote `{name}` in {} defined multiple times - \
                             not updating to new definition",
                            path.map_or_else(|| Cow::from("<unknown>"), |p| p.to_string_lossy())
                        );
                    } else {
                        footnote_defs.insert(name, def_events);
                    }
                    None
                }
                Event::FootnoteReference(name) => {
                    let name = special_escape(&name);
                    let len = footnote_numbers.len() + 1;
                    let (n, count) = footnote_numbers.entry(name.clone()).or_insert((len, 0));
                    *count += 1;
                    let html = Event::Html(
                        format!(
                            "<sup class=\"footnote-reference\" id=\"fr-{name}-{count}\">\
                                <a href=\"#footnote-{name}\">{n}</a>\
                             </sup>"
                        )
                        .into(),
                    );
                    if in_footnote_name.is_empty() {
                        Some(html)
                    } else {
                        // While inside a footnote, we need to accumulate.
                        in_footnote.push(html);
                        None
                    }
                }
                // While inside a footnote, accumulate all events into a local.
                _ if !in_footnote_name.is_empty() => {
                    in_footnote.push(event);
                    None
                }
                _ => Some(event),
            }
        });

    html::push_html(&mut body, events);

    if !footnote_defs.is_empty() {
        add_footnote_defs(
            &mut body,
            path,
            footnote_defs.into_iter().collect(),
            &footnote_numbers,
        );
    }

    body
}

/// Adds all footnote definitions into `body`.
fn add_footnote_defs(
    body: &mut String,
    path: Option<&Path>,
    mut defs: Vec<(String, Vec<Event<'_>>)>,
    numbers: &HashMap<String, (usize, u32)>,
) {
    // Remove unused.
    defs.retain(|(name, _)| {
        if !numbers.contains_key(name) {
            log::warn!(
                "footnote `{name}` in `{}` is defined but not referenced",
                path.map_or_else(|| Cow::from("<unknown>"), |p| p.to_string_lossy())
            );
            false
        } else {
            true
        }
    });

    defs.sort_by_cached_key(|(name, _)| numbers[name].0);

    body.push_str(
        "<hr>\n\
         <ol class=\"footnote-definition\">",
    );

    // Insert the backrefs to the definition, and put the definitions in the output.
    for (name, mut fn_events) in defs {
        let count = numbers[&name].1;
        fn_events.insert(
            0,
            Event::Html(format!("<li id=\"footnote-{name}\">").into()),
        );
        // Generate the linkbacks.
        for usage in 1..=count {
            let nth = if usage == 1 {
                String::new()
            } else {
                usage.to_string()
            };
            let backlink =
                Event::Html(format!(" <a href=\"#fr-{name}-{usage}\">↩{nth}</a>").into());
            if matches!(fn_events.last(), Some(Event::End(TagEnd::Paragraph))) {
                // Put the linkback at the end of the last paragraph instead
                // of on a line by itself.
                fn_events.insert(fn_events.len() - 1, backlink);
            } else {
                // Not a clear place to put it in this circumstance, so put it
                // at the end.
                fn_events.push(backlink);
            }
        }
        fn_events.push(Event::Html("</li>\n".into()));
        html::push_html(body, fn_events.into_iter());
    }

    body.push_str("</ol>");
}

/// Wraps tables in a `.table-wrapper` class to apply overflow-x rules to.
fn wrap_tables(event: Event<'_>) -> (Option<Event<'_>>, Option<Event<'_>>) {
    match event {
        Event::Start(Tag::Table(_)) => (
            Some(Event::Html(r#"<div class="table-wrapper">"#.into())),
            Some(event),
        ),
        Event::End(TagEnd::Table) => (Some(event), Some(Event::Html(r#"</div>"#.into()))),
        _ => (Some(event), None),
    }
}

fn clean_codeblock_headers(event: Event<'_>) -> Event<'_> {
    match event {
        Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(ref info))) => {
            let info: String = info
                .chars()
                .map(|x| match x {
                    ' ' | '\t' => ',',
                    _ => x,
                })
                .filter(|ch| !ch.is_whitespace())
                .collect();

            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(CowStr::from(info))))
        }
        _ => event,
    }
}

/// Fix links to the correct location.
///
/// This adjusts links, such as turning `.md` extensions to `.html`.
///
/// `path` is the path to the page being rendered relative to the root of the
/// book. This is used for the `print.html` page so that links on the print
/// page go to the original location. Normal page rendering sets `path` to
/// None. Ideally, print page links would link to anchors on the print page,
/// but that is very difficult.
fn adjust_links<'a>(event: Event<'a>, path: Option<&Path>) -> Event<'a> {
    static SCHEME_LINK: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[a-z][a-z0-9+.-]*:").unwrap());
    static MD_LINK: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?P<link>.*)\.md(?P<anchor>#.*)?").unwrap());

    fn fix<'a>(dest: CowStr<'a>, path: Option<&Path>) -> CowStr<'a> {
        if dest.starts_with('#') {
            // Fragment-only link.
            if let Some(path) = path {
                let mut base = path.display().to_string();
                if base.ends_with(".md") {
                    base.replace_range(base.len() - 3.., ".html");
                }
                return format!("{base}{dest}").into();
            } else {
                return dest;
            }
        }
        // Don't modify links with schemes like `https`.
        if !SCHEME_LINK.is_match(&dest) {
            // This is a relative link, adjust it as necessary.
            let mut fixed_link = String::new();
            if let Some(path) = path {
                let base = path
                    .parent()
                    .expect("path can't be empty")
                    .to_str()
                    .expect("utf-8 paths only");
                if !base.is_empty() {
                    write!(fixed_link, "{base}/").unwrap();
                }
            }

            if let Some(caps) = MD_LINK.captures(&dest) {
                fixed_link.push_str(&caps["link"]);
                fixed_link.push_str(".html");
                if let Some(anchor) = caps.name("anchor") {
                    fixed_link.push_str(anchor.as_str());
                }
            } else {
                fixed_link.push_str(&dest);
            };
            return CowStr::from(fixed_link);
        }
        dest
    }

    fn fix_html<'a>(html: CowStr<'a>, path: Option<&Path>) -> CowStr<'a> {
        // This is a terrible hack, but should be reasonably reliable. Nobody
        // should ever parse a tag with a regex. However, there isn't anything
        // in Rust that I know of that is suitable for handling partial html
        // fragments like those generated by pulldown_cmark.
        //
        // There are dozens of HTML tags/attributes that contain paths, so
        // feel free to add more tags if desired; these are the only ones I
        // care about right now.
        static HTML_LINK: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r#"(<(?:a|img) [^>]*?(?:src|href)=")([^"]+?)""#).unwrap());

        HTML_LINK
            .replace_all(&html, |caps: &regex::Captures<'_>| {
                let fixed = fix(caps[2].into(), path);
                format!("{}{}\"", &caps[1], fixed)
            })
            .into_owned()
            .into()
    }

    match event {
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Link {
            link_type,
            dest_url: fix(dest_url, path),
            title,
            id,
        }),
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Image {
            link_type,
            dest_url: fix(dest_url, path),
            title,
            id,
        }),
        Event::Html(html) => Event::Html(fix_html(html, path)),
        Event::InlineHtml(html) => Event::InlineHtml(fix_html(html, path)),
        _ => event,
    }
}

/// Escape characters to make it safe for an HTML string.
pub fn special_escape(mut s: &str) -> String {
    let mut escaped = String::with_capacity(s.len());
    let needs_escape: &[char] = &['<', '>', '\'', '"', '\\', '&'];
    while let Some(next) = s.find(needs_escape) {
        escaped.push_str(&s[..next]);
        match s.as_bytes()[next] {
            b'<' => escaped.push_str("&lt;"),
            b'>' => escaped.push_str("&gt;"),
            b'\'' => escaped.push_str("&#39;"),
            b'"' => escaped.push_str("&quot;"),
            b'\\' => escaped.push_str("&#92;"),
            b'&' => escaped.push_str("&amp;"),
            _ => unreachable!(),
        }
        s = &s[next + 1..];
    }
    escaped.push_str(s);
    escaped
}
