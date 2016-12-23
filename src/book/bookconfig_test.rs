#[cfg(test)]

use std::path::Path;
use book::bookconfig::*;

#[test]
fn it_parses_json_config() {
    let text = r#"
{
    "title": "mdBook Documentation",
    "description": "Create book from markdown files. Like Gitbook but implemented in Rust",
    "author": "Mathieu David"
}"#;

    // TODO don't require path argument, take pwd
    let mut config = BookConfig::new(Path::new("."));

    config.parse_from_json_string(&text.to_string());

    let expected = r#"BookConfig {
    root: ".",
    dest: "./book",
    src: "./src",
    theme_path: "./theme",
    title: "mdBook Documentation",
    author: "Mathieu David",
    description: "Create book from markdown files. Like Gitbook but implemented in Rust",
    indent_spaces: 4,
    multilingual: false
}"#;

    assert_eq!(format!("{:#?}", config), expected);
}

#[test]
fn it_parses_toml_config() {
    let text = r#"
title = "mdBook Documentation"
description = "Create book from markdown files. Like Gitbook but implemented in Rust"
author = "Mathieu David"
"#;

    // TODO don't require path argument, take pwd
    let mut config = BookConfig::new(Path::new("."));

    config.parse_from_toml_string(&text.to_string());

    println!("{:#?}", config);

    let expected = r#"BookConfig {
    root: ".",
    dest: "./book",
    src: "./src",
    theme_path: "./theme",
    title: "mdBook Documentation",
    author: "Mathieu David",
    description: "Create book from markdown files. Like Gitbook but implemented in Rust",
    indent_spaces: 4,
    multilingual: false
}"#;

    assert_eq!(format!("{:#?}", config), expected);
}
