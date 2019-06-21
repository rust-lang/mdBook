use crate::book::{Book, BookItem};
use crate::config::{Config, HtmlConfig};
use crate::errors::*;
use crate::renderer::html_handlebars::hbs_wrapper::{HbsConfig, HbsWrapper};
use crate::renderer::{RenderContext, Renderer};
use crate::theme::{self, playpen_editor, Theme};
use crate::utils;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct HtmlHandlebars;

impl HtmlHandlebars {
    pub fn new() -> Self {
        HtmlHandlebars
    }

    fn render_item(
        &self,
        item: &BookItem,
        mut ctx: RenderItemContext<'_>,
        print_content: &mut String,
    ) -> Result<()> {
        // FIXME: This should be made DRY-er and rely less on mutable state
        if let BookItem::Chapter(ref ch) = *item {
            let content = ch.content.clone();
            let content = utils::render_markdown(&content, ctx.html_config.curly_quotes);

            let string_path = ch.path.parent().unwrap().display().to_string();

            let fixed_content = utils::render_markdown_with_base(
                &ch.content,
                ctx.html_config.curly_quotes,
                &string_path,
            );
            print_content.push_str(&fixed_content);

            // Update the context with data for this file
            let path = ch
                .path
                .to_str()
                .chain_err(|| "Could not convert path to str")?;
            let filepath = Path::new(&ch.path).with_extension("html");

            // "print.html" is used for the print page.
            if ch.path == Path::new("print.md") {
                bail!(ErrorKind::ReservedFilenameError(ch.path.clone()));
            };

            // Non-lexical lifetimes needed :'(
            let title: String;
            {
                let book_title = ctx
                    .data
                    .get("book_title")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                title = ch.name.clone() + " - " + book_title;
            }

            ctx.data.insert("path".to_owned(), json!(path));
            ctx.data.insert("content".to_owned(), json!(content));
            ctx.data.insert("chapter_title".to_owned(), json!(ch.name));
            ctx.data.insert("title".to_owned(), json!(title));
            ctx.data.insert(
                "path_to_root".to_owned(),
                json!(utils::fs::path_to_root(&ch.path)),
            );

            // Render the handlebars template with the data
            debug!("Render template");
            let rendered = ctx
                .handlebars
                .render("index", &ctx.data, &ctx.html_config.playpen)?;

            // Write to file
            debug!("Creating {}", filepath.display());
            utils::fs::write_file(&ctx.destination, &filepath, rendered.as_bytes())?;

            if ctx.is_index {
                ctx.data.insert("path".to_owned(), json!("index.md"));
                ctx.data.insert("path_to_root".to_owned(), json!(""));
                let rendered_index =
                    ctx.handlebars
                        .render("index", &ctx.data, &ctx.html_config.playpen)?;
                debug!("Creating index.html from {}", path);
                utils::fs::write_file(&ctx.destination, "index.html", rendered_index.as_bytes())?;
            }
        }

        Ok(())
    }

    fn copy_static_files(
        &self,
        destination: &Path,
        theme: &Theme,
        html_config: &HtmlConfig,
    ) -> Result<()> {
        use crate::utils::fs::write_file;

        write_file(
            destination,
            ".nojekyll",
            b"This file makes sure that Github Pages doesn't process mdBook's output.",
        )?;

        write_file(destination, "book.js", &theme.js)?;
        write_file(destination, "css/general.css", &theme.general_css)?;
        write_file(destination, "css/chrome.css", &theme.chrome_css)?;
        write_file(destination, "css/print.css", &theme.print_css)?;
        write_file(destination, "css/variables.css", &theme.variables_css)?;
        write_file(destination, "favicon.png", &theme.favicon)?;
        write_file(destination, "highlight.css", &theme.highlight_css)?;
        write_file(destination, "tomorrow-night.css", &theme.tomorrow_night_css)?;
        write_file(destination, "ayu-highlight.css", &theme.ayu_highlight_css)?;
        write_file(destination, "highlight.js", &theme.highlight_js)?;
        write_file(destination, "clipboard.min.js", &theme.clipboard_js)?;
        write_file(
            destination,
            "FontAwesome/css/font-awesome.css",
            theme::FONT_AWESOME,
        )?;
        write_file(
            destination,
            "FontAwesome/fonts/fontawesome-webfont.eot",
            theme::FONT_AWESOME_EOT,
        )?;
        write_file(
            destination,
            "FontAwesome/fonts/fontawesome-webfont.svg",
            theme::FONT_AWESOME_SVG,
        )?;
        write_file(
            destination,
            "FontAwesome/fonts/fontawesome-webfont.ttf",
            theme::FONT_AWESOME_TTF,
        )?;
        write_file(
            destination,
            "FontAwesome/fonts/fontawesome-webfont.woff",
            theme::FONT_AWESOME_WOFF,
        )?;
        write_file(
            destination,
            "FontAwesome/fonts/fontawesome-webfont.woff2",
            theme::FONT_AWESOME_WOFF2,
        )?;
        write_file(
            destination,
            "FontAwesome/fonts/FontAwesome.ttf",
            theme::FONT_AWESOME_TTF,
        )?;

        let playpen_config = &html_config.playpen;

        // Ace is a very large dependency, so only load it when requested
        if playpen_config.editable && playpen_config.copy_js {
            // Load the editor
            write_file(destination, "editor.js", playpen_editor::JS)?;
            write_file(destination, "ace.js", playpen_editor::ACE_JS)?;
            write_file(destination, "mode-rust.js", playpen_editor::MODE_RUST_JS)?;
            write_file(destination, "theme-dawn.js", playpen_editor::THEME_DAWN_JS)?;
            write_file(
                destination,
                "theme-tomorrow_night.js",
                playpen_editor::THEME_TOMORROW_NIGHT_JS,
            )?;
        }

        Ok(())
    }

