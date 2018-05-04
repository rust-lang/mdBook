# Command Line Tool

mdBook can be used either as a command line tool or a [Rust crate](https://crates.io/crates/mdbook).
Let's focus on the command line tool capabilities first.

## Install

### Pre-requisite

mdBook is written in **[Rust](https://www.rust-lang.org/)** and therefore needs to be compiled with **Cargo**, because we don't yet offer ready-to-go binaries. If you haven't already installed Rust, please go ahead and [install it](https://www.rust-lang.org/downloads.html) now.

### Install Crates.io version

Installing mdBook is relatively easy if you already have Rust and Cargo installed. You just have to type this snippet in your terminal:

```bash
cargo install mdbook
```

This will fetch the source code from [Crates.io](https://crates.io/) and compile it. You will have to add Cargo's `bin` directory to your `PATH`.

Run `mdbook help` in your terminal to verify if it works. Congratulations, you have installed mdBook!


### Install Git version

The **[git version](https://github.com/rust-lang-nursery/mdBook)** contains all the latest bug-fixes and features, that will be released in the next version on **Crates.io**, if you can't wait until the next release. You can build the git version yourself. Open your terminal and navigate to the directory of you choice. We need to clone the git repository and then build it with Cargo.

```bash
git clone --depth=1 https://github.com/rust-lang-nursery/mdBook.git
cd mdBook
cargo build --release
```

The executable `mdbook` will be in the `./target/release` folder, this should be added to the path.
