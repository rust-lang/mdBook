use std::path::Path;

use theme::load_file_contents;

pub static JS: &'static [u8] = include_bytes!("editor.js");
pub static ACE_JS: &'static [u8] = include_bytes!("ace.js");
pub static MODE_RUST_JS: &'static [u8] = include_bytes!("mode-rust.js");
pub static THEME_DAWN_JS: &'static [u8] = include_bytes!("theme-dawn.js");
pub static THEME_TOMORROW_NIGHT_JS: &'static [u8] = include_bytes!("theme-tomorrow_night.js");

/// Integration of a JavaScript editor for playpens.
/// Uses the Ace editor: https://ace.c9.io/.
/// The Ace editor itself, the mode, and the theme files are the
/// generated minified no conflict versions.
///
/// The `PlaypenEditor` struct should be used instead of the static variables because
/// the `new()` method
/// will look if the user has an editor directory in his source folder and use
/// the users editor instead
/// of the default.
///
/// You should exceptionnaly use the static variables only if you need the
/// default editor even if the
/// user has specified another editor.
pub struct PlaypenEditor {
    pub js: Vec<u8>,
    pub ace_js: Vec<u8>,
    pub mode_rust_js: Vec<u8>,
    pub theme_dawn_js: Vec<u8>,
    pub theme_tomorrow_night_js: Vec<u8>,
}

impl PlaypenEditor {
    pub fn new(src: &Path) -> Self {
        let mut editor = PlaypenEditor {
            js: JS.to_owned(),
            ace_js: ACE_JS.to_owned(),
            mode_rust_js: MODE_RUST_JS.to_owned(),
            theme_dawn_js: THEME_DAWN_JS.to_owned(),
            theme_tomorrow_night_js: THEME_TOMORROW_NIGHT_JS.to_owned(),
        };

        // Check if the given path exists
        if !src.exists() || !src.is_dir() {
            return editor;
        }

        // Check for individual files if they exist
        {
            let files = vec![
                (src.join("editor.js"), &mut editor.js),
                (src.join("ace.js"), &mut editor.ace_js),
                (src.join("mode-rust.js"), &mut editor.mode_rust_js),
                (src.join("theme-dawn.js"), &mut editor.theme_dawn_js),
                (src.join("theme-tomorrow_night.js"), &mut editor.theme_tomorrow_night_js),
            ];

            for (filename, dest) in files {
                if !filename.exists() {
                    continue;
                }

                if let Err(e) = load_file_contents(&filename, dest) {
                    warn!("Couldn't load custom file, {}: {}", filename.display(), e);
                }
            }
        }

        editor
    }
}
