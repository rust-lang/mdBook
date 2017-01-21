# The init command

There is some minimal boilerplate that is the same for every new book. It's for this purpose that mdBook includes an `init` command.

Use this in an empty folder that you intend to use for your book.

Create a folder and invoke `init`:

```bash
mkdir thebook
cd ./thebook
mdbook init
```

When using the `init` command for the first time, a couple of files will be set up for you:

```
thebook
├── book.toml
└── src
    ├── SUMMARY.md
    ├── first-chapter.md
    ├── glossary.md
    └── introduction.md
```

In brief, `book.toml` has the book's general details and configuration, `src/` contains the chapters and `SUMMARY.md` defines the order of chapters as a Table of Contents.

`mdbook build` will generate the HTML format in the `book/` folder. This can be read locally in a brower or uploaded to be served as a static site.

See [Files and Folders](folders/folders.html) for more details on the conventions.

#### Tip & Trick: Outline Creation

Chapters files in the `SUMMARY.md` will be created if they don't exist. This allows you to outline the structure of the book and let mdBook create the files an folders as described.

#### Specify a directory

When using the `init` command, you can also specify a directory, instead of using the current working directory,
by appending a path to the command:

```bash
mdbook init path/to/book
```

## --copy-assets

When you use the `--copy-assets` argument, the default template with its static
assets will be copied to the `assets/` folder. When a local template is found,
it will be used instead of the default. Modifying the template and static assets
this way allows customisation.

