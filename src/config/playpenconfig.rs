use std::path::{Path, PathBuf};

use super::tomlconfig::TomlPlaypenConfig;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlaypenConfig {
    editor: PathBuf,
    editable: bool,
}

impl PlaypenConfig {
    /// Creates a new `PlaypenConfig` for playpen configuration.
    ///
    /// ```
    /// # use std::path::PathBuf;
    /// # use mdbook::config::PlaypenConfig;
    /// #
    /// let editor = PathBuf::from("root/editor");
    /// let config = PlaypenConfig::new(PathBuf::from("root"));
    ///
    /// assert_eq!(config.get_editor(), &editor);
    /// assert_eq!(config.is_editable(), false);
    /// ```
    pub fn new<T: Into<PathBuf>>(root: T) -> Self {
        PlaypenConfig {
            editor: root.into().join("editor"),
            editable: false,
        }
    }

    pub fn fill_from_tomlconfig<T: Into<PathBuf>>(&mut self, root: T, tomlplaypenconfig: TomlPlaypenConfig) -> &mut Self {
        let root = root.into();

        if let Some(editor) = tomlplaypenconfig.editor {
            if editor.is_relative() {
                self.editor = root.join(editor);
            } else {
                self.editor = editor;
            }
        }

        if let Some(editable) = tomlplaypenconfig.editable {
            self.editable = editable;
        }

        self
    }

    pub fn is_editable(&self) -> bool {
        self.editable
    }

    pub fn get_editor(&self) -> &Path {
        &self.editor
    }

    pub fn set_editor<T: Into<PathBuf>>(&mut self, root: T, editor: T) -> &mut Self {
        let editor = editor.into();

        if editor.is_relative() {
            self.editor = root.into().join(editor);
        } else {
            self.editor = editor;
        }

        self
    }
}
