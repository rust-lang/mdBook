#!/usr/bin/env bash
# Install/update rustup.
# The first argument should be the toolchain to install.
#
# It is helpful to have this as a separate script due to some issues on
# Windows where immediately after `rustup self update`, rustup can fail with
# "Device or resource busy".

set -ex
if [ -z "$1" ]
then
    echo "First parameter must be toolchain to install."
    exit 1
fi
TOOLCHAIN="$1"

# Install/update rustup.
if command -v rustup
then
    echo `command -v rustup` `rustup -V` already installed
    rustup self update
else
    # macOS currently does not have rust pre-installed.
    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain $TOOLCHAIN --profile=minimal
    echo "##[add-path]$HOME/.cargo/bin"
fi
