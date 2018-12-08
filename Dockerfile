FROM rust:1.30.1-stretch

ARG VERSION

RUN cargo install --vers "$VERSION" mdbook --no-default-features
