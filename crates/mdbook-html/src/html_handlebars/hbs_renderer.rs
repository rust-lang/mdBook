use super::helpers;
use super::static_files::StaticFiles;
use crate::html::ChapterTree;
use crate::html::{build_trees, render_markdown, serialize};
use crate::theme::Theme;
use crate::utils::ToUrlPath;
use anyhow::{Context, Result, bail};
use handlebars::Handlebars;
use mdbook_core::book::{Book, BookItem, Chapter};
use mdbook_core::config::{BookConfig, Config, HtmlConfig};
use mdbook_core::utils::fs;
use mdbook_renderer::{RenderContext, Renderer};
use serde_json::json;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use tracing::error;
use tracing::{debug, info, trace, warn};

/// The HTML renderer for mdBook.
#[derive(Default)]
#[non_exhaustive]
pub struct HtmlHandlebars;

impl HtmlHandlebars {
    /// Returns a new instance of [`HtmlHandlebars`].
    pub fn new() -> Self {
        HtmlHandlebars
    }

    fn render_chapter(
        &self,
        chapter_tree: &ChapterTree<'_>,
        prev_ch: Option<&Chapter>,
        next_ch: Option<&Chapter>,
        mut ctx: RenderChapterContext<'_>,
    ) -> Result<()> {
        // FIXME: This should be made DRY-er and rely less on mutable state
        let ch = chapter_tree.chapter;

        let path = ch.path.as_ref().unwrap();
        // "print.html" is used for the print page.
        if path == Path::new("print.md") {
            bail!("{} is reserved for internal use", path.display());
        };

        if let Some(ref edit_url_template) = ctx.html_config.edit_url_template {
            let full_path = ctx.book_config.src.to_str().unwrap_or_default().to_owned()
                + "/"
                + ch.source_path
                    .clone()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default();

            let edit_url = edit_url_template.replace("{path}", &full_path);
            ctx.data
                .insert("git_repository_edit_url".to_owned(), json!(edit_url));
        }

        let mut content = String::new();
        serialize(&chapter_tree.tree, &mut content);

        let ctx_path = path
            .to_str()
            .with_context(|| "Could not convert path to str")?;
        let filepath = Path::new(&ctx_path).with_extension("html");

        let book_title = ctx
            .data
            .get("book_title")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");

        let title = if let Some(title) = ctx.chapter_titles.get(path) {
            title.clone()
        } else if book_title.is_empty() {
            ch.name.clone()
        } else {
            ch.name.clone() + " - " + book_title
        };

        ctx.data.insert("path".to_owned(), json!(path));
        ctx.data.insert("content".to_owned(), json!(content));
        ctx.data.insert("chapter_title".to_owned(), json!(ch.name));
        ctx.data.insert("title".to_owned(), json!(title));
        ctx.data
            .insert("path_to_root".to_owned(), json!(fs::path_to_root(path)));
        if let Some(ref section) = ch.number {
            ctx.data
                .insert("section".to_owned(), json!(section.to_string()));
        }

        let redirects = collect_redirects_for_path(&filepath, &ctx.html_config.redirect)?;
        if !redirects.is_empty() {
            ctx.data.insert(
                "fragment_map".to_owned(),
                json!(serde_json::to_string(&redirects)?),
            );
        }

        let mut nav = |name: &str, ch: Option<&Chapter>| {
            let Some(ch) = ch else { return };
            let path = ch
                .path
                .as_ref()
                .unwrap()
                .with_extension("html")
                .to_url_path();
            let obj = json!( {
                "title": ch.name,
                "link": path,
            });
            ctx.data.insert(name.to_string(), obj);
        };
        nav("previous", prev_ch);
        nav("next", next_ch);

        // Render the handlebars template with the data
        debug!("Render template");
        let rendered = ctx.handlebars.render("index", &ctx.data)?;

        // Write to file
        let out_path = ctx.destination.join(filepath);
        fs::write(&out_path, rendered)?;

        if prev_ch.is_none() {
            ctx.data.insert("path".to_owned(), json!("index.md"));
            ctx.data.insert("path_to_root".to_owned(), json!(""));
            ctx.data.insert("is_index".to_owned(), json!(true));
            let rendered_index = ctx.handlebars.render("index", &ctx.data)?;
            debug!("Creating index.html from {}", ctx_path);
            fs::write(ctx.destination.join("index.html"), rendered_index)?;
        }

        Ok(())
    }

