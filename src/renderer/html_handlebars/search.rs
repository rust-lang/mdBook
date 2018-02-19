extern crate elasticlunr;

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;

use pulldown_cmark::*;
use serde_json;
use self::elasticlunr::Index;

use book::{Book, BookItem};
use config::Search;
use errors::*;
use utils;
use theme::searcher;

/// Creates all files required for search.
pub fn create_files(search_config: &Search, destination: &Path, book: &Book) -> Result<()> {
    let mut index = Index::new(&["title", "body", "breadcrumbs"]);

    for item in book.iter() {
        render_item(&mut index, &search_config, item)?;
    }

    let json = write_to_json(index, &search_config)?;
    if search_config.copy_js {
        utils::fs::write_file(destination, "searchindex.json", json.as_bytes())?;
        utils::fs::write_file(destination, "searcher.js", searcher::JS)?;
        utils::fs::write_file(destination, "mark.min.js", searcher::MARK_JS)?;
        utils::fs::write_file(destination, "elasticlunr.min.js", searcher::ELASTICLUNR_JS)?;
        debug!("Copying search files ✓");
    }

    Ok(())
}

fn make_doc_ref<'a>(anchor_base: &'a str, section_id: &Option<String>) -> Cow<'a, str> {
    if let &Some(ref id) = section_id {
        format!("{}#{}", anchor_base, id).into()
    } else {
        anchor_base.into()
    }
}

fn add_doc(index: &mut Index, anchor_base: &str, section_id: &Option<String>, items: &[&str]) {
    let doc_ref = make_doc_ref(anchor_base, section_id);
    let doc_ref = utils::collapse_whitespace(doc_ref.trim());
    let items = items.iter().map(|&x| utils::collapse_whitespace(x.trim()));
    index.add_doc(&doc_ref, items);
}

/// Renders markdown into flat unformatted text and adds it to the search index.
fn render_item(index: &mut Index, search_config: &Search, item: &BookItem) -> Result<()> {
    let chapter = match item {
        &BookItem::Chapter(ref ch) => ch,
        _ => return Ok(()),
    };

    let filepath = Path::new(&chapter.path).with_extension("html");
    let filepath = filepath
        .to_str()
        .chain_err(|| "Could not convert HTML path to str")?;
    let anchor_base = utils::fs::normalize_path(filepath);

    let mut opts = Options::empty();
    opts.insert(OPTION_ENABLE_TABLES);
    opts.insert(OPTION_ENABLE_FOOTNOTES);
    let p = Parser::new_ext(&chapter.content, opts);

    let mut in_header = false;
    let max_section_depth = search_config.heading_split_level as i32;
    let mut section_id = None;
    let mut heading = String::new();
    let mut body = String::new();
    let mut breadcrumbs = chapter.parent_names.clone();
    let mut footnote_numbers = HashMap::new();

    for event in p {
        match event {
            Event::Start(Tag::Header(i)) if i <= max_section_depth => {
                if heading.len() > 0 {
                    // Section finished, the next header is following now
                    // Write the data to the index, and clear it for the next section
                    add_doc(
                        index,
                        &anchor_base,
                        &section_id,
                        &[&heading, &body, &breadcrumbs.join(" » ")],
                    );
                    section_id = None;
                    heading.clear();
                    body.clear();
                    breadcrumbs.pop();
                }

                in_header = true;
            }
            Event::End(Tag::Header(i)) if i <= max_section_depth => {
                in_header = false;
                section_id = Some(utils::id_from_content(&heading));
                breadcrumbs.push(heading.clone());
            }
            Event::Start(Tag::FootnoteDefinition(name)) => {
                let number = footnote_numbers.len() + 1;
                footnote_numbers.entry(name).or_insert(number);
            }
            Event::Start(_) | Event::End(_) | Event::SoftBreak | Event::HardBreak => {
                // Insert spaces where HTML output would usually seperate text
                // to ensure words don't get merged together
                if in_header {
                    heading.push(' ');
                } else {
                    body.push(' ');
                }
            }
            Event::Text(text) => {
                if in_header {
                    heading.push_str(&text);
                } else {
                    body.push_str(&text);
                }
            }
            Event::Html(html) | Event::InlineHtml(html) => {
                let textified = utils::remove_html_tags(&html);
                body.push_str(&textified);
            }
            Event::FootnoteReference(name) => {
                let len = footnote_numbers.len() + 1;
                let number = footnote_numbers.entry(name).or_insert(len);
                body.push_str(&format!(" [{}] ", number));
            }
        }
    }

    if heading.len() > 0 {
        // Make sure the last section is added to the index
        add_doc(
            index,
            &anchor_base,
            &section_id,
            &[&heading, &body, &breadcrumbs.join(" » ")],
        );
    }

    Ok(())
}

/// Converts the index and search options to a JSON string
fn write_to_json(index: Index, search_config: &Search) -> Result<String> {
    // These structs mirror the configuration javascript object accepted by
    // http://elasticlunr.com/docs/configuration.js.html

    #[derive(Serialize)]
    struct SearchOptionsField {
        boost: u8,
    }

    #[derive(Serialize)]
    struct SearchOptionsFields {
        title: SearchOptionsField,
        body: SearchOptionsField,
        breadcrumbs: SearchOptionsField,
    }

    #[derive(Serialize)]
    struct SearchOptions {
        bool: String,
        expand: bool,
        limit_results: u32,
        teaser_word_count: u32,
        fields: SearchOptionsFields,
    }

    #[derive(Serialize)]
    struct SearchindexJson {
        /// The searchoptions for elasticlunr.js
        searchoptions: SearchOptions,
        /// The index for elasticlunr.js
        index: elasticlunr::Index,
    }

    let searchoptions = SearchOptions {
        bool: if search_config.use_boolean_and {
            "AND".into()
        } else {
            "OR".into()
        },
        expand: search_config.expand,
        limit_results: search_config.limit_results,
        teaser_word_count: search_config.teaser_word_count,
        fields: SearchOptionsFields {
            title: SearchOptionsField {
                boost: search_config.boost_title,
            },
            body: SearchOptionsField {
                boost: search_config.boost_paragraph,
            },
            breadcrumbs: SearchOptionsField {
                boost: search_config.boost_hierarchy,
            },
        },
    };

    let json_contents = SearchindexJson {
        searchoptions: searchoptions,
        index: index,
    };

    Ok(serde_json::to_string(&json_contents)?)
}
