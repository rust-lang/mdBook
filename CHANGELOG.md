# Changelog

## mdBook 0.3.1
[69a08ef...9cd47eb](https://github.com/rust-lang-nursery/mdBook/compare/69a08ef...9cd47eb)

### Added
- 🔥 Added ability to include files using anchor points instead of line numbers.
  [#851](https://github.com/rust-lang-nursery/mdBook/pull/851)
- Added `language` configuration value to set the language of the book, which
  will affect things like the `<html lang="en">` tag.
  [#941](https://github.com/rust-lang-nursery/mdBook/pull/941)

### Changed
- Updated to handlebars 2.0.
  [#977](https://github.com/rust-lang-nursery/mdBook/pull/977)

### Fixed
- Fixed memory leak warning.
  [#967](https://github.com/rust-lang-nursery/mdBook/pull/967)
- Fix more print.html links.
  [#963](https://github.com/rust-lang-nursery/mdBook/pull/963)
- Fixed crash on some unicode input.
  [#978](https://github.com/rust-lang-nursery/mdBook/pull/978)

## mdBook 0.3.0
[6cbc41d...69a08ef](https://github.com/rust-lang-nursery/mdBook/compare/6cbc41d...69a08ef)

### Added
- Added ability to resize the sidebar.
  [#849](https://github.com/rust-lang-nursery/mdBook/pull/849)
- Added `load_with_config_and_summary` function to `MDBook` to be able to
  build a book with a custom `Summary`.
  [#883](https://github.com/rust-lang-nursery/mdBook/pull/883)
- Set `noindex` on `print.html` page to prevent robots from indexing it.
  [#844](https://github.com/rust-lang-nursery/mdBook/pull/844)
- Added support for ~~strikethrough~~ and GitHub-style tasklists.
  [#952](https://github.com/rust-lang-nursery/mdBook/pull/952)

### Changed
- Command-line help output is now colored.
  [#861](https://github.com/rust-lang-nursery/mdBook/pull/861)
- The build directory is now deleted before rendering starts, instead of after
  if finishes.
  [#878](https://github.com/rust-lang-nursery/mdBook/pull/878)
- Removed dependency on `same-file` crate.
  [#903](https://github.com/rust-lang-nursery/mdBook/pull/903)
- 💥 Renamed `with_preprecessor` to `with_preprocessor`.
  [#906](https://github.com/rust-lang-nursery/mdBook/pull/906)
- Updated ACE editor to 1.4.4, should remove a JavaScript console warning.
  [#935](https://github.com/rust-lang-nursery/mdBook/pull/935)
- Dependencies have been updated.
  [#934](https://github.com/rust-lang-nursery/mdBook/pull/934)
  [#945](https://github.com/rust-lang-nursery/mdBook/pull/945)
- Highlight.js has been updated. This fixes some TOML highlighting, and adds
  Julia support.
  [#942](https://github.com/rust-lang-nursery/mdBook/pull/942)
- 🔥 Updated to pulldown-cmark 0.5. This may have significant changes to the
  formatting of existing books, as the newer version has more accurate
  interpretation of the CommonMark spec and a large number of bug fixes and
  changes.
  [#898](https://github.com/rust-lang-nursery/mdBook/pull/898)
- The `diff` language should now highlight correctly.
  [#943](https://github.com/rust-lang-nursery/mdBook/pull/943)
- Make the blank region of a header not clickable.
  [#948](https://github.com/rust-lang-nursery/mdBook/pull/948)
- Rustdoc tests now use the preprocessed content instead of the raw,
  unpreprocessed content.
  [#891](https://github.com/rust-lang-nursery/mdBook/pull/891)

### Fixed
- Fixed file change detection so that `mdbook serve` only reloads once when
  multiple files are changed at once.
  [#870](https://github.com/rust-lang-nursery/mdBook/pull/870)
- Fixed on-hover color highlighting for links in sidebar.
  [#834](https://github.com/rust-lang-nursery/mdBook/pull/834)
- Fixed loss of focus when clicking the "Copy" button in code blocks.
  [#867](https://github.com/rust-lang-nursery/mdBook/pull/867)
- Fixed incorrectly stripping the path for `additional-js` files.
  [#796](https://github.com/rust-lang-nursery/mdBook/pull/796)
- Fixed color of `code spans` that are links.
  [#905](https://github.com/rust-lang-nursery/mdBook/pull/905)
- Fixed "next" navigation on index.html.
  [#916](https://github.com/rust-lang-nursery/mdBook/pull/916)
- Fixed keyboard chapter navigation for `file` urls.
  [#915](https://github.com/rust-lang-nursery/mdBook/pull/915)
- Fixed bad wrapping for inline code on some browsers.
  [#818](https://github.com/rust-lang-nursery/mdBook/pull/818)
- Properly load an existing `SUMMARY.md` in `mdbook init`.
  [#841](https://github.com/rust-lang-nursery/mdBook/pull/841)
- Fixed some broken links in `print.html`.
  [#871](https://github.com/rust-lang-nursery/mdBook/pull/871)
- The Rust Playground link now supports the 2018 edition.
  [#946](https://github.com/rust-lang-nursery/mdBook/pull/946)

## mdBook 0.2.3 (2018-01-18)
[2c20c99...6cbc41d](https://github.com/rust-lang-nursery/mdBook/compare/2c20c99...6cbc41d)

### Added
- Added an optional button to the top of the page which will link to a git
  repository. Use the `git-repository-url` and `git-repository-icon` options
  in the `[output.html]` section to enable it and set its appearance.
  [#802](https://github.com/rust-lang-nursery/mdBook/pull/802)
- Added a `default-theme` option to the `[output.html]` section.
  [#804](https://github.com/rust-lang-nursery/mdBook/pull/804)

### Changed
- 💥 Header ID anchors no longer add an arbitrary `a` character for headers
  that start with a non-ascii-alphabetic character.
  [#788](https://github.com/rust-lang-nursery/mdBook/pull/788)

### Fixed
- Fix websocket hostname usage
  [#865](https://github.com/rust-lang-nursery/mdBook/pull/865)
- Fixing links in print.html
  [#866](https://github.com/rust-lang-nursery/mdBook/pull/866)

## mdBook 0.2.2 (2018-10-19)
[7e2e095...2c20c99](https://github.com/rust-lang-nursery/mdBook/compare/7e2e095...2c20c99)

### Added
- 🎉 Process-based custom preprocessors. See [the
  docs](https://rust-lang-nursery.github.io/mdBook/for_developers/preprocessors.html)
  for more.
  [#792](https://github.com/rust-lang-nursery/mdBook/pull/792)

- 🎉 Configurable preprocessors.

  Added `build.use-default-preprocessors` boolean TOML key to allow disabling
  the built-in `links` and `index` preprocessors.

  Added `[preprocessor]` TOML tables to configure each preprocessor.

  Specifying `[preprocessor.links]` or `[preprocessor.index]` will enable the
  respective built-in preprocessor if `build.use-default-preprocessors` is
  `false`.

  Added `fn supports_renderer(&self, renderer: &str) -> bool` to the
  `Preprocessor` trait to specify if the preprocessor supports the given
  renderer. The default implementation always returns `true`.

  `Preprocessor::run` now takes a book by value instead of a mutable
  reference. It should return a `Book` value with the intended modifications.

  Added `PreprocessorContext::renderer` to indicate the renderer being used.

  [#658](https://github.com/rust-lang-nursery/mdBook/pull/658)
  [#787](https://github.com/rust-lang-nursery/mdBook/pull/787)

### Fixed
- Fix paths to additional CSS and JavaScript files
  [#777](https://github.com/rust-lang-nursery/mdBook/pull/777)
- Ensure section numbers are correctly incremented after a horizontal
  separator
  [#790](https://github.com/rust-lang-nursery/mdBook/pull/790)

## mdBook 0.2.1 (2018-08-22)
[91ffca1...7e2e095](https://github.com/rust-lang-nursery/mdBook/compare/91ffca1...7e2e095)

### Changed
- Update to handlebars-rs 1.0
  [#761](https://github.com/rust-lang-nursery/mdBook/pull/761)

### Fixed
- Fix table colors, broken by Stylus -> CSS transition
  [#765](https://github.com/rust-lang-nursery/mdBook/pull/765)

## mdBook 0.2.0 (2018-08-02)

### Changed
- 💥 This release changes how links are handled in mdBook. Previously, relative
  links were interpreted relative to the book's root. In `0.2.0`+ links are
  relative to the page they are in, and use the `.md` extension. This has [several
  advantages](https://github.com/rust-lang-nursery/mdBook/pull/603#issue-166701447),
  such as making links work in other markdown viewers like GitHub. You will
  likely have to change links in your book to accommodate this change. For
  example, a book with this layout:

  ```
  chapter_1/
      section_1.md
      section_2.md
  SUMMARY.md
  ```

  Previously a link in `section_1.md` to `section_2.md` would look like this:
  ```markdown
  [section_2](chapter_1/section_2.html)
  ```

  Now it must be changed to this:
  ```markdown
  [section_2](section_2.md)
  ```

- 💥 `mdbook test --library-path` now accepts a comma-delimited list of
  arguments rather than taking all following arguments. This makes it easier
  to handle the trailing book directory argument without always needing to put
  ` -- ` before it. Multiple instances of the option continue to be accepted:
  `mdbook test -L foo -L bar`.

- 💥 `mdbook serve` has some of its options renamed for clarity. See `mdbook
  help serve` for details.

- Embedded rust playpens now use the "stable" playground API.
  [#754](https://github.com/rust-lang-nursery/mdBook/pull/754)

### Fixed
- Escaped includes (`\{{#include file.rs}}`) will now render correctly.
  [f30ce01](https://github.com/rust-lang-nursery/mdBook/commit/f30ce0184d71e342141145472bf816419d30a2c5)
- `index.html` will now render correctly when the book's first section is
  inside a subdirectory.
  [#756](https://github.com/rust-lang-nursery/mdBook/pull/756)
