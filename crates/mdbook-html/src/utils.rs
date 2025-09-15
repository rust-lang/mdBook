//! Utilities for processing HTML.

use std::path::Path;

/// Helper trait for converting a [`Path`] to a string suitable for an HTML path.
pub(crate) trait ToUrlPath {
    fn to_url_path(&self) -> String;
}

impl ToUrlPath for Path {
    fn to_url_path(&self) -> String {
        // We're generally assuming that all paths we deal with are utf-8.
        // The replace here is to handle Windows paths.
        self.to_str().unwrap().replace('\\', "/")
    }
}
