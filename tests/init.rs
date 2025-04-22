use mdbook::MDBook;
use pretty_assertions::assert_eq;
use tempfile::Builder as TempFileBuilder;

#[test]
fn copy_theme() {
    let temp = TempFileBuilder::new().prefix("mdbook").tempdir().unwrap();
    MDBook::init(temp.path()).copy_theme(true).build().unwrap();
    let expected = vec![
        "book.js",
        "css/chrome.css",
        "css/general.css",
        "css/print.css",
        "css/variables.css",
        "favicon.png",
        "favicon.svg",
        "fonts/OPEN-SANS-LICENSE.txt",
        "fonts/SOURCE-CODE-PRO-LICENSE.txt",
        "fonts/fonts.css",
        "fonts/open-sans-v17-all-charsets-300.woff2",
        "fonts/open-sans-v17-all-charsets-300italic.woff2",
        "fonts/open-sans-v17-all-charsets-600.woff2",
        "fonts/open-sans-v17-all-charsets-600italic.woff2",
        "fonts/open-sans-v17-all-charsets-700.woff2",
        "fonts/open-sans-v17-all-charsets-700italic.woff2",
        "fonts/open-sans-v17-all-charsets-800.woff2",
        "fonts/open-sans-v17-all-charsets-800italic.woff2",
        "fonts/open-sans-v17-all-charsets-italic.woff2",
        "fonts/open-sans-v17-all-charsets-regular.woff2",
        "fonts/source-code-pro-v11-all-charsets-500.woff2",
        "highlight.css",
        "highlight.js",
        "index.hbs",
    ];
    let theme_dir = temp.path().join("theme");
    let mut actual: Vec<_> = walkdir::WalkDir::new(&theme_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !e.file_type().is_dir())
        .map(|e| {
            e.path()
                .strip_prefix(&theme_dir)
                .unwrap()
                .to_str()
                .unwrap()
                .replace('\\', "/")
        })
        .collect();
    actual.sort();
    assert_eq!(actual, expected);
}
