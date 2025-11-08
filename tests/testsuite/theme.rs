//! Tests for theme handling.

use crate::prelude::*;

// Checks what happens if the theme directory is missing.
#[test]
fn missing_theme() {
    BookTest::from_dir("theme/missing_theme").run("build", |cmd| {
        cmd.expect_failure().expect_stderr(str![[r#"
 INFO Book building has started
 INFO Running the html backend
ERROR Rendering failed
[TAB]Caused by: theme dir [ROOT]/./non-existent-directory does not exist

"#]]);
    });
}

// Checks what happens if the theme directory is empty.
#[test]
fn empty_theme() {
    BookTest::from_dir("theme/empty_theme").run("build", |cmd| {
        std::fs::create_dir(cmd.dir.join("theme")).unwrap();
        cmd.expect_stderr(str![[r#"
 INFO Book building has started
 INFO Running the html backend
 INFO HTML book written to `[ROOT]/book`

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
        .check_file_contains("book/index.html", "fonts/fonts-[..].css")
        .check_file_list(
            "book/fonts",
            str![[r#"
book/fonts/OPEN-SANS-LICENSE.txt
book/fonts/SOURCE-CODE-PRO-LICENSE.txt
book/fonts/fonts-[..].css
book/fonts/open-sans-v17-all-charsets-300-[..].woff2
book/fonts/open-sans-v17-all-charsets-300italic-[..].woff2
book/fonts/open-sans-v17-all-charsets-600-[..].woff2
book/fonts/open-sans-v17-all-charsets-600italic-[..].woff2
book/fonts/open-sans-v17-all-charsets-700-[..].woff2
book/fonts/open-sans-v17-all-charsets-700italic-[..].woff2
book/fonts/open-sans-v17-all-charsets-800-[..].woff2
book/fonts/open-sans-v17-all-charsets-800italic-[..].woff2
book/fonts/open-sans-v17-all-charsets-italic-[..].woff2
book/fonts/open-sans-v17-all-charsets-regular-[..].woff2
book/fonts/source-code-pro-v11-all-charsets-500-[..].woff2
"#]],
        );
}

// When the theme is initialized, what does the fonts list look like?
#[test]
fn theme_fonts_copied() {
    BookTest::init(|bb| {
        bb.copy_theme(true);
    })
    .check_file_contains("book/index.html", "fonts/fonts-[..].css")
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
    // Note that license files get hashed, which is not like the behavior when
    // the theme directory is empty. It kinda makes sense, but is weird.
    .check_file_list(
        "book/fonts",
        str![[r#"
book/fonts/OPEN-SANS-LICENSE-[..].txt
book/fonts/SOURCE-CODE-PRO-LICENSE-[..].txt
book/fonts/fonts-[..].css
book/fonts/open-sans-v17-all-charsets-300-[..].woff2
book/fonts/open-sans-v17-all-charsets-300italic-[..].woff2
book/fonts/open-sans-v17-all-charsets-600-[..].woff2
book/fonts/open-sans-v17-all-charsets-600italic-[..].woff2
book/fonts/open-sans-v17-all-charsets-700-[..].woff2
book/fonts/open-sans-v17-all-charsets-700italic-[..].woff2
book/fonts/open-sans-v17-all-charsets-800-[..].woff2
book/fonts/open-sans-v17-all-charsets-800italic-[..].woff2
book/fonts/open-sans-v17-all-charsets-italic-[..].woff2
book/fonts/open-sans-v17-all-charsets-regular-[..].woff2
book/fonts/source-code-pro-v11-all-charsets-500-[..].woff2
"#]],
    );
}

// Custom fonts.css.
#[test]
fn fonts_css() {
    BookTest::from_dir("theme/fonts_css")
        .check_file_contains("book/index.html", "fonts/fonts-[..].css")
        .check_file(
            "book/fonts/fonts-*.css",
            str![[r#"
/*custom*/

"#]],
        )
        .check_file("book/fonts/myfont-*.woff", str![[""]])
        .check_file_list(
            "book/fonts",
            str![[r#"
book/fonts/fonts-[..].css
book/fonts/myfont-[..].woff
"#]],
        );
}

// Empty fonts.css should not copy the default fonts.
#[test]
fn empty_fonts_css() {
    BookTest::from_dir("theme/empty_fonts_css")
        .run("build", |cmd| {
            cmd.expect_stderr(str![[r#"
 INFO Book building has started
 INFO Running the html backend
 INFO HTML book written to `[ROOT]/book`

"#]]);
        })
        .check_file_contains("book/index.html", "fonts.css")
        .check_file_list("book/fonts", str![[""]]);
}

// Custom fonts.css file shouldn't copy default fonts.
#[test]
fn custom_fonts_css() {
    BookTest::from_dir("theme/custom_fonts_css")
        .run("build", |cmd| {
            cmd.expect_stderr(str![[r#"
 INFO Book building has started
 INFO Running the html backend
 INFO HTML book written to `[ROOT]/book`

"#]]);
        })
        .check_file_contains("book/index.html", "fonts-[..].css")
        .check_file_list(
            "book/fonts",
            str![[r#"
book/fonts/fonts-[..].css
book/fonts/myfont-[..].woff
"#]],
        );
}
