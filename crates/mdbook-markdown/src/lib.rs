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

use pulldown_cmark::{CodeBlockKind, CowStr, Event, LinkType, Options, Parser, Tag, TagEnd, html};
use regex::Regex;
use std::collections::HashMap;
use std::fmt::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::LazyLock;
use tracing::warn;

#[doc(inline)]
pub use pulldown_cmark;

#[cfg(test)]
mod tests;

/// Options for parsing markdown.
#[non_exhaustive]
pub struct MarkdownOptions {
    /// Enables smart punctuation.
    ///
    /// Converts quotes to curly quotes, `...` to `…`, `--` to en-dash, and
    /// `---` to em-dash.
    ///
    /// This is `true` by default.
    pub smart_punctuation: bool,
}

impl Default for MarkdownOptions {
    fn default() -> MarkdownOptions {
        MarkdownOptions {
            smart_punctuation: true,
        }
    }
}

/// Options for converting markdown to HTML.
#[non_exhaustive]
pub struct HtmlRenderOptions<'a> {
    /// Options for parsing markdown.
    pub markdown_options: MarkdownOptions,
    /// The chapter's location, relative to the `SUMMARY.md` file.
    pub path: &'a Path,
    /// If true, render for the print page.
    pub for_print: bool,
    /// The path to the page being rendered.
    pub redirect: &'a HashMap<String, String>,
}

impl<'a> HtmlRenderOptions<'a> {
    /// Creates a new [`HtmlRenderOptions`].
    pub fn new(path: &'a Path, redirect: &'a HashMap<String, String>) -> HtmlRenderOptions<'a> {
        HtmlRenderOptions {
            markdown_options: MarkdownOptions::default(),
            path,
            for_print: false,
            redirect,
        }
    }
}

/// Improve the path to try remove and solve .. token,
/// This assumes that `a/b/../c` is `a/c`.
///
/// This function ensures a given path ending with '/' will also
/// end with '/' after normalization.
/// <https://stackoverflow.com/a/68233480>
fn normalize_path<P: AsRef<Path>>(path: P) -> String {
    let ends_with_slash = path.as_ref().to_str().map_or(false, |s| s.ends_with('/'));
    let mut normalized = PathBuf::new();
    for component in path.as_ref().components() {
        match &component {
            Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push(component);
                }
            }
            Component::CurDir => {}
            _ => {
                normalized.push(component);
            }
        }
    }
    if ends_with_slash {
        normalized.push("");
    }
    normalized
        .to_str()
        .unwrap()
        .replace("\\", "/")
        .trim_start_matches('/')
        .to_string()
}

/// Converts a relative URL path to a reference ID for the print page.
fn normalize_print_page_id(mut path: String) -> String {
    path = path
        .replace("/", "-")
        .replace(".html#", "-")
        .replace("#", "-")
        .to_ascii_lowercase();
    if path.ends_with(".html") {
        path.truncate(path.len() - 5);
    }
    path
}

/// Creates a new pulldown-cmark parser of the given text.
pub fn new_cmark_parser<'text>(text: &'text str, options: &MarkdownOptions) -> Parser<'text> {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    if options.smart_punctuation {
        opts.insert(Options::ENABLE_SMART_PUNCTUATION);
    }
    Parser::new_ext(text, opts)
}