    fn render_404(
        &self,
        ctx: &RenderContext,
        html_config: &HtmlConfig,
        src_dir: &Path,
        handlebars: &mut Handlebars<'_>,
        data: &mut serde_json::Map<String, serde_json::Value>,
    ) -> Result<()> {
        let content_404 = if let Some(ref filename) = html_config.input_404 {
            let path = src_dir.join(filename);
            fs::read_to_string(&path).with_context(|| "failed to read the 404 input file")?
        } else {
            // 404 input not explicitly configured try the default file 404.md
            let default_404_location = src_dir.join("404.md");
            if default_404_location.exists() {
                fs::read_to_string(&default_404_location)
                    .with_context(|| "failed to read the 404 input file")?
            } else {
                "# Document not found (404)\n\nThis URL is invalid, sorry. Please use the \
                navigation bar or search to continue."
                    .to_string()
            }
        };
        let options = crate::html::HtmlRenderOptions::new(
            Path::new("404.md"),
            html_config,
            ctx.config.rust.edition,
        );
        let html_content_404 = render_markdown(&content_404, &options);

        let mut data_404 = data.clone();
        let base_url = if let Some(site_url) = &html_config.site_url {
            site_url
        } else {
            debug!(
                "HTML 'site-url' parameter not set, defaulting to '/'. Please configure \
                this to ensure the 404 page work correctly, especially if your site is hosted in a \
                subdirectory on the HTTP server."
            );
            "/"
        };
        data_404.insert("base_url".to_owned(), json!(base_url));
        // Set a dummy path to ensure other paths (e.g. in the TOC) are generated correctly
        data_404.insert("path".to_owned(), json!("404.md"));
        data_404.insert("content".to_owned(), json!(html_content_404));

        let mut title = String::from("Page not found");
        if let Some(book_title) = &ctx.config.book.title {
            title.push_str(" - ");
            title.push_str(book_title);
        }
        data_404.insert("title".to_owned(), json!(title));
        let rendered = handlebars.render("index", &data_404)?;

        let output_file = ctx.destination.join(html_config.get_404_output_file());
        fs::write(output_file, rendered)?;
        debug!("Creating 404.html ✓");
        Ok(())
    }

    fn render_print_page(
        &self,
        ctx: &RenderContext,
        handlebars: &Handlebars<'_>,
        data: &mut serde_json::Map<String, serde_json::Value>,
        chapter_trees: Vec<ChapterTree<'_>>,
    ) -> Result<String> {
        let print_content = crate::html::render_print_page(chapter_trees);

        if let Some(ref title) = ctx.config.book.title {
            data.insert("title".to_owned(), json!(title));
        } else {
            // Make sure that the Print chapter does not display the title from
            // the last rendered chapter by removing it from its context
            data.remove("title");
        }
        data.insert("is_print".to_owned(), json!(true));
        data.insert("path".to_owned(), json!("print.md"));
        data.insert("content".to_owned(), json!(print_content));
        data.insert(
            "path_to_root".to_owned(),
            json!(fs::path_to_root(Path::new("print.md"))),
        );

        debug!("Render template");
        let rendered = handlebars.render("index", &data)?;
        Ok(rendered)
    }

    fn register_hbs_helpers(&self, handlebars: &mut Handlebars<'_>, html_config: &HtmlConfig) {
        handlebars.register_helper(
            "toc",
            Box::new(helpers::toc::RenderToc {
                no_section_label: html_config.no_section_label,
            }),
        );
        handlebars.register_helper("fa", Box::new(helpers::fontawesome::fa_helper));
    }

    fn emit_redirects(
        &self,
        root: &Path,
        handlebars: &Handlebars<'_>,
        redirects: &HashMap<String, String>,
    ) -> Result<()> {
        if redirects.is_empty() {
            return Ok(());
        }

        debug!("Emitting redirects");
        let redirects = combine_fragment_redirects(redirects);

        for (original, (dest, fragment_map)) in redirects {
            // Note: all paths are relative to the build directory, so the
            // leading slash in an absolute path means nothing (and would mess
            // up `root.join(original)`).
            let original = original.trim_start_matches('/');
            let filename = root.join(original);
            if filename.exists() {
                // This redirect is handled by the in-page fragment mapper.
                continue;
            }
            if dest.is_empty() {
                bail!(
                    "redirect entry for `{original}` only has source paths with `#` fragments\n\
                     There must be an entry without the `#` fragment to determine the default \
                     destination."
                );
            }
            debug!("Redirecting \"{}\" → \"{}\"", original, dest);
            self.emit_redirect(handlebars, &filename, &dest, &fragment_map)?;
        }

        Ok(())
    }

