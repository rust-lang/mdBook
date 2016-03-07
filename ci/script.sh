# `script` phase: you usually build, test and generate docs in this phase

set -ex

cargo build --target $TARGET --verbose
cargo test --target $TARGET --verbose
