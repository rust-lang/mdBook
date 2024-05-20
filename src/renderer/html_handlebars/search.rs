use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::path::Path;
use std::str::FromStr;

use elasticlunr::{Index, IndexBuilder};
use once_cell::sync::Lazy;
use pulldown_cmark::*;

use crate::book::{Book, BookItem};
use crate::config::Search;
use crate::errors::*;
use crate::theme::searcher;
use crate::utils;
use log::{debug, warn};
use serde::Serialize;

const MAX_WORD_LENGTH_TO_INDEX: usize = 80;

/// Tokenizes in the same way as elasticlunr-rs (for English), but also drops long tokens.
fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| c.is_whitespace() || c == '-')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_lowercase())
        .filter(|s| s.len() <= MAX_WORD_LENGTH_TO_INDEX)
        .collect()
}

/// Enum representing a language that is supported by elasticlunr,
/// but requires extra work to get full search support.
/// Languages that wouldn't work with the current feature flag config are included.
#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
pub enum SupportedNonEnglishLanguage {
    Arabic,
    Chinese,
    Danish,
    Dutch,
    Finnish,
    French,
    German,
    Hungarian,
    Italian,
    Japanese,
    Korean,
    Norwegian,
    Portuguese,
    Romanian,
    Russian,
    Spanish,
    Swedish,
    Turkish,
}

impl FromStr for SupportedNonEnglishLanguage {
    type Err = ();

    /// A language tag can be like "zh" / "zh-CN" / "zh-Hans" / "zh-Hans-CN")
    /// See: https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/lang#language_tag_syntax
    fn from_str(language_tag: &str) -> Result<Self, Self::Err> {
        use SupportedNonEnglishLanguage::*;
        match language_tag
            .split('-')
            .next()
            .expect("splitting a string always returns at least 1 fragment")
            .to_ascii_lowercase()
            .as_str()
        {
            "ar" => Ok(Arabic),
            "zh" => Ok(Chinese),
            "da" => Ok(Danish),
            "nl" => Ok(Dutch),
            "fi" => Ok(Finnish),
            "fr" => Ok(French),
            "de" => Ok(German),
            "hu" => Ok(Hungarian),
            "it" => Ok(Italian),
            "ja" => Ok(Japanese),
            "ko" => Ok(Korean),
            "no" => Ok(Norwegian),
            "pt" => Ok(Portuguese),
            "ro" => Ok(Romanian),
            "ru" => Ok(Russian),
            "es" => Ok(Spanish),
            "sv" => Ok(Swedish),
            "tr" => Ok(Turkish),
            _ => Err(()),
        }
    }
}

impl TryFrom<SupportedNonEnglishLanguage> for Box<dyn elasticlunr::Language> {
    type Error = ();

    #[cfg(feature = "search-non-english")]
    /// Returns `Ok` if and only if `language.lunr_js_content()` returns `Some`.
    fn try_from(language: SupportedNonEnglishLanguage) -> std::result::Result<Self, Self::Error> {
        use elasticlunr::lang as el;
        use SupportedNonEnglishLanguage::*;
        match language {
            Arabic => Ok(Box::new(el::Arabic::new())),
            Chinese => Ok(Box::new(el::Chinese::new())),
            Danish => Ok(Box::new(el::Danish::new())),
            Dutch => Ok(Box::new(el::Dutch::new())),
            Finnish => Ok(Box::new(el::Finnish::new())),
            French => Ok(Box::new(el::French::new())),
            German => Ok(Box::new(el::German::new())),
            Hungarian => Ok(Box::new(el::Hungarian::new())),
            Italian => Ok(Box::new(el::Italian::new())),
            Japanese => Ok(Box::new(el::Japanese::new())),
            Korean => Ok(Box::new(el::Korean::new())),
            Norwegian => Ok(Box::new(el::Norwegian::new())),
            Portuguese => Ok(Box::new(el::Portuguese::new())),
            Romanian => Ok(Box::new(el::Romanian::new())),
            Russian => Ok(Box::new(el::Russian::new())),
            Spanish => Ok(Box::new(el::Spanish::new())),
            Swedish => Ok(Box::new(el::Swedish::new())),
            Turkish => Ok(Box::new(el::Turkish::new())),
        }
    }

