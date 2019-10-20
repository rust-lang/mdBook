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
rustup update $TOOLCHAIN
rustup default $TOOLCHAIN
rustup -V
rustc -Vv
cargo -V
