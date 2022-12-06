use log::warn;
use std::path::{Path, PathBuf};

pub fn remove_ignored_files(book_root: &Path, paths: &[PathBuf]) -> Vec<PathBuf> {
    if paths.is_empty() {
        return vec![];
    }

    match find_ignorefile(book_root) {
        Some(gitignore_path) => {
            match gitignore::File::new(gitignore_path.as_path()) {
                Ok(exclusion_checker) => filter_ignored_files(exclusion_checker, paths),
                Err(_) => {
                    // We're unable to read the .gitignore file, so we'll silently allow everything.
                    // Please see discussion: https://github.com/rust-lang/mdBook/pull/1051
                    paths.iter().map(|path| path.to_path_buf()).collect()
                }
            }
        }
        None => {
            // There is no .gitignore file.
            paths.iter().map(|path| path.to_path_buf()).collect()
        }
    }
}

fn find_ignorefile(book_root: &Path) -> Option<PathBuf> {
    book_root
        .ancestors()
        .flat_map(|p| vec![p.join(".bookignore"), p.join(".gitignore")].into_iter())
        .find(|p| p.exists())
}

fn filter_ignored_files(exclusion_checker: gitignore::File<'_>, paths: &[PathBuf]) -> Vec<PathBuf> {
    paths
        .iter()
        .filter(|path| match exclusion_checker.is_excluded(path) {
            Ok(exclude) => !exclude,
            Err(error) => {
                warn!(
                    "Unable to determine if {:?} is excluded: {:?}. Including it.",
                    &path, error
                );
                true
            }
        })
        .map(|path| path.to_path_buf())
        .collect()
}
