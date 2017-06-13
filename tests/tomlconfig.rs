extern crate mdbook;
use mdbook::config::BookConfig;
use mdbook::config::tomlconfig::TomlConfig;

use std::path::PathBuf;

// Tests that the `title` key is correcly parsed in the TOML config
#[test]
fn from_toml_source() {
    let toml = r#"source = "source""#;

    let parsed = TomlConfig::from_toml(&toml).expect("This should parse");
    let config = BookConfig::from_tomlconfig("root", parsed);

    assert_eq!(config.get_source(), PathBuf::from("root/source"));
}

// Tests that the `title` key is correcly parsed in the TOML config
#[test]
fn from_toml_title() {
    let toml = r#"title = "Some title""#;

    let parsed = TomlConfig::from_toml(&toml).expect("This should parse");
    let config = BookConfig::from_tomlconfig("root", parsed);

    assert_eq!(config.get_title(), "Some title");
}

// Tests that the `description` key is correcly parsed in the TOML config
#[test]
fn from_toml_description() {
    let toml = r#"description = "This is a description""#;

    let parsed = TomlConfig::from_toml(&toml).expect("This should parse");
    let config = BookConfig::from_tomlconfig("root", parsed);

    assert_eq!(config.get_description(), "This is a description");
}

// Tests that the `author` key is correcly parsed in the TOML config
#[test]
fn from_toml_author() {
    let toml = r#"author = "John Doe""#;

    let parsed = TomlConfig::from_toml(&toml).expect("This should parse");
    let config = BookConfig::from_tomlconfig("root", parsed);

    assert_eq!(config.get_authors(), &[String::from("John Doe")]);
}

// Tests that the `authors` key is correcly parsed in the TOML config
#[test]
fn from_toml_authors() {
    let toml = r#"authors = ["John Doe", "Jane Doe"]"#;

    let parsed = TomlConfig::from_toml(&toml).expect("This should parse");
    let config = BookConfig::from_tomlconfig("root", parsed);

    assert_eq!(config.get_authors(), &[String::from("John Doe"), String::from("Jane Doe")]);
}

// Tests that the `output.html.destination` key is correcly parsed in the TOML config
#[test]
fn from_toml_output_html_destination() {
    let toml = r#"[output.html]
    destination = "htmlbook""#;

    let parsed = TomlConfig::from_toml(&toml).expect("This should parse");
    let config = BookConfig::from_tomlconfig("root", parsed);

    let htmlconfig = config.get_html_config().expect("There should be an HtmlConfig");

    assert_eq!(htmlconfig.get_destination(), PathBuf::from("root/htmlbook"));
}

// Tests that the `output.html.theme` key is correcly parsed in the TOML config
#[test]
fn from_toml_output_html_theme() {
    let toml = r#"[output.html]
    theme = "theme""#;

    let parsed = TomlConfig::from_toml(&toml).expect("This should parse");
    let config = BookConfig::from_tomlconfig("root", parsed);

    let htmlconfig = config.get_html_config().expect("There should be an HtmlConfig");

    assert_eq!(htmlconfig.get_theme().expect("the theme key was provided"), &PathBuf::from("root/theme"));
}

// Tests that the `output.html.google-analytics` key is correcly parsed in the TOML config
#[test]
fn from_toml_output_html_google_analytics() {
    let toml = r#"[output.html]
    google-analytics = "123456""#;

    let parsed = TomlConfig::from_toml(&toml).expect("This should parse");
    let config = BookConfig::from_tomlconfig("root", parsed);

    let htmlconfig = config.get_html_config().expect("There should be an HtmlConfig");

    assert_eq!(htmlconfig.get_google_analytics_id().expect("the google-analytics key was provided"), String::from("123456"));
}


// Tests that the `output.html.additional-css` key is correcly parsed in the TOML config
#[test]
fn from_toml_output_html_additional_stylesheet() {
    let toml = r#"[output.html]
    additional-css = ["custom.css", "two/custom.css"]"#;

    let parsed = TomlConfig::from_toml(&toml).expect("This should parse");
    let config = BookConfig::from_tomlconfig("root", parsed);

    let htmlconfig = config.get_html_config().expect("There should be an HtmlConfig");

    assert_eq!(htmlconfig.get_additional_css(), &[PathBuf::from("root/custom.css"), PathBuf::from("root/two/custom.css")]);
}

// Tests that the `output.html.additional-js` key is correcly parsed in the TOML config
#[test]
fn from_toml_output_html_additional_scripts() {
    let toml = r#"[output.html]
    additional-js = ["custom.js", "two/custom.js"]"#;

    let parsed = TomlConfig::from_toml(&toml).expect("This should parse");
    let config = BookConfig::from_tomlconfig("root", parsed);

    let htmlconfig = config.get_html_config().expect("There should be an HtmlConfig");

    assert_eq!(htmlconfig.get_additional_js(), &[PathBuf::from("root/custom.js"), PathBuf::from("root/two/custom.js")]);
}
