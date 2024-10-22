# Installation

There are multiple ways to install the mdBook CLI tool.
Choose any one of the methods below that best suit your needs.
If you are installing mdBook for automatic deployment, check out the [continuous integration] chapter for more examples on how to install.

[continuous integration]: ../continuous-integration.md

## Pre-compiled binaries

Executable binaries are available for download on the [GitHub Releases page][releases].
Download the binary for your platform (Windows, macOS, or Linux) and extract the archive.
The archive contains an `mdbook` executable which you can run to build your books.

To make it easier to run, put the path to the binary into your `PATH`. If you have an installation of Rust, you can move the binary to [Cargo's global installation directory](#cargos-global-installation-directory).

[releases]: https://github.com/rust-lang/mdBook/releases

## Build from source using Rust

To build the `mdbook` executable from source, you will first need to install Rust and Cargo.
Follow the instructions on the [Rust installation page].
mdBook currently requires at least Rust version 1.74.

Once you have installed Rust, the following command can be used to build and install mdBook:

```sh
cargo install mdbook
```

This will automatically download mdBook from [crates.io], build it, and install it in [Cargo's global binary directory](#cargos-global-installation-directory).

You can run `cargo install mdbook` again whenever you want to update to a new version.
That command will check if there is a newer version, and re-install mdBook if a newer version is found.

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

## Cargo's global installation directory

In order to use mdBook, you don't need an installation of Rust. However, if you do, you can find it convenient to store the mdBook binary in the Cargo's global installation directory.

* **On Linux/macOS**, it is located at `~/.cargo/bin`.
* **On Windows**, it is located at `%USERPROFILE%\.cargo\bin`.

If you have an installation of Rust, this directory is most likely already in your `PATH`.

## Modifying and contributing

If you are interested in making modifications to mdBook itself, check out the [Contributing Guide] for more information.

[Contributing Guide]: https://github.com/rust-lang/mdBook/blob/master/CONTRIBUTING.md
