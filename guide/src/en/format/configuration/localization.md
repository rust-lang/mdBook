# Localization

It's possible to write your book in more than one language and bundle all of its
translations into a single output folder, with the ability for readers to switch
between each one in the rendered output. The available languages for your book
are defined in the `[language]` table:

```toml
[language.en]
name = "English"

[language.ja]
name = "日本語"
title = "本のサンプル"
description = "この本は実例です。"
authors = ["Ruin0x11"]
```

Each language must have a human-readable `name` defined. Also, if the
`[language]` table is defined, you must define `book.language` to be a key of
this table, which will indicate the language whose files will be used for
fallbacks if a page is missing in a translation.

The `title` and `description` fields, if defined, will override the ones set in
the `[book]` section. This way you can translate the book's title and
description. `authors` provides a list of this translation's authors.

After defining a new language like `[language.ja]`, add a new subdirectory
`src/ja` and create your `SUMMARY.md` and other files there.

> **Note:** Whether or not the `[language]` table is defined changes the format
> of the `src` directory that mdBook expects to see. If there is no `[language]`
> table, mdBook will treat the `src` directory as a single translation of the
> book, with `SUMMARY.md` at the root:
>
> ```
> ├── book.toml
> └── src
>     ├── chapter
>     │   ├── 1.md
>     │   ├── 2.md
>     │   └── README.md
>     ├── README.md
>     └── SUMMARY.md
> ```
> 
> If the `[language]` table is defined, mdBook will instead expect to find
> subdirectories under `src` named after the keys in the table:
>
> ```
> ├── book.toml
> └── src
>     ├── en
>     │   ├── chapter
>     │   │   ├── 1.md
>     │   │   ├── 2.md
>     │   │   └── README.md
>     │   ├── README.md
>     │   └── SUMMARY.md
>     └── ja
>         ├── chapter
>         │   ├── 1.md
>         │   ├── 2.md
>         │   └── README.md
>         ├── README.md
>         └── SUMMARY.md
> ```

If the `[language]` table is used, you can pass the `-l <language id>` argument
to commands like `mdbook build` to build the book for only a single language. In
this example, `<language id>` can be `en` or `ja`.

Some extra notes on translations:

- In a translation's `SUMMARY.md` or inside Markdown files, you can link to
  pages, images or other files that don't exist in the current translation, but
  do exist in the default translation. This is so you can have a fallback in
  case new pages get added in the default language that haven't been translated
  yet.
- Each translation can have its own `SUMMARY.md` with differing content from
  other translations. Even if the translation's summary goes out of sync with
  the default language, the links will continue to work so long as the pages
  exist in either translation.
- Each translation can have its own pages listed in `SUMMARY.md` that don't
  exist in the default translation at all, in case extra information specific to
  that language is needed.
