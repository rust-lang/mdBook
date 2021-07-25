# Introduction

**mdBook** is a command line tool and Rust crate to create books with Markdown. The output resembles tools like Gitbook,
and is ideal for creating product or API documentation, tutorials, course materials or anything that requires a clean,
easily navigable and customizable presentation. mdBook is written in [Rust](https://www.rust-lang.org); its performance
and simplicity made it ideal for use as a tool to publish directly to hosted websites such
as [GitHub Pages](https://pages.github.com) via automation. This guide, in fact, serves as both the mdBook documentation
and a fine example of what mdBook produces.

mdBook includes built in support for both preprocessing your Markdown and alternative renderers for producing formats
other than HTML. These facilities also enable other functionality such as
validation. [Searching](https://crates.io/search?q=mdbook&sort=relevance) Rust's [crates.io](https://crates.io) is a
great way to discover more extensions.

## API Documentation

In addition to the above features, mdBook also has a Rust [API](https://docs.rs/mdbook/*/mdbook/). This allows you to
write your own preprocessor or renderer, as well as incorporate mdBook features into other applications.
The [For Developers](for_developers) section of this guide contains more information and some examples.

## Markdown

mdBook's [parser](https://github.com/raphlinus/pulldown-cmark) adheres to the [CommonMark](https://commonmark.org/)
specification. You can take a quick [tutorial](https://commonmark.org/help/tutorial/),
or [try out](https://spec.commonmark.org/dingus/) CommonMark in real time. For a more in-depth experience, check out the
[Markdown Guide](https://www.markdownguide.org).

## Contributing

mdBook is free and open source. You can find the source code on
[GitHub](https://github.com/rust-lang/mdBook) and issues and feature requests can be posted on
the [GitHub issue tracker](https://github.com/rust-lang/mdBook/issues). mdBook relies on the community to fix bugs and
add features: if you'd like to contribute, please read
the [CONTRIBUTING](https://github.com/rust-lang/mdBook/blob/master/CONTRIBUTING.md) guide and consider opening
a [pull request](https://github.com/rust-lang/mdBook/pulls).

## License

The mdBook source and documentation are released under
the [Mozilla Public License v2.0](https://www.mozilla.org/MPL/2.0/).
