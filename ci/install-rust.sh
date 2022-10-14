#!/usr/bin/env bash
# Install/update rust.
# The first argument should be the toolchain to install.

set -ex
if [ -z "$1" ]
then
    echo "First parameter must be toolchain to install."
    exit 1
fi
TOOLCHAIN="$1"

rustup set profile minimal
rustup component remove --toolchain=$TOOLCHAIN rust-docs || echo "already removed"
rustup update --no-self-update $TOOLCHAIN
if [ -n "$2" ]
then
    TARGET="$2"
    HOST=$(rustc -Vv | grep ^host: | sed -e "s/host: //g")
    if [ "$HOST" != "$TARGET" ]
    then
        rustup component add llvm-tools-preview --toolchain=$TOOLCHAIN
        rustup component add rust-std-$TARGET --toolchain=$TOOLCHAIN
    fi
fi

rustup default $TOOLCHAIN
rustup -V
rustc -Vv
cargo -V
