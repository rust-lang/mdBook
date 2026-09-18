//! The mdbook CLI.

fn main() {
    // Note that `main` lives in a library to allow consumers
    // to access the mdbook entrypoint without the need for
    // a nightly toolchain to use `-Z bindeps`
    // https://github.com/rust-lang/cargo/issues/9096
    mdbook::main()
}
