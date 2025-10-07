//! Mdbook's configuration system.
//!
//! The main entrypoint of the `config` module is the `Config` struct. This acts
//! essentially as a bag of configuration information, with a couple
//! pre-determined tables ([`BookConfig`] and [`BuildConfig`]) as well as support
//! for arbitrary data which is exposed to plugins and alternative backends.
//!
//!
//! # Examples
//!
//! ```rust
//! # use anyhow::Result;
//! use std::path::PathBuf;
//! use std::str::FromStr;
//! use mdbook_core::config::Config;
//!
//! # fn run() -> Result<()> {
//! let src = r#"
//! [book]
//! title = "My Book"
//! authors = ["Michael-F-Bryan"]
//!
//! [preprocessor.my-preprocessor]
//! bar = 123
//! "#;
//!
//! // load the `Config` from a toml string
//! let mut cfg = Config::from_str(src)?;
//!
//! // retrieve a nested value
//! let bar = cfg.get::<i32>("preprocessor.my-preprocessor.bar")?;
//! assert_eq!(bar, Some(123));
//!
//! // Set the `output.html.theme` directory
//! assert!(cfg.get::<toml::Value>("output.html")?.is_none());
//! cfg.set("output.html.theme", "./themes");
//!
//! // then load it again, automatically deserializing to a `PathBuf`.
//! let got = cfg.get("output.html.theme")?;
//! assert_eq!(got, Some(PathBuf::from("./themes")));
//! # Ok(())
//! # }
//! # run().unwrap()
//! ```

use crate::utils::{TomlExt, fs, log_backtrace};
use anyhow::{Context, Error, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::env;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use toml::Value;
use toml::value::Table;
use tracing::{debug, trace};

/// The overall configuration object for MDBook, essentially an in-memory
/// representation of `book.toml`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
#[non_exhaustive]
pub struct Config {
    /// Metadata about the book.
    pub book: BookConfig,
    /// Information about the build environment.
    #[serde(skip_serializing_if = "is_default")]
    pub build: BuildConfig,
    /// Information about Rust language support.
    #[serde(skip_serializing_if = "is_default")]
    pub rust: RustConfig,
    /// The renderer configurations.
    #[serde(skip_serializing_if = "toml_is_empty")]
    output: Value,
    /// The preprocessor configurations.
    #[serde(skip_serializing_if = "toml_is_empty")]
    preprocessor: Value,
}

/// Helper for serde serialization.
fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

/// Helper for serde serialization.
fn toml_is_empty(table: &Value) -> bool {
    table.as_table().unwrap().is_empty()
}

impl FromStr for Config {
    type Err = Error;

    /// Load a `Config` from some string.
    fn from_str(src: &str) -> Result<Self> {
        toml::from_str(src).with_context(|| "Invalid configuration file")
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            book: BookConfig::default(),
            build: BuildConfig::default(),
            rust: RustConfig::default(),
            output: Value::Table(Table::default()),
            preprocessor: Value::Table(Table::default()),
        }
    }
}

impl Config {
    /// Load the configuration file from disk.
    pub fn from_disk<P: AsRef<Path>>(config_file: P) -> Result<Config> {
        let cfg = fs::read_to_string(config_file)?;
        Config::from_str(&cfg)
    }

