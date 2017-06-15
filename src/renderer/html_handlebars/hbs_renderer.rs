use renderer::html_handlebars::helpers;
use renderer::Renderer;
use book::MDBook;
use book::bookitem::BookItem;
use utils;
use theme::{self, Theme};
use regex::{Regex, Captures};

use std::ascii::AsciiExt;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::error::Error;
use std::io::{self, Read};
use std::collections::BTreeMap;
use std::collections::HashMap;

use handlebars::Handlebars;

use serde_json;


#[derive(Default)]
pub struct HtmlHandlebars;

impl HtmlHandlebars {
    pub fn new() -> Self {
        HtmlHandlebars
    }

    fn render_item(&self, item: &BookItem, book: &MDBook, data: &mut serde_json::Map<String, serde_json::Value>,
                   print_content: &mut String, handlebars: &mut Handlebars, index: &mut bool)
                   -> Result<(), Box<Error>> {
        match *item {
            BookItem::Chapter(_, ref ch) |
            BookItem::Affix(ref ch) => {
                if ch.path != PathBuf::new() {

                    let path = book.get_source().join(&ch.path);

                    debug!("[*]: Opening file: {:?}", path);
                    let mut f = File::open(&path)?;
                    let mut content: String = String::new();

                    debug!("[*]: Reading file");
                    f.read_to_string(&mut content)?;

                    // Parse for playpen links
                    if let Some(p) = path.parent() {
                        content = helpers::playpen::render_playpen(&content, p);
                    }

                    // Render markdown using the pulldown-cmark crate
                    content = utils::render_markdown(&content);
                    print_content.push_str(&content);

                    // Update the context with data for this file
                    let path =
                        ch.path
                            .to_str()
                            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Could not convert path to str"))?;
                    data.insert("path".to_owned(), json!(path));
                    data.insert("content".to_owned(), json!(content));
                    data.insert("chapter_title".to_owned(), json!(ch.name));
                    data.insert("path_to_root".to_owned(), json!(utils::fs::path_to_root(&ch.path)));

                    // Render the handlebars template with the data
                    debug!("[*]: Render template");
                    let rendered = handlebars.render("index", &data)?;

                    let filename = Path::new(&ch.path).with_extension("html");

                    // Do several kinds of post-processing
                    let rendered = build_header_links(rendered, filename.to_str().unwrap_or(""));
                    let rendered = fix_anchor_links(rendered, filename.to_str().unwrap_or(""));
                    let rendered = fix_code_blocks(rendered);
                    let rendered = add_playpen_pre(rendered);

                    // Write to file
                    info!("[*] Creating {:?} ✓", filename.display());
                    book.write_file(filename, &rendered.into_bytes())?;

                    // Create an index.html from the first element in SUMMARY.md
                    if *index {
                        debug!("[*]: index.html");

                        let mut content = String::new();

                        let _source = File::open(
                                book.get_destination()
                                    .expect("If the HTML renderer is called, one would assume the HtmlConfig is set... (3)")
                                    .join(&ch.path.with_extension("html"))
                            )?.read_to_string(&mut content);

                        // This could cause a problem when someone displays
                        // code containing <base href=...>
                        // on the front page, however this case should be very very rare...
                        content = content
                            .lines()
                            .filter(|line| !line.contains("<base href="))
                            .collect::<Vec<&str>>()
                            .join("\n");

                        book.write_file("index.html", content.as_bytes())?;

                        info!("[*] Creating index.html from {:?} ✓",
                                  book.get_destination()
                                      .expect("If the HTML renderer is called, one would assume the HtmlConfig is set... (4)")
                                      .join(&ch.path.with_extension("html"))
                            );
                        *index = false;
                    }
                }
            },
            _ => {},
        }

        Ok(())
    }

    fn post_processing(&self, rendered: String) -> String {
        let rendered = build_header_links(rendered, "print.html");
        let rendered = fix_anchor_links(rendered, "print.html");
        let rendered = fix_code_blocks(rendered);
        let rendered = add_playpen_pre(rendered);

        rendered
    }

