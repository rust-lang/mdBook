//! Tests for HTML rendering.

use crate::prelude::*;

// Checks that edit-url-template works.
#[test]
fn edit_url_template() {
    BookTest::from_dir("rendering/edit_url_template").check_file_contains(
        "book/index.html",
        "<a href=\"https://github.com/rust-lang/mdBook/edit/master/guide/src/README.md\" \
         title=\"Suggest an edit\" aria-label=\"Suggest an edit\">",
    );
}

// Checks that an alternate `src` setting works with the edit url template.
#[test]
fn edit_url_template_explicit_src() {
    BookTest::from_dir("rendering/edit_url_template_explicit_src").check_file_contains(
        "book/index.html",
        "<a href=\"https://github.com/rust-lang/mdBook/edit/master/guide/src2/README.md\" \
         title=\"Suggest an edit\" aria-label=\"Suggest an edit\">",
    );
}