    /// Updates the `Config` from the available environment variables.
    ///
    /// Variables starting with `MDBOOK_` are used for configuration. The key is
    /// created by removing the `MDBOOK_` prefix and turning the resulting
    /// string into `kebab-case`. Double underscores (`__`) separate nested
    /// keys, while a single underscore (`_`) is replaced with a dash (`-`).
    ///
    /// For example:
    ///
    /// - `MDBOOK_foo` -> `foo`
    /// - `MDBOOK_FOO` -> `foo`
    /// - `MDBOOK_FOO__BAR` -> `foo.bar`
    /// - `MDBOOK_FOO_BAR` -> `foo-bar`
    /// - `MDBOOK_FOO_bar__baz` -> `foo-bar.baz`
    ///
    /// So by setting the `MDBOOK_BOOK__TITLE` environment variable you can
    /// override the book's title without needing to touch your `book.toml`.
    ///
    /// > **Note:** To facilitate setting more complex config items, the value
    /// > of an environment variable is first parsed as JSON, falling back to a
    /// > string if the parse fails.
    /// >
    /// > This means, if you so desired, you could override all book metadata
    /// > when building the book with something like
    /// >
    /// > ```text
    /// > $ export MDBOOK_BOOK='{"title": "My Awesome Book", "authors": ["Michael-F-Bryan"]}'
    /// > $ mdbook build
    /// > ```
    ///
    /// The latter case may be useful in situations where `mdbook` is invoked
    /// from a script or CI, where it sometimes isn't possible to update the
    /// `book.toml` before building.
    pub fn update_from_env(&mut self) {
        debug!("Updating the config from environment variables");

        let overrides =
            env::vars().filter_map(|(key, value)| parse_env(&key).map(|index| (index, value)));

        for (key, value) in overrides {
            if key == "log" {
                // MDBOOK_LOG is used to control logging.
                continue;
            }
            trace!("{} => {}", key, value);
            let parsed_value = serde_json::from_str(&value)
                .unwrap_or_else(|_| serde_json::Value::String(value.to_string()));

            if key == "book" || key == "build" {
                if let serde_json::Value::Object(ref map) = parsed_value {
                    // To `set` each `key`, we wrap them as `prefix.key`
                    for (k, v) in map {
                        let full_key = format!("{key}.{k}");
                        self.set(&full_key, v).expect("unreachable");
                    }
                    return;
                }
            }

            self.set(key, parsed_value).expect("unreachable");
        }
    }

    /// Get a value from the configuration.
    ///
    /// This fetches a value from the book configuration. The key can have
    /// dotted indices to access nested items (e.g. `output.html.playground`
    /// will fetch the "playground" out of the html output table).
    ///
    /// This can only access the `output` and `preprocessor` tables.
    ///
    /// Returns `Ok(None)` if the field is not set.
    ///
    /// Returns `Err` if it fails to deserialize.
    pub fn get<'de, T: Deserialize<'de>>(&self, name: &str) -> Result<Option<T>> {
        let (key, table) = if let Some(key) = name.strip_prefix("output.") {
            (key, &self.output)
        } else if let Some(key) = name.strip_prefix("preprocessor.") {
            (key, &self.preprocessor)
        } else {
            bail!(
                "unable to get `{name}`, only `output` and `preprocessor` table entries are allowed"
            );
        };
        table
            .read(key)
            .map(|value| {
                value
                    .clone()
                    .try_into()
                    .with_context(|| "Failed to deserialize `{name}`")
            })
            .transpose()
    }

    /// Returns the configuration for all preprocessors.
    pub fn preprocessors<'de, T: Deserialize<'de>>(&self) -> Result<BTreeMap<String, T>> {
        self.preprocessor
            .clone()
            .try_into()
            .with_context(|| "Failed to read preprocessors")
    }

    /// Returns the configuration for all renderers.
    pub fn outputs<'de, T: Deserialize<'de>>(&self) -> Result<BTreeMap<String, T>> {
        self.output
            .clone()
            .try_into()
            .with_context(|| "Failed to read renderers")
    }

    /// Convenience method for getting the html renderer's configuration.
    ///
    /// # Note
    ///
    /// This is for compatibility only. It will be removed completely once the
    /// HTML renderer is refactored to be less coupled to `mdbook` internals.
    #[doc(hidden)]
    pub fn html_config(&self) -> Option<HtmlConfig> {
        match self.get("output.html") {
            Ok(Some(config)) => Some(config),
            Ok(None) => None,
            Err(e) => {
                log_backtrace(&e);
                None
            }
        }
    }

    /// Set a config key, clobbering any existing values along the way.
    ///
    /// The key can have dotted indices for nested items (e.g.
    /// `output.html.playground` will set the "playground" in the html output
    /// table).
    ///
    /// The only way this can fail is if we can't serialize `value` into a
    /// `toml::Value`.
    pub fn set<S: Serialize, I: AsRef<str>>(&mut self, index: I, value: S) -> Result<()> {
        let index = index.as_ref();

        let value = Value::try_from(value)
            .with_context(|| "Unable to represent the item as a JSON Value")?;

        if let Some(key) = index.strip_prefix("book.") {
            self.book.update_value(key, value);
        } else if let Some(key) = index.strip_prefix("build.") {
            self.build.update_value(key, value);
        } else if let Some(key) = index.strip_prefix("rust.") {
            self.rust.update_value(key, value);
        } else if let Some(key) = index.strip_prefix("output.") {
            self.output.update_value(key, value);
        } else if let Some(key) = index.strip_prefix("preprocessor.") {
            self.preprocessor.update_value(key, value);
        } else {
            bail!("invalid key `{index}`");
        }

        Ok(())
    }
}