    #[cfg(not(feature = "search-non-english"))]
    fn try_from(_: SupportedNonEnglishLanguage) -> std::result::Result<Self, Self::Error> {
        Err(())
    }
}

impl Display for SupportedNonEnglishLanguage {
    /// Displays as language subtag (e.g. "zh" for Chinese).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use SupportedNonEnglishLanguage::*;
        f.write_str(match self {
            Arabic => "ar",
            Chinese => "zh",
            Danish => "da",
            Dutch => "nl",
            Finnish => "fi",
            French => "fr",
            German => "de",
            Hungarian => "hu",
            Italian => "it",
            Japanese => "ja",
            Korean => "ko",
            Norwegian => "no",
            Portuguese => "pt",
            Romanian => "ro",
            Russian => "ru",
            Spanish => "es",
            Swedish => "sv",
            Turkish => "tr",
        })
    }
}

#[cfg(feature = "search-non-english")]
impl SupportedNonEnglishLanguage {
    /// Returns `Some` if and only if `self.try_into::<Box<dyn elasticlunr::Language>>()` returns `Ok`.
    pub(crate) fn lunr_js_content(self) -> Option<&'static [u8]> {
        use searcher::lang::*;
        use SupportedNonEnglishLanguage::*;
        match self {
            Arabic => Some(ARABIC_JS),
            Chinese => Some(CHINESE_JS),
            Danish => Some(DANISH_JS),
            Dutch => Some(DUTCH_JS),
            Finnish => Some(FINNISH_JS),
            French => Some(FRENCH_JS),
            German => Some(GERMAN_JS),
            Hungarian => Some(HUNGARIAN_JS),
            Italian => Some(ITALIAN_JS),
            Japanese => Some(JAPANESE_JS),
            Korean => Some(KOREAN_JS),
            Norwegian => Some(NORWEGIAN_JS),
            Portuguese => Some(PORTUGUESE_JS),
            Romanian => Some(ROMANIAN_JS),
            Russian => Some(RUSSIAN_JS),
            Spanish => Some(SPANISH_JS),
            Swedish => Some(SWEDISH_JS),
            Turkish => Some(TURKISH_JS),
        }
    }
}

/// Creates all files required for search.
/// Returns the language subtag if extra `lunr.stemmer.support.js` &
/// `lunr.*.js` files should be imported.
/// E.g., returns "zh" when `lunr.stemmer.support.js` & `lunr.zh.js` should be imported.
pub fn create_files(
    search_config: &Search,
    language: Option<SupportedNonEnglishLanguage>,
    destination: &Path,
    book: &Book,
) -> Result<Option<String>> {
    #[allow(unused_variables)]
    let (mut index, extra_language_subtag) = match language.and_then(|l| l.try_into().ok()) {
        None => {
            if let Some(non_english_language) = language {
                warn!(
                    "mdBook compiled without {non_english_language:?}(`{non_english_language}`) \
                    search support though it's available"
                );
                warn!(
                    "please reinstall with `cargo install mdbook --force --features \
                     search-non-english`"
                );
                warn!("to enable {non_english_language:?} search support")
            }
            (
                IndexBuilder::new()
                    .add_field_with_tokenizer("title", Box::new(&tokenize))
                    .add_field_with_tokenizer("body", Box::new(&tokenize))
                    .add_field_with_tokenizer("breadcrumbs", Box::new(&tokenize))
                    .build(),
                None,
            )
        }
        Some(elasticlunr_language) => (
            Index::with_language(elasticlunr_language, &["title", "body", "breadcrumbs"]),
            language.map(|l| l.to_string()),
        ),
    };

    let mut doc_urls = Vec::with_capacity(book.sections.len());

    for item in book.iter() {
        render_item(&mut index, search_config, &mut doc_urls, item)?;
    }

    let index = write_to_json(index, search_config, doc_urls)?;
    debug!("Writing search index ✓");
    if index.len() > 10_000_000 {
        warn!("searchindex.json is very large ({} bytes)", index.len());
    }

    if search_config.copy_js {
        utils::fs::write_file(destination, "searchindex.json", index.as_bytes())?;
        utils::fs::write_file(
            destination,
            "searchindex.js",
            format!("Object.assign(window.search, {});", index).as_bytes(),
        )?;
        utils::fs::write_file(destination, "searcher.js", searcher::JS)?;
        utils::fs::write_file(destination, "mark.min.js", searcher::MARK_JS)?;
        utils::fs::write_file(destination, "elasticlunr.min.js", searcher::ELASTICLUNR_JS)?;
        #[cfg(feature = "search-non-english")]
        if extra_language_subtag.is_some() {
            let language = language.expect("non-English index is only built when specified");
            utils::fs::write_file(
                destination,
                "lunr.stemmer.support.js",
                searcher::lang::STEMMER_SUPPORT_JS,
            )?;
            utils::fs::write_file(
                destination,
                format!("lunr.{language}.js"),
                language.lunr_js_content().expect(
                    "if language.try_into::<Box<dyn elasticlunr::Language>>() returns Ok, \
                    then language.lunr_js_content() should return Some",
                ),
            )?;
        }
        debug!("Copying search files ✓");
    }

    Ok(extra_language_subtag)
}

