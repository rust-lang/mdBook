#![cfg(test)]

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
                content: None,
                src_path: Some(
                    "misc/introduction.md"
                ),
                dest_path: Some(
                    "index.html"
                ),
                translation_links: None,
                translation_id: None,
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
                content: None,
                src_path: Some(
                    "README.md"
                ),
                dest_path: Some(
                    "README.html"
                ),
                translation_links: None,
                translation_id: None,
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
                content: None,
                src_path: Some(
                    "cli/cli-tool.md"
                ),
                dest_path: Some(
                    "cli/cli-tool.html"
                ),
                translation_links: None,
                translation_id: None,
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
                                content: None,
                                src_path: Some(
                                    "cli/init.md"
                                ),
                                dest_path: Some(
                                    "cli/init.html"
                                ),
                                translation_links: None,
                                translation_id: None,
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
                                content: None,
                                src_path: Some(
                                    "cli/build.md"
                                ),
                                dest_path: Some(
                                    "cli/build.html"
                                ),
                                translation_links: None,
                                translation_id: None,
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
                                content: None,
                                src_path: Some(
                                    "cli/watch.md"
                                ),
                                dest_path: Some(
                                    "cli/watch.html"
                                ),
                                translation_links: None,
                                translation_id: None,
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
                                content: None,
                                src_path: Some(
                                    "cli/serve.md"
                                ),
                                dest_path: Some(
                                    "cli/serve.html"
                                ),
                                translation_links: None,
                                translation_id: None,
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
                                content: None,
                                src_path: Some(
                                    "cli/test.md"
                                ),
                                dest_path: Some(
                                    "cli/test.html"
                                ),
                                translation_links: None,
                                translation_id: None,
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
                content: None,
                src_path: Some(
                    "format/format.md"
                ),
                dest_path: Some(
                    "format/format.html"
                ),
                translation_links: None,
                translation_id: None,
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
                                content: None,
                                src_path: Some(
                                    "format/summary.md"
                                ),
                                dest_path: Some(
                                    "format/summary.html"
                                ),
                                translation_links: None,
                                translation_id: None,
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
                                content: None,
                                src_path: Some(
                                    "format/config.md"
                                ),
                                dest_path: Some(
                                    "format/config.html"
                                ),
                                translation_links: None,
                                translation_id: None,
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
                                content: None,
                                src_path: Some(
                                    "format/theme/theme.md"
                                ),
                                dest_path: Some(
                                    "format/theme/theme.html"
                                ),
                                translation_links: None,
                                translation_id: None,
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
                                                content: None,
                                                src_path: Some(
                                                    "format/theme/index-hbs.md"
                                                ),
                                                dest_path: Some(
                                                    "format/theme/index-hbs.html"
                                                ),
                                                translation_links: None,
                                                translation_id: None,
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
                                                content: None,
                                                src_path: Some(
                                                    "format/theme/syntax-highlighting.md"
                                                ),
                                                dest_path: Some(
                                                    "format/theme/syntax-highlighting.html"
                                                ),
                                                translation_links: None,
                                                translation_id: None,
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
                                content: None,
                                src_path: Some(
                                    "format/mathjax.md"
                                ),
                                dest_path: Some(
                                    "format/mathjax.html"
                                ),
                                translation_links: None,
                                translation_id: None,
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
                                content: None,
                                src_path: Some(
                                    "format/rust.md"
                                ),
                                dest_path: Some(
                                    "format/rust.html"
                                ),
                                translation_links: None,
                                translation_id: None,
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
                content: None,
                src_path: Some(
                    "lib/lib.md"
                ),
                dest_path: Some(
                    "lib/lib.html"
                ),
                translation_links: None,
                translation_id: None,
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
                content: None,
                src_path: Some(
                    "misc/contributors.md"
                ),
                dest_path: Some(
                    "misc/contributors.html"
                ),
                translation_links: None,
                translation_id: None,
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