fn parse_env(key: &str) -> Option<String> {
    key.strip_prefix("MDBOOK_")
        .map(|key| key.to_lowercase().replace("__", ".").replace('_', "-"))
}

/// Configuration options which are specific to the book and required for
/// loading it from disk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct BookConfig {
    /// The book's title.
    pub title: Option<String>,
    /// The book's authors.
    pub authors: Vec<String>,
    /// An optional description for the book.
    pub description: Option<String>,
    /// Location of the book source relative to the book's root directory.
    #[serde(skip_serializing_if = "is_default_src")]
    pub src: PathBuf,
    /// The main language of the book.
    pub language: Option<String>,
    /// The direction of text in the book: Left-to-right (LTR) or Right-to-left (RTL).
    /// When not specified, the text direction is derived from [`BookConfig::language`].
    pub text_direction: Option<TextDirection>,
}

/// Helper for serde serialization.
fn is_default_src(src: &PathBuf) -> bool {
    src == Path::new("src")
}

impl Default for BookConfig {
    fn default() -> BookConfig {
        BookConfig {
            title: None,
            authors: Vec::new(),
            description: None,
            src: PathBuf::from("src"),
            language: Some(String::from("en")),
            text_direction: None,
        }
    }
}

impl BookConfig {
    /// Gets the realized text direction, either from [`BookConfig::text_direction`]
    /// or derived from [`BookConfig::language`], to be used by templating engines.
    pub fn realized_text_direction(&self) -> TextDirection {
        if let Some(direction) = self.text_direction {
            direction
        } else {
            TextDirection::from_lang_code(self.language.as_deref().unwrap_or_default())
        }
    }
}

/// Text direction to use for HTML output
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TextDirection {
    /// Left to right.
    #[serde(rename = "ltr")]
    LeftToRight,
    /// Right to left
    #[serde(rename = "rtl")]
    RightToLeft,
}

impl TextDirection {
    /// Gets the text direction from language code
    pub fn from_lang_code(code: &str) -> Self {
        match code {
            // list sourced from here: https://github.com/abarrak/rtl/blob/master/lib/rtl/core.rb#L16
            "ar" | "ara" | "arc" | "ae" | "ave" | "egy" | "he" | "heb" | "nqo" | "pal" | "phn"
            | "sam" | "syc" | "syr" | "fa" | "per" | "fas" | "ku" | "kur" | "ur" | "urd"
            | "pus" | "ps" | "yi" | "yid" => TextDirection::RightToLeft,
            _ => TextDirection::LeftToRight,
        }
    }
}

/// Configuration for the build procedure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct BuildConfig {
    /// Where to put built artifacts relative to the book's root directory.
    pub build_dir: PathBuf,
    /// Should non-existent markdown files specified in `SUMMARY.md` be created
    /// if they don't exist?
    pub create_missing: bool,
    /// Should the default preprocessors always be used when they are
    /// compatible with the renderer?
    pub use_default_preprocessors: bool,
    /// Extra directories to trigger rebuild when watching/serving
    pub extra_watch_dirs: Vec<PathBuf>,
}

impl Default for BuildConfig {
    fn default() -> BuildConfig {
        BuildConfig {
            build_dir: PathBuf::from("book"),
            create_missing: true,
            use_default_preprocessors: true,
            extra_watch_dirs: Vec::new(),
        }
    }
}

/// Configuration for the Rust compiler(e.g., for playground)
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct RustConfig {
    /// Rust edition used in playground
    pub edition: Option<RustEdition>,
}

/// Rust edition to use for the code.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum RustEdition {
    /// The 2024 edition of Rust
    #[serde(rename = "2024")]
    E2024,
    /// The 2021 edition of Rust
    #[serde(rename = "2021")]
    E2021,
    /// The 2018 edition of Rust
    #[serde(rename = "2018")]
    E2018,
    /// The 2015 edition of Rust
    #[serde(rename = "2015")]
    E2015,
}