/// Uses the given arguments to construct a search document, then inserts it to the given index.
fn add_doc(
    index: &mut Index,
    doc_urls: &mut Vec<String>,
    anchor_base: &str,
    heading: &str,
    id_counter: &mut HashMap<String, usize>,
    section_id: &Option<CowStr<'_>>,
    items: &[&str],
) {
    // Either use the explicit section id the user specified, or generate one
    // from the heading content.
    let section_id = section_id.as_ref().map(|id| id.to_string()).or_else(|| {
        if heading.is_empty() {
            // In the case where a chapter has no heading, don't set a section id.
            None
        } else {
            Some(utils::unique_id_from_content(heading, id_counter))
        }
    });

    let url = if let Some(id) = section_id {
        Cow::Owned(format!("{}#{}", anchor_base, id))
    } else {
        Cow::Borrowed(anchor_base)
    };
    let url = utils::collapse_whitespace(url.trim());
    let doc_ref = doc_urls.len().to_string();
    doc_urls.push(url.into());

    let items = items.iter().map(|&x| utils::collapse_whitespace(x.trim()));
    index.add_doc(&doc_ref, items);
}

/// Renders markdown into flat unformatted text and adds it to the search index.
fn render_item(
    index: &mut Index,
    search_config: &Search,
    doc_urls: &mut Vec<String>,
    item: &BookItem,
) -> Result<()> {
    let chapter = match *item {
        BookItem::Chapter(ref ch) if !ch.is_draft_chapter() => ch,
        _ => return Ok(()),
    };

    let chapter_path = chapter
        .path
        .as_ref()
        .expect("Checked that path exists above");
    let filepath = Path::new(&chapter_path).with_extension("html");
    let filepath = filepath
        .to_str()
        .with_context(|| "Could not convert HTML path to str")?;
    let anchor_base = utils::fs::normalize_path(filepath);

    let mut p = utils::new_cmark_parser(&chapter.content, false).peekable();

    let mut in_heading = false;
    let max_section_depth = u32::from(search_config.heading_split_level);
    let mut section_id = None;
    let mut heading = String::new();
    let mut body = String::new();
    let mut breadcrumbs = chapter.parent_names.clone();
    let mut footnote_numbers = HashMap::new();

    breadcrumbs.push(chapter.name.clone());

    let mut id_counter = HashMap::new();
    while let Some(event) = p.next() {
        match event {
            Event::Start(Tag::Heading { level, id, .. }) if level as u32 <= max_section_depth => {
                if !heading.is_empty() {
                    // Section finished, the next heading is following now
                    // Write the data to the index, and clear it for the next section
                    add_doc(
                        index,
                        doc_urls,
                        &anchor_base,
                        &heading,
                        &mut id_counter,
                        &section_id,
                        &[&heading, &body, &breadcrumbs.join(" » ")],
                    );
                    heading.clear();
                    body.clear();
                    breadcrumbs.pop();
                }

                section_id = id;
                in_heading = true;
            }
            Event::End(TagEnd::Heading(level)) if level as u32 <= max_section_depth => {
                in_heading = false;
                breadcrumbs.push(heading.clone());
            }
            Event::Start(Tag::FootnoteDefinition(name)) => {
                let number = footnote_numbers.len() + 1;
                footnote_numbers.entry(name).or_insert(number);
            }
            Event::Html(html) => {
                let mut html_block = html.into_string();

                // As of pulldown_cmark 0.6, html events are no longer contained
                // in an HtmlBlock tag. We must collect consecutive Html events
                // into a block ourselves.
                while let Some(Event::Html(html)) = p.peek() {
                    html_block.push_str(html);
                    p.next();
                }
                body.push_str(&clean_html(&html_block));
            }
            Event::InlineHtml(html) => {
                // This is not capable of cleaning inline tags like
                // `foo <script>…</script>`. The `<script>` tags show up as
                // individual InlineHtml events, and the content inside is
                // just a regular Text event. There isn't a very good way to
                // know how to collect all the content in-between. I'm not
                // sure if this is easily fixable. It should be extremely
                // rare, since script and style tags should almost always be
                // blocks, and worse case you have some noise in the index.
                body.push_str(&clean_html(&html));
            }
            Event::Start(_) | Event::End(_) | Event::Rule | Event::SoftBreak | Event::HardBreak => {
                // Insert spaces where HTML output would usually separate text
                // to ensure words don't get merged together
                if in_heading {
                    heading.push(' ');
                } else {
                    body.push(' ');
                }
            }
            Event::Text(text) | Event::Code(text) => {
                if in_heading {
                    heading.push_str(&text);
                } else {
                    body.push_str(&text);
                }
            }
            Event::FootnoteReference(name) => {
                let len = footnote_numbers.len() + 1;
                let number = footnote_numbers.entry(name).or_insert(len);
                body.push_str(&format!(" [{}] ", number));
            }
            Event::TaskListMarker(_checked) => {}
        }
    }

    if !body.is_empty() || !heading.is_empty() {
        let title = if heading.is_empty() {
            if let Some(chapter) = breadcrumbs.first() {
                chapter
            } else {
                ""
            }
        } else {
            &heading
        };
        // Make sure the last section is added to the index
        add_doc(
            index,
            doc_urls,
            &anchor_base,
            &heading,
            &mut id_counter,
            &section_id,
            &[title, &body, &breadcrumbs.join(" » ")],
        );
    }

    Ok(())
}

