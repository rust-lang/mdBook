//! Tests for theme handling.

use crate::prelude::*;

// Checks what happens if the theme directory is missing.
#[test]
fn missing_theme() {
    BookTest::from_dir("theme/missing_theme")
    .run("build", |cmd| {
cmd.expect_failure()
        .expect_stderr(str![[r#"
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Book building has started
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Running the html backend
[TIMESTAMP] [ERROR] (mdbook_core::utils): Error: Rendering failed
[TIMESTAMP] [ERROR] (mdbook_core::utils): [TAB]Caused By: theme dir [ROOT]/./non-existent-directory does not exist

"#]]);
    });
}

// Checks what happens if the theme directory is empty.
#[test]
fn empty_theme() {
    BookTest::from_dir("theme/empty_theme").run("build", |cmd| {
        std::fs::create_dir(cmd.dir.join("theme")).unwrap();
        cmd.expect_stderr(str![[r#"
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Book building has started
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Running the html backend
[TIMESTAMP] [INFO] (mdbook_html::html_handlebars::hbs_renderer): HTML book written to `[ROOT]/book`

"#]]);
    });
}

// Checks overriding index.hbs.
#[test]
fn override_index() {
    BookTest::from_dir("theme/override_index").check_file(
        "book/index.html",
        str![[r#"
This is a modified index.hbs!

"#]],
    );
}

// After building, what are the default set of fonts?
#[test]
fn default_fonts() {
    BookTest::init(|_| {})
        .check_file_contains("book/index.html", "fonts/fonts.css")
        .check_file_list(
            "book/fonts",
            str![[r#"
book/fonts/OPEN-SANS-LICENSE.txt
book/fonts/SOURCE-CODE-PRO-LICENSE.txt
book/fonts/fonts.css
book/fonts/open-sans-v17-all-charsets-300.woff2
book/fonts/open-sans-v17-all-charsets-300italic.woff2
book/fonts/open-sans-v17-all-charsets-600.woff2
book/fonts/open-sans-v17-all-charsets-600italic.woff2
book/fonts/open-sans-v17-all-charsets-700.woff2
book/fonts/open-sans-v17-all-charsets-700italic.woff2
book/fonts/open-sans-v17-all-charsets-800.woff2
book/fonts/open-sans-v17-all-charsets-800italic.woff2
book/fonts/open-sans-v17-all-charsets-italic.woff2
book/fonts/open-sans-v17-all-charsets-regular.woff2
book/fonts/source-code-pro-v11-all-charsets-500.woff2
"#]],
        );
}

// When the theme is initialized, what does the fonts list look like?
#[test]
fn theme_fonts_copied() {
    BookTest::init(|bb| {
        bb.copy_theme(true);
    })
    .check_file_contains("book/index.html", "fonts/fonts.css")
    .check_file_list(
        "theme/fonts",
        str![[r#"
theme/fonts/OPEN-SANS-LICENSE.txt
theme/fonts/SOURCE-CODE-PRO-LICENSE.txt
theme/fonts/fonts.css
theme/fonts/open-sans-v17-all-charsets-300.woff2
theme/fonts/open-sans-v17-all-charsets-300italic.woff2
theme/fonts/open-sans-v17-all-charsets-600.woff2
theme/fonts/open-sans-v17-all-charsets-600italic.woff2
theme/fonts/open-sans-v17-all-charsets-700.woff2
theme/fonts/open-sans-v17-all-charsets-700italic.woff2
theme/fonts/open-sans-v17-all-charsets-800.woff2
theme/fonts/open-sans-v17-all-charsets-800italic.woff2
theme/fonts/open-sans-v17-all-charsets-italic.woff2
theme/fonts/open-sans-v17-all-charsets-regular.woff2
theme/fonts/source-code-pro-v11-all-charsets-500.woff2
"#]],
    )
    .check_file_list(
        "book/fonts",
        str![[r#"
book/fonts/OPEN-SANS-LICENSE.txt
book/fonts/SOURCE-CODE-PRO-LICENSE.txt
book/fonts/fonts.css
book/fonts/open-sans-v17-all-charsets-300.woff2
book/fonts/open-sans-v17-all-charsets-300italic.woff2
book/fonts/open-sans-v17-all-charsets-600.woff2
book/fonts/open-sans-v17-all-charsets-600italic.woff2
book/fonts/open-sans-v17-all-charsets-700.woff2
book/fonts/open-sans-v17-all-charsets-700italic.woff2
book/fonts/open-sans-v17-all-charsets-800.woff2
book/fonts/open-sans-v17-all-charsets-800italic.woff2
book/fonts/open-sans-v17-all-charsets-italic.woff2
book/fonts/open-sans-v17-all-charsets-regular.woff2
book/fonts/source-code-pro-v11-all-charsets-500.woff2
"#]],
    );
}

// Custom fonts.css.
#[test]
fn fonts_css() {
    BookTest::from_dir("theme/fonts_css")
        .check_file_contains("book/index.html", "fonts/fonts.css")
        .check_file(
            "book/fonts/fonts.css",
            str![[r#"
/*custom*/

"#]],
        )
        .check_file("book/fonts/myfont.woff", str![[""]])
        .check_file_list(
            "book/fonts",
            str![[r#"
book/fonts/fonts.css
book/fonts/myfont.woff
"#]],
        );
}

// copy-fonts=false, no theme, deprecated
#[test]
fn copy_fonts_false_no_theme() {
    BookTest::from_dir("theme/copy_fonts_false_no_theme")
        .run("build", |cmd| {
            cmd.expect_stderr(str![[r#"
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Book building has started
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Running the html backend
[TIMESTAMP] [WARN] (mdbook_html::html_handlebars::static_files): output.html.copy-fonts is deprecated.
This book appears to have copy-fonts=false in book.toml without a fonts.css file.
Add an empty `theme/fonts/fonts.css` file to squelch this warning.
[TIMESTAMP] [INFO] (mdbook_html::html_handlebars::hbs_renderer): HTML book written to `[ROOT]/book`

"#]]);
        })
        .check_file_doesnt_contain("book/index.html", "fonts.css")
        .check_file_list("book/fonts", str![[""]]);
}

// copy-fonts=false, empty fonts.css
#[test]
fn copy_fonts_false_with_empty_fonts_css() {
    BookTest::from_dir("theme/copy_fonts_false_with_empty_fonts_css")
        .run("build", |cmd| {
            cmd.expect_stderr(str![[r#"
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Book building has started
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Running the html backend
[TIMESTAMP] [INFO] (mdbook_html::html_handlebars::hbs_renderer): HTML book written to `[ROOT]/book`

"#]]);
        })
        .check_file_doesnt_contain("book/index.html", "fonts.css")
        .check_file_list("book/fonts", str![[""]]);
}

// copy-fonts=false, fonts.css has contents
#[test]
fn copy_fonts_false_with_fonts_css() {
    BookTest::from_dir("theme/copy_fonts_false_with_fonts_css")
        .run("build", |cmd| {
            cmd.expect_stderr(str![[r#"
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Book building has started
[TIMESTAMP] [INFO] (mdbook_driver::mdbook): Running the html backend
[TIMESTAMP] [INFO] (mdbook_html::html_handlebars::hbs_renderer): HTML book written to `[ROOT]/book`

"#]]);
        })
        .check_file_contains("book/index.html", "fonts.css")
        .check_file_list(
            "book/fonts",
            str![[r#"
book/fonts/fonts.css
book/fonts/myfont.woff
"#]],
        );
}
