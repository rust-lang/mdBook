# Command Line Tool

mdBook can be used either as a command line tool or a [Rust crate](https://crates.io/crates/mdbook).
Let's focus on the command line tool capabilities first.

## Overview

Install with `cargo install mdbook`.

Create a folder and invoke `init`:

```bash
mkdir thebook
cd ./thebook
mdbook init
```

`init` will create files for a basic book:

```
thebook
├── book.toml
└── src
    ├── SUMMARY.md
    ├── first-chapter.md
    ├── glossary.md
    └── introduction.md
```

`mdbook build` will generate the HTML in the `book/` folder. 

If your book has images or other static assets, put them in an `assets/` folder
and they will be copied to `book/` when building. Folders which begin with an
underscore (e.g. `assets/_graphviz/`) will be excluded.

For further command options, invoke `mdbook help`:

```
USAGE:
    mdbook <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    build    Build the book from the markdown files
    help     Prints this message or the help of the given subcommand(s)
    init     Create boilerplate structure and files in the directory
    serve    Serve the book at http://localhost:3000. Rebuild and reload on change.
    test     Test that code samples compile
    watch    Watch the files for changes
```

## Install

### Pre-requisite

mdBook is written in **[Rust](https://www.rust-lang.org/)** and therefore needs to be compiled with **Cargo**, because we don't yet offer ready-to-go binaries. If you haven't already installed Rust, please go ahead and [install it](https://www.rust-lang.org/en-US/install.html) now.

### Install Crates.io version

Installing mdBook is relatively easy if you already have Rust and Cargo installed. You just have to type this snippet in your terminal:

```bash
cargo install mdbook
```

This will fetch the source code from [Crates.io](https://crates.io/) and compile it. You will have to add Cargo's `bin` directory to your `PATH`.

Run `mdbook help` in your terminal to verify if it works. Congratulations, you have installed mdBook!


### Install Git version

The **[git version](https://github.com/azerupi/mdBook)** contains all the latest bug-fixes and features, that will be released in the next version on **Crates.io**, if you can't wait until the next release. You can build the git version yourself. Open your terminal and navigate to the directory of you choice. We need to clone the git repository and then build it with Cargo.

```bash
git clone --depth=1 https://github.com/azerupi/mdBook.git
cd mdBook
cargo install --force
```

The executable `mdbook` will be copied to `~/.cargo/bin` which should be added to the path.