    fn copy_static_files(&self, book: &MDBook, theme: &Theme) -> Result<(), Box<Error>> {
        book.write_file("book.js", &theme.js)?;
        book.write_file("book.css", &theme.css)?;
        book.write_file("favicon.png", &theme.favicon)?;
        book.write_file("jquery.js", &theme.jquery)?;
        book.write_file("highlight.css", &theme.highlight_css)?;
        book.write_file("tomorrow-night.css", &theme.tomorrow_night_css)?;
        book.write_file("ayu-highlight.css", &theme.ayu_highlight_css)?;
        book.write_file("highlight.js", &theme.highlight_js)?;
        book.write_file("clipboard.min.js", &theme.clipboard_js)?;
        book.write_file("store.js", &theme.store_js)?;
        book.write_file("_FontAwesome/css/font-awesome.css", theme::FONT_AWESOME)?;
        book.write_file("_FontAwesome/fonts/fontawesome-webfont.eot", theme::FONT_AWESOME_EOT)?;
        book.write_file("_FontAwesome/fonts/fontawesome-webfont.svg", theme::FONT_AWESOME_SVG)?;
        book.write_file("_FontAwesome/fonts/fontawesome-webfont.ttf", theme::FONT_AWESOME_TTF)?;
        book.write_file("_FontAwesome/fonts/fontawesome-webfont.woff", theme::FONT_AWESOME_WOFF)?;
        book.write_file("_FontAwesome/fonts/fontawesome-webfont.woff2", theme::FONT_AWESOME_WOFF2)?;
        book.write_file("_FontAwesome/fonts/FontAwesome.ttf", theme::FONT_AWESOME_TTF)?;

        Ok(())
    }


    fn write_custom_file(&self, custom_file: &Path, book: &MDBook) -> Result<(), Box<Error>> {
            let mut data = Vec::new();
            let mut f = File::open(custom_file)?;
            f.read_to_end(&mut data)?;

            let name = match custom_file.strip_prefix(book.get_root()) {
                Ok(p) => p.to_str().expect("Could not convert to str"),
                Err(_) => {
                    custom_file
                        .file_name()
                        .expect("File has a file name")
                        .to_str()
                        .expect("Could not convert to str")
                },
            };

            book.write_file(name, &data)?;

            Ok(())
    }

    /// Update the context with data for this file
    fn configure_print_version(&self, data: &mut serde_json::Map<String, serde_json::Value>, print_content: &str) {
        data.insert("path".to_owned(), json!("print.md"));
        data.insert("content".to_owned(), json!(print_content));
        data.insert("path_to_root".to_owned(), json!(utils::fs::path_to_root(Path::new("print.md"))));
    }

    fn register_hbs_helpers(&self, handlebars: &mut Handlebars) {
        handlebars.register_helper("toc", Box::new(helpers::toc::RenderToc));
        handlebars.register_helper("previous", Box::new(helpers::navigation::previous));
        handlebars.register_helper("next", Box::new(helpers::navigation::next));
    }

    fn copy_additional_css(&self, book: &MDBook) -> Result<(), Box<Error>> {
        for custom_file in book.get_additional_css()
                .iter()
                .chain(book.get_additional_js().iter()) {
                    self.write_custom_file(custom_file, book)?;
        }

        Ok(())
    }
}


