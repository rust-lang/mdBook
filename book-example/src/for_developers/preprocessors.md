# Preprocessors

A *preprocessor* is simply a bit of code which gets run immediately after the
book is loaded and before it gets rendered, allowing you to update and mutate
the book. Possible use cases are:

- Creating custom helpers like `{{#include /path/to/file.md}}`
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