    fn emit_redirect(
        &self,
        handlebars: &Handlebars<'_>,
        original: &Path,
        destination: &str,
        fragment_map: &BTreeMap<String, String>,
    ) -> Result<()> {
        if let Some(parent) = original.parent() {
            fs::create_dir_all(parent)?
        }

        let js_map = serde_json::to_string(fragment_map)?;

        let ctx = json!({
            "fragment_map": js_map,
            "url": destination,
        });
        let rendered = handlebars.render("redirect", &ctx).with_context(|| {
            format!(
                "Unable to create a redirect file at `{}`",
                original.display()
            )
        })?;
        fs::write(original, rendered)?;

        Ok(())
    }
}

impl Renderer for HtmlHandlebars {
    fn name(&self) -> &str {
        "html"
    }

    fn render(&self, ctx: &RenderContext) -> Result<()> {
        let book_config = &ctx.config.book;
        let html_config = ctx.config.html_config().unwrap_or_default();
        let src_dir = ctx.root.join(&ctx.config.book.src);
        let destination = &ctx.destination;
        let book = &ctx.book;
        let build_dir = ctx.root.join(&ctx.config.build.build_dir);

        if destination.exists() {
            fs::remove_dir_content(destination)
                .with_context(|| "Unable to remove stale HTML output")?;
        }

        trace!("render");
        let mut handlebars = Handlebars::new();

        let theme_dir = match html_config.theme {
            Some(ref theme) => {
                let dir = ctx.root.join(theme);
                if !dir.is_dir() {
                    bail!("theme dir {} does not exist", dir.display());
                }
                dir
            }
            None => ctx.root.join("theme"),
        };

        let theme = Theme::new(theme_dir);

        debug!("Register the index handlebars template");
        handlebars.register_template_string("index", String::from_utf8(theme.index.clone())?)?;

        debug!("Register the head handlebars template");
        handlebars.register_partial("head", String::from_utf8(theme.head.clone())?)?;

        debug!("Register the redirect handlebars template");
        handlebars
            .register_template_string("redirect", String::from_utf8(theme.redirect.clone())?)?;

        debug!("Register the header handlebars template");
        handlebars.register_partial("header", String::from_utf8(theme.header.clone())?)?;

        debug!("Register the toc handlebars template");
        handlebars.register_template_string("toc_js", String::from_utf8(theme.toc_js.clone())?)?;
        handlebars
            .register_template_string("toc_html", String::from_utf8(theme.toc_html.clone())?)?;

        debug!("Register handlebars helpers");
        self.register_hbs_helpers(&mut handlebars, &html_config);

        let mut data = make_data(&ctx.root, book, &ctx.config, &html_config, &theme)?;

        let chapter_trees = build_trees(book, &html_config, ctx.config.rust.edition);

        fs::create_dir_all(destination)
            .with_context(|| "Unexpected error when constructing destination path")?;

        let mut static_files = StaticFiles::new(&theme, &html_config, &ctx.root)?;

        // Render search index
        #[cfg(feature = "search")]
        {
            let default = mdbook_core::config::Search::default();
            let search = html_config.search.as_ref().unwrap_or(&default);
            if search.enable {
                super::search::create_files(&search, &mut static_files, &chapter_trees)?;
            }
        }

        debug!("Render toc js");
        {
            let rendered_toc = handlebars.render("toc_js", &data)?;
            static_files.add_builtin("toc.js", rendered_toc.as_bytes());
            debug!("Creating toc.js ✓");
        }

        if html_config.hash_files {
            static_files.hash_files()?;
        }

        debug!("Copy static files");
        let resource_helper = static_files
            .write_files(&destination)
            .with_context(|| "Unable to copy across static files")?;

        handlebars.register_helper("resource", Box::new(resource_helper));

        debug!("Render toc html");
        {
            data.insert("is_toc_html".to_owned(), json!(true));
            data.insert("path".to_owned(), json!("toc.html"));
            let rendered_toc = handlebars.render("toc_html", &data)?;
            fs::write(destination.join("toc.html"), rendered_toc)?;
            debug!("Creating toc.html ✓");
            data.remove("path");
            data.remove("is_toc_html");
        }

        fs::write(
            destination.join(".nojekyll"),
            b"This file makes sure that Github Pages doesn't process mdBook's output.\n",
        )?;

        if let Some(cname) = &html_config.cname {
            fs::write(destination.join("CNAME"), format!("{cname}\n"))?;
        }

        for (i, chapter_tree) in chapter_trees.iter().enumerate() {
            let previous = (i != 0).then(|| chapter_trees[i - 1].chapter);
            let next = (i != chapter_trees.len() - 1).then(|| chapter_trees[i + 1].chapter);
            let ctx = RenderChapterContext {
                handlebars: &handlebars,
                destination: destination.to_path_buf(),
                data: data.clone(),
                book_config: book_config.clone(),
                html_config: html_config.clone(),
                chapter_titles: &ctx.chapter_titles,
            };
            self.render_chapter(chapter_tree, previous, next, ctx)?;
        }

        // Render 404 page
        if html_config.input_404 != Some("".to_string()) {
            self.render_404(ctx, &html_config, &src_dir, &mut handlebars, &mut data)?;
        }

        // Render the print version.
        if html_config.print.enable {
            let print_rendered =
                self.render_print_page(ctx, &handlebars, &mut data, chapter_trees)?;

            fs::write(destination.join("print.html"), print_rendered)?;
            debug!("Creating print.html ✓");
        }

        self.emit_redirects(&ctx.destination, &handlebars, &html_config.redirect)
            .context("Unable to emit redirects")?;

        // Copy all remaining files, avoid a recursive copy from/to the book build dir
        fs::copy_files_except_ext(&src_dir, destination, true, Some(&build_dir), &["md"])?;

        info!("HTML book written to `{}`", destination.display());

        Ok(())
    }
}