/// Configuration for the HTML renderer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct HtmlConfig {
    /// The theme directory, if specified.
    pub theme: Option<PathBuf>,
    /// The default theme to use, defaults to 'light'
    pub default_theme: Option<String>,
    /// The theme to use if the browser requests the dark version of the site.
    /// Defaults to 'navy'.
    pub preferred_dark_theme: Option<String>,
    /// Supports smart quotes, apostrophes, ellipsis, en-dash, and em-dash.
    pub smart_punctuation: bool,
    /// Support for definition lists.
    pub definition_lists: bool,
    /// Support for admonitions.
    pub admonitions: bool,
    /// Should mathjax be enabled?
    pub mathjax_support: bool,
    /// Additional CSS stylesheets to include in the rendered page's `<head>`.
    pub additional_css: Vec<PathBuf>,
    /// Additional JS scripts to include at the bottom of the rendered page's
    /// `<body>`.
    pub additional_js: Vec<PathBuf>,
    /// Fold settings.
    pub fold: Fold,
    /// Playground settings.
    #[serde(alias = "playpen")]
    pub playground: Playground,
    /// Code settings.
    pub code: Code,
    /// Print settings.
    pub print: Print,
    /// Don't render section labels.
    pub no_section_label: bool,
    /// Search settings. If `None`, the default will be used.
    pub search: Option<Search>,
    /// Git repository url. If `None`, the git button will not be shown.
    pub git_repository_url: Option<String>,
    /// FontAwesome icon class to use for the Git repository link.
    /// Defaults to `fa-github` if `None`.
    pub git_repository_icon: Option<String>,
    /// Input path for the 404 file, defaults to 404.md, set to "" to disable 404 file output
    pub input_404: Option<String>,
    /// Absolute url to site, used to emit correct paths for the 404 page, which might be accessed in a deeply nested directory
    pub site_url: Option<String>,
    /// The DNS subdomain or apex domain at which your book will be hosted. This
    /// string will be written to a file named CNAME in the root of your site,
    /// as required by GitHub Pages (see [*Managing a custom domain for your
    /// GitHub Pages site*][custom domain]).
    ///
    /// [custom domain]: https://docs.github.com/en/github/working-with-github-pages/managing-a-custom-domain-for-your-github-pages-site
    pub cname: Option<String>,
    /// Edit url template, when set shows a "Suggest an edit" button for
    /// directly jumping to editing the currently viewed page.
    /// Contains {path} that is replaced with chapter source file path
    pub edit_url_template: Option<String>,
    /// Endpoint of websocket, for livereload usage. Value loaded from .toml
    /// file is ignored, because our code overrides this field with an
    /// internal value (`LIVE_RELOAD_ENDPOINT)
    ///
    /// This config item *should not be edited* by the end user.
    #[doc(hidden)]
    pub live_reload_endpoint: Option<String>,
    /// The mapping from old pages to new pages/URLs to use when generating
    /// redirects.
    pub redirect: HashMap<String, String>,
    /// If this option is turned on, "cache bust" static files by adding
    /// hashes to their file names.
    ///
    /// The default is `true`.
    pub hash_files: bool,
    /// If enabled, the sidebar includes navigation for headers on the current
    /// page. Default is `true`.
    pub sidebar_header_nav: bool,
}

impl Default for HtmlConfig {
    fn default() -> HtmlConfig {
        HtmlConfig {
            theme: None,
            default_theme: None,
            preferred_dark_theme: None,
            smart_punctuation: true,
            definition_lists: true,
            admonitions: true,
            mathjax_support: false,
            additional_css: Vec::new(),
            additional_js: Vec::new(),
            fold: Fold::default(),
            playground: Playground::default(),
            code: Code::default(),
            print: Print::default(),
            no_section_label: false,
            search: None,
            git_repository_url: None,
            git_repository_icon: None,
            input_404: None,
            site_url: None,
            cname: None,
            edit_url_template: None,
            live_reload_endpoint: None,
            redirect: HashMap::new(),
            hash_files: true,
            sidebar_header_nav: true,
        }
    }
}

impl HtmlConfig {
    /// Returns the directory of theme from the provided root directory. If the
    /// directory is not present it will append the default directory of "theme"
    pub fn theme_dir(&self, root: &Path) -> PathBuf {
        match self.theme {
            Some(ref d) => root.join(d),
            None => root.join("theme"),
        }
    }

