FROM rust:1.30.1-stretch

RUN cargo install mdbook --no-default-features