impl Renderer for HtmlHandlebars {
    fn render(&self, book: &MDBook) -> Result<(), Box<Error>> {
        debug!("[fn]: render");
        let mut handlebars = Handlebars::new();

        // Load theme
        let theme = theme::Theme::new(book.get_theme_path());

        // Register template
        debug!("[*]: Register handlebars template");
        handlebars
            .register_template_string("index", String::from_utf8(theme.index.clone())?)?;

        debug!("[*]: Register handlebars helpers");
        self.register_hbs_helpers(&mut handlebars);

        let mut data = make_data(book)?;

        // Print version
        let mut print_content: String = String::new();

        debug!("[*]: Check if destination directory exists");
        let destination = book.get_destination()
                .expect("If the HTML renderer is called, one would assume the HtmlConfig is set... (2)");

        if fs::create_dir_all(&destination).is_err() {
            return Err(Box::new(io::Error::new(io::ErrorKind::Other,
                                               "Unexpected error when constructing destination path")));
        }

        let mut index = true;
        for item in book.iter() {
            self.render_item(item, book, &mut data, &mut print_content, &mut handlebars, &mut index)?;
        }

        // Print version
        self.configure_print_version(&mut data, &print_content);

        // Render the handlebars template with the data
        debug!("[*]: Render template");

        let rendered = handlebars.render("index", &data)?;
        let rendered = self.post_processing(rendered);

        book.write_file(Path::new("print").with_extension("html"), &rendered.into_bytes())?;
        info!("[*] Creating print.html ✓");

        // Copy static files (js, css, images, ...)
        debug!("[*] Copy static files");
        self.copy_static_files(book, &theme)?;
        self.copy_additional_css(book)?;

        // Copy all remaining files
        utils::fs::copy_files_except_ext(book.get_source(), &destination, true, &["md"])?;

        Ok(())
    }
}

fn make_data(book: &MDBook) -> Result<serde_json::Map<String, serde_json::Value>, Box<Error>> {
    debug!("[fn]: make_data");

    let mut data = serde_json::Map::new();
    data.insert("language".to_owned(), json!("en"));
    data.insert("title".to_owned(), json!(book.get_title()));
    data.insert("description".to_owned(), json!(book.get_description()));
    data.insert("favicon".to_owned(), json!("favicon.png"));
    if let Some(livereload) = book.get_livereload() {
        data.insert("livereload".to_owned(), json!(livereload));
    }

    // Add google analytics tag
    if let Some(ref ga) = book.get_google_analytics_id() {
        data.insert("google_analytics".to_owned(), json!(ga));
    }

    // Add check to see if there is an additional style
    if book.has_additional_css() {
        let mut css = Vec::new();
        for style in book.get_additional_css() {
            match style.strip_prefix(book.get_root()) {
                Ok(p) => css.push(p.to_str().expect("Could not convert to str")),
                Err(_) => {
                    css.push(style
                                 .file_name()
                                 .expect("File has a file name")
                                 .to_str()
                                 .expect("Could not convert to str"))
                },
            }
        }
        data.insert("additional_css".to_owned(), json!(css));
    }

    // Add check to see if there is an additional script
    if book.has_additional_js() {
        let mut js = Vec::new();
        for script in book.get_additional_js() {
            match script.strip_prefix(book.get_root()) {
                Ok(p) => js.push(p.to_str().expect("Could not convert to str")),
                Err(_) => {
                    js.push(script
                                .file_name()
                                .expect("File has a file name")
                                .to_str()
                                .expect("Could not convert to str"))
                },
            }
        }
        data.insert("additional_js".to_owned(), json!(js));
    }

    let mut chapters = vec![];

    for item in book.iter() {
        // Create the data to inject in the template
        let mut chapter = BTreeMap::new();

        match *item {
            BookItem::Affix(ref ch) => {
                chapter.insert("name".to_owned(), json!(ch.name));
                let path = ch.path
                    .to_str()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Could not convert path to str"))?;
                chapter.insert("path".to_owned(), json!(path));
            },
            BookItem::Chapter(ref s, ref ch) => {
                chapter.insert("section".to_owned(), json!(s));
                chapter.insert("name".to_owned(), json!(ch.name));
                let path = ch.path
                    .to_str()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Could not convert path to str"))?;
                chapter.insert("path".to_owned(), json!(path));
            },
            BookItem::Spacer => {
                chapter.insert("spacer".to_owned(), json!("_spacer_"));
            },

        }

        chapters.push(chapter);
    }

    data.insert("chapters".to_owned(), json!(chapters));

    debug!("[*]: JSON constructed");
    Ok(data)
}

