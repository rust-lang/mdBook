# Command Line Tool

mdBook can be used either as a command line tool or a [Rust crate](https://crates.io/crates/mdbook).
Let's focus on the command line tool capabilities first.

## Install

At the moment, the only way to install mdBook is by downloading the source code from Github and building it yourself. Fortunately
this is made very easy with Cargo.

If you haven't already, you should begin by installing [Rust](https://www.rust-lang.org/install.html) and [Git](https://git-scm.com/downloads)

Open your terminal and navigate to the directory of you choice. We need to clone the git repository and then build it with Cargo.

```
git clone --depth=1 https://github.com/azerupi/mdBook.git
cd mdBook
cargo build --release
```

The executable `mdbook` will be in the `./target/release` folder, this should be added to the path.
