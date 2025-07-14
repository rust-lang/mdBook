//! Tests for HTML rendering.

use crate::prelude::*;

// Checks that edit-url-template works.
#[test]
fn edit_url_template() {
    BookTest::from_dir("rendering/edit_url_template").check_file_contains(
        "book/index.html",
        "<a href=\"https://github.com/rust-lang/mdBook/edit/master/guide/src/README.md\" \
         title=\"Suggest an edit\" aria-label=\"Suggest an edit\" rel=\"edit\">",
    );
}

// Checks that an alternate `src` setting works with the edit url template.
#[test]
fn edit_url_template_explicit_src() {
    BookTest::from_dir("rendering/edit_url_template_explicit_src").check_file_contains(
        "book/index.html",
        "<a href=\"https://github.com/rust-lang/mdBook/edit/master/guide/src2/README.md\" \
         title=\"Suggest an edit\" aria-label=\"Suggest an edit\" rel=\"edit\">",
    );
}

// Checks that index.html is generated correctly, even when the first few
// chapters are drafts.
#[test]
fn first_chapter_is_copied_as_index_even_if_not_first_elem() {
    BookTest::from_dir("rendering/first_chapter_is_copied_as_index_even_if_not_first_elem")
        // These two files should be equal.
        .check_main_file(
            "book/chapter_1.html",
            str![[
                r##"<h1 id="chapter-1"><a class="header" href="#chapter-1">Chapter 1</a></h1>"##
            ]],
        )
        .check_main_file(
            "book/index.html",
            str![[
                r##"<h1 id="chapter-1"><a class="header" href="#chapter-1">Chapter 1</a></h1>"##
            ]],
        );
}
