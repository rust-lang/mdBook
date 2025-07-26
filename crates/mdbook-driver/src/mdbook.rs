//! The high-level interface for loading and rendering books.

use crate::builtin_preprocessors::{CmdPreprocessor, IndexPreprocessor, LinkPreprocessor};
use crate::builtin_renderers::{CmdRenderer, MarkdownRenderer};
use crate::init::BookBuilder;
use crate::load::{load_book, load_book_from_disk};
use anyhow::{Context, Error, Result, bail};
use log::{debug, error, info, log_enabled, trace, warn};
use mdbook_core::book::{Book, BookItem, BookItems};
use mdbook_core::config::{Config, RustEdition};
use mdbook_core::utils;
use mdbook_html::HtmlHandlebars;
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use mdbook_renderer::{RenderContext, Renderer};
use mdbook_summary::Summary;
use serde::Deserialize;
use std::collections::HashMap;
use std::ffi::OsString;
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::Builder as TempFileBuilder;
use topological_sort::TopologicalSort;

#[cfg(test)]
mod tests;

/// The object used to manage and build a book.
pub struct MDBook {
    /// The book's root directory.
    pub root: PathBuf,
    /// The configuration used to tweak now a book is built.
    pub config: Config,
    /// A representation of the book's contents in memory.
    pub book: Book,
    renderers: Vec<Box<dyn Renderer>>,

    /// List of pre-processors to be run on the book.
    preprocessors: Vec<Box<dyn Preprocessor>>,
}

impl MDBook {
    /// Load a book from its root directory on disk.
    pub fn load<P: Into<PathBuf>>(book_root: P) -> Result<MDBook> {
        let book_root = book_root.into();
        let config_location = book_root.join("book.toml");

        // the book.json file is no longer used, so we should emit a warning to
        // let people know to migrate to book.toml
        if book_root.join("book.json").exists() {
            warn!("It appears you are still using book.json for configuration.");
            warn!("This format is no longer used, so you should migrate to the");
            warn!("book.toml format.");
            warn!("Check the user guide for migration information:");
            warn!("\thttps://rust-lang.github.io/mdBook/format/config.html");
        }

        let mut config = if config_location.exists() {
            debug!("Loading config from {}", config_location.display());
            Config::from_disk(&config_location)?
        } else {
            Config::default()
        };

        config.update_from_env();

        if let Some(html_config) = config.html_config() {
            if html_config.curly_quotes {
                warn!(
                    "The output.html.curly-quotes field has been renamed to \
                     output.html.smart-punctuation.\n\
                     Use the new name in book.toml to remove this warning."
                );
            }
        }

        if log_enabled!(log::Level::Trace) {
            for line in format!("Config: {config:#?}").lines() {
                trace!("{}", line);
            }
        }

        MDBook::load_with_config(book_root, config)
    }

    /// Load a book from its root directory using a custom `Config`.
    pub fn load_with_config<P: Into<PathBuf>>(book_root: P, config: Config) -> Result<MDBook> {
        let root = book_root.into();

        let src_dir = root.join(&config.book.src);
        let book = load_book(src_dir, &config.build)?;

        let renderers = determine_renderers(&config)?;
        let preprocessors = determine_preprocessors(&config)?;

        Ok(MDBook {
            root,
            config,
            book,
            renderers,
            preprocessors,
        })
    }

    /// Load a book from its root directory using a custom `Config` and a custom summary.
    pub fn load_with_config_and_summary<P: Into<PathBuf>>(
        book_root: P,
        config: Config,
        summary: Summary,
    ) -> Result<MDBook> {
        let root = book_root.into();

        let src_dir = root.join(&config.book.src);
        let book = load_book_from_disk(&summary, src_dir)?;

        let renderers = determine_renderers(&config)?;
        let preprocessors = determine_preprocessors(&config)?;

        Ok(MDBook {
            root,
            config,
            book,
            renderers,
            preprocessors,
        })
    }

