use std::path::{Path, Component};

pub fn path_to_root(path: &Path) -> String {
    // Remove filename and add "../" for every directory

    path.to_path_buf().parent().expect("")
        .components().fold(String::new(), |mut s, c| {
            match c {
                Component::Normal(_) => s.push_str("../"),
                _ => {}
            }
            s
        })
}