fn make_data(
    root: &Path,
    book: &Book,
    config: &Config,
    html_config: &HtmlConfig,
    theme: &Theme,
) -> Result<serde_json::Map<String, serde_json::Value>> {
    trace!("make_data");

    let mut data = serde_json::Map::new();
    data.insert(
        "language".to_owned(),
        json!(config.book.language.clone().unwrap_or_default()),
    );
    data.insert(
        "text_direction".to_owned(),
        json!(config.book.realized_text_direction()),
    );
    data.insert(
        "book_title".to_owned(),
        json!(config.book.title.clone().unwrap_or_default()),
    );
    data.insert(
        "description".to_owned(),
        json!(config.book.description.clone().unwrap_or_default()),
    );
    if theme.favicon_png.is_some() {
        data.insert("favicon_png".to_owned(), json!("favicon.png"));
    }
    if theme.favicon_svg.is_some() {
        data.insert("favicon_svg".to_owned(), json!("favicon.svg"));
    }
    if let Some(ref live_reload_endpoint) = html_config.live_reload_endpoint {
        data.insert(
            "live_reload_endpoint".to_owned(),
            json!(live_reload_endpoint),
        );
    }

    let default_theme = match html_config.default_theme {
        Some(ref theme) => theme.to_lowercase(),
        None => "light".to_string(),
    };
    data.insert("default_theme".to_owned(), json!(default_theme));

    let preferred_dark_theme = match html_config.preferred_dark_theme {
        Some(ref theme) => theme.to_lowercase(),
        None => "navy".to_string(),
    };
    data.insert(
        "preferred_dark_theme".to_owned(),
        json!(preferred_dark_theme),
    );

    if html_config.mathjax_support {
        data.insert("mathjax_support".to_owned(), json!(true));
    }

    // Add check to see if there is an additional style
    if !html_config.additional_css.is_empty() {
        let mut css = Vec::new();
        for style in &html_config.additional_css {
            match style.strip_prefix(root) {
                Ok(p) => css.push(p.to_str().expect("Could not convert to str")),
                Err(_) => css.push(style.to_str().expect("Could not convert to str")),
            }
        }
        data.insert("additional_css".to_owned(), json!(css));
    }

    // Add check to see if there is an additional script
    if !html_config.additional_js.is_empty() {
        let mut js = Vec::new();
        for script in &html_config.additional_js {
            match script.strip_prefix(root) {
                Ok(p) => js.push(p.to_str().expect("Could not convert to str")),
                Err(_) => js.push(script.to_str().expect("Could not convert to str")),
            }
        }
        data.insert("additional_js".to_owned(), json!(js));
    }

    if html_config.playground.editable && html_config.playground.copy_js {
        data.insert("playground_js".to_owned(), json!(true));
        if html_config.playground.line_numbers {
            data.insert("playground_line_numbers".to_owned(), json!(true));
        }
    }
    if html_config.playground.copyable {
        data.insert("playground_copyable".to_owned(), json!(true));
    }

    data.insert("print_enable".to_owned(), json!(html_config.print.enable));
    data.insert("fold_enable".to_owned(), json!(html_config.fold.enable));
    data.insert("fold_level".to_owned(), json!(html_config.fold.level));
    data.insert(
        "sidebar_header_nav".to_owned(),
        json!(html_config.sidebar_header_nav),
    );

    let search = html_config.search.clone();
    if cfg!(feature = "search") {
        let search = search.unwrap_or_default();
        data.insert("search_enabled".to_owned(), json!(search.enable));
        data.insert(
            "search_js".to_owned(),
            json!(search.enable && search.copy_js),
        );
    } else if search.is_some() {
        warn!("mdBook compiled without search support, ignoring `output.html.search` table");
        warn!(
            "please reinstall with `cargo install mdbook --force --features search`to use the \
             search feature"
        )
    }

    if let Some(ref git_repository_url) = html_config.git_repository_url {
        data.insert("git_repository_url".to_owned(), json!(git_repository_url));
    }

    let git_repository_icon = match html_config.git_repository_icon {
        Some(ref git_repository_icon) => git_repository_icon,
        None => "fab-github",
    };
    let git_repository_icon_class = match git_repository_icon.split('-').next() {
        Some("fa") => "regular",
        Some("fas") => "solid",
        Some("fab") => "brands",
        _ => "regular",
    };
    data.insert("git_repository_icon".to_owned(), json!(git_repository_icon));
    data.insert(
        "git_repository_icon_class".to_owned(),
        json!(git_repository_icon_class),
    );

    let mut chapters = vec![];

    for item in book.iter() {
        // Create the data to inject in the template
        let mut chapter = BTreeMap::new();

        match *item {
            BookItem::PartTitle(ref title) => {
                chapter.insert("part".to_owned(), json!(title));
            }
            BookItem::Chapter(ref ch) => {
                if let Some(ref section) = ch.number {
                    chapter.insert("section".to_owned(), json!(section.to_string()));
                }

                chapter.insert(
                    "has_sub_items".to_owned(),
                    json!((!ch.sub_items.is_empty()).to_string()),
                );

                chapter.insert("name".to_owned(), json!(ch.name));
                if let Some(ref path) = ch.path {
                    let p = path
                        .to_str()
                        .with_context(|| "Could not convert path to str")?;
                    chapter.insert("path".to_owned(), json!(p));
                }
            }
            BookItem::Separator => {
                chapter.insert("spacer".to_owned(), json!("_spacer_"));
            }
        }

        chapters.push(chapter);
    }

    data.insert("chapters".to_owned(), json!(chapters));

    debug!("[*]: JSON constructed");
    Ok(data)
}