fn build_header_links(html: String, filename: &str) -> String {
    let regex = Regex::new(r"<h(\d)>(.*?)</h\d>").unwrap();
    let mut id_counter = HashMap::new();

    regex
        .replace_all(&html, |caps: &Captures| {
            let level = &caps[1];
            let text = &caps[2];
            let mut id = text.to_string();
            let repl_sub = vec!["<em>",
                                "</em>",
                                "<code>",
                                "</code>",
                                "<strong>",
                                "</strong>",
                                "&lt;",
                                "&gt;",
                                "&amp;",
                                "&#39;",
                                "&quot;"];
            for sub in repl_sub {
                id = id.replace(sub, "");
            }
            let id = id.chars()
                .filter_map(|c| if c.is_alphanumeric() || c == '-' || c == '_' {
                                if c.is_ascii() {
                                    Some(c.to_ascii_lowercase())
                                } else {
                                    Some(c)
                                }
                            } else if c.is_whitespace() && c.is_ascii() {
                                Some('-')
                            } else {
                                None
                            })
                .collect::<String>();

            let id_count = *id_counter.get(&id).unwrap_or(&0);
            id_counter.insert(id.clone(), id_count + 1);

            let id = if id_count > 0 {
                format!("{}-{}", id, id_count)
            } else {
                id
            };

            format!("<a class=\"header\" href=\"{filename}#{id}\" id=\"{id}\"><h{level}>{text}</h{level}></a>",
                    level = level,
                    id = id,
                    text = text,
                    filename = filename)
        })
        .into_owned()
}

// anchors to the same page (href="#anchor") do not work because of
// <base href="../"> pointing to the root folder. This function *fixes*
// that in a very inelegant way
fn fix_anchor_links(html: String, filename: &str) -> String {
    let regex = Regex::new(r##"<a([^>]+)href="#([^"]+)"([^>]*)>"##).unwrap();
    regex
        .replace_all(&html, |caps: &Captures| {
            let before = &caps[1];
            let anchor = &caps[2];
            let after = &caps[3];

            format!("<a{before}href=\"{filename}#{anchor}\"{after}>",
                    before = before,
                    filename = filename,
                    anchor = anchor,
                    after = after)
        })
        .into_owned()
}


// The rust book uses annotations for rustdoc to test code snippets,
// like the following:
// ```rust,should_panic
// fn main() {
//     // Code here
// }
// ```
// This function replaces all commas by spaces in the code block classes
fn fix_code_blocks(html: String) -> String {
    let regex = Regex::new(r##"<code([^>]+)class="([^"]+)"([^>]*)>"##).unwrap();
    regex
        .replace_all(&html, |caps: &Captures| {
            let before = &caps[1];
            let classes = &caps[2].replace(",", " ");
            let after = &caps[3];

            format!("<code{before}class=\"{classes}\"{after}>", before = before, classes = classes, after = after)
        })
        .into_owned()
}

fn add_playpen_pre(html: String) -> String {
    let regex = Regex::new(r##"((?s)<code[^>]?class="([^"]+)".*?>(.*?)</code>)"##).unwrap();
    regex
        .replace_all(&html, |caps: &Captures| {
            let text = &caps[1];
            let classes = &caps[2];
            let code = &caps[3];

            if classes.contains("language-rust") && !classes.contains("ignore") {
                // wrap the contents in an external pre block

                if text.contains("fn main") || text.contains("quick_main!") {
                    format!("<pre class=\"playpen\">{}</pre>", text)
                } else {
                    // we need to inject our own main
                    let (attrs, code) = partition_source(code);
                    format!("<pre class=\"playpen\"><code class=\"{}\"># #![allow(unused_variables)]
{}#fn main() {{
\
                             {}
#}}</code></pre>",
                            classes,
                            attrs,
                            code)
                }
            } else {
                // not language-rust, so no-op
                format!("{}", text)
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
        let header = trimline.chars().all(|c| c.is_whitespace()) || trimline.starts_with("#![");
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
