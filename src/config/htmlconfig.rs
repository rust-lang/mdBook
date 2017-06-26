use std::path::{PathBuf, Path};

use super::tomlconfig::TomlHtmlConfig;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HtmlConfig {
    destination: PathBuf,
    theme: PathBuf,
    curly_quotes: bool,
    mathjax_support: bool,
    google_analytics: Option<String>,
    additional_css: Vec<PathBuf>,
    additional_js: Vec<PathBuf>,
}

impl HtmlConfig {
    /// Creates a new `HtmlConfig` struct containing the configuration parameters for the HTML renderer.
    ///
    /// ```
    /// # use std::path::PathBuf;
    /// # use mdbook::config::HtmlConfig;
    /// #
    /// let output = PathBuf::from("root/book");
    /// let config = HtmlConfig::new(PathBuf::from("root"));
    ///
    /// assert_eq!(config.get_destination(), &output);
    /// ```
    pub fn new<T: Into<PathBuf>>(root: T) -> Self {
        let root = root.into();
        HtmlConfig {
            destination: root.clone().join("book"),
            theme: root.join("theme"),
            curly_quotes: false,
            mathjax_support: false,
            google_analytics: None,
            additional_css: Vec::new(),
            additional_js: Vec::new(),
        }
    }

    pub fn fill_from_tomlconfig<T: Into<PathBuf>>(&mut self, root: T, tomlconfig: TomlHtmlConfig) -> &mut Self {
        let root = root.into();

        if let Some(d) = tomlconfig.destination {
            self.set_destination(&root, &d);
        }

        if let Some(t) = tomlconfig.theme {
            self.set_theme(&root, &t);
        }

        if let Some(curly_quotes) = tomlconfig.curly_quotes {
            self.curly_quotes = curly_quotes;
        }

        if let Some(mathjax_support) = tomlconfig.mathjax_support {
            self.mathjax_support = mathjax_support;
        }

        if tomlconfig.google_analytics.is_some() {
            self.google_analytics = tomlconfig.google_analytics;
        }

        if let Some(stylepaths) = tomlconfig.additional_css {
            for path in stylepaths {
                if path.is_relative() {
                    self.additional_css.push(root.join(path));
                } else {
                    self.additional_css.push(path);
                }
            }
        }

        if let Some(scriptpaths) = tomlconfig.additional_js {
            for path in scriptpaths {
                if path.is_relative() {
                    self.additional_js.push(root.join(path));
                } else {
                    self.additional_js.push(path);
                }
            }
        }

        self
    }

    pub fn set_destination<T: Into<PathBuf>>(&mut self, root: T, destination: T) -> &mut Self {
        let d = destination.into();
        if d.is_relative() {
            self.destination = root.into().join(d);
        } else {
            self.destination = d;
        }

        self
    }

    pub fn get_destination(&self) -> &Path {
        &self.destination
    }

    pub fn get_theme(&self) -> &Path {
        &self.theme
    }

    pub fn set_theme<T: Into<PathBuf>>(&mut self, root: T, theme: T) -> &mut Self {
        let d = theme.into();
        if d.is_relative() {
            self.theme = root.into().join(d);
        } else {
            self.theme = d;
        }

        self
    }

    pub fn get_curly_quotes(&self) -> bool {
        self.curly_quotes
    }

    pub fn set_curly_quotes(&mut self, curly_quotes: bool) {
        self.curly_quotes = curly_quotes;
    }

    pub fn get_mathjax_support(&self) -> bool {
        self.mathjax_support
    }

    pub fn set_mathjax_support(&mut self, mathjax_support: bool) {
        self.mathjax_support = mathjax_support;
    }

    pub fn get_google_analytics_id(&self) -> Option<String> {
        self.google_analytics.clone()
    }

    pub fn set_google_analytics_id(&mut self, id: Option<String>) -> &mut Self {
        self.google_analytics = id;
        self
    }

    pub fn has_additional_css(&self) -> bool {
        !self.additional_css.is_empty()
    }

    pub fn get_additional_css(&self) -> &[PathBuf] {
        &self.additional_css
    }

    pub fn has_additional_js(&self) -> bool {
        !self.additional_js.is_empty()
    }

    pub fn get_additional_js(&self) -> &[PathBuf] {
        &self.additional_js
    }
}
