extern crate regex;
extern crate toml;

use regex::Regex;

use std::str::FromStr;
use std::error::Error;
use std::collections::BTreeMap;

use serde_json;

pub mod fs;

use pulldown_cmark::{Parser, html, Options, OPTION_ENABLE_TABLES, OPTION_ENABLE_FOOTNOTES};

/// Wrapper around the pulldown-cmark parser and renderer to render markdown
pub fn render_markdown(text: &str) -> String {
    let mut s = String::with_capacity(text.len() * 3 / 2);

    let mut opts = Options::empty();
    opts.insert(OPTION_ENABLE_TABLES);
    opts.insert(OPTION_ENABLE_FOOTNOTES);

    let p = Parser::new_ext(&text, opts);
    html::push_html(&mut s, p);
    s
}

pub fn last_name_first(name: &str) -> String {
    let mut s = name.split_whitespace().collect::<Vec<&str>>();
    let last = s.pop().unwrap();
    format!("{}, {}", last, s.join(" "))
}

pub fn toml_str_to_btreemap(text: &str) -> Result<BTreeMap<String, toml::Value>, String> {
    let mut parser = toml::Parser::new(text);
    match parser.parse() {
        Some(x) => Ok(x),
        None => Err(format!("{:#?}", parser.errors)),
    }
}

/// Parses the string to JSON and converts it to BTreeMap<String, toml::Value>.
pub fn json_str_to_btreemap(text: &str) -> Result<BTreeMap<String, toml::Value>, String> {
    let c: serde_json::Value = match serde_json::from_str(text) {
        Ok(x) => x,
        Err(e) => return Err(format!("{:#?}", e)),
    };

    Ok(json_object_to_btreemap(&c.as_object().unwrap()))
}

pub fn json_object_to_btreemap(json: &serde_json::Map<String, serde_json::Value>) -> BTreeMap<String, toml::Value> {
    let mut config: BTreeMap<String, toml::Value> = BTreeMap::new();

    for (key, value) in json.iter() {
        config.insert(
            String::from_str(key).unwrap(),
            json_value_to_toml_value(value.to_owned())
        );
    }

    config
}

pub fn json_value_to_toml_value(json: serde_json::Value) -> toml::Value {
    match json {
        serde_json::Value::Null => toml::Value::String("".to_string()),
        serde_json::Value::Bool(x) => toml::Value::Boolean(x),
        serde_json::Value::I64(x) => toml::Value::Integer(x),
        serde_json::Value::U64(x) => toml::Value::Integer(x as i64),
        serde_json::Value::F64(x) => toml::Value::Float(x),
        serde_json::Value::String(x) => toml::Value::String(x),
        serde_json::Value::Array(x) => {
            toml::Value::Array(x.iter().map(|v| json_value_to_toml_value(v.to_owned())).collect())
        },
        serde_json::Value::Object(x) => {
            toml::Value::Table(json_object_to_btreemap(&x))
        },
    }
}

pub fn strip_toml_header(text: &str) -> String {
    let re: Regex = Regex::new(r"(?ms)^\+\+\+\n.*\n\+\+\+\n").unwrap();
    let mut out = text.to_owned();
    match re.captures(text) {
        Some(caps) => {
            if let Some(s) = caps.at(0) {
                out = text.replace(s, "");
            }
        },
        None => {}
    }
    out
}
