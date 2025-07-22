//! Theme dependencies for the playground editor.

pub static JS: &[u8] = include_bytes!("../../front-end/playground_editor/editor.js");
pub static ACE_JS: &[u8] = include_bytes!("../../front-end/playground_editor/ace.js");
pub static MODE_RUST_JS: &[u8] = include_bytes!("../../front-end/playground_editor/mode-rust.js");
pub static THEME_DAWN_JS: &[u8] = include_bytes!("../../front-end/playground_editor/theme-dawn.js");
pub static THEME_TOMORROW_NIGHT_JS: &[u8] =
    include_bytes!("../../front-end/playground_editor/theme-tomorrow_night.js");
