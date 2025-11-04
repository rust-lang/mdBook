# Changelog

## 0.5 Migration Guide

During the pre-release phase of the 0.5 release, the documentation may be found at <https://rust-lang.github.io/mdBook/pre-release/>.

The 0.5 release contains several breaking changes from the 0.4 release. Preprocessors and renderers will need to be migrated to continue to work with this release. After updating your configuration, it is recommended to carefully compare and review how your book renders to ensure everything is working correctly.

If you have overridden any of the theme files, you will likely need to update them to match the current version.

The following is a summary of the changes that may require your attention when updating to 0.5:

### Config changes

- Unknown fields in config are now an error.
  [#2787](https://github.com/rust-lang/mdBook/pull/2787)
  [#2801](https://github.com/rust-lang/mdBook/pull/2801)
- Removed `curly-quotes`, use `output.html.smart-punctuation` instead.
  [#2788](https://github.com/rust-lang/mdBook/pull/2788)
- Removed `output.html.copy-fonts`. The default fonts are now always copied unless you override the `theme/fonts/fonts.css` file.
  [#2790](https://github.com/rust-lang/mdBook/pull/2790)
- If the `command` path for a renderer or preprocessor is relative, it is now always relative to the book root.
  [#2792](https://github.com/rust-lang/mdBook/pull/2792)
  [#2796](https://github.com/rust-lang/mdBook/pull/2796)
- Added the `optional` field for preprocessors. The default is `false`, so this also means it is an error by default if the preprocessor is missing.
  [#2797](https://github.com/rust-lang/mdBook/pull/2797)
- `output.html.smart-punctuation` is now `true` by default.
  [#2810](https://github.com/rust-lang/mdBook/pull/2810)
- `output.html.hash-files` is now `true` by default.
  [#2820](https://github.com/rust-lang/mdBook/pull/2820)
- Removed support for google-analytics. Use a theme extension (like `head.hbs`) if you need to continue to support this.
  [#2776](https://github.com/rust-lang/mdBook/pull/2776)
- Removed the `book.multilingual` field. This was never used.
  [#2775](https://github.com/rust-lang/mdBook/pull/2775)
- Removed the very old legacy config support. Warnings have been displayed in previous versions on how to migrate.
  [#2783](https://github.com/rust-lang/mdBook/pull/2783)

### Theme changes

- Replaced the `{{#previous}}` and `{{#next}}` handlebars helpers with simple objects that contain the previous and next values.
  [#2794](https://github.com/rust-lang/mdBook/pull/2794)
- Removed the `{{theme_option}}` handlebars helper. It has not been used for a while.
  [#2795](https://github.com/rust-lang/mdBook/pull/2795)

### Rendering changes

- Updated to a newer version of `pulldown-cmark`. This brings a large number of fixes to markdown processing.
  [#2401](https://github.com/rust-lang/mdBook/pull/2401)
- The font-awesome font is no longer loaded as a font. Instead, the corresponding SVG is embedded in the output for the corresponding `<i>` tags. Additionally, a handlebars helper has been added for the `hbs` files.
  [#1330](https://github.com/rust-lang/mdBook/pull/1330)
- Changed all internal HTML IDs to have an `mdbook-` prefix. This helps avoid namespace conflicts with header IDs.
  [#2808](https://github.com/rust-lang/mdBook/pull/2808)
- There is a new internal HTML rendering pipeline. This is primarily intended to give mdBook more flexibility in generating its HTML output. This resulted in some small changes to the HTML structure. HTML parsing may now be more strict than before.
  [#2844](https://github.com/rust-lang/mdBook/pull/2844)
- Links on the print page now link to elements on the print page instead of linking out to the individual chapters.
  [#2844](https://github.com/rust-lang/mdBook/pull/2844)
- Added support for definition lists. These are enabled by default, with the option `output.html.definition-lists` to disable it.
  [#2847](https://github.com/rust-lang/mdBook/pull/2847)
- Added support for admonitions. These are enabled by default, with the option `output.html.admonitions` to disable it.
  [#2851](https://github.com/rust-lang/mdBook/pull/2851)
- Headers that start or end with HTML characters like `<`, `&`, or `>` now replace those characters in the link ID with `-` instead of being stripped. This brings the header ID generation closer to other tools and sites.
  [#2844](https://github.com/rust-lang/mdBook/pull/2844)

### CLI changes

- Removed the `--dest-dir` option to `mdbook test`. It was unused since `mdbook test` does not generate output.
  [#2805](https://github.com/rust-lang/mdBook/pull/2805)
- Changed CLI `--dest-dir` to be relative to the current directory, not the book root.
  [#2806](https://github.com/rust-lang/mdBook/pull/2806)

### Rust API

- The Rust API has been split into several crates ([#2766](https://github.com/rust-lang/mdBook/pull/2766)). In summary, the different crates are:
  - `mdbook` — The CLI binary.
  - [`mdbook-driver`](https://docs.rs/mdbook-driver/latest/mdbook_driver/) — The high-level library for running mdBook, primarily through the `MDBook` type. If you are driving mdBook programmatically, this is the crate you want.
  - [`mdbook-preprocessor`](https://docs.rs/mdbook-preprocessor/latest/mdbook_preprocessor/) — Support for implementing preprocessors. If you have a preprocessor, then this is the crate you should depend on.
  - [`mdbook-renderer`](https://docs.rs/mdbook-renderer/latest/mdbook_renderer/) — Support for implementing renderers. If you have a custom renderer, this is the crate you should depend on.
  - [`mdbook-markdown`](https://docs.rs/mdbook-markdown/latest/mdbook_markdown/) — The Markdown renderer. If you are processing markdown, this is the crate you should depend on. This is essentially a thin wrapper around `pulldown-cmark`, and re-exports that crate so that you can ensure the version stays in sync with mdBook.
  - [`mdbook-summary`](https://docs.rs/mdbook-summary/latest/mdbook_summary/) — The `SUMMARY.md` parser.
  - [`mdbook-html`](https://docs.rs/mdbook-html/latest/mdbook_html/) — The HTML renderer.
  - [`mdbook-core`](https://docs.rs/mdbook-core/latest/mdbook_core/) — An internal library that is used by the other crates for shared types. You should not depend on this crate directly since types from this crate are re-exported from the other crates as appropriate.
- Changes to `Config`:
  - [`Config::get`](https://docs.rs/mdbook-core/latest/mdbook_core/config/struct.Config.html#method.get) is now generic over the return value, using `serde` to deserialize the value. It also returns a `Result` to handle deserialization errors. [#2773](https://github.com/rust-lang/mdBook/pull/2773)
  - Removed `Config::get_deserialized`. Use `Config::get` instead.
  - Removed `Config::get_deserialized_opt`. Use `Config::get` instead.
  - Removed `Config::get_mut`. Use `Config::set` instead.
  - Removed deprecated `Config::get_deserialized_opt`. Use `Config::get` instead.
  - Removed `Config::get_renderer`. Use `Config::get` instead.
  - Removed `Config::get_preprocessor`. Use `Config::get` instead.
- Public types have been switch to use the `#[non_exhaustive]` attribute to help allow them to change in a backwards-compatible way.
  [#2779](https://github.com/rust-lang/mdBook/pull/2779)
  [#2823](https://github.com/rust-lang/mdBook/pull/2823)
- Changed `MDBook` `with_renderer`/`with_preprocessor` to overwrite the entry if an extension of the same name is already loaded. This allows the caller to replace an entry.
  [#2802](https://github.com/rust-lang/mdBook/pull/2802)
- Added `MarkdownOptions` struct to specify settings for markdown rendering for `mdbook_markdown::new_cmark_parser`.
  [#2809](https://github.cocm/rust-lang/mdBook/pull/2809)
- Renamed `Book::sections` to `Book::items`.
  [#2813](https://github.com/rust-lang/mdBook/pull/2813)
- `mdbook::book::load_book` is now private. Instead, use one of the `MDBook` load functions like `MDBook::load_with_config`.
- Removed `HtmlConfig::smart_punctuation` method, use the field of the same name.
- `CmdPreprocessor::parse_input` moved to `mdbook_preprocessor::parse_input`.
- `Preprocessor::supports_renderer` now returns a `Result<bool>` instead of `bool` to be able to handle errors.
- Most of the types from the `theme` module are now private. The `Theme` struct is still exposed for working with themes.
- Various functions in the `utils::fs` module have been removed, renamed, or reworked.
- Most of the functions in the `utils` module have been moved, removed, or made private.

## mdBook 0.5.0-beta.1
[v0.5.0-alpha.1...v0.5.0-beta.1](https://github.com/rust-lang/mdBook/compare/v0.5.0-alpha.1...v0.5.0-beta.1)

### Changed

- Reworked the look of the header navigation.
  [#2898](https://github.com/rust-lang/mdBook/pull/2898)
- Update cargo dependencies.
  [#2896](https://github.com/rust-lang/mdBook/pull/2896)
- Improved the heading nav debug.
  [#2892](https://github.com/rust-lang/mdBook/pull/2892)

### Fixed

- Fixed error message for config.get deserialization error.
  [#2902](https://github.com/rust-lang/mdBook/pull/2902)
- Filter `<mark>` tags from sidebar heading nav.
  [#2899](https://github.com/rust-lang/mdBook/pull/2899)
- Avoid divide-by-zero in heading nav computation
  [#2891](https://github.com/rust-lang/mdBook/pull/2891)
- Fixed heading nav with folded chapters.
  [#2893](https://github.com/rust-lang/mdBook/pull/2893)

## mdBook 0.5.0-alpha.1
[v0.4.52...v0.5.0-alpha.1](https://github.com/rust-lang/mdBook/compare/v0.4.52...v0.5.0-alpha.1)

### Added

- The location of the generated HTML book is now displayed on the console.
  [#2729](https://github.com/rust-lang/mdBook/pull/2729)
- ❗ Added the `optional` field for preprocessors. The default is `false`, so this also changes it so that it is an error if the preprocessor is missing.
  [#2797](https://github.com/rust-lang/mdBook/pull/2797)
- ❗ Added `MarkdownOptions` struct to specify settings for markdown rendering.
  [#2809](https://github.cocm/rust-lang/mdBook/pull/2809)
- Added sidebar heading navigation. This includes the `output.html.sidebar-header-nav` option to disable it.
  [#2822](https://github.com/rust-lang/mdBook/pull/2822)
- Added the mdbook version to the guide.
  [#2826](https://github.com/rust-lang/mdBook/pull/2826)
- Added `Book::chapters` and `Book::for_each_chapter_mut` to more conveniently iterate over chapters (instead of all items).
  [#2838](https://github.com/rust-lang/mdBook/pull/2838)
- ❗ Added support for definition lists. These are enabled by default, with the option `output.html.definition-lists` to disable it.
  [#2847](https://github.com/rust-lang/mdBook/pull/2847)
- ❗ Added support for admonitions. These are enabled by default, with the option `output.html.admonitions` to disable it.
  [#2851](https://github.com/rust-lang/mdBook/pull/2851)

### Changed

- ❗ The `mdbook` crate has been split into multiple crates.
  [#2766](https://github.com/rust-lang/mdBook/pull/2766)
- The minimum Rust version has been updated to 1.88.
  [#2844](https://github.com/rust-lang/mdBook/pull/2844)
- ❗ `pulldown-cmark` has been upgraded to 0.13.0, bringing a large number of fixes to markdown processing.
  [#2401](https://github.com/rust-lang/mdBook/pull/2401)
- ❗ Switched public types to `non_exhaustive` to help allow them to change in a backwards-compatible way.
  [#2779](https://github.com/rust-lang/mdBook/pull/2779)
  [#2823](https://github.com/rust-lang/mdBook/pull/2823)
- ❗ Unknown fields in config are now an error.
  [#2787](https://github.com/rust-lang/mdBook/pull/2787)
  [#2801](https://github.com/rust-lang/mdBook/pull/2801)
- ❗ Changed `id_from_content` to be private.
  [#2791](https://github.com/rust-lang/mdBook/pull/2791)
- ❗ Changed preprocessor `command` to use paths relative to the book root.
  [#2796](https://github.com/rust-lang/mdBook/pull/2796)
- ❗ Replaced the `{{#previous}}` and `{{#next}}` handelbars navigation helpers with objects.
  [#2794](https://github.com/rust-lang/mdBook/pull/2794)
- ❗ Use embedded SVG instead of fonts for icons, font-awesome 6.2.
  [#1330](https://github.com/rust-lang/mdBook/pull/1330)
- The `book.src` field is no longer serialized if it is the default of "src".
  [#2800](https://github.com/rust-lang/mdBook/pull/2800)
- ❗ Changed `MDBook` `with_renderer`/`with_preprocessor` to overwrite the entry if an extension of the same name is already loaded.
  [#2802](https://github.com/rust-lang/mdBook/pull/2802)
- ❗ Changed CLI `--dest-dir` to be relative to the current directory, not the book root.
  [#2806](https://github.com/rust-lang/mdBook/pull/2806)
- ❗ Changed all internal HTML IDs to have an `mdbook-` prefix. This helps avoid namespace conflicts with header IDs.
  [#2808](https://github.com/rust-lang/mdBook/pull/2808)
- ❗ `output.html.smart-punctuation` is now `true` by default.
  [#2810](https://github.com/rust-lang/mdBook/pull/2810)
- ❗ Renamed `Book::sections` to `Book::items`.
  [#2813](https://github.com/rust-lang/mdBook/pull/2813)
- ❗ `output.html.hash-files` is now `true` by default.
  [#2820](https://github.com/rust-lang/mdBook/pull/2820)
- Switched from `log` to `tracing`.
  [#2829](https://github.com/rust-lang/mdBook/pull/2829)
- ❗ Rewrote the HTML rendering pipeline.
  [#2844](https://github.com/rust-lang/mdBook/pull/2844)
- ❗ Links on the print page now link to elements on the print page instead of linking out to the individual chapters.
  [#2844](https://github.com/rust-lang/mdBook/pull/2844)
- ❗ Moved theme copy to the Theme type and reduced visibility.
  [#2857](https://github.com/rust-lang/mdBook/pull/2857)
- ❗ Cleaned up some fs-related utilities.
  [#2856](https://github.com/rust-lang/mdBook/pull/2856)
- ❗ Moved `get_404_output_file` to `HtmlConfig`.
  [#2855](https://github.com/rust-lang/mdBook/pull/2855)
- ❗ Moved `take_lines` functions to `mdbook-driver` and made private.
  [#2854](https://github.com/rust-lang/mdBook/pull/2854)
- Updated dependencies.
  [#2793](https://github.com/rust-lang/mdBook/pull/2793)
  [#2869](https://github.com/rust-lang/mdBook/pull/2869)

### Removed

- ❗ Removed `toml` as a public dependency.
  [#2773](https://github.com/rust-lang/mdBook/pull/2773)
- ❗ Removed the `book.multilingual` field. This was never used.
  [#2775](https://github.com/rust-lang/mdBook/pull/2775)
- ❗ Removed support for google-analytics.
  [#2776](https://github.com/rust-lang/mdBook/pull/2776)
- ❗ Removed the very old legacy config support.
  [#2783](https://github.com/rust-lang/mdBook/pull/2783)
- ❗ Removed `curly-quotes`, use `output.html.smart-punctuation` instead.
  [#2788](https://github.com/rust-lang/mdBook/pull/2788)
- Removed old warning about `book.json`.
  [#2789](https://github.com/rust-lang/mdBook/pull/2789)
- ❗ Removed `output.html.copy-fonts`. The default fonts are now always copied unless you override the `theme/fonts/fonts.css` file.
  [#2790](https://github.com/rust-lang/mdBook/pull/2790)
- ❗ Removed legacy relative renderer command paths. Relative renderer command paths now must always be relative to the book root.
  [#2792](https://github.com/rust-lang/mdBook/pull/2792)
- ❗ Removed the `{{theme_option}}` handlebars helper. It has not been used for a while.
  [#2795](https://github.com/rust-lang/mdBook/pull/2795)
- ❗ Removed the `--dest-dir` option to `mdbook test`.
  [#2805](https://github.com/rust-lang/mdBook/pull/2805)

### Fixed

- Fixed handling of multiple footnotes in a row.
  [#2807](https://github.com/rust-lang/mdBook/pull/2807)
- Fixed ID collisions when the numeric suffix gets used.
  [#2846](https://github.com/rust-lang/mdBook/pull/2846)
- Fixed missing css vars for no-js dark mode.
  [#2850](https://github.com/rust-lang/mdBook/pull/2850)

## mdBook 0.4.52
[v0.4.51...v0.4.52](https://github.com/rust-lang/mdBook/compare/v0.4.51...v0.4.52)

**Note:** If you have a custom `index.hbs` theme file, it is recommended that you update it to the latest version to pick up the fixes in this release.

### Added
- Added the ability to redirect `#` HTML fragments using the existing `output.html.redirect` table.
  [#2747](https://github.com/rust-lang/mdBook/pull/2747)
- Added the `rel="edit"` attribute to the edit page button.
  [#2702](https://github.com/rust-lang/mdBook/pull/2702)

### Changed
- The search index is now only loaded when the search input is opened instead of always being loaded.
  [#2553](https://github.com/rust-lang/mdBook/pull/2553)
  [#2735](https://github.com/rust-lang/mdBook/pull/2735)
- The `mdbook serve` command has switched its underlying server library from warp to axum.
  [#2748](https://github.com/rust-lang/mdBook/pull/2748)
- Updated dependencies.
  [#2752](https://github.com/rust-lang/mdBook/pull/2752)

### Fixed
- The sidebar is now set to `display:none` when it is hidden in order to prevent the browser's search from thinking the sidebar's text is visible.
  [#2725](https://github.com/rust-lang/mdBook/pull/2725)
- Fixed search index URL not updating correctly when `hash-files` is enabled.
  [#2742](https://github.com/rust-lang/mdBook/pull/2742)
  [#2746](https://github.com/rust-lang/mdBook/pull/2746)
- Fixed several sidebar animation bugs, particularly when manually resizing.
  [#2750](https://github.com/rust-lang/mdBook/pull/2750)

## mdBook 0.4.51
[v0.4.50...v0.4.51](https://github.com/rust-lang/mdBook/compare/v0.4.50...v0.4.51)

### Fixed
- Fixed regression that broke the `S` search hotkey.
  [#2713](https://github.com/rust-lang/mdBook/pull/2713)

## mdBook 0.4.50
[v0.4.49...v0.4.50](https://github.com/rust-lang/mdBook/compare/v0.4.49...v0.4.50)

### Added

- Added a keyboard shortcut help popup when pressing `?`.
  [#2608](https://github.com/rust-lang/mdBook/pull/2608)

### Changed

- Changed the look of the sidebar resize handle to match the new rustdoc format.
  [#2691](https://github.com/rust-lang/mdBook/pull/2691)
- `/` can now be used to open the search bar.
  [#2698](https://github.com/rust-lang/mdBook/pull/2698)
- Pressing enter from the search bar will navigate to the first entry.
  [#2698](https://github.com/rust-lang/mdBook/pull/2698)
- Updated `opener` to drop some dependencies.
  [#2709](https://github.com/rust-lang/mdBook/pull/2709)
- Updated dependencies, MSRV raised to 1.82.
  [#2711](https://github.com/rust-lang/mdBook/pull/2711)

### Fixed

- Fixed uncaught exception when pressing down when there are no search results.
  [#2698](https://github.com/rust-lang/mdBook/pull/2698)
- Fixed syntax highlighting of Rust code in the ACE editor.
  [#2710](https://github.com/rust-lang/mdBook/pull/2710)

## mdBook 0.4.49
[v0.4.48...v0.4.49](https://github.com/rust-lang/mdBook/compare/v0.4.48...v0.4.49)

### Added

- Added a warning on unused fields in the root of `book.toml`.
  [#2622](https://github.com/rust-lang/mdBook/pull/2622)

### Changed

- Updated dependencies.
  [#2650](https://github.com/rust-lang/mdBook/pull/2650)
  [#2688](https://github.com/rust-lang/mdBook/pull/2688)
- Updated minimum Rust version to 1.81.
  [#2688](https://github.com/rust-lang/mdBook/pull/2688)
- The unused `book.multilingual` field is no longer serialized, or shown in `mdbook init`.
  [#2689](https://github.com/rust-lang/mdBook/pull/2689)
- Speed up search index loading by using `JSON.parse` instead of parsing JavaScript.
  [#2633](https://github.com/rust-lang/mdBook/pull/2633)

### Fixed

- Search highlighting will not try to highlight in SVG `<text>` elements because it breaks the element.
  [#2668](https://github.com/rust-lang/mdBook/pull/2668)
- Fixed scrolling of the sidebar when a search highlight term is in the URL.
  [#2675](https://github.com/rust-lang/mdBook/pull/2675)
- Fixed issues when multiple footnote definitions use the same ID. Now, only one definition is used, and a warning is displayed.
  [#2681](https://github.com/rust-lang/mdBook/pull/2681)
- The sidebar is now restricted to 80% of the viewport width to make it possible to collapse it when the viewport is very narrow.
  [#2679](https://github.com/rust-lang/mdBook/pull/2679)

## mdBook 0.4.48
[v0.4.47...v0.4.48](https://github.com/rust-lang/mdBook/compare/v0.4.47...v0.4.48)

### Added

- Footnotes now have back-reference links. These links bring the reader back to the original location. As part of this change, footnotes are now only rendered at the bottom of the page. This also includes some styling updates and fixes for footnote rendering.
  [#2626](https://github.com/rust-lang/mdBook/pull/2626)
- Added an "Auto" theme selection option which will default to the system-preferred mode. This will also automatically switch when the system changes the preferred mode.
  [#2576](https://github.com/rust-lang/mdBook/pull/2576)

### Changed

- The `searchindex.json` file has been removed; only the `searchindex.js` file will be generated.
  [#2552](https://github.com/rust-lang/mdBook/pull/2552)
- Updated Javascript code to use eslint.
  [#2554](https://github.com/rust-lang/mdBook/pull/2554)
- An error is generated if there are duplicate files in `SUMMARY.md`.
  [#2613](https://github.com/rust-lang/mdBook/pull/2613)

## mdBook 0.4.47
[v0.4.46...v0.4.47](https://github.com/rust-lang/mdBook/compare/v0.4.46...v0.4.47)

### Fixed

- Fixed search not showing up in sub-directories.
  [#2586](https://github.com/rust-lang/mdBook/pull/2586)

## mdBook 0.4.46
[v0.4.45...v0.4.46](https://github.com/rust-lang/mdBook/compare/v0.4.45...v0.4.46)

### Changed

- The `output.html.hash-files` config option has been added to add hashes to static filenames to bust any caches when a book is updated. `{{resource}}` template tags have been added so that links can be properly generated to those files.
  [#1368](https://github.com/rust-lang/mdBook/pull/1368)

### Fixed

- Playground links for Rust 2024 now set the edition correctly.
  [#2557](https://github.com/rust-lang/mdBook/pull/2557)

## mdBook 0.4.45
[v0.4.44...v0.4.45](https://github.com/rust-lang/mdBook/compare/v0.4.44...v0.4.45)

### Changed

- Added context to error message when rustdoc is not found.
  [#2545](https://github.com/rust-lang/mdBook/pull/2545)
- Slightly changed the styling rules around margins of footnotes.
  [#2524](https://github.com/rust-lang/mdBook/pull/2524)

### Fixed

- Fixed an issue where it would panic if a source_path is not set.
  [#2550](https://github.com/rust-lang/mdBook/pull/2550)

## mdBook 0.4.44
[v0.4.43...v0.4.44](https://github.com/rust-lang/mdBook/compare/v0.4.43...v0.4.44)

### Added

- Added pre-built aarch64-apple-darwin binaries to the releases.
  [#2500](https://github.com/rust-lang/mdBook/pull/2500)
- `mdbook clean` now shows a summary of what it did.
  [#2458](https://github.com/rust-lang/mdBook/pull/2458)
- Added the `output.html.search.chapter` config setting to disable search indexing of individual chapters.
  [#2533](https://github.com/rust-lang/mdBook/pull/2533)

### Fixed

- Fixed auto-scrolling the side-bar when loading a page with a `#` fragment URL.
  [#2517](https://github.com/rust-lang/mdBook/pull/2517)
- Fixed display of sidebar when javascript is disabled.
  [#2529](https://github.com/rust-lang/mdBook/pull/2529)
- Fixed the sidebar visibility getting out of sync with the button.
  [#2532](https://github.com/rust-lang/mdBook/pull/2532)

### Changed

- ❗ Rust code block hidden lines now follow the same logic as rustdoc. This requires a space after the `#` symbol.
  [#2530](https://github.com/rust-lang/mdBook/pull/2530)
- ❗ Updated the Linux pre-built binaries which requires a newer version of glibc (2.34).
  [#2523](https://github.com/rust-lang/mdBook/pull/2523)
- Updated dependencies
  [#2538](https://github.com/rust-lang/mdBook/pull/2538)
  [#2539](https://github.com/rust-lang/mdBook/pull/2539)

## mdBook 0.4.43
[v0.4.42...v0.4.43](https://github.com/rust-lang/mdBook/compare/v0.4.42...v0.4.43)

### Fixed

- Fixed setting the title in `mdbook init` when no git user is configured.
  [#2486](https://github.com/rust-lang/mdBook/pull/2486)

### Changed

- The Rust 2024 edition no longer needs `-Zunstable-options`.
  [#2495](https://github.com/rust-lang/mdBook/pull/2495)

## mdBook 0.4.42
[v0.4.41...v0.4.42](https://github.com/rust-lang/mdBook/compare/v0.4.41...v0.4.42)

### Fixed

- Fixed chapter list folding.
  [#2473](https://github.com/rust-lang/mdBook/pull/2473)

## mdBook 0.4.41
[v0.4.40...v0.4.41](https://github.com/rust-lang/mdBook/compare/v0.4.40...v0.4.41)

**Note:** If you have a custom `index.hbs` theme file, you will need to update it to the latest version.

### Added

- Added preliminary support for Rust 2024 edition.
  [#2398](https://github.com/rust-lang/mdBook/pull/2398)
- Added a full example of the remove-emphasis preprocessor.
  [#2464](https://github.com/rust-lang/mdBook/pull/2464)

### Changed

- Adjusted styling of clipboard/play icons.
  [#2421](https://github.com/rust-lang/mdBook/pull/2421)
- Updated to handlebars v6.
  [#2416](https://github.com/rust-lang/mdBook/pull/2416)
- Attr and section rules now have specific code highlighting.
  [#2448](https://github.com/rust-lang/mdBook/pull/2448)
- The sidebar is now loaded from a common file, significantly reducing the book size when there are many chapters.
  [#2414](https://github.com/rust-lang/mdBook/pull/2414)
- Updated dependencies.
  [#2470](https://github.com/rust-lang/mdBook/pull/2470)

### Fixed

- Improved theme support when JavaScript is disabled.
  [#2454](https://github.com/rust-lang/mdBook/pull/2454)
- Fixed broken themes when localStorage has an invalid theme id.
  [#2463](https://github.com/rust-lang/mdBook/pull/2463)
- Adjusted the line-height of superscripts (and footnotes) to avoid adding extra space between lines.
  [#2465](https://github.com/rust-lang/mdBook/pull/2465)

## mdBook 0.4.40
[v0.4.39...v0.4.40](https://github.com/rust-lang/mdBook/compare/v0.4.39...v0.4.40)

### Fixed

- Reverted the update to pulldown-cmark which broke the semver API.
  [#2388](https://github.com/rust-lang/mdBook/pull/2388)

## mdBook 0.4.39
[v0.4.38...v0.4.39](https://github.com/rust-lang/mdBook/compare/v0.4.38...v0.4.39)

### Fixed

- Fixed the automatic deploy broken in the previous release.
  [#2383](https://github.com/rust-lang/mdBook/pull/2383)

## mdBook 0.4.38
[v0.4.37...v0.4.38](https://github.com/rust-lang/mdBook/compare/v0.4.37...v0.4.38)

### Added

- Added `nix` to the default set of languages supported for syntax highlighting.
  [#2262](https://github.com/rust-lang/mdBook/pull/2262)

### Changed

- The `output.html.curly-quotes` option has been renamed to `output.html.smart-punctuation` to better reflect what it does. The old option `curly-quotes` is kept for compatibility, but may be removed in the future.
  [#2327](https://github.com/rust-lang/mdBook/pull/2327)
- The file-watcher used in `mdbook serve` and `mdbook watch` now uses a poll-based watcher instead of the native operating system notifications. This should fix issues on various systems and environments, and more accurately detect when files change. The native watcher can still be used with the `--watcher native` CLI option.
  [#2325](https://github.com/rust-lang/mdBook/pull/2325)
- `mdbook test` output now includes color, and shows relative paths to the source.
  [#2259](https://github.com/rust-lang/mdBook/pull/2259)
- Updated dependencies, MSRV raised to 1.74
  [#2350](https://github.com/rust-lang/mdBook/pull/2350)
  [#2351](https://github.com/rust-lang/mdBook/pull/2351)
  [#2378](https://github.com/rust-lang/mdBook/pull/2378)
  [#2381](https://github.com/rust-lang/mdBook/pull/2381)

### Fixed

- Reduced memory allocation when copying files.
  [#2355](https://github.com/rust-lang/mdBook/pull/2355)
- Fixed the horizontal divider in `SUMMARY.md` from being indented into the previous nested section.
  [#2364](https://github.com/rust-lang/mdBook/pull/2364)
- Removed unnecessary `@import` in the CSS.
  [#2260](https://github.com/rust-lang/mdBook/pull/2260)

## mdBook 0.4.37
[v0.4.36...v0.4.37](https://github.com/rust-lang/mdBook/compare/v0.4.36...v0.4.37)

### Changed
- ❗️ Updated the markdown parser. This brings in many changes to more closely follow the CommonMark spec. This may cause some small rendering changes. It is recommended to compare the output of the old and new version to check for changes. See <https://github.com/raphlinus/pulldown-cmark/releases/tag/v0.10.0> for more information.
  [#2308](https://github.com/rust-lang/mdBook/pull/2308)
- The warning about the legacy `src/theme` directory has been removed.
  [#2263](https://github.com/rust-lang/mdBook/pull/2263)
- Updated dependencies. MSRV raised to 1.71.0.
  [#2283](https://github.com/rust-lang/mdBook/pull/2283)
  [#2293](https://github.com/rust-lang/mdBook/pull/2293)
  [#2297](https://github.com/rust-lang/mdBook/pull/2297)
  [#2310](https://github.com/rust-lang/mdBook/pull/2310)
  [#2309](https://github.com/rust-lang/mdBook/pull/2309)
- Some internal performance/memory improvements.
  [#2273](https://github.com/rust-lang/mdBook/pull/2273)
  [#2290](https://github.com/rust-lang/mdBook/pull/2290)
- Made the `pathdiff` dependency optional based on the `watch` feature.
  [#2291](https://github.com/rust-lang/mdBook/pull/2291)

### Fixed
- The `s` shortcut key handler should not trigger when focus is in an HTML form.
  [#2311](https://github.com/rust-lang/mdBook/pull/2311)

## mdBook 0.4.36
[v0.4.35...v0.4.36](https://github.com/rust-lang/mdBook/compare/v0.4.35...v0.4.36)

### Added
- Added Nim to the default highlighted languages.
  [#2232](https://github.com/rust-lang/mdBook/pull/2232)
- Added a small indicator for the sidebar resize handle.
  [#2209](https://github.com/rust-lang/mdBook/pull/2209)

### Changed
- Updated dependencies. MSRV raised to 1.70.0.
  [#2173](https://github.com/rust-lang/mdBook/pull/2173)
  [#2250](https://github.com/rust-lang/mdBook/pull/2250)
  [#2252](https://github.com/rust-lang/mdBook/pull/2252)

### Fixed
- Fixed blank column in print page when the sidebar was visible.
  [#2235](https://github.com/rust-lang/mdBook/pull/2235)
- Fixed indentation of code blocks when Javascript is disabled.
  [#2162](https://github.com/rust-lang/mdBook/pull/2162)
- Fixed a panic when `mdbook serve` or `mdbook watch` were given certain kinds of paths.
  [#2229](https://github.com/rust-lang/mdBook/pull/2229)

## mdBook 0.4.35
[v0.4.34...v0.4.35](https://github.com/rust-lang/mdBook/compare/v0.4.34...v0.4.35)

### Added
- Added the `book.text-direction` setting for explicit support for right-to-left languages.
  [#1641](https://github.com/rust-lang/mdBook/pull/1641)
- Added `rel=prefetch` to the "next" links to potentially improve browser performance.
  [#2168](https://github.com/rust-lang/mdBook/pull/2168)
- Added a `.warning` CSS class which is styled for displaying warning blocks.
  [#2187](https://github.com/rust-lang/mdBook/pull/2187)

### Changed
- Better support of the sidebar when JavaScript is disabled.
  [#2175](https://github.com/rust-lang/mdBook/pull/2175)

## mdBook 0.4.34
[v0.4.33...v0.4.34](https://github.com/rust-lang/mdBook/compare/v0.4.33...v0.4.34)

### Fixed
- Fixed file change watcher failing on macOS with a large number of files.
  [#2157](https://github.com/rust-lang/mdBook/pull/2157)

## mdBook 0.4.33
[v0.4.32...v0.4.33](https://github.com/rust-lang/mdBook/compare/v0.4.32...v0.4.33)

### Added
- The `color-scheme` CSS property is now set based on the light/dark theme, which applies some slight color differences in browser elements like scroll bars on some browsers.
  [#2134](https://github.com/rust-lang/mdBook/pull/2134)

### Fixed
- Fixed watching of extra-watch-dirs when not running in the book root directory.
  [#2146](https://github.com/rust-lang/mdBook/pull/2146)
- Reverted the dependency update to the `toml` crate (again!). This was an unintentional breaking change in 0.4.32.
  [#2021](https://github.com/rust-lang/mdBook/pull/2021)
- Changed macOS change notifications to use the kqueue implementation which should fix some issues with repeated rebuilds when a file changed.
  [#2152](https://github.com/rust-lang/mdBook/pull/2152)
- Don't set a background color in the print page for code blocks in a header.
  [#2150](https://github.com/rust-lang/mdBook/pull/2150)

## mdBook 0.4.32
[v0.4.31...v0.4.32](https://github.com/rust-lang/mdBook/compare/v0.4.31...v0.4.32)

### Fixed
- Fixed theme-color meta tag not syncing with the theme.
  [#2118](https://github.com/rust-lang/mdBook/pull/2118)

### Changed
- Updated all dependencies.
  [#2121](https://github.com/rust-lang/mdBook/pull/2121)
  [#2122](https://github.com/rust-lang/mdBook/pull/2122)
  [#2123](https://github.com/rust-lang/mdBook/pull/2123)
  [#2124](https://github.com/rust-lang/mdBook/pull/2124)
  [#2125](https://github.com/rust-lang/mdBook/pull/2125)
  [#2126](https://github.com/rust-lang/mdBook/pull/2126)

## mdBook 0.4.31
[v0.4.30...v0.4.31](https://github.com/rust-lang/mdBook/compare/v0.4.30...v0.4.31)

### Fixed
- Fixed menu border render flash during page navigation.
  [#2101](https://github.com/rust-lang/mdBook/pull/2101)
- Fixed flicker setting sidebar scroll position.
  [#2104](https://github.com/rust-lang/mdBook/pull/2104)
- Fixed compile error with proc-macro2 on latest Rust nightly.
  [#2109](https://github.com/rust-lang/mdBook/pull/2109)

## mdBook 0.4.30
[v0.4.29...v0.4.30](https://github.com/rust-lang/mdBook/compare/v0.4.29...v0.4.30)

### Added
- Added support for heading attributes.
  Attributes are specified in curly braces just after the heading text.
  An HTML ID can be specified with `#` and classes with `.`.
  For example: `## My heading {#custom-id .class1 .class2}`
  [#2013](https://github.com/rust-lang/mdBook/pull/2013)
- Added support for hidden code lines for languages other than Rust.
  The `output.html.code.hidelines` table allows you to define the prefix character that will be used to hide code lines based on the language.
  [#2093](https://github.com/rust-lang/mdBook/pull/2093)

### Fixed
- Fixed a few minor markdown rendering issues.
  [#2092](https://github.com/rust-lang/mdBook/pull/2092)

## mdBook 0.4.29
[v0.4.28...v0.4.29](https://github.com/rust-lang/mdBook/compare/v0.4.28...v0.4.29)

### Changed
- Built-in fonts are no longer copied when `fonts/fonts.css` is overridden in the theme directory.
  Additionally, the warning about `copy-fonts` has been removed if `fonts/fonts.css` is specified.
  [#2080](https://github.com/rust-lang/mdBook/pull/2080)
- `mdbook init --force` now skips all interactive prompts as intended.
  [#2057](https://github.com/rust-lang/mdBook/pull/2057)
- Updated dependencies
  [#2063](https://github.com/rust-lang/mdBook/pull/2063)
  [#2086](https://github.com/rust-lang/mdBook/pull/2086)
  [#2082](https://github.com/rust-lang/mdBook/pull/2082)
  [#2084](https://github.com/rust-lang/mdBook/pull/2084)
  [#2085](https://github.com/rust-lang/mdBook/pull/2085)

### Fixed
- Switched from the `gitignore` library to `ignore`. This should bring some improvements with gitignore handling.
  [#2076](https://github.com/rust-lang/mdBook/pull/2076)

## mdBook 0.4.28
[v0.4.27...v0.4.28](https://github.com/rust-lang/mdBook/compare/v0.4.27...v0.4.28)

### Changed
- The sidebar is now shown on wide screens when localstorage is disabled.
  [#2017](https://github.com/rust-lang/mdBook/pull/2017)
- Preprocessors are now run with `mdbook test`.
  [#1986](https://github.com/rust-lang/mdBook/pull/1986)

### Fixed
- Fixed regression in 0.4.26 that prevented the title bar from scrolling properly on smaller screens.
  [#2039](https://github.com/rust-lang/mdBook/pull/2039)

## mdBook 0.4.27
[v0.4.26...v0.4.27](https://github.com/rust-lang/mdBook/compare/v0.4.26...v0.4.27)

### Changed
- Reverted the dependency update to the `toml` crate. This was an unintentional breaking change in 0.4.26.
  [#2021](https://github.com/rust-lang/mdBook/pull/2021)

## mdBook 0.4.26
[v0.4.25...v0.4.26](https://github.com/rust-lang/mdBook/compare/v0.4.25...v0.4.26)

**The 0.4.26 release has been yanked due to an unintentional breaking change.**

### Changed
- Removed custom scrollbars for webkit browsers
  [#1961](https://github.com/rust-lang/mdBook/pull/1961)
- Updated some dependencies
  [#1998](https://github.com/rust-lang/mdBook/pull/1998)
  [#2009](https://github.com/rust-lang/mdBook/pull/2009)
  [#2011](https://github.com/rust-lang/mdBook/pull/2011)
- Fonts are now part of the theme.
  The `output.html.copy-fonts` option has been deprecated.
  To define custom fonts, be sure to define `theme/fonts.css`.
  [#1987](https://github.com/rust-lang/mdBook/pull/1987)

### Fixed
- Fixed overflow viewport issue with mobile Safari
  [#1994](https://github.com/rust-lang/mdBook/pull/1994)

## mdBook 0.4.25
[e14d381...1ba74a3](https://github.com/rust-lang/mdBook/compare/e14d381...1ba74a3)

### Fixed
- Fixed a regression where `mdbook test -L deps path-to-book` would not work.
  [#1959](https://github.com/rust-lang/mdBook/pull/1959)

## mdBook 0.4.24
[eb77083...8767ebf](https://github.com/rust-lang/mdBook/compare/eb77083...8767ebf)

### Fixed
- The precompiled linux-gnu mdbook binary available on [GitHub Releases](https://github.com/rust-lang/mdBook/releases) inadvertently switched to a newer version of glibc. This release goes back to an older version that should be more compatible on older versions of Linux.
  [#1955](https://github.com/rust-lang/mdBook/pull/1955)

## mdBook 0.4.23
[678b469...68a75da](https://github.com/rust-lang/mdBook/compare/678b469...68a75da)

### Changed
- Updated all dependencies
  [#1951](https://github.com/rust-lang/mdBook/pull/1951)
  [#1952](https://github.com/rust-lang/mdBook/pull/1952)
  [#1844](https://github.com/rust-lang/mdBook/pull/1844)
- Updated minimum Rust version to 1.60.
  [#1951](https://github.com/rust-lang/mdBook/pull/1951)

### Fixed
- Fixed a regression where playground code was missing hidden lines, preventing it from compiling correctly.
  [#1950](https://github.com/rust-lang/mdBook/pull/1950)

## mdBook 0.4.22
[40c06f5...4844f72](https://github.com/rust-lang/mdBook/compare/40c06f5...4844f72)

### Added
- Added a `--chapter` option to `mdbook test` to specify a specific chapter to test.
  [#1741](https://github.com/rust-lang/mdBook/pull/1741)
- Added CSS styling for `<kbd>` tags.
  [#1906](https://github.com/rust-lang/mdBook/pull/1906)
- Added pre-compiled binaries for `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl` (see [Releases](https://github.com/rust-lang/mdBook/releases)).
  [#1862](https://github.com/rust-lang/mdBook/pull/1862)
- Added `build.extra-watch-dirs` which is an array of additional directories to watch for changes when running `mdbook serve`.
  [#1884](https://github.com/rust-lang/mdBook/pull/1884)

### Changed
- Removed the `type="text/javascript"` attribute from `<script>` tags.
  [#1881](https://github.com/rust-lang/mdBook/pull/1881)
- Switched to building with Rust Edition 2021.
  This raises the minimum supported Rust version to 1.56.
  [#1887](https://github.com/rust-lang/mdBook/pull/1887)
- When hidden code is hidden, the hidden parts are no longer copied to the clipboard via the copy button.
  [#1911](https://github.com/rust-lang/mdBook/pull/1911)
- Various HTML changes and fixes to be more compliant with HTML5.
  [#1924](https://github.com/rust-lang/mdBook/pull/1924)
- The theme picker now shows which theme is currently selected.
  [#1935](https://github.com/rust-lang/mdBook/pull/1935)

### Fixed
- Avoid blank line at the end of an ACE code block
  [#1836](https://github.com/rust-lang/mdBook/pull/1836)


## mdBook 0.4.21
[92afe9b...8f01d02](https://github.com/rust-lang/mdBook/compare/92afe9b...8f01d02)

### Fixed
- Fixed an issue where mdBook would fail to compile with Rust nightly-2022-07-22.
  [#1861](https://github.com/rust-lang/mdBook/pull/1861)

## mdBook 0.4.20
[53055e0...da166e0](https://github.com/rust-lang/mdBook/compare/53055e0...da166e0)

### Fixed
- Fixed a regression in 0.4.19 where inline code would have excessive padding
  in some situations such as headings.
  [#1855](https://github.com/rust-lang/mdBook/pull/1855)

## mdBook 0.4.19
[ae275ad...53055e0](https://github.com/rust-lang/mdBook/compare/ae275ad...53055e0)

### Added
- The `serve` command now supports HEAD requests.
  [#1825](https://github.com/rust-lang/mdBook/pull/1825)

### Changed
- An error is now generated when a custom theme directory does not exist.
  [#1791](https://github.com/rust-lang/mdBook/pull/1791)
- Very wide tables now have independent horizontal scrolling so that scrolling
  to see the rest of the table will not scroll the entire page.
  [#1617](https://github.com/rust-lang/mdBook/pull/1617)
- The buttons on code blocks are now only shown when the mouse cursor hovers
  over them (or tapped on mobile). There is also some extra spacing to reduce
  the overlap with the code.
  [#1806](https://github.com/rust-lang/mdBook/pull/1806)
- The first chapter always generates an `index.html` file. Previously it would
  only generate the index file for prefix chapters.
  [#1829](https://github.com/rust-lang/mdBook/pull/1829)

### Fixed
- `mdbook serve --open` now properly handles the case if the first chapter is a draft.
  [#1714](https://github.com/rust-lang/mdBook/pull/1714)
  [#1830](https://github.com/rust-lang/mdBook/pull/1830)
- Very long words (over 80 characters) are no longer indexed to avoid a stack overflow.
  [#1833](https://github.com/rust-lang/mdBook/pull/1833)

## mdBook 0.4.18
[981b79b...ae275ad](https://github.com/rust-lang/mdBook/compare/981b79b...ae275ad)

### Fixed
- Fixed rendering of SUMMARY links that contain markdown escapes or other
  markdown elements.
  [#1785](https://github.com/rust-lang/mdBook/pull/1785)

## mdBook 0.4.17
[a5fddfa...981b79b](https://github.com/rust-lang/mdBook/compare/a5fddfa...981b79b)

### Fixed
- Fixed parsing of `output.html.print` configuration table.
  [#1775](https://github.com/rust-lang/mdBook/pull/1775)

## mdBook 0.4.16
[68a5c09...a5fddfa](https://github.com/rust-lang/mdBook/compare/68a5c09...a5fddfa)

### Added
- Added `output.html.print.page-break` config option to control whether or not
  there is a page break between chapters in the print output.
  [#1728](https://github.com/rust-lang/mdBook/pull/1728)
- Added `output.html.playground.runnable` config option to globally disable
  the run button in code blocks.
  [#1546](https://github.com/rust-lang/mdBook/pull/1546)

### Changed
- The `mdbook serve` live reload websocket now uses the protocol, host, and
  port of the current page, allowing access through a proxy.
  [#1771](https://github.com/rust-lang/mdBook/pull/1771)
- The 404 not-found page now includes the books title in the HTML title tag.
  [#1693](https://github.com/rust-lang/mdBook/pull/1693)
- Migrated to clap 3.0 which handles CLI option parsing.
  [#1731](https://github.com/rust-lang/mdBook/pull/1731)

### Fixed
- Minor fixes to the markdown parser.
  [#1729](https://github.com/rust-lang/mdBook/pull/1729)
- Fixed incorrect parsing in `SUMMARY.md` when it didn't start with a title.
  [#1744](https://github.com/rust-lang/mdBook/pull/1744)
- Fixed duplicate anchor IDs for links in search results.
  [#1749](https://github.com/rust-lang/mdBook/pull/1749)

## mdBook 0.4.15
[5eb7d46...68a5c09](https://github.com/rust-lang/mdBook/compare/5eb7d46...68a5c09)

### Changed
- Major update to expand the documentation located at <https://rust-lang.github.io/mdBook/>.
  [#1709](https://github.com/rust-lang/mdBook/pull/1709)
  [#1710](https://github.com/rust-lang/mdBook/pull/1710)
- Updated the markdown parser with various fixes for common-mark compliance.
  [#1712](https://github.com/rust-lang/mdBook/pull/1712)

## mdBook 0.4.14
[ffa8284...c9b6be8](https://github.com/rust-lang/mdBook/compare/ffa8284...c9b6be8)

### Added
- The 2021 Rust edition option has been stabilized.
  [#1642](https://github.com/rust-lang/mdBook/pull/1642)

### Changed
- Header anchors no longer include any HTML tags. Previously only a small
  subset were excluded.
  [#1683](https://github.com/rust-lang/mdBook/pull/1683)
- Deprecated the google-analytics option. Books using this option should place
  the appropriate code in the `theme/head.hbs` file instead.
  [#1675](https://github.com/rust-lang/mdBook/pull/1675)

### Fixed
- Updated the markdown parser which brings in a few small fixes and removes
  the custom smart quote handling.
  [#1668](https://github.com/rust-lang/mdBook/pull/1668)
- Fixed iOS Safari enlarging text when going into landscape mode.
  [#1685](https://github.com/rust-lang/mdBook/pull/1685)

## mdBook 0.4.13
[e6629cd...f55028b](https://github.com/rust-lang/mdBook/compare/e6629cd...f55028b)

### Added

- Added the ability to specify the preprocessor order.
  [#1607](https://github.com/rust-lang/mdBook/pull/1607)

### Fixed

- Include chapters with no headers in the search index
  [#1637](https://github.com/rust-lang/mdBook/pull/1637)
- Switched to the `opener` crate for opening a web browser, which should fix
  some issues with blocking.
  [#1656](https://github.com/rust-lang/mdBook/pull/1656)
- Fixed clicking the border of the theme switcher breaking the theme selection.
  [#1651](https://github.com/rust-lang/mdBook/pull/1651)

## mdBook 0.4.12
[14add9c...8b4e488](https://github.com/rust-lang/mdBook/compare/14add9c...8b4e488)

### Changed
- Reverted the change to update to highlight.js 11, as it broke hidden code lines.
  [#1597](https://github.com/rust-lang/mdBook/pull/1621)
 
## mdBook 0.4.11
[e440094...2cf00d0](https://github.com/rust-lang/mdBook/compare/e440094...2cf00d0)

### Added
- Added support for Rust 2021 edition.
  [#1596](https://github.com/rust-lang/mdBook/pull/1596)
- Added `mdbook completions` subcommand which provides shell completions.
  [#1425](https://github.com/rust-lang/mdBook/pull/1425)
- Added `--title` and `--ignore` flags to `mdbook init` to avoid the
  interactive input.
  [#1559](https://github.com/rust-lang/mdBook/pull/1559)

### Changed
- If running a Rust example does not have any output, it now displays the text
  "No output" instead of not showing anything.
  [#1599](https://github.com/rust-lang/mdBook/pull/1599)
- Code block language tags can now be separated by space or tab (along with
  commas) to match the behavior of other sites like GitHub and rustdoc.
  [#1469](https://github.com/rust-lang/mdBook/pull/1469)
- Updated `warp` (the web server) to the latest version.
  This also updates the minimum supported Rust version to 1.46.
  [#1612](https://github.com/rust-lang/mdBook/pull/1612)
- Updated to highlight.js 11. This has various highlighting improvements.
  [#1597](https://github.com/rust-lang/mdBook/pull/1597)

### Fixed
- Inline code blocks inside a header are no longer highlighted when
  `output.html.playground.editable` is `true`.
  [#1613](https://github.com/rust-lang/mdBook/pull/1613)

## mdBook 0.4.10
[2f7293a...dc2062a](https://github.com/rust-lang/mdBook/compare/2f7293a...dc2062a)

### Changed
- Reverted breaking change in 0.4.9 that removed the `__non_exhaustive` marker
  on the `Book` struct.
  [#1572](https://github.com/rust-lang/mdBook/pull/1572)
- Updated handlebars to 4.0.
  [#1550](https://github.com/rust-lang/mdBook/pull/1550)
- Removed the `chapter_begin` id on the print page's chapter separators.
  [#1541](https://github.com/rust-lang/mdBook/pull/1541)

## mdBook 0.4.9
[7e01cf9...d325c60](https://github.com/rust-lang/mdBook/compare/7e01cf9...d325c60)

### Changed
- Updated all dependencies and raised the minimum Rust version to 1.42.
  [#1528](https://github.com/rust-lang/mdBook/pull/1528)
- Added more detail to error message when a preprocessor fails.
  [#1526](https://github.com/rust-lang/mdBook/pull/1526)
- Set max-width of HTML video tags to 100% to match img tags.
  [#1542](https://github.com/rust-lang/mdBook/pull/1542)

### Fixed
- Type errors when parsing `book.toml` are no longer ignored.
  [#1539](https://github.com/rust-lang/mdBook/pull/1539)
- Better handling if `mdbook serve` fails to start the http server.
  [#1555](https://github.com/rust-lang/mdBook/pull/1555)
- Fixed the path for `edit-url-template` if the book used a source directory
  other than `src`.
  [#1554](https://github.com/rust-lang/mdBook/pull/1554)

## mdBook 0.4.8
[fcceee4...b592b10](https://github.com/rust-lang/mdBook/compare/fcceee4...b592b10)

### Added
- Added the option `output.html.edit-url-template` which can be a URL which is
  linked on each page to direct the user to a site (such as GitHub) where the
  user can directly suggest an edit for the page they are currently reading.
  [#1506](https://github.com/rust-lang/mdBook/pull/1506)

### Changed
- Printed output now includes a page break between chapters.
  [#1485](https://github.com/rust-lang/mdBook/pull/1485)

### Fixed
- HTML, such as HTML comments, is now ignored if it appears above the title line
  in `SUMMARY.md`.
  [#1437](https://github.com/rust-lang/mdBook/pull/1437)

## mdBook 0.4.7
[9a9eb01...c83bbd6](https://github.com/rust-lang/mdBook/compare/9a9eb01...c83bbd6)

### Changed
- Updated shlex parser to fix a minor parsing issue (used by the
  preprocessor/backend custom command config).
  [#1471](https://github.com/rust-lang/mdBook/pull/1471)
- Enhanced text contrast of `light` theme to improve accessibility.
  [#1470](https://github.com/rust-lang/mdBook/pull/1470)

### Fixed
- Fixed some issues with fragment scrolling and linking.
  [#1463](https://github.com/rust-lang/mdBook/pull/1463)

## mdBook 0.4.6
[eaa6914...1a0c296](https://github.com/rust-lang/mdBook/compare/eaa6914...1a0c296)

### Changed
- The chapter name is now included in the search breadcrumbs.
  [#1389](https://github.com/rust-lang/mdBook/pull/1389)
- Pressing Escape will remove the `?highlight` argument from the URL.
  [#1427](https://github.com/rust-lang/mdBook/pull/1427)
- `mdbook init --theme` will now place the theme in the root of the book
  directory instead of in the `src` directory.
  [#1432](https://github.com/rust-lang/mdBook/pull/1432)
- A custom renderer that sets the `command` to a relative path now interprets
  the relative path relative to the book root. Previously it was inconsistent
  based on the platform (either relative to the current directory, or relative
  to the renderer output directory). Paths relative to the output directory
  are still supported with a deprecation warning.
  [#1418](https://github.com/rust-lang/mdBook/pull/1418)
- The `theme` directory in the config is now interpreted as relative to the
  book root, instead of the current directory.
  [#1405](https://github.com/rust-lang/mdBook/pull/1405)
- Handle UTF-8 BOM for chapter sources.
  [#1285](https://github.com/rust-lang/mdBook/pull/1285)
- Removed extra whitespace added to `{{#playground}}` snippets.
  [#1375](https://github.com/rust-lang/mdBook/pull/1375)

### Fixed
- Clicking on a search result with multiple search words will now correctly
  highlight all of the words.
  [#1426](https://github.com/rust-lang/mdBook/pull/1426)
- Properly handle `<` and `>` characters in the table of contents.
  [#1376](https://github.com/rust-lang/mdBook/pull/1376)
- Fixed to properly serialize the `build` table in the config, which prevented
  setting it in the API.
  [#1378](https://github.com/rust-lang/mdBook/pull/1378)

## mdBook 0.4.5
[eaa6914...f66df09](https://github.com/rust-lang/mdBook/compare/eaa6914...f66df09)

### Fixed

- Fixed XSS in the search page.
  [CVE-2020-26297](https://groups.google.com/g/rustlang-security-announcements/c/3-sO6of29O0)
  [648c9ae](https://github.com/rust-lang/mdBook/commit/648c9ae772bec83f0a5954d17b4287d5bb1d6606)

## mdBook 0.4.4
[4df9ec9...01836ba](https://github.com/rust-lang/mdBook/compare/4df9ec9...01836ba)

### Added
- Added the `output.html.print.enable` configuration value to disable the
  "print" page.
  [#1169](https://github.com/rust-lang/mdBook/pull/1169)
- Added a list of supported languages for syntax-highlighting to the
  documentation.
  [#1345](https://github.com/rust-lang/mdBook/pull/1345)

### Fixed
- Now supports symbolic links for files in the `src` directory.
  [#1323](https://github.com/rust-lang/mdBook/pull/1323)

## mdBook 0.4.3
[9278b83...4df9ec9](https://github.com/rust-lang/mdBook/compare/9278b83...4df9ec9)

### Added
- Added `output.html.cname` option to emit a `CNAME` file which is used by
  GitHub Pages to know which domain is being used.
  [#1311](https://github.com/rust-lang/mdBook/pull/1311)

### Changed
- `mdbook test` no longer stops on the first test failure, but instead will
  run all the tests.
  [#1313](https://github.com/rust-lang/mdBook/pull/1313)
- Removed the `local` font source for Source Code Pro, as the locally
  installed font may not render properly on FireFox on macOS.
  [#1307](https://github.com/rust-lang/mdBook/pull/1307)

### Fixed
- Added newline to end of `.nojekyll` file.
  [#1310](https://github.com/rust-lang/mdBook/pull/1310)
- Fixed missing space before draft chapter titles.
  [#1309](https://github.com/rust-lang/mdBook/pull/1309)

## mdBook 0.4.2
[649f355...9278b83](https://github.com/rust-lang/mdBook/compare/649f355...9278b83)

### Changed
- The "show hidden lines" icon has changed from the "expand" icon to an "eye".
  [#1281](https://github.com/rust-lang/mdBook/pull/1281)
- Updated highlight.js. This adds several languages: c, c-like (effectively
  cpp), csharp (replaces cs), kotlin, less, lua, php-template, plaintext,
  python-repl, r, scss, typescript.
  [#1277](https://github.com/rust-lang/mdBook/pull/1277)

### Fixed
- Fixed SUMMARY links that contained newlines.
  [#1291](https://github.com/rust-lang/mdBook/pull/1291)
- Fixed SUMMARY links that contain `%20` spaces.
  [#1293](https://github.com/rust-lang/mdBook/pull/1293)
- Fixed favicon so that if only the png or svg is overridden, the other is not
  automatically included in the `<link>` tag.
  [#1272](https://github.com/rust-lang/mdBook/pull/1272)

## mdBook 0.4.1
[d4df7e7...649f355](https://github.com/rust-lang/mdBook/compare/d4df7e7...649f355)

### Changed
- Removed several outdated dev-dependencies.
  [#1267](https://github.com/rust-lang/mdBook/pull/1267)

### Fixed
- Fixed sidebar scrolling if the book includes part titles.
  [#1265](https://github.com/rust-lang/mdBook/pull/1265)
- Don't include the default favicon if only one of the PNG or SVG is overridden.
  [#1266](https://github.com/rust-lang/mdBook/pull/1266)

## mdBook 0.4.0
[99ecd4f...d4df7e7](https://github.com/rust-lang/mdBook/compare/99ecd4f...d4df7e7)

### Breaking Changes
- Several of the changes in the release have altered the public API of the
  mdbook library.
- Many dependencies have been updated or replaced.
  This also removes the `--websocket-hostname` and `--websocket-port` from
  the `serve` command.
  [#1211](https://github.com/rust-lang/mdBook/pull/1211)
- A new "404" page is now automatically rendered. This requires knowledge of
  the base URL of your site to work properly. If you decide to use this as
  your 404 page, you should set the `site-url` setting in the book
  configuration so mdbook can generate the links correctly. Alternatively you
  can disable the 404 page generation, or set up your own 404 handling in your
  web server.
  [#1221](https://github.com/rust-lang/mdBook/pull/1221)
- The `debug` and `output` features have been removed as they were unused.
  [#1211](https://github.com/rust-lang/mdBook/pull/1211)
- If you are using customized themes, you may want to consider setting the
  `preferred-dark-theme` config setting, as it now defaults to "navy".
  [#1199](https://github.com/rust-lang/mdBook/pull/1199)
- "Playpen" has been renamed to "playground". This is generally backwards
  compatible for users, but `{{#playpen}}` will now display warnings. This may
  impact books that have modified the "playpen" elements in the theme.
  [#1241](https://github.com/rust-lang/mdBook/pull/1241)
- If a renderer is not installed, it is now treated as an error. If you want
  the old behavior of ignoring missing renderers, set the `optional` setting
  for that renderer.
  [#1122](https://github.com/rust-lang/mdBook/pull/1122)
- If you have a custom favicon, you may need to look into adding an SVG
  version, otherwise the default SVG icon will be displayed.
  [#1230](https://github.com/rust-lang/mdBook/pull/1230)

### Added
- Added a new `[rust]` configuration section to `book.toml`, which allows
  setting the default edition with `edition = "2018"`.
  [#1163](https://github.com/rust-lang/mdBook/pull/1163)
- Renderers can now be marked as `optional`, so that they will be ignored if
  the renderer is not installed.
  [#1122](https://github.com/rust-lang/mdBook/pull/1122)
- Added `head.hbs` to allow adding content to the `<head>` section in HTML.
  [#1206](https://github.com/rust-lang/mdBook/pull/1206)
- Added "draft chapters". These are chapters listed without a link to indicate
  content yet to be written.
  [#1153](https://github.com/rust-lang/mdBook/pull/1153)
- Added "parts" to split a book into different sections. Headers can be added
  to `SUMMARY.md` to signify different sections.
  [#1171](https://github.com/rust-lang/mdBook/pull/1171)
- Added generation of a "404" page for handling missing pages and broken links.
  [#1221](https://github.com/rust-lang/mdBook/pull/1221)
- Added configuration section for specifying URL redirects.
  [#1237](https://github.com/rust-lang/mdBook/pull/1237)
- Added an SVG favicon that works with light and dark colors schemes.
  [#1230](https://github.com/rust-lang/mdBook/pull/1230)

### Changed
- Changed default Rust attribute of `allow(unused_variables)` to `allow(unused)`.
  [#1195](https://github.com/rust-lang/mdBook/pull/1195)
- Fonts are now served locally instead of from the Google Fonts CDN. The
  `copy-fonts` option was added to disable this if you want to supply your own
  fonts.
  [#1188](https://github.com/rust-lang/mdBook/pull/1188)
- Switched the built-in webserver for the `serve` command to a new
  implementation. This results in some internal differences in how websockets
  are handled, which removes the separate websocket options. This should also
  make it easier to serve multiple books at once.
  [#1211](https://github.com/rust-lang/mdBook/pull/1211)
- The default dark theme is now "navy".
  [#1199](https://github.com/rust-lang/mdBook/pull/1199)
- "Playpen" has been renamed to "playground", matching the actual name of the
  service which was renamed many years ago.
  [#1241](https://github.com/rust-lang/mdBook/pull/1241)

### Fixed
- Links with the `+` symbol should now work.
  [#1208](https://github.com/rust-lang/mdBook/pull/1208)
- The `MDBOOK_BOOK` environment variable now correctly allows overriding the
  entire book configuration.
  [#1207](https://github.com/rust-lang/mdBook/pull/1207)
- The sidebar can no longer be dragged outside of the window.
  [#1229](https://github.com/rust-lang/mdBook/pull/1229)
- Hide the Rust Playground "play" button for `no_run` code samples.
  [#1249](https://github.com/rust-lang/mdBook/pull/1249)
- Fixed the `--dest-dir` command-line option for the `serve` and `watch`
  commands.
  [#1228](https://github.com/rust-lang/mdBook/pull/1228)
- Hotkey handlers are now disabled in `text` input fields (for example, typing
  `S` in a custom text input field).
  [#1244](https://github.com/rust-lang/mdBook/pull/1244)

## mdBook 0.3.7
[88684d8...99ecd4f](https://github.com/rust-lang/mdBook/compare/88684d8...99ecd4f)

### Changed
- Code spans in headers are no longer highlighted as code.
  [#1162](https://github.com/rust-lang/mdBook/pull/1162)
- The sidebar will now scroll the activate page to the middle instead of the top.
  [#1161](https://github.com/rust-lang/mdBook/pull/1161)
- Reverted change to reject build output within the `src` directory, and
  instead add a check that prevents infinite copies.
  [#1181](https://github.com/rust-lang/mdBook/pull/1181)
  [#1026](https://github.com/rust-lang/mdBook/pull/1026)

### Fixed
- Fixed sidebar line-height jumping for collapsed chapters.
  [#1182](https://github.com/rust-lang/mdBook/pull/1182)
- Fixed theme selector focus.
  [#1170](https://github.com/rust-lang/mdBook/pull/1170)

## mdBook 0.3.6
[efdb832...88684d8](https://github.com/rust-lang/mdBook/compare/efdb832...88684d8)

### Added
- `MDBook::execute_build_process` is now publicly accessible in the API so
  that plugins can more easily initiate the build process.
  [#1099](https://github.com/rust-lang/mdBook/pull/1099)

### Changed
- Use a different color for Ayu theme's highlighting for Rust attributes (uses
  a bright color instead of the comment color).
  [#1133](https://github.com/rust-lang/mdBook/pull/1133)
- Adjusted spacing of sidebar entries.
  [#1137](https://github.com/rust-lang/mdBook/pull/1137)
- Slightly adjusted line-height of `<p>`, `<ul>`, and `<ol>`.
  [#1136](https://github.com/rust-lang/mdBook/pull/1136)
- Handlebars updated to 3.0.
  [#1130](https://github.com/rust-lang/mdBook/pull/1130)

### Fixed
- Fix an issue with sidebar scroll position on reload.
  [#1108](https://github.com/rust-lang/mdBook/pull/1108)
- `mdbook serve` will retain the current scroll position when the page is reloaded.
  [#1097](https://github.com/rust-lang/mdBook/pull/1097)
- Fixed the page name if the book didn't have a title to not be prefixed with ` - `.
  [#1145](https://github.com/rust-lang/mdBook/pull/1145)
- HTML attributes `rel=next` and `rel=previous` are now supported in "wide"
  mode (previously they were only set in narrow mode).
  [#1150](https://github.com/rust-lang/mdBook/pull/1150)
- Prevent recursive copies when the destination directory is contained in the
  source directory.
  [#1135](https://github.com/rust-lang/mdBook/pull/1135)
- Adjusted the menu bar animation to not immediately obscure the top content.
  [#989](https://github.com/rust-lang/mdBook/pull/989)
- Fix for comments in SUMMARY.md that appear between items.
  [#1167](https://github.com/rust-lang/mdBook/pull/1167)

## mdBook 0.3.5
[6e0d0fa...efdb832](https://github.com/rust-lang/mdBook/compare/6e0d0fa...efdb832)

### Changed
- The `default-theme` config setting is now case-insensitive.
  [#1079](https://github.com/rust-lang/mdBook/pull/1079)

### Fixed
- Fixed `#` hidden Rust code lines not rendering properly.
  [#1088](https://github.com/rust-lang/mdBook/pull/1088)
- Updated pulldown-cmark to 0.6.1, fixing several issues.
  [#1021](https://github.com/rust-lang/mdBook/pull/1021)

## mdBook 0.3.4
[e5f77aa...6e0d0fa](https://github.com/rust-lang/mdBook/compare/e5f77aa...6e0d0fa)

### Changed
- Switch to relative `rem` font sizes from `px`.
  [#894](https://github.com/rust-lang/mdBook/pull/894)
- Migrated repository to https://github.com/rust-lang/mdBook/
  [#1083](https://github.com/rust-lang/mdBook/pull/1083)

## mdBook 0.3.3
[2b649fe...e5f77aa](https://github.com/rust-lang/mdBook/compare/2b649fe...e5f77aa)

### Changed
- Improvements to the automatic dark theme selection.
  [#1069](https://github.com/rust-lang/mdBook/pull/1069)
- Fragment links now prevent scrolling the header behind the menu bar.
  [#1077](https://github.com/rust-lang/mdBook/pull/1077)

### Fixed
- Fixed error when building a book that has a spacer immediately after the
  first chapter.
  [#1075](https://github.com/rust-lang/mdBook/pull/1075)

## mdBook 0.3.2
[9cd47eb...2b649fe](https://github.com/rust-lang/mdBook/compare/9cd47eb...2b649fe)

### Added
- Added a markdown renderer, which is off by default. This may be useful for
  debugging preprocessors.
  [#1018](https://github.com/rust-lang/mdBook/pull/1018)
- Code samples may now include line numbers with the
  `output.html.playpen.line-numbers` configuration value.
  [#1035](https://github.com/rust-lang/mdBook/pull/1035)
- The `watch` and `serve` commands will now ignore files listed in
  `.gitignore`.
  [#1044](https://github.com/rust-lang/mdBook/pull/1044)
- Added automatic dark-theme detection based on the CSS `prefers-color-scheme`
  feature. This may be enabled by setting `output.html.preferred-dark-theme`
  to your preferred dark theme.
  [#1037](https://github.com/rust-lang/mdBook/pull/1037)
- Added `rustdoc_include` preprocessor. This makes it easier to include
  portions of an external Rust source file. The rest of the file is hidden,
  but the user may expand it to see the entire file, and will continue to work
  with `mdbook test`.
  [#1003](https://github.com/rust-lang/mdBook/pull/1003)
- Added Ctrl-Enter shortcut to the playpen editor to automatically run the
  sample.
  [#1066](https://github.com/rust-lang/mdBook/pull/1066)
- Added `output.html.playpen.copyable` configuration option to disable
  the copy button.
  [#1050](https://github.com/rust-lang/mdBook/pull/1050)
- Added ability to dynamically expand and fold sections within the sidebar.
  See the `output.html.fold` configuration to enable this feature.
  [#1027](https://github.com/rust-lang/mdBook/pull/1027)

### Changed
- Use standard `scrollbar-color` CSS along with webkit extension
  [#816](https://github.com/rust-lang/mdBook/pull/816)
- The renderer build directory is no longer deleted before the renderer is
  run. This allows a backend to cache results between runs.
  [#985](https://github.com/rust-lang/mdBook/pull/985)
- Next/prev links now highlight on hover to indicate it is clickable.
  [#994](https://github.com/rust-lang/mdBook/pull/994)
- Increase padding of table headers.
  [#824](https://github.com/rust-lang/mdBook/pull/824)
- Errors in `[output.html]` config are no longer ignored.
  [#1033](https://github.com/rust-lang/mdBook/pull/1033)
- Updated highlight.js for syntax highlighting updates (primarily to add
  async/await to Rust highlighting).
  [#1041](https://github.com/rust-lang/mdBook/pull/1041)
- Raised minimum supported rust version to 1.35.
  [#1003](https://github.com/rust-lang/mdBook/pull/1003)
- Hidden code lines are no longer dynamically removed via JavaScript, but
  instead managed with CSS.
  [#846](https://github.com/rust-lang/mdBook/pull/846)
  [#1065](https://github.com/rust-lang/mdBook/pull/1065)
- Changed the default font set for the ACE editor, giving preference to
  "Source Code Pro".
  [#1062](https://github.com/rust-lang/mdBook/pull/1062)
- Windows 32-bit releases are no longer published.
  [#1071](https://github.com/rust-lang/mdBook/pull/1071)

### Fixed
- Fixed sidebar auto-scrolling.
  [#1052](https://github.com/rust-lang/mdBook/pull/1052)
- Fixed error message when running `clean` multiple times.
  [#1055](https://github.com/rust-lang/mdBook/pull/1055)
- Actually fix the "next" link on index.html. The previous fix didn't work.
  [#1005](https://github.com/rust-lang/mdBook/pull/1005)
- Stop using `inline-block` for `inline code`, fixing selection highlighting
  and some rendering issues.
  [#1058](https://github.com/rust-lang/mdBook/pull/1058)
- Fix header auto-hide on browsers with momentum scrolling that allows
  negative `scrollTop`.
  [#1070](https://github.com/rust-lang/mdBook/pull/1070)

## mdBook 0.3.1
[69a08ef...9cd47eb](https://github.com/rust-lang/mdBook/compare/69a08ef...9cd47eb)

### Added
- 🔥 Added ability to include files using anchor points instead of line numbers.
  [#851](https://github.com/rust-lang/mdBook/pull/851)
- Added `language` configuration value to set the language of the book, which
  will affect things like the `<html lang="en">` tag.
  [#941](https://github.com/rust-lang/mdBook/pull/941)

### Changed
- Updated to handlebars 2.0.
  [#977](https://github.com/rust-lang/mdBook/pull/977)

### Fixed
- Fixed memory leak warning.
  [#967](https://github.com/rust-lang/mdBook/pull/967)
- Fix more print.html links.
  [#963](https://github.com/rust-lang/mdBook/pull/963)
- Fixed crash on some unicode input.
  [#978](https://github.com/rust-lang/mdBook/pull/978)

## mdBook 0.3.0
[6cbc41d...69a08ef](https://github.com/rust-lang/mdBook/compare/6cbc41d...69a08ef)

### Added
- Added ability to resize the sidebar.
  [#849](https://github.com/rust-lang/mdBook/pull/849)
- Added `load_with_config_and_summary` function to `MDBook` to be able to
  build a book with a custom `Summary`.
  [#883](https://github.com/rust-lang/mdBook/pull/883)
- Set `noindex` on `print.html` page to prevent robots from indexing it.
  [#844](https://github.com/rust-lang/mdBook/pull/844)
- Added support for ~~strikethrough~~ and GitHub-style tasklists.
  [#952](https://github.com/rust-lang/mdBook/pull/952)

### Changed
- Command-line help output is now colored.
  [#861](https://github.com/rust-lang/mdBook/pull/861)
- The build directory is now deleted before rendering starts, instead of after
  if finishes.
  [#878](https://github.com/rust-lang/mdBook/pull/878)
- Removed dependency on `same-file` crate.
  [#903](https://github.com/rust-lang/mdBook/pull/903)
- 💥 Renamed `with_preprecessor` to `with_preprocessor`.
  [#906](https://github.com/rust-lang/mdBook/pull/906)
- Updated ACE editor to 1.4.4, should remove a JavaScript console warning.
  [#935](https://github.com/rust-lang/mdBook/pull/935)
- Dependencies have been updated.
  [#934](https://github.com/rust-lang/mdBook/pull/934)
  [#945](https://github.com/rust-lang/mdBook/pull/945)
- Highlight.js has been updated. This fixes some TOML highlighting, and adds
  Julia support.
  [#942](https://github.com/rust-lang/mdBook/pull/942)
- 🔥 Updated to pulldown-cmark 0.5. This may have significant changes to the
  formatting of existing books, as the newer version has more accurate
  interpretation of the CommonMark spec and a large number of bug fixes and
  changes.
  [#898](https://github.com/rust-lang/mdBook/pull/898)
- The `diff` language should now highlight correctly.
  [#943](https://github.com/rust-lang/mdBook/pull/943)
- Make the blank region of a header not clickable.
  [#948](https://github.com/rust-lang/mdBook/pull/948)
- Rustdoc tests now use the preprocessed content instead of the raw,
  unpreprocessed content.
  [#891](https://github.com/rust-lang/mdBook/pull/891)

### Fixed
- Fixed file change detection so that `mdbook serve` only reloads once when
  multiple files are changed at once.
  [#870](https://github.com/rust-lang/mdBook/pull/870)
- Fixed on-hover color highlighting for links in sidebar.
  [#834](https://github.com/rust-lang/mdBook/pull/834)
- Fixed loss of focus when clicking the "Copy" button in code blocks.
  [#867](https://github.com/rust-lang/mdBook/pull/867)
- Fixed incorrectly stripping the path for `additional-js` files.
  [#796](https://github.com/rust-lang/mdBook/pull/796)
- Fixed color of `code spans` that are links.
  [#905](https://github.com/rust-lang/mdBook/pull/905)
- Fixed "next" navigation on index.html.
  [#916](https://github.com/rust-lang/mdBook/pull/916)
- Fixed keyboard chapter navigation for `file` urls.
  [#915](https://github.com/rust-lang/mdBook/pull/915)
- Fixed bad wrapping for inline code on some browsers.
  [#818](https://github.com/rust-lang/mdBook/pull/818)
- Properly load an existing `SUMMARY.md` in `mdbook init`.
  [#841](https://github.com/rust-lang/mdBook/pull/841)
- Fixed some broken links in `print.html`.
  [#871](https://github.com/rust-lang/mdBook/pull/871)
- The Rust Playground link now supports the 2018 edition.
  [#946](https://github.com/rust-lang/mdBook/pull/946)

## mdBook 0.2.3 (2018-01-18)
[2c20c99...6cbc41d](https://github.com/rust-lang/mdBook/compare/2c20c99...6cbc41d)

### Added
- Added an optional button to the top of the page which will link to a git
  repository. Use the `git-repository-url` and `git-repository-icon` options
  in the `[output.html]` section to enable it and set its appearance.
  [#802](https://github.com/rust-lang/mdBook/pull/802)
- Added a `default-theme` option to the `[output.html]` section.
  [#804](https://github.com/rust-lang/mdBook/pull/804)

### Changed
- 💥 Header ID anchors no longer add an arbitrary `a` character for headers
  that start with a non-ascii-alphabetic character.
  [#788](https://github.com/rust-lang/mdBook/pull/788)

### Fixed
- Fix websocket hostname usage
  [#865](https://github.com/rust-lang/mdBook/pull/865)
- Fixing links in print.html
  [#866](https://github.com/rust-lang/mdBook/pull/866)

## mdBook 0.2.2 (2018-10-19)
[7e2e095...2c20c99](https://github.com/rust-lang/mdBook/compare/7e2e095...2c20c99)

### Added
- 🎉 Process-based custom preprocessors. See [the
  docs](https://rust-lang.github.io/mdBook/for_developers/preprocessors.html)
  for more.
  [#792](https://github.com/rust-lang/mdBook/pull/792)

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

  [#658](https://github.com/rust-lang/mdBook/pull/658)
  [#787](https://github.com/rust-lang/mdBook/pull/787)

### Fixed
- Fix paths to additional CSS and JavaScript files
  [#777](https://github.com/rust-lang/mdBook/pull/777)
- Ensure section numbers are correctly incremented after a horizontal
  separator
  [#790](https://github.com/rust-lang/mdBook/pull/790)

## mdBook 0.2.1 (2018-08-22)
[91ffca1...7e2e095](https://github.com/rust-lang/mdBook/compare/91ffca1...7e2e095)

### Changed
- Update to handlebars-rs 1.0
  [#761](https://github.com/rust-lang/mdBook/pull/761)

### Fixed
- Fix table colors, broken by Stylus -> CSS transition
  [#765](https://github.com/rust-lang/mdBook/pull/765)

## mdBook 0.2.0 (2018-08-02)

### Changed
- 💥 This release changes how links are handled in mdBook. Previously, relative
  links were interpreted relative to the book's root. In `0.2.0`+ links are
  relative to the page they are in, and use the `.md` extension. This has [several
  advantages](https://github.com/rust-lang/mdBook/pull/603#issue-166701447),
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
  [#754](https://github.com/rust-lang/mdBook/pull/754)

### Fixed
- Escaped includes (`\{{#include file.rs}}`) will now render correctly.
  [f30ce01](https://github.com/rust-lang/mdBook/commit/f30ce0184d71e342141145472bf816419d30a2c5)
- `index.html` will now render correctly when the book's first section is
  inside a subdirectory.
  [#756](https://github.com/rust-lang/mdBook/pull/756)
