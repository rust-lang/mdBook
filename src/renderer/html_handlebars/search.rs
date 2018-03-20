extern crate ammonia;
extern crate elasticlunr;

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
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

    let index = write_to_js(index, &search_config)?;
    debug!("Writing search index ✓");

    if search_config.copy_js {
        utils::fs::write_file(destination, "searchindex.js", index.as_bytes())?;
        utils::fs::write_file(destination, "searcher.js", searcher::JS)?;
        utils::fs::write_file(destination, "mark.min.js", searcher::MARK_JS)?;
        utils::fs::write_file(destination, "elasticlunr.min.js", searcher::ELASTICLUNR_JS)?;
        debug!("Copying search files ✓");
    }

    Ok(())
}

/// Uses the given arguments to construct a search document, then inserts it to the given index.
fn add_doc<'a>(
    index: &mut Index,
    anchor_base: &'a str,
    section_id: &Option<String>,
    items: &[&str],
) {
    let doc_ref: Cow<'a, str> = if let &Some(ref id) = section_id {
        format!("{}#{}", anchor_base, id).into()
    } else {
        anchor_base.into()
    };
    let doc_ref = utils::collapse_whitespace(doc_ref.trim());
    let items = items.iter().map(|&x| utils::collapse_whitespace(x.trim()));
    index.add_doc(&doc_ref, items);
}

/// Renders markdown into flat unformatted text and adds it to the search index.
fn render_item(
    index: &mut Index,
    search_config: &Search,
    item: &BookItem,
) -> Result<()> {
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
                body.push_str(&clean_html(&html));
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

/// Exports the index and search options to a JS script which stores the index in `window.search`.
/// Using a JS script is a workaround for CORS in `file://` URIs. It also removes the need for
/// downloading/parsing JSON in JS.
fn write_to_js(index: Index, search_config: &Search) -> Result<String> {
    use std::collections::BTreeMap;
    use self::elasticlunr::config::{SearchBool, SearchOptions, SearchOptionsField};

    #[derive(Serialize)]
    struct ResultsOptions {
        limit_results: u32,
        teaser_word_count: u32,
    }

    #[derive(Serialize)]
    struct SearchindexJson {
        /// The options used for displaying search results
        resultsoptions: ResultsOptions,
        /// The searchoptions for elasticlunr.js
        searchoptions: SearchOptions,
        /// The index for elasticlunr.js
        index: elasticlunr::Index,
    }

    let mut fields = BTreeMap::new();
    let mut opt = SearchOptionsField::default();
    opt.boost = Some(search_config.boost_title);
    fields.insert("title".into(), opt);
    opt.boost = Some(search_config.boost_paragraph);
    fields.insert("body".into(), opt);
    opt.boost = Some(search_config.boost_hierarchy);
    fields.insert("breadcrumbs".into(), opt);

    let searchoptions = SearchOptions {
        bool: if search_config.use_boolean_and {
            SearchBool::And
        } else {
            SearchBool::Or
        },
        expand: search_config.expand,
        fields,
    };

    let resultsoptions = ResultsOptions {
        limit_results: search_config.limit_results,
        teaser_word_count: search_config.teaser_word_count,
    };

    let json_contents = SearchindexJson {
        resultsoptions,
        searchoptions,
        index,
    };
    let json_contents = serde_json::to_string(&json_contents)?;

    Ok(format!("window.search = {};", json_contents))
}

fn clean_html(html: &str) -> String {
    lazy_static! {
        static ref AMMONIA: ammonia::Builder<'static> = {
            let mut clean_content = HashSet::new();
            clean_content.insert("script");
            clean_content.insert("style");
            let mut builder = ammonia::Builder::new();
            builder
                .tags(HashSet::new())
                .tag_attributes(HashMap::new())
                .generic_attributes(HashSet::new())
                .link_rel(None)
                .allowed_classes(HashMap::new())
                .clean_content_tags(clean_content);
            builder
        };
    }
    AMMONIA.clean(html).to_string()
}
