use crate::html::tokenizer::parse_html;
use html5ever::tokenizer::{Tag, TagKind, Token};

// Basic tokenizer behavior of a script.
#[test]
fn parse_html_script() {
    let script = r#"
if (3 < 5 > 10)
{
    alert("The sky is falling!");
}
"#;
    let t = format!("<script>{script}</script>");
    let ts = parse_html(&t);
    eprintln!("{ts:#?}",);
    let mut output = String::new();
    let mut in_script = false;
    for t in ts {
        match t {
            Token::ParseError(e) => panic!("{e:?}"),
            Token::CharacterTokens(s) => {
                if in_script {
                    output.push_str(&s)
                }
            }
            Token::TagToken(Tag {
                kind: TagKind::StartTag,
                ..
            }) => in_script = true,
            Token::TagToken(Tag {
                kind: TagKind::EndTag,
                ..
            }) => in_script = false,
            _ => {}
        }
    }
    assert_eq!(output, script);
}

#[test]
fn parse_html_svg_with_xml_decl() {
    let html = r#"<svg xmlns="http://www.w3.org/2000/svg"><?xml version="1.0"?><rect/></svg>"#;
    let ts = parse_html(html);
    for t in &ts {
        if let Token::ParseError(e) = t {
            panic!("unexpected parse error: {e:?}");
        }
    }
}

#[test]
fn parse_html_pre_with_svg_xml_decl() {
    let html = r#"<pre><?xml version="1.0" encoding="utf-8"?><svg xmlns="http://www.w3.org/2000/svg"><rect/></svg></pre>"#;
    let ts = parse_html(html);
    for t in &ts {
        if let Token::ParseError(e) = t {
            panic!("unexpected parse error: {e:?}");
        }
    }
}

// What happens if a script doesn't end.
#[test]
fn parse_html_script_unclosed() {
    let t = r#"<script>
// Test
"#;
    let ts = parse_html(t);
    eprintln!("{ts:#?}",);
    for t in ts {
        if let Token::ParseError(e) = t {
            panic!("{e:?}",);
        }
    }
}