    /// Returns a flat depth-first iterator over the [`BookItem`]s of the book.
    ///
    /// ```no_run
    /// # use mdbook_driver::MDBook;
    /// # use mdbook_driver::book::BookItem;
    /// # let book = MDBook::load("mybook").unwrap();
    /// for item in book.iter() {
    ///     match *item {
    ///         BookItem::Chapter(ref chapter) => {},
    ///         BookItem::Separator => {},
    ///         BookItem::PartTitle(ref title) => {}
    ///     }
    /// }
    ///
    /// // would print something like this:
    /// // 1. Chapter 1
    /// // 1.1 Sub Chapter
    /// // 1.2 Sub Chapter
    /// // 2. Chapter 2
    /// //
    /// // etc.
    /// ```
    pub fn iter(&self) -> BookItems<'_> {
        self.book.iter()
    }

    /// `init()` gives you a `BookBuilder` which you can use to setup a new book
    /// and its accompanying directory structure.
    ///
    /// The `BookBuilder` creates some boilerplate files and directories to get
    /// you started with your book.
    ///
    /// ```text
    /// book-test/
    /// ├── book
    /// └── src
    ///     ├── chapter_1.md
    ///     └── SUMMARY.md
    /// ```
    ///
    /// It uses the path provided as the root directory for your book, then adds
    /// in a `src/` directory containing a `SUMMARY.md` and `chapter_1.md` file
    /// to get you started.
    pub fn init<P: Into<PathBuf>>(book_root: P) -> BookBuilder {
        BookBuilder::new(book_root)
    }

    /// Tells the renderer to build our book and put it in the build directory.
    pub fn build(&self) -> Result<()> {
        info!("Book building has started");

        for renderer in &self.renderers {
            self.execute_build_process(&**renderer)?;
        }

        Ok(())
    }

    /// Run preprocessors and return the final book.
    pub fn preprocess_book(&self, renderer: &dyn Renderer) -> Result<(Book, PreprocessorContext)> {
        let preprocess_ctx = PreprocessorContext::new(
            self.root.clone(),
            self.config.clone(),
            renderer.name().to_string(),
        );
        let mut preprocessed_book = self.book.clone();
        for preprocessor in &self.preprocessors {
            if preprocessor_should_run(&**preprocessor, renderer, &self.config)? {
                debug!("Running the {} preprocessor.", preprocessor.name());
                preprocessed_book = preprocessor.run(&preprocess_ctx, preprocessed_book)?;
            }
        }
        Ok((preprocessed_book, preprocess_ctx))
    }

    /// Run the entire build process for a particular [`Renderer`].
    pub fn execute_build_process(&self, renderer: &dyn Renderer) -> Result<()> {
        let (preprocessed_book, preprocess_ctx) = self.preprocess_book(renderer)?;

        let name = renderer.name();
        let build_dir = self.build_dir_for(name);

        let mut render_context = RenderContext::new(
            self.root.clone(),
            preprocessed_book,
            self.config.clone(),
            build_dir,
        );
        render_context
            .chapter_titles
            .extend(preprocess_ctx.chapter_titles.borrow_mut().drain());

        info!("Running the {} backend", renderer.name());
        renderer
            .render(&render_context)
            .with_context(|| "Rendering failed")
    }

    /// You can change the default renderer to another one by using this method.
    /// The only requirement is that your renderer implement the [`Renderer`]
    /// trait.
    pub fn with_renderer<R: Renderer + 'static>(&mut self, renderer: R) -> &mut Self {
        self.renderers.push(Box::new(renderer));
        self
    }

    /// Register a [`Preprocessor`] to be used when rendering the book.
    pub fn with_preprocessor<P: Preprocessor + 'static>(&mut self, preprocessor: P) -> &mut Self {
        self.preprocessors.push(Box::new(preprocessor));
        self
    }

    /// Run `rustdoc` tests on the book, linking against the provided libraries.
    pub fn test(&mut self, library_paths: Vec<&str>) -> Result<()> {
        // test_chapter with chapter:None will run all tests.
        self.test_chapter(library_paths, None)
    }

    /// Run `rustdoc` tests on a specific chapter of the book, linking against the provided libraries.
    /// If `chapter` is `None`, all tests will be run.
    pub fn test_chapter(&mut self, library_paths: Vec<&str>, chapter: Option<&str>) -> Result<()> {
        let cwd = std::env::current_dir()?;
        let library_args: Vec<OsString> = library_paths
            .into_iter()
            .flat_map(|path| {
                let path = Path::new(path);
                let path = if path.is_relative() {
                    cwd.join(path).into_os_string()
                } else {
                    path.to_path_buf().into_os_string()
                };
                [OsString::from("-L"), path]
            })
            .collect();

        let temp_dir = TempFileBuilder::new().prefix("mdbook-").tempdir()?;

        let mut chapter_found = false;

        struct TestRenderer;
        impl Renderer for TestRenderer {
            // FIXME: Is "test" the proper renderer name to use here?
            fn name(&self) -> &str {
                "test"
            }

            fn render(&self, _: &RenderContext) -> Result<()> {
                Ok(())
            }
        }

        // Index Preprocessor is disabled so that chapter paths
        // continue to point to the actual markdown files.
        self.preprocessors = determine_preprocessors(&self.config)?
            .into_iter()
            .filter(|pre| pre.name() != IndexPreprocessor::NAME)
            .collect();
        let (book, _) = self.preprocess_book(&TestRenderer)?;

        let color_output = std::io::stderr().is_terminal();
        let mut failed = false;
        for item in book.iter() {
            if let BookItem::Chapter(ref ch) = *item {
                let chapter_path = match ch.path {
                    Some(ref path) if !path.as_os_str().is_empty() => path,
                    _ => continue,
                };

                if let Some(chapter) = chapter {
                    if ch.name != chapter && chapter_path.to_str() != Some(chapter) {
                        if chapter == "?" {
                            info!("Skipping chapter '{}'...", ch.name);
                        }
                        continue;
                    }
                }
                chapter_found = true;
                info!("Testing chapter '{}': {:?}", ch.name, chapter_path);

                // write preprocessed file to tempdir
                let path = temp_dir.path().join(chapter_path);
                let mut tmpf = utils::fs::create_file(&path)?;
                tmpf.write_all(ch.content.as_bytes())?;

                let mut cmd = Command::new("rustdoc");
                cmd.current_dir(temp_dir.path())
                    .arg(chapter_path)
                    .arg("--test")
                    .args(&library_args);

                if let Some(edition) = self.config.rust.edition {
                    match edition {
                        RustEdition::E2015 => {
                            cmd.args(["--edition", "2015"]);
                        }
                        RustEdition::E2018 => {
                            cmd.args(["--edition", "2018"]);
                        }
                        RustEdition::E2021 => {
                            cmd.args(["--edition", "2021"]);
                        }
                        RustEdition::E2024 => {
                            cmd.args(["--edition", "2024"]);
                        }
                    }
                }

                if color_output {
                    cmd.args(["--color", "always"]);
                }

                debug!("running {:?}", cmd);
                let output = cmd
                    .output()
                    .with_context(|| "failed to execute `rustdoc`")?;

                if !output.status.success() {
                    failed = true;
                    error!(
                        "rustdoc returned an error:\n\
                        \n--- stdout\n{}\n--- stderr\n{}",
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            }
        }
        if failed {
            bail!("One or more tests failed");
        }
        if let Some(chapter) = chapter {
            if !chapter_found {
                bail!("Chapter not found: {}", chapter);
            }
        }
        Ok(())
    }

    /// The logic for determining where a backend should put its build
    /// artefacts.
    ///
    /// If there is only 1 renderer, put it in the directory pointed to by the
    /// `build.build_dir` key in [`Config`]. If there is more than one then the
    /// renderer gets its own directory within the main build dir.
    ///
    /// i.e. If there were only one renderer (in this case, the HTML renderer):
    ///
    /// - build/
    ///   - index.html
    ///   - ...
    ///
    /// Otherwise if there are multiple:
    ///
    /// - build/
    ///   - epub/
    ///     - my_awesome_book.epub
    ///   - html/
    ///     - index.html
    ///     - ...
    ///   - latex/
    ///     - my_awesome_book.tex
    ///
    pub fn build_dir_for(&self, backend_name: &str) -> PathBuf {
        let build_dir = self.root.join(&self.config.build.build_dir);

        if self.renderers.len() <= 1 {
            build_dir
        } else {
            build_dir.join(backend_name)
        }
    }

    /// Get the directory containing this book's source files.
    pub fn source_dir(&self) -> PathBuf {
        self.root.join(&self.config.book.src)
    }

    /// Get the directory containing the theme resources for the book.
    pub fn theme_dir(&self) -> PathBuf {
        self.config
            .html_config()
            .unwrap_or_default()
            .theme_dir(&self.root)
    }
}

/// An `output` table.
#[derive(Deserialize)]
struct OutputConfig {
    command: Option<String>,
}

/// Look at the `Config` and try to figure out what renderers to use.
fn determine_renderers(config: &Config) -> Result<Vec<Box<dyn Renderer>>> {
    let mut renderers = Vec::new();

    match config.get::<HashMap<String, OutputConfig>>("output") {
        Ok(Some(output_table)) => {
            renderers.extend(output_table.into_iter().map(|(key, table)| {
                if key == "html" {
                    Box::new(HtmlHandlebars::new()) as Box<dyn Renderer>
                } else if key == "markdown" {
                    Box::new(MarkdownRenderer::new()) as Box<dyn Renderer>
                } else {
                    let command = table.command.unwrap_or_else(|| format!("mdbook-{key}"));
                    Box::new(CmdRenderer::new(key, command))
                }
            }));
        }
        Ok(None) => {}
        Err(e) => bail!("failed to get output table config: {e}"),
    }

    // if we couldn't find anything, add the HTML renderer as a default
    if renderers.is_empty() {
        renderers.push(Box::new(HtmlHandlebars::new()));
    }

    Ok(renderers)
}

const DEFAULT_PREPROCESSORS: &[&str] = &["links", "index"];

fn is_default_preprocessor(pre: &dyn Preprocessor) -> bool {
    let name = pre.name();
    name == LinkPreprocessor::NAME || name == IndexPreprocessor::NAME
}

/// A `preprocessor` table.
#[derive(Deserialize)]
struct PreprocessorConfig {
    command: Option<String>,
    #[serde(default)]
    before: Vec<String>,
    #[serde(default)]
    after: Vec<String>,
}

/// Look at the `MDBook` and try to figure out what preprocessors to run.
fn determine_preprocessors(config: &Config) -> Result<Vec<Box<dyn Preprocessor>>> {
    // Collect the names of all preprocessors intended to be run, and the order
    // in which they should be run.
    let mut preprocessor_names = TopologicalSort::<String>::new();

    if config.build.use_default_preprocessors {
        for name in DEFAULT_PREPROCESSORS {
            preprocessor_names.insert(name.to_string());
        }
    }

    let preprocessor_table = match config.get::<HashMap<String, PreprocessorConfig>>("preprocessor")
    {
        Ok(Some(preprocessor_table)) => preprocessor_table,
        Ok(None) => HashMap::new(),
        Err(e) => bail!("failed to get preprocessor table config: {e}"),
    };

    for (name, table) in preprocessor_table.iter() {
        preprocessor_names.insert(name.to_string());

        let exists = |name| {
            (config.build.use_default_preprocessors && DEFAULT_PREPROCESSORS.contains(&name))
                || preprocessor_table.contains_key(name)
        };

        for after in &table.before {
            if !exists(&after) {
                // Only warn so that preprocessors can be toggled on and off (e.g. for
                // troubleshooting) without having to worry about order too much.
                warn!(
                    "preprocessor.{}.after contains \"{}\", which was not found",
                    name, after
                );
            } else {
                preprocessor_names.add_dependency(name, after);
            }
        }

        for before in &table.after {
            if !exists(&before) {
                // See equivalent warning above for rationale
                warn!(
                    "preprocessor.{}.before contains \"{}\", which was not found",
                    name, before
                );
            } else {
                preprocessor_names.add_dependency(before, name);
            }
        }
    }

    // Now that all links have been established, queue preprocessors in a suitable order
    let mut preprocessors = Vec::with_capacity(preprocessor_names.len());
    // `pop_all()` returns an empty vector when no more items are not being depended upon
    for mut names in std::iter::repeat_with(|| preprocessor_names.pop_all())
        .take_while(|names| !names.is_empty())
    {
        // The `topological_sort` crate does not guarantee a stable order for ties, even across
        // runs of the same program. Thus, we break ties manually by sorting.
        // Careful: `str`'s default sorting, which we are implicitly invoking here, uses code point
        // values ([1]), which may not be an alphabetical sort.
        // As mentioned in [1], doing so depends on locale, which is not desirable for deciding
        // preprocessor execution order.
        // [1]: https://doc.rust-lang.org/stable/std/cmp/trait.Ord.html#impl-Ord-14
        names.sort();
        for name in names {
            let preprocessor: Box<dyn Preprocessor> = match name.as_str() {
                "links" => Box::new(LinkPreprocessor::new()),
                "index" => Box::new(IndexPreprocessor::new()),
                _ => {
                    // The only way to request a custom preprocessor is through the `preprocessor`
                    // table, so it must exist, be a table, and contain the key.
                    let table = &preprocessor_table[&name];
                    let command = table
                        .command
                        .to_owned()
                        .unwrap_or_else(|| format!("mdbook-{name}"));
                    Box::new(CmdPreprocessor::new(name, command))
                }
            };
            preprocessors.push(preprocessor);
        }
    }

    // "If `pop_all` returns an empty vector and `len` is not 0, there are cyclic dependencies."
    // Normally, `len() == 0` is equivalent to `is_empty()`, so we'll use that.
    if preprocessor_names.is_empty() {
        Ok(preprocessors)
    } else {
        Err(Error::msg("Cyclic dependency detected in preprocessors"))
    }
}

/// Check whether we should run a particular `Preprocessor` in combination
/// with the renderer, falling back to `Preprocessor::supports_renderer()`
/// method if the user doesn't say anything.
///
/// The `build.use-default-preprocessors` config option can be used to ensure
/// default preprocessors always run if they support the renderer.
fn preprocessor_should_run(
    preprocessor: &dyn Preprocessor,
    renderer: &dyn Renderer,
    cfg: &Config,
) -> Result<bool> {
    // default preprocessors should be run by default (if supported)
    if cfg.build.use_default_preprocessors && is_default_preprocessor(preprocessor) {
        return Ok(preprocessor.supports_renderer(renderer.name()));
    }

    let key = format!("preprocessor.{}.renderers", preprocessor.name());
    let renderer_name = renderer.name();

    match cfg.get::<Vec<String>>(&key) {
        Ok(Some(explicit_renderers)) => {
            Ok(explicit_renderers.iter().any(|name| name == renderer_name))
        }
        Ok(None) => Ok(preprocessor.supports_renderer(renderer_name)),
        Err(e) => bail!("failed to get `{key}`: {e}"),
    }
}
