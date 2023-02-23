################
##### Builder
##### docker buildx create --use --name multi-builder --platform linux/arm64,linux/amd64
# https://github.com/docker/buildx/issues/318#issuecomment-1023226339
#FROM --platform=$BUILDPLATFORM rustlang/rust:nightly-buster-slim as builder
FROM rustlang/rust:nightly-buster-slim as builder

# Installing git because of the workaround for the config
# https://github.com/rust-lang/cargo/issues/10781#issuecomment-1441071052
RUN apt-get update && apt-get install -y git

WORKDIR /usr/src/github.com/rust-lang

# Create blank project
RUN USER=root cargo new mdBook

# We want dependencies cached, so copy those first.
COPY Cargo.toml Cargo.lock /usr/src/github.com/rust-lang/mdBook
# examples is referenced in Cargo.toml
COPY examples /usr/src/github.com/rust-lang/mdBook/examples

WORKDIR /usr/src/github.com/rust-lang/mdBook

## Install target platform (Cross-Compilation) --> Needed for Alpine
#RUN rustup install nightly  
RUN rustup target add x86_64-unknown-linux-musl

# This is a dummy build to get the dependencies cached.
# https://github.com/rust-lang/cargo/issues/8172#issuecomment-659056517
# Very slow builds: https://github.com/rust-lang/cargo/issues/9167#issuecomment-1219251978
# Logs verbose: https://github.com/rust-lang/cargo/issues/1106#issuecomment-141555744
RUN cargo build -vv --config "net.git-fetch-with-cli=true" --target x86_64-unknown-linux-musl --release

# Now copy in the rest of the sources
COPY src /usr/src/github.com/rust-lang/mdBook/src

# This is the actual application build: # ./target/x86_64-unknown-linux-musl/release/mdbook
RUN cargo build --locked --bin mdbook --release --target x86_64-unknown-linux-musl

################
##### Runtime
FROM --platform=$BUILDPLATFORM alpine:3.16.0 AS runtime 

# Copy application binary from builder image
COPY --from=builder /usr/src/github.com/rust-lang/mdBook/target/x86_64-unknown-linux-musl/release/mdbook /usr/local/bin/mdbook

ARG TARGETPLATFORM
ARG BUILDPLATFORM
RUN echo "I am running on $BUILDPLATFORM, building for $TARGETPLATFORM" > /etc/build.log

EXPOSE 3000
ENTRYPOINT ["mdbook", "serve", "--hostname", "0.0.0.0"]