    /// Update the context with data for this file
    fn configure_print_version(
        &self,
        data: &mut serde_json::Map<String, serde_json::Value>,
        print_content: &str,
    ) {
        // Make sure that the Print chapter does not display the title from
        // the last rendered chapter by removing it from its context
        data.remove("title");
        data.insert("is_print".to_owned(), json!(true));
        data.insert("path".to_owned(), json!("print.md"));
        data.insert("content".to_owned(), json!(print_content));
        data.insert(
            "path_to_root".to_owned(),
            json!(utils::fs::path_to_root(Path::new("print.md"))),
        );
    }

    /// Copy across any additional CSS and JavaScript files which the book
    /// has been configured to use.
    fn copy_additional_css_and_js(
        &self,
        html: &HtmlConfig,
        root: &Path,
        destination: &Path,
    ) -> Result<()> {
        let custom_files = html.additional_css.iter().chain(html.additional_js.iter());

        debug!("Copying additional CSS and JS");

        for custom_file in custom_files {
            let input_location = root.join(custom_file);
            let output_location = destination.join(custom_file);
            if let Some(parent) = output_location.parent() {
                fs::create_dir_all(parent)
                    .chain_err(|| format!("Unable to create {}", parent.display()))?;
            }
            debug!(
                "Copying {} -> {}",
                input_location.display(),
                output_location.display()
            );

            fs::copy(&input_location, &output_location).chain_err(|| {
                format!(
                    "Unable to copy {} to {}",
                    input_location.display(),
                    output_location.display()
                )
            })?;
        }

        Ok(())
    }
}

