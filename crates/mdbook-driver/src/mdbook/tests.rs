use super::*;
use std::str::FromStr;
use toml::value::{Table, Value};

#[test]
fn config_defaults_to_html_renderer_if_empty() {
    let cfg = Config::default();

    // make sure we haven't got anything in the `output` table
    assert!(cfg.get::<Value>("output").unwrap().is_none());

    let got = determine_renderers(&cfg).unwrap();

    assert_eq!(got.len(), 1);
    assert_eq!(got[0].name(), "html");
}

#[test]
fn add_a_random_renderer_to_the_config() {
    let mut cfg = Config::default();
    cfg.set("output.random", Table::new()).unwrap();

    let got = determine_renderers(&cfg).unwrap();

    assert_eq!(got.len(), 1);
    assert_eq!(got[0].name(), "random");
}

#[test]
fn add_a_random_renderer_with_custom_command_to_the_config() {
    let mut cfg = Config::default();

    let mut table = Table::new();
    table.insert("command".to_string(), Value::String("false".to_string()));
    cfg.set("output.random", table).unwrap();

    let got = determine_renderers(&cfg).unwrap();

    assert_eq!(got.len(), 1);
    assert_eq!(got[0].name(), "random");
}

#[test]
fn config_defaults_to_link_and_index_preprocessor_if_not_set() {
    let cfg = Config::default();

    // make sure we haven't got anything in the `preprocessor` table
    assert!(cfg.get::<Value>("preprocessor").unwrap().is_none());

    let got = determine_preprocessors(&cfg);

    assert!(got.is_ok());
    assert_eq!(got.as_ref().unwrap().len(), 2);
    assert_eq!(got.as_ref().unwrap()[0].name(), "index");
    assert_eq!(got.as_ref().unwrap()[1].name(), "links");
}

#[test]
fn use_default_preprocessors_works() {
    let mut cfg = Config::default();
    cfg.build.use_default_preprocessors = false;

    let got = determine_preprocessors(&cfg).unwrap();

    assert_eq!(got.len(), 0);
}

#[test]
fn can_determine_third_party_preprocessors() {
    let cfg_str = r#"
        [book]
        title = "Some Book"

        [preprocessor.random]

        [build]
        build-dir = "outputs"
        create-missing = false
        "#;

    let cfg = Config::from_str(cfg_str).unwrap();

    // make sure the `preprocessor.random` table exists
    assert!(cfg.get::<Value>("preprocessor.random").unwrap().is_some());

    let got = determine_preprocessors(&cfg).unwrap();

    assert!(got.into_iter().any(|p| p.name() == "random"));
}

#[test]
fn preprocessors_can_provide_their_own_commands() {
    let cfg_str = r#"
        [preprocessor.random]
        command = "python random.py"
        "#;

    let cfg = Config::from_str(cfg_str).unwrap();

    // make sure the `preprocessor.random` table exists
    let random = cfg
        .get::<OutputConfig>("preprocessor.random")
        .unwrap()
        .unwrap();
    assert_eq!(random.command, Some("python random.py".to_string()));
}

#[test]
fn preprocessor_before_must_be_array() {
    let cfg_str = r#"
        [preprocessor.random]
        before = 0
        "#;

    let cfg = Config::from_str(cfg_str).unwrap();

    assert!(determine_preprocessors(&cfg).is_err());
}

#[test]
fn preprocessor_after_must_be_array() {
    let cfg_str = r#"
        [preprocessor.random]
        after = 0
        "#;

    let cfg = Config::from_str(cfg_str).unwrap();

    assert!(determine_preprocessors(&cfg).is_err());
}

#[test]
fn preprocessor_order_is_honored() {
    let cfg_str = r#"
        [preprocessor.random]
        before = [ "last" ]
        after = [ "index" ]

        [preprocessor.last]
        after = [ "links", "index" ]
        "#;

    let cfg = Config::from_str(cfg_str).unwrap();

    let preprocessors = determine_preprocessors(&cfg).unwrap();
    let index = |name| {
        preprocessors
            .iter()
            .enumerate()
            .find(|(_, preprocessor)| preprocessor.name() == name)
            .unwrap()
            .0
    };
    let assert_before = |before, after| {
        if index(before) >= index(after) {
            eprintln!("Preprocessor order:");
            for preprocessor in &preprocessors {
                eprintln!("  {}", preprocessor.name());
            }
            panic!("{before} should come before {after}");
        }
    };

    assert_before("index", "random");
    assert_before("index", "last");
    assert_before("random", "last");
    assert_before("links", "last");
}

#[test]
fn cyclic_dependencies_are_detected() {
    let cfg_str = r#"
        [preprocessor.links]
        before = [ "index" ]

        [preprocessor.index]
        before = [ "links" ]
        "#;

    let cfg = Config::from_str(cfg_str).unwrap();

    assert!(determine_preprocessors(&cfg).is_err());
}

#[test]
fn dependencies_dont_register_undefined_preprocessors() {
    let cfg_str = r#"
        [preprocessor.links]
        before = [ "random" ]
        "#;

    let cfg = Config::from_str(cfg_str).unwrap();

    let preprocessors = determine_preprocessors(&cfg).unwrap();

    assert!(
        !preprocessors
            .iter()
            .any(|preprocessor| preprocessor.name() == "random")
    );
}

#[test]
fn dependencies_dont_register_builtin_preprocessors_if_disabled() {
    let cfg_str = r#"
        [preprocessor.random]
        before = [ "links" ]

        [build]
        use-default-preprocessors = false
        "#;

    let cfg = Config::from_str(cfg_str).unwrap();

    let preprocessors = determine_preprocessors(&cfg).unwrap();

    assert!(
        !preprocessors
            .iter()
            .any(|preprocessor| preprocessor.name() == "links")
    );
}

#[test]
fn config_respects_preprocessor_selection() {
    let cfg_str = r#"
        [preprocessor.links]
        renderers = ["html"]
        "#;

    let cfg = Config::from_str(cfg_str).unwrap();

    let html_renderer = HtmlHandlebars;
    let pre = LinkPreprocessor::new();

    let should_run = preprocessor_should_run(&pre, &html_renderer, &cfg).unwrap();
    assert!(should_run);
}

struct BoolPreprocessor(bool);
impl Preprocessor for BoolPreprocessor {
    fn name(&self) -> &str {
        "bool-preprocessor"
    }

    fn run(&self, _ctx: &PreprocessorContext, _book: Book) -> Result<Book> {
        unimplemented!()
    }

    fn supports_renderer(&self, _renderer: &str) -> bool {
        self.0
    }
}

#[test]
fn preprocessor_should_run_falls_back_to_supports_renderer_method() {
    let cfg = Config::default();
    let html = HtmlHandlebars::new();

    let should_be = true;
    let got = preprocessor_should_run(&BoolPreprocessor(should_be), &html, &cfg).unwrap();
    assert_eq!(got, should_be);

    let should_be = false;
    let got = preprocessor_should_run(&BoolPreprocessor(should_be), &html, &cfg).unwrap();
    assert_eq!(got, should_be);
}