    /// Returns the name of the file used for HTTP 404 "not found" with the `.html` extension.
    pub fn get_404_output_file(&self) -> String {
        self.input_404
            .as_ref()
            .unwrap_or(&"404.md".to_string())
            .replace(".md", ".html")
    }
}

/// Configuration for how to render the print icon, print.html, and print.css.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct Print {
    /// Whether print support is enabled.
    pub enable: bool,
    /// Insert page breaks between chapters. Default: `true`.
    pub page_break: bool,
}

impl Default for Print {
    fn default() -> Self {
        Self {
            enable: true,
            page_break: true,
        }
    }
}

/// Configuration for how to fold chapters of sidebar.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct Fold {
    /// When off, all folds are open. Default: `false`.
    pub enable: bool,
    /// The higher the more folded regions are open. When level is 0, all folds
    /// are closed.
    /// Default: `0`.
    pub level: u8,
}

/// Configuration for tweaking how the HTML renderer handles the playground.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct Playground {
    /// Should playground snippets be editable? Default: `false`.
    pub editable: bool,
    /// Display the copy button. Default: `true`.
    pub copyable: bool,
    /// Copy JavaScript files for the editor to the output directory?
    /// Default: `true`.
    pub copy_js: bool,
    /// Display line numbers on playground snippets. Default: `false`.
    pub line_numbers: bool,
    /// Display the run button. Default: `true`
    pub runnable: bool,
}

impl Default for Playground {
    fn default() -> Playground {
        Playground {
            editable: false,
            copyable: true,
            copy_js: true,
            line_numbers: false,
            runnable: true,
        }
    }
}

/// Configuration for tweaking how the HTML renderer handles code blocks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct Code {
    /// Enable or disable the default line hiding with '#' for rust. Default: `true`.
    pub default_hidelines: bool,
    /// A prefix string to hide lines per language (one or more chars).
    pub hidelines: HashMap<String, String>,
}

impl Default for Code {
    fn default() -> Code {
        Code {
            default_hidelines: true,
            hidelines: HashMap::<String, String>::default(),
        }
    }
}

/// Configuration of the search functionality of the HTML renderer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct Search {
    /// Enable the search feature. Default: `true`.
    pub enable: bool,
    /// Maximum number of visible results. Default: `30`.
    pub limit_results: u32,
    /// The number of words used for a search result teaser. Default: `30`.
    pub teaser_word_count: u32,
    /// Define the logical link between multiple search words.
    /// If true, all search words must appear in each result. Default: `false`.
    pub use_boolean_and: bool,
    /// Boost factor for the search result score if a search word appears in the header.
    /// Default: `2`.
    pub boost_title: u8,
    /// Boost factor for the search result score if a search word appears in the hierarchy.
    /// The hierarchy contains all titles of the parent documents and all parent headings.
    /// Default: `1`.
    pub boost_hierarchy: u8,
    /// Boost factor for the search result score if a search word appears in the text.
    /// Default: `1`.
    pub boost_paragraph: u8,
    /// True if the searchword `micro` should match `microwave`. Default: `true`.
    pub expand: bool,
    /// Documents are split into smaller parts, separated by headings. This defines, until which
    /// level of heading documents should be split. Default: `3`. (`### This is a level 3 heading`)
    pub heading_split_level: u8,
    /// Copy JavaScript files for the search functionality to the output directory?
    /// Default: `true`.
    pub copy_js: bool,
    /// Specifies search settings for the given path.
    ///
    /// The path can be for a specific chapter, or a directory. This will
    /// merge recursively, with more specific paths taking precedence.
    pub chapter: HashMap<String, SearchChapterSettings>,
}

impl Default for Search {
    fn default() -> Search {
        // Please update the documentation of `Search` when changing values!
        Search {
            enable: true,
            limit_results: 30,
            teaser_word_count: 30,
            use_boolean_and: false,
            boost_title: 2,
            boost_hierarchy: 1,
            boost_paragraph: 1,
            expand: true,
            heading_split_level: 3,
            copy_js: true,
            chapter: HashMap::new(),
        }
    }
}

/// Search options for chapters (or paths).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct SearchChapterSettings {
    /// Whether or not indexing is enabled, default `true`.
    pub enable: Option<bool>,
}

