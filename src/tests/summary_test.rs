#![cfg(test)]

use std::path::PathBuf;

use book::toc::TocItem;
use parse::summary::parse_level;

#[ignore]// FIXME failing on Windows https://ci.appveyor.com/project/azerupi/mdbook/build/1.0.145
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

    let result: Vec<TocItem> = parse_level(&mut text.split('\n').collect(), 0, vec![0], true).unwrap();

    assert_eq!(result.iter().count(), 7);

    {
        let a = match result[0].clone() {
            TocItem::Unnumbered(x) => x,
            _ => { panic!("{:#?}", result[0]); },
        };

        assert_eq!(a.chapter.title, "Introduction".to_string());
        assert_eq!(a.chapter.get_src_path().unwrap().as_os_str(),
                   PathBuf::from("misc").join("introduction.md").as_os_str());
    }

    {
        let a = match result[2].clone() {
            TocItem::Numbered(x) => x,
            _ => { panic!("{:#?}", result[0]); },
        };

        assert_eq!(a.chapter.title, "Command Line Tool".to_string());
        assert_eq!(a.chapter.get_src_path().unwrap().as_os_str(),
                   PathBuf::from("cli").join("cli-tool.md").as_os_str());

        let b = match a.sub_items {
            Some(x) => x,
            None => { panic!("No sub items! {:#?}", a); }
        };

        assert_eq!(b.iter().count(), 5);
    }
}
