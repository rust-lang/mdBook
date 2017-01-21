# Files and Folders

Let's take this example book, which has a custom page template and static assets
(e.g. css, images, etc.):

```
thebook
├── assets
│  ├── _html-template
│  │  └── _layouts
│  │     └── page.hbs
│  ├── _sass
│  │  └── main.sass
│  ├── css
│  │  ├── custom.css
│  │  └── main.css
│  └── img
│     └── diagram.png
├── book
│  ├── css
│  │  ├── custom.css
│  │  └── main.css
│  ├── img
│  │  └── diagram.png
│  ├── first-chapter.html
│  ├── glossary.html
│  ├── index.html
│  └── print.html
├── src
│  ├── SUMMARY.md
│  ├── first-chapter.md
│  ├── glossary.md
│  └── introduction.md
└── book.toml
```

`book.toml` describes the general details of the book such as `title` and
`author`.

The `src/` folder is were you write your book in Markdown.

`src/SUMMARY.md` contains a list of the chapters as a Table of Contents, the
format is discussed in more detail in another [chapter](format/summary.html).

`book/` is the default output folder where your book is rendered. This can be
opened locally in a browser or uploaded to be served as a static site.

### Chapters

Chapters are Markdown files. Specific data can be given in a TOML header between
`+++` lines:

```
+++
title = "The Library of Babel"
author = "Jorge Luis Borges"
translator = "James E. Irby"
+++

# Babel

The universe (which others call the Library) is composed of an indefinite and
perhaps infinite number of hexagonal galleries, with vast air shafts between,
surrounded by very low railings. From any of the hexagons one can see,
interminably, the upper and lower floors.
```

### assets

Local assets are optional. If not present, mdBook will use its default templates
and copy its default CSS to the output folder.

If an `assets/` folder is present, its contents will be copied to the output
folder instead of using the defaults. Folders which begin with an underscore
will be excluded. I.e. `assets/css/` will be copied, but not `assets/_sass/`.

Chapter pages are rendered either with the default template, or with
`assets/_html-template/_layouts/page.hbs` if present, which is a Handlebars
template.

Note that you can get a copy of the default template files by calling `mdbook
init --copy-assets`.

#### custom.css

If a `custom.css` file is found, the page template's `{{customcss}}` helper will
place a `<link>` to it in the html `<head>`.

This allows you to add small tweaks without copying and maintaining the
template's whole CSS.

The following paths are checked:

- `assets/css/custom.css`
- `assets/stylesheets/custom.css`
- `assets/custom.css`

## Multiple Languages

### book.toml

Declare the translations as separate blocks as in the example below.

The main language is recognized as the first given in the TOML. Otherwise it has
to be marked with `is_main_book = true`.

The language code will always be the translation key, the language name can be
set optionally.

```
[[translations.en]]
title = "Alice's Adventures in Wonderland"
author = "Lewis Carroll"

[[translations.fr]]
title = "Alice au pays des merveilles"
author = "Lewis Carroll"
language_name = "Français"

[[translations.hu]]
title = "Alice Csodaországban"
author = "Lewis Carroll"
```

### Folders

Put each translation in a sub-folder in `src/` with the translation key as the
sub-folder. Each translation should include a `SUMMARY.md`.

The chapter file names can be translated if you wish the output URLs to reflect
the translation's language.

```
wonderland/
├── book.toml
├── assets
│  └── images
│     └── Rabbit.png
└── src
   ├── en
   │  ├── SUMMARY.md
   │  └── rabbit-hole.md
   ├── fr
   │  ├── SUMMARY.md
   │  └── terrier.md
   └── hu
      ├── SUMMARY.md
      └── nyuszi.md
```

### Translation cross-linking

There are some mechanisms for automatic chapter-to-chapter linking between translations, or manual linking can be defined.

They will appear as:

- links to the top-level index pages of the translations are displayed above the
  TOC in the sidebar
- chapter translations are displayed in the title bar when the application can
  find a translation, otherwise it displays a grayed-out text of the language
  code

Translations are identified step by step this way:

- taking the manual links if given in the TOML header
- finding a match by a specific `translation_id` string given in the TOML header
- finding a match by the chapter file's path and name
- finding a match by section number, if the TOC is structurally the same.

Probably you want automatic cross-linking, which would happen if either you use
the same chapter file paths across translations, or at least you don't change
the number of sections and sub-sections in the TOC.

Otherwise, you can use a translation ID string when file names and the TOC
structure are different:

In one of the translations:

```
+++
translation_id = "caucus race"
+++

# A Caucus-Race and a Long Tale

![Tail](images/Tail.png)
```

And in the other:

```
+++
translation_id = "caucus race"
+++

# Körbecsukó meg az egér hosszú tarka farka

![Tarka-farka](images/Tail.png)
```

Or else, you can define the links manually. Note, that this will break when the
target translation's chapter file name changes.

```
+++
[[translation_links]]
code = "fr"
link = "fr/terrier.html"

[[translation_links]]
code = "hu"
link = "hu/nyuszi.html"
+++

# Down The Rabbit-Hole

![Rabbit](images/Rabbit.png)
```

