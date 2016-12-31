#[cfg(test)]

use std::path::PathBuf;

use parse::summary::parse_level;

#[test]
fn it_parses_summary_to_tocitems() {
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

    let result = parse_level(&mut text.split('\n').collect(), 0, vec![0], true).unwrap();

    let expected = r#"[
    Unnumbered(
        TocContent {
            chapter: Chapter {
                title: "Introduction",
                path: "misc/introduction.md",
                dest_path: Some(
                    "index.html"
                ),
                authors: None,
                translators: None,
                description: None,
                css_class: None
            },
            sub_items: None,
            section: None
        }
    ),
    Numbered(
        TocContent {
            chapter: Chapter {
                title: "mdBook",
                path: "README.md",
                dest_path: None,
                authors: None,
                translators: None,
                description: None,
                css_class: None
            },
            sub_items: None,
            section: Some(
                [
                    1
                ]
            )
        }
    ),
    Numbered(
        TocContent {
            chapter: Chapter {
                title: "Command Line Tool",
                path: "cli/cli-tool.md",
                dest_path: None,
                authors: None,
                translators: None,
                description: None,
                css_class: None
            },
            sub_items: Some(
                [
                    Numbered(
                        TocContent {
                            chapter: Chapter {
                                title: "init",
                                path: "cli/init.md",
                                dest_path: None,
                                authors: None,
                                translators: None,
                                description: None,
                                css_class: None
                            },
                            sub_items: None,
                            section: Some(
                                [
                                    2,
                                    1
                                ]
                            )
                        }
                    ),
                    Numbered(
                        TocContent {
                            chapter: Chapter {
                                title: "build",
                                path: "cli/build.md",
                                dest_path: None,
                                authors: None,
                                translators: None,
                                description: None,
                                css_class: None
                            },
                            sub_items: None,
                            section: Some(
                                [
                                    2,
                                    2
                                ]
                            )
                        }
                    ),
                    Numbered(
                        TocContent {
                            chapter: Chapter {
                                title: "watch",
                                path: "cli/watch.md",
                                dest_path: None,
                                authors: None,
                                translators: None,
                                description: None,
                                css_class: None
                            },
                            sub_items: None,
                            section: Some(
                                [
                                    2,
                                    3
                                ]
                            )
                        }
                    ),
                    Numbered(
                        TocContent {
                            chapter: Chapter {
                                title: "serve",
                                path: "cli/serve.md",
                                dest_path: None,
                                authors: None,
                                translators: None,
                                description: None,
                                css_class: None
                            },
                            sub_items: None,
                            section: Some(
                                [
                                    2,
                                    4
                                ]
                            )
                        }
                    ),
                    Numbered(
                        TocContent {
                            chapter: Chapter {
                                title: "test",
                                path: "cli/test.md",
                                dest_path: None,
                                authors: None,
                                translators: None,
                                description: None,
                                css_class: None
                            },
                            sub_items: None,
                            section: Some(
                                [
                                    2,
                                    5
                                ]
                            )
                        }
                    )
                ]
            ),
            section: Some(
                [
                    2
                ]
            )
        }
    ),
    Numbered(
        TocContent {
            chapter: Chapter {
                title: "Format",
                path: "format/format.md",
                dest_path: None,
                authors: None,
                translators: None,
                description: None,
                css_class: None
            },
            sub_items: Some(
                [
                    Numbered(
                        TocContent {
                            chapter: Chapter {
                                title: "SUMMARY.md",
                                path: "format/summary.md",
                                dest_path: None,
                                authors: None,
                                translators: None,
                                description: None,
                                css_class: None
                            },
                            sub_items: None,
                            section: Some(
                                [
                                    3,
                                    1
                                ]
                            )
                        }
                    ),
                    Numbered(
                        TocContent {
                            chapter: Chapter {
                                title: "Configuration",
                                path: "format/config.md",
                                dest_path: None,
                                authors: None,
                                translators: None,
                                description: None,
                                css_class: None
                            },
                            sub_items: None,
                            section: Some(
                                [
                                    3,
                                    2
                                ]
                            )
                        }
                    ),
                    Numbered(
                        TocContent {
                            chapter: Chapter {
                                title: "Theme",
                                path: "format/theme/theme.md",
                                dest_path: None,
                                authors: None,
                                translators: None,
                                description: None,
                                css_class: None
                            },
                            sub_items: Some(
                                [
                                    Numbered(
                                        TocContent {
                                            chapter: Chapter {
                                                title: "index.hbs",
                                                path: "format/theme/index-hbs.md",
                                                dest_path: None,
                                                authors: None,
                                                translators: None,
                                                description: None,
                                                css_class: None
                                            },
                                            sub_items: None,
                                            section: Some(
                                                [
                                                    3,
                                                    3,
                                                    1
                                                ]
                                            )
                                        }
                                    ),
                                    Numbered(
                                        TocContent {
                                            chapter: Chapter {
                                                title: "Syntax highlighting",
                                                path: "format/theme/syntax-highlighting.md",
                                                dest_path: None,
                                                authors: None,
                                                translators: None,
                                                description: None,
                                                css_class: None
                                            },
                                            sub_items: None,
                                            section: Some(
                                                [
                                                    3,
                                                    3,
                                                    2
                                                ]
                                            )
                                        }
                                    )
                                ]
                            ),
                            section: Some(
                                [
                                    3,
                                    3
                                ]
                            )
                        }
                    ),
                    Numbered(
                        TocContent {
                            chapter: Chapter {
                                title: "MathJax Support",
                                path: "format/mathjax.md",
                                dest_path: None,
                                authors: None,
                                translators: None,
                                description: None,
                                css_class: None
                            },
                            sub_items: None,
                            section: Some(
                                [
                                    3,
                                    4
                                ]
                            )
                        }
                    ),
                    Numbered(
                        TocContent {
                            chapter: Chapter {
                                title: "Rust code specific features",
                                path: "format/rust.md",
                                dest_path: None,
                                authors: None,
                                translators: None,
                                description: None,
                                css_class: None
                            },
                            sub_items: None,
                            section: Some(
                                [
                                    3,
                                    5
                                ]
                            )
                        }
                    )
                ]
            ),
            section: Some(
                [
                    3
                ]
            )
        }
    ),
    Numbered(
        TocContent {
            chapter: Chapter {
                title: "Rust Library",
                path: "lib/lib.md",
                dest_path: None,
                authors: None,
                translators: None,
                description: None,
                css_class: None
            },
            sub_items: None,
            section: Some(
                [
                    4
                ]
            )
        }
    ),
    Spacer,
    Unnumbered(
        TocContent {
            chapter: Chapter {
                title: "Contributors",
                path: "misc/contributors.md",
                dest_path: None,
                authors: None,
                translators: None,
                description: None,
                css_class: None
            },
            sub_items: None,
            section: None
        }
    )
]"#;

    assert_eq!(expected, format!("{:#?}", result));
}