/// Allows you to "update" any arbitrary field in a struct by round-tripping via
/// a `toml::Value`.
///
/// This is definitely not the most performant way to do things, which means you
/// should probably keep it away from tight loops...
trait Updateable<'de>: Serialize + Deserialize<'de> {
    fn update_value<S: Serialize>(&mut self, key: &str, value: S) {
        let mut raw = Value::try_from(&self).expect("unreachable");

        if let Ok(value) = Value::try_from(value) {
            raw.insert(key, value);
        } else {
            return;
        }

        if let Ok(updated) = raw.try_into() {
            *self = updated;
        }
    }
}

impl<'de, T> Updateable<'de> for T where T: Serialize + Deserialize<'de> {}

#[cfg(test)]
mod tests {
    use super::*;

    const COMPLEX_CONFIG: &str = r#"
        [book]
        title = "Some Book"
        authors = ["Michael-F-Bryan <michaelfbryan@gmail.com>"]
        description = "A completely useless book"
        src = "source"
        language = "ja"

        [build]
        build-dir = "outputs"
        create-missing = false
        use-default-preprocessors = true

        [output.html]
        theme = "./themedir"
        default-theme = "rust"
        smart-punctuation = true
        additional-css = ["./foo/bar/baz.css"]
        git-repository-url = "https://foo.com/"
        git-repository-icon = "fa-code-fork"

        [output.html.playground]
        editable = true

        [output.html.redirect]
        "index.html" = "overview.html"
        "nexted/page.md" = "https://rust-lang.org/"

        [preprocessor.first]

        [preprocessor.second]
        "#;

    #[test]
    fn load_a_complex_config_file() {
        let src = COMPLEX_CONFIG;

        let book_should_be = BookConfig {
            title: Some(String::from("Some Book")),
            authors: vec![String::from("Michael-F-Bryan <michaelfbryan@gmail.com>")],
            description: Some(String::from("A completely useless book")),
            src: PathBuf::from("source"),
            language: Some(String::from("ja")),
            text_direction: None,
        };
        let build_should_be = BuildConfig {
            build_dir: PathBuf::from("outputs"),
            create_missing: false,
            use_default_preprocessors: true,
            extra_watch_dirs: Vec::new(),
        };
        let rust_should_be = RustConfig { edition: None };
        let playground_should_be = Playground {
            editable: true,
            copyable: true,
            copy_js: true,
            line_numbers: false,
            runnable: true,
        };
        let html_should_be = HtmlConfig {
            smart_punctuation: true,
            additional_css: vec![PathBuf::from("./foo/bar/baz.css")],
            theme: Some(PathBuf::from("./themedir")),
            default_theme: Some(String::from("rust")),
            playground: playground_should_be,
            git_repository_url: Some(String::from("https://foo.com/")),
            git_repository_icon: Some(String::from("fa-code-fork")),
            redirect: vec![
                (String::from("index.html"), String::from("overview.html")),
                (
                    String::from("nexted/page.md"),
                    String::from("https://rust-lang.org/"),
                ),
            ]
            .into_iter()
            .collect(),
            ..Default::default()
        };

        let got = Config::from_str(src).unwrap();

        assert_eq!(got.book, book_should_be);
        assert_eq!(got.build, build_should_be);
        assert_eq!(got.rust, rust_should_be);
        assert_eq!(got.html_config().unwrap(), html_should_be);
    }

    #[test]
    fn disable_runnable() {
        let src = r#"
        [book]
        title = "Some Book"
        description = "book book book"
        authors = ["Shogo Takata"]

        [output.html.playground]
        runnable = false
        "#;

        let got = Config::from_str(src).unwrap();
        assert!(!got.html_config().unwrap().playground.runnable);
    }

    #[test]
    fn edition_2015() {
        let src = r#"
        [book]
        title = "mdBook Documentation"
        description = "Create book from markdown files. Like Gitbook but implemented in Rust"
        authors = ["Mathieu David"]
        src = "./source"
        [rust]
        edition = "2015"
        "#;

        let book_should_be = BookConfig {
            title: Some(String::from("mdBook Documentation")),
            description: Some(String::from(
                "Create book from markdown files. Like Gitbook but implemented in Rust",
            )),
            authors: vec![String::from("Mathieu David")],
            src: PathBuf::from("./source"),
            ..Default::default()
        };

        let got = Config::from_str(src).unwrap();
        assert_eq!(got.book, book_should_be);

        let rust_should_be = RustConfig {
            edition: Some(RustEdition::E2015),
        };
        let got = Config::from_str(src).unwrap();
        assert_eq!(got.rust, rust_should_be);
    }

