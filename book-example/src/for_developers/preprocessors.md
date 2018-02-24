# Preprocessors

A *preprocessor* is simply a bit of code which gets run immediately after the
book is loaded and before it gets rendered, allowing you to update and mutate
the book. Possible use cases are:

- Creating custom helpers like `\{{#include /path/to/file.md}}`
- Updating links so `[some chapter](some_chapter.md)` is automatically changed
  to `[some chapter](some_chapter.html)` for the HTML renderer
- Substituting in latex-style expressions (`$$ \frac{1}{3} $$`) with their
  mathjax equivalents


## Implementing a Preprocessor

A preprocessor is represented by the `Preprocessor` trait.

```rust
pub trait Preprocessor {
    fn name(&self) -> &str;
    fn run(&self, ctx: &PreprocessorContext, book: &mut Book) -> Result<()>;
}
```

Where the `PreprocessorContext` is defined as

```rust
pub struct PreprocessorContext {
    pub root: PathBuf,
    pub config: Config,
}
```

## A complete Example

The magic happens within the `run(...)` method of the [`Preprocessor`][preprocessor-docs] trait implementation.

As direct access to the chapters is not possible, you will probably end up iterating
them using `for_each_mut(...)`:

```rust
book.for_each_mut(|item: &mut BookItem| {
    if let BookItem::Chapter(ref mut chapter) = *item {
      eprintln!("{}: processing chapter '{}'", self.name(), chapter.name);
      res = Some(
          match Deemphasize::remove_emphasis(&mut num_removed_items, chapter) {
              Ok(md) => {
                  chapter.content = md;
                  Ok(())
              }
              Err(err) => Err(err),
          },
      );
  }
});
```

The `chapter.content` is just a markdown formatted string, and you will have to
process it in some way. Even though it's entirely possible to implement some sort of
manual find & replace operation, if that feels too unsafe you can use [`pulldown-cmark`][pc]
to parse the string into events and work on them instead.

Finally you can use [`pulldown-cmark-to-cmark`][pctc] to transform these events back to
a string.

The following code block shows how to remove all emphasis from markdown, and do so
safely.

```rust
fn remove_emphasis(num_removed_items: &mut i32, chapter: &mut Chapter) -> Result<String> {
    let mut buf = String::with_capacity(chapter.content.len());
    let events = Parser::new(&chapter.content).filter(|e| {
        let should_keep = match *e {
            Event::Start(Tag::Emphasis)
            | Event::Start(Tag::Strong)
            | Event::End(Tag::Emphasis)
            | Event::End(Tag::Strong) => false,
            _ => true,
        };
        if !should_keep {
            *num_removed_items += 1;
        }
        should_keep
    });
    cmark(events, &mut buf, None)
        .map(|_| buf)
        .map_err(|err| Error::from(format!("Markdown serialization failed: {}", err)))
}
```

For everything else, have a look [at the complete example][example].

[preprocessor-docs]: https://docs.rs/mdbook/0.1.3/mdbook/preprocess/trait.Preprocessor.html
[pc]: https://crates.io/crates/pulldown-cmark
[pctc]: https://crates.io/crates/pulldown-cmark-to-cmark
[example]: https://github.com/rust-lang-nursery/mdBook/blob/master/examples/de-emphasize.rs
