use std::path::{Path, PathBuf};

pub mod toc;
pub mod navigation;

/// Rewrite filename of path to directory index if matches any of filename
/// pattern in `rewrite_names`.
///
/// * `path` - Path reference.
/// * `rewrite_names` - Array of filename pattern to be rewritten.
pub fn rewrite_to_dir_index<P: AsRef<Path>>(path: P, rewrite_names: &[String]) -> PathBuf {
    let p = path.as_ref();
    for name in rewrite_names.iter() {
        if name.as_str() == p.file_name().unwrap_or_default() {
            return p.with_file_name("");
        }
    }
    return p.to_owned();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrite_to_dir_success() {
        let names = vec!["index.html".to_owned(), "index.md".to_owned()];

        let path = PathBuf::from("index.html");
        assert_eq!(rewrite_to_dir_index(&path, &names), PathBuf::from(""));

        let path = PathBuf::from("index.md");
        assert_eq!(rewrite_to_dir_index(&path, &names), PathBuf::from(""));

        let path = PathBuf::from("index.asp");
        assert_eq!(rewrite_to_dir_index(&path, &names), path);
    }
}