    #[test]
    fn edition_2018() {
        let src = r#"
        [book]
        title = "mdBook Documentation"
        description = "Create book from markdown files. Like Gitbook but implemented in Rust"
        authors = ["Mathieu David"]
        src = "./source"
        [rust]
        edition = "2018"
        "#;

        let rust_should_be = RustConfig {
            edition: Some(RustEdition::E2018),
        };

        let got = Config::from_str(src).unwrap();
        assert_eq!(got.rust, rust_should_be);
    }

    #[test]
    fn edition_2021() {
        let src = r#"
        [book]
        title = "mdBook Documentation"
        description = "Create book from markdown files. Like Gitbook but implemented in Rust"
        authors = ["Mathieu David"]
        src = "./source"
        [rust]
        edition = "2021"
        "#;

        let rust_should_be = RustConfig {
            edition: Some(RustEdition::E2021),
        };

        let got = Config::from_str(src).unwrap();
        assert_eq!(got.rust, rust_should_be);
    }

    #[test]
    fn load_arbitrary_output_type() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct RandomOutput {
            foo: u32,
            bar: String,
            baz: Vec<bool>,
        }

        let src = r#"
        [output.random]
        foo = 5
        bar = "Hello World"
        baz = [true, true, false]
        "#;

        let should_be = RandomOutput {
            foo: 5,
            bar: String::from("Hello World"),
            baz: vec![true, true, false],
        };

        let cfg = Config::from_str(src).unwrap();
        let got: RandomOutput = cfg.get("output.random").unwrap().unwrap();

        assert_eq!(got, should_be);

        let got_baz: Vec<bool> = cfg.get("output.random.baz").unwrap().unwrap();
        let baz_should_be = vec![true, true, false];

