use super::static_files::StaticFiles;
use crate::theme::searcher;
use anyhow::{Context, Result, bail};
use elasticlunr::{Index, IndexBuilder};
use log::{debug, warn};
use mdbook_core::book::{Book, BookItem, Chapter};
use mdbook_core::config::{Search, SearchChapterSettings};
use mdbook_core::utils;
use mdbook_markdown::new_cmark_parser;
use pulldown_cmark::*;
use serde::Serialize;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

const MAX_WORD_LENGTH_TO_INDEX: usize = 80;

/// Tokenizes in the same way as elasticlunr-rs (for English), but also drops long tokens.
fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| c.is_whitespace() || c == '-')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_lowercase())
        .filter(|s| s.len() <= MAX_WORD_LENGTH_TO_INDEX)
        .collect()
}

/// Creates all files required for search.
pub(super) fn create_files(
    search_config: &Search,
    static_files: &mut StaticFiles,
    book: &Book,
) -> Result<()> {
    let mut index = IndexBuilder::new()
        .add_field_with_tokenizer("title", Box::new(&tokenize))
        .add_field_with_tokenizer("body", Box::new(&tokenize))
        .add_field_with_tokenizer("breadcrumbs", Box::new(&tokenize))
        .build();

    let mut doc_urls = Vec::with_capacity(book.sections.len());

    let chapter_configs = sort_search_config(&search_config.chapter);
    validate_chapter_config(&chapter_configs, book)?;

    for item in book.iter() {
        let chapter = match item {
            BookItem::Chapter(ch) if !ch.is_draft_chapter() => ch,
            _ => continue,
        };
        if let Some(path) = settings_path(chapter) {
            let chapter_settings = get_chapter_settings(&chapter_configs, path);
            if !chapter_settings.enable.unwrap_or(true) {
                continue;
            }
        }
        render_item(&mut index, search_config, &mut doc_urls, chapter)?;
    }

    let index = write_to_json(index, search_config, doc_urls)?;
    debug!("Writing search index ✓");
    if index.len() > 10_000_000 {
        warn!("search index is very large ({} bytes)", index.len());
    }

    if search_config.copy_js {
        static_files.add_builtin(
            "searchindex.js",
            // To reduce the size of the generated JSON by preventing all `"` characters to be
            // escaped, we instead surround the string with much less common `'` character.
            format!(
                "window.search = Object.assign(window.search, JSON.parse('{}'));",
                index.replace("\\", "\\\\").replace("'", "\\'")
            )
            .as_bytes(),
        );
        static_files.add_builtin("searcher.js", searcher::JS);
        static_files.add_builtin("mark.min.js", searcher::MARK_JS);
        static_files.add_builtin("elasticlunr.min.js", searcher::ELASTICLUNR_JS);
        debug!("Copying search files ✓");
    }

    Ok(())
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
        Cow::Owned(format!("{anchor_base}#{id}"))
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
    chapter: &Chapter,
) -> Result<()> {
    let chapter_path = chapter
        .path
        .as_ref()
        .expect("Checked that path exists above");
    let filepath = Path::new(&chapter_path).with_extension("html");
    let filepath = filepath
        .to_str()
        .with_context(|| "Could not convert HTML path to str")?;
    let anchor_base = utils::fs::normalize_path(filepath);

    let mut p = new_cmark_parser(&chapter.content, false).peekable();

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
            Event::InlineMath(text) | Event::DisplayMath(text) => {
                if in_heading {
                    heading.push_str(&text);
                } else {
                    body.push_str(&text);
                }
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
                body.push_str(&format!(" [{number}] "));
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
    static AMMONIA: LazyLock<ammonia::Builder<'static>> = LazyLock::new(|| {
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

fn settings_path(ch: &Chapter) -> Option<&Path> {
    ch.source_path.as_deref().or_else(|| ch.path.as_deref())
}

fn validate_chapter_config(
    chapter_configs: &[(PathBuf, SearchChapterSettings)],
    book: &Book,
) -> Result<()> {
    for (path, _) in chapter_configs {
        let found = book
            .iter()
            .filter_map(|item| match item {
                BookItem::Chapter(ch) if !ch.is_draft_chapter() => settings_path(ch),
                _ => None,
            })
            .any(|source_path| source_path.starts_with(path));
        if !found {
            bail!(
                "[output.html.search.chapter] key `{}` does not match any chapter paths",
                path.display()
            );
        }
    }
    Ok(())
}

fn sort_search_config(
    map: &HashMap<String, SearchChapterSettings>,
) -> Vec<(PathBuf, SearchChapterSettings)> {
    let mut settings: Vec<_> = map
        .iter()
        .map(|(key, value)| (PathBuf::from(key), value.clone()))
        .collect();
    // Note: This is case-sensitive, and assumes the author uses the same case
    // as the actual filename.
    settings.sort_by(|a, b| a.0.cmp(&b.0));
    settings
}

fn get_chapter_settings(
    chapter_configs: &[(PathBuf, SearchChapterSettings)],
    source_path: &Path,
) -> SearchChapterSettings {
    let mut result = SearchChapterSettings::default();
    for (path, config) in chapter_configs {
        if source_path.starts_with(path) {
            result.enable = config.enable.or(result.enable);
        }
    }
    result
}

#[test]
fn chapter_settings_priority() {
    let cfg = r#"
        [output.html.search.chapter]
        "cli/watch.md" = { enable = true }
        "cli" = { enable = false }
        "cli/inner/foo.md" = { enable = false }
        "cli/inner" = { enable = true }
        "foo" = {} # Just to make sure empty table is allowed.
    "#;
    let cfg: mdbook_core::config::Config = toml::from_str(cfg).unwrap();
    let html = cfg.html_config().unwrap();
    let chapter_configs = sort_search_config(&html.search.unwrap().chapter);
    for (path, enable) in [
        ("foo.md", None),
        ("cli/watch.md", Some(true)),
        ("cli/index.md", Some(false)),
        ("cli/inner/index.md", Some(true)),
        ("cli/inner/foo.md", Some(false)),
    ] {
        assert_eq!(
            get_chapter_settings(&chapter_configs, Path::new(path)),
            SearchChapterSettings { enable }
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_basic() {
        assert_eq!(tokenize("hello world"), vec!["hello", "world"]);
    }

    #[test]
    fn test_tokenize_with_hyphens() {
        assert_eq!(
            tokenize("hello-world test-case"),
            vec!["hello", "world", "test", "case"]
        );
    }

    #[test]
    fn test_tokenize_mixed_whitespace() {
        assert_eq!(
            tokenize("hello\tworld\ntest\r\ncase"),
            vec!["hello", "world", "test", "case"]
        );
    }

    #[test]
    fn test_tokenize_empty_string() {
        assert_eq!(tokenize(""), Vec::<String>::new());
    }

    #[test]
    fn test_tokenize_only_whitespace() {
        assert_eq!(tokenize("   \t\n  "), Vec::<String>::new());
    }

    #[test]
    fn test_tokenize_case_normalization() {
        assert_eq!(tokenize("Hello WORLD Test"), vec!["hello", "world", "test"]);
    }

    #[test]
    fn test_tokenize_trim_whitespace() {
        assert_eq!(tokenize("  hello   world  "), vec!["hello", "world"]);
    }

    #[test]
    fn test_tokenize_long_words_filtered() {
        let long_word = "a".repeat(MAX_WORD_LENGTH_TO_INDEX + 1);
        let short_word = "a".repeat(MAX_WORD_LENGTH_TO_INDEX);
        let input = format!("{} hello {}", long_word, short_word);
        assert_eq!(tokenize(&input), vec!["hello", &short_word]);
    }

    #[test]
    fn test_tokenize_max_length_word() {
        let max_word = "a".repeat(MAX_WORD_LENGTH_TO_INDEX);
        assert_eq!(tokenize(&max_word), vec![max_word]);
    }

    #[test]
    fn test_tokenize_special_characters() {
        assert_eq!(
            tokenize("hello,world.test!case?"),
            vec!["hello,world.test!case?"]
        );
    }

    #[test]
    fn test_tokenize_unicode() {
        assert_eq!(
            tokenize("café naïve résumé"),
            vec!["café", "naïve", "résumé"]
        );
    }

    #[test]
    fn test_tokenize_unicode_rtl_hebre() {
        assert_eq!(tokenize("שלום עולם"), vec!["שלום", "עולם"]);
    }

    #[test]
    fn test_tokenize_numbers() {
        assert_eq!(
            tokenize("test123 456-789 hello"),
            vec!["test123", "456", "789", "hello"]
        );
    }
}