// TODO(mattico): Remove some time after the 0.1.8 release
fn maybe_wrong_theme_dir(dir: &Path) -> Result<bool> {
    fn entry_is_maybe_book_file(entry: fs::DirEntry) -> Result<bool> {
        Ok(entry.file_type()?.is_file()
            && entry.path().extension().map_or(false, |ext| ext == "md"))
    }

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            if entry_is_maybe_book_file(entry?).unwrap_or(false) {
                return Ok(false);
            }
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

impl Renderer for HtmlHandlebars {
    fn name(&self) -> &str {
        "html"
    }

    fn render(&self, ctx: &RenderContext) -> Result<()> {
        let html_config = ctx.config.html_config().unwrap_or_default();
        let src_dir = ctx.root.join(&ctx.config.book.src);
        let destination = &ctx.destination;
        let book = &ctx.book;

        trace!("render");
        let theme_dir = match html_config.theme {
            Some(ref theme) => theme.to_path_buf(),
            None => ctx.root.join("theme"),
        };

        if html_config.theme.is_none()
            && maybe_wrong_theme_dir(&src_dir.join("theme")).unwrap_or(false)
        {
            warn!(
                "Previous versions of mdBook erroneously accepted `./src/theme` as an automatic \
                 theme directory"
            );
            warn!("Please move your theme files to `./theme` for them to continue being used");
        }

        let theme = theme::Theme::new(theme_dir);
        let config = HbsConfig {
            index_template: String::from_utf8(theme.index.clone())?,
            header_template: String::from_utf8(theme.header.clone())?,
            no_section_label: html_config.no_section_label,
        };
        let handlebars = HbsWrapper::with_config(config)?;

        let mut data = make_data(&ctx.root, &book, &ctx.config, &html_config)?;

        // Print version
        let mut print_content = String::new();

        fs::create_dir_all(&destination)
            .chain_err(|| "Unexpected error when constructing destination path")?;

        let mut is_index = true;
        for item in book.iter() {
            let ctx = RenderItemContext {
                handlebars: &handlebars,
                destination: destination.to_path_buf(),
                data: data.clone(),
                is_index,
                html_config: html_config.clone(),
            };
            self.render_item(item, ctx, &mut print_content)?;
            is_index = false;
        }

        // Print version
        self.configure_print_version(&mut data, &print_content);
        if let Some(ref title) = ctx.config.book.title {
            data.insert("title".to_owned(), json!(title));
        }

        // Render the handlebars template with the data
        debug!("Render template");
        let rendered = handlebars.render("index", &data, &html_config.playpen)?;

        utils::fs::write_file(&destination, "print.html", rendered.as_bytes())?;
        debug!("Creating print.html ✓");

        debug!("Copy static files");
        self.copy_static_files(&destination, &theme, &html_config)
            .chain_err(|| "Unable to copy across static files")?;
        self.copy_additional_css_and_js(&html_config, &ctx.root, &destination)
            .chain_err(|| "Unable to copy across additional CSS and JS")?;

        // Render search index
        #[cfg(feature = "search")]
        {
            let search = html_config.search.unwrap_or_default();
            if search.enable {
                super::search::create_files(&search, &destination, &book)?;
            }
        }

        // Copy all remaining files
        utils::fs::copy_files_except_ext(&src_dir, &destination, true, &["md"])?;

        Ok(())
    }
}

fn make_data(
    root: &Path,
    book: &Book,
    config: &Config,
    html_config: &HtmlConfig,
) -> Result<serde_json::Map<String, serde_json::Value>> {
    trace!("make_data");
    let html = config.html_config().unwrap_or_default();

    let mut data = serde_json::Map::new();
    data.insert(
        "language".to_owned(),
        json!(config.book.language.clone().unwrap_or_default()),
    );
    data.insert(
        "book_title".to_owned(),
        json!(config.book.title.clone().unwrap_or_default()),
    );
    data.insert(
        "description".to_owned(),
        json!(config.book.description.clone().unwrap_or_default()),
    );
    data.insert("favicon".to_owned(), json!("favicon.png"));
    if let Some(ref livereload) = html_config.livereload_url {
        data.insert("livereload".to_owned(), json!(livereload));
    }

    let default_theme = match html_config.default_theme {
        Some(ref theme) => theme,
        None => "light",
    };
    data.insert("default_theme".to_owned(), json!(default_theme));

    // Add google analytics tag
    if let Some(ref ga) = config.html_config().and_then(|html| html.google_analytics) {
        data.insert("google_analytics".to_owned(), json!(ga));
    }

    if html.mathjax_support {
        data.insert("mathjax_support".to_owned(), json!(true));
    }

    // Add check to see if there is an additional style
    if !html.additional_css.is_empty() {
        let mut css = Vec::new();
        for style in &html.additional_css {
            match style.strip_prefix(root) {
                Ok(p) => css.push(p.to_str().expect("Could not convert to str")),
                Err(_) => css.push(style.to_str().expect("Could not convert to str")),
            }
        }
        data.insert("additional_css".to_owned(), json!(css));
    }

    // Add check to see if there is an additional script
    if !html.additional_js.is_empty() {
        let mut js = Vec::new();
        for script in &html.additional_js {
            match script.strip_prefix(root) {
                Ok(p) => js.push(p.to_str().expect("Could not convert to str")),
                Err(_) => js.push(script.to_str().expect("Could not convert to str")),
            }
        }
        data.insert("additional_js".to_owned(), json!(js));
    }

    if html.playpen.editable && html.playpen.copy_js {
        data.insert("playpen_js".to_owned(), json!(true));
    }

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
        None => "fa-github",
    };
    data.insert("git_repository_icon".to_owned(), json!(git_repository_icon));

    let mut chapters = vec![];

    for item in book.iter() {
        // Create the data to inject in the template
        let mut chapter = BTreeMap::new();

        match *item {
            BookItem::Chapter(ref ch) => {
                if let Some(ref section) = ch.number {
                    chapter.insert("section".to_owned(), json!(section.to_string()));
                }

                chapter.insert("name".to_owned(), json!(ch.name));
                let path = ch
                    .path
                    .to_str()
                    .chain_err(|| "Could not convert path to str")?;
                chapter.insert("path".to_owned(), json!(path));
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

struct RenderItemContext<'a> {
    handlebars: &'a HbsWrapper,
    destination: PathBuf,
    data: serde_json::Map<String, serde_json::Value>,
    is_index: bool,
    html_config: HtmlConfig,
}
