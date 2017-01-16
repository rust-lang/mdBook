#[cfg(test)]

use book::chapter::Chapter;
use book::toc::{TocItem, TocContent, flat_toc, toc_node_count_id};
use parse::summary::parse_level;

#[test]
fn it_should_produce_the_section_as_string() {
    let mut c = TocContent::default();
    c.section = Some(vec![1, 9, 4]);
    let result = c.section_as_string();
    let expected = "1.9.4.".to_string();
    assert_eq!(result, expected);
}

#[test]
fn it_flattens_toc() {
    let text = r#"
# Summary

[Introduction](misc/introduction.md)

- [mdBook](README.md)
- [Command Line Tool](cli/cli-tool.md)
    - [init](cli/init.md)
    - [build](cli/build.md)
    - [watch](cli/watch.md)
    - [serve](cli/serve.md)
    - [test](cli/test.md)
- [Format](format/format.md)
    - [SUMMARY.md](format/summary.md)
    - [Configuration](format/config.md)
    - [Theme](format/theme/theme.md)
        - [index.hbs](format/theme/index-hbs.md)
        - [Syntax highlighting](format/theme/syntax-highlighting.md)
    - [MathJax Support](format/mathjax.md)
    - [Rust code specific features](format/rust.md)
- [Rust Library](lib/lib.md)
-----------
[Contributors](misc/contributors.md)
"#;

    let toc = parse_level(&mut text.split('\n').collect(), 0, vec![0], true).unwrap();

    let flat = flat_toc(&toc);

    let result: Vec<String> = flat.iter().map(|x| {
        match *x {
            TocItem::Numbered(ref i) |
            TocItem::Unnumbered(ref i) |
            TocItem::Unlisted(ref i) => i.chapter.title.to_owned(),
            TocItem::Spacer => "spacer".to_string(),
        }
    }).collect::<Vec<String>>();

    let expected = r#"[
    "Introduction",
    "mdBook",
    "Command Line Tool",
    "init",
    "build",
    "watch",
    "serve",
    "test",
    "Format",
    "SUMMARY.md",
    "Configuration",
    "Theme",
    "index.hbs",
    "Syntax highlighting",
    "MathJax Support",
    "Rust code specific features",
    "Rust Library",
    "spacer",
    "Contributors"
]"#;

    assert_eq!(format!("{:#?}", result), expected);
}

#[test]
fn it_counts_toc_id_string() {
    let text = r#"
# Summary

[Introduction](misc/introduction.md)

- [mdBook](README.md)
- [Command Line Tool](cli/cli-tool.md)
    - [init](cli/init.md)
    - [build](cli/build.md)
    - [watch](cli/watch.md)
    - [serve](cli/serve.md)
    - [test](cli/test.md)
- [Format](format/format.md)
    - [SUMMARY.md](format/summary.md)
    - [Configuration](format/config.md)
    - [Theme](format/theme/theme.md)
        - [index.hbs](format/theme/index-hbs.md)
        - [Syntax highlighting](format/theme/syntax-highlighting.md)
    - [MathJax Support](format/mathjax.md)
    - [Rust code specific features](format/rust.md)
- [Rust Library](lib/lib.md)
-----------
[Contributors](misc/contributors.md)
"#;

    let toc = parse_level(&mut text.split('\n').collect(), 0, vec![0], true).unwrap();

    let counters = toc_node_count_id(&toc);

    assert_eq!(counters, "6552".to_string());
}