struct RenderChapterContext<'a> {
    handlebars: &'a Handlebars<'a>,
    destination: PathBuf,
    data: serde_json::Map<String, serde_json::Value>,
    book_config: BookConfig,
    html_config: HtmlConfig,
    chapter_titles: &'a HashMap<PathBuf, String>,
}

/// Redirect mapping.
///
/// The key is the source path (like `foo/bar.html`). The value is a tuple
/// `(destination_path, fragment_map)`. The `destination_path` is the page to
/// redirect to. `fragment_map` is the map of fragments that override the
/// destination. For example, a fragment `#foo` could redirect to any other
/// page or site.
type CombinedRedirects = BTreeMap<String, (String, BTreeMap<String, String>)>;
fn combine_fragment_redirects(redirects: &HashMap<String, String>) -> CombinedRedirects {
    let mut combined: CombinedRedirects = BTreeMap::new();
    // This needs to extract the fragments to generate the fragment map.
    for (original, new) in redirects {
        if let Some((source_path, source_fragment)) = original.rsplit_once('#') {
            let e = combined.entry(source_path.to_string()).or_default();
            if let Some(old) = e.1.insert(format!("#{source_fragment}"), new.clone()) {
                error!(
                    "internal error: found duplicate fragment redirect \
                     {old} for {source_path}#{source_fragment}"
                );
            }
        } else {
            let e = combined.entry(original.to_string()).or_default();
            e.0 = new.clone();
        }
    }
    combined
}

/// Collects fragment redirects for an existing page.
///
/// The returned map has keys like `#foo` and the value is the new destination
/// path or URL.
fn collect_redirects_for_path(
    path: &Path,
    redirects: &HashMap<String, String>,
) -> Result<BTreeMap<String, String>> {
    let path = format!("/{}", path.to_url_path());
    if redirects.contains_key(&path) {
        bail!(
            "redirect found for existing chapter at `{path}`\n\
            Either delete the redirect or remove the chapter."
        );
    }

    let key_prefix = format!("{path}#");
    let map = redirects
        .iter()
        .filter_map(|(source, dest)| {
            source
                .strip_prefix(&key_prefix)
                .map(|fragment| (format!("#{fragment}"), dest.to_string()))
        })
        .collect();
    Ok(map)
}
