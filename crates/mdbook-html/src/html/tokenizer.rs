//! Support for parsing HTML.
//!
//! The primary entry point is [`parse_html`] which uses [`html5ever`] to
//! tokenize the input.

use html5ever::TokenizerResult;
use html5ever::tendril::ByteTendril;
use html5ever::tokenizer::states::RawKind;
use html5ever::tokenizer::{
    BufferQueue, TagKind, Token, TokenSink, TokenSinkResult, Tokenizer, TokenizerOpts,
};
use std::cell::RefCell;

/// Collector for HTML tokens.
#[derive(Default)]
struct TokenCollector {
    /// Parsed HTML tokens.
    tokens: RefCell<Vec<Token>>,
}

impl TokenSink for TokenCollector {
    type Handle = ();

    fn process_token(&self, token: Token, _line_number: u64) -> TokenSinkResult<()> {
        match &token {
            Token::DoctypeToken(_) => {}
            Token::TagToken(tag) => {
                let tag_name = tag.name.as_bytes();
                // TODO: This could probably use special support for SVG and MathML.
                if tag_name == b"script" {
                    match tag.kind {
                        TagKind::StartTag => {
                            self.tokens.borrow_mut().push(token);
                            return TokenSinkResult::RawData(RawKind::ScriptData);
                        }
                        TagKind::EndTag => {}
                    }
                }
                if tag_name == b"style" {
                    match tag.kind {
                        TagKind::StartTag => {
                            self.tokens.borrow_mut().push(token);
                            return TokenSinkResult::RawData(RawKind::Rawtext);
                        }
                        TagKind::EndTag => {}
                    }
                }
                self.tokens.borrow_mut().push(token);
            }
            Token::CommentToken(_) => {
                self.tokens.borrow_mut().push(token);
            }
            Token::CharacterTokens(_) => {
                self.tokens.borrow_mut().push(token);
            }
            Token::NullCharacterToken => {}
            Token::EOFToken => {}
            Token::ParseError(_) => {
                self.tokens.borrow_mut().push(token);
            }
        }
        TokenSinkResult::Continue
    }
}

/// Parse HTML into tokens.
pub(crate) fn parse_html(html: &str) -> Vec<Token> {
    let tendril: ByteTendril = html.as_bytes().into();
    let mut queue = BufferQueue::default();
    queue.push_back(tendril.try_reinterpret().unwrap());

    let collector = TokenCollector::default();
    let tok = Tokenizer::new(collector, TokenizerOpts::default());
    let result = tok.feed(&mut queue);
    assert_eq!(result, TokenizerResult::Done);
    assert!(
        queue.is_empty(),
        "queue wasn't empty: {:?}",
        queue.pop_front()
    );
    tok.end();
    tok.sink.tokens.take()
}
