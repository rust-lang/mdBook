#![cfg(test)]

use std::path::Path;
use serde_json;
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

    let mut expected = BookConfig::new(Path::new("."));
    expected.title = "mdBook Documentation".to_string();
    expected.author = "Mathieu David".to_string();
    expected.description = "Create book from markdown files. Like Gitbook but implemented in Rust".to_string();

    assert_eq!(format!("{:#?}", config), format!("{:#?}", expected));
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

    let mut expected = BookConfig::new(Path::new("."));
    expected.title = "mdBook Documentation".to_string();
    expected.author = "Mathieu David".to_string();
    expected.description = "Create book from markdown files. Like Gitbook but implemented in Rust".to_string();

    assert_eq!(format!("{:#?}", config), format!("{:#?}", expected));
}

#[test]
fn it_parses_json_nested_array_to_toml() {

    // Example from:
    // toml-0.2.1/tests/valid/arrays-nested.json

    let text = r#"
{
    "nest": {
        "type": "array",
        "value": [
            {"type": "array", "value": [
                {"type": "string", "value": "a"}
            ]},
            {"type": "array", "value": [
                {"type": "string", "value": "b"}
            ]}
        ]
    }
}"#;

    let c: serde_json::Value = serde_json::from_str(&text).unwrap();

    let result = json_object_to_btreemap(&c.as_object().unwrap());

    let expected = r#"{
    "nest": Table(
        {
            "type": String(
                "array"
            ),
            "value": Array(
                [
                    Table(
                        {
                            "type": String(
                                "array"
                            ),
                            "value": Array(
                                [
                                    Table(
                                        {
                                            "type": String(
                                                "string"
                                            ),
                                            "value": String(
                                                "a"
                                            )
                                        }
                                    )
                                ]
                            )
                        }
                    ),
                    Table(
                        {
                            "type": String(
                                "array"
                            ),
                            "value": Array(
                                [
                                    Table(
                                        {
                                            "type": String(
                                                "string"
                                            ),
                                            "value": String(
                                                "b"
                                            )
                                        }
                                    )
                                ]
                            )
                        }
                    )
                ]
            )
        }
    )
}"#;

    assert_eq!(format!("{:#?}", result), expected);
}


#[test]
fn it_parses_json_arrays_to_toml() {

    // Example from:
    // toml-0.2.1/tests/valid/arrays.json

    let text = r#"
{
    "ints": {
        "type": "array",
        "value": [
            {"type": "integer", "value": "1"},
            {"type": "integer", "value": "2"},
            {"type": "integer", "value": "3"}
        ]
    },
    "floats": {
        "type": "array",
        "value": [
            {"type": "float", "value": "1.1"},
            {"type": "float", "value": "2.1"},
            {"type": "float", "value": "3.1"}
        ]
    },
    "strings": {
        "type": "array",
        "value": [
            {"type": "string", "value": "a"},
            {"type": "string", "value": "b"},
            {"type": "string", "value": "c"}
        ]
    },
    "dates": {
        "type": "array",
        "value": [
            {"type": "datetime", "value": "1987-07-05T17:45:00Z"},
            {"type": "datetime", "value": "1979-05-27T07:32:00Z"},
            {"type": "datetime", "value": "2006-06-01T11:00:00Z"}
        ]
    }
}"#;

    let c: serde_json::Value = serde_json::from_str(&text).unwrap();

    let result = json_object_to_btreemap(&c.as_object().unwrap());

    let expected = r#"{
    "dates": Table(
        {
            "type": String(
                "array"
            ),
            "value": Array(
                [
                    Table(
                        {
                            "type": String(
                                "datetime"
                            ),
                            "value": String(
                                "1987-07-05T17:45:00Z"
                            )
                        }
                    ),
                    Table(
                        {
                            "type": String(
                                "datetime"
                            ),
                            "value": String(
                                "1979-05-27T07:32:00Z"
                            )
                        }
                    ),
                    Table(
                        {
                            "type": String(
                                "datetime"
                            ),
                            "value": String(
                                "2006-06-01T11:00:00Z"
                            )
                        }
                    )
                ]
            )
        }
    ),
    "floats": Table(
        {
            "type": String(
                "array"
            ),
            "value": Array(
                [
                    Table(
                        {
                            "type": String(
                                "float"
                            ),
                            "value": String(
                                "1.1"
                            )
                        }
                    ),
                    Table(
                        {
                            "type": String(
                                "float"
                            ),
                            "value": String(
                                "2.1"
                            )
                        }
                    ),
                    Table(
                        {
                            "type": String(
                                "float"
                            ),
                            "value": String(
                                "3.1"
                            )
                        }
                    )
                ]
            )
        }
    ),
    "ints": Table(
        {
            "type": String(
                "array"
            ),
            "value": Array(
                [
                    Table(
                        {
                            "type": String(
                                "integer"
                            ),
                            "value": String(
                                "1"
                            )
                        }
                    ),
                    Table(
                        {
                            "type": String(
                                "integer"
                            ),
                            "value": String(
                                "2"
                            )
                        }
                    ),
                    Table(
                        {
                            "type": String(
                                "integer"
                            ),
                            "value": String(
                                "3"
                            )
                        }
                    )
                ]
            )
        }
    ),
    "strings": Table(
        {
            "type": String(
                "array"
            ),
            "value": Array(
                [
                    Table(
                        {
                            "type": String(
                                "string"
                            ),
                            "value": String(
                                "a"
                            )
                        }
                    ),
                    Table(
                        {
                            "type": String(
                                "string"
                            ),
                            "value": String(
                                "b"
                            )
                        }
                    ),
                    Table(
                        {
                            "type": String(
                                "string"
                            ),
                            "value": String(
                                "c"
                            )
                        }
                    )
                ]
            )
        }
    )
}"#;

    assert_eq!(format!("{:#?}", result), expected);
}
