//! The internal representation of a book and infrastructure for loading it from
//! disk and building it.
//!
//! For examples on using `MDBook`, consult the [top-level documentation][1].
//!
//! [1]: ../index.html

mod summary;
mod book;
mod init;

pub use self::book::{load_book, Book, BookItem, BookItems, Chapter};
pub use self::summary::{parse_summary, Link, SectionNumber, Summary, SummaryItem};
pub use self::init::BookBuilder;

use std::path::PathBuf;
use std::io::Write;
use std::process::Command;
use tempfile::Builder as TempFileBuilder;
use toml::Value;

use utils;
use renderer::{CmdRenderer, HtmlHandlebars, RenderContext, Renderer};
use preprocess::{LinkPreprocessor, Preprocessor, PreprocessorContext};
use errors::*;

use config::Config;

/// The object used to manage and build a book.
pub struct MDBook {
    /// The book's root directory.
    pub root: PathBuf,
    /// The configuration used to tweak now a book is built.
    pub config: Config,
    /// A representation of the book's contents in memory.
    pub book: Book,
    renderers: Vec<Box<Renderer>>,

    /// List of pre-processors to be run on the book
    preprocessors: Vec<Box<Preprocessor>>,
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
            warn!("\thttps://rust-lang-nursery.github.io/mdBook/format/config.html");
        }

        let mut config = if config_location.exists() {
            debug!("Loading config from {}", config_location.display());
            Config::from_disk(&config_location)?
        } else {
            Config::default()
        };

        config.update_from_env();

        if log_enabled!(::log::Level::Trace) {
            for line in format!("Config: {:#?}", config).lines() {
                trace!("{}", line);
            }
        }

        MDBook::load_with_config(book_root, config)
    }

    /// Load a book from its root directory using a custom config.
    pub fn load_with_config<P: Into<PathBuf>>(book_root: P, config: Config) -> Result<MDBook> {
        let root = book_root.into();

        let src_dir = root.join(&config.book.src);
        let book = book::load_book(&src_dir, &config.build)?;

        let renderers = determine_renderers(&config);
        let preprocessors = determine_preprocessors(&config)?;

        Ok(MDBook {
            root,
            config,
            book,
            renderers,
            preprocessors,
        })
    }

    /// Returns a flat depth-first iterator over the elements of the book,
    /// it returns an [BookItem enum](bookitem.html):
    /// `(section: String, bookitem: &BookItem)`
    ///
    /// ```no_run
    /// # extern crate mdbook;
    /// # use mdbook::MDBook;
    /// # use mdbook::book::BookItem;
    /// # #[allow(unused_variables)]
    /// # fn main() {
    /// # let book = MDBook::load("mybook").unwrap();
    /// for item in book.iter() {
    ///     match *item {
    ///         BookItem::Chapter(ref chapter) => {},
    ///         BookItem::Separator => {},
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
    /// # }
    /// ```
    pub fn iter(&self) -> BookItems {
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

        let mut preprocessed_book = self.book.clone();
        let preprocess_ctx = PreprocessorContext::new(self.root.clone(), self.config.clone());

        for preprocessor in &self.preprocessors {
            debug!("Running the {} preprocessor.", preprocessor.name());
            preprocessor.run(&preprocess_ctx, &mut preprocessed_book)?;
        }

        for renderer in &self.renderers {
            info!("Running the {} backend", renderer.name());
            self.run_renderer(&preprocessed_book, renderer.as_ref())?;
        }

        Ok(())
    }

    fn run_renderer(&self, preprocessed_book: &Book, renderer: &Renderer) -> Result<()> {
        let name = renderer.name();
        let build_dir = self.build_dir_for(name);
        if build_dir.exists() {
            debug!(
                "Cleaning build dir for the \"{}\" renderer ({})",
                name,
                build_dir.display()
            );

            utils::fs::remove_dir_content(&build_dir)
                .chain_err(|| "Unable to clear output directory")?;
        }

        let render_context = RenderContext::new(
            self.root.clone(),
            preprocessed_book.clone(),
            self.config.clone(),
            build_dir,
        );

        renderer
            .render(&render_context)
            .chain_err(|| "Rendering failed")
    }

    /// You can change the default renderer to another one by using this method.
    /// The only requirement is for your renderer to implement the [`Renderer`
    /// trait](../renderer/trait.Renderer.html)
    pub fn with_renderer<R: Renderer + 'static>(&mut self, renderer: R) -> &mut Self {
        self.renderers.push(Box::new(renderer));
        self
    }

    /// Register a [`Preprocessor`](../preprocess/trait.Preprocessor.html) to be used when rendering the book.
    pub fn with_preprecessor<P: Preprocessor + 'static>(&mut self, preprocessor: P) -> &mut Self {
        self.preprocessors.push(Box::new(preprocessor));
        self
    }

    /// Run `rustdoc` tests on the book, linking against the provided libraries.
    pub fn test(&mut self, library_paths: Vec<&str>) -> Result<()> {
        let library_args: Vec<&str> = (0..library_paths.len())
            .map(|_| "-L")
            .zip(library_paths.into_iter())
            .flat_map(|x| vec![x.0, x.1])
            .collect();

        let temp_dir = TempFileBuilder::new().prefix("mdbook").tempdir()?;

        let preprocess_context = PreprocessorContext::new(self.root.clone(), self.config.clone());

        LinkPreprocessor::new().run(&preprocess_context, &mut self.book)?;

        for item in self.iter() {
            if let BookItem::Chapter(ref ch) = *item {
                if !ch.path.as_os_str().is_empty() {
                    let path = self.source_dir().join(&ch.path);
                    let content = utils::fs::file_to_string(&path)?;
                    info!("Testing file: {:?}", path);

                    // write preprocessed file to tempdir
                    let path = temp_dir.path().join(&ch.path);
                    let mut tmpf = utils::fs::create_file(&path)?;
                    tmpf.write_all(content.as_bytes())?;

                    let output = Command::new("rustdoc")
                        .arg(&path)
                        .arg("--test")
                        .args(&library_args)
                        .output()?;

                    if !output.status.success() {
                        bail!(ErrorKind::Subprocess(
                            "Rustdoc returned an error".to_string(),
                            output
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// The logic for determining where a backend should put its build
    /// artefacts.
    ///
    /// If there is only 1 renderer, put it in the directory pointed to by the
    /// `build.build_dir` key in `Config`. If there is more than one then the
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

/// Look at the `Config` and try to figure out what renderers to use.
fn determine_renderers(config: &Config) -> Vec<Box<Renderer>> {
    let mut renderers: Vec<Box<Renderer>> = Vec::new();

    if let Some(output_table) = config.get("output").and_then(|o| o.as_table()) {
        for (key, table) in output_table.iter() {
            // the "html" backend has its own Renderer
            if key == "html" {
                renderers.push(Box::new(HtmlHandlebars::new()));
            } else {
                let renderer = interpret_custom_renderer(key, table);
                renderers.push(renderer);
            }
        }
    }

    // if we couldn't find anything, add the HTML renderer as a default
    if renderers.is_empty() {
        renderers.push(Box::new(HtmlHandlebars::new()));
    }

    renderers
}

fn default_preprocessors() -> Vec<Box<Preprocessor>> {
    vec![Box::new(LinkPreprocessor::new())]
}

/// Look at the `MDBook` and try to figure out what preprocessors to run.
fn determine_preprocessors(config: &Config) -> Result<Vec<Box<Preprocessor>>> {
    let preprocess_list = match config.build.preprocess {
        Some(ref p) => p,
        // If no preprocessor field is set, default to the LinkPreprocessor. This allows you
        // to disable the LinkPreprocessor by setting "preprocess" to an empty list.
        None => return Ok(default_preprocessors()),
    };

    let mut preprocessors: Vec<Box<Preprocessor>> = Vec::new();

    for key in preprocess_list {
        match key.as_ref() {
            "links" => preprocessors.push(Box::new(LinkPreprocessor::new())),
            _ => bail!("{:?} is not a recognised preprocessor", key),
        }
    }

    Ok(preprocessors)
}

fn interpret_custom_renderer(key: &str, table: &Value) -> Box<Renderer> {
    // look for the `command` field, falling back to using the key
    // prepended by "mdbook-"
    let table_dot_command = table
        .get("command")
        .and_then(|c| c.as_str())
        .map(|s| s.to_string());

    let command = table_dot_command.unwrap_or_else(|| format!("mdbook-{}", key));

    Box::new(CmdRenderer::new(key.to_string(), command.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use toml::value::{Table, Value};

    #[test]
    fn config_defaults_to_html_renderer_if_empty() {
        let cfg = Config::default();

        // make sure we haven't got anything in the `output` table
        assert!(cfg.get("output").is_none());

        let got = determine_renderers(&cfg);

        assert_eq!(got.len(), 1);
        assert_eq!(got[0].name(), "html");
    }

    #[test]
    fn add_a_random_renderer_to_the_config() {
        let mut cfg = Config::default();
        cfg.set("output.random", Table::new()).unwrap();

        let got = determine_renderers(&cfg);

        assert_eq!(got.len(), 1);
        assert_eq!(got[0].name(), "random");
    }

    #[test]
    fn add_a_random_renderer_with_custom_command_to_the_config() {
        let mut cfg = Config::default();

        let mut table = Table::new();
        table.insert("command".to_string(), Value::String("false".to_string()));
        cfg.set("output.random", table).unwrap();

        let got = determine_renderers(&cfg);

        assert_eq!(got.len(), 1);
        assert_eq!(got[0].name(), "random");
    }

    #[test]
    fn config_defaults_to_link_preprocessor_if_not_set() {
        let cfg = Config::default();

        // make sure we haven't got anything in the `output` table
        assert!(cfg.build.preprocess.is_none());

        let got = determine_preprocessors(&cfg);

        assert!(got.is_ok());
        assert_eq!(got.as_ref().unwrap().len(), 1);
        assert_eq!(got.as_ref().unwrap()[0].name(), "links");
    }

    #[test]
    fn config_doesnt_default_if_empty() {
        let cfg_str: &'static str = r#"
        [book]
        title = "Some Book"

        [build]
        build-dir = "outputs"
        create-missing = false
        preprocess = []
        "#;

        let cfg = Config::from_str(cfg_str).unwrap();

        // make sure we have something in the `output` table
        assert!(cfg.build.preprocess.is_some());

        let got = determine_preprocessors(&cfg);

        assert!(got.is_ok());
        assert!(got.unwrap().is_empty());
    }

    #[test]
    fn config_complains_if_unimplemented_preprocessor() {
        let cfg_str: &'static str = r#"
        [book]
        title = "Some Book"

        [build]
        build-dir = "outputs"
        create-missing = false
        preprocess = ["random"]
        "#;

        let cfg = Config::from_str(cfg_str).unwrap();

        // make sure we have something in the `output` table
        assert!(cfg.build.preprocess.is_some());

        let got = determine_preprocessors(&cfg);

        assert!(got.is_err());
    }
}