        assert_eq!(got_baz, baz_should_be);
    }

    #[test]
    fn set_special_tables() {
        let mut cfg = Config::default();
        assert_eq!(cfg.book.title, None);
        cfg.set("book.title", "my title").unwrap();
        assert_eq!(cfg.book.title, Some("my title".to_string()));

        assert_eq!(&cfg.build.build_dir, Path::new("book"));
        cfg.set("build.build-dir", "some-directory").unwrap();
        assert_eq!(&cfg.build.build_dir, Path::new("some-directory"));

        assert_eq!(cfg.rust.edition, None);
        cfg.set("rust.edition", "2024").unwrap();
        assert_eq!(cfg.rust.edition, Some(RustEdition::E2024));

        cfg.set("output.foo.value", "123").unwrap();
        let got: String = cfg.get("output.foo.value").unwrap().unwrap();
        assert_eq!(got, "123");

        cfg.set("preprocessor.bar.value", "456").unwrap();
        let got: String = cfg.get("preprocessor.bar.value").unwrap().unwrap();
        assert_eq!(got, "456");
    }

    #[test]
    fn set_invalid_keys() {
        let mut cfg = Config::default();
        let err = cfg.set("foo", "test").unwrap_err();
        assert!(err.to_string().contains("invalid key `foo`"));
    }

    #[test]
    fn parse_env_vars() {
        let inputs = vec![
            ("FOO", None),
            ("MDBOOK_foo", Some("foo")),
            ("MDBOOK_FOO__bar__baz", Some("foo.bar.baz")),
            ("MDBOOK_FOO_bar__baz", Some("foo-bar.baz")),
        ];

        for (src, should_be) in inputs {
            let got = parse_env(src);
            let should_be = should_be.map(ToString::to_string);

            assert_eq!(got, should_be);
        }
    }

    #[test]
    fn file_404_default() {
        let src = r#"
        [output.html]
        "#;

        let got = Config::from_str(src).unwrap();
        let html_config = got.html_config().unwrap();
        assert_eq!(html_config.input_404, None);
        assert_eq!(html_config.get_404_output_file(), "404.html");
    }

    #[test]
    fn file_404_custom() {
        let src = r#"
        [output.html]
        input-404= "missing.md"
        "#;

        let got = Config::from_str(src).unwrap();
        let html_config = got.html_config().unwrap();
        assert_eq!(html_config.input_404, Some("missing.md".to_string()));
        assert_eq!(html_config.get_404_output_file(), "missing.html");
    }

    #[test]
    fn text_direction_ltr() {
        let src = r#"
        [book]
        text-direction = "ltr"
        "#;

        let got = Config::from_str(src).unwrap();
        assert_eq!(got.book.text_direction, Some(TextDirection::LeftToRight));
    }

    #[test]
    fn text_direction_rtl() {
        let src = r#"
        [book]
        text-direction = "rtl"
        "#;

        let got = Config::from_str(src).unwrap();
        assert_eq!(got.book.text_direction, Some(TextDirection::RightToLeft));
    }

    #[test]
    fn text_direction_none() {
        let src = r#"
        [book]
        "#;

        let got = Config::from_str(src).unwrap();
        assert_eq!(got.book.text_direction, None);
    }

    #[test]
    fn test_text_direction() {
        let mut cfg = BookConfig::default();

        // test deriving the text direction from language codes
        cfg.language = Some("ar".into());
        assert_eq!(cfg.realized_text_direction(), TextDirection::RightToLeft);

        cfg.language = Some("he".into());
        assert_eq!(cfg.realized_text_direction(), TextDirection::RightToLeft);

        cfg.language = Some("en".into());
        assert_eq!(cfg.realized_text_direction(), TextDirection::LeftToRight);

        cfg.language = Some("ja".into());
        assert_eq!(cfg.realized_text_direction(), TextDirection::LeftToRight);

        // test forced direction
        cfg.language = Some("ar".into());
        cfg.text_direction = Some(TextDirection::LeftToRight);
        assert_eq!(cfg.realized_text_direction(), TextDirection::LeftToRight);

        cfg.language = Some("ar".into());
        cfg.text_direction = Some(TextDirection::RightToLeft);
        assert_eq!(cfg.realized_text_direction(), TextDirection::RightToLeft);

        cfg.language = Some("en".into());
        cfg.text_direction = Some(TextDirection::LeftToRight);
        assert_eq!(cfg.realized_text_direction(), TextDirection::LeftToRight);

        cfg.language = Some("en".into());
        cfg.text_direction = Some(TextDirection::RightToLeft);
        assert_eq!(cfg.realized_text_direction(), TextDirection::RightToLeft);
    }

    #[test]
    #[should_panic(expected = "Invalid configuration file")]
    fn invalid_language_type_error() {
        let src = r#"
        [book]
        title = "mdBook Documentation"
        language = ["en", "pt-br"]
        description = "Create book from markdown files. Like Gitbook but implemented in Rust"
        authors = ["Mathieu David"]
        src = "./source"
        "#;

        Config::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "Invalid configuration file")]
    fn invalid_title_type() {
        let src = r#"
        [book]
        title = 20
        language = "en"
        description = "Create book from markdown files. Like Gitbook but implemented in Rust"
        authors = ["Mathieu David"]
        src = "./source"
        "#;

        Config::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "Invalid configuration file")]
    fn invalid_build_dir_type() {
        let src = r#"
        [build]
        build-dir = 99
        create-missing = false
        "#;

        Config::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(expected = "Invalid configuration file")]
    fn invalid_rust_edition() {
        let src = r#"
        [rust]
        edition = "1999"
        "#;

        Config::from_str(src).unwrap();
    }

    #[test]
    #[should_panic(
        expected = "unknown variant `1999`, expected one of `2024`, `2021`, `2018`, `2015`\n"
    )]
    fn invalid_rust_edition_expected() {
        let src = r#"
        [rust]
        edition = "1999"
        "#;

        Config::from_str(src).unwrap();
    }

    #[test]
    fn print_config() {
        let src = r#"
        [output.html.print]
        enable = false
        "#;
        let got = Config::from_str(src).unwrap();
        let html_config = got.html_config().unwrap();
        assert!(!html_config.print.enable);
        assert!(html_config.print.page_break);
        let src = r#"
        [output.html.print]
        page-break = false
        "#;
        let got = Config::from_str(src).unwrap();
        let html_config = got.html_config().unwrap();
        assert!(html_config.print.enable);
        assert!(!html_config.print.page_break);
    }

    #[test]
    fn test_json_direction() {
        use serde_json::json;
        assert_eq!(json!(TextDirection::RightToLeft), json!("rtl"));
        assert_eq!(json!(TextDirection::LeftToRight), json!("ltr"));
    }
}
