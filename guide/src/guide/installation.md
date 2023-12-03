# Installation

There are multiple ways to install the mdBook CLI tool.
Choose any one of the methods below that best suit your needs.
If you are installing mdBook for automatic deployment, check out the [continuous integration] chapter for more examples on how to install.

[continuous integration]: ../continuous-integration.md

## Pre-compiled binaries

Executable binaries are available for download on the [GitHub Releases page][releases].
Download the binary for your platform (Windows, macOS, or Linux) and extract the archive.
The archive contains an `mdbook` executable which you can run to build your books.

To make it easier to run, put the path to the binary into your `PATH`.

[releases]: https://github.com/rust-lang/mdBook/releases

## Build from source using Rust

To build the `mdbook` executable from source, you will first need to install Rust and Cargo.
Follow the instructions on the [Rust installation page].
mdBook currently requires at least Rust version 1.70.

Once you have installed Rust, the following command can be used to build and install mdBook:

```sh
cargo install mdbook
```

This will automatically download mdBook from [crates.io], build it, and install it in Cargo's global binary directory (`~/.cargo/bin/` by default).

To uninstall, run the command `cargo uninstall mdbook`.

[Rust installation page]: https://www.rust-lang.org/tools/install
[crates.io]: https://crates.io/

### Installing the latest master version

The version published to crates.io will ever so slightly be behind the version hosted on GitHub.
If you need the latest version you can build the git version of mdBook yourself.
Cargo makes this ***super easy***!

```sh
cargo install --git https://github.com/rust-lang/mdBook.git mdbook
```

Again, make sure to add the Cargo bin directory to your `PATH`.

If you are interested in making modifications to mdBook itself, check out the [Contributing Guide] for more information.

[Contributing Guide]: https://github.com/rust-lang/mdBook/blob/master/CONTRIBUTING.md