/// Renders markdown to HTML.
///
/// `path` is the path to the page being rendered relative to the root of the
/// book. This is used for the `print.html` page so that links on the print
/// page go to the anchors that has a path id prefix. Normal page rendering
/// sets `path` to None.
///
/// `redirects` is also only for the print page. It's for adjusting links to
/// a redirected location to go to the correct spot on the `print.html` page.
pub fn render_markdown(text: &str, options: &HtmlRenderOptions<'_>) -> String {
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
    // This is used to add space between consecutive footnotes. I was unable
    // to figure out a way to do this just with pure CSS.
    let mut prev_was_footnote = false;

    let events = new_cmark_parser(text, &options.markdown_options)
        .map(clean_codeblock_headers)
        .map(|event| adjust_links(event, options))
        .flat_map(|event| {
            let (a, b) = wrap_tables(event);
            a.into_iter().chain(b)
        })
        // Footnote rewriting must go last to ensure inner definition contents
        // are processed (since they get pulled out of the initial stream).
        .filter_map(|event| {
            match event {
                Event::Start(Tag::FootnoteDefinition(name)) => {
                    prev_was_footnote = false;
                    if !in_footnote.is_empty() {
                        warn!(
                            "internal bug: nested footnote not expected in {:?}",
                            options.path
                        );
                    }
                    in_footnote_name = special_escape(&name);
                    None
                }
                Event::End(TagEnd::FootnoteDefinition) => {
                    let def_events = std::mem::take(&mut in_footnote);
                    let name = std::mem::take(&mut in_footnote_name);

                    if footnote_defs.contains_key(&name) {
                        warn!(
                            "footnote `{name}` in {} defined multiple times - \
                             not updating to new definition",
                            options.path.display()
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
                    let mut html = String::new();
                    if prev_was_footnote {
                        write!(html, " ").unwrap();
                    }
                    prev_was_footnote = true;
                    write!(
                        html,
                        "<sup class=\"footnote-reference\" id=\"fr-{name}-{count}\">\
                            <a href=\"#footnote-{name}\">{n}</a>\
                         </sup>"
                    )
                    .unwrap();
                    let html = Event::Html(html.into());
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
                    prev_was_footnote = false;
                    None
                }
                _ => {
                    prev_was_footnote = false;
                    Some(event)
                }
            }
        });

    html::push_html(&mut body, events);

    if !footnote_defs.is_empty() {
        add_footnote_defs(
            &mut body,
            options,
            footnote_defs.into_iter().collect(),
            &footnote_numbers,
        );
    }

    body
}

/// Adds all footnote definitions into `body`.
fn add_footnote_defs(
    body: &mut String,
    options: &HtmlRenderOptions<'_>,
    mut defs: Vec<(String, Vec<Event<'_>>)>,
    numbers: &HashMap<String, (usize, u32)>,
) {
    // Remove unused.
    defs.retain(|(name, _)| {
        if !numbers.contains_key(name) {
            warn!(
                "footnote `{name}` in `{}` is defined but not referenced",
                options.path.display()
            );
            false
        } else {
            true
        }
    });

    let prefix = if options.for_print {
        let mut base = options.path.display().to_string();
        if base.ends_with(".md") {
            base.truncate(base.len() - 3);
        }
        base = normalize_print_page_id(normalize_path(base));

        if base.is_empty() {
            String::new()
        } else {
            format!("{}-", base)
        }
    } else {
        String::new()
    };

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
                Event::Html(format!(" <a href=\"#{prefix}fr-{name}-{usage}\">↩{nth}</a>").into());
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
fn adjust_links<'a>(event: Event<'a>, options: &HtmlRenderOptions<'_>) -> Event<'a> {
    static SCHEME_LINK: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[a-z][a-z0-9+.-]*:").unwrap());
    static HTML_MD_LINK: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?P<link>.*)\.(html|md)(?P<anchor>#.*)?").unwrap());

    fn add_base(options: &HtmlRenderOptions<'_>) -> String {
        let mut fixed_link = String::new();
        if options.for_print {
            let base = options
                .path
                .parent()
                .expect("path can't be empty")
                .to_str()
                .expect("utf-8 paths only");
            if !base.is_empty() {
                write!(fixed_link, "{base}/").unwrap();
            }
        }
        fixed_link.to_string()
    }

    fn fix_print_page_link<'a>(
        mut normalized_path: String,
        redirects: &HashMap<String, String>,
    ) -> CowStr<'a> {
        // Fix redirect links
        let (path_no_fragment, fragment) = match normalized_path.split_once('#') {
            Some((a, b)) => (a, Some(b)),
            None => (normalized_path.as_str(), None),
        };
        for (original, redirect) in redirects {
            if !normalize_path(original.trim_start_matches('/'))
                .eq_ignore_ascii_case(&normalized_path)
                && !normalize_path(original.trim_start_matches('/'))
                    .eq_ignore_ascii_case(&path_no_fragment)
            {
                continue;
            }

            let mut unnormalized_path = String::new();
            if SCHEME_LINK.is_match(&redirect) {
                unnormalized_path = redirect.to_string();
            } else {
                let base = PathBuf::from(path_no_fragment)
                    .parent()
                    .expect("path can't be empty")
                    .to_str()
                    .expect("utf-8 paths only")
                    .to_owned();

                let normalized_base = normalize_path(base).trim_matches('/').to_owned();
                if !normalized_base.is_empty() {
                    write!(unnormalized_path, "{normalized_base}/{redirect}").unwrap();
                } else {
                    unnormalized_path = redirect.to_string().trim_start_matches('/').to_string();
                }
            }

            // original without anchors, need to append link anchors
            if !original.contains("#") {
                if let Some(fragment) = fragment {
                    if !unnormalized_path.contains("#") {
                        unnormalized_path.push('#');
                    } else {
                        unnormalized_path.push('-');
                    }
                    unnormalized_path.push_str(fragment);
                }
            }

            if SCHEME_LINK.is_match(&redirect) {
                return CowStr::from(unnormalized_path);
            } else {
                normalized_path = normalize_path(unnormalized_path);
            }
            break;
        }

        // Check again to make sure anchors are the html links inside the book.
        if normalized_path.starts_with("../") || normalized_path.contains("/../") {
            return CowStr::from(normalized_path);
        }

        let mut fixed_anchor_for_print = String::new();
        fixed_anchor_for_print.push_str("#");
        fixed_anchor_for_print.push_str(&normalize_print_page_id(normalized_path));
        CowStr::from(fixed_anchor_for_print)
    }

    /// Fix resource links like img to the correct location.
    fn fix_resource_links<'a>(dest: CowStr<'a>, options: &HtmlRenderOptions<'_>) -> CowStr<'a> {
        // Don't modify links with schemes like `https`.
        // Only fix relative links
        if SCHEME_LINK.is_match(&dest) || dest.starts_with('/') {
            return dest;
        }

        // This is a relative link, adjust it as necessary.
        let mut fixed_link = add_base(options);
        fixed_link.push_str(&dest);
        CowStr::from(fixed_link)
    }

    fn fix_a_links_with_type<'a>(
        dest: CowStr<'a>,
        options: &HtmlRenderOptions<'_>,
        link_type: LinkType,
    ) -> CowStr<'a> {
        if link_type == LinkType::Email {
            return dest;
        }
        fix_a_links(dest, options)
    }

    /// Adjust markdown file to correct point in the html file.
    fn fix_a_links<'a>(dest: CowStr<'a>, options: &HtmlRenderOptions<'_>) -> CowStr<'a> {
        if dest.starts_with('#') {
            // Fragment-only link.
            if options.for_print {
                let mut base = options.path.display().to_string();
                if base.ends_with(".md") {
                    base.truncate(base.len() - 3);
                }
                return format!(
                    "#{}{}",
                    normalize_print_page_id(normalize_path(base)),
                    dest.replace("#", "-")
                )
                .into();
            } else {
                return dest;
            };
        }

        // Don't modify links with schemes like `https`.
        if SCHEME_LINK.is_match(&dest) {
            return dest;
        }

        let mut fixed_link = if dest.starts_with('/') {
            String::new()
        } else {
            // This is a relative link, adjust it as necessary.
            add_base(options)
        };

        if let Some(caps) = HTML_MD_LINK.captures(&dest) {
            fixed_link.push_str(&caps["link"]);
            fixed_link.push_str(".html");
            if let Some(anchor) = caps.name("anchor") {
                fixed_link.push_str(anchor.as_str());
            }
        } else {
            fixed_link.push_str(&dest);
        };

        let normalized_path = normalize_path(&fixed_link);

        // Judge if the html link is inside the book.
        if !normalized_path.starts_with("../") && !normalized_path.contains("/../") {
            // In `print.html`, print page links would all link to anchors on the print page.
            if options.for_print {
                return fix_print_page_link(normalized_path, options.redirect);
            }
        }
        // In normal page rendering, links to anchors on another page.
        CowStr::from(fixed_link)
    }

    fn fix_html<'a>(html: CowStr<'a>, options: &HtmlRenderOptions<'_>) -> CowStr<'a> {
        // This is a terrible hack, but should be reasonably reliable. Nobody
        // should ever parse a tag with a regex. However, there isn't anything
        // in Rust that I know of that is suitable for handling partial html
        // fragments like those generated by pulldown_cmark.
        //
        // There are dozens of HTML tags/attributes that contain paths, so
        // feel free to add more tags if desired; these are the only ones I
        // care about right now.
        static A_LINK: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r#"(<a [^>]*?href=")([^"]+?)""#).unwrap());
        static A_NAME: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r#"(<a [^>]*?name=")([^"]+?)""#).unwrap());
        static IMG_LINK: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r#"(<img [^>]*?src=")([^"]+?)""#).unwrap());

        let img_link_fixed_html = IMG_LINK.replace_all(&html, |caps: &regex::Captures<'_>| {
            let fixed = fix_resource_links(caps[2].into(), options);
            format!("{}{}\"", &caps[1], fixed)
        });

        let a_name_fixed_html =
            A_NAME.replace_all(&img_link_fixed_html, |caps: &regex::Captures<'_>| {
                // This is a relative link, adjust it as necessary.
                let origin_name = &caps[2].to_string();
                format!(
                    "{}{}\"",
                    &caps[1],
                    CowStr::from(if options.for_print {
                        let mut base = options.path.display().to_string();
                        if base.ends_with(".md") {
                            base.truncate(base.len() - 3);
                        }
                        format!(
                            "{}-{}",
                            normalize_print_page_id(normalize_path(base)),
                            origin_name.to_string()
                        )
                    } else {
                        origin_name.to_string()
                    })
                )
            });

        A_LINK
            .replace_all(&a_name_fixed_html, |caps: &regex::Captures<'_>| {
                let fixed = fix_a_links(caps[2].into(), options);
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
            dest_url: fix_a_links_with_type(dest_url, options, link_type),
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
            dest_url: fix_resource_links(dest_url, options),
            title,
            id,
        }),
        Event::Html(html) => Event::Html(fix_html(html, options)),
        Event::InlineHtml(html) => Event::InlineHtml(fix_html(html, options)),
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
