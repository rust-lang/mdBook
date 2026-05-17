//! Merges blockquote events split by pulldown-cmark when an admonition header is
//! followed by a blank `>` line.
//!
//! See <https://github.com/pulldown-cmark/pulldown-cmark/issues/890>.

use pulldown_cmark::{Event, Tag, TagEnd};

/// An iterator adapter to collapse together the incorrectly split [`Event::Start`]
/// / [`Event::End`] blockquote pairs produced by pulldown-cmark when using GFM
/// admonitions with an extra blank line after the header.
pub(crate) struct AdmonitionBlockquoteMerge<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    inner: std::iter::Peekable<I>,
    buffer: Vec<Option<Event<'a>>>,
}

impl<'a, I> AdmonitionBlockquoteMerge<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    pub(crate) fn new(iterator: I) -> Self {
        Self {
            inner: iterator.peekable(),
            buffer: Vec::new(),
        }
    }
}

impl<'a, I> Iterator for AdmonitionBlockquoteMerge<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match (self.inner.peek(), self.buffer.as_slice()) {
                // If we see 3 after accumulating 1&2, drop 2&3 and return 1.
                (
                    Some(Event::Start(Tag::BlockQuote(None))),
                    [
                        Some(Event::Start(Tag::BlockQuote(Some(_)))),
                        Some(Event::End(TagEnd::BlockQuote(_))),
                    ],
                ) => {
                    let _ = self.inner.next();
                    let e = self.buffer.swap_remove(0);
                    self.buffer.clear();
                    return e;
                }
                // If we see 2 and we've accumulated 1, buffer it and go around again.
                (
                    Some(Event::End(TagEnd::BlockQuote(_))),
                    [Some(Event::Start(Tag::BlockQuote(Some(_))))],
                ) => {
                    self.buffer.push(self.inner.next());
                }
                // If we see 1 and the buffer is empty, buffer it and go around again.
                (Some(Event::Start(Tag::BlockQuote(Some(_)))), []) => {
                    self.buffer.push(self.inner.next());
                }
                // Otherwise, if the buffer is empty, just pass it through.
                (_, []) => return self.inner.next(),
                // Otherwise, drain the buffer.
                (_, [_, ..]) => return self.buffer.remove(0),
            }
        }
    }
}
