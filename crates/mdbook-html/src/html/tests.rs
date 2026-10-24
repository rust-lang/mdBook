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