fn write_to_json(index: Index, search_config: &Search, doc_urls: Vec<String>) -> Result<String> {
    use elasticlunr::config::{SearchBool, SearchOptions, SearchOptionsField};
    use std::collections::BTreeMap;

    #[derive(Serialize)]
    struct ResultsOptions {
        limit_results: u32,
        teaser_word_count: u32,
    }

    #[derive(Serialize)]
    struct SearchindexJson {
        /// The options used for displaying search results
        results_options: ResultsOptions,
        /// The searchoptions for elasticlunr.js
        search_options: SearchOptions,
        /// Used to lookup a document's URL from an integer document ref.
        doc_urls: Vec<String>,
        /// The index for elasticlunr.js
        index: elasticlunr::Index,
    }

    let mut fields = BTreeMap::new();
    let mut opt = SearchOptionsField::default();
    let mut insert_boost = |key: &str, boost| {
        opt.boost = Some(boost);
        fields.insert(key.into(), opt);
    };
    insert_boost("title", search_config.boost_title);
    insert_boost("body", search_config.boost_paragraph);
    insert_boost("breadcrumbs", search_config.boost_hierarchy);

    let search_options = SearchOptions {
        bool: if search_config.use_boolean_and {
            SearchBool::And
        } else {
            SearchBool::Or
        },
        expand: search_config.expand,
        fields,
    };

    let results_options = ResultsOptions {
        limit_results: search_config.limit_results,
        teaser_word_count: search_config.teaser_word_count,
    };

    let json_contents = SearchindexJson {
        results_options,
        search_options,
        doc_urls,
        index,
    };

    // By converting to serde_json::Value as an intermediary, we use a
    // BTreeMap internally and can force a stable ordering of map keys.
    let json_contents = serde_json::to_value(&json_contents)?;
    let json_contents = serde_json::to_string(&json_contents)?;

    Ok(json_contents)
}

fn clean_html(html: &str) -> String {
    static AMMONIA: Lazy<ammonia::Builder<'static>> = Lazy::new(|| {
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
    });
    AMMONIA.clean(html).to_string()
}